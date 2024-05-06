use std::{cell::RefCell, collections::VecDeque, rc::Rc, time::Duration};

/// Game controller that runs on a wasm client and communicates with the server via a websocket

use crate::{game_state::{GameState, Robot}, http::{ClientMsg, ServerMsg}};
use gloo_timers::future::sleep;
use nalgebra::Point2;
use tracing::{info, warn};
use rapier2d_f64::dynamics::RigidBodyHandle;
use wasm_sockets::{EventClient, Message};

const HOST: &'static str = "127.0.0.1:1234";

pub struct GC {
    socket: EventClient,
    gs: Rc<RefCell<GameState>>,
    ball: RigidBodyHandle,
    blue1: RigidBodyHandle,
    blue2: RigidBodyHandle,
    green1: RigidBodyHandle,
    green2: RigidBodyHandle,
    /// Find entity at requests (yes it's ugly)
    find_entity_at: Rc<RefCell<VecDeque<Rc<RefCell<Option<Option<RigidBodyHandle>>>>>>>
}
impl GC {
    pub async fn new(
            blue_team_name: String,
            green_team_name: String,
            blue_team_positive: bool,
    ) -> Self {
        info!("New GC");
        let mut socket = EventClient::new(&format!("ws://{}", HOST)).unwrap();

        socket.set_on_connection(Some(Box::new(|socket| {
            info!("Socket connected");
        })));
        
        let msg = Rc::new(RefCell::new(None));
        let msg_rc = msg.clone();
        socket.set_on_message(Some(Box::new(move |socket, msg| {
            let bits = match msg {
                Message::Binary(bits) => bits,
                Message::Text(string) => string.into_bytes()
            };
            let msg: ServerMsg = match bitcode::deserialize(&bits) {
                Ok(msg) => msg,
                Err(e) => {
                    warn!("{}", e);
                    return
                }
            };
            if let ServerMsg::Initial(msg) = msg {
                msg_rc.replace(Some(msg));
            }
        })));
        while msg.borrow().is_none() {
            sleep(Duration::from_millis(0)).await;
        }
        let initial_msg = msg.take().unwrap();

        let gs = Rc::new(RefCell::new(GameState::default()));
        let gs_rc = gs.clone();
        let find_entity_at: Rc<RefCell<VecDeque<Rc<RefCell<Option<Option<RigidBodyHandle>>>>>>> = Rc::new(RefCell::new(VecDeque::new()));
        let mut fia_rc = find_entity_at.clone();
        socket.set_on_message(Some(Box::new(move |socket, msg| {
            let msg_bits = match msg {
                Message::Binary(bits) => bits,
                Message::Text(string) => string.into_bytes()
            };
            let msg: ServerMsg = match bitcode::deserialize(&msg_bits) {
                Ok(msg) => msg,
                Err(e) => {
                    warn!("{}", e);
                    return
                }
            };
            match msg {
                ServerMsg::GameState(gs) => {gs_rc.replace(gs);},
                ServerMsg::FindEntityAtRes(res) => match (*fia_rc).borrow_mut().pop_front() {
                    Some(rc) => {rc.replace(Some(res));},
                    None => {
                        warn!("Find entity response but no request");
                        return
                    }
                },
                msg => warn!("Unknown msg : {:?}", msg)
            }
        })));

        Self {
            socket,
            gs,
            find_entity_at,
            ball: initial_msg.ball,
            blue1: initial_msg.blue1,
            blue2: initial_msg.blue2,
            green1: initial_msg.green1,
            green2: initial_msg.green2
        }
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
    pub async fn step(&mut self) {
        // We do nothing because the server is already handling the simulation
    }
    pub fn get_game_state(&self) -> crate::game_state::GameState {
        self.gs.borrow().clone()
    }
    pub fn get_ball_handle(&self) -> RigidBodyHandle {
        self.ball
    }
    pub fn get_robot_handle(&self, id: crate::game_state::Robot) -> RigidBodyHandle {
        match id {
            Robot::Blue1 => self.blue1,
            Robot::Blue2 => self.blue2,
            Robot::Green1 => self.green1,
            Robot::Green2 => self.green2
        }
    }
    pub fn teleport_entity(&mut self, entity: RigidBodyHandle, pos: Point2<f64>, r: Option<f64>) {
        let msg = ClientMsg::TeleportEntity(entity, pos, r);
        let msg_bits = bitcode::serialize(&msg).unwrap();
        self.socket.send_binary(msg_bits).unwrap();
    }
    pub fn find_entity_at_rc(&self, pos: Point2<f64>, rc: Rc<RefCell<Option<RigidBodyHandle>>>, default: Option<RigidBodyHandle>) {
        let msg = ClientMsg::FindEntityAt(pos);
        let msg_bits = bitcode::serialize(&msg).unwrap();
        self.socket.send_binary(msg_bits).unwrap();
        let id = Rc::new(RefCell::new(None));
        let id_rc = id.clone();
        (*self.find_entity_at).borrow_mut().push_back(id_rc);
        wasm_bindgen_futures::spawn_local(async move {
            while id.borrow().is_none() {
                sleep(Duration::from_millis(0)).await;
            }
            rc.replace(match id.take().unwrap() {
                Some(id) => Some(id),
                None => default
            });
        });
    }
    pub async fn find_entity_at(&mut self, pos: Point2<f64>) -> Option<RigidBodyHandle> {
        let msg = ClientMsg::FindEntityAt(pos);
        let msg_bits = bitcode::serialize(&msg).unwrap();
        self.socket.send_binary(msg_bits).unwrap();
        let id = Rc::new(RefCell::new(None));
        let id_rc = id.clone();
        (*self.find_entity_at).borrow_mut().push_back(id_rc);
        while id.borrow().is_none() {
            sleep(Duration::from_millis(0)).await;
        }
        id.take().unwrap()
    }
    pub fn reset(&mut self) {
        let msg_bits = bitcode::serialize(&ClientMsg::Reset).unwrap();
        self.socket.send_binary(msg_bits).unwrap();
    }
}