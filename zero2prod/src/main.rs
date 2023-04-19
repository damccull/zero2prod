use zero2prod::{configuration::get_configuration, startup::Application, telemetry};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up tracing
    let subscriber = telemetry::get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber);

    // Set up configuration
    let configuration = get_configuration().expect("failed to read configuration");

    let app = Application::build(configuration).await?;
    app.run_until_stopped().await?;
    Ok(())
}
