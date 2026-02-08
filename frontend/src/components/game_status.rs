// src/components/game_status.rs

// dependencies
use checkers_common::{Game, Player};
use crate::components::ResetButton;
use crate::state::State;
use yew::prelude::*;
use yewdux::prelude::*;

#[function_component]
pub fn GameStatus() -> Html {
    let (state, dispatch) = use_store::<State>();

    let reset_game = {
        Callback::from(move |_| {
            dispatch.reduce_mut(|state| {
                state.current_game = Game::new();
            })
        })
    };

    if let Some(winner) = &state.current_game.winner {
        let winner_text = match winner {
            Player::Dark => "Dark",
            Player::Light => "Light",
        };

        html! {
            <div class="winner-banner">
                <h2>{winner_text}{" wins!"}</h2>
                <ResetButton id="reset_button" onclick={reset_game} />
            </div>
        }
    } else {
        html! {}
    }
}
