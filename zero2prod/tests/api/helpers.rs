use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use wiremock::MockServer;
use zero2prod::{
    configuration::{get_configuration, DatabaseSettings},
    startup::{get_db_pool, Application},
    telemetry::{get_subscriber, init_subscriber},
};

static TRACING: Lazy<()> = Lazy::new(|| {
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(
            "test".into(),
            "zero2prod=debug,info".into(),
            std::io::stdout,
        );
        init_subscriber(subscriber);
    } else {
        let subscriber =
            get_subscriber("test".into(), "zero2prod=debug,info".into(), std::io::sink);
        init_subscriber(subscriber);
    }
});

pub async fn spawn_app() -> TestApp {
    // Set up subscriber for logging, only first time per run. Other times use existing subscriber.
    Lazy::force(&TRACING);

    let email_server = MockServer::start().await;
    let configuration = {
        // Get the configuration from file
        let mut c = get_configuration().expect("Failed to read configuration");
        // Use a different database for each test
        c.database.database_name = Uuid::new_v4().to_string();
        // Use a random OS port
        c.application.port = 0;
        c.email_client.base_url = email_server.uri();
        c
    };

    // Set up database connection pool
    configure_database(&configuration.database).await;

    // Start the server
    let app = Application::build(configuration.clone())
        .await
        .expect("Failed to build application");
    let port = app.port();
    let address = format!("http://127.0.0.1:{}", app.port());
    tokio::spawn(app.run_until_stopped());

    let test_app = TestApp {
        address,
        port,
        db_pool: get_db_pool(&configuration.database),
        email_server,
    };

    add_test_user(&test_app.db_pool).await;

    test_app
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create a database
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("failed to connect to Postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("failed to create the database");

    //Run migrations on the database
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("failed to connect to Postgres");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("failed to migrate the database");

    //Return the connection pool
    connection_pool
}

async fn add_test_user(pool: &PgPool) {
    sqlx::query!(
        "INSERT INTO users (user_id, username, password)\
        VALUES ($1, $2, $3)",
        Uuid::new_v4(),
        Uuid::new_v4().to_string(),
        Uuid::new_v4().to_string(),
    )
    .execute(pool)
    .await
    .expect("Failed to create test users.");
}

pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub db_pool: PgPool,
    pub email_server: MockServer,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("failed to execute request")
    }

    pub async fn post_newsletters(&self, body: serde_json::Value) -> reqwest::Response {
        let (username, password) = self.test_user().await;
        reqwest::Client::new()
            .post(&format!("{}/newsletters", &self.address))
            .basic_auth(username, Some(password))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn test_user(&self) -> (String, String) {
        let row = sqlx::query!("SELECT username, password FROM users LIMIT 1")
            .fetch_one(&self.db_pool)
            .await
            .expect("Failed to fetch test users.");
        (row.username, row.password)
    }

    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

        // Extract the link from one of the request fields
        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);

            let raw_link = links[0].as_str().to_owned();

            let mut confirmation_link = reqwest::Url::parse(&raw_link).unwrap();

            // Make sure we don't call random APIs on the web
            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
            // Rewrite URL to include the port
            confirmation_link.set_port(Some(self.port)).unwrap();

            confirmation_link
        };

        let html = get_link(body["HtmlBody"].as_str().unwrap());
        let plain_text = get_link(body["TextBody"].as_str().unwrap());

        ConfirmationLinks { html, plain_text }
    }
}

pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}
