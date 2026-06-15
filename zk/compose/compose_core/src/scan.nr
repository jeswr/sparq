//! Per-triple-pattern scan proof (the v1 per-property circuit).
//!
//! Statement, for one BGP triple pattern over the union of `K` committed
//! graphs (plan S2.2/S2.4 with flat commitments -- the whole graph is
//! in-circuit, so membership is equality against witnessed slots and scan
//! completeness is a linear sweep; no Merkle machinery):
//!
//! 1. **Commitment recompute**: for every graph `g`, the witnessed per-slot
//!    term encodings `enc[g]` re-hash (leaf `h3`, then `commit_fold` over the
//!    first `counts[g]` leaves) to the PUBLIC commitment `commitments[g]`.
//! 1b. **Distinct-graph strict ordering** (plan S2.5, [OPUS-4.8]): the public
//!    per-graph commitment representatives are constrained STRICTLY INCREASING,
//!    `C(G_0) < C(G_1) < ... < C(G_{K-1})` (numeric `<` on the field
//!    representative). This forces the `K` committed graphs to be pairwise
//!    distinct and closes the DUPLICATE-INCLUSION / COUNT-forgery class: without
//!    it a prover could pass the SAME commitment twice (`[C(G), C(G)]`, or any
//!    out-of-order/equal pair) to forge a COUNT-style "I hold >=2 tickets" claim
//!    by including one credential twice at two block indices. The aggregate
//!    fragment that would consume such a claim is out of v1 scope, so the class
//!    is latent today, but the guard is ~free (one full-width `Field` comparison
//!    per adjacent pair -- zero for K=1) and lands BEFORE any aggregate/COUNT
//!    support so the class is never reachable. The host builder canonicalises the
//!    commitment order (and the matching per-graph witnesses) ascending; the
//!    verifier additionally rejects non-increasing commitments structurally
//!    (defence in depth, before any bb proof) so a witness-only manifest cannot
//!    smuggle a duplicate past this gate either.
//! 2. **Row soundness**: each of the `row_count` disclosed rows (term-encoding
//!    triples) matches the pattern's constant slots and occurs in some active
//!    slot of some graph.
//! 3. **Scan completeness**: every active slot that matches the pattern
//!    occurs among the disclosed rows -- the disclosed per-pattern match set
//!    is COMPLETE w.r.t. the union (the differentiator over soundness-only
//!    prior art; plan S1.1).
//!
//! `challenge` is a public input bound by the proof but unconstrained in the
//! relation: replay binding (plan S2.5, `binding: challenge`) is enforced by
//! the verifier checking the public-input bytes carry its fresh nonce.
//!
//! 4. **Per-graph source attribution** (audit #8, [OPUS-4.8]): the PUBLIC
//!    `attribution[g]` bit is constrained to be true **iff** graph `g` holds at
//!    least one active slot matching the pattern's constants -- i.e. iff graph
//!    `g` actually CONTRIBUTES a matched triple to this pattern's disclosed
//!    result. This is the exact per-pattern matched-graph set, surfaced as a
//!    public input so the verifier can byte-bind it (the audit #1 reconstruction
//!    carries it) and compare it against `manifest.attributions[pattern]`. The
//!    Q6 cross-graph-bnode-join obligation gate (`verify.rs`) then operates over
//!    a PROOF-BOUND attribution map instead of a prover-controlled JSON field: a
//!    prover whose pattern genuinely matches in two graphs can no longer declare
//!    `[[0],[0]]` to collapse them -- the bound `attribution[1] = true`
//!    contradicts the declared graph-0-only set and the verifier rejects.
//!
//! What this circuit does NOT prove (v1 deferrals, see
//! crates/sparq-zk-compose/README.md): issuer signatures over the commitments
//! are checked VERIFIER-SIDE (audit #3 -- `commitments[g]` is byte-bound into
//! these public inputs, and the verifier checks an issuer Schnorr signature
//! over each with the key in a disclosed key-set K; the in-circuit
//! undisclosed-key set-membership upgrade is the next step, see
//! `sparq_zk::sig`); join consistency across patterns is verifier-side over
//! disclosed rows, per the proof-vs-clear dispatch rule.

use crate::hashes::{commit_fold, h3};

/// One BGP triple pattern, slot-wise: `is_const[s]` discloses whether slot
/// `s` (subject/predicate/object) is a constant; for constant slots
/// `const_enc[s]` is the term encoding the verifier recomputed from the
/// query text. Variable slots leave `const_enc[s] = 0` (unconstrained).
pub struct PatternSpec {
    pub is_const: [bool; 3],
    pub const_enc: [Field; 3],
}

/// The generic scan relation; each `scan_k{K}_n{N}_r{R}` bin package is one
/// monomorphisation. See module docs for the statement.
///
/// `attribution[g]` (PUBLIC) is the per-graph source-attribution bit (audit #8):
/// it MUST equal "graph `g` holds an active slot matching the pattern", which
/// the relation enforces (step 4). The verifier byte-binds it (audit #1) and
/// checks `manifest.attributions[pattern]` is consistent with it.
///
/// `commitments` (PUBLIC) MUST be strictly increasing on the field
/// representative (`commitments[0] < commitments[1] < ...`, plan S2.5 step 1b):
/// the relation enforces it, forcing the `K` committed graphs pairwise distinct
/// and closing the duplicate-inclusion / COUNT-forgery class. The host builder
/// emits commitments (and the matching per-graph witnesses) in ascending order.
pub fn scan_check<let K: u32, let N: u32, let R: u32>(
    commitments: [Field; K],
    pattern: PatternSpec,
    rows: [[Field; 3]; R],
    row_count: u32,
    attribution: [bool; K],
    counts: [u32; K],
    enc: [[[Field; 3]; N]; K],
) {
    assert(row_count <= R, "row_count exceeds circuit bucket");

    // 1. Commitment recompute (binds every active slot's encodings).
    for g in 0..K {
        assert(counts[g] <= N, "graph size exceeds circuit bucket");
        let mut leaves: [Field; N] = [0; N];
        for i in 0..N {
            leaves[i] = h3(enc[g][i][0], enc[g][i][1], enc[g][i][2]);
        }
        assert_eq(
            commit_fold(leaves, counts[g]),
            commitments[g],
            "graph commitment mismatch",
        );
    }

    // 1b. Distinct-graph strict ordering (plan S2.5, [OPUS-4.8]): each successive
    // per-graph commitment representative must be STRICTLY GREATER than the
    // previous, `C(G_{g-1}) < C(G_g)`. Strictness (not `<=`) makes the K graphs
    // pairwise DISTINCT, closing the duplicate-inclusion / COUNT-forgery class
    // (the same credential committed twice would repeat a commitment, which the
    // `<` rejects). `Field::lt` is the full-width BN254 comparison (commitments
    // are full field elements, not small typed values), so this is a real
    // ordering over the whole representative; ~one comparison per adjacent pair
    // (none for K=1 -- the loop body is empty).
    for g in 1..K {
        assert(
            commitments[g - 1].lt(commitments[g]),
            "per-graph commitments not strictly increasing (duplicate-inclusion / COUNT forgery)",
        );
    }

    // 2. Row soundness.
    for j in 0..R {
        let row_active = j < row_count;
        // A disclosed row must itself match the pattern's constant slots
        // (otherwise a prover could disclose graph triples the pattern never
        // selects and skew the verifier's re-join).
        for s in 0..3 {
            assert(
                !row_active | !pattern.is_const[s] | (rows[j][s] == pattern.const_enc[s]),
                "disclosed row does not match the pattern",
            );
        }
        // ... and be present in some active slot of some committed graph.
        let mut present = false;
        for g in 0..K {
            for i in 0..N {
                let slot_active = i < counts[g];
                let eq = (enc[g][i][0] == rows[j][0])
                    & (enc[g][i][1] == rows[j][1])
                    & (enc[g][i][2] == rows[j][2]);
                present |= slot_active & eq;
            }
        }
        assert(!row_active | present, "disclosed row not in any committed graph");
    }

    // 3. Scan completeness: matching slots must all be disclosed.
    // 4. Per-graph source attribution (audit #8): accumulate, per graph,
    //    whether ANY active slot matches the pattern -- the exact set of graphs
    //    that contribute a matched triple to this pattern. Folded into the same
    //    sweep that already computes `matches` (no extra equality work; one OR
    //    accumulator + one bool-equality assert per graph).
    for g in 0..K {
        let mut graph_matches = false;
        for i in 0..N {
            let slot_active = i < counts[g];
            let mut matches = slot_active;
            for s in 0..3 {
                matches &= !pattern.is_const[s] | (enc[g][i][s] == pattern.const_enc[s]);
            }
            graph_matches |= matches;
            let mut disclosed = false;
            for j in 0..R {
                let row_active = j < row_count;
                let eq = (enc[g][i][0] == rows[j][0])
                    & (enc[g][i][1] == rows[j][1])
                    & (enc[g][i][2] == rows[j][2]);
                disclosed |= row_active & eq;
            }
            assert(!matches | disclosed, "matching triple missing from disclosed rows");
        }
        // The disclosed public attribution bit MUST equal the true matched-graph
        // bit. A prover cannot declare `attribution[g] = false` for a graph that
        // genuinely contributes (the [[0],[0]] collapse-two-graphs lie of audit
        // #8), nor claim a contribution that did not happen.
        assert(attribution[g] == graph_matches, "attribution does not match the proved source graphs");
    }
}
