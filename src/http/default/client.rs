use std::time::Duration;
use std::{cell::RefCell, rc::Rc};

use serde_json::Value;
use tracing::{info, error};
use wasm_sockets::{ConnectionStatus, EventClient, Message};
use wasm_timer::Instant;

use crate::game_state::{GameState, Robot, RobotTasks};
use crate::http::default::{ClientMsg, ServerMsg};
use crate::native;

use crate::control::CtrlRes;

const HOST: &'static str = "127.0.0.1:1234";
const PUBLISH_RATE: Duration = Duration::from_millis(50);

pub fn main() {
    use log::info;
    use url::Url;
    use rand::distributions::{Alphanumeric, DistString};

    console_log::init_with_level(log::Level::Debug).expect("error initializing log");
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let mut location = web_sys::window().unwrap().location();
    let mut url = Url::parse(&location.href().unwrap()).unwrap();
    if url.path().len() <= 1 {
        url.set_path(&("/".to_string() + &Alphanumeric.sample_string(&mut rand::thread_rng(), 5)));
        location.set_href(url.as_str()).unwrap();
        return
    }

    let mut session_id = url.path();
    if session_id.starts_with("/") {
        session_id = &session_id[1..];
    }
    info!("New session (id : {})", session_id);

    let mut gc = native::gc::GC::new("".to_string(), "".to_string(), session_id.to_string(), session_id.to_string(), false, session_id);

    native::gui::BevyGUI::run(gc);
}

pub struct Control {
    socket: EventClient,
    last_publish: Instant
}
impl Control {
    pub fn new(keys: [String; 2], tasks: Rc<RefCell<[RobotTasks; 4]>>, session_id: &str) -> Self {
        let mut socket = EventClient::new(&format!("ws://{}/{}", HOST, session_id)).unwrap();

        let sid = session_id.to_string();
        socket.set_on_connection(Some(Box::new(move |socket| {
            socket.send_binary(
                bitcode::serialize(&ClientMsg::InitialMsg(sid.clone())).unwrap()
            ).unwrap();
            info!("Socket connected");
        })));

        socket.set_on_message(Some(Box::new(move |socket, msg| {
            let req = match msg {
                Message::Text(string) => string,
                Message::Binary(_) => {
                    error!("Received binary message from ws");
                    return
                },
            };
            let msg: ServerMsg = serde_json::from_str(&req).unwrap();
            let mut res = CtrlRes::UnknownError;
            match msg {
                ServerMsg::Ctrl(key, team, number, cmd) => match team.as_str() {
                    "blue" | "green" => {
                        let num = (team == "green") as usize;
                        if keys[num] != key {
                            res = CtrlRes::BadKey(team);
                        } else {
                            // TODO: Add option to disable control for one team
                            if let Some(r) = match (team.as_str(), number) {
                                ("blue", 1) => Some(Robot::Blue1),
                                ("blue", 2) => Some(Robot::Blue2),
                                ("green", 1) => Some(Robot::Green1),
                                ("green", 2) => Some(Robot::Green2),
                                _ => None
                            } {
                                let mut tasks = tasks.borrow_mut();
                                let mut preempted = false;
                                if let Some((r, _, _)) = tasks[r as usize].penalty {
                                    preempted = true;
                                    res = CtrlRes::Preempted(team, number, r.to_string());
                                }
                                if !preempted {
                                    match cmd.len() {
                                        4 => match &cmd[0] {
                                            Value::String(c) => match c.as_str() {
                                                "control" => {
                                                    tasks[r as usize].control = (
                                                        cmd[1].as_f64().unwrap_or(0.) as f32,
                                                        cmd[2].as_f64().unwrap_or(0.) as f32,
                                                        cmd[3].as_f64().unwrap_or(0.) as f32
                                                    );
                                                    res = CtrlRes::Ok;
                                                }
                                                _ => res = CtrlRes::UnknownCommand
                                            },
                                            _ => res = CtrlRes::UnknownCommand
                                        },
                                        2 => match &cmd[0] {
                                            Value::String(c) => match c.as_str() {
                                                "kick" => {
                                                    tasks[r as usize].kick = Some(cmd[1].as_f64().unwrap_or(0.) as f32);
                                                    res = CtrlRes::Ok;
                                                }
                                                _ => res = CtrlRes::UnknownCommand
                                            },
                                            _ => res = CtrlRes::UnknownCommand
                                        }
                                        _ => res = CtrlRes::UnknownCommand
                                    }
                                }
                            } else {
                                res = CtrlRes::UnknownRobot(team, number);
                            }
                        }
                    },
                    "ball" => todo!(),
                    _ => {dbg!(key, team, number, cmd);}
                }
            }
            let res = serde_json::to_vec(&res).unwrap();
            socket.send_binary(
                bitcode::serialize(&ClientMsg::CtrlRes(res)).unwrap()
            ).unwrap();
        })));

        Self {
            socket,
            last_publish: Instant::now()
        }
    }
    /// Send new game state to client
    pub fn publish(&mut self, gs: GameState) {
        if self.last_publish.elapsed() > PUBLISH_RATE {
            if let ConnectionStatus::Connected = self.socket.status.borrow().clone() {
                self.socket.send_binary(
                    bitcode::serialize(&ClientMsg::GameState(gs)).unwrap()
                ).unwrap();
            }
            self.last_publish = Instant::now();
        }
    }
}