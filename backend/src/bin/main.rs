// backend/src/main.rs

// dependencies
use rama::telemetry::tracing;
use rusty_checkers_server_lib::config::get_configuration;
use rusty_checkers_server_lib::errors::{AppBoxError, AppErrorContext, AppOpaqueError};
use rusty_checkers_server_lib::startup::Application;
use rusty_checkers_server_lib::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> Result<(), AppBoxError> {
    // initialize tracing
    let subscriber = get_subscriber(
        "rusty-checkers".into(),
        "info,rama=debug".into(),
        std::io::stdout,
    );
    init_subscriber(subscriber);

    // build the app configuration
    tracing::info!("Reading app configuration...");
    let configuration = get_configuration().expect("Failed to read configuration");

    // build and run the application
    tracing::info!("Building the application...");
    Application::build(&configuration)
        .await
        .map_err(AppOpaqueError::from_boxed)
        .context("Unable to build the server on the configured host and port.")?
        .run(&configuration)
        .await
        .map_err(AppOpaqueError::from_boxed)
        .context("Unable to run the server")?;

    Ok(())
}
