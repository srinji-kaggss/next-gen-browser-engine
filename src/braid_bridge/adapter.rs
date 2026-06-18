//! Traceability: AXIOM_BRAID_CANONICAL, AXIOM_DERIVED_LENS.
use crate::{
    braid_bridge::term::*, browser_types::*, observation::anchor::observation_from_payload,
};
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// Bidirectional adapter between canonical `WebAnchor` facts and Braid IR terms.
pub struct BraidAdapter;

impl BraidAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Project a canonical `WebAnchor` into a typed Braid term.
    ///
    /// Traceability: AXIOM_BRAID_CANONICAL, AXIOM_OBSERVABILITY_TYPED.
    pub fn to_braid(anchor: &WebAnchor) -> Result<BraidTerm, &'static str> {
        match anchor.term_family {
            TermFamily::Observation => {
                let obs = observation_from_payload(&anchor.payload)?;
                let mut facts: Vec<(String, String)> = Vec::new();
                facts.push(("kind".to_string(), obs.kind.as_str().to_string()));
                for fact in &obs.facts {
                    facts.push((fact.predicate.clone(), fact.object.clone()));
                }
                Ok(BraidTerm::Observation(WebObservation {
                    kind: obs.kind.as_str().to_string(),
                    target_cid: obs.target_cid,
                    facts,
                }))
            }
            TermFamily::Element => {
                // Payload is currently opaque for Element anchors.
                Ok(BraidTerm::Element(WebElement {
                    tag: "unknown".to_string(),
                    attrs: Vec::new(),
                    text: None,
                }))
            }
            TermFamily::Action => Ok(BraidTerm::Action(WebActionTerm {
                verb: "unknown".to_string(),
                target_cid: anchor.cid.clone(),
                parameters: Vec::new(),
            })),
            TermFamily::Capability => Ok(BraidTerm::Capability(WebCapabilityTerm {
                issuer: "unknown".to_string(),
                subject: "unknown".to_string(),
                scope: Vec::new(),
            })),
            TermFamily::Verdict => Ok(BraidTerm::Verdict(WebVerdict {
                decision: "unknown".to_string(),
                reason: "unknown".to_string(),
            })),
            _ => Err("term family not yet mapped to Braid IR"),
        }
    }

    /// Project a Braid term back into a canonical `WebAnchor`.
    ///
    /// This is the inverse of `to_braid` for round-trip verification.
    pub fn from_braid(_term: &BraidTerm) -> Result<WebAnchor, &'static str> {
        // Deferred: full inverse requires canonical serialization of every Braid variant.
        Err("from_braid deferred behind seam")
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

    #[test]
    fn observation_anchor_to_braid() {
        let obs = ObservationAnchor {
            kind: ObservationKind::Element,
            target_cid: "a1b2c3".to_string(),
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
        let term = BraidAdapter::to_braid(&anchor).unwrap();
        match term {
            BraidTerm::Observation(o) => {
                assert_eq!(o.kind, "element");
                assert_eq!(o.target_cid, "a1b2c3");
                assert!(o.facts.iter().any(|(k, _)| k == "tag"));
            }
            _ => panic!("expected Observation term"),
        }
    }

    #[test]
    fn unknown_term_family_errors() {
        let anchor = WebAnchor {
            cid: "aabbcc".to_string(),
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
