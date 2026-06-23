//! # be-taint — Data flow taint tracking
//!
//! Every data flow through the browser has a security label (taint).
//! Taint is assigned at the source and propagates through all transformations.
//!
//! ## The rules
//!
//! - SECRET data can never be mixed with PAGE data
//! - PAGE data cannot become AI instructions
//! - USER data cannot escalate to SECRET
//!
//! ## Blast radius
//!
//! This crate has NO dependencies on logic-os-kernel or Braid.
//! It is self-contained. Changes here affect all subsystems that handle data.
//!
//! TODO: move to canvas-syscall when the taint model stabilizes.

use serde::{Deserialize, Serialize};

/// Security labels for data flows.
///
/// Ordering: Clean < User < Page < Ai < Secret
/// Higher taint = more sensitive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Taint {
    /// Browser-internal data. Can mix with anything.
    Clean = 0,
    /// User input. Can mix with Clean and User.
    User = 1,
    /// Page content (untrusted). Can mix with Clean only.
    Page = 2,
    /// AI-generated content. Can mix with Clean and User.
    Ai = 3,
    /// Secrets (cookies, credentials, tokens). Can mix with Clean and Secret only.
    Secret = 4,
}

/// Errors from taint checking.
#[derive(Debug)]
pub enum TaintError {
    FlowViolation { source: Taint, sink: Taint },
    MixViolation { a: Taint, b: Taint },
}

impl std::fmt::Display for TaintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaintError::FlowViolation { source, sink } => {
                write!(f, "taint violation: cannot flow from {} to {}", source, sink)
            }
            TaintError::MixViolation { a, b } => {
                write!(f, "taint mix violation: {} and {} cannot be in the same context", a, b)
            }
        }
    }
}

impl std::error::Error for TaintError {}

impl std::fmt::Display for Taint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Taint::Clean => write!(f, "Clean"),
            Taint::User => write!(f, "User"),
            Taint::Page => write!(f, "Page"),
            Taint::Ai => write!(f, "Ai"),
            Taint::Secret => write!(f, "Secret"),
        }
    }
}

impl Taint {
    /// Check if data with this taint can flow to a sink with the given taint.
    ///
    /// Rules:
    /// - Clean flows anywhere
    /// - User flows to Clean, User, Ai
    /// - Page flows to Clean, Page
    /// - Ai flows to Clean, User, Ai
    /// - Secret flows to Clean, Secret
    pub fn can_flow_to(self, sink: Taint) -> bool {
        matches!(
            (self, sink),
            (Taint::Clean, _)
                | (_, Taint::Clean)
                | (Taint::User, Taint::User)
                | (Taint::User, Taint::Ai)
                | (Taint::Page, Taint::Page)
                | (Taint::Ai, Taint::User)
                | (Taint::Ai, Taint::Ai)
                | (Taint::Secret, Taint::Secret)
        )
    }

    /// Check if two taint labels can coexist in the same context.
    ///
    /// Rules:
    /// - Clean mixes with anything
    /// - Same labels mix
    /// - User and Ai mix
    /// - Nothing else mixes
    pub fn can_mix_with(self, other: Taint) -> bool {
        match (self, other) {
            (Taint::Clean, _) | (_, Taint::Clean) => true,
            (a, b) if a == b => true,
            (Taint::User, Taint::Ai) | (Taint::Ai, Taint::User) => true,
            _ => false,
        }
    }

    /// Propagate taint through a transformation.
    ///
    /// - Copy: taint stays the same
    /// - Concat: max of the two taints
    /// - Encode: taint stays the same
    pub fn propagate(self, other: Option<Taint>) -> Taint {
        match other {
            Some(other) => std::cmp::max(self, other),
            None => self,
        }
    }
}

/// Check a data flow and return an error if it violates taint rules.
pub fn check_flow(source: Taint, sink: Taint) -> Result<(), TaintError> {
    if source.can_flow_to(sink) {
        Ok(())
    } else {
        Err(TaintError::FlowViolation { source, sink })
    }
}

/// Check if two taint labels can coexist and return an error if not.
pub fn check_mix(a: Taint, b: Taint) -> Result<(), TaintError> {
    if a.can_mix_with(b) {
        Ok(())
    } else {
        Err(TaintError::MixViolation { a, b })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_flows_anywhere() {
        assert!(Taint::Clean.can_flow_to(Taint::Secret));
        assert!(Taint::Clean.can_flow_to(Taint::Page));
    }

    #[test]
    fn test_secret_does_not_flow_to_page() {
        assert!(!Taint::Secret.can_flow_to(Taint::Page));
        assert!(!Taint::Page.can_flow_to(Taint::Secret));
    }

    #[test]
    fn test_page_does_not_become_ai_instructions() {
        assert!(!Taint::Page.can_flow_to(Taint::Ai));
    }

    #[test]
    fn test_user_does_not_escalate_to_secret() {
        assert!(!Taint::User.can_flow_to(Taint::Secret));
    }

    #[test]
    fn test_secret_mix_only_with_clean_or_secret() {
        assert!(Taint::Secret.can_mix_with(Taint::Clean));
        assert!(Taint::Secret.can_mix_with(Taint::Secret));
        assert!(!Taint::Secret.can_mix_with(Taint::Page));
        assert!(!Taint::Secret.can_mix_with(Taint::User));
    }

    #[test]
    fn test_check_flow() {
        assert!(check_flow(Taint::Secret, Taint::Clean).is_ok());
        assert!(check_flow(Taint::Secret, Taint::Page).is_err());
    }
}
