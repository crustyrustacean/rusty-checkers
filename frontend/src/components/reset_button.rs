// src/components/rest_button.rs

// dependencies
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ResetButtonProps {
    pub id: String,
    pub onclick: Callback<MouseEvent>,
}

#[function_component]
pub fn ResetButton(props: &ResetButtonProps) -> Html {
    html! {
        <button onclick={props.onclick.clone()}>
            { "Play Again" }
        </button>
    }
}
