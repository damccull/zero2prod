#![allow(unused)]

use std::net::TcpListener;

use sqlx::{Connection, PgConnection, PgPool};
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
    let db_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    // Create a `TcpListener` to pass to the web server.
    let listener = TcpListener::bind(format!(
        "127.0.0.1:{}",
        configuration.application.listen_port
    ))
    .expect("Failed to bind port.");
    run(listener, db_pool)?.await
}
