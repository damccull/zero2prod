use axum::{
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use axum_session::SessionRedisPool;
use http::Request;

use crate::session_state::TypedSession;

pub async fn reject_anonymous_users<B>(
    session: TypedSession<SessionRedisPool>,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, impl IntoResponse> {
    match session.get_user_id() {
        Some(_) => Ok(next.run(request).await),
        None => {
            tracing::error!("User has not logged in.");
            Err(Redirect::to("/login"))
        }
    }
}
