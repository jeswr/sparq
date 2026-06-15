// [OPUS-4.8] written while Fable 5 unavailable — re-review when Fable returns.
//! sq-sfsi: the `bind_joins` verifier-gate soundness suite (step 4 of sq-bwwl,
//! the hidden cross-credential JOIN). The adversarial complement to the join
//! manifest schema (sq-fi03): construct a join manifest that violates EXACTLY one
//! of `bind_joins`'s enforcement points, then assert the verifier REJECTS it with
//! the right `CheckError`.
//!
//! The `bind_joins` gate enforces THREE properties (see `verifier::bind_joins`):
//! 1. **Commitment-matching (anti-A2):** the `join_eq` proof's public
//!    `commit_a`/`commit_b` must byte-equal the two referenced scans' bound
//!    `commitments[graph_*]` — a join cannot bind rows from unrelated/forged scans.
//! 2. **Canonical VK (anti-A1):** the `join_eq` proof is verified against the
//!    canonical vk in `verify_manifest`'s per-sub-proof loop (audit-#2). Exercised
//!    on the FULL bb path (toolchain-gated, `#[ignore]`).
//! 3. **UnboundJoin / slot binding:** the `join_eq`'s public `slot_a`/`slot_b` must
//!    equal the query-derived slots a variable SHARED across the two answered
//!    patterns occupies — a join over the wrong column, OR a SPURIOUS join over an
//!    unrelated scan pair, is rejected (`JoinSlotMismatch`).
//!
//! ## Disclosed-vs-hidden scope (load-bearing honesty)
//! `bind_joins` rigorously validates every DECLARED hidden `JoinEdge`. It does NOT
//! *demand* a hidden join for every query cross-scan shared variable: such a
//! variable can ALSO be discharged by the DISCLOSED-row path (the existing
//! `join_obligations`/`verify::recheck` non-bnode obligation gate + the disclosed
//! scan rows), which is the default and is verified SEPARATELY. The hidden
//! `JoinEdge` is the OPT-IN privacy alternative. So "drop the hidden edge" is NOT a
//! soundness hole here — it falls back to the disclosed path (which `recheck` still
//! enforces); demanding a hidden join would WRONGLY break disclosed joins. This
//! suite asserts that boundary explicitly (`drop_edge_falls_back_to_disclosed_path`).
//!
//! These structural checks run inside `prefilter_manifest_structure` (the same
//! structural stage `verify_manifest` runs first), so the reject paths — the
//! security-critical direction — run in default CI WITHOUT the nargo/bb toolchain.
//!
//! ## Accept-proof honesty
//! The `join_eq` PROVING path (`prover_toml_for`'s JoinEq arm) is deferred (sq-fi03
//! left it `Err`), so this suite CANNOT generate a real `join_eq` bb proof in-test.
//! The structural ACCEPT (a well-formed join edge passes `bind_joins` — commitments
//! match, slots match, the demanded join is covered) IS exercised, witness-only,
//! through `prefilter_manifest_structure`. A FULL bb accept (real proof + canonical
//! vk + nonce binding) is marked `#[ignore]` pending the join_eq proving path
//! (tracked: see the join-proving bead referenced in the PR). We do NOT claim the
//! bb accept path works until that proof can be produced.

use oxrdf::{Literal, NamedNode, NamedOrBlankNode, Term, Triple};
use sparq_zk::commit::{commit_triples, GraphCommitment};
use sparq_zk::encode::salt_from_bytes;
use sparq_zk::field::Fr;
use sparq_zk::sig::{public_key_to_hex, SecretKey, SignatureScheme};
use sparq_zk_compose::build::{build_scan, Pattern, Slot};
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

/// A salt + status-bound attestation over `commitment` (the scan-verify-path shape).
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

// --- Two single-graph credentials sharing a join value `<ex/p>` --------------
//
// Graph A: `<ex/a> <ex/knows> <ex/p>`  (object slot 2 = the join key `<ex/p>`)
// Graph B: `<ex/p> <ex/age> "30"`      (subject slot 0 = the join key `<ex/p>`)
//
// Query: `{ ?a <ex/knows> ?p . ?p <ex/age> ?o }` — the shared variable `?p` is at
// pattern-0 slot 2 (object) and pattern-1 slot 0 (subject). The two patterns carry
// distinct predicate constants, so two DISTINCT scans answer them => a cross-scan
// join the gate demands a JoinEdge for. The join SLOTS are (2, 0).

const JOIN_QUERY: &str =
    "SELECT ?a ?p ?o WHERE { ?a <http://ex/knows> ?p . ?p <http://ex/age> ?o }";
const SLOT_A: u32 = 2; // ?p is the OBJECT of `?a <ex/knows> ?p`
const SLOT_B: u32 = 0; // ?p is the SUBJECT of `?p <ex/age> ?o`

fn graph_a() -> GraphCommitment {
    let g = vec![Triple::new(
        NamedOrBlankNode::NamedNode(iri("http://ex/a")),
        iri("http://ex/knows"),
        Term::NamedNode(iri("http://ex/p")),
    )];
    commit_triples(&g, salt_from_bytes(&[31u8; 32])).unwrap()
}
fn graph_b() -> GraphCommitment {
    let g = vec![Triple::new(
        NamedOrBlankNode::NamedNode(iri("http://ex/p")),
        iri("http://ex/age"),
        int_lit(30),
    )];
    commit_triples(&g, salt_from_bytes(&[32u8; 32])).unwrap()
}

/// A SECOND credential answering pattern A (`?a <ex/knows> ?p`), from a different
/// committed graph (distinct salt => distinct commitment). Used to construct a
/// multi-scan-per-pattern manifest: TWO scans legitimately answer pattern A, the
/// same triple pattern satisfied by two different credentials. [OPUS-4.8] sq-sfsi.
fn graph_a2() -> GraphCommitment {
    let g = vec![Triple::new(
        NamedOrBlankNode::NamedNode(iri("http://ex/a2")),
        iri("http://ex/knows"),
        Term::NamedNode(iri("http://ex/p")),
    )];
    commit_triples(&g, salt_from_bytes(&[33u8; 32])).unwrap()
}

fn scan_inputs(c: &GraphCommitment, pattern: Pattern) -> ProofInputs {
    build_scan(std::slice::from_ref(c), &pattern).expect("scan builds").inputs
}

/// The scan over pattern A (`?a <ex/knows> ?p`).
fn scan_a_inputs(c: &GraphCommitment) -> ProofInputs {
    scan_inputs(c, Pattern {
        s: Slot::Var,
        p: Slot::Const(Term::NamedNode(iri("http://ex/knows"))),
        o: Slot::Var,
    })
}
/// The scan over pattern B (`?p <ex/age> ?o`).
fn scan_b_inputs(c: &GraphCommitment) -> ProofInputs {
    scan_inputs(c, Pattern {
        s: Slot::Var,
        p: Slot::Const(Term::NamedNode(iri("http://ex/age"))),
        o: Slot::Var,
    })
}

fn commitment_hex(inputs: &ProofInputs) -> FieldHex {
    match inputs {
        ProofInputs::Scan { commitments, .. } => commitments[0].clone(),
        _ => unreachable!("scan inputs"),
    }
}

/// A `join_eq` sub-proof's inputs (witness-only — empty proof_hex) whose public
/// `commit_a`/`commit_b` are the two scans' commitments and whose slots are
/// `(slot_a, slot_b)`. `join_commitment` is an arbitrary hiding value (the gate
/// never opens it; only the bb stage binds it).
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

/// Build the honest cross-scan join manifest. sub_proofs = [scan_a, scan_b, join_eq];
/// join_edges = [{scan_a:0, graph_a:0, scan_b:1, graph_b:0, join_proof:2}]. The
/// join_eq's commit_a/commit_b match the two scans; its slots are (SLOT_A, SLOT_B).
/// `tweak` lets a forge mutate exactly one field before verification.
fn join_manifest() -> ProofManifest {
    let (ca, sa) = {
        let c = graph_a();
        (c.commitment, c.salt)
    };
    let (cb, sb) = {
        let c = graph_b();
        (c.commitment, c.salt)
    };
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
        // Two patterns; each draws from its own (single) committed graph.
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

/// Run a manifest through the STRUCTURAL prefilter (the stage `verify_manifest`
/// runs first, where `bind_joins` lives). No bb — so the structural join gates are
/// reached without the toolchain.
fn prefilter(m: &ProofManifest) -> Result<(), CheckError> {
    prefilter_manifest_structure(m, &trusted_k(&test_issuer_sk(1)), &fresh_policy()).map(|_| ())
}

// === ACCEPT (structural, witness-only) ======================================

/// A well-formed join edge — commitments match the referenced scans, slots match
/// the query, and the demanded cross-scan join is covered — passes every
/// structural gate, INCLUDING `bind_joins`. (Witness-only: no bb proof, so the
/// FULL `verify_manifest` would then hit MissingProof; the structural prefilter is
/// the layer `bind_joins` runs in, so passing it proves the gate ACCEPTS honest
/// joins — the gate's completeness.)
#[test]
fn honest_join_passes_structural_bind_joins() {
    let m = join_manifest();
    prefilter(&m).expect("honest cross-scan join must pass the structural bind_joins gate");
}

// === ACCEPT (multi-scan-per-pattern): a pattern answered by TWO scans ========
//
// sq-sfsi soundness fix. A query pattern may LEGITIMATELY be answered by more than
// one scan (the same triple pattern satisfied by two different credentials). The
// old `bind_joins` used `position(..)` (FIRST match), which would (a) REJECT a
// valid join whose edge points at the SECOND scan answering the pattern, or worse
// (b) let scan ordering decide which scan the slot binding validates against. The
// fix binds the slot check to the SPECIFIC `edge.scan_a`/`edge.scan_b`, so the join
// is validated against the referenced scan regardless of `sub_proofs` order.

/// Build a multi-scan-per-pattern manifest. sub_proofs =
/// [scan_a (graph_a), scan_a2 (graph_a2, SAME pattern A), scan_b, join_eq].
/// The join edge references scan_a2 (index 1) — the SECOND scan answering pattern
/// A — for `scan_a`, and scan_b (index 2). The join_eq carries scan_a2's commitment.
fn multi_scan_manifest() -> ProofManifest {
    let (ca, sa) = {
        let c = graph_a();
        (c.commitment, c.salt)
    };
    let (ca2, sa2) = {
        let c = graph_a2();
        (c.commitment, c.salt)
    };
    let (cb, sb) = {
        let c = graph_b();
        (c.commitment, c.salt)
    };
    let scan_a = scan_a_inputs(&graph_a());
    let scan_a2 = scan_a_inputs(&graph_a2());
    let scan_b = scan_b_inputs(&graph_b());
    // The join binds scan_a2 (the SECOND scan answering pattern A) to scan_b.
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
        // scan_a -> sub-proof 1 (scan_a2, the SECOND scan answering pattern A),
        // scan_b -> sub-proof 2, join_proof -> sub-proof 3.
        join_edges: vec![JoinEdge { scan_a: 1, graph_a: 0, scan_b: 2, graph_b: 0, join_proof: 3 }],
        hidden_revocation: None,
        hidden_issuer_attestations: vec![],
    }
}

/// A join edge pointing at the SECOND of two scans answering a pattern passes the
/// structural `bind_joins` gate. With the old first-match `position(..)` this was
/// WRONGLY rejected (`scan_for_pattern(pattern_a)` resolved to scan-0, not the
/// referenced scan-1). The membership-based fix accepts it. [OPUS-4.8] sq-sfsi.
#[test]
fn multi_scan_join_edge_points_at_second_scan_passes() {
    let m = multi_scan_manifest();
    prefilter(&m).expect(
        "a join edge referencing the SECOND scan answering a pattern must pass bind_joins \
         (multi-scan-per-pattern is a legitimate configuration)",
    );
}

/// Anti-forgery preserved under multi-scan: re-point the edge's `scan_a` at scan_b
/// (sub-proof 2), which does NOT answer pattern A at slot_a. The slot binding has
/// no pattern answered by the referenced scan with `?p` at SLOT_A => REJECT
/// (`JoinSlotMismatch`). Confirms the fix did not weaken the binding: the edge
/// still must reference a scan that genuinely answers the right pattern at the
/// right slot, not merely SOME scan answering the pattern. [OPUS-4.8] sq-sfsi.
#[test]
fn multi_scan_join_edge_points_at_non_answering_scan_rejected() {
    let mut m = multi_scan_manifest();
    // sub-proof 2 (scan_b, pattern B) does not answer pattern A at SLOT_A; but its
    // commitment must still match commit_a or we'd trip the commitment gate first.
    // Point scan_a at scan_b AND set the join_eq's commit_a to scan_b's commitment
    // so we reach (and exercise) the slot gate specifically.
    m.join_edges[0].scan_a = 2;
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
            "an edge whose scan_a does not answer pattern A at slot_a must be \
             JoinSlotMismatch, got {other:?}"
        ),
    }
}

// === REJECT 1: commitment-matching (cross-scan forgery, anti-A2) ============

/// Forge: the join_eq's `commit_a` does NOT equal scan A's bound commitment (a
/// join binding a row from an unrelated/forged scan). `bind_joins` must REJECT
/// (`JoinCommitmentMismatch`).
#[test]
fn forge_join_commit_a_not_matching_scan_rejected() {
    let mut m = join_manifest();
    if let ProofInputs::JoinEq { commit_a, .. } = &mut m.sub_proofs[2].inputs {
        // A different (but well-formed) field — not scan A's attested commitment.
        *commit_a = FieldHex("0x0deadbeef".to_string());
    } else {
        unreachable!("join_eq inputs");
    }
    match prefilter(&m) {
        Err(CheckError::JoinCommitmentMismatch { edge: 0 }) => {}
        other => panic!("cross-scan commit_a forgery must be JoinCommitmentMismatch, got {other:?}"),
    }
}

/// Forge: the join_eq's `commit_b` does NOT equal scan B's bound commitment.
#[test]
fn forge_join_commit_b_not_matching_scan_rejected() {
    let mut m = join_manifest();
    if let ProofInputs::JoinEq { commit_b, .. } = &mut m.sub_proofs[2].inputs {
        *commit_b = FieldHex("0x0c0ffee".to_string());
    } else {
        unreachable!("join_eq inputs");
    }
    match prefilter(&m) {
        Err(CheckError::JoinCommitmentMismatch { edge: 0 }) => {}
        other => panic!("cross-scan commit_b forgery must be JoinCommitmentMismatch, got {other:?}"),
    }
}

/// Forge: re-point `graph_a` at scan B's commitment by swapping the EDGE's scan
/// reference so commit_a (scan A's) no longer matches commitments[graph_a] of the
/// pointed scan. Confirms the gate anchors on the SCAN the edge names, not on a
/// prover-chosen commitment read in isolation.
#[test]
fn forge_join_edge_points_at_wrong_scan_rejected() {
    let mut m = join_manifest();
    // Point scan_a at sub-proof 1 (scan B) while the join_eq still carries scan A's
    // commit_a => commit_a != scan_b.commitments[0].
    m.join_edges[0].scan_a = 1;
    match prefilter(&m) {
        Err(CheckError::JoinCommitmentMismatch { edge: 0 }) => {}
        other => panic!("edge pointed at the wrong scan must be JoinCommitmentMismatch, got {other:?}"),
    }
}

// === REJECT 2: slot binding (wrong column, §4.4 / audit-#6 analogue) ========

/// Forge: the join_eq's public `slot_a` is NOT the slot `?p` occupies in pattern A
/// (it proves the equality over the wrong column). `bind_joins` must REJECT
/// (`JoinSlotMismatch`).
#[test]
fn forge_join_wrong_slot_a_rejected() {
    let mut m = join_manifest();
    if let ProofInputs::JoinEq { slot_a, .. } = &mut m.sub_proofs[2].inputs {
        *slot_a = 0; // ?p is at slot 2 (object) of pattern A, not slot 0.
    } else {
        unreachable!("join_eq inputs");
    }
    match prefilter(&m) {
        // The per-edge slot check fails first (the edge's join_eq slots are no
        // longer the shared variable's query positions).
        Err(CheckError::JoinSlotMismatch { edge: 0 }) => {}
        other => panic!("wrong join slot_a must be JoinSlotMismatch, got {other:?}"),
    }
}

/// Forge: the join_eq's public `slot_b` is the wrong column for pattern B.
#[test]
fn forge_join_wrong_slot_b_rejected() {
    let mut m = join_manifest();
    if let ProofInputs::JoinEq { slot_b, .. } = &mut m.sub_proofs[2].inputs {
        *slot_b = 2; // ?p is at slot 0 (subject) of pattern B, not slot 2.
    } else {
        unreachable!("join_eq inputs");
    }
    match prefilter(&m) {
        Err(CheckError::JoinSlotMismatch { edge: 0 }) => {}
        other => panic!("wrong join slot_b must be JoinSlotMismatch, got {other:?}"),
    }
}

// === SCOPE: dropping the hidden edge falls back to the disclosed path ========

/// Honesty/scope assertion (NOT a forge): DROP the JoinEdge. `bind_joins` does NOT
/// demand a hidden join for a query cross-scan shared variable — that variable is
/// discharged by the DISCLOSED-row path (`recheck`/`join_obligations`), the opt-in
/// hidden join being an alternative. So the manifest with NO `join_edges` passes
/// the structural prefilter (the hidden-join gate has nothing to validate). This
/// pins the disclosed-vs-hidden boundary: a dropped hidden join is not a soundness
/// hole, it reverts to the disclosed mechanism (verified separately).
#[test]
fn drop_edge_falls_back_to_disclosed_path() {
    let mut m = join_manifest();
    m.join_edges.clear();
    // With no hidden edge declared, `bind_joins` is a no-op; the disclosed-row
    // obligation gate (`recheck`) governs the cross-scan `?p`. The manifest passes
    // every structural gate (it would reach MissingProof in the full bb verify).
    prefilter(&m).expect(
        "dropping the hidden JoinEdge falls back to the disclosed-row path and passes the structural gate",
    );
}

// === REJECT 3: spurious / wrong-slot hidden join (anti-spurious, §4.4) =======

/// Forge: the JoinEdge binds the right scans but its join_eq's slots don't match
/// the shared variable's query positions. The per-edge slot check must REJECT
/// (`JoinSlotMismatch`) — a slot-mismatched / spurious hidden join cannot pass.
#[test]
fn forge_join_edge_present_but_wrong_slots_rejected() {
    let mut m = join_manifest();
    if let ProofInputs::JoinEq { slot_a, slot_b, .. } = &mut m.sub_proofs[2].inputs {
        // Swap the slots: (0, 2) instead of (2, 0) — neither maps to ?p's query
        // positions in (pattern A, pattern B) respectively.
        *slot_a = 0;
        *slot_b = 2;
    } else {
        unreachable!("join_eq inputs");
    }
    match prefilter(&m) {
        Err(CheckError::JoinSlotMismatch { edge: 0 }) => {}
        other => panic!("a slot-mismatched join edge must be JoinSlotMismatch, got {other:?}"),
    }
}

// === REJECT 4: structural malformed edges ===================================

/// Forge: the JoinEdge's `join_proof` index is out of range (dangling).
#[test]
fn forge_join_dangling_join_proof_rejected() {
    let mut m = join_manifest();
    m.join_edges[0].join_proof = 99;
    match prefilter(&m) {
        Err(CheckError::JoinDanglingEdge { edge: 0 }) => {}
        other => panic!("a dangling join_proof index must be JoinDanglingEdge, got {other:?}"),
    }
}

/// Forge: the JoinEdge's `graph_a` index is out of the referenced scan's
/// `commitments` range (dangling committed-graph index).
#[test]
fn forge_join_dangling_graph_index_rejected() {
    let mut m = join_manifest();
    m.join_edges[0].graph_a = 5; // scan A has only commitments[0].
    match prefilter(&m) {
        Err(CheckError::JoinDanglingEdge { edge: 0 }) => {}
        other => panic!("a dangling graph_a index must be JoinDanglingEdge, got {other:?}"),
    }
}

/// Forge: the JoinEdge's `join_proof` points at a SCAN sub-proof (not a join_eq).
/// The kind check must REJECT (`JoinEdgeKindMismatch`).
#[test]
fn forge_join_proof_not_join_eq_rejected() {
    let mut m = join_manifest();
    m.join_edges[0].join_proof = 0; // sub-proof 0 is a scan, not a join_eq.
    match prefilter(&m) {
        Err(CheckError::JoinEdgeKindMismatch { edge: 0 }) => {}
        other => panic!("join_proof pointing at a scan must be JoinEdgeKindMismatch, got {other:?}"),
    }
}

/// Forge: the JoinEdge's `scan_a` points at the join_eq sub-proof (not a scan).
#[test]
fn forge_join_scan_a_not_scan_rejected() {
    let mut m = join_manifest();
    m.join_edges[0].scan_a = 2; // sub-proof 2 is the join_eq, not a scan.
    match prefilter(&m) {
        Err(CheckError::JoinEdgeKindMismatch { edge: 0 }) => {}
        other => panic!("scan_a pointing at a join_eq must be JoinEdgeKindMismatch, got {other:?}"),
    }
}

// === ACCEPT (FULL bb path) — deferred join_eq proving ========================

/// Canonical-VK / public-input / nonce binding ACCEPT over a REAL `join_eq` bb
/// proof. DEFERRED: `prover_toml_for`'s JoinEq arm returns `Err` (sq-fi03 left the
/// join_eq proving path out of scope), so this suite cannot produce a real join_eq
/// proof in-test. Enforcement point 2 (canonical VK) is therefore not exercised
/// end-to-end here. Marked `#[ignore]` until the join_eq proving path lands; the
/// reject paths above (the security-critical direction) are fully exercised
/// without it. Tracked: sq-r2s8 (join_eq proving path + this full-bb accept).
#[test]
#[ignore = "join_eq proving path deferred (sq-fi03 prover_toml_for JoinEq => Err); sq-r2s8 implements build_join/prover_toml_for + un-ignores this for a real bb accept"]
fn full_bb_join_accept_deferred() {
    // Intentionally empty: documents the missing accept-proof capability honestly
    // rather than asserting a property we cannot exercise.
}
