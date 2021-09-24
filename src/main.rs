use std::net::TcpListener;

use sqlx::{Connection, PgConnection};
use zero2prod::{configuration::get_configuration, startup::run};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Panic if config can't be read
    let configuration = get_configuration().expect("Failed to read the configuration.");

    // Set up the database connection
    let connection = PgConnection::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    // Port comes from the settings file
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, connection)?.await
}
