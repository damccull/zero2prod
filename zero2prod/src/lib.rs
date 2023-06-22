use error::ResponseInternalServerError;

pub mod authentication;
pub mod configuration;
pub mod domain;
pub mod email_client;
pub mod error;
pub mod idempotency;
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

pub fn e500<T>(e: T) -> ResponseInternalServerError<T>
where
    T: std::fmt::Debug + std::fmt::Display + 'static,
{
    ResponseInternalServerError::from(e)
}
