// [OPUS-5] sq-kndw (deferred remainder of sq-6qe): circuit-family member for the
// FULLY-HIDDEN revocation proof -- status-list IRI + version hidden IN ADDITION to
// the index and the liveness bit. D=10 (status lists up to 2^10 = 1024 indices,
// matching `revoke_unset_d10`), A=4 (accepted sets up to 2^4 = 16 `(list, version)`
// pairs, matching the `hidden_issuer_d4` / `holder_set_d4` set-depth convention).
// The relation lives in `sparq_zk_compose_core::revoke::revoke_hidden_ref_check`
// (see its docs for the four sub-relations and the public/private split).
//
// Do NOT reorder `main`'s parameters: the verifier recomputes the public-input
// vector in declaration order (`crate::verifier::bind_fully_hidden_revocation`).
// PUBLIC are only `challenge`, `ref_commitment`, `index_commitment`,
// `accepted_set_root` and `min_version`; the list id, the version, both blindings,
// the status-list root, the accepted-set slot/path, the index, the leaf bit and the
// status-list path are all PRIVATE. So the presentation discloses neither WHICH
// list, WHICH publication epoch, WHICH slot, nor the liveness bit -- only that SOME
// `(list, version)` in the relying party's committed accepted set, at or above its
// public epoch floor, has the issuer-committed index unset.
//
// The two commitments are HIDING but STABLE per issuance: they must be re-blinded
// per presentation or they become cross-presentation linkage handles (design sec 4).
// The verifier enforces single-use of the pair; see `bind_fully_hidden_revocation`.
//
// NOT externally audited (sq-qhy4). No soundness / privacy property is asserted as
// achieved -- this is research-grade.
use sparq_zk_compose_core::revoke::revoke_hidden_ref_check;

global D: u32 = 10;
global A: u32 = 4;

fn main(
    // -- public --
    challenge: pub Field,          // verifier nonce (replay binding S2.5)
    ref_commitment: pub Field,     // hiding commitment to (list, version); issuer-signed
    index_commitment: pub Field,   // hiding commitment to the index; issuer-signed (sq-ayv)
    accepted_set_root: pub Field,  // the relying party's committed accepted-set root
    min_version: pub u64,          // the relying party's PUBLIC epoch floor
    // -- private --
    list_id: Field,                // H(status-list IRI) -- HIDDEN
    version: u64,                  // the status-list publication epoch -- HIDDEN
    ref_blinding: Field,           // ref-commitment blinding -- HIDDEN
    status_list_root: Field,       // the status-list Merkle root -- HIDDEN, bound by (b)
    set_index: Field,              // accepted-set slot -- HIDDEN
    set_siblings: [Field; A],      // accepted-set authentication path (bottom-up)
    index: Field,                  // the holder's list slot -- HIDDEN
    bit: Field,                    // the status bit at `index` (must be 0 = active)
    blinding: Field,               // index-commitment blinding -- HIDDEN
    siblings: [Field; D],          // status-list authentication path (bottom-up)
) {
    // challenge is bound by the proof's public inputs; no relation constraint
    // (the verifier byte-binds its fresh nonce, mirroring the rest of the family).
    let _ = challenge;
    revoke_hidden_ref_check::<D, A>(
        ref_commitment,
        index_commitment,
        accepted_set_root,
        min_version,
        list_id,
        version,
        ref_blinding,
        status_list_root,
        set_index,
        set_siblings,
        index,
        bit,
        blinding,
        siblings,
    );
}
