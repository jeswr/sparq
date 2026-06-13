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
    // 4-digit value -> filter_int_d4: a member no other concurrent test
    // touches (subprocess proving shares one Prover.toml per package, so
    // parallel tests MUST target distinct members — see README concurrency
    // note). 1234 >= 18 is true.
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
    prover.gen_witness(&id, &toml).expect("witness satisfiable");
}

#[test]
fn witness_gen_filter_int_rejects_false_verdict() {
    if !toolchain_available() {
        eprintln!("nargo/bb absent; skipping");
        return;
    }
    // 17 >= 18 is false; claim true -> witness generation must fail.
    let operand_enc = encode_int_literal(17);
    let (filter, digits) =
        build_filter_int(operand_enc, 17, FilterOp::Ge, 18, true).unwrap();
    let (id, toml) =
        prover_toml_for(&filter, &FieldHex("0x2a".into()), &[], &[], &digits);
    let prover = CircuitProver::from_crate_root();
    prover.compile(&id).unwrap();
    assert!(
        prover.gen_witness(&id, &toml).is_err(),
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
    prover.gen_witness(&id, &toml).expect("scan witness satisfiable");
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
    let art = prover.prove(&id, &toml, &out).expect("prove succeeds");
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
    let art = prover.prove(&id, &toml, &out).unwrap();

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
    let art = prover.prove(&id, &toml, &out).expect("prove succeeds");
    (filter, art)
}

fn filter_manifest(inputs: ProofInputs, proof_hex: String, challenge: FieldHex) -> ProofManifest {
    ProofManifest {
        r#type: "urn:sparq:zk:ProofManifest".into(),
        // A 1-pattern query so recheck/arity pass; the FILTER-semantics binding
        // (#5/#6) is a later agent's, so the query need only be in-fragment.
        query: "SELECT ?s ?o WHERE { ?s <http://ex/age> ?o }".into(),
        issuers: vec!["did:key:zSampleIssuer".into()],
        attributions: vec![vec![0]],
        join_obligations: vec![],
        entailment_regime: EntailmentRegime::Simple,
        binding: BindingMode::Challenge { challenge },
        revocation: None,
        sub_proofs: vec![SubProof { inputs, proof_hex }],
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
    // 5 < 10 is true.
    let (inputs, art) =
        honest_filter_d1(5, FilterOp::Lt, 10, true, &challenge, &prover, "forge_pos");
    let m = filter_manifest(inputs, encode_artifacts(&art), challenge);
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
    let (mut inputs, art) =
        honest_filter_d1(5, FilterOp::Lt, 10, true, &challenge, &prover, "forge_sub");
    // Lie: declare bound=99 while proof_hex still proves 5 < 10.
    if let ProofInputs::FilterInt { bound, .. } = &mut inputs {
        *bound = 99;
    }
    let m = filter_manifest(inputs, encode_artifacts(&art), challenge);
    match verify_manifest(&m, &prover, &scratch("forge_sub_verify")) {
        Err(CheckError::PublicInputMismatch { proof: 0 }) => {}
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
    // Honest: 5 >= 10 is FALSE.
    let (mut inputs, art) =
        honest_filter_d1(5, FilterOp::Ge, 10, false, &challenge, &prover, "forge_verdict");
    // Lie: flip the declared verdict to true.
    if let ProofInputs::FilterInt { expected, .. } = &mut inputs {
        *expected = true;
    }
    let m = filter_manifest(inputs, encode_artifacts(&art), challenge);
    match verify_manifest(&m, &prover, &scratch("forge_verdict_verify")) {
        Err(CheckError::PublicInputMismatch { proof: 0 }) => {}
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
    let (inputs, art) =
        honest_filter_d1(5, FilterOp::Lt, 10, true, &proof_challenge, &prover, "forge_chal");
    // Present under a DIFFERENT binding challenge than the proof carries.
    let m = filter_manifest(inputs, encode_artifacts(&art), FieldHex("0xdead".into()));
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
    let m = filter_manifest(inputs, encode_artifacts(&art), challenge);
    let prover = CircuitProver::from_crate_root();
    match verify_manifest(&m, &prover, &scratch("forge_vk_verify")) {
        Err(CheckError::ProofRejected { proof: 0 }) => {}
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
    let (inputs, mut art) =
        honest_filter_d1(5, FilterOp::Lt, 10, true, &challenge, &prover, "forge_ignorevk");
    // Corrupt the bundled vk — the verifier must not use it.
    for b in art.vk.iter_mut() {
        *b ^= 0xff;
    }
    let m = filter_manifest(inputs, encode_artifacts(&art), challenge);
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
