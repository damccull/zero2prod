use std::net::TcpListener;

use env_logger::Env;
use log::LevelFilter;
use sqlx::{postgres::PgPoolOptions, ConnectOptions, PgPool};
use zero2prod::{configuration::get_configuration, startup::run};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Set up logging to console, print all logs info and above by default
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

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
