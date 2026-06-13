// [OPUS-4.8] sq-3e5 + sq-h2v: hidden-index revocation (bit-unset) proof.
//! Hidden-index status-list inclusion + bit-unset proof (plan SS2.6 privacy
//! upgrade; sq-3e5 / sq-h2v).
//!
//! # The problem this closes
//! The verifier-side revocation check (audit #12, `verifier::bind_revocation`)
//! reads the credential's status bit at a CLEAR index disclosed in
//! `manifest.revocation`. That index deanonymises the holder: a relying party
//! correlates two presentations of the same credential by its list slot. This
//! circuit proves the liveness fact IN ZERO KNOWLEDGE -- "the bit at my (hidden)
//! index in the committed status list is 0" -- disclosing NEITHER the index NOR
//! the other bits.
//!
//! # Commitment scheme
//! The status list is committed as a depth-`D` binary MERKLE TREE whose leaves
//! are the status bits (one bit per leaf, `0` = active, `1` = revoked/suspended)
//! and whose internal nodes are `h2(left, right) = Poseidon2::hash([l, r], 2)` --
//! the SAME `h2` the scan-commitment layer uses (`hashes::h2`), so the root is
//! recomputable bit-for-bit in Rust (`poseidon2::hash(&[l, r])`). The MERKLE
//! ROOT is the PUBLIC commitment; the relying party derives it from its OWN
//! authoritative snapshot (the audit-#12 re-audit trust anchor) and checks the
//! proof's public root equals it. A list of up to `2^D` indices is supported by
//! a depth-`D` tree; `revoke_unset_d10` (D=10) covers 1024 indices -- see the
//! crate verifier docs for the production sparse/compressed-inclusion gap.
//!
//! # Public vs private
//! - PUBLIC:  `challenge` (family replay-binding tag, unconstrained in-circuit,
//!            verifier byte-binds it), `root` (the authoritative status-list
//!            Merkle root).
//! - PRIVATE: `index` (the holder's list slot), `bit` (the leaf value at
//!            `index`), `siblings[D]` (the Merkle authentication path, sibling
//!            hash at each level, bottom-up).
//!
//! # What is proved
//! 1. **Bit unset**: `bit == 0` (the credential is NOT revoked). A `bit == 1`
//!    witness fails this assertion -- a revoked credential CANNOT produce a proof.
//! 2. **Index well-formed**: `index` decomposes to exactly `D` bits (so
//!    `index < 2^D`, in range for the tree) and those bits ARE the path
//!    directions used in the fold. This binds the authentication path to the
//!    actual leaf position: a prover cannot present the path of some OTHER
//!    (unset) leaf while claiming a different index -- the direction bits and the
//!    range are the same `D` bits.
//! 3. **Inclusion**: folding `leaf = bit` up through `siblings` using those
//!    direction bits recomputes the PUBLIC `root`. So the proven `(index, bit)`
//!    pair is genuinely the committed list's leaf at `index`.
//!
//! Index, bit, and all siblings are PRIVATE, so the proof reveals only "some
//! in-range index of the committed list (version bound by the root the verifier
//! supplies) holds an unset bit" -- exactly the hidden-index guarantee.
//!
//! # Cost note (noir-optimisation S5)
//! One `h2` (~74 gates) per tree level, `D` levels -- `revoke_unset_d10` is ~10
//! permutations plus the `D`-bit decomposition. Tiny next to the scan family.

use crate::hashes::{h2, h3};

// [OPUS-4.8] sq-ayv: domain tag for the hiding index commitment, IDENTICAL to the
// Rust `sparq_zk::sig::status_index_commitment` tag ("ZKSIG_IC" = 0x5a4b_5349_475f_4943).
// Both compute `Poseidon2::hash([DOMAIN, index, blinding], 3)` so the in-circuit
// commitment byte-equals the issuer-signed one (single source of truth).
pub global SIG_DOMAIN_STATUS_IDX_COMMIT: Field = 0x5a4b5349475f4943;

/// Hidden-index bit-unset inclusion relation over a depth-`D` Merkle status
/// list. See module docs for the statement and the public/private split.
///
/// `leaf_hash(bit)` is the leaf preimage: the status bit is used DIRECTLY as the
/// leaf field (0 or 1). The bit-domain is tiny, so no extra hashing is needed --
/// the Rust root builder mirrors this exactly (`leaf = Fr::from(bit)`), and the
/// `bit in {0,1}` constraint below (via the `bit == 0` assertion) keeps the leaf
/// canonical.
pub fn revoke_unset_check<let D: u32>(
    root: Field,
    index: Field,
    bit: Field,
    siblings: [Field; D],
) {
    // 1. The credential must be ACTIVE: the proven leaf bit is UNSET. A revoked
    //    (bit==1) credential cannot satisfy this, so it cannot produce a proof.
    assert(bit == 0, "credential is revoked: status bit is set");

    // 2. Decompose `index` to exactly `D` little-endian bits IN-CIRCUIT. The
    //    `to_le_bits` gadget constrains the output to be the canonical bit
    //    decomposition of `index` AND constrains every bit beyond position `D`
    //    to zero -- i.e. it enforces `index < 2^D` (the leaf is in range for the
    //    tree). Deriving the directions from `index` itself (rather than taking
    //    a separate `dirs` witness) means the path directions are, by
    //    construction, the actual leaf position: a prover cannot present the
    //    authentication path of some OTHER leaf while claiming a different index.
    let dirs: [bool; D] = index.to_le_bits();

    // 3. Fold the leaf up to the root along the authentication path. `dirs[k]`
    //    is the direction at level `k`: false => this node is the LEFT child
    //    (sibling on the right), true => this node is the RIGHT child (sibling on
    //    the left).
    let mut node = bit; // leaf preimage = the status bit itself (0 here)
    for k in 0..D {
        let sib = siblings[k];
        let is_right = dirs[k];
        let left = if is_right { sib } else { node };
        let right = if is_right { node } else { sib };
        node = h2(left, right);
    }
    assert(node == root, "merkle inclusion does not recompute the committed root");
}

/// Hidden-index bit-unset inclusion relation that ALSO cross-binds a hiding
/// COMMITMENT to the index (sq-ayv). The privacy upgrade over
/// [`revoke_unset_check`]: it proves the SAME "bit at my hidden index is unset"
/// statement AND that the PUBLIC `index_commitment` is a Poseidon2 commitment to
/// the SAME private `index` (with a private `blinding`).
///
/// # Why the cross-binding is load-bearing
/// The issuer signs `status_ref_commit_digest(list, index_commitment, version)`
/// (sq-ayv) -- a commitment to the index, NOT the clear index -- so a
/// hidden-revocation presentation can withhold the index entirely. But for
/// SOUNDNESS the verifier must know the index the issuer committed to is the SAME
/// index proven UNSET here; otherwise a holder could obtain a signature over a
/// commitment to its REVOKED index and prove bit-unset for some OTHER (active)
/// index. By recomputing `index_commitment` from THIS circuit's private `index`
/// and exposing it as a public input the verifier byte-matches against the
/// issuer-signed commitment, the two indices are provably identical. The clear
/// index is disclosed in NEITHER the signed object NOR any public input.
///
/// PUBLIC: `root`, `index_commitment`. PRIVATE: `index`, `bit`, `blinding`,
/// `siblings`. (The `challenge` public tag is handled by `main`.)
// [OPUS-4.8] sq-ayv: hidden-index bit-unset + cross-bound index commitment.
pub fn revoke_unset_check_committed<let D: u32>(
    root: Field,
    index_commitment: Field,
    index: Field,
    bit: Field,
    blinding: Field,
    siblings: [Field; D],
) {
    // 1-3. The hidden-index bit-unset inclusion (identical to revoke_unset_check):
    //      bit==0, range-bound index-derived directions, Merkle fold to `root`.
    revoke_unset_check::<D>(root, index, bit, siblings);

    // 4. CROSS-BIND the index commitment to the SAME private index proven unset
    //    above. `index_commitment` is a PUBLIC input the verifier byte-matches
    //    against the ISSUER-SIGNED commitment; recomputing it here from `index`
    //    (the same value the Merkle fold used) + the private `blinding` proves the
    //    issuer committed to the index this proof shows is active. A holder cannot
    //    sign over a commitment to a revoked index and prove a different one.
    let recomputed = h3(SIG_DOMAIN_STATUS_IDX_COMMIT, index, blinding);
    assert(
        recomputed == index_commitment,
        "index commitment does not bind the proven-unset index",
    );
}
