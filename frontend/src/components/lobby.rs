// frontend/src/components/lobby.rs

// dependencies
use crate::components::{BracketView, MpGrid, RulesCard, TournamentLogin};
use crate::websocket::GameSocket;
use checkers_common::tournament::TournamentView;
use checkers_common::{AiDifficulty, ClientMessage, Game, Player, ServerMessage};
use std::cell::RefCell;
use std::rc::Rc;
use yew::prelude::*;

pub enum LobbyState {
    Connecting,
    SelectMode,
    TournamentLogin,
    TournamentLobby {
        view: TournamentView,
        is_host: bool,
    },
    TournamentBracket {
        view: TournamentView,
        is_host: bool,
    },
    WaitingForOpponent,
    Playing {
        my_color: Player,
        game: Game,
        tournament_view: Option<TournamentView>,
        is_tournament: bool,
    },
}

#[component]
pub fn Lobby() -> Html {
    let state = use_state(|| LobbyState::Connecting);
    let socket: UseStateHandle<Option<Rc<GameSocket>>> = use_state(|| None);
    let my_color_ref = use_memo((), |_| RefCell::new(None::<Player>));
    let is_host_ref = use_memo((), |_| RefCell::new(false));
    let tournament_view_ref = use_memo((), |_| RefCell::new(None::<TournamentView>));
    let in_tournament_ref = use_memo((), |_| RefCell::new(false));

    {
        let state = state.clone();
        let socket = socket.clone();
        let my_color_ref = my_color_ref.clone();
        let is_host_ref = is_host_ref.clone();
        let tournament_view_ref = tournament_view_ref.clone();
        let in_tournament_ref = in_tournament_ref.clone();
        use_effect_with((), move |_| {
            log::info!("Effect running - attempting to connect");

            let on_open = {
                let state = state.clone();
                Callback::from(move |_: ()| {
                    state.set(LobbyState::SelectMode);
                })
            };

            let on_message = {
                let state = state.clone();
                let my_color_ref = my_color_ref.clone();
                let is_host_ref = is_host_ref.clone();
                let tournament_view_ref = tournament_view_ref.clone();
                let in_tournament_ref = in_tournament_ref.clone();
                Callback::from(move |msg: ServerMessage| {
                    log::info!("Received: {:?}", msg);
                    match msg {
                        ServerMessage::GamePending => {
                            state.set(LobbyState::WaitingForOpponent);
                        }
                        ServerMessage::GameStarted(player) => {
                            log::info!("Game started! I am {:?}", player);
                            *my_color_ref.borrow_mut() = Some(player.clone());
                            let is_tournament = *in_tournament_ref.borrow();
                            state.set(LobbyState::Playing {
                                my_color: player,
                                game: Game::new(),
                                tournament_view: tournament_view_ref.borrow().clone(),
                                is_tournament,
                            });
                        }
                        ServerMessage::GameState(json_value) => {
                            match serde_json::from_value::<Game>(json_value) {
                                Ok(game) => {
                                    let color = my_color_ref.borrow().clone();
                                    log::info!("GameState received & parsed. Color: {:?}", color);
                                    if let Some(color) = color {
                                        let is_tournament = *in_tournament_ref.borrow();
                                        state.set(LobbyState::Playing {
                                            my_color: color,
                                            game,
                                            tournament_view: tournament_view_ref.borrow().clone(),
                                            is_tournament,
                                        });
                                    } else {
                                        log::warn!("GameState received but no color set yet");
                                    }
                                }
                                Err(e) => {
                                    log::error!("Failed to parse GameState JSON: {}", e);
                                }
                            }
                        }
                        ServerMessage::OpponentDisconnected => {
                            log::warn!("Opponent disconnected");
                        }
                        ServerMessage::Error(e) => {
                            log::error!("Server error: {}", e);
                        }
                        ServerMessage::TournamentCreated { code } => {
                            log::info!("Tournament created: {}", code);
                            *is_host_ref.borrow_mut() = true;
                            *in_tournament_ref.borrow_mut() = true;
                        }
                        ServerMessage::TournamentUpdate(view) => {
                            log::info!("Tournament update: {:?}", view.state);
                            *tournament_view_ref.borrow_mut() = Some(view.clone());
                            let is_host = *is_host_ref.borrow();

                            match view.state {
                                checkers_common::tournament::TournamentState::Lobby => {
                                    state.set(LobbyState::TournamentLobby {
                                        view,
                                        is_host,
                                    });
                                }
                                checkers_common::tournament::TournamentState::InProgress
                                | checkers_common::tournament::TournamentState::Finished => {
                                    // Only switch to bracket if we're not currently playing
                                    let color = my_color_ref.borrow().clone();
                                    if color.is_none() {
                                        state.set(LobbyState::TournamentBracket {
                                            view,
                                            is_host,
                                        });
                                    }
                                    // If we have a color, we're in a game - the state
                                    // will be updated when the game ends
                                }
                            }
                        }
                        ServerMessage::MatchStart { game_id, opponent_name } => {
                            log::info!(
                                "Match starting! game_id: {}, opponent: {}",
                                game_id,
                                opponent_name
                            );
                            *in_tournament_ref.borrow_mut() = true;
                            // The GameStarted message that follows will
                            // transition us to Playing state
                        }
                    }
                })
            };

            match GameSocket::connect(on_open, on_message) {
                Ok(gs) => {
                    log::info!("GameSocket created successfully");
                    socket.set(Some(Rc::new(gs)));
                }
                Err(e) => {
                    log::error!("Failed to connect: {:?}", e);
                }
            }
        });
    }

    match &*state {
        LobbyState::Connecting => html! {
            <div class="panel">
                <h2>{"Connecting..."}</h2>
            </div>
        },
        LobbyState::SelectMode => {
            let play_human = {
                let socket = socket.clone();
                Callback::from(move |_: MouseEvent| {
                    if let Some(gs) = socket.as_ref() {
                        gs.send(ClientMessage::JoinGame);
                    }
                })
            };

            let play_ai_easy = {
                let socket = socket.clone();
                Callback::from(move |_: MouseEvent| {
                    if let Some(gs) = socket.as_ref() {
                        gs.send(ClientMessage::PlayVsAI {
                            difficulty: AiDifficulty::Easy,
                        });
                    }
                })
            };

            let play_ai_medium = {
                let socket = socket.clone();
                Callback::from(move |_: MouseEvent| {
                    if let Some(gs) = socket.as_ref() {
                        gs.send(ClientMessage::PlayVsAI {
                            difficulty: AiDifficulty::Medium,
                        });
                    }
                })
            };

            let play_ai_hard = {
                let socket = socket.clone();
                Callback::from(move |_: MouseEvent| {
                    if let Some(gs) = socket.as_ref() {
                        gs.send(ClientMessage::PlayVsAI {
                            difficulty: AiDifficulty::Hard,
                        });
                    }
                })
            };

            let open_tournament = {
                let state = state.clone();
                Callback::from(move |_: MouseEvent| {
                    state.set(LobbyState::TournamentLogin);
                })
            };

            html! {
                <div class="panel">
                    <h2>{"Select Game Mode"}</h2>
                    <div class="mode-buttons">
                        <button class="mode-button" onclick={play_human}>{"Play vs Human"}</button>
                        <button class="mode-button" onclick={play_ai_easy}>{"Play vs AI (Easy)"}</button>
                        <button class="mode-button" onclick={play_ai_medium}>{"Play vs AI (Medium)"}</button>
                        <button class="mode-button" onclick={play_ai_hard}>{"Play vs AI (Hard)"}</button>
                        <button class="mode-button tournament-btn" onclick={open_tournament}>{"Tournament"}</button>
                    </div>
                </div>
            }
        }
        LobbyState::TournamentLogin => {
            let on_create = {
                let socket = socket.clone();
                Callback::from(move |name: String| {
                    if let Some(gs) = socket.as_ref() {
                        gs.send(ClientMessage::CreateTournament { host_name: name });
                    }
                })
            };

            let on_join = {
                let socket = socket.clone();
                Callback::from(move |(code, name): (String, String)| {
                    if let Some(gs) = socket.as_ref() {
                        gs.send(ClientMessage::JoinTournament { code, name });
                    }
                })
            };

            let on_back = {
                let state = state.clone();
                Callback::from(move |_: MouseEvent| {
                    state.set(LobbyState::SelectMode);
                })
            };

            html! {
                <TournamentLogin {on_create} {on_join} {on_back} />
            }
        }
        LobbyState::TournamentLobby { view, is_host } => {
            let on_start = {
                let socket = socket.clone();
                Callback::from(move |_: MouseEvent| {
                    if let Some(gs) = socket.as_ref() {
                        gs.send(ClientMessage::StartTournament);
                    }
                })
            };

            html! {
                <BracketView view={view.clone()} is_host={*is_host} {on_start} />
            }
        }
        LobbyState::TournamentBracket { view, is_host } => {
            let on_start = {
                Callback::from(|_: MouseEvent| {})
            };

            html! {
                <BracketView view={view.clone()} is_host={*is_host} {on_start} />
            }
        }
        LobbyState::WaitingForOpponent => html! {
            <div class="panel">
                <h2>{"Waiting for opponent..."}</h2>
            </div>
        },
        LobbyState::Playing {
            my_color,
            game,
            tournament_view,
            is_tournament,
        } => {
            let on_move = {
                let socket = socket.clone();
                Callback::from(move |(start, end): ((usize, usize), (usize, usize))| {
                    log::info!("on_move callback fired: {:?} -> {:?}", start, end);
                    if let Some(gs) = socket.as_ref() {
                        log::info!("Sending via socket");
                        gs.send(ClientMessage::MakeMove { start, end });
                    } else {
                        log::error!("No socket available!");
                    }
                })
            };

            let play_again = {
                let socket = socket.clone();
                Callback::from(move |_: MouseEvent| {
                    if let Some(gs) = socket.as_ref() {
                        gs.send(ClientMessage::PlayAgain);
                    }
                })
            };

            let back_to_bracket = {
                let state = state.clone();
                let tv = tournament_view.clone();
                let is_host = *is_tournament;
                let my_color_ref = my_color_ref.clone();
                Callback::from(move |_: MouseEvent| {
                    // Clear color so we re-enter bracket view
                    *my_color_ref.borrow_mut() = None;
                    if let Some(view) = &tv {
                        state.set(LobbyState::TournamentBracket {
                            view: view.clone(),
                            is_host,
                        });
                    }
                })
            };

            let winner_text = if game.winner.as_ref() == Some(my_color) {
                "You win!"
            } else {
                "You lose!"
            };

            html! {
                <>
                    <div class="board-wrapper">
                        <MpGrid game={game.clone()} my_color={my_color.clone()} on_move={on_move} />
                    </div>
                    <div class="sidebar">
                        if game.winner.is_some() {
                            <div class="winner-banner">
                                <h2>{winner_text}</h2>
                                <p>{format!("{:?} wins the game", game.winner.as_ref().unwrap())}</p>
                                if *is_tournament {
                                    <button onclick={back_to_bracket}>{"Back to Bracket"}</button>
                                } else {
                                    <button onclick={play_again}>{"Play Again"}</button>
                                }
                            </div>
                        } else {
                            <div class="panel">
                                <h2>{format!("You are {:?}", my_color)}</h2>
                                <p>{format!("Current turn: {:?}", game.current_player)}</p>
                                if game.current_player == *my_color {
                                    <p class="your-turn">{"Your turn!"}</p>
                                }
                            </div>
                        }
                        <RulesCard />
                    </div>
                </>
            }
        }
    }
}
