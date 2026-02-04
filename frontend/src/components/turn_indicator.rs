// src/components/grid.rs

// dependencies
use crate::state::State;
use yew::prelude::*;
use yewdux::prelude::*;

// turn_indicator component
#[function_component]
pub fn TurnIndicator() -> Html {
    let (_state, _dispatch) = use_store::<State>();
    
    html! {
        <section>
            <div>
                { "Player: Dark"}
            </div>
            <div>
                { "Player: Light"}
            </div>
        </section>
    }
}