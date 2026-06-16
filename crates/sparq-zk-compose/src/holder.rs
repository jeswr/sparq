// [OPUS-4.8] sq-xqfg (HolderPoP T5): in-circuit holder-PoK host-side helpers.
//! Host-side witness + `Prover.toml` machinery for the in-circuit **holder
//! Proof-of-Possession** proof (sq-xqfg, design `research/zk-holder-pop-design.md`
//! §2.B/B2, §3.4). The in-circuit relation is
//! `zk/compose/compose_core/src/holder.nr` (the `holder_pok` member); this module
//! is its Rust mirror — the analogue of [`crate::issuer`] for the hidden-issuer
//! member.
//!
//! The B2 tier proves, in zero knowledge, knowledge of a holder secret `hsk`
//! whose public key `hpk = hsk·G` matches the issuer-attested holder-key digest
//! `holder_pk_digest = Poseidon2([ZKSIG_HK, hpk.x, hpk.y])`
//! ([`sparq_zk::sig::holder_key_digest`]) — WITHOUT disclosing `hsk` or `hpk`
//! (only the digest is public). It is ONE Baby-JubJub scalar-mul + a Poseidon2
//! digest, strictly cheaper than the hidden-issuer member's two scalar-muls.
//!
//! # Single source of truth (SECURITY-critical, design §4.3)
//! The witness ([`sparq_zk::sig::in_circuit_holder_witness`]) computes
//! `holder_pk_digest` via [`sparq_zk::sig::holder_key_digest`], which is the SAME
//! value the issuer folded into `commitment_message_with_holder` at mint and the
//! SAME value the in-circuit `holder::holder_key_digest` recomputes. So the host
//! digest, the issuer-attested digest, and the in-circuit digest are bit-identical
//! by construction — the cross-check the verifier gate (T6/sq-i1dt) relies on.
//!
//! # Scope
//! This module is the circuit-member WIRING only (witness + `Prover.toml`): the
//! [`CircuitId::HolderPok`](crate::CircuitId::HolderPok) member is buildable and
//! known. The verifier binding gate (`bind_holder_pok` — reconstruct + byte-equal
//! the public inputs, bind `holder_pk_digest` to the issuer attestation, `bb
//! verify`) is T6/sq-i1dt and is NOT implemented here.

use sparq_zk::field::{field_to_hex, Fr};
use sparq_zk::poseidon2;
use sparq_zk::sig::{
    holder_key_digest, in_circuit_holder_witness, InCircuitHolderPokWitness, PublicKey, SecretKey,
};

/// The complete private witness for one holder Proof-of-Possession: the holder
/// secret `hsk` (base-field embedding) and the holder public key `hpk = hsk·G`
/// affine coordinates, plus the issuer-attested `holder_pk_digest` (the PUBLIC
/// input). A re-export of [`InCircuitHolderPokWitness`] so callers stay within the
/// `sparq_zk_compose` surface, mirroring how [`crate::HiddenIssuerWitness`] bundles
/// the hidden-issuer member's witness.
pub type HolderPokWitness = InCircuitHolderPokWitness;

/// Build the [`HolderPokWitness`] for a holder key pair from its secret `hsk`
/// (the prover-side input to the `holder_pok` member). Returns `None` if the
/// derived public key is the identity (no affine coordinates) — which the
/// in-circuit gadget rejects anyway (fail-closed). Thin re-export of
/// [`sparq_zk::sig::in_circuit_holder_witness`] so the host wiring lives on the
/// `sparq_zk_compose` surface alongside the member registration.
pub fn holder_pok_witness(hsk: &SecretKey) -> Option<HolderPokWitness> {
    in_circuit_holder_witness(hsk)
}

/// Render the `Prover.toml` body for the `holder_pok` member. Order MUST match
/// `holder_pok/src/main.nr`:
/// PUBLIC: challenge, holder_pk_digest;
/// PRIVATE: hsk, hpk_x, hpk_y.
///
/// `challenge` is the verifier's fresh nonce (the public-input field-0 convention
/// the whole circuit family shares); `holder_pk_digest` is the issuer-attested
/// digest [`HolderPokWitness::holder_pk_digest`], which the verifier (T6) binds to
/// the issuer attestation. `hsk`, `hpk_x`, `hpk_y` are the private witness.
pub fn holder_pok_prover_toml(challenge: &Fr, witness: &HolderPokWitness) -> String {
    let mut s = String::new();
    s.push_str(&format!("challenge = \"{}\"\n", field_to_hex(challenge)));
    s.push_str(&format!(
        "holder_pk_digest = \"{}\"\n",
        field_to_hex(&witness.holder_pk_digest)
    ));
    s.push_str(&format!("hsk = \"{}\"\n", field_to_hex(&witness.hsk)));
    s.push_str(&format!("hpk_x = \"{}\"\n", field_to_hex(&witness.hpk_x)));
    s.push_str(&format!("hpk_y = \"{}\"\n", field_to_hex(&witness.hpk_y)));
    s
}

// ===========================================================================
// [OPUS-4.8] sq-3c00 (HolderPoP hidden-holder-SET tier): host-side Merkle
// commitment + membership witness for the `holder_set_d{depth}` member. The
// in-circuit relation is `holder.nr::hidden_holder_set`; this is its Rust mirror,
// the analogue of [`crate::issuer`]'s `key_set_root` / `key_membership_witness`
// for the hidden-issuer member.
//
// The holder SET is committed as a depth-`D` Poseidon2 Merkle tree whose leaf for
// holder `H` is `holder_key_digest(hpk_H)` (the canonical attested holder
// identity, the SAME value `holder_pok` makes public in the clear-digest tier and
// the issuer folds into `commitment_message_with_holder`). The RELYING PARTY
// commits this root over its OWN authoritative holder registry (the trust anchor);
// the proof's PUBLIC `holder_set_root` must byte-equal it. WHICH holder is hidden;
// the trust source is not.
//
// NOT-yet-sound (sq-qhy4 / sq-9hrn; remediation epic sq-1s2); opt-in. No
// soundness / ZK-privacy property is asserted as achieved.
// ===========================================================================

/// The Poseidon2 two-input compression for Merkle internal nodes -- the Rust
/// mirror of the circuit's `h2(a, b) = Poseidon2::hash([a, b], 2)`. Identical to
/// `issuer::h2` / `revocation::h2` (cross-tested bit-identical), shared one source
/// of truth for the tree shape.
fn h2(a: Fr, b: Fr) -> Fr {
    poseidon2::hash(&[a, b])
}

/// The padding leaf for holder-set slots past the set size: `Fr::from(0)`. A
/// genuine holder leaf is `holder_key_digest(hpk)` (a domain-separated Poseidon2
/// digest, never `0` in practice for a non-identity key); the membership proof
/// additionally binds the index, so a padding slot is never a usable member. Same
/// discipline as `issuer::padding_leaf`.
fn padding_leaf() -> Fr {
    Fr::from(0u64)
}

/// The `2^depth` holder-set leaves: leaf `i` = `holder_key_digest(holders[i])` for
/// `i < holders.len()`, else the padding leaf. `None` if any holder key is the
/// identity (no coordinates -- never a usable key, fail-closed), `depth > 31`, or
/// the set overflows the tree.
fn holder_leaves(holders: &[PublicKey], depth: u32) -> Option<Vec<Fr>> {
    if depth > 31 {
        return None;
    }
    let n_leaves = 1usize << depth;
    if holders.len() > n_leaves {
        return None; // the set does not fit the tree at this depth
    }
    let mut leaves = Vec::with_capacity(n_leaves);
    for hpk in holders {
        // holder_key_digest is the in-circuit leaf (holder_set_leaf in holder.nr);
        // None for an identity key (fail-closed, matches the in-circuit guard).
        leaves.push(holder_key_digest(hpk).ok()?);
    }
    leaves.resize(n_leaves, padding_leaf());
    Some(leaves)
}

/// Commit the trusted holder set as a depth-`depth` Poseidon2 Merkle tree and
/// return the root (leaf `i` = `holder_key_digest(holders[i])`). This is what the
/// RELYING PARTY computes over its OWN authoritative holder registry (the trust
/// anchor the proof's public `holder_set_root` is checked against) and what the
/// PROVER computes over the same set. Returns `None` if `depth > 31`, the set
/// overflows the tree, or any holder key is the identity (fail-closed). Mirrors
/// [`crate::issuer::key_set_root`].
pub fn holder_set_root(holders: &[PublicKey], depth: u32) -> Option<Fr> {
    let mut level = holder_leaves(holders, depth)?;
    for _ in 0..depth {
        level = level.chunks(2).map(|pair| h2(pair[0], pair[1])).collect();
    }
    debug_assert_eq!(level.len(), 1, "fold reduces to a single root");
    level.first().copied()
}

/// The authentication path (sibling at each level, BOTTOM-UP -- level 0 first) for
/// the holder at `index` in the depth-`depth` tree. The private input the circuit
/// folds. `None` if `index >= 2^depth`, the set overflows the tree, `depth > 31`,
/// or any holder key is the identity. Mirrors
/// [`crate::issuer::key_membership_witness`].
pub fn holder_set_membership_witness(
    holders: &[PublicKey],
    depth: u32,
    index: u64,
) -> Option<Vec<Fr>> {
    let mut level = holder_leaves(holders, depth)?;
    let n_leaves = 1u64 << depth;
    if index >= n_leaves {
        return None;
    }
    let mut pos = index as usize;
    let mut siblings = Vec::with_capacity(depth as usize);
    for _ in 0..depth {
        let sib = pos ^ 1;
        siblings.push(level[sib]);
        level = level.chunks(2).map(|pair| h2(pair[0], pair[1])).collect();
        pos /= 2;
    }
    Some(siblings)
}

/// The complete private witness for one hidden-holder-SET proof: the holder PoK
/// witness ([`HolderPokWitness`], carrying `hsk` + the `hpk` coords) plus the
/// holder-set membership path. Mirrors the private inputs of
/// `holder_set_d{depth}::main` (the analogue of [`crate::HiddenIssuerWitness`]).
/// The `holder_pk_digest` field of the inner [`HolderPokWitness`] is NOT a public
/// input of this member (the hidden-holder upgrade hides it); it is retained only
/// as the host-side leaf cross-check.
#[derive(Debug, Clone)]
pub struct HolderSetWitness {
    /// The holder PoK witness (`hsk`, `hpk_x`, `hpk_y`, and the host
    /// `holder_pk_digest` leaf cross-check). `hsk`/`hpk` are PRIVATE in-circuit.
    pub pok: HolderPokWitness,
    /// The holder's index in the set (private; the circuit derives the path
    /// directions from it).
    pub index: u64,
    /// The Merkle authentication path: `siblings[k]` is the sibling at level `k`
    /// (level 0 = leaves), bottom-up. Length = `depth`.
    pub siblings: Vec<Fr>,
}

/// Render the `Prover.toml` body for the `holder_set_d{depth}` member. Order MUST
/// match `holder_set_d{depth}/src/main.nr`:
/// PUBLIC: challenge, holder_set_root;
/// PRIVATE: hsk, hpk_x, hpk_y, index, siblings.
///
/// `challenge` is the verifier's fresh nonce (the public-input field-0 convention
/// the whole circuit family shares); `holder_set_root` is the committed set root
/// the verifier binds to its OWN authoritative holder registry. `hsk`, `hpk_x`,
/// `hpk_y`, `index`, `siblings` are the private witness (so the proof discloses
/// NEITHER the holder key NOR which holder).
pub fn holder_set_prover_toml(
    challenge: &Fr,
    holder_set_root: &Fr,
    witness: &HolderSetWitness,
) -> String {
    let w = &witness.pok;
    let mut s = String::new();
    s.push_str(&format!("challenge = \"{}\"\n", field_to_hex(challenge)));
    s.push_str(&format!(
        "holder_set_root = \"{}\"\n",
        field_to_hex(holder_set_root)
    ));
    s.push_str(&format!("hsk = \"{}\"\n", field_to_hex(&w.hsk)));
    s.push_str(&format!("hpk_x = \"{}\"\n", field_to_hex(&w.hpk_x)));
    s.push_str(&format!("hpk_y = \"{}\"\n", field_to_hex(&w.hpk_y)));
    s.push_str(&format!("index = \"{}\"\n", witness.index));
    let sibs: Vec<String> = witness
        .siblings
        .iter()
        .map(|sib| format!("\"{}\"", field_to_hex(sib)))
        .collect();
    s.push_str(&format!("siblings = [{}]\n", sibs.join(", ")));
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CircuitId;
    use sparq_zk::sig::holder_key_digest;

    // The `holder_pok` CircuitId maps to the on-disk package directory, so the
    // verifier (T6) and the driver resolve the same compiled member.
    #[test]
    fn holder_pok_circuit_id_package_name() {
        assert_eq!(CircuitId::HolderPok.package(), "holder_pok");
    }

    // The witness is assembled directly from a holder secret; its digest is
    // EXACTLY `holder_key_digest(hpk)` (the single-source-of-truth the verifier
    // cross-check and the in-circuit digest both rely on), and `hpk` is the
    // secret's public key (the `hpk = hsk·G` relation the circuit re-derives).
    #[test]
    fn holder_pok_witness_digest_matches_holder_key_digest() {
        let hsk = SecretKey::from_seed(0xc0ffee_u64);
        let w = holder_pok_witness(&hsk).expect("non-identity holder key has a witness");
        let hpk = hsk.public_key();
        let (x, y) = hpk
            .coords()
            .expect("non-identity holder key has coordinates");
        assert_eq!(w.hpk_x, x, "witness hpk_x is the secret's public key x");
        assert_eq!(w.hpk_y, y, "witness hpk_y is the secret's public key y");
        assert_eq!(
            w.holder_pk_digest,
            holder_key_digest(&hpk).expect("non-identity holder key digests"),
            "witness digest must equal holder_key_digest(hpk) — the T6 cross-check anchor"
        );
    }

    // Distinct holder keys produce distinct digests (the binding is key-specific:
    // a malicious holder A cannot reuse B's digest with A's own key).
    #[test]
    fn holder_pok_witness_is_key_distinguishing() {
        let w_a = holder_pok_witness(&SecretKey::from_seed(201)).unwrap();
        let w_b = holder_pok_witness(&SecretKey::from_seed(202)).unwrap();
        assert_ne!(w_a.holder_pk_digest, w_b.holder_pk_digest);
        assert_ne!((w_a.hpk_x, w_a.hpk_y), (w_b.hpk_x, w_b.hpk_y));
    }

    // CROSS-VECTOR PIN (SECURITY-critical): the seed-102 holder witness matches the
    // exact hex constants the Noir `tests.nr` `holder_key_digest_cross_vector` /
    // `holder_pok_accepts_valid_possession` tests assert. If the host digest or the
    // scalar/coordinate embedding ever drifts from the in-circuit gadget, this pin
    // (and its Noir twin) fails -- the T6 verifier cross-check rests on them agreeing
    // bit-for-bit.
    #[test]
    fn holder_pok_witness_matches_noir_cross_vector() {
        let w = holder_pok_witness(&SecretKey::from_seed(102)).unwrap();
        assert_eq!(
            field_to_hex(&w.hsk),
            "0x04c49ec34f100efeb528ac3d436a6e1a2cb6b0c85fab6a485462c74c12a82d15"
        );
        assert_eq!(
            field_to_hex(&w.hpk_x),
            "0x2f55331c1d80c2398a6ca962b853ac337350e2ff4c17f1842024337ed07190c2"
        );
        assert_eq!(
            field_to_hex(&w.hpk_y),
            "0x10427ec74fc160b890be66dbc372aa61108183ddd27765186f476ff50044ddbe"
        );
        assert_eq!(
            field_to_hex(&w.holder_pk_digest),
            "0x19e7f4238df100483b1786ae2bd0e8e1c06fc0660ee1016430e95bf824d6e12b"
        );
    }

    // The Prover.toml renders all five fields in main's declaration order
    // (PUBLIC challenge, holder_pk_digest; PRIVATE hsk, hpk_x, hpk_y).
    #[test]
    fn holder_pok_prover_toml_renders_all_fields() {
        let hsk = SecretKey::from_seed(303);
        let w = holder_pok_witness(&hsk).unwrap();
        let toml = holder_pok_prover_toml(&Fr::from(0x2au64), &w);
        for field in ["challenge", "holder_pk_digest", "hsk", "hpk_x", "hpk_y"] {
            assert!(toml.contains(field), "toml must render {field}");
        }
        // The PUBLIC inputs come first (the verifier reconstructs them in order).
        let challenge_pos = toml.find("challenge").unwrap();
        let digest_pos = toml.find("holder_pk_digest").unwrap();
        let hsk_pos = toml.find("hsk").unwrap();
        assert!(challenge_pos < digest_pos && digest_pos < hsk_pos);
    }

    // ----- [OPUS-4.8] sq-3c00: hidden-holder-SET host helpers -----

    fn holder_set(seeds: &[u64]) -> Vec<PublicKey> {
        seeds.iter().map(|s| SecretKey::from_seed(*s).public_key()).collect()
    }

    // The `holder_set_d{depth}` CircuitId maps to the on-disk package directory, so
    // the verifier (bind_holder_set) and the driver resolve the same compiled member.
    #[test]
    fn holder_set_circuit_id_package_name() {
        assert_eq!(CircuitId::HolderSet { depth: 4 }.package(), "holder_set_d4");
        assert_eq!(CircuitId::HolderSet { depth: 2 }.package(), "holder_set_d2");
    }

    // The holder-set LEAF is the holder-key digest (NOT the issuer h2(x,y) shape):
    // a depth-2 tree (4 holders) built by hand from holder_key_digest leaves
    // matches `holder_set_root`, and the membership witness for each index re-folds
    // to the root (the circuit's fold). This is the single-source-of-truth pin the
    // bind_holder_set anchor rests on.
    #[test]
    fn holder_set_root_and_witness_recompute() {
        let holders = holder_set(&[100, 101, 102, 103]);
        let depth = 2;
        let leaves: Vec<Fr> = holders
            .iter()
            .map(|h| holder_key_digest(h).unwrap())
            .collect();
        let n01 = h2(leaves[0], leaves[1]);
        let n23 = h2(leaves[2], leaves[3]);
        let expected_root = h2(n01, n23);
        assert_eq!(holder_set_root(&holders, depth), Some(expected_root));

        for index in 0..4u64 {
            let sibs = holder_set_membership_witness(&holders, depth, index).unwrap();
            // Re-fold bottom-up using directions = LSB-first bits of index.
            let mut node = leaves[index as usize];
            let mut pos = index;
            for sib in &sibs {
                let is_right = pos & 1 == 1;
                node = if is_right { h2(*sib, node) } else { h2(node, *sib) };
                pos /= 2;
            }
            assert_eq!(node, expected_root, "fold for index {index} reaches the root");
        }
    }

    // The holder-set leaf is the DIGEST, distinct from the issuer key-set leaf
    // `h2(pk.x, pk.y)` over the SAME key (the domain tag ZKSIG_HK separates them) --
    // so a holder-set root can never be confused with an issuer key-set root.
    #[test]
    fn holder_set_leaf_differs_from_issuer_key_leaf() {
        let hpk = SecretKey::from_seed(102).public_key();
        let (x, y) = hpk.coords().unwrap();
        let holder_leaf = holder_key_digest(&hpk).unwrap();
        let issuer_leaf = h2(x, y);
        assert_ne!(
            holder_leaf, issuer_leaf,
            "holder-set leaf (digest) must differ from issuer key-set leaf (h2(x,y))"
        );
    }

    // A holder set smaller than the tree is padded; padding leaves are 0, distinct
    // from any digest leaf (so a holder never collides into a padding slot).
    #[test]
    fn padded_holder_set_root_is_defined() {
        let holders = holder_set(&[100, 101, 102]); // 3 holders in a 4-slot tree
        let depth = 2;
        let leaves: Vec<Fr> = holders
            .iter()
            .map(|h| holder_key_digest(h).unwrap())
            .collect();
        let n01 = h2(leaves[0], leaves[1]);
        let n23 = h2(leaves[2], padding_leaf()); // slot 3 is padding
        let expected = h2(n01, n23);
        assert_eq!(holder_set_root(&holders, depth), Some(expected));
        for leaf in &leaves {
            assert_ne!(*leaf, padding_leaf(), "a holder leaf is never the padding leaf");
        }
    }

    #[test]
    fn out_of_range_index_and_overflow_are_none() {
        let holders = holder_set(&[100, 101, 102, 103]);
        assert_eq!(holder_set_membership_witness(&holders, 2, 4), None); // idx 4 OOB
        // 5 holders do not fit a depth-2 (4-slot) tree.
        let big = holder_set(&[1, 2, 3, 4, 5]);
        assert_eq!(holder_set_root(&big, 2), None);
    }

    // The full witness assembly: an in-set holder's PoK witness's hpk hashes to the
    // leaf at the claimed index, and the membership path re-folds to the root the
    // verifier would derive. The Prover.toml renders all seven fields in main's
    // declaration order (PUBLIC challenge, holder_set_root; PRIVATE hsk, hpk_x,
    // hpk_y, index, siblings).
    #[test]
    fn holder_set_witness_assembles_consistently() {
        let holders = holder_set(&[100, 101, 102, 103]);
        let depth = 2u32;
        let holder_idx = 2usize;
        let holder_sk = SecretKey::from_seed(102);
        let pok = holder_pok_witness(&holder_sk).unwrap();
        let siblings = holder_set_membership_witness(&holders, depth, holder_idx as u64).unwrap();

        // The PoK witness's hpk coords digest to the leaf at holder_idx.
        let leaf = holder_key_digest(&holders[holder_idx]).unwrap();
        assert_eq!(leaf, pok.holder_pk_digest, "witness digest is the set leaf");

        let w = HolderSetWitness { pok, index: holder_idx as u64, siblings };
        let root = holder_set_root(&holders, depth).unwrap();
        let toml = holder_set_prover_toml(&Fr::from(0x2au64), &root, &w);
        for field in ["challenge", "holder_set_root", "hsk", "hpk_x", "hpk_y", "index", "siblings"] {
            assert!(toml.contains(field), "toml must render {field}");
        }
        // The PUBLIC inputs come first (challenge, then holder_set_root), then private.
        let challenge_pos = toml.find("challenge").unwrap();
        let root_pos = toml.find("holder_set_root").unwrap();
        let hsk_pos = toml.find("hsk").unwrap();
        assert!(challenge_pos < root_pos && root_pos < hsk_pos);
    }
}
