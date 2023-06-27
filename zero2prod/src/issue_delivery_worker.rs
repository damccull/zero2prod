use crate::{domain::SubscriberEmail, email_client::EmailClient};
use sqlx::{PgPool, Postgres, Transaction};
use tracing::{field::display, Span};
use uuid::Uuid;

#[tracing::instrument(
    skip_all,
    fields(
        newsletter_issue_id=tracing::field::Empty,
        subscriber_email=tracing::field::Empty,
     ),
     err
)]
async fn try_execute_task(pool: &PgPool, email_client: &EmailClient) -> Result<(), anyhow::Error> {
    if let Some(task) = dequeue_task(pool).await? {
        Span::current()
            .record("newsletter_issue_id", &display(task.issue_id))
            .record("subscriber_email", &display(&task.email));

        match SubscriberEmail::parse(task.email.clone()) {
            Ok(email) => {
                let issue = get_issue(pool, task.issue_id).await?;
                if let Err(e) = email_client
                    .send_email(
                        &email,
                        &issue.title,
                        &issue.html_content,
                        &issue.text_content,
                    )
                    .await
                {
                    tracing::error!(
                            error.cause_chain = ?e,
                            error.message = %e,
                            "Failed to deliver issue to a confirmed subscriber. Skipping.",
                    );
                }
            }
            Err(e) => {
                tracing::error!(
                    error.cause_chain = ?e,
                        error.message = %e,
                        "Skipping a confirmed subscriber. Their stored contact details are invalid.",
                );
            }
        }
        delete_task(task).await?;
    }
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn dequeue_task(pool: &PgPool) -> Result<Option<EmailTask>, anyhow::Error> {
    let mut transaction = pool.begin().await?;

    let r = sqlx::query!(
        r#"
        SELECT newsletter_issue_id, subscriber_email
        FROM issue_delivery_queue
        FOR UPDATE
        SKIP LOCKED
        LIMIT 1
        "#
    )
    .fetch_optional(&mut transaction)
    .await?;

    if let Some(r) = r {
        Ok(Some(EmailTask {
            transaction,
            issue_id: r.newsletter_issue_id,
            email: r.subscriber_email,
        }))
    } else {
        Ok(None)
    }
}

#[tracing::instrument(skip_all)]
async fn get_issue(pool: &PgPool, issue_id: Uuid) -> Result<NewsletterIssue, anyhow::Error> {
    let issue = sqlx::query_as!(
        NewsletterIssue,
        r#"
        SELECT title, text_content, html_content
        FROM newsletter_issues
        WHERE
            newsletter_issue_id = $1
        "#,
        issue_id
    )
    .fetch_one(pool)
    .await?;
    Ok(issue)
}

#[tracing::instrument(skip_all)]
async fn delete_task(mut task: EmailTask) -> Result<(), anyhow::Error> {
    sqlx::query!(
        r#"
        DELETE FROM issue_delivery_queue
        WHERE
            newsletter_issue_id = $1 AND
            subscriber_email = $2
        "#,
        task.issue_id,
        task.email,
    )
    .execute(&mut task.transaction)
    .await?;
    task.transaction.commit().await?;
    Ok(())
}

type PgTransaction = Transaction<'static, Postgres>;
struct EmailTask {
    transaction: PgTransaction,
    issue_id: Uuid,
    email: String,
}

struct NewsletterIssue {
    title: String,
    text_content: String,
    html_content: String,
}
