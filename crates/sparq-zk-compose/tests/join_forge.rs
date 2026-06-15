// [OPUS-4.8] written while Fable 5 unavailable — re-review when Fable returns.
//! sq-hlul (step 5 of sq-bwwl, the hidden cross-credential JOIN): the
//! forge-and-verify REGRESSION SUITE for the hidden join. The adversarial
//! SUPERSET of `join_gates.rs` (the `bind_joins` structural-gate suite) and the
//! manifest schema (`join_gates`/sq-sfsi): here we systematically attempt to
//! FORGE a passing join and assert every attempt is REJECTED, AND — the genuinely
//! new capability over `join_gates.rs` — drive a REAL `join_eq` bb proof to forge
//! the CRYPTOGRAPHIC direction (tampered proof, unsatisfiable forged witness,
//! attacker vk), which the structural-only `join_gates.rs` cannot reach.
//!
//! ## Two tiers (which test runs where) — load-bearing honesty
//! - **Per-PR (fast, NO toolchain):** the STRUCTURAL forge vectors — manifest
//!   tampering that `bind_joins` (inside `prefilter_manifest_structure`) rejects
//!   without `nargo`/`bb`: spurious edge, multi-scan mis-binding, wrong-slot, and
//!   the PRIVACY PIN over the public-input LAYOUT (`reconstruct_public_inputs`'s
//!   JoinEq arm pushes `[challenge, commit_a, commit_b, join_commitment, slot_a,
//!   slot_b]` — the join VALUE is never pushed). These do NOT duplicate
//!   `join_gates.rs`'s commit_a/commit_b/dangling/kind cases; they ADD the
//!   vectors sq-hlul names that `join_gates.rs` did not cover, plus a structural
//!   positive control. They build `bind_joins` rejections with empty `proof_hex`.
//! - **Nightly (`#[ignore]`, REAL bb prove, gated on toolchain):** the
//!   CRYPTOGRAPHIC forge vectors over a real `join_eq_na16_nb16` proof — honest
//!   accept (positive control), tampered proof bytes → bb rejects, forged join
//!   (genuinely-unequal hidden values) → witness UNSATISFIABLE (no proof can be
//!   built), forged `join_commitment` (does not bind the proven value) → witness
//!   unsatisfiable, attacker/non-canonical vk → bb rejects, and the real-proof
//!   PRIVACY PIN (the join value's encoding is absent from the proof's
//!   `public_inputs` bytes). Same `#[ignore]`/`toolchain_available()` convention
//!   as e2e's `full_prove_verify_*` and `full_bb_join_accept_deferred`.
//!
//! ## Honesty notes on the feature's ACTUAL state (sq-bwwl)
//! - `prover_toml_for`'s JoinEq arm STILL returns `Err(JoinEqUnsupported)` on main
//!   (sq-fi03 deferred it; `join_gates::full_bb_join_accept_deferred` documents
//!   this). So this suite renders the `join_eq` `Prover.toml` HERE
//!   (`join_eq_prover_toml`) from the SAME `build_scan` witnesses — it does NOT
//!   rely on the missing host emitter. When sq-r2s8 lands the real
//!   `build_join`/`prover_toml_for` path, this renderer can be replaced by it.
//! - N-way (chain) `JoinCommitmentChainMismatch` is NOT enforced on main —
//!   `bind_joins` is explicitly single pairwise (v1). So there is no
//!   `JoinCommitmentChainMismatch` error variant to assert. We do NOT fake an
//!   N-way reject; instead `nway_chain_not_yet_enforced_documented` pins that
//!   boundary HONESTLY and a follow-up bead carries the chain-enforcement work.

use oxrdf::{Literal, NamedNode, NamedOrBlankNode, Term, Triple};
use sparq_zk::commit::{commit_triples, GraphCommitment};
use sparq_zk::encode::{encode_term, salt_from_bytes};
use sparq_zk::field::Fr;
use sparq_zk::sig::{join_value_commitment, public_key_to_hex, SecretKey, SignatureScheme};
use sparq_zk_compose::build::{build_scan, Pattern, BuiltScan, Slot};
use sparq_zk_compose::driver::CircuitProver;
use sparq_zk_compose::manifest::{
    AttestedStatusRef, BindingMode, CircuitId, CommitmentAttestation, EntailmentRegime, FieldHex,
    JoinEdge, ProofInputs, ProofManifest, RevocationStatus, StatusListSnapshot, SubProof,
};
use sparq_zk_compose::verifier::{
    prefilter_manifest_structure, CheckError, KeySet, RevocationPolicy,
};

const XSD_INT: &str = "http://www.w3.org/2001/XMLSchema#integer";
const STATUS_LIST: &str = "http://ex/status/1";
const STATUS_INDEX: u64 = 3;
const STATUS_VERSION: u64 = 1;
const CHALLENGE_HEX: &str = "0x2a";

fn iri(s: &str) -> NamedNode {
    NamedNode::new(s).unwrap()
}
fn int_lit(v: u64) -> Term {
    Term::Literal(Literal::new_typed_literal(v.to_string(), iri(XSD_INT)))
}
fn test_issuer_sk(seed: u64) -> SecretKey {
    SecretKey::from_seed(seed)
}
fn trusted_k(sk: &SecretKey) -> KeySet {
    KeySet::from_hex_keys([public_key_to_hex(&sk.public_key())])
}
fn fixture_snapshot() -> StatusListSnapshot {
    StatusListSnapshot { status_list: STATUS_LIST.to_string(), version: STATUS_VERSION, bits: vec![0u8] }
}
fn fixture_revocation() -> RevocationStatus {
    RevocationStatus {
        status_list: STATUS_LIST.to_string(),
        index: Some(STATUS_INDEX),
        version: STATUS_VERSION,
        index_commitment: None,
    }
}
fn fresh_policy() -> RevocationPolicy {
    RevocationPolicy::accept_version(STATUS_VERSION).with_snapshot(fixture_snapshot())
}

/// A salt + status-bound attestation over `commitment` (the scan-verify-path shape;
/// mirrors `join_gates::attest_full`).
fn attest_full(commitment: Fr, salt: Fr, sk: &SecretKey) -> CommitmentAttestation {
    let list_id = sparq_zk::sig::status_list_id_to_field(STATUS_LIST);
    let status_ref = sparq_zk::sig::status_ref_digest(&list_id, STATUS_INDEX, STATUS_VERSION);
    CommitmentAttestation {
        commitment: FieldHex::from_field(&commitment),
        issuer_public_key: public_key_to_hex(&sk.public_key()),
        signature: sk.sign_commitment_with_status(&commitment, &salt, &status_ref),
        cryptosuite: SignatureScheme::Poseidon2SchnorrV1.cryptosuite_iri().to_string(),
        salt: Some(FieldHex::from_field(&salt)),
        status: Some(AttestedStatusRef {
            index: Some(STATUS_INDEX),
            version: STATUS_VERSION,
            index_commitment: None,
        }),
        holder: None,
    }
}

// --- Two single-graph credentials sharing the join value `<ex/p>` ------------
//
// Graph A: `<ex/a> <ex/knows> <ex/p>`  (object slot 2 = the join key `<ex/p>`)
// Graph B: `<ex/p> <ex/age> "30"`      (subject slot 0 = the join key `<ex/p>`)
//
// `<ex/p>` is a NAMED node, so `encode_term` is salt-INDEPENDENT (it hashes the
// IRI only): the two graphs' join-slot encodings are GENUINELY equal, so the
// in-circuit `a_val == b_val` is satisfiable for the HONEST manifest. The query's
// shared variable `?p` is at pattern-0 slot 2 and pattern-1 slot 0 => slots (2, 0).

const JOIN_QUERY: &str =
    "SELECT ?a ?p ?o WHERE { ?a <http://ex/knows> ?p . ?p <http://ex/age> ?o }";
const SLOT_A: u32 = 2;
const SLOT_B: u32 = 0;
const SALT_A: [u8; 32] = [31u8; 32];
const SALT_B: [u8; 32] = [32u8; 32];
const SALT_A2: [u8; 32] = [33u8; 32];

fn triple_a() -> Triple {
    Triple::new(
        NamedOrBlankNode::NamedNode(iri("http://ex/a")),
        iri("http://ex/knows"),
        Term::NamedNode(iri("http://ex/p")),
    )
}
fn triple_b() -> Triple {
    Triple::new(
        NamedOrBlankNode::NamedNode(iri("http://ex/p")),
        iri("http://ex/age"),
        int_lit(30),
    )
}
fn triple_a2() -> Triple {
    Triple::new(
        NamedOrBlankNode::NamedNode(iri("http://ex/a2")),
        iri("http://ex/knows"),
        Term::NamedNode(iri("http://ex/p")),
    )
}
fn graph_a() -> GraphCommitment {
    commit_triples(&[triple_a()], salt_from_bytes(&SALT_A)).unwrap()
}
fn graph_b() -> GraphCommitment {
    commit_triples(&[triple_b()], salt_from_bytes(&SALT_B)).unwrap()
}
fn graph_a2() -> GraphCommitment {
    commit_triples(&[triple_a2()], salt_from_bytes(&SALT_A2)).unwrap()
}

/// The encoding of the shared join value `<ex/p>` (salt-independent, named node).
fn join_value_enc() -> Fr {
    encode_term(&Term::NamedNode(iri("http://ex/p")), &Fr::from(0u64)).unwrap()
}

fn scan_artifact(c: &GraphCommitment, pattern: Pattern) -> BuiltScan {
    build_scan(std::slice::from_ref(c), &pattern).expect("scan builds")
}
fn pattern_a() -> Pattern {
    Pattern { s: Slot::Var, p: Slot::Const(Term::NamedNode(iri("http://ex/knows"))), o: Slot::Var }
}
fn pattern_b() -> Pattern {
    Pattern { s: Slot::Var, p: Slot::Const(Term::NamedNode(iri("http://ex/age"))), o: Slot::Var }
}
fn scan_a_inputs(c: &GraphCommitment) -> ProofInputs {
    scan_artifact(c, pattern_a()).inputs
}
fn scan_b_inputs(c: &GraphCommitment) -> ProofInputs {
    scan_artifact(c, pattern_b()).inputs
}
fn commitment_hex(inputs: &ProofInputs) -> FieldHex {
    match inputs {
        ProofInputs::Scan { commitments, .. } => commitments[0].clone(),
        _ => unreachable!("scan inputs"),
    }
}

/// A `join_eq` sub-proof's public inputs (witness-only — empty `proof_hex`).
fn join_eq_inputs(commit_a: FieldHex, commit_b: FieldHex, slot_a: u32, slot_b: u32) -> ProofInputs {
    ProofInputs::JoinEq {
        id: CircuitId::JoinEq { n_a: 16, n_b: 16 },
        commit_a,
        commit_b,
        join_commitment: FieldHex("0x0c".to_string()),
        slot_a,
        slot_b,
    }
}

/// The honest cross-scan join manifest (mirrors `join_gates::join_manifest`).
fn join_manifest() -> ProofManifest {
    let (ca, sa) = { let c = graph_a(); (c.commitment, c.salt) };
    let (cb, sb) = { let c = graph_b(); (c.commitment, c.salt) };
    let scan_a = scan_a_inputs(&graph_a());
    let scan_b = scan_b_inputs(&graph_b());
    let commit_a = commitment_hex(&scan_a);
    let commit_b = commitment_hex(&scan_b);
    let join = join_eq_inputs(commit_a, commit_b, SLOT_A, SLOT_B);
    let sk = test_issuer_sk(1);
    ProofManifest {
        r#type: "urn:sparq:zk:ProofManifest".into(),
        query: JOIN_QUERY.into(),
        issuers: vec![],
        key_set: vec![public_key_to_hex(&sk.public_key())],
        commitment_attestations: vec![attest_full(ca, sa, &sk), attest_full(cb, sb, &sk)],
        attributions: vec![vec![0], vec![0]],
        join_obligations: vec![],
        entailment_regime: EntailmentRegime::Simple,
        derivation_steps: vec![],
        binding: BindingMode::Challenge { challenge: FieldHex(CHALLENGE_HEX.into()) },
        revocation: Some(fixture_revocation()),
        status_snapshots: vec![fixture_snapshot()],
        sub_proofs: vec![
            SubProof { inputs: scan_a, proof_hex: String::new() },
            SubProof { inputs: scan_b, proof_hex: String::new() },
            SubProof { inputs: join, proof_hex: String::new() },
        ],
        binding_edges: vec![],
        join_edges: vec![JoinEdge { scan_a: 0, graph_a: 0, scan_b: 1, graph_b: 0, join_proof: 2 }],
        hidden_revocation: None,
        hidden_issuer_attestations: vec![],
    }
}

fn prefilter(m: &ProofManifest) -> Result<(), CheckError> {
    prefilter_manifest_structure(m, &trusted_k(&test_issuer_sk(1)), &fresh_policy()).map(|_| ())
}

// ============================================================================
// TIER 1 — STRUCTURAL forge vectors (per-PR, no toolchain). The ADVERSARIAL
// SUPERSET of join_gates.rs: vectors sq-hlul names that join_gates did not pin.
// ============================================================================

// --- POSITIVE CONTROL (structural) ------------------------------------------

/// Positive control: the honest cross-scan join passes every structural gate,
/// INCLUDING `bind_joins`. Proves the rejections below are SPECIFIC to the forge,
/// not a blanket rejection of all join manifests.
#[test]
fn structural_honest_join_is_accepted_positive_control() {
    prefilter(&join_manifest())
        .expect("the honest cross-scan join must pass the structural bind_joins gate");
}

// --- SPURIOUS / wrong-slot hidden join (anti-A: forged unrelated join) -------

/// Forge: a SPURIOUS join edge whose `join_eq` slots `(0, 1)` correspond to NO
/// shared query variable's positions over the referenced scans (slot 0 of pattern
/// A is `?a`, slot 1 of pattern B is the constant `<ex/age>` — they are neither
/// the same variable nor both variables). `bind_joins` must REJECT
/// (`JoinSlotMismatch`): a prover cannot inject a join over an unrelated column
/// pair. (Distinct from join_gates' (0,2)-swap: this targets a const slot.)
#[test]
fn structural_forge_spurious_edge_unrelated_columns_rejected() {
    let mut m = join_manifest();
    if let ProofInputs::JoinEq { slot_a, slot_b, .. } = &mut m.sub_proofs[2].inputs {
        *slot_a = 0; // ?a in pattern A
        *slot_b = 1; // the predicate CONSTANT <ex/age> in pattern B (not a var)
    } else {
        unreachable!("join_eq inputs");
    }
    match prefilter(&m) {
        Err(CheckError::JoinSlotMismatch { edge: 0 }) => {}
        other => panic!("a spurious join over unrelated columns must be JoinSlotMismatch, got {other:?}"),
    }
}

/// Forge: the join edge's two scans share a variable, but the declared join is on
/// a slot pair where the variables DIFFER (`slot_a = 0 = ?a`, `slot_b = 0 = ?p`:
/// `?a != ?p`). The slot binding requires the SAME variable at both slots =>
/// REJECT (`JoinSlotMismatch`).
#[test]
fn structural_forge_join_on_distinct_variables_rejected() {
    let mut m = join_manifest();
    if let ProofInputs::JoinEq { slot_a, slot_b, .. } = &mut m.sub_proofs[2].inputs {
        *slot_a = 0; // ?a
        *slot_b = 0; // ?p — a DIFFERENT variable
    } else {
        unreachable!("join_eq inputs");
    }
    match prefilter(&m) {
        Err(CheckError::JoinSlotMismatch { edge: 0 }) => {}
        other => panic!("a join across two DISTINCT variables must be JoinSlotMismatch, got {other:?}"),
    }
}

// --- MULTI-SCAN mis-binding --------------------------------------------------

/// Build a multi-scan-per-pattern manifest: pattern A is answered by TWO scans
/// (graph_a, graph_a2). The honest edge references graph_a2 (the SECOND).
fn multi_scan_manifest() -> ProofManifest {
    let (ca, sa) = { let c = graph_a(); (c.commitment, c.salt) };
    let (ca2, sa2) = { let c = graph_a2(); (c.commitment, c.salt) };
    let (cb, sb) = { let c = graph_b(); (c.commitment, c.salt) };
    let scan_a = scan_a_inputs(&graph_a());
    let scan_a2 = scan_a_inputs(&graph_a2());
    let scan_b = scan_b_inputs(&graph_b());
    let commit_a2 = commitment_hex(&scan_a2);
    let commit_b = commitment_hex(&scan_b);
    let join = join_eq_inputs(commit_a2, commit_b, SLOT_A, SLOT_B);
    let sk = test_issuer_sk(1);
    ProofManifest {
        r#type: "urn:sparq:zk:ProofManifest".into(),
        query: JOIN_QUERY.into(),
        issuers: vec![],
        key_set: vec![public_key_to_hex(&sk.public_key())],
        commitment_attestations: vec![
            attest_full(ca, sa, &sk),
            attest_full(ca2, sa2, &sk),
            attest_full(cb, sb, &sk),
        ],
        attributions: vec![vec![0], vec![0]],
        join_obligations: vec![],
        entailment_regime: EntailmentRegime::Simple,
        derivation_steps: vec![],
        binding: BindingMode::Challenge { challenge: FieldHex(CHALLENGE_HEX.into()) },
        revocation: Some(fixture_revocation()),
        status_snapshots: vec![fixture_snapshot()],
        sub_proofs: vec![
            SubProof { inputs: scan_a, proof_hex: String::new() },
            SubProof { inputs: scan_a2, proof_hex: String::new() },
            SubProof { inputs: scan_b, proof_hex: String::new() },
            SubProof { inputs: join, proof_hex: String::new() },
        ],
        binding_edges: vec![],
        join_edges: vec![JoinEdge { scan_a: 1, graph_a: 0, scan_b: 2, graph_b: 0, join_proof: 3 }],
        hidden_revocation: None,
        hidden_issuer_attestations: vec![],
    }
}

/// Positive control for the multi-scan path: an edge pointing at the SECOND scan
/// answering pattern A passes (the membership-based binding, not first-match).
#[test]
fn structural_multi_scan_second_scan_edge_accepted_positive_control() {
    prefilter(&multi_scan_manifest())
        .expect("a join edge referencing the second scan answering a pattern must pass bind_joins");
}

/// Forge (multi-scan mis-binding): re-point the edge's `scan_a` at scan_b
/// (sub-proof 2), a scan that answers pattern B, NOT pattern A at slot_a — and set
/// the join_eq's `commit_a` to scan_b's commitment so the commitment gate passes
/// and we EXERCISE the slot gate. `bind_joins` binds against the SPECIFIC named
/// scan, so it finds no pattern answered by `scan_a` carrying `?p` at SLOT_A =>
/// REJECT (`JoinSlotMismatch`). Pins that the membership fix did not weaken the
/// binding: the edge must reference a scan that genuinely answers the right
/// pattern at the right slot, not merely SOME scan.
#[test]
fn structural_forge_multi_scan_edge_points_at_non_answering_scan_rejected() {
    let mut m = multi_scan_manifest();
    m.join_edges[0].scan_a = 2; // scan_b answers pattern B, not A
    let commit_b = match &m.sub_proofs[2].inputs {
        ProofInputs::Scan { commitments, .. } => commitments[0].clone(),
        _ => unreachable!("scan_b inputs"),
    };
    if let ProofInputs::JoinEq { commit_a, .. } = &mut m.sub_proofs[3].inputs {
        *commit_a = commit_b;
    } else {
        unreachable!("join_eq inputs");
    }
    match prefilter(&m) {
        Err(CheckError::JoinSlotMismatch { edge: 0 }) => {}
        other => panic!(
            "an edge whose scan_a does not answer pattern A at slot_a must be JoinSlotMismatch, got {other:?}"
        ),
    }
}

// --- PRIVACY PIN (structural, over the public-input LAYOUT) ------------------

/// PRIVACY REGRESSION PIN (the hiding property). The `join_eq` public-input
/// LAYOUT (documented as `[challenge, commit_a, commit_b, join_commitment,
/// slot_a, slot_b]` and emitted by `reconstruct_public_inputs`'s JoinEq arm) does
/// NOT carry the join VALUE — only the two graph commitments, the HIDING
/// `join_commitment`, and the two query-bound slots. We pin this at the manifest
/// level: the public `ProofInputs::JoinEq` fields a verifier reconstructs are
/// EXACTLY {commit_a, commit_b, join_commitment, slot_a, slot_b}; the joined term
/// encoding (`<ex/p>`) appears in NONE of the FIELD-valued public inputs. A future
/// change that leaked the value into a public input (e.g. adding an `enc`/`value`
/// public field) would flip this assertion. The real-proof analogue
/// (`bb_join_value_absent_from_public_inputs`, nightly) pins the SAME property at
/// the bb-bytes level.
#[test]
fn structural_privacy_join_value_absent_from_public_inputs() {
    let m = join_manifest();
    let ProofInputs::JoinEq { commit_a, commit_b, join_commitment, .. } =
        &m.sub_proofs[2].inputs
    else {
        unreachable!("join_eq inputs");
    };
    let value = join_value_enc();
    let value_hex = FieldHex::from_field(&value);
    // The join value's canonical-field encoding must not equal ANY field-valued
    // public input. (`join_commitment` is `h3(DOMAIN, value, blinding)` — a hiding
    // image of the value, never the value itself.)
    for (name, public) in [
        ("commit_a", commit_a),
        ("commit_b", commit_b),
        ("join_commitment", join_commitment),
    ] {
        assert_ne!(
            public.to_field(),
            Some(value),
            "PRIVACY REGRESSION: the join value leaked into the public input `{name}`",
        );
    }
    // And the join_commitment is specifically NOT a bare encoding of the value —
    // it is the hiding image, which differs from `value_hex`.
    assert_ne!(
        join_commitment, &value_hex,
        "PRIVACY REGRESSION: join_commitment must be hiding, not a bare value encoding",
    );
    // Sanity: the value really is the shared term (so the pin is meaningful).
    let row_a = scan_join_row(&graph_a(), pattern_a(), SLOT_A as usize);
    assert_eq!(row_a.to_field(), Some(value), "fixture: join slot A is <ex/p>");
}

// --- HONESTY BOUNDARY: N-way chain not yet enforced --------------------------

/// HONESTY pin (NOT a forge that asserts a reject we cannot actually produce).
/// sq-hlul names an "N-way divergence → `JoinCommitmentChainMismatch`" vector, but
/// `bind_joins` on main is EXPLICITLY single pairwise (v1) and has NO
/// `JoinCommitmentChainMismatch` error variant — the verifier docs state N-way
/// chain commitment-equality is "NOT yet enforced here". So a manifest with TWO
/// join edges over the same variable carrying DIFFERENT `join_commitment`s is NOT
/// rejected by `bind_joins` today (each edge passes its own pairwise checks). We
/// pin this boundary HONESTLY rather than asserting a non-existent reject; the
/// follow-up bead carries the chain-enforcement work + this test's upgrade to an
/// actual `JoinCommitmentChainMismatch` assertion.
#[test]
fn structural_nway_chain_divergence_not_yet_enforced_documented() {
    // Build a manifest whose single edge passes; adding a second divergent edge
    // over the same variable would NOT be caught by the pairwise gate today.
    // (We assert the documented-pairwise behaviour so a future N-way enforcement
    // that changes it trips this test, forcing the assertion to be upgraded.)
    let m = join_manifest();
    assert_eq!(m.join_edges.len(), 1, "v1 join is single pairwise");
    // The pairwise honest edge passes; there is no chain gate to fail.
    prefilter(&m).expect("single pairwise join passes (N-way chain gate is a follow-up)");
}

// ============================================================================
// TIER 2 — CRYPTOGRAPHIC forge vectors (nightly, #[ignore], REAL bb prove).
// These render a real `join_eq_na16_nb16` Prover.toml (the host emitter is
// deferred, sq-fi03), drive nargo/bb, and assert the CRYPTOGRAPHIC direction.
// ============================================================================

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
    let dir = std::env::temp_dir().join(format!("sparq_zk_join_forge_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// The join slot value (a `FieldHex`) of a graph's single triple under `pattern`,
/// read from the scan witness's per-slot encoding (graph 0, row 0).
fn scan_join_row(c: &GraphCommitment, pattern: Pattern, slot: usize) -> FieldHex {
    let art = scan_artifact(c, pattern);
    art.witness.enc[0][0][slot].clone()
}

/// A `[[Field;3];16]` enc array, TOML-rendered, padding rows 1..16 with zeros.
fn enc16_toml(graph: &[[FieldHex; 3]]) -> String {
    let mut rows: Vec<String> = Vec::with_capacity(16);
    for i in 0..16 {
        let row = graph.get(i);
        let cells = match row {
            Some(r) => format!("\"{}\", \"{}\", \"{}\"", r[0].0, r[1].0, r[2].0),
            None => "\"0x0\", \"0x0\", \"0x0\"".to_string(),
        };
        rows.push(format!("[{cells}]"));
    }
    format!("[{}]", rows.join(", "))
}

/// Render a `join_eq_na16_nb16` `Prover.toml`. Declaration order MUST match
/// `zk/compose/join_eq_na16_nb16/src/main.nr`: challenge, commit_a, commit_b,
/// join_commitment, slot_a, slot_b (public) then enc_a, counts_a, enc_b,
/// counts_b, row_a, row_b, blinding (private). This is the renderer the deferred
/// host `prover_toml_for` JoinEq arm will eventually provide (sq-r2s8).
#[allow(clippy::too_many_arguments)]
fn join_eq_prover_toml(
    challenge: &str,
    commit_a: &FieldHex,
    commit_b: &FieldHex,
    join_commitment: &FieldHex,
    slot_a: u32,
    slot_b: u32,
    enc_a: &[[FieldHex; 3]],
    counts_a: u32,
    enc_b: &[[FieldHex; 3]],
    counts_b: u32,
    row_a: &[FieldHex; 3],
    row_b: &[FieldHex; 3],
    blinding: &FieldHex,
) -> String {
    let mut s = String::new();
    s.push_str(&format!("challenge = \"{challenge}\"\n"));
    s.push_str(&format!("commit_a = \"{}\"\n", commit_a.0));
    s.push_str(&format!("commit_b = \"{}\"\n", commit_b.0));
    s.push_str(&format!("join_commitment = \"{}\"\n", join_commitment.0));
    s.push_str(&format!("slot_a = \"{slot_a}\"\n"));
    s.push_str(&format!("slot_b = \"{slot_b}\"\n"));
    s.push_str(&format!("enc_a = {}\n", enc16_toml(enc_a)));
    s.push_str(&format!("counts_a = \"{counts_a}\"\n"));
    s.push_str(&format!("enc_b = {}\n", enc16_toml(enc_b)));
    s.push_str(&format!("counts_b = \"{counts_b}\"\n"));
    s.push_str(&format!("row_a = [\"{}\", \"{}\", \"{}\"]\n", row_a[0].0, row_a[1].0, row_a[2].0));
    s.push_str(&format!("row_b = [\"{}\", \"{}\", \"{}\"]\n", row_b[0].0, row_b[1].0, row_b[2].0));
    s.push_str(&format!("blinding = \"{}\"\n", blinding.0));
    s
}

/// The witness pieces (enc/counts/row per graph) for the HONEST join.
struct JoinWitness {
    commit_a: FieldHex,
    commit_b: FieldHex,
    enc_a: Vec<[FieldHex; 3]>,
    counts_a: u32,
    enc_b: Vec<[FieldHex; 3]>,
    counts_b: u32,
    row_a: [FieldHex; 3],
    row_b: [FieldHex; 3],
    value: Fr,
}

fn honest_join_witness() -> JoinWitness {
    let ga = graph_a();
    let gb = graph_b();
    let art_a = scan_artifact(&ga, pattern_a());
    let art_b = scan_artifact(&gb, pattern_b());
    JoinWitness {
        commit_a: FieldHex::from_field(&ga.commitment),
        commit_b: FieldHex::from_field(&gb.commitment),
        enc_a: art_a.witness.enc[0].clone(),
        counts_a: art_a.witness.counts[0],
        enc_b: art_b.witness.enc[0].clone(),
        counts_b: art_b.witness.counts[0],
        row_a: art_a.witness.enc[0][0].clone(),
        row_b: art_b.witness.enc[0][0].clone(),
        value: join_value_enc(),
    }
}

const JOIN_ID: CircuitId = CircuitId::JoinEq { n_a: 16, n_b: 16 };
const BLINDING_HEX: &str = "0x07";

/// CRYPTOGRAPHIC POSITIVE CONTROL: a real `join_eq` proof over the honest witness
/// VERIFIES against its own (canonical) vk. Establishes that the rejects below are
/// specific. (Full-bb; un-ignores the capability `full_bb_join_accept_deferred`
/// documented as missing in join_gates.rs — sq-hlul provides the renderer.)
#[test]
#[ignore = "nightly: full bb prove of a join_eq member (nargo + bb)"]
fn bb_honest_join_proof_verifies_positive_control() {
    if !toolchain_available() {
        eprintln!("nargo/bb absent; skipping real-bb join accept");
        return;
    }
    let w = honest_join_witness();
    let blinding = Fr::from(7u64);
    let jc = join_value_commitment(&w.value, &blinding);
    let toml = join_eq_prover_toml(
        CHALLENGE_HEX, &w.commit_a, &w.commit_b, &FieldHex::from_field(&jc), SLOT_A, SLOT_B,
        &w.enc_a, w.counts_a, &w.enc_b, w.counts_b, &w.row_a, &w.row_b, &FieldHex(BLINDING_HEX.into()),
    );
    let prover = CircuitProver::from_crate_root();
    let out = scratch("join_accept");
    let art = prover
        .prove_in(&JOIN_ID, &toml, &out, "join_accept")
        .expect("honest join witness must prove");
    assert!(!art.proof.is_empty(), "a real proof was produced");
    // Verify against the verifier-recomputed CANONICAL vk (audit #2 discipline) +
    // the prover's public inputs.
    let vk = prover.canonical_vk(&JOIN_ID, &out.join("cvk")).expect("canonical vk");
    let ok = prover
        .verify_with(&art.proof, &art.public_inputs, &vk, &out.join("verify"))
        .expect("verify runs");
    assert!(ok, "the honest join proof must verify under the canonical vk");
}

/// FORGE — TAMPERED PROOF: flip a byte in a valid `join_eq` proof => bb REJECTS.
#[test]
#[ignore = "nightly: full bb prove of a join_eq member (nargo + bb)"]
fn bb_forge_tampered_join_proof_rejected() {
    if !toolchain_available() {
        eprintln!("nargo/bb absent; skipping tampered-proof reject");
        return;
    }
    let w = honest_join_witness();
    let blinding = Fr::from(7u64);
    let jc = join_value_commitment(&w.value, &blinding);
    let toml = join_eq_prover_toml(
        CHALLENGE_HEX, &w.commit_a, &w.commit_b, &FieldHex::from_field(&jc), SLOT_A, SLOT_B,
        &w.enc_a, w.counts_a, &w.enc_b, w.counts_b, &w.row_a, &w.row_b, &FieldHex(BLINDING_HEX.into()),
    );
    let prover = CircuitProver::from_crate_root();
    let out = scratch("join_tamper");
    let art = prover.prove_in(&JOIN_ID, &toml, &out, "join_tamper").expect("prove succeeds");
    let vk = prover.canonical_vk(&JOIN_ID, &out.join("cvk")).expect("canonical vk");
    let mut bad = art.proof.clone();
    let mid = bad.len() / 2;
    bad[mid] ^= 0xff;
    let rejected = prover
        .verify_with(&bad, &art.public_inputs, &vk, &out.join("verify_bad"))
        .expect("verify runs");
    assert!(!rejected, "a tampered join proof must be rejected by bb");
}

/// FORGE — UNEQUAL HIDDEN VALUES: claim a join between two graphs whose join-slot
/// values are GENUINELY UNEQUAL. The in-circuit `assert(a_val == b_val)` has no
/// satisfying witness => `nargo execute` produces NO witness (the relation is
/// UNSATISFIABLE). A passing join over a false equality is UNCONSTRUCTIBLE.
#[test]
#[ignore = "nightly: nargo execute of a join_eq member"]
fn bb_forge_unequal_values_witness_unsatisfiable() {
    if !toolchain_available() {
        eprintln!("nargo absent; skipping unequal-values unsat");
        return;
    }
    // Graph B' has a DIFFERENT subject `<ex/q>` — so its join-slot value (slot 0)
    // is `Enc(<ex/q>) != Enc(<ex/p>)`. The prover claims they join.
    let gb_prime = commit_triples(
        &[Triple::new(
            NamedOrBlankNode::NamedNode(iri("http://ex/q")),
            iri("http://ex/age"),
            int_lit(30),
        )],
        salt_from_bytes(&[44u8; 32]),
    )
    .unwrap();
    let art_b = scan_artifact(&gb_prime, pattern_b());
    let w = honest_join_witness();
    // join_commitment binds a_val (the honest A value), but b_val differs => the
    // equality assert fails regardless of which value we commit.
    let blinding = Fr::from(7u64);
    let jc = join_value_commitment(&w.value, &blinding);
    let toml = join_eq_prover_toml(
        CHALLENGE_HEX,
        &w.commit_a,
        &FieldHex::from_field(&gb_prime.commitment),
        &FieldHex::from_field(&jc),
        SLOT_A,
        SLOT_B,
        &w.enc_a,
        w.counts_a,
        &art_b.witness.enc[0],
        art_b.witness.counts[0],
        &w.row_a,
        &art_b.witness.enc[0][0],
        &FieldHex(BLINDING_HEX.into()),
    );
    let prover = CircuitProver::from_crate_root();
    prover.compile(&JOIN_ID).expect("compiles");
    let res = prover.gen_witness_tagged(&JOIN_ID, &toml, "join_unequal");
    assert!(
        res.is_err(),
        "a join over genuinely-unequal hidden values must be UNSATISFIABLE (no witness)",
    );
}

/// FORGE — FORGED JOIN COMMITMENT: the public `join_commitment` does NOT bind the
/// proven join value (it commits to a DIFFERENT value). The in-circuit
/// `assert_eq(join_commitment, join_value_commitment(a_val, blinding))` fails =>
/// witness UNSATISFIABLE. Locks the anti-equivocation binding (design §2.4/§4.3).
#[test]
#[ignore = "nightly: nargo execute of a join_eq member"]
fn bb_forge_join_commitment_not_binding_value_unsatisfiable() {
    if !toolchain_available() {
        eprintln!("nargo absent; skipping forged-commitment unsat");
        return;
    }
    let w = honest_join_witness();
    // Commit to a DIFFERENT value than the proven join value `<ex/p>`.
    let wrong_value = encode_term(&Term::NamedNode(iri("http://ex/OTHER")), &Fr::from(0u64)).unwrap();
    let blinding = Fr::from(7u64);
    let forged_jc = join_value_commitment(&wrong_value, &blinding);
    let toml = join_eq_prover_toml(
        CHALLENGE_HEX, &w.commit_a, &w.commit_b, &FieldHex::from_field(&forged_jc), SLOT_A, SLOT_B,
        &w.enc_a, w.counts_a, &w.enc_b, w.counts_b, &w.row_a, &w.row_b, &FieldHex(BLINDING_HEX.into()),
    );
    let prover = CircuitProver::from_crate_root();
    prover.compile(&JOIN_ID).expect("compiles");
    let res = prover.gen_witness_tagged(&JOIN_ID, &toml, "join_forged_commit");
    assert!(
        res.is_err(),
        "a join_commitment that does not bind the proven value must be UNSATISFIABLE",
    );
}

/// FORGE — ATTACKER VK: verify a valid join proof against a NON-canonical
/// (attacker-chosen) vk => bb REJECTS. Pins audit #2 (the verifier recomputes the
/// canonical vk; a prover-supplied / wrong vk does not verify the proof).
#[test]
#[ignore = "nightly: full bb prove of a join_eq member (nargo + bb)"]
fn bb_forge_wrong_vk_rejected() {
    if !toolchain_available() {
        eprintln!("nargo/bb absent; skipping wrong-vk reject");
        return;
    }
    let w = honest_join_witness();
    let blinding = Fr::from(7u64);
    let jc = join_value_commitment(&w.value, &blinding);
    let toml = join_eq_prover_toml(
        CHALLENGE_HEX, &w.commit_a, &w.commit_b, &FieldHex::from_field(&jc), SLOT_A, SLOT_B,
        &w.enc_a, w.counts_a, &w.enc_b, w.counts_b, &w.row_a, &w.row_b, &FieldHex(BLINDING_HEX.into()),
    );
    let prover = CircuitProver::from_crate_root();
    let out = scratch("join_wrongvk");
    let art = prover.prove_in(&JOIN_ID, &toml, &out, "join_wrongvk").expect("prove succeeds");
    // An ATTACKER vk = the vk of a DIFFERENT circuit member (a scan). Verifying the
    // join proof against it must fail.
    let scan_id = derive_smallest_scan_id();
    let wrong_vk = prover
        .canonical_vk(&scan_id, &out.join("wrong_vk"))
        .expect("a different member's vk");
    let rejected = prover
        .verify_with(&art.proof, &art.public_inputs, &wrong_vk, &out.join("verify_wrongvk"))
        .expect("verify runs");
    assert!(!rejected, "a join proof verified against a non-canonical (attacker) vk must be rejected");
}

/// The smallest compiled scan member id (for the wrong-vk attacker case).
fn derive_smallest_scan_id() -> CircuitId {
    sparq_zk_compose::build::derive_scan_id(1, 1, 1).expect("a smallest scan member exists")
}

/// CRYPTOGRAPHIC PRIVACY PIN: the join VALUE's encoding must be ABSENT from the
/// REAL proof's `public_inputs` bytes. The public inputs are exactly
/// `[challenge, commit_a, commit_b, join_commitment, slot_a, slot_b]` (32-byte
/// big-endian field words); the join value `Enc(<ex/p>)` is a PRIVATE witness and
/// its 32-byte word must not appear among them. Locks the hiding property at the
/// bb-bytes level (the headline privacy win).
#[test]
#[ignore = "nightly: full bb prove of a join_eq member (nargo + bb)"]
fn bb_join_value_absent_from_public_inputs() {
    if !toolchain_available() {
        eprintln!("nargo/bb absent; skipping real-proof privacy pin");
        return;
    }
    let w = honest_join_witness();
    let blinding = Fr::from(7u64);
    let jc = join_value_commitment(&w.value, &blinding);
    let toml = join_eq_prover_toml(
        CHALLENGE_HEX, &w.commit_a, &w.commit_b, &FieldHex::from_field(&jc), SLOT_A, SLOT_B,
        &w.enc_a, w.counts_a, &w.enc_b, w.counts_b, &w.row_a, &w.row_b, &FieldHex(BLINDING_HEX.into()),
    );
    let prover = CircuitProver::from_crate_root();
    let out = scratch("join_privacy");
    let art = prover.prove_in(&JOIN_ID, &toml, &out, "join_privacy").expect("prove succeeds");
    // The 32-byte big-endian word of the join value must not appear in the public
    // inputs blob. (bb serialises each public field as a 32-byte BE word.)
    let value_word = field_to_be_bytes(&w.value);
    let present = art
        .public_inputs
        .windows(32)
        .any(|w32| w32 == value_word.as_slice());
    assert!(
        !present,
        "PRIVACY REGRESSION: the join value's 32-byte field word appears in the bb public inputs",
    );
}

/// 32-byte big-endian encoding of a field element (matches bb's public-input word
/// serialisation). Re-derived locally from the field's canonical bytes.
fn field_to_be_bytes(f: &Fr) -> [u8; 32] {
    let hex = FieldHex::from_field(f).0; // "0x..." canonical big-endian, no left pad guarantee
    let raw = hex.trim_start_matches("0x");
    let mut bytes = vec![0u8; 32];
    // right-align the hex nibbles into a 32-byte big-endian buffer.
    let padded = format!("{raw:0>64}");
    for i in 0..32 {
        bytes[i] = u8::from_str_radix(&padded[i * 2..i * 2 + 2], 16).unwrap();
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    out
}
