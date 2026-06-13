// [OPUS-4.8] written while Fable 5 unavailable — re-review when Fable returns.
//! Issuer signatures over per-graph commitments (audit #3, the M1 attestation
//! foundation).
//!
//! # The hole this closes
//! Before this module, `C(G)` (`crate::commit`) was a prover-supplied public
//! input with NO signature: the scan circuit only proves
//! `commit_fold(witnessed_leaves, count) == commitments[g]` for a
//! PROVER-CHOSEN `commitments[g]`, and the verifier never resolved an issuer
//! key or checked a signature. So the prover was effectively the issuer of
//! every fact it proved — it could invent any triple set, or drop a
//! suspension/revocation triple and recommit over the truncated leaves. This
//! module makes every commitment carry an issuer signature the verifier checks
//! against a DISCLOSED key-set `K`: an unsigned/prover-invented commitment, a
//! truncated-leaf suppression, and a key-not-in-`K` signature all fail.
//!
//! # Scheme — Schnorr over Baby-JubJub (the BN254 embedded curve), Poseidon2
//! challenge hash (`zk:poseidon2-schnorr-v1`).
//!
//! Baby-JubJub's *base* field is exactly BN254's scalar field — i.e. point
//! coordinates live in [`crate::field::Fr`] (= Noir's `Field`). That is the
//! whole point of picking this curve over Ed25519: the challenge hash is a
//! Poseidon2 sponge over field coordinates, so this signature is EXACTLY the
//! one an in-circuit verifier would later check — the privacy-preserving
//! upgrade ("prove a commitment was signed by SOME key in `K`, without
//! revealing which") is *move this verification in-circuit + a set-membership
//! gadget over `K`*, not a scheme swap.
//!
//! ## v1 placement: VERIFIER-SIDE (the sound interim)
//! The full-privacy choice the goal wants — in-circuit signature + set
//! membership over an UNDISCLOSED signing key — is expensive (in-circuit
//! Baby-JubJub scalar mul is a heavy black box; see `noir-optimisation` cost
//! table) and needs new circuit members + a vk recompute. v1 verifies the
//! signature VERIFIER-SIDE over the already-public-input-bound `commitments[g]`
//! (audit #1's reconstruction byte-binds `C(G)` into the proof), which closes
//! the unsigned-commitment hole NOW. **Privacy gap of the interim (documented
//! loudly):** the verifier checks `pk_i` in the clear, so it reveals WHICH
//! issuer signed each graph, not merely "some key in `K`". The in-circuit
//! undisclosed-key upgrade removes that leak.
//!
//! ## Modularity (per Jesse's modular-commitment/signature design)
//! [`SignatureScheme`] tags the scheme so BBS+ / SD-JWT-VC / a post-quantum
//! candidate can ship as parallel options for the paper's per-signature
//! performance + security table. v1 ships `poseidon2-schnorr-v1` only.

use crate::field::Fr;
use crate::poseidon2;
use ark_ec::{twisted_edwards::Affine, AffineRepr, CurveGroup, PrimeGroup};
use ark_ed_on_bn254::{EdwardsConfig, EdwardsProjective, Fr as JjScalar};
use ark_ff::{BigInteger, PrimeField, UniformRand};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};

/// Domain separator folded into the signed message (so an issuer signature
/// over `C(G)` can never be replayed as a signature over some other field
/// element with a different meaning — domain-separation discipline,
/// `verifiable-credentials-zk` skill). Distinct from the leaf/commitment IVs.
const SIG_DOMAIN_COMMITMENT: u64 = 0x5a4b_5349_475f_4331; // "ZKSIG_C1"

/// Which signature scheme an issuer key/signature uses. v1 ships Schnorr over
/// Baby-JubJub only; the enum is the modularity swap-point (BBS+, SD-JWT-VC,
/// post-quantum candidates ship as parallel variants).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureScheme {
    /// Schnorr over Baby-JubJub with a Poseidon2 challenge (`zk:poseidon2-schnorr-v1`).
    Poseidon2SchnorrV1,
}

impl SignatureScheme {
    /// The `zk:cryptosuite` IRI for this scheme.
    pub const POSEIDON2_SCHNORR_V1_IRI: &'static str =
        "https://sparq.dev/ns/zk#poseidon2-schnorr-v1";

    pub fn cryptosuite_iri(self) -> &'static str {
        match self {
            SignatureScheme::Poseidon2SchnorrV1 => Self::POSEIDON2_SCHNORR_V1_IRI,
        }
    }

    pub fn from_cryptosuite_iri(iri: &str) -> Option<Self> {
        match iri {
            Self::POSEIDON2_SCHNORR_V1_IRI => Some(SignatureScheme::Poseidon2SchnorrV1),
            _ => None,
        }
    }
}

/// An issuer's public verification key: a Baby-JubJub point `pk = sk·G`. Its
/// affine coordinates `(x, y)` are field elements (= Noir `Field`), so the key
/// itself is in the same arena as the commitment it signs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PublicKey(pub Affine<EdwardsConfig>);

/// An issuer secret key (a Baby-JubJub scalar). Test/issuance-side only — a
/// relying party never sees it.
#[derive(Debug, Clone, Copy)]
pub struct SecretKey(pub JjScalar);

/// A Schnorr signature `(R, s)` over a message field element: `R = k·G`,
/// `s = k + e·sk` with `e = Poseidon2(DOMAIN, R.x, R.y, pk.x, pk.y, m)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Signature {
    /// The nonce commitment `R = k·G`.
    pub r: Affine<EdwardsConfig>,
    /// `s = k + e·sk` in the curve's scalar field.
    pub s: JjScalar,
}

impl SecretKey {
    /// Derive the public key `pk = sk·G`.
    pub fn public_key(&self) -> PublicKey {
        let g = EdwardsProjective::generator();
        PublicKey((g * self.0).into_affine())
    }
}

/// The challenge `e = Poseidon2([DOMAIN, R.x, R.y, pk.x, pk.y, m])`, reduced
/// into the curve's scalar field. Computed over the *base* field (= `Fr`) so it
/// is in-circuit-recomputable, then reduced via its big-endian bytes (the
/// standard Schnorr "hash-to-scalar" step). Both signer and verifier call this,
/// so a drift in the challenge derivation can never make a forged signature
/// verify (single source of truth, mirroring the `verify_*_relation` discipline
/// in the noir-optimisation skill).
fn challenge(r: &Affine<EdwardsConfig>, pk: &Affine<EdwardsConfig>, m: &Fr) -> JjScalar {
    let (rx, ry) = r.xy().unwrap_or((Fr::from(0u64), Fr::from(0u64)));
    let (px, py) = pk.xy().unwrap_or((Fr::from(0u64), Fr::from(0u64)));
    let e_base: Fr = poseidon2::hash(&[
        Fr::from(SIG_DOMAIN_COMMITMENT),
        rx,
        ry,
        px,
        py,
        *m,
    ]);
    // Reduce the base-field challenge into the scalar field (big-endian bytes,
    // mod the scalar order). Deterministic and recomputable in-circuit.
    JjScalar::from_be_bytes_mod_order(&e_base.into_bigint().to_bytes_be())
}

/// The signed message for a per-graph commitment: a domain-separated binding of
/// `C(G)` to the issuer. The signature is over THIS value, never the raw
/// commitment — so an issuer's commitment signature is not interchangeable with
/// any other use of the same field element.
pub fn commitment_message(commitment: &Fr) -> Fr {
    poseidon2::hash(&[Fr::from(SIG_DOMAIN_COMMITMENT), *commitment])
}

/// Sign a message field element `m` with `sk`. The nonce `k` is drawn from
/// `rng`; this is the issuance-side path (tests + an issuer tool). A relying
/// party only ever calls [`verify`].
pub fn sign<R: ark_std::rand::RngCore + ark_std::rand::CryptoRng>(
    sk: &SecretKey,
    m: &Fr,
    rng: &mut R,
) -> Signature {
    let g = EdwardsProjective::generator();
    let k = JjScalar::rand(rng);
    let r_pt = (g * k).into_affine();
    let pk = sk.public_key().0;
    let e = challenge(&r_pt, &pk, m);
    let s = k + e * sk.0;
    Signature { r: r_pt, s }
}

/// Verify a Schnorr signature: `s·G == R + e·pk`. Returns `false` (never
/// panics) on any malformed input. This is the verifier-side gate audit #3
/// requires — the relying party resolves `pk` from the disclosed key-set `K`,
/// recomputes `e`, and checks this equation over the commitment message.
pub fn verify(pk: &PublicKey, m: &Fr, sig: &Signature) -> bool {
    // Reject the identity / off-curve / non-prime-order points defensively:
    // `R` and `pk` must be on the curve and in the prime-order subgroup, else a
    // small-subgroup point could let a forger pass the equation.
    if !sig.r.is_on_curve()
        || !pk.0.is_on_curve()
        || !sig.r.is_in_correct_subgroup_assuming_on_curve()
        || !pk.0.is_in_correct_subgroup_assuming_on_curve()
    {
        return false;
    }
    let g = EdwardsProjective::generator();
    let e = challenge(&sig.r, &pk.0, m);
    // lhs = s·G ; rhs = R + e·pk
    let lhs = g * sig.s;
    let rhs = sig.r.into_group() + pk.0 * e;
    lhs == rhs
}

// --- serialization (registry literals + manifest fields) ------------------

/// Serialize a public key to lowercase hex (compressed Baby-JubJub point). The
/// `zk:issuerKey` registry literal and the manifest key-set carry this.
pub fn public_key_to_hex(pk: &PublicKey) -> String {
    let mut bytes = Vec::new();
    pk.0
        .serialize_compressed(&mut bytes)
        .expect("affine point serializes");
    to_hex(&bytes)
}

/// Parse a public key from hex. `None` on malformed/odd hex or a point that is
/// not a valid compressed Baby-JubJub point (fail-closed).
pub fn public_key_from_hex(s: &str) -> Option<PublicKey> {
    let bytes = from_hex(s)?;
    Affine::<EdwardsConfig>::deserialize_compressed(&bytes[..])
        .ok()
        .map(PublicKey)
}

/// Serialize a signature to hex: `compressed(R) ‖ scalar(s)` (each
/// canonical-serialized, concatenated).
pub fn signature_to_hex(sig: &Signature) -> String {
    let mut bytes = Vec::new();
    sig.r
        .serialize_compressed(&mut bytes)
        .expect("R serializes");
    sig.s
        .serialize_compressed(&mut bytes)
        .expect("s serializes");
    to_hex(&bytes)
}

/// Parse a signature from hex (`compressed(R) ‖ scalar(s)`). `None` on any
/// malformed input (fail-closed — prover-controlled bytes never panic).
pub fn signature_from_hex(s: &str) -> Option<Signature> {
    let bytes = from_hex(s)?;
    // The compressed Edwards point is 32 bytes; the scalar is the remainder.
    if bytes.len() < 32 {
        return None;
    }
    let (r_bytes, s_bytes) = bytes.split_at(32);
    let r = Affine::<EdwardsConfig>::deserialize_compressed(r_bytes).ok()?;
    let s_scalar = JjScalar::deserialize_compressed(s_bytes).ok()?;
    Some(Signature { r, s: s_scalar })
}

fn to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn from_hex(s: &str) -> Option<Vec<u8>> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    if s.len() % 2 != 0 {
        return None;
    }
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(s.len() / 2);
    let mut i = 0;
    while i < bytes.len() {
        let hi = (bytes[i] as char).to_digit(16)?;
        let lo = (bytes[i + 1] as char).to_digit(16)?;
        out.push((hi * 16 + lo) as u8);
        i += 2;
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_std::rand::SeedableRng;

    fn rng() -> ark_std::rand::rngs::StdRng {
        ark_std::rand::rngs::StdRng::seed_from_u64(42)
    }

    fn keypair(seed: u64) -> (SecretKey, PublicKey) {
        let mut r = ark_std::rand::rngs::StdRng::seed_from_u64(seed);
        let sk = SecretKey(JjScalar::rand(&mut r));
        let pk = sk.public_key();
        (sk, pk)
    }

    #[test]
    fn sign_verify_round_trip() {
        let (sk, pk) = keypair(1);
        let c = Fr::from(0x1234u64);
        let m = commitment_message(&c);
        let sig = sign(&sk, &m, &mut rng());
        assert!(verify(&pk, &m, &sig), "honest signature must verify");
    }

    #[test]
    fn wrong_message_rejected() {
        // The truncated-leaf-suppression shape: a signature over C(G) must NOT
        // verify against C(G') for a different (truncated) graph commitment.
        let (sk, pk) = keypair(2);
        let m = commitment_message(&Fr::from(100u64));
        let sig = sign(&sk, &m, &mut rng());
        let m_other = commitment_message(&Fr::from(101u64));
        assert!(!verify(&pk, &m_other, &sig), "sig over a different commitment must fail");
    }

    #[test]
    fn wrong_key_rejected() {
        // The key-not-in-K shape: a signature by issuer A must not verify under
        // issuer B's key.
        let (sk_a, _pk_a) = keypair(3);
        let (_sk_b, pk_b) = keypair(4);
        let m = commitment_message(&Fr::from(7u64));
        let sig = sign(&sk_a, &m, &mut rng());
        assert!(!verify(&pk_b, &m, &sig), "sig under a different key must fail");
    }

    #[test]
    fn tampered_signature_rejected() {
        let (sk, pk) = keypair(5);
        let m = commitment_message(&Fr::from(9u64));
        let mut sig = sign(&sk, &m, &mut rng());
        sig.s += JjScalar::from(1u64); // tamper s
        assert!(!verify(&pk, &m, &sig), "tampered s must fail");
    }

    #[test]
    fn pubkey_hex_round_trip() {
        let (_sk, pk) = keypair(6);
        let h = public_key_to_hex(&pk);
        assert_eq!(public_key_from_hex(&h), Some(pk));
        assert_eq!(public_key_from_hex(&format!("0x{h}")), Some(pk));
        assert!(public_key_from_hex("zz").is_none());
        assert!(public_key_from_hex("abc").is_none()); // odd length
    }

    #[test]
    fn signature_hex_round_trip() {
        let (sk, pk) = keypair(7);
        let m = commitment_message(&Fr::from(55u64));
        let sig = sign(&sk, &m, &mut rng());
        let h = signature_to_hex(&sig);
        let back = signature_from_hex(&h).expect("round-trips");
        assert!(verify(&pk, &m, &back), "deserialized sig verifies");
        assert!(signature_from_hex("zz").is_none());
        assert!(signature_from_hex("00").is_none()); // too short for R
    }

    #[test]
    fn cryptosuite_iri_round_trip() {
        let s = SignatureScheme::Poseidon2SchnorrV1;
        assert_eq!(
            SignatureScheme::from_cryptosuite_iri(s.cryptosuite_iri()),
            Some(s)
        );
        assert_eq!(SignatureScheme::from_cryptosuite_iri("urn:other"), None);
    }
}
