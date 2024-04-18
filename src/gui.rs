use crate::game_controller::GC;

#[cfg(feature = "bevy_gui")]
mod bevy;

#[cfg(feature = "http_gui")]
mod http;

/// Trait for GUI implementations
pub trait GUITrait {
    fn run(gc: GC);
}

#[cfg(all(feature = "bevy_gui", feature = "http_gui"))]
compile_error!("Multiple GUI features enabled. You can only enable one GUI feature.");

#[cfg(feature = "bevy_gui")]
pub use bevy::BevyGUI as GUI;

#[cfg(feature = "http_gui")]
pub use http::HttpGUI as GUI;

#[cfg(not(any(feature = "bevy_gui", feature = "http_gui")))]
pub struct GUI;
#[cfg(not(any(feature = "bevy_gui", feature = "http_gui")))]
impl GUITrait for GUI {
    fn run(gc: GameController) {
        use log::info;
        info!("No GUI feature enabled, skipping");
    }
}
