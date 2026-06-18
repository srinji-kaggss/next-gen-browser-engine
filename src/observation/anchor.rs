//! Traceability: AXIOM_OBSERVABILITY_TYPED, AXIOM_BRAID_CANONICAL, AXIOM_PRIVACY_TIER.
use crate::browser_types::*;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

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
    /// Deterministic canonical bytes used to compute this observation's CID.
    ///
    /// Traceability: AXIOM_BRAID_CANONICAL, AXIOM_PROVENANCE_MANDATORY.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(b"observation\n");
        write_field(&mut out, "kind", self.kind.as_str());
        write_field(&mut out, "target_cid", &self.target_cid);
        write_field(&mut out, "observed_at", &self.observed_at);
        write_field(&mut out, "privacy_tier", self.privacy_tier.as_str());
        write_field(&mut out, "trust_class", self.trust_class.as_str());

        // Sort facts by predicate for determinism.
        let mut sorted = self.facts.clone();
        sorted.sort_by(|a, b| a.predicate.cmp(&b.predicate));
        for fact in &sorted {
            out.extend_from_slice(b"fact\n");
            write_field(&mut out, "predicate", &fact.predicate);
            write_field(&mut out, "object", &fact.object);
            if let Some(s) = fact.sensitivity {
                write_field(&mut out, "sensitivity", s.as_str());
            }
        }
        out
    }

    /// Seal this observation as a content-addressed `WebAnchor`.
    pub fn to_anchor(&self, provenance: Provenance) -> WebAnchor {
        let payload = self.canonical_bytes();
        let cid = cid_from_bytes(&payload);
        WebAnchor {
            cid,
            term_family: TermFamily::Observation,
            created_at: self.observed_at.clone(),
            provenance,
            payload,
        }
    }
}

fn write_field(out: &mut Vec<u8>, key: &str, value: &str) {
    out.extend_from_slice(key.as_bytes());
    out.push(b'=');
    out.extend_from_slice(value.as_bytes());
    out.push(b'\n');
}

/// Recover an `ObservationAnchor` from the canonical payload of a `WebAnchor`.
///
/// Traceability: AXIOM_BRAID_CANONICAL.
pub fn observation_from_payload(payload: &[u8]) -> Result<ObservationAnchor, &'static str> {
    let s = core::str::from_utf8(payload).map_err(|_| "invalid utf8")?;
    let mut lines = s.lines();
    let first = lines.next().ok_or("missing observation header")?;
    if first != "observation" {
        return Err("missing observation prefix");
    }

    let mut kind: Option<ObservationKind> = None;
    let mut target_cid: Option<String> = None;
    let mut observed_at: Option<String> = None;
    let mut privacy_tier: Option<PrivacyTier> = None;
    let mut trust_class: Option<TrustClass> = None;
    let mut facts: Vec<Fact> = Vec::new();
    let mut current: Option<Fact> = None;

    for line in lines {
        if line == "fact" {
            if let Some(f) = current.take() {
                facts.push(f);
            }
            current = Some(Fact {
                predicate: String::new(),
                object: String::new(),
                sensitivity: None,
            });
            continue;
        }
        let (key, value) = line.split_once('=').ok_or("malformed field")?;
        match key {
            "kind" => kind = Some(parse_observation_kind(value)?),
            "target_cid" => target_cid = Some(value.to_string()),
            "observed_at" => observed_at = Some(value.to_string()),
            "privacy_tier" => privacy_tier = Some(parse_privacy_tier(value)?),
            "trust_class" => trust_class = Some(parse_trust_class(value)?),
            "predicate" => {
                current
                    .as_mut()
                    .ok_or("predicate outside fact")?
                    .predicate = value.to_string();
            }
            "object" => {
                current.as_mut().ok_or("object outside fact")?.object = value.to_string();
            }
            "sensitivity" => {
                current.as_mut().ok_or("sensitivity outside fact")?.sensitivity =
                    Some(parse_sensitivity_class(value)?);
            }
            _ => return Err("unknown field"),
        }
    }
    if let Some(f) = current.take() {
        facts.push(f);
    }

    Ok(ObservationAnchor {
        kind: kind.ok_or("missing kind")?,
        target_cid: target_cid.ok_or("missing target_cid")?,
        observed_at: observed_at.ok_or("missing observed_at")?,
        facts,
        sensitivity: None,
        privacy_tier: privacy_tier.ok_or("missing privacy_tier")?,
        trust_class: trust_class.ok_or("missing trust_class")?,
        raw_source: None,
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
    use alloc::vec;
    use alloc::string::ToString;

    fn sample_observation() -> ObservationAnchor {
        ObservationAnchor {
            kind: ObservationKind::Element,
            target_cid: "a1b2c3".to_string(),
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
        assert_eq!(anchor.cid.len(), 64);
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
