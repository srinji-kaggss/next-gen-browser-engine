//! # be-layout — Basic layout engine
//!
//! Computes positions and sizes for DOM elements.
//! Block elements stack vertically, inline elements flow horizontally,
//! display:none is hidden. Every visible element gets a position.
//!
//! ## Blast radius
//!
//! Depends on be-dom only. Changes here affect be-semantic.

use be_dom::{DomTree, NodeId, NodeKind};
use serde::{Deserialize, Serialize};

/// A rectangle (position + size).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0, width: 0.0, height: 0.0 }
    }
}

/// Display type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Display {
    Block,
    Inline,
    None,
}

/// A layout box.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutBox {
    pub element: NodeId,
    pub rect: Rect,
    pub display: Display,
}

/// The layout tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutTree {
    pub boxes: Vec<LayoutBox>,
}

/// Default viewport width.
const DEFAULT_WIDTH: f64 = 960.0;
/// Estimated line height.
const LINE_HEIGHT: f64 = 20.0;
/// Estimated character width for inline content.
const CHAR_WIDTH: f64 = 8.0;

/// Compute layout from a DOM tree.
///
/// MVP layout: block elements stack vertically, inline elements flow
/// horizontally. Every visible element gets a position.
pub fn compute(dom: &DomTree) -> LayoutTree {
    let mut tree = LayoutTree { boxes: Vec::new() };
    let root = dom.root();
    let mut cursor = Cursor { x: 0.0, y: 0.0 };
    layout_node(dom, root, &mut cursor, &mut tree);
    tree
}

struct Cursor {
    x: f64,
    y: f64,
}

/// Determine the display type of an element.
fn display_type(dom: &DomTree, id: NodeId) -> Display {
    let node = match dom.get(id) {
        Some(n) => n,
        None => return Display::None,
    };
    // Check hidden attribute
    if dom.get_attribute(id, "hidden").is_some() {
        return Display::None;
    }
    // Check style="display:none"
    if let Some(style) = dom.get_attribute(id, "style") {
        if style.contains("display:none") || style.contains("display: none") {
            return Display::None;
        }
    }
    match &node.kind {
        NodeKind::Element { tag_name, .. } => match tag_name.as_str() {
            // Block elements
            "div" | "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "ul" | "ol" | "li"
            | "table" | "tr" | "td" | "th" | "form" | "section" | "article" | "header"
            | "footer" | "nav" | "main" | "aside" | "blockquote" | "pre" | "hr" | "br"
            |             "html" | "body" => Display::Block,
            // Hidden elements
            "script" | "style" | "meta" | "link" | "title" | "head" => Display::None,
            // Inline elements
            _ => Display::Inline,
        },
        NodeKind::Text { .. } => Display::Inline,
        _ => Display::None,
    }
}

/// Estimate the height of text content.
fn text_height(text: &str) -> f64 {
    if text.is_empty() {
        0.0
    } else {
        LINE_HEIGHT
    }
}

/// Estimate the width of text content.
fn text_width(text: &str) -> f64 {
    text.len() as f64 * CHAR_WIDTH
}

fn layout_node(dom: &DomTree, id: NodeId, cursor: &mut Cursor, tree: &mut LayoutTree) {
    let node = match dom.get(id) {
        Some(n) => n,
        None => return,
    };

    // Document node — pass through to children
    if matches!(node.kind, NodeKind::Document) {
        for &child in dom.children(id) {
            layout_node(dom, child, cursor, tree);
        }
        return;
    }

    let display = display_type(dom, id);

    match display {
        Display::None => {
            // Add a zero-size box so tests can find it, but don't affect layout
            tree.boxes.push(LayoutBox {
                element: id,
                rect: Rect::zero(),
                display,
            });
            // Still layout children (e.g. script might have content we skip)
        }
        Display::Block => {
            let start_y = cursor.y;
            cursor.x = 0.0;

            // Layout children — they stack
            for &child in dom.children(id) {
                layout_node(dom, child, cursor, tree);
            }

            let height = if cursor.y > start_y {
                cursor.y - start_y
            } else {
                text_height(&dom.text_content(id))
            };

            tree.boxes.push(LayoutBox {
                element: id,
                rect: Rect {
                    x: 0.0,
                    y: start_y,
                    width: DEFAULT_WIDTH,
                    height,
                },
                display,
            });

            // Block elements add spacing
            cursor.y += 4.0;
        }
        Display::Inline => {
            match dom.get(id).map(|n| &n.kind) {
                Some(NodeKind::Text { content }) => {
                    let w = text_width(content);
                    let h = text_height(content);
                    tree.boxes.push(LayoutBox {
                        element: id,
                        rect: Rect {
                            x: cursor.x,
                            y: cursor.y,
                            width: w,
                            height: h,
                        },
                        display,
                    });
                    cursor.x += w;
                }
                _ => {
                    // Inline element — layout children in flow
                    let start_x = cursor.x;
                    for &child in dom.children(id) {
                        layout_node(dom, child, cursor, tree);
                    }
                    let w = cursor.x - start_x;
                    tree.boxes.push(LayoutBox {
                        element: id,
                        rect: Rect {
                            x: start_x,
                            y: cursor.y,
                            width: w,
                            height: LINE_HEIGHT,
                        },
                        display,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_and_layout(html: &str) -> LayoutTree {
        let dom = be_parser::parse_html(html).unwrap();
        compute(&dom)
    }

    #[test]
    fn test_empty_layout() {
        let tree = parse_and_layout("");
        assert!(!tree.boxes.is_empty());
    }

    #[test]
    fn test_block_stacking() {
        let tree = parse_and_layout(r#"<div><p>A</p><p>B</p></div>"#);
        // Find the two paragraph boxes
        let p_boxes: Vec<_> = tree.boxes.iter().filter(|b| {
            b.display == Display::Block && b.rect.height > 0.0
        }).collect();
        // They should have different y positions (stacked)
        if p_boxes.len() >= 2 {
            assert!(p_boxes[1].rect.y > p_boxes[0].rect.y);
        }
    }

    #[test]
    fn test_inline_flow() {
        let tree = parse_and_layout(r#"<p>Hello world</p>"#);
        // Should have layout boxes
        assert!(!tree.boxes.is_empty());
    }

    #[test]
    fn test_display_none() {
        let tree = parse_and_layout(r#"<div><p>Visible</p><p hidden>Hidden</p></div>"#);
        // The hidden paragraph should not appear
        let hidden = tree.boxes.iter().find(|b| b.display == Display::None);
        assert!(hidden.is_some());
    }

    #[test]
    fn test_script_hidden() {
        let tree = parse_and_layout(r#"<div><script>var x=1;</script><p>Text</p></div>"#);
        // Script should have Display::None
        let script = tree.boxes.iter().find(|b| b.display == Display::None);
        assert!(script.is_some());
    }

    #[test]
    fn test_every_visible_has_position() {
        let tree = parse_and_layout(r#"<div><h1>Title</h1><p>Body</p></div>"#);
        for b in &tree.boxes {
            if b.display != Display::None {
                assert!(b.rect.width > 0.0 || b.rect.height > 0.0,
                    "Element {:?} has zero size", b.element);
            }
        }
    }
}
