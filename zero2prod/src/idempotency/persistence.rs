use std::io::Read;

use axum::{body::Full, response::Response};
use axum_macros::debug_handler;
use http::{HeaderName, HeaderValue, StatusCode};
use hyper::body::HttpBody;
use sqlx::PgPool;
use uuid::Uuid;

use super::IdempotencyKey;

// #[debug_handler(state = crate::startup::AppState)]
pub async fn get_saved_response(
    pool: &PgPool,
    idempotency_key: &IdempotencyKey,
    user_id: Uuid,
) -> Result<Option<Response>, anyhow::Error> {
    let saved_response = sqlx::query!(
        r#"
        SELECT
            response_status_code,
            response_headers as "response_headers: Vec<HeaderPairRecord>",
            response_body
        FROM idempotency
        WHERE
            user_id = $1 AND
            idempotency_key = $2
        "#,
        user_id,
        idempotency_key.as_ref()
    )
    .fetch_optional(pool)
    .await?;

    if let Some(r) = saved_response {
        let status_code = StatusCode::from_u16(r.response_status_code.try_into()?)?;
        let mut response = Response::builder().status(status_code);
        if let Some(hdrs) = response.headers_mut() {
            for HeaderPairRecord { name, value } in r.response_headers {
                let nam = HeaderName::try_from(name)?;
                let val = HeaderValue::try_from(value)?;
                tracing::trace!("{:?}", &val);
                hdrs.insert(nam, val);
            }
        }
        let body = Full::new(axum::body::Bytes::from(r.response_body.as_slice())).boxed_unsync();
        let resp = response.body(body)?;
        Ok(Some(resp))
    } else {
        Ok(None)
    }
}

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "header_pair")]
struct HeaderPairRecord {
    name: String,
    value: Vec<u8>,
}
