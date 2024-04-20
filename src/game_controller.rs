use rapier2d::prelude::*;
use crate::game_state::{GameState, Robot};

#[cfg(feature = "standard_game_controller")]
mod standard;

#[cfg(feature = "http_game_controller")]
mod http;

pub trait GCTrait {
    fn new(
        blue_team_name: String,
        green_team_name: String,
        blue_team_positive: bool,
    ) -> Self;
    fn step(&mut self);
    fn get_game_state(&self) -> GameState;
    fn teleport_ball(&mut self, pos: Point<f32>);
    fn teleport_robot(&mut self, id: Robot, pos: Point<f32>);
    fn move_entity(&mut self, entity: RigidBodyHandle, pos: Point<f32>);
    fn find_entity_at(&self, pos: Point<f32>) -> Option<RigidBodyHandle>;
    fn get_ball_handle(&self) -> RigidBodyHandle;
    fn get_robot_handle(&self, id: Robot) -> RigidBodyHandle;
}

#[cfg(all(feature = "standard_game_controller", feature = "http_game_controller"))]
compile_error!("Multiple game controller features enabled. You can only enable one game controller feature.");

#[cfg(feature = "standard_game_controller")]
pub use standard::GC;

#[cfg(feature = "http_game_controller")]
pub use http::GC;

#[cfg(not(any(feature = "standard_game_controller", feature = "http_game_controller")))]
compile_error!(
    "No game controller feature enabled. You need to enable at least one game controller feature."
);
