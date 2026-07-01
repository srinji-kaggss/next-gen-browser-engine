//! # be-semantic — Semantic graph builder
//!
//! Merges the a11y tree and layout tree into a structured semantic graph.
//! Computes affordances — what the AI can do with each element.
//!
//! ## Blast radius
//!
//! Depends on be-dom, be-a11y, be-layout, be-capability.
//! Changes here affect be-pulse and be-api.

use be_a11y::{A11yTree, Role};
use be_capability::Capability;
use be_dom::{DomTree, NodeId};
use be_layout::{Display, LayoutBox, LayoutTree, Rect};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A stable element handle — survives DOM mutation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ElementHandle(pub u64);

/// A node in the semantic graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticNode {
    pub id: usize,
    pub element: NodeId,
    pub role: Role,
    pub label: String,
    pub position: Rect,
    pub handle: ElementHandle,
    pub children: Vec<usize>,
}

/// An affordance — what the AI can do.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Affordance {
    pub action: Action,
    pub target: ElementHandle,
    pub required_capability: Capability,
}

/// An action the AI can take.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Click,
    Fill { field_type: FieldType },
    Submit,
    ReadText,
    Select,
}

/// Field type for form inputs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldType {
    Text,
    Email,
    Password,
    Number,
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Click => write!(f, "click"),
            Action::Fill { field_type } => write!(f, "fill:{:?}", field_type),
            Action::Submit => write!(f, "submit"),
            Action::ReadText => write!(f, "read"),
            Action::Select => write!(f, "select"),
        }
    }
}

/// The semantic graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticGraph {
    pub nodes: Vec<SemanticNode>,
    pub affordances: Vec<Affordance>,
}

/// Build a semantic graph from DOM, a11y, and layout trees.
///
/// Merges a11y roles/labels with layout positions. Computes affordances
/// based on element roles — what the AI agent can interact with.
pub fn build(_dom: &DomTree, a11y: &A11yTree, layout: &LayoutTree) -> SemanticGraph {
    // Index layout boxes by element ID for fast lookup
    let layout_index: HashMap<NodeId, &LayoutBox> = layout
        .boxes
        .iter()
        .filter(|b| b.display != Display::None)
        .map(|b| (b.element, b))
        .collect();

    let mut nodes = Vec::new();
    let mut affordances = Vec::new();

    // Convert each a11y node to a semantic node
    for (idx, a11y_node) in a11y.nodes.iter().enumerate() {
        let position = layout_index
            .get(&a11y_node.element)
            .map(|b| b.rect)
            .unwrap_or(Rect::zero());

        let handle = ElementHandle(idx as u64);

        let semantic_node = SemanticNode {
            id: idx,
            element: a11y_node.element,
            role: a11y_node.role.clone(),
            label: a11y_node.label.clone(),
            position,
            handle,
            children: a11y_node.children.clone(),
        };

        // Compute affordances for this node
        let node_affordances = compute_affordances(&a11y_node.role, &a11y_node.label, handle);
        affordances.extend(node_affordances);

        nodes.push(semantic_node);
    }

    SemanticGraph { nodes, affordances }
}

/// Compute affordances for an element based on its role.
fn compute_affordances(role: &Role, label: &str, handle: ElementHandle) -> Vec<Affordance> {
    match role {
        Role::Button => vec![Affordance {
            action: Action::Click,
            target: handle,
            required_capability: Capability::DomActionClick,
        }],
        Role::Link => vec![Affordance {
            action: Action::Click,
            target: handle,
            required_capability: Capability::DomActionClick,
        }],
        Role::Textbox => vec![Affordance {
            action: Action::Fill {
                field_type: FieldType::Text,
            },
            target: handle,
            required_capability: Capability::DomActionType,
        }],
        Role::Checkbox => vec![Affordance {
            action: Action::Click,
            target: handle,
            required_capability: Capability::DomActionClick,
        }],
        Role::RadioButton => vec![Affordance {
            action: Action::Click,
            target: handle,
            required_capability: Capability::DomActionClick,
        }],
        Role::ComboBox => vec![
            Affordance {
                action: Action::Click,
                target: handle,
                required_capability: Capability::DomActionClick,
            },
            Affordance {
                action: Action::Select,
                target: handle,
                required_capability: Capability::DomActionClick,
            },
        ],
        Role::MenuItem => vec![Affordance {
            action: Action::Click,
            target: handle,
            required_capability: Capability::DomActionClick,
        }],
        Role::Heading | Role::Paragraph | Role::Image => {
            if !label.is_empty() {
                vec![Affordance {
                    action: Action::ReadText,
                    target: handle,
                    required_capability: Capability::AiRead,
                }]
            } else {
                vec![]
            }
        }
        Role::Generic => {
            if !label.is_empty() {
                vec![Affordance {
                    action: Action::ReadText,
                    target: handle,
                    required_capability: Capability::AiRead,
                }]
            } else {
                vec![]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_build_all(html: &str) -> (DomTree, A11yTree, LayoutTree, SemanticGraph) {
        let dom = be_parser::parse_html(html).unwrap();
        let a11y = be_a11y::build(&dom);
        let layout = be_layout::compute(&dom);
        let graph = build(&dom, &a11y, &layout);
        (dom, a11y, layout, graph)
    }

    #[test]
    fn test_empty_graph() {
        let (_, _, _, graph) = parse_build_all("");
        // Should have some nodes from the html structure
        assert!(!graph.nodes.is_empty());
    }

    #[test]
    fn test_button_affordance() {
        let (_, _, _, graph) = parse_build_all(r#"<button>Click me</button>"#);
        let click = graph
            .affordances
            .iter()
            .find(|a| matches!(a.action, Action::Click));
        assert!(click.is_some());
    }

    #[test]
    fn test_link_affordance() {
        let (_, _, _, graph) = parse_build_all(r#"<a href="/home">Home</a>"#);
        let click = graph
            .affordances
            .iter()
            .find(|a| matches!(a.action, Action::Click));
        assert!(click.is_some());
    }

    #[test]
    fn test_textbox_affordance() {
        let (_, _, _, graph) = parse_build_all(r#"<input type="text" aria-label="Name">"#);
        let fill = graph
            .affordances
            .iter()
            .find(|a| matches!(a.action, Action::Fill { .. }));
        assert!(fill.is_some());
    }

    #[test]
    fn test_readtext_affordance() {
        let (_, _, _, graph) = parse_build_all(r#"<h1>Title</h1>"#);
        let read = graph
            .affordances
            .iter()
            .find(|a| matches!(a.action, Action::ReadText));
        assert!(read.is_some());
    }

    #[test]
    fn test_checkbox_affordance() {
        let (_, _, _, graph) = parse_build_all(r#"<input type="checkbox">"#);
        let click = graph
            .affordances
            .iter()
            .find(|a| matches!(a.action, Action::Click));
        assert!(click.is_some());
    }

    #[test]
    fn test_combo_affordances() {
        let (_, _, _, graph) = parse_build_all(r#"<select><option>A</option></select>"#);
        // ComboBox should have both Click and Select
        let select = graph
            .affordances
            .iter()
            .find(|a| matches!(a.action, Action::Select));
        assert!(select.is_some());
    }

    #[test]
    fn test_node_has_position() {
        let (_, _, _, graph) = parse_build_all(r#"<p>Hello</p>"#);
        let p = graph.nodes.iter().find(|n| n.role == Role::Paragraph);
        assert!(p.is_some());
        // Position should be set (not zero for visible elements)
        let p = p.unwrap();
        assert!(p.position.height > 0.0);
    }

    #[test]
    fn test_node_has_handle() {
        let (_, _, _, graph) = parse_build_all(r#"<button>OK</button>"#);
        let button = graph.nodes.iter().find(|n| n.role == Role::Button);
        assert!(button.is_some());
        // Handle should be valid
        assert_eq!(
            button.unwrap().handle,
            ElementHandle(button.unwrap().id as u64)
        );
    }

    #[test]
    fn test_required_capability() {
        let (_, _, _, graph) = parse_build_all(r#"<button>OK</button>"#);
        let click = graph
            .affordances
            .iter()
            .find(|a| matches!(a.action, Action::Click));
        assert!(click.is_some());
        assert_eq!(
            click.unwrap().required_capability,
            Capability::DomActionClick
        );
    }
}
