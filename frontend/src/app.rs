// frontend/src/app.rs

// dependencies
use crate::components::Lobby;
use crate::views::GameView;
use yew::prelude::*;

#[function_component]
pub fn App() -> Html {
    html! {
        <GameView>
            <Lobby />
        </GameView>
    }
}
