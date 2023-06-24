use axum::response::{IntoResponse, Response};
use http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use hyper::body::to_bytes;
use sqlx::{postgres::PgHasArrayType, PgPool};
use uuid::Uuid;

use super::IdempotencyKey;

#[tracing::instrument(name = "Getting cached newsletter response", skip(pool))]
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
        let mut headers = HeaderMap::new();
        for HeaderPairRecord { name, value } in r.response_headers {
            let nam = HeaderName::try_from(name)?;
            let val = HeaderValue::try_from(value)?;
            headers.insert(nam, val);
        }
        tracing::trace!("SAVED HEADERS {:#?}", &headers);
        let resp = (status_code, headers, r.response_body).into_response();
        Ok(Some(resp))
    } else {
        Ok(None)
    }
}

#[tracing::instrument(name = "Saving cached newsletter response", skip(pool, http_response))]
pub async fn save_response(
    pool: &PgPool,
    idempotency_key: &IdempotencyKey,
    user_id: Uuid,
    http_response: Response,
) -> Result<Response, anyhow::Error> {
    let (response_head, body) = http_response.into_parts();
    let body = to_bytes(body).await.map_err(|e| anyhow::anyhow!("{}", e))?;
    let status_code = response_head.status.as_u16() as i16;
    let headers = {
        let mut h = Vec::with_capacity(response_head.headers.len());
        for (name, value) in response_head.headers.iter() {
            let name = name.as_str().to_owned();
            let value = value.as_bytes().to_owned();
            h.push(HeaderPairRecord { name, value });
        }
        h
    };

    sqlx::query_unchecked!(
        r#"
        INSERT INTO idempotency (
            user_id,
            idempotency_key,
            response_status_code,
            response_headers,
            response_body,
            created_at
        )
        VALUES ($1, $2, $3, $4, $5, now())
        "#,
        user_id,
        idempotency_key.as_ref(),
        status_code,
        headers,
        body.as_ref()
    )
    .execute(pool)
    .await?;
    let http_response = (response_head, body).into_response();
    Ok(http_response)
}

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "header_pair")]
struct HeaderPairRecord {
    name: String,
    value: Vec<u8>,
}

impl PgHasArrayType for HeaderPairRecord {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("_header_pair")
    }
}
