//! Integration test: HTML → DOM → a11y → layout → semantic graph → PULSE frames.
//!
//! Tests the full pipeline from raw HTML to AI-consumable affordances.

use be_a11y::Role;
use be_parser::parse_html;
use be_pulse::{build_frames, encode_affordances, Frame};
use be_semantic::{Action, SemanticGraph};

/// Parse HTML through the full pipeline and return the semantic graph.
fn pipeline(html: &str) -> SemanticGraph {
    let dom = parse_html(html).expect("parse failed");
    let a11y = be_a11y::build(&dom);
    let layout = be_layout::compute(&dom);
    be_semantic::build(&dom, &a11y, &layout)
}

#[test]
fn test_full_pipeline_button() {
    let graph = pipeline(r#"<button>Sign In</button>"#);

    // Should have a button node with click affordance
    let button = graph.nodes.iter().find(|n| n.role == Role::Button);
    assert!(button.is_some(), "Button node missing from semantic graph");
    assert_eq!(button.unwrap().label, "Sign In");

    let click = graph
        .affordances
        .iter()
        .find(|a| matches!(a.action, Action::Click) && a.target == button.unwrap().handle);
    assert!(click.is_some(), "Click affordance missing for button");
}

#[test]
fn test_full_pipeline_form() {
    let html = r#"
        <form>
            <label for="email">Email</label>
            <input type="email" id="email" aria-label="Email address">
            <button type="submit">Submit</button>
        </form>
    "#;
    let graph = pipeline(html);

    // Should have a textbox and a button
    let textbox = graph.nodes.iter().find(|n| n.role == Role::Textbox);
    assert!(textbox.is_some(), "Textbox missing");

    let button = graph.nodes.iter().find(|n| n.role == Role::Button);
    assert!(button.is_some(), "Submit button missing");

    // Textbox should have fill affordance
    let fill = graph
        .affordances
        .iter()
        .find(|a| matches!(a.action, Action::Fill { .. }));
    assert!(fill.is_some(), "Fill affordance missing for textbox");

    // Button should have click affordance
    let click = graph
        .affordances
        .iter()
        .find(|a| matches!(a.action, Action::Click));
    assert!(
        click.is_some(),
        "Click affordance missing for submit button"
    );
}

#[test]
fn test_full_pipeline_links() {
    let html = r#"<nav><a href="/about">About</a><a href="/contact">Contact</a></nav>"#;
    let graph = pipeline(html);

    let links: Vec<_> = graph
        .nodes
        .iter()
        .filter(|n| n.role == Role::Link)
        .collect();
    assert_eq!(links.len(), 2, "Expected 2 links, got {}", links.len());
}

#[test]
fn test_full_pipeline_pulse_encoding() {
    let html = r#"<button>Click Me</button>"#;
    let graph = pipeline(html);
    let encoded = encode_affordances(&graph);

    assert!(encoded.starts_with("ok affordances"));
    assert!(encoded.contains("click"));
    assert!(encoded.contains("Click Me"));
}

#[test]
fn test_full_pipeline_pulse_frames() {
    let html = r#"<h1>Welcome</h1><button>Start</button>"#;
    let graph = pipeline(html);
    let frame = build_frames(&graph);

    match frame {
        Frame::OkAffordances { affordances } => {
            // Should have at least a read (heading) and click (button)
            let has_read = affordances.iter().any(|a| a.action == "read");
            let has_click = affordances.iter().any(|a| a.action == "click");
            assert!(has_read, "Missing read affordance for heading");
            assert!(has_click, "Missing click affordance for button");
        }
        _ => panic!("Expected OkAffordances frame"),
    }
}

#[test]
fn test_full_pipeline_checkbox() {
    let html = r#"<input type="checkbox" checked aria-label="Accept terms">"#;
    let graph = pipeline(html);

    let checkbox = graph.nodes.iter().find(|n| n.role == Role::Checkbox);
    assert!(checkbox.is_some(), "Checkbox missing");
    assert_eq!(checkbox.unwrap().label, "Accept terms");
}

#[test]
fn test_full_pipeline_hidden_elements() {
    let html = r#"<div><p>Visible</p><p hidden>Hidden</p></div>"#;
    let graph = pipeline(html);

    // Hidden paragraph should not have affordances
    let hidden_p = graph
        .nodes
        .iter()
        .find(|n| n.role == Role::Paragraph && n.label == "Hidden");
    // Hidden elements may still be in the graph but should have zero position
    if let Some(hidden) = hidden_p {
        assert_eq!(
            hidden.position.width, 0.0,
            "Hidden element should have zero width"
        );
    }
}

#[test]
fn test_full_pipeline_complex_page() {
    let html = r#"
        <html>
        <body>
            <header>
                <h1>My App</h1>
                <nav>
                    <a href="/home">Home</a>
                    <a href="/settings">Settings</a>
                </nav>
            </header>
            <main>
                <form>
                    <input type="text" aria-label="Search">
                    <button>Search</button>
                </form>
                <div>
                    <h2>Results</h2>
                    <p>Found 42 items</p>
                </div>
            </main>
            <footer>
                <p>2026 My App</p>
            </footer>
        </body>
        </html>
    "#;
    let graph = pipeline(html);

    // Should have multiple affordances
    assert!(
        graph.affordances.len() >= 5,
        "Expected at least 5 affordances, got {}",
        graph.affordances.len()
    );

    // Check specific roles exist
    let has_heading = graph.nodes.iter().any(|n| n.role == Role::Heading);
    let has_link = graph.nodes.iter().any(|n| n.role == Role::Link);
    let has_textbox = graph.nodes.iter().any(|n| n.role == Role::Textbox);
    let has_button = graph.nodes.iter().any(|n| n.role == Role::Button);
    let has_paragraph = graph.nodes.iter().any(|n| n.role == Role::Paragraph);

    assert!(has_heading, "Missing heading");
    assert!(has_link, "Missing link");
    assert!(has_textbox, "Missing textbox");
    assert!(has_button, "Missing button");
    assert!(has_paragraph, "Missing paragraph");
}

#[test]
fn test_full_pipeline_malformed_html() {
    // html5ever should handle error recovery
    let html = r#"<div><p>unclosed<div>new</div>"#;
    let graph = pipeline(html);

    // Should still produce a valid graph
    assert!(!graph.nodes.is_empty());
}

#[test]
fn test_full_pipeline_empty() {
    let graph = pipeline("");
    // Empty HTML still has a document structure
    assert!(!graph.nodes.is_empty());
}
