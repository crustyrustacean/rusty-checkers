// src/state.rs

// dependencies
use checkers_common::Game;
use yewdux::Store;

#[derive(Default, Clone, PartialEq, Store)]
pub struct State {
    pub current_game: Game,
    pub selected_piece: Option<(usize, usize)>,
    pub valid_moves: Vec<(usize, usize)>,
}
