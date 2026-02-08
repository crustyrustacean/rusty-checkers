// common/src/player.rs

// dependencies
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
pub enum Player {
    Dark,
    Light,
}