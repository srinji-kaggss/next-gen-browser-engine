use thiserror::Error;

#[derive(Error, Debug)]
pub enum NetError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),

    #[error("Response too large: {size} bytes exceeds limit {limit}")]
    ResponseTooLarge { size: usize, limit: usize },

    #[error("Unsupported content type: {0}")]
    UnsupportedContentType(String),
}
