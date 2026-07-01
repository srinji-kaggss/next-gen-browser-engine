use crate::scope::Scope;

pub type NodeId = u64;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NodeKind(pub &'static str);

impl NodeKind {
    pub const ELEMENT: NodeKind = NodeKind("element");
    pub const TEXT: NodeKind = NodeKind("text");
    pub const FRAGMENT: NodeKind = NodeKind("fragment");

    pub fn as_str(&self) -> &str {
        self.0
    }
}

#[derive(Clone, Debug, Default)]
pub struct SanitizedExcerpt(pub String);

impl std::ops::Deref for SanitizedExcerpt {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub struct Candidate {
    pub node_id: NodeId,
    pub kind: NodeKind,
    pub provenance: Scope,
    pub excerpt: SanitizedExcerpt,
    pub evidence_refs: Vec<u64>,
}

impl Default for Candidate {
    fn default() -> Self {
        Self {
            node_id: 0,
            kind: NodeKind::ELEMENT,
            provenance: Scope::construct(0, 0, 0, Default::default()),
            excerpt: Default::default(),
            evidence_refs: vec![],
        }
    }
}
