use std::net::TcpListener;
use std::sync::Arc;

use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;

use crate::routes::*;

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    // Wrap the connection in an Arc smart pointer so each instance of App can have its own reference.
    let db_connection = Arc::new(db_pool);

    let server = HttpServer::new(move || {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(db_connection.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
