// [OPUS-4.8] sq-pzet (car-hire family completion, sq-8dx2): wire the remaining
// scan-family monomorphisation the (k, n, r) lattice can derive but that had no
// compiled member. NOT a soundness change -- the verifier remains NOT-yet-sound
// (sq-qhy4 / sq-9hrn); this only closes the silent-unprovability gap (sq-wto
// class) where `derive_scan_id` returned an id with no package.
// Circuit-family member (Q5 lattice): K=1 graphs, N=64 triple slots per graph,
// R=4 disclosed match rows. Generated shape -- the relation lives in
// sparq_zk_compose_core::scan (see its docs for the statement and the
// public-input contract). Do not reorder `main`'s parameters: the verifier
// recomputes the public-input vector in declaration order.
use sparq_zk_compose_core::scan::{scan_check, PatternSpec};

fn main(
    // -- public --
    challenge: pub Field,              // verifier nonce (replay binding S2.5)
    commitments: pub [Field; 1],      // per-graph flat Poseidon2 commitments
    pattern_is_const: pub [bool; 3],   // BGP slot constancy (s, p, o)
    pattern_const_enc: pub [Field; 3], // term encodings of constant slots (0 = var)
    rows: pub [[Field; 3]; 4],        // disclosed matched rows (term encodings)
    row_count: pub u32,
    attribution: pub [bool; 1],       // per-graph source attribution (audit #8)
    // -- private --
    counts: [u32; 1],                 // triples per graph (<= N)
    enc: [[[Field; 3]; 64]; 1],       // per-slot term encodings
) {
    // challenge is bound by the proof's public inputs; no relation constraint.
    let _ = challenge;
    scan_check::<1, 64, 4>(
        commitments,
        PatternSpec { is_const: pattern_is_const, const_enc: pattern_const_enc },
        rows,
        row_count,
        attribution,
        counts,
        enc,
    );
}
