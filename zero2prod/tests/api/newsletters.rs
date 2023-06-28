use std::time::Duration;

use wiremock::{
    matchers::{any, method, path},
    Mock, ResponseTemplate,
};
use zero2prod::routes::newsletters::PUBLISH_SUCCESS_INFO_MESSAGE;

use crate::{helpers::spawn_app, login::assert_is_redirect_to};

mod newsletter_helpers {
    use fake::{
        faker::{internet::en::SafeEmail, name::en::Name},
        Fake,
    };
    use wiremock::{
        matchers::{method, path},
        Mock, ResponseTemplate,
    };

    use crate::helpers::{ConfirmationLinks, TestApp};
    /// Use the public API of the applixation under test to create
    /// and unconfirmed subscriber.
    pub(crate) async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks {
        let name: String = Name().fake();
        let email: String = SafeEmail().fake();
        let body = serde_urlencoded::to_string(serde_json::json!({
            "name":name,
            "email":email,
        }))
        .unwrap();

        let _mock_guard = Mock::given(path("/email"))
            .and(method("POST"))
            .respond_with(ResponseTemplate::new(200))
            .named("Create unconfirmed subscriber")
            .expect(1)
            .mount_as_scoped(&app.email_server)
            .await;

        app.post_subscriptions(body)
            .await
            .error_for_status()
            .unwrap();

        // We now inspect the requests received by the mock Postmark server
        // to retrieve the confirmation link and return it
        let email_request = &app
            .email_server
            .received_requests()
            .await
            .unwrap()
            .pop()
            .unwrap();

        app.get_confirmation_links(email_request)
    }

    pub(crate) async fn create_confirmed_subscriber(app: &TestApp) {
        // We can reuse the other helper to create an unconfirmed subscriber
        // then in this one follow the link to confirm it.
        let confirmation_link = create_unconfirmed_subscriber(app).await;
        reqwest::get(confirmation_link.html)
            .await
            .unwrap()
            .error_for_status()
            .unwrap();
    }
}

use newsletter_helpers::*;

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    // Arrange
    let app = spawn_app().await;
    create_unconfirmed_subscriber(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        // Assert that no requests were sent to Postmark by checking for
        // zero requests on the mock server
        .expect(0)
        .mount(&app.email_server)
        .await;

    // Act - Part 1 - Login
    let response = app.test_user.login(&app).await;
    assert_is_redirect_to(&response, "/admin/dashboard");

    // Act - Part 2 - Send Newsletter
    let body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
    });
    let response = app.post_publish_newsletter(&body).await;

    assert_is_redirect_to(&response, "/admin/newsletters");

    // Act - Part 3 - Follow the redirect
    let html = app.get_publish_newsletter_html().await;

    assert!(html.contains(PUBLISH_SUCCESS_INFO_MESSAGE));
    app.dispatch_all_pending_emails().await;
    //Mock verifies on Drop that we havne't send the newsletter email
}

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    // Arrange
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act - Part 1 - Login
    let response = app.test_user.login(&app).await;
    assert_is_redirect_to(&response, "/admin/dashboard");

    // Act - Part 2 - Send Newsletter
    let body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
    "idempotency_key": uuid::Uuid::new_v4().to_string(),
    });

    let response = app.post_publish_newsletter(&body).await;

    assert_is_redirect_to(&response, "/admin/newsletters");

    // Act - Part 3 - Follow the redirect
    let html = app.get_publish_newsletter_html().await;

    assert!(html.contains(PUBLISH_SUCCESS_INFO_MESSAGE));
    app.dispatch_all_pending_emails().await;
    // Mock verifies on Drop that we have send the emails
}

#[tokio::test]
async fn newsletters_fails_for_invalid_data() {
    // Arrange
    let app = spawn_app().await;
    let test_cases = vec![
        (
            serde_json::json!({
                "text_content": "Newsletter body as plain text",
                "html_content": "<p>Newsletter body as HTML</p>",
            }),
            "missing title",
        ),
        (
            serde_json::json!({
                "title": "Newsletter title",
                "html_content": "<p>Newsletter body as HTML</p>",
            }),
            "missing plaintext content",
        ),
        (
            serde_json::json!({
                "title": "Newsletter title",
                "text_content": "Newsletter body as plain text",
            }),
            "missing HTML content",
        ),
    ];

    // Act - Part 1 - Login
    let response = app.test_user.login(&app).await;
    assert_is_redirect_to(&response, "/admin/dashboard");

    for (invalid_body, error_message) in test_cases {
        // Act - Part 2 - Post the newsletter cases
        let _ = app
            .post_publish_newsletter(&invalid_body)
            .await
            .text()
            .await;

        // Act - Part 3 - Follow the redirect
        let html = app.get_publish_newsletter_html().await;

        assert!(
            html.contains("Part of the form is not filled out"),
            "The API did not fail when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn requests_missing_authorization_are_rejected() {
    // Arrange
    let app = spawn_app().await;
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        // Endpoint expects the idempotency key as part of the
        // form data, not as a header
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });

    let response = app.post_publish_newsletter(&newsletter_request_body).await;

    // Assert
    assert_is_redirect_to(&response, "/login")
}

#[tokio::test]
async fn newsletter_create_is_idempotent() {
    // Arrange
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    let response = app.test_user.login(&app).await;
    assert_is_redirect_to(&response, "/admin/dashboard");
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act - Part 1 - Submit the newsletter form
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        // Endpoint expects the idempotency key as part of the
        // form data, not as a header
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });
    let response = app.post_publish_newsletter(&newsletter_request_body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    // Act - Part 2 - Follow the redirect
    let html_page = app.get_publish_newsletter_html().await;
    assert!(html_page.contains(PUBLISH_SUCCESS_INFO_MESSAGE));

    // Act - Part 3 - Submit the newsletter form _again_
    let response = app.post_publish_newsletter(&newsletter_request_body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    // Act - Part 4 - Follow the redirect
    let html_page = app.get_publish_newsletter_html().await;
    assert!(html_page.contains(PUBLISH_SUCCESS_INFO_MESSAGE));
    app.dispatch_all_pending_emails().await;
    // Mock verifies on drop that we have sent the newsletter
}

#[tokio::test]
async fn concurrent_form_submission_is_handled_gracefully() {
    // Arrange
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.test_user.login(&app).await;
    Mock::given(path("/email"))
        .and(method("POST"))
        // Setting a long delay to ensure the second request arrives before the first one completes
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(2)))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act - Submit two newsletter forms concurrently
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        // Endpoint expects the idempotency key as part of the
        // form data, not as a header
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });
    let response1 = app.post_publish_newsletter(&newsletter_request_body);
    let response2 = app.post_publish_newsletter(&newsletter_request_body);
    let (response1, response2) = tokio::join!(response1, response2);

    assert_eq!(response1.status(), response2.status());
    assert_eq!(
        response1.text().await.unwrap(),
        response2.text().await.unwrap()
    );
    app.dispatch_all_pending_emails().await;
    // Mock verifies on drop that we have only sent the newsletter once
}

#[tokio::test]
async fn transient_errors_get_retried() {
    // Arrange
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(1)
        .expect(1)
        .mount(&app.email_server)
        .await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .up_to_n_times(1)
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act - Part 1 - Login
    app.test_user.login(&app).await;

    // Act - Part 2 - Send Newsletter
    let body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
    });

    app.post_publish_newsletter(&body).await;

    app.dispatch_all_pending_emails().await;

    // Mock verifies on Drop that we have attempted to send the email twice.
    // and the second time should have been successful.
}

#[tokio::test]
async fn old_idempotency_entries_are_cleaned_up() {
    // Arrange
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;

    // Get the new subscriber's user_id
    let user = sqlx::query!("SELECT user_id FROM users")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");
    let user_id = user.user_id;

    // Manually create a test idempotency record. No need for actual data
    // because we're just testing the cleaner cleans old ones out.
    sqlx::query!(
        r#"
        INSERT INTO idempotency (
            idempotency_key,
            user_id,
            created_at 
        )
        VALUES ($1, $2, now() - interval '6 days')
        "#,
        uuid::Uuid::new_v4().to_string(),
        &user_id,
    )
    .execute(&app.db_pool)
    .await
    .expect("Couldn't create an idempotency entry");

    // Create a second one that won't be cleaned because it's too new
    sqlx::query!(
        r#"
        INSERT INTO idempotency (
            idempotency_key,
            user_id,
            created_at 
        )
        VALUES ($1, $2, now())
        "#,
        uuid::Uuid::new_v4().to_string(),
        user_id,
    )
    .execute(&app.db_pool)
    .await
    .expect("Couldn't create an idempotency entry");

    // Act
    app.clean_up_idempotency().await;

    // struct Count {
    //     value: Option<i64>,
    // }

    // Assert
    let count = sqlx::query!(
        r#"
        SELECT COUNT (*) as "value"
        FROM idempotency
        "#
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("Couldn't get the idempotency count");

    assert_eq!(1, count.value.unwrap());
}
