// backend/src/state.rs

// dependencies

#[derive(Clone, Debug)]
pub struct AppState {
    pub assets_dir: String,
}

impl AppState {
    pub fn new(assets_dir: String) -> Self {
        Self {
            assets_dir,
        }
    }
}