use crate::game_state::GameState;

#[cfg(feature = "standard_game_controller")]
mod standard;

pub trait GCTrait {
    fn new(
        blue_team_name: &'static str,
        green_team_name: &'static str,
        blue_team_positive: bool,
    ) -> Self;
    fn step(&mut self);
    fn get_game_state(&self) -> GameState;
}

#[cfg(feature = "standard_game_controller")]
pub use standard::GC;

#[cfg(not(any(feature = "standard_game_controller")))]
compile_error!(
    "No game controller feature enabled. You need to enable at least one game controller feature."
);
