// src/components/game_status.rs

// dependencies
use crate::domain::Player;
use crate::state::State;
use yew::prelude::*;
use yewdux::prelude::*;

// turn_indicator component
#[function_component]
pub fn GameStatus() -> Html {
    let (state, _) = use_store::<State>();

    if let Some(winner) = &state.current_game.winner {
        let winner_text = match winner {
            Player::Dark => "Dark",
            Player::Light => "Light",
        };

        html! {
            <section>
                <article>
                    <p>{"Game over! "}{winner_text}{" wins!"}</p>
                </article>
            </section>
        }
    } else {
        html! {}
    }
}
