use rapier2d::prelude::*;

// Distances in meters, mass in killograms, origin at the top left
const FIELD_LENGTH: f32 = 1.83;
const FIELD_HEIGHT: f32 = 1.22;
const MARGIN: f32 = 0.31;
const GOAL_HEIGHT: f32 = 0.6;

const DEFAULT_BALL_POS: Point<f32> = point![MARGIN+(FIELD_LENGTH/2.), MARGIN+(FIELD_HEIGHT/2.)];
const BALL_RADIUS: f32 = 0.0427;
const BALL_RESTITUTION: f32 = 0.7; // TODO: Mesure it
const BALL_MASS: f32 = 0.008;

#[cfg(feature = "gui")]
mod gui;

fn main() {
    let mut rigid_body_set = RigidBodySet::new();
    let mut collider_set = ColliderSet::new();

    // Create the goals
    let goals = [
        collider_set.insert(ColliderBuilder::segment(
            point![MARGIN, (FIELD_HEIGHT-GOAL_HEIGHT)/2.],
            point![MARGIN, (FIELD_HEIGHT+GOAL_HEIGHT)/2.]
        )),
        collider_set.insert(ColliderBuilder::segment(
            point![FIELD_LENGTH+MARGIN, (FIELD_HEIGHT-GOAL_HEIGHT)/2.],
            point![FIELD_LENGTH+MARGIN, (FIELD_HEIGHT+GOAL_HEIGHT)/2.]
        ))
    ];

    // Create the ball
    let ball = rigid_body_set.insert(
        RigidBodyBuilder::dynamic()
            .position(DEFAULT_BALL_POS.into())
    );
    collider_set.insert_with_parent(
        ColliderBuilder::ball(BALL_RADIUS)
            .restitution(BALL_RESTITUTION)
            .mass(BALL_MASS),
        ball,
        &mut rigid_body_set
    );

    /* Create other structures necessary for the simulation. */
    let gravity = vector![0.0, 0.0];
    let integration_parameters = IntegrationParameters::default();
    let mut physics_pipeline = PhysicsPipeline::new();
    let mut island_manager = IslandManager::new();
    let mut broad_phase = BroadPhase::new();
    let mut narrow_phase = NarrowPhase::new();
    let mut impulse_joint_set = ImpulseJointSet::new();
    let mut multibody_joint_set = MultibodyJointSet::new();
    let mut ccd_solver = CCDSolver::new();
    let mut query_pipeline = QueryPipeline::new();
    let physics_hooks = ();
    let event_handler = ();

    /* Run the game loop, stepping the simulation once per frame. */
    for _ in 0..200 {
        physics_pipeline.step(
        &gravity,
        &integration_parameters,
        &mut island_manager,
        &mut broad_phase,
        &mut narrow_phase,
        &mut rigid_body_set,
        &mut collider_set,
        &mut impulse_joint_set,
        &mut multibody_joint_set,
        &mut ccd_solver,
        Some(&mut query_pipeline),
        &physics_hooks,
        &event_handler,
        );

        let ball_body = &rigid_body_set[ball];
        println!(
        "Ball pos: {}",
        ball_body.translation()
        );
    }
}