use axum::response::IntoResponse;
use http::StatusCode;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct ResponseBadRequestError<T>(#[from] T);

impl<T: std::fmt::Debug> IntoResponse for ResponseBadRequestError<T> {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{:?}", self);
        StatusCode::BAD_REQUEST.into_response()
    }
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct ResponseInternalServerError<T>(#[from] T);

impl<T: std::fmt::Debug> IntoResponse for ResponseInternalServerError<T> {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{:?}", self);
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}
