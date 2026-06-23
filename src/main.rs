//! Browser engine entrypoint.
//!
//! For MVP: starts the HTTP API server.

use be_api::router;

#[tokio::main]
async fn main() {
    println!("Browser engine starting...");

    let app = router();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Listening on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}
