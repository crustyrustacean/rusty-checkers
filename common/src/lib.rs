// common/src/lib.rs

// module declarations
mod game;
mod messages;
mod piece;
mod player;

// re-exports
pub use game::Game;
pub use piece::GamePiece;
pub use messages::*;
pub use player::Player;
