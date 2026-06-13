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
//! What this circuit does NOT prove (v1 deferrals, see
//! crates/sparq-zk-compose/README.md): issuer signatures over the commitments
//! are checked VERIFIER-SIDE (audit #3 -- `commitments[g]` is byte-bound into
//! these public inputs, and the verifier checks an issuer Schnorr signature
//! over each with the key in a disclosed key-set K; the in-circuit
//! undisclosed-key set-membership upgrade is the next step, see
//! `sparq_zk::sig`); per-row graph attributions are NOT bound in-circuit (the
//! verifier conservatively assumes every pattern may draw from every graph,
//! audit #8); join consistency across patterns is verifier-side over disclosed
//! rows, per the proof-vs-clear dispatch rule.

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
pub fn scan_check<let K: u32, let N: u32, let R: u32>(
    commitments: [Field; K],
    pattern: PatternSpec,
    rows: [[Field; 3]; R],
    row_count: u32,
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
    for g in 0..K {
        for i in 0..N {
            let slot_active = i < counts[g];
            let mut matches = slot_active;
            for s in 0..3 {
                matches &= !pattern.is_const[s] | (enc[g][i][s] == pattern.const_enc[s]);
            }
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
    }
}
