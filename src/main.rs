#![allow(unused)]

use std::net::TcpListener;

use zero2prod::{configuration::get_configuration, startup::run};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Panic if config can't be read
    let configuration = get_configuration().expect("Failed to read configuration");

    // Create a `TcpListener` to pass to the web server.
    let listener = TcpListener::bind(format!(
        "127.0.0.1:{}",
        configuration.application.listen_port
    ))
    .expect("Failed to bind port.");
    run(listener)?.await
}
