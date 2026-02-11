// common/src/lib.rs

// module declarations
mod ai;
mod game;
mod messages;
mod piece;
mod player;

// re-exports
pub use ai::{AiPlayer, MinimaxAi, RandomAi};
pub use game::Game;
pub use messages::*;
pub use piece::GamePiece;
pub use player::Player;
