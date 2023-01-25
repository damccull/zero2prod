use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use zero2prod::run;

#[tokio::main]
async fn main() {
    // Set up tracing, see the method definition
    setup_tracing();

    tracing::info!("Starting server and listening on 8000");

    let _ = run().await;
}

/// Sets up a tracing subscriber.
fn setup_tracing() {
    let fmt_layer = fmt::layer().compact().with_target(true);
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    let subscriber = tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer);

    let _ = tracing::subscriber::set_global_default(subscriber)
        .map_err(|_err| eprintln!("Unable to set global default subscriber"));
}
