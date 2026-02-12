// common/src/traits.rs

// dependencies
use crate::game::MoveResult;
use crate::Player;
use serde_json::Value;
use std::any::Any;

pub trait BoardGame: Send + Sync + Any {
    fn apply_move(&mut self, start: (usize, usize), end: (usize, usize)) -> MoveResult;
    fn get_valid_moves(&self, start: (usize, usize)) -> Vec<(usize, usize)>;
    fn current_player(&self) -> Player;
    fn winner(&self) -> Option<Player>;
    fn to_json(&self) -> Value;
    fn as_any(&self) -> &dyn Any;
    fn box_clone(&self) -> Box<dyn BoardGame>;
}

impl Clone for Box<dyn BoardGame> {
    fn clone(&self) -> Box<dyn BoardGame> {
        self.box_clone()
    }
}