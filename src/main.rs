#![feature(duration_millis_float)]

mod constants;
mod game_state;
mod native;

#[cfg(feature = "referee")]
mod referee;

#[cfg(feature = "simulation")]
mod simulation;

#[cfg(any(feature = "alternative_http_client", feature = "alternative_http_server", feature = "http_client", feature = "http_server"))]
mod http;

#[cfg(feature = "control")]
mod control;

#[cfg(feature = "native_tui")]
mod terminal;

#[cfg(feature = "wasm_server_runner")]
mod wasm_server_runner;

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub type Control = native::control::Control;
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub type GC = native::gc::GC;
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
fn main() {
    tracing_subscriber::fmt::fmt()
        .without_time()
        .init();
    let gc = native::gc::GC::new("".to_string(), "".to_string(), "".to_string(), "".to_string(), false);

    native::gui::BevyGUI::run(gc);
}

#[cfg(all(feature = "native_tui", not(target_arch = "wasm32")))]
pub type Control = native::control::Control;
#[cfg(all(feature = "native_tui", not(target_arch = "wasm32")))]
pub type GC = native::gc::GC;
#[cfg(all(feature = "native_tui", not(target_arch = "wasm32")))]
fn main() {
    tracing_subscriber::fmt::fmt()
        .without_time()
        .init();
    let gc = native::gc::GC::new("".to_string(), "".to_string(), "".to_string(), "".to_string(), false);

    terminal::gui::TUI::run(gc);
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

#[cfg(all(feature = "http_server", not(target_arch = "wasm32")))]
#[tokio::main]
async fn main() {
    http::default::server::main().await
}

#[cfg(all(feature = "http_client", target_arch = "wasm32"))]
pub type Control = http::default::client::Control;
#[cfg(all(feature = "http_client", target_arch = "wasm32"))]
pub type GC = native::gc::GC;
#[cfg(all(feature = "http_client", target_arch = "wasm32"))]
fn main() {
    http::default::client::main()
}

#[cfg(all(feature = "alternative_http_client", target_arch = "wasm32"))]
pub type GC = http::alternative::client::GC;
#[cfg(all(feature = "alternative_http_client", target_arch = "wasm32"))]
fn main() {
    http::alternative::client::main()
}

#[cfg(all(feature = "alternative_http_server", not(target_arch = "wasm32")))]
#[tokio::main]
async fn main() {
    http::alternative::server::main().await
}