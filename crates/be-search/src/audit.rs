//! Append-only audit trail (spec §9, P9).
//!
//! Every query is journaled BEFORE execution (audit-before-effect, §2.4).
//! The trail records the sanitized `QueryPlan` — never raw user input.
//! An audit write failure denies the search (fail-closed, §2.3).

use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::candidate::Candidate;
use crate::error::FailClosed;
use crate::query::QueryPlan;
use crate::scope::Scope;

/// Terminal outcome of a search, plus `Submitted` for the pre-execution
/// intent record (journal-before-effect). §9 lists the three terminal states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Submitted,
    Success,
    Denied,
    Fallback,
}

/// Monotonic trusted timestamp (unix seconds). Never derived from client input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TrustedTime(pub u64);

impl TrustedTime {
    pub fn now() -> Self {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| TrustedTime(d.as_secs()))
            .unwrap_or(TrustedTime(0))
    }
}

/// One record in the audit trail (spec §9). `candidates` holds node ids (u64),
/// matching `Candidate::node_id`.
#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub scope: Scope,
    pub plan: QueryPlan,
    pub reason: String,
    pub candidates: Vec<u64>,
    pub timestamp: TrustedTime,
    pub outcome: Outcome,
}

/// Append-only audit log. Writes are fail-closed: a poisoned lock or any
/// write failure surfaces as `Err(FailClosed::AuditFail)`, which the
/// orchestrator MUST propagate as a denied search.
#[derive(Debug, Default)]
pub struct AuditLog {
    entries: Mutex<Vec<AuditEntry>>,
}

impl AuditLog {
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(Vec::new()),
        }
    }

    /// Journal the query BEFORE execution (L5). Records scope + sanitized plan
    /// + reason, with empty candidates and `Outcome::Submitted`.
    pub fn log_query(
        &self,
        scope: &Scope,
        plan: &QueryPlan,
        reason: &str,
    ) -> Result<(), FailClosed> {
        let entry = AuditEntry {
            scope: scope.clone(),
            plan: plan.clone(),
            reason: reason.to_string(),
            candidates: Vec::new(),
            timestamp: TrustedTime::now(),
            outcome: Outcome::Submitted,
        };
        match self.entries.lock() {
            Ok(mut trail) => {
                trail.push(entry);
                Ok(())
            }
            Err(_) => Err(FailClosed::AuditFail),
        }
    }

    /// Completion receipt (L9). Records the resolved candidate node ids under
    /// `Outcome::Success`. The plan is omitted here — the intent record already
    /// carries it; this entry is the effect-side receipt.
    pub fn log_results(&self, scope: &Scope, candidates: &[Candidate]) -> Result<(), FailClosed> {
        let node_ids = candidates.iter().map(|c| c.node_id).collect();
        let entry = AuditEntry {
            scope: scope.clone(),
            plan: QueryPlan::Empty,
            reason: String::new(),
            candidates: node_ids,
            timestamp: TrustedTime::now(),
            outcome: Outcome::Success,
        };
        match self.entries.lock() {
            Ok(mut trail) => {
                trail.push(entry);
                Ok(())
            }
            Err(_) => Err(FailClosed::AuditFail),
        }
    }

    /// Record a non-success terminal state (fallback / explicit deny).
    pub fn log_outcome(&self, scope: &Scope, outcome: Outcome) -> Result<(), FailClosed> {
        let entry = AuditEntry {
            scope: scope.clone(),
            plan: QueryPlan::Empty,
            reason: String::new(),
            candidates: Vec::new(),
            timestamp: TrustedTime::now(),
            outcome,
        };
        match self.entries.lock() {
            Ok(mut trail) => {
                trail.push(entry);
                Ok(())
            }
            Err(_) => Err(FailClosed::AuditFail),
        }
    }

    pub fn len(&self) -> usize {
        self.entries.lock().map(|t| t.len()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Snapshot of the trail, filtered to a caller's own scope. Scope filtering
    /// is enforced here as defense-in-depth (§9: caller reads only their trail).
    pub fn snapshot(&self, scope: &Scope) -> Vec<AuditEntry> {
        match self.entries.lock() {
            Ok(trail) => trail
                .iter()
                .filter(|e| &e.scope == scope)
                .cloned()
                .collect(),
            Err(_) => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scope::Scope;

    /// claim = demonstration: a poisoned audit lock MUST surface as
    /// `FailClosed::AuditFail` (fail-closed), never silently continue.
    #[test]
    fn audit_write_failure_is_fail_closed() {
        let log = AuditLog::new();
        let scope = Scope::construct(1, 1, 1, Default::default());

        // Deliberately poison the inner mutex, then recover.
        let guard = log.entries.lock().unwrap();
        let blew = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _hold = guard;
            panic!("intentional poison");
        }));
        assert!(blew.is_err());

        // Any subsequent write must now fail closed.
        let res = log.log_query(&scope, &QueryPlan::Empty, "t");
        assert!(matches!(res, Err(FailClosed::AuditFail)));
    }

    #[test]
    fn audit_records_query_then_results() {
        let log = AuditLog::new();
        let scope = Scope::construct(2, 2, 2, Default::default());
        log.log_query(&scope, &QueryPlan::Empty, "why").unwrap();
        assert_eq!(log.len(), 1);
        log.log_results(&scope, &[]).unwrap();
        assert_eq!(log.len(), 2);
        let snap = log.snapshot(&scope);
        assert_eq!(snap.len(), 2);
    }
}
