//! Canonical TLV wire format — tag-length-value over LEB128.
//!
//! Copied from logicalworks-/axiom/wire.py with Rust rewrite.
//!
//! TODO: consolidate with braid-ir wire format when Braid stabilizes.
//!
//! Two wire types: VARINT (0) for ints/bools, LEN (2) for bytes/strings/nested.
//! Canonicality: fields sorted by (field_no, value), minimal varints.
//! One byte form per message — no ambiguity.

use thiserror::Error;

/// Wire type for integer/boolean values.
pub const VARINT: u8 = 0;
/// Wire type for byte/string/nested values.
pub const LEN: u8 = 2;

/// A TLV field: (field_number, wire_type, value).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Field {
    /// Integer field.
    Varint { field_no: u32, value: u64 },
    /// Bytes field.
    Len { field_no: u32, value: Vec<u8> },
}

/// Errors during wire encoding/decoding.
#[derive(Debug, Error)]
pub enum WireError {
    #[error("field number must be >= 1, got {0}")]
    InvalidFieldNumber(u32),
    #[error("unknown wire type {0}")]
    UnknownWireType(u8),
    #[error("bad varint: {0}")]
    BadVarint(String),
    #[error("truncated: need {needed} bytes at offset {offset}, have {have}")]
    Truncated { offset: usize, needed: usize, have: usize },
    #[error("non-canonical wire bytes")]
    NonCanonical,
}

// === LEB128 encoding/decoding ===

/// Encode a u64 as LEB128.
pub fn encode_uleb128(mut value: u64) -> Vec<u8> {
    let mut out = Vec::new();
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if value == 0 {
            break;
        }
    }
    out
}

/// Decode a LEB128 u64 from bytes at offset.
pub fn decode_uleb128(data: &[u8], offset: usize) -> Result<(u64, usize), WireError> {
    let mut result: u64 = 0;
    let mut shift = 0;
    let mut pos = offset;
    loop {
        if pos >= data.len() {
            return Err(WireError::BadVarint("truncated".into()));
        }
        let byte = data[pos];
        result |= ((byte & 0x7F) as u64) << shift;
        pos += 1;
        if byte & 0x80 == 0 {
            return Ok((result, pos));
        }
        shift += 7;
        if shift >= 64 {
            return Err(WireError::BadVarint("too long".into()));
        }
    }
}

// === Field sort key (for canonical ordering) ===

fn field_sort_key(f: &Field) -> (u32, u8, Vec<u8>) {
    match f {
        Field::Varint { field_no, value } => (*field_no, 0, encode_uleb128(*value)),
        Field::Len { field_no, value } => (*field_no, 1, value.clone()),
    }
}

// === Encode/Decode ===

/// Encode fields into canonical TLV bytes.
///
/// Fields are sorted by (field_no, wire_type, value) for canonicality.
pub fn encode(fields: &[Field]) -> Vec<u8> {
    let mut sorted: Vec<&Field> = fields.iter().collect();
    sorted.sort_by_key(|f| field_sort_key(f));

    let mut out = Vec::new();
    for field in sorted {
        match field {
            Field::Varint { field_no, value } => {
                if *field_no == 0 {
                    continue; // skip invalid
                }
                let tag = (field_no << 3) | (VARINT as u32);
                out.extend(encode_uleb128(tag as u64));
                out.extend(encode_uleb128(*value));
            }
            Field::Len { field_no, value } => {
                if *field_no == 0 {
                    continue; // skip invalid
                }
                let tag = (field_no << 3) | (LEN as u32);
                out.extend(encode_uleb128(tag as u64));
                out.extend(encode_uleb128(value.len() as u64));
                out.extend(value);
            }
        }
    }
    out
}

/// Decode TLV bytes into fields.
///
/// Preserves all fields (known or not) in stream order.
pub fn decode(data: &[u8]) -> Result<Vec<Field>, WireError> {
    let mut fields = Vec::new();
    let mut offset = 0;

    while offset < data.len() {
        let (tag, new_offset) = decode_uleb128(data, offset)
            .map_err(|e| WireError::BadVarint(format!("tag at {}: {}", offset, e)))?;
        offset = new_offset;

        let field_no = (tag >> 3) as u32;
        let wire_type = (tag & 0x7) as u8;

        if field_no == 0 {
            return Err(WireError::InvalidFieldNumber(0));
        }

        match wire_type {
            VARINT => {
                let (value, new_offset) = decode_uleb128(data, offset)
                    .map_err(|e| WireError::BadVarint(format!("value for field {}: {}", field_no, e)))?;
                offset = new_offset;
                fields.push(Field::Varint { field_no, value });
            }
            LEN => {
                let (length, new_offset) = decode_uleb128(data, offset)
                    .map_err(|e| WireError::BadVarint(format!("length for field {}: {}", field_no, e)))?;
                offset = new_offset;
                let length = length as usize;
                if offset + length > data.len() {
                    return Err(WireError::Truncated {
                        offset,
                        needed: length,
                        have: data.len() - offset,
                    });
                }
                fields.push(Field::Len {
                    field_no,
                    value: data[offset..offset + length].to_vec(),
                });
                offset += length;
            }
            _ => return Err(WireError::UnknownWireType(wire_type)),
        }
    }

    Ok(fields)
}

/// Decode AND enforce canonicality.
///
/// The input must be byte-identical to re-encoding what we decoded.
pub fn decode_canonical(data: &[u8]) -> Result<Vec<Field>, WireError> {
    let fields = decode(data)?;
    let re_encoded = encode(&fields);
    if re_encoded != data {
        return Err(WireError::NonCanonical);
    }
    Ok(fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leb128_roundtrip() {
        for value in [0u64, 1, 127, 128, 16383, 16384, u64::MAX] {
            let encoded = encode_uleb128(value);
            let (decoded, _) = decode_uleb128(&encoded, 0).unwrap();
            assert_eq!(value, decoded);
        }
    }

    #[test]
    fn test_wire_roundtrip() {
        let fields = vec![
            Field::Varint { field_no: 1, value: 42 },
            Field::Len { field_no: 2, value: b"hello".to_vec() },
            Field::Varint { field_no: 3, value: 0 },
        ];
        let encoded = encode(&fields);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(fields, decoded);
    }

    #[test]
    fn test_wire_canonical() {
        let fields = vec![
            Field::Varint { field_no: 1, value: 42 },
            Field::Len { field_no: 2, value: b"hello".to_vec() },
        ];
        let encoded = encode(&fields);
        let decoded = decode_canonical(&encoded).unwrap();
        assert_eq!(fields, decoded);
    }

    #[test]
    fn test_wire_non_canonical_rejected() {
        // Manually construct non-canonical bytes (wrong field order)
        let mut bad = Vec::new();
        // Field 2 first (should be field 1)
        bad.extend(encode_uleb128((2 << 3) | LEN as u64));
        bad.extend(encode_uleb128(3));
        bad.extend(b"abc");
        // Field 1 second
        bad.extend(encode_uleb128((1 << 3) | VARINT as u64));
        bad.extend(encode_uleb128(42));

        assert!(decode_canonical(&bad).is_err());
    }
}
