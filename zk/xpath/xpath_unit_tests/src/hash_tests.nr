//! Unit tests for the circuit-native string hash + hex formatter.
//!
//! [OPUS-4.8] sq-y73. `string_pedersen_hash` is a content commitment (NOT a
//! SPARQL MD5/SHA* digest -- those have no stdlib primitive at the pinned
//! toolchain; see xpath/src/hash.nr). `bytes_to_lower_hex` is the SPARQL hash
//! return-value (lowercase hex) formatter, ready to wrap a future byte digest.

use dep::xpath::{bytes_to_lower_hex, string_pedersen_hash};

#[test]
fn test_pedersen_hash_equal_inputs_via_api() {
    let a: str<6> = "sparql";
    let b: str<6> = "sparql";
    assert(string_pedersen_hash::<6, 2>(a) == string_pedersen_hash::<6, 2>(b));
}

#[test]
fn test_pedersen_hash_distinct_inputs_via_api() {
    let a: str<6> = "sparql";
    let b: str<6> = "sparqm";
    assert(string_pedersen_hash::<6, 2>(a) != string_pedersen_hash::<6, 2>(b));
}

#[test]
fn test_pedersen_hash_collision_resistance_length() {
    // "ab" vs "ab " (trailing space) must differ -- the space is real content.
    let a: str<3> = "ab\0";
    let b: str<3> = "ab ";
    assert(string_pedersen_hash::<3, 2>(a) != string_pedersen_hash::<3, 2>(b));
}

#[test]
fn test_hex_formatter_via_api() {
    // 0x0f 0xa0 -> "0fa0"
    let digest: [u8; 2] = [0x0F, 0xA0];
    let (hex, len) = bytes_to_lower_hex::<2, 4>(digest);
    assert(len == 4);
    let expected = "0fa0".as_bytes();
    for i in 0..4 {
        assert(hex[i] == expected[i]);
    }
}
