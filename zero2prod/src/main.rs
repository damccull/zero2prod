use std::net::TcpListener;

use sqlx::postgres::PgPoolOptions;
use zero2prod::{
    configuration::get_configuration, email_client::EmailClient, startup::run, telemetry,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up tracing
    let subscriber = telemetry::get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber);

    // Set up configuration
    let configuration = get_configuration().expect("failed to read configuration");

    // Build a database connection pool
    let db = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());

    // Build an email clientz
    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender address.");
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
    );

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    tracing::info!("Starting server and listening on {}", address);

    let listener = TcpListener::bind(address.to_string()).map_err(|e| {
        tracing::error!("failed to bind port {}", address);
        e
    })?;

    let _ = run(listener, db, email_client).await;
    Ok(())
}
