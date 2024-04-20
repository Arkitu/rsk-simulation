use game_controller::{GCTrait, GC};

mod constants;
mod game_controller;
mod game_state;
mod simulation;

#[cfg(feature = "http")]
mod http;

#[cfg(feature = "gui")]
mod gui;

fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(log::Level::Debug).expect("error initializing log");
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    }

    #[cfg(feature = "gui")]
    {
        let gc = GC::new("".to_string(), "".to_string(), true);
        use gui::GUITrait;
        gui::GUI::run(gc);
    }
    #[cfg(not(any(feature = "gui")))]
    {
        let mut gc = GC::new("", "", true);
        loop {
            gc.step();
        }
    }
}
