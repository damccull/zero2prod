
#[actix_rt::test]
async fn health_check_works() {
    // Arrange
    spawn_app();

    // Use reqwest to test the endpoint over an http request
    let client = reqwest::Client::new();

    // Act
    let response = client.get("http://127.0.0.1:8000/health_check").send().await.expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());

}

// Somehow launch the app in the background
    let server = zero2prod::run().expect("Failed to bind address");
fn spawn_app() {
    let _ = tokio::spawn(server);
}