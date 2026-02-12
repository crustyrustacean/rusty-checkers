// common/src/tournament.rs

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A player registered in a tournament.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct TournamentPlayer {
    pub id: Uuid,
    pub name: String,
    pub connected: bool,
}

impl TournamentPlayer {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            connected: true,
        }
    }
}

/// A single match within a tournament bracket.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Match {
    pub id: Uuid,
    pub round: usize,
    pub position: usize,
    pub player1_id: Option<Uuid>,
    pub player2_id: Option<Uuid>,
    pub game_id: Option<String>,
    pub winner_id: Option<Uuid>,
}

impl Match {
    pub fn new(round: usize, position: usize) -> Self {
        Self {
            id: Uuid::new_v4(),
            round,
            position,
            player1_id: None,
            player2_id: None,
            game_id: None,
            winner_id: None,
        }
    }

    /// Returns `true` if both player slots are filled.
    pub fn is_ready(&self) -> bool {
        self.player1_id.is_some() && self.player2_id.is_some()
    }

    /// Returns `true` if a winner has been determined.
    pub fn is_complete(&self) -> bool {
        self.winner_id.is_some()
    }
}

/// The lifecycle state of a tournament.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum TournamentState {
    Lobby,
    InProgress,
    Finished,
}

/// A serializable snapshot of the tournament for clients.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct TournamentView {
    pub code: String,
    pub state: TournamentState,
    pub players: Vec<TournamentPlayer>,
    pub matches: Vec<Match>,
    pub current_round: usize,
    pub total_rounds: usize,
    pub winner: Option<TournamentPlayer>,
}
