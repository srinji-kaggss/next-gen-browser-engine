//! Traceability: AXIOM_BRAID_CANONICAL, AXIOM_DERIVED_LENS.
use crate::{browser_types::*, observation::anchor::observation_from_payload};
use alloc::string::ToString;
use alloc::vec::Vec;
use braid_ir::{canon, Value};

/// Adapter between canonical `WebAnchor` facts and Braid IR values.
pub struct BraidAdapter;

impl BraidAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Project a canonical `WebAnchor` into the Braid value universe.
    ///
    /// Traceability: AXIOM_BRAID_CANONICAL, AXIOM_OBSERVABILITY_TYPED.
    pub fn to_braid(anchor: &WebAnchor) -> Result<Value, &'static str> {
        match anchor.term_family {
            TermFamily::Observation => {
                let obs = observation_from_payload(&anchor.payload)?;
                Ok(obs.to_value())
            }
            _ => Err("term family not yet projected to Braid Value"),
        }
    }

    /// Project a Braid value back into a canonical `WebAnchor`.
    ///
    /// This is the inverse of `to_braid` for round-trip verification.
    pub fn from_braid(value: &Value) -> Result<WebAnchor, &'static str> {
        let payload = canon::encode(value);
        let obs = observation_from_payload(&payload)?;
        Ok(obs.to_anchor(Provenance {
            source: "braid-ir".to_string(),
            input_cids: Vec::new(),
            trust_class: obs.trust_class,
            did_principal: None,
        }))
    }
}

impl Default for BraidAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observation::anchor::{Fact, ObservationAnchor, ObservationKind};
    use alloc::string::ToString;
    use alloc::vec;
    use alloc::vec::Vec;

    #[test]
    fn observation_anchor_to_braid() {
        let obs = ObservationAnchor {
            kind: ObservationKind::Element,
            target_cid: Cid::compute(WEB_ELEMENT_DOMAIN, b"a1b2c3"),
            observed_at: "2026-06-18T00:00:00Z".to_string(),
            facts: vec![Fact {
                predicate: "tag".to_string(),
                object: "a".to_string(),
                sensitivity: None,
            }],
            sensitivity: None,
            privacy_tier: PrivacyTier::LocalFull,
            trust_class: TrustClass::UntrustedContent,
            raw_source: None,
        };
        let anchor = obs.to_anchor(Provenance {
            source: "mac-eye".to_string(),
            input_cids: Vec::new(),
            trust_class: TrustClass::UntrustedContent,
            did_principal: None,
        });
        let value = BraidAdapter::to_braid(&anchor).unwrap();
        assert_eq!(value.get("kind"), Some(&Value::Text("element".to_string())));
        assert_eq!(
            value.get("target_cid"),
            Some(&Value::Bytes(
                Cid::compute(WEB_ELEMENT_DOMAIN, b"a1b2c3").0.to_vec()
            ))
        );
        let facts = match value.get("facts") {
            Some(Value::List(facts)) => facts,
            _ => panic!("expected facts list"),
        };
        assert!(facts.iter().any(|fact| {
            fact.get("predicate") == Some(&Value::Text("tag".to_string()))
                && fact.get("object") == Some(&Value::Text("a".to_string()))
        }));
    }

    #[test]
    fn observation_value_round_trips_back_to_anchor() {
        let obs = ObservationAnchor {
            kind: ObservationKind::Element,
            target_cid: Cid::compute(WEB_ELEMENT_DOMAIN, b"a1b2c3"),
            observed_at: "2026-06-18T00:00:00Z".to_string(),
            facts: vec![Fact {
                predicate: "tag".to_string(),
                object: "a".to_string(),
                sensitivity: None,
            }],
            sensitivity: None,
            privacy_tier: PrivacyTier::LocalFull,
            trust_class: TrustClass::UntrustedContent,
            raw_source: None,
        };
        let anchor = obs.to_anchor(Provenance {
            source: "mac-eye".to_string(),
            input_cids: Vec::new(),
            trust_class: TrustClass::UntrustedContent,
            did_principal: None,
        });
        let value = BraidAdapter::to_braid(&anchor).unwrap();
        let recovered = BraidAdapter::from_braid(&value).unwrap();

        assert_eq!(recovered.cid, anchor.cid);
        assert_eq!(recovered.payload, anchor.payload);
        assert_eq!(recovered.term_family, TermFamily::Observation);
    }

    #[test]
    fn unknown_term_family_errors() {
        let anchor = WebAnchor {
            cid: Cid::compute(WEB_ANCHOR_DOMAIN, b"aabbcc"),
            term_family: TermFamily::Transition,
            created_at: "2026-06-18T00:00:00Z".to_string(),
            provenance: Provenance {
                source: "test".to_string(),
                input_cids: Vec::new(),
                trust_class: TrustClass::SystemPolicy,
                did_principal: None,
            },
            payload: Vec::new(),
        };
        assert!(BraidAdapter::to_braid(&anchor).is_err());
    }
}
