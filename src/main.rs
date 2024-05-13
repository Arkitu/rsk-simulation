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
    use std::{net::SocketAddr, sync::Arc};

    use futures_util::{future::{select, Either}, SinkExt, StreamExt};
    use tmq::{Context, Multipart};
    use tokio::{join, net::TcpListener, sync::{mpsc, Mutex}};
    use tokio_tungstenite::tungstenite::Message;
    use tracing::{info, error};
    use crate::http::{WS_PORT, default::ClientMsg};

    struct Session {
        id: String,
        ws: SocketAddr,
        // [ctrl, state]
        clients: Vec<[SocketAddr; 2]>,
        ctrl_port: u16,
        state_port: u16
    }

    struct Sessions {
        inner: Vec<Session>,
        available_ports: Vec<u16>
    }
    impl Sessions {
        // Return error if there are not enough ports available
        fn insert(&mut self, id: String, ws: SocketAddr) -> Result<(u16, u16), ()> {
            let ports = (
                self.available_ports.pop().ok_or(())?,
                self.available_ports.pop().ok_or(())?
            );
            self.inner.push(Session {
                id,
                ws,
                clients: Vec::new(),
                ctrl_port: ports.0,
                state_port: ports.1
            });
            Ok(ports)
        }
        fn find_with_addr(&self, addr: &SocketAddr) -> Option<&Session> {
            for s in self.inner.iter() {
                if &s.ws == addr {
                    return Some(s);
                }
                for c in s.clients.iter() {
                    for a in c.iter() {
                        if a == addr {
                            return Some(s);
                        }
                    }
                }
            }
            None
        }
    }

    // Host the page and wasm file
    tokio::spawn(wasm_server_runner::main(
        "./target/wasm32-unknown-unknown/debug/rsk-simulation.wasm".to_string(),
    ));

    let sessions: Arc<Mutex<Sessions>> = Arc::new(Mutex::new(Sessions {
        inner: Vec::new(),
        available_ports: (10200..10500).collect() // Arbitrary
    }));
    let s = sessions.clone();

    // Redirect tcp messages to the good session
    tokio::spawn(async move {
        let (ctrl, state) = join!(
            TcpListener::bind("127.0.0.1:7557"),
            TcpListener::bind("127.0.0.1:7558")
        );
        let ctrl = ctrl.unwrap();
        let state = state.unwrap();

        // The socket waiting to be paired
        let orphan = Arc::new(Mutex::new(None));
        // The pairs that are not yet matched with a simulation
        let pairs = Arc::new(Mutex::new(Vec::new()));

        // ctrl thread
        let o = orphan.clone();
        let p = pairs.clone();
        tokio::spawn(async move {
            while let Ok((stream, addr)) = ctrl.accept().await {
                dbg!("ctrl", addr);
                let mut orphan = o.lock().await;
                match orphan.take() {
                    None => {
                        *orphan = Some(addr);
                    },
                    Some(addr2) => {
                        p.lock().await.push((addr, addr2));
                    }
                }
            }
        });

        // state thread
        let o = orphan;
        let p = pairs;
        tokio::spawn(async move {
            while let Ok((stream, addr)) = state.accept().await {
                dbg!("state", addr);
                let mut orphan = o.lock().await;
                match orphan.take() {
                    None => {
                        *orphan = Some(addr);
                    },
                    Some(addr2) => {
                        p.lock().await.push((addr, addr2));
                    }
                }
            }
        });

        s;
    });

    let ctx = Context::new();
    let ws = TcpListener::bind(format!("127.0.0.1:{}", WS_PORT)).await.expect("Can't create TcpListener");
    while let Ok((stream, addr)) = ws.accept().await {
        let sessions: Arc<Mutex<Sessions>> = sessions.clone();
        let ctrl_socket = tmq::reply(&ctx);
        let state_socket = tmq::publish(&ctx);
        tokio::spawn(async move {
            info!(target:"ws", "New incoming connection : {}", addr);
            let stream = tokio_tungstenite::accept_async(stream).await.unwrap();
            let (mut ws_write, mut ws_read) = stream.split();
            
            let (ctrl_socket, mut state_socket) = if let Message::Binary(bits) = ws_read.next().await.unwrap().unwrap() {
                if let ClientMsg::InitialMsg(id) = bitcode::deserialize(&bits).unwrap() {
                    let ports = match sessions.lock().await.insert(id, addr) {
                        Ok(p) => p,
                        Err(_) => {
                            error!("Not enough available ports");
                            return
                        }
                    };
                    (
                        ctrl_socket.bind(&format!("tcp://*:{}", ports.0)).unwrap(),
                        state_socket.bind(&format!("tcp://*:{}", ports.1)).unwrap()
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
                                    if let Err(e) = state_socket.send(vec![&json]).await {
                                        error!(target: "zmq", "Error when sending msg : {}", e);
                                        return
                                        // return Err(TError::ConnectionClosed)
                                    }
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