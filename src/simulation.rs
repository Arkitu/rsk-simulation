use crate::{constants::simu::*, game_state::Robot};
use nalgebra::Isometry2;
use rapier2d_f64::prelude::*;

pub struct Simulation {
    pub bodies: RigidBodySet,
    pub colliders: ColliderSet,
    pub goals: [ColliderHandle; 2],
    pub ball: RigidBodyHandle,
    pub robots: [RigidBodyHandle; 4],
    gravity: Vector<f64>,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    islands: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
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

        // Create the goals
        let goals = [
            colliders.insert(ColliderBuilder::segment(GREEN_GOAL.0, GREEN_GOAL.1).sensor(true)),
            colliders.insert(ColliderBuilder::segment(BLUE_GOAL.0, BLUE_GOAL.1).sensor(true)),
        ];

        // Create the ball
        let ball = bodies.insert(
            RigidBodyBuilder::dynamic()
                .position(DEFAULT_BALL_POS.into())
                .linear_damping(BALL_DAMPING)
                .can_sleep(false)
                .dominance_group(-1),
        );
        colliders.insert_with_parent(
            ColliderBuilder::ball(BALL_RADIUS)
                .restitution(BALL_RESTITUTION)
                .mass(BALL_MASS),
            ball,
            &mut bodies,
        );

        // Create the robots
        let robots = std::array::from_fn(|i| {
            bodies.insert(
                RigidBodyBuilder::dynamic()
                    .position(DEFAULT_ROBOTS_POS[i].into())
                    .rotation(DEFAULT_ROBOTS_ANGLE[i])
                    .linear_damping(ROBOT_DAMPING)
                    .angular_damping(ROBOT_ANGULAR_DAMPING)
                    .can_sleep(false),
            )
        });
        for robot in robots.iter() {
            const R: f64 = ROBOT_RADIUS - 0.001;
            colliders.insert_with_parent(
                // Collider is a regular hexagon with radius ROBOT_RADIUS
                ColliderBuilder::round_convex_hull(
                    &[
                        point![0., R],
                        point![R * 0.866, R * 0.5],
                        point![R * 0.866, -R * 0.5],
                        point![0., -R],
                        point![-R * 0.866, -R * 0.5],
                        point![-R * 0.866, R * 0.5],
                    ],
                    0.001,
                )
                .unwrap()
                .mass(ROBOT_MASS)
                .restitution(ROBOT_RESTITUTION),
                *robot,
                &mut bodies,
            );
        }

        Self {
            bodies,
            colliders,
            goals,
            ball,
            robots,
            gravity: vector![0.0, 0.0],
            integration_parameters: IntegrationParameters {
                dt: DT,
                ..IntegrationParameters::default()
            },
            physics_pipeline: PhysicsPipeline::new(),
            islands: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joints: ImpulseJointSet::new(),
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
        iso.rotation = r
            .map(|r| Rotation::new(r))
            .unwrap_or_else(|| *body.rotation());
        body.set_position(iso, true);
    }
    pub fn teleport_ball(&mut self, pos: Point<f64>) {
        self.teleport_entity(self.get_ball_handle(), pos, None);
    }
    pub fn teleport_robot(&mut self, id: Robot, pos: Point<f64>, r: Option<f64>) {
        self.teleport_entity(self.get_robot_handle(id), pos, r);
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
            self.teleport_robot(
                r,
                DEFAULT_ROBOTS_POS[r as usize],
                Some(DEFAULT_ROBOTS_ANGLE[r as usize]),
            );
        }
    }
}
