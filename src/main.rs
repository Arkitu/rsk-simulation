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

#[cfg(feature = "http_server_control")]
#[tokio::main]
async fn main() {
    use control::Control;

    Control::run(["".to_string(), "".to_string()]).await;
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