//! AI opponents for the checkers game.
//!
//! Provides a trait [`AiPlayer`] and two implementations:
//! - [`RandomAi`] picks a random legal move each turn.
//! - [`MinimaxAi`] uses minimax with alpha-beta pruning at a configurable depth.

use crate::game::Game;
use crate::player::Player;
use rand::seq::SliceRandom;

/// Trait for AI players that can select a move given a game state.
pub trait AiPlayer {
    /// Select a move for the given `color` in the current `game` state.
    ///
    /// Returns `Some((start, end))` with the source and destination coordinates,
    /// or `None` if no legal moves exist.
    fn select_move(&self, game: &Game, color: &Player) -> Option<((usize, usize), (usize, usize))>;
}

/// An AI that picks a random legal move each turn.
pub struct RandomAi;

impl AiPlayer for RandomAi {
    fn select_move(&self, game: &Game, color: &Player) -> Option<((usize, usize), (usize, usize))> {
        let captures_exist = game.check_captures_moves(color);

        let mut all_moves: Vec<((usize, usize), (usize, usize))> = Vec::new();

        for piece in game.pieces.iter().filter(|p| p.owner == *color) {
            let moves = game.valid_moves(piece);
            for dest in moves {
                let is_capture = (dest.0 as i32 - piece.row as i32).abs() == 2;
                if captures_exist && !is_capture {
                    continue;
                }
                all_moves.push(((piece.row, piece.col), dest));
            }
        }

        let mut rng = rand::thread_rng();
        all_moves.choose(&mut rng).copied()
    }
}

/// An AI that uses minimax with alpha-beta pruning.
///
/// The `depth` parameter controls how many moves ahead the AI looks.
/// Suggested difficulty levels:
/// - Easy: depth 2
/// - Medium: depth 4
/// - Hard: depth 6
pub struct MinimaxAi {
    depth: u32,
}

impl MinimaxAi {
    /// Create a new minimax AI with the given search depth.
    pub fn new(depth: u32) -> Self {
        Self { depth }
    }

    /// Evaluate a board position from the perspective of `ai_color`.
    ///
    /// Positive scores favor the AI; negative scores favor the opponent.
    fn evaluate(game: &Game, ai_color: &Player) -> i32 {
        let mut score = 0;

        for piece in &game.pieces {
            let is_ai = piece.owner == *ai_color;
            let piece_value = if piece.is_kinged { 3 } else { 1 };

            // Center bonus: columns 2-5 and rows 2-5 are more central
            let center_bonus = if (2..=5).contains(&piece.col) && (2..=5).contains(&piece.row) {
                1
            } else {
                0
            };

            // Advancement bonus: how close to promotion row
            let advancement_bonus = if !piece.is_kinged {
                match piece.owner {
                    Player::Dark => piece.row as i32, // Dark advances toward row 7
                    Player::Light => (7 - piece.row) as i32, // Light advances toward row 0
                }
            } else {
                0
            };

            let total = (piece_value * 10) + center_bonus + advancement_bonus;

            if is_ai {
                score += total;
            } else {
                score -= total;
            }
        }

        score
    }

    /// Collect all legal moves for `color`, respecting forced captures.
    fn collect_moves(game: &Game, color: &Player) -> Vec<((usize, usize), (usize, usize))> {
        let captures_exist = game.check_captures_moves(color);
        let mut all_moves = Vec::new();

        for piece in game.pieces.iter().filter(|p| p.owner == *color) {
            for dest in game.valid_moves(piece) {
                let is_capture = (dest.0 as i32 - piece.row as i32).abs() == 2;
                if captures_exist && !is_capture {
                    continue;
                }
                all_moves.push(((piece.row, piece.col), dest));
            }
        }

        all_moves
    }

    /// Apply a move to a cloned game state, completing any multi-jump
    /// chain greedily (picking the first available continuation).
    /// Returns the resulting game state after the full turn.
    fn apply_move(game: &Game, start: (usize, usize), end: (usize, usize)) -> Game {
        use crate::game::MoveResult;

        let mut game = game.clone();

        match game.play_move(start, end) {
            Ok(MoveResult::ContinueJump(row, col)) => {
                // Greedily continue the multi-jump chain
                let jump_moves = Self::collect_jump_moves(&game, (row, col));
                if let Some(&next_dest) = jump_moves.first() {
                    return Self::apply_move(&game, (row, col), next_dest);
                }
                // No continuation found (shouldn't happen), return as-is
                game
            }
            Ok(MoveResult::TurnComplete) | Ok(MoveResult::GameWon(_)) => game,
            Ok(MoveResult::InvalidMove(_)) => game,
            Err(_) => game, // Invalid move — return unchanged state
        }
    }

    /// Collect only jump destinations for a piece at the given position.
    fn collect_jump_moves(game: &Game, pos: (usize, usize)) -> Vec<(usize, usize)> {
        let Some(piece) = game.pieces.iter().find(|p| p.row == pos.0 && p.col == pos.1) else {
            return Vec::new();
        };
        game.valid_moves(piece)
            .into_iter()
            .filter(|(r, _)| (*r as i32 - pos.0 as i32).abs() == 2)
            .collect()
    }

    /// Minimax with alpha-beta pruning.
    ///
    /// Returns the evaluation score of the position.
    fn minimax(
        game: &Game,
        depth: u32,
        mut alpha: i32,
        mut beta: i32,
        maximizing: bool,
        ai_color: &Player,
    ) -> i32 {
        if depth == 0 || game.winner.is_some() {
            return Self::evaluate(game, ai_color);
        }

        let current_color = &game.current_player;
        let moves = Self::collect_moves(game, current_color);

        if moves.is_empty() {
            return Self::evaluate(game, ai_color);
        }

        if maximizing {
            let mut max_eval = i32::MIN;
            for (start, end) in &moves {
                let next_state = Self::apply_move(game, *start, *end);
                let next_maximizing = next_state.current_player == *ai_color;
                let eval = Self::minimax(
                    &next_state,
                    depth - 1,
                    alpha,
                    beta,
                    next_maximizing,
                    ai_color,
                );
                max_eval = max_eval.max(eval);
                alpha = alpha.max(eval);
                if beta <= alpha {
                    break;
                }
            }
            max_eval
        } else {
            let mut min_eval = i32::MAX;
            for (start, end) in &moves {
                let next_state = Self::apply_move(game, *start, *end);
                let next_maximizing = next_state.current_player == *ai_color;
                let eval = Self::minimax(
                    &next_state,
                    depth - 1,
                    alpha,
                    beta,
                    next_maximizing,
                    ai_color,
                );
                min_eval = min_eval.min(eval);
                beta = beta.min(eval);
                if beta <= alpha {
                    break;
                }
            }
            min_eval
        }
    }
}

impl AiPlayer for MinimaxAi {
    fn select_move(&self, game: &Game, color: &Player) -> Option<((usize, usize), (usize, usize))> {
        let moves = Self::collect_moves(game, color);

        if moves.is_empty() {
            return None;
        }

        let mut best_move = None;
        let mut best_score = i32::MIN;

        for (start, end) in &moves {
            let next_state = Self::apply_move(game, *start, *end);
            let next_maximizing = next_state.current_player == *color;
            let score = Self::minimax(
                &next_state,
                self.depth - 1,
                i32::MIN,
                i32::MAX,
                next_maximizing,
                color,
            );

            if score > best_score {
                best_score = score;
                best_move = Some((*start, *end));
            }
        }

        best_move
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::piece::GamePiece;

    /// Helper: create an empty game with specific pieces.
    fn game_with_pieces(pieces: Vec<GamePiece>, current: Player) -> Game {
        Game {
            pieces,
            captured_pieces: vec![],
            current_player: current,
            winner: None,
            must_jump_from: None,
        }
    }

    #[test]
    fn test_random_ai_returns_valid_move() {
        let game = Game::new();
        let ai = RandomAi;

        let result = ai.select_move(&game, &Player::Dark);
        assert!(
            result.is_some(),
            "RandomAi should return a move from the starting position"
        );

        let ((sr, sc), (er, ec)) = result.unwrap();
        let piece = game
            .pieces
            .iter()
            .find(|p| p.row == sr && p.col == sc)
            .expect("move should start from an existing piece");
        assert_eq!(piece.owner, Player::Dark);

        let valid = game.valid_moves(piece);
        assert!(
            valid.contains(&(er, ec)),
            "move destination should be valid"
        );
    }

    #[test]
    fn test_random_ai_respects_forced_captures() {
        // Set up a board where Dark has a forced capture
        let game = game_with_pieces(
            vec![
                GamePiece::new(Player::Dark, 3, 2),
                GamePiece::new(Player::Light, 4, 3),
                // Empty at (5, 4) so Dark can jump
            ],
            Player::Dark,
        );

        let ai = RandomAi;
        let result = ai.select_move(&game, &Player::Dark);
        assert!(result.is_some());

        let ((sr, sc), (er, ec)) = result.unwrap();
        assert_eq!((sr, sc), (3, 2));
        assert_eq!((er, ec), (5, 4), "RandomAi must take the forced capture");
    }

    #[test]
    fn test_random_ai_returns_none_when_no_moves() {
        // Dark piece boxed in at corner with no moves
        let game = game_with_pieces(vec![GamePiece::new(Player::Dark, 7, 0)], Player::Dark);

        let ai = RandomAi;
        let result = ai.select_move(&game, &Player::Dark);
        assert!(
            result.is_none(),
            "should return None when no moves available"
        );
    }

    #[test]
    fn test_minimax_returns_valid_move() {
        let game = Game::new();
        let ai = MinimaxAi::new(2);

        let result = ai.select_move(&game, &Player::Dark);
        assert!(
            result.is_some(),
            "MinimaxAi should return a move from the starting position"
        );

        let ((sr, sc), (er, ec)) = result.unwrap();
        let piece = game
            .pieces
            .iter()
            .find(|p| p.row == sr && p.col == sc)
            .expect("move should start from an existing piece");
        assert_eq!(piece.owner, Player::Dark);

        let valid = game.valid_moves(piece);
        assert!(
            valid.contains(&(er, ec)),
            "move destination should be valid"
        );
    }

    #[test]
    fn test_minimax_takes_obvious_capture() {
        // Dark at (3,2), Light at (4,3), empty at (5,4) — obvious capture
        let game = game_with_pieces(
            vec![
                GamePiece::new(Player::Dark, 3, 2),
                GamePiece::new(Player::Light, 4, 3),
            ],
            Player::Dark,
        );

        let ai = MinimaxAi::new(4);
        let result = ai.select_move(&game, &Player::Dark);
        assert!(result.is_some());

        let (start, end) = result.unwrap();
        assert_eq!(start, (3, 2));
        assert_eq!(end, (5, 4), "MinimaxAi should take the obvious capture");
    }

    #[test]
    fn test_minimax_blocks_obvious_threat() {
        // Light at (4,3) can capture Dark at (3,2) if Dark doesn't move.
        // Dark has another piece at (2,5) that can move safely.
        // Dark should move (3,2) out of danger rather than the other piece.
        let game = game_with_pieces(
            vec![
                GamePiece::new(Player::Dark, 3, 2),
                GamePiece::new(Player::Dark, 2, 5),
                GamePiece::new(Player::Light, 4, 3),
            ],
            Player::Dark,
        );

        let ai = MinimaxAi::new(4);
        let result = ai.select_move(&game, &Player::Dark);
        assert!(result.is_some());

        let (start, _end) = result.unwrap();
        // The AI should move the threatened piece at (3,2) rather than
        // the safe piece at (2,5), or at least take the capture itself.
        // Either moving (3,2) away or capturing with it is acceptable.
        assert_eq!(start, (3, 2), "MinimaxAi should move the threatened piece");
    }

    #[test]
    fn test_minimax_evaluation_favors_more_pieces() {
        // AI has 2 pieces, opponent has 1 — AI should have positive eval
        let game = game_with_pieces(
            vec![
                GamePiece::new(Player::Dark, 3, 2),
                GamePiece::new(Player::Dark, 2, 5),
                GamePiece::new(Player::Light, 5, 4),
            ],
            Player::Dark,
        );

        let score = MinimaxAi::evaluate(&game, &Player::Dark);
        assert!(
            score > 0,
            "AI with more pieces should have positive evaluation, got {}",
            score
        );
    }

    #[test]
    fn test_minimax_evaluation_king_worth_more() {
        // One Dark king vs one Light regular piece
        let mut king = GamePiece::new(Player::Dark, 4, 3);
        king.is_kinged = true;

        let game = game_with_pieces(
            vec![king, GamePiece::new(Player::Light, 5, 4)],
            Player::Dark,
        );

        let score = MinimaxAi::evaluate(&game, &Player::Dark);
        assert!(
            score > 0,
            "King should be worth more than regular piece, got {}",
            score
        );
    }
}
