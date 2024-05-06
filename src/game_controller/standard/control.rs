use std::{sync::{Arc, Mutex}, thread, time::Instant};
use serde::{ser::SerializeTuple, Deserialize, Serialize};
use serde_json::Value;
use zmq::{Context, Socket};
use crate::game_state::{GameState, Robot};

use super::RobotTask;

enum CtrlReq {
    Control(Robot, Order)
}

#[derive(Debug)]
enum CtrlRes {
    UnknownError,
    // (team)
    BadKey(String),
    Preempted(String, u8, String),
    UnknownRobot(String, u8),
    UnknownCommand,
    Ok
}
impl Serialize for CtrlRes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        match self {
            &CtrlRes::UnknownError => {
                // [False, "Unknown error"]
                let mut tup = serializer.serialize_tuple(2)?;
                tup.serialize_element(&false)?;
                tup.serialize_element("Unknown error")?;
                tup.end()
            },
            &CtrlRes::BadKey(ref team) => {
                // [False, "Bad key for team {team}"]
                let mut tup = serializer.serialize_tuple(2)?;
                tup.serialize_element(&false)?;
                tup.serialize_element(&format!("Bad key for team {}", team))?;
                tup.end()
            },
            &CtrlRes::Preempted(ref team, robot_number, ref reason) => {
                // [2, "Robot {number} of team {team} is preempted: {reasons}"]
                let mut tup = serializer.serialize_tuple(2)?;
                tup.serialize_element(&2)?;
                tup.serialize_element(&format!("Robot {} of team {} is preempted: {}", robot_number, team, reason))?;
                tup.end()
            },
            &CtrlRes::UnknownRobot(ref team, robot_number) => {
                // [False, "Unknown robot: {marker}"]
                let mut tup = serializer.serialize_tuple(2)?;
                tup.serialize_element(&false)?;
                tup.serialize_element(&format!("Unknown robot: {}{}", team, robot_number))?;
                tup.end()
            },
            
            &CtrlRes::UnknownCommand => {
                // [2, "Unknown command"]
                let mut tup = serializer.serialize_tuple(2)?;
                tup.serialize_element(&2)?;
                tup.serialize_element("Unknown command")?;
                tup.end()
            },
            &CtrlRes::Ok => {
                // [True, "ok"]
                let mut tup = serializer.serialize_tuple(2)?;
                tup.serialize_element(&true)?;
                tup.serialize_element("ok")?;
                tup.end()
            }
        }
    }
}

#[derive(Default)]
/// Order (like `robot.control((x, y, r))` in python api)
struct Order {
    pub x: f32,
    pub y: f32,
    /// rotation
    pub r: f32
}

pub struct Control {
    ctrl_thread: thread::JoinHandle<()>,
    state_socket: Socket,
    /// [blue1, blue2, green1, green2]
    orders: Arc<Mutex<[Order; 4]>>
}
impl Control {
    pub fn new(keys: [String; 2], tasks: Arc<Mutex<[Option<RobotTask>; 4]>>) -> Self {
        let ctx = Context::new();
        
        let state_socket = ctx.socket(zmq::PUB).unwrap();
        state_socket.bind("tcp://*:7557").unwrap();

        let ctrl_socket = ctx.socket(zmq::REP).unwrap();
        ctrl_socket.bind("tcp://*:7558").unwrap();

        let orders = Arc::new(Mutex::new(std::array::from_fn(|_| Order::default())));
        let orders_ref = orders.clone();
        Self {
            ctrl_thread: thread::spawn(move || {
                loop {
                    let req = ctrl_socket.recv_bytes(0).unwrap();
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
                                    let mut tasks = tasks.lock().unwrap();
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
                                                            r: cmd[3].as_f64().unwrap_or(0.) as f32
                                                        });
                                                        res = CtrlRes::Ok;
                                                    },
                                                    _ => res = CtrlRes::UnknownCommand
                                                },
                                                _ => res = CtrlRes::UnknownCommand
                                            },
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
                    ctrl_socket.send(serde_json::to_vec(&res).unwrap(), 0).unwrap();
                }
            }),
            state_socket,
            orders: orders_ref
        }
    }
    /// Send new game state to client
    pub fn publish(&self, gs: &GameState) {
        let json = serde_json::to_vec(gs).unwrap();
        self.state_socket.send(json, 0).unwrap();
    }
}