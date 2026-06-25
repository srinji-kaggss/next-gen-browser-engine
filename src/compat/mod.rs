//! Traceability: AXIOM_CONFINEMENT, AXIOM_CAPABILITY_BOUNDARY, AXIOM_CLOSED_ACTIONS.
use alloc::vec::Vec;

/// Developer-facing intake surface for "bring your existing app" compatibility.
///
/// This is deliberately not an engine selector. It classifies app/runtime
/// demands into bounded lanes so legacy compatibility cannot smuggle ambient
/// authority into the core substrate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeSurface {
    HtmlCssDom,
    JavaScript,
    WebAssembly,
    NativeCodec,
    JavaBytecode,
    NpapiPlugin,
    BrowserExtension,
    HostFilesystem,
    NetworkEgress,
    UnknownLegacy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatibilityMode {
    /// Supported by the brought-in browser engine core.
    NativeEngine,
    /// Supported only as policy-admitted guest compute.
    GuestCompute,
    /// Accepted only behind a compatibility adapter and sandbox.
    QuarantinedLegacy,
    /// Rejected until represented by a closed capability vocabulary.
    Reject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatibilityReason {
    WebPlatformCore,
    BoundedGuestCompute,
    LegacyJvmQuarantine,
    AmbientPluginAuthority,
    ProductShellSurface,
    ExplicitHostWriteRequired,
    MissingClosedCapability,
    UnknownLegacySurface,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompatibilityFinding {
    pub surface: RuntimeSurface,
    pub mode: CompatibilityMode,
    pub reason: CompatibilityReason,
    pub required_capability: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkloadProfile {
    pub surfaces: Vec<RuntimeSurface>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompatibilityPlan {
    pub findings: Vec<CompatibilityFinding>,
}

impl CompatibilityPlan {
    pub fn accepts_legacy_without_core_jvm(&self) -> bool {
        self.findings.iter().all(|finding| {
            finding.surface != RuntimeSurface::JavaBytecode
                || finding.mode == CompatibilityMode::QuarantinedLegacy
        })
    }

    pub fn has_rejections(&self) -> bool {
        self.findings
            .iter()
            .any(|finding| finding.mode == CompatibilityMode::Reject)
    }
}

pub struct CompatibilityPlanner;

impl CompatibilityPlanner {
    pub fn new() -> Self {
        Self
    }

    pub fn plan(&self, profile: &WorkloadProfile) -> CompatibilityPlan {
        CompatibilityPlan {
            findings: profile
                .surfaces
                .iter()
                .copied()
                .map(classify_surface)
                .collect(),
        }
    }
}

impl Default for CompatibilityPlanner {
    fn default() -> Self {
        Self::new()
    }
}

fn classify_surface(surface: RuntimeSurface) -> CompatibilityFinding {
    match surface {
        RuntimeSurface::HtmlCssDom | RuntimeSurface::NativeCodec => finding(
            surface,
            CompatibilityMode::NativeEngine,
            CompatibilityReason::WebPlatformCore,
            None,
        ),
        RuntimeSurface::JavaScript | RuntimeSurface::WebAssembly => finding(
            surface,
            CompatibilityMode::GuestCompute,
            CompatibilityReason::BoundedGuestCompute,
            Some(braid_vocab_web::COMPUTE_LOCAL_NAME),
        ),
        RuntimeSurface::JavaBytecode => finding(
            surface,
            CompatibilityMode::QuarantinedLegacy,
            CompatibilityReason::LegacyJvmQuarantine,
            Some(braid_vocab_web::COMPUTE_LOCAL_NAME),
        ),
        RuntimeSurface::HostFilesystem => finding(
            surface,
            CompatibilityMode::QuarantinedLegacy,
            CompatibilityReason::ExplicitHostWriteRequired,
            Some(braid_vocab_web::FS_WRITE_NAME),
        ),
        RuntimeSurface::NetworkEgress => finding(
            surface,
            CompatibilityMode::Reject,
            CompatibilityReason::MissingClosedCapability,
            None,
        ),
        RuntimeSurface::NpapiPlugin => finding(
            surface,
            CompatibilityMode::Reject,
            CompatibilityReason::AmbientPluginAuthority,
            None,
        ),
        RuntimeSurface::BrowserExtension => finding(
            surface,
            CompatibilityMode::Reject,
            CompatibilityReason::ProductShellSurface,
            None,
        ),
        RuntimeSurface::UnknownLegacy => finding(
            surface,
            CompatibilityMode::Reject,
            CompatibilityReason::UnknownLegacySurface,
            None,
        ),
    }
}

fn finding(
    surface: RuntimeSurface,
    mode: CompatibilityMode,
    reason: CompatibilityReason,
    required_capability: Option<&'static str>,
) -> CompatibilityFinding {
    CompatibilityFinding {
        surface,
        mode,
        reason,
        required_capability,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn java_bytecode_is_quarantined_not_core_runtime() {
        let plan = CompatibilityPlanner::new().plan(&WorkloadProfile {
            surfaces: vec![RuntimeSurface::JavaBytecode],
        });

        assert_eq!(plan.findings[0].mode, CompatibilityMode::QuarantinedLegacy);
        assert_eq!(
            plan.findings[0].reason,
            CompatibilityReason::LegacyJvmQuarantine
        );
        assert_eq!(
            plan.findings[0].required_capability,
            Some(braid_vocab_web::COMPUTE_LOCAL_NAME)
        );
        assert!(plan.accepts_legacy_without_core_jvm());
    }

    #[test]
    fn plugin_and_unknown_legacy_surfaces_fail_closed() {
        let plan = CompatibilityPlanner::new().plan(&WorkloadProfile {
            surfaces: vec![RuntimeSurface::NpapiPlugin, RuntimeSurface::UnknownLegacy],
        });

        assert!(plan.has_rejections());
        assert!(plan
            .findings
            .iter()
            .all(|finding| finding.mode == CompatibilityMode::Reject));
    }

    #[test]
    fn js_and_wasm_enter_bounded_compute_lane() {
        let plan = CompatibilityPlanner::new().plan(&WorkloadProfile {
            surfaces: vec![RuntimeSurface::JavaScript, RuntimeSurface::WebAssembly],
        });

        assert!(plan.findings.iter().all(|finding| {
            finding.mode == CompatibilityMode::GuestCompute
                && finding.required_capability == Some(braid_vocab_web::COMPUTE_LOCAL_NAME)
        }));
    }

    #[test]
    fn network_egress_is_missing_until_vocab_represents_it() {
        let plan = CompatibilityPlanner::new().plan(&WorkloadProfile {
            surfaces: vec![RuntimeSurface::NetworkEgress],
        });

        assert_eq!(plan.findings[0].mode, CompatibilityMode::Reject);
        assert_eq!(
            plan.findings[0].reason,
            CompatibilityReason::MissingClosedCapability
        );
        assert_eq!(plan.findings[0].required_capability, None);
    }
}
