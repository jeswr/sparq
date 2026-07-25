// [OPUS-4.8] written while Fable 5 unavailable — re-review when Fable returns.
//! Off-circuit W3C Verifiable-Credential ingest bridge (sq-9c5e, design
//! `research/zk-configurable-commitment-design.md` §5).
//!
//! # What this is
//! A **host-side, off-circuit** bridge that brings a W3C Data-Integrity VC into
//! sparq's commitment pipeline:
//!
//! ```text
//! VC (issuer-signed under eddsa-rdfc-2022 / ecdsa-rdfc-2019)
//!    │
//!    ├─ verify the VC's Data-Integrity proof OFF-circuit (host), under the named
//!    │   cryptosuite, against the issuer's published key.            [fail closed]
//!    ├─ RDFC10-canonicalise the credential graph (sparq already does — `canon`).
//!    ├─ commit under the sparq commitment pipeline (`commit::commit_triples`).
//!    └─ record provenance:
//!         zk:cryptosuite       = the sparq scheme   (what the query proof checks)
//!         zk:sourceCryptosuite = the VC's W3C suite  (provenance / back-compat)
//! ```
//!
//! It exists so the maintainer's requested **per-cryptosuite performance
//! comparison** (cost of ingest+re-commit per W3C suite) and the
//! **backwards-compatibility discussion** (which suites round-trip) can happen on
//! real data, WITHOUT claiming sparq verifies the VC's cryptographic proof inside
//! a query circuit.
//!
//! # The two distinct signatures (design §5.1) — DO NOT conflate
//! 1. **The VC's own Data-Integrity proof** — Ed25519 / ECDSA under a W3C
//!    cryptosuite. Checked HERE, off-circuit, at ingest **only**.
//! 2. **sparq's commitment signature** — `Poseidon2SchnorrV1` over `C(G)`
//!    ([`crate::sig`]) — what the in-circuit / verifier-side query proof binds to.
//!
//! These are **not the same** and the query proof does **NOT** re-verify the VC's
//! Ed25519/ECDSA proof in-circuit (§5.3). `zk:sourceCryptosuite` is **provenance,
//! not a re-verifiable in-proof property**.
//!
//! # Honest scope boundary (load-bearing)
//! - **In scope:** the off-circuit DI verification for the *whole-credential*
//!   `rdfc` suites `eddsa-rdfc-2022` (Ed25519) + `ecdsa-rdfc-2019` (ECDSA P-256),
//!   then re-commit + record provenance.
//! - **NOT in scope:** `bbs-2023` / `ecdsa-sd-2023` selective disclosure (the
//!   natural per-leaf match, but a real BBS verifier is not in-repo — deferred
//!   seam, design §5.3); **in-circuit** VC-proof verification (explicitly
//!   excluded — would be an overclaim); a full **JSON-LD processor** (this bridge
//!   operates over the credential's RDF graph — its canonical N-Quads — which is
//!   the form the DI suites hash; producing that RDF from a JSON-LD document is
//!   `oxjsonld`'s job, a separate ingest step). The host **does not fetch** the
//!   issuer key from a `did:`/URL — the caller supplies the resolved key bytes.
//! - **P-384 ECDSA** (`ecdsa-rdfc-2019` with a P-384 key, SHA-384 profile) is
//!   rejected ([`VcBridgeError::UnsupportedKeyCurve`]); only the P-256 / SHA-256
//!   profile is implemented. (Follow-up seam.)
//!
//! # Soundness posture
//! NOT externally audited (sq-qhy4). This module asserts **no** in-circuit or
//! query-soundness property; it is an *ingest-time* host verifier of the DI
//! hashing. A `false`/`Err` from any path is fail-closed — malformed key /
//! signature / cryptosuite bytes reject, never panic.
//!
//! OPT-IN: the whole module is behind the OFF-by-default `vc-bridge` cargo
//! feature, so the default build pulls no Ed25519/ECDSA/SHA-256 dependency.

use crate::commit::{commit_triples, CommitError, GraphCommitment};
use crate::field::Fr;
use crate::registry::RegistryEntry;
use oxrdf::{NamedNode, Triple};
use sha2::{Digest, Sha256};

/// The W3C Data-Integrity cryptosuites this bridge verifies off-circuit. Only the
/// whole-credential `rdfc` suites are in scope (design §5.4); the
/// selective-disclosure suites (`bbs-2023`, `ecdsa-sd-2023`) are a deferred seam.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VcCryptosuite {
    /// `eddsa-rdfc-2022` — Ed25519 over the RDFC10 canonical credential, SHA-256
    /// hash (W3C *Data Integrity EdDSA Cryptosuites v1.0*).
    EddsaRdfc2022,
    /// `ecdsa-rdfc-2019` — ECDSA P-256 over the RDFC10 canonical credential,
    /// SHA-256 hash (W3C *Data Integrity ECDSA Cryptosuites v1.0*, P-256 profile).
    EcdsaRdfc2019,
}

impl VcCryptosuite {
    /// The verbatim W3C `proof.cryptosuite` token (what `zk:sourceCryptosuite`
    /// records and what appears in a VC's `proof`).
    pub const fn token(self) -> &'static str {
        match self {
            VcCryptosuite::EddsaRdfc2022 => "eddsa-rdfc-2022",
            VcCryptosuite::EcdsaRdfc2019 => "ecdsa-rdfc-2019",
        }
    }

    /// Parse a W3C cryptosuite token. Fail-closed: an unknown / out-of-scope suite
    /// (incl. `bbs-2023`, `ecdsa-sd-2023`) returns `None`, never a default.
    pub fn from_token(token: &str) -> Option<Self> {
        match token {
            "eddsa-rdfc-2022" => Some(VcCryptosuite::EddsaRdfc2022),
            "ecdsa-rdfc-2019" => Some(VcCryptosuite::EcdsaRdfc2019),
            _ => None,
        }
    }
}

/// Off-circuit VC-bridge failures. Every variant is a **fail-closed reject** — a
/// relying/ingesting party feeds issuer-/prover-controlled bytes, so the bridge
/// never panics on malformed input.
#[derive(Debug)]
pub enum VcBridgeError {
    /// The `proof.cryptosuite` token is unknown or out of the bridge's scope
    /// (e.g. `bbs-2023`). Carries the offending token.
    UnsupportedCryptosuite(String),
    /// The issuer public key is for a curve this bridge does not implement (e.g.
    /// a P-384 ECDSA key — only the P-256 / SHA-256 profile is in scope).
    UnsupportedKeyCurve,
    /// The issuer public-key bytes do not parse as a valid key for the suite.
    MalformedPublicKey,
    /// The proof's signature bytes do not parse as a valid signature for the suite.
    MalformedSignature,
    /// The Data-Integrity proof did **not** verify against the canonical credential
    /// + proof-config under the issuer key (the off-circuit verification failed).
    VerificationFailed,
    /// Canonicalizing / committing the credential graph failed.
    Commit(CommitError),
    /// The credential graph is empty (nothing to commit).
    EmptyCredential,
}

impl std::fmt::Display for VcBridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // [OPUS-4.8] positional format args (CodeQL rust/unused-variable).
            VcBridgeError::UnsupportedCryptosuite(t) => {
                write!(f, "unsupported source cryptosuite: {}", t)
            }
            VcBridgeError::UnsupportedKeyCurve => {
                write!(
                    f,
                    "unsupported issuer key curve (only Ed25519 / ECDSA-P256 are in scope)"
                )
            }
            VcBridgeError::MalformedPublicKey => write!(f, "malformed issuer public key"),
            VcBridgeError::MalformedSignature => write!(f, "malformed proof signature"),
            VcBridgeError::VerificationFailed => {
                write!(f, "source VC Data-Integrity proof did not verify")
            }
            VcBridgeError::Commit(e) => write!(f, "re-commit failed: {}", e),
            VcBridgeError::EmptyCredential => write!(f, "credential graph is empty"),
        }
    }
}

impl std::error::Error for VcBridgeError {}

/// The domain-separated **hashData** the W3C `rdfc` cryptosuites sign: the SHA-256
/// of the RDFC10-canonical proof config, concatenated with the SHA-256 of the
/// RDFC10-canonical credential document (design §5.1; W3C DI *Create/Verify
/// Proof* algorithm). `proofConfigHash || transformedDocumentHash`.
///
/// Both the proof config and the credential are taken as already-RDFC10-canonical
/// **N-Quads** byte strings (the form sparq's `canon` layer produces — see
/// [`hash_data_from_triples`] for the triples-in convenience). Returns the 64-byte
/// concatenation that the Ed25519 / ECDSA signature is over.
fn hash_data(proof_config_canonical: &[u8], credential_canonical: &[u8]) -> [u8; 64] {
    let proof_hash = Sha256::digest(proof_config_canonical);
    let doc_hash = Sha256::digest(credential_canonical);
    let mut out = [0u8; 64];
    out[..32].copy_from_slice(&proof_hash);
    out[32..].copy_from_slice(&doc_hash);
    out
}

/// Build the signed **hashData** from the credential + proof-config **triples**:
/// RDFC10-canonicalise each (the DI suites canonicalise before hashing), then
/// `proofConfigHash || documentHash`. This is the off-circuit transform every
/// supported suite shares — the suites differ only in the SIGNATURE check over
/// this 64-byte hashData.
pub fn hash_data_from_triples(
    credential: &[Triple],
    proof_config: &[Triple],
) -> Result<[u8; 64], VcBridgeError> {
    let cred_canon = crate::canon::canonicalize_triples(credential)
        .map_err(|e| VcBridgeError::Commit(CommitError::Canon(e)))?;
    let proof_canon = crate::canon::canonicalize_triples(proof_config)
        .map_err(|e| VcBridgeError::Commit(CommitError::Canon(e)))?;
    Ok(hash_data(
        proof_canon.to_nquads().as_bytes(),
        cred_canon.to_nquads().as_bytes(),
    ))
}

/// Verify an Ed25519 (`eddsa-rdfc-2022`) signature over `hash_data`. Fail-closed:
/// a 32-byte public key and 64-byte signature are required; any malformed length
/// / point / scalar rejects. PUBLIC data only — no secret here.
fn verify_eddsa(pk_bytes: &[u8], sig_bytes: &[u8], hash_data: &[u8]) -> Result<(), VcBridgeError> {
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};
    let pk_arr: [u8; 32] = pk_bytes
        .try_into()
        .map_err(|_| VcBridgeError::MalformedPublicKey)?;
    let vk = VerifyingKey::from_bytes(&pk_arr).map_err(|_| VcBridgeError::MalformedPublicKey)?;
    let sig_arr: [u8; 64] = sig_bytes
        .try_into()
        .map_err(|_| VcBridgeError::MalformedSignature)?;
    let sig = Signature::from_bytes(&sig_arr);
    vk.verify(hash_data, &sig)
        .map_err(|_| VcBridgeError::VerificationFailed)
}

/// Verify an ECDSA-P256 (`ecdsa-rdfc-2019`, P-256 / SHA-256 profile) signature
/// over `hash_data`. Fail-closed: an SEC1 public key (compressed 33B or
/// uncompressed 65B) and a 64-byte fixed-width `(r,s)` signature are required.
/// PUBLIC data only.
///
/// NOTE: the W3C ECDSA suite's `hashData` is itself already the
/// `proofConfigHash || documentHash` concatenation; the ECDSA primitive then
/// hashes THAT with SHA-256 before the curve op (`Verifier::verify` over the raw
/// message does this digest internally), matching the suite's "hash the hashData"
/// step for the P-256 profile.
fn verify_ecdsa_p256(
    pk_bytes: &[u8],
    sig_bytes: &[u8],
    hash_data: &[u8],
) -> Result<(), VcBridgeError> {
    use p256::ecdsa::{signature::Verifier, Signature, VerifyingKey};
    // A P-384 key (SEC1 compressed 49B / uncompressed 97B) is the well-known
    // OUT-OF-SCOPE curve for the `ecdsa-rdfc-2019` SHA-384 profile — distinguish it
    // from genuinely malformed bytes so the caller gets the honest reason.
    if pk_bytes.len() == 49 || pk_bytes.len() == 97 {
        return Err(VcBridgeError::UnsupportedKeyCurve);
    }
    let vk =
        VerifyingKey::from_sec1_bytes(pk_bytes).map_err(|_| VcBridgeError::MalformedPublicKey)?;
    // Fixed-width `(r || s)` 64-byte signature (the DI ECDSA proofValue form).
    let sig = Signature::from_slice(sig_bytes).map_err(|_| VcBridgeError::MalformedSignature)?;
    vk.verify(hash_data, &sig)
        .map_err(|_| VcBridgeError::VerificationFailed)
}

/// **Off-circuit** verification of a source VC's W3C Data-Integrity proof
/// (design §5.2). RDFC10-canonicalises the credential + proof-config triples,
/// derives the suite's `hashData`, and verifies the signature under the
/// caller-resolved issuer public key for the named suite.
///
/// - `credential` — the credential graph's triples (the `proof` triple(s)
///   excluded, per the DI transform; the caller separates them).
/// - `proof_config` — the proof's triples WITHOUT `proofValue` (the proof
///   options the suite canonicalises).
/// - `suite` — the W3C cryptosuite.
/// - `issuer_pk` — the resolved issuer verification key bytes (Ed25519 32B; ECDSA
///   P-256 SEC1). The host does NOT dereference a `did:`/URL — supply the bytes.
/// - `signature` — the proof's signature bytes (`proofValue`, decoded to raw
///   bytes: Ed25519 64B; ECDSA-P256 64B fixed-width).
///
/// Fail-closed: returns `Err` on any malformed input or a non-verifying proof;
/// `Ok(())` ONLY on a cryptographically valid DI proof. Asserts no in-circuit /
/// query-soundness property (sq-qhy4).
pub fn verify_source_proof(
    credential: &[Triple],
    proof_config: &[Triple],
    suite: VcCryptosuite,
    issuer_pk: &[u8],
    signature: &[u8],
) -> Result<(), VcBridgeError> {
    let hd = hash_data_from_triples(credential, proof_config)?;
    match suite {
        VcCryptosuite::EddsaRdfc2022 => verify_eddsa(issuer_pk, signature, &hd),
        VcCryptosuite::EcdsaRdfc2019 => verify_ecdsa_p256(issuer_pk, signature, &hd),
    }
}

/// As [`verify_source_proof`] but the cryptosuite arrives as its W3C **token**
/// (the verbatim `proof.cryptosuite` value, e.g. `"eddsa-rdfc-2022"`), resolved
/// fail-closed. An unknown / out-of-scope token (incl. `bbs-2023`,
/// `ecdsa-sd-2023`) returns [`VcBridgeError::UnsupportedCryptosuite`] WITHOUT
/// touching the key/signature — the caller never has to pre-resolve the enum.
pub fn verify_source_proof_by_token(
    credential: &[Triple],
    proof_config: &[Triple],
    cryptosuite_token: &str,
    issuer_pk: &[u8],
    signature: &[u8],
) -> Result<(), VcBridgeError> {
    let suite = VcCryptosuite::from_token(cryptosuite_token)
        .ok_or_else(|| VcBridgeError::UnsupportedCryptosuite(cryptosuite_token.to_string()))?;
    verify_source_proof(credential, proof_config, suite, issuer_pk, signature)
}

/// A credential brought in through the bridge: its re-commitment under sparq's
/// pipeline, the source W3C cryptosuite it was verified under, and the document
/// IRI. Produced by [`ingest_verified_vc`] ONLY after the source DI proof verified
/// off-circuit, so an [`IngestedCredential`] is evidence the VC's proof checked at
/// ingest (it is NOT evidence of any in-circuit property).
#[derive(Debug, Clone)]
pub struct IngestedCredential {
    /// The credential document IRI (= its content-graph name in the registry).
    pub document: NamedNode,
    /// sparq's per-graph commitment `C(G)` over the RDFC10-canonical credential.
    pub commitment: GraphCommitment,
    /// The W3C cryptosuite the source VC's proof was verified under (provenance).
    pub source_cryptosuite: VcCryptosuite,
}

impl IngestedCredential {
    /// The (unattested) `<urn:sparq:zk>` registry entry for this bridged
    /// credential: it records `C(G)`, the per-graph salt, AND the
    /// `zk:sourceCryptosuite` provenance (the source W3C suite). The sparq issuer
    /// signature (`zk:commitmentSignature`) is added by the attestation path
    /// ([`RegistryEntry::issued`]), not here — the bridge attests the *source*
    /// proof, not sparq's commitment.
    pub fn registry_entry(&self) -> RegistryEntry {
        RegistryEntry::new(
            self.document.clone(),
            self.commitment.commitment,
            self.commitment.salt,
        )
        .with_source_cryptosuite(self.source_cryptosuite.token())
    }
}

/// The full bridge (design §5.2): **verify** the source VC's Data-Integrity proof
/// off-circuit, then **re-commit** the credential graph under sparq's pipeline and
/// record the source cryptosuite. The single fail-closed entry point.
///
/// - Verification runs FIRST: if the source proof does not verify, NO commitment
///   is produced (`Err`). A credential only enters the pipeline after its DI proof
///   checked — the bridge never re-commits an unverified VC.
/// - `salt` is the per-graph RDFC10 bnode salt the credential is committed under
///   (mint it via [`crate::ingest::SaltMint`] for global uniqueness).
///
/// Returns an [`IngestedCredential`] carrying `C(G)` + the source-suite provenance.
/// Asserts no in-circuit / query-soundness property (sq-qhy4); it is an ingest-time
/// host verification of the DI hashing.
pub fn ingest_verified_vc(
    document: NamedNode,
    credential: &[Triple],
    proof_config: &[Triple],
    suite: VcCryptosuite,
    issuer_pk: &[u8],
    signature: &[u8],
    salt: Fr,
) -> Result<IngestedCredential, VcBridgeError> {
    if credential.is_empty() {
        return Err(VcBridgeError::EmptyCredential);
    }
    // Verify the source DI proof FIRST — fail closed before committing anything.
    verify_source_proof(credential, proof_config, suite, issuer_pk, signature)?;
    // Only now re-commit under sparq's pipeline.
    let commitment = commit_triples(credential, salt).map_err(VcBridgeError::Commit)?;
    Ok(IngestedCredential {
        document,
        commitment,
        source_cryptosuite: suite,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::salt_from_bytes;
    use oxrdf::{Literal, NamedNode, Term};

    // A small credential graph (subject/predicate/object triples).
    fn credential() -> Vec<Triple> {
        vec![
            Triple::new(
                NamedNode::new("https://dmv.example/vc/lic-7").unwrap(),
                NamedNode::new("https://example.org/holder").unwrap(),
                NamedNode::new("https://people.example/alice").unwrap(),
            ),
            Triple::new(
                NamedNode::new("https://people.example/alice").unwrap(),
                NamedNode::new("http://schema.org/birthDate").unwrap(),
                Term::Literal(Literal::new_simple_literal("1990-01-01")),
            ),
        ]
    }

    // The proof options (proof config) the suite canonicalises — WITHOUT proofValue.
    fn proof_config(suite: VcCryptosuite) -> Vec<Triple> {
        vec![Triple::new(
            oxrdf::BlankNode::new("proof").unwrap(),
            NamedNode::new("https://w3id.org/security#cryptosuite").unwrap(),
            Term::Literal(Literal::new_simple_literal(suite.token())),
        )]
    }

    // --- suite token round-trips -------------------------------------------------

    #[test]
    fn cryptosuite_token_round_trips() {
        for suite in [VcCryptosuite::EddsaRdfc2022, VcCryptosuite::EcdsaRdfc2019] {
            assert_eq!(VcCryptosuite::from_token(suite.token()), Some(suite));
        }
    }

    #[test]
    fn unknown_and_deferred_suites_fail_closed() {
        // Deferred selective-disclosure suites and unknown tokens are rejected.
        for t in [
            "bbs-2023",
            "ecdsa-sd-2023",
            "ecdsa-rdfc-2019-p384",
            "nonsense",
            "",
        ] {
            assert_eq!(VcCryptosuite::from_token(t), None, "{} must not resolve", t);
        }
    }

    // --- hashData is deterministic + RDF-isomorphism invariant -------------------

    #[test]
    fn hash_data_is_canonicalization_invariant() {
        // Re-ordering the triples (RDF is a SET) must not change hashData, because
        // it is over the RDFC10-canonical form.
        let mut cred_a = credential();
        let cfg = proof_config(VcCryptosuite::EddsaRdfc2022);
        let hd1 = hash_data_from_triples(&cred_a, &cfg).unwrap();
        cred_a.reverse();
        let hd2 = hash_data_from_triples(&cred_a, &cfg).unwrap();
        assert_eq!(
            hd1, hd2,
            "hashData must be invariant under input triple order"
        );
    }

    #[test]
    fn hash_data_changes_with_credential_content() {
        let cfg = proof_config(VcCryptosuite::EddsaRdfc2022);
        let hd1 = hash_data_from_triples(&credential(), &cfg).unwrap();
        let mut tampered = credential();
        tampered[1] = Triple::new(
            NamedNode::new("https://people.example/alice").unwrap(),
            NamedNode::new("http://schema.org/birthDate").unwrap(),
            // changed value -> different commitment -> different hashData
            Term::Literal(Literal::new_simple_literal("2010-01-01")),
        );
        let hd2 = hash_data_from_triples(&tampered, &cfg).unwrap();
        assert_ne!(hd1, hd2, "a content change must change hashData");
    }

    // --- Ed25519 (eddsa-rdfc-2022): the REAL verify path -------------------------

    #[test]
    fn eddsa_real_proof_verifies_and_ingests() {
        use ed25519_dalek::{Signer, SigningKey};
        let cred = credential();
        let cfg = proof_config(VcCryptosuite::EddsaRdfc2022);
        let hd = hash_data_from_triples(&cred, &cfg).unwrap();

        // A real Ed25519 issuer key signs the REAL hashData (exercises the actual
        // verify path, not a mock).
        let sk = SigningKey::from_bytes(&[7u8; 32]);
        let pk = sk.verifying_key();
        let sig = sk.sign(&hd);

        // verify_source_proof accepts the genuine proof.
        verify_source_proof(
            &cred,
            &cfg,
            VcCryptosuite::EddsaRdfc2022,
            pk.as_bytes(),
            &sig.to_bytes(),
        )
        .expect("genuine eddsa-rdfc-2022 proof must verify");

        // ...and the full bridge re-commits + records provenance.
        let doc = NamedNode::new("https://dmv.example/vc/lic-7").unwrap();
        let salt = salt_from_bytes(&[1u8; 32]);
        let ing = ingest_verified_vc(
            doc.clone(),
            &cred,
            &cfg,
            VcCryptosuite::EddsaRdfc2022,
            pk.as_bytes(),
            &sig.to_bytes(),
            salt,
        )
        .expect("verified VC must ingest");
        assert_eq!(ing.source_cryptosuite, VcCryptosuite::EddsaRdfc2022);
        assert_eq!(ing.commitment.salt, salt);

        // The registry entry carries the source-cryptosuite provenance verbatim.
        let entry = ing.registry_entry();
        assert_eq!(entry.source_cryptosuite.as_deref(), Some("eddsa-rdfc-2022"));
        assert_eq!(entry.document, doc);
        assert_eq!(entry.commitment, ing.commitment.commitment);
    }

    #[test]
    fn eddsa_tampered_credential_fails_closed() {
        use ed25519_dalek::{Signer, SigningKey};
        let cred = credential();
        let cfg = proof_config(VcCryptosuite::EddsaRdfc2022);
        let hd = hash_data_from_triples(&cred, &cfg).unwrap();
        let sk = SigningKey::from_bytes(&[7u8; 32]);
        let pk = sk.verifying_key();
        let sig = sk.sign(&hd);

        // Tamper with the credential AFTER signing: the proof must NOT verify, and
        // the bridge must refuse to ingest (no commitment leaks through).
        let mut tampered = credential();
        tampered[1] = Triple::new(
            NamedNode::new("https://people.example/alice").unwrap(),
            NamedNode::new("http://schema.org/birthDate").unwrap(),
            Term::Literal(Literal::new_simple_literal("1980-01-01")),
        );
        assert!(matches!(
            verify_source_proof(
                &tampered,
                &cfg,
                VcCryptosuite::EddsaRdfc2022,
                pk.as_bytes(),
                &sig.to_bytes(),
            ),
            Err(VcBridgeError::VerificationFailed)
        ));
        assert!(ingest_verified_vc(
            NamedNode::new("https://dmv.example/vc/lic-7").unwrap(),
            &tampered,
            &cfg,
            VcCryptosuite::EddsaRdfc2022,
            pk.as_bytes(),
            &sig.to_bytes(),
            salt_from_bytes(&[1u8; 32]),
        )
        .is_err());
    }

    #[test]
    fn eddsa_wrong_key_fails_closed() {
        use ed25519_dalek::{Signer, SigningKey};
        let cred = credential();
        let cfg = proof_config(VcCryptosuite::EddsaRdfc2022);
        let hd = hash_data_from_triples(&cred, &cfg).unwrap();
        let sk = SigningKey::from_bytes(&[7u8; 32]);
        let sig = sk.sign(&hd);
        // A DIFFERENT issuer key must reject the signature.
        let other_pk = SigningKey::from_bytes(&[8u8; 32]).verifying_key();
        assert!(matches!(
            verify_source_proof(
                &cred,
                &cfg,
                VcCryptosuite::EddsaRdfc2022,
                other_pk.as_bytes(),
                &sig.to_bytes(),
            ),
            Err(VcBridgeError::VerificationFailed)
        ));
    }

    #[test]
    fn malformed_key_and_signature_fail_closed() {
        let cred = credential();
        let cfg = proof_config(VcCryptosuite::EddsaRdfc2022);
        // Wrong-length key.
        assert!(matches!(
            verify_source_proof(
                &cred,
                &cfg,
                VcCryptosuite::EddsaRdfc2022,
                &[0u8; 16],
                &[0u8; 64]
            ),
            Err(VcBridgeError::MalformedPublicKey)
        ));
        // Wrong-length signature (valid-length key bytes that parse, bad sig len).
        let sk = ed25519_dalek::SigningKey::from_bytes(&[7u8; 32]);
        assert!(matches!(
            verify_source_proof(
                &cred,
                &cfg,
                VcCryptosuite::EddsaRdfc2022,
                sk.verifying_key().as_bytes(),
                &[0u8; 10],
            ),
            Err(VcBridgeError::MalformedSignature)
        ));
    }

    // --- ECDSA P-256 (ecdsa-rdfc-2019): the REAL verify path ---------------------

    #[test]
    fn ecdsa_p256_real_proof_verifies_and_ingests() {
        use p256::ecdsa::{signature::Signer, Signature, SigningKey};
        let cred = credential();
        let cfg = proof_config(VcCryptosuite::EcdsaRdfc2019);
        let hd = hash_data_from_triples(&cred, &cfg).unwrap();

        let sk = SigningKey::from_slice(&[9u8; 32]).unwrap();
        let vk = sk.verifying_key();
        let sig: Signature = sk.sign(&hd);
        // SEC1 compressed public-key bytes (the DI ECDSA verification-key form).
        let pk_sec1 = vk.to_encoded_point(true).as_bytes().to_vec();

        verify_source_proof(
            &cred,
            &cfg,
            VcCryptosuite::EcdsaRdfc2019,
            &pk_sec1,
            &sig.to_bytes(),
        )
        .expect("genuine ecdsa-rdfc-2019 proof must verify");

        let ing = ingest_verified_vc(
            NamedNode::new("https://dmv.example/vc/lic-7").unwrap(),
            &cred,
            &cfg,
            VcCryptosuite::EcdsaRdfc2019,
            &pk_sec1,
            &sig.to_bytes(),
            salt_from_bytes(&[2u8; 32]),
        )
        .expect("verified P-256 VC must ingest");
        assert_eq!(
            ing.registry_entry().source_cryptosuite.as_deref(),
            Some("ecdsa-rdfc-2019")
        );
    }

    #[test]
    fn ecdsa_p256_tampered_fails_closed() {
        use p256::ecdsa::{signature::Signer, Signature, SigningKey};
        let cred = credential();
        let cfg = proof_config(VcCryptosuite::EcdsaRdfc2019);
        let hd = hash_data_from_triples(&cred, &cfg).unwrap();
        let sk = SigningKey::from_slice(&[9u8; 32]).unwrap();
        let vk = sk.verifying_key();
        let sig: Signature = sk.sign(&hd);

        let mut tampered = credential();
        tampered.truncate(1); // drop a triple -> different canonical form
        assert!(matches!(
            verify_source_proof(
                &tampered,
                &cfg,
                VcCryptosuite::EcdsaRdfc2019,
                vk.to_encoded_point(true).as_bytes(),
                &sig.to_bytes(),
            ),
            Err(VcBridgeError::VerificationFailed)
        ));
    }

    #[test]
    fn verify_by_token_resolves_and_rejects_unknown() {
        use ed25519_dalek::{Signer, SigningKey};
        let cred = credential();
        let cfg = proof_config(VcCryptosuite::EddsaRdfc2022);
        let hd = hash_data_from_triples(&cred, &cfg).unwrap();
        let sk = SigningKey::from_bytes(&[7u8; 32]);
        let sig = sk.sign(&hd);
        // A genuine proof verifies through the token entry point.
        verify_source_proof_by_token(
            &cred,
            &cfg,
            "eddsa-rdfc-2022",
            sk.verifying_key().as_bytes(),
            &sig.to_bytes(),
        )
        .expect("token path must verify a genuine proof");
        // An out-of-scope / unknown token fails closed WITHOUT touching the key.
        assert!(matches!(
            verify_source_proof_by_token(&cred, &cfg, "bbs-2023", &[0u8; 32], &[0u8; 64]),
            Err(VcBridgeError::UnsupportedCryptosuite(t)) if t == "bbs-2023"
        ));
    }

    #[test]
    fn ecdsa_p384_key_size_is_unsupported_curve() {
        let cred = credential();
        let cfg = proof_config(VcCryptosuite::EcdsaRdfc2019);
        // A P-384 SEC1 compressed key is 49 bytes — known curve, out of scope.
        assert!(matches!(
            verify_source_proof(
                &cred,
                &cfg,
                VcCryptosuite::EcdsaRdfc2019,
                &[0x02u8; 49],
                &[0u8; 64],
            ),
            Err(VcBridgeError::UnsupportedKeyCurve)
        ));
    }

    #[test]
    fn empty_credential_fails_closed() {
        let cfg = proof_config(VcCryptosuite::EddsaRdfc2022);
        assert!(matches!(
            ingest_verified_vc(
                NamedNode::new("https://dmv.example/vc/lic-7").unwrap(),
                &[],
                &cfg,
                VcCryptosuite::EddsaRdfc2022,
                &[0u8; 32],
                &[0u8; 64],
                salt_from_bytes(&[1u8; 32]),
            ),
            Err(VcBridgeError::EmptyCredential)
        ));
    }
}
