use rapier2d_f64::prelude::*;

use crate::game_state::GameState;

pub const WS_PORT: u16 = 1234;

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
    TeleportEntity(RigidBodyHandle, Point<f64>, Option<f64>),
    FindEntityAt(Point<f64>),
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