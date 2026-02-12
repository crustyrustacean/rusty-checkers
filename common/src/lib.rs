// common/src/lib.rs

// module declarations
mod ai;
mod game;
mod messages;
mod piece;
mod player;
pub mod tournament;
mod traits;

#[cfg(test)]
mod game_tests;

// re-exports
pub use ai::{AiPlayer, MinimaxAi, RandomAi};
pub use game::{Game, MoveResult};
pub use messages::*;
pub use piece::GamePiece;
pub use player::Player;
pub use tournament::*;
pub use traits::BoardGame;
