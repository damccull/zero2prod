#![allow(unused)]

use std::{net::TcpListener, str::FromStr};

use log::LevelFilter;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    ConnectOptions, Connection, PgConnection, PgPool,
};
use zero2prod::{configuration::get_configuration, startup::run};

use env_logger::Env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize the logger
    // `init` does call `set_logger`, so this is all we need to do
    // Falling back to printing all logs at info-level or above if
    // RUST_LOG env var is not set.
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Panic if config can't be read
    let configuration = get_configuration().expect("Failed to read configuration");

    // Create a database connection for the web server.
    let db_connect_options =
        PgConnectOptions::from_str(&configuration.database.connection_string())
            .unwrap()
            .log_statements(LevelFilter::Trace)
            .to_owned();

    let db_pool = PgPool::connect_with(db_connect_options)
        .await
        .expect("Failed to connect to Postgres.");
    //let db_pool = PgPoolOptions::new()

    // Create a `TcpListener` to pass to the web server.
    let listener = TcpListener::bind(format!(
        "127.0.0.1:{}",
        configuration.application.listen_port
    ))
    .expect("Failed to bind port.");
    run(listener, db_pool)?.await
}
