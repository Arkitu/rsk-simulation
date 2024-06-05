use std::{sync::Arc, thread};
use tokio::{join, runtime::Handle, sync::Mutex};
use tracing::warn;
use zeromq::{PubSocket, RepSocket, Socket, SocketSend, SocketRecv};

use serde_json::Value;

use crate::game_state::{GameState, Robot, RobotTasks};

use super::CtrlRes;

pub struct Control {
    state_socket: PubSocket
}
impl Control {
    pub fn new(keys: [String; 2], tasks: Arc<Mutex<[RobotTasks; 4]>>) -> Self {
        let mut state_socket = PubSocket::new();
        let mut ctrl_socket = RepSocket::new();

        let handle = Handle::current();
        handle.block_on(state_socket.bind("tcp://127.0.0.1:7557"));
        handle.block_on(ctrl_socket.bind("tcp://127.0.0.1:7558"));

        tokio::spawn(async move {
            loop {
                let msg = ctrl_socket.recv().await.unwrap();
                let req = match msg.get(0) {
                    Some(req) => req,
                    None => {
                        warn!("Received empty message");
                        continue
                    }
                };
                let mut res = CtrlRes::UnknownError;
                let (key, team, number, cmd) : (String, String, u8, Vec<Value>) = serde_json::from_slice(&req).unwrap();
                match team.as_str() {
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
                                let mut tasks = tasks.lock().await;
                                let mut preempted = false;
                                if let Some((r, _)) = tasks[r as usize].penalty {
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
                ctrl_socket.send(serde_json::to_vec(&res).unwrap().into()).await;
            }
        });
        Self {
            state_socket
        }
    }
    /// Send new game state to client
    pub fn publish(&mut self, gs: GameState) {
        let json = serde_json::to_vec(&gs).unwrap();
        Handle::current().block_on(self.state_socket.send(json.into())).unwrap();
    }
}