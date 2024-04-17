use game_controller::GameController;


mod constants;
mod simulation;
mod game_state;
mod game_controller;

#[cfg(feature = "gui")]
mod gui;

fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(log::Level::Debug).expect("error initializing log");
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    }
    
    let gc = GameController::new(
        "",
        "",
        true
    );

    #[cfg(feature = "gui")]
    {
        use gui::GUITrait;
        gui::GUI::run(gc);
    }
    
}