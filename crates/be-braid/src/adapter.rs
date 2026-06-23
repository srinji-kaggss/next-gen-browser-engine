//! # BraidAdapter — abstract over Braid IR operations
//!
//! Every Braid dependency is behind this trait. When Braid changes,
//! only the adapter implementation changes.
//!
//! The default implementation uses real Braid crates directly.
//! A mock implementation is available for testing.

use braid_capability::Capability;
use braid_ir::capsule::Capsule;
use braid_ir::cid::CAPSULE_DOMAIN;
use braid_ir::term::TermRegistry;
use braid_ir::Cid;
use braid_verify::Verdict;
use thiserror::Error;

/// Errors from Braid operations.
#[derive(Debug, Error)]
pub enum BraidError {
    #[error("verification failed at stage {stage:?}: {reason}")]
    VerificationFailed {
        stage: braid_verify::Stage,
        reason: String,
    },
    #[error("encoding error: {0}")]
    Encoding(String),
    #[error("registry error: {0}")]
    Registry(String),
}

/// Abstract interface to Braid IR operations.
///
/// The browser engine never touches `braid-ir` types directly —
/// everything goes through this adapter. When Braid changes,
/// only `DefaultBraidAdapter` changes.
pub trait BraidAdapter {
    /// Get the web.* term registry.
    fn registry(&self) -> &TermRegistry;

    /// Verify a capsule against the registry and ambient capabilities.
    fn verify(&self, capsule_bytes: &[u8], ambient: &[Capability]) -> Result<Cid, BraidError>;

    /// Encode a capsule to canonical bytes.
    fn encode_capsule(&self, capsule: &Capsule) -> Result<Vec<u8>, BraidError>;

    /// Compute the CID of bytes.
    fn cid(&self, data: &[u8]) -> Cid;
}

/// Default adapter — uses real Braid crates directly.
pub struct DefaultBraidAdapter {
    registry: TermRegistry,
}

impl DefaultBraidAdapter {
    /// Create a new adapter with the web.* vocabulary.
    pub fn new() -> Self {
        Self {
            registry: crate::vocab::registry_v0(),
        }
    }

    /// Create with a custom registry.
    pub fn with_registry(registry: TermRegistry) -> Self {
        Self { registry }
    }
}

impl Default for DefaultBraidAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl BraidAdapter for DefaultBraidAdapter {
    fn registry(&self) -> &TermRegistry {
        &self.registry
    }

    fn verify(&self, capsule_bytes: &[u8], ambient: &[Capability]) -> Result<Cid, BraidError> {
        match braid_verify::verify(capsule_bytes, &self.registry, ambient) {
            Verdict::Admit { capsule_cid } => Ok(capsule_cid),
            Verdict::Reject { stage, reason } => Err(BraidError::VerificationFailed { stage, reason }),
        }
    }

    fn encode_capsule(&self, capsule: &Capsule) -> Result<Vec<u8>, BraidError> {
        let value = capsule.to_canon();
        Ok(braid_ir::canon::encode(&value))
    }

    fn cid(&self, data: &[u8]) -> Cid {
        Cid::compute(CAPSULE_DOMAIN, data)
    }
}

/// Mock adapter for testing — always admits, uses a minimal registry.
#[cfg(test)]
pub struct MockBraidAdapter {
    registry: TermRegistry,
}

#[cfg(test)]
impl MockBraidAdapter {
    pub fn new() -> Self {
        Self {
            registry: crate::vocab::registry_v0(),
        }
    }
}

#[cfg(test)]
impl BraidAdapter for MockBraidAdapter {
    fn registry(&self) -> &TermRegistry {
        &self.registry
    }

    fn verify(&self, _capsule_bytes: &[u8], _ambient: &[Capability]) -> Result<Cid, BraidError> {
        // Always admit in tests
        Ok(Cid::compute(CAPSULE_DOMAIN, b"mock"))
    }

    fn encode_capsule(&self, capsule: &Capsule) -> Result<Vec<u8>, BraidError> {
        let value = capsule.to_canon();
        Ok(braid_ir::canon::encode(&value))
    }

    fn cid(&self, data: &[u8]) -> Cid {
        Cid::compute(CAPSULE_DOMAIN, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_adapter_has_web_registry() {
        let adapter = DefaultBraidAdapter::new();
        assert!(adapter.registry().get("web.click").is_some());
        assert!(adapter.registry().get("web.navigate").is_some());
    }

    #[test]
    fn mock_adapter_always_admits() {
        let adapter = MockBraidAdapter::new();
        let result = adapter.verify(b"fake capsule bytes", &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn cid_is_deterministic() {
        let adapter = DefaultBraidAdapter::new();
        let cid1 = adapter.cid(b"hello");
        let cid2 = adapter.cid(b"hello");
        assert_eq!(cid1, cid2);
    }
}
