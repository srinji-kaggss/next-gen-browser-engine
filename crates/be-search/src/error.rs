#![allow(dead_code)]
#![allow(missing_docs)]

use std::fmt;

#[derive(Debug, Clone)]
pub enum FailClosed {
    NoScope,
    BadPlan,
    IndexDown,
    AuditFail,
    PartitionMissing,
    Unauthorized,
}

impl fmt::Display for FailClosed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FailClosed::NoScope => write!(f, "no scope provided"),
            FailClosed::BadPlan => write!(f, "query plan rejected"),
            FailClosed::IndexDown => write!(f, "search index unavailable"),
            FailClosed::AuditFail => write!(f, "audit log write failed"),
            FailClosed::PartitionMissing => write!(f, "scope partition missing"),
            FailClosed::Unauthorized => write!(f, "scope lacks capability"),
        }
    }
}

impl std::error::Error for FailClosed {}
