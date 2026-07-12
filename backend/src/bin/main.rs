// backend/src/main.rs

// dependencies
use rama::telemetry::tracing;
use rusty_checkers_server_lib::config::get_configuration;
use rusty_checkers_server_lib::errors::{ServerBoxError, ServerErrorContext, ServerErrorExt};
use rusty_checkers_server_lib::startup::Server;
use rusty_checkers_server_lib::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> Result<(), ServerBoxError> {
    // initialize tracing
    let subscriber = get_subscriber(
        "rusty-checkers".into(),
        "info,rama=debug".into(),
        std::io::stdout,
    );
    init_subscriber(subscriber);

    // build the server configuration
    tracing::info!("Reading app configuration...");
    let configuration = get_configuration().expect("Failed to read configuration");

    // build and run the server
    tracing::info!("Building the server...");
    Server::build(&configuration)
        .map_err(|e| e.into_opaque_error())
        .context("Unable to build the server on the configured host and port.")?
        .run(&configuration)
        .await
        .map_err(|e| e.into_opaque_error())
        .context("Unable to run the server")?;

    Ok(())
}
