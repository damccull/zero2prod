use axum::response::{IntoResponse, Redirect};
use axum_flash::Flash;
use axum_session::SessionRedisPool;

use crate::{error::ResponseInternalServerError, session_state::TypedSession};

pub async fn log_out(
    flash: Flash,
    session: TypedSession<SessionRedisPool>,
) -> Result<impl IntoResponse, ResponseInternalServerError<anyhow::Error>> {
    session.log_out();
    let flash = flash.info("You have successfully logged out.");
    Ok((flash, Redirect::to("/login")).into_response())
}
