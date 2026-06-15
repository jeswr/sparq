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
use sparq_zk::sig::{in_circuit_holder_witness, InCircuitHolderPokWitness, SecretKey};

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
}
