//! Traceability: AXIOM_DERIVED_LENS, AXIOM_HUMAN_DEFERRAL.
use crate::{
    browser_types::Cid,
    observation::anchor::{observation_from_payload, ObservationKind},
};
use alloc::vec::Vec;

/// Bidirectional mapping between screen pixels and canonical CIDs.
pub struct PixelAnchor {
    lenses: Vec<LayoutLens>,
}

impl PixelAnchor {
    pub fn new(lenses: Vec<LayoutLens>) -> Self {
        Self { lenses }
    }

    /// Build a layout index from observation anchors that carry `bounds`
    /// facts in `x,y,width,height` form.
    pub fn from_observations(anchors: &[crate::browser_types::WebAnchor]) -> Self {
        let mut lenses = Vec::new();
        for anchor in anchors {
            let Ok(obs) = observation_from_payload(&anchor.payload) else {
                continue;
            };
            if obs.kind != ObservationKind::Element {
                continue;
            }
            let mut bounds = None;
            let mut visible = true;
            let mut z_order = 0i32;
            for fact in &obs.facts {
                match fact.predicate.as_str() {
                    "bounds" => bounds = parse_bounds(&fact.object),
                    "visible" => visible = fact.object != "false",
                    "z_order" => z_order = fact.object.parse::<i32>().unwrap_or(0),
                    _ => {}
                }
            }
            if let Some(bounds) = bounds {
                lenses.push(LayoutLens {
                    element_cid: obs.target_cid,
                    observation_cid: anchor.cid,
                    bounds,
                    z_order,
                    visible,
                });
            }
        }
        Self { lenses }
    }

    /// Given screen coordinates, resolve the stable CID of the element.
    pub fn resolve(&self, x: u32, y: u32) -> Vec<PixelCandidate> {
        let mut candidates = Vec::new();
        for lens in &self.lenses {
            if !lens.visible || !lens.bounds.contains(x, y) {
                continue;
            }
            candidates.push(PixelCandidate {
                element_cid: lens.element_cid,
                observation_cid: lens.observation_cid,
                coverage_micros: 1_000_000,
                z_order: lens.z_order,
                bounds: lens.bounds,
            });
        }
        candidates.sort_by(|left, right| {
            right
                .z_order
                .cmp(&left.z_order)
                .then_with(|| left.bounds.area().cmp(&right.bounds.area()))
                .then_with(|| left.element_cid.0.cmp(&right.element_cid.0))
        });
        candidates
    }

    /// Given a CID, return the bounding box in screen coordinates.
    pub fn bounds(&self, cid: &Cid) -> Option<PixelBounds> {
        self.lenses
            .iter()
            .find(|lens| &lens.element_cid == cid)
            .map(|lens| lens.bounds)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayoutLens {
    pub element_cid: Cid,
    pub observation_cid: Cid,
    pub bounds: PixelBounds,
    pub z_order: i32,
    pub visible: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PixelCandidate {
    pub element_cid: Cid,
    pub observation_cid: Cid,
    /// Fixed-point coverage in millionths; `1_000_000` means full point hit.
    pub coverage_micros: u32,
    pub z_order: i32,
    pub bounds: PixelBounds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelBounds {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl PixelBounds {
    fn contains(&self, x: u32, y: u32) -> bool {
        x >= self.x
            && y >= self.y
            && x < self.x.saturating_add(self.width)
            && y < self.y.saturating_add(self.height)
            && self.width > 0
            && self.height > 0
    }

    fn area(&self) -> u64 {
        self.width as u64 * self.height as u64
    }
}

fn parse_bounds(raw: &str) -> Option<PixelBounds> {
    let mut parts = raw.split(',');
    let x = parts.next()?.trim().parse().ok()?;
    let y = parts.next()?.trim().parse().ok()?;
    let width = parts.next()?.trim().parse().ok()?;
    let height = parts.next()?.trim().parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some(PixelBounds {
        x,
        y,
        width,
        height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        browser_types::{
            Provenance, TermFamily, TrustClass, WEB_ANCHOR_DOMAIN, WEB_ELEMENT_DOMAIN,
        },
        observation::anchor::{Fact, ObservationAnchor},
        PrivacyTier,
    };
    use alloc::string::ToString;
    use alloc::vec;

    fn element(seed: &[u8], bounds: &str, z_order: &str, visible: &str) -> crate::WebAnchor {
        let cid = Cid::compute(WEB_ELEMENT_DOMAIN, seed);
        ObservationAnchor {
            kind: ObservationKind::Element,
            target_cid: cid,
            observed_at: "2026-06-18T00:00:00Z".to_string(),
            facts: vec![
                Fact {
                    predicate: "bounds".to_string(),
                    object: bounds.to_string(),
                    sensitivity: None,
                },
                Fact {
                    predicate: "z_order".to_string(),
                    object: z_order.to_string(),
                    sensitivity: None,
                },
                Fact {
                    predicate: "visible".to_string(),
                    object: visible.to_string(),
                    sensitivity: None,
                },
            ],
            sensitivity: None,
            privacy_tier: PrivacyTier::LocalFull,
            trust_class: TrustClass::UntrustedContent,
            raw_source: None,
        }
        .to_anchor(Provenance {
            source: "test".to_string(),
            input_cids: Vec::new(),
            trust_class: TrustClass::UntrustedContent,
            did_principal: None,
        })
    }

    #[test]
    fn resolve_returns_topmost_visible_candidate_first() {
        let lower = element(b"lower", "0,0,100,100", "1", "true");
        let upper = element(b"upper", "10,10,20,20", "2", "true");
        let hidden = element(b"hidden", "10,10,20,20", "3", "false");
        let upper_cid = observation_from_payload(&upper.payload).unwrap().target_cid;

        let anchor = PixelAnchor::from_observations(&[lower, upper, hidden]);
        let candidates = anchor.resolve(15, 15);

        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].element_cid, upper_cid);
        assert_eq!(candidates[0].coverage_micros, 1_000_000);
    }

    #[test]
    fn bounds_projects_cid_to_screen_box() {
        let observation = element(b"button", "12,34,56,78", "0", "true");
        let element_cid = observation_from_payload(&observation.payload)
            .unwrap()
            .target_cid;
        let anchor = PixelAnchor::from_observations(&[observation]);

        assert_eq!(
            anchor.bounds(&element_cid),
            Some(PixelBounds {
                x: 12,
                y: 34,
                width: 56,
                height: 78
            })
        );
    }

    #[test]
    fn malformed_bounds_are_ignored() {
        let observation = crate::WebAnchor {
            cid: Cid::compute(WEB_ANCHOR_DOMAIN, b"bad"),
            term_family: TermFamily::Observation,
            created_at: "2026-06-18T00:00:00Z".to_string(),
            provenance: Provenance {
                source: "test".to_string(),
                input_cids: Vec::new(),
                trust_class: TrustClass::UntrustedContent,
                did_principal: None,
            },
            payload: ObservationAnchor {
                kind: ObservationKind::Element,
                target_cid: Cid::compute(WEB_ELEMENT_DOMAIN, b"bad"),
                observed_at: "2026-06-18T00:00:00Z".to_string(),
                facts: vec![Fact {
                    predicate: "bounds".to_string(),
                    object: "not,bounds".to_string(),
                    sensitivity: None,
                }],
                sensitivity: None,
                privacy_tier: PrivacyTier::LocalFull,
                trust_class: TrustClass::UntrustedContent,
                raw_source: None,
            }
            .canonical_bytes(),
        };

        let anchor = PixelAnchor::from_observations(&[observation]);

        assert!(anchor.resolve(0, 0).is_empty());
    }
}
