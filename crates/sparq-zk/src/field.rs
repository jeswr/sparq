//! BN254 scalar-field helpers.
//!
//! `Fr` (ark-bn254's scalar field) is exactly Noir's `Field` type for the
//! default BN254 backend, so every value produced here can be handed to a
//! Noir circuit witness verbatim.

use ark_ff::{BigInteger, PrimeField};

pub type Fr = ark_bn254::Fr;

/// Decodes a big-endian hex string (no `0x` prefix) and reduces it into the
/// field. Mirrors noir's `field_from_hex` (`from_be_bytes_reduce`).
pub(crate) fn field_from_hex(hex: &str) -> Fr {
    let bytes = decode_hex(hex);
    Fr::from_be_bytes_mod_order(&bytes)
}

/// Canonical lowercase hex (64 nibbles, `0x` prefixed) of a field element —
/// the representation stored in `<urn:sparq:zk>` registry literals and
/// compared against `nargo` output in the cross-tests.
pub fn field_to_hex(f: &Fr) -> String {
    let bytes = f.into_bigint().to_bytes_be();
    let mut s = String::with_capacity(2 + 64);
    s.push_str("0x");
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Parses a `0x`-prefixed (or bare) hex field literal, accepting fewer than
/// 64 nibbles (left-padded), reducing mod p.
pub fn field_from_hex_str(s: &str) -> Option<Fr> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    if s.is_empty() || s.len() > 64 || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let padded = format!("{s:0>64}");
    Some(field_from_hex(&padded))
}

/// Maps 32 hash bytes (e.g. a Blake3 digest) into the field by truncating to
/// the low 31 bytes (248 bits < 254-bit modulus) interpreted big-endian.
/// Truncation rather than modular reduction keeps the encoding bias-free and
/// trivially recomputable in-circuit (the value is a witness; the circuit
/// never re-derives it from bytes — see plan §2.2's off-circuit `h_s`).
pub fn field_from_hash_bytes(bytes: &[u8; 32]) -> Fr {
    Fr::from_be_bytes_mod_order(&bytes[1..])
}

fn decode_hex(hex: &str) -> Vec<u8> {
    assert!(hex.len() % 2 == 0, "hex string must have even length");
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).expect("valid hex"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_round_trip() {
        let f = field_from_hex_str("0x1a2b").unwrap();
        assert_eq!(f, Fr::from(0x1a2bu64));
        let h = field_to_hex(&f);
        assert_eq!(h, format!("0x{:0>64}", "1a2b"));
        assert_eq!(field_from_hex_str(&h).unwrap(), f);
    }

    #[test]
    fn hash_bytes_truncate_to_248_bits() {
        let bytes = [0xffu8; 32];
        let f = field_from_hash_bytes(&bytes);
        // 2^248 - 1 < p, so the value must round-trip exactly.
        let expected = field_from_hex_str(&"ff".repeat(31)).unwrap();
        assert_eq!(f, expected);
    }

    #[test]
    fn rejects_garbage() {
        assert!(field_from_hex_str("0xzz").is_none());
        assert!(field_from_hex_str("").is_none());
        assert!(field_from_hex_str(&"f".repeat(65)).is_none());
    }
}
