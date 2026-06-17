//! Traceability: AXIOM_BRAID_CANONICAL, AXIOM_LLMS_SENSOR_ONLY.
use crate::browser_types::*;
use alloc::string::String;
use alloc::vec::Vec;

/// Adapter to the native macOS WKWebView bridge (mac-eye).
/// Translates WebKit output into Braid observation facts.
pub struct WebKitAdapter;

impl WebKitAdapter {
    pub fn new() -> Self {
        Self
    }

    pub fn load(&self, _url: &Url) -> Result<Cid, &str> {
        todo!("navigate via native bridge and emit observation")
    }

    pub fn observe(&self) -> Result<Vec<WebAnchor>, &str> {
        todo!("collect typed observations from WebKit")
    }

    pub fn execute_js(&self, _script: &str) -> Result<String, &str> {
        todo!("execute deterministic JS through capability broker")
    }
}

impl Default for WebKitAdapter {
    fn default() -> Self {
        Self::new()
    }
}
