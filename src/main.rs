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
/// Sorry, this part of the code is ugly. If you want me to clean it and document it, ask me -- Arkitu
#[tokio::main]
async fn main() {
    use std::{sync::Arc, time::Duration};
    use dashmap::DashMap;
    use futures_util::{SinkExt, StreamExt};
    use http::default::ServerMsg;
    use serde_json::Value;
    use tokio::{net::TcpListener, sync::mpsc, time::sleep};
    use tokio_tungstenite::tungstenite::Message;
    use tracing::{error, info, warn};
    use crate::zeromq::{prelude::*, util::PeerIdentity};
    use crate::http::{default::ClientMsg, WS_PORT};
    use crate::game_state::GameState;
    use crate::control::CtrlRes;

    // Host the page and wasm file
    tokio::spawn(wasm_server_runner::main(
        "./target/wasm32-unknown-unknown/debug/rsk-simulation.wasm".to_string(),
    ));

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
        let json = serde_json::to_string(&GameState::default()).unwrap();
        loop {
            state.send(("".to_string(), json.as_bytes().to_vec())).unwrap();
            sleep(Duration::from_millis(500)).await;
        }
    });

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