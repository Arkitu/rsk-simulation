use std::collections::VecDeque;

use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info, warn};
use tmq::{Context, Multipart};
use futures_util::{SinkExt, StreamExt};

use crate::http::{default::ClientMsg, WS_PORT};
use crate::wasm_server_runner;

pub struct Control;
impl Control {
    pub async fn run(keys: [String; 2]) {
        // Host the page and wasm file
        tokio::spawn(wasm_server_runner::main(
            "./target/wasm32-unknown-unknown/debug/rsk-simulation.wasm".to_string(),
        ));

        let ctx = Context::new();

        let ws = TcpListener::bind(format!("127.0.0.1:{}", WS_PORT)).await.expect("Can't create TcpListener");

        while let Ok((stream, _)) = ws.accept().await {
            let mut state_socket = tmq::publish(&ctx).bind("tcp://*:7557").unwrap();
            let ctrl_socket = tmq::reply(&ctx).bind("tcp://*:7558").unwrap();
            tokio::spawn(async move {
                let addr = stream.peer_addr().unwrap();
                info!(target:"ws", "New incoming connection : {}", addr);
                let stream = tokio_tungstenite::accept_async(stream).await.unwrap();
                let (mut ws_write, mut ws_read) = stream.split();

                let (res_sender, mut res_receiver) = mpsc::unbounded_channel();

                tokio::spawn(async move {
                    loop {
                        match ws_read.next().await.unwrap().unwrap() {
                            Message::Binary(bits) => {
                                match bitcode::deserialize(&bits) {
                                    Ok(ClientMsg::GameState(gs)) => {
                                        let json = serde_json::to_string(&gs).unwrap();
                                        if let Err(e) = state_socket.send(vec![&json]).await {
                                            error!(target: "zmq", "Error when sending msg : {}", e);
                                            break
                                            // return Err(TError::ConnectionClosed)
                                        }
                                    },
                                    Ok(ClientMsg::CtrlRes(res)) => {
                                        res_sender.send(res).unwrap();
                                    }
                                    Err(e) => {
                                        error!("Error when deserializing msg : {}", e);
                                        break
                                        // return Err(TError::ConnectionClosed)
                                    }
                                }
                            },
                            Message::Close(_) => {
                                info!(target: "ws", "socket closed");
                                break
                                // return Err(TError::ConnectionClosed)
                            },
                            _ => unimplemented!()
                        }
                    }
                });

                // Using TError::ConnectionClosed even if it's not the good error
                // tokio::spawn(ws_read.try_for_each(|msg| {
                //     async {
                //         match msg {
                //             Message::Binary(bits) => {
                //                 match bitcode::deserialize(&bits) {
                //                     Ok(ClientMsg::GameState(gs)) => {
                //                         let json = serde_json::to_string(&gs).unwrap();
                //                         if let Err(e) = state_socket.send(&json, 0) {
                //                             error!("Error when sending msg : {}", e);
                //                             return Err(TError::ConnectionClosed)
                //                         }
                //                     },
                //                     Ok(ClientMsg::CtrlRes(res)) => {
                //                         res_sender.send(res).unwrap();
                //                     }
                //                     Err(e) => {
                //                         error!("Error when deserializing msg : {}", e);
                //                         return Err(TError::ConnectionClosed)
                //                     }
                //                 }
                //             },
                //             Message::Close(_) => {
                //                 warn!("socket closed");
                //                 return Err(TError::ConnectionClosed)
                //             },
                //             _ => unimplemented!()
                //         }
                //         Ok(())
                //     }
                // }));

                // control thread
                // forwards controls to the websocket
                tokio::spawn(async move {
                    let mut receiver = Some(ctrl_socket);
                    loop {
                        let (mut multipart, sender) = receiver.take().unwrap().recv().await.unwrap();
                        let msg = match multipart.len() {
                            1 => multipart.pop_front().unwrap(),
                            x => {
                                error!(target: "zmq", "Received message with {} parts!", x);
                                break
                            }
                        };
                        if let Err(e) = ws_write.send(Message::Binary(msg.to_vec())).await {
                            error!(target: "ws", "Error when sending msg : {}", e);
                            break
                        };
                        let res = res_receiver.recv().await.unwrap();
                        match sender.send(Multipart(vec![res.into()].into())).await {
                            Ok(r) => {
                                receiver = Some(r);
                            },
                            Err(e) => {
                                error!(target: "ctrl_socket", "Error when sending msg : {}", e);
                                break
                            }
                        }
                    }
                });
            });
        }

        // server.incoming().for_each(|(upgrade, addr)| {
        //     tokio::spawn(async move {
        //         info!(target: "server_ws", "Incoming connection");

        //     let stream = upgrade.accept().and_then(|(s, x)| {
        //         s.into()
        //     });
        //     let (mut listener, mut sender) = stream.split().unwrap();

        //     let state_socket = ctx.socket(zmq::PUB).unwrap();

        //     let ctrl_socket = ctx.socket(zmq::REP).unwrap();

        //     let (res_sender, res_receiver) = mpsc::channel();
        //     // game state thread
        //     // forwards game state from the websocket to the control socket
        //     thread::spawn(move || {
        //         state_socket.bind("tcp://*:7557").unwrap();

        //         for msg in listener.incoming_messages() {
        //             match msg {
        //                 Ok(OwnedMessage::Binary(bits)) => {
        //                     match bitcode::deserialize(&bits) {
        //                         Ok(ClientMsg::GameState(gs)) => {
        //                             let json = serde_json::to_string(&gs).unwrap();
        //                             if let Err(e) = state_socket.send(&json, 0) {
        //                                 error!(target: "ws_server", "Error when sending msg : {}", e);
        //                                 break
        //                             }
        //                         },
        //                         Ok(ClientMsg::CtrlRes(res)) => {
        //                             if let Err(e) = res_sender.send(res) {
        //                                 error!(target: "ws_server", "Error when sending msg : {}", e);
        //                                 break
        //                             }
        //                         }
        //                         Err(e) => {
        //                             error!(target: "ws_server", "Error when deserializing msg : {}", e);
        //                             break
        //                         }
        //                     }
        //                 },
        //                 Err(e) => {
        //                     error!(target: "server_ws", "{:?}", e);
        //                     break
        //                 },
        //                 Ok(OwnedMessage::Close(_)) => {
        //                     break
        //                 }
        //                 _ => unimplemented!()
        //             }
        //         }

        //         if let Err(e) = state_socket.unbind("tcp://*:7557") {
        //             error!("{}", e);
        //         }
        //     });
        //     // control thread
        //     // forwards controls to the websocket
        //     thread::spawn(move || {
        //         ctrl_socket.bind("tcp://*:7558").unwrap();

        //         loop {
        //             let req = ctrl_socket.recv_bytes(0).unwrap();
        //             if let Err(e) = sender.send_message(&Message::binary(req)) {
        //                 error!(target: "ws_server", "Error when sending msg : {}", e);
        //                 break
        //             }
        //             let res = res_receiver.recv().unwrap();
        //             if let Err(e) = ctrl_socket.send(res, 0) {
        //                 error!(target: "ws_server", "Error when sending msg : {}", e);
        //                 break
        //             }
        //         }

        //         if let Err(e) = ctrl_socket.unbind("tcp://*:7558") {
        //             error!("{}", e);
        //         }
        //     });
        //     });
            

        //     Ok(())
        // });
    }
}