//! Traceability: AXIOM_CAPABILITY_BOUNDARY, AXIOM_CLOSED_ACTIONS.
use crate::browser_types::*;

/// Unified compute lane for JS and Wasm under a single capability broker and tape.
pub struct ComputeLaneManager;

impl ComputeLaneManager {
    pub fn new() -> Self {
        Self
    }

    pub fn run_js(
        &self,
        _cap: &crate::capability::WebCapability,
        _script: &str,
    ) -> Result<ActionVerb, &'static str> {
        todo!("execute JS in sandbox with effect typing")
    }

    pub fn run_wasm(
        &self,
        _cap: &crate::capability::WebCapability,
        _module: &[u8],
    ) -> Result<ActionVerb, &'static str> {
        todo!("execute Wasm in sandbox with effect typing")
    }
}

impl Default for ComputeLaneManager {
    fn default() -> Self {
        Self::new()
    }
}
