// src/components/grid.rs

// dependencies
use crate::state::State;
use checkers_common::Player;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use yew::prelude::*;
use yewdux::prelude::*;

// grid component
#[function_component]
pub fn Grid() -> Html {
    let (state, dispatch) = use_store::<State>();
    let canvas_ref = use_node_ref();

    let on_click = {
        let canvas_ref = canvas_ref.clone();
        let state = state.clone();
        Callback::from(move |event: MouseEvent| {
            let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() else {
                return;
            };

            let rect = canvas.get_bounding_client_rect();
            let x = event.client_x() as f64 - rect.left();
            let y = event.client_y() as f64 - rect.top();

            let col = (x / 100.0) as usize;
            let row = (y / 100.0) as usize;

            if let Some((sel_row, sel_col)) = state.selected_piece
                && state.valid_moves.contains(&(row, col))
            {
                let piece = state
                    .current_game
                    .pieces
                    .iter()
                    .find(|p| p.row == sel_row && p.col == sel_col)
                    .unwrap();
                dispatch.reduce_mut(|state| {
                    let was_jump = (row as i32 - sel_row as i32).abs() == 2;

                    state.current_game.advance(piece, row, col);

                    if was_jump {
                        let captured_row = (sel_row + row) / 2;
                        let captured_col = (sel_col + col) / 2;
                        state.current_game.capture(captured_row, captured_col);
                    }

                    let mut just_kinged = false;
                    if let Some(p) = state
                        .current_game
                        .pieces
                        .iter_mut()
                        .find(|p| p.row == row && p.col == col)
                    {
                        if (p.row == 0 && p.owner == Player::Light)
                            || (p.row == 7 && p.owner == Player::Dark)
                        {
                            if !p.is_kinged {
                                p.is_kinged = true;
                                just_kinged = true;
                            }
                        }
                    }

                    let can_jump_again = if was_jump && !just_kinged {
                        state
                            .current_game
                            .pieces
                            .iter()
                            .find(|p| p.row == row && p.col == col)
                            .map(|p| state.current_game.has_available_jumps(p))
                            .unwrap_or(false)
                    } else {
                        false
                    };

                    if can_jump_again {
                        // Multi-jump: Keep the piece selected and filter moves to only jumps
                        state.selected_piece = Some((row, col));

                        // We find the piece one last time to get its valid moves
                        if let Some(p) = state
                            .current_game
                            .pieces
                            .iter()
                            .find(|p| p.row == row && p.col == col)
                        {
                            state.valid_moves = state
                                .current_game
                                .valid_moves(p)
                                .into_iter()
                                .filter(|(r, _)| (*r as i32 - row as i32).abs() == 2)
                                .collect();
                        }
                    } else {
                        state.current_game.switch_turn();

                        let current_player = state.current_game.current_player.clone();
                        let next_player_has_moves = state
                            .current_game
                            .pieces
                            .iter()
                            .filter(|p| p.owner == current_player)
                            .any(|p| !state.current_game.valid_moves(p).is_empty());

                        if !next_player_has_moves {
                            state.current_game.winner = Some(match current_player {
                                Player::Dark => Player::Light,
                                Player::Light => Player::Dark,
                            });
                        }
                        state.selected_piece = None;
                        state.valid_moves = vec![];
                    }
                });
            }

            if state.selected_piece.is_none() {
                for piece in &state.current_game.pieces {
                    if piece.row == row
                        && piece.col == col
                        && state.current_game.current_player == piece.owner
                    {
                        log::info!("Found piece at row {}, col {}", row, col);
                        dispatch.reduce_mut(|state| state.selected_piece = Some((row, col)));
                        let moves = &state.current_game.valid_moves(piece);
                        let captures_exist = state
                            .current_game
                            .check_captures_moves(&state.current_game.current_player);
                        let filtered_moves = if captures_exist {
                            moves
                                .iter()
                                .filter(|(dest_row, _dest_col)| {
                                    (*dest_row as i32 - row as i32).abs() == 2
                                })
                                .cloned()
                                .collect()
                        } else {
                            moves.to_vec()
                        };
                        for (dest_row, dest_col) in moves.iter() {
                            log::info!("Valid move to: row {}, col {}", dest_row, dest_col);
                        }
                        dispatch.reduce_mut(|state| state.valid_moves = filtered_moves);
                        return;
                    }
                }
            }

            log::info!("Empty square at row {}, col {}", row, col);
            dispatch.reduce_mut(|state| {
                state.selected_piece = None;
                state.valid_moves = vec![];
            });
        })
    };

    {
        let canvas_ref = canvas_ref.clone();
        use_effect_with(state.clone(), move |state| {
            log::info!("Effect running");
            let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() else {
                log::error!("Failed to get a canvas element to draw the game board on.");
                return;
            };

            let Ok(Some(context)) = canvas.get_context("2d") else {
                log::error!("Failed to get 2d context.");
                return;
            };

            let Ok(ctx) = context.dyn_into::<CanvasRenderingContext2d>() else {
                log::error!("Failed to cast to CanvasRenderingContext2d");
                return;
            };

            {
                for i in 0..8 {
                    for j in 0..8 {
                        let x = (i * 100) as f64;
                        let y = (j * 100) as f64;

                        let colour = if state.selected_piece == Some((j, i)) {
                            "yellow"
                        } else if state
                            .valid_moves
                            .iter()
                            .any(|(row, col)| *row == j && *col == i)
                        {
                            "green"
                        } else if (i + j) % 2 == 0 {
                            "grey"
                        } else {
                            "black"
                        };

                        ctx.set_fill_style_str(colour);
                        ctx.fill_rect(x, y, 100.0, 100.0);
                    }
                }

                let game = &state.current_game;
                for piece in &game.pieces {
                    if piece.owner == Player::Dark {
                        ctx.set_fill_style_str("red");
                    } else {
                        ctx.set_fill_style_str("white");
                    }

                    let row = piece.row;
                    let col = piece.col;

                    let center_x = (col * 100 + 50) as f64;
                    let center_y = (row * 100 + 50) as f64;
                    let radius = 40.0;

                    ctx.begin_path();
                    let _ = ctx.arc(center_x, center_y, radius, 0.0, 2.0 * std::f64::consts::PI);
                    ctx.fill();

                    if piece.is_kinged {
                        ctx.set_font("30px Arial");
                        ctx.set_fill_style_str("gold");
                        let _ = ctx.fill_text("K", center_x - 10.0, center_y + 10.0);
                    }
                }
            }
        });
    }

    html! {
        <canvas ref={canvas_ref} onclick={on_click} id="board" height=800 width=800 style="border: 1px solid black">
        </canvas>
    }
}
