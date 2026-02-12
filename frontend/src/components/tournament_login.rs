// frontend/src/components/tournament_login.rs

use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TournamentLoginProps {
    pub on_create: Callback<String>,
    pub on_join: Callback<(String, String)>,
    pub on_back: Callback<MouseEvent>,
}

#[component]
pub fn TournamentLogin(props: &TournamentLoginProps) -> Html {
    let nickname = use_state(String::new);
    let room_code = use_state(String::new);

    let on_nick_input = {
        let nickname = nickname.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                nickname.set(input.value());
            }
        })
    };

    let on_code_input = {
        let room_code = room_code.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                room_code.set(input.value().to_uppercase());
            }
        })
    };

    let on_create = {
        let nickname = nickname.clone();
        let on_create = props.on_create.clone();
        Callback::from(move |_: MouseEvent| {
            let name = (*nickname).clone();
            if !name.is_empty() {
                on_create.emit(name);
            }
        })
    };

    let on_join = {
        let nickname = nickname.clone();
        let room_code = room_code.clone();
        let on_join = props.on_join.clone();
        Callback::from(move |_: MouseEvent| {
            let name = (*nickname).clone();
            let code = (*room_code).clone();
            if !name.is_empty() && !code.is_empty() {
                on_join.emit((code, name));
            }
        })
    };

    html! {
        <div class="panel tournament-login">
            <h2>{"Tournament Mode"}</h2>
            <div class="form-group">
                <label>{"Nickname"}</label>
                <input type="text"
                    class="form-input"
                    placeholder="Enter your name"
                    value={(*nickname).clone()}
                    oninput={on_nick_input}
                    maxlength="20"
                />
            </div>
            <div class="form-group">
                <label>{"Room Code (to join)"}</label>
                <input type="text"
                    class="form-input room-code-input"
                    placeholder="e.g. AB34"
                    value={(*room_code).clone()}
                    oninput={on_code_input}
                    maxlength="4"
                />
            </div>
            <div class="tournament-buttons">
                <button class="mode-button" onclick={on_create}>{"Create Room"}</button>
                <button class="mode-button" onclick={on_join}>{"Join Room"}</button>
            </div>
            <button class="back-button" onclick={props.on_back.clone()}>{"Back"}</button>
        </div>
    }
}
