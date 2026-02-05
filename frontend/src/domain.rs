// frontend/src/domain.rs

// dependencies

#[derive(Clone, Debug, PartialEq)]
pub struct Game {
    pub pieces: Vec<GamePiece>,
    pub captured_pieces: Vec<GamePiece>,
    pub current_player: Player,
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
            captured_pieces: vec![],
            current_player: Player::Dark,
        }
    }

    pub fn valid_moves(&self, piece: &GamePiece) -> Vec<(usize, usize)> {
        let mut valid_moves: Vec<(usize, usize)> = Vec::new();

        match piece.owner {
            Player::Dark => {
                if piece.col > 0 && piece.row < 7 {
                    let destination = (piece.row + 1, piece.col - 1);
                    let is_occupied = self
                        .pieces
                        .iter()
                        .any(|p| p.row == destination.0 && p.col == destination.1);

                    if !is_occupied {
                        valid_moves.push(destination);
                    } else {
                        let adjacent_piece = self.pieces.iter().find(|p| p.row == destination.0 && p.col == destination.1);
                        if let Some(adj) = adjacent_piece {
                            if adj.owner == Player::Light {
                                if piece.col > 1 && piece.row < 6 {
                                    let destination = (piece.row + 2, piece.col - 2);
                                    let is_occupied = self
                                        .pieces
                                        .iter()
                                        .any(|p| p.row == destination.0 && p.col == destination.1);

                                    if !is_occupied {
                                        valid_moves.push(destination);
                                    }
                                }
                            }
                        }
                    }
                }

                if piece.col < 7 && piece.row < 7 {
                    let destination = (piece.row + 1, piece.col + 1);
                    let is_occupied = self
                        .pieces
                        .iter()
                        .any(|p| p.row == destination.0 && p.col == destination.1);

                    if !is_occupied {
                        valid_moves.push(destination);
                    } else {
                        let adjacent_piece = self.pieces.iter().find(|p| p.row == destination.0 && p.col == destination.1);
                        if let Some(adj) = adjacent_piece {
                            if adj.owner == Player::Light {
                                if piece.col < 6 && piece.row < 6 {
                                    let destination = (piece.row + 2, piece.col + 2);
                                    let is_occupied = self
                                        .pieces
                                        .iter()
                                        .any(|p| p.row == destination.0 && p.col == destination.1);

                                    if !is_occupied {
                                        valid_moves.push(destination);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Player::Light => {
                if piece.col < 7 && piece.row > 0 {
                    let destination = (piece.row - 1, piece.col + 1);
                    let is_occupied = self
                        .pieces
                        .iter()
                        .any(|p| p.row == destination.0 && p.col == destination.1);

                    if !is_occupied {
                        valid_moves.push(destination);
                    } else {
                        let adjacent_piece = self.pieces.iter().find(|p| p.row == destination.0 && p.col == destination.1);
                        if let Some(adj) = adjacent_piece {
                            if adj.owner == Player::Dark {
                                if piece.col < 6 && piece.row > 1 {
                                    let destination = (piece.row - 2, piece.col + 2);
                                    let is_occupied = self
                                        .pieces
                                        .iter()
                                        .any(|p| p.row == destination.0 && p.col == destination.1);

                                    if !is_occupied {
                                        valid_moves.push(destination);
                                    }
                                }
                            }
                        }
                    }
                }

                if piece.col > 0 && piece.row > 0 {
                    let destination = (piece.row - 1, piece.col - 1);
                    let is_occupied = self
                        .pieces
                        .iter()
                        .any(|p| p.row == destination.0 && p.col == destination.1);

                    if !is_occupied {
                        valid_moves.push(destination);
                    } else {
                        let adjacent_piece = self.pieces.iter().find(|p| p.row == destination.0 && p.col == destination.1);
                        if let Some(adj) = adjacent_piece {
                            if adj.owner == Player::Dark{
                                if piece.col > 1 && piece.row > 1 {
                                    let destination = (piece.row - 2, piece.col - 2);
                                    let is_occupied = self
                                        .pieces
                                        .iter()
                                        .any(|p| p.row == destination.0 && p.col == destination.1);

                                    if !is_occupied {
                                        valid_moves.push(destination);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        valid_moves
    }

    pub fn advance(&mut self, piece: &GamePiece, dest_row: usize, dest_col: usize) {
        if let Some(p) = self.pieces.iter_mut().find(|p| p.row == piece.row && p.col == piece.col) {
            p.row = dest_row;
            p.col = dest_col;
        }
    }

    pub fn capture(&mut self, row: usize, col: usize) {
        let index = self.pieces.iter().position(|p| p.row == row && p.col == col).unwrap();
        let piece = self.pieces.remove(index);
        self.captured_pieces.push(piece);
    }

    pub fn switch_turn(&mut self) {
        self.current_player = match self.current_player {
            Player::Dark => Player::Light,
            Player::Light => Player::Dark,
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
    pub is_kinged: bool,
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
