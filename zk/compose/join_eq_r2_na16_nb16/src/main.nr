// [OPUS-5] sq-uii0: CARDINALITY-HIDING hidden cross-credential JOIN member (v2
// of sq-bwwl's join_eq). Circuit-family member (Q5 lattice): R=2 joined lanes,
// N_A=16, N_B=16 triple slots per graph.
//
// Generated shape -- the relation lives in `sparq_zk_compose_core::join`
// (`join_eq_r_check`; see its docs for the statement, the padding discipline
// that hides N, and the explicit list of what is NOT proved). Do not reorder
// `main`'s parameters: the verifier recomputes the public-input vector in
// declaration order (audit-#1 discipline).
//
// The join VALUES are PRIVATE; only the two graph commitments, the R HIDING
// per-lane commitments, and the two query-bound slots are public. Relative to
// `join_eq_na16_nb16` (one pair per sub-proof, so N sub-proofs disclose N),
// this member discloses only `N in [1, R]` -- the residual-leakage R2 tier of
// research/zk-hidden-join-design.md S2.5.
//
// NOT a soundness change: the composition verifier remains NOT-yet-sound
// pending external audit (sq-qhy4). No soundness / ZK-privacy property is
// asserted as achieved.
use sparq_zk_compose_core::join::join_eq_r_check;

fn main(
    // -- public --
    challenge: pub Field,                // verifier nonce (replay binding S2.5)
    commit_a: pub Field,                 // graph-A commitment C(G_a) == scan-A.commitments[g_a]
    commit_b: pub Field,                 // graph-B commitment C(G_b) == scan-B.commitments[g_b]
    join_commitments: pub [Field; 2],    // per-lane HIDING commitments (design S2.4)
    slot_a: pub u32,                     // graph-A join slot in {0,1,2} (== query-derived s_a)
    slot_b: pub u32,                     // graph-B join slot in {0,1,2} (== query-derived s_b)
    // -- private --
    enc_a: [[Field; 3]; 16],             // graph-A per-slot term encodings
    counts_a: u32,                       // triples in graph A (<= N_A)
    enc_b: [[Field; 3]; 16],             // graph-B per-slot term encodings
    counts_b: u32,                       // triples in graph B (<= N_B)
    rows_a: [[Field; 3]; 2],             // the joined rows of A, one per lane
    rows_b: [[Field; 3]; 2],             // the joined rows of B, one per lane
    blindings: [Field; 2],               // per-lane, per-presentation blinders
) {
    // challenge is bound by the proof's public inputs; no relation constraint.
    let _ = challenge;
    join_eq_r_check::<16, 16, 2>(
        commit_a,
        commit_b,
        join_commitments,
        slot_a,
        slot_b,
        enc_a,
        counts_a,
        enc_b,
        counts_b,
        rows_a,
        rows_b,
        blindings,
    );
}
