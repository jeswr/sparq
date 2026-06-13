// Circuit-family member (Q5 lattice): K=2 graphs, N=16 triple slots per
// graph, R=8 disclosed match rows. Generated shape -- the relation lives in
// sparq_zk_compose_core::scan (see its docs for the statement and the
// public-input contract). Do not reorder `main`'s parameters: the verifier
// recomputes the public-input vector in declaration order.
use sparq_zk_compose_core::scan::{scan_check, PatternSpec};

fn main(
    // -- public --
    challenge: pub Field,              // verifier nonce (replay binding S2.5)
    commitments: pub [Field; 2],      // per-graph flat Poseidon2 commitments
    pattern_is_const: pub [bool; 3],   // BGP slot constancy (s, p, o)
    pattern_const_enc: pub [Field; 3], // term encodings of constant slots (0 = var)
    rows: pub [[Field; 3]; 8],        // disclosed matched rows (term encodings)
    row_count: pub u32,
    // -- private --
    counts: [u32; 2],                 // triples per graph (<= N)
    enc: [[[Field; 3]; 16]; 2],       // per-slot term encodings
) {
    // challenge is bound by the proof's public inputs; no relation constraint.
    let _ = challenge;
    scan_check::<2, 16, 8>(
        commitments,
        PatternSpec { is_const: pattern_is_const, const_enc: pattern_const_enc },
        rows,
        row_count,
        counts,
        enc,
    );
}
