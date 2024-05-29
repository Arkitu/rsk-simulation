use std::{sync::{Arc, Mutex}, thread};

use serde_json::Value;
use zmq::{Context, Socket};

use crate::game_state::{GameState, Robot, RobotTasks};

use super::CtrlRes;

pub struct Control {
    ctrl_thread: thread::JoinHandle<()>,
    state_socket: Socket
}
impl Control {
    pub fn new(keys: [String; 2], tasks: Arc<Mutex<[RobotTasks; 4]>>) -> Self {
        let ctx = Context::new();
        
        let state_socket = ctx.socket(zmq::PUB).unwrap();
        state_socket.bind("tcp://*:7557").unwrap();

        let ctrl_socket = ctx.socket(zmq::REP).unwrap();
        ctrl_socket.bind("tcp://*:7558").unwrap();

        Self {
            ctrl_thread: thread::spawn(move || {
                loop {
                    let req = ctrl_socket.recv_bytes(0).unwrap();
                    let mut res = CtrlRes::UnknownError;
                    let (key, team, number, cmd) : (String, String, u8, Vec<Value>) = serde_json::from_slice(&req).unwrap();
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
                                        _ => None
                                    } {
                                        let mut tasks = tasks.lock().unwrap();
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
                                                            tasks[r as usize].control = Some((
                                                                cmd[1].as_f64().unwrap_or(0.) as f32,
                                                                cmd[2].as_f64().unwrap_or(0.) as f32,
                                                                cmd[3].as_f64().unwrap_or(0.) as f32
                                                            ));
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
                        _ => {}
                    }
                    ctrl_socket.send(serde_json::to_vec(&res).unwrap(), 0).unwrap();
                }
            }),
            state_socket
        }
    }
    /// Send new game state to client
    pub fn publish(&self, gs: GameState) {
        let json = serde_json::to_vec(&gs).unwrap();
        self.state_socket.send(json, 0).unwrap();
    }
}