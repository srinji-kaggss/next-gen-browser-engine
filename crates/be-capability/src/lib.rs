//! # be-capability — Browser capabilities
//!
//! Browser-specific capabilities that gate every operation.
//! These are a superset of canvas-protocol capabilities.
//!
//! ## Blast radius
//!
//! This crate has NO dependencies on logic-os-kernel or Braid.
//! Changes here affect all subsystems.
//!
//! TODO: merge into canvas-protocol Capability enum when stable.

use serde::{Deserialize, Serialize};
use strum::EnumIter;
use thiserror::Error;

/// Browser capabilities — what operations are allowed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter)]
pub enum Capability {
    // === DOM ===
    /// Read DOM structure, attributes, text content.
    DomRead,
    /// Write DOM attributes, text content.
    DomWrite,
    /// Click an element.
    DomActionClick,
    /// Submit a form.
    DomActionSubmit,
    /// Focus an element.
    DomActionFocus,
    /// Type into an input.
    DomActionType,

    // === Network ===
    /// Make outbound network requests.
    NetworkEgress,
    /// Read network responses.
    NetworkRead,

    // === Storage ===
    /// Read cookies, localStorage, sessionStorage.
    StorageRead,
    /// Write cookies, localStorage, sessionStorage.
    StorageWrite,

    // === Media ===
    /// Play audio/video.
    MediaPlay,
    /// Capture camera/microphone.
    MediaCapture,

    // === Navigation ===
    /// Navigate to a URL.
    NavigationNavigate,
    /// Go back/forward.
    NavigationHistory,

    // === Code ===
    /// Execute dynamic code (eval, Function).
    CodeDynamic,
    /// Execute Braid IR.
    CodeBraid,

    // === AI ===
    /// Query affordances.
    AiAffordances,
    /// Take actions via PULSE frames.
    AiActions,
    /// Read page content.
    AiRead,
}

/// Errors from capability checking.
#[derive(Debug, Error)]
pub enum CapabilityError {
    #[error("capability denied: {required:?} not in granted set")]
    Denied { required: Capability },
    #[error("capability escalation: cannot widen {original:?} to {wider:?}")]
    Escalation {
        original: Capability,
        wider: Capability,
    },
}

/// The privacy level — controls what capabilities are available.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PrivacyLevel {
    /// No AI access. Human-only mode.
    Off,
    /// Read-only. AI can read page content and affordances.
    Low,
    /// Read + safe actions. AI can fill forms, click elements.
    Medium,
    /// Read + write + actions. AI can submit forms, navigate.
    High,
    /// Everything. AI can execute dynamic code.
    Full,
}

impl PrivacyLevel {
    /// Get the capabilities available at this privacy level.
    pub fn capabilities(&self) -> Vec<Capability> {
        match self {
            PrivacyLevel::Off => vec![],
            PrivacyLevel::Low => vec![
                Capability::DomRead,
                Capability::AiAffordances,
                Capability::AiRead,
            ],
            PrivacyLevel::Medium => vec![
                Capability::DomRead,
                Capability::DomWrite,
                Capability::DomActionClick,
                Capability::DomActionFocus,
                Capability::DomActionType,
                Capability::NetworkRead,
                Capability::StorageRead,
                Capability::AiAffordances,
                Capability::AiActions,
                Capability::AiRead,
            ],
            PrivacyLevel::High => vec![
                Capability::DomRead,
                Capability::DomWrite,
                Capability::DomActionClick,
                Capability::DomActionSubmit,
                Capability::DomActionFocus,
                Capability::DomActionType,
                Capability::NetworkEgress,
                Capability::NetworkRead,
                Capability::StorageRead,
                Capability::StorageWrite,
                Capability::NavigationNavigate,
                Capability::NavigationHistory,
                Capability::MediaPlay,
                Capability::AiAffordances,
                Capability::AiActions,
                Capability::AiRead,
            ],
            PrivacyLevel::Full => vec![
                Capability::DomRead,
                Capability::DomWrite,
                Capability::DomActionClick,
                Capability::DomActionSubmit,
                Capability::DomActionFocus,
                Capability::DomActionType,
                Capability::NetworkEgress,
                Capability::NetworkRead,
                Capability::StorageRead,
                Capability::StorageWrite,
                Capability::NavigationNavigate,
                Capability::NavigationHistory,
                Capability::MediaPlay,
                Capability::MediaCapture,
                Capability::CodeDynamic,
                Capability::CodeBraid,
                Capability::AiAffordances,
                Capability::AiActions,
                Capability::AiRead,
            ],
        }
    }

    /// Check if a capability is available at this privacy level.
    pub fn has(&self, cap: Capability) -> bool {
        self.capabilities().contains(&cap)
    }
}

/// Check if a capability is granted.
pub fn check_capability(
    granted: &[Capability],
    required: Capability,
) -> Result<(), CapabilityError> {
    if granted.contains(&required) {
        Ok(())
    } else {
        Err(CapabilityError::Denied { required })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_off_no_capabilities() {
        assert!(PrivacyLevel::Off.capabilities().is_empty());
    }

    #[test]
    fn test_low_read_only() {
        assert!(PrivacyLevel::Low.has(Capability::DomRead));
        assert!(PrivacyLevel::Low.has(Capability::AiRead));
        assert!(!PrivacyLevel::Low.has(Capability::DomWrite));
        assert!(!PrivacyLevel::Low.has(Capability::AiActions));
    }

    #[test]
    fn test_medium_safe_actions() {
        assert!(PrivacyLevel::Medium.has(Capability::DomActionClick));
        assert!(PrivacyLevel::Medium.has(Capability::AiActions));
        assert!(!PrivacyLevel::Medium.has(Capability::DomActionSubmit));
        assert!(!PrivacyLevel::Medium.has(Capability::CodeDynamic));
    }

    #[test]
    fn test_high_navigation() {
        assert!(PrivacyLevel::High.has(Capability::NavigationNavigate));
        assert!(PrivacyLevel::High.has(Capability::DomActionSubmit));
        assert!(!PrivacyLevel::High.has(Capability::CodeDynamic));
    }

    #[test]
    fn test_full_everything() {
        assert!(PrivacyLevel::Full.has(Capability::CodeDynamic));
        assert!(PrivacyLevel::Full.has(Capability::MediaCapture));
    }

    #[test]
    fn test_check_capability() {
        let granted = PrivacyLevel::Medium.capabilities();
        assert!(check_capability(&granted, Capability::DomRead).is_ok());
        assert!(check_capability(&granted, Capability::CodeDynamic).is_err());
    }
}
