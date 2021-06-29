use std::{net::TcpListener, str::FromStr};

use log::LevelFilter;
use once_cell::sync::Lazy;
use sqlx::{
    postgres::PgConnectOptions, ConnectOptions, Connection, Executor, PgConnection, PgPool,
};
use startup::run;
use uuid::Uuid;
use zero2prod::{
    configuration::{get_configuration, DatabaseSettings},
    startup,
    telemetry::{get_subscriber, init_subscriber},
};

// Ensure that the `tracing` stack is only initialized once use `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    // The output of `get_subscriber` cannot be assigned to a variable based on the
    // value of `TEST_LOG` because the sink is part of the type returned by `get_subscriber`,
    // therefore they are not the same type. We could work around it, but this is the most
    // straight-forward way of moving forward.

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

pub async fn spawn_app() -> TestApp {
    // Initialize logging
    // The first time that `inistialize` is invoked the code in `TRACING` is executed.
    // All other invocations will instead skip execution.
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind a random port.");

    let port = listener.local_addr().unwrap().port();

    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    // Randomize the database name to guarantee tests will not interact
    configuration.database.database_name = Uuid::new_v4().to_string();
    // Create the database in postgres
    let db_pool = configure_database(&configuration.database).await;

    let server = run(listener, db_pool.clone()).expect("Failed to bind address.");
    let _ = tokio::spawn(server);

    TestApp { address, db_pool }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create the database
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres.");
    connection
        .execute(&*format!(r#"CREATE DATABASE "{}";"#, config.database_name))
        .await
        .expect("Failed to create database.");

    // Migrate the database
    // let connection_pool = PgPool::connect(&config.connection_string())
    //     .await
    //     .expect("Failed to connect to Postgre.");

    // Create a database pool for the web server specifying that sqlx logs should be at the 'trace' level.
    let db_connect_options = PgConnectOptions::from_str(&config.connection_string())
        .unwrap()
        .log_statements(LevelFilter::Trace)
        .to_owned();

    let db_pool = PgPool::connect_with(db_connect_options)
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to migrate the database.");

    db_pool
}
