//! Content Identifier (CID) computation.
//!
//! Copied from logicalworks-/axiom/rust/src/lib.rs with modifications:
//! - Switched from blake2 to blake3 (already a workspace dependency)
//! - Added serde support for serialization
//! - Added verification method
//!
//! TODO: consolidate with braid-ir CID when Braid stabilizes.

use blake3::Hasher;
use serde::{Deserialize, Serialize};

/// A content identifier — deterministic hash of content.
///
/// CIDs are used to:
/// - Identify Braid IR programs (same source → same CID)
/// - Identify audit entries (tamper-evident)
/// - Identify PULSE frames (idempotency)
///
/// Format: `b3:hex(blake3(data))`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Cid(String);

impl Cid {
    /// Compute CID from raw bytes.
    pub fn from_bytes(data: &[u8]) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(data);
        let hash = hasher.finalize();
        Cid(format!("b3:{}", hex::encode(hash.as_bytes())))
    }

    /// Compute CID from a string.
    pub fn from_text(s: &str) -> Self {
        Self::from_bytes(s.as_bytes())
    }

    /// Get the CID as a string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Verify that data matches this CID.
    pub fn verify(&self, data: &[u8]) -> bool {
        let computed = Self::from_bytes(data);
        computed == *self
    }
}

impl std::fmt::Display for Cid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cid_deterministic() {
        let data = b"hello world";
        let cid1 = Cid::from_bytes(data);
        let cid2 = Cid::from_bytes(data);
        assert_eq!(cid1, cid2);
    }

    #[test]
    fn test_cid_different_data() {
        let cid1 = Cid::from_bytes(b"hello");
        let cid2 = Cid::from_bytes(b"world");
        assert_ne!(cid1, cid2);
    }

    #[test]
    fn test_cid_verify() {
        let data = b"test data";
        let cid = Cid::from_bytes(data);
        assert!(cid.verify(data));
        assert!(!cid.verify(b"wrong data"));
    }

    #[test]
    fn test_cid_format() {
        let cid = Cid::from_bytes(b"test");
        assert!(cid.as_str().starts_with("b3:"));
    }
}
