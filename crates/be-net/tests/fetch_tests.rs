use be_net::{NetError, build_client, fetch_url};
use url::Url;

#[tokio::test]
async fn test_build_client() {
    let client = build_client();
    assert!(client.is_ok(), "Client should build successfully");
}

#[tokio::test]
async fn test_fetch_unreachable_host() {
    let client = build_client().unwrap();
    let url = Url::parse("http://127.0.0.1:1").unwrap();
    let result = fetch_url(&client, &url, &[]).await;
    assert!(result.is_err(), "Fetch to unreachable host should fail");
}

#[tokio::test]
async fn test_fetch_unsupported_scheme() {
    let client = build_client().unwrap();
    let url = Url::parse("ftp://example.com/file.txt").unwrap();
    let result = fetch_url(&client, &url, &[]).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        NetError::UnsupportedContentType(msg) => assert!(msg.contains("ftp")),
        _ => panic!("Expected UnsupportedContentType error"),
    }
}

#[tokio::test]
async fn test_fetch_httpbin_get() {
    let client = build_client().unwrap();
    let url = Url::parse("https://httpbin.org/get").unwrap();
    let result = fetch_url(&client, &url, &[]).await;
    assert!(result.is_ok(), "Fetch should succeed: {:?}", result.err());
    let resp = result.unwrap();
    assert_eq!(resp.status, 200);
    assert!(resp.mime_type.contains("json") || resp.mime_type.contains("text"));
    assert!(!resp.body.is_empty());
}

#[tokio::test]
async fn test_fetch_returns_headers() {
    let client = build_client().unwrap();
    let url = Url::parse("https://httpbin.org/headers").unwrap();
    let result = fetch_url(&client, &url, &[]).await;
    assert!(result.is_ok());
    let resp = result.unwrap();
    assert!(!resp.headers.is_empty());
}
