//! # be-search — Semantic Index state fabric
//!
//! Scoped, capability-gated retrieval over the semantic graph, hardened with a
//! 10-layer defense-in-depth pipeline (spec §5, §6). Every layer is
//! independently sufficient and composition is AND/intersection
//! (most-restrictive-wins), never OR/union (§2.6, Elastic DLS footgun).
//!
//! The ONLY public search entrypoint is [`BrowserSearch::search`].

pub mod audit;
pub mod cache;
pub mod candidate;
pub mod error;
pub mod fallback;
pub mod filter;
pub mod index;
pub mod journal;
pub mod partition;
pub mod query;
pub mod scope;

pub use audit::{AuditEntry, AuditLog, Outcome, TrustedTime};
pub use cache::{CacheKey, ScopeCache};
pub use candidate::{Candidate, NodeKind, SanitizedExcerpt};
pub use error::FailClosed;
pub use fallback::deterministic_fallback;
pub use filter::inject_scope_filter;
pub use query::QueryPlan;
pub use scope::{CapSet, Scope};

use query::{Field, NormalizedTerm};

pub type SearchResult = Result<Vec<Candidate>, FailClosed>;

const MAX_QUERY_LEN: usize = 4096;
const MAX_TERM_LEN: usize = 128;
const MAX_TERMS: usize = 64;

#[derive(Debug, Clone)]
pub struct SearchRequest {
    pub query: String,
    pub reason: String,
    pub limit: usize,
}

impl SearchRequest {
    pub fn new(query: impl Into<String>, limit: usize) -> Self {
        Self {
            query: query.into(),
            reason: String::new(),
            limit,
        }
    }

    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = reason.into();
        self
    }
}

#[derive(Debug)]
pub struct BrowserSearch {
    partitions: partition::PartitionStore,
    audit: AuditLog,
    cache: ScopeCache,
}

impl Default for BrowserSearch {
    fn default() -> Self {
        Self::new()
    }
}

impl BrowserSearch {
    pub fn new() -> Self {
        Self {
            partitions: partition::PartitionStore::new(),
            audit: AuditLog::new(),
            cache: ScopeCache::new(),
        }
    }

    pub fn search(&self, request: SearchRequest, scope: &Scope) -> SearchResult {
        // L2 — Sanitize the raw query into an eval-free AST.
        let plan = sanitize_parse(&request.query)?;

        // L3 — Field allowlist authorization.
        self.authorize(&plan)?;

        // L5 — Journal the query BEFORE execution (audit-before-effect).
        self.audit.log_query(scope, &plan, &request.reason)?;

        // Scope-bound cache (P8). Key is hashed with scope; read re-verifies.
        let key = CacheKey::derive(scope, &plan_fingerprint(&plan));
        if let Some(cached) = self.cache.get(&key, scope) {
            self.audit.log_results(scope, &cached)?;
            return Ok(cached);
        }

        // L4 + L6 — Build the scope-guarded query (scope predicate AND'd in)
        // and execute against the partitioned (per-scope) index.
        let guarded_query = filter::build_scoped_query(&plan, scope);
        let hits = match self
            .partitions
            .search(guarded_query.as_ref(), scope, request.limit)
        {
            Ok(hits) => hits,
            Err(_) => {
                let _ = self.audit.log_outcome(scope, Outcome::Fallback);
                return deterministic_fallback(&plan, scope);
            }
        };

        // L8 — Score suppression: Candidate has NO score field.
        let candidates: Vec<Candidate> = hits
            .into_iter()
            .map(|hit| Candidate {
                node_id: hit.doc_address.doc_id as u64,
                kind: NodeKind::ELEMENT,
                provenance: scope.clone(),
                ..Default::default()
            })
            .collect();

        // L9 — Completion receipt (audit evidence).
        self.audit.log_results(scope, &candidates)?;

        // Populate the scope-bound cache for future scoped queries.
        self.cache.insert(key, scope, candidates.clone());

        Ok(candidates)
    }

    fn authorize(&self, plan: &QueryPlan) -> Result<(), FailClosed> {
        if !field_is_allowed_tree(plan) {
            return Err(FailClosed::Unauthorized);
        }
        Ok(())
    }

    pub fn audit_trail(&self, scope: &Scope) -> Vec<AuditEntry> {
        self.audit.snapshot(scope)
    }
}

fn sanitize_parse(raw: &str) -> Result<QueryPlan, FailClosed> {
    if raw.len() > MAX_QUERY_LEN {
        return Err(FailClosed::BadPlan);
    }
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(QueryPlan::Empty);
    }
    if trimmed.chars().any(is_forbidden_char) {
        return Err(FailClosed::BadPlan);
    }
    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    if tokens.len() > MAX_TERMS {
        return Err(FailClosed::BadPlan);
    }
    let mut plan = QueryPlan::Empty;
    for tok in tokens {
        let term = normalize_term(tok)?;
        let leaf = QueryPlan::Term(Field::Text, term);
        plan = match plan {
            QueryPlan::Empty => leaf,
            acc => QueryPlan::And(Box::new(acc), Box::new(leaf)),
        };
    }
    Ok(plan)
}

fn normalize_term(raw: &str) -> Result<NormalizedTerm, FailClosed> {
    if raw.is_empty() || raw.len() > MAX_TERM_LEN {
        return Err(FailClosed::BadPlan);
    }
    if raw.chars().any(is_forbidden_char) {
        return Err(FailClosed::BadPlan);
    }
    let normalized = raw.trim().to_lowercase();
    if normalized.is_empty() {
        return Err(FailClosed::BadPlan);
    }
    NormalizedTerm::new(normalized)
}

fn is_forbidden_char(c: char) -> bool {
    if c.is_control() {
        return true;
    }
    matches!(
        c,
        '(' | ')'
            | '['
            | ']'
            | '{'
            | '}'
            | '"'
            | '\''
            | '\\'
            | '|'
            | '&'
            | '!'
            | '^'
            | '~'
            | '*'
            | '?'
            | ':'
            | '/'
            | '<'
            | '>'
            | '$'
            | '+'
            | '.'
    )
}

fn field_is_allowed_tree(plan: &QueryPlan) -> bool {
    match plan {
        QueryPlan::Term(f, _) | QueryPlan::Phrase(f, _) => *f == Field::Text,
        QueryPlan::And(a, b) | QueryPlan::Or(a, b) => {
            field_is_allowed_tree(a) && field_is_allowed_tree(b)
        }
        QueryPlan::Vector(_, _) => false,
        QueryPlan::Empty => true,
    }
}

fn plan_fingerprint(plan: &QueryPlan) -> Vec<u8> {
    let mut out = Vec::new();
    walk(plan, &mut out);
    out
}

fn walk(plan: &QueryPlan, out: &mut Vec<u8>) {
    match plan {
        QueryPlan::Term(f, t) => {
            out.extend(b"T;");
            out.extend(field_tag(f));
            out.extend(t.as_str().as_bytes());
            out.push(0);
        }
        QueryPlan::Phrase(f, terms) => {
            out.extend(b"P;");
            out.extend(field_tag(f));
            for t in terms {
                out.extend(t.as_str().as_bytes());
                out.push(b' ');
            }
            out.push(0);
        }
        QueryPlan::Vector(_, sf) => {
            out.extend(b"V;");
            out.extend(sf.to_le_bytes());
        }
        QueryPlan::And(a, b) => {
            out.extend(b"AND(");
            walk(a, out);
            out.push(b',');
            walk(b, out);
            out.push(b')');
        }
        QueryPlan::Or(a, b) => {
            out.extend(b"OR(");
            walk(a, out);
            out.push(b',');
            walk(b, out);
            out.push(b')');
        }
        QueryPlan::Empty => out.extend(b"EMPTY"),
    }
}

fn field_tag(f: &Field) -> &'static [u8] {
    match f {
        Field::Tag => b"tag",
        Field::Role => b"role",
        Field::Text => b"text",
        Field::AriaLabel => b"aria",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_parse_rejects_operators_and_oversized() {
        assert!(sanitize_parse("hello world").is_ok());
        assert!(sanitize_parse("").is_ok());
        assert!(sanitize_parse("a&b").is_err());
        assert!(sanitize_parse("a\"b").is_err());
        assert!(sanitize_parse(&"x".repeat(MAX_QUERY_LEN + 1)).is_err());
    }

    #[test]
    fn fingerprint_is_deterministic() {
        let a = sanitize_parse("hello world").unwrap();
        let b = sanitize_parse("hello world").unwrap();
        let c = sanitize_parse("hello").unwrap();
        assert_eq!(plan_fingerprint(&a), plan_fingerprint(&b));
        assert_ne!(plan_fingerprint(&a), plan_fingerprint(&c));
    }

    #[test]
    fn allowlist_accepts_text_denies_others() {
        let ok = QueryPlan::Term(Field::Text, NormalizedTerm::new("x").unwrap());
        assert!(field_is_allowed_tree(&ok));

        let bad = QueryPlan::Term(Field::Tag, NormalizedTerm::new("x").unwrap());
        assert!(!field_is_allowed_tree(&bad));

        let vec = QueryPlan::Vector(query::EmbeddingRef, 0);
        assert!(!field_is_allowed_tree(&vec));
    }
}
