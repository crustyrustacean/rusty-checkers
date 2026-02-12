// frontend/src/components/bracket_view.rs

use checkers_common::tournament::{TournamentState, TournamentView};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct BracketViewProps {
    pub view: TournamentView,
    pub is_host: bool,
    pub on_start: Callback<MouseEvent>,
}

#[component]
pub fn BracketView(props: &BracketViewProps) -> Html {
    let view = &props.view;

    let player_name = |id: &uuid::Uuid| -> String {
        view.players
            .iter()
            .find(|p| p.id == *id)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "TBD".into())
    };

    let lobby_html = if view.state == TournamentState::Lobby {
        html! {
            <div class="bracket-lobby">
                <h3>{"Players in Lobby"}</h3>
                <ul class="player-list">
                    { for view.players.iter().map(|p| html! {
                        <li class="player-item">
                            <span class={if p.connected { "connected" } else { "disconnected" }}>
                                { &p.name }
                            </span>
                        </li>
                    })}
                </ul>
                if props.is_host && view.players.len() >= 2 {
                    <button class="start-button" onclick={props.on_start.clone()}>
                        {"Start Tournament"}
                    </button>
                }
                if !props.is_host {
                    <p class="waiting-text">{"Waiting for host to start..."}</p>
                }
                if view.players.len() < 2 {
                    <p class="waiting-text">{"Need at least 2 players"}</p>
                }
            </div>
        }
    } else {
        html! {}
    };

    let bracket_html = if view.state != TournamentState::Lobby {
        let max_round = view.total_rounds;
        html! {
            <div class="bracket-container">
                { for (0..max_round).map(|round| {
                    let round_matches: Vec<_> = view.matches.iter()
                        .filter(|m| m.round == round)
                        .collect();
                    html! {
                        <div class="bracket-round">
                            <h3 class="round-title">
                                { if round + 1 == max_round {
                                    "Final".to_string()
                                } else {
                                    format!("Round {}", round + 1)
                                }}
                            </h3>
                            { for round_matches.iter().map(|m| {
                                let p1 = m.player1_id.map(|id| player_name(&id)).unwrap_or("BYE".into());
                                let p2 = m.player2_id.map(|id| player_name(&id)).unwrap_or("BYE".into());
                                let winner = m.winner_id;
                                let p1_class = if winner == m.player1_id && winner.is_some() {
                                    "match-player winner"
                                } else if winner.is_some() && winner != m.player1_id {
                                    "match-player eliminated"
                                } else {
                                    "match-player"
                                };
                                let p2_class = if winner == m.player2_id && winner.is_some() {
                                    "match-player winner"
                                } else if winner.is_some() && winner != m.player2_id {
                                    "match-player eliminated"
                                } else {
                                    "match-player"
                                };
                                let status = if m.winner_id.is_some() {
                                    "Complete"
                                } else if m.game_id.is_some() {
                                    "In Progress"
                                } else if m.player1_id.is_some() || m.player2_id.is_some() {
                                    "Waiting"
                                } else {
                                    "TBD"
                                };
                                html! {
                                    <div class="bracket-match">
                                        <div class={p1_class}>{ &p1 }</div>
                                        <div class="match-vs">{"vs"}</div>
                                        <div class={p2_class}>{ &p2 }</div>
                                        <div class="match-status">{ status }</div>
                                    </div>
                                }
                            })}
                        </div>
                    }
                })}
            </div>
        }
    } else {
        html! {}
    };

    let winner_html = if let Some(winner) = &view.winner {
        html! {
            <div class="tournament-winner">
                <h2>{"Tournament Champion"}</h2>
                <p class="champion-name">{ &winner.name }</p>
            </div>
        }
    } else {
        html! {}
    };

    html! {
        <div class="panel bracket-panel">
            <h2>{"Tournament"}</h2>
            <div class="room-code-display">
                <span class="code-label">{"Room Code: "}</span>
                <span class="code-value">{ &view.code }</span>
            </div>
            { lobby_html }
            { bracket_html }
            { winner_html }
        </div>
    }
}
