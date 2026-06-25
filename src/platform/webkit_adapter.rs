//! Traceability: AXIOM_BRAID_CANONICAL, AXIOM_LLMS_SENSOR_ONLY.
use crate::browser_types::*;
use crate::observation::anchor::ObservationAnchor;
#[cfg(feature = "std")]
use crate::observation::anchor::{Fact, ObservationKind};
use alloc::string::{String, ToString};
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
}
