// common/src/game.rs

// dependencies
use crate::GamePiece;
use crate::Player;
use crate::traits::BoardGame;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
        if let Some(piece) = self.pieces.iter().find(|p| p.row == start.0 && p.col == start.1) {
            self.valid_moves(piece)
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

    }

#[derive(Debug, Clone, PartialEq)]
pub enum MoveResult {
    TurnComplete,
    ContinueJump(usize, usize),
    GameWon(Player),
    InvalidMove(String),
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
            must_jump_from: None,
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
            must_jump_from: None,
        };

        let dark_piece = &game.pieces[0];
        let moves = game.valid_moves(dark_piece);
        println!("Moves found: {:?}", moves);
        assert_eq!(moves.len(), 2, "Dark piece should have 2 valid moves");
        assert!(moves.contains(&(3, 2)), "Should be able to move down-left");
        assert!(moves.contains(&(3, 4)), "Should be able to move down-right");
    }

    /// Helper to build a game with specific pieces and current player.
    fn game_with(pieces: Vec<GamePiece>, current: Player) -> Game {
        Game {
            pieces,
            captured_pieces: vec![],
            current_player: current,
            winner: None,
            must_jump_from: None,
        }
    }

    // --- play_move tests ---

    #[test]
    fn play_move_simple_advance_returns_turn_complete() {
        let mut game = game_with(
            vec![
                GamePiece::new(Player::Dark, 2, 3),
                GamePiece::new(Player::Light, 5, 4), // opponent has moves
            ],
            Player::Dark,
        );
        let result = game.play_move((2, 3), (3, 4));
        assert_eq!(result, Ok(MoveResult::TurnComplete));
        assert_eq!(game.current_player, Player::Light, "turn should switch");
        assert!(
            game.pieces.iter().any(|p| p.row == 3 && p.col == 4),
            "piece should be at destination"
        );
    }

    #[test]
    fn play_move_jump_captures_piece_and_completes() {
        let mut game = game_with(
            vec![
                GamePiece::new(Player::Dark, 3, 2),
                GamePiece::new(Player::Light, 4, 3),
                GamePiece::new(Player::Light, 6, 1), // extra Light so game doesn't end
            ],
            Player::Dark,
        );

        let result = game.play_move((3, 2), (5, 4));
        assert_eq!(result, Ok(MoveResult::TurnComplete));
        // Captured piece should be removed from active pieces
        assert!(
            !game.pieces.iter().any(|p| p.row == 4 && p.col == 3),
            "captured piece should be removed"
        );
        assert_eq!(game.captured_pieces.len(), 1);
        assert_eq!(game.current_player, Player::Light);
    }

    #[test]
    fn play_move_multi_jump_returns_continue_then_complete() {
        // Dark at (2,1), Light at (3,2), (5,4), and (7,0) (extra so game doesn't end)
        // Dark jumps (2,1)->(4,3) capturing (3,2), then must continue (4,3)->(6,5) capturing (5,4)
        let mut game = game_with(
            vec![
                GamePiece::new(Player::Dark, 2, 1),
                GamePiece::new(Player::Light, 3, 2),
                GamePiece::new(Player::Light, 5, 4),
                GamePiece::new(Player::Light, 7, 0), // extra Light piece
            ],
            Player::Dark,
        );

        let result = game.play_move((2, 1), (4, 3));
        assert_eq!(result, Ok(MoveResult::ContinueJump(4, 3)));
        assert_eq!(game.captured_pieces.len(), 1);
        assert_eq!(
            game.must_jump_from,
            Some((4, 3)),
            "must_jump_from should be set"
        );

        let result = game.play_move((4, 3), (6, 5));
        assert_eq!(result, Ok(MoveResult::TurnComplete));
        assert_eq!(game.captured_pieces.len(), 2);
        assert!(game.must_jump_from.is_none(), "must_jump_from should be cleared");
        assert_eq!(game.current_player, Player::Light);
    }

    #[test]
    fn play_move_from_empty_square_returns_err() {
        let mut game = game_with(vec![GamePiece::new(Player::Dark, 2, 3)], Player::Dark);
        let result = game.play_move((0, 0), (1, 1));
        assert!(result.is_err());
    }

    #[test]
    fn play_move_opponents_piece_returns_err() {
        let mut game = game_with(
            vec![
                GamePiece::new(Player::Dark, 2, 3),
                GamePiece::new(Player::Light, 5, 4),
            ],
            Player::Dark,
        );

        let result = game.play_move((5, 4), (4, 3));
        assert!(result.is_err());
    }

    #[test]
    fn play_move_illegal_destination_returns_err() {
        let mut game = game_with(vec![GamePiece::new(Player::Dark, 2, 3)], Player::Dark);
        // Try to move backwards (Dark can only move forward)
        let result = game.play_move((2, 3), (1, 2));
        assert!(result.is_err());
    }

    #[test]
    fn play_move_kings_dark_at_row_7() {
        let mut game = game_with(
            vec![
                GamePiece::new(Player::Dark, 6, 3),
                GamePiece::new(Player::Light, 1, 0), // opponent has moves
            ],
            Player::Dark,
        );
        let result = game.play_move((6, 3), (7, 4));
        assert_eq!(result, Ok(MoveResult::TurnComplete));
        let piece = game.pieces.iter().find(|p| p.row == 7 && p.col == 4).unwrap();
        assert!(piece.is_kinged, "Dark piece should be kinged at row 7");
    }

    #[test]
    fn play_move_kings_light_at_row_0() {
        let mut game = game_with(
            vec![
                GamePiece::new(Player::Light, 1, 2),
                GamePiece::new(Player::Dark, 6, 1), // opponent has moves
            ],
            Player::Light,
        );
        let result = game.play_move((1, 2), (0, 1));
        assert_eq!(result, Ok(MoveResult::TurnComplete));
        let piece = game.pieces.iter().find(|p| p.row == 0 && p.col == 1).unwrap();
        assert!(piece.is_kinged, "Light piece should be kinged at row 0");
    }

    #[test]
    fn play_move_wins_when_opponent_has_no_moves() {
        // Dark captures the only Light piece, leaving Light with nothing
        let mut game = game_with(
            vec![
                GamePiece::new(Player::Dark, 3, 2),
                GamePiece::new(Player::Light, 4, 3),
            ],
            Player::Dark,
        );

        let result = game.play_move((3, 2), (5, 4));
        assert_eq!(result, Ok(MoveResult::GameWon(Player::Dark)));
        assert_eq!(game.winner, Some(Player::Dark));
    }

    #[test]
    fn play_move_must_jump_from_enforced() {
        // Set up a board mid-multi-jump: Dark is at (4,3) and must continue from there
        let mut game = game_with(
            vec![
                GamePiece::new(Player::Dark, 4, 3),
                GamePiece::new(Player::Dark, 1, 0), // another Dark piece
                GamePiece::new(Player::Light, 5, 4),
            ],
            Player::Dark,
        );
        game.must_jump_from = Some((4, 3));

        // Try to move the other piece instead
        let result = game.play_move((1, 0), (2, 1));
        assert!(result.is_err(), "should reject move from wrong piece during multi-jump");
        assert!(result.unwrap_err().contains("must continue jumping"));
    }
}
