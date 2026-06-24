//! Traceability: AXIOM_TAPE_APPEND_ONLY, AXIOM_BRAID_CANONICAL.
use crate::browser_types::*;
use alloc::string::String;
use alloc::vec::Vec;

/// Append-only, content-addressed, hash-chained fact store.
pub struct FactStore {
    pub head: Option<Cid>,
    pub entries: Vec<TapeEntry>,
}

impl FactStore {
    pub fn new() -> Self {
        Self {
            head: None,
            entries: Vec::new(),
        }
    }

    pub fn append(&mut self, anchor: &WebAnchor) -> Result<Cid, &'static str> {
        let entry = TapeEntry {
            index: self.entries.len(),
            cid: anchor.cid,
            term_family: anchor.term_family.as_str().into(),
            timestamp: anchor.created_at.clone(),
        };
        self.entries.push(entry);
        self.head = Some(anchor.cid);
        Ok(anchor.cid)
    }

    pub fn verify_chain(&self) -> Result<(), &'static str> {
        todo!("verify hash chain integrity")
    }
}

impl Default for FactStore {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TapeEntry {
    pub index: usize,
    pub cid: Cid,
    pub term_family: String,
    pub timestamp: String,
}
