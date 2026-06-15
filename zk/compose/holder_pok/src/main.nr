// [OPUS-4.8] sq-xqfg (HolderPoP T5): circuit-family member for the in-circuit
// holder Proof-of-Possession (B2, hidden-key tier). The relation lives in
// sparq_zk_compose_core::holder (see its docs for the statement, the reused
// Baby-JubJub scalar-mul / on-curve / `< L` gadgets, the holder-key-digest
// binding, and the public/private split).
//
// Do NOT reorder `main`'s parameters: the verifier (T6/sq-i1dt) recomputes the
// public-input vector in declaration order. ONLY `challenge` and
// `holder_pk_digest` are public -- the holder secret `hsk` and the holder public
// key `(hpk_x, hpk_y)` are PRIVATE, so the proof discloses NEITHER the secret NOR
// the key (hidden-holder). `challenge` is bound by the proof's public inputs
// (replay binding S2.5); `holder_pk_digest` is the issuer-attested digest the
// verifier folds from `commitment_message_with_holder` (the cross-check is T6,
// NOT this member). The in-circuit digest matches `sparq_zk::sig::holder_key_digest`
// bit-for-bit.
use sparq_zk_compose_core::holder::{holder_pok, Point};

fn main(
    // -- public --
    challenge: pub Field,        // verifier nonce (replay binding S2.5)
    holder_pk_digest: pub Field, // issuer-attested holder-key digest (ZKSIG_HK)
    // -- private --
    hsk: Field,                  // holder secret scalar, canonical < L (HIDDEN)
    hpk_x: Field,                // holder public-key affine x (HIDDEN)
    hpk_y: Field,                // holder public-key affine y (HIDDEN)
) {
    // challenge is bound by the proof's public inputs; no relation constraint
    // (the verifier byte-binds its fresh nonce, mirroring the scan/filter/revoke/
    // hidden_issuer family).
    let _ = challenge;
    let hpk = Point { x: hpk_x, y: hpk_y };
    holder_pok(holder_pk_digest, hsk, hpk);
}
