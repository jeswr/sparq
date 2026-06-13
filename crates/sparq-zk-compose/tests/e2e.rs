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
