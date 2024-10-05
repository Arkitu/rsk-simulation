use crate::{constants::simu::*, game_state::Robot};
use tracing::info;
use nalgebra::Isometry2;
use rapier2d_f64::prelude::*;

const BALL_COLLISION_GROUP: Group = Group::GROUP_1;
const ROBOT_COLLISION_GROUPS: [Group; 4] = [
    Group::GROUP_2,
    Group::GROUP_3,
    Group::GROUP_4,
    Group::GROUP_5
];
const KICKER_COLLISION_GROUP: Group = Group::GROUP_6;

pub struct Simulation {
    pub bodies: RigidBodySet,
    pub colliders: ColliderSet,
    pub goals: [ColliderHandle; 2],
    pub ball: RigidBodyHandle,
    pub ball_col: ColliderHandle,
    pub robots: [RigidBodyHandle; 4],
    pub kickers: [RigidBodyHandle; 4],
    pub kicker_joints: [ImpulseJointHandle; 4],
    pub kicker_timer: [usize; 4],
    gravity: Vector<f64>,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    islands: IslandManager,
    broad_phase: DefaultBroadPhase,
    pub narrow_phase: NarrowPhase,
    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline,
    physics_hooks: (),
    events: (),
    // Actual frame
    pub t: usize,
}
impl Simulation {
    pub fn new() -> Self {
        let mut bodies = RigidBodySet::new();
        let mut colliders = ColliderSet::new();
        let mut impulse_joints = ImpulseJointSet::new();

        // Create the goals
        let goals = [
            colliders.insert(ColliderBuilder::segment(BLUE_GOAL.0, BLUE_GOAL.1).sensor(true)),
            colliders.insert(ColliderBuilder::segment(GREEN_GOAL.0, GREEN_GOAL.1).sensor(true)),
        ];

        // Create the ball
        let ball = bodies.insert(
            RigidBodyBuilder::dynamic()
                .position(DEFAULT_BALL_POS.into())
                .linear_damping(BALL_DAMPING)
                .can_sleep(false)
                // .dominance_group(-1)
        );
        let ball_col = colliders.insert_with_parent(
            ColliderBuilder::ball(BALL_RADIUS)
                .restitution(BALL_RESTITUTION)
                .restitution_combine_rule(CoefficientCombineRule::Min)
                .mass(BALL_MASS)
                .collision_groups(InteractionGroups::new(BALL_COLLISION_GROUP, Group::all())),
            ball,
            &mut bodies,
        );

        // Create the robots
        let robots = std::array::from_fn(|i| bodies.insert(
            RigidBodyBuilder::dynamic()
                .position(DEFAULT_ROBOTS_POS[i].into())
                .rotation(DEFAULT_ROBOTS_ANGLE[i])
                .linear_damping(ROBOT_DAMPING)
                .angular_damping(ROBOT_ANGULAR_DAMPING)
                .can_sleep(false)
        ));
        for (robot, collision_group) in robots.iter().zip(ROBOT_COLLISION_GROUPS.iter()) {
            const r: f64 = ROBOT_RADIUS - 0.001;
            colliders.insert_with_parent(
                // Collider is a regular hexagon with radius ROBOT_RADIUS
                ColliderBuilder::round_convex_hull(&[
                    point![0., r],
                    point![r * 0.866, r * 0.5],
                    point![r * 0.866, -r * 0.5],
                    point![0., -r],
                    point![-r * 0.866, -r * 0.5],
                    point![-r * 0.866, r * 0.5],
                ], 0.001).unwrap()
                    .mass(10.)
                    .restitution(ROBOT_RESTITUTION)
                    .restitution_combine_rule(CoefficientCombineRule::Min)
                    .collision_groups(InteractionGroups::new(*collision_group, Group::all())),
                *robot,
                &mut bodies,
            );
        }

        // Create kickers
        let kickers = std::array::from_fn(|i| bodies.insert(
            RigidBodyBuilder::dynamic()
                .position(Isometry::new(Vector::new(DEFAULT_ROBOTS_POS[i].x + (if i < 2 {1.} else {-1.} * ((ROBOT_RADIUS*0.866) + (KICKER_THICKNESS/2.))), DEFAULT_ROBOTS_POS[i].y), DEFAULT_ROBOTS_ANGLE[i]))
                .ccd_enabled(true)
                .can_sleep(false)
        ));
        let mut kicker_joints = kickers.iter().zip(robots.iter()).zip(ROBOT_COLLISION_GROUPS.iter()).map(|((kicker, robot), collision_group)| {
            let mut col = ColliderBuilder::cuboid(KICKER_THICKNESS, ROBOT_RADIUS)
                .position(Point::new(-0.77, 0.).into())
                .restitution(ROBOT_RESTITUTION)
                .restitution_combine_rule(CoefficientCombineRule::Min)
                .collision_groups(InteractionGroups::new(KICKER_COLLISION_GROUP, collision_group.complement()))
                .build();
            colliders.insert_with_parent(
                col,
                *kicker,
                &mut bodies
            );
            impulse_joints.insert(
                *robot,
                *kicker,
                PrismaticJointBuilder::new(UnitVector::new_normalize(Vector::x()))
                    .local_anchor1(Point::new(ROBOT_RADIUS*0.866, 0.))
                    .local_anchor2(Point::new(0., 0.))
                    .limits([0.0, 0.3])
                    .motor_position(0., KICKER_STRENGTH, 0.),
                true
            )
        });
        let kicker_joints = [
            kicker_joints.next().unwrap(),
            kicker_joints.next().unwrap(),
            kicker_joints.next().unwrap(),
            kicker_joints.next().unwrap()
        ];

        Self {
            bodies,
            colliders,
            goals,
            ball,
            ball_col,
            robots,
            kickers,
            kicker_joints,
            kicker_timer: [0; 4],
            gravity: vector![0.0, 0.0],
            integration_parameters: IntegrationParameters {
                dt: DT,
                ..IntegrationParameters::default()
            },
            physics_pipeline: PhysicsPipeline::new(),
            islands: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joints,
            multibody_joints: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
            physics_hooks: (),
            events: (),
            t: 0,
        }
    }
    pub fn step(&mut self) {
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.islands,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &self.physics_hooks,
            &self.events,
        );
        self.t += 1;
        for (((t, r), k), kj) in self.kicker_timer.iter_mut().zip(self.robots.iter()).zip(self.kickers.iter()).zip(self.kicker_joints.iter()) {
            if *t == 0 {
                let mut pos = *self.bodies.get(*r).unwrap().position();
                pos.append_translation_mut(&Translation::new(ROBOT_RADIUS*0.866*pos.rotation.angle().cos(), ROBOT_RADIUS*0.866*pos.rotation.angle().sin()));
                self.bodies.get_mut(*k)
                    .unwrap()
                    .set_position(pos, true);
                self.impulse_joints.get_mut(*kj)
                    .unwrap()
                    .data
                    .as_prismatic_mut()
                    .unwrap()
                    .set_motor_position(0., KICKER_STRENGTH, 0.);
            } else {
                *t -= 1;
            }
        }
    }
    pub fn find_entity_at(&self, pos: Point<f64>) -> Option<RigidBodyHandle> {
        let filter = QueryFilter::default();

        if let Some((handle, projection)) =
            self.query_pipeline
                .project_point(&self.bodies, &self.colliders, &pos, true, filter)
        {
            if projection.is_inside {
                return self.colliders[handle].parent();
            }
        }
        None
    }
    pub const fn get_ball_handle(&self) -> RigidBodyHandle {
        self.ball
    }
    pub const fn get_robot_handle(&self, id: Robot) -> RigidBodyHandle {
        self.robots[id as usize]
    }
    pub fn teleport_entity(&mut self, entity: RigidBodyHandle, pos: Point<f64>, r: Option<f64>) {
        let body = &mut self.bodies[entity];
        let mut iso: Isometry2<f64> = pos.into();
        iso.rotation = r.map(|r| Rotation::new(r)).unwrap_or_else(|| *body.rotation());
        body.set_position(iso, true);
        body.set_linvel(Vector::zeros(), true);
        body.set_angvel(0., true);
    }
    pub fn teleport_ball(&mut self, pos: Point<f64>) {
        self.teleport_entity(self.get_ball_handle(), pos, None);
    }
    pub fn teleport_robot(&mut self, id: Robot, pos: Point<f64>, r: Option<f64>) {
        self.teleport_entity(self.get_robot_handle(id), pos, r);
    }
    /// f between 0. and 1.
    pub fn kick(&mut self, id: Robot, f: f64) {
        self.impulse_joints.get_mut(self.kicker_joints[id as usize])
            .unwrap()
            .data
            .as_prismatic_mut()
            .unwrap()
            .set_motor_position(10., KICKER_STRENGTH*f, 0.);
        self.kicker_timer[id as usize] = 10;
    }
    pub fn reset(&mut self) {
        for (_, b) in self.bodies.iter_mut() {
            b.reset_forces(true);
            b.reset_torques(true);
            b.set_linvel(vector![0., 0.], true);
            b.set_angvel(0., true);
        }
        self.teleport_ball(DEFAULT_BALL_POS);
        for r in Robot::all() {
            self.teleport_robot(r, DEFAULT_ROBOTS_POS[r as usize], Some(DEFAULT_ROBOTS_ANGLE[r as usize]));
        }
        for i in 0..4 {
            self.bodies[self.kickers[i]].set_position(Isometry::new(Vector::new(DEFAULT_ROBOTS_POS[i].x + (if i < 2 {1.} else {-1.} * ((ROBOT_RADIUS*0.866) + (KICKER_THICKNESS/2.))), DEFAULT_ROBOTS_POS[i].y), DEFAULT_ROBOTS_ANGLE[i]), true);
        }
    }
}
