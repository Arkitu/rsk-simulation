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
    use futures_util::{future::{select, Either}, SinkExt, StreamExt};
    use serde_json::Value;
    use tmq::{publish::Publish, request_reply::RequestReceiver, Context, Multipart};
    use tokio::{io::{AsyncReadExt, AsyncWriteExt}, join, net::{TcpListener, TcpSocket, TcpStream}, sync::{mpsc, oneshot, watch, Mutex}, time::Instant};
    use tokio_tungstenite::tungstenite::Message;
    use tracing::{error, info, warn};
    use crate::http::{WS_PORT, default::ClientMsg};
    use crate::game_state::GameState;

    let DEFAULT_GAME_STATE_JSON = serde_json::to_string(&GameState::default()).unwrap();

    #[derive(Clone)]
    struct Session {
        id: String,
        ws: SocketAddr,
        // Receives game state json
        state_rcv: watch::Receiver<String>,
        ctrl_port: u16,
    }

    struct Sessions {
        inner: Vec<Session>,
        lobby: Session,
        available_ports: Vec<u16>
    }
    impl Sessions {
        // Return error if there are not enough ports available
        fn insert(&mut self, id: String, ws: SocketAddr) -> Result<(u16, watch::Sender<String>), ()> {
            let ctrl_port = self.available_ports.pop().ok_or(())?;
            let (tx, state_rcv) = watch::channel(serde_json::to_string(&GameState::default()).unwrap());
            self.inner.push(Session {
                id,
                ws,
                state_rcv,
                ctrl_port
            });
            Ok((ctrl_port, tx))
        }
        // fn find_with_addr(&self, addr: &SocketAddr) -> Option<&Session> {
        //     self.inner.iter().find(|s| {
        //         &s.ws == addr || s.ctrls.contains(addr) || s.states.contains(addr)
        //     })
        // }
        // fn find_with_ip(&self, ip: &IpAddr) -> Option<&Session> {
        //     self.inner.iter().find(|s| {
        //         &s.ws.ip() == ip || s.ctrls.iter().any(|a| &a.ip() == ip) || s.states.iter().any(|a| &a.ip() == ip)
        //     })
        // }
        fn find_with_id(&self, id: &str) -> Option<&Session> {
            self.inner.iter().find(|s| {
                &s.id == id
            })
        }
        /// Session with id="" that serves as a lobby for the sockets that are not yet matched with their session
        const fn lobby(&self) -> &Session {
            &self.lobby
        }
    }

    // Host the page and wasm file
    tokio::spawn(wasm_server_runner::main(
        "./target/wasm32-unknown-unknown/debug/rsk-simulation.wasm".to_string(),
    ));

    let (tx, rx) = watch::channel(DEFAULT_GAME_STATE_JSON.clone());

    let dgsj = DEFAULT_GAME_STATE_JSON.clone();
    tokio::spawn(async move {
        loop {
            tx.send(dgsj.clone()).unwrap();
        }
    });

    let sessions: Arc<Mutex<Sessions>> = Arc::new(Mutex::new(Sessions {
        inner: Vec::new(),
        lobby: Session {
            id: "".to_string(),
            ws: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
            state_rcv: rx,
            ctrl_port: 10200
        },
        available_ports: (10202..10500).collect() // Arbitrary
    }));
    
    // Redirect tcp messages to the good session
    let ctrl = TcpListener::bind("127.0.0.1:7557").await.unwrap();
    let state = TcpListener::bind("127.0.0.1:7558").await.unwrap();

    // The socket waiting to be paired
    let state_orphan = Arc::new(Mutex::new(None::<(TcpStream, SocketAddr)>));
    // The pairs that are not yet matched with a simulation
    let available_ports = Arc::new(Mutex::new((10200..10500).collect::<Vec<u16>>()));
    let context = Context::new();

    // Thread that manages incoming ctrls sockets
    let ss = sessions.clone();
    let so = state_orphan.clone();
    let ctx = context.clone();
    tokio::spawn(async move {
        while let Ok((mut stream, addr)) = ctrl.accept().await {
            dbg!("ctrl", addr);
            let port = match available_ports.lock().await.pop() {
                Some(p) => p,
                None => {
                    error!("Not enough available ports");
                    continue
                }
            };
            // Redirect all incoming traffic in the port
            tokio::spawn(async move {
                let mut local_stream = TcpSocket::new_v4()
                    .unwrap()
                    .connect(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port)))
                    .await
                    .unwrap();
                loop {
                    let mut data = [0u8; 64];
                    stream.read(&mut data).await.unwrap();
                    local_stream.write(&data).await.unwrap();
                }
            });
            // Create the zmq socket that receives controls
            let socket = tmq::reply(&ctx).bind(&format!("tcp://*:{}", port)).unwrap();
            
            let state_socket = Arc::new(Mutex::new(None::<u16>));
            // Wait for the state socket
            let state_s = state_socket.clone();
            let so = so.clone();
            tokio::spawn(async move {
                let start = Instant::now();
                loop {
                    if start.elapsed() > Duration::from_millis(3000) {
                        // Poison state_socket to crash pair thread
                        let _lock = state_s.lock().await;
                        panic!("Timeout for state socket detection (this is a normal error)");
                    }
                    if let Some(socket) = so.lock().await.take() {
                        *state_s.lock().await = Some(socket);
                        break
                    }
                }
            });
            
            let (state_rcv_tx, state_rcv_rx) = oneshot::channel::<watch::Receiver<String>>();
            let mut rcv = ss.lock().await.lobby().state_rcv.clone();
            let state_socket = tmq::publish(&ctx);
            // State socket
            tokio::spawn(async move {
                
                loop {
                    match state_rcv_rx.try_recv() {
                        Ok(r) => {
                            rcv = r;
                            break
                        },
                        Err(oneshot::error::TryRecvError::Empty) => {

                        }
                    }
                }
            });

            let sessions = ss.clone();
            let session_socket = tmq::request(&ctx);
            tokio::spawn(async move {
                // Wait for the first message to match the socket with the session which's id correspond to the key of the message
                let (msg, sender) = socket.recv().await.unwrap();
                let (key, _, _, _) : (String, String, u8, Vec<Value>) = serde_json::from_slice(msg.iter().last().unwrap()).unwrap();
                let session = sessions.lock().await.find_with_id(&key).unwrap().clone();
                let session_socket = session_socket.connect(&format!("tcp://127.0.0.1:{}", session.ctrl_port)).unwrap();
                
                // Forward all ctrls to session's ctrl socket
                let mut msg = Some(msg);
                let mut session_sender = Some(session_socket);
                let mut client_sender = Some(sender);
                loop {
                    let session_receiver = session_sender.take().unwrap().send(msg.take().unwrap()).await.unwrap();
                    let (res, s_sender) = session_receiver.recv().await.unwrap();
                    session_sender = Some(s_sender);
                    let client_receiver = client_sender.take().unwrap().send(res).await.unwrap();
                    let (req, c_sender) = client_receiver.recv().await.unwrap();
                    msg = Some(req);
                    client_sender = Some(c_sender);
                }
            });
        }
    });

    let ss = sessions.clone();
    let ctx = context.clone();
    tokio::spawn(async move {
        while let Ok((mut stream, addr)) = state.accept().await {
            let port = match ss.lock().await.available_ports.pop() {
                Some(p) => p,
                None => {
                    error!("Not enough available ports");
                    continue;
                }
            };
        }
    });

    let ws = TcpListener::bind(format!("127.0.0.1:{}", WS_PORT)).await.expect("Can't create TcpListener");
    while let Ok((stream, addr)) = ws.accept().await {
        let ctrl_socket = tmq::reply(&context);
        let sessions = sessions.clone();
        tokio::spawn(async move {
            info!(target:"ws", "New incoming connection : {}", addr);
            let stream = tokio_tungstenite::accept_async(stream).await.unwrap();
            let (mut ws_write, mut ws_read) = stream.split();
            
            let (ctrl_socket, state_sender) = if let Message::Binary(bits) = ws_read.next().await.unwrap().unwrap() {
                if let ClientMsg::InitialMsg(id) = bitcode::deserialize(&bits).unwrap() {
                    let (ctrl_port, state_sender) = match sessions.lock().await.insert(id, addr) {
                        Ok(p) => p,
                        Err(_) => {
                            error!("Not enough available ports");
                            return
                        }
                    };
                    (
                        ctrl_socket.bind(&format!("tcp://*:{}", ctrl_port)).unwrap(),
                        state_sender
                    )
                } else {
                    error!(target: "ws", "First message is not InitialMsg");
                    return
                }
            } else {
                error!(target: "ws", "Expected bytes");
                return
            };

            let (res_sender, mut res_receiver) = mpsc::unbounded_channel();

            tokio::spawn(async move {
                loop {
                    match ws_read.next().await.unwrap().unwrap() {
                        Message::Binary(bits) => {
                            match bitcode::deserialize(&bits).unwrap() {
                                ClientMsg::InitialMsg(_) => {
                                    error!(target: "ws", "Received an second InitialMsg");
                                    return
                                }
                                ClientMsg::GameState(gs) => {
                                    let json = serde_json::to_string(&gs).unwrap();
                                    state_sender.send(json).unwrap()
                                },
                                ClientMsg::CtrlRes(res) => {
                                    res_sender.send(res).unwrap();
                                }
                            }
                        },
                        Message::Close(_) => {
                            info!(target: "ws", "socket closed");
                            return
                            // return Err(TError::ConnectionClosed)
                        },
                        _ => {
                            error!(target: "ws", "Expected bytes");
                            return
                        }
                    }
                }
            });

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

    let mut gc = game_controller::GC::new("".to_string(), "".to_string(), "".to_string(), "".to_string(), false, session_id);

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