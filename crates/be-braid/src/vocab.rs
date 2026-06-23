//! # web.* vocabulary — the browser's Braid capability space
//!
//! Defines the term registry and capability names for browser operations.
//! Modeled after `braid-vocab-cms`. Each capability is a dotted string
//! token (`web.dom.read`, `web.navigate`, etc.) — foreign to the kernel's
//! `signal.*` and JS's `js.*` spaces.
//!
//! The browser's `web.*` vocabulary is owned by this crate, not by Braid.

use braid_capability::Capability;
use braid_ir::term::{EffectClass, Exposure, TermRegistry, TermSpec, TypeTag};

/// Vocabulary version — bump when terms or capabilities change.
pub const VOCAB_VERSION: u32 = 1;

// ── capability constants (the browser's dotted names) ──

pub const WEB_DOM_READ_NAME: &str = "web.dom.read";
pub const WEB_DOM_WRITE_NAME: &str = "web.dom.write";
pub const WEB_DOM_CLICK_NAME: &str = "web.dom.click";
pub const WEB_DOM_SUBMIT_NAME: &str = "web.dom.submit";
pub const WEB_DOM_FOCUS_NAME: &str = "web.dom.focus";
pub const WEB_DOM_TYPE_NAME: &str = "web.dom.type";
pub const WEB_OBSERVE_NAME: &str = "web.observe";
pub const WEB_NAVIGATE_NAME: &str = "web.navigate";
pub const WEB_EGRESS_NAME: &str = "web.egress";
pub const WEB_STORAGE_READ_NAME: &str = "web.storage.read";
pub const WEB_STORAGE_WRITE_NAME: &str = "web.storage.write";
pub const WEB_AI_READ_NAME: &str = "web.ai.read";
pub const WEB_AI_ACT_NAME: &str = "web.ai.act";

/// Wrap a `&'static str` dotted name into a `Capability`.
pub fn wrap_cap(name: &'static str) -> Capability {
    Capability::new(name)
}

/// Macro for capability construction at use sites.
#[macro_export]
macro_rules! web_cap {
    ($name:expr) => {
        $crate::vocab::wrap_cap($name)
    };
}

/// `TypeTag::Opaque("web.element", [])` — a DOM element reference.
pub fn element() -> TypeTag {
    TypeTag::Opaque("web.element".into(), Vec::new())
}

/// `TypeTag::Opaque("web.affordance", [])` — an AI-consumable affordance.
pub fn affordance() -> TypeTag {
    TypeTag::Opaque("web.affordance".into(), Vec::new())
}

/// `TypeTag::Opaque("web.page", [])` — a page reference.
pub fn page() -> TypeTag {
    TypeTag::Opaque("web.page".into(), Vec::new())
}

/// Table-row constructor for TermSpec.
#[allow(clippy::too_many_arguments)]
fn t(
    id: &str,
    inputs: Vec<TypeTag>,
    output: TypeTag,
    capability: Option<Capability>,
    effect: EffectClass,
    source_exposure: Exposure,
    egress_ceiling: Option<Exposure>,
    cost: u64,
) -> TermSpec {
    TermSpec {
        id: id.into(),
        inputs,
        output,
        capability,
        effect,
        source_exposure,
        egress_ceiling,
        cost,
    }
}

/// Build the v0 web.* registry.
pub fn registry_v0() -> TermRegistry {
    use EffectClass::*;
    use Exposure::*;
    use TypeTag::*;

    let specs = vec![
        // ── DOM reads (pure, public) ──
        t(
            "web.read_dom",
            vec![page()],
            element(),
            Some(web_cap!(WEB_DOM_READ_NAME)),
            Read,
            Public,
            None,
            1,
        ),
        t(
            "web.read_text",
            vec![element()],
            Text,
            Some(web_cap!(WEB_DOM_READ_NAME)),
            Read,
            Public,
            None,
            1,
        ),
        t(
            "web.read_attr",
            vec![element(), Text],
            Text,
            Some(web_cap!(WEB_DOM_READ_NAME)),
            Read,
            Public,
            None,
            1,
        ),
        // ── DOM writes (reversible) ──
        t(
            "web.set_attr",
            vec![element(), Text, Text],
            element(),
            Some(web_cap!(WEB_DOM_WRITE_NAME)),
            ReversibleWrite,
            Internal,
            None,
            3,
        ),
        t(
            "web.set_text",
            vec![element(), Text],
            element(),
            Some(web_cap!(WEB_DOM_WRITE_NAME)),
            ReversibleWrite,
            Internal,
            None,
            3,
        ),
        // ── DOM actions ──
        t(
            "web.click",
            vec![element()],
            element(),
            Some(web_cap!(WEB_DOM_CLICK_NAME)),
            ReversibleWrite,
            Internal,
            None,
            2,
        ),
        t(
            "web.submit",
            vec![element()],
            element(),
            Some(web_cap!(WEB_DOM_SUBMIT_NAME)),
            Irreversible,
            Internal,
            Some(Internal),
            5,
        ),
        t(
            "web.focus",
            vec![element()],
            element(),
            Some(web_cap!(WEB_DOM_FOCUS_NAME)),
            ReversibleWrite,
            Internal,
            None,
            2,
        ),
        t(
            "web.type",
            vec![element(), Text],
            element(),
            Some(web_cap!(WEB_DOM_TYPE_NAME)),
            ReversibleWrite,
            Internal,
            None,
            3,
        ),
        // ── Observation (AI surface) ──
        t(
            "web.query_affordances",
            vec![page()],
            List(Box::new(affordance())),
            Some(web_cap!(WEB_OBSERVE_NAME)),
            Read,
            Internal,
            None,
            5,
        ),
        t(
            "web.observe_mutation",
            vec![page()],
            Cid,
            Some(web_cap!(WEB_OBSERVE_NAME)),
            Read,
            Internal,
            None,
            5,
        ),
        // ── Navigation ──
        t(
            "web.navigate",
            vec![Text],
            page(),
            Some(web_cap!(WEB_NAVIGATE_NAME)),
            Irreversible,
            Internal,
            Some(Internal),
            10,
        ),
        // ── AI actions ──
        t(
            "web.ai_read",
            vec![page()],
            Text,
            Some(web_cap!(WEB_AI_READ_NAME)),
            Read,
            Internal,
            None,
            3,
        ),
        t(
            "web.ai_act",
            vec![affordance()],
            element(),
            Some(web_cap!(WEB_AI_ACT_NAME)),
            ReversibleWrite,
            Internal,
            None,
            5,
        ),
        // ── Egress (the network door) ──
        t(
            "web.fetch",
            vec![Text],
            Bytes,
            Some(web_cap!(WEB_EGRESS_NAME)),
            Egress,
            Internal,
            Some(Internal),
            15,
        ),
    ];

    let mut reg = TermRegistry::new(VOCAB_VERSION);
    for spec in specs {
        reg.insert(spec)
            .expect("registry_v0 specs are statically valid");
    }
    reg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_v0_builds() {
        let r = registry_v0();
        assert_eq!(r.len(), 15);
        assert!(r.get("web.click").is_some());
        assert!(r.get("web.navigate").is_some());
        assert!(r.get("web.ai_read").is_some());
        assert!(r.get("kernel.signal").is_none());
    }

    #[test]
    fn capability_names_are_dotted() {
        assert_eq!(WEB_DOM_READ_NAME, "web.dom.read");
        assert_eq!(WEB_NAVIGATE_NAME, "web.navigate");
        assert_eq!(WEB_EGRESS_NAME, "web.egress");
        assert_eq!(WEB_AI_READ_NAME, "web.ai.read");
    }

    #[test]
    fn registry_round_trips_canonically() {
        let r = registry_v0();
        let bytes = braid_ir::canon::encode(&r.to_canon());
        let v = braid_ir::decode_strict(&bytes).expect("canonical");
        let r2 = TermRegistry::from_canon(&v).expect("decodes");
        assert_eq!(r, r2);
        assert_eq!(r.cid(), r2.cid());
    }

    #[test]
    fn web_caps_are_foreign_to_kernel() {
        // web.* and signal.* should never collide
        assert_ne!(WEB_DOM_READ_NAME, "signal.emit");
        assert_ne!(WEB_EGRESS_NAME, "compute.remote");
    }
}
