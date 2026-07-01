use axum::http::StatusCode;
use axum::Router;
use tower::ServiceExt;

// Helper to create the test app
fn test_app() -> Router {
    be_api::app()
}

#[tokio::test]
async fn test_health_endpoint() {
    let app = test_app();
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/health")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_fetch_missing_url() {
    let app = test_app();
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/fetch")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_fetch_invalid_url() {
    let app = test_app();
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/fetch?url=not-a-url")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_transpile_empty_body() {
    let app = test_app();
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/transpile")
                .header("content-type", "text/plain")
                .body(axum::body::Body::from(""))
                .unwrap(),
        )
        .await
        .unwrap();
    // Empty JS should still parse (empty module)
    assert!(response.status() == StatusCode::OK || response.status() == StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_transpile_valid_js() {
    let app = test_app();
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/transpile")
                .header("content-type", "text/plain")
                .body(axum::body::Body::from("var x = 1;"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["terms"].is_array());
}

#[tokio::test]
async fn test_load_missing_url() {
    let app = test_app();
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/load")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
