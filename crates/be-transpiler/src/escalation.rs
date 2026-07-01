// Escalation records for capability inference
// When the transpiler cannot determine capabilities from static analysis,
// it creates an EscalationRecord instead of guessing or leaving a TODO.

use be_capability::Capability;

/// The confidence level of capability inference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InferenceLevel {
    /// Fully deterministic — capability inferred from known API pattern.
    Symbolic,
    /// Partially deterministic — capability inferred from heuristics.
    Fuzzy,
    /// Cannot determine — needs human review or dynamic analysis.
    Blackbox,
}

/// An escalation record created when capability inference is uncertain.
#[derive(Debug, Clone)]
pub struct EscalationRecord {
    /// Why this escalation exists.
    pub reason: String,
    /// Confidence level of the inference.
    pub level: InferenceLevel,
    /// What blocks deterministic resolution.
    pub blocked_by: Vec<String>,
    /// Evidence paths (AST nodes, source locations) supporting the inference.
    pub evidence_paths: Vec<String>,
    /// Capabilities that might be required (conservative over-approximation).
    pub possible_capabilities: Vec<Capability>,
    /// Allowed outputs from this escalation.
    pub allowed_outputs: Vec<String>,
}

impl EscalationRecord {
    /// Create a new escalation record.
    pub fn new(reason: &str, level: InferenceLevel) -> Self {
        Self {
            reason: reason.to_string(),
            level,
            blocked_by: vec![],
            evidence_paths: vec![],
            possible_capabilities: vec![],
            allowed_outputs: vec![],
        }
    }

    /// Add a blocker.
    pub fn blocked_by(mut self, blocker: &str) -> Self {
        self.blocked_by.push(blocker.to_string());
        self
    }

    /// Add an evidence path.
    pub fn evidence(mut self, path: &str) -> Self {
        self.evidence_paths.push(path.to_string());
        self
    }

    /// Add a possible capability.
    pub fn capability(mut self, cap: Capability) -> Self {
        self.possible_capabilities.push(cap);
        self
    }

    /// Add an allowed output.
    pub fn allowed_output(mut self, output: &str) -> Self {
        self.allowed_outputs.push(output.to_string());
        self
    }
}
