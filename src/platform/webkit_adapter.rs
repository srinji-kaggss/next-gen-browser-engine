//! Traceability: AXIOM_BRAID_CANONICAL, AXIOM_LLMS_SENSOR_ONLY.
use crate::{
    action::Action,
    browser_types::*,
    capability::WebCapability,
    observation::anchor::{Fact, ObservationAnchor, ObservationKind},
    policy::PolicyBroker,
};
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

/// Adapter to the native macOS WKWebView bridge (mac-eye).
/// Translates WebKit JSONL output into Braid observation facts.
///
/// Process spawning is intentionally kept out of this core seam. The driver
/// binary (or test harness) is responsible for invoking `lgwks-mac-eye` and
/// feeding its stdout to `WebKitAdapter::observe_from_jsonl`.
pub struct WebKitAdapter;

impl WebKitAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Navigate and return the CID of the resulting load observation.
    pub fn load(&self, _url: &Url) -> Result<Cid, &'static str> {
        // Deferred: navigation requires the native bridge to signal completion
        // and emit a load observation. For now this is a seam stub.
        Err("load deferred behind seam")
    }

    /// Parse typed observations from a mac-eye JSONL string.
    ///
    /// Expected input: one JSON observation per line:
    /// ```json
    /// {"kind":"element","path":"body>div>a:0","facts":[["tag","a"],["text","Sign in"],["bounds","12,34,56,78"],["interactable","true"]]}
    /// ```
    pub fn observe_from_jsonl(&self, jsonl: &str) -> Result<Vec<WebAnchor>, &'static str> {
        let mut anchors = Vec::new();
        for line in jsonl.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let obs = parse_observation_line(line).map_err(|_| "invalid observation line")?;
            let provenance = Provenance {
                source: "mac-eye".to_string(),
                input_cids: Vec::new(),
                trust_class: TrustClass::UntrustedContent,
                did_principal: None,
            };
            anchors.push(obs.to_anchor(provenance));
        }
        Ok(anchors)
    }

    pub fn execute_js(&self, _script: &str) -> Result<String, &'static str> {
        // Deferred: JS execution routes through the capability broker.
        Err("execute_js deferred behind seam")
    }

    /// Route a proposed JS action through policy and emit a canonical trace
    /// observation. This does not execute JS; the native driver remains the
    /// only execution boundary.
    pub fn execute_js_trace(
        &self,
        facts: &[WebAnchor],
        action: &Action,
        caps: &[WebCapability],
    ) -> Result<WebAnchor, &'static str> {
        if action.verb != ActionVerb::ExecuteJs {
            return Err("action is not web.execute_js");
        }

        let verdict = PolicyBroker::new().decide(facts, action, caps);
        let mut input_cids: Vec<Cid> = facts.iter().map(|fact| fact.cid).collect();
        input_cids.push(action.capability_cid);
        let obs = ObservationAnchor {
            kind: ObservationKind::Semantic,
            target_cid: action.target_cid,
            observed_at: "1970-01-01T00:00:00Z".to_string(),
            facts: vec![
                Fact {
                    predicate: "action".to_string(),
                    object: action.verb.as_str().to_string(),
                    sensitivity: None,
                },
                Fact {
                    predicate: "origin".to_string(),
                    object: action.origin.clone(),
                    sensitivity: None,
                },
                Fact {
                    predicate: "verdict".to_string(),
                    object: verdict.as_str().to_string(),
                    sensitivity: None,
                },
                Fact {
                    predicate: "executed".to_string(),
                    object: (verdict == Verdict::Allow).to_string(),
                    sensitivity: None,
                },
            ],
            sensitivity: None,
            privacy_tier: PrivacyTier::LocalFull,
            trust_class: TrustClass::SystemPolicy,
            raw_source: None,
        };

        Ok(obs.to_anchor(Provenance {
            source: "webkit-adapter.policy".to_string(),
            input_cids,
            trust_class: TrustClass::SystemPolicy,
            did_principal: None,
        }))
    }
}

impl Default for WebKitAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "std")]
fn parse_observation_line(line: &str) -> Result<ObservationAnchor, &'static str> {
    #[derive(serde::Deserialize)]
    struct RawObservation {
        kind: String,
        path: String,
        facts: Vec<(String, String)>,
    }

    let raw: RawObservation =
        serde_json::from_str(line).map_err(|_| "failed to parse observation JSON")?;

    let kind = match raw.kind.as_str() {
        "load" => ObservationKind::Load,
        "layout" => ObservationKind::Layout,
        "paint" => ObservationKind::Paint,
        "element" => ObservationKind::Element,
        "network" => ObservationKind::Network,
        "console" => ObservationKind::Console,
        "a11y" => ObservationKind::A11y,
        "semantic" => ObservationKind::Semantic,
        "aip_state" => ObservationKind::AipState,
        "aip_policy" => ObservationKind::AipPolicy,
        _ => return Err("unknown observation kind"),
    };

    let facts: Vec<Fact> = raw
        .facts
        .into_iter()
        .map(|(predicate, object)| Fact {
            predicate,
            object,
            sensitivity: None,
        })
        .collect();

    // The target CID is content-addressed from the stable path. Combined with
    // the anchor CID (which hashes the full canonical observation including
    // facts), the same DOM element with the same text yields stable IDs.
    let target_cid = Cid::compute(WEB_ELEMENT_DOMAIN, raw.path.as_bytes());

    Ok(ObservationAnchor {
        kind,
        target_cid,
        observed_at: "1970-01-01T00:00:00Z".to_string(),
        facts,
        sensitivity: None,
        privacy_tier: PrivacyTier::LocalFull,
        trust_class: TrustClass::UntrustedContent,
        raw_source: Some(line.to_string()),
    })
}

#[cfg(not(feature = "std"))]
fn parse_observation_line(_line: &str) -> Result<ObservationAnchor, &'static str> {
    Err("observation parsing requires std feature")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::Attenuation;
    use alloc::vec;

    fn cap(scope: &str, verbs: Vec<ActionVerb>, origins: Vec<&str>) -> WebCapability {
        WebCapability {
            issuer_did: "did:system".to_string(),
            subject_did: "did:agent".to_string(),
            scope: vec![scope.to_string()],
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

    fn execute_js_action(origin: &str) -> Action {
        Action {
            verb: ActionVerb::ExecuteJs,
            target_cid: Cid::compute(WEB_ELEMENT_DOMAIN, origin.as_bytes()),
            origin: origin.to_string(),
            capability_cid: Cid::compute(WEB_ANCHOR_DOMAIN, b"capability"),
            risk: Risk::Low,
            parameters: Vec::new(),
            effect_signature: Vec::new(),
        }
    }

    #[test]
    #[cfg(feature = "std")]
    fn parse_element_observation_line() {
        let line = r#"{"kind":"element","path":"body>div>a:0","facts":[["tag","a"],["text","Sign in"],["bounds","12,34,56,78"],["interactable","true"]]}"#;
        let obs = parse_observation_line(line).unwrap();
        assert_eq!(obs.kind, ObservationKind::Element);
        assert_eq!(
            obs.target_cid,
            Cid::compute(WEB_ELEMENT_DOMAIN, "body>div>a:0".as_bytes())
        );
        assert_eq!(obs.facts.len(), 4);
    }

    #[test]
    #[cfg(feature = "std")]
    fn parse_load_observation_line() {
        let line = r#"{"kind":"load","path":"load:0","facts":[["url","https://example.com"],["title","Example Domain"]]}"#;
        let obs = parse_observation_line(line).unwrap();
        assert_eq!(obs.kind, ObservationKind::Load);
        assert_eq!(obs.facts.len(), 2);
    }

    #[test]
    #[cfg(feature = "std")]
    fn observe_from_jsonl_returns_anchors() {
        let jsonl = concat!(
            r#"{"kind":"load","path":"load:0","facts":[["url","https://example.com"],["title","Example Domain"]]}"#,
            "\n",
            r#"{"kind":"element","path":"body>div>a:0","facts":[["tag","a"],["text","Sign in"],["interactable","true"]]}"#
        );
        let adapter = WebKitAdapter::new();
        let anchors = adapter.observe_from_jsonl(jsonl).unwrap();
        assert_eq!(anchors.len(), 2);
        assert_eq!(anchors[0].term_family, TermFamily::Observation);
        assert_eq!(anchors[1].term_family, TermFamily::Observation);
    }

    #[test]
    fn execute_js_trace_marks_allowed_action_executed() {
        let adapter = WebKitAdapter::new();
        let action = execute_js_action("example.com");
        let capability = cap(
            braid_vocab_web::COMPUTE_LOCAL_NAME,
            vec![ActionVerb::ExecuteJs],
            vec!["example.com"],
        );
        let trace = adapter
            .execute_js_trace(&[], &action, &[capability])
            .expect("trace");
        let obs = crate::observation::anchor::observation_from_payload(&trace.payload).unwrap();

        assert!(obs
            .facts
            .iter()
            .any(|f| f.predicate == "verdict" && f.object == "allow"));
        assert!(obs
            .facts
            .iter()
            .any(|f| f.predicate == "executed" && f.object == "true"));
    }

    #[test]
    fn execute_js_trace_marks_denied_action_unexecuted() {
        let adapter = WebKitAdapter::new();
        let action = execute_js_action("evil.com");
        let capability = cap(
            braid_vocab_web::INTERACT_NAME,
            vec![ActionVerb::Click],
            vec!["example.com"],
        );
        let trace = adapter
            .execute_js_trace(&[], &action, &[capability])
            .expect("trace");
        let obs = crate::observation::anchor::observation_from_payload(&trace.payload).unwrap();

        assert!(obs
            .facts
            .iter()
            .any(|f| f.predicate == "verdict" && f.object == "deny"));
        assert!(obs
            .facts
            .iter()
            .any(|f| f.predicate == "executed" && f.object == "false"));
    }

    #[test]
    fn execute_js_trace_rejects_non_js_action() {
        let adapter = WebKitAdapter::new();
        let mut action = execute_js_action("example.com");
        action.verb = ActionVerb::Click;

        assert_eq!(
            adapter.execute_js_trace(&[], &action, &[]),
            Err("action is not web.execute_js")
        );
    }
}
