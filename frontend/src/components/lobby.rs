
// frontend/src/components/lobby.rs

// dependencies
use checkers_common::{ClientMessage, ServerMessage, Player, Game};
use crate::components::{MpGrid, RulesCard};
use crate::websocket::GameSocket;
use std::rc::Rc;
use yew::prelude::*;

pub enum LobbyState {
    Connecting,
    WaitingForOpponent,
    Playing { my_color: Player, game: Game },
}

#[function_component]
pub fn Lobby() -> Html {
    let state = use_state(|| LobbyState::Connecting);
    let socket: UseStateHandle<Option<Rc<GameSocket>>> = use_state(|| None);

    {
        let state = state.clone();
        let socket = socket.clone();
        use_effect_with((), move |_| {
            log::info!("Effect running - attempting to connect");

            let on_message = {
                let state = state.clone();
                Callback::from(move |msg: ServerMessage| {
                    log::info!("Received: {:?}", msg);
                    match msg {
                        ServerMessage::GamePending => {
                            state.set(LobbyState::WaitingForOpponent);
                        }
                        ServerMessage::GameStarted(player) => {
                            log::info!("Game started! I am {:?}", player);
                            state.set(LobbyState::Playing {
                                my_color: player,
                                game: Game::new(),
                            });
                        }
                        ServerMessage::GameState(game) => {
                            if let LobbyState::Playing { my_color, .. } = &*state {
                                state.set(LobbyState::Playing {
                                    my_color: my_color.clone(),
                                    game,
                                });
                            }
                        }
                        ServerMessage::OpponentDisconnected => {
                            log::warn!("Opponent disconnected");
                        }
                        ServerMessage::Error(e) => {
                            log::error!("Server error: {}", e);
                        }
                    }
                })
            };

            match GameSocket::connect(on_message) {
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
        LobbyState::WaitingForOpponent => html! {
            <div class="panel">
                <h2>{"Waiting for opponent..."}</h2>
            </div>
        },
        LobbyState::Playing { my_color, game } => {
    let on_move = {
        let socket = socket.clone();
        Callback::from(move |(start, end): ((usize, usize), (usize, usize))| {
            if let Some(gs) = socket.as_ref() {
                gs.send(ClientMessage::MakeMove { start, end });
            }
        })
    };

    html! {
        <>
            <div class="board-wrapper">
                <MpGrid game={game.clone()} my_color={my_color.clone()} on_move={on_move} />
            </div>
            <div class="sidebar">
                <div class="panel">
                    <h2>{format!("You are {:?}", my_color)}</h2>
                    <p>{format!("Current turn: {:?}", game.current_player)}</p>
                    if game.current_player == *my_color {
                        <p class="your-turn">{"Your turn!"}</p>
                    }
                </div>
                <RulesCard />
            </div>
        </>
    }
}
    }
}