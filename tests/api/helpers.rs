use once_cell::sync::Lazy;
use sqlx::{postgres::PgPoolOptions, ConnectOptions, Connection, Executor, PgConnection, PgPool};
use tracing::log::LevelFilter;
use uuid::Uuid;
use zero2prod::{
    configuration::{get_configuration, DatabaseSettings},
    startup::{get_connection_pool, Application},
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

    // Set up the configuration
    let configuration = {
        let mut c = get_configuration().expect("Failed to load configuration.");
        // Use a different db for each test
        c.database.database_name = Uuid::new_v4().to_string();
        // Use a random OS port
        c.application.port = 0;
        c
    };

    // Create and migrate the db
    configure_database(&configuration.database).await;

    // Launch the application as a background task
    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build the application.");
    let address = format!("http://127.0.0.1:{}", application.port());
    // Execute the server in an Executor
    let _ = tokio::spawn(application.run_until_stopped());

    TestApp {
        address,
        db_pool: get_connection_pool(&configuration.database),
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
