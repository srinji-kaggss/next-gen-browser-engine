//! Traceability: AXIOM_TAPE_APPEND_ONLY, AXIOM_BRAID_CANONICAL.
use crate::browser_types::*;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use braid_ir::{canon, Value};

pub const WEB_TAPE_ENTRY_DOMAIN: &[u8] = b"lw.browser.tape.entry.v0";

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
        let prev_hash = self.head;
        let entry = TapeEntry {
            index: self.entries.len(),
            cid: anchor.cid,
            term_family: anchor.term_family.as_str().into(),
            timestamp: anchor.created_at.clone(),
            prev_hash,
            record_hash: Cid::compute(WEB_TAPE_ENTRY_DOMAIN, b"pending"),
        };
        let mut entry = entry;
        entry.record_hash = entry.compute_hash();
        let record_hash = entry.record_hash;
        self.entries.push(entry);
        self.head = Some(record_hash);
        Ok(record_hash)
    }

    pub fn verify_chain(&self) -> Result<(), &'static str> {
        let mut previous = None;
        for (index, entry) in self.entries.iter().enumerate() {
            if entry.index != index {
                return Err("tape entry index mismatch");
            }
            if entry.prev_hash != previous {
                return Err("tape entry previous hash mismatch");
            }
            if entry.record_hash != entry.compute_hash() {
                return Err("tape entry hash mismatch");
            }
            previous = Some(entry.record_hash);
        }
        if self.head != previous {
            return Err("tape head mismatch");
        }
        Ok(())
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
    pub prev_hash: Option<Cid>,
    pub record_hash: Cid,
}

impl TapeEntry {
    fn compute_hash(&self) -> Cid {
        Cid::compute(WEB_TAPE_ENTRY_DOMAIN, &canon::encode(&self.to_value()))
    }

    fn to_value(&self) -> Value {
        let prev_hash = match self.prev_hash {
            Some(cid) => Value::Bytes(cid.0.to_vec()),
            None => Value::List(Vec::new()),
        };
        Value::map(vec![
            ("cid", Value::Bytes(self.cid.0.to_vec())),
            ("index", Value::Int(self.index as i64)),
            ("prev_hash", prev_hash),
            ("term_family", Value::Text(self.term_family.clone())),
            ("timestamp", Value::Text(self.timestamp.clone())),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    fn anchor(seed: &[u8], created_at: &str) -> WebAnchor {
        WebAnchor {
            cid: Cid::compute(WEB_ANCHOR_DOMAIN, seed),
            term_family: TermFamily::Observation,
            created_at: created_at.to_string(),
            provenance: Provenance {
                source: "test".to_string(),
                input_cids: Vec::new(),
                trust_class: TrustClass::SystemPolicy,
                did_principal: None,
            },
            payload: seed.to_vec(),
        }
    }

    #[test]
    fn append_builds_hash_chain() {
        let mut store = FactStore::new();
        let first = store
            .append(&anchor(b"first", "2026-06-18T00:00:00Z"))
            .unwrap();
        let second = store
            .append(&anchor(b"second", "2026-06-18T00:00:01Z"))
            .unwrap();

        assert_ne!(first, second);
        assert_eq!(store.head, Some(second));
        assert_eq!(store.entries[0].prev_hash, None);
        assert_eq!(store.entries[1].prev_hash, Some(first));
        assert!(store.verify_chain().is_ok());
    }

    #[test]
    fn verify_rejects_tampered_entry() {
        let mut store = FactStore::new();
        store
            .append(&anchor(b"first", "2026-06-18T00:00:00Z"))
            .unwrap();
        store.entries[0].term_family = "web.action".to_string();

        assert_eq!(store.verify_chain(), Err("tape entry hash mismatch"));
    }

    #[test]
    fn verify_rejects_head_mismatch() {
        let mut store = FactStore::new();
        store
            .append(&anchor(b"first", "2026-06-18T00:00:00Z"))
            .unwrap();
        store.head = Some(Cid::compute(WEB_TAPE_ENTRY_DOMAIN, b"wrong"));

        assert_eq!(store.verify_chain(), Err("tape head mismatch"));
    }
}
