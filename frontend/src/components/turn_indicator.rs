// frontend/src/components/turn_indicator.rs

// dependencies
use checkers_common::Player;
use crate::state::State;
use yew::prelude::*;
use yewdux::prelude::*;

#[function_component]
pub fn TurnIndicator() -> Html {
    let (state, _dispatch) = use_store::<State>();
    let current = &state.current_game.current_player;

    html! {
        <div class="panel">
            <h2>{"Current Turn"}</h2>
            <div class={classes!("player-indicator", (current == &Player::Dark).then_some("active"))}>
                <div class="piece-preview dark"></div>
                <span>{"Dark"}</span>
            </div>
            <div class={classes!("player-indicator", (current == &Player::Light).then_some("active"))}>
                <div class="piece-preview light"></div>
                <span>{"Light"}</span>
            </div>
        </div>
    }
}
