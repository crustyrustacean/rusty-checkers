// src/view/game_view.rs

// dependencies
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct GameViewProps {
    pub children: Children,
}

#[function_component]
pub fn GameView(props: &GameViewProps) -> Html {
    html! {
        <div style="display: flex; flex-direction: row,">
            { props.children.clone() }
        </div>
    }
}
