// backend/src/lib.rs

// module declarations
pub mod config;
pub mod errors;
pub mod game_server;
pub mod response;
pub mod routes;
pub mod startup;
pub mod state;
pub mod telemetry;
pub mod tournament;

// re-exports
pub use config::*;
pub use errors::*;
pub use game_server::*;
pub use response::*;
pub use startup::*;
pub use state::*;
pub use telemetry::*;
