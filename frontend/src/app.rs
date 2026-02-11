// frontend/src/app.rs

// dependencies
use crate::components::{Landing, Lobby};
use crate::views::GameView;
use yew::prelude::*;

#[function_component]
pub fn App() -> Html {
    let show_lobby = use_state(|| false);

    let on_play = {
        let show_lobby = show_lobby.clone();
        Callback::from(move |_: MouseEvent| {
            show_lobby.set(true);
        })
    };

    if *show_lobby {
        html! {
            <GameView>
                <Lobby />
            </GameView>
        }
    } else {
        html! {
            <Landing on_play={on_play} />
        }
    }
}
