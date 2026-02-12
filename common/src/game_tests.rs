// common/src/game_tests.rs

// dependencies
use crate::game::{Game, MoveResult};
use crate::{GamePiece, Player};

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