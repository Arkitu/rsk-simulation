use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use nalgebra::{vector, Point2};
use tracing::{info, warn};
use websocket::{Message, OwnedMessage};

use crate::constants::*;
use crate::game_controller::GC;
use crate::game_state::Robot;
use crate::gui::GUITrait;
use crate::http::alternative::{ClientMsg, InitialMsg, ServerMsg, WS_PORT};
use crate::wasm_server_runner;

pub struct GUI;
impl GUITrait for GUI {
    fn run(mut gc: GC) {
        let threads = [
            // Host the page and wasm file
            thread::spawn(|| {
                wasm_server_runner::main(
                    "./target/wasm32-unknown-unknown/debug/rsk-simulation.wasm".to_string(),
                )
                .unwrap();
            }),
            // Send game state to client via websocket (one client only)
            thread::spawn(move || {
                let mut server = websocket::server::sync::Server::bind(format!("127.0.0.1:{}", WS_PORT)).unwrap();
                let gc_mutex = Arc::new(Mutex::new(gc));
                while let Ok(mut stream) = server.accept() {
                    info!(target: "server_ws", "Incoming connection");
                    let mut stream = stream.accept().unwrap();
                    let gc = gc_mutex.lock().unwrap();
                    let initial_msg = InitialMsg {
                        ball: gc.get_ball_handle(),
                        blue1: gc.get_robot_handle(Robot::Blue1),
                        blue2: gc.get_robot_handle(Robot::Blue2),
                        green1: gc.get_robot_handle(Robot::Green1),
                        green2: gc.get_robot_handle(Robot::Green2)
                    };
                    drop(gc);
                    let initial_msg_bits = bitcode::serialize(&ServerMsg::Initial(initial_msg)).unwrap();
                    if let Err(e) = stream.send_message(&Message::binary(initial_msg_bits)) {
                        warn!(target: "server_ws", "{}", e);
                    }
                    let (mut listener, sender) = stream.split().unwrap();
                    let sender = Arc::new(Mutex::new(sender));
                    // Listener thread
                    let gc_mutex_ref = gc_mutex.clone();
                    let sender_ref = sender.clone();
                    thread::spawn(move || {
                        for msg in listener.incoming_messages() {
                            match msg {
                                Ok(msg) => match msg {
                                    OwnedMessage::Binary(bits) => {
                                        let msg: ClientMsg = match bitcode::deserialize(&bits) {
                                            Ok(msg) => msg,
                                            Err(e) => {
                                                warn!(target: "ws_server", "Error when deserializing msg : {}", e);
                                                break
                                            }
                                        };
                                        match msg {
                                            ClientMsg::FindEntityAt(pos) => {
                                                let entity = gc_mutex_ref.lock().unwrap().find_entity_at(pos);
                                                let res = ServerMsg::FindEntityAtRes(entity);
                                                let res_bits = bitcode::serialize(&res).unwrap();
                                                let mut s = sender_ref.lock().unwrap();
                                                s.send_message(&Message::binary(res_bits)).unwrap();
                                                drop(s);
                                            },
                                            ClientMsg::TeleportEntity(entity, pos, r) => {
                                                gc_mutex_ref.lock().unwrap().teleport_entity(entity, pos, r);
                                            },
                                            ClientMsg::Reset => {
                                                gc_mutex_ref.lock().unwrap().reset();
                                            }
                                        }
                                    },
                                    OwnedMessage::Close(_) => {
                                        break
                                    }
                                    _ => unimplemented!()
                                },
                                Err(e) => {
                                    warn!(target: "server_ws", "{:?}", e);
                                    break
                                }
                            }
                        }
                    });
                    let start = Instant::now();
                    let mut last_step = Instant::now();
                    loop {
                        let mut gc = gc_mutex.lock().unwrap();
                        // Just for debug
                        if start.elapsed() > Duration::from_millis(5000) {
                            let ball = gc.get_ball_handle();
                            gc.simu.bodies[ball].set_linvel(vector![0.3, 0.05], true);
                        }
                        // End of debug
                        while last_step.elapsed() > Duration::from_millis(FRAME_DURATION as u64)/2 {
                            gc.step();
                            last_step += Duration::from_millis(FRAME_DURATION as u64);
                        }
                        let gs = gc.get_game_state();
                        drop(gc);
                        let gs_bits = bitcode::serialize(&ServerMsg::GameState(gs)).unwrap();
                        // Send game state to client but break if client disconnects
                        let mut s = sender.lock().unwrap();
                        if let Err(e) = s.send_message(&Message::binary(gs_bits)) {
                            warn!(target: "server_ws", "{}", e);
                            break;
                        }
                        drop(s);
                    }
                    info!(target: "server_ws", "End of connection");
                }
            })
        ];
        for t in threads {
            t.join().unwrap();
        }
    }
}
