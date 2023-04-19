use std::{future::Future, net::TcpListener, sync::Arc};

use axum::{
    extract::FromRef,
    routing::{get, post},
    Router,
};
use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::{configuration::Settings, telemetry::RouterExt};
use crate::{
    email_client::EmailClient,
    routes::{health_check, subscribe},
};

pub async fn build(
    configuration: Settings,
) -> Result<impl Future<Output = hyper::Result<()>>, Box<dyn std::error::Error>> {
    // Build a database connection pool
    let db = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());

    // Build an email clientz
    let timeout = configuration.email_client.timeout();
    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender address.");
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
        timeout,
    );
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address.to_string()).map_err(|e| {
        tracing::error!("failed to bind port {}", address);
        e
    })?;
    Ok(run(listener, db, email_client))
}

pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
) -> impl Future<Output = hyper::Result<()>> {
    // Build app state
    let app_state = AppState {
        db_pool,
        email_client: Arc::new(email_client),
    };

    // Create a router that will contain and match all routes for the application
    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .add_axum_tracing_layer()
        .with_state(app_state);

    // Start the axum server and set up to use supplied listener
    axum::Server::from_tcp(listener)
        .expect("failed to create server from listener")
        .serve(app.into_make_service())
}

#[derive(Clone)]
struct AppState {
    db_pool: PgPool,
    email_client: Arc<EmailClient>,
}

impl FromRef<AppState> for PgPool {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.db_pool.clone()
    }
}

impl FromRef<AppState> for Arc<EmailClient> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.email_client.clone()
    }
}
