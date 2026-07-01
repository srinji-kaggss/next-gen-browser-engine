//! # be-braid — Braid adapter and web.* vocabulary
//!
//! Bridges the Braid IR substrate into the browser engine.
//! Houses the `web.*` vocabulary — the browser's capability space.
//!
//! ## Blast radius
//!
//! Depends on braid-ir, braid-capability, braid-verify (Braid substrate)
//! and be-capability (browser's privacy levels).
//!
//! Every Braid dependency is behind the `BraidAdapter` trait.
//! When Braid changes, only this crate changes.

pub mod adapter;
pub mod bridge;
pub mod vocab;

pub use adapter::{BraidAdapter, BraidError, DefaultBraidAdapter};
pub use bridge::browser_caps_to_braid;
pub use vocab::registry_v0;
