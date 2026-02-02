// src/components/grid.rs

// dependencies
use crate::domain::{Game, Player};
use crate::state::State;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use yew::prelude::*;
use yewdux::prelude::*;

// grid component
#[function_component]
pub fn Grid() -> Html {
    let (state, _) = use_store::<State>();
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

            for piece in &state.current_game.pieces {
                if piece.row == row && piece.col == col {
                    log::info!("Found piece at row {}, col {}", row, col);
                    return;
                }
            }
            log::info!("Empty square at row {}, col {}", row, col);
            })
    };

    {
        let canvas_ref = canvas_ref.clone();
        use_effect(move || {
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
                        if (i + j) % 2 == 0 {
                            let colour = "grey";
                            ctx.set_fill_style_str(colour);
                            ctx.fill_rect(x, y, 100.0, 100.0);
                        } else {
                            let colour = "black";
                            ctx.set_fill_style_str(colour);
                            ctx.fill_rect(x, y, 100.0, 100.0);
                        }
                    }
                }

                let game = &state.current_game;
                for piece in &game.pieces {
                    log::info!("Piece at row {}, col {}", piece.row, piece.col);
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
                }
            }
        });
    }

    html! {
        <canvas ref={canvas_ref} onclick={on_click} id="board" height=800 width=800 style="border: 1px solid black">
        </canvas>
    }
}
