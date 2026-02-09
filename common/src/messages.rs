// common/src/messages.rs

// dependencies
use crate::game::Game;
use crate::player::Player;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ClientMessage {
    JoinGame,
    MakeMove {
        start: (usize, usize),
        end: (usize, usize),
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ServerMessage {
    GamePending,
    GameStarted(Player),
    GameState(Game),
    OpponentDisconnected,
    Error(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Player;

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
}
