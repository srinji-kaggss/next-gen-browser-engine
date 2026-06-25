//! Traceability: AXIOM_BRAID_CANONICAL, AXIOM_PRIVACY_TIER, AXIOM_DID_DELEGATION.
use alloc::string::String;
use alloc::vec::Vec;
/// The canonical content identifier IS Braid's domain-separated BLAKE3 CID
/// (`braid_ir::Cid`) — the one scheme shared verbatim with logic-os. We do not
/// declare a second CID. Hex is only the text-wire form (`to_hex`/`from_hex`).
pub use braid_ir::Cid;

/// Domain for an observation fact's own content address (D8 separation — a
/// Braid CID under one domain can never collide one under another).
pub const WEB_ANCHOR_DOMAIN: &[u8] = b"lw.browser.anchor.v0";
/// Domain for an element identity derived from its stable DOM path.
pub const WEB_ELEMENT_DOMAIN: &[u8] = b"lw.browser.element.v0";
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
    use alloc::collections::BTreeSet;
    use alloc::string::ToString;

    #[test]
    fn domain_separation_distinguishes_anchor_and_element_cids() {
        // The CID mechanics are Braid's (and tested there). What is OURS to
        // prove is the domain choice: the same bytes under our two domains must
        // never collide (D8), and the address is deterministic.
        let anchor = Cid::compute(WEB_ANCHOR_DOMAIN, b"body>div>a:0");
        let element = Cid::compute(WEB_ELEMENT_DOMAIN, b"body>div>a:0");
        assert_ne!(anchor, element);
        assert_eq!(anchor, Cid::compute(WEB_ANCHOR_DOMAIN, b"body>div>a:0"));
    }

    #[test]
    fn action_verb_view_matches_braid_web_registry() {
        let registry = braid_vocab_web::registry_v0();
        let enum_verbs: BTreeSet<_> = ActionVerb::ALL
            .iter()
            .map(|verb| verb.as_str().to_string())
            .collect();
        let registry_verbs: BTreeSet<_> = registry.terms().map(|term| term.id.clone()).collect();
        assert_eq!(enum_verbs, registry_verbs);
        for verb in ActionVerb::ALL {
            assert!(verb.is_registered());
        }
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
    pub const ALL: [ActionVerb; 9] = [
        ActionVerb::Navigate,
        ActionVerb::Observe,
        ActionVerb::Click,
        ActionVerb::Type,
        ActionVerb::Scroll,
        ActionVerb::Download,
        ActionVerb::Wait,
        ActionVerb::ExecuteJs,
        ActionVerb::ExecuteWasm,
    ];

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

    pub fn is_registered(&self) -> bool {
        braid_vocab_web::registry_v0().get(self.as_str()).is_some()
    }
}

/// Verdict from the policy broker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Allow,
    Deny,
    Confirm,
}
