use zero2prod::{
    configuration::get_configuration, issue_delivery_worker::run_worker_until_stopped,
    startup::Application, telemetry,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up tracing
    let subscriber = telemetry::get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber);

    // Set up configuration
    let configuration = get_configuration().expect("failed to read configuration");

    let app = Application::build(configuration.clone()).await?;
    let app_task = tokio::spawn(app.run_until_stopped());
    let email_delivery_worker_task = tokio::spawn(run_worker_until_stopped(configuration));

    tokio::select! {
        _ = app_task => {},
        _ = email_delivery_worker_task => {},
    };

    Ok(())
}
