//! Hash functions for SPARQL/XPath strings -- circuit-native commitment hash
//!
//! [OPUS-4.8] Added for beads task sq-y73 (string/regex/hash XPath functions).
//! Flag for re-review when Fable returns.
//!
//! ## Scope and honesty note
//!
//! SPARQL 1.1 defines the hash builtins `MD5`, `SHA1`, `SHA256`, `SHA384`,
//! `SHA512`, each mapping a lexical string to a **lowercase hex digest string**.
//! None of these is implemented here, for concrete, measured toolchain reasons:
//!
//! - **No stdlib primitive at this toolchain pin.** On the pinned toolchain
//!   (`nargo 1.0.0-beta.21`, `bb 5.0.0-nightly.20260324`) the Noir standard
//!   library exposes only the *algebraic* hashes -- `std::hash::pedersen_hash`,
//!   `pedersen_commitment`, `pedersen_hash_with_separator` (verified by
//!   probe: `std::hash::sha256`, `std::keccak256`, `std::hash::poseidon2` and
//!   friends all fail to resolve at beta.21; the byte-oriented crypto hashes
//!   moved out to external crates). Implementing SHA-256/512 would require
//!   vendoring an external Noir crate (e.g. `sha256`/`keccak256`), which is a
//!   dependency-policy decision for the orchestrator, not a self-contained
//!   in-circuit primitive.
//! - **MD5 and SHA-1 should not be hand-rolled.** Both are cryptographically
//!   broken (MD5 collisions are trivial; SHA-1 is `SHAttered`). There is no
//!   Noir primitive for either, and adding a bespoke bit-twiddled MD5/SHA-1
//!   compression function to a ZK crypto library would be a footgun: high
//!   constraint cost for a hash nobody should rely on. Scoped out deliberately.
//! - **Hex-string output is the expensive half.** Even with a digest, SPARQL's
//!   hash functions return the *hex string*; producing a 64-char `str<64>` from
//!   32 digest bytes is the byte-array-to-string pattern this crate already
//!   handles with the `([u8; N], len)` convention (`bytes_to_lower_hex` below),
//!   so that half is provided and reusable once a digest primitive lands.
//!
//! ## What IS provided
//!
//! 1. `string_pedersen_hash` -- a **circuit-native content hash** over the bytes
//!    of a string, returning a single `Field`. Pedersen is the hash the
//!    Barretenberg backend computes cheaply, and is exactly the right tool for
//!    the ZK use case the SPARQL hash builtins are usually a proxy for:
//!    *committing to / comparing string identity inside the proof* (e.g.
//!    proving two private literals are equal by equal commitment, or binding a
//!    literal to a public commitment). It is **not** MD5/SHA -- it is the
//!    appropriate substitute when the goal is an in-circuit content digest.
//!    Domain-separated so digests of different "kinds" cannot collide.
//! 2. `bytes_to_lower_hex` -- the digest->hex-string formatter (the SPARQL hash
//!    return-value shape), ready to wrap any future byte digest.
//!
//! ## Cost model
//!
//! `string_pedersen_hash` packs the `N` bytes into `ceil((N+8)/31)` field limbs
//! (31 bytes/limb keeps each limb < the BN254 modulus; an extra leading limb
//! carries the domain tag + logical length) and calls `pedersen_hash` once over
//! the limb array. Packing is O(N) cheap field MACs; the Pedersen cost is the
//! dominant term and is fixed-size in the number of limbs. See
//! `noir-circuit-patterns` "Triple commitment patterns" -- this is the
//! per-value sponge-style commitment, domain-separated.

/// Domain-separation tag for string content hashes. Mixed into the length limb
/// so a string digest can never collide with a digest of some other committed
/// object that happens to reuse `pedersen_hash` (cf. the domain-separation
/// discipline in `noir-circuit-patterns` / `verifiable-credentials-zk`).
global STRING_HASH_DOMAIN: Field = 0x73_70_61_72_71_5f_73_74; // "sparq_st"

/// Number of raw bytes packed into one BN254 field limb. 31 bytes < 2^248 stays
/// safely under the ~254-bit field modulus with no reduction ambiguity.
global BYTES_PER_LIMB: u32 = 31;

/// Logical length of a `str<N>` payload: bytes up to the first NUL terminator
/// (the capacity convention used across this crate).
fn logical_len<let N: u32>(bytes: [u8; N]) -> u32 {
    let mut len: u32 = 0;
    let mut ended = false;
    for i in 0..N {
        let is_zero = bytes[i] == 0;
        if (!ended) & (!is_zero) {
            len += 1;
        }
        ended = ended | is_zero;
    }
    len
}

/// Circuit-native content hash of a string's bytes -> `Field`.
///
/// Domain-separated and length-prefixed Pedersen hash over the byte payload.
/// Two strings hash equal **iff** they have the same logical length and the
/// same bytes in `[0, len)` (trailing capacity NULs are ignored because the
/// length is bound into the digest and bytes past `len` are not packed).
///
/// This is the right primitive for in-proof string identity / commitment. It is
/// NOT a SPARQL `SHA256`-compatible digest (see the module note).
///
/// `LIMBS` must satisfy `LIMBS * BYTES_PER_LIMB >= N` plus one for the header
/// limb; the function asserts the capacity so a too-small `LIMBS` fails loudly
/// rather than silently truncating.
pub fn string_pedersen_hash<let N: u32, let LIMBS: u32>(s: str<N>) -> Field {
    let bytes = s.as_bytes();
    let len = logical_len(bytes);

    // Header limb binds the domain tag and the logical length.
    let mut limbs: [Field; LIMBS] = [0; LIMBS];
    assert(LIMBS >= 1, "hash: need at least the header limb");
    limbs[0] = STRING_HASH_DOMAIN * 0x1_00_00_00_00 + (len as Field);

    // Pack the payload bytes (only those < len) into the remaining limbs,
    // BYTES_PER_LIMB per limb, big-endian within a limb.
    let payload_limbs = LIMBS - 1;
    assert(
        payload_limbs * BYTES_PER_LIMB >= N,
        "hash: LIMBS too small for string capacity N",
    );

    for i in 0..N {
        // Only fold real payload bytes; bytes at/after len are skipped so the
        // digest depends only on the logical content.
        if (i as u32) < len {
            let limb_idx = 1 + ((i as u32) / BYTES_PER_LIMB);
            // Shift the running limb left one byte and add the new byte.
            limbs[limb_idx] = limbs[limb_idx] * 256 + (bytes[i] as Field);
        }
    }

    std::hash::pedersen_hash(limbs)
}

/// Convert a nibble (0-15) to its lowercase hex ASCII byte.
fn nibble_to_lower_hex(n: u8) -> u8 {
    if n < 10 {
        48 + n // '0'..'9'
    } else {
        97 + n - 10 // 'a'..'f'
    }
}

/// Format a digest byte array as a lowercase hex string -- the SPARQL hash
/// return-value shape (`MD5`/`SHA*` all return lowercase hex). Each input byte
/// becomes two hex chars, so `OUT` must be `>= 2*N`. Returns `([u8; OUT], len)`.
///
/// Provided so that, once a byte-digest primitive is available (vendored SHA-256
/// crate or a future stdlib re-add), the SPARQL `SHA256(?s)` facade is a
/// one-liner: hash to `[u8; 32]`, then `bytes_to_lower_hex::<32, 64>`.
pub fn bytes_to_lower_hex<let N: u32, let OUT: u32>(digest: [u8; N]) -> ([u8; OUT], u32) {
    let mut out: [u8; OUT] = [0; OUT];
    let mut out_len: u32 = 0;
    for i in 0..N {
        if out_len + 2 <= (OUT as u32) {
            let b = digest[i];
            out[out_len] = nibble_to_lower_hex((b >> 4) & 0x0F);
            out[out_len + 1] = nibble_to_lower_hex(b & 0x0F);
            out_len += 2;
        }
    }
    (out, out_len)
}

// ============================================================================
// Tests
// ============================================================================

#[test]
fn test_string_pedersen_hash_deterministic() {
    let a: str<5> = "hello";
    let b: str<5> = "hello";
    assert(string_pedersen_hash::<5, 2>(a) == string_pedersen_hash::<5, 2>(b));
}

#[test]
fn test_string_pedersen_hash_distinguishes() {
    let a: str<5> = "hello";
    let b: str<5> = "world";
    assert(string_pedersen_hash::<5, 2>(a) != string_pedersen_hash::<5, 2>(b));
}

#[test]
fn test_string_pedersen_hash_length_bound() {
    // "ab\0\0" (logical "ab") must NOT collide with "abc" -- length is bound in.
    let a: str<4> = "ab\0\0";
    let b: str<3> = "abc";
    assert(string_pedersen_hash::<4, 2>(a) != string_pedersen_hash::<3, 2>(b));
}

#[test]
fn test_string_pedersen_hash_ignores_trailing_capacity() {
    // Same logical content "ab" in different capacities hashes the same.
    let a: str<2> = "ab";
    let b: str<5> = "ab\0\0\0";
    assert(string_pedersen_hash::<2, 2>(a) == string_pedersen_hash::<5, 2>(b));
}

#[test]
fn test_bytes_to_lower_hex() {
    // 0xDE 0xAD 0xBE 0xEF -> "deadbeef"
    let digest: [u8; 4] = [0xDE, 0xAD, 0xBE, 0xEF];
    let (hex, len) = bytes_to_lower_hex::<4, 8>(digest);
    assert(len == 8);
    let expected = "deadbeef".as_bytes();
    for i in 0..8 {
        assert(hex[i] == expected[i]);
    }
}

#[test]
fn test_bytes_to_lower_hex_zero_and_max() {
    // 0x00 0xFF -> "00ff"
    let digest: [u8; 2] = [0x00, 0xFF];
    let (hex, len) = bytes_to_lower_hex::<2, 4>(digest);
    assert(len == 4);
    let expected = "00ff".as_bytes();
    for i in 0..4 {
        assert(hex[i] == expected[i]);
    }
}
