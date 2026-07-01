//! # be-transpiler — JS source transpiler
//!
//! Converts raw JavaScript source into semantic terms for downstream analysis.
//!
//! ## Blast radius
//!
//! Standalone crate. Changes here affect be-api's /transpile and /load endpoints.

mod errors;
mod escalation;
mod strategies;
pub mod transpiler;

pub use errors::TranspileError;
pub use escalation::{EscalationRecord, InferenceLevel};
pub use strategies::StrategyRegistry;
pub use transpiler::{parse_js, transpile};
