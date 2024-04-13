use crate::game_state::GameState;

#[cfg(feature = "bevy_gui")]
mod bevy;

pub trait GUITrait {
    fn new() -> Self;
    fn update(&mut self, game_state: &GameState);
}

pub struct GUI {
    #[cfg(feature = "bevy_gui")]
    bevy: bevy::BevyGUI
}
impl GUITrait for GUI {
    fn new() -> Self {
        Self {
            #[cfg(feature = "bevy_gui")]
            bevy: bevy::BevyGUI::new()
        }
    }
    fn update(&mut self, game_state: &GameState) {
        #[cfg(feature = "bevy_gui")]
        self.bevy.update(game_state);
    }
}