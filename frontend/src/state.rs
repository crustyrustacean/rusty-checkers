// src/state.rs

// dependencies
use crate::domain::{Game, GamePiece};
use yewdux::Store;

#[derive(Default, Clone, PartialEq, Store)]
pub struct State {
    pub current_game: Game,
    pub selected_piece: Option<(usize, usize)>,
    pub valid_moves: Vec<(usize, usize)>,
    pub captured_pieces: Vec<GamePiece>,
}
