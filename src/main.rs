#![allow(unused)]

use std::{net::TcpListener, str::FromStr};

use log::LevelFilter;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    ConnectOptions, Connection, PgConnection, PgPool,
};
use tracing::{subscriber::set_global_default, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use zero2prod::{
    configuration::get_configuration,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //********** Initialize the logger
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // Panic if config can't be read
    let configuration = get_configuration().expect("Failed to read configuration");

    // Create a database connection for the web server.
    let db_connect_options =
        PgConnectOptions::from_str(&configuration.database.connection_string())
            .unwrap()
            .log_statements(LevelFilter::Trace)
            .to_owned();

    let db_pool = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_with(db_connect_options)
        .await
        .expect("Failed to connect to postgres.");

    // let db_pool = PgPool::connect_with(db_connect_options)
    //     .await
    //     .expect("Failed to connect to Postgres.");

    // Create a `TcpListener` to pass to the web server.
    let listener = TcpListener::bind(format!(
        "{}:{}",
        configuration.application.listen_address, configuration.application.listen_port
    ))
    .expect("Failed to bind port.");
    run(listener, db_pool)?.await
}
