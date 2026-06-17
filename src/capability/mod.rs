use crate::{browser_types::*, ActionVerb};
use alloc::string::String;
use alloc::vec::Vec;

/// DAL-A capability broker: signed, scoped, attenuation-only tokens.
pub struct CapabilityBroker;

impl CapabilityBroker {
    pub fn new() -> Self {
        Self
    }

    /// Issue a capability token bound to a subject and scope.
    pub fn issue(
        &self,
        _issuer: &str,
        _subject: &str,
        _scope: Vec<String>,
        _attenuation: Attenuation,
    ) -> Result<WebCapability, &'static str> {
        todo!("issue signed capability")
    }

    /// Verify and attenuate a capability to a narrower scope.
    pub fn attenuate(
        &self,
        _parent: &WebCapability,
        _narrower: Attenuation,
    ) -> Result<WebCapability, &'static str> {
        todo!("attenuate-only delegation")
    }
}

impl Default for CapabilityBroker {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WebCapability {
    pub issuer_did: Did,
    pub subject_did: Did,
    pub scope: Vec<String>,
    pub privacy_tier: PrivacyTier,
    pub attenuation: Attenuation,
    pub issued_at: String,
    pub expires_at: String,
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attenuation {
    pub allowed_verbs: Vec<ActionVerb>,
    pub allowed_origins: Vec<Origin>,
    pub max_bytes: usize,
    pub max_calls: usize,
}
