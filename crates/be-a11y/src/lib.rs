//! # be-a11y — Accessibility tree builder
//!
//! Builds the accessibility tree from the DOM tree.
//! Maps HTML elements to roles, computes labels, and tracks states.
//!
//! ## Blast radius
//!
//! Depends on be-dom only. Changes here affect be-semantic and be-pulse.

use be_dom::{DomTree, NodeId, NodeKind};
use serde::{Deserialize, Serialize};

/// Role of an element in the accessibility tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Button,
    Link,
    Textbox,
    Checkbox,
    RadioButton,
    ComboBox,
    MenuItem,
    Heading,
    Paragraph,
    Image,
    Generic,
}

/// States an element can have.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct States {
    pub disabled: bool,
    pub checked: Option<bool>,
    pub focused: bool,
    pub expanded: Option<bool>,
    pub selected: bool,
}

/// An accessibility tree node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A11yNode {
    pub element: NodeId,
    pub role: Role,
    pub label: String,
    pub states: States,
    pub children: Vec<usize>,
}

/// The accessibility tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A11yTree {
    pub nodes: Vec<A11yNode>,
}

/// Build an accessibility tree from a DOM tree.
///
/// Walks the DOM tree, maps elements to a11y roles, computes labels
/// from attributes (aria-label, title, alt) or text content, and
/// tracks element states (disabled, checked, etc.).
pub fn build(dom: &DomTree) -> A11yTree {
    let mut tree = A11yTree { nodes: Vec::new() };
    let root = dom.root();
    build_subtree(dom, root, &mut tree);
    tree
}

fn build_subtree(dom: &DomTree, node_id: NodeId, tree: &mut A11yTree) -> Option<usize> {
    let node = dom.get(node_id)?;
    match &node.kind {
        NodeKind::Document => {
            // Process children, return first meaningful child
            for &child in dom.children(node_id) {
                if let Some(idx) = build_subtree(dom, child, tree) {
                    return Some(idx);
                }
            }
            None
        }
        NodeKind::Element { tag_name, .. } => {
            let role = map_role(tag_name, dom, node_id);
            let label = compute_label(dom, node_id, tag_name);
            let states = compute_states(dom, node_id);

            let mut children = Vec::new();

            // Build children first
            for &child in dom.children(node_id) {
                if let Some(idx) = build_subtree(dom, child, tree) {
                    children.push(idx);
                }
            }

            let idx = tree.nodes.len();
            tree.nodes.push(A11yNode {
                element: node_id,
                role,
                label,
                states,
                children,
            });

            Some(idx)
        }
        NodeKind::Text { content } => {
            let trimmed = content.trim();
            if trimmed.is_empty() {
                return None;
            }
            let idx = tree.nodes.len();
            tree.nodes.push(A11yNode {
                element: node_id,
                role: Role::Generic,
                label: trimmed.to_string(),
                states: States::default(),
                children: Vec::new(),
            });
            Some(idx)
        }
        _ => None,
    }
}

/// Map an HTML tag to an a11y role.
fn map_role(tag: &str, dom: &DomTree, id: NodeId) -> Role {
    match tag {
        "button" => Role::Button,
        "a" => Role::Link,
        "input" => {
            let input_type = dom.get_attribute(id, "type").unwrap_or("text");
            match input_type {
                "checkbox" => Role::Checkbox,
                "radio" => Role::RadioButton,
                _ => Role::Textbox,
            }
        }
        "select" => Role::ComboBox,
        "option" => Role::MenuItem,
        "textarea" => Role::Textbox,
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => Role::Heading,
        "p" => Role::Paragraph,
        "img" => Role::Image,
        _ => Role::Generic,
    }
}

/// Compute a label for an element from attributes or text content.
fn compute_label(dom: &DomTree, id: NodeId, tag: &str) -> String {
    // Priority: aria-label > title > alt > value > text content
    if let Some(label) = dom.get_attribute(id, "aria-label") {
        return label.to_string();
    }
    if let Some(title) = dom.get_attribute(id, "title") {
        return title.to_string();
    }
    if tag == "img" {
        if let Some(alt) = dom.get_attribute(id, "alt") {
            return alt.to_string();
        }
        return "[image]".to_string();
    }
    if let Some(value) = dom.get_attribute(id, "value") {
        if !value.is_empty() {
            return value.to_string();
        }
    }
    // Fall back to text content
    let text = dom.text_content(id);
    let trimmed = text.trim();
    if trimmed.is_empty() {
        String::new()
    } else {
        trimmed.to_string()
    }
}

/// Compute states from DOM attributes.
fn compute_states(dom: &DomTree, id: NodeId) -> States {
    States {
        disabled: dom.get_attribute(id, "disabled").is_some(),
        checked: dom
            .get_attribute(id, "checked")
            .map(|_| true)
            .or_else(|| dom.get_attribute(id, "aria-checked").map(|v| v == "true")),
        focused: dom.get_attribute(id, "autofocus").is_some(),
        expanded: dom.get_attribute(id, "aria-expanded").map(|v| v == "true"),
        selected: dom.get_attribute(id, "selected").is_some(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_and_build(html: &str) -> A11yTree {
        let dom = be_parser::parse_html(html).unwrap();
        build(&dom)
    }

    #[test]
    fn test_empty_tree() {
        let tree = parse_and_build("");
        // Should have at least the html structure
        assert!(!tree.nodes.is_empty());
    }

    #[test]
    fn test_button_role() {
        let tree = parse_and_build(r#"<button>Click me</button>"#);
        let button = tree.nodes.iter().find(|n| n.role == Role::Button);
        assert!(button.is_some());
        assert_eq!(button.unwrap().label, "Click me");
    }

    #[test]
    fn test_link_role() {
        let tree = parse_and_build(r#"<a href="/home">Home</a>"#);
        let link = tree.nodes.iter().find(|n| n.role == Role::Link);
        assert!(link.is_some());
        assert_eq!(link.unwrap().label, "Home");
    }

    #[test]
    fn test_heading_role() {
        let tree = parse_and_build(r#"<h1>Title</h1>"#);
        let heading = tree.nodes.iter().find(|n| n.role == Role::Heading);
        assert!(heading.is_some());
    }

    #[test]
    fn test_input_textbox() {
        let tree = parse_and_build(r#"<input type="text" aria-label="Name">"#);
        let textbox = tree.nodes.iter().find(|n| n.role == Role::Textbox);
        assert!(textbox.is_some());
        assert_eq!(textbox.unwrap().label, "Name");
    }

    #[test]
    fn test_input_checkbox() {
        let tree = parse_and_build(r#"<input type="checkbox" checked>"#);
        let checkbox = tree.nodes.iter().find(|n| n.role == Role::Checkbox);
        assert!(checkbox.is_some());
        assert_eq!(checkbox.unwrap().states.checked, Some(true));
    }

    #[test]
    fn test_disabled_state() {
        let tree = parse_and_build(r#"<button disabled>Off</button>"#);
        let button = tree.nodes.iter().find(|n| n.role == Role::Button);
        assert!(button.is_some());
        assert!(button.unwrap().states.disabled);
    }

    #[test]
    fn test_image_alt() {
        let tree = parse_and_build(r#"<img alt="Logo" src="logo.png">"#);
        let img = tree.nodes.iter().find(|n| n.role == Role::Image);
        assert!(img.is_some());
        assert_eq!(img.unwrap().label, "Logo");
    }

    #[test]
    fn test_paragraph() {
        let tree = parse_and_build(r#"<p>Hello world</p>"#);
        let p = tree.nodes.iter().find(|n| n.role == Role::Paragraph);
        assert!(p.is_some());
        assert_eq!(p.unwrap().label, "Hello world");
    }
}
