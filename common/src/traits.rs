// common/src/traits.rs

// dependencies
use crate::game::MoveResult;
use crate::Player;
use serde_json::Value;

pub trait BoardGame: Send + Sync {
    fn apply_move(&mut self, start: (usize, usize), end: (usize, usize)) -> MoveResult;
    fn get_valid_moves(&self, start: (usize, usize)) -> Vec<(usize, usize)>;
    fn current_player(&self) -> Player;
    fn winner(&self) -> Option<Player>;
    fn to_json(&self) -> Value;
}