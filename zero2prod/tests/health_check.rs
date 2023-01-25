#[tokio::test]
async fn health_check_works() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    spawn_app();
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get("http://127.0.0.1:8000/health_check")
        .send()
        .await
        .expect("failed to execute request");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
    Ok(())
}

fn spawn_app() {
    let server = zero2prod::run();
    let _ = tokio::spawn(server);
}
