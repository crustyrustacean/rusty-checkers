// frontend/src/app.rs

// dependencies
use crate::components::Grid;
use yew::prelude::*;

#[component]
pub fn App() -> Html {
    html! {
        <Grid />
    }
}
