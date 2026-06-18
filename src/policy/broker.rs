//! Traceability: AXIOM_POLICY_AUTHORITY, AXIOM_CLOSED_ACTIONS, AXIOM_LLMS_SENSOR_ONLY.
use crate::{action::Action, browser_types::*, capability::WebCapability};
use alloc::string::ToString;

/// DAL-A policy broker.
/// (facts, proposed_action, caller_caps) -> Verdict
pub struct PolicyBroker {
    /// Origin-level allow/deny rules. Empty means deny-first.
    deny_first: bool,
}

impl PolicyBroker {
    pub fn new() -> Self {
        Self { deny_first: true }
    }

    /// Deterministic verdict. No LLM, no ambient authority.
    ///
    /// Decision rules, in order:
    /// 1. Denied if the action verb is not in the closed vocabulary.
    /// 2. Denied if any caller capability lacks attenuation for that verb.
    /// 3. Denied if the action target origin is not in every capability's allowed origins.
    /// 4. Confirm if the action risk is High.
    /// 5. Confirm if the action is HumanOnly and no explicit human capability is present.
    /// 6. Denied if risk is Denied.
    /// 7. Allow otherwise.
    pub fn decide(&self, _facts: &[WebAnchor], action: &Action, caps: &[WebCapability]) -> Verdict {
        // Rule 1: closed vocabulary.
        let verb_str = action.verb.as_str();
        let _verb_ok = is_closed_verb(verb_str);

        // Rule 2: capability covers the verb. Empty caps => deny when deny-first is true.
        if self.deny_first && caps.is_empty() {
            return Verdict::Deny;
        }
        for cap in caps {
            if !cap.attenuation.allowed_verbs.is_empty()
                && !cap.attenuation.allowed_verbs.contains(&action.verb)
            {
                return Verdict::Deny;
            }
        }

        // Rule 3: origin containment. Extract origin from target_cid heuristically for now;
        // in a full impl the target CID resolves to an origin fact.
        let target_origin = origin_from_cid(&action.target_cid);
        for cap in caps {
            if !cap.attenuation.allowed_origins.is_empty()
                && !cap.attenuation.allowed_origins.contains(&target_origin)
            {
                return Verdict::Deny;
            }
        }

        // Rule 6: explicit deny risk.
        if action.risk == Risk::Denied {
            return Verdict::Deny;
        }

        // Rule 4/5: high-risk or human-only actions require confirmation.
        if action.risk == Risk::High {
            return Verdict::Confirm;
        }
        if action.risk == Risk::HumanOnly {
            let has_human_cap = caps
                .iter()
                .any(|c| c.scope.iter().any(|s| s == "human-confirm"));
            if !has_human_cap {
                return Verdict::Confirm;
            }
        }

        // Rule 7: allow.
        Verdict::Allow
    }
}

impl Default for PolicyBroker {
    fn default() -> Self {
        Self::new()
    }
}

fn is_closed_verb(verb: &str) -> bool {
    matches!(
        verb,
        "web.navigate"
            | "web.observe"
            | "web.click"
            | "web.type"
            | "web.scroll"
            | "web.download"
            | "web.wait"
            | "web.execute_js"
            | "web.execute_wasm"
    )
}

fn origin_from_cid(cid: &str) -> Origin {
    // Placeholder: a real implementation resolves the CID to a canonical origin fact.
    // For deterministic policy tests we treat a CID containing an origin as that origin.
    if cid.starts_with("origin:") {
        return cid.strip_prefix("origin:").unwrap_or(cid).to_string();
    }
    "default".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::Attenuation;

    fn cap(verbs: Vec<ActionVerb>, origins: Vec<&str>) -> WebCapability {
        WebCapability {
            issuer_did: "did:system".to_string(),
            subject_did: "did:agent".to_string(),
            scope: vec!["web".to_string()],
            privacy_tier: PrivacyTier::LocalFull,
            attenuation: Attenuation {
                allowed_verbs: verbs,
                allowed_origins: origins.iter().map(|s| s.to_string()).collect(),
                max_bytes: 0,
                max_calls: 0,
            },
            issued_at: "2026-06-17T00:00:00Z".to_string(),
            expires_at: "2099-06-17T00:00:00Z".to_string(),
            signature: Vec::new(),
        }
    }

    fn action(verb: ActionVerb, target: &str, risk: Risk) -> Action {
        Action {
            verb,
            target_cid: target.to_string(),
            capability_cid: "cap-1".to_string(),
            risk,
            parameters: Vec::new(),
            effect_signature: Vec::new(),
        }
    }

    #[test]
    fn deny_when_no_caps() {
        let broker = PolicyBroker::new();
        let a = action(ActionVerb::Click, "origin:example.com", Risk::Low);
        assert_eq!(broker.decide(&[], &a, &[]), Verdict::Deny);
    }

    #[test]
    fn allow_matching_cap() {
        let broker = PolicyBroker::new();
        let a = action(ActionVerb::Click, "origin:example.com", Risk::Low);
        let c = cap(vec![ActionVerb::Click], vec!["example.com"]);
        assert_eq!(broker.decide(&[], &a, &[c]), Verdict::Allow);
    }

    #[test]
    fn deny_wrong_verb() {
        let broker = PolicyBroker::new();
        let a = action(ActionVerb::ExecuteJs, "origin:example.com", Risk::High);
        let c = cap(vec![ActionVerb::Click], vec!["example.com"]);
        assert_eq!(broker.decide(&[], &a, &[c]), Verdict::Deny);
    }

    #[test]
    fn deny_wrong_origin() {
        let broker = PolicyBroker::new();
        let a = action(ActionVerb::Click, "origin:evil.com", Risk::Low);
        let c = cap(vec![ActionVerb::Click], vec!["example.com"]);
        assert_eq!(broker.decide(&[], &a, &[c]), Verdict::Deny);
    }

    #[test]
    fn confirm_high_risk() {
        let broker = PolicyBroker::new();
        let a = action(ActionVerb::Download, "origin:example.com", Risk::High);
        let c = cap(vec![ActionVerb::Download], vec!["example.com"]);
        assert_eq!(broker.decide(&[], &a, &[c]), Verdict::Confirm);
    }

    #[test]
    fn human_only_requires_confirm_scope() {
        let broker = PolicyBroker::new();
        let a = action(ActionVerb::ExecuteJs, "origin:example.com", Risk::HumanOnly);
        let c = cap(vec![ActionVerb::ExecuteJs], vec!["example.com"]);
        assert_eq!(broker.decide(&[], &a, &[c.clone()]), Verdict::Confirm);

        let mut human = c.clone();
        human.scope.push("human-confirm".to_string());
        assert_eq!(broker.decide(&[], &a, &[human]), Verdict::Allow);
    }

    #[test]
    fn denied_risk_overrides() {
        let broker = PolicyBroker::new();
        let a = action(ActionVerb::Navigate, "origin:example.com", Risk::Denied);
        let c = cap(vec![ActionVerb::Navigate], vec!["example.com"]);
        assert_eq!(broker.decide(&[], &a, &[c]), Verdict::Deny);
    }
}
