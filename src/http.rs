use rapier2d::prelude::*;

use crate::game_state::GameState;

const WS_PORT: u16 = 1234;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ServerMsg {
    Initial(InitialMsg),
    GameState(GameState),
    FindEntityAtRes(Option<RigidBodyHandle>)
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ClientMsg {
    MoveEntity(RigidBodyHandle, Point<f32>),
    FindEntityAt(Point<f32>)
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InitialMsg {
    pub ball: RigidBodyHandle,
    pub blue1: RigidBodyHandle,
    pub blue2: RigidBodyHandle,
    pub green1: RigidBodyHandle,
    pub green2: RigidBodyHandle
}