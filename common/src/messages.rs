// common/src/messages.rs

// dependencies
use crate::player::Player;
use crate::tournament::TournamentView;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum AiDifficulty {
    Easy,
    Medium,
    Hard,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ClientMessage {
    JoinGame,
    PlayVsAI {
        difficulty: AiDifficulty,
    },
    MakeMove {
        start: (usize, usize),
        end: (usize, usize),
    },
    PlayAgain,
    CreateTournament {
        host_name: String,
    },
    JoinTournament {
        code: String,
        name: String,
    },
    StartTournament,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ServerMessage {
    GamePending,
    GameStarted(Player),
    GameState(serde_json::Value),
    OpponentDisconnected,
    Error(String),
    TournamentCreated {
        code: String,
    },
    TournamentUpdate(TournamentView),
    MatchStart {
        game_id: String,
        opponent_name: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Player;
    use crate::tournament::{Match, TournamentPlayer, TournamentState};

    #[test]
    fn test_client_message_serialization() {
        // 1. Test JoinGame (Unit Variant)
        let msg = ClientMessage::JoinGame;
        let json = serde_json::to_string(&msg).unwrap();
        // By default, Serde serializes unit variants as simple strings
        assert_eq!(json, "\"JoinGame\"");

        // 2. Test MakeMove (Struct Variant)
        let msg = ClientMessage::MakeMove {
            start: (2, 0),
            end: (3, 1),
        };
        let json = serde_json::to_string(&msg).unwrap();
        // Verifies the "wire format" is exactly what we expect:
        // {"MakeMove":{"start":[2,0],"end":[3,1]}}
        assert_eq!(json, r#"{"MakeMove":{"start":[2,0],"end":[3,1]}}"#);
    }

    #[test]
    fn test_server_message_deserialization() {
        // Simulate receiving a JSON string from the server
        let input_json = r#"{"GameStarted":"Dark"}"#;

        // Attempt to parse it back into Rust
        let msg: ServerMessage = serde_json::from_str(input_json).unwrap();

        // Verify we got the correct enum variant
        match msg {
            ServerMessage::GameStarted(player) => assert_eq!(player, Player::Dark),
            _ => panic!("Expected GameStarted(Dark), got {:?}", msg),
        }
    }

    #[test]
    fn test_create_tournament_roundtrip() {
        let msg = ClientMessage::CreateTournament {
            host_name: "Alice".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
        match parsed {
            ClientMessage::CreateTournament { host_name } => {
                assert_eq!(host_name, "Alice");
            }
            _ => panic!("Expected CreateTournament, got {:?}", parsed),
        }
    }

    #[test]
    fn test_join_tournament_roundtrip() {
        let msg = ClientMessage::JoinTournament {
            code: "ABCD".into(),
            name: "Bob".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
        match parsed {
            ClientMessage::JoinTournament { code, name } => {
                assert_eq!(code, "ABCD");
                assert_eq!(name, "Bob");
            }
            _ => panic!("Expected JoinTournament, got {:?}", parsed),
        }
    }

    #[test]
    fn test_start_tournament_roundtrip() {
        let msg = ClientMessage::StartTournament;
        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(json, "\"StartTournament\"");
        let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, ClientMessage::StartTournament));
    }

    #[test]
    fn test_tournament_created_roundtrip() {
        let msg = ServerMessage::TournamentCreated {
            code: "XY12".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ServerMessage = serde_json::from_str(&json).unwrap();
        match parsed {
            ServerMessage::TournamentCreated { code } => assert_eq!(code, "XY12"),
            _ => panic!("Expected TournamentCreated, got {:?}", parsed),
        }
    }

    #[test]
    fn test_tournament_view_roundtrip() {
        let player = TournamentPlayer::new("Alice".into());
        let view = TournamentView {
            code: "ABCD".into(),
            state: TournamentState::Lobby,
            players: vec![player.clone()],
            matches: vec![Match::new(0, 0)],
            current_round: 0,
            total_rounds: 2,
            winner: None,
        };

        let msg = ServerMessage::TournamentUpdate(view.clone());
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ServerMessage = serde_json::from_str(&json).unwrap();

        match parsed {
            ServerMessage::TournamentUpdate(parsed_view) => {
                assert_eq!(parsed_view.code, "ABCD");
                assert_eq!(parsed_view.state, TournamentState::Lobby);
                assert_eq!(parsed_view.players.len(), 1);
                assert_eq!(parsed_view.players[0].name, "Alice");
                assert_eq!(parsed_view.matches.len(), 1);
                assert_eq!(parsed_view.total_rounds, 2);
                assert!(parsed_view.winner.is_none());
            }
            _ => panic!("Expected TournamentUpdate, got {:?}", parsed),
        }
    }

    #[test]
    fn test_match_start_roundtrip() {
        let msg = ServerMessage::MatchStart {
            game_id: "game-123".into(),
            opponent_name: "Bob".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ServerMessage = serde_json::from_str(&json).unwrap();
        match parsed {
            ServerMessage::MatchStart {
                game_id,
                opponent_name,
            } => {
                assert_eq!(game_id, "game-123");
                assert_eq!(opponent_name, "Bob");
            }
            _ => panic!("Expected MatchStart, got {:?}", parsed),
        }
    }
}
