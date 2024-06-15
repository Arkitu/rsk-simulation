//! A server-client implementation of the simulation. In this version, the client only runs the GUI. The server serves a single client.
//! In tests, we find it slower than the default http.

#[cfg(target_arch = "wasm32")]
pub mod client;

#[cfg(not(target_arch = "wasm32"))]
pub mod server;

use nalgebra::Point2;
use rapier2d_f64::dynamics::RigidBodyHandle;

use crate::game_state::GameState;

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ServerMsg {
    Initial(InitialMsg),
    GameState(GameState),
    FindEntityAtRes(Option<RigidBodyHandle>)
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ClientMsg {
    TeleportEntity(RigidBodyHandle, Point2<f64>, Option<f64>),
    FindEntityAt(Point2<f64>),
    AllKick, // make all robots kick
    Reset
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InitialMsg {
    pub ball: RigidBodyHandle,
    pub blue1: RigidBodyHandle,
    pub blue2: RigidBodyHandle,
    pub green1: RigidBodyHandle,
    pub green2: RigidBodyHandle
}

pub const WS_PORT: u16 = 1234;