//! # be-axiom — Content-addressed primitives
//!
//! Provides CID (content identifier), wire format (TLV encoding), and
//! content-addressed storage primitives. These are the building blocks for:
//! - Braid IR program identification (same source → same CID)
//! - PULSE frame encoding (compact wire format)
//! - Audit trail (tamper-evident entries)
//!
//! ## Blast radius
//!
//! This crate has NO dependencies on logic-os-kernel or Braid.
//! It is self-contained. Changes here only affect be-pulse and be-api.
//!
//! ## Example
//!
//! ```rust
//! use be_axiom::cid::Cid;
//! use be_axiom::wire::{self, Field};
//!
//! // Compute CID
//! let cid = Cid::from_bytes(b"hello world");
//! assert!(cid.verify(b"hello world"));
//!
//! // Encode fields
//! let fields = vec![
//!     Field::Varint { field_no: 1, value: 42 },
//!     Field::Len { field_no: 2, value: b"hello".to_vec() },
//! ];
//! let encoded = wire::encode(&fields);
//! let decoded = wire::decode_canonical(&encoded).unwrap();
//! assert_eq!(fields, decoded);
//! ```

pub mod cid;
pub mod wire;

// Re-export for convenience
pub use cid::Cid;
pub use wire::Field;
