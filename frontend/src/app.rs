// frontend/src/app.rs

// dependencies
use crate::components::{GameStatus, Grid, RulesCard, TurnIndicator};
use crate::state::State;
use crate::views::GameView;
use yew::prelude::*;
use yewdux::prelude::*;

#[function_component]
pub fn App() -> Html {
    let (_state, _dispatch) = use_store::<State>();

    html! {
        <GameView>
            <div class="board-wrapper">
                <Grid />
            </div>
            <div class="sidebar">
                <TurnIndicator />
                <GameStatus />
                <RulesCard />
            </div>
        </GameView>
    }
}
