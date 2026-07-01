//! Traceability: AXIOM_OBSERVABILITY_TYPED, AXIOM_BRAID_CANONICAL, AXIOM_PRIVACY_TIER.
use crate::browser_types::*;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use braid_ir::{canon, Value};

/// A typed observation fact derived from WebKit output.
pub struct ObservationAnchor {
    pub kind: ObservationKind,
    pub target_cid: Cid,
    pub observed_at: String,
    pub facts: Vec<Fact>,
    pub sensitivity: Option<SensitivityClass>,
    pub privacy_tier: PrivacyTier,
    pub trust_class: TrustClass,
    pub raw_source: Option<String>,
}

impl ObservationAnchor {
    /// Deterministic canonical bytes used to compute this observation's CID:
    /// Braid's bijection-guarded canonical encoding (`canon::encode`) of the
    /// typed value — NOT a hand-rolled format. One observation ⇒ one byte
    /// string, and decode re-encodes to verify (T3). Map-key order is Braid's.
    ///
    /// Traceability: AXIOM_BRAID_CANONICAL, AXIOM_PROVENANCE_MANDATORY.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        canon::encode(&self.to_value())
    }

    /// Project this observation into the canonical Braid value universe.
    pub fn to_value(&self) -> Value {
        let mut sorted = self.facts.clone();
        sorted.sort_by(|a, b| a.predicate.cmp(&b.predicate));
        let facts: Vec<Value> = sorted
            .iter()
            .map(|f| {
                let mut m = vec![
                    ("object", Value::Text(f.object.clone())),
                    ("predicate", Value::Text(f.predicate.clone())),
                ];
                if let Some(s) = f.sensitivity {
                    m.push(("sensitivity", Value::Text(s.as_str().to_string())));
                }
                Value::map(m)
            })
            .collect();
        Value::map(vec![
            ("facts", Value::List(facts)),
            ("kind", Value::Text(self.kind.as_str().to_string())),
            ("observed_at", Value::Text(self.observed_at.clone())),
            (
                "privacy_tier",
                Value::Text(self.privacy_tier.as_str().to_string()),
            ),
            ("target_cid", Value::Bytes(self.target_cid.0.to_vec())),
            (
                "trust_class",
                Value::Text(self.trust_class.as_str().to_string()),
            ),
        ])
    }

    /// Seal this observation as a content-addressed `WebAnchor`.
    pub fn to_anchor(&self, provenance: Provenance) -> WebAnchor {
        let payload = self.canonical_bytes();
        let cid = Cid::compute(WEB_ANCHOR_DOMAIN, &payload);
        WebAnchor {
            cid,
            term_family: TermFamily::Observation,
            created_at: self.observed_at.clone(),
            provenance,
            payload,
        }
    }
}

/// Recover an `ObservationAnchor` from the canonical payload of a `WebAnchor`.
///
/// Traceability: AXIOM_BRAID_CANONICAL.
/// Strict: only Braid-canonical bytes decode (bijection-guarded). Unknown
/// fields are rejected (T3); absence is never defaulted (fail-closed L9).
///
/// Traceability: AXIOM_BRAID_CANONICAL.
pub fn observation_from_payload(payload: &[u8]) -> Result<ObservationAnchor, &'static str> {
    let v = canon::decode_strict(payload).map_err(|_| "non-canonical observation payload")?;
    if !v.require_only_keys(&[
        "facts",
        "kind",
        "observed_at",
        "privacy_tier",
        "target_cid",
        "trust_class",
    ]) {
        return Err("observation: unknown field");
    }
    let kind = match v.get("kind") {
        Some(Value::Text(s)) => parse_observation_kind(s)?,
        _ => return Err("missing kind"),
    };
    let target_cid = match v.get("target_cid") {
        Some(Value::Bytes(b)) if b.len() == 32 => {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(b);
            Cid(arr)
        }
        _ => return Err("missing or malformed target_cid"),
    };
    let observed_at = match v.get("observed_at") {
        Some(Value::Text(s)) => s.clone(),
        _ => return Err("missing observed_at"),
    };
    let privacy_tier = match v.get("privacy_tier") {
        Some(Value::Text(s)) => parse_privacy_tier(s)?,
        _ => return Err("missing privacy_tier"),
    };
    let trust_class = match v.get("trust_class") {
        Some(Value::Text(s)) => parse_trust_class(s)?,
        _ => return Err("missing trust_class"),
    };
    let facts = match v.get("facts") {
        Some(Value::List(items)) => items
            .iter()
            .map(fact_from_value)
            .collect::<Result<Vec<_>, _>>()?,
        _ => return Err("missing facts"),
    };
    Ok(ObservationAnchor {
        kind,
        target_cid,
        observed_at,
        facts,
        sensitivity: None,
        privacy_tier,
        trust_class,
        raw_source: None,
    })
}

fn fact_from_value(v: &Value) -> Result<Fact, &'static str> {
    if !v.require_only_keys(&["object", "predicate", "sensitivity"]) {
        return Err("fact: unknown field");
    }
    let predicate = match v.get("predicate") {
        Some(Value::Text(s)) => s.clone(),
        _ => return Err("missing predicate"),
    };
    let object = match v.get("object") {
        Some(Value::Text(s)) => s.clone(),
        _ => return Err("missing object"),
    };
    let sensitivity = match v.get("sensitivity") {
        None => None,
        Some(Value::Text(s)) => Some(parse_sensitivity_class(s)?),
        Some(_) => return Err("malformed sensitivity"),
    };
    Ok(Fact {
        predicate,
        object,
        sensitivity,
    })
}

fn parse_observation_kind(s: &str) -> Result<ObservationKind, &'static str> {
    match s {
        "load" => Ok(ObservationKind::Load),
        "layout" => Ok(ObservationKind::Layout),
        "paint" => Ok(ObservationKind::Paint),
        "element" => Ok(ObservationKind::Element),
        "network" => Ok(ObservationKind::Network),
        "console" => Ok(ObservationKind::Console),
        "a11y" => Ok(ObservationKind::A11y),
        "semantic" => Ok(ObservationKind::Semantic),
        "aip_state" => Ok(ObservationKind::AipState),
        "aip_policy" => Ok(ObservationKind::AipPolicy),
        _ => Err("unknown observation kind"),
    }
}

fn parse_privacy_tier(s: &str) -> Result<PrivacyTier, &'static str> {
    match s {
        "local_full" => Ok(PrivacyTier::LocalFull),
        "cloud_redacted" => Ok(PrivacyTier::CloudRedacted),
        "cloud_selective_reveal" => Ok(PrivacyTier::CloudSelectiveReveal),
        "cloud_full_context_explicit" => Ok(PrivacyTier::CloudFullContextExplicit),
        _ => Err("unknown privacy tier"),
    }
}

fn parse_trust_class(s: &str) -> Result<TrustClass, &'static str> {
    match s {
        "SYSTEM_POLICY" => Ok(TrustClass::SystemPolicy),
        "DEVELOPER_POLICY" => Ok(TrustClass::DeveloperPolicy),
        "USER_INTENT" => Ok(TrustClass::UserIntent),
        "TRUSTED_STATE" => Ok(TrustClass::TrustedState),
        "UNTRUSTED_CONTENT" => Ok(TrustClass::UntrustedContent),
        _ => Err("unknown trust class"),
    }
}

fn parse_sensitivity_class(s: &str) -> Result<SensitivityClass, &'static str> {
    match s {
        "PUBLIC" => Ok(SensitivityClass::Public),
        "LOW_SENSITIVITY" => Ok(SensitivityClass::LowSensitivity),
        "PERSONAL" => Ok(SensitivityClass::Personal),
        "CONFIDENTIAL" => Ok(SensitivityClass::Confidential),
        "SECRET" => Ok(SensitivityClass::Secret),
        "AUTHENTICATOR" => Ok(SensitivityClass::Authenticator),
        "PAYMENT" => Ok(SensitivityClass::Payment),
        "HEALTH" => Ok(SensitivityClass::Health),
        "LEGAL" => Ok(SensitivityClass::Legal),
        "FINANCIAL" => Ok(SensitivityClass::Financial),
        "CHILD_OR_DEPENDENT" => Ok(SensitivityClass::ChildOrDependent),
        _ => Err("unknown sensitivity class"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use alloc::vec;

    fn sample_observation() -> ObservationAnchor {
        ObservationAnchor {
            kind: ObservationKind::Element,
            target_cid: Cid::compute(WEB_ELEMENT_DOMAIN, b"a1b2c3"),
            observed_at: "2026-06-18T00:00:00Z".to_string(),
            facts: vec![
                Fact {
                    predicate: "tag".to_string(),
                    object: "a".to_string(),
                    sensitivity: None,
                },
                Fact {
                    predicate: "text".to_string(),
                    object: "Sign in".to_string(),
                    sensitivity: None,
                },
            ],
            sensitivity: None,
            privacy_tier: PrivacyTier::LocalFull,
            trust_class: TrustClass::UntrustedContent,
            raw_source: None,
        }
    }

    #[test]
    fn canonical_bytes_are_deterministic() {
        let o1 = sample_observation();
        let o2 = sample_observation();
        assert_eq!(o1.canonical_bytes(), o2.canonical_bytes());
    }

    #[test]
    fn to_anchor_is_content_addressed() {
        let obs = sample_observation();
        let anchor = obs.to_anchor(Provenance {
            source: "test".to_string(),
            input_cids: Vec::new(),
            trust_class: TrustClass::SystemPolicy,
            did_principal: None,
        });
        assert_eq!(anchor.term_family, TermFamily::Observation);
        assert_eq!(anchor.cid.to_hex().len(), 64);
        assert!(!anchor.payload.is_empty());
    }

    #[test]
    fn round_trip_payload() {
        let obs = sample_observation();
        let anchor = obs.to_anchor(Provenance {
            source: "test".to_string(),
            input_cids: Vec::new(),
            trust_class: TrustClass::SystemPolicy,
            did_principal: None,
        });
        let recovered = observation_from_payload(&anchor.payload).unwrap();
        assert_eq!(recovered.kind, obs.kind);
        assert_eq!(recovered.target_cid, obs.target_cid);
        assert_eq!(recovered.facts.len(), obs.facts.len());
    }

    #[test]
    fn parser_rejects_malformed_payload() {
        assert!(observation_from_payload(b"not-an-observation").is_err());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservationKind {
    Load,
    Layout,
    Paint,
    Element,
    Network,
    Console,
    A11y,
    Semantic,
    AipState,
    AipPolicy,
}

impl ObservationKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ObservationKind::Load => "load",
            ObservationKind::Layout => "layout",
            ObservationKind::Paint => "paint",
            ObservationKind::Element => "element",
            ObservationKind::Network => "network",
            ObservationKind::Console => "console",
            ObservationKind::A11y => "a11y",
            ObservationKind::Semantic => "semantic",
            ObservationKind::AipState => "aip_state",
            ObservationKind::AipPolicy => "aip_policy",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fact {
    pub predicate: String,
    pub object: String,
    pub sensitivity: Option<SensitivityClass>,
}
