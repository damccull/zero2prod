use std::ops::Deref;

use axum::{
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use axum_session::SessionRedisPool;
use http::Request;
use uuid::Uuid;

use crate::session_state::TypedSession;

pub async fn reject_anonymous_users<B>(
    session: TypedSession<SessionRedisPool>,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, impl IntoResponse> {
    match session.get_user_id() {
        Some(uid) => {
            request.extensions_mut().insert(UserId(uid));
            Ok(next.run(request).await)
        }
        None => {
            tracing::error!("User has not logged in.");
            Err(Redirect::to("/login"))
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct UserId(Uuid);

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Deref for UserId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
