use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use dashmap::DashMap;
use tantivy::query::Query;

use crate::error::FailClosed;
use crate::index::{Hit, SearchIndex};
use crate::scope::Scope;

pub type PartitionKey = u64;

pub fn scope_hash(scope: &Scope) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    scope.hash(&mut h);
    h.finish()
}

pub fn partition_key(scope: &Scope) -> u64 {
    let mut h = DefaultHasher::new();
    scope.tenant.hash(&mut h);
    scope.page_origin.hash(&mut h);
    h.finish()
}

#[derive(Debug)]
pub struct PartitionStore {
    partitions: DashMap<PartitionKey, Arc<SearchIndex>>,
}

impl PartitionStore {
    pub fn new() -> Self {
        Self {
            partitions: DashMap::new(),
        }
    }

    fn get_or_create(&self, scope: &Scope) -> Arc<SearchIndex> {
        let key = partition_key(scope);
        self.partitions
            .entry(key)
            .or_insert_with(|| Arc::new(SearchIndex::create_partition(scope)))
            .clone()
    }

    pub fn search(
        &self,
        query: &(dyn Query + Sync),
        scope: &Scope,
        limit: usize,
    ) -> Result<Vec<Hit>, FailClosed> {
        let index = self.get_or_create(scope);
        let addrs = index.search(query, limit)?;
        let hash = scope_hash(scope);
        Ok(addrs
            .into_iter()
            .map(|doc_address| Hit {
                doc_address,
                scope_hash: hash,
            })
            .collect())
    }

    pub fn partition_count(&self) -> usize {
        self.partitions.len()
    }
}

impl Default for PartitionStore {
    fn default() -> Self {
        Self::new()
    }
}
