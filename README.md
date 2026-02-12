# Rusty Checkers

A modern, networked take on the classic Checkers, written front to back in Rust!

## Overview

Rusty Checkers is a full-stack checkers game with a Rust backend serving a WebAssembly
frontend over WebSockets. The game implements standard American checkers rules on an 8x8
board, including forced captures, multi-jump sequences, and king promotion. It supports
real-time multiplayer (human vs human) and single-player modes against AI opponents of
varying difficulty.

## Architecture

```
rusty-checkers/
├── common/           # Shared game domain library
│   └── src/
│       ├── lib.rs         # Public API and module re-exports
│       ├── game.rs        # Game struct and core checkers logic
│       ├── piece.rs       # GamePiece struct
│       ├── player.rs      # Player enum (Dark / Light)
│       ├── ai.rs          # AI opponents (RandomAi, MinimaxAi)
│       ├── messages.rs    # Client/server message types
│       ├── traits.rs      # BoardGame trait abstraction
│       └── game_tests.rs  # Unit tests for game logic
├── backend/          # HTTP + WebSocket server (Rama + Tokio)
│   ├── src/
│   │   ├── bin/           # Server entry point
│   │   ├── routes/        # API endpoints (health check, WebSocket)
│   │   ├── game_server.rs # Game session management
│   │   ├── config.rs
│   │   ├── errors.rs
│   │   ├── response.rs
│   │   ├── startup.rs
│   │   ├── state.rs
│   │   └── telemetry.rs
│   ├── tests/        # Integration tests
│   └── config/       # Environment-specific YAML configs
├── frontend/         # WASM client (Yew)
│   ├── src/
│   │   ├── bin/           # WASM entry point
│   │   ├── app.rs         # Main app component and routing
│   │   ├── websocket.rs   # WebSocket client
│   │   ├── components/
│   │   │   ├── landing.rs     # Landing page with crab logo
│   │   │   ├── lobby.rs       # Game mode selection
│   │   │   ├── mp_grid.rs     # 8x8 board canvas and click handling
│   │   │   ├── rules_card.rs  # Game rules display
│   │   │   └── reset_button.rs # New game button
│   │   └── views/
│   │       └── game_view.rs   # Main game layout
│   └── index.html
├── Dockerfile         # Multi-stage production build
├── fly.toml           # Fly.io deployment config
└── justfile           # Task runner recipes
```

| Layer    | Crate      | Framework         | Role                                                  |
|----------|------------|-------------------|-------------------------------------------------------|
| Common   | `common`   | ---               | Shared game types, rules engine, AI, and message protocol |
| Backend  | `backend`  | Rama, Tokio       | HTTP server, WebSocket game sessions, static assets   |
| Frontend | `frontend` | Yew               | Game UI compiled to WebAssembly                       |

## Game Modes

- **Multiplayer** -- Two players connect via WebSocket. The first player creates a game
  and waits; the second player is matched in. Turns alternate in real time.
- **vs AI (Easy)** -- Play against a random-move AI.
- **vs AI (Medium)** -- Play against a minimax AI with alpha-beta pruning (depth 4).
- **vs AI (Hard)** -- Play against a minimax AI with alpha-beta pruning (depth 6).

## Game Rules

- Standard 8x8 board, 12 pieces per side (Dark and Light)
- Pieces move diagonally forward one square
- Captures are mandatory when available
- Multi-jump sequences must be completed
- A piece reaching the opposite end of the board is promoted to a king
- Kings move diagonally in all four directions
- The game ends when a player has no legal moves remaining

## Prerequisites

- [Rust](https://rustup.rs/) (1.93+, 2024 edition)
- [Trunk](https://trunkrs.dev/) for building the WASM frontend:
  ```sh
  cargo install trunk
  ```
- The `wasm32-unknown-unknown` target:
  ```sh
  rustup target add wasm32-unknown-unknown
  ```
- (Optional) [just](https://github.com/casey/just) task runner
- (Optional) [Docker](https://www.docker.com/) for containerized builds

## Getting Started

### Development

Start the frontend dev server with hot reload:

```sh
just dev
```

Or directly:

```sh
cd frontend && trunk serve --open
```

This compiles the WASM frontend and opens it in your browser.

To run the backend separately:

```sh
cargo run -p backend
```

The backend reads configuration from `backend/config/`. The local environment
binds to `127.0.0.1:8000` and serves frontend assets from `../public` by default.

### Building for Release

Build the full workspace:

```sh
cargo build --release
```

Build just the frontend:

```sh
cd frontend && trunk build --release
```

The frontend build output goes to `public/` at the workspace root (configured in
`frontend/Trunk.toml`).

## Testing

Run all workspace tests:

```sh
cargo test
```

The test suite includes:

- **Common unit tests** (`common/src/game_tests.rs`) -- move validation, captures,
  blocked-piece detection, and message serialization
- **Backend integration tests** (`backend/tests/api/`) -- health check and WebSocket
  endpoint verification

## Docker

Build and run with Docker:

```sh
docker build -t rusty-checkers .
docker run -p 8080:8080 rusty-checkers
```

The Dockerfile uses a multi-stage build with
[cargo-chef](https://github.com/LukeMathWalker/cargo-chef) for dependency
caching:

1. **Planner** -- generates a dependency recipe for layer caching
2. **Frontend builder** -- compiles the Yew app to WASM via Trunk
3. **Backend builder** -- compiles the server binary with cached dependencies
4. **Runtime** -- minimal Debian Slim image with the server binary and static assets

## Deployment

The project is configured for [Fly.io](https://fly.io/) deployment:

```sh
fly deploy
```

Production settings (`backend/config/production.yaml`) bind to `0.0.0.0:8080`.
The Fly.io configuration (`fly.toml`) targets the `ord` (Chicago) region with
auto-start/stop machine scaling.

## Configuration

The backend uses layered YAML configuration in `backend/config/`:

| File              | Purpose                              |
|-------------------|--------------------------------------|
| `base.yaml`       | Shared defaults (port, assets path, shutdown timeout) |
| `local.yaml`      | Local development overrides (localhost binding)       |
| `production.yaml` | Production overrides (0.0.0.0, port 8080)            |

The `APP_ENVIRONMENT` environment variable selects which overlay to apply
(`local` or `production`). The `ASSETS_DIR` environment variable overrides the
static assets path.

## Dependencies

### Common

The `common` crate (`checkers_common`) is the shared game domain library. It
contains the core types, rules engine, AI opponents, and network message
protocol for the checkers game.

Key exports:

| Type / Trait    | Purpose                                        |
|-----------------|-------------------------------------------------|
| `Game`          | Board state, move validation, captures, turns   |
| `GamePiece`     | Individual piece with position and king status   |
| `Player`        | Dark / Light player enum                         |
| `MoveResult`    | Outcome of a move (TurnComplete, ContinueJump, GameWon, InvalidMove) |
| `BoardGame`     | Trait abstraction over board game implementations |
| `AiPlayer`      | Trait for AI move selection                      |
| `RandomAi`      | Random move AI (Easy)                            |
| `MinimaxAi`     | Alpha-beta pruning minimax AI (Medium / Hard)    |
| `ClientMessage`  | Messages sent from client to server             |
| `ServerMessage`  | Messages sent from server to client             |

Dependencies: `serde` / `serde_json` (serialization), `rand` (random move
selection), `getrandom` (WASM-compatible randomness).

### Backend

| Crate                       | Purpose                         |
|-----------------------------|---------------------------------|
| `common`                    | Shared game domain types and logic |
| `rama`                      | HTTP server framework           |
| `tokio`                     | Async runtime                   |
| `serde` / `serde-aux`      | Serialization                   |
| `config`                    | YAML configuration loading      |
| `thiserror`                 | Error type derivation           |
| `chrono`                    | Date/time handling              |
| `uuid`                      | Request ID generation           |
| `tracing-*`                 | Structured logging and tracing  |

### Frontend

| Crate                         | Purpose                           |
|-------------------------------|-----------------------------------|
| `common`                      | Shared game domain types and logic |
| `yew`                         | Component-based UI framework      |
| `web-sys`                     | Web API bindings (Canvas, DOM)    |
| `wasm-bindgen`                | Rust/JS interop                   |
| `gloo-net`                    | HTTP requests from WASM           |
| `console_error_panic_hook`    | Better panic messages in browser  |

## License

This project is licensed under the [MIT License](License.txt).
