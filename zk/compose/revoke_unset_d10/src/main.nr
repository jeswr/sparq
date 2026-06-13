// [OPUS-4.8] sq-3e5 + sq-h2v: circuit-family member for the hidden-index
// revocation (bit-unset) proof, depth D=10 (status lists up to 2^10 = 1024
// indices). The relation lives in sparq_zk_compose_core::revoke (see its docs
// for the statement, the commitment scheme, and the public/private split).
//
// Do NOT reorder `main`'s parameters: the verifier recomputes the public-input
// vector in declaration order. ONLY `challenge`, `root`, and `index_commitment`
// are public -- the index, the leaf bit, the blinding, and the authentication
// path are PRIVATE, so the proof discloses neither WHICH credential nor the
// other bits nor the index itself (the linkability channel the clear-index check
// leaked). The verifier binds `root` to its OWN authoritative status-list snapshot
// (audit #12 re-audit trust anchor) and byte-matches `index_commitment` against
// the ISSUER-SIGNED index commitment (sq-ayv cross-binding), so the index proven
// unset is provably the one the issuer committed to.
use sparq_zk_compose_core::revoke::revoke_unset_check_committed;

global D: u32 = 10;

fn main(
    // -- public --
    challenge: pub Field,        // verifier nonce (replay binding S2.5)
    root: pub Field,             // authoritative status-list Merkle root (commitment)
    index_commitment: pub Field, // [OPUS-4.8] sq-ayv: hiding commitment to the index,
                                 // cross-bound to the proven-unset index in-circuit and
                                 // byte-matched to the issuer-signed index commitment.
    // -- private --
    index: Field,                // the holder's hidden list index
    bit: Field,                  // the status bit at `index` (must be 0 = active)
    blinding: Field,             // [OPUS-4.8] sq-ayv: index-commitment blinding (HIDDEN)
    siblings: [Field; D],        // Merkle authentication path (bottom-up siblings)
) {
    // challenge is bound by the proof's public inputs; no relation constraint
    // (the verifier byte-binds its fresh nonce, mirroring the scan/filter family).
    let _ = challenge;
    revoke_unset_check_committed::<D>(root, index_commitment, index, bit, blinding, siblings);
}
