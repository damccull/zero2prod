use std::net::TcpListener;

use log::LevelFilter;
use sqlx::{postgres::PgPoolOptions, ConnectOptions};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, EnvFilter, Registry};
use zero2prod::{configuration::get_configuration, startup::run};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    ////
    // Set up logging using tracing, tracing-subscriber, and tracing-bunyan-formatter

    // Redirect all "Log" events to tracing
    LogTracer::init().expect("Failed to set logger.");

    // Set up the tracing stuff
    // Print all lines at info-level or above if RUST_LOG has not been set
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new(
        "zero2prod".into(),
        // Output the spans to stdout
        std::io::stdout,
    );
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    // Tell the application that the new subscriber we just layered together should be the one to process spans
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber.");

    ////
    // Panic if config can't be read
    let configuration = get_configuration().expect("Failed to read the configuration.");

    // // Set up the database connection
    // let connection_pool = PgPool::connect(&configuration.database.connection_string())
    //     .await
    //     .expect("Failed to connect to Postgres.");

    // Create a database connection for the web server.
    let db_connect_options = configuration
        .database
        .with_db()
        .log_statements(LevelFilter::Trace)
        .to_owned();

    let connection_pool = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_with(db_connect_options)
        .await
        .expect("Failed to connect to Postgres.");

    // Port comes from the settings file
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await
}
