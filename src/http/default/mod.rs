use serde_json::Value;
use crate::game_state::GameState;

#[cfg(all(feature = "http_client", target_arch = "wasm32"))]
pub mod client;

#[cfg(all(feature = "http_server", not(target_arch = "wasm32")))]
pub mod server;

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

pub const WS_PORT: u16 = 1234;