//! Traceability: AXIOM_CONFINEMENT, AXIOM_ANTIVIRUS.
use crate::browser_types::Url;

/// Deny-first URL and origin policy.
pub struct UrlPolicy;

impl UrlPolicy {
    pub fn new() -> Self {
        Self
    }

    pub fn allowed(&self, _url: &Url) -> Result<(), &'static str> {
        todo!("deny-first origin / scheme / private-IP policy")
    }
}

impl Default for UrlPolicy {
    fn default() -> Self {
        Self::new()
    }
}
