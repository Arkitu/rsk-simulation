use rapier2d_f64::prelude::*;

pub const WS_PORT: u16 = 1234;

#[cfg(feature = "alternative_http")]
pub mod alternative {
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
}

#[cfg(feature = "default_http")]
pub mod default {
    use serde_json::Value;
    use crate::game_state::GameState;

    #[derive(Debug)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub enum ClientMsg {
        // (id)
        InitialMsg(String),
        GameState(GameState),
        CtrlRes(Vec<u8>)
    }

    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub enum ServerMsg {
        Ctrl(String, String, u8, Vec<Value>)
    }
}