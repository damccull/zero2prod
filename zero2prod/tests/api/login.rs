use crate::helpers::spawn_app;

pub fn assert_is_redirect_to(response: &reqwest::Response, location: &str) {
    assert_eq!(303, response.status().as_u16());
    assert_eq!(location, response.headers().get("Location").unwrap());
}

#[tokio::test]
async fn an_error_flash_message_is_set_on_failure() {
    // Arrange
    let app = spawn_app().await;
    // Act
    let login_body = serde_json::json!({
        "username": "random-username",
        "password":"random-password"
    });
    let response = app.post_login(&login_body).await;

    let flash_cookie = response.cookies().find(|c| c.name() == "_flash").unwrap();

    // Assert
    assert_is_redirect_to(&response, "/login");
    assert_eq!("Authentication failed", flash_cookie.value());
}
