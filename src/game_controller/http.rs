use std::{borrow::BorrowMut, cell::RefCell, rc::Rc};

/// Game controller that runs on a wasm client and communicates with the server via a websocket

use crate::{game_controller::GCTrait, game_state::GameState, http::Msg};
use log::warn;
use rapier2d::dynamics::RigidBodyHandle;
use wasm_sockets::{ConnectionStatus, EventClient, Message, PollingClient};

const HOST: &'static str = "127.0.0.1:1234";

pub struct GC {
    socket: PollingClient,
    gs: GameState,
    ball_handle: RigidBodyHandle
}
impl GCTrait for GC {
    fn new(
            blue_team_name: String,
            green_team_name: String,
            blue_team_positive: bool,
    ) -> Self {
        let gs = Rc::new(RefCell::new(GameState::default()));
        let gs_ref = gs.clone();
        let mut socket = EventClient::new(&format!("ws://{}", HOST)).unwrap();
        socket.set_on_message(Some(Box::new(|socket, msg| {
            let gs_bits = match msg {
                Message::Binary(gs_bits) => gs_bits,
                Message::Text(gs_bits) => gs_bits.into_bytes()
            };
            let msg: Msg = match bitcode::deserialize(&gs_bits) {
                Ok(gs) => gs,
                Err(e) => {
                    warn!("{}", e);
                    return
                }
            };
            match msg {
                Msg::Initial(msg) => todo!(),
                Msg::GameState(gs) => {gs_ref.replace(gs);}
            }
        })));
        // let mut socket = PollingClient::new(&format!("ws://{}", HOST)).unwrap();
        // while socket.status() == ConnectionStatus::Connecting {
        //     // Wait for the socket to connect
        // }
        // match socket.status() 
        // Self {
        //     socket,
        //     gs: GameState::default()
        // }
    }
    fn step(&mut self) {
        // We do nothing because the server is already handling the simulation
    }
    fn get_game_state(&self) -> crate::game_state::GameState {
        self.gs.clone()
    }
    fn get_ball_handle(&self) -> rapier2d::prelude::RigidBodyHandle {
        
    }
}