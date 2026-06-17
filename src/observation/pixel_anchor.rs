//! Traceability: AXIOM_DERIVED_LENS, AXIOM_HUMAN_DEFERRAL.
use crate::browser_types::Cid;

/// Bidirectional mapping between screen pixels and canonical CIDs.
pub struct PixelAnchor;

impl PixelAnchor {
    /// Given screen coordinates, resolve the stable CID of the element.
    pub fn resolve(_x: u32, _y: u32) -> Option<Cid> {
        todo!("pixel-to-cid reverse lookup")
    }

    /// Given a CID, return the bounding box in screen coordinates.
    pub fn bounds(_cid: &Cid) -> Option<PixelBounds> {
        todo!("cid-to-pixel forward lookup")
    }
}

pub struct PixelBounds {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
