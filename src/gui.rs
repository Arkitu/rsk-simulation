use log::info;

use crate::{game_controller::GC, game_state::GameState, simulation::Simulation};

#[cfg(feature = "bevy_gui")]
mod bevy;

/// Trait for GUI implementations
pub trait GUITrait {
    fn run(gc: GC);
}

#[cfg(feature = "bevy_gui")]
pub use bevy::BevyGUI as GUI;

#[cfg(not(any(feature = "bevy_gui")))]
pub struct GUI;
#[cfg(not(any(feature = "bevy_gui")))]
impl GUITrait for GUI {
    fn run(gc: GameController) {
        info!("No GUI feature enabled, skipping");
    }
}
