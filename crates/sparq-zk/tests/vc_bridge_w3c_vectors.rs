#![cfg(feature = "vc-bridge")]
//! Canonical W3C Data-Integrity test vectors for the off-circuit VC ingest
//! bridge (sq-wklnb; Opus review follow-up on #1155).
//!
//! # Why this file exists
//! The in-module `vc_bridge` tests mint their own key, sign the `hashData`
//! **this crate just computed**, and check that it verifies. That witnesses the
//! fail-closed behaviour (tamper / wrong key / malformed bytes all reject) but
//! it is self-referential: if sparq's canonicalization, its SHA-256 ordering, or
//! its `proofConfigHash || documentHash` concatenation disagreed with the W3C
//! algorithm, those tests would still pass — they would just be internally
//! consistent about the wrong bytes.
//!
//! This suite closes that gap with the **published** vectors, so every step is
//! pinned against bytes sparq did not produce:
//!
//! 1. sparq's RDFC-1.0 canonical N-Quads equal the spec's canonical document
//!    **byte for byte** (including the trailing newline the hash covers);
//! 2. [`hash_data_from_triples`] reproduces the spec's published `hashData`;
//! 3. a proof issued by a **third party** (the W3C editors' key, signature
//!    transcribed from the spec's `proofValue`) verifies through the bridge's
//!    public entry points and ingests end to end.
//!
//! Each positive assertion is paired with a mutation that must go red — a
//! single-bit flip in the signature, a dropped credential triple, the other
//! suite's proof config, the other suite's key — so the suite cannot pass
//! vacuously.
//!
//! Vectors are vendored under `tests/fixtures/w3c-di-vectors/` (see its
//! `PROVENANCE.md` for sources, licence, and the published hashes).
//!
//! # Scope (honest)
//! This is the bridge's **off-circuit, ingest-time** host verification. Interop
//! here says the DI hashing + signature check agree with the W3C algorithm for
//! the two whole-credential `rdfc` suites; it asserts **no** in-circuit or
//! query-soundness property, and the ZK estate is **not externally audited**
//! (sq-qhy4). The selective-disclosure suites (`bbs-2023`, `ecdsa-sd-2023`) and
//! the P-384 profile are out of the bridge's scope and have no vectors here.

use oxrdf::{NamedNode, Triple};
use sha2::{Digest, Sha256};
use sparq_core::Graph;
use sparq_zk::canon::{canonicalize_triples, graph_triples};
use sparq_zk::encode::salt_from_bytes;
use sparq_zk::vc_bridge::{
    hash_data_from_triples, ingest_verified_vc, verify_source_proof, verify_source_proof_by_token,
    VcBridgeError, VcCryptosuite,
};

// ---------------------------------------------------------------------------
// The vendored vectors.
// ---------------------------------------------------------------------------

/// vc-di-eddsa Example 9 == vc-di-ecdsa Example 7 — the canonical form of the
/// unsigned Alumni Credential, shared by both suites.
const CREDENTIAL_NQ: &str = include_str!("fixtures/w3c-di-vectors/credential.nq");
/// vc-di-eddsa Example 10 / vc-di-ecdsa Example 8 — SHA-256 of the above.
const CREDENTIAL_HASH_HEX: &str =
    "517744132ae165a5349155bef0bb0cf2258fff99dfe1dbd914b938d775a36017";

/// vc-di-eddsa Example 12 — canonical proof options for `eddsa-rdfc-2022`.
const EDDSA_PROOF_CONFIG_NQ: &str =
    include_str!("fixtures/w3c-di-vectors/eddsa-rdfc-2022.proof-config.nq");
/// vc-di-ecdsa Example 10 — canonical proof options for `ecdsa-rdfc-2019`.
const ECDSA_PROOF_CONFIG_NQ: &str =
    include_str!("fixtures/w3c-di-vectors/ecdsa-rdfc-2019.proof-config.nq");

/// One published `rdfc` vector, quoted field by field from its spec section.
struct Vector {
    /// The suite under test.
    suite: VcCryptosuite,
    /// The vendored canonical proof-options document.
    proof_config_nq: &'static str,
    /// SHA-256 of `proof_config_nq` as published (eddsa Ex. 13 / ecdsa Ex. 11).
    proof_config_hash_hex: &'static str,
    /// `proofConfigHash || documentHash` as published (eddsa Ex. 14 / ecdsa Ex. 12).
    hash_data_hex: &'static str,
    /// The raw signature over `hash_data` (eddsa Ex. 15 / ecdsa Ex. 13).
    signature_hex: &'static str,
    /// The issuer verification key (eddsa Ex. 7 / ecdsa Ex. 5), multibase
    /// base58-btc over a multicodec-prefixed public key.
    public_key_multibase: &'static str,
    /// The multicodec prefix `public_key_multibase` must carry once decoded:
    /// `ed25519-pub` = `0xed 0x01`, `p256-pub` = `0x80 0x24`.
    key_multicodec: &'static [u8],
    /// The `proof.proofValue` of the signed credential (eddsa Ex. 16 / ecdsa
    /// Ex. 14) — multibase base58-btc over `signature_hex`.
    proof_value: &'static str,
}

const EDDSA_RDFC_2022: Vector = Vector {
    suite: VcCryptosuite::EddsaRdfc2022,
    proof_config_nq: EDDSA_PROOF_CONFIG_NQ,
    proof_config_hash_hex: "bea7b7acfbad0126b135104024a5f1733e705108f42d59668b05c0c50004c6b0",
    hash_data_hex: "bea7b7acfbad0126b135104024a5f1733e705108f42d59668b05c0c50004c6b0\
                    517744132ae165a5349155bef0bb0cf2258fff99dfe1dbd914b938d775a36017",
    signature_hex: "4d8e53c2d5b3f2a7891753eb16ca993325bdb0d3cfc5be1093d0a18426f5ef85\
                    78cadc0fd4b5f4dd0d1ce0aefd15ab120b7a894d0eb094ffda4e6553cd1ed50d",
    public_key_multibase: "z6MkrJVnaZkeFzdQyMZu1cgjg7k1pZZ6pvBQ7XJPt4swbTQ2",
    key_multicodec: &[0xed, 0x01],
    proof_value:
        "z2YwC8z3ap7yx1nZYCg4L3j3ApHsF8kgPdSb5xoS1VR7vPG3F561B52hYnQF9iseabecm3ijx4K1FBTQsCZahKZme",
};

const ECDSA_RDFC_2019_P256: Vector = Vector {
    suite: VcCryptosuite::EcdsaRdfc2019,
    proof_config_nq: ECDSA_PROOF_CONFIG_NQ,
    proof_config_hash_hex: "3a8a522f689025727fb9d1f0fa99a618da023e8494ac74f51015d009d35abc2e",
    hash_data_hex: "3a8a522f689025727fb9d1f0fa99a618da023e8494ac74f51015d009d35abc2e\
                    517744132ae165a5349155bef0bb0cf2258fff99dfe1dbd914b938d775a36017",
    signature_hex: "1cb4290918ffb04a55ff7ae1e55e316a9990fda8eec67325eac7fcbf2ddf9dd2\
                    b06716a657e72b284c9604df3a172ecbf06a1a475b49ac807b1d9162df855636",
    public_key_multibase: "zDnaepBuvsQ8cpsWrVKw8fbpGpvPeNSjVPTWoq6cRqaYzBKVP",
    key_multicodec: &[0x80, 0x24],
    proof_value:
        "zaHXrr7AQdydBk3ahpCDpWbxfLokDqmCToYm2dyWvpcFVyWooC2he63w1f7UNQoAMKdhaRtcnaE2KTo5o5vTCcfw",
};

const VECTORS: [&Vector; 2] = [&EDDSA_RDFC_2022, &ECDSA_RDFC_2019_P256];

// ---------------------------------------------------------------------------
// Small transcoders, so the vectors above stay verbatim-quotable from the spec.
// (Deliberately no new dependency for two hex/base58 helpers.)
// ---------------------------------------------------------------------------

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// base58-btc (the `z` multibase). Schoolbook big-integer decode over a byte
/// vector — the inputs here are under 100 characters.
fn base58btc_decode(s: &str) -> Vec<u8> {
    const ALPHABET: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    let mut out: Vec<u8> = Vec::new();
    for c in s.bytes() {
        let mut carry = ALPHABET
            .iter()
            .position(|&a| a == c)
            .expect("base58-btc alphabet") as u32;
        for byte in out.iter_mut().rev() {
            let v = (*byte as u32) * 58 + carry;
            *byte = (v & 0xff) as u8;
            carry = v >> 8;
        }
        while carry > 0 {
            out.insert(0, (carry & 0xff) as u8);
            carry >>= 8;
        }
    }
    // Leading '1's are leading zero bytes.
    let zeros = s.bytes().take_while(|&c| c == b'1').count();
    let mut prefixed = vec![0u8; zeros];
    prefixed.extend_from_slice(&out);
    prefixed
}

/// Decode a `publicKeyMultibase`, asserting the multibase prefix and multicodec
/// the vector declares, and return the raw key bytes the bridge consumes
/// (Ed25519: 32B; P-256: 33B SEC1 compressed).
fn multibase_public_key(v: &Vector) -> Vec<u8> {
    let (base, rest) = v.public_key_multibase.split_at(1);
    assert_eq!(base, "z", "publicKeyMultibase must be base58-btc");
    let decoded = base58btc_decode(rest);
    assert_eq!(
        &decoded[..v.key_multicodec.len()],
        v.key_multicodec,
        "unexpected multicodec prefix for {}",
        v.suite.token()
    );
    decoded[v.key_multicodec.len()..].to_vec()
}

/// Decode a `proofValue` (multibase base58-btc, no multicodec prefix) into the
/// raw signature bytes the bridge consumes.
fn proof_value_signature(v: &Vector) -> Vec<u8> {
    let (base, rest) = v.proof_value.split_at(1);
    assert_eq!(base, "z", "proofValue must be base58-btc");
    base58btc_decode(rest)
}

/// Parse a vendored canonical N-Quads document back into triples. The vectors
/// are single default-graph documents, which is exactly the shape the bridge's
/// `&[Triple]` API takes.
fn triples(nquads: &str) -> Vec<Triple> {
    let g = Graph::load_str(nquads, "ntriples").expect("vendored vector must parse");
    graph_triples(&g).expect("vendored vector must materialize as triples")
}

// ---------------------------------------------------------------------------
// 1. The vendored bytes are intact.
// ---------------------------------------------------------------------------

/// Drift guard: each fixture file, **as stored**, must hash to the value the
/// spec publishes next to it. A stray edit (or a lost trailing newline — the
/// hash covers it) fails here rather than silently weakening everything below.
#[test]
fn vendored_vectors_hash_to_published_values() {
    assert_eq!(
        hex_encode(&Sha256::digest(CREDENTIAL_NQ.as_bytes())),
        CREDENTIAL_HASH_HEX,
        "credential.nq no longer matches its published SHA-256"
    );
    for v in VECTORS {
        assert_eq!(
            hex_encode(&Sha256::digest(v.proof_config_nq.as_bytes())),
            v.proof_config_hash_hex,
            "{} proof config no longer matches its published SHA-256",
            v.suite.token()
        );
    }
}

/// The published `hashData` is `proofConfigHash || documentHash` — pin the
/// concatenation ORDER against the vector itself, so a transposed
/// implementation cannot hide behind a self-consistent test.
#[test]
fn published_hash_data_is_proof_config_then_document() {
    for v in VECTORS {
        assert_eq!(
            v.hash_data_hex,
            format!("{}{}", v.proof_config_hash_hex, CREDENTIAL_HASH_HEX),
            "{} vector transcription is internally inconsistent",
            v.suite.token()
        );
    }
}

/// The spec's `proofValue` and its hex signature are two encodings of the same
/// bytes — checking both pins the base58-btc transcoder used below.
#[test]
fn proof_value_decodes_to_published_signature() {
    for v in VECTORS {
        assert_eq!(
            hex_encode(&proof_value_signature(v)),
            v.signature_hex,
            "{} proofValue must decode to the published signature",
            v.suite.token()
        );
        assert_eq!(proof_value_signature(v).len(), 64);
    }
}

// ---------------------------------------------------------------------------
// 2. sparq's canonicalization reproduces the W3C canonical documents.
// ---------------------------------------------------------------------------

/// **Byte-level interop, step one.** Round-trip each vendored canonical
/// document through sparq's RDFC-1.0 layer (parse → canonicalize → serialize)
/// and require the result to be byte-identical to the spec's document — same
/// term serialization, same code-point ordering, same `c14n0` blank-node
/// labelling, same trailing newline.
#[test]
fn sparq_canonicalization_reproduces_w3c_canonical_documents() {
    let cases: [(&str, &str); 3] = [
        ("credential", CREDENTIAL_NQ),
        ("eddsa-rdfc-2022 proof config", EDDSA_PROOF_CONFIG_NQ),
        ("ecdsa-rdfc-2019 proof config", ECDSA_PROOF_CONFIG_NQ),
    ];
    for (label, nq) in cases {
        let canonical = canonicalize_triples(&triples(nq))
            .expect("vendored vector must canonicalize")
            .to_nquads();
        assert_eq!(canonical, nq, "{} canonical N-Quads diverge from W3C", label);
    }
}

/// **Byte-level interop, step two.** The bridge's public transform must derive
/// the spec's published `hashData` from the vectors' triples — this is the
/// whole DI hashing step (canonicalize both documents, SHA-256 each, and
/// concatenate proof-config-first) checked against bytes sparq did not produce.
#[test]
fn hash_data_matches_published_vector() {
    let cred = triples(CREDENTIAL_NQ);
    for v in VECTORS {
        let hd = hash_data_from_triples(&cred, &triples(v.proof_config_nq))
            .expect("hashData must be derivable from the vector");
        assert_eq!(
            hex_encode(&hd),
            v.hash_data_hex,
            "{} hashData diverges from the published vector",
            v.suite.token()
        );
    }
}

/// Mutation guard for the test above: a `hashData` derived from the *other*
/// suite's proof config must NOT match, so the assertion is discriminating
/// rather than trivially true.
#[test]
fn hash_data_is_bound_to_its_own_proof_config() {
    let cred = triples(CREDENTIAL_NQ);
    let crossed = hash_data_from_triples(&cred, &triples(ECDSA_PROOF_CONFIG_NQ)).unwrap();
    assert_ne!(
        hex_encode(&crossed),
        EDDSA_RDFC_2022.hash_data_hex,
        "hashData must be bound to the proof config it was derived from"
    );
}

// ---------------------------------------------------------------------------
// 3. A third-party-issued W3C proof verifies through the bridge.
// ---------------------------------------------------------------------------

/// **The interop assertion.** The signature was produced by the W3C editors'
/// key over the W3C `hashData`; nothing in this repo signed it. It must verify
/// through the bridge for both in-scope suites, which is the property the
/// self-signed in-module tests structurally cannot witness.
#[test]
fn w3c_issued_proofs_verify_through_the_bridge() {
    let cred = triples(CREDENTIAL_NQ);
    for v in VECTORS {
        verify_source_proof(
            &cred,
            &triples(v.proof_config_nq),
            v.suite,
            &multibase_public_key(v),
            &proof_value_signature(v),
        )
        .unwrap_or_else(|e| panic!("published {} proof must verify: {}", v.suite.token(), e));
    }
}

/// Same vectors through the token entry point a caller reaches from a VC's
/// verbatim `proof.cryptosuite` value.
#[test]
fn w3c_issued_proofs_verify_by_cryptosuite_token() {
    let cred = triples(CREDENTIAL_NQ);
    for v in VECTORS {
        verify_source_proof_by_token(
            &cred,
            &triples(v.proof_config_nq),
            v.suite.token(),
            &multibase_public_key(v),
            &proof_value_signature(v),
        )
        .unwrap_or_else(|e| {
            panic!("published {} proof must verify by token: {}", v.suite.token(), e)
        });
    }
}

/// End to end: a published W3C credential goes through verify → re-commit →
/// provenance, and the registry entry records the source suite verbatim.
#[test]
fn w3c_vectors_ingest_and_record_source_cryptosuite() {
    let cred = triples(CREDENTIAL_NQ);
    let document = NamedNode::new("urn:uuid:58172aac-d8ba-11ed-83dd-0b3aef56cc33").unwrap();
    let salt = salt_from_bytes(&[3u8; 32]);
    for v in VECTORS {
        let ingested = ingest_verified_vc(
            document.clone(),
            &cred,
            &triples(v.proof_config_nq),
            v.suite,
            &multibase_public_key(v),
            &proof_value_signature(v),
            salt,
        )
        .unwrap_or_else(|e| panic!("published {} VC must ingest: {}", v.suite.token(), e));

        assert_eq!(ingested.source_cryptosuite, v.suite);
        assert_eq!(ingested.commitment.salt, salt);
        let entry = ingested.registry_entry();
        assert_eq!(entry.document, document);
        assert_eq!(entry.commitment, ingested.commitment.commitment);
        assert_eq!(entry.source_cryptosuite.as_deref(), Some(v.suite.token()));
    }
    // Both suites commit the SAME credential graph under the SAME salt, so the
    // re-commitment is a function of the content alone — the source suite is
    // provenance, not an input to `C(G)`.
    let commit_of = |v: &Vector| {
        ingest_verified_vc(
            document.clone(),
            &cred,
            &triples(v.proof_config_nq),
            v.suite,
            &multibase_public_key(v),
            &proof_value_signature(v),
            salt,
        )
        .unwrap()
        .commitment
        .commitment
    };
    assert_eq!(
        commit_of(&EDDSA_RDFC_2022),
        commit_of(&ECDSA_RDFC_2019_P256),
        "C(G) must depend on the credential graph, not the source suite"
    );
}

// ---------------------------------------------------------------------------
// 4. Mutations of the published vectors must go red.
// ---------------------------------------------------------------------------

/// Flipping a single bit of the published signature must reject. Without this
/// the positive assertions above could pass against a verifier that accepted
/// anything.
#[test]
fn single_bit_flip_in_published_signature_fails_closed() {
    let cred = triples(CREDENTIAL_NQ);
    for v in VECTORS {
        let mut sig = proof_value_signature(v);
        sig[0] ^= 0x01;
        assert!(
            matches!(
                verify_source_proof(
                    &cred,
                    &triples(v.proof_config_nq),
                    v.suite,
                    &multibase_public_key(v),
                    &sig,
                ),
                Err(VcBridgeError::VerificationFailed)
            ),
            "{}: a one-bit signature mutation must not verify",
            v.suite.token()
        );
    }
}

/// Dropping one credential triple changes the canonical document, hence the
/// `hashData` — the published proof must no longer verify, and the bridge must
/// refuse to commit the mutated graph.
#[test]
fn mutated_credential_fails_closed_and_never_commits() {
    for v in VECTORS {
        let mut mutated = triples(CREDENTIAL_NQ);
        mutated.pop().expect("vector has triples");
        let cfg = triples(v.proof_config_nq);
        let pk = multibase_public_key(v);
        let sig = proof_value_signature(v);
        assert!(
            matches!(
                verify_source_proof(&mutated, &cfg, v.suite, &pk, &sig),
                Err(VcBridgeError::VerificationFailed)
            ),
            "{}: a mutated credential must not verify",
            v.suite.token()
        );
        assert!(
            ingest_verified_vc(
                NamedNode::new("urn:uuid:58172aac-d8ba-11ed-83dd-0b3aef56cc33").unwrap(),
                &mutated,
                &cfg,
                v.suite,
                &pk,
                &sig,
                salt_from_bytes(&[3u8; 32]),
            )
            .is_err(),
            "{}: a mutated credential must never reach the commitment pipeline",
            v.suite.token()
        );
    }
}

/// Substituting the other suite's proof config (same credential, same key,
/// same signature) changes the `hashData` and must reject — the proof is bound
/// to the proof options it was created over.
#[test]
fn swapped_proof_config_fails_closed() {
    let cred = triples(CREDENTIAL_NQ);
    for (v, other) in [
        (&EDDSA_RDFC_2022, ECDSA_PROOF_CONFIG_NQ),
        (&ECDSA_RDFC_2019_P256, EDDSA_PROOF_CONFIG_NQ),
    ] {
        assert!(
            matches!(
                verify_source_proof(
                    &cred,
                    &triples(other),
                    v.suite,
                    &multibase_public_key(v),
                    &proof_value_signature(v),
                ),
                Err(VcBridgeError::VerificationFailed)
            ),
            "{}: a proof must not verify under a foreign proof config",
            v.suite.token()
        );
    }
}

/// Suite confusion: each vector's key and signature, presented under the OTHER
/// suite, must be rejected — never silently accepted, and never a panic.
#[test]
fn cross_suite_key_and_signature_fail_closed() {
    let cred = triples(CREDENTIAL_NQ);
    for (v, wrong_suite) in [
        (&EDDSA_RDFC_2022, VcCryptosuite::EcdsaRdfc2019),
        (&ECDSA_RDFC_2019_P256, VcCryptosuite::EddsaRdfc2022),
    ] {
        let err = verify_source_proof(
            &cred,
            &triples(v.proof_config_nq),
            wrong_suite,
            &multibase_public_key(v),
            &proof_value_signature(v),
        )
        .expect_err("cross-suite verification must fail closed");
        assert!(
            matches!(
                err,
                VcBridgeError::MalformedPublicKey
                    | VcBridgeError::MalformedSignature
                    | VcBridgeError::VerificationFailed
            ),
            "{} under {}: unexpected error {:?}",
            v.suite.token(),
            wrong_suite.token(),
            err
        );
    }
}
