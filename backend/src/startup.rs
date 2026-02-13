// backend/src/startup.rs

// dependencies
use crate::config::Settings;
use crate::errors::ServerBoxError;
use crate::errors::ServerErrorContext;
use crate::errors::ServerOpaqueError;
use crate::game_server::GameServer;
use crate::routes::{health_check, web_socket::game_handler};
use crate::state::ServerState;
use crate::telemetry::make_request_span;
use rama::{
    Layer,
    error::BoxError,
    graceful::Shutdown,
    http::layer::trace::TraceLayer,
    http::server::HttpServer,
    http::service::fs::{DirectoryServeMode::NotFound, ServeDir, ServeFile},
    http::service::web::Router,
    http::ws::handshake::server::WebSocketAcceptor,
    rt::Executor,
    service::service_fn,
    tcp::server::TcpListener,
    telemetry::tracing,
};
use std::time::Duration;

pub struct Server {
    pub router: Router<ServerState>,
    pub listener: TcpListener,
}

impl Server {
    pub async fn build(configuration: &Settings) -> Result<Self, ServerBoxError> {
        // build server state
        let state = ServerState::new();

        // build the server router
        let router = Self::build_server_router(state);

        // configure the host and port
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(address)
            .await
            .map_err(ServerOpaqueError::from_boxed)
            .context(format!(
                "Unable to create TCP listener on: Host: {}, Port: {}",
                configuration.application.host, configuration.application.port
            ))?;

        tracing::info!(
            "Listening on: Host: {}, Port: {}",
            configuration.application.host,
            configuration.application.port
        );

        Ok(Self { router, listener })
    }

    pub fn build_server_router(state: ServerState) -> Router<ServerState> {
        let assets_dir = std::env::var("ASSETS_DIR").unwrap_or_else(|_| "../public".to_string());

        let game_server = GameServer::new();

        // Spawn background task to clean up stale tournaments every 5 minutes
        {
            let gs = game_server.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(300));
                loop {
                    interval.tick().await;
                    let mut tournaments = gs.tournaments.write().await;
                    let before = tournaments.len();
                    tournaments.retain(|_code, tm| !tm.is_stale());
                    let removed = before - tournaments.len();
                    if removed > 0 {
                        tracing::info!("Cleaned up {} stale tournament(s)", removed);
                    }
                }
            });
        }

        let gs = game_server.clone();

        Router::new_with_state(state)
            .with_sub_router_make_fn("/api", |router| {
                router.with_sub_router_make_fn("/v1", |router| {
                    router
                        .with_get("/health_check", health_check)
                        .with_sub_service(
                            "/ws",
                            WebSocketAcceptor::new().into_service(service_fn(move |ws| {
                                let server = gs.clone();
                                async move { game_handler(ws, server).await }
                            })),
                        )
                })
            })
            .with_sub_service(
                "/public",
                ServeDir::new(&assets_dir).with_directory_serve_mode(NotFound),
            )
            .with_get("/", ServeFile::new(format!("{}/index.html", assets_dir)))
    }

    pub async fn run(self, configuration: &Settings) -> Result<(), BoxError> {
        let graceful = Shutdown::default();

        let router = self.router;
        let listener = self.listener;

        let http_service_with_tracing =
            TraceLayer::new_for_http().make_span_with(make_request_span);

        tracing::info!("Running the application...");
        graceful.spawn_task_fn(async |guard| {
            let exec = Executor::graceful(guard.clone());
            let http_service =
                HttpServer::auto(exec).service(http_service_with_tracing.into_layer(router));
            listener.serve_graceful(guard, http_service).await;
        });

        graceful
            .shutdown_with_limit(Duration::from_secs(
                configuration.application.shutdown_timeout,
            ))
            .await?;

        Ok(())
    }
}
