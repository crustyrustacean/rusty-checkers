// common/src/ai.rs

// depedencies
use crate::game::Game;
use crate::player::Player;
use crate::traits::BoardGame;
use rand::seq::SliceRandom;

pub trait AiPlayer {
    fn select_move(
        &self,
        game: &dyn BoardGame,
        color: &Player,
    ) -> Option<((usize, usize), (usize, usize))>;
}

pub struct RandomAi;

impl AiPlayer for RandomAi {
    fn select_move(
        &self,
        game: &dyn BoardGame,
        _color: &Player,
    ) -> Option<((usize, usize), (usize, usize))> {
        let mut all_moves = Vec::new();

        for r in 0..8 {
            for c in 0..8 {
                let moves = game.get_valid_moves((r, c));
                for dest in moves {
                    // Note: We can't easily check 'forced captures' here without
                    // extra trait methods, but Game::apply_move will reject invalid ones anyway.
                    all_moves.push(((r, c), dest));
                }
            }
        }

        let mut rng = rand::thread_rng();
        all_moves.choose(&mut rng).copied()
    }
}

pub struct MinimaxAi {
    depth: u32,
}

impl MinimaxAi {
    /// Create a new minimax AI with the given search depth.
    pub fn new(depth: u32) -> Self {
        Self { depth }
    }

    pub fn evaluate(game: &Game, ai_color: &Player) -> i32 {
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

    fn collect_moves(game: &Game, color: &Player) -> Vec<((usize, usize), (usize, usize))> {
        if let Some(pos) = game.must_jump_from {
            return Self::collect_jump_moves(game, pos)
                .into_iter()
                .map(|dest| (pos, dest))
                .collect();
        }
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

    fn collect_jump_moves(game: &Game, pos: (usize, usize)) -> Vec<(usize, usize)> {
        let Some(piece) = game
            .pieces
            .iter()
            .find(|p| p.row == pos.0 && p.col == pos.1)
        else {
            return Vec::new();
        };
        game.valid_moves(piece)
            .into_iter()
            .filter(|(r, _)| (*r as i32 - pos.0 as i32).abs() == 2)
            .collect()
    }

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
    fn select_move(
        &self,
        game: &dyn BoardGame,
        color: &Player,
    ) -> Option<((usize, usize), (usize, usize))> {
         let game = game.as_any().downcast_ref::<Game>()?;

         let moves = Self::collect_moves(game, color);

        if moves.is_empty() {
            return None;
        }

        // (Copy the rest of your original select_move logic here)
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
