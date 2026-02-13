// backend/src/state.rs

// dependencies

#[derive(Clone, Debug)]
pub struct ServerState {}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerState {
    pub fn new() -> Self {
        Self {}
    }
}
