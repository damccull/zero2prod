use axum::Router;
use tower::ServiceBuilder;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::{Level, Subscriber};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan, MakeWriter},
    prelude::*,
    EnvFilter,
};

/// Sets up a tracing subscriber.
pub fn get_subscriber<Sink>(
    _name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let filter_layer =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));

    let fmt_layer = fmt::layer()
        .compact()
        .with_target(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::ACTIVE)
        .with_writer(sink);

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
}

/// Sets the global default subscriber. Should only be called once.
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    let _ = tracing::subscriber::set_global_default(subscriber)
        .map_err(|_err| eprintln!("Unable to set global default subscriber"));
}
pub trait RouterExt {
    fn add_axum_tracing_layer(self) -> Router;
}

impl RouterExt for Router {
    fn add_axum_tracing_layer(self) -> Self {
        self.layer(
            ServiceBuilder::new().layer(
                TraceLayer::new_for_http().make_span_with(
                    DefaultMakeSpan::new()
                        .include_headers(true)
                        .level(Level::INFO),
                ),
            ),
        )
    }
}
