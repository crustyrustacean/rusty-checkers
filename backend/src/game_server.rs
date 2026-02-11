// backend/src/game_server.rs

// dependencies
use checkers_common::{AiPlayer, Game, Player};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};

pub type GameId = String;
pub type PlayerSender = mpsc::UnboundedSender<String>;

pub struct GameSession {
    pub game: Game,
    pub dark_player: Option<PlayerSender>,
    pub light_player: Option<PlayerSender>,
    pub ai_opponent: Option<Box<dyn AiPlayer + Send + Sync>>,
    pub ai_color: Option<Player>,
}

pub struct GameServer {
    pub games: RwLock<HashMap<GameId, GameSession>>,
    pub waiting_player: RwLock<Option<GameId>>,
}

impl GameServer {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            games: RwLock::new(HashMap::new()),
            waiting_player: RwLock::new(None),
        })
    }
}
