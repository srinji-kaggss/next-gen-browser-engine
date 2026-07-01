use crate::{browser_types::*, ActionVerb};
use alloc::string::{String, ToString};
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
        issuer: &str,
        subject: &str,
        scope: Vec<String>,
        attenuation: Attenuation,
    ) -> Result<WebCapability, &'static str> {
        if issuer.is_empty() || subject.is_empty() {
            return Err("capability principal is empty");
        }
        if scope.is_empty() {
            return Err("capability scope is empty");
        }
        if attenuation.allowed_verbs.is_empty() {
            return Err("capability must explicitly allow verbs");
        }
        Err("capability signer not configured")
    }

    /// Verify and attenuate a capability to a narrower scope.
    pub fn attenuate(
        &self,
        parent: &WebCapability,
        narrower: Attenuation,
    ) -> Result<WebCapability, &'static str> {
        if !is_attenuation_subset(&parent.attenuation, &narrower) {
            return Err("attenuation widens parent capability");
        }
        if parent.signature.is_empty() {
            return Err("parent capability is unsigned");
        }
        Err("capability signer not configured")
    }

    /// Check whether an already-issued capability may exercise one closed
    /// browser verb. This is the runtime gate used by policy and compute
    /// admission; it does not mint or resign credentials.
    pub fn authorize(
        &self,
        cap: &WebCapability,
        verb: ActionVerb,
        origin: &str,
        bytes: usize,
    ) -> Result<(), &'static str> {
        if !verb.is_registered() {
            return Err("action verb is not registered in braid-vocab-web");
        }

        if let Some(required_scope) = required_scope_for_verb(verb)? {
            if !cap.scope.iter().any(|scope| scope == &required_scope) {
                return Err("capability scope does not grant required authority");
            }
        }

        if !cap.attenuation.allowed_verbs.contains(&verb) {
            return Err("capability does not explicitly allow verb");
        }
        if !origin.is_empty()
            && !cap
                .attenuation
                .allowed_origins
                .contains(&origin.to_string())
        {
            return Err("capability does not allow origin");
        }
        if cap.attenuation.max_bytes != 0 && bytes > cap.attenuation.max_bytes {
            return Err("capability byte budget exceeded");
        }
        Ok(())
    }

    /// Return the Braid vocabulary capability required by a browser verb.
    /// Pure verbs such as `web.wait` have no capability requirement.
    pub fn required_scope(&self, verb: ActionVerb) -> Result<Option<String>, &'static str> {
        required_scope_for_verb(verb)
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

fn required_scope_for_verb(verb: ActionVerb) -> Result<Option<String>, &'static str> {
    let registry = braid_vocab_web::registry_v0();
    let term = registry
        .get(verb.as_str())
        .ok_or("action verb is not registered in braid-vocab-web")?;
    Ok(term
        .capability
        .as_ref()
        .map(|capability| capability.as_str().to_string()))
}

fn is_attenuation_subset(parent: &Attenuation, child: &Attenuation) -> bool {
    let verbs_ok = child
        .allowed_verbs
        .iter()
        .all(|verb| parent.allowed_verbs.contains(verb));
    let origins_ok = child
        .allowed_origins
        .iter()
        .all(|origin| parent.allowed_origins.contains(origin));
    let bytes_ok =
        parent.max_bytes == 0 || (child.max_bytes != 0 && child.max_bytes <= parent.max_bytes);
    let calls_ok =
        parent.max_calls == 0 || (child.max_calls != 0 && child.max_calls <= parent.max_calls);
    verbs_ok && origins_ok && bytes_ok && calls_ok
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use alloc::vec;

    fn cap(
        scope: &str,
        verbs: Vec<ActionVerb>,
        origins: Vec<&str>,
        max_bytes: usize,
    ) -> WebCapability {
        WebCapability {
            issuer_did: "did:system".to_string(),
            subject_did: "did:agent".to_string(),
            scope: vec![scope.to_string()],
            privacy_tier: PrivacyTier::LocalFull,
            attenuation: Attenuation {
                allowed_verbs: verbs,
                allowed_origins: origins.iter().map(|origin| origin.to_string()).collect(),
                max_bytes,
                max_calls: 0,
            },
            issued_at: "2026-06-17T00:00:00Z".to_string(),
            expires_at: "2099-06-17T00:00:00Z".to_string(),
            signature: Vec::new(),
        }
    }

    #[test]
    fn authorizes_when_scope_verb_origin_and_budget_match() {
        let c = cap(
            braid_vocab_web::COMPUTE_LOCAL_NAME,
            vec![ActionVerb::ExecuteJs],
            vec!["example.com"],
            64,
        );
        assert_eq!(
            CapabilityBroker::new().authorize(&c, ActionVerb::ExecuteJs, "example.com", 12),
            Ok(())
        );
    }

    #[test]
    fn required_scope_is_read_from_braid_vocab_web() {
        assert_eq!(
            CapabilityBroker::new().required_scope(ActionVerb::ExecuteJs),
            Ok(Some(braid_vocab_web::COMPUTE_LOCAL_NAME.to_string()))
        );
        assert_eq!(
            CapabilityBroker::new().required_scope(ActionVerb::Wait),
            Ok(None)
        );
    }

    #[test]
    fn rejects_wrong_braid_scope_even_when_verb_matches() {
        let c = cap(
            braid_vocab_web::INTERACT_NAME,
            vec![ActionVerb::ExecuteJs],
            vec!["example.com"],
            64,
        );
        assert_eq!(
            CapabilityBroker::new().authorize(&c, ActionVerb::ExecuteJs, "example.com", 12),
            Err("capability scope does not grant required authority")
        );
    }

    #[test]
    fn rejects_implicit_verb_and_over_budget_execution() {
        let mut c = cap(
            braid_vocab_web::COMPUTE_LOCAL_NAME,
            Vec::new(),
            vec!["example.com"],
            4,
        );
        assert_eq!(
            CapabilityBroker::new().authorize(&c, ActionVerb::ExecuteJs, "example.com", 2),
            Err("capability does not explicitly allow verb")
        );

        c.attenuation.allowed_verbs.push(ActionVerb::ExecuteJs);
        assert_eq!(
            CapabilityBroker::new().authorize(&c, ActionVerb::ExecuteJs, "example.com", 8),
            Err("capability byte budget exceeded")
        );
    }

    #[test]
    fn attenuation_cannot_widen_parent() {
        let parent = Attenuation {
            allowed_verbs: vec![ActionVerb::Click],
            allowed_origins: vec!["example.com".to_string()],
            max_bytes: 32,
            max_calls: 4,
        };
        let wider = Attenuation {
            allowed_verbs: vec![ActionVerb::Click, ActionVerb::ExecuteJs],
            allowed_origins: vec!["example.com".to_string()],
            max_bytes: 32,
            max_calls: 4,
        };
        assert!(!is_attenuation_subset(&parent, &wider));
    }

    #[test]
    fn issue_fails_closed_until_signer_exists() {
        let attenuation = Attenuation {
            allowed_verbs: vec![ActionVerb::Click],
            allowed_origins: vec!["example.com".to_string()],
            max_bytes: 0,
            max_calls: 0,
        };
        assert_eq!(
            CapabilityBroker::new().issue(
                "did:system",
                "did:agent",
                vec![braid_vocab_web::INTERACT_NAME.to_string()],
                attenuation
            ),
            Err("capability signer not configured")
        );
    }
}
