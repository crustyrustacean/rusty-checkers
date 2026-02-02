// src/state.rs

// dependencies
use crate::domain::Game;
use yewdux::Store;

#[derive(Default, Clone, PartialEq, Store)]
pub struct State {
    pub current_game: Game,
}