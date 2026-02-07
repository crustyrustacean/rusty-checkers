// frontend/src/domain.rs

// dependencies

#[derive(Clone, Debug, PartialEq)]
pub struct Game {
    pub pieces: Vec<GamePiece>,
    pub captured_pieces: Vec<GamePiece>,
    pub current_player: Player,
    pub winner: Option<Player>,
}

impl Game {
    pub fn new() -> Self {
        let pieces: Vec<GamePiece> = (0..8)
            .flat_map(|r| (0..8).map(move |c| (r, c)))
            .filter(|(r, c)| (r + c) % 2 != 0)
            .filter_map(|(r, c)| {
                if r < 3 {
                    Some(GamePiece::new(Player::Dark, r, c))
                } else if r > 4 {
                    Some(GamePiece::new(Player::Light, r, c))
                } else {
                    None
                }
            })
            .collect();

        Self {
            pieces,
            captured_pieces: vec![],
            current_player: Player::Dark,
            winner: None,
        }
    }

    pub fn valid_moves(&self, piece: &GamePiece) -> Vec<(usize, usize)> {
        let mut valid_moves: Vec<(usize, usize)> = Vec::new();

        let directions = if piece.is_kinged {
            vec![(-1, 1), (-1, -1), (1, 1), (1, -1)]
        } else {
            match piece.owner {
                Player::Dark => vec![(1, 1), (1, -1)],
                Player::Light => vec![(-1, 1), (-1, -1)],
            }
        };

        for (row_offset, col_offset) in &directions {
            if (*row_offset == -1 && piece.row == 0) || (*row_offset == 1 && piece.row == 7) {
                continue;
            }
            if (*col_offset == -1 && piece.col == 0) || (*col_offset == 1 && piece.col == 7) {
                continue;
            }

            let dest_row = (piece.row as i32 + row_offset) as usize;
            let dest_col = (piece.col as i32 + col_offset) as usize;

            let is_occupied = self
                .pieces
                .iter()
                .any(|p| p.row == dest_row && p.col == dest_col);

            if !is_occupied {
                valid_moves.push((dest_row, dest_col));
            } else {
                let adjacent_piece = self
                    .pieces
                    .iter()
                    .find(|p| p.row == dest_row && p.col == dest_col);
                if let Some(adj) = adjacent_piece
                    && adj.owner != piece.owner
                {
                    let land_row_i32 = dest_row as i32 + row_offset;
                    let land_col_i32 = dest_col as i32 + col_offset;

                    if (0..=7).contains(&land_row_i32) && (0..=7).contains(&land_col_i32) {
                        let land_row = land_row_i32 as usize;
                        let land_col = land_col_i32 as usize;

                        let is_occupied = self
                            .pieces
                            .iter()
                            .any(|p| p.row == land_row && p.col == land_col);

                        if !is_occupied {
                            valid_moves.push((land_row, land_col));
                        }
                    }
                }
            }
        }

        valid_moves
    }

    pub fn advance(&mut self, piece: &GamePiece, dest_row: usize, dest_col: usize) {
        if let Some(p) = self
            .pieces
            .iter_mut()
            .find(|p| p.row == piece.row && p.col == piece.col)
        {
            p.row = dest_row;
            p.col = dest_col;
        }
    }

    pub fn check_captures_moves(&self, player: &Player) -> bool {
    self.pieces
        .iter()
        .filter(|p| p.owner == *player)
        .any(|p| {
            self.valid_moves(p)
                .iter()
                .any(|(dest_row, _dest_col)| (*dest_row as i32 - p.row as i32).abs() == 2)
        })
}

    pub fn capture(&mut self, row: usize, col: usize) {
        let index = self
            .pieces
            .iter()
            .position(|p| p.row == row && p.col == col)
            .unwrap();
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
            row,
            col,
            is_kinged: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_valid_moves_when_blocked() {
        // Dark piece blocked by own pieces (can't jump your own)
        let game = Game {
            pieces: vec![
                GamePiece::new(Player::Dark, 0, 1), // Dark piece in top row
                GamePiece::new(Player::Dark, 1, 0), // Dark blocking down-left
                GamePiece::new(Player::Dark, 1, 2), // Dark blocking down-right
            ],
            captured_pieces: vec![],
            current_player: Player::Dark,
            winner: None,
        };

        let dark_piece = &game.pieces[0];
        let moves = game.valid_moves(dark_piece);

        assert!(moves.is_empty(), "Dark piece should have no valid moves");
    }

    #[test]
    fn test_has_valid_moves_when_not_blocked() {
        let game = Game {
            pieces: vec![
                GamePiece::new(Player::Dark, 2, 3), // Dark piece in middle of board
            ],
            captured_pieces: vec![],
            current_player: Player::Dark,
            winner: None,
        };

        let dark_piece = &game.pieces[0];
        let moves = game.valid_moves(dark_piece);
        println!("Moves found: {:?}", moves);
        assert_eq!(moves.len(), 2, "Dark piece should have 2 valid moves");
        assert!(moves.contains(&(3, 2)), "Should be able to move down-left");
        assert!(moves.contains(&(3, 4)), "Should be able to move down-right");
    }
}
