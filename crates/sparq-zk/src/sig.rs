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
use ark_ff::{BigInteger, PrimeField, UniformRand, Zero};
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

    /// A deterministic secret key from a `u64` seed — issuance-tooling / test
    /// convenience so callers in crates without an `ark` dependency can mint a
    /// key without touching the curve types. NOT for production key generation
    /// (use OS entropy). Distinct seeds give distinct keys.
    // [OPUS-4.8] audit #3 test/tooling helper.
    pub fn from_seed(seed: u64) -> Self {
        use ark_std::rand::SeedableRng;
        let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(seed);
        SecretKey(JjScalar::rand(&mut rng))
    }

    /// Sign a per-graph commitment under this key, returning the signature hex
    /// (`compressed(R) ‖ s`). The signed message is the domain-separated
    /// commitment message, matching [`verify`] on the verifier side.
    ///
    /// # Nonce derivation (audit #3 codex finding #4 fix)
    /// The Schnorr nonce `k` is derived DETERMINISTICALLY from (secret key,
    /// message) via the Poseidon2 sponge (an RFC6979-style derivation in the
    /// signature's native field), NOT from any caller-supplied seed. The old
    /// `sign_commitment_seeded(.., seed: u64)` was an unsafe public API: a caller
    /// reusing a `seed` across two distinct messages produced two signatures with
    /// the SAME `R = k·G` but different `s`, from which `sk = (s1 - s2)/(e1 - e2)`
    /// is trivially recovered. Deriving `k` from `(sk, m)` removes the seed
    /// entirely — the same message always yields the same (safe) signature, and
    /// distinct messages get distinct nonces, with no way for a caller to force a
    /// nonce collision. Production keys must still be generated from OS entropy
    /// (see [`SecretKey::from_seed`]'s warning); deterministic nonces only remove
    /// the *signing-time* entropy requirement, exactly as RFC6979 does for ECDSA.
    // [OPUS-4.8] audit #3 codex #4: deterministic (sk, m) nonce, no caller seed.
    pub fn sign_commitment(&self, commitment: &Fr) -> String {
        let sig = sign_deterministic(self, &commitment_message(commitment));
        signature_to_hex(&sig)
    }
}

/// Derive a deterministic Schnorr nonce `k` from the secret key and the message
/// (RFC6979-style, in the curve's native field). The secret key's scalar is
/// mapped into the base field [`Fr`] (its big-endian bytes, reduced) and folded
/// with the message and a nonce-specific domain tag through the Poseidon2 sponge;
/// the resulting base-field digest is reduced into the scalar field. This is a
/// PRF over `(sk, m)`: it never repeats a nonce for distinct messages and never
/// leaks `sk`, so no signing-time entropy is needed and seed-reuse is impossible
/// by construction (there is no seed). Both this and the in-circuit-recomputable
/// challenge live in the same base field, keeping the scheme circuit-friendly.
// [OPUS-4.8] audit #3 codex #4.
fn derive_nonce(sk: &SecretKey, m: &Fr) -> JjScalar {
    // A distinct domain tag from the challenge so the nonce-PRF output can never
    // collide with / be mistaken for a challenge value.
    const SIG_DOMAIN_NONCE: u64 = 0x5a4b_5349_475f_4e31; // "ZKSIG_N1"
    // Map the secret scalar into the base field via its big-endian bytes. The
    // base field (BN254 scalar field) is larger than the Baby-JubJub scalar
    // field, so this reduction is injective on canonical sk encodings.
    let sk_base = Fr::from_be_bytes_mod_order(&sk.0.into_bigint().to_bytes_be());
    let k_base: Fr = poseidon2::hash(&[Fr::from(SIG_DOMAIN_NONCE), sk_base, *m]);
    let k = JjScalar::from_be_bytes_mod_order(&k_base.into_bigint().to_bytes_be());
    // Guard the degenerate k == 0 (would make R the identity); fold once more.
    if k.is_zero() {
        let k2_base: Fr = poseidon2::hash(&[Fr::from(SIG_DOMAIN_NONCE), k_base, *m]);
        JjScalar::from_be_bytes_mod_order(&k2_base.into_bigint().to_bytes_be())
    } else {
        k
    }
}

/// Sign `m` with `sk` using a DETERMINISTIC nonce derived from `(sk, m)` (no
/// entropy source, no caller seed — see [`derive_nonce`]). This is the
/// issuance-side path used by [`SecretKey::sign_commitment`]; a relying party
/// only ever calls [`verify`]. Equivalent in shape to [`sign`] but with the
/// nonce pinned, so it is replay-stable and seed-reuse-proof.
// [OPUS-4.8] audit #3 codex #4.
pub fn sign_deterministic(sk: &SecretKey, m: &Fr) -> Signature {
    let g = EdwardsProjective::generator();
    let k = derive_nonce(sk, m);
    let r_pt = (g * k).into_affine();
    let pk = sk.public_key().0;
    let e = challenge(&r_pt, &pk, m);
    let s = k + e * sk.0;
    Signature { r: r_pt, s }
}

/// The challenge `e = Poseidon2([DOMAIN, R.x, R.y, pk.x, pk.y, m])`, reduced
/// into the curve's scalar field. Computed over the *base* field (= `Fr`) so it
/// is in-circuit-recomputable, then reduced via its big-endian bytes (the
/// standard Schnorr "hash-to-scalar" step). Both signer and verifier call this,
/// so a drift in the challenge derivation can never make a forged signature
/// verify (single source of truth, mirroring the `verify_*_relation` discipline
/// in the noir-optimisation skill).
fn challenge(r: &Affine<EdwardsConfig>, pk: &Affine<EdwardsConfig>, m: &Fr) -> JjScalar {
    // [OPUS-4.8] codex #2 (false positive, confirmed): in ark-ec 0.5
    // `AffineRepr::xy()` returns `Option<(Self::BaseField, Self::BaseField)>` —
    // OWNED `BaseField` values, not references (see ark-ec-0.5.0
    // models/twisted_edwards/affine.rs:164). `BaseField == Fr` here, so the
    // `unwrap_or` with owned `Fr` zeros is type-correct and compiles. (The
    // identity is the only point with `xy() == None`; it is rejected in `verify`
    // and `public_key_from_hex` before reaching here — codex #3.)
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

/// Sign a message field element `m` with `sk`, drawing the nonce `k` from `rng`.
///
/// # Test-only (codex job 2216 MEDIUM)
/// This random-nonce path is `#[cfg(test)]` and `pub(crate)`: no public API takes
/// a caller-supplied RNG/seed. A reused/cloned RNG state would reuse a Schnorr
/// nonce across two distinct messages, yielding two signatures with the same
/// `R = k·G`, from which `sk = (s1 - s2)/(e1 - e2)` is trivially recovered. All
/// issuance now goes through the deterministic `(sk, m)`-nonce path
/// ([`sign_deterministic`] / [`SecretKey::sign_commitment`]); this remains only
/// to exercise [`verify`] against an independently-nonce'd honest signature in
/// this crate's own tests.
// [OPUS-4.8] codex 2216 MEDIUM: random-nonce signer is test-only; no public caller-RNG API.
#[cfg(test)]
pub(crate) fn sign<R: ark_std::rand::RngCore + ark_std::rand::CryptoRng>(
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
    // [OPUS-4.8] codex #3: reject the IDENTITY public key. The identity point is
    // on-curve and in-subgroup, but `e·pk = 0` for it, so the verification
    // equation collapses to `s·G == R`; a forger picks any `s`, sets `R = s·G`,
    // and a signature verifies for the identity key under ANY message. Rejecting
    // `pk.0.is_zero()` here (and in `public_key_from_hex`, fail-closed at parse
    // time) closes that universal forgery.
    if pk.0.is_zero() {
        return false;
    }
    // Reject the off-curve / non-prime-order points defensively: `R` and `pk`
    // must be on the curve and in the prime-order subgroup, else a small-subgroup
    // point could let a forger pass the equation.
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
    let pt = Affine::<EdwardsConfig>::deserialize_compressed(&bytes[..]).ok()?;
    // [OPUS-4.8] codex #3: reject the identity point at parse time (fail-closed).
    // A signature is universally forgeable for the identity key (e·pk = 0), so an
    // identity key must never enter a key-set K or an attestation. `verify` also
    // rejects it defensively, but rejecting here keeps it out of K entirely.
    if pt.is_zero() {
        return None;
    }
    Some(PublicKey(pt))
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

    // --- audit #3 codex #3: identity-key forgery rejection ----------------

    /// [OPUS-4.8] codex #3: a signature is universally forgeable for the IDENTITY
    /// public key (`e·pk = 0` ⇒ the equation is `s·G == R`, satisfied by any `s`
    /// with `R = s·G`). Such a forgery — valid for ANY message — MUST be rejected
    /// by `verify`. Before the fix this returned `true`.
    #[test]
    fn identity_key_forgery_rejected() {
        // The identity / neutral point of Baby-JubJub.
        let id_pk = PublicKey(Affine::<EdwardsConfig>::zero());
        assert!(id_pk.0.is_zero(), "constructed the identity point");
        // Forge: pick any s, set R = s·G. Then s·G == R + e·0 holds for all m.
        let s = JjScalar::from(123456u64);
        let r = (EdwardsProjective::generator() * s).into_affine();
        let forged = Signature { r, s };
        let m = commitment_message(&Fr::from(42u64));
        assert!(
            !verify(&id_pk, &m, &forged),
            "identity-key signature must be rejected (universal forgery)"
        );
        // And it must never be admissible from hex either (fail-closed at parse).
        let id_hex = {
            let mut b = Vec::new();
            Affine::<EdwardsConfig>::zero()
                .serialize_compressed(&mut b)
                .unwrap();
            to_hex(&b)
        };
        assert!(
            public_key_from_hex(&id_hex).is_none(),
            "identity key must not parse into a usable PublicKey"
        );
    }

    // --- audit #3 codex #4: deterministic nonce (no caller seed) ----------

    /// [OPUS-4.8] codex #4: `sign_commitment` derives the nonce from `(sk, m)`,
    /// so it is deterministic (replay-stable) and seed-reuse is impossible by
    /// construction. The signature verifies, and re-signing the same commitment
    /// yields byte-identical output.
    #[test]
    fn deterministic_sign_commitment_round_trip_and_stable() {
        let sk = SecretKey::from_seed(11);
        let pk = sk.public_key();
        let c = Fr::from(0xc0ffeeu64);
        let h1 = sk.sign_commitment(&c);
        let h2 = sk.sign_commitment(&c);
        assert_eq!(h1, h2, "deterministic signing must be replay-stable");
        let sig = signature_from_hex(&h1).expect("round-trips");
        assert!(
            verify(&pk, &commitment_message(&c), &sig),
            "deterministic signature must verify"
        );
    }

    /// [OPUS-4.8] codex #4: distinct messages get DISTINCT nonces `R` — the
    /// property that makes seed-reuse key-extraction impossible (the old seeded
    /// API let a caller force `R1 == R2` across messages, leaking `sk`).
    #[test]
    fn deterministic_nonce_differs_per_message() {
        let sk = SecretKey::from_seed(12);
        let r1 = sign_deterministic(&sk, &commitment_message(&Fr::from(1u64))).r;
        let r2 = sign_deterministic(&sk, &commitment_message(&Fr::from(2u64))).r;
        assert_ne!(r1, r2, "distinct messages must use distinct nonces R");
    }
}
