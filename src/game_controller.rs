use std::{cell::RefCell, rc::Rc};

use rapier2d::prelude::*;
use crate::game_state::{GameState, Robot};

#[cfg(feature = "standard_gc")]
mod standard;

#[cfg(feature = "http_gc")]
mod http;

#[cfg(not(feature = "async"))]
pub trait GCTrait {
    fn new(
        blue_team_name: String,
        green_team_name: String,
        blue_team_positive: bool,
    ) -> Self;
    fn step(&mut self);
    fn get_game_state(&self) -> GameState;
    fn teleport_ball(&mut self, pos: Point<f32>) {
        self.teleport_entity(self.get_ball_handle(), pos);
    }
    fn teleport_robot(&mut self, id: Robot, pos: Point<f32>) {
        self.teleport_entity(self.get_robot_handle(id), pos);
    }
    fn teleport_entity(&mut self, entity: RigidBodyHandle, pos: Point<f32>);
    fn find_entity_at(&mut self, pos: Point<f32>) -> Option<RigidBodyHandle>;
    fn get_ball_handle(&self) -> RigidBodyHandle;
    fn get_robot_handle(&self, id: Robot) -> RigidBodyHandle;
}

#[cfg(feature = "async")]
pub trait GCTrait {
    async fn new(
        blue_team_name: String,
        green_team_name: String,
        blue_team_positive: bool,
    ) -> Self;
    async fn step(&mut self);
    fn get_game_state(&self) -> GameState;
    fn teleport_ball(&mut self, pos: Point<f32>) {
        self.teleport_entity(self.get_ball_handle(), pos);
    }
    fn teleport_robot(&mut self, id: Robot, pos: Point<f32>) {
        self.teleport_entity(self.get_robot_handle(id), pos);
    }
    fn teleport_entity(&mut self, entity: RigidBodyHandle, pos: Point<f32>);
    async fn find_entity_at(&mut self, pos: Point<f32>) -> Option<RigidBodyHandle>;
    /// Same as find_entity_at but put result (or default if it's None) in rc
    /// (Sorry for this part of code, it's for a very specific problem between http_gc and bevy_gui, that I probably didn't solve in the most elegant way)
    fn find_entity_at_rc(&self, pos: Point<f32>, rc: Rc<RefCell<Option<RigidBodyHandle>>>, default: Option<RigidBodyHandle>);
    fn get_ball_handle(&self) -> RigidBodyHandle;
    fn get_robot_handle(&self, id: Robot) -> RigidBodyHandle;
}

#[cfg(all(feature = "standard_gc", feature = "http_gc"))]
compile_error!("Multiple game controller features enabled. You can only enable one game controller feature.");

#[cfg(feature = "standard_gc")]
pub use standard::GC;

#[cfg(feature = "http_gc")]
pub use http::GC;

#[cfg(not(any(feature = "standard_gc", feature = "http_gc")))]
compile_error!(
    "No game controller feature enabled. You need to enable at least one game controller feature."
);
