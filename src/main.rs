mod constants;
mod game_state;
mod simulation;

#[cfg(feature = "gc")]
mod game_controller;

#[cfg(any(feature = "alternative_http", feature = "default_http"))]
mod http;

#[cfg(feature = "gui")]
mod gui;

#[cfg(feature = "control")]
mod control;

#[cfg(feature = "wasm_server_runner")]
mod wasm_server_runner;

#[cfg(feature = "zeromq")]
mod zeromq;

#[cfg(all(feature = "standard_gc", not(feature = "http_client_control")))]
fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(log::Level::Debug).expect("error initializing log");
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    }

    let mut gc = game_controller::GC::new("".to_string(), "".to_string(), "".to_string(), "".to_string(), false);

    #[cfg(feature = "gui")]
    {
        use gui::GUITrait;
        gui::GUI::run(gc);
    }
    #[cfg(not(any(feature = "gui")))]
    {
        loop {
            gc.step();
        }
    }
}

#[cfg(all(feature = "http_client_gc", target_arch = "wasm32"))]
fn main() {
    use tracing::{debug, info};

    wasm_bindgen_futures::spawn_local(async {
        //console_log::init_with_level().expect("error initializing log");
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        tracing_wasm::set_as_global_default();

        #[cfg(feature = "gui")]
        {
            let gc = game_controller::GC::new("".to_string(), "".to_string(), true).await;
            use gui::GUITrait;
            gui::GUI::run(gc);
        }
        #[cfg(not(any(feature = "gui")))]
        {
            let mut gc = game_controller::GC::new("", "", true);
            loop {
                gc.step();
            }
        }
    })
}

#[cfg(feature = "http_server")]
#[tokio::main]
async fn main() {
    use std::{collections::HashMap, net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4}, sync::Arc, time::Duration};
    use dashmap::DashMap;
    use futures_util::{future::{select, Either}, SinkExt, StreamExt};
    use http::default::ServerMsg;
    use serde_json::Value;
    use tokio::{io::{AsyncReadExt, AsyncWriteExt}, join, net::{TcpListener, TcpSocket, TcpStream}, sync::{mpsc, oneshot, watch, Mutex}, time::{sleep, Instant}};
    use tokio_tungstenite::tungstenite::Message;
    use tracing::{error, info, warn};
    use crate::zeromq::{prelude::*, util::PeerIdentity};
    use crate::http::{default::ClientMsg, WS_PORT};
    use crate::game_state::GameState;
    use crate::control::CtrlRes;

    // #[derive(Clone)]
    // struct Session {
    //     id: String,
    //     ws: SocketAddr,
    //     // Receives game state json
    //     state_rcv: watch::Receiver<String>,
    //     ctrl_port: u16,
    // }

    // struct Sessions {
    //     inner: Vec<Session>,
    //     lobby: Session,
    //     available_ports: Vec<u16>
    // }
    // impl Sessions {
    //     // Return error if there are not enough ports available
    //     fn insert(&mut self, id: String, ws: SocketAddr) -> Result<(u16, watch::Sender<String>), ()> {
    //         let ctrl_port = self.available_ports.pop().ok_or(())?;
    //         let (tx, state_rcv) = watch::channel(serde_json::to_string(&GameState::default()).unwrap());
    //         self.inner.push(Session {
    //             id,
    //             ws,
    //             state_rcv,
    //             ctrl_port
    //         });
    //         Ok((ctrl_port, tx))
    //     }
    //     // fn find_with_addr(&self, addr: &SocketAddr) -> Option<&Session> {
    //     //     self.inner.iter().find(|s| {
    //     //         &s.ws == addr || s.ctrls.contains(addr) || s.states.contains(addr)
    //     //     })
    //     // }
    //     // fn find_with_ip(&self, ip: &IpAddr) -> Option<&Session> {
    //     //     self.inner.iter().find(|s| {
    //     //         &s.ws.ip() == ip || s.ctrls.iter().any(|a| &a.ip() == ip) || s.states.iter().any(|a| &a.ip() == ip)
    //     //     })
    //     // }
    //     fn find_with_id(&self, id: &str) -> Option<&Session> {
    //         self.inner.iter().find(|s| {
    //             &s.id == id
    //         })
    //     }
    //     /// Session with id="" that serves as a lobby for the sockets that are not yet matched with their session
    //     const fn lobby(&self) -> &Session {
    //         &self.lobby
    //     }
    // }

    // Host the page and wasm file
    tokio::spawn(wasm_server_runner::main(
        "./target/wasm32-unknown-unknown/debug/rsk-simulation.wasm".to_string(),
    ));

    // let sessions: Arc<Mutex<Sessions>> = Arc::new(Mutex::new(Sessions {
    //     inner: Vec::new(),
    //     lobby: Session {
    //         id: "".to_string(),
    //         ws: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
    //         state_rcv: rx,
    //         ctrl_port: 10200
    //     },
    //     available_ports: (10202..10500).collect() // Arbitrary
    // }));

    // session_id --> (sender, receiver)
    let ctrl_sessions = Arc::new(DashMap::<
            String, // session's id
            (
                mpsc::UnboundedSender<(String, u8, Vec<Value>)>, // (team, number, command)
                mpsc::UnboundedReceiver<Vec<u8>> // response in bytes
            )
        >::new());

    let state_socket = zeromq::PubSocket::new();

    // ctrl socket
    let ctrls = ctrl_sessions.clone();
    let orphan_sub = state_socket.backend.orphan_sub.clone();
    let pairs: Arc<DashMap<zeromq::util::PeerIdentity, zeromq::util::PeerIdentity>> = state_socket.backend.pairs.clone();
    let subscribers_session = state_socket.backend.subscribers_session.clone();
    tokio::spawn(async move {
        // Pairs represented by their ctrl peer id
        let mut matched_pairs: Vec<PeerIdentity> = Vec::new();
        let mut socket = zeromq::RepSocket::new();
        *socket.backend.orphan_sub.lock().await = orphan_sub;
        socket.bind("tcp://127.0.0.1:7558").await.unwrap();

        loop {
            let msg = socket.recv().await.unwrap();
            let req = match msg.get(0) {
                Some(req) => req,
                None => {
                    warn!("Received empty message");
                    continue
                }
            };
            let (key, team, number, cmd) : (String, String, u8, Vec<Value>) = match serde_json::from_slice(req) {
                Ok(t) => t,
                Err(e) => {
                    warn!("Error when deserializing req : {}", e);
                    continue
                }
            };
            
            let res = match ctrls.get_mut(&key) {
                Some(mut ctrl) => {
                    let ctrl_id = socket.current_request.clone().unwrap();
                    dbg!(&ctrl_id);
                    if !matched_pairs.contains(&ctrl_id) {
                        // Set the session for subscriber
                        subscribers_session.insert(pairs.get(&ctrl_id).unwrap().clone(), key);
                        matched_pairs.push(ctrl_id);
                    }
                    let (_, (sender, receiver)) = ctrl.pair_mut();
                    sender.send((team, number, cmd)).unwrap();
                    receiver.recv().await.unwrap()
                },
                None => serde_json::to_vec(&CtrlRes::BadKey("you must put your session's id in your key".to_string())).unwrap()
            };
            dbg!(String::from_utf8(res.clone()).unwrap());
            socket.send(res.into()).await.unwrap();
        }
    });

    let mut socket = state_socket;
    let (state_socket, mut rcv) = mpsc::unbounded_channel::<(String, Vec<u8>)>();
    tokio::spawn(async move {
        socket.bind("tcp://127.0.0.1:7557").await.unwrap();
        loop {
            let (id, msg) = rcv.recv().await.unwrap();
            socket.send_for_id(msg.into(), &id).await.unwrap();
        }
    });

    // Lobby
    let state = state_socket.clone();
    tokio::spawn(async move {
        let DEFAULT_GAME_STATE_JSON = serde_json::to_string(&GameState::default()).unwrap();
        loop {
            state.send(("".to_string(), DEFAULT_GAME_STATE_JSON.as_bytes().to_vec())).unwrap();
            sleep(Duration::from_millis(500)).await;
        }
    });

    // // Redirect tcp messages to the good session
    // let ctrl = TcpListener::bind("127.0.0.1:7557").await.unwrap();
    // let state = TcpListener::bind("127.0.0.1:7558").await.unwrap();

    // // The socket waiting to be paired
    // let state_orphan = Arc::new(Mutex::new(None::<oneshot::Sender<watch::Receiver<String>>>));

    // // Thread that manages incoming ctrls sockets
    // let ss = sessions.clone();
    // let so = state_orphan.clone();
    // tokio::spawn(async move {
    //     while let Ok((mut stream, addr)) = ctrl.accept().await {
    //         dbg!("ctrl", addr);
    //         let port = match ss.lock().await.available_ports.pop() {
    //             Some(p) => p,
    //             None => {
    //                 error!("Not enough available ports");
    //                 continue
    //             }
    //         };

    //         // Create the zmq socket that receives controls
    //         dbg!(port);
    //         let ctrl_socket = tmq::reply(&ctx).bind(&format!("tcp://*:{}", port)).unwrap();
    //         sleep(Duration::from_secs(1)).await;

    //         let intern_stream = TcpSocket::new_v4()
    //                 .unwrap()
    //                 .connect(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port)))
    //                 .await
    //                 .unwrap();

    //         let (mut incoming_read, mut incoming_write) = stream.into_split();
    //         let (mut intern_read, mut intern_write) = intern_stream.into_split();

    //         // Forward traffic from incoming to intern
    //         tokio::spawn(async move {
    //             loop {
    //                 let mut buf = [0u8; 64];
    //                 let n = incoming_read.read(&mut buf).await.unwrap();
    //                 dbg!(n);
    //                 intern_write.write_all(&buf[0..n]).await.unwrap();
    //                 if n == 0 {
    //                     return
    //                 }
    //             }
    //         });

    //         // Forward traffic from intern to incoming
    //         tokio::spawn(async move {
    //             loop {
    //                 let mut buf = [0u8; 64];
    //                 let n = intern_read.read(&mut buf).await.unwrap();
    //                 dbg!(n);
    //                 incoming_write.write_all(&buf[0..n]).await.unwrap();
    //                 if n == 0 {
    //                     return
    //                 }
    //             }
    //         });
            
    //         let (state_rcv_tx, mut state_rcv_rx) = oneshot::channel::<watch::Receiver<String>>();

    //         // Wait for the state socket
    //         let so = so.clone();
    //         tokio::spawn(async move {
    //             let start = Instant::now();
    //             while start.elapsed() < Duration::from_millis(3000) {
    //                 if let Some(tx) = so.lock().await.take() {
    //                     tx.send(state_rcv_rx.await.unwrap()).unwrap();
    //                     return
    //                 }
    //             }
    //             warn!("No state socket matching ctrl socket received within a 3 seconde delay");
    //             state_rcv_rx.close();
    //         });

    //         let sessions = ss.clone();
    //         let session_socket = tmq::request(&ctx);
    //         tokio::spawn(async move {
    //             // Wait for the first message to match the socket with the session which's id correspond to the key of the message
    //             let (msg, sender) = ctrl_socket.recv().await.unwrap();
    //             let (key, _, _, _) : (String, String, u8, Vec<Value>) = serde_json::from_slice(msg.iter().last().unwrap()).unwrap();
    //             let session = sessions.lock().await.find_with_id(&key).unwrap().clone();
    //             state_rcv_tx.send(session.state_rcv.clone()).unwrap();
    //             let session_socket = session_socket.connect(&format!("tcp://127.0.0.1:{}", session.ctrl_port)).unwrap();
                
    //             // Forward all ctrls to session's ctrl socket
    //             let mut msg = Some(msg);
    //             let mut session_sender = Some(session_socket);
    //             let mut client_sender = Some(sender);
    //             loop {
    //                 let session_receiver = session_sender.take().unwrap().send(msg.take().unwrap()).await.unwrap();
    //                 let (res, s_sender) = session_receiver.recv().await.unwrap();
    //                 session_sender = Some(s_sender);
    //                 let client_receiver = client_sender.take().unwrap().send(res).await.unwrap();
    //                 let (req, c_sender) = client_receiver.recv().await.unwrap();
    //                 msg = Some(req);
    //                 client_sender = Some(c_sender);
    //             }
    //         });
    //     }
    // });

    // // Thread that manages incoming state sockets
    // let ss = sessions.clone();
    // let so = state_orphan.clone();
    // let ctx = context.clone();
    // tokio::spawn(async move {
    //     while let Ok((mut stream, addr)) = state.accept().await {
    //         dbg!("state", addr);
    //         let port = match ss.lock().await.available_ports.pop() {
    //             Some(p) => p,
    //             None => {
    //                 error!("Not enough available ports");
    //                 continue;
    //             }
    //         };
    //         dbg!(port);

    //         let (state_rcv_tx, mut state_rcv_rx) = oneshot::channel::<watch::Receiver<String>>();
    //         *so.lock().await = Some(state_rcv_tx);

    //         let mut rcv = ss.lock().await.lobby().state_rcv.clone();
    //         let ctx = ctx.clone();
    //         // let mut state_socket = tmq::publish(&ctx).bind(&format!("tcp://*:{}", port)).unwrap();
    //         let mut state_socket = tmq::publish(&ctx).bind(&format!("tcp://*:7558")).unwrap();
    //         let json = rcv.borrow().clone();
    //         state_socket.send(vec![&json]).await.unwrap();
    //         sleep(Duration::from_secs(1)).await;
    //         tokio::spawn(async move {
    //             // Send lobby game state while waiting for the good session
    //             loop {
    //                 match state_rcv_rx.try_recv() {
    //                     Ok(r) => {
    //                         rcv = r;
    //                         break
    //                     },
    //                     Err(oneshot::error::TryRecvError::Empty) => {
    //                         let json = rcv.borrow().clone();
    //                         state_socket.send(vec![&json]).await.unwrap();
    //                     },
    //                     Err(e) => Err(e).unwrap()
    //                 }
    //             }
    //             // Send session's game state
    //             loop {
    //                 let json = rcv.borrow().clone();
    //                 state_socket.send(vec![&json]).await.unwrap();
    //             }
    //         });

    //         let intern_stream = TcpSocket::new_v4()
    //                 .unwrap()
    //                 .connect(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port)))
    //                 .await
    //                 .unwrap();

    //         let (mut incoming_read, mut incoming_write) = stream.into_split();
    //         let (mut intern_read, mut intern_write) = intern_stream.into_split();

    //         // Forward traffic from incoming to intern
    //         tokio::spawn(async move {
    //             loop {
    //                 let mut buf = [0u8; 64];
    //                 let n = incoming_read.read(&mut buf).await.unwrap();
    //                 dbg!(n);
    //                 intern_write.write_all(&buf[0..n]).await.unwrap();
    //                 if n == 0 {
    //                     return
    //                 }
    //             }
    //         });

    //         // Forward traffic from intern to incoming
    //         tokio::spawn(async move {
    //             loop {
    //                 let mut buf = [0u8; 128];
    //                 let n = intern_read.read(&mut buf).await.unwrap();
    //                 dbg!(n);
    //                 incoming_write.write_all(&buf[0..n]).await.unwrap();
    //                 // if n == 0 {
    //                 //     return
    //                 // }
    //             }
    //         });
    //     }
    // });

    let ws = TcpListener::bind(format!("127.0.0.1:{}", WS_PORT)).await.expect("Can't create TcpListener");
    while let Ok((stream, addr)) = ws.accept().await {
        // let ctrl_socket = tmq::reply(&context);
        let state_socket = state_socket.clone();
        let ctrl_sessions = ctrl_sessions.clone();
        tokio::spawn(async move {
            info!(target:"ws", "New incoming connection : {}", addr);
            let stream = tokio_tungstenite::accept_async(stream).await.unwrap();
            let (mut ws_write, mut ws_read) = stream.split();
            
            let session_id = if let Message::Binary(bits) = ws_read.next().await.unwrap().unwrap() {
                if let ClientMsg::InitialMsg(id) = bitcode::deserialize(&bits).unwrap() {
                    id
                } else {
                    error!(target: "ws", "First message is not InitialMsg");
                    return
                }
            } else {
                error!(target: "ws", "Expected bytes");
                return
            };

            let (snd, mut ctrl_receiver) = mpsc::unbounded_channel();
            let (ctrl_sender, rcv) = mpsc::unbounded_channel();

            ctrl_sessions.insert(session_id.clone(), (
                snd,
                rcv
            ));

            let (res_sender, mut res_receiver) = mpsc::unbounded_channel();

            let s_id = session_id.clone();
            tokio::spawn(async move {
                loop {
                    match ws_read.next().await.unwrap().unwrap() {
                        Message::Binary(bits) => {
                            match bitcode::deserialize(&bits).unwrap() {
                                ClientMsg::InitialMsg(_) => {
                                    error!(target: "ws", "Received an second InitialMsg");
                                    ctrl_sessions.remove(&s_id);
                                    return
                                }
                                ClientMsg::GameState(gs) => {
                                    let json = serde_json::to_vec(&gs).unwrap();
                                    state_socket.send((s_id.clone(), json)).unwrap();
                                },
                                ClientMsg::CtrlRes(res) => {
                                    res_sender.send(res).unwrap();
                                }
                            }
                        },
                        Message::Close(_) => {
                            info!(target: "ws", "socket closed");
                            ctrl_sessions.remove(&s_id);
                            return
                            // return Err(TError::ConnectionClosed)
                        },
                        _ => {
                            error!(target: "ws", "Expected bytes");
                            ctrl_sessions.remove(&s_id);
                            return
                        }
                    }
                }
            });

            // control thread
            // forwards controls to the websocket
            tokio::spawn(async move {
                loop {
                    let (team, number, cmd) = match ctrl_receiver.recv().await {
                        Some(t) => t,
                        None => {
                            info!("ctrl thread closed");
                            break
                        }
                    };
                    if let Err(e) = ws_write.send(Message::Text(serde_json::to_string(&ServerMsg::Ctrl(session_id.clone(), team, number, cmd)).unwrap())).await {
                        error!(target: "ws", "Error when sending msg : {}", e);
                        break
                    };
                    let res = res_receiver.recv().await.unwrap();
                    if let Err(_) = ctrl_sender.send(res) {
                        warn!("channel closed");
                        break
                    }
                }
            });
        });
    }
}

#[cfg(all(feature = "http_client_control", target_arch = "wasm32"))]
fn main() {
    use log::info;
    use url::Url;
    use rand::distributions::{Alphanumeric, DistString};

    console_log::init_with_level(log::Level::Debug).expect("error initializing log");
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    info!("test");
    let mut location = web_sys::window().unwrap().location();
    let mut url = Url::parse(&location.href().unwrap()).unwrap();
    if url.path().len() <= 1 {
        url.set_path(&("/".to_string() + &Alphanumeric.sample_string(&mut rand::thread_rng(), 5)));
        info!("{:?}", url);
        location.set_href(url.as_str()).unwrap();
        return
    }

    let mut session_id = url.path();
    if session_id.starts_with("/") {
        session_id = &session_id[1..];
    }

    let mut gc = game_controller::GC::new("".to_string(), "".to_string(), session_id.to_string(), session_id.to_string(), false, session_id);

    #[cfg(feature = "gui")]
    {
        use gui::GUITrait;
        gui::GUI::run(gc);
    }
    #[cfg(not(any(feature = "gui")))]
    {
        loop {
            gc.step();
        }
    }
}