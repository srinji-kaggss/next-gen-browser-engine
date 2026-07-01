#![allow(dead_code)]
#![allow(missing_docs)]

use std::hash::Hash;

pub type OriginHash = u64;
pub type SessionId = u64;
pub type TenantId = u64;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CapSet(pub u64);

impl CapSet {
    pub fn empty() -> Self {
        CapSet(0)
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    /// Build a CapSet from raw capability bits.
    pub fn from_bits(bits: u64) -> Self {
        CapSet(bits)
    }

    /// Check whether this set includes ALL bits from `required` (AND composition).
    pub fn includes(&self, required: CapSet) -> bool {
        (self.0 & required.0) == required.0
    }

    /// Check whether a specific bit index is set.
    pub fn contains(&self, bit: u64) -> bool {
        (self.0 & (1 << bit)) != 0
    }

    /// Intersection (AND) — the only valid composition operator.
    /// union/OR is structurally denied: no `union` method exists.
    pub fn intersection(&self, other: CapSet) -> Self {
        CapSet(self.0 & other.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Scope {
    pub page_origin: OriginHash,
    pub session: SessionId,
    pub tenant: TenantId,
    pub capabilities: CapSet,
}

#[cfg(feature = "test-fixtures")]
impl Scope {
    /// Test-only constructor. Feature-gated so production builds remain
    /// unforgeable (spec §4.1).
    pub fn for_test(
        page_origin: OriginHash,
        session: SessionId,
        tenant: TenantId,
        cap_bits: u64,
    ) -> Self {
        Self {
            page_origin,
            session,
            tenant,
            capabilities: CapSet(cap_bits),
        }
    }
}

impl Scope {
    /// Create a scope from verified session/auth data. This is the canonical
    /// production constructor — called by the session/auth layer when a request
    /// arrives with a valid session token (spec §4.1).
    pub fn new(
        page_origin: OriginHash,
        session: SessionId,
        tenant: TenantId,
        capabilities: CapSet,
    ) -> Self {
        Self {
            page_origin,
            session,
            tenant,
            capabilities,
        }
    }

    pub(crate) fn construct(
        page_origin: OriginHash,
        session: SessionId,
        tenant: TenantId,
        capabilities: CapSet,
    ) -> Self {
        Self {
            page_origin,
            session,
            tenant,
            capabilities,
        }
    }

    pub fn page_origin(&self) -> OriginHash {
        self.page_origin
    }

    pub fn session(&self) -> SessionId {
        self.session
    }

    pub fn tenant(&self) -> TenantId {
        self.tenant
    }

    pub fn capabilities(&self) -> &CapSet {
        &self.capabilities
    }

    /// Check whether the scope's capabilities include the required bits (AND
    /// composition). The scope serves as both identity and capability guard
    /// (spec §6 L7).
    pub fn allows(&self, required: CapSet) -> bool {
        (self.capabilities.0 & required.0) == required.0
    }

    /// Compute a deterministic digest for the scope. Used by the query filter
    /// to build the scope guard term (spec §5 L4).
    pub fn digest(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        Hash::hash(self, &mut h);
        h.finish()
    }
}
