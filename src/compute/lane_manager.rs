//! Traceability: AXIOM_CAPABILITY_BOUNDARY, AXIOM_CLOSED_ACTIONS, AXIOM_PRIVACY_TIER.
use crate::{browser_types::*, capability::CapabilityBroker};

/// Unified compute lane for JS and Wasm under a single capability broker and tape.
pub struct ComputeLaneManager;

impl ComputeLaneManager {
    pub fn new() -> Self {
        Self
    }

    pub fn run_js(
        &self,
        cap: &crate::capability::WebCapability,
        script: &str,
    ) -> Result<ActionVerb, &'static str> {
        CapabilityBroker::new().authorize(cap, ActionVerb::ExecuteJs, "", script.len())?;
        Ok(ActionVerb::ExecuteJs)
    }

    pub fn run_wasm(
        &self,
        cap: &crate::capability::WebCapability,
        module: &[u8],
    ) -> Result<ActionVerb, &'static str> {
        CapabilityBroker::new().authorize(cap, ActionVerb::ExecuteWasm, "", module.len())?;
        Ok(ActionVerb::ExecuteWasm)
    }
}

impl Default for ComputeLaneManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::{Attenuation, WebCapability};
    use alloc::string::ToString;
    use alloc::vec;
    use alloc::vec::Vec;

    fn cap(scope: &str, verbs: Vec<ActionVerb>, max_bytes: usize) -> WebCapability {
        WebCapability {
            issuer_did: "did:system".to_string(),
            subject_did: "did:agent".to_string(),
            scope: vec![scope.to_string()],
            privacy_tier: PrivacyTier::LocalFull,
            attenuation: Attenuation {
                allowed_verbs: verbs,
                allowed_origins: Vec::new(),
                max_bytes,
                max_calls: 0,
            },
            issued_at: "2026-06-17T00:00:00Z".to_string(),
            expires_at: "2099-06-17T00:00:00Z".to_string(),
            signature: Vec::new(),
        }
    }

    #[test]
    fn js_lane_admits_only_compute_local_capability() {
        let lane = ComputeLaneManager::new();
        let allowed = cap(
            braid_vocab_web::COMPUTE_LOCAL_NAME,
            vec![ActionVerb::ExecuteJs],
            64,
        );
        assert_eq!(lane.run_js(&allowed, "1 + 1"), Ok(ActionVerb::ExecuteJs));

        let denied = cap(
            braid_vocab_web::INTERACT_NAME,
            vec![ActionVerb::ExecuteJs],
            64,
        );
        assert_eq!(
            lane.run_js(&denied, "1 + 1"),
            Err("capability scope does not grant required authority")
        );
    }

    #[test]
    fn wasm_lane_checks_verb_and_byte_budget() {
        let lane = ComputeLaneManager::new();
        let wrong_verb = cap(
            braid_vocab_web::COMPUTE_LOCAL_NAME,
            vec![ActionVerb::ExecuteJs],
            64,
        );
        assert_eq!(
            lane.run_wasm(&wrong_verb, &[0, 97, 115, 109]),
            Err("capability does not explicitly allow verb")
        );

        let too_small = cap(
            braid_vocab_web::COMPUTE_LOCAL_NAME,
            vec![ActionVerb::ExecuteWasm],
            2,
        );
        assert_eq!(
            lane.run_wasm(&too_small, &[0, 97, 115, 109]),
            Err("capability byte budget exceeded")
        );
    }
}
