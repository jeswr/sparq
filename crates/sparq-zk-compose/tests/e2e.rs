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
    BindingEdge, BindingMode, CircuitId, EntailmentRegime, FieldHex, FilterOp, ProofInputs,
    ProofManifest, RevocationStatus, SubProof,
};
use sparq_zk_compose::toml::prover_toml_for;
use sparq_zk_compose::verifier::{
    encode_artifacts, verify_manifest, verify_manifest_structure, CheckError,
};

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

    ProofManifest {
        r#type: "urn:sparq:zk:ProofManifest".into(),
        query: "SELECT ?s ?o WHERE { ?s <http://ex/age> ?o }".into(),
        issuers: vec!["did:key:zSampleIssuer".into()],
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
    }
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
    verify_manifest_structure(&m).expect("structure verifies");
}

#[test]
fn structure_rejects_inconsistent_binding_edge() {
    let mut m = sample_manifest();
    // Tamper: point the filter's operand at a different encoding than the
    // scanned column the binding edge claims.
    if let ProofInputs::FilterInt { operand_enc, .. } = &mut m.sub_proofs[1].inputs {
        *operand_enc = FieldHex("0xdeadbeef".into());
    }
    match verify_manifest_structure(&m) {
        Err(CheckError::BindingInconsistent { edge: 0 }) => {}
        other => panic!("expected BindingInconsistent, got {other:?}"),
    }
}

#[test]
fn structure_rejects_arity_mismatch() {
    let mut m = sample_manifest();
    // The query has 1 BGP pattern; declare 2 attributions.
    m.attributions = vec![vec![0], vec![0]];
    assert!(verify_manifest_structure(&m).is_err());
}

#[test]
fn structure_rejects_circuit_id_mismatch() {
    let mut m = sample_manifest();
    // Swap the declared scan id's k to a value its commitments don't support.
    if let ProofInputs::Scan { id, .. } = &mut m.sub_proofs[0].inputs {
        *id = CircuitId::Scan { k: 2, n: 16, r: 4 };
    }
    match verify_manifest_structure(&m) {
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
        verify_manifest_structure(&m),
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

    let manifest = ProofManifest {
        r#type: "urn:sparq:zk:ProofManifest".into(),
        query: "SELECT ?s ?o WHERE { ?s <http://ex/age> ?o }".into(),
        issuers: vec!["did:key:zSampleIssuer".into()],
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
    verify_manifest(&manifest, &prover, &scratch("manifest_verify"))
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
    ProofManifest {
        r#type: "urn:sparq:zk:ProofManifest".into(),
        query: "SELECT ?s ?o WHERE { ?s <http://ex/age> ?o }".into(),
        issuers: vec!["did:key:zSampleIssuer".into()],
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
    }
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
    verify_manifest(&m, &prover, &scratch("forge_pos_verify")).expect("honest manifest verifies");
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
    match verify_manifest(&m, &prover, &scratch("forge_sub_verify")) {
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
    match verify_manifest(&m, &prover, &scratch("forge_verdict_verify")) {
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
    match verify_manifest(&m, &prover, &scratch("forge_chal_verify")) {
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
    match verify_manifest(&m, &prover, &scratch("forge_vk_verify")) {
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
    verify_manifest(&m, &prover, &scratch("forge_ignorevk_verify"))
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
    match verify_manifest_structure(&m) {
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
        attributions: vec![vec![0]],
        join_obligations: vec![],
        entailment_regime: EntailmentRegime::Simple,
        binding: BindingMode::Challenge { challenge: FieldHex("0x2a".into()) },
        revocation: None,
        sub_proofs: vec![SubProof { inputs: scan, proof_hex: String::new() }],
        binding_edges: vec![],
    };
    match verify_manifest_structure(&m) {
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
        attributions: vec![vec![0]],
        join_obligations: vec![],
        entailment_regime: EntailmentRegime::Simple,
        binding: BindingMode::Challenge { challenge: FieldHex("0x2a".into()) },
        revocation: None,
        sub_proofs: vec![SubProof { inputs: scan, proof_hex: String::new() }],
        binding_edges: vec![],
    };
    match verify_manifest_structure(&m) {
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
    match verify_manifest_structure(&m) {
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
    match verify_manifest_structure(&m) {
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
        attributions: vec![vec![0]],
        join_obligations: vec![],
        entailment_regime: EntailmentRegime::Simple,
        binding: BindingMode::Challenge { challenge: FieldHex("0x2a".into()) },
        revocation: None,
        sub_proofs: vec![SubProof { inputs: scan, proof_hex: String::new() }],
        binding_edges: vec![],
    };
    match verify_manifest_structure(&m) {
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
    let m = ProofManifest {
        r#type: "urn:sparq:zk:ProofManifest".into(),
        query: "SELECT ?s ?o WHERE { ?s <http://ex/age> ?o FILTER(?o >= \"18\"^^<http://www.w3.org/2001/XMLSchema#integer>) }".into(),
        issuers: vec![],
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
    verify_manifest_structure(&m).expect("correct composed FILTER manifest verifies structurally");
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
    match verify_manifest_structure(&m) {
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
    let m = ProofManifest {
        r#type: "urn:sparq:zk:ProofManifest".into(),
        query: "SELECT ?s ?o WHERE { ?s <http://ex/age> ?o FILTER(?o >= \"15\"^^<http://www.w3.org/2001/XMLSchema#integer>) }".into(),
        issuers: vec![],
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
    verify_manifest_structure(&m).expect("both rows gated true => verifies");
}
