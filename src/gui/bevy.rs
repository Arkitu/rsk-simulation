/// Bevy is only used to visualize the simulation
use bevy::prelude::*;
use crate::{game_state::GameState, gui::GUITrait};
use std::{sync::{Arc, Mutex}, thread};


#[derive(Resource)]
struct BevyGameState (Arc<Mutex<Option<GameState>>>);

pub struct BevyGUI {
    game_state: Arc<Mutex<Option<GameState>>>
}
impl GUITrait for BevyGUI {
    fn new() -> Self {
        let game_state = Arc::new(Mutex::new(None));
        let gs = game_state.clone();
        thread::spawn(move || {
            App::new()
            .add_plugins(DefaultPlugins)
            .insert_resource(BevyGameState(gs))
            .run();
        });
        Self {
            game_state
        }
    }
    fn update(&mut self, game_state: &GameState) {
        *self.game_state.lock().unwrap() = Some(game_state.clone())
    }
}