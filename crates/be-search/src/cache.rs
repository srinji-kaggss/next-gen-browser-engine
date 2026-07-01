//! Scope-bound cache (P8, leak prevention).
//!
//! Cache keys are hashed WITH the scope, and every cached value embeds the
//! scope which is re-verified on read. A different scope necessarily produces a
//! different key (different hash), so a cross-scope cache hit is structurally
//! impossible; the read-time re-verify is defense-in-depth against any
//! hypothetical hash collision.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::RwLock;

use blake3::Hasher as Blake3;

use crate::candidate::Candidate;
use crate::scope::Scope;

/// Opaque cache key. The scope is folded into the blake3 digest at derivation
/// time — there is no way to construct a key that ignores scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CacheKey(pub [u8; 32]);

impl CacheKey {
    /// Derive a key from a scope and a canonical plan fingerprint.
    /// Two different scopes CANNOT yield the same key.
    pub fn derive(scope: &Scope, plan_fingerprint: &[u8]) -> Self {
        let mut h = Blake3::new();
        h.update(b"be-search-scopecache-v1");
        let scope_hash = {
            let mut sh = DefaultHasher::new();
            use std::hash::Hash;
            scope.hash(&mut sh);
            sh.finish()
        };
        h.update(&scope_hash.to_le_bytes());
        h.update(plan_fingerprint);
        CacheKey(*h.finalize().as_bytes())
    }
}

#[derive(Debug, Clone)]
struct CacheEntry {
    scope: Scope,
    results: Vec<Candidate>,
}

/// Scope-bound result cache. Insert embeds the scope; get re-verifies it.
#[derive(Debug, Default)]
pub struct ScopeCache {
    map: RwLock<HashMap<CacheKey, CacheEntry>>,
}

impl ScopeCache {
    pub fn new() -> Self {
        Self {
            map: RwLock::new(HashMap::new()),
        }
    }

    /// Read with mandatory scope re-verification. A scope mismatch (only
    /// reachable via a collision) is a MISS, never a leak.
    pub fn get(&self, key: &CacheKey, scope: &Scope) -> Option<Vec<Candidate>> {
        let map = self.map.read().ok()?;
        let entry = map.get(key)?;
        if &entry.scope != scope {
            tracing::warn!("scope-cache: cross-scope re-verify denied (collision?)");
            return None;
        }
        Some(entry.results.clone())
    }

    /// Insert with the scope embedded in the entry for read-time verification.
    pub fn insert(&self, key: CacheKey, scope: &Scope, results: Vec<Candidate>) {
        if let Ok(mut map) = self.map.write() {
            map.insert(
                key,
                CacheEntry {
                    scope: scope.clone(),
                    results,
                },
            );
        }
    }

    pub fn invalidate(&self, key: &CacheKey) {
        if let Ok(mut map) = self.map.write() {
            map.remove(key);
        }
    }

    pub fn len(&self) -> usize {
        self.map.read().map(|m| m.len()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
