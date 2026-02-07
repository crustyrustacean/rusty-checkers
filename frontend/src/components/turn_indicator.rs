// src/components/grid.rs

// dependencies
use crate::domain::Player;
use crate::state::State;
use yew::prelude::*;
use yewdux::prelude::*;

// turn_indicator component
#[function_component]
pub fn TurnIndicator() -> Html {
    let (state, _dispatch) = use_store::<State>();

    html! {
        <section>
            <div style={if state.current_game.current_player == Player::Dark {"background: yellow"} else { "" }}>
                { "Player: Dark"}
            </div>
            <div style={if state.current_game.current_player == Player::Light {"background: yellow"} else { "" }}>
                { "Player: Light"}
            </div>
        </section>
    }
}
