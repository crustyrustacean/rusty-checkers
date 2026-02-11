// frontend/src/components/mp_grid.rs

// dependencies
use checkers_common::{Game, Player};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct MpGridProps {
    pub game: Game,
    pub my_color: Player,
    pub on_move: Callback<((usize, usize), (usize, usize))>,
}

#[component]
pub fn MpGrid(props: &MpGridProps) -> Html {
    let canvas_ref = use_node_ref();
    let selected = use_state(|| None::<(usize, usize)>);
    let valid_moves = use_state(Vec::<(usize, usize)>::new);
    let dark_img: UseStateHandle<Option<HtmlImageElement>> = use_state(|| None);
    let light_img: UseStateHandle<Option<HtmlImageElement>> = use_state(|| None);

    // Load images once on mount
    {
        let dark_img = dark_img.clone();
        let light_img = light_img.clone();
        use_effect_with((), move |_| {
            // Load dark piece
            let img = HtmlImageElement::new().unwrap();
            let img_clone = img.clone();
            let dark_img_clone = dark_img.clone();
            let onload = Closure::once(move || {
                dark_img_clone.set(Some(img_clone));
            });
            img.set_onload(Some(onload.as_ref().unchecked_ref()));
            onload.forget();
            img.set_src("/public/assets/dark_piece.svg");

            // Load light piece
            let img = HtmlImageElement::new().unwrap();
            let img_clone = img.clone();
            let light_img_clone = light_img.clone();
            let onload = Closure::once(move || {
                light_img_clone.set(Some(img_clone));
            });
            img.set_onload(Some(onload.as_ref().unchecked_ref()));
            onload.forget();
            img.set_src("/public/assets/light_piece.svg");
        });
    }

    let on_click = {
        let canvas_ref = canvas_ref.clone();
        let game = props.game.clone();
        let my_color = props.my_color.clone();
        let on_move = props.on_move.clone();
        let selected = selected.clone();
        let valid_moves = valid_moves.clone();

        Callback::from(move |event: MouseEvent| {
            if game.current_player != my_color {
                return;
            }

            let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() else {
                return;
            };

            let rect = canvas.get_bounding_client_rect();
            let x = event.client_x() as f64 - rect.left();
            let y = event.client_y() as f64 - rect.top();

            let col = (x / 100.0) as usize;
            let row = (y / 100.0) as usize;

            if let Some((sel_row, sel_col)) = *selected
                && valid_moves.contains(&(row, col))
            {
                log::info!(
                    "Sending move: ({}, {}) -> ({}, {})",
                    sel_row,
                    sel_col,
                    row,
                    col
                );
                on_move.emit(((sel_row, sel_col), (row, col)));
                selected.set(None);
                valid_moves.set(vec![]);
                return;
            }

            if let Some(piece) = game
                .pieces
                .iter()
                .find(|p| p.row == row && p.col == col && p.owner == my_color)
            {
                let moves = game.valid_moves(piece);
                let captures_exist = game.check_captures_moves(&my_color);
                let filtered = if captures_exist {
                    moves
                        .into_iter()
                        .filter(|(r, _)| (*r as i32 - row as i32).abs() == 2)
                        .collect()
                } else {
                    moves
                };
                selected.set(Some((row, col)));
                valid_moves.set(filtered);
            } else {
                selected.set(None);
                valid_moves.set(vec![]);
            }
        })
    };

    // Draw the board
    {
        let canvas_ref = canvas_ref.clone();
        let game = props.game.clone();
        let selected = selected.clone();
        let valid_moves = valid_moves.clone();
        let dark_img = dark_img.clone();
        let light_img = light_img.clone();

        use_effect_with(
            (
                game.clone(),
                (*selected),
                (*valid_moves).clone(),
                (*dark_img).clone(),
                (*light_img).clone(),
            ),
            move |_| {
                let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() else {
                    return;
                };

                let Ok(Some(context)) = canvas.get_context("2d") else {
                    return;
                };

                let Ok(ctx) = context.dyn_into::<CanvasRenderingContext2d>() else {
                    return;
                };

                // Draw squares
                for i in 0..8 {
                    for j in 0..8 {
                        let x = (i * 100) as f64;
                        let y = (j * 100) as f64;

                        let colour = if *selected == Some((j, i)) {
                            "yellow"
                        } else if valid_moves.iter().any(|(r, c)| *r == j && *c == i) {
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

                // Draw pieces
                for piece in &game.pieces {
                    let img_opt = if piece.owner == Player::Dark {
                        &*dark_img
                    } else {
                        &*light_img
                    };

                    let x = (piece.col * 100 + 10) as f64;
                    let y = (piece.row * 100 + 10) as f64;

                    if let Some(img) = img_opt {
                        let _ = ctx.draw_image_with_html_image_element_and_dw_and_dh(
                            img, x, y, 80.0, 80.0,
                        );
                    } else {
                        // Fallback to circles while loading
                        if piece.owner == Player::Dark {
                            ctx.set_fill_style_str("red");
                        } else {
                            ctx.set_fill_style_str("white");
                        }
                        let center_x = (piece.col * 100 + 50) as f64;
                        let center_y = (piece.row * 100 + 50) as f64;
                        ctx.begin_path();
                        let _ = ctx.arc(center_x, center_y, 40.0, 0.0, 2.0 * std::f64::consts::PI);
                        ctx.fill();
                    }

                    if piece.is_kinged {
                        ctx.set_font("30px Arial");
                        ctx.set_fill_style_str("gold");
                        let center_x = (piece.col * 100 + 50) as f64;
                        let center_y = (piece.row * 100 + 50) as f64;
                        let _ = ctx.fill_text("K", center_x - 10.0, center_y + 10.0);
                    }
                }
            },
        );
    }

    html! {
        <canvas ref={canvas_ref} onclick={on_click} id="mp-board" height=800 width=800 style="border: 1px solid black">
        </canvas>
    }
}
