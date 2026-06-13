// [OPUS-4.8] sq-3e5 + sq-h2v: hidden-index revocation host-side helpers.
//! Host-side commitment + witness machinery for the hidden-index revocation
//! proof (sq-3e5 / sq-h2v). The in-circuit relation is
//! `zk/compose/compose_core/src/revoke.nr`; this module is its Rust mirror:
//!
//! - [`merkle_root`] commits a [`StatusListSnapshot`] as a depth-`D` Poseidon2
//!   Merkle tree (leaves = status bits, internal nodes = `h2` =
//!   `poseidon2::hash(&[l, r])` — bit-identical to the circuit's `h2`). The
//!   RELYING PARTY computes this over its OWN authoritative snapshot; the proof's
//!   public `root` must byte-equal it. The PROVER computes the same root over the
//!   list it holds.
//! - [`merkle_witness`] builds the prover's private witness (the leaf bit + the
//!   authentication path of siblings) for a given index.
//! - [`revoke_prover_toml`] renders the `Prover.toml` for the
//!   `revoke_unset_d{depth}` member.
//!
//! # Leaf / tree layout (MUST match `revoke.nr`)
//! - A status list of `2^D` slots. Bit `i` (LSB-first within each byte, the
//!   StatusList2021 convention — see [`StatusListSnapshot::bit`]) is the leaf at
//!   index `i`. Leaf field value = `Fr::from(bit as u64)` (0 or 1), exactly the
//!   circuit's `leaf = bit`.
//! - Internal node = `h2(left, right)`. Level 0 is the leaves; we fold pairwise
//!   up `D` levels to the single root.
//! - Indices past the snapshot's `bits` length read as SET (revoked) via
//!   [`StatusListSnapshot::bit`] — fail-closed, mirroring the verifier.
//!
//! # Scope (honest)
//! This is a DENSE tree of `2^D` leaves: `merkle_root` hashes all `2^D` leaves,
//! so it is `O(2^D)` host work and the depth-10 member covers 1024 indices. A
//! production status list (StatusList2021 lists are commonly 2^17+) needs a
//! SPARSE / compressed-inclusion commitment (most leaves are 0) — see the crate
//! verifier docs (`bind_hidden_revocation`) for the deferred-work note. The
//! circuit relation itself is depth-generic; only this dense host builder and the
//! single compiled `d10` member bound the representative size.

use crate::manifest::StatusListSnapshot;
use sparq_zk::field::{field_to_hex, Fr};
use sparq_zk::poseidon2;

/// The Poseidon2 two-input compression used for Merkle internal nodes — the Rust
/// mirror of the circuit's `h2(a, b) = Poseidon2::hash([a, b], 2)`. Cross-tested
/// bit-identical (`crates/sparq-zk/tests/poseidon2_noir_cross.rs`).
fn h2(a: Fr, b: Fr) -> Fr {
    poseidon2::hash(&[a, b])
}

/// The field value of the leaf at `index`: the status bit (0 = active, 1 =
/// revoked) as a field element. Out-of-range reads as 1 (revoked), fail-closed —
/// identical to [`StatusListSnapshot::bit`].
fn leaf(snapshot: &StatusListSnapshot, index: u64) -> Fr {
    Fr::from(u64::from(snapshot.bit(index)))
}

/// Commit `snapshot` as a depth-`depth` Poseidon2 Merkle tree and return the
/// root. Leaves are the `2^depth` status bits (indices `0..2^depth`, padded with
/// SET/revoked bits past the snapshot length — fail-closed). Internal nodes are
/// `h2`. This is what the RELYING PARTY computes over its authoritative snapshot
/// (the trust anchor the proof's public root is checked against) and what the
/// PROVER computes over the list it holds.
///
/// Returns `None` if `depth` is implausibly large (`> 31`, i.e. more than ~2^31
/// leaves) — a guard so a hostile/buggy `depth` cannot trigger an astronomical
/// allocation. (The compiled member is `d10`; real callers pass small depths.)
pub fn merkle_root(snapshot: &StatusListSnapshot, depth: u32) -> Option<Fr> {
    if depth > 31 {
        return None;
    }
    let n_leaves = 1usize << depth;
    // Level 0: the leaves.
    let mut level: Vec<Fr> = (0..n_leaves as u64).map(|i| leaf(snapshot, i)).collect();
    // Fold pairwise up to the root.
    for _ in 0..depth {
        level = level
            .chunks(2)
            .map(|pair| h2(pair[0], pair[1]))
            .collect();
    }
    debug_assert_eq!(level.len(), 1, "fold reduces to a single root");
    level.first().copied()
}

/// The hidden-index revocation witness for `index`: the leaf bit and the
/// authentication path (sibling at each level, BOTTOM-UP — level 0 first), the
/// private inputs the `revoke_unset_d{depth}` circuit folds. Mirrors `revoke.nr`'s
/// `(bit, siblings)` exactly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MerkleWitness {
    /// The status bit at `index` (the circuit's private `bit`; must be 0 for a
    /// satisfiable, i.e. unrevoked, proof).
    pub bit: Fr,
    /// The authentication path: `siblings[k]` is the sibling hash at level `k`
    /// (level 0 = leaves), bottom-up. Length = `depth`.
    pub siblings: Vec<Fr>,
}

/// Build the [`MerkleWitness`] for `index` in `snapshot`'s depth-`depth` tree.
/// `None` if `index >= 2^depth` (out of range for the tree — the circuit also
/// range-bounds the index) or `depth > 31`.
pub fn merkle_witness(snapshot: &StatusListSnapshot, depth: u32, index: u64) -> Option<MerkleWitness> {
    if depth > 31 {
        return None;
    }
    let n_leaves = 1u64 << depth;
    if index >= n_leaves {
        return None;
    }
    let bit = leaf(snapshot, index);
    // Recompute each level, recording the sibling on the path to `index`.
    let mut level: Vec<Fr> = (0..n_leaves).map(|i| leaf(snapshot, i)).collect();
    let mut pos = index as usize;
    let mut siblings = Vec::with_capacity(depth as usize);
    for _ in 0..depth {
        // Sibling is the partner in this node's pair: flip the low bit of pos.
        let sib = pos ^ 1;
        siblings.push(level[sib]);
        level = level.chunks(2).map(|pair| h2(pair[0], pair[1])).collect();
        pos /= 2;
    }
    Some(MerkleWitness { bit, siblings })
}

/// Render the `Prover.toml` body for the `revoke_unset_d{depth}` member. Order
/// MUST match `revoke_unset_d{depth}/src/main.nr`: challenge, root,
/// index_commitment (public); index, bit, blinding, siblings (private).
///
/// [OPUS-4.8] sq-ayv: the member now binds a HIDING index commitment. The
/// `index_commitment` PUBLIC input is recomputed in-circuit from `index` +
/// `blinding` and byte-matched by the verifier against the ISSUER-SIGNED
/// commitment, so the index proven unset is provably the one the issuer committed
/// to. `index_commitment` MUST equal `sparq_zk::sig::status_index_commitment(index,
/// blinding)` (the issuer-signed value); the host computes it that way.
pub fn revoke_prover_toml(
    challenge: &Fr,
    root: &Fr,
    index_commitment: &Fr,
    index: u64,
    blinding: &Fr,
    witness: &MerkleWitness,
) -> String {
    let mut s = String::new();
    s.push_str(&format!("challenge = \"{}\"\n", field_to_hex(challenge)));
    s.push_str(&format!("root = \"{}\"\n", field_to_hex(root)));
    s.push_str(&format!("index_commitment = \"{}\"\n", field_to_hex(index_commitment)));
    s.push_str(&format!("index = \"{index}\"\n"));
    s.push_str(&format!("bit = \"{}\"\n", field_to_hex(&witness.bit)));
    s.push_str(&format!("blinding = \"{}\"\n", field_to_hex(blinding)));
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

    fn snap(bits: Vec<u8>) -> StatusListSnapshot {
        StatusListSnapshot { status_list: "http://ex/s".to_string(), version: 1, bits }
    }

    // The Rust h2 mirrors the circuit's documented vector: h2(1,2) ==
    // 0x038682aa... (compose_core/tests.nr cross-vector).
    #[test]
    fn h2_matches_documented_cross_vector() {
        let got = field_to_hex(&h2(Fr::from(1u64), Fr::from(2u64)));
        assert_eq!(
            got,
            "0x038682aa1cb5ae4e0a3f13da432a95c77c5c111f6f030faf9cad641ce1ed7383"
        );
    }

    // A depth-2 tree (4 leaves) built by hand matches `merkle_root`. Leaves:
    // [0,1,0,0] -> root = h2(h2(0,1), h2(0,0)). Same construction the Noir
    // accept tests use.
    #[test]
    fn merkle_root_depth2_matches_hand_built() {
        let s = snap(vec![0b0000_0010]); // bit 1 set, rest unset
        let expected = h2(h2(Fr::from(0u64), Fr::from(1u64)), h2(Fr::from(0u64), Fr::from(0u64)));
        assert_eq!(merkle_root(&s, 2), Some(expected));
    }

    // A witness for an unset index recomputes the root via the same fold the
    // circuit does (bit + siblings + index-derived directions).
    #[test]
    fn witness_recomputes_root() {
        let s = snap(vec![0b0000_0010]); // index 1 revoked; 0,2,3 active
        let depth = 2;
        let root = merkle_root(&s, depth).unwrap();
        for index in [0u64, 2, 3] {
            let w = merkle_witness(&s, depth, index).unwrap();
            assert_eq!(w.bit, Fr::from(0u64), "index {index} should be active");
            // Re-fold bottom-up using directions = LSB-first bits of index.
            let mut node = w.bit;
            let mut pos = index;
            for sib in &w.siblings {
                let is_right = pos & 1 == 1;
                node = if is_right { h2(*sib, node) } else { h2(node, *sib) };
                pos /= 2;
            }
            assert_eq!(node, root, "fold for index {index} must reach the root");
        }
        // The revoked index has bit 1 (the circuit's bit-unset assert would fail).
        assert_eq!(merkle_witness(&s, depth, 1).unwrap().bit, Fr::from(1u64));
    }

    #[test]
    fn out_of_range_index_is_none() {
        let s = snap(vec![0u8]);
        assert_eq!(merkle_witness(&s, 2, 4), None); // 2^2 = 4 leaves, index 4 OOB
    }

    #[test]
    fn implausible_depth_is_none() {
        let s = snap(vec![0u8]);
        assert_eq!(merkle_root(&s, 32), None);
        assert_eq!(merkle_witness(&s, 32, 0), None);
    }
}
