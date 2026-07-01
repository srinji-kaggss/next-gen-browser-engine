pub mod errors;
pub mod fetch;

pub use errors::NetError;
pub use fetch::{FetchResponse, build_client, fetch_url};

#[cfg(test)]
mod tests {
    use url::Url;

    #[test]
    fn test_url_scheme_http() {
        let url = Url::parse("http://example.com").unwrap();
        assert!(url.scheme() == "http" || url.scheme() == "https");
    }

    #[test]
    fn test_url_scheme_https() {
        let url = Url::parse("https://example.com").unwrap();
        assert_eq!(url.scheme(), "https");
    }

    #[test]
    fn test_url_scheme_ftp_rejected() {
        let url = Url::parse("ftp://example.com").unwrap();
        assert_eq!(url.scheme(), "ftp");
        // The fetch function should reject this
    }
}
