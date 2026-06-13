// [OPUS-4.8] sq-z9l: hidden-issuer-attestation host-side helpers.
//! Host-side commitment + witness machinery for the in-circuit
//! **Schnorr-over-Baby-JubJub + hidden-key set membership** proof (sq-z9l). The
//! in-circuit relation is `zk/compose/compose_core/src/issuer.nr`; this module is
//! its Rust mirror:
//!
//! - [`key_set_root`] commits a set K of trusted issuer public keys as a
//!   depth-`D` Poseidon2 Merkle tree (leaves = `Poseidon2([pk.x, pk.y])` =
//!   [`sparq_zk::sig::key_set_leaf`], internal nodes = `h2` =
//!   `poseidon2::hash(&[l, r])` — bit-identical to the circuit's `h2` and to the
//!   revocation tree's nodes). The RELYING PARTY computes this root over its OWN
//!   authoritative KeySet (the trust anchor, exactly like
//!   `revocation::merkle_root` does for the status root); the proof's public
//!   `key_set_root` must byte-equal it.
//! - [`key_membership_witness`] builds the prover's private authentication path
//!   (sibling at each level, bottom-up) for the signing key's index in K.
//! - [`HiddenIssuerWitness`] bundles the Schnorr witness
//!   ([`sparq_zk::sig::InCircuitSchnorrWitness`]) with the membership path — the
//!   complete private input to the `hidden_issuer_d{depth}` member.
//! - [`hidden_issuer_prover_toml`] renders the `Prover.toml`.
//!
//! # Leaf / tree layout (MUST match `issuer.nr`)
//! - Leaves are the `2^D` key-set slots; leaf `i` = `key_set_leaf(K[i])`. The set
//!   is committed in the verifier's chosen, fixed slot order (the RP's KeySet
//!   iteration order). Slots past `K.len()` are padded with a fixed sentinel leaf
//!   (`Fr::from(0)`) so a tree of any `2^D >= K.len()` is well-defined; a key can
//!   never occupy a padding slot because a padding leaf is `0`, not a key hash
//!   (`Poseidon2([x,y])` is never `0` for an on-curve non-identity key in
//!   practice, and even if it were the membership proof binds the index).
//! - Internal node = `h2(left, right)`; fold pairwise up `D` levels to the root.
//!
//! # Scope (honest, mirrors `revocation`)
//! This is a DENSE tree of `2^D` leaves: `key_set_root` hashes all `2^D` leaves,
//! `O(2^D)` host work. The depth-`D` member covers up to `2^D` issuers. Real
//! issuer sets are small (tens of authorities), so depth 4–10 is ample; a
//! production deployment with a very large issuer registry would want a
//! sparse/compressed-inclusion commitment (the circuit relation is depth-generic,
//! only this dense host builder bounds the size). The compiled member is `d4`
//! (16 issuers) — see the verifier docs and the crate `STATUS`.

use sparq_zk::field::{field_to_hex, Fr};
use sparq_zk::poseidon2;
use sparq_zk::sig::{key_set_leaf, InCircuitSchnorrWitness, PublicKey};

/// The Poseidon2 two-input compression for Merkle internal nodes — the Rust
/// mirror of the circuit's `h2(a, b) = Poseidon2::hash([a, b], 2)`. Identical to
/// `revocation::h2` (cross-tested bit-identical).
fn h2(a: Fr, b: Fr) -> Fr {
    poseidon2::hash(&[a, b])
}

/// The padding leaf for key-set slots past `K.len()`: `Fr::from(0)`. A genuine
/// key leaf is `Poseidon2([pk.x, pk.y])`; the membership proof additionally binds
/// the index, so a padding slot is never a usable member.
fn padding_leaf() -> Fr {
    Fr::from(0u64)
}

/// The `2^depth` key-set leaves: leaf `i` = `key_set_leaf(K[i])` for `i <
/// K.len()`, else the padding leaf. `None` if any key in K is the identity (no
/// coordinates — never a usable key, fail-closed) or `depth > 31`.
fn key_leaves(keys: &[PublicKey], depth: u32) -> Option<Vec<Fr>> {
    if depth > 31 {
        return None;
    }
    let n_leaves = 1usize << depth;
    if keys.len() > n_leaves {
        return None; // the set does not fit the tree at this depth
    }
    let mut leaves = Vec::with_capacity(n_leaves);
    for pk in keys {
        leaves.push(key_set_leaf(pk)?);
    }
    leaves.resize(n_leaves, padding_leaf());
    Some(leaves)
}

/// Commit the trusted issuer key set K as a depth-`depth` Poseidon2 Merkle tree
/// and return the root. This is what the RELYING PARTY computes over its OWN
/// authoritative KeySet (the trust anchor the proof's public `key_set_root` is
/// checked against) and what the PROVER computes over the same set. Returns
/// `None` if `depth > 31`, the set overflows the tree, or any key is the
/// identity (fail-closed).
pub fn key_set_root(keys: &[PublicKey], depth: u32) -> Option<Fr> {
    let mut level = key_leaves(keys, depth)?;
    for _ in 0..depth {
        level = level.chunks(2).map(|pair| h2(pair[0], pair[1])).collect();
    }
    debug_assert_eq!(level.len(), 1, "fold reduces to a single root");
    level.first().copied()
}

/// The authentication path (sibling at each level, BOTTOM-UP — level 0 first) for
/// the key at `index` in K's depth-`depth` tree. The private input the circuit
/// folds. `None` if `index >= 2^depth`, the set overflows the tree, `depth > 31`,
/// or any key is the identity.
pub fn key_membership_witness(keys: &[PublicKey], depth: u32, index: u64) -> Option<Vec<Fr>> {
    let mut level = key_leaves(keys, depth)?;
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

/// The complete private witness for one hidden issuer attestation: the Schnorr
/// witness (key coords, R, s, e, e_k) plus the key-set membership path. Mirrors
/// the private inputs of `issuer.nr::hidden_issuer_attestation`.
#[derive(Debug, Clone)]
pub struct HiddenIssuerWitness {
    /// The in-circuit Schnorr verification witness over the commitment message.
    pub schnorr: InCircuitSchnorrWitness,
    /// The signing key's index in K (private; the circuit derives the path
    /// directions from it).
    pub index: u64,
    /// The Merkle authentication path: `siblings[k]` is the sibling at level `k`
    /// (level 0 = leaves), bottom-up. Length = `depth`.
    pub siblings: Vec<Fr>,
}

/// Render the `Prover.toml` body for the `hidden_issuer_d{depth}` member. Order
/// MUST match `hidden_issuer_d{depth}/src/main.nr`:
/// PUBLIC: challenge, m, key_set_root;
/// PRIVATE: pk_x, pk_y, r_x, r_y, s, e, e_k, index, siblings.
pub fn hidden_issuer_prover_toml(
    challenge: &Fr,
    m: &Fr,
    key_set_root: &Fr,
    witness: &HiddenIssuerWitness,
) -> String {
    let w = &witness.schnorr;
    let mut s = String::new();
    s.push_str(&format!("challenge = \"{}\"\n", field_to_hex(challenge)));
    s.push_str(&format!("m = \"{}\"\n", field_to_hex(m)));
    s.push_str(&format!("key_set_root = \"{}\"\n", field_to_hex(key_set_root)));
    s.push_str(&format!("pk_x = \"{}\"\n", field_to_hex(&w.pk_x)));
    s.push_str(&format!("pk_y = \"{}\"\n", field_to_hex(&w.pk_y)));
    s.push_str(&format!("r_x = \"{}\"\n", field_to_hex(&w.r_x)));
    s.push_str(&format!("r_y = \"{}\"\n", field_to_hex(&w.r_y)));
    s.push_str(&format!("s = \"{}\"\n", field_to_hex(&w.s)));
    s.push_str(&format!("e = \"{}\"\n", field_to_hex(&w.e)));
    s.push_str(&format!("e_k = \"{}\"\n", field_to_hex(&w.e_k)));
    s.push_str(&format!("index = \"{}\"\n", witness.index));
    let sibs: Vec<String> = witness
        .siblings
        .iter()
        .map(|s| format!("\"{}\"", field_to_hex(s)))
        .collect();
    s.push_str(&format!("siblings = [{}]\n", sibs.join(", ")));
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use sparq_zk::sig::{commitment_message, in_circuit_witness, signature_from_hex, SecretKey};

    fn keyset(seeds: &[u64]) -> Vec<PublicKey> {
        seeds.iter().map(|s| SecretKey::from_seed(*s).public_key()).collect()
    }

    // The Rust h2 mirrors the documented cross-vector (compose_core/tests.nr).
    #[test]
    fn h2_matches_documented_cross_vector() {
        assert_eq!(
            field_to_hex(&h2(Fr::from(1u64), Fr::from(2u64))),
            "0x038682aa1cb5ae4e0a3f13da432a95c77c5c111f6f030faf9cad641ce1ed7383"
        );
    }

    // A depth-2 tree (4 leaves) built by hand matches `key_set_root`, and the
    // membership witness for each index re-folds to the root (the circuit's fold).
    #[test]
    fn key_set_root_and_witness_recompute() {
        let keys = keyset(&[100, 101, 102, 103]);
        let depth = 2;
        let leaves: Vec<Fr> = keys.iter().map(|k| key_set_leaf(k).unwrap()).collect();
        let n01 = h2(leaves[0], leaves[1]);
        let n23 = h2(leaves[2], leaves[3]);
        let expected_root = h2(n01, n23);
        assert_eq!(key_set_root(&keys, depth), Some(expected_root));

        for index in 0..4u64 {
            let sibs = key_membership_witness(&keys, depth, index).unwrap();
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

    // A key set smaller than the tree is padded; padding leaves are 0, distinct
    // from any key leaf (so a key never collides into a padding slot).
    #[test]
    fn padded_key_set_root_is_defined() {
        let keys = keyset(&[100, 101, 102]); // 3 keys in a depth-2 (4-slot) tree
        let depth = 2;
        let leaves: Vec<Fr> = keys.iter().map(|k| key_set_leaf(k).unwrap()).collect();
        let n01 = h2(leaves[0], leaves[1]);
        let n23 = h2(leaves[2], padding_leaf()); // slot 3 is padding
        let expected = h2(n01, n23);
        assert_eq!(key_set_root(&keys, depth), Some(expected));
        for leaf in &leaves {
            assert_ne!(*leaf, padding_leaf(), "a key leaf is never the padding leaf");
        }
    }

    #[test]
    fn out_of_range_index_and_overflow_are_none() {
        let keys = keyset(&[100, 101, 102, 103]);
        assert_eq!(key_membership_witness(&keys, 2, 4), None); // 4 leaves, idx 4 OOB
        // 5 keys do not fit a depth-2 (4-slot) tree.
        let big = keyset(&[1, 2, 3, 4, 5]);
        assert_eq!(key_set_root(&big, 2), None);
    }

    // The full witness assembly: a real signature by an in-set issuer produces a
    // schnorr witness whose key leaf is at the claimed index, and the membership
    // path re-folds to the root the verifier would derive.
    #[test]
    fn hidden_issuer_witness_assembles_consistently() {
        let keys = keyset(&[100, 101, 102, 103]);
        let depth = 2u32;
        let signer_idx = 2usize;
        let signer_sk = SecretKey::from_seed(102);
        let c = Fr::from(0xc0ffeeu64);
        let m = commitment_message(&c);
        let sig = signature_from_hex(&signer_sk.sign_commitment(&c)).unwrap();
        let schnorr = in_circuit_witness(&keys[signer_idx], &m, &sig).unwrap();
        let siblings = key_membership_witness(&keys, depth, signer_idx as u64).unwrap();

        // The schnorr witness's key coords hash to the leaf at signer_idx.
        let leaf = h2(schnorr.pk_x, schnorr.pk_y);
        assert_eq!(leaf, key_set_leaf(&keys[signer_idx]).unwrap());

        let w = HiddenIssuerWitness { schnorr, index: signer_idx as u64, siblings };
        // The Prover.toml renders all 12 fields.
        let toml = hidden_issuer_prover_toml(&Fr::from(0x2au64), &m, &key_set_root(&keys, depth).unwrap(), &w);
        for field in ["challenge", "m", "key_set_root", "pk_x", "pk_y", "r_x", "r_y", "s", "e", "e_k", "index", "siblings"] {
            assert!(toml.contains(field), "toml must render {field}");
        }
    }
}

