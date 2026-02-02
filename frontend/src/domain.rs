// frontend/src/domain.rs

// dependencies

#[derive(Clone, Debug, PartialEq)]
pub struct Game {
    pub pieces: Vec<GamePiece>,
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
                GamePiece::new(Player::Light, 5, 0),
                GamePiece::new(Player::Light, 5, 2),
                GamePiece::new(Player::Light, 5, 4),
                GamePiece::new(Player::Light, 5, 6),
                GamePiece::new(Player::Light, 6, 1),
                GamePiece::new(Player::Light, 6, 3),
                GamePiece::new(Player::Light, 6, 5),
                GamePiece::new(Player::Light, 6, 7),
                GamePiece::new(Player::Light, 7, 0),
                GamePiece::new(Player::Light, 7, 2),
                GamePiece::new(Player::Light, 7, 4),
                GamePiece::new(Player::Light, 7, 6),
            ],
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Player {
    Dark,
    Light,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GamePiece {
    pub owner: Player,
    pub row: usize,
    pub col: usize,
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
