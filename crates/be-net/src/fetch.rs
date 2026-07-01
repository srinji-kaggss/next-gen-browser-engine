use crate::errors::NetError;
use be_capability::Capability;
use reqwest::Client;
use url::Url;

/// Maximum number of redirects to follow.
const MAX_REDIRECTS: usize = 10;

/// Maximum response body size (10 MB).
const MAX_BODY_SIZE: usize = 10 * 1024 * 1024;

/// A fetched response from the network.
#[derive(Debug)]
pub struct FetchResponse {
    /// Final URL after redirects.
    pub url: Url,
    /// HTTP status code.
    pub status: u16,
    /// Response headers.
    pub headers: Vec<(String, String)>,
    /// Response body bytes.
    pub body: Vec<u8>,
    /// Detected MIME type.
    pub mime_type: String,
    /// Cookies set during the fetch.
    pub cookies: Vec<String>,
}

/// Fetch a URL and return the response.
/// Adapted from logic-os-kernel canvas-backend/src/auth.rs PulseAttestor pattern.
pub async fn fetch_url(
    client: &Client,
    url: &Url,
    _required_capabilities: &[Capability],
) -> Result<FetchResponse, NetError> {
    // Validate URL scheme
    match url.scheme() {
        "http" | "https" => {}
        scheme => {
            return Err(NetError::UnsupportedContentType(format!(
                "Unsupported scheme: {}",
                scheme
            )));
        }
    }

    let response = client
        .get(url.as_str())
        .send()
        .await
        .map_err(NetError::Http)?;

    // Follow redirects (reqwest does this automatically, but we track them)
    let final_url = response.url().clone();
    let status = response.status().as_u16();

    // Collect headers
    let headers: Vec<(String, String)> = response
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    // Detect MIME from Content-Type header
    let mime_type = headers
        .iter()
        .find(|(k, _)| k.to_lowercase() == "content-type")
        .map(|(_, v)| {
            v.split(';')
                .next()
                .unwrap_or("application/octet-stream")
                .to_string()
        })
        .unwrap_or_else(|| "application/octet-stream".to_string());

    // Collect body with size limit
    let body = response.bytes().await.map_err(NetError::Http)?;

    if body.len() > MAX_BODY_SIZE {
        return Err(NetError::ResponseTooLarge {
            size: body.len(),
            limit: MAX_BODY_SIZE,
        });
    }

    Ok(FetchResponse {
        url: final_url,
        status,
        headers,
        body: body.to_vec(),
        mime_type,
        cookies: vec![], // TODO: extract from cookie jar
    })
}

/// Build a default reqwest client with browser-like settings.
pub fn build_client() -> Result<Client, NetError> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::limited(MAX_REDIRECTS))
        .gzip(true)
        .brotli(true)
        .deflate(true)
        .http2_adaptive_window(true)
        .build()
        .map_err(NetError::Http)?;

    Ok(client)
}
