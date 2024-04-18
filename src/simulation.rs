use crate::constants::*;
use rapier2d::prelude::*;

pub struct Simulation {
    pub bodies: RigidBodySet,
    pub colliders: ColliderSet,
    pub goals: [ColliderHandle; 2],
    pub ball: RigidBodyHandle,
    pub robots: [RigidBodyHandle; 4],
    gravity: Vector<f32>,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    islands: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    ccd_solver: CCDSolver,
    pub query_pipeline: QueryPipeline,
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
            colliders.insert(ColliderBuilder::segment(
                point![MARGIN, MARGIN + ((FIELD.1 - GOAL_HEIGHT) / 2.)],
                point![MARGIN, MARGIN + ((FIELD.1 + GOAL_HEIGHT) / 2.)],
            )),
            colliders.insert(ColliderBuilder::segment(
                point![FIELD.0 + MARGIN, MARGIN + ((FIELD.1 - GOAL_HEIGHT) / 2.)],
                point![FIELD.0 + MARGIN, MARGIN + ((FIELD.1 + GOAL_HEIGHT) / 2.)],
            )),
        ];

        // Create the ball
        let ball = bodies.insert(RigidBodyBuilder::dynamic().position(DEFAULT_BALL_POS.into()));
        colliders.insert_with_parent(
            ColliderBuilder::ball(BALL_RADIUS)
                .restitution(BALL_RESTITUTION)
                .mass(BALL_MASS),
            ball,
            &mut bodies,
        );

        // Create the robots
        let robots = [
            bodies.insert(RigidBodyBuilder::dynamic().position(DEFAULT_BLUE1_POS.into())),
            bodies.insert(RigidBodyBuilder::dynamic().position(DEFAULT_BLUE2_POS.into())),
            bodies.insert(RigidBodyBuilder::dynamic().position(DEFAULT_GREEN1_POS.into())),
            bodies.insert(RigidBodyBuilder::dynamic().position(DEFAULT_GREEN2_POS.into())),
        ];
        for robot in robots.iter() {
            colliders.insert_with_parent(
                // Collider is a regular hexagon with radius ROBOT_RADIUS
                ColliderBuilder::convex_polyline(vec![
                    point![0., ROBOT_RADIUS],
                    point![ROBOT_RADIUS * 0.866, ROBOT_RADIUS * 0.5],
                    point![ROBOT_RADIUS * 0.866, -ROBOT_RADIUS * 0.5],
                    point![0., -ROBOT_RADIUS],
                    point![-ROBOT_RADIUS * 0.866, -ROBOT_RADIUS * 0.5],
                    point![-ROBOT_RADIUS * 0.866, ROBOT_RADIUS * 0.5],
                ])
                .unwrap(),
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
            broad_phase: BroadPhase::new(),
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
}
