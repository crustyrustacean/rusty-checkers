# Rusty Checkers

A modern, networked take on the classic Checkers, written front to back in Rust!

## Overview

Rusty Checkers is a full-stack checkers game with a Rust backend serving a WebAssembly
frontend. The game implements standard American checkers rules on an 8x8 board,
including forced captures, multi-jump sequences, and king promotion.

## Architecture

```
rusty-checkers/
├── common/           # Shared game domain library
│   └── src/
│       ├── lib.rs     # Public API (re-exports Game, GamePiece, Player)
│       ├── game.rs    # Game struct and core checkers logic
│       ├── piece.rs   # GamePiece struct
│       └── player.rs  # Player enum
├── backend/          # HTTP server (Rama + Tokio)
│   ├── src/
│   │   ├── bin/      # Server entry point
│   │   ├── routes/   # API endpoints
│   │   ├── config.rs
│   │   ├── errors.rs
│   │   ├── response.rs
│   │   ├── startup.rs
│   │   ├── state.rs
│   │   └── telemetry.rs
│   ├── tests/        # Integration tests
│   └── config/       # Environment-specific YAML configs
├── frontend/         # WASM client (Yew + Yewdux)
│   ├── src/
│   │   ├── bin/      # WASM entry point
│   │   ├── components/
│   │   │   ├── grid.rs           # Board canvas and click handling
│   │   │   ├── game_status.rs    # Game over display
│   │   │   ├── turn_indicator.rs # Current turn display
│   │   │   └── reset_button.rs   # New game button
│   │   ├── views/
│   │   │   └── game_view.rs      # Main game layout
│   │   └── state.rs   # Application state
│   └── index.html
├── Dockerfile         # Multi-stage production build
├── fly.toml           # Fly.io deployment config
└── justfile           # Task runner recipes
```

| Layer    | Crate      | Framework         | Role                                          |
|----------|------------|-------------------|------------------------------------------------|
| Common   | `common`   | —                 | Shared game domain types and core checkers logic |
| Backend  | `backend`  | Rama, Tokio       | Serves static assets and health check API      |
| Frontend | `frontend` | Yew, Yewdux       | Game UI, compiled to WebAssembly               |

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

- **Common unit tests** (`common/src/game.rs`) — move validation and board state
  logic
- **Backend integration tests** (`backend/tests/api/`) — health check endpoint
  verification

## Docker

Build and run with Docker:

```sh
docker build -t rusty-checkers .
docker run -p 8080:8080 rusty-checkers
```

The Dockerfile uses a multi-stage build with
[cargo-chef](https://github.com/LukeMathWalker/cargo-chef) for dependency
caching:

1. **Planner** — generates a dependency recipe for layer caching
2. **Frontend builder** — compiles the Yew app to WASM via Trunk
3. **Backend builder** — compiles the server binary with cached dependencies
4. **Runtime** — minimal Debian Slim image with the server binary and static assets

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
contains the core types and logic for the checkers game and has **no external
dependencies**, keeping it lightweight and portable across both native and WASM
targets.

It exports three types:

| Type        | Purpose                                        |
|-------------|------------------------------------------------|
| `Game`      | Board state, move validation, captures, turns  |
| `GamePiece` | Individual piece with position and king status |
| `Player`    | Dark / Light player enum                       |

### Backend

| Crate                       | Purpose                         |
|-----------------------------|---------------------------------|
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
| `checkers_common`             | Shared game domain types and logic |
| `yew`                         | Component-based UI framework      |
| `yewdux`                      | Global state management           |
| `web-sys`                     | Web API bindings (Canvas, DOM)    |
| `wasm-bindgen`                | Rust/JS interop                   |
| `gloo-net`                    | HTTP requests from WASM           |
| `console_error_panic_hook`    | Better panic messages in browser  |

## License

This project is licensed under the [MIT License](License.txt).
