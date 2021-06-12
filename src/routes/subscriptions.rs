use std::ops::Deref;
use std::sync::Arc;

use actix_web::web::{self, HttpResponse};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn subscribe(
    form: web::Form<FormData>,
    db_pool: web::Data<Arc<PgPool>>,
) -> Result<HttpResponse, HttpResponse> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(db_pool.get_ref().deref())
    .await
    .map_err(|e| {
        eprintln!("Failed to execute query: {}", e);
        HttpResponse::InternalServerError().finish()
    })?;

    Ok(HttpResponse::Ok().finish())
}

#[derive(Clone, Debug, Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}
