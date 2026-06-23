//! # be-dom — DOM tree
//!
//! The DOM tree is the structural truth of a web page.
//! It is the input to:
//! - A11y tree builder (be-a11y)
//! - Layout engine (be-layout)
//! - Semantic graph builder (be-semantic)
//!
//! ## Blast radius
//!
//! This crate has NO external dependencies (only serde).
//! Changes here affect be-parser, be-a11y, be-layout, be-semantic.
//!
//! ## Example
//!
//! ```rust
//! use be_dom::*;
//!
//! let mut tree = DomTree::new();
//! let root = tree.root();
//! let div = tree.create_element("div", Namespace::Html);
//! tree.append_child(root, div);
//! assert_eq!(tree.children(root).len(), 1);
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Node identifier — stable for the lifetime of the tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub usize);

/// Which namespace an element belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Namespace {
    Html,
    Svg,
    MathMl,
}

/// What kind of node this is.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeKind {
    /// The root document node.
    Document,
    /// An HTML element.
    Element {
        tag_name: String,
        namespace: Namespace,
    },
    /// A text node.
    Text { content: String },
    /// A comment node.
    Comment { content: String },
}

/// A node in the DOM tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub kind: NodeKind,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub attributes: HashMap<String, String>,
}

/// The DOM tree — structural truth of a web page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomTree {
    nodes: Vec<Node>,
}

impl DomTree {
    /// Create a new DOM tree with a document root.
    pub fn new() -> Self {
        let root = Node {
            id: NodeId(0),
            kind: NodeKind::Document,
            parent: None,
            children: Vec::new(),
            attributes: HashMap::new(),
        };
        DomTree { nodes: vec![root] }
    }

    /// Get the root node ID.
    pub fn root(&self) -> NodeId {
        NodeId(0)
    }

    /// Get a node by ID.
    pub fn get(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id.0)
    }

    /// Get a mutable node by ID.
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(id.0)
    }

    /// Get the number of nodes.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if the tree is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Get children of a node.
    pub fn children(&self, id: NodeId) -> &[NodeId] {
        self.get(id)
            .map(|n| n.children.as_slice())
            .unwrap_or(&[])
    }

    /// Get the parent of a node.
    pub fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.get(id).and_then(|n| n.parent)
    }

    /// Create a new element node.
    pub fn create_element(&mut self, tag_name: &str, namespace: Namespace) -> NodeId {
        let id = NodeId(self.nodes.len());
        let node = Node {
            id,
            kind: NodeKind::Element {
                tag_name: tag_name.to_string(),
                namespace,
            },
            parent: None,
            children: Vec::new(),
            attributes: HashMap::new(),
        };
        self.nodes.push(node);
        id
    }

    /// Create a new text node.
    pub fn create_text(&mut self, content: &str) -> NodeId {
        let id = NodeId(self.nodes.len());
        let node = Node {
            id,
            kind: NodeKind::Text {
                content: content.to_string(),
            },
            parent: None,
            children: Vec::new(),
            attributes: HashMap::new(),
        };
        self.nodes.push(node);
        id
    }

    /// Create a new comment node.
    pub fn create_comment(&mut self, content: &str) -> NodeId {
        let id = NodeId(self.nodes.len());
        let node = Node {
            id,
            kind: NodeKind::Comment {
                content: content.to_string(),
            },
            parent: None,
            children: Vec::new(),
            attributes: HashMap::new(),
        };
        self.nodes.push(node);
        id
    }

    /// Append a child to a parent node.
    pub fn append_child(&mut self, parent: NodeId, child: NodeId) {
        // Remove child from old parent
        if let Some(old_parent) = self.get(child).and_then(|n| n.parent) {
            if let Some(p) = self.get_mut(old_parent) {
                p.children.retain(|&c| c != child);
            }
        }
        // Set new parent
        if let Some(child_node) = self.get_mut(child) {
            child_node.parent = Some(parent);
        }
        // Add to parent's children
        if let Some(parent_node) = self.get_mut(parent) {
            parent_node.children.push(child);
        }
    }

    /// Set an attribute on an element.
    pub fn set_attribute(&mut self, id: NodeId, name: &str, value: &str) {
        if let Some(node) = self.get_mut(id) {
            node.attributes.insert(name.to_string(), value.to_string());
        }
    }

    /// Get an attribute from an element.
    pub fn get_attribute(&self, id: NodeId, name: &str) -> Option<&str> {
        self.get(id)
            .and_then(|n| n.attributes.get(name))
            .map(|s| s.as_str())
    }

    /// Get the tag name of an element.
    pub fn tag_name(&self, id: NodeId) -> Option<&str> {
        self.get(id).and_then(|n| match &n.kind {
            NodeKind::Element { tag_name, .. } => Some(tag_name.as_str()),
            _ => None,
        })
    }

    /// Get the text content of a node (concatenated text of all descendant text nodes).
    pub fn text_content(&self, id: NodeId) -> String {
        let mut text = String::new();
        self.collect_text(id, &mut text);
        text
    }

    fn collect_text(&self, id: NodeId, text: &mut String) {
        if let Some(node) = self.get(id) {
            match &node.kind {
                NodeKind::Text { content } => text.push_str(content),
                _ => {
                    for child in &node.children {
                        self.collect_text(*child, text);
                    }
                }
            }
        }
    }

    /// Check if the tree is acyclic (no node is its own ancestor).
    pub fn is_acyclic(&self) -> bool {
        for node in &self.nodes {
            let mut visited = std::collections::HashSet::new();
            let mut current = node.parent;
            while let Some(id) = current {
                if !visited.insert(id) {
                    return false; // cycle detected
                }
                current = self.get(id).and_then(|n| n.parent);
            }
        }
        true
    }
}

impl Default for DomTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_tree_has_root() {
        let tree = DomTree::new();
        assert_eq!(tree.root(), NodeId(0));
        assert_eq!(tree.len(), 1);
    }

    #[test]
    fn test_create_element() {
        let mut tree = DomTree::new();
        let div = tree.create_element("div", Namespace::Html);
        assert_eq!(tree.tag_name(div), Some("div"));
    }

    #[test]
    fn test_append_child() {
        let mut tree = DomTree::new();
        let root = tree.root();
        let div = tree.create_element("div", Namespace::Html);
        tree.append_child(root, div);
        assert_eq!(tree.children(root), &[div]);
        assert_eq!(tree.parent(div), Some(root));
    }

    #[test]
    fn test_attributes() {
        let mut tree = DomTree::new();
        let div = tree.create_element("div", Namespace::Html);
        tree.set_attribute(div, "class", "test");
        assert_eq!(tree.get_attribute(div, "class"), Some("test"));
    }

    #[test]
    fn test_text_content() {
        let mut tree = DomTree::new();
        let root = tree.root();
        let div = tree.create_element("div", Namespace::Html);
        let text = tree.create_text("hello");
        tree.append_child(root, div);
        tree.append_child(div, text);
        assert_eq!(tree.text_content(div), "hello");
    }

    #[test]
    fn test_acyclic() {
        let mut tree = DomTree::new();
        let root = tree.root();
        let div = tree.create_element("div", Namespace::Html);
        tree.append_child(root, div);
        assert!(tree.is_acyclic());
    }
}
