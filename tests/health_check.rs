use std::net::TcpListener;

#[actix_rt::test]
async fn health_check_works() {
    // Arrange
    let address = spawn_app();

    // Use reqwest to test the endpoint over an http request
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

// Somehow launch the app in the background
fn spawn_app() -> String {
    // Bind a random OS port
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    // Get the random port from the listener
    let port = listener.local_addr().unwrap().port();
    // Use the listener for spinning up a server
    let server = zero2prod::run(listener).expect("Failed to bind address");
    // Execute the server in an Executor
    let _ = tokio::spawn(server);

    // Return the port the the called
    format!("http://127.0.0.1:{}", port)
}
