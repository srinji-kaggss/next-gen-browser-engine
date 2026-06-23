//! # Bridge — map be-capability enum to braid_capability tokens
//!
//! The browser engine uses `be_capabaility::Capability` (enum) internally
//! for privacy levels and capability checking. Braid uses `braid_capability::Capability`
//! (string newtype). This module bridges the two.

use be_capability::Capability as BrowserCap;
use braid_capability::Capability as BraidCap;

/// Map a browser capability enum to its Braid dotted-name token.
///
/// Returns `None` for capabilities that don't map to a web.* term
/// (e.g., `CodeDynamic` has no direct web.* equivalent).
pub fn to_braid_cap(cap: BrowserCap) -> Option<BraidCap> {
    let name = match cap {
        BrowserCap::DomRead => "web.dom.read",
        BrowserCap::DomWrite => "web.dom.write",
        BrowserCap::DomActionClick => "web.dom.click",
        BrowserCap::DomActionSubmit => "web.dom.submit",
        BrowserCap::DomActionFocus => "web.dom.focus",
        BrowserCap::DomActionType => "web.dom.type",
        BrowserCap::NetworkEgress => "web.egress",
        BrowserCap::NetworkRead => "web.dom.read", // network read ≈ dom read for taint
        BrowserCap::StorageRead => "web.storage.read",
        BrowserCap::StorageWrite => "web.storage.write",
        BrowserCap::AiAffordances => "web.observe",
        BrowserCap::AiActions => "web.ai.act",
        BrowserCap::AiRead => "web.ai.read",
        BrowserCap::NavigationNavigate => "web.navigate",
        // No direct web.* mapping for these
        BrowserCap::MediaPlay | BrowserCap::MediaCapture => return None,
        BrowserCap::NavigationHistory => return None,
        BrowserCap::CodeDynamic | BrowserCap::CodeBraid => return None,
    };
    Some(BraidCap::new(name))
}

/// Convert a set of browser capabilities to Braid capabilities.
/// Filters out unmapped ones.
pub fn browser_caps_to_braid(caps: &[BrowserCap]) -> Vec<BraidCap> {
    caps.iter().filter_map(|c| to_braid_cap(*c)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use be_capability::PrivacyLevel;

    #[test]
    fn dom_read_maps_to_web_dom_read() {
        let braid = to_braid_cap(BrowserCap::DomRead).unwrap();
        assert_eq!(braid.as_str(), "web.dom.read");
    }

    #[test]
    fn click_maps_to_web_dom_click() {
        let braid = to_braid_cap(BrowserCap::DomActionClick).unwrap();
        assert_eq!(braid.as_str(), "web.dom.click");
    }

    #[test]
    fn media_has_no_mapping() {
        assert!(to_braid_cap(BrowserCap::MediaPlay).is_none());
    }

    #[test]
    fn medium_privacy_maps_to_braid_caps() {
        let browser_caps = PrivacyLevel::Medium.capabilities();
        let braid_caps = browser_caps_to_braid(&browser_caps);
        // Medium includes DomRead, DomWrite, Click, Focus, Type, etc.
        assert!(braid_caps.len() >= 5);
        let names: Vec<&str> = braid_caps.iter().map(|c| c.as_str()).collect();
        assert!(names.contains(&"web.dom.read"));
        assert!(names.contains(&"web.dom.click"));
        assert!(names.contains(&"web.ai.read"));
    }

    #[test]
    fn off_privacy_maps_to_empty() {
        let browser_caps = PrivacyLevel::Off.capabilities();
        let braid_caps = browser_caps_to_braid(&browser_caps);
        assert!(braid_caps.is_empty());
    }
}
