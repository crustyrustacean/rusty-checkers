// frontend/src/components/landing.rs

// dependencies
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct LandingProps {
    pub on_play: Callback<MouseEvent>,
}

#[component]
pub fn Landing(props: &LandingProps) -> Html {
    html! {
        <div class="landing-page">
            <div class="landing-content">
                <svg class="logo" viewBox="0 0 200 200" xmlns="http://www.w3.org/2000/svg">
                    // Mini 2x2 checkerboard base
                    <rect x="50" y="130" width="50" height="50" fill="#16213e" />
                    <rect x="100" y="130" width="50" height="50" fill="#e94560" />
                    <rect x="50" y="80" width="50" height="50" fill="#e94560" />
                    <rect x="100" y="80" width="50" height="50" fill="#16213e" />
                    <rect x="48" y="78" width="104" height="104" rx="4" fill="none" stroke="#e94560" stroke-width="2" />

                    // Crab silhouette sitting on the board
                    // Body (oval)
                    <ellipse cx="100" cy="95" rx="30" ry="20" fill="#c41e3a" />
                    // Left claw
                    <path d="M70 90 Q55 75 45 80 Q40 85 50 88 Z" fill="#c41e3a" />
                    <path d="M45 80 L38 70 M45 80 L52 72" stroke="#c41e3a" stroke-width="3" stroke-linecap="round" />
                    // Right claw
                    <path d="M130 90 Q145 75 155 80 Q160 85 150 88 Z" fill="#c41e3a" />
                    <path d="M155 80 L162 70 M155 80 L148 72" stroke="#c41e3a" stroke-width="3" stroke-linecap="round" />
                    // Left legs
                    <line x1="75" y1="100" x2="58" y2="115" stroke="#c41e3a" stroke-width="2.5" stroke-linecap="round" />
                    <line x1="72" y1="105" x2="55" y2="122" stroke="#c41e3a" stroke-width="2.5" stroke-linecap="round" />
                    // Right legs
                    <line x1="125" y1="100" x2="142" y2="115" stroke="#c41e3a" stroke-width="2.5" stroke-linecap="round" />
                    <line x1="128" y1="105" x2="145" y2="122" stroke="#c41e3a" stroke-width="2.5" stroke-linecap="round" />
                    // Eyes
                    <circle cx="90" cy="88" r="3" fill="#eee" />
                    <circle cx="110" cy="88" r="3" fill="#eee" />

                    // Crown
                    <polygon points="78,78 82,60 90,72 100,55 110,72 118,60 122,78" fill="#e94560" />
                    <circle cx="82" cy="59" r="2.5" fill="#e94560" />
                    <circle cx="100" cy="53" r="2.5" fill="#e94560" />
                    <circle cx="118" cy="59" r="2.5" fill="#e94560" />
                </svg>

                <h1 class="title">{"RUSTY CHECKERS"}</h1>
                <p class="tagline">{"Capture the competition"}</p>
                <button class="play-button" onclick={props.on_play.clone()}>{"PLAY NOW"}</button>
            </div>
            <footer class="footer">
                {"Built with Rust \u{2022} Yew \u{2022} Rama"}
            </footer>
        </div>
    }
}
