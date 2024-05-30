/// Distances in meters, mass in killograms, origin at the center of field

pub const DT: f64 = 1.0 / 100.0;
pub const FRAME_DURATION: usize = (DT * 1000.) as usize; // in ms
pub const PENALTY_DURATION: usize = 5000 / FRAME_DURATION; // in frames


/// Constants in simulation are multiplied because rapier bugs with small numbers
pub mod simu {
    use nalgebra::Point2;
    use super::real;

    pub use super::{DT, FRAME_DURATION, PENALTY_DURATION};
    pub use real::{DEFAULT_ROBOTS_ANGLE, BALL_RESTITUTION, BALL_DAMPING, ROBOT_DAMPING, ROBOT_ANGULAR_DAMPING, ROBOT_RESTITUTION};

    pub const MULTIPLIER: f64 = 100.;

    pub const FIELD: (f64, f64) = (real::FIELD.0*MULTIPLIER, real::FIELD.1*MULTIPLIER);
    pub const MARGIN: f64 = real::MARGIN/MULTIPLIER;
    pub const CARPET: (f64, f64) = (FIELD.0 + 2. * MARGIN, FIELD.1 + 2. * MARGIN);

    pub const GOAL_HEIGHT: f64 = real::GOAL_HEIGHT * MULTIPLIER;
    pub const GREEN_GOAL: (Point2<f64>, Point2<f64>) = (
        Point2::new(-FIELD.0/2., GOAL_HEIGHT/2.),
        Point2::new(-FIELD.0/2., -GOAL_HEIGHT/2.)
    );
    pub const BLUE_GOAL: (Point2<f64>, Point2<f64>) = (
        Point2::new(FIELD.0/2., GOAL_HEIGHT/2.),
        Point2::new(FIELD.0/2., -GOAL_HEIGHT/2.)
    );

    pub const DEFAULT_ROBOTS_POS: [Point2<f64>; 4] = [
        Point2::new(-FIELD.0/2., 0.),
        Point2::new(-FIELD.0/4., 0.),
        Point2::new(FIELD.0/4., 0.),
        Point2::new(FIELD.0/2., 0.)
    ];

    pub const DEFAULT_BALL_POS: Point2<f64> = Point2::new(0., 0.);
    pub const BALL_RADIUS: f64 = real::BALL_RADIUS * MULTIPLIER;
    pub const BALL_MASS: f64 = real::BALL_MASS * MULTIPLIER*MULTIPLIER*MULTIPLIER;

    pub const ROBOT_RADIUS: f64 = real::ROBOT_RADIUS * MULTIPLIER;
    pub const KICKER_THICKNESS: f64 = real::KICKER_THICKNESS * MULTIPLIER;
    pub const ROBOT_MASS: f64 = real::ROBOT_MASS * MULTIPLIER*MULTIPLIER*MULTIPLIER;
    pub const ROBOT_SPEED: f64 = real::ROBOT_SPEED * MULTIPLIER;
    pub const ROBOT_ANGULAR_SPEED: f64 = real::ROBOT_ANGULAR_SPEED * MULTIPLIER;
    pub const KICKER_REACH: f64 = real::KICKER_REACH * MULTIPLIER;
}

/// Real constants, without multiplier
pub mod real {
    use std::f64::consts::PI;

    use nalgebra::Point2;
    pub use super::{DT, FRAME_DURATION, PENALTY_DURATION};

    pub const FIELD: (f64, f64) = (1.83, 1.22);
    pub const MARGIN: f64 = 0.31;
    pub const CARPET: (f64, f64) = (FIELD.0 + 2. * MARGIN, FIELD.1 + 2. * MARGIN);

    pub const GOAL_HEIGHT: f64 = 0.6;
    pub const GREEN_GOAL: (Point2<f64>, Point2<f64>) = (
        Point2::new(-FIELD.0/2., GOAL_HEIGHT/2.),
        Point2::new(-FIELD.0/2., -GOAL_HEIGHT/2.)
    );
    pub const BLUE_GOAL: (Point2<f64>, Point2<f64>) = (
        Point2::new(FIELD.0/2., GOAL_HEIGHT/2.),
        Point2::new(FIELD.0/2., -GOAL_HEIGHT/2.)
    );

    pub const DEFAULT_ROBOTS_POS: [Point2<f64>; 4] = [
        Point2::new(-FIELD.0/2., 0.),
        Point2::new(-FIELD.0/4., 0.),
        Point2::new(FIELD.0/4., 0.),
        Point2::new(FIELD.0/2., 0.)
    ];
    pub const DEFAULT_ROBOTS_ANGLE: [f64; 4] = [
        0., 0., PI, PI
    ];

    pub const DEFAULT_BALL_POS: Point2<f64> = Point2::new(0., 0.);
    pub const BALL_RADIUS: f64 = 0.0213;
    pub const BALL_RESTITUTION: f64 = 0.; // Arbitrary. TODO: Mesure it
    pub const BALL_MASS: f64 = 0.008;
    pub const BALL_DAMPING: f64 = 1.;

    pub const ROBOT_RADIUS: f64 = 0.088; // From the python simulation. TODO: Mesure it
    pub const ROBOT_MASS: f64 = 1.; // Arbitrary. TODO: Mesure it
    pub const ROBOT_DAMPING: f64 = 0.; // Arbitrary. TODO: Mesure it
    pub const ROBOT_ANGULAR_DAMPING: f64 = 0.; // Arbitrary. TODO: Mesure it
    pub const ROBOT_RESTITUTION: f64 = 0.; // Arbitrary. TODO: Mesure it
    pub const ROBOT_SPEED: f64 = 0.035*PI*2.*150. / 60.; // Arbitrary. TODO: Mesure it
    pub const ROBOT_ANGULAR_SPEED: f64 = 0.1; // Arbitrary. TODO: Mesure it

    pub const KICKER_THICKNESS: f64 = 0.01; // Arbitrary. TODO: Mesure it
    pub const KICKER_REACH: f64 = 0.03; // Arbitrary. TODO: Mesure it
}