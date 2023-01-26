use std::{future::Future, net::TcpListener};

use axum::{
    routing::{get, post},
    Router,
};

use crate::routes::{health_check, subscribe};

pub fn run(listener: TcpListener) -> impl Future<Output = hyper::Result<()>> {
    // Create a router that will contain and match all routes for the application
    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe));

    // Start the axum server and set up to use supplied listener
    axum::Server::from_tcp(listener)
        .expect("failed to create server from listener")
        .serve(app.into_make_service())
}
