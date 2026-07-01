pub use tantivy::query::Occur;
use tantivy::query::{AllQuery, BooleanQuery, EmptyQuery, PhraseQuery, Query, TermQuery};
use tantivy::schema::{Field, IndexRecordOption};
use tantivy::Term;

use crate::index::{canonical_fields, Hit, IndexFields};
use crate::partition::scope_hash;
use crate::query::{Field as QField, QueryPlan};
use crate::scope::Scope;

/// A scoped query result with an inspectable guard. Returned by
/// [`inject_scope_filter`] so security tests can verify the guard.
pub struct GuardedPlan {
    pub query: Box<dyn Query + Sync>,
    pub guard: Occur,
    pub scope_hash: u64,
}

impl GuardedPlan {
    pub fn as_query(&self) -> &(dyn Query + Sync) {
        self.query.as_ref()
    }
}

/// Inject a scope filter (mandatory `Occur::Must` TermQuery) alongside a user
/// query. Returns a [`GuardedPlan`] with inspectable `.guard` and `.scope_hash`.
pub fn inject_scope_filter(plan: QueryPlan, scope: &Scope) -> GuardedPlan {
    let fields = canonical_fields();
    let user_query = build_from_plan(&plan, fields);
    let hash = scope_hash(scope);
    let guard = scope_guard(fields.scope_hash, hash);

    GuardedPlan {
        query: Box::new(BooleanQuery::new(vec![
            (Occur::Must, user_query),
            (Occur::Must, guard),
        ])),
        guard: Occur::Must,
        scope_hash: hash,
    }
}

pub fn build_scoped_query(plan: &QueryPlan, scope: &Scope) -> Box<dyn Query + Sync> {
    let fields = canonical_fields();
    let user_query = build_from_plan(plan, fields);
    let hash = scope_hash(scope);
    let guard = scope_guard(fields.scope_hash, hash);

    Box::new(BooleanQuery::new(vec![
        (Occur::Must, user_query),
        (Occur::Must, guard),
    ]))
}

pub fn post_filter(hits: Vec<Hit>, _scope: &Scope) -> Vec<Hit> {
    hits
}

fn scope_guard(scope_hash_field: Field, hash: u64) -> Box<dyn Query + Sync> {
    let term = Term::from_field_u64(scope_hash_field, hash);
    Box::new(TermQuery::new(term, IndexRecordOption::Basic))
}

fn resolve_field(f: &QField, fields: IndexFields) -> Field {
    match f {
        QField::Tag => fields.tag,
        QField::Role => fields.role,
        QField::Text => fields.text,
        QField::AriaLabel => fields.aria_label,
    }
}

fn build_from_plan(plan: &QueryPlan, fields: IndexFields) -> Box<dyn Query + Sync> {
    match plan {
        QueryPlan::Term(f, term) => {
            let field = resolve_field(f, fields);
            let t = Term::from_field_text(field, term.as_str());
            Box::new(TermQuery::new(t, IndexRecordOption::WithFreqs))
        }
        QueryPlan::Phrase(f, terms) => {
            let field = resolve_field(f, fields);
            let phrase: Vec<Term> = terms
                .iter()
                .map(|t| Term::from_field_text(field, t.as_str()))
                .collect();
            Box::new(PhraseQuery::new(phrase))
        }
        QueryPlan::And(a, b) => Box::new(BooleanQuery::new(vec![
            (Occur::Must, build_from_plan(a, fields)),
            (Occur::Must, build_from_plan(b, fields)),
        ])),
        QueryPlan::Or(a, b) => Box::new(BooleanQuery::new(vec![
            (Occur::Should, build_from_plan(a, fields)),
            (Occur::Should, build_from_plan(b, fields)),
        ])),
        QueryPlan::Empty => Box::new(AllQuery),
        QueryPlan::Vector(_, _) => Box::new(EmptyQuery),
    }
}
