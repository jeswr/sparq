// [OPUS-4.8] sq-z9l: circuit-family member for the hidden-issuer-attestation
// proof, depth D=4 (issuer key sets up to 2^4 = 16 keys). The relation lives in
// sparq_zk_compose_core::issuer (see its docs for the statement, the
// Schnorr-over-Baby-JubJub verify gadget, the scalar-reduction binding, and the
// public/private split).
//
// Do NOT reorder `main`'s parameters: the verifier recomputes the public-input
// vector in declaration order. ONLY `challenge`, `m`, and `key_set_root` are
// public -- the issuer key, the signature (R, s), the challenge-reduction witness
// (e, e_k), and the membership index/path are PRIVATE, so the proof discloses
// neither WHICH issuer signed nor the signature. The verifier binds `key_set_root`
// to its OWN authoritative KeySet K (the trust anchor, like bind_hidden_revocation
// does for the status root) and binds `m` to the disclosed commitment + reference.
use sparq_zk_compose_core::issuer::{hidden_issuer_attestation, Point};

global D: u32 = 4;

fn main(
    // -- public --
    challenge: pub Field,    // verifier nonce (replay binding S2.5)
    m: pub Field,            // the signed commitment message (issuer-bound)
    key_set_root: pub Field, // authoritative issuer key-set Merkle root (commitment)
    // -- private --
    pk_x: Field,             // issuer public-key affine x (HIDDEN)
    pk_y: Field,             // issuer public-key affine y (HIDDEN)
    r_x: Field,              // signature R affine x (HIDDEN)
    r_y: Field,              // signature R affine y (HIDDEN)
    s: Field,                // signature scalar s, canonical < L (HIDDEN)
    e: Field,                // reduced challenge e = e_base mod L (HIDDEN)
    e_k: Field,              // reduction quotient, e_base = e + e_k*L (HIDDEN)
    index: Field,            // the issuer's hidden slot in K (HIDDEN)
    siblings: [Field; D],    // key-set Merkle authentication path (HIDDEN)
) {
    // challenge is bound by the proof's public inputs; no relation constraint
    // (the verifier byte-binds its fresh nonce, mirroring the scan/filter/revoke
    // family).
    let _ = challenge;
    let pk = Point { x: pk_x, y: pk_y };
    let r = Point { x: r_x, y: r_y };
    hidden_issuer_attestation::<D>(m, key_set_root, pk, r, s, e, e_k, index, siblings);
}
