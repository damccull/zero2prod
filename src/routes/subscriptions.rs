use actix_web::web::{self, HttpResponse};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, db_pool),
    fields(
        request_id = %Uuid::new_v4(),
        email = %form.email,
        name = %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, HttpResponse> {
    insert_subscriber(&db_pool, &form)
        .await
        .map_err(|_| HttpResponse::InternalServerError().finish())?;
    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(form, db_pool)
)]
pub async fn insert_subscriber(db_pool: &PgPool, form: &FormData) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4);
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}

#[derive(Clone, Debug, Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}
