//! Traceability: AXIOM_TAPE_APPEND_ONLY, AXIOM_OBSERVABILITY_TYPED.
use crate::browser_types::*;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

/// Audit lens: project canonical facts into human/AI inspectable views.
pub struct AuditLens;

impl AuditLens {
    pub fn new() -> Self {
        Self
    }

    pub fn provenance(anchor: &WebAnchor) -> Result<Vec<Cid>, &'static str> {
        let mut chain = Vec::new();
        for cid in &anchor.provenance.input_cids {
            if chain.contains(cid) {
                return Err("provenance cycle");
            }
            chain.push(*cid);
        }
        Ok(chain)
    }

    pub fn diff(left: &[WebAnchor], right: &[WebAnchor]) -> Vec<Diff> {
        let left_by_cid = anchors_by_cid(left);
        let right_by_cid = anchors_by_cid(right);
        let mut out = Vec::new();

        for (cid, left_anchor) in &left_by_cid {
            match right_by_cid.get(cid) {
                None => out.push(Diff {
                    cid: *cid,
                    before: Some(left_anchor.payload.clone()),
                    after: None,
                }),
                Some(right_anchor) if left_anchor.payload != right_anchor.payload => {
                    out.push(Diff {
                        cid: *cid,
                        before: Some(left_anchor.payload.clone()),
                        after: Some(right_anchor.payload.clone()),
                    });
                }
                Some(_) => {}
            }
        }

        for (cid, right_anchor) in &right_by_cid {
            if !left_by_cid.contains_key(cid) {
                out.push(Diff {
                    cid: *cid,
                    before: None,
                    after: Some(right_anchor.payload.clone()),
                });
            }
        }

        out.sort_by_key(|diff| diff.cid.0);
        out
    }
}

impl Default for AuditLens {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Diff {
    pub cid: Cid,
    pub before: Option<Vec<u8>>,
    pub after: Option<Vec<u8>>,
}

fn anchors_by_cid(anchors: &[WebAnchor]) -> BTreeMap<Cid, &WebAnchor> {
    let mut out = BTreeMap::new();
    for anchor in anchors {
        out.insert(anchor.cid, anchor);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use alloc::vec;

    fn anchor(seed: &[u8], inputs: Vec<Cid>, payload: Vec<u8>) -> WebAnchor {
        WebAnchor {
            cid: Cid::compute(WEB_ANCHOR_DOMAIN, seed),
            term_family: TermFamily::Observation,
            created_at: "2026-06-18T00:00:00Z".to_string(),
            provenance: Provenance {
                source: "test".to_string(),
                input_cids: inputs,
                trust_class: TrustClass::SystemPolicy,
                did_principal: None,
            },
            payload,
        }
    }

    #[test]
    fn provenance_returns_input_chain() {
        let first = Cid::compute(WEB_ANCHOR_DOMAIN, b"first");
        let second = Cid::compute(WEB_ANCHOR_DOMAIN, b"second");
        let anchor = anchor(b"root", vec![first, second], Vec::new());

        assert_eq!(AuditLens::provenance(&anchor), Ok(vec![first, second]));
    }

    #[test]
    fn provenance_rejects_duplicate_cycle() {
        let cid = Cid::compute(WEB_ANCHOR_DOMAIN, b"dup");
        let anchor = anchor(b"root", vec![cid, cid], Vec::new());

        assert_eq!(AuditLens::provenance(&anchor), Err("provenance cycle"));
    }

    #[test]
    fn diff_reports_added_removed_and_changed_anchors() {
        let unchanged = anchor(b"same", Vec::new(), b"same".to_vec());
        let removed = anchor(b"removed", Vec::new(), b"old".to_vec());
        let mut changed_right = anchor(b"changed", Vec::new(), b"new".to_vec());
        let changed_left = anchor(b"changed", Vec::new(), b"old".to_vec());
        changed_right.cid = changed_left.cid;
        let added = anchor(b"added", Vec::new(), b"added".to_vec());

        let diffs = AuditLens::diff(
            &[unchanged.clone(), removed.clone(), changed_left.clone()],
            &[unchanged, changed_right.clone(), added.clone()],
        );

        assert_eq!(diffs.len(), 3);
        assert!(diffs.iter().any(|d| d.cid == removed.cid
            && d.before == Some(b"old".to_vec())
            && d.after.is_none()));
        assert!(diffs.iter().any(|d| d.cid == added.cid
            && d.before.is_none()
            && d.after == Some(b"added".to_vec())));
        assert!(diffs.iter().any(|d| d.cid == changed_left.cid
            && d.before == Some(b"old".to_vec())
            && d.after == Some(b"new".to_vec())));
    }
}
