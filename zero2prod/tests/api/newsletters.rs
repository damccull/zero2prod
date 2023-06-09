use wiremock::{
    matchers::{any, method, path},
    Mock, ResponseTemplate,
};
use zero2prod::routes::newsletters::BodyData;

use crate::{helpers::spawn_app, login::assert_is_redirect_to};

mod newsletter_helpers {
    use wiremock::{
        matchers::{method, path},
        Mock, ResponseTemplate,
    };

    use crate::helpers::{ConfirmationLinks, TestApp};
    /// Use the public API of the applixation under test to create
    /// and unconfirmed subscriber.
    pub(crate) async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks {
        let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

        let _mock_guard = Mock::given(path("/email"))
            .and(method("POST"))
            .respond_with(ResponseTemplate::new(200))
            .named("Create unconfirmed subscriver")
            .expect(1)
            .mount_as_scoped(&app.email_server)
            .await;

        app.post_subscriptions(body.into())
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
    let login_body = serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password,
    });

    let response = app.post_login(&login_body).await;
    assert_is_redirect_to(&response, "/admin/dashboard");

    // Act - Part 2 - Send Newsletter
    let body = BodyData {
        title: "Newsletter title".to_string(),
        text: "Newsletter body as plain text".to_string(),
        html: "<p>Newsletter body ad HTML</p>".to_string(),
    };

    let response = app.post_newsletters(&body).await;

    assert_is_redirect_to(&response, "/admin/newsletters");

    // Act - Part 3 - Follow the redirect
    let html = app.get_admin_newsletters_html().await;

    assert!(html.contains("Successfully sent newsletter"));
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
    let login_body = serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password,
    });

    let response = app.post_login(&login_body).await;
    assert_is_redirect_to(&response, "/admin/dashboard");

    // Act - Part 2 - Send Newsletter
    let body = BodyData {
        title: "Newsletter title".to_string(),
        text: "Newsletter body as plain text".to_string(),
        html: "<p>Newsletter body ad HTML</p>".to_string(),
    };

    let response = app.post_newsletters(&body).await;

    assert_is_redirect_to(&response, "/admin/newsletters");

    // Act - Part 3 - Follow the redirect
    let html = app.get_admin_newsletters_html().await;

    assert!(html.contains("Successfully sent newsletter"));
}

#[tokio::test]
async fn newsletters_returns_422_for_invalid_data() {
    // Arrange
    let app = spawn_app().await;
    let test_cases = vec![
        (
            serde_json::json!({
                "content": {
                    "text":"Newsletter body as plain text",
                    "html": "<p>Newsletter body as HTML</p>",
                }
            }),
            "missing title",
        ),
        (
            serde_json::json!({
                "title": "Newsletter!"
            }),
            "missing content",
        ),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = app.post_newsletters(invalid_body).await;
        // Assert
        assert_eq!(
            422,
            response.status().as_u16(),
            "The API did not fail with 422 Unprocessable Entity when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn requests_missing_authorization_are_rejected() {
    // Arrange
    let app = spawn_app().await;
    let body = BodyData {
        title: "Newsletter title".to_string(),
        text: "Newsletter body as plain text".to_string(),
        html: "<p>Newsletter body ad HTML</p>".to_string(),
    };

    let response = app.post_newsletters(&body).await;

    // Assert
    assert_is_redirect_to(&response, "/login")
}
