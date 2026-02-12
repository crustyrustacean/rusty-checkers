// backend/src/tournament/tests.rs

#[cfg(test)]
mod tests {
    use crate::tournament::manager::TournamentManager;
    use checkers_common::tournament::TournamentState;
    use uuid::Uuid;

    fn create_tournament_with_players(count: usize) -> (TournamentManager, Vec<Uuid>) {
        let mut tm = TournamentManager::new("ABCD".into(), "Host".into());
        let host_id = tm.host_id();
        let mut ids = vec![host_id];
        for i in 1..count {
            let id = tm.add_player(format!("Player{}", i)).unwrap();
            ids.push(id);
        }
        (tm, ids)
    }

    // --- Lobby Tests ---

    #[test]
    fn new_tournament_is_in_lobby_state() {
        let tm = TournamentManager::new("ABCD".into(), "Host".into());
        assert_eq!(tm.view().state, TournamentState::Lobby);
        assert_eq!(tm.view().players.len(), 1);
        assert_eq!(tm.view().players[0].name, "Host");
    }

    #[test]
    fn add_player_to_lobby() {
        let mut tm = TournamentManager::new("ABCD".into(), "Host".into());
        let id = tm.add_player("Alice".into()).unwrap();
        assert_eq!(tm.view().players.len(), 2);
        assert!(tm.view().players.iter().any(|p| p.id == id && p.name == "Alice"));
    }

    #[test]
    fn cannot_add_player_after_started() {
        let (mut tm, _ids) = create_tournament_with_players(4);
        tm.start().unwrap();
        let result = tm.add_player("Late".into());
        assert!(result.is_err());
    }

    // --- Bracket Generation: 4 Players ---

    #[test]
    fn bracket_generation_4_players() {
        let (mut tm, ids) = create_tournament_with_players(4);
        let actions = tm.start().unwrap();

        let view = tm.view();
        assert_eq!(view.state, TournamentState::InProgress);
        // 4 players → 2 matches in round 1, total_rounds = 2
        assert_eq!(view.total_rounds, 2);

        let round1: Vec<_> = view.matches.iter().filter(|m| m.round == 0).collect();
        assert_eq!(round1.len(), 2);

        // Every player should appear exactly once
        let mut all_player_ids: Vec<Uuid> = round1
            .iter()
            .flat_map(|m| [m.player1_id.unwrap(), m.player2_id.unwrap()])
            .collect();
        all_player_ids.sort();
        let mut expected = ids.clone();
        expected.sort();
        assert_eq!(all_player_ids, expected);

        // Both matches should be ready (both slots filled)
        for m in &round1 {
            assert!(m.is_ready());
        }

        // start() should return 2 match-start actions
        assert_eq!(actions.len(), 2);
    }

    // --- Bracket Generation: 3 Players (Bye) ---

    #[test]
    fn bracket_generation_3_players_with_bye() {
        let (mut tm, _ids) = create_tournament_with_players(3);
        let actions = tm.start().unwrap();

        let view = tm.view();
        assert_eq!(view.state, TournamentState::InProgress);
        assert_eq!(view.total_rounds, 2);

        let round1: Vec<_> = view.matches.iter().filter(|m| m.round == 0).collect();
        // With 3 players: 1 match + 1 bye → the bye auto-advances
        assert_eq!(round1.len(), 2);

        // One match should be a bye (only one player, already complete with a winner)
        let bye_match = round1.iter().find(|m| m.player2_id.is_none()).unwrap();
        assert!(bye_match.is_complete(), "Bye match should auto-complete");
        assert_eq!(bye_match.winner_id, bye_match.player1_id);

        // The other match should have two real players
        let real_match = round1.iter().find(|m| m.player2_id.is_some()).unwrap();
        assert!(real_match.is_ready());
        assert!(!real_match.is_complete());

        // start() should return 1 action (only the real match starts a game)
        assert_eq!(actions.len(), 1);
    }

    // --- Progression ---

    #[test]
    fn progression_4_players() {
        let (mut tm, _ids) = create_tournament_with_players(4);
        tm.start().unwrap();

        let view = tm.view();
        let round1: Vec<_> = view.matches.iter().filter(|m| m.round == 0).collect();
        let match_a_id = round1[0].id;
        let match_a_winner = round1[0].player1_id.unwrap();
        let match_b_id = round1[1].id;
        let match_b_winner = round1[1].player2_id.unwrap();

        // Record win for match A
        let action_a = tm.advance(match_a_id, match_a_winner).unwrap();
        assert!(action_a.is_none(), "Final match shouldn't start until both semifinalists are known");

        // Record win for match B — this should create the final match
        let action_b = tm.advance(match_b_id, match_b_winner).unwrap();
        assert!(action_b.is_some(), "Both winners known, final match should start");

        let view = tm.view();
        let finals: Vec<_> = view.matches.iter().filter(|m| m.round == 1).collect();
        assert_eq!(finals.len(), 1);
        assert_eq!(finals[0].player1_id, Some(match_a_winner));
        assert_eq!(finals[0].player2_id, Some(match_b_winner));
        assert!(finals[0].is_ready());
    }

    #[test]
    fn progression_completes_tournament() {
        let (mut tm, _ids) = create_tournament_with_players(4);
        tm.start().unwrap();

        let view = tm.view();
        let round1: Vec<_> = view.matches.iter().filter(|m| m.round == 0).collect();
        let match_a_winner = round1[0].player1_id.unwrap();
        let match_b_winner = round1[1].player2_id.unwrap();

        tm.advance(round1[0].id, match_a_winner).unwrap();
        tm.advance(round1[1].id, match_b_winner).unwrap();

        // Now play the final
        let view = tm.view();
        let final_match = view.matches.iter().find(|m| m.round == 1).unwrap();
        let final_winner = match_b_winner;
        let action = tm.advance(final_match.id, final_winner).unwrap();
        assert!(action.is_none(), "No more matches after the final");

        let view = tm.view();
        assert_eq!(view.state, TournamentState::Finished);
        assert!(view.winner.is_some());
        assert_eq!(view.winner.as_ref().unwrap().id, final_winner);
    }

    #[test]
    fn progression_3_players_with_bye() {
        let (mut tm, _ids) = create_tournament_with_players(3);
        tm.start().unwrap();

        let view = tm.view();
        let round1: Vec<_> = view.matches.iter().filter(|m| m.round == 0).collect();

        // Find the real match (with two players)
        let real_match = round1.iter().find(|m| m.player2_id.is_some()).unwrap();
        let real_winner = real_match.player1_id.unwrap();

        // Advancing the real match should trigger the final (bye winner already advanced)
        let action = tm.advance(real_match.id, real_winner).unwrap();
        assert!(action.is_some(), "Final should start after real match completes");

        let view = tm.view();
        let finals: Vec<_> = view.matches.iter().filter(|m| m.round == 1).collect();
        assert_eq!(finals.len(), 1);
        assert!(finals[0].is_ready());
    }

    // --- Error Cases ---

    #[test]
    fn cannot_start_with_one_player() {
        let mut tm = TournamentManager::new("ABCD".into(), "Host".into());
        let result = tm.start();
        assert!(result.is_err());
    }

    #[test]
    fn advance_with_invalid_match_id_returns_error() {
        let (mut tm, _ids) = create_tournament_with_players(4);
        tm.start().unwrap();
        let result = tm.advance(Uuid::new_v4(), Uuid::new_v4());
        assert!(result.is_err());
    }

    #[test]
    fn advance_with_invalid_winner_returns_error() {
        let (mut tm, _ids) = create_tournament_with_players(4);
        tm.start().unwrap();

        let view = tm.view();
        let match_id = view.matches[0].id;
        let bogus_winner = Uuid::new_v4();

        let result = tm.advance(match_id, bogus_winner);
        assert!(result.is_err());
    }

    // --- Game ID Tracking ---

    #[test]
    fn match_start_actions_contain_game_ids() {
        let (mut tm, _ids) = create_tournament_with_players(4);
        let actions = tm.start().unwrap();

        for action in &actions {
            assert!(!action.game_id.is_empty());
            assert!(action.player1_id != Uuid::nil());
            assert!(action.player2_id != Uuid::nil());
        }
    }

    #[test]
    fn find_tournament_match_by_game_id() {
        let (mut tm, _ids) = create_tournament_with_players(4);
        let actions = tm.start().unwrap();

        let game_id = &actions[0].game_id;
        let match_id = tm.find_match_by_game_id(game_id);
        assert!(match_id.is_some());
    }

    // --- Forfeit ---

    #[test]
    fn forfeit_advances_opponent() {
        let (mut tm, _ids) = create_tournament_with_players(4);
        tm.start().unwrap();

        let view = tm.view();
        let first_match = &view.matches[0];
        let forfeiter = first_match.player1_id.unwrap();
        let opponent = first_match.player2_id.unwrap();

        let _action = tm.forfeit(first_match.id, forfeiter).unwrap();
        // Opponent should now be the winner
        let view = tm.view();
        let updated_match = view.matches.iter().find(|m| m.id == first_match.id).unwrap();
        assert_eq!(updated_match.winner_id, Some(opponent));
    }

    // --- Double Advance ---

    #[test]
    fn cannot_advance_already_completed_match() {
        let (mut tm, _ids) = create_tournament_with_players(4);
        tm.start().unwrap();

        let view = tm.view();
        let match_id = view.matches[0].id;
        let winner = view.matches[0].player1_id.unwrap();

        // First advance succeeds
        tm.advance(match_id, winner).unwrap();
        // Second advance on same match should fail
        let result = tm.advance(match_id, winner);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already completed"));
    }

    // --- current_round Tracking ---

    #[test]
    fn current_round_advances_correctly() {
        let (mut tm, _ids) = create_tournament_with_players(4);
        tm.start().unwrap();

        // After start, current_round should be 0 (first round is active)
        let view = tm.view();
        assert_eq!(view.current_round, 0);

        let round1: Vec<_> = view.matches.iter().filter(|m| m.round == 0).collect();
        let match_a_winner = round1[0].player1_id.unwrap();
        let match_b_winner = round1[1].player2_id.unwrap();

        // Complete one match — round 0 still has an incomplete match
        tm.advance(round1[0].id, match_a_winner).unwrap();
        let view = tm.view();
        assert_eq!(view.current_round, 0, "Round 0 still has an incomplete match");

        // Complete the other match — round 0 done, round 1 begins
        tm.advance(round1[1].id, match_b_winner).unwrap();
        let view = tm.view();
        assert_eq!(view.current_round, 1, "All round 0 matches complete, should advance to round 1");

        // Complete the final
        let final_match = view.matches.iter().find(|m| m.round == 1).unwrap();
        tm.advance(final_match.id, match_a_winner).unwrap();
        let view = tm.view();
        assert_eq!(view.state, TournamentState::Finished);
        // When finished, current_round should be the last round
        assert_eq!(view.current_round, 1);
    }

    // --- find_match ---

    #[test]
    fn find_match_returns_correct_match() {
        let (mut tm, _ids) = create_tournament_with_players(4);
        tm.start().unwrap();

        let view = tm.view();
        let expected_id = view.matches[0].id;

        let found = tm.find_match(expected_id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, expected_id);

        // Non-existent match returns None
        assert!(tm.find_match(Uuid::new_v4()).is_none());
    }

    // --- set_code ---

    #[test]
    fn set_code_updates_tournament_code() {
        let mut tm = TournamentManager::new(String::new(), "Host".into());
        assert_eq!(tm.code(), "");
        tm.set_code("XY42".into());
        assert_eq!(tm.code(), "XY42");
        assert_eq!(tm.view().code, "XY42");
    }
}
