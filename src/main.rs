use std::net::TcpListener;

use zero2prod::{configuration::get_configuration, startup::run};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Panic if config can't be read
    let configuration = get_configuration().expect("Failed to read the configuration.");

    // Port comes from the settings file
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener)?.await
}
