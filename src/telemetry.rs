use tracing::{subscriber::set_global_default, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{fmt::MakeWriter, layer::SubscriberExt, EnvFilter, Registry};

/// Compose multiple layers in a `tracing`'s subscriber.
///
/// # Implementation Notes
///
/// We are using `impl Subscriber` as a return type to avoid having
/// to spell out the actual type of the returned subscriber, which is
/// indeed quite complex.
/// We need to explicitly call out that the returned subscriber is `Send`
/// and `Sync` to make is possible to pass it to `init_subscriber` later on.
pub fn get_subscriber(
    name: String,
    env_filter: String,
    // A function that returns a sink - a place a log can be written to
    sink: impl MakeWriter + Send + Sync + 'static,
) -> impl Subscriber + Send + Sync {
    // Fall back to printing all spans at the level of `env_filter` or above if the
    // RUST_LOG environment variable is not set.
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));

    let formatting_layer = BunyanFormattingLayer::new(name, sink);

    // The `with` method is provided by `SubscriberExt`, an extension
    // trait for `Subscriber` exposed by `tracing_subscriber`
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

/// Register a subscriber as global default to process span data.
///
/// It should only be called once!
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    // Redirect all `log`'s events to our subscriber
    LogTracer::init().expect("Failed to set logger");

    // `set_global_default` can be used by applications to specify what subscriber
    // should be used to process spans.
    set_global_default(subscriber).expect("Failed to set subscriber");
}
