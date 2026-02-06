// frontend/src/app.rs

// dependencies
use crate::components::{GameStatus, Grid, TurnIndicator};
use crate::state::State;
use crate::views::GameView;
use yew::prelude::*;
use yewdux::prelude::*;

#[function_component]
pub fn App() -> Html {
    let (_state, _dispatch) = use_store::<State>();

    html! {
        <GameView>
            <Grid />
            <TurnIndicator />
            <GameStatus />
        </GameView>
    }
}
