// backend/src/game_server.rs

// dependencies
use checkers_common::Game;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type GameId = String;

pub struct GameSession {
    pub game: Game,
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