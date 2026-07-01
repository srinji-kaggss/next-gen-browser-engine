use crate::error::FailClosed;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NormalizedTerm(pub String);

impl NormalizedTerm {
    pub fn new(s: impl AsRef<str>) -> Result<Self, FailClosed> {
        let s = s.as_ref();
        if s.is_empty() {
            return Err(FailClosed::BadPlan);
        }
        if s.chars().any(|c| {
            c.is_control() || matches!(c, '"' | '\'' | '\\' | '/' | '(' | ')' | '$' | '+' | '.')
        }) {
            return Err(FailClosed::BadPlan);
        }
        Ok(NormalizedTerm(s.to_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Field {
    Tag,
    Role,
    Text,
    AriaLabel,
}

#[derive(Clone, Debug, Default)]
pub struct EmbeddingRef;

pub type ScopeFilter = u64;

#[derive(Clone, Debug)]
pub enum QueryPlan {
    Term(Field, NormalizedTerm),
    Phrase(Field, Vec<NormalizedTerm>),
    Vector(EmbeddingRef, ScopeFilter),
    And(Box<QueryPlan>, Box<QueryPlan>),
    Or(Box<QueryPlan>, Box<QueryPlan>),
    Empty,
}

const MAX_QUERY_LEN: usize = 1024;

fn is_forbidden(c: char) -> bool {
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

impl QueryPlan {
    /// Parse raw user input into an eval-free AST. Rejects operator chars,
    /// control chars, and overlong input (spec §4.2, L2).
    pub fn parse(raw: &str) -> Result<Self, FailClosed> {
        if raw.len() > MAX_QUERY_LEN {
            return Err(FailClosed::BadPlan);
        }
        // Reject control chars BEFORE trimming — a bare "\n" contains only
        // whitespace+control and trimming would mask it.
        if raw.chars().any(|c| c.is_control()) {
            return Err(FailClosed::BadPlan);
        }
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Ok(QueryPlan::Empty);
        }
        if trimmed.chars().any(is_forbidden) {
            return Err(FailClosed::BadPlan);
        }
        let terms: Vec<QueryPlan> = trimmed
            .split_whitespace()
            .map(|w| QueryPlan::Term(Field::Text, NormalizedTerm::new(w).unwrap()))
            .collect();
        if terms.is_empty() {
            return Ok(QueryPlan::Empty);
        }
        Ok(terms
            .into_iter()
            .reduce(|acc, t| QueryPlan::And(Box::new(acc), Box::new(t)))
            .unwrap())
    }

    pub fn empty() -> Self {
        QueryPlan::Empty
    }
}
