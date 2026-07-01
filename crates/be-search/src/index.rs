use std::sync::{Mutex, OnceLock};

use tantivy::collector::TopDocs;
use tantivy::indexer::UserOperation;
use tantivy::schema::{Facet, FacetOptions, Field, NumericOptions, Schema, TEXT};
use tantivy::{doc, Index, IndexReader, IndexWriter, Opstamp, ReloadPolicy, TantivyDocument, Term};

use crate::error::FailClosed;
use crate::scope::Scope;

const WRITER_HEAP_BYTES: usize = 50_000_000;

#[derive(Clone, Copy, Debug)]
pub(crate) struct IndexFields {
    pub tag: Field,
    pub role: Field,
    pub text: Field,
    pub aria_label: Field,
    pub scope_hash: Field,
    pub node_kind: Field,
}

fn build_schema() -> (Schema, IndexFields) {
    let mut b = Schema::builder();
    let tag = b.add_text_field("tag", TEXT);
    let role = b.add_text_field("role", TEXT);
    let text = b.add_text_field("text", TEXT);
    let aria_label = b.add_text_field("aria_label", TEXT);
    let scope_hash = b.add_u64_field(
        "scope_hash",
        NumericOptions::default().set_indexed().set_fast(),
    );
    let node_kind = b.add_facet_field("node_kind", FacetOptions::default());
    (
        b.build(),
        IndexFields {
            tag,
            role,
            text,
            aria_label,
            scope_hash,
            node_kind,
        },
    )
}

static CANONICAL: OnceLock<(Schema, IndexFields)> = OnceLock::new();

fn canonical() -> &'static (Schema, IndexFields) {
    CANONICAL.get_or_init(build_schema)
}

pub(crate) fn canonical_fields() -> IndexFields {
    canonical().1
}

#[derive(Clone, Debug)]
pub struct Hit {
    pub doc_address: tantivy::DocAddress,
    pub scope_hash: u64,
}

#[derive(Clone, Debug)]
pub struct NodeDoc {
    pub tag: String,
    pub role: String,
    pub text: String,
    pub aria_label: String,
    pub scope_hash: u64,
    pub node_kind: String,
}

impl NodeDoc {
    pub(crate) fn to_tantivy(&self, fields: IndexFields) -> TantivyDocument {
        let kind = Facet::from_text(&self.node_kind).unwrap_or_else(|_| Facet::root());
        doc!(
            fields.tag => self.tag.as_str(),
            fields.role => self.role.as_str(),
            fields.text => self.text.as_str(),
            fields.aria_label => self.aria_label.as_str(),
            fields.scope_hash => self.scope_hash,
            fields.node_kind => kind,
        )
    }
}

pub struct SearchIndex {
    #[allow(dead_code)]
    index: Index,
    fields: IndexFields,
    writer: Mutex<IndexWriter<TantivyDocument>>,
    reader: IndexReader,
}

impl std::fmt::Debug for SearchIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SearchIndex")
            .field("fields", &self.fields)
            .finish()
    }
}

impl SearchIndex {
    pub fn create_partition(scope: &Scope) -> Self {
        let schema = canonical().0.clone();
        let fields = canonical().1;
        let index = Index::create_in_ram(schema);
        let writer = index
            .writer_with_num_threads(1, WRITER_HEAP_BYTES)
            .expect("RAM index writer allocation must succeed");
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()
            .expect("reader construction for a fresh RAM index must succeed");
        let _ = scope;
        Self {
            index,
            fields,
            writer: Mutex::new(writer),
            reader,
        }
    }

    pub(crate) fn fields(&self) -> IndexFields {
        self.fields
    }

    pub(crate) fn run_ops(
        &self,
        ops: Vec<UserOperation<TantivyDocument>>,
    ) -> Result<Opstamp, FailClosed> {
        let writer = self.writer.lock().map_err(|_| FailClosed::IndexDown)?;
        writer.run(ops).map_err(|_| FailClosed::IndexDown)
    }

    pub fn add_node(&self, node: &NodeDoc) -> Result<Opstamp, FailClosed> {
        let document = node.to_tantivy(self.fields);
        let writer = self.writer.lock().map_err(|_| FailClosed::IndexDown)?;
        writer
            .add_document(document)
            .map_err(|_| FailClosed::IndexDown)
    }

    pub fn delete_by_scope(&self, scope_hash: u64) -> Result<Opstamp, FailClosed> {
        let term = Term::from_field_u64(self.fields.scope_hash, scope_hash);
        let writer = self.writer.lock().map_err(|_| FailClosed::IndexDown)?;
        Ok(writer.delete_term(term))
    }

    pub fn commit(&self) -> Result<(), FailClosed> {
        let mut writer = self.writer.lock().map_err(|_| FailClosed::IndexDown)?;
        writer.commit().map_err(|_| FailClosed::IndexDown)?;
        drop(writer);
        self.reader.reload().map_err(|_| FailClosed::IndexDown)?;
        Ok(())
    }

    pub fn search(
        &self,
        query: &(dyn tantivy::query::Query + Sync),
        limit: usize,
    ) -> Result<Vec<tantivy::DocAddress>, FailClosed> {
        let searcher = self.reader.searcher();
        let hits = searcher
            .search(query, &TopDocs::with_limit(limit).order_by_score())
            .map_err(|_| FailClosed::IndexDown)?;
        Ok(hits.into_iter().map(|(_score, addr)| addr).collect())
    }
}
