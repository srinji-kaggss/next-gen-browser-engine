//! # be-lanes — Compression lanes
//!
//! Compression contexts are typed. Data from different security domains
//! is never compressed together. This prevents compression side-channel attacks.
//!
//! ## The rule
//!
//! SECRET and PAGE data can never be in the same compression context.
//!
//! ## Blast radius
//!
//! Depends on be-taint only. Changes here affect network and storage subsystems.
//!
//! TODO: move to canvas-protocol when the lane model stabilizes.

use be_taint::Taint;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Compression lane — a security-typed compression context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Lane {
    /// Safe global dictionary compression. Schema, element roles, event types.
    PublicStatic,
    /// Safe session compression. Page state, scroll position, focus.
    PublicDynamic,
    /// Private/session-isolated. Element handles, node IDs.
    PrivateReference,
    /// Private/secret. Cookies, credentials, tokens. Never compressed with other lanes.
    PrivateSecret,
    /// Attacker-controlled. Page text, attributes, URLs. Separate context.
    AttackerControlled,
    /// Bulk data. Images, media. Chunked, content-addressed.
    BulkBlob,
}

/// Errors from lane checking.
#[derive(Debug, Error)]
pub enum LaneError {
    #[error("lane violation: {lane:?} cannot contain taint {taint:?}")]
    TaintViolation { lane: Lane, taint: Taint },
    #[error("lane mix violation: {a:?} and {b:?} cannot be in the same compression context")]
    MixViolation { a: Lane, b: Lane },
}

impl Lane {
    /// Get the allowed taint labels for this lane.
    pub fn allowed_taints(&self) -> &[Taint] {
        match self {
            Lane::PublicStatic => &[Taint::Clean],
            Lane::PublicDynamic => &[Taint::Clean, Taint::User],
            Lane::PrivateReference => &[Taint::Clean, Taint::User],
            Lane::PrivateSecret => &[Taint::Secret],
            Lane::AttackerControlled => &[Taint::Page],
            Lane::BulkBlob => &[Taint::Clean, Taint::User, Taint::Page],
        }
    }

    /// Check if a taint label is allowed in this lane.
    pub fn allows_taint(&self, taint: Taint) -> bool {
        self.allowed_taints().contains(&taint)
    }

    /// Check if two lanes can coexist in the same compression context.
    ///
    /// Rules:
    /// - PrivateSecret never mixes with anything except itself
    /// - AttackerControlled never mixes with PrivateSecret
    /// - Same lane always mixes
    pub fn can_mix_with(&self, other: &Lane) -> bool {
        match (self, other) {
            (Lane::PrivateSecret, Lane::PrivateSecret) => true,
            (Lane::PrivateSecret, _) | (_, Lane::PrivateSecret) => false,
            (Lane::AttackerControlled, Lane::AttackerControlled) => true,
            (a, b) => a == b,
        }
    }

    /// Get the lane for a given taint label.
    pub fn for_taint(taint: Taint) -> Lane {
        match taint {
            Taint::Clean => Lane::PublicStatic,
            Taint::User => Lane::PublicDynamic,
            Taint::Page => Lane::AttackerControlled,
            Taint::Ai => Lane::PublicDynamic,
            Taint::Secret => Lane::PrivateSecret,
        }
    }
}

/// Check if a taint is allowed in a lane.
pub fn check_lane_taint(lane: Lane, taint: Taint) -> Result<(), LaneError> {
    if lane.allows_taint(taint) {
        Ok(())
    } else {
        Err(LaneError::TaintViolation { lane, taint })
    }
}

/// Check if two lanes can coexist.
pub fn check_lane_mix(a: Lane, b: Lane) -> Result<(), LaneError> {
    if a.can_mix_with(&b) {
        Ok(())
    } else {
        Err(LaneError::MixViolation { a, b })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_lane_only_secret() {
        assert!(Lane::PrivateSecret.allows_taint(Taint::Secret));
        assert!(!Lane::PrivateSecret.allows_taint(Taint::Page));
        assert!(!Lane::PrivateSecret.allows_taint(Taint::Clean));
    }

    #[test]
    fn test_secret_never_mixes() {
        assert!(!Lane::PrivateSecret.can_mix_with(&Lane::PublicStatic));
        assert!(!Lane::PrivateSecret.can_mix_with(&Lane::AttackerControlled));
        assert!(Lane::PrivateSecret.can_mix_with(&Lane::PrivateSecret));
    }

    #[test]
    fn test_attacker_controlled_isolation() {
        assert!(Lane::AttackerControlled.can_mix_with(&Lane::AttackerControlled));
        assert!(!Lane::AttackerControlled.can_mix_with(&Lane::PrivateSecret));
        assert!(!Lane::AttackerControlled.can_mix_with(&Lane::PublicStatic));
    }

    #[test]
    fn test_lane_for_taint() {
        assert_eq!(Lane::for_taint(Taint::Secret), Lane::PrivateSecret);
        assert_eq!(Lane::for_taint(Taint::Page), Lane::AttackerControlled);
        assert_eq!(Lane::for_taint(Taint::Clean), Lane::PublicStatic);
    }
}
