// [OPUS-4.8] written while Fable 5 unavailable — re-review when Fable returns.
//! End-to-end + tamper tests for sparq-zk-compose.
//!
//! Fast tests (default): manifest serde round-trip, structural verification,
//! and tamper/negative cases that the structural gate must reject WITHOUT
//! invoking bb. They also exercise nargo witness generation (cheap) on a small
//! filter circuit to prove the relation is satisfiable end-to-end in the
//! engine seam.
//!
//! Slow tests (`#[ignore]`): full bb prove -> verify, plus a bb tamper case.
//! Run with `cargo test -p sparq-zk-compose -- --ignored`.
//!
//! At least one NON-ignored test runs a real (small) full prove+verify:
//! `full_prove_verify_filter_int_d1`. It is gated behind the `nargo`/`bb`
//! toolchain being on PATH (skips cleanly if absent, like sparq-zk's live
//! cross-check), so default `cargo test` stays green in minimal CI while still
//! exercising the real cryptographic path when the toolchain is present.

use oxrdf::{Literal, NamedNode, NamedOrBlankNode, Term, Triple};
use sparq_zk::commit::commit_triples;
use sparq_zk::encode::salt_from_bytes;
use sparq_zk_compose::build::{
    build_filter_int, build_scan, encode_int_literal, Pattern, Slot,
};
use sparq_zk_compose::driver::CircuitProver;
use sparq_zk_compose::manifest::{
    BindingEdge, BindingMode, CircuitId, CommitmentAttestation, EntailmentRegime, FieldHex,
    FilterOp, ProofInputs, ProofManifest, RevocationStatus, SubProof,
};
use sparq_zk_compose::toml::prover_toml_for;
use sparq_zk_compose::verifier::{
    encode_artifacts, verify_manifest, verify_manifest_structure, CheckError, KeySet,
};
use sparq_zk::field::Fr;
use sparq_zk::sig::{public_key_to_hex, SecretKey, SignatureScheme};

// [OPUS-4.8] audit #3 codex #1: the EXTERNAL relying-party trust anchor K. Tests
// build it from the issuer keys the *relying party* decides to trust — NOT from
// the manifest. `trusted_k(&sk)` trusts exactly that one issuer; `empty_k()`
// trusts none (the fail-closed default). The #3 negative tests pass a K that
// deliberately does/doesn't contain the signing key to exercise the gate.

/// External trust anchor containing exactly the public key of `sk`.
fn trusted_k(sk: &SecretKey) -> KeySet {
    KeySet::from_hex_keys([public_key_to_hex(&sk.public_key())])
}

/// External trust anchor trusting no issuer (fail-closed).
fn empty_k() -> KeySet {
    KeySet::empty()
}

// --- issuer-signature test plumbing (audit #3) ----------------------------
// [OPUS-4.8] A fixed deterministic test issuer key + helpers that attest a
// committed graph, so every manifest carrying a scan can present a valid,
// in-key-set issuer attestation over its commitment(s). Tests that probe a
// DIFFERENT gate (#1/#2/#5/#6/#10) attach a valid attestation so they reach the
// gate under test; the #3-specific tests deliberately omit/forge it.

fn test_issuer_sk(seed: u64) -> SecretKey {
    SecretKey::from_seed(seed)
}

/// Sign a commitment field element with `sk`, returning an attestation.
fn attest(commitment: Fr, sk: &SecretKey) -> CommitmentAttestation {
    CommitmentAttestation {
        commitment: FieldHex::from_field(&commitment),
        issuer_public_key: public_key_to_hex(&sk.public_key()),
        signature: sk.sign_commitment(&commitment), // [OPUS-4.8] codex #4: deterministic nonce
        cryptosuite: SignatureScheme::Poseidon2SchnorrV1.cryptosuite_iri().to_string(),
        salt: None, // [OPUS-4.8] audit #9: salt-unbound legacy attestation
    }
}

/// [OPUS-4.8] audit #9: a SALT-BOUND attestation — the issuer signs
/// `(commitment, salt)`, and the manifest discloses `salt`, so the verifier
/// recomputes the salt-bound message and rejects salt reuse across graphs.
fn attest_with_salt(commitment: Fr, salt: Fr, sk: &SecretKey) -> CommitmentAttestation {
    CommitmentAttestation {
        commitment: FieldHex::from_field(&commitment),
        issuer_public_key: public_key_to_hex(&sk.public_key()),
        signature: sk.sign_commitment_with_salt(&commitment, &salt),
        cryptosuite: SignatureScheme::Poseidon2SchnorrV1.cryptosuite_iri().to_string(),
        salt: Some(FieldHex::from_field(&salt)),
    }
}

/// Collect the per-graph commitments of every scan sub-proof in a manifest, as
/// field elements (for attesting them).
fn scan_commitments(m: &ProofManifest) -> Vec<Fr> {
    let mut out = Vec::new();
    for sp in &m.sub_proofs {
        if let ProofInputs::Scan { commitments, .. } = &sp.inputs {
            for c in commitments {
                if let Some(f) = c.to_field() {
                    out.push(f);
                }
            }
        }
    }
    out
}

/// Attach valid, in-K, SALT-BOUND issuer attestations for EVERY scan commitment
/// in `m` under a fixed test issuer key + the per-graph `salt` they were
/// committed under, and disclose that key in K. After this the manifest passes
/// the #3 + #9 attestation gate, so a test can reach whatever OTHER gate it is
/// probing.
///
/// [OPUS-4.8] codex 2221 HIGH: a scan-covering attestation MUST be salt-bound,
/// so this signs `commitment_message_with_salt(C(G), salt)`. The single-graph
/// fixtures that call this all commit under one fixed salt, passed here. (A
/// distinct salt per graph would defeat the salt-uniqueness check; the
/// multi-graph fixtures use `attest_with_salt` per commitment directly.)
fn attest_all(m: &mut ProofManifest, sk: &SecretKey, salt: Fr) {
    let pk_hex = public_key_to_hex(&sk.public_key());
    let mut seen = std::collections::BTreeSet::new();
    for c in scan_commitments(m) {
        let key = sparq_zk::field::field_to_hex(&c);
        if seen.insert(key) {
            m.commitment_attestations.push(attest_with_salt(c, salt, sk));
        }
    }
    if !m.key_set.contains(&pk_hex) {
        m.key_set.push(pk_hex);
    }
}

// --- small credential-style graph helpers --------------------------------

fn iri(s: &str) -> NamedNode {
    NamedNode::new(s).unwrap()
}

fn int_lit(v: u64) -> Term {
    Term::Literal(Literal::new_typed_literal(
        v.to_string(),
        iri("http://www.w3.org/2001/XMLSchema#integer"),
    ))
}

/// A tiny named-graph credential: subject `alice` with an age and a role.
fn credential_graph() -> Vec<Triple> {
    let alice = NamedOrBlankNode::NamedNode(iri("http://ex/alice"));
    vec![
        Triple::new(alice.clone(), iri("http://ex/age"), int_lit(25)),
        Triple::new(
            alice,
            iri("http://ex/role"),
            Term::NamedNode(iri("http://ex/admin")),
        ),
    ]
}

fn toolchain_available() -> bool {
    fn on_path(tool: &str) -> bool {
        std::process::Command::new(tool)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    on_path("nargo") && on_path("bb")
}

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("sparq_zk_compose_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

// --- manifest serde -------------------------------------------------------

fn sample_manifest() -> ProofManifest {
    let salt = salt_from_bytes(&[7u8; 32]);
    let commit = commit_triples(&credential_graph(), salt).unwrap();
    let pattern = Pattern {
        s: Slot::Var,
        p: Slot::Const(Term::NamedNode(iri("http://ex/age"))),
        o: Slot::Var,
    };
    let scan = build_scan(&[commit], &pattern).expect("scan builds");
    let operand_enc = match &scan.inputs {
        ProofInputs::Scan { rows, .. } => rows[0][2].clone(),
        _ => unreachable!(),
    };
    let (filter, _digits) =
        build_filter_int(operand_enc, 25, FilterOp::Ge, 18, true).expect("filter builds");

    let mut m = ProofManifest {
        r#type: "urn:sparq:zk:ProofManifest".into(),
        query: "SELECT ?s ?o WHERE { ?s <http://ex/age> ?o }".into(),
        issuers: vec!["did:key:zSampleIssuer".into()],
        key_set: vec![],
        commitment_attestations: vec![],
        attributions: vec![vec![0]],
        join_obligations: vec![],
        entailment_regime: EntailmentRegime::Simple,
        binding: BindingMode::Challenge {
            challenge: FieldHex("0x2a".into()),
        },
        revocation: Some(RevocationStatus {
            status_list: "http://ex/status/1".into(),
            index: 3,
        }),
        sub_proofs: vec![
            SubProof { inputs: scan.inputs, proof_hex: String::new() },
            SubProof { inputs: filter, proof_hex: String::new() },
        ],
        binding_edges: vec![BindingEdge {
            from_proof: 0,
            from_row: 0,
            from_slot: 2,
            to_proof: 1,
        }],
    };
    // [OPUS-4.8] audit #3/#9: attest the scan commitment (salt-bound) so the
    // sample manifest passes the issuer-signature gate (tests that probe #3
    // strip/forge this).
    attest_all(&mut m, &test_issuer_sk(1), salt);
    m
}

#[test]
fn manifest_serde_round_trip() {
    let m = sample_manifest();
    let json = m.to_json();
    let back = ProofManifest::from_json(&json).expect("round-trips");
    assert_eq!(m, back);
    // Spot-check the public shape is present.
    assert!(json.contains("did:key:zSampleIssuer"));
    assert!(json.contains("\"entailment_regime\": \"simple\""));
}

// --- structural verification (fast) --------------------------------------

#[test]
fn structure_accepts_well_formed_manifest() {
    let m = sample_manifest();
    verify_manifest_structure(&m, &trusted_k(&test_issuer_sk(1))).expect("structure verifies");
}

#[test]
fn structure_rejects_inconsistent_binding_edge() {
    let mut m = sample_manifest();
    // Tamper: point the filter's operand at a different encoding than the
    // scanned column the binding edge claims.
    if let ProofInputs::FilterInt { operand_enc, .. } = &mut m.sub_proofs[1].inputs {
        *operand_enc = FieldHex("0xdeadbeef".into());
    }
    match verify_manifest_structure(&m, &trusted_k(&test_issuer_sk(1))) {
        Err(CheckError::BindingInconsistent { edge: 0 }) => {}
        other => panic!("expected BindingInconsistent, got {other:?}"),
    }
}

#[test]
fn structure_rejects_arity_mismatch() {
    let mut m = sample_manifest();
    // The query has 1 BGP pattern; declare 2 attributions.
    m.attributions = vec![vec![0], vec![0]];
    assert!(verify_manifest_structure(&m, &trusted_k(&test_issuer_sk(1))).is_err());
}

#[test]
fn structure_rejects_circuit_id_mismatch() {
    let mut m = sample_manifest();
    // Swap the declared scan id's k to a value its commitments don't support.
    if let ProofInputs::Scan { id, .. } = &mut m.sub_proofs[0].inputs {
        *id = CircuitId::Scan { k: 2, n: 16, r: 4 };
    }
    match verify_manifest_structure(&m, &trusted_k(&test_issuer_sk(1))) {
        Err(CheckError::CircuitIdMismatch { proof: 0, .. }) => {}
        other => panic!("expected CircuitIdMismatch, got {other:?}"),
    }
}

#[test]
fn structure_rejects_cross_graph_bnode_join() {
    // Two patterns sharing a variable, each attributable to a different graph,
    // with no declared non-bnode obligation: the sparq-zk Q6 guard must fire.
    let mut m = sample_manifest();
    m.query =
        "SELECT ?x WHERE { ?x <http://ex/age> ?a . ?x <http://ex/role> ?r }".into();
    m.attributions = vec![vec![0], vec![1]];
    m.join_obligations = vec![]; // omit the obligation on ?x
    assert!(matches!(
        verify_manifest_structure(&m, &trusted_k(&test_issuer_sk(1))),
        Err(CheckError::Sparqzk(_))
    ));
}

// --- nargo witness generation (cheap relation check) ---------------------

#[test]
fn witness_gen_filter_int_satisfiable() {
    if !toolchain_available() {
        eprintln!("nargo/bb absent; skipping witness-gen test");
        return;
    }
    // [OPUS-4.8] tag-isolated witness gen: previously this relied on targeting a
    // member (filter_int_d4) no other concurrent test touches, because the
    // witness path was shared per package. The driver now isolates the
    // prover-input toml + witness by tag, so concurrency is safe regardless of
    // member overlap (roborev job 2180). 1234 >= 18 is true.
    let operand_enc = encode_int_literal(1234);
    let (filter, digits) =
        build_filter_int(operand_enc, 1234, FilterOp::Ge, 18, true).unwrap();
    let (id, toml) = prover_toml_for(
        &filter,
        &FieldHex("0x2a".into()),
        &[],
        &[],
        &digits,
    );
    let prover = CircuitProver::from_crate_root();
    prover.compile(&id).expect("compiles");
    prover
        .gen_witness_tagged(&id, &toml, "wg_filter_d4")
        .expect("witness satisfiable");
}

#[test]
fn witness_gen_filter_int_rejects_false_verdict() {
    if !toolchain_available() {
        eprintln!("nargo/bb absent; skipping");
        return;
    }
    // 17 >= 18 is false; claim true -> witness generation must fail.
    // [OPUS-4.8] tag-isolated: this targets filter_int_d1, shared with the
    // forge/full-prove tests, so it MUST NOT use the shared witness path.
    let operand_enc = encode_int_literal(17);
    let (filter, digits) =
        build_filter_int(operand_enc, 17, FilterOp::Ge, 18, true).unwrap();
    let (id, toml) =
        prover_toml_for(&filter, &FieldHex("0x2a".into()), &[], &[], &digits);
    let prover = CircuitProver::from_crate_root();
    prover.compile(&id).unwrap();
    assert!(
        prover.gen_witness_tagged(&id, &toml, "wg_filter_d1_false").is_err(),
        "a lying verdict must be unsatisfiable"
    );
}

// [OPUS-4.8] roborev codex 2207: end-to-end `!=` (FilterCmp/FilterOp::Ne).
// The verifier-side parser now binds SPARQL `?v != c` (spargebra
// `Not(Equal(..))`) to `Ne`; these confirm the `Ne` op code drives the real
// `filter_int` circuit's `!eq` verdict branch through the engine seam.
#[test]
fn witness_gen_filter_int_ne_satisfiable() {
    if !toolchain_available() {
        eprintln!("nargo/bb absent; skipping witness-gen Ne test");
        return;
    }
    // 30 != 18 is true. Tag-isolated witness gen (filter_int_d2, value "30").
    let operand_enc = encode_int_literal(30);
    let (filter, digits) =
        build_filter_int(operand_enc, 30, FilterOp::Ne, 18, true).unwrap();
    let (id, toml) =
        prover_toml_for(&filter, &FieldHex("0x2a".into()), &[], &[], &digits);
    let prover = CircuitProver::from_crate_root();
    prover.compile(&id).expect("compiles");
    prover
        .gen_witness_tagged(&id, &toml, "wg_filter_ne_d2")
        .expect("30 != 18 witness satisfiable");
}

#[test]
fn witness_gen_filter_int_ne_rejects_false_verdict() {
    if !toolchain_available() {
        eprintln!("nargo/bb absent; skipping");
        return;
    }
    // Forge: 18 != 18 is FALSE; claim the `!=` verdict is true -> witness
    // generation must fail (the circuit's `!eq` branch cannot be satisfied).
    // Tag-isolated (filter_int_d2, value "18"), distinct from the satisfiable
    // case's tag so concurrent runs are independent.
    let operand_enc = encode_int_literal(18);
    let (filter, digits) =
        build_filter_int(operand_enc, 18, FilterOp::Ne, 18, true).unwrap();
    let (id, toml) =
        prover_toml_for(&filter, &FieldHex("0x2a".into()), &[], &[], &digits);
    let prover = CircuitProver::from_crate_root();
    prover.compile(&id).unwrap();
    assert!(
        prover.gen_witness_tagged(&id, &toml, "wg_filter_ne_d2_false").is_err(),
        "a lying `!=` verdict (18 != 18 claimed true) must be unsatisfiable"
    );
}

#[test]
fn witness_gen_scan_satisfiable() {
    if !toolchain_available() {
        eprintln!("nargo/bb absent; skipping");
        return;
    }
    let salt = salt_from_bytes(&[7u8; 32]);
    let commit = commit_triples(&credential_graph(), salt).unwrap();
    let pattern = Pattern {
        s: Slot::Var,
        p: Slot::Const(Term::NamedNode(iri("http://ex/age"))),
        o: Slot::Var,
    };
    let scan = build_scan(&[commit], &pattern).expect("scan builds");
    let (id, toml) = prover_toml_for(
        &scan.inputs,
        &FieldHex("0x2a".into()),
        &scan.witness.counts,
        &scan.witness.enc,
        &[],
    );
    let prover = CircuitProver::from_crate_root();
    prover.compile(&id).expect("compiles");
    // [OPUS-4.8] tag-isolated witness gen.
    prover
        .gen_witness_tagged(&id, &toml, "wg_scan")
        .expect("scan witness satisfiable");
}

// --- full bb prove -> verify ---------------------------------------------

/// NON-ignored full prove+verify on the smallest member (gated on toolchain).
#[test]
fn full_prove_verify_filter_int_d1() {
    if !toolchain_available() {
        eprintln!("nargo/bb absent; skipping full prove+verify");
        return;
    }
    let operand_enc = encode_int_literal(5);
    let (filter, digits) =
        build_filter_int(operand_enc, 5, FilterOp::Lt, 10, true).unwrap();
    let (id, toml) =
        prover_toml_for(&filter, &FieldHex("0x2a".into()), &[], &[], &digits);
    assert_eq!(id, CircuitId::FilterInt { d: 1 });

    let prover = CircuitProver::from_crate_root();
    let out = scratch("full_d1");
    // [OPUS-4.8] tag-isolated prove (shares filter_int_d1 with the forge tests).
    let art = prover.prove_in(&id, &toml, &out, "full_d1").expect("prove succeeds");
    assert!(!art.proof.is_empty());
    let ok = prover.verify(&art, &out.join("verify")).expect("verify runs");
    assert!(ok, "valid proof must verify");

    // Tamper: flip a proof byte -> bb must reject.
    let mut bad = art.clone();
    let mid = bad.proof.len() / 2;
    bad.proof[mid] ^= 0xff;
    let rejected = prover
        .verify(&bad, &out.join("verify_bad"))
        .expect("verify runs");
    assert!(!rejected, "tampered proof must be rejected");
}

/// [OPUS-4.8] roborev codex 2207: NON-ignored full prove+verify of a `Ne`
/// FILTER on the smallest member. Confirms `FilterOp::Ne` drives an honest
/// proof that bb actually verifies (the "binds + verifies" positive for the
/// newly-bound `!=` fragment), and that a flipped proof byte is rejected.
/// Tag-isolated (shares filter_int_d1 with the other full-prove/forge tests).
#[test]
fn full_prove_verify_filter_int_ne_d1() {
    if !toolchain_available() {
        eprintln!("nargo/bb absent; skipping full prove+verify (Ne)");
        return;
    }
    // 5 != 9 is true.
    let operand_enc = encode_int_literal(5);
    let (filter, digits) =
        build_filter_int(operand_enc, 5, FilterOp::Ne, 9, true).unwrap();
    let (id, toml) =
        prover_toml_for(&filter, &FieldHex("0x2a".into()), &[], &[], &digits);
    assert_eq!(id, CircuitId::FilterInt { d: 1 });

    let prover = CircuitProver::from_crate_root();
    let out = scratch("full_ne_d1");
    let art = prover.prove_in(&id, &toml, &out, "full_ne_d1").expect("prove succeeds");
    assert!(!art.proof.is_empty());
    let ok = prover.verify(&art, &out.join("verify")).expect("verify runs");
    assert!(ok, "valid `!=` proof must verify");

    // Tamper: flip a proof byte -> bb must reject.
    let mut bad = art.clone();
    let mid = bad.proof.len() / 2;
    bad.proof[mid] ^= 0xff;
    let rejected = prover
        .verify(&bad, &out.join("verify_bad"))
        .expect("verify runs");
    assert!(!rejected, "tampered `!=` proof must be rejected");
}

/// Full manifest prove -> verify with the artifact-bundling path.
#[test]
#[ignore = "slow: full bb prove of a scan member"]
fn full_manifest_prove_verify_scan() {
    if !toolchain_available() {
        eprintln!("nargo/bb absent; skipping");
        return;
    }
    let salt = salt_from_bytes(&[7u8; 32]);
    let commit = commit_triples(&credential_graph(), salt).unwrap();
    let pattern = Pattern {
        s: Slot::Var,
        p: Slot::Const(Term::NamedNode(iri("http://ex/age"))),
        o: Slot::Var,
    };
    let scan = build_scan(&[commit], &pattern).unwrap();
    let challenge = FieldHex("0x2a".into());
    let (id, toml) = prover_toml_for(
        &scan.inputs,
        &challenge,
        &scan.witness.counts,
        &scan.witness.enc,
        &[],
    );
    let prover = CircuitProver::from_crate_root();
    let out = scratch("manifest_scan");
    // [OPUS-4.8] tag-isolated prove.
    let art = prover.prove_in(&id, &toml, &out, "manifest_scan").unwrap();

    let mut manifest = ProofManifest {
        r#type: "urn:sparq:zk:ProofManifest".into(),
        query: "SELECT ?s ?o WHERE { ?s <http://ex/age> ?o }".into(),
        issuers: vec!["did:key:zSampleIssuer".into()],
        key_set: vec![],
        commitment_attestations: vec![],
        attributions: vec![vec![0]],
        join_obligations: vec![],
        entailment_regime: EntailmentRegime::Simple,
        binding: BindingMode::Challenge { challenge },
        revocation: None,
        sub_proofs: vec![SubProof {
            inputs: scan.inputs,
            proof_hex: encode_artifacts(&art),
        }],
        binding_edges: vec![],
    };
    attest_all(&mut manifest, &test_issuer_sk(1), salt); // [OPUS-4.8] audit #3/#9 (salt-bound)
    verify_manifest(&manifest, &prover, &scratch("manifest_verify"), &trusted_k(&test_issuer_sk(1)))
        .expect("manifest verifies");
}

// --- forge-and-verify NEGATIVE tests (audit #1/#2) ------------------------
//
// [OPUS-4.8] These are the binding-soundness tests the audit + test-bench
// design (§5.1) require: construct a GENUINE bb proof of statement A, then lie
// in the manifest (declare statement B, or bundle a non-canonical vk) while
// leaving proof_hex pointing at the real proof, and assert verify_manifest
// returns Err. Before fixes #1/#2 every one of these returned Ok(()). They are
// toolchain-gated (skip cleanly if nargo/bb absent), like the happy-path full
// prove+verify.

use sparq_zk_compose::driver::ProofArtifacts;

/// Build a real filter_int_d1 proof over (operand=value, op, bound, expected)
/// and the matching honest ProofInputs. Returns (inputs, artifacts).
fn honest_filter_d1(
    value: u64,
    op: FilterOp,
    bound: u64,
    expected: bool,
    challenge: &FieldHex,
    prover: &CircuitProver,
    tag: &str,
) -> (ProofInputs, ProofArtifacts) {
    let operand_enc = encode_int_literal(value);
    let (filter, digits) =
        build_filter_int(operand_enc, value, op, bound, expected).unwrap();
    let (id, toml) = prover_toml_for(&filter, challenge, &[], &[], &digits);
    assert_eq!(id, CircuitId::FilterInt { d: 1 });
    let out = scratch(tag);
    // [OPUS-4.8] tag-isolated prove: every forge test targets filter_int_d1, so
    // under default parallel `cargo test` they'd otherwise race on the shared
    // Prover.toml / witness and prove the WRONG statement (roborev job 2180).
    let art = prover.prove_in(&id, &toml, &out, tag).expect("prove succeeds");
    (filter, art)
}

/// Prove a REAL scan over `{ ?s <http://ex/age> ?o }` on the credential graph
/// (one active age row). The query-correctness binding (#10) now requires every
/// query BGP pattern to have a backing scan sub-proof, so the #1/#2 FILTER
/// crypto-forge tests below carry this honest scan at index 0 and forge only the
/// (unreferenced) FILTER sub-proof at index 1 — stage 2b passes on the scan,
/// stage 2c is a no-op (the query has no FILTER), and stage 3 still byte-compares
/// the forged FILTER => the original #1/#2 assertions are preserved. Returns
/// (scan inputs, scan proof_hex).
fn honest_age_scan(
    challenge: &FieldHex,
    prover: &CircuitProver,
    tag: &str,
) -> (ProofInputs, String) {
    let salt = salt_from_bytes(&[7u8; 32]);
    let commit = commit_triples(&credential_graph(), salt).unwrap();
    let pattern = Pattern {
        s: Slot::Var,
        p: Slot::Const(Term::NamedNode(iri("http://ex/age"))),
        o: Slot::Var,
    };
    let scan = build_scan(&[commit], &pattern).expect("scan builds");
    let (id, toml) = prover_toml_for(
        &scan.inputs,
        challenge,
        &scan.witness.counts,
        &scan.witness.enc,
        &[],
    );
    let out = scratch(tag);
    let art = prover.prove_in(&id, &toml, &out, tag).expect("scan prove succeeds");
    (scan.inputs, encode_artifacts(&art))
}

/// A composed manifest carrying an honest age scan (index 0) + a FILTER sub-proof
/// (index 1). The query is a plain 1-pattern scan query with NO FILTER, so the
/// FILTER sub-proof is unreferenced by the query (no binding edge): the #1/#2
/// tests forge the FILTER's public inputs and stage 3 byte-compares it.
fn filter_manifest(
    scan_inputs: ProofInputs,
    scan_hex: String,
    inputs: ProofInputs,
    proof_hex: String,
    challenge: FieldHex,
) -> ProofManifest {
    let mut m = ProofManifest {
        r#type: "urn:sparq:zk:ProofManifest".into(),
        query: "SELECT ?s ?o WHERE { ?s <http://ex/age> ?o }".into(),
        issuers: vec!["did:key:zSampleIssuer".into()],
        key_set: vec![],
        commitment_attestations: vec![],
        attributions: vec![vec![0]],
        join_obligations: vec![],
        entailment_regime: EntailmentRegime::Simple,
        binding: BindingMode::Challenge { challenge },
        revocation: None,
        sub_proofs: vec![
            SubProof { inputs: scan_inputs, proof_hex: scan_hex },
            SubProof { inputs, proof_hex },
        ],
        binding_edges: vec![],
    };
    // [OPUS-4.8] audit #3/#9: attest the honest scan (salt-bound) so the #1/#2
    // forge tests reach the crypto gate (the FILTER forge they probe), not the
    // #3 attestation gate. The scan comes from `honest_age_scan`, which commits
    // under salt byte 7 — so the attestation salt must match.
    attest_all(&mut m, &test_issuer_sk(1), salt_from_bytes(&[7u8; 32]));
    m
}

/// Positive control: an honest filter proof verifies through the full
/// verify_manifest path (reconstruction byte-matches, canonical vk verifies).
#[test]
fn forge_positive_honest_filter_verifies() {
    if !toolchain_available() {
        eprintln!("nargo/bb absent; skipping");
        return;
    }
    let challenge = FieldHex("0x2a".into());
    let prover = CircuitProver::from_crate_root();
    let (scan_inputs, scan_hex) = honest_age_scan(&challenge, &prover, "forge_pos_scan");
    // 5 < 10 is true.
    let (inputs, art) =
        honest_filter_d1(5, FilterOp::Lt, 10, true, &challenge, &prover, "forge_pos");
    let m = filter_manifest(scan_inputs, scan_hex, inputs, encode_artifacts(&art), challenge);
    verify_manifest(&m, &prover, &scratch("forge_pos_verify"), &trusted_k(&test_issuer_sk(1)))
        .expect("honest manifest verifies");
}

/// Audit #1: a GENUINE proof over statement A (5 < 10 = true) presented under a
/// manifest declaring a DIFFERENT statement B (5 < 99) must be REJECTED. The
/// proof_hex is the real proof of A; only the declared `bound` is changed. The
/// reconstructed public-input vector (bound=99) cannot byte-match the proof's
/// (bound=10) => PublicInputMismatch. Before #1 this returned Ok(()).
#[test]
fn forge_reject_statement_substitution() {
    if !toolchain_available() {
        eprintln!("nargo/bb absent; skipping");
        return;
    }
    let challenge = FieldHex("0x2a".into());
    let prover = CircuitProver::from_crate_root();
    let (scan_inputs, scan_hex) = honest_age_scan(&challenge, &prover, "forge_sub_scan");
    let (mut inputs, art) =
        honest_filter_d1(5, FilterOp::Lt, 10, true, &challenge, &prover, "forge_sub");
    // Lie: declare bound=99 while proof_hex still proves 5 < 10.
    if let ProofInputs::FilterInt { bound, .. } = &mut inputs {
        *bound = 99;
    }
    let m = filter_manifest(scan_inputs, scan_hex, inputs, encode_artifacts(&art), challenge);
    match verify_manifest(&m, &prover, &scratch("forge_sub_verify"), &trusted_k(&test_issuer_sk(1))) {
        Err(CheckError::PublicInputMismatch { proof: 1 }) => {}
        other => panic!("expected PublicInputMismatch, got {other:?}"),
    }
}

/// Audit #1: substituting the declared verdict (claim 5 >= 10 is true, while the
/// proof is for the genuine false verdict) is rejected by the byte-compare.
#[test]
fn forge_reject_verdict_substitution() {
    if !toolchain_available() {
        eprintln!("nargo/bb absent; skipping");
        return;
    }
    let challenge = FieldHex("0x2a".into());
    let prover = CircuitProver::from_crate_root();
    let (scan_inputs, scan_hex) = honest_age_scan(&challenge, &prover, "forge_verdict_scan");
    // Honest: 5 >= 10 is FALSE.
    let (mut inputs, art) =
        honest_filter_d1(5, FilterOp::Ge, 10, false, &challenge, &prover, "forge_verdict");
    // Lie: flip the declared verdict to true.
    if let ProofInputs::FilterInt { expected, .. } = &mut inputs {
        *expected = true;
    }
    let m = filter_manifest(scan_inputs, scan_hex, inputs, encode_artifacts(&art), challenge);
    match verify_manifest(&m, &prover, &scratch("forge_verdict_verify"), &trusted_k(&test_issuer_sk(1))) {
        Err(CheckError::PublicInputMismatch { proof: 1 }) => {}
        other => panic!("expected PublicInputMismatch, got {other:?}"),
    }
}

/// Audit #1 (challenge binding seam for #4): a manifest whose binding challenge
/// differs from the challenge the proof was made under is rejected by the
/// byte-compare (the JSON challenge is now byte-bound into field 0). This is
/// the seam a later agent extends to a verifier-issued fresh nonce.
#[test]
fn forge_reject_challenge_rebind() {
    if !toolchain_available() {
        eprintln!("nargo/bb absent; skipping");
        return;
    }
    let prover = CircuitProver::from_crate_root();
    let proof_challenge = FieldHex("0x2a".into());
    let (scan_inputs, scan_hex) = honest_age_scan(&proof_challenge, &prover, "forge_chal_scan");
    let (inputs, art) =
        honest_filter_d1(5, FilterOp::Lt, 10, true, &proof_challenge, &prover, "forge_chal");
    // Present under a DIFFERENT binding challenge than the proofs carry. Both
    // sub-proofs byte-bind the challenge into field 0, so the first-checked
    // (scan, proof 0) already mismatches.
    let m = filter_manifest(scan_inputs, scan_hex, inputs, encode_artifacts(&art), FieldHex("0xdead".into()));
    match verify_manifest(&m, &prover, &scratch("forge_chal_verify"), &trusted_k(&test_issuer_sk(1))) {
        Err(CheckError::PublicInputMismatch { proof: 0 }) => {}
        other => panic!("expected PublicInputMismatch, got {other:?}"),
    }
}

/// Audit #2: a prover-supplied NON-CANONICAL vk is never used — an attacker
/// circuit with the same public-input arity as filter_int_d1 but zero soundness
/// constraints lets the prover "prove" a false statement under its OWN vk. The
/// declared ProofInputs match the attacker's fabricated public inputs (so #1's
/// byte-compare passes), but verify_manifest recomputes the CANONICAL d1 vk,
/// under which the attacker proof does not verify => ProofRejected. Before #2
/// (trusting art.vk) this returned Ok(()).
#[test]
fn forge_reject_noncanonical_vk() {
    if !toolchain_available() {
        eprintln!("nargo/bb absent; skipping");
        return;
    }
    let challenge = FieldHex("0x2a".into());
    // The attacker's fabricated statement: operand_enc = Enc(17), op=Ge,
    // bound=18, expected=true (i.e. "17 >= 18 is true" — a lie). We declare
    // exactly this so the reconstructed public inputs byte-match the attacker's.
    let operand_enc = encode_int_literal(17);
    let inputs = ProofInputs::FilterInt {
        id: CircuitId::FilterInt { d: 1 },
        operand_enc: operand_enc.clone(),
        op: FilterOp::Ge,
        bound: 18,
        expected: true,
    };
    let art = attacker_filter_d1_artifacts(&challenge, &operand_enc, 3 /*Ge*/, 18, true);
    let prover = CircuitProver::from_crate_root();
    let (scan_inputs, scan_hex) = honest_age_scan(&challenge, &prover, "forge_vk_scan");
    let m = filter_manifest(scan_inputs, scan_hex, inputs, encode_artifacts(&art), challenge);
    match verify_manifest(&m, &prover, &scratch("forge_vk_verify"), &trusted_k(&test_issuer_sk(1))) {
        Err(CheckError::ProofRejected { proof: 1 }) => {}
        other => panic!("expected ProofRejected (canonical vk defeats attacker vk), got {other:?}"),
    }
}

/// Audit #2 corollary: the prover-supplied `art.vk` is genuinely IGNORED — an
/// honest proof+statement verifies even when `art.vk` is replaced with garbage,
/// because verify_manifest recomputes the canonical vk. (Proves the canonical
/// vk, not art.vk, is the one bb uses.)
#[test]
fn forge_artvk_is_ignored() {
    if !toolchain_available() {
        eprintln!("nargo/bb absent; skipping");
        return;
    }
    let challenge = FieldHex("0x2a".into());
    let prover = CircuitProver::from_crate_root();
    let (scan_inputs, scan_hex) = honest_age_scan(&challenge, &prover, "forge_ignorevk_scan");
    let (inputs, mut art) =
        honest_filter_d1(5, FilterOp::Lt, 10, true, &challenge, &prover, "forge_ignorevk");
    // Corrupt the bundled vk — the verifier must not use it.
    for b in art.vk.iter_mut() {
        *b ^= 0xff;
    }
    let m = filter_manifest(scan_inputs, scan_hex, inputs, encode_artifacts(&art), challenge);
    verify_manifest(&m, &prover, &scratch("forge_ignorevk_verify"), &trusted_k(&test_issuer_sk(1)))
        .expect("honest proof verifies despite a garbage bundled vk (canonical vk is used)");
}

/// Compile + prove a trivial ATTACKER circuit that has the same 5 public inputs
/// as filter_int_d1 (challenge, operand_enc, op, bound, expected) but NO
/// soundness constraints, over the fabricated public values. Returns the
/// (attacker proof, its public_inputs, its attacker vk).
fn attacker_filter_d1_artifacts(
    challenge: &FieldHex,
    operand_enc: &FieldHex,
    op: u32,
    bound: u64,
    expected: bool,
) -> ProofArtifacts {
    let dir = scratch("attacker_circuit");
    let src = dir.join("src");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(
        dir.join("Nargo.toml"),
        "[package]\nname = \"attacker_filter\"\ntype = \"bin\"\nauthors = [\"\"]\ncompiler_version = \">=1.0.0\"\n",
    )
    .unwrap();
    std::fs::write(
        src.join("main.nr"),
        // Same arity as filter_int_d1's main; zero constraints.
        "fn main(challenge: pub Field, operand_enc: pub Field, op: pub u32, bound: pub u64, expected: pub bool) {\n    let _ = (challenge, operand_enc, op, bound, expected);\n}\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("Prover.toml"),
        format!(
            "challenge = \"{}\"\noperand_enc = \"{}\"\nop = \"{op}\"\nbound = \"{bound}\"\nexpected = {expected}\n",
            challenge.0, operand_enc.0
        ),
    )
    .unwrap();
    let nargo = |args: &[&str]| {
        let out = std::process::Command::new("nargo")
            .args(args)
            .current_dir(&dir)
            .output()
            .expect("nargo runs");
        assert!(out.status.success(), "nargo {args:?}: {}", String::from_utf8_lossy(&out.stderr));
    };
    nargo(&["compile"]);
    nargo(&["execute", "attacker_w"]);
    let acir = dir.join("target/attacker_filter.json");
    let wit = dir.join("target/attacker_w.gz");
    let out = dir.join("out");
    std::fs::create_dir_all(&out).unwrap();
    let bb = std::process::Command::new("bb")
        .args(["prove", "-b"])
        .arg(&acir)
        .arg("-w")
        .arg(&wit)
        .arg("-o")
        .arg(&out)
        .args(["--write_vk", "-t", "noir-recursive"])
        .output()
        .expect("bb runs");
    assert!(bb.status.success(), "bb prove: {}", String::from_utf8_lossy(&bb.stderr));
    ProofArtifacts {
        proof: std::fs::read(out.join("proof")).unwrap(),
        public_inputs: std::fs::read(out.join("public_inputs")).unwrap(),
        vk: std::fs::read(out.join("vk")).unwrap(),
    }
}

// --- query-correctness FILTER-binding NEGATIVE tests (audit #5/#6/#7/#10) -----
//
// [OPUS-4.8] These exercise the VERIFIER-SIDE query-correctness binding added in
// this phase. The bb public-input vector already cryptographically binds the
// scan pattern constants and the FILTER op/bound/expected/operand_enc (audit #1,
// landed on main); this stage checks those bound values MATCH the query the
// relying party reads. They are STRUCTURAL (`verify_manifest_structure`, no bb)
// because every value the gate inspects is in the declared ProofInputs — so they
// run in default CI without the toolchain, and they cannot be masked by a later
// crypto failure (the structural gate runs first). The happy-path composed
// manifest (a query WITH a FILTER + a correct edge) verifies — see
// `filter_binding_happy_path_structure`.

/// A credential graph with both an age and a salary numeric literal, for the
/// operand-slot / constant-swap forges.
fn pensioner_graph() -> Vec<Triple> {
    let p = NamedOrBlankNode::NamedNode(iri("http://ex/p"));
    // Salary fits the d=4 filter_int member (FILTER_INT_D_VALUES = [1,2,4]).
    vec![
        Triple::new(p.clone(), iri("http://ex/hasSalary"), int_lit(7000)),
        Triple::new(p, iri("http://ex/hasAge"), int_lit(40)),
    ]
}

/// Build a scan `ProofInputs` (no proving) for a single-constant-predicate
/// pattern `{ ?s <pred> ?o }` over `graph`.
fn scan_inputs_for(graph: &[Triple], pred: &str) -> ProofInputs {
    let salt = salt_from_bytes(&[9u8; 32]);
    let commit = commit_triples(graph, salt).unwrap();
    let pattern = Pattern {
        s: Slot::Var,
        p: Slot::Const(Term::NamedNode(iri(pred))),
        o: Slot::Var,
    };
    build_scan(&[commit], &pattern).expect("scan builds").inputs
}

/// A filter `ProofInputs` over an xsd:integer `value` (no proving).
fn filter_inputs(value: u64, op: FilterOp, bound: u64, expected: bool) -> ProofInputs {
    let operand_enc = encode_int_literal(value);
    build_filter_int(operand_enc, value, op, bound, expected)
        .expect("filter builds")
        .0
}

/// Audit #5 (comparison-substitution): a `filter_int` over (op=Ge, bound=17,
/// expected=true) — a genuinely-true `17 >= 17` instance — must NOT satisfy a
/// query `FILTER(?o >= 18)`. The bound (17) differs from the query constant
/// (18), so no edge matches => UnboundFilter. This is the headline age-17-vs-`>=18`
/// forge.
#[test]
fn filter_reject_comparison_substitution_17_vs_18() {
    let scan = scan_inputs_for(&credential_graph(), "http://ex/age");
    let scan_operand = match &scan {
        ProofInputs::Scan { rows, .. } => rows[0][2].clone(),
        _ => unreachable!(),
    };
    // The honest filter proves 17 >= 17 (bound=17), but the query asks >= 18.
    let mut filt = filter_inputs(17, FilterOp::Ge, 17, true);
    // Point the operand at the scanned slot so stage-2 binding-edge equality holds.
    if let ProofInputs::FilterInt { operand_enc, .. } = &mut filt {
        *operand_enc = scan_operand;
    }
    let m = ProofManifest {
        r#type: "urn:sparq:zk:ProofManifest".into(),
        query: "SELECT ?s ?o WHERE { ?s <http://ex/age> ?o FILTER(?o >= \"18\"^^<http://www.w3.org/2001/XMLSchema#integer>) }".into(),
        issuers: vec![],
        key_set: vec![],
        commitment_attestations: vec![],
        attributions: vec![vec![0]],
        join_obligations: vec![],
        entailment_regime: EntailmentRegime::Simple,
        binding: BindingMode::Challenge { challenge: FieldHex("0x2a".into()) },
        revocation: None,
        sub_proofs: vec![
            SubProof { inputs: scan, proof_hex: String::new() },
            SubProof { inputs: filt, proof_hex: String::new() },
        ],
        binding_edges: vec![BindingEdge { from_proof: 0, from_row: 0, from_slot: 2, to_proof: 1 }],
    };
    match verify_manifest_structure(&m, &empty_k()) {
        Err(CheckError::UnboundFilter { variable }) if variable == "o" => {}
        other => panic!("expected UnboundFilter(o), got {other:?}"),
    }
}

/// Audit #10 (FILTER-add): a scan-ONLY manifest presented under a query that
/// carries a FILTER must be REJECTED — the disclosed (unfiltered) rows would be
/// read as satisfying the FILTER. No filter sub-proof => UnboundFilter.
#[test]
fn filter_reject_filter_add_on_scan_only() {
    let scan = scan_inputs_for(&credential_graph(), "http://ex/age");
    let m = ProofManifest {
        r#type: "urn:sparq:zk:ProofManifest".into(),
        query: "SELECT ?s ?o WHERE { ?s <http://ex/age> ?o FILTER(?o >= \"18\"^^<http://www.w3.org/2001/XMLSchema#integer>) }".into(),
        issuers: vec![],
        key_set: vec![],
        commitment_attestations: vec![],
        attributions: vec![vec![0]],
        join_obligations: vec![],
        entailment_regime: EntailmentRegime::Simple,
        binding: BindingMode::Challenge { challenge: FieldHex("0x2a".into()) },
        revocation: None,
        sub_proofs: vec![SubProof { inputs: scan, proof_hex: String::new() }],
        binding_edges: vec![],
    };
    match verify_manifest_structure(&m, &empty_k()) {
        Err(CheckError::UnboundFilter { variable }) if variable == "o" => {}
        other => panic!("expected UnboundFilter(o), got {other:?}"),
    }
}

/// Audit #10 (constant-swap): an age scan (pattern_const_enc = Enc(<hasAge>))
/// presented under a query whose pattern uses <hasSalary> must be REJECTED — the
/// query pattern's constant has no scan binding it => UnboundPattern.
#[test]
fn filter_reject_constant_swap_age_as_salary() {
    let scan = scan_inputs_for(&credential_graph(), "http://ex/age");
    let m = ProofManifest {
        r#type: "urn:sparq:zk:ProofManifest".into(),
        query: "SELECT ?s ?o WHERE { ?s <http://ex/salary> ?o }".into(),
        issuers: vec![],
        key_set: vec![],
        commitment_attestations: vec![],
        attributions: vec![vec![0]],
        join_obligations: vec![],
        entailment_regime: EntailmentRegime::Simple,
        binding: BindingMode::Challenge { challenge: FieldHex("0x2a".into()) },
        revocation: None,
        sub_proofs: vec![SubProof { inputs: scan, proof_hex: String::new() }],
        binding_edges: vec![],
    };
    match verify_manifest_structure(&m, &empty_k()) {
        Err(CheckError::UnboundPattern { pattern: 0 }) => {}
        other => panic!("expected UnboundPattern(0), got {other:?}"),
    }
}

/// Audit #6 (operand-slot substitution): for `FILTER(?age >= 65)` over a
/// two-pattern query { ?p <hasSalary> ?sal . ?p <hasAge> ?age }, the prover
/// points the binding edge at the SALARY scan's object slot (a value that
/// satisfies >= 65) instead of the AGE scan's object slot. The edge's
/// (from_proof, from_slot) does not correspond to ?age's scanned column =>
/// UnboundFilter. (The salary scan answers pattern 0; ?age binds only in
/// pattern 1.)
#[test]
fn filter_reject_operand_slot_substitution() {
    let g = pensioner_graph();
    let salary_scan = scan_inputs_for(&g, "http://ex/hasSalary"); // pattern 0
    let age_scan = scan_inputs_for(&g, "http://ex/hasAge"); // pattern 1
    // Operand points at the SALARY object (7000 >= 65 is true).
    let salary_operand = match &salary_scan {
        ProofInputs::Scan { rows, .. } => rows[0][2].clone(),
        _ => unreachable!(),
    };
    let mut filt = filter_inputs(7000, FilterOp::Ge, 65, true);
    if let ProofInputs::FilterInt { operand_enc, .. } = &mut filt {
        *operand_enc = salary_operand;
    }
    let m = ProofManifest {
        r#type: "urn:sparq:zk:ProofManifest".into(),
        query: "SELECT ?p WHERE { ?p <http://ex/hasSalary> ?sal . ?p <http://ex/hasAge> ?age FILTER(?age >= \"65\"^^<http://www.w3.org/2001/XMLSchema#integer>) }".into(),
        issuers: vec![],
        key_set: vec![],
        commitment_attestations: vec![],
        attributions: vec![vec![0], vec![0]],
        join_obligations: vec![],
        entailment_regime: EntailmentRegime::Simple,
        binding: BindingMode::Challenge { challenge: FieldHex("0x2a".into()) },
        revocation: None,
        sub_proofs: vec![
            SubProof { inputs: salary_scan, proof_hex: String::new() }, // proof 0
            SubProof { inputs: age_scan, proof_hex: String::new() },    // proof 1
            SubProof { inputs: filt, proof_hex: String::new() },        // proof 2
        ],
        // Edge points at proof 0 (salary scan) slot 2 — the WRONG column for ?age.
        binding_edges: vec![BindingEdge { from_proof: 0, from_row: 0, from_slot: 2, to_proof: 2 }],
    };
    match verify_manifest_structure(&m, &empty_k()) {
        Err(CheckError::UnboundFilter { variable }) if variable == "age" => {}
        other => panic!("expected UnboundFilter(age), got {other:?}"),
    }
}

/// Audit #5/#6 (verdict gating): a FILTER row whose honest verdict is FALSE
/// (expected=false) may not be presented as passing. The filter declares
/// expected=false; stage 2c requires expected==true => UnboundFilter.
#[test]
fn filter_reject_false_verdict_row() {
    let scan = scan_inputs_for(&credential_graph(), "http://ex/age");
    let scan_operand = match &scan {
        ProofInputs::Scan { rows, .. } => rows[0][2].clone(),
        _ => unreachable!(),
    };
    // age=25, query FILTER(?o >= 18): the honest verdict is TRUE, but the prover
    // declares expected=false (e.g. to mis-gate). A false-verdict row must not
    // satisfy the FILTER's row-inclusion obligation.
    let mut filt = filter_inputs(25, FilterOp::Ge, 18, false);
    if let ProofInputs::FilterInt { operand_enc, .. } = &mut filt {
        *operand_enc = scan_operand;
    }
    let m = ProofManifest {
        r#type: "urn:sparq:zk:ProofManifest".into(),
        query: "SELECT ?s ?o WHERE { ?s <http://ex/age> ?o FILTER(?o >= \"18\"^^<http://www.w3.org/2001/XMLSchema#integer>) }".into(),
        issuers: vec![],
        key_set: vec![],
        commitment_attestations: vec![],
        attributions: vec![vec![0]],
        join_obligations: vec![],
        entailment_regime: EntailmentRegime::Simple,
        binding: BindingMode::Challenge { challenge: FieldHex("0x2a".into()) },
        revocation: None,
        sub_proofs: vec![
            SubProof { inputs: scan, proof_hex: String::new() },
            SubProof { inputs: filt, proof_hex: String::new() },
        ],
        binding_edges: vec![BindingEdge { from_proof: 0, from_row: 0, from_slot: 2, to_proof: 1 }],
    };
    match verify_manifest_structure(&m, &empty_k()) {
        Err(CheckError::UnboundFilter { variable }) if variable == "o" => {}
        other => panic!("expected UnboundFilter(o), got {other:?}"),
    }
}

/// Audit #10 (unbindable FILTER fails closed): a FILTER outside the bindable
/// integer fragment (here a string-literal comparison) must be REJECTED, never
/// silently disclosed unproven. The recheck/fragment_filters parse rejects it.
#[test]
fn filter_reject_unbindable_filter_fragment() {
    let scan = scan_inputs_for(&credential_graph(), "http://ex/age");
    let m = ProofManifest {
        r#type: "urn:sparq:zk:ProofManifest".into(),
        query: "SELECT ?s ?o WHERE { ?s <http://ex/age> ?o FILTER(?o >= \"18\") }".into(),
        issuers: vec![],
        key_set: vec![],
        commitment_attestations: vec![],
        attributions: vec![vec![0]],
        join_obligations: vec![],
        entailment_regime: EntailmentRegime::Simple,
        binding: BindingMode::Challenge { challenge: FieldHex("0x2a".into()) },
        revocation: None,
        sub_proofs: vec![SubProof { inputs: scan, proof_hex: String::new() }],
        binding_edges: vec![],
    };
    match verify_manifest_structure(&m, &empty_k()) {
        Err(CheckError::Sparqzk(_)) => {}
        other => panic!("expected Sparqzk(UnsupportedFragment), got {other:?}"),
    }
}

/// Happy path (structural): a query WITH a correct FILTER, a matching scan, a
/// matching filter (op/bound/verdict), and a correct binding edge (operand slot =
/// the FILTER variable's scanned slot) — all the query-correctness gates pass.
#[test]
fn filter_binding_happy_path_structure() {
    let scan = scan_inputs_for(&credential_graph(), "http://ex/age"); // age=25
    let scan_operand = match &scan {
        ProofInputs::Scan { rows, .. } => rows[0][2].clone(),
        _ => unreachable!(),
    };
    // 25 >= 18 is true.
    let mut filt = filter_inputs(25, FilterOp::Ge, 18, true);
    if let ProofInputs::FilterInt { operand_enc, .. } = &mut filt {
        *operand_enc = scan_operand;
    }
    let mut m = ProofManifest {
        r#type: "urn:sparq:zk:ProofManifest".into(),
        query: "SELECT ?s ?o WHERE { ?s <http://ex/age> ?o FILTER(?o >= \"18\"^^<http://www.w3.org/2001/XMLSchema#integer>) }".into(),
        issuers: vec![],
        key_set: vec![],
        commitment_attestations: vec![],
        attributions: vec![vec![0]],
        join_obligations: vec![],
        entailment_regime: EntailmentRegime::Simple,
        binding: BindingMode::Challenge { challenge: FieldHex("0x2a".into()) },
        revocation: None,
        sub_proofs: vec![
            SubProof { inputs: scan, proof_hex: String::new() },
            SubProof { inputs: filt, proof_hex: String::new() },
        ],
        binding_edges: vec![BindingEdge { from_proof: 0, from_row: 0, from_slot: 2, to_proof: 1 }],
    };
    // [OPUS-4.8] audit #3/#9: attest the scan (salt-bound). `scan_inputs_for`
    // commits under salt byte 9, so the attestation salt must match.
    attest_all(&mut m, &test_issuer_sk(1), salt_from_bytes(&[9u8; 32]));
    verify_manifest_structure(&m, &trusted_k(&test_issuer_sk(1)))
        .expect("correct composed FILTER manifest verifies structurally");
}

/// Two-subject graph: one age passes the FILTER, one fails. Used to exercise
/// per-row verdict gating (a multi-row disclosed result).
fn two_age_graph() -> Vec<Triple> {
    vec![
        Triple::new(
            NamedOrBlankNode::NamedNode(iri("http://ex/alice")),
            iri("http://ex/age"),
            int_lit(25), // passes >= 18
        ),
        Triple::new(
            NamedOrBlankNode::NamedNode(iri("http://ex/bob")),
            iri("http://ex/age"),
            int_lit(15), // FAILS >= 18 — must not be disclosed as passing
        ),
    ]
}

/// Audit #5/#6 (per-row verdict gating): a scan disclosing TWO age rows (25 and
/// 15) under FILTER(?o >= 18), where the prover supplies a true-verdict filter
/// proof ONLY for the passing row (25). The failing row (15) is still disclosed
/// but has no true-verdict edge => UnboundFilter. Without per-row gating a prover
/// could disclose the failing row as if it passed.
#[test]
fn filter_reject_unproven_failing_row() {
    let scan = scan_inputs_for(&two_age_graph(), "http://ex/age");
    let (rows, row_count) = match &scan {
        ProofInputs::Scan { rows, row_count, .. } => (rows.clone(), *row_count),
        _ => unreachable!(),
    };
    assert_eq!(row_count, 2, "two disclosed age rows");
    // A true-verdict filter for the FIRST disclosed row only (whichever it is).
    let mut filt = filter_inputs(25, FilterOp::Ge, 18, true);
    if let ProofInputs::FilterInt { operand_enc, .. } = &mut filt {
        *operand_enc = rows[0][2].clone();
    }
    let m = ProofManifest {
        r#type: "urn:sparq:zk:ProofManifest".into(),
        query: "SELECT ?s ?o WHERE { ?s <http://ex/age> ?o FILTER(?o >= \"18\"^^<http://www.w3.org/2001/XMLSchema#integer>) }".into(),
        issuers: vec![],
        key_set: vec![],
        commitment_attestations: vec![],
        attributions: vec![vec![0]],
        join_obligations: vec![],
        entailment_regime: EntailmentRegime::Simple,
        binding: BindingMode::Challenge { challenge: FieldHex("0x2a".into()) },
        revocation: None,
        sub_proofs: vec![
            SubProof { inputs: scan, proof_hex: String::new() },
            SubProof { inputs: filt, proof_hex: String::new() },
        ],
        // Edge only for row 0 — row 1 has no true-verdict filter proof.
        binding_edges: vec![BindingEdge { from_proof: 0, from_row: 0, from_slot: 2, to_proof: 1 }],
    };
    match verify_manifest_structure(&m, &empty_k()) {
        Err(CheckError::UnboundFilter { variable }) if variable == "o" => {}
        other => panic!("expected UnboundFilter(o) for the unproven failing row, got {other:?}"),
    }
}

/// Per-row gating positive: BOTH disclosed rows carry a true-verdict filter
/// proof over their own operand slot => the composed FILTER manifest verifies.
#[test]
fn filter_two_rows_both_gated_verifies() {
    let scan = scan_inputs_for(&two_age_graph(), "http://ex/age");
    let rows = match &scan {
        ProofInputs::Scan { rows, row_count, .. } => {
            assert_eq!(*row_count, 2);
            rows.clone()
        }
        _ => unreachable!(),
    };
    // Each disclosed row's age (25 and 15) is >= 15, so use bound=15 so BOTH
    // verdicts are honestly true; one filter proof per row, one edge per row.
    let mut filt0 = filter_inputs(25, FilterOp::Ge, 15, true);
    let mut filt1 = filter_inputs(15, FilterOp::Ge, 15, true);
    if let ProofInputs::FilterInt { operand_enc, .. } = &mut filt0 {
        *operand_enc = rows[0][2].clone();
    }
    if let ProofInputs::FilterInt { operand_enc, .. } = &mut filt1 {
        *operand_enc = rows[1][2].clone();
    }
    let mut m = ProofManifest {
        r#type: "urn:sparq:zk:ProofManifest".into(),
        query: "SELECT ?s ?o WHERE { ?s <http://ex/age> ?o FILTER(?o >= \"15\"^^<http://www.w3.org/2001/XMLSchema#integer>) }".into(),
        issuers: vec![],
        key_set: vec![],
        commitment_attestations: vec![],
        attributions: vec![vec![0]],
        join_obligations: vec![],
        entailment_regime: EntailmentRegime::Simple,
        binding: BindingMode::Challenge { challenge: FieldHex("0x2a".into()) },
        revocation: None,
        sub_proofs: vec![
            SubProof { inputs: scan, proof_hex: String::new() },  // proof 0
            SubProof { inputs: filt0, proof_hex: String::new() }, // proof 1
            SubProof { inputs: filt1, proof_hex: String::new() }, // proof 2
        ],
        binding_edges: vec![
            BindingEdge { from_proof: 0, from_row: 0, from_slot: 2, to_proof: 1 },
            BindingEdge { from_proof: 0, from_row: 1, from_slot: 2, to_proof: 2 },
        ],
    };
    // [OPUS-4.8] audit #3/#9: attest the scan (salt-bound). `scan_inputs_for`
    // commits under salt byte 9.
    attest_all(&mut m, &test_issuer_sk(1), salt_from_bytes(&[9u8; 32]));
    verify_manifest_structure(&m, &trusted_k(&test_issuer_sk(1)))
        .expect("both rows gated true => verifies");
}

// --- issuer-signature / key-set NEGATIVE tests (audit #3) -----------------
//
// [OPUS-4.8] The forge-and-verify suite the brief + test-bench design (§5.1 #3)
// require. These are STRUCTURAL (no bb): stage 2d inspects the declared
// commitments[] + commitment_attestations + key_set, all of which are also
// byte-bound into the proof's public inputs by the audit #1 reconstruction.
// They run in default CI without the toolchain, and cannot be masked by a later
// crypto failure (the structural gate runs first). The minimum bar:
//   (a) unsigned / no-valid-issuer-sig commitment        => REJECT
//   (b) drop-a-triple-and-recommit (truncated-leaf)      => REJECT
//   (c) signature by a key NOT in K                       => REJECT
//   (d) happy path (issuer-signed commitment, key in K)   => VERIFIES

/// A scan-only manifest over `{ ?s <http://ex/age> ?o }` on `graph` under
/// `salt`, with NO attestation/key-set yet (the caller wires #3). Returns the
/// manifest, its single commitment, and the salt it was committed under — the
/// salt is needed because a scan-covering attestation MUST now be salt-bound
/// (codex 2221 HIGH), so the caller builds it with `attest_with_salt(c, salt, ..)`.
fn scan_only_manifest(graph: &[Triple], salt_byte: u8) -> (ProofManifest, Fr, Fr) {
    let salt = salt_from_bytes(&[salt_byte; 32]);
    let commit = commit_triples(graph, salt).unwrap();
    let commitment_fr = commit.commitment;
    let pattern = Pattern {
        s: Slot::Var,
        p: Slot::Const(Term::NamedNode(iri("http://ex/age"))),
        o: Slot::Var,
    };
    let scan = build_scan(&[commit], &pattern).expect("scan builds");
    let m = ProofManifest {
        r#type: "urn:sparq:zk:ProofManifest".into(),
        query: "SELECT ?s ?o WHERE { ?s <http://ex/age> ?o }".into(),
        issuers: vec![],
        key_set: vec![],
        commitment_attestations: vec![],
        attributions: vec![vec![0]],
        join_obligations: vec![],
        entailment_regime: EntailmentRegime::Simple,
        binding: BindingMode::Challenge { challenge: FieldHex("0x2a".into()) },
        revocation: None,
        sub_proofs: vec![SubProof { inputs: scan.inputs, proof_hex: String::new() }],
        binding_edges: vec![],
    };
    (m, commitment_fr, salt)
}

/// (a) An unsigned commitment (no attestation present) must be REJECTED: the
/// prover-invented commitment has no issuer backing.
#[test]
fn issuer_reject_unsigned_commitment() {
    let (m, _c, _salt) = scan_only_manifest(&credential_graph(), 7);
    // No commitment_attestations, no key_set. The external K trusts a real
    // issuer, so the rejection is "unattested", not "untrusted".
    match verify_manifest_structure(&m, &trusted_k(&test_issuer_sk(1))) {
        Err(CheckError::UnattestedCommitment { proof: 0, .. }) => {}
        other => panic!("expected UnattestedCommitment, got {other:?}"),
    }
}

/// (a') A commitment with an attestation whose SIGNATURE is invalid (wrong
/// commitment signed) must be REJECTED — an attestation present but not
/// cryptographically valid is no attestation.
#[test]
fn issuer_reject_invalid_signature() {
    let (mut m, c, salt) = scan_only_manifest(&credential_graph(), 7);
    let sk = test_issuer_sk(1);
    // Attest a DIFFERENT commitment value (salt-bound, so it reaches the
    // signature check rather than the salt-missing gate), then relabel it as
    // `c` — the signature is over the wrong message, so it cannot verify
    // against `c`.
    let wrong = attest_with_salt(c + Fr::from(1u64), salt, &sk);
    m.commitment_attestations.push(CommitmentAttestation {
        commitment: FieldHex::from_field(&c), // claim it covers `c`
        ..wrong
    });
    m.key_set.push(public_key_to_hex(&sk.public_key()));
    // External K trusts sk (so the declared key_set is a valid subset); the
    // failure is the invalid signature, not the trust anchor.
    match verify_manifest_structure(&m, &trusted_k(&sk)) {
        Err(CheckError::InvalidIssuerSignature { .. }) => {}
        other => panic!("expected InvalidIssuerSignature, got {other:?}"),
    }
}

/// (b) Drop-a-triple-and-recommit (truncated-leaf suppression): the issuer
/// signs the FULL credential's commitment; the prover drops the role triple,
/// recommits over the truncated leaves, and presents the truncated commitment.
/// The truncated `C(G')` differs from the signed `C(G)`, so the only
/// attestation in K does not cover it => REJECT.
#[test]
fn issuer_reject_drop_triple_recommit_suppression() {
    let sk = test_issuer_sk(1);
    let salt = salt_from_bytes(&[7u8; 32]);
    // Issuer attests the FULL credential (age + role).
    let full = commit_triples(&credential_graph(), salt).unwrap();
    let full_attestation = attest(full.commitment, &sk);

    // Prover truncates: keep only the age triple, recommit, build a scan.
    let truncated: Vec<Triple> = vec![credential_graph()[0].clone()];
    let trunc_commit = commit_triples(&truncated, salt).unwrap();
    assert_ne!(
        trunc_commit.commitment, full.commitment,
        "truncation must change the commitment"
    );
    let pattern = Pattern {
        s: Slot::Var,
        p: Slot::Const(Term::NamedNode(iri("http://ex/age"))),
        o: Slot::Var,
    };
    let scan = build_scan(&[trunc_commit], &pattern).expect("scan builds");
    let m = ProofManifest {
        r#type: "urn:sparq:zk:ProofManifest".into(),
        query: "SELECT ?s ?o WHERE { ?s <http://ex/age> ?o }".into(),
        issuers: vec![],
        // The issuer's key is in K and its attestation over the FULL commitment
        // is present and valid — but it does not cover the truncated commitment.
        key_set: vec![public_key_to_hex(&sk.public_key())],
        commitment_attestations: vec![full_attestation],
        attributions: vec![vec![0]],
        join_obligations: vec![],
        entailment_regime: EntailmentRegime::Simple,
        binding: BindingMode::Challenge { challenge: FieldHex("0x2a".into()) },
        revocation: None,
        sub_proofs: vec![SubProof { inputs: scan.inputs, proof_hex: String::new() }],
        binding_edges: vec![],
    };
    match verify_manifest_structure(&m, &trusted_k(&sk)) {
        Err(CheckError::UnattestedCommitment { proof: 0, .. }) => {}
        other => panic!("expected UnattestedCommitment for the truncated recommit, got {other:?}"),
    }
}

/// (c) A signature by a key NOT in the disclosed key-set K must be REJECTED,
/// even though the signature itself is cryptographically valid.
#[test]
fn issuer_reject_key_not_in_keyset() {
    let (mut m, c, salt) = scan_only_manifest(&credential_graph(), 7);
    let signer = test_issuer_sk(2); // a real, valid signature ...
    m.commitment_attestations.push(attest_with_salt(c, salt, &signer));
    // ... but the EXTERNAL trust anchor K trusts a DIFFERENT issuer (sk3). The
    // manifest's declared key_set lists sk3 too (a valid subset of external K),
    // so the rejection is specifically that the ATTESTATION's key (sk2) is not in
    // the external K — not a subset violation.
    let trusted = test_issuer_sk(3);
    m.key_set.push(public_key_to_hex(&trusted.public_key()));
    match verify_manifest_structure(&m, &trusted_k(&trusted)) {
        Err(CheckError::IssuerKeyNotInKeySet { .. }) => {}
        other => panic!("expected IssuerKeyNotInKeySet, got {other:?}"),
    }
}

/// (c') An empty key-set K trusts no issuer: even a valid, present attestation
/// is rejected (fail closed).
#[test]
fn issuer_reject_empty_keyset() {
    let (mut m, c, salt) = scan_only_manifest(&credential_graph(), 7);
    let signer = test_issuer_sk(2);
    m.commitment_attestations.push(attest_with_salt(c, salt, &signer));
    // The EXTERNAL K is empty (trusts no issuer); the declared key_set is empty
    // too, so the subset check is vacuous and the attestation key falls outside K.
    match verify_manifest_structure(&m, &empty_k()) {
        Err(CheckError::IssuerKeyNotInKeySet { .. }) => {}
        other => panic!("expected IssuerKeyNotInKeySet (empty K), got {other:?}"),
    }
}

/// (d) Happy path: an issuer-signed commitment whose key is in K VERIFIES
/// (structurally). The positive control for the #3 gate.
#[test]
fn issuer_accept_signed_commitment_in_keyset() {
    let (mut m, c, salt) = scan_only_manifest(&credential_graph(), 7);
    let sk = test_issuer_sk(1);
    m.commitment_attestations.push(attest_with_salt(c, salt, &sk));
    m.key_set.push(public_key_to_hex(&sk.public_key()));
    // The relying party's EXTERNAL K trusts exactly this issuer.
    verify_manifest_structure(&m, &trusted_k(&sk))
        .expect("issuer-signed, in-K commitment verifies");
}

/// (d') An unknown cryptosuite is unverifiable => REJECT (fail closed), even
/// with a key in K — the verifier will not silently accept a scheme it cannot
/// check.
#[test]
fn issuer_reject_unknown_cryptosuite() {
    let (mut m, c, salt) = scan_only_manifest(&credential_graph(), 7);
    let sk = test_issuer_sk(1);
    let mut att = attest_with_salt(c, salt, &sk);
    att.cryptosuite = "https://sparq.dev/ns/zk#some-future-scheme".into();
    m.commitment_attestations.push(att);
    m.key_set.push(public_key_to_hex(&sk.public_key()));
    match verify_manifest_structure(&m, &trusted_k(&sk)) {
        Err(CheckError::InvalidIssuerSignature { .. }) => {}
        other => panic!("expected InvalidIssuerSignature (unknown cryptosuite), got {other:?}"),
    }
}

// --- codex #1: the PROVER-CONTROLLED-TRUST-ANCHOR forge (the soundness hole) --
//
// [OPUS-4.8] This is the test the prior round was MISSING. Before the fix the
// verifier read `manifest.key_set` (PROVER-supplied) as the trusted issuer set,
// so a malicious prover could: (1) generate its OWN issuer key, (2) sign a
// forged commitment with it, (3) self-list that key in `manifest.key_set`, and
// the attestation gate passed — giving NO real "authoritative source"
// guarantee. The fix makes K an EXTERNAL relying-party input; the prover's
// self-listed key is not in it, so the manifest is REJECTED.

/// codex #1 (headline): a prover signs a forged commitment with its OWN key and
/// self-lists that key in `manifest.key_set`, but the EXTERNAL trusted K does
/// NOT contain it ⇒ MUST be REJECTED. The prover may not widen the trust anchor:
/// declaring its own key in `key_set` is a subset violation against the external
/// K, caught as `UntrustedDeclaredKey`. (Before the fix this verified — the hole.)
#[test]
fn issuer_reject_prover_self_signed_key_not_in_external_k() {
    // The prover's OWN issuer key — a perfectly valid keypair it controls.
    let prover_key = test_issuer_sk(42);
    let (mut m, c, salt) = scan_only_manifest(&credential_graph(), 7);
    // A cryptographically VALID, salt-bound signature over the real commitment,
    // under the prover's own key — so the per-attestation signature check would
    // pass; the rejection is purely the trust-anchor (declared-key) violation.
    m.commitment_attestations.push(attest_with_salt(c, salt, &prover_key));
    // The prover self-lists its key, exactly as the old prover-trusts-manifest
    // path required. This is the forge.
    m.key_set.push(public_key_to_hex(&prover_key.public_key()));

    // The relying party's EXTERNAL K trusts a DIFFERENT, real issuer (the DMV,
    // say) — it has never heard of the prover's self-minted key.
    let real_issuer = test_issuer_sk(1);
    match verify_manifest_structure(&m, &trusted_k(&real_issuer)) {
        // The prover tried to WIDEN the external trust anchor with its own key.
        Err(CheckError::UntrustedDeclaredKey { .. }) => {}
        other => panic!(
            "expected UntrustedDeclaredKey (prover self-listed key not in external K), got {other:?}"
        ),
    }
}

/// codex #1 (variant): even if the prover does NOT declare its self-minted key
/// in `manifest.key_set` (leaving the subset check vacuous), the ATTESTATION's
/// key is still checked against the EXTERNAL K — so a forged self-signed
/// commitment is rejected as `IssuerKeyNotInKeySet`. This proves the gate does
/// not depend on the manifest's key_set at all: the external K is the only
/// anchor for the accept decision.
#[test]
fn issuer_reject_prover_self_signed_empty_declared_keyset() {
    let prover_key = test_issuer_sk(42);
    let (mut m, c, salt) = scan_only_manifest(&credential_graph(), 7);
    m.commitment_attestations.push(attest_with_salt(c, salt, &prover_key));
    // manifest.key_set deliberately EMPTY (no subset violation to lean on).
    assert!(m.key_set.is_empty());
    let real_issuer = test_issuer_sk(1);
    match verify_manifest_structure(&m, &trusted_k(&real_issuer)) {
        Err(CheckError::IssuerKeyNotInKeySet { .. }) => {}
        other => panic!(
            "expected IssuerKeyNotInKeySet (forged self-signed key not in external K), got {other:?}"
        ),
    }
}

/// codex #1 (positive control): the SAME forged manifest VERIFIES once the
/// relying party's EXTERNAL K is widened to trust the prover's key — confirming
/// the only thing that changed the verdict is the external anchor, not anything
/// in the prover-controlled manifest. (Sanity: the signature itself was always
/// valid; trust is what the fix gates on.)
#[test]
fn issuer_accept_when_external_k_trusts_the_key() {
    let key = test_issuer_sk(42);
    let (mut m, c, salt) = scan_only_manifest(&credential_graph(), 7);
    m.commitment_attestations.push(attest_with_salt(c, salt, &key));
    m.key_set.push(public_key_to_hex(&key.public_key()));
    // The relying party DECIDES to trust this issuer, out of band.
    verify_manifest_structure(&m, &trusted_k(&key))
        .expect("verifies once the EXTERNAL K trusts the signing key");
}

// --- codex 2216 LOW: declared-key_set consistency -------------------------
//
// [OPUS-4.8] When the prover DECLARES a narrowed `manifest.key_set` (non-empty),
// an accepted attestation key must ALSO be a member of it. The accept decision
// stays anchored on the EXTERNAL K (an attestation key is always required to be
// in K); this rule additionally forbids advertising a tighter issuer set than
// was actually used. A declared set that omits the real signing key is an
// inconsistent narrowing and is rejected.

/// codex 2216 LOW (headline): the external K trusts BOTH issuer A and issuer B,
/// the commitment is genuinely signed by B (whose key IS in external K, so the
/// soundness anchor is satisfied), but the prover's DECLARED `manifest.key_set`
/// lists only A. The declared narrowing is inconsistent with the proven
/// attestation ⇒ REJECT with `AttestationKeyNotInDeclaredSet`.
#[test]
fn issuer_reject_declared_keyset_omits_attestation_key() {
    let signer = test_issuer_sk(2); // issuer B — actually signs
    let declared_only = test_issuer_sk(3); // issuer A — declared but unused
    let (mut m, c, salt) = scan_only_manifest(&credential_graph(), 7);
    m.commitment_attestations.push(attest_with_salt(c, salt, &signer));
    // The prover declares a NARROWED set that omits the real signer B.
    m.key_set.push(public_key_to_hex(&declared_only.public_key()));
    // External K trusts BOTH A and B (so the external-K anchor for B passes, and
    // the declared key A is a valid subset of K — no UntrustedDeclaredKey).
    let trusted = KeySet::from_hex_keys([
        public_key_to_hex(&signer.public_key()),
        public_key_to_hex(&declared_only.public_key()),
    ]);
    match verify_manifest_structure(&m, &trusted) {
        Err(CheckError::AttestationKeyNotInDeclaredSet { .. }) => {}
        other => panic!(
            "expected AttestationKeyNotInDeclaredSet (declared narrowing omits real signer), got {other:?}"
        ),
    }
}

/// codex 2216 LOW (positive control): the SAME manifest VERIFIES once the
/// declared `key_set` is widened to include the real signer's key — proving the
/// only thing the rejection turned on was the declared-set consistency, and the
/// external-K anchor is unchanged.
#[test]
fn issuer_accept_declared_keyset_includes_attestation_key() {
    let signer = test_issuer_sk(2);
    let (mut m, c, salt) = scan_only_manifest(&credential_graph(), 7);
    m.commitment_attestations.push(attest_with_salt(c, salt, &signer));
    m.key_set.push(public_key_to_hex(&signer.public_key()));
    verify_manifest_structure(&m, &trusted_k(&signer))
        .expect("verifies once the declared key_set contains the real signer");
}

/// codex 2216 LOW (no-narrowing control): an EMPTY declared `key_set` means "no
/// narrowing declared" — the external K alone governs, so a valid in-K
/// attestation still VERIFIES even though `manifest.key_set` does not list it.
#[test]
fn issuer_accept_empty_declared_keyset_skips_consistency_check() {
    let signer = test_issuer_sk(2);
    let (mut m, c, salt) = scan_only_manifest(&credential_graph(), 7);
    m.commitment_attestations.push(attest_with_salt(c, salt, &signer));
    // Deliberately leave the declared key_set empty (no narrowing).
    assert!(m.key_set.is_empty());
    verify_manifest_structure(&m, &trusted_k(&signer))
        .expect("empty declared key_set => external K governs, in-K attestation verifies");
}

/// Serde: the new key-set + attestation fields round-trip through JSON.
#[test]
fn issuer_attestation_serde_round_trip() {
    let (mut m, c, _salt) = scan_only_manifest(&credential_graph(), 7);
    let sk = test_issuer_sk(1);
    // [OPUS-4.8] keep a salt-LESS attestation here on purpose: this is a pure
    // JSON round-trip (no verify), so it still exercises that the legacy
    // `salt: None` shape serdes correctly even though the verifier now rejects it
    // on a scan-covering path (codex 2221 HIGH).
    m.commitment_attestations.push(attest(c, &sk));
    m.key_set.push(public_key_to_hex(&sk.public_key()));
    let json = m.to_json();
    assert!(json.contains("commitment_attestations"));
    assert!(json.contains("key_set"));
    assert!(json.contains("poseidon2-schnorr-v1"));
    let back = ProofManifest::from_json(&json).expect("round-trips");
    assert_eq!(m, back);
}

// ===========================================================================
// [OPUS-4.8] audit #8: per-row source-graph attribution binding.
// ===========================================================================
//
// The hole (issue #8): the Q6 cross-graph-bnode-join obligation gate decided
// obligations purely from `manifest.attributions` — a PROVER-controlled JSON
// field, unbound to any proof. A prover whose data joins a bnode-valued variable
// across two genuinely-distinct graphs could declare both joined patterns as
// drawn from the SAME graph (`[[0],[0]]`), so `|{0}| = 1`, no obligation, and the
// forbidden cross-graph bnode correlation slipped through.
//
// The fix: `scan.nr` step 4 constrains a PUBLIC per-graph `attribution[g]` bit to
// the true matched-graph set; the verifier byte-binds it (audit #1) and Stage 2e
// (`bind_attributions`) requires `manifest.attributions[pattern]` to be a SUPERSET
// of the proof-bound set. Under-declaring (the forge) is rejected.

/// Two single-graph credentials that share an entity (`alice`) but live in
/// SEPARATE committed graphs — the cross-graph join setup. G0 holds the age
/// triple, G1 the role triple.
fn alice_age_graph() -> Vec<Triple> {
    vec![Triple::new(
        NamedOrBlankNode::NamedNode(iri("http://ex/alice")),
        iri("http://ex/age"),
        int_lit(25),
    )]
}
fn alice_role_graph() -> Vec<Triple> {
    vec![Triple::new(
        NamedOrBlankNode::NamedNode(iri("http://ex/alice")),
        iri("http://ex/role"),
        Term::NamedNode(iri("http://ex/admin")),
    )]
}

/// Build a two-pattern cross-graph manifest: pattern 0 `?x <age> ?a`, pattern 1
/// `?x <role> ?r`, each scanning BOTH committed graphs (K=2). Pattern 0 matches
/// only in G0, pattern 1 only in G1 — so each scan's in-circuit attribution is a
/// singleton, and the honest declared attributions are `[[0],[1]]`. `declared`
/// is the (possibly dishonest) `manifest.attributions`; `obligations` the
/// declared non-bnode join obligations.
fn cross_graph_manifest(
    declared: Vec<Vec<usize>>,
    obligations: Vec<(String, usize, usize)>,
    sk: &SecretKey,
) -> ProofManifest {
    let salt0 = salt_from_bytes(&[10u8; 32]);
    let salt1 = salt_from_bytes(&[11u8; 32]);
    let g0 = commit_triples(&alice_age_graph(), salt0).unwrap();
    let g1 = commit_triples(&alice_role_graph(), salt1).unwrap();
    let commits = [g0.clone(), g1.clone()];

    let age_pat = Pattern {
        s: Slot::Var,
        p: Slot::Const(Term::NamedNode(iri("http://ex/age"))),
        o: Slot::Var,
    };
    let role_pat = Pattern {
        s: Slot::Var,
        p: Slot::Const(Term::NamedNode(iri("http://ex/role"))),
        o: Slot::Var,
    };
    let scan_age = build_scan(&commits, &age_pat).expect("age scan builds");
    let scan_role = build_scan(&commits, &role_pat).expect("role scan builds");

    let mut m = ProofManifest {
        r#type: "urn:sparq:zk:ProofManifest".into(),
        query: "SELECT ?x WHERE { ?x <http://ex/age> ?a . ?x <http://ex/role> ?r }".into(),
        issuers: vec![],
        key_set: vec![],
        commitment_attestations: vec![],
        attributions: declared,
        join_obligations: obligations,
        entailment_regime: EntailmentRegime::Simple,
        binding: BindingMode::Challenge { challenge: FieldHex("0x2a".into()) },
        revocation: None,
        sub_proofs: vec![
            SubProof { inputs: scan_age.inputs, proof_hex: String::new() },
            SubProof { inputs: scan_role.inputs, proof_hex: String::new() },
        ],
        binding_edges: vec![],
    };
    // Salt-bound attestations for BOTH commitments (audit #3 + #9), distinct salts.
    m.commitment_attestations.push(attest_with_salt(g0.commitment, salt0, sk));
    m.commitment_attestations.push(attest_with_salt(g1.commitment, salt1, sk));
    m.key_set.push(public_key_to_hex(&sk.public_key()));
    m
}

/// The minimum-bar #8 forge: two genuinely-distinct committed graphs joined on a
/// bnode-capable variable `?x`, declared `[[0],[0]]` to collapse them so the Q6
/// gate demands NO non-bnode obligation. The in-circuit attribution of the
/// role-pattern scan proves it drew from graph 1, which `[[0],[0]]` omits ⇒
/// REJECT with `AttributionUnderDeclared`. (Before #8 this PASSED — the hole.)
#[test]
fn attribution_lie_collapse_two_graphs_rejected() {
    let sk = test_issuer_sk(1);
    let m = cross_graph_manifest(vec![vec![0], vec![0]], vec![], &sk);
    match verify_manifest_structure(&m, &trusted_k(&sk)) {
        Err(CheckError::AttributionUnderDeclared { proof_graph: 1, .. }) => {}
        other => panic!(
            "expected AttributionUnderDeclared (the [[0],[0]] collapse-two-graphs forge), got {other:?}"
        ),
    }
}

/// Companion: even the OTHER collapse direction (`[[1],[1]]`, hiding graph 0) is
/// rejected — the age-pattern scan proves a graph-0 contribution `[[1]]` omits.
#[test]
fn attribution_lie_collapse_onto_graph_one_rejected() {
    let sk = test_issuer_sk(1);
    let m = cross_graph_manifest(vec![vec![1], vec![1]], vec![], &sk);
    match verify_manifest_structure(&m, &trusted_k(&sk)) {
        Err(CheckError::AttributionUnderDeclared { pattern: 0, proof_graph: 0 }) => {}
        other => panic!("expected AttributionUnderDeclared on pattern 0, got {other:?}"),
    }
}

/// Happy path: HONEST `[[0],[1]]` cross-graph attribution, distinct salts, WITH
/// the required non-bnode obligation on `?x` ⇒ VERIFIES (structurally). The
/// positive control: honest attribution that matches the proof-bound sets and
/// satisfies the Q6 obligation passes.
#[test]
fn attribution_honest_cross_graph_with_obligation_verifies() {
    let sk = test_issuer_sk(1);
    let m = cross_graph_manifest(
        vec![vec![0], vec![1]],
        vec![("x".to_string(), 0, 1)], // the required non-bnode obligation on ?x
        &sk,
    );
    verify_manifest_structure(&m, &trusted_k(&sk))
        .expect("honest [[0],[1]] cross-graph with a satisfied obligation verifies");
}

/// Honest `[[0],[1]]` WITHOUT the obligation ⇒ the Q6 gate (stage 1a) rejects the
/// undeclared cross-graph edge. Confirms the attribution binding does not
/// short-circuit the obligation gate — the two layers compose.
#[test]
fn attribution_honest_cross_graph_without_obligation_rejected() {
    let sk = test_issuer_sk(1);
    let m = cross_graph_manifest(vec![vec![0], vec![1]], vec![], &sk);
    match verify_manifest_structure(&m, &trusted_k(&sk)) {
        Err(CheckError::Sparqzk(_)) => {} // MissingObligation
        other => panic!("expected Sparqzk(MissingObligation), got {other:?}"),
    }
}

/// Over-declaring is conservative-safe: declaring `[[0,1],[1]]` (pattern 0 may
/// draw from BOTH graphs though it only matched in 0) is a SUPERSET of the
/// proof-bound set, so attribution binding accepts it — but the wider set demands
/// the obligation, which here IS declared ⇒ VERIFIES. (Confirms superset, not
/// equality, is the gate.)
#[test]
fn attribution_over_declared_is_accepted() {
    let sk = test_issuer_sk(1);
    let m = cross_graph_manifest(
        vec![vec![0, 1], vec![1]],
        vec![("x".to_string(), 0, 1)],
        &sk,
    );
    verify_manifest_structure(&m, &trusted_k(&sk))
        .expect("over-declared (superset) attribution with obligation verifies");
}

// ===========================================================================
// [OPUS-4.8] audit #9: cross-graph bnode salt separation.
// ===========================================================================
//
// The hole (issue #9): the Q6 "bnodes from different graphs are distinct by
// construction" guarantee rests on each graph having a globally-unique per-graph
// salt, but the salt never entered any circuit and the verifier never compared
// salts — a salt-reusing ingester could make a same-label bnode encode
// identically across two graphs (a cross-graph correlation handle).
//
// The fix (leveraging #3): the issuer signs `(commitment, salt)` so the salt is
// ISSUER-ATTESTED, and the verifier rejects two DISTINCT commitments sharing a
// salt. A salt-reusing ingester cannot get a valid issuer signature over the
// reused salt, nor pass the verifier's salt-uniqueness check.

/// Salt-reuse cross-graph correlation: two genuinely-distinct committed graphs
/// presented under the SAME salt ⇒ REJECT with `SaltReused`. This is the
/// correlation channel #9 closes.
#[test]
fn salt_reuse_across_distinct_graphs_rejected() {
    let sk = test_issuer_sk(1);
    let reused = salt_from_bytes(&[42u8; 32]);
    // Two distinct graphs, BOTH committed under the same salt.
    let g0 = commit_triples(&alice_age_graph(), reused).unwrap();
    let g1 = commit_triples(&alice_role_graph(), reused).unwrap();
    assert_ne!(g0.commitment, g1.commitment, "distinct content => distinct C(G)");

    let mut m = cross_graph_manifest(vec![vec![0], vec![1]], vec![("x".into(), 0, 1)], &sk);
    // Re-commit the two sub-proofs + attestations under the reused salt so the
    // manifest's commitments and salts are the salt-reuse case.
    let age_pat = Pattern { s: Slot::Var, p: Slot::Const(Term::NamedNode(iri("http://ex/age"))), o: Slot::Var };
    let role_pat = Pattern { s: Slot::Var, p: Slot::Const(Term::NamedNode(iri("http://ex/role"))), o: Slot::Var };
    let commits = [g0.clone(), g1.clone()];
    m.sub_proofs[0].inputs = build_scan(&commits, &age_pat).unwrap().inputs;
    m.sub_proofs[1].inputs = build_scan(&commits, &role_pat).unwrap().inputs;
    m.commitment_attestations = vec![
        attest_with_salt(g0.commitment, reused, &sk),
        attest_with_salt(g1.commitment, reused, &sk),
    ];
    match verify_manifest_structure(&m, &trusted_k(&sk)) {
        Err(CheckError::SaltReused { .. }) => {}
        other => panic!("expected SaltReused (cross-graph salt collision), got {other:?}"),
    }
}

/// A salt-bound attestation whose disclosed salt does NOT match the salt the
/// issuer signed ⇒ the salt-bound signature does not verify ⇒ REJECT. Confirms
/// the salt is genuinely issuer-attested (a prover cannot swap in a different
/// salt to dodge the uniqueness check). Single-scan manifest so the ONLY fault
/// is the salt swap (attribution/obligation gates are satisfied).
#[test]
fn salt_swap_breaks_issuer_signature() {
    let sk = test_issuer_sk(1);
    let salt = salt_from_bytes(&[7u8; 32]);
    let (mut m, c, _salt) = scan_only_manifest(&credential_graph(), 7);
    // Attestation signed over the TRUE salt, but DISCLOSING a different salt —
    // the salt-bound message the verifier recomputes (over the disclosed salt)
    // won't match the signature (made over the true salt).
    let mut att = attest_with_salt(c, salt, &sk);
    att.salt = Some(FieldHex::from_field(&salt_from_bytes(&[99u8; 32]))); // swapped
    m.commitment_attestations = vec![att];
    m.key_set.push(public_key_to_hex(&sk.public_key()));
    match verify_manifest_structure(&m, &trusted_k(&sk)) {
        Err(CheckError::InvalidIssuerSignature { .. }) => {}
        other => panic!("expected InvalidIssuerSignature (salt swap), got {other:?}"),
    }
}

/// Happy path: distinct issuer-attested salts across the two graphs ⇒ the salt
/// gate passes (and the manifest verifies structurally). Positive control for #9.
#[test]
fn distinct_salts_verify() {
    let sk = test_issuer_sk(1);
    // cross_graph_manifest already uses distinct salts (10 vs 11) and salt-bound
    // attestations; with the obligation declared it verifies end to end.
    let m = cross_graph_manifest(vec![vec![0], vec![1]], vec![("x".into(), 0, 1)], &sk);
    verify_manifest_structure(&m, &trusted_k(&sk))
        .expect("distinct issuer-attested salts verify");
}

// ===========================================================================
// [OPUS-4.8] codex 2221 HIGH/MEDIUM: fail-closed on omittable security fields.
// ===========================================================================
//
// Two "optional field ⇒ the new check is skipped" holes:
//   HIGH  — a scan-covering attestation could omit `salt` (`salt: None`) and pass
//           via the legacy bare-commitment_message path, bypassing the #9
//           salt-separation guarantee entirely. The fix: a scan-covering
//           attestation MUST be salt-bound; `salt: None` ⇒ REJECT.
//   MEDIUM — `ProofInputs::Scan.attribution` is `#[serde(default)]` (empty vec)
//           and `bind_attributions` only checks the bits PROVIDED, so an
//           omitted/short attribution makes the #8 cross-check vacuous (the
//           `[[0],[0]]` collapse forge resurfaces). The fix: attribution MUST be
//           present and EXACTLY `CircuitId.k` bits; missing/short/long ⇒ REJECT.

/// HIGH forge: a scan-covering attestation with `salt: None` (the legacy
/// salt-less shape) ⇒ REJECT with `ScanCommitmentSaltMissing`. Before the fix
/// this passed via the bare `commitment_message` path, silently bypassing #9.
/// The signature here is a VALID salt-less signature (so the only fault is the
/// missing salt, not a bad signature) — proving the rejection is the
/// fail-closed salt-mandatory gate, not an incidental signature failure.
#[test]
fn forge_scan_covering_attestation_salt_none_rejected() {
    let sk = test_issuer_sk(1);
    let (mut m, c, _salt) = scan_only_manifest(&credential_graph(), 7);
    // A cryptographically valid SALT-LESS attestation over the scan commitment
    // (signs the bare `commitment_message`), key in K. The ONLY defect is that a
    // scan-covering attestation may no longer be salt-less.
    m.commitment_attestations.push(attest(c, &sk)); // salt: None
    m.key_set.push(public_key_to_hex(&sk.public_key()));
    match verify_manifest_structure(&m, &trusted_k(&sk)) {
        Err(CheckError::ScanCommitmentSaltMissing { proof: 0, .. }) => {}
        other => panic!(
            "expected ScanCommitmentSaltMissing (salt-less scan-covering attestation must be rejected), got {other:?}"
        ),
    }
}

/// MEDIUM forge (omitted attribution): a scan whose `attribution` vector is
/// EMPTY (the `#[serde(default)]` default a prover gets by omitting the field)
/// ⇒ REJECT with `AttributionMalformed`. Before the fix `bind_attributions`
/// iterated zero bits, so the #8 under-declaration cross-check was vacuous.
#[test]
fn forge_omitted_attribution_rejected() {
    let sk = test_issuer_sk(1);
    let (mut m, c, salt) = scan_only_manifest(&credential_graph(), 7);
    // Strip the attribution to empty (k=1 here), as a hand-crafted / omitted-field
    // manifest would have it.
    if let ProofInputs::Scan { attribution, .. } = &mut m.sub_proofs[0].inputs {
        attribution.clear();
        assert!(attribution.is_empty());
    } else {
        unreachable!("sub-proof 0 is a scan");
    }
    m.commitment_attestations.push(attest_with_salt(c, salt, &sk));
    m.key_set.push(public_key_to_hex(&sk.public_key()));
    match verify_manifest_structure(&m, &trusted_k(&sk)) {
        Err(CheckError::AttributionMalformed { proof: 0, expected: 1, got: 0 }) => {}
        other => panic!(
            "expected AttributionMalformed (omitted attribution must be rejected), got {other:?}"
        ),
    }
}

/// MEDIUM forge (wrong-length attribution): a scan over k=1 graph whose
/// `attribution` carries 2 bits ⇒ REJECT with `AttributionMalformed`. A
/// mismatched length is rejected up front (it would also fail the audit #1 byte
/// reconstruction, but the structural gate catches it without bb).
#[test]
fn forge_wrong_length_attribution_rejected() {
    let sk = test_issuer_sk(1);
    let (mut m, c, salt) = scan_only_manifest(&credential_graph(), 7);
    // k is 1 (single commitment); push a spurious extra bit.
    if let ProofInputs::Scan { attribution, .. } = &mut m.sub_proofs[0].inputs {
        attribution.push(true);
        assert_eq!(attribution.len(), 2);
    } else {
        unreachable!("sub-proof 0 is a scan");
    }
    m.commitment_attestations.push(attest_with_salt(c, salt, &sk));
    m.key_set.push(public_key_to_hex(&sk.public_key()));
    match verify_manifest_structure(&m, &trusted_k(&sk)) {
        Err(CheckError::AttributionMalformed { proof: 0, expected: 1, got: 2 }) => {}
        other => panic!(
            "expected AttributionMalformed (wrong-length attribution must be rejected), got {other:?}"
        ),
    }
}

/// [OPUS-4.8] PROBE (ignored): re-dump the real bb `public_inputs` for the new
/// scan_k1_n16_r4 layout (now carrying `attribution[k]`) so the
/// `reconstruct_scan_matches_real_bb_public_inputs` unit-test constant stays an
/// honest empirical anchor. Run with `--ignored` when the toolchain is present;
/// paste the printed hex into the unit test.
#[test]
#[ignore = "probe: prints real bb public_inputs hex for the scan reconstruct unit test"]
fn probe_scan_public_inputs_hex() {
    if !toolchain_available() {
        eprintln!("nargo/bb absent; skipping probe");
        return;
    }
    let salt = salt_from_bytes(&[7u8; 32]);
    let commit = commit_triples(&credential_graph(), salt).unwrap();
    let pattern = Pattern {
        s: Slot::Var,
        p: Slot::Const(Term::NamedNode(iri("http://ex/age"))),
        o: Slot::Var,
    };
    let scan = build_scan(&[commit], &pattern).unwrap();
    let challenge = FieldHex("0x2a".into());
    let (id, toml) =
        prover_toml_for(&scan.inputs, &challenge, &scan.witness.counts, &scan.witness.enc, &[]);
    let prover = CircuitProver::from_crate_root();
    let out = scratch("probe_scan_pi");
    let art = prover.prove_in(&id, &toml, &out, "probe_scan_pi").unwrap();
    let hex: String = art.public_inputs.iter().map(|b| format!("{b:02x}")).collect();
    eprintln!("SCAN_PUBLIC_INPUTS_LEN={}", art.public_inputs.len());
    eprintln!("SCAN_PUBLIC_INPUTS_HEX={hex}");
    eprintln!("SCAN_INPUTS_JSON={}", serde_json::to_string(&scan.inputs).unwrap());
}
