use actix_web::web::{self, HttpResponse};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use tracing::Instrument;
use uuid::Uuid;

pub async fn subscribe(
    form: web::Form<FormData>,
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, HttpResponse> {
    let request_id = Uuid::new_v4();

    let request_span = tracing::info_span!(
        "Adding a new subscriber.",
        %request_id,
        email = %form.email,
        name = %form.name
    );

    // Activate the span... should not do this in async stuff supposedly...
    let _request_span_guard = request_span.enter();


    // tracing::info!(
    //     "request_id {} - Adding '{}' '{}' as a new subscriber.",
    //     request_id,
    //     form.email,
    //     form.name
    // );
    let query_span = tracing::info_span!(
        "Saving new subscriber details in the database"
    );

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
    .execute(db_pool.get_ref())
    .instrument(query_span)
    .await
    .map_err(|e| {
        tracing::error!(
            "request_id {} - Failed to execute query: {:?}",
            request_id,
            e
        );
        HttpResponse::InternalServerError().finish()
    })?;
    tracing::info!(
        "request_id {} - New subscriber details have been saved",
        request_id
    );
    Ok(HttpResponse::Ok().finish())
}

#[derive(Clone, Debug, Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}
