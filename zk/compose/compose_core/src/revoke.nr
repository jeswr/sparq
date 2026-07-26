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

use crate::hashes::{h2, h3, h4};

// [OPUS-4.8] sq-ayv: domain tag for the hiding index commitment, IDENTICAL to the
// Rust `sparq_zk::sig::status_index_commitment` tag ("ZKSIG_IC" = 0x5a4b_5349_475f_4943).
// Both compute `Poseidon2::hash([DOMAIN, index, blinding], 3)` so the in-circuit
// commitment byte-equals the issuer-signed one (single source of truth).
pub global SIG_DOMAIN_STATUS_IDX_COMMIT: Field = 0x5a4b5349475f4943;

// [OPUS-5] sq-kndw (sq-6qe): domain tag for the HIDING (list, version) reference
// commitment, IDENTICAL to the Rust `sparq_zk::sig::status_ref_commitment` tag
// ("ZKSIG_RC" = 0x5a4b_5349_475f_5243). Both compute
// `Poseidon2::hash([DOMAIN, list_id, version, ref_blinding], 4)`.
pub global SIG_DOMAIN_STATUS_REF_COMMITMENT: Field = 0x5a4b5349475f5243;

// [OPUS-5] sq-kndw (sq-6qe): domain tag for the ACCEPTED-SET Merkle leaf,
// IDENTICAL to the Rust `sparq_zk::sig::accepted_status_leaf` tag
// ("ZKSIG_AL" = 0x5a4b_5349_475f_414c). Both compute
// `Poseidon2::hash([DOMAIN, list_id, version, status_list_root], 4)`.
pub global SIG_DOMAIN_ACCEPTED_STATUS_LEAF: Field = 0x5a4b5349475f414c;

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

/// [OPUS-5] sq-kndw (deferred remainder of sq-6qe): the FULLY-HIDDEN revocation
/// relation -- sub-option A of `research/zk-statuslist-hide-iri-version.md` sec 3.
/// It hides the status-list IRI and the version ON TOP of the index + liveness
/// bit that [`revoke_unset_check_committed`] already hides.
///
/// # The four sub-relations (sec 3 (a)-(d))
/// - **(a) ref open**: the PUBLIC `ref_commitment` recomputes as
///   `h4(ZKSIG_RC, list_id, version, ref_blinding)` from the PRIVATE triple. This
///   ties the proof to the reference the ISSUER signed (the issuer's message is
///   `status_ref_fully_committed_digest(ref_commitment, index_commitment)`), so
///   the hidden `(list, version)` is the issuer's, not a prover choice.
/// - **(b) accepted-set membership**: `h4(ZKSIG_AL, list_id, version,
///   status_list_root)` is a leaf of the PUBLIC `accepted_set_root` at a PRIVATE
///   index/path. Because the leaf commits the triple ATOMICALLY this also
///   PRIVATELY BINDS the `status_list_root` used by (d) -- a prover cannot pair
///   list1's identity with list2's root. Structurally identical to
///   `issuer::key_set_membership`, generalised from a 2-input to a 4-input leaf.
/// - **(c) freshness**: `version >= min_version` for the PUBLIC `min_version`
///   epoch floor. `version` has NO witness freedom (it is bound through both (a)
///   and (b)), so an old version cannot be passed off as fresh. Kept a typed
///   `u64` compare -- the cheap primitive (`noir-optimisation`); do NOT cast to
///   `Field` for the comparison.
/// - **(d) bit-unset inclusion**: the existing
///   [`revoke_unset_check_committed`] body, folding against the PRIVATE
///   `status_list_root` from (b) instead of a public root, and cross-binding
///   `index_commitment` to the proven-unset index exactly as today.
///
/// # Public vs private
/// PUBLIC: `ref_commitment`, `index_commitment`, `accepted_set_root`,
/// `min_version` (plus the `challenge` tag handled by `main`).
/// PRIVATE: `list_id`, `version`, `ref_blinding`, `status_list_root`,
/// `set_index`, `set_siblings`, `index`, `bit`, `blinding`, `siblings`.
///
/// # `ref_commitment` is PUBLIC -- a deliberate deviation from the design record
/// `research/zk-statuslist-hide-iri-version.md` sec 3 lists the public inputs as
/// `{challenge, index_commitment, accepted_set_root, min_version}`, omitting
/// `ref_commitment`. That omission is UNSOUND: with `ref_commitment` private,
/// relation (a) computes a value that is never compared to anything the verifier
/// knows, so it constrains nothing and a holder could prove liveness for ANY
/// accepted `(list, version)` -- including one whose reference the issuer never
/// signed for this credential. Making it public restores exactly the sq-ayv
/// cross-binding discipline: the verifier byte-matches the proof's
/// `ref_commitment` against the ISSUER-SIGNED one carried in the attestation. It
/// is a HIDING commitment (a fresh `ref_blinding` per issuance), so publishing it
/// discloses neither the IRI nor the version -- but it IS a stable per-credential
/// handle, which is why per-presentation re-blinding is mandatory (sec 4).
///
/// # Cost -- MEASURED, and the dominant term is NOT what it looks like
/// `revoke_hidden_ref_d10_a4` measures **4308** `circuit_size`
/// (`bb gates -s ultra_honk`, pinned toolchain) against `revoke_unset_d10`'s 899.
/// Attributed by probe circuits, NOT inferred:
///
/// - `+2785` -- the FIRST integer-typed input. This is lookup/range-TABLE SETUP,
///   not the comparison: an UNUSED `u64` input measures the same delta. It is the
///   dominant term by far, and it contradicts the design record's original prior
///   that the freshness compare would be "essentially free".
/// - `+~10`  -- the `version >= min_version` compare itself (matching the
///   `noir-optimisation` ~13 gates/call figure -- the per-call number was right;
///   the one-time table cost was the unaccounted part).
/// - `+~150` -- two `h4` permutations (the ref open + the accepted-set leaf).
/// - `+~463` -- the `A = 4` accepted-set fold (`A` `h2` + a `to_le_bits`).
///
/// There is still NO scalar mul on this path, so the member stays far below the
/// Schnorr family (`hidden_issuer_d4` = 24452). Relation (c) is defence-in-depth
/// on top of the host-side freshness CURATION of the accepted set, so a future
/// member could drop it to reclaim the table -- at the cost of resting freshness
/// on curation alone. Re-measure with `bb gates`, never estimate.
pub fn revoke_hidden_ref_check<let D: u32, let A: u32>(
    // -- public --
    ref_commitment: Field,
    index_commitment: Field,
    accepted_set_root: Field,
    min_version: u64,
    // -- private --
    list_id: Field,
    version: u64,
    ref_blinding: Field,
    status_list_root: Field,
    set_index: Field,
    set_siblings: [Field; A],
    index: Field,
    bit: Field,
    blinding: Field,
    siblings: [Field; D],
) {
    // The version enters both hashes as its canonical field embedding, matching
    // the host's `Fr::from(version: u64)`. The COMPARISON below stays on the u64.
    let version_f = version as Field;

    // (a) REF OPEN -- the private (list, version, ref_blinding) opens the public,
    //     issuer-signed reference commitment.
    let recomputed_ref = h4(
        SIG_DOMAIN_STATUS_REF_COMMITMENT,
        list_id,
        version_f,
        ref_blinding,
    );
    assert(
        recomputed_ref == ref_commitment,
        "reference commitment does not open to the proven (list, version)",
    );

    // (c) FRESHNESS -- the hidden version is at or above the relying party's
    //     public epoch floor. A typed u64 `>=` (plookup-backed range check).
    assert(
        version >= min_version,
        "status-list version is older than the policy floor",
    );

    // (b) ACCEPTED-SET MEMBERSHIP -- the (list, version, status_list_root) triple
    //     is a leaf of the relying party's committed policy. This is what
    //     PRIVATELY BINDS `status_list_root` for step (d): the root is not a free
    //     witness, it is the one the relying party itself published for this
    //     hidden (list, version).
    let accepted_leaf = h4(
        SIG_DOMAIN_ACCEPTED_STATUS_LEAF,
        list_id,
        version_f,
        status_list_root,
    );
    // Directions come from `set_index` itself (never a separate witness), so the
    // path is by construction the path of the claimed slot AND `set_index < 2^A`.
    let set_dirs: [bool; A] = set_index.to_le_bits();
    let mut set_node = accepted_leaf;
    for k in 0..A {
        let sib = set_siblings[k];
        let is_right = set_dirs[k];
        let left = if is_right { sib } else { set_node };
        let right = if is_right { set_node } else { sib };
        set_node = h2(left, right);
    }
    assert(
        set_node == accepted_set_root,
        "(list, version, root) is not a member of the accepted set",
    );

    // (d) BIT-UNSET INCLUSION against the PRIVATELY-BOUND status-list root, with
    //     the sq-ayv index cross-binding unchanged.
    revoke_unset_check_committed::<D>(
        status_list_root,
        index_commitment,
        index,
        bit,
        blinding,
        siblings,
    );
}
