use std::net::TcpListener;

use env_logger::Env;
use sqlx::PgPool;
use zero2prod::{configuration::get_configuration, startup::run};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Set up logging to console, print all logs info and above by default
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Panic if config can't be read
    let configuration = get_configuration().expect("Failed to read the configuration.");

    // Set up the database connection
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    // Port comes from the settings file
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await
}
