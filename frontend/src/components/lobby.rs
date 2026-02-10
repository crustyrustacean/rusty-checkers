// frontend/src/components/lobby.rs

// dependencies
use crate::components::{MpGrid, RulesCard};
use crate::websocket::GameSocket;
use checkers_common::{ClientMessage, Game, Player, ServerMessage};
use std::cell::RefCell;
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
    let my_color_ref = use_memo((), |_| RefCell::new(None::<Player>));

    {
        let state = state.clone();
        let socket = socket.clone();
        let my_color_ref = my_color_ref.clone();
        use_effect_with((), move |_| {
            log::info!("Effect running - attempting to connect");

            let on_message = {
                let state = state.clone();
                let my_color_ref = my_color_ref.clone();
                Callback::from(move |msg: ServerMessage| {
                    log::info!("Received: {:?}", msg);
                    match msg {
                        ServerMessage::GamePending => {
                            state.set(LobbyState::WaitingForOpponent);
                        }
                        ServerMessage::GameStarted(player) => {
                            log::info!("Game started! I am {:?}", player);
                            *my_color_ref.borrow_mut() = Some(player.clone());
                            state.set(LobbyState::Playing {
                                my_color: player,
                                game: Game::new(),
                            });
                        }
                        ServerMessage::GameState(game) => {
                            let color = my_color_ref.borrow().clone();
                            log::info!("GameState received, my_color_ref: {:?}", color);
                            if let Some(color) = color {
                                log::info!("Updating game state with color: {:?}", color);
                                state.set(LobbyState::Playing {
                                    my_color: color,
                                    game,
                                });
                            } else {
                                log::warn!("GameState received but no color set yet");
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
                                <button onclick={play_again}>{"Play Again"}</button>
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