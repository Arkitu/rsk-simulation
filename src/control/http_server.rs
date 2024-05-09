use std::{sync::mpsc, thread};

use tracing::{error, info};
use websocket::{Message, OwnedMessage};
use zmq::Context;

use crate::http::{default::ClientMsg, WS_PORT};
use crate::wasm_server_runner;

pub struct Control;
impl Control {
    pub fn run(keys: [String; 2]) {
        // Host the page and wasm file
        thread::spawn(|| {
            wasm_server_runner::main(
                "./target/wasm32-unknown-unknown/debug/rsk-simulation.wasm".to_string(),
            )
            .unwrap();
        });

        let ctx = Context::new();

        let mut server = websocket::server::sync::Server::bind(format!("127.0.0.1:{}", WS_PORT)).unwrap();
        while let Ok(stream) = server.accept() {
            info!(target: "server_ws", "Incoming connection");
            let stream = stream.accept().unwrap();
            let (mut listener, mut sender) = stream.split().unwrap();

            let state_socket = ctx.socket(zmq::PUB).unwrap();

            let ctrl_socket = ctx.socket(zmq::REP).unwrap();

            let (res_sender, res_receiver) = mpsc::channel();
            // game state thread
            // forwards game state from the websocket to the control socket
            thread::spawn(move || {
                state_socket.bind("tcp://*:7557").unwrap();

                for msg in listener.incoming_messages() {
                    match msg {
                        Ok(OwnedMessage::Binary(bits)) => {
                            match bitcode::deserialize(&bits) {
                                Ok(ClientMsg::GameState(gs)) => {
                                    let json = serde_json::to_string(&gs).unwrap();
                                    if let Err(e) = state_socket.send(&json, 0) {
                                        error!(target: "ws_server", "Error when sending msg : {}", e);
                                        break
                                    }
                                },
                                Ok(ClientMsg::CtrlRes(res)) => {
                                    if let Err(e) = res_sender.send(res) {
                                        error!(target: "ws_server", "Error when sending msg : {}", e);
                                        break
                                    }
                                }
                                Err(e) => {
                                    error!(target: "ws_server", "Error when deserializing msg : {}", e);
                                    break
                                }
                            }
                        },
                        Err(e) => {
                            error!(target: "server_ws", "{:?}", e);
                            break
                        },
                        _ => unimplemented!()
                    }
                }

                state_socket.unbind("tcp://*:7557").unwrap();
            });
            // control thread
            // forwards controls to the websocket
            thread::spawn(move || {
                ctrl_socket.bind("tcp://*:7558").unwrap();

                loop {
                    let req = ctrl_socket.recv_bytes(0).unwrap();
                    if let Err(e) = sender.send_message(&Message::binary(req)) {
                        error!(target: "ws_server", "Error when sending msg : {}", e);
                        break
                    }
                    let res = res_receiver.recv().unwrap();
                    if let Err(e) = ctrl_socket.send(res, 0) {
                        error!(target: "ws_server", "Error when sending msg : {}", e);
                        break
                    }
                }
            });
        }
    }
}