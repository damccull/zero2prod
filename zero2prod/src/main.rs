use std::fmt::{Debug, Display};

use tokio::task::JoinError;
use zero2prod::{
    configuration::get_configuration, idempotency_remover_worker, issue_delivery_worker,
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
    let email_delivery_worker_task = tokio::spawn(issue_delivery_worker::run_worker_until_stopped(
        configuration.clone(),
    ));
    let idempotency_cleaner_task = tokio::spawn(
        idempotency_remover_worker::run_worker_until_stopped(configuration),
    );

    tokio::select! {
        o = app_task => report_exit("API", o),
        o = email_delivery_worker_task => report_exit("Email Delivery Worker", o),
        o = idempotency_cleaner_task => report_exit("Idempotency Cleaner Worker", o)
    };

    Ok(())
}

fn report_exit(task_name: &str, outcome: Result<Result<(), impl Debug + Display>, JoinError>) {
    match outcome {
        Ok(Ok(())) => {
            tracing::info!("{} has exited", task_name)
        }
        Ok(Err(e)) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{} failed",
                task_name
            )
        }
        Err(e) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{} task failed to complete",
                task_name
            )
        }
    }
}
