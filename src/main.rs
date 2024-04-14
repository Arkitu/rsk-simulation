use game_controller::GC;
use rapier2d::prelude::*;

pub const DT: f32 = 1.0 / 100.0;

// Distances in meters, mass in killograms, origin at the top left
const FIELD_LENGTH: f32 = 1.83;
const FIELD_HEIGHT: f32 = 1.22;
const MARGIN: f32 = 0.31;
const GOAL_HEIGHT: f32 = 0.6;

const DEFAULT_BLUE1_POS: Point<f32> = point![MARGIN, MARGIN+(FIELD_HEIGHT/2.)];
const DEFAULT_BLUE2_POS: Point<f32> = point![MARGIN+(FIELD_LENGTH/4.), MARGIN+(FIELD_HEIGHT/2.)];
const DEFAULT_GREEN1_POS: Point<f32> = point![MARGIN+(FIELD_LENGTH*3./4.), MARGIN+(FIELD_HEIGHT/2.)];
const DEFAULT_GREEN2_POS: Point<f32> = point![MARGIN+FIELD_LENGTH, MARGIN+(FIELD_HEIGHT/2.)];

const DEFAULT_BALL_POS: Point<f32> = point![MARGIN+(FIELD_LENGTH/2.), MARGIN+(FIELD_HEIGHT/2.)];
const BALL_RADIUS: f32 = 0.0213;
const BALL_RESTITUTION: f32 = 0.7; // TODO: Mesure it
const BALL_MASS: f32 = 0.008;

const ROBOT_RADIUS: f32 = 0.088; // From the python simulation. TODO: Mesure it

mod game_state;
mod game_controller;

#[cfg(feature = "gui")]
mod gui;

fn main() {
    let mut bodies = RigidBodySet::new();
    let mut colliders = ColliderSet::new();

    // Create the goals
    let goals = [
        colliders.insert(ColliderBuilder::segment(
            point![MARGIN, (FIELD_HEIGHT-GOAL_HEIGHT)/2.],
            point![MARGIN, (FIELD_HEIGHT+GOAL_HEIGHT)/2.]
        )),
        colliders.insert(ColliderBuilder::segment(
            point![FIELD_LENGTH+MARGIN, (FIELD_HEIGHT-GOAL_HEIGHT)/2.],
            point![FIELD_LENGTH+MARGIN, (FIELD_HEIGHT+GOAL_HEIGHT)/2.]
        ))
    ];

    // Create the ball
    let ball = bodies.insert(
        RigidBodyBuilder::dynamic()
            .position(DEFAULT_BALL_POS.into())
    );
    colliders.insert_with_parent(
        ColliderBuilder::ball(BALL_RADIUS)
            .restitution(BALL_RESTITUTION)
            .mass(BALL_MASS),
        ball,
        &mut bodies
    );

    // Create the robots
    let robots = [
        bodies.insert(
            RigidBodyBuilder::dynamic()
                .position(DEFAULT_BLUE1_POS.into())
        ),
        bodies.insert(
            RigidBodyBuilder::dynamic()
                .position(DEFAULT_BLUE2_POS.into())
        ),
        bodies.insert(
            RigidBodyBuilder::dynamic()
                .position(DEFAULT_GREEN1_POS.into())
        ),
        bodies.insert(
            RigidBodyBuilder::dynamic()
                .position(DEFAULT_GREEN2_POS.into())
        )
    ];
    for robot in robots.iter() {
        colliders.insert_with_parent(
            // Collider is a regular hexagon with radius ROBOT_RADIUS
            ColliderBuilder::convex_polyline(vec![
                point![0., ROBOT_RADIUS],
                point![ROBOT_RADIUS*0.866, ROBOT_RADIUS*0.5],
                point![ROBOT_RADIUS*0.866, -ROBOT_RADIUS*0.5],
                point![0., -ROBOT_RADIUS],
                point![-ROBOT_RADIUS*0.866, -ROBOT_RADIUS*0.5],
                point![-ROBOT_RADIUS*0.866, ROBOT_RADIUS*0.5],
            ]).unwrap(),
            *robot,
            &mut bodies
        );
    }

    /* Create other structures necessary for the simulation. */
    let gravity = vector![0.0, 0.0];
    let integration_parameters = IntegrationParameters {
        dt: DT,
        ..IntegrationParameters::default()
    };
    let mut physics_pipeline = PhysicsPipeline::new();
    let mut islands = IslandManager::new();
    let mut broad_phase = BroadPhase::new();
    let mut narrow_phase = NarrowPhase::new();
    let mut impulse_joints = ImpulseJointSet::new();
    let mut multibody_joints = MultibodyJointSet::new();
    let mut ccd_solver = CCDSolver::new();
    let mut query_pipeline = QueryPipeline::new();
    let physics_hooks = ();
    let events = ();

    let mut gc = GC::new(
        ball,
        robots,
        "",
        "",
        true
    );

    /* Run the game loop, stepping the simulation once per frame. */
    for t in 0..200 {
        physics_pipeline.step(
            &gravity,
            &integration_parameters,
            &mut islands,
            &mut broad_phase,
            &mut narrow_phase,
            &mut bodies,
            &mut colliders,
            &mut impulse_joints,
            &mut multibody_joints,
            &mut ccd_solver,
            Some(&mut query_pipeline),
            &physics_hooks,
            &events,
        );

        println!("{:#?}", gc.get_game_state(&bodies, t));
    }
}