use tantivy::indexer::UserOperation;
use tantivy::Opstamp;
use tantivy::Term;

use crate::error::FailClosed;
use crate::index::{NodeDoc, SearchIndex};

#[derive(Clone, Debug)]
pub enum JournalOp {
    Add(NodeDoc),
    Delete(u64),
}

#[derive(Clone, Debug, Default)]
pub struct SearchJournal {
    entries: Vec<(Opstamp, JournalOp)>,
}

impl SearchJournal {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&mut self, op: JournalOp) -> Opstamp {
        let stamp = self.next_opstamp();
        self.entries.push((stamp, op));
        stamp
    }

    pub fn next_opstamp(&self) -> Opstamp {
        self.entries.len() as Opstamp
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn replay(&self, index: &SearchIndex) -> Result<(), FailClosed> {
        let fields = index.fields();
        let ops: Vec<UserOperation<tantivy::TantivyDocument>> = self
            .entries
            .iter()
            .map(|(_, op)| match op {
                JournalOp::Add(doc) => UserOperation::Add(doc.to_tantivy(fields)),
                JournalOp::Delete(scope_hash) => {
                    UserOperation::Delete(Term::from_field_u64(fields.scope_hash, *scope_hash))
                }
            })
            .collect();

        if !ops.is_empty() {
            index.run_ops(ops)?;
        }
        index.commit()?;
        Ok(())
    }
}
