// src/components/game_status.rs

// dependencies
use crate::components::ResetButton;
use crate::domain::{Game, Player};
use crate::state::State;
use yew::prelude::*;
use yewdux::prelude::*;

// turn_indicator component
#[function_component]
pub fn GameStatus() -> Html {
    let (state, dispatch) = use_store::<State>();

    let reset_game = {
        Callback::from( move |_| {
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
            <section>
                <article>
                    <p>{"Game over! "}{winner_text}{" wins!"}</p>
                    <ResetButton id="reset_button" onclick={reset_game} />
                </article>
            </section>
        }
    } else {
        html! {}
    }
}
