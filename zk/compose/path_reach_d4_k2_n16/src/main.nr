// Circuit-family member (bounded-depth path reachability, sq-3kd2g.2 /
// #1591): D=4 unrolled chain steps, K=2 graphs, N=16 triple slots per
// graph. Generated shape -- the relation lives in
// sparq_zk_compose_core::path (see its docs for the bounded EXISTENCE-ONLY
// statement and the public-input contract). The compile-time depth bound is
// the member's identity: distinct D => distinct deterministic verification
// key (mechanism (a) on #1591), so a verifier learns the disclosed bound from
// WHICH member it accepts; `depth_bound` re-states it as a public input,
// belt-and-braces. Do not reorder `main`'s parameters: the verifier
// recomputes the public-input vector in declaration order.
use sparq_zk_compose_core::path::path_reach_check;

fn main(
    // -- public --
    challenge: pub Field,              // verifier nonce (replay binding S2.5)
    commitments: pub [Field; 2],      // per-graph flat Poseidon2 commitments
    pred_enc: pub Field,               // term encoding of the path predicate
    src_enc: pub Field,                // term encoding of the chain source
    dst_enc: pub Field,                // term encoding of the chain destination
    allow_zero: pub bool,              // operator admits the zero-length path (p*/p?)
    depth_bound: pub u32,              // MUST equal D (constant-constrained)
    attribution: pub [bool; 2],       // per-graph source attribution (chain-relative)
    // -- private --
    path_len: u32,                     // actual chain length l <= D (hidden)
    nodes: [Field; 4],                // chain nodes after each step
    counts: [u32; 2],                 // triples per graph (<= N)
    enc: [[[Field; 3]; 16]; 2],       // per-slot term encodings
) {
    // challenge is bound by the proof's public inputs; no relation constraint.
    let _ = challenge;
    path_reach_check::<4, 2, 16>(
        commitments,
        pred_enc,
        src_enc,
        dst_enc,
        allow_zero,
        depth_bound,
        attribution,
        path_len,
        nodes,
        counts,
        enc,
    );
}
