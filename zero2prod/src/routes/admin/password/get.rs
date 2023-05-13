use axum::response::IntoResponse;
use http::StatusCode;

use crate::error::ResponseInternalServerError;

pub async fn change_password_form(
) -> Result<impl IntoResponse, ResponseInternalServerError<anyhow::Error>> {
    Ok(StatusCode::OK.into_response())
}
