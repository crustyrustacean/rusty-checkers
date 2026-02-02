// frontend/src/app.rs

// dependencies
use crate::components::Grid;
use crate::state::State;
use yew::prelude::*;
use yewdux::prelude::*;

#[function_component]
pub fn App() -> Html {
    let (_state, _dispatch) = use_store::<State>();

    html! {
        <Grid />
    }
}
