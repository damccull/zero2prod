use std::net::TcpListener;

use once_cell::sync::Lazy;
use sqlx::{ConnectOptions, Connection, Executor, PgConnection, PgPool, postgres::PgPoolOptions};
use tracing::log::LevelFilter;
use uuid::Uuid;
use zero2prod::{
    configuration::{get_configuration, DatabaseSettings},
    email_client::EmailClient,
    telemetry::{get_subscriber, init_subscriber},
};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

// Launch the app in the background
pub async fn spawn_app() -> TestApp {
    //
    // Set up Logging
    Lazy::force(&TRACING);

    //
    // Bind a random OS port
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    // Get the random port from the listener
    let port = listener.local_addr().unwrap().port();
    // Format an address string
    let address = format!("http://127.0.0.1:{}", port);

    // Get the app configuration
    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();

    // Get a db connection pool
    let connection_pool = configure_database(&configuration.database).await;

    // Build an `EmailClient` using the configuration
    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address.");
    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
        timeout,
    );

    // Use the listener for spinning up a server
    let server = zero2prod::startup::run(listener, connection_pool.clone(), email_client)
        .expect("Failed to bind address");
    // Execute the server in an Executor
    let _ = tokio::spawn(server);

    TestApp {
        address,
        db_pool: connection_pool,
    }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create the new database
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres.");
    connection
        .execute(&*format!(r#"CREATE DATABASE "{}";"#, config.database_name))
        .await
        .expect("Failed to create database.");

    // Create a database pool for the web server specifying that sqlx logs should be at the 'trace' level.
    let db_connect_options = config
        .with_db()
        .log_statements(LevelFilter::Trace)
        .to_owned();

    let connection_pool = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_with(db_connect_options)
        .await
        .expect("Failed to connect to Postgres.");

    // Migrate the database
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database.");

    connection_pool
}
