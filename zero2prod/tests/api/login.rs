use crate::helpers::spawn_app;

pub fn assert_is_redirect_to(response: &reqwest::Response, location: &str) {
    assert_eq!(303, response.status().as_u16());
    assert_eq!(location, response.headers().get("Location").unwrap());
}

#[tokio::test]
async fn an_error_flash_message_is_set_on_failure() {
    // Arrange
    let app = spawn_app().await;

    // Act 1
    let login_body = serde_json::json!({
        "username": "random-username",
        "password":"random-password"
    });
    let response = app.post_login(&login_body).await;

    // Assert 1
    assert_is_redirect_to(&response, "/login");

    // Act 2
    let html_page = app.get_login_html().await;

    // Assert 2
    // Follow the redirect
    assert!(html_page.contains(r#"Authentication failed"#));

    // Act 3
    //Reload the page
    let html_page = app.get_login_html().await;

    // Assert 3
    // Should NOT have the 'Authentication failed' message
    assert!(!html_page.contains(r#"Authentication failed"#));
}

#[tokio::test]
async fn redirect_to_admin_dashboard_after_login_success() {
    // Arrange
    let app = spawn_app().await;

    // Act - Part 1 - Login
    let login_body = serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password,
    });

    let response = app.post_login(&login_body).await;

    // Assert - Part 1
    assert_is_redirect_to(&response, "/admin/dashboard");

    // Act - Part 2 - Follow the redirect
    let html_page = app.get_admin_dashboard_html().await;
    assert!(html_page.contains(&format!("Welcome {}", app.test_user.username)));

    // Assert - Part 2
}
