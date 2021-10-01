use zero2prod::{
    configuration::get_configuration,
    startup::build,
    telemetry::{get_subscriber, init_subscriber},
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //
    // Set up logging using tracing, tracing-subscriber, and tracing-bunyan-formatter
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    //
    // Panic if config can't be read
    let configuration = get_configuration().expect("Failed to read the configuration.");

    let server = build(configuration).await?;
    server.await?;
    Ok(())
}
