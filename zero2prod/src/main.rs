use std::net::TcpListener;

use secrecy::ExposeSecret;
use sqlx::PgPool;
use zero2prod::{configuration::get_configuration, startup::run, telemetry};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up tracing
    let subscriber = telemetry::get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber);

    // Set up configuration
    let configuration = get_configuration().expect("failed to read configuration");

    let db = PgPool::connect(configuration.database.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres");

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    tracing::info!("Starting server and listening on {}", address);

    let listener = TcpListener::bind(format!("[::]:{address}")).map_err(|e| {
        tracing::error!("failed to bind port {}", address);
        e
    })?;

    let _ = run(listener, db).await;
    Ok(())
}
