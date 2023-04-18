use std::{future::Future, net::TcpListener, sync::Arc};

use axum::{
    extract::FromRef,
    routing::{get, post},
    Router,
};
use sqlx::PgPool;

use crate::telemetry::RouterExt;
use crate::{
    email_client::EmailClient,
    routes::{health_check, subscribe},
};

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
