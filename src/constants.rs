use rapier2d::prelude::*;

pub const DT: f32 = 1.0 / 100.0;

// Distances in meters, mass in killograms, origin at the top left
pub const FIELD: (f32, f32) = (1.83, 1.22);
pub const MARGIN: f32 = 0.31;
pub const CARPET: (f32, f32) = (FIELD.0 + 2. * MARGIN, FIELD.1 + 2. * MARGIN);
pub const GOAL_HEIGHT: f32 = 0.6;

pub const DEFAULT_BLUE1_POS: Point<f32> = point![MARGIN, CARPET.1 / 2.];
pub const DEFAULT_BLUE2_POS: Point<f32> = point![MARGIN + (FIELD.0 / 4.), CARPET.1 / 2.];
pub const DEFAULT_GREEN1_POS: Point<f32> = point![MARGIN + (FIELD.0 * 3. / 4.), CARPET.1 / 2.];
pub const DEFAULT_GREEN2_POS: Point<f32> = point![MARGIN + FIELD.0, CARPET.1 / 2.];

pub const DEFAULT_BALL_POS: Point<f32> = point![MARGIN + (FIELD.0 / 2.), MARGIN + (FIELD.1 / 2.)];
pub const BALL_RADIUS: f32 = 0.0213;
pub const BALL_RESTITUTION: f32 = 0.7; // TODO: Mesure it
pub const BALL_MASS: f32 = 0.008;

pub const ROBOT_RADIUS: f32 = 0.088; // From the python simulation. TODO: Mesure it

pub const FRAME_DURATION: usize = (DT * 1000.) as usize; // in ms
pub const PENALTY_DURATION: usize = 5000 / FRAME_DURATION; // in frames
