// [OPUS-4.8] sq-pzet (car-hire family completion, sq-8dx2): wire the remaining
// hidden-join graph-size monomorphisations so a `join_eq` proof can compose with
// a scan over the N=64 bucket (the larger-arity members the prior single
// `na16_nb16` member could not express). NOT a soundness change -- the verifier
// remains NOT-yet-sound (sq-qhy4 / sq-9hrn / sq-1s2); the in-circuit relation is
// `join_eq_check` verbatim, only the const-generic bucket changes.
// Circuit-family member (Q5 lattice): N_A=64, N_B=64 triple slots per graph.
// Generated shape -- the relation lives in sparq_zk_compose_core::join (see its
// docs for the statement and the public-input contract). Do not reorder `main`'s
// parameters: the verifier recomputes the public-input vector in declaration
// order (audit-#1 discipline). The join VALUE is PRIVATE; only the two graph
// commitments, the two query-bound slots, and a HIDING commitment are public.
use sparq_zk_compose_core::join::join_eq_check;

fn main(
    // -- public --
    challenge: pub Field,            // verifier nonce (replay binding S2.5)
    commit_a: pub Field,             // graph-A commitment C(G_a) == scan-A.commitments[g_a]
    commit_b: pub Field,             // graph-B commitment C(G_b) == scan-B.commitments[g_b]
    join_commitment: pub Field,      // HIDING commitment to the join value (design S2.4)
    slot_a: pub u32,                 // graph-A join slot in {0,1,2} (== query-derived s_a)
    slot_b: pub u32,                 // graph-B join slot in {0,1,2} (== query-derived s_b)
    // -- private --
    enc_a: [[Field; 3]; 64],         // graph-A per-slot term encodings
    counts_a: u32,                   // triples in graph A (<= N_A)
    enc_b: [[Field; 3]; 64],         // graph-B per-slot term encodings
    counts_b: u32,                   // triples in graph B (<= N_B)
    row_a: [Field; 3],               // the joined row of A
    row_b: [Field; 3],               // the joined row of B
    blinding: Field,                 // per-presentation blinder for join_commitment
) {
    // challenge is bound by the proof's public inputs; no relation constraint.
    let _ = challenge;
    join_eq_check::<64, 64>(
        commit_a,
        commit_b,
        join_commitment,
        slot_a,
        slot_b,
        enc_a,
        counts_a,
        enc_b,
        counts_b,
        row_a,
        row_b,
        blinding,
    );
}
