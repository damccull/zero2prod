use tracing::Subscriber;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Sets up a tracing subscriber.
pub fn get_subscriber(_name: String, env_filter: String) -> impl Subscriber + Send + Sync {
    let filter_layer =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));

    let fmt_layer = fmt::layer().compact().with_target(true);

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
}

/// Sets the global default subscriber. Should only be called once.
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    let _ = tracing::subscriber::set_global_default(subscriber)
        .map_err(|_err| eprintln!("Unable to set global default subscriber"));
}
