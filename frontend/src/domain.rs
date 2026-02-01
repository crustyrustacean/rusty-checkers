// frontend/src/domain.rs

// dependencies

#[derive(Debug)]
pub struct Game {
    pieces: Vec<GamePiece>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            pieces: vec![
                GamePiece::new(Player::Dark, 0, 1),
                GamePiece::new(Player::Dark, 0, 3),
                GamePiece::new(Player::Dark, 0, 5),
                GamePiece::new(Player::Dark, 0, 7),
                GamePiece::new(Player::Dark, 1, 0),
                GamePiece::new(Player::Dark, 1, 2),
                GamePiece::new(Player::Dark, 1, 4),
                GamePiece::new(Player::Dark, 1, 6),
                GamePiece::new(Player::Dark, 2, 1),
                GamePiece::new(Player::Dark, 2, 3),
                GamePiece::new(Player::Dark, 2, 5),
                GamePiece::new(Player::Dark, 2, 7),
                GamePiece::new(Player::Light, 5, 1),
                GamePiece::new(Player::Light, 5, 3),
                GamePiece::new(Player::Light, 5, 5),
                GamePiece::new(Player::Light, 5, 7),
                GamePiece::new(Player::Light, 6, 0),
                GamePiece::new(Player::Light, 6, 2),
                GamePiece::new(Player::Light, 6, 4),
                GamePiece::new(Player::Light, 6, 6),
                GamePiece::new(Player::Light, 7, 1),
                GamePiece::new(Player::Light, 7, 3),
                GamePiece::new(Player::Light, 7, 5),
                GamePiece::new(Player::Light, 7, 7),
            ],
        }
    }
}

#[derive(Debug)]
pub enum Player {
    Dark,
    Light,
}

#[derive(Debug)]
pub struct GamePiece {
    owner: Player,
    row: usize,
    col: usize,
    is_kinged: bool,
}

impl GamePiece {
    pub fn new(player: Player, row: usize, col: usize) -> Self {
        Self {
            owner: player,
            row: row,
            col: col,
            is_kinged: false,
        }
    }
}