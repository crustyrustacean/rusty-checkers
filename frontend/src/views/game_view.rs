// src/view/game_view.rs

// dependencies
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct GameViewProps {
    pub children: Children,
}

#[component]
pub fn GameView(props: &GameViewProps) -> Html {
    html! {
        <div class="game-container">
            { props.children.clone() }
        </div>
    }
}
