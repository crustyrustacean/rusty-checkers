// common/src/game.rs

// dependencies
use crate::GamePiece;
use crate::Player;
use crate::traits::BoardGame;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::any::Any;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Game {
    pub pieces: Vec<GamePiece>,
    pub captured_pieces: Vec<GamePiece>,
    pub current_player: Player,
    pub winner: Option<Player>,
    /// When set, the current player must continue jumping from this square.
    #[serde(skip)]
    pub must_jump_from: Option<(usize, usize)>,
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
            must_jump_from: None,
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
        self.pieces.iter().filter(|p| p.owner == *player).any(|p| {
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

    pub fn has_available_jumps(&self, piece: &GamePiece) -> bool {
        self.valid_moves(piece)
            .iter()
            .any(|(dest_row, _)| (*dest_row as i32 - piece.row as i32).abs() == 2)
    }

    pub fn play_move(
        &mut self,
        start: (usize, usize),
        end: (usize, usize),
    ) -> Result<MoveResult, String> {
        // Enforce multi-jump continuation
        if let Some(required) = self.must_jump_from
            && start != required
        {
            return Err(format!(
                "You must continue jumping from ({}, {})",
                required.0, required.1
            ));
        }

        let piece = self
            .pieces
            .iter()
            .find(|p| p.row == start.0 && p.col == start.1)
            .ok_or("No piece at start position")?
            .clone();

        if piece.owner != self.current_player {
            return Err("It is not your turn (or not your piece)".to_string());
        }

        let valid_moves = self.valid_moves(&piece);
        if !valid_moves.contains(&end) {
            return Err("Invalid move".to_string());
        }

        let was_jump = (end.0 as i32 - start.0 as i32).abs() == 2;

        self.advance(&piece, end.0, end.1);

        if was_jump {
            let captured_row = (start.0 + end.0) / 2;
            let captured_col = (start.1 + end.1) / 2;
            self.capture(captured_row, captured_col);
        }

        let mut just_kinged = false;
        if let Some(p) = self
            .pieces
            .iter_mut()
            .find(|p| p.row == end.0 && p.col == end.1)
        {
            let reached_end =
                (p.row == 0 && p.owner == Player::Light) || (p.row == 7 && p.owner == Player::Dark);

            if reached_end && !p.is_kinged {
                p.is_kinged = true;
                just_kinged = true;
            }
        }

        if was_jump && !just_kinged {
            let piece_at_dest = self
                .pieces
                .iter()
                .find(|p| p.row == end.0 && p.col == end.1)
                .unwrap();

            if self.has_available_jumps(piece_at_dest) {
                self.must_jump_from = Some((end.0, end.1));
                return Ok(MoveResult::ContinueJump(end.0, end.1));
            }
        }

        self.must_jump_from = None;
        self.switch_turn();

        let current = self.current_player.clone();
        let has_moves = self
            .pieces
            .iter()
            .filter(|p| p.owner == current)
            .any(|p| !self.valid_moves(p).is_empty());

        if !has_moves {
            let winner = match current {
                Player::Dark => Player::Light,
                Player::Light => Player::Dark,
            };
            self.winner = Some(winner.clone());
            return Ok(MoveResult::GameWon(winner));
        }

        Ok(MoveResult::TurnComplete)
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl BoardGame for Game {
    fn apply_move(&mut self, start: (usize, usize), end: (usize, usize)) -> MoveResult {
        match self.play_move(start, end) {
            Ok(result) => result,
            Err(reason) => MoveResult::InvalidMove(reason),
        }
    }

    fn get_valid_moves(&self, start: (usize, usize)) -> Vec<(usize, usize)> {
        if let Some(required) = self.must_jump_from
            && start != required {
                return vec![];
            }

        if let Some(piece) = self.pieces.iter().find(|p| p.row == start.0 && p.col == start.1) {
            if piece.owner != self.current_player {
                return vec![];
            }
            let moves = self.valid_moves(piece);

            if moves.is_empty() {
                return vec![];
            }

            let captures_exist = self.check_captures_moves(&piece.owner);

            if captures_exist {
                
                moves
                    .into_iter()
                    .filter(|(r, _)| (*r as i32 - piece.row as i32).abs() == 2)
                    .collect()
            } else {
                
                moves
            }
        } else {
            vec![]
        }
    }

    fn current_player(&self) -> Player {
        self.current_player.clone()
    }

    fn winner(&self) -> Option<Player> {
        self.winner.clone()
    }

    fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn box_clone(&self) -> Box<dyn BoardGame> {
        Box::new(self.clone())
    }

}

#[derive(Debug, Clone, PartialEq)]
pub enum MoveResult {
    TurnComplete,
    ContinueJump(usize, usize),
    GameWon(Player),
    InvalidMove(String),
}


