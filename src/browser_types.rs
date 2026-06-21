//! Traceability: AXIOM_BRAID_CANONICAL, AXIOM_PRIVACY_TIER, AXIOM_DID_DELEGATION.
use alloc::string::String;
use alloc::vec::Vec;
use sha2::{Digest, Sha256};

pub type Cid = String;

/// Compute a content-addressed CID as a 64-character lowercase hex SHA-256 digest.
///
/// Traceability: AXIOM_BRAID_CANONICAL.
/// Note: this basement seam uses SHA-256 today; the target hash function is BLAKE3.
/// The interface (`Cid` as a 64-hex content-address string) is final.
const HEX_LOWER: [u8; 16] = *b"0123456789abcdef";

pub fn cid_from_bytes(bytes: &[u8]) -> Cid {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(64);
    for byte in digest.iter() {
        hex.push(HEX_LOWER[(byte >> 4) as usize] as char);
        hex.push(HEX_LOWER[(byte & 0x0f) as usize] as char);
    }
    hex
}
pub type Origin = String;
pub type Url = String;
pub type Did = String;

/// A content-addressed fact in the Braid fabric.
#[derive(Debug, Clone, PartialEq)]
pub struct WebAnchor {
    pub cid: Cid,
    pub term_family: TermFamily,
    pub created_at: String,
    pub provenance: Provenance,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TermFamily {
    Element,
    Observation,
    Action,
    Capability,
    Verdict,
    Transition,
    AipState,
    AipPolicy,
    AipAction,
    AipDelegation,
}

impl TermFamily {
    pub fn as_str(&self) -> &'static str {
        match self {
            TermFamily::Element => "web.element",
            TermFamily::Observation => "web.observation",
            TermFamily::Action => "web.action",
            TermFamily::Capability => "web.capability",
            TermFamily::Verdict => "web.verdict",
            TermFamily::Transition => "web.transition",
            TermFamily::AipState => "web.obs.aip_state",
            TermFamily::AipPolicy => "web.obs.aip_policy",
            TermFamily::AipAction => "web.act.aip_action",
            TermFamily::AipDelegation => "web.cap.aip_delegation",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Provenance {
    pub source: String,
    pub input_cids: Vec<Cid>,
    pub trust_class: TrustClass,
    pub did_principal: Option<Did>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustClass {
    SystemPolicy,
    DeveloperPolicy,
    UserIntent,
    TrustedState,
    UntrustedContent,
}

impl TrustClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            TrustClass::SystemPolicy => "SYSTEM_POLICY",
            TrustClass::DeveloperPolicy => "DEVELOPER_POLICY",
            TrustClass::UserIntent => "USER_INTENT",
            TrustClass::TrustedState => "TRUSTED_STATE",
            TrustClass::UntrustedContent => "UNTRUSTED_CONTENT",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivacyTier {
    LocalFull,
    CloudRedacted,
    CloudSelectiveReveal,
    CloudFullContextExplicit,
}

impl PrivacyTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            PrivacyTier::LocalFull => "local_full",
            PrivacyTier::CloudRedacted => "cloud_redacted",
            PrivacyTier::CloudSelectiveReveal => "cloud_selective_reveal",
            PrivacyTier::CloudFullContextExplicit => "cloud_full_context_explicit",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensitivityClass {
    Public,
    LowSensitivity,
    Personal,
    Confidential,
    Secret,
    Authenticator,
    Payment,
    Health,
    Legal,
    Financial,
    ChildOrDependent,
}

impl SensitivityClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            SensitivityClass::Public => "PUBLIC",
            SensitivityClass::LowSensitivity => "LOW_SENSITIVITY",
            SensitivityClass::Personal => "PERSONAL",
            SensitivityClass::Confidential => "CONFIDENTIAL",
            SensitivityClass::Secret => "SECRET",
            SensitivityClass::Authenticator => "AUTHENTICATOR",
            SensitivityClass::Payment => "PAYMENT",
            SensitivityClass::Health => "HEALTH",
            SensitivityClass::Legal => "LEGAL",
            SensitivityClass::Financial => "FINANCIAL",
            SensitivityClass::ChildOrDependent => "CHILD_OR_DEPENDENT",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Risk {
    Low,
    Medium,
    High,
    HumanOnly,
    Denied,
}

impl Risk {
    pub fn as_str(&self) -> &'static str {
        match self {
            Risk::Low => "low",
            Risk::Medium => "medium",
            Risk::High => "high",
            Risk::HumanOnly => "human_only",
            Risk::Denied => "denied",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cid_is_deterministic_and_hex() {
        let a = cid_from_bytes(b"hello");
        let b = cid_from_bytes(b"hello");
        assert_eq!(a, b);
        assert_eq!(a.len(), 64);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn cid_changes_with_content() {
        let a = cid_from_bytes(b"hello");
        let b = cid_from_bytes(b"world");
        assert_ne!(a, b);
    }
}

/// Closed-vocabulary action verbs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionVerb {
    Navigate,
    Observe,
    Click,
    Type,
    Scroll,
    Download,
    Wait,
    ExecuteJs,
    ExecuteWasm,
}

impl ActionVerb {
    pub fn as_str(&self) -> &'static str {
        match self {
            ActionVerb::Navigate => "web.navigate",
            ActionVerb::Observe => "web.observe",
            ActionVerb::Click => "web.click",
            ActionVerb::Type => "web.type",
            ActionVerb::Scroll => "web.scroll",
            ActionVerb::Download => "web.download",
            ActionVerb::Wait => "web.wait",
            ActionVerb::ExecuteJs => "web.execute_js",
            ActionVerb::ExecuteWasm => "web.execute_wasm",
        }
    }
}

/// Verdict from the policy broker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Allow,
    Deny,
    Confirm,
}
