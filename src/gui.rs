use crate::game_controller::GC;

#[cfg(feature = "bevy_gui")]
mod bevy;

#[cfg(feature = "http_server_gui")]
mod http_server;

/// Trait for GUI implementations
pub trait GUITrait {
    fn run(gc: GC);
}

#[cfg(all(feature = "bevy_gui", feature = "http_server_gui"))]
compile_error!("Multiple GUI features enabled. You can only enable one GUI feature.");

#[cfg(feature = "bevy_gui")]
pub use bevy::BevyGUI as GUI;

#[cfg(feature = "http_server_gui")]
pub use http_server::GUI;

#[cfg(not(any(feature = "bevy_gui", feature = "http_server_gui")))]
pub struct GUI;
#[cfg(not(any(feature = "bevy_gui", feature = "http_server_gui")))]
impl GUITrait for GUI {
    fn run(gc: GameController) {
        use log::info;
        info!("No GUI feature enabled, skipping");
    }
}
