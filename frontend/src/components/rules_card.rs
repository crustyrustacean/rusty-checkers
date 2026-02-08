use yew::prelude::*;

#[function_component]
pub fn RulesCard() -> Html {
    html! {
        <div class="panel">
            <h2>{ "Rules" }</h2>
            <p class="rules-subtitle">{ "American Rules" }</p>
            <ul class="rules-list">
                <li>{ "Dark (red) moves first." }</li>
                <li>{ "If a jump is available, it must be taken." }</li>
                <li>{ "Captures occur by jumping over an opponent's piece. Multiple jumps are allowed in a single turn." }</li>
                <li>{ "Reaching the opponent's back row promotes a piece to a king, allowing it to move backward." }</li>
                <li>{ "Win by capturing all pieces or leaving the opponent with no legal moves." }</li>
            </ul>
        </div>
    }
}