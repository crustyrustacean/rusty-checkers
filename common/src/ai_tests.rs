// common/src/ai_tests.rs

// dependencies
use crate::game::Game;
use crate::{AiPlayer, GamePiece, MinimaxAi, Player, RandomAi};

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

#[test]
fn test_minimax_respects_must_jump_from_in_multi_jump() {
    // Setup: Dark has just jumped to (4, 2).
    // It has a forced continuation jump over Light at (5, 3) to (6, 4).
    // Dark ALSO has a piece at (2, 0) which could normally move to (3, 1).
    let mut game = game_with_pieces(
        vec![
            GamePiece::new(Player::Dark, 4, 2), // The jumping piece (must move)
            GamePiece::new(Player::Dark, 2, 0), // The distraction (should be ignored)
            GamePiece::new(Player::Light, 5, 3), // The victim
        ],
        Player::Dark,
    );

    // Manually simulate the "mid-multi-jump" state
    game.must_jump_from = Some((4, 2));

    let ai = MinimaxAi::new(2);
    let result = ai.select_move(&game, &Player::Dark);

    assert!(result.is_some(), "AI should find the forced move");
    let ((start_row, start_col), (end_row, end_col)) = result.unwrap();

    // ASSERTION 1: The move MUST originate from the piece locked in `must_jump_from`
    assert_eq!(
        (start_row, start_col),
        (4, 2),
        "AI moved the wrong piece! It ignored must_jump_from."
    );

    // ASSERTION 2: The move MUST be the jump to (6, 4)
    assert_eq!(
        (end_row, end_col),
        (6, 4),
        "AI did not take the forced jump."
    );
}
