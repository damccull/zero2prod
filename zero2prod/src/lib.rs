use error::ResponseError;
use http::StatusCode;

pub mod authentication;
pub mod configuration;
pub mod domain;
pub mod email_client;
pub mod error;
pub mod idempotency;
pub mod idempotency_remover_worker;
pub mod issue_delivery_worker;
pub mod routes;
pub mod session_state;
pub mod startup;
pub mod telemetry;

pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}

pub fn e400<T>(e: T) -> ResponseError
where
    T: std::fmt::Debug,
    T: std::fmt::Display + 'static,
    T: Into<Box<dyn std::error::Error>>,
{
    ResponseError::from(e).set_status(StatusCode::BAD_REQUEST)
}

pub fn e500<T>(e: T) -> ResponseError
where
    T: std::fmt::Debug,
    T: std::fmt::Display + 'static,
    T: Into<Box<dyn std::error::Error>>,
{
    // ResponseBadRequestError::from(e)
    ResponseError::from(e)
}
