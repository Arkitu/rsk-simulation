use std::{cell::RefCell, rc::Rc};

use serde_json::Value;
use tracing::info;
use wasm_sockets::{ConnectionStatus, EventClient, Message};

use crate::game_state::{GameState, Robot, RobotTask};
use crate::http::default::ClientMsg;

use super::CtrlRes;

const HOST: &'static str = "127.0.0.1:1234";

pub struct Control {
    socket: EventClient,
}
impl Control {
    pub fn new(keys: [String; 2], tasks: Rc<RefCell<[Option<RobotTask>; 4]>>) -> Self {
        let mut socket = EventClient::new(&format!("ws://{}", HOST)).unwrap();

        socket.set_on_connection(Some(Box::new(|socket| {
            info!("Socket connected");
        })));

        socket.set_on_message(Some(Box::new(move |socket, msg| {
            let req = match msg {
                Message::Binary(bits) => bits,
                Message::Text(string) => string.into_bytes(),
            };
            let mut res = CtrlRes::UnknownError;
            match serde_json::from_slice::<(String, String, u8, Vec<Value>)>(&req) {
                Ok((key, team, number, cmd)) => match team.as_str() {
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
                                _ => None,
                            } {
                                let mut tasks = tasks.borrow_mut();
                                let mut preempted = false;
                                if let Some(ref t) = tasks[r as usize] {
                                    if let Some(r) = t.preemption_reason(r) {
                                        preempted = true;
                                        res = CtrlRes::Preempted(team, number, r);
                                    }
                                }
                                if !preempted {
                                    match cmd.len() {
                                        4 => match &cmd[0] {
                                            Value::String(c) => match c.as_str() {
                                                "control" => {
                                                    tasks[r as usize] = Some(RobotTask::Control {
                                                        x: cmd[1].as_f64().unwrap_or(0.) as f32,
                                                        y: cmd[2].as_f64().unwrap_or(0.) as f32,
                                                        r: cmd[3].as_f64().unwrap_or(0.) as f32,
                                                    });
                                                    res = CtrlRes::Ok;
                                                }
                                                _ => res = CtrlRes::UnknownCommand,
                                            },
                                            _ => res = CtrlRes::UnknownCommand,
                                        },
                                        _ => res = CtrlRes::UnknownCommand,
                                    }
                                }
                            } else {
                                res = CtrlRes::UnknownRobot(team, number);
                            }
                        }
                    }
                    "ball" => todo!(),
                    _ => {
                        dbg!(key, team, number, cmd);
                    }
                },
                _ => {}
            }
            let res = serde_json::to_vec(&res).unwrap();
            socket
                .send_binary(bitcode::serialize(&ClientMsg::CtrlRes(res)).unwrap())
                .unwrap();
        })));

        Self { socket }
    }
    /// Send new game state to client
    pub fn publish(&self, gs: GameState) {
        if let ConnectionStatus::Connected = self.socket.status.borrow().clone() {
            self.socket
                .send_binary(bitcode::serialize(&ClientMsg::GameState(gs)).unwrap())
                .unwrap();
        }
    }
}
