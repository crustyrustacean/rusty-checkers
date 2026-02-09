// common/src/lib.rs

// module declarations
mod game;
mod messages;
mod piece;
mod player;

// re-exports
pub use game::Game;
pub use messages::*;
pub use piece::GamePiece;
pub use player::Player;
