//! # be-parser — HTML parser
//!
//! Parses HTML source into a DOM tree using Servo's html5ever.
//!
//! ## Blast radius
//!
//! Depends on be-dom only. Changes here affect all downstream subsystems.

use be_dom::{DomTree, Namespace, NodeId};
use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use std::io::Cursor;
use thiserror::Error;

/// Errors during HTML parsing.
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("parse error: {0}")]
    Parse(String),
}

/// Parse HTML source into a DOM tree.
///
/// Uses Servo's html5ever parser. Handles full HTML5 parsing
/// including implied tags, error recovery, and namespace handling.
pub fn parse_html(source: &str) -> Result<DomTree, ParseError> {
    let dom = parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .read_from(&mut Cursor::new(source.as_bytes()))
        .map_err(|e| ParseError::Parse(format!("{:?}", e)))?;

    let mut tree = DomTree::new();
    let root = tree.root();
    convert_node(&dom.document, &mut tree, root);
    Ok(tree)
}

/// Recursively convert an html5ever RcDom node into our DomTree.
fn convert_node(handle: &Handle, tree: &mut DomTree, parent: NodeId) {
    let node = handle;
    match &node.data {
        NodeData::Document => {
            // Document node — just process children
            for child in node.children.borrow().iter() {
                convert_node(child, tree, parent);
            }
        }
        NodeData::Element { name, attrs, .. } => {
            let tag = &name.local;
            let ns = match name.ns.as_ref() {
                "http://www.w3.org/1999/xhtml" => Namespace::Html,
                "http://www.w3.org/2000/svg" => Namespace::Svg,
                "http://www.w3.org/1998/Math/MathML" => Namespace::MathMl,
                _ => Namespace::Html,
            };
            let elem = tree.create_element(tag.as_ref(), ns);
            // Copy attributes
            for attr in attrs.borrow().iter() {
                tree.set_attribute(elem, &attr.name.local, &attr.value);
            }
            tree.append_child(parent, elem);
            // Process children
            for child in node.children.borrow().iter() {
                convert_node(child, tree, elem);
            }
        }
        NodeData::Text { contents } => {
            let text = contents.borrow().to_string();
            if !text.is_empty() {
                let text_node = tree.create_text(&text);
                tree.append_child(parent, text_node);
            }
        }
        NodeData::Comment { contents } => {
            let comment = tree.create_comment(contents);
            tree.append_child(parent, comment);
        }
        _ => {
            // Doctype, processing instructions — skip for now
            for child in node.children.borrow().iter() {
                convert_node(child, tree, parent);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let html = r#"<html><body><h1>Hello</h1></body></html>"#;
        let dom = parse_html(html).unwrap();
        assert!(dom.len() > 5); // document + html + head + body + h1 + text
    }

    #[test]
    fn test_parse_empty() {
        let dom = parse_html("").unwrap();
        assert!(dom.len() > 0);
    }

    #[test]
    fn test_parse_attributes() {
        let html = r#"<div class="test" id="main">content</div>"#;
        let dom = parse_html(html).unwrap();
        // Find the div — it'll be under html > body > div
        let root = dom.root();
        let html_node = dom.children(root)[0]; // html
        // html has head and body as children
        let body = dom.children(html_node).iter().find(|&&id| {
            dom.tag_name(id) == Some("body")
        }).copied().unwrap();
        let div = dom.children(body)[0];
        assert_eq!(dom.tag_name(div), Some("div"));
        assert_eq!(dom.get_attribute(div, "class"), Some("test"));
        assert_eq!(dom.get_attribute(div, "id"), Some("main"));
    }

    #[test]
    fn test_parse_nesting() {
        let html = r#"<ul><li>A</li><li>B</li></ul>"#;
        let dom = parse_html(html).unwrap();
        assert!(dom.len() > 5); // document + html + head + body + ul + 2*li + 2*text
    }

    #[test]
    fn test_parse_malformed() {
        // html5ever handles error recovery
        let html = r#"<div><p>unclosed<div>new</div>"#;
        let dom = parse_html(html).unwrap();
        assert!(dom.len() > 0);
    }
}
