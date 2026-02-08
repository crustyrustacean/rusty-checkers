// common/src/piece.rs

// dependencies
use crate::Player;

#[derive(Debug, Clone, PartialEq)]
pub struct GamePiece {
    pub owner: Player,
    pub row: usize,
    pub col: usize,
    pub is_kinged: bool,
}

impl GamePiece {
    pub fn new(player: Player, row: usize, col: usize) -> Self {
        Self {
            owner: player,
            row,
            col,
            is_kinged: false,
        }
    }
}