use std::net::TcpListener;

#[tokio::test]
async fn health_check_works() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let address = spawn_app();
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(format!("{}/health_check", address))
        .send()
        .await
        .expect("failed to execute request");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
    Ok(())
}

fn spawn_app() -> String {
    // Set up listener and cache the port
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind random port");
    let port = listener.local_addr().unwrap().port();

    // Start the server
    let server = zero2prod::run(listener);
    let _ = tokio::spawn(server);

    // Return the full address so tests can use it
    format!("http://127.0.0.1:{}", port)
}
