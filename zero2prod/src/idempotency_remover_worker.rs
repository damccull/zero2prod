use std::time::Duration;

use sqlx::PgPool;

use crate::{configuration::Settings, startup::get_db_pool};

pub async fn run_worker_until_stopped(configuration: Settings) -> Result<(), anyhow::Error> {
    // Set up the worker
    let connection_pool = get_db_pool(&configuration.database);
    worker_loop(connection_pool).await
}

async fn worker_loop(pool: PgPool) -> Result<(), anyhow::Error> {
    loop {
        remove_old_idempotency_entries(&pool).await?;
        tokio::time::sleep(Duration::from_secs(60 * 60 * 24)).await;
    }
}

#[tracing::instrument(skip_all)]
pub async fn remove_old_idempotency_entries(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        DELETE FROM idempotency
        WHERE
            created_at < now() - interval '5 days'
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}
