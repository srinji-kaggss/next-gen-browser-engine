//! # be-pulse — PULSE frame encoder
//!
//! Encodes semantic graph affordances as compact PULSE frames.
//! PULSE is the API surface for AI agents — not the DOM.
//!
//! ## Blast radius
//!
//! Depends on be-semantic and be-axiom.
//! Changes here affect be-api.

use be_axiom::Cid;
use be_semantic::{ElementHandle, SemanticGraph};
use serde::{Deserialize, Serialize};

/// A PULSE frame — the wire format between browser and AI agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Frame {
    /// Query: what can the AI do?
    AskAffordances { target: String },
    /// Response: here's what you can do.
    OkAffordances { affordances: Vec<AffordanceEntry> },
    /// Action: do something.
    DoAction { action: String, handle: u64 },
    /// Result: action succeeded.
    OkAction { action: String, handle: u64, result: String },
    /// Mutation: page changed.
    PageChanged { added: Vec<String>, removed: Vec<String> },
}

/// A single affordance entry in a PULSE response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffordanceEntry {
    pub handle: u64,
    pub action: String,
    pub label: String,
    pub required_capability: String,
}

/// Encode affordances from a semantic graph as a compact PULSE frame string.
///
/// Format: `ok affordances [handle:action:capability "label", ...]`
///
/// This is the wire format an AI agent receives. It contains everything
/// needed to identify an element, understand what it can do, and act on it.
pub fn encode_affordances(graph: &SemanticGraph) -> String {
    let entries: Vec<String> = graph
        .affordances
        .iter()
        .map(|a| {
            let node = graph.nodes.iter().find(|n| n.handle == a.target);
            let label = node.map(|n| n.label.as_str()).unwrap_or("");
            format!(
                "{}:{}:{:?} \"{}\"",
                a.target.0,
                a.action,
                a.required_capability,
                label
            )
        })
        .collect();
    format!("ok affordances [{}]", entries.join(", "))
}

/// Build PULSE frames from a semantic graph.
///
/// Returns the OkAffordances frame with structured entries.
pub fn build_frames(graph: &SemanticGraph) -> Frame {
    let affordances: Vec<AffordanceEntry> = graph
        .affordances
        .iter()
        .map(|a| {
            let node = graph.nodes.iter().find(|n| n.handle == a.target);
            let label = node.map(|n| n.label.clone()).unwrap_or_default();
            AffordanceEntry {
                handle: a.target.0,
                action: format!("{}", a.action),
                label,
                required_capability: format!("{:?}", a.required_capability),
            }
        })
        .collect();
    Frame::OkAffordances { affordances }
}

/// Encode a "do action" frame as a string.
///
/// Format: `do action {action} {handle}`
pub fn encode_action(action: &str, handle: ElementHandle) -> String {
    format!("do action {} {}", action, handle.0)
}

/// Encode an "ok action" result frame as a string.
///
/// Format: `ok action {action} {handle} {result}`
pub fn encode_action_result(action: &str, handle: ElementHandle, result: &str) -> String {
    format!("ok action {} {} {}", action, handle.0, result)
}

/// Get the CID of a frame.
pub fn frame_cid(frame: &Frame) -> Cid {
    let bytes = serde_json::to_vec(frame).unwrap_or_default();
    Cid::from_bytes(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_build_encode(html: &str) -> (SemanticGraph, String) {
        let dom = be_parser::parse_html(html).unwrap();
        let a11y = be_a11y::build(&dom);
        let layout = be_layout::compute(&dom);
        let graph = be_semantic::build(&dom, &a11y, &layout);
        let encoded = encode_affordances(&graph);
        (graph, encoded)
    }

    #[test]
    fn test_encode_button() {
        let (_, encoded) = parse_build_encode(r#"<button>Submit</button>"#);
        assert!(encoded.contains("ok affordances"));
        assert!(encoded.contains("click"));
        assert!(encoded.contains("Submit"));
    }

    #[test]
    fn test_encode_link() {
        let (_, encoded) = parse_build_encode(r#"<a href="/home">Home</a>"#);
        assert!(encoded.contains("click"));
        assert!(encoded.contains("Home"));
    }

    #[test]
    fn test_encode_textbox() {
        let (_, encoded) = parse_build_encode(r#"<input type="text" aria-label="Name">"#);
        assert!(encoded.contains("fill"));
        assert!(encoded.contains("Name"));
    }

    #[test]
    fn test_encode_heading() {
        let (_, encoded) = parse_build_encode(r#"<h1>Hello</h1>"#);
        assert!(encoded.contains("read"));
        assert!(encoded.contains("Hello"));
    }

    #[test]
    fn test_build_frames() {
        let (graph, _) = parse_build_encode(r#"<button>OK</button>"#);
        let frame = build_frames(&graph);
        match frame {
            Frame::OkAffordances { affordances } => {
                let click = affordances.iter().find(|a| a.action == "click");
                assert!(click.is_some(), "Expected a click affordance, got: {:?}", affordances);
                assert_eq!(click.unwrap().label, "OK");
            }
            _ => panic!("Expected OkAffordances frame"),
        }
    }

    #[test]
    fn test_encode_action_string() {
        let s = encode_action("click", ElementHandle(5));
        assert_eq!(s, "do action click 5");
    }

    #[test]
    fn test_encode_result_string() {
        let s = encode_action_result("click", ElementHandle(5), "done");
        assert_eq!(s, "ok action click 5 done");
    }

    #[test]
    fn test_frame_cid_deterministic() {
        let frame = Frame::DoAction { action: "click".to_string(), handle: 0 };
        let cid1 = frame_cid(&frame);
        let cid2 = frame_cid(&frame);
        assert_eq!(cid1, cid2);
    }

    #[test]
    fn test_empty_affordances() {
        let dom = be_parser::parse_html("").unwrap();
        let a11y = be_a11y::build(&dom);
        let layout = be_layout::compute(&dom);
        let graph = be_semantic::build(&dom, &a11y, &layout);
        let encoded = encode_affordances(&graph);
        assert!(encoded.contains("ok affordances"));
    }
}
