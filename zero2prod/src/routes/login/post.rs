use axum::response::{IntoResponse, Redirect};

#[tracing::instrument(name = "Login posted")]
pub async fn login() -> impl IntoResponse {
    Redirect::to("/").into_response()
}
