// backend/src/game_server.rs

// dependencies
use crate::tournament::manager::TournamentManager;
use checkers_common::{AiPlayer, BoardGame, Player};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;

pub type GameId = String;
pub type TournamentCode = String;
pub type PlayerSender = mpsc::UnboundedSender<String>;

pub struct GameSession {
    pub game: Box<dyn BoardGame>,
    pub dark_player: Option<PlayerSender>,
    pub light_player: Option<PlayerSender>,
    pub ai_opponent: Option<Box<dyn AiPlayer + Send + Sync>>,
    pub ai_color: Option<Player>,
    /// If this game is part of a tournament, the room code.
    pub tournament_code: Option<TournamentCode>,
}

/// Per-player state tracked by the server for tournament participants.
pub struct TournamentConnection {
    pub tournament_code: TournamentCode,
    pub player_id: Uuid,
    pub tx: PlayerSender,
}

pub struct GameServer {
    pub games: RwLock<HashMap<GameId, GameSession>>,
    pub waiting_player: RwLock<Option<GameId>>,
    pub tournaments: RwLock<HashMap<TournamentCode, TournamentManager>>,
    /// Maps a tournament player's Uuid to their WebSocket sender and tournament code.
    pub tournament_connections: RwLock<HashMap<Uuid, TournamentConnection>>,
}

impl GameServer {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            games: RwLock::new(HashMap::new()),
            waiting_player: RwLock::new(None),
            tournaments: RwLock::new(HashMap::new()),
            tournament_connections: RwLock::new(HashMap::new()),
        })
    }
}
