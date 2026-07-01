//! # be-api — HTTP API server
//!
//! Exposes the semantic graph and PULSE frames over HTTP.
//!
//! ## Blast radius
//!
//! Depends on be-parser, be-semantic, be-pulse.
//! This is the outermost layer. Changes here don't affect internals.

use axum::extract::Query;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
pub fn app() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/parse", post(parse_handler))
        .route("/fetch", get(fetch_page))
        .route("/transpile", post(transpile_js))
        .route("/load", get(load_page))
}

async fn health() -> &'static str {
    "ok"
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

async fn fetch_page(
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let url = params.get("url").ok_or((
        axum::http::StatusCode::BAD_REQUEST,
        "Missing 'url' parameter".to_string(),
    ))?;
    let parsed_url = url::Url::parse(url).map_err(|e| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            format!("Invalid URL: {}", e),
        )
    })?;

    let client = be_net::build_client().map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Client build failed: {}", e),
        )
    })?;
    let response = be_net::fetch_url(&client, &parsed_url, &[])
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::BAD_GATEWAY,
                format!("Fetch failed: {}", e),
            )
        })?;

    Ok(Json(serde_json::json!({
        "url": response.url.to_string(),
        "status": response.status,
        "mime_type": response.mime_type,
        "body_size": response.body.len(),
        "headers": response.headers.iter().map(|(k, v)| (k.clone(), v.clone())).collect::<HashMap<_, _>>(),
    })))
}

async fn transpile_js(
    source: String,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let result = be_transpiler::transpile(&source).map_err(|e| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            format!("Transpile failed: {}", e),
        )
    })?;

    Ok(Json(serde_json::json!({
        "terms": result.terms,
        "escalations": result.escalations.len(),
        "escalation_details": result.escalations.iter().map(|e| serde_json::json!({
            "reason": e.reason,
            "level": format!("{:?}", e.level),
        })).collect::<Vec<_>>(),
    })))
}

async fn load_page(
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let url = params.get("url").ok_or((
        axum::http::StatusCode::BAD_REQUEST,
        "Missing 'url' parameter".to_string(),
    ))?;
    let parsed_url = url::Url::parse(url).map_err(|e| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            format!("Invalid URL: {}", e),
        )
    })?;

    // Step 1: Fetch
    let client = be_net::build_client().map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Client build failed: {}", e),
        )
    })?;
    let response = be_net::fetch_url(&client, &parsed_url, &[])
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::BAD_GATEWAY,
                format!("Fetch failed: {}", e),
            )
        })?;

    // Step 2: Parse HTML
    let html = String::from_utf8_lossy(&response.body).to_string();
    let dom = be_parser::parse_html(&html).map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Parse failed: {}", e),
        )
    })?;

    // Step 3: Extract scripts and transpile
    let script_ids = find_script_nodes(&dom);
    let mut transpile_results = Vec::new();
    for &id in &script_ids {
        let text = dom.text_content(id);
        if let Ok(result) = be_transpiler::transpile(&text) {
            transpile_results.push(serde_json::json!({
                "terms": result.terms,
                "escalations": result.escalations.len(),
            }));
        }
    }

    Ok(Json(serde_json::json!({
        "url": response.url.to_string(),
        "status": response.status,
        "mime_type": response.mime_type,
        "dom_nodes": dom.len(),
        "scripts_found": script_ids.len(),
        "scripts_transpiled": transpile_results.len(),
        "transpile_results": transpile_results,
    })))
}

/// Walk the DOM tree and collect NodeIds of all <script> elements.
fn find_script_nodes(dom: &be_dom::DomTree) -> Vec<be_dom::NodeId> {
    let mut result = Vec::new();
    walk_for_tag(dom, dom.root(), "script", &mut result);
    result
}

fn walk_for_tag(
    dom: &be_dom::DomTree,
    node_id: be_dom::NodeId,
    target_tag: &str,
    out: &mut Vec<be_dom::NodeId>,
) {
    if dom.tag_name(node_id) == Some(target_tag) {
        out.push(node_id);
    }
    for &child in dom.children(node_id) {
        walk_for_tag(dom, child, target_tag, out);
    }
}
