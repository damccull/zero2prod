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

    assert!(html.contains("The newsletter issue has been published"));
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

    assert!(html.contains("The newsletter issue has been published"));
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

    // Mock verifies on drop that we have only sent the newsletter once
}

// Helper fn for common mocking setup
fn when_sending_an_email() -> wiremock::MockBuilder {
    Mock::given(path("/email")).and(method("POST"))
}

#[tokio::test]
async fn transient_errors_do_not_cause_duplicate_deliveries_on_retries() {
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

    // Make two subscribers instead of one
    create_confirmed_subscriber(&app).await;
    create_confirmed_subscriber(&app).await;

    app.test_user.login(&app).await;

    // Part 1 - Submit newsletter form
    // Email deliver fails for the second subscriber
    when_sending_an_email()
        .respond_with(ResponseTemplate::new(200))
        .up_to_n_times(1)
        .expect(1)
        .mount(&app.email_server)
        .await;
    when_sending_an_email()
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(1)
        .expect(1)
        .mount(&app.email_server)
        .await;

    let response = app.post_publish_newsletter(&newsletter_request_body).await;
    assert_eq!(500, response.status().as_u16());

    // Part 2 - Retry submitting the form
    // Email delivery will succeed for both subscribers now
    when_sending_an_email()
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .named("Delivery retry")
        .mount(&app.email_server)
        .await;

    let response = app.post_publish_newsletter(&newsletter_request_body).await;
    assert_eq!(303, response.status().as_u16());
    // Mock verifies on Drop that we did not send out duplicates
}
