// [OPUS-4.8] sq-3c00 (HolderPoP hidden-holder-SET tier): circuit-family member for
// the hidden-holder set-membership proof, depth D=4 (holder sets up to 2^4 = 16
// holders). The relation lives in sparq_zk_compose_core::holder (see its docs for
// the statement, the reused Baby-JubJub scalar-mul / on-curve / `< L` gadgets, the
// holder-key-digest leaf, the Merkle membership fold, and the public/private split).
//
// Do NOT reorder `main`'s parameters: the verifier recomputes the public-input
// vector in declaration order. ONLY `challenge` and `holder_set_root` are public --
// the holder secret `hsk`, the holder public key `(hpk_x, hpk_y)`, the membership
// `index`, and the authentication path `siblings` are PRIVATE, so the proof
// discloses NEITHER the holder key NOR WHICH holder in the set it is. The verifier
// (bind_holder_set) binds `holder_set_root` to its OWN authoritative holder
// registry root (the trust anchor, like bind_hidden_issuer_attestations does for
// the key-set root). `holder_pk_digest` is NOT a public input here -- the
// hidden-holder upgrade over the clear-digest `holder_pok` member.
//
// NOT-yet-sound (sq-qhy4 / sq-9hrn; remediation epic sq-1s2); opt-in. No
// soundness / ZK-privacy property is asserted as achieved.
use sparq_zk_compose_core::holder::{hidden_holder_set, Point};

global D: u32 = 4;

fn main(
    // -- public --
    challenge: pub Field,       // verifier nonce (replay binding S2.5)
    holder_set_root: pub Field, // authoritative holder-set Merkle root (commitment)
    // -- private --
    hsk: Field,                 // holder secret scalar, canonical < L (HIDDEN)
    hpk_x: Field,               // holder public-key affine x (HIDDEN)
    hpk_y: Field,               // holder public-key affine y (HIDDEN)
    index: Field,               // the holder's hidden slot in the set (HIDDEN)
    siblings: [Field; D],       // holder-set Merkle authentication path (HIDDEN)
) {
    // challenge is bound by the proof's public inputs; no relation constraint
    // (the verifier byte-binds its fresh nonce, mirroring the scan/filter/revoke/
    // hidden_issuer/holder_pok family).
    let _ = challenge;
    let hpk = Point { x: hpk_x, y: hpk_y };
    hidden_holder_set::<D>(holder_set_root, hsk, hpk, index, siblings);
}
