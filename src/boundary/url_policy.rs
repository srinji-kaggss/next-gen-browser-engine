//! Traceability: AXIOM_CONFINEMENT, AXIOM_ANTIVIRUS.
use crate::browser_types::Url;
use alloc::string::{String, ToString};

/// Deny-first URL and origin policy.
///
/// Default posture:
/// - Only `http` and `https` schemes are allowed.
/// - Loopback or private-IP destinations require an explicit capability.
/// - Non-ASCII hostnames and userinfo are rejected.
/// - Empty or missing origins are rejected.
pub struct UrlPolicy {
    /// If true, private-IP / loopback URLs are denied unless explicitly allowed.
    deny_private: bool,
}

impl UrlPolicy {
    pub fn new() -> Self {
        Self { deny_private: true }
    }

    /// Returns `Ok(())` if the URL may be navigated, otherwise `Err(reason)`.
    /// This is a deny-first gate: anything not explicitly allowed is rejected.
    pub fn allowed(&self, url: &Url) -> Result<(), &'static str> {
        let lower = url.to_lowercase();

        if lower.is_empty() {
            return Err("empty url");
        }

        // Scheme check.
        if !(lower.starts_with("http://") || lower.starts_with("https://")) {
            return Err("disallowed scheme");
        }

        // Reject URLs with embedded credentials.
        if url.contains('@') {
            return Err("url contains userinfo");
        }

        // Extract origin-ish host for private-IP screening.
        let host = extract_host(&lower);
        if host.is_empty() {
            return Err("missing host");
        }

        if is_private_or_loopback(&host) && self.deny_private {
            return Err("private or loopback origin denied");
        }

        // Non-ASCII host check (simple pre-IDNA safety).
        if host.bytes().any(|b| b >= 0x80) {
            return Err("non-ascii host");
        }

        Ok(())
    }

    /// Disable private-IP / loopback deny for tests or trusted local modes.
    pub fn allow_private(mut self) -> Self {
        self.deny_private = false;
        self
    }
}

impl Default for UrlPolicy {
    fn default() -> Self {
        Self::new()
    }
}

fn extract_host(url: &str) -> String {
    let after_scheme = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
        .unwrap_or(url);
    let without_path = after_scheme.split('/').next().unwrap_or(after_scheme);
    let without_port = without_path.split(':').next().unwrap_or(without_path);
    without_port.to_string()
}

fn is_private_or_loopback(host: &str) -> bool {
    if host == "localhost" {
        return true;
    }
    if host.starts_with("127.") || host == "::1" {
        return true;
    }
    if host.starts_with("10.") || host.starts_with("192.168.") {
        return true;
    }
    if let Some(second) = host.strip_prefix("172.") {
        if let Some(octet) = second.split('.').next().and_then(|s| s.parse::<u8>().ok()) {
            if (16..=31).contains(&octet) {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allow_https_example() {
        let p = UrlPolicy::new();
        assert!(p.allowed(&"https://example.com/path".to_string()).is_ok());
    }

    #[test]
    fn deny_non_http_scheme() {
        let p = UrlPolicy::new();
        assert_eq!(
            p.allowed(&"ftp://example.com".to_string()),
            Err("disallowed scheme")
        );
    }

    #[test]
    fn deny_empty() {
        let p = UrlPolicy::new();
        assert_eq!(p.allowed(&"".to_string()), Err("empty url"));
    }

    #[test]
    fn deny_userinfo() {
        let p = UrlPolicy::new();
        assert_eq!(
            p.allowed(&"https://user:pass@example.com".to_string()),
            Err("url contains userinfo")
        );
    }

    #[test]
    fn deny_localhost() {
        let p = UrlPolicy::new();
        assert_eq!(
            p.allowed(&"http://localhost:8080".to_string()),
            Err("private or loopback origin denied")
        );
    }

    #[test]
    fn allow_localhost_when_private_allowed() {
        let p = UrlPolicy::new().allow_private();
        assert!(p.allowed(&"http://localhost:8080".to_string()).is_ok());
    }

    #[test]
    fn deny_private_ip() {
        let p = UrlPolicy::new();
        assert_eq!(
            p.allowed(&"http://192.168.1.1".to_string()),
            Err("private or loopback origin denied")
        );
    }

    #[test]
    fn deny_non_ascii_host() {
        let p = UrlPolicy::new();
        assert_eq!(
            p.allowed(&"https://exämple.com".to_string()),
            Err("non-ascii host")
        );
    }
}
