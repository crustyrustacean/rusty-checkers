// src/components/grid.rs

// dependencies
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d};
use yew::prelude::*;

// grid component
#[component]
pub fn Grid() -> Html {
    let canvas_ref = use_node_ref();

    {
        let canvas_ref = canvas_ref.clone();
        use_effect(move || {
            if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                let ctx = canvas
                    .get_context("2d")
                    .unwrap()
                    .unwrap()
                    .dyn_into::<CanvasRenderingContext2d>()
                    .unwrap();

                
                for i in 0 .. 8 {
                    for j in 0 .. 8 {
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

                ctx.set_fill_style_str("red");
                let row = 0;
                let col = 1;

                let center_x = (col * 100 + 50) as f64;
                let center_y = (row * 100 + 50) as f64;
                let radius = 40.0;

                ctx.begin_path();
                ctx.arc(center_x, center_y, radius, 0.0, 2.0 * std::f64::consts::PI).unwrap();
                ctx.fill();
            }
        });
    }
    
    html! {
        <canvas ref={canvas_ref} id="board" height=800 width=800 style="border: 1px solid black">
        </canvas>
    }
}