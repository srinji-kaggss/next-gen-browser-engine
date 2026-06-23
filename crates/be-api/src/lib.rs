//! # be-api — HTTP API server
//!
//! Exposes the semantic graph and PULSE frames over HTTP.
//!
//! ## Blast radius
//!
//! Depends on be-parser, be-semantic, be-pulse.
//! This is the outermost layer. Changes here don't affect internals.

use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};

/// Request to parse HTML and get semantic graph.
#[derive(Debug, Deserialize)]
pub struct ParseRequest {
    pub html: String,
}

/// Response with semantic graph.
#[derive(Debug, Serialize)]
pub struct ParseResponse {
    pub node_count: usize,
    pub affordance_count: usize,
}

/// Create the API router.
pub fn router() -> Router {
    Router::new().route("/parse", post(parse_handler))
}

async fn parse_handler(Json(req): Json<ParseRequest>) -> Json<ParseResponse> {
    let dom = be_parser::parse_html(&req.html).unwrap_or_else(|_| be_dom::DomTree::new());
    let a11y = be_a11y::build(&dom);
    let layout = be_layout::compute(&dom);
    let graph = be_semantic::build(&dom, &a11y, &layout);

    Json(ParseResponse {
        node_count: graph.nodes.len(),
        affordance_count: graph.affordances.len(),
    })
}
