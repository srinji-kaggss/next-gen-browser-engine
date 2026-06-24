//! Traceability: AXIOM_BRAID_CANONICAL, AXIOM_LLMS_SENSOR_ONLY.
//!
//! Adapter to a Chrome DevTools Protocol (CDP) driver.
//! Translates CDP JSONL output into Braid observation facts.
//!
//! This seam intentionally does not open WebSockets or spawn processes inside
//! the core crate. The driver binary (or test harness) is responsible for
//! attaching to Chrome, dumping the accessibility/DOM snapshot, and feeding the
//! resulting JSONL to `ChromeAdapter::observe_from_jsonl`.
use crate::browser_types::*;
use crate::observation::anchor::ObservationAnchor;
#[cfg(feature = "std")]
use crate::observation::anchor::{Fact, ObservationKind};
use alloc::string::{String, ToString};
use alloc::vec::Vec;

pub struct ChromeAdapter;

impl ChromeAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Navigate and return the CID of the resulting load observation.
    pub fn load(&self, _url: &Url) -> Result<Cid, &'static str> {
        // Deferred: navigation completion signaling and load observation emission
        // must come from the CDP driver (Page.loadEventFired / DOMSnapshot).
        Err("load deferred behind seam")
    }

    /// Parse typed observations from a CDP driver JSONL string.
    ///
    /// Expected input: one JSON observation per line:
    /// ```json
    /// {"kind":"element","path":"html>body>div>a:0","facts":[["tag","a"],["text","Sign in"],["bounds","12,34,56,78"],["interactable","true"],["backend_node_id","123"]]}
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
                source: "cdp".to_string(),
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

impl Default for ChromeAdapter {
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

    let target_cid = cid_from_bytes(raw.path.as_bytes());

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
        let line = r#"{"kind":"element","path":"html>body>div>a:0","facts":[["tag","a"],["text","Sign in"],["bounds","12,34,56,78"],["interactable","true"],["backend_node_id","123"]]}"#;
        let adapter = ChromeAdapter::new();
        let anchors = adapter.observe_from_jsonl(line).unwrap();
        assert_eq!(anchors.len(), 1);
        assert_eq!(anchors[0].term_family, TermFamily::Observation);
        assert_eq!(anchors[0].provenance.source, "cdp");
    }

    #[test]
    #[cfg(feature = "std")]
    fn parse_load_observation_line() {
        let line = r#"{"kind":"load","path":"load:0","facts":[["url","https://example.com"],["title","Example Domain"]]}"#;
        let adapter = ChromeAdapter::new();
        let anchors = adapter.observe_from_jsonl(line).unwrap();
        assert_eq!(anchors.len(), 1);
        assert_eq!(anchors[0].term_family, TermFamily::Observation);
    }

    #[test]
    #[cfg(feature = "std")]
    fn observe_from_jsonl_returns_multiple_anchors() {
        let jsonl = concat!(
            r#"{"kind":"load","path":"load:0","facts":[["url","https://example.com"],["title","Example Domain"]]}"#,
            "\n",
            r#"{"kind":"element","path":"html>body>div>a:0","facts":[["tag","a"],["text","Sign in"],["interactable","true"]]}"#
        );
        let adapter = ChromeAdapter::new();
        let anchors = adapter.observe_from_jsonl(jsonl).unwrap();
        assert_eq!(anchors.len(), 2);
        assert_eq!(anchors[0].provenance.source, "cdp");
        assert_eq!(anchors[1].provenance.source, "cdp");
    }
}
