use axum::{
    body::HttpBody,
    response::{IntoResponse, Response},
    Router,
};
use http::StatusCode;
use tower::ServiceBuilder;
use tower_http::{
    request_id::MakeRequestUuid,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    ServiceBuilderExt,
};
use tracing::{Level, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{fmt::MakeWriter, prelude::*, EnvFilter, Registry};

/// Sets up a tracing subscriber.
pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let filter_layer =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));

    // --This code uses tracing-subscriber--
    // let fmt_layer = fmt::layer()
    //     .compact()
    //     .with_target(true)
    //     .with_line_number(true)
    //     .with_span_events(FmtSpan::NONE)
    //     .with_writer(sink);

    // tracing_subscriber::registry()
    //     .with(filter_layer)
    //     .with(fmt_layer)
    // ----

    let bunyan_format = BunyanFormattingLayer::new(name, sink);

    Registry::default()
        .with(filter_layer)
        .with(JsonStorageLayer)
        .with(bunyan_format)
}

/// Sets the global default subscriber. Should only be called once.
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    let _ = tracing::subscriber::set_global_default(subscriber)
        .map_err(|_err| eprintln!("Unable to set global default subscriber"));
}

pub trait RouterExt {
    fn add_axum_tracing_layer(self) -> Self;
}

impl<S, B> RouterExt for Router<S, B>
where
    B: HttpBody + Send + 'static,
    S: Clone + Send + Sync + 'static,
{
    fn add_axum_tracing_layer(self) -> Self {
        self.layer(
            ServiceBuilder::new()
                .set_x_request_id(MakeRequestUuid)
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(
                            DefaultMakeSpan::new()
                                .include_headers(true)
                                .level(Level::INFO),
                        )
                        .on_response(DefaultOnResponse::new().include_headers(true)),
                )
                .propagate_x_request_id(),
        )
    }
}

#[derive(Debug)]
pub struct MyErrorResponse {
    status_code: StatusCode,
    source: Option<Box<dyn std::error::Error>>,
}

impl MyErrorResponse {
    /// Creates a new [`MyErrorResponse`] with the specified [`StatusCode`] and the
    /// error source set to None.
    pub fn new(status_code: StatusCode) -> Self {
        Self {
            status_code,
            source: None,
        }
    }

    /// Sets the source to the [`Error`](std::error::Error) supplied.
    pub fn source(mut self, source: &'static (dyn std::error::Error + 'static)) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    /// Sets the status code to a valid [`StatusCode`]
    pub fn status_code(mut self, status_code: StatusCode) -> Self {
        self.status_code = status_code;
        self
    }
}

impl<T> From<T> for MyErrorResponse
where
    T: std::error::Error + 'static,
{
    fn from(error: T) -> Self {
        Self {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            source: Some(Box::new(error)),
        }
    }
}

impl IntoResponse for MyErrorResponse {
    fn into_response(self) -> Response {
        match &self.source {
            Some(e) => tracing::error!("Caused by:\n\t{:?}", e),
            None => tracing::error!("No source error attached. Cause unknown."),
        }
        self.status_code.into_response()
    }
}
