//! 13 hard axioms of the web-cognition substrate.
//! These are statements of policy, not implementation detail.
//! Every line of code below the broker must be traceable to one of these.

pub const AXIOM_BRAID_CANONICAL: &str =
    "Canonical state is a Braid-like typed term graph addressed by content id.";

pub const AXIOM_DERIVED_LENS: &str =
    "DOM, accessibility tree, pixels, OKF, and human UI are derived lenses, never canonical.";

pub const AXIOM_CLOSED_ACTIONS: &str =
    "Only the nine closed action verbs are representable in the engine boundary.";

pub const AXIOM_CAPABILITY_BOUNDARY: &str =
    "Capability tokens are signed, scoped, and attenuation-only.";

pub const AXIOM_POLICY_AUTHORITY: &str =
    "The deterministic policy broker is the sole authority for action verdicts.";

pub const AXIOM_TAPE_APPEND_ONLY: &str =
    "The causal tape is append-only, content-addressed, and hash-chained.";

pub const AXIOM_OBSERVABILITY_TYPED: &str = "Observability is typed facts, not raw browser dumps.";

pub const AXIOM_CONFINEMENT: &str =
    "Confinement makes dangerous effects unrepresentable; detection is a last resort.";

pub const AXIOM_DO178C: &str = "DO-178B/C applies to the entire codebase as a hard CI gate.";

pub const AXIOM_DETERMINISTIC_QUIESCENCE: &str =
    "Quiescence is a deterministic property of the state machine.";

pub const AXIOM_LLMS_SENSOR_ONLY: &str = "LLMs are sensors / planners, never authorities.";

pub const AXIOM_HUMAN_DEFERRAL: &str =
    "Human UI is a Phase-2+ derived lens over the canonical anchor.";

pub const AXIOM_ANTIVIRUS: &str =
    "The browser acts as world-class anti-malware: deny-first, confine, and audit.";

pub const AXIOM_AIP_WIRE_LENS: &str =
    "AIP is a first-class wire lens consumed by the Braid bridge, not a competing canonical IR.";

pub const AXIOM_PRIVACY_TIER: &str =
    "Privacy tier, sensitivity classification, and redaction precede any model exposure.";

pub const AXIOM_DID_DELEGATION: &str =
    "Every capability token is bound to a DID principal chain; unbounded delegation is unrepresentable.";

pub const AXIOM_SMALL_MODEL: &str =
    "Basic tasks expose a finite typed action set small enough for a local 2B model.";
