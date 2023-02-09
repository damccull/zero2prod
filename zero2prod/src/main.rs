use std::net::TcpListener;

use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use zero2prod::{configuration::get_configuration, startup::run};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up tracing, see the method definition
    setup_tracing();

    // Set up configuration
    let configuration = get_configuration().expect("failed to read configuration");

    let port = configuration.application.port;
    tracing::info!("Starting server and listening on {}", port);

    let listener = TcpListener::bind(format!("[::]:{port}")).map_err(|e| {
        tracing::error!("failed to bind port {}", port);
        e
    })?;

    let _ = run(listener).await;
    Ok(())
}

/// Sets up a tracing subscriber.
fn setup_tracing() {
    let fmt_layer = fmt::layer().compact().with_target(true);
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    let subscriber = tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer);

    let _ = tracing::subscriber::set_global_default(subscriber)
        .map_err(|_err| eprintln!("Unable to set global default subscriber"));
}
