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

    /// Sign a per-graph commitment AND its salt (audit #9), returning the
    /// signature hex. The signed message is [`commitment_message_with_salt`], so
    /// the salt under which the graph was committed is issuer-attested. Same
    /// deterministic `(sk, m)` nonce discipline as [`Self::sign_commitment`].
    // [OPUS-4.8] audit #9: salt-bound issuance.
    pub fn sign_commitment_with_salt(&self, commitment: &Fr, salt: &Fr) -> String {
        let sig = sign_deterministic(self, &commitment_message_with_salt(commitment, salt));
        signature_to_hex(&sig)
    }

    /// Sign a per-graph commitment, its salt, AND its status-list reference
    /// digest (audit #12), returning the signature hex. The signed message is
    /// [`commitment_message_with_status`], so the credential's revocation
    /// reference (list id + index + version) is issuer-attested and cannot be
    /// omitted or forged by the prover. Same deterministic `(sk, m)` nonce
    /// discipline as [`Self::sign_commitment`].
    // [OPUS-4.8] audit #12: status-bound issuance.
    pub fn sign_commitment_with_status(
        &self,
        commitment: &Fr,
        salt: &Fr,
        status_ref: &Fr,
    ) -> String {
        let sig = sign_deterministic(
            self,
            &commitment_message_with_status(commitment, salt, status_ref),
        );
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

/// The signed message for a per-graph commitment that ALSO binds the per-graph
/// RDFC10 bnode salt (audit #9). The issuer signs `(C(G), salt_G)` under a
/// distinct domain tag, so the salt under which a graph was committed becomes
/// ISSUER-ATTESTED: a salt-reusing ingester cannot present a graph under a salt
/// the trusted issuer did not sign, and the verifier — which sees `salt_G` in
/// the attestation — can detect salt reuse across distinct commitments (the Q6
/// cross-graph bnode-correlation channel).
///
/// # Why bind the salt at all (the audit #9 fix)
/// The Q6 "bnodes from different graphs are distinct by construction" guarantee
/// rests on each graph having a globally-unique salt (so a recurring canonical
/// bnode label `c14n0` encodes to DIFFERENT field elements in different graphs).
/// The salt never enters any circuit and the bare `commitment_message` does not
/// bind it, so a salt-reusing ingester could correlate bnodes across graphs. By
/// folding `salt_G` into the signed object, the property is anchored on the
/// TRUSTED ISSUER (who draws a fresh OS-random salt per graph at mint, see
/// `encode::salt_from_bytes`) rather than on an unenforced honest-ingest
/// convention. A forger who reuses a salt cannot obtain a valid issuer signature
/// over the reused `(C(G), salt)` pair.
// [OPUS-4.8] audit #9: salt-bound commitment message.
const SIG_DOMAIN_COMMITMENT_SALT: u64 = 0x5a4b_5349_475f_4332; // "ZKSIG_C2"
pub fn commitment_message_with_salt(commitment: &Fr, salt: &Fr) -> Fr {
    poseidon2::hash(&[Fr::from(SIG_DOMAIN_COMMITMENT_SALT), *commitment, *salt])
}

/// A domain-separated digest of a credential's STATUS-LIST REFERENCE (audit
/// #12): the status list the credential's liveness is tracked in, the
/// credential's index into that list, and the list VERSION (a monotone
/// freshness counter, e.g. the issuer's status-list `validFrom` epoch or
/// publication sequence number). Folded into the issuer signature by
/// [`commitment_message_with_status`] so the reference cannot be omitted,
/// forged, or swapped by the prover.
///
/// `list_id` is the caller-hashed status-list IRI (a field element — the
/// verifier hashes the disclosed `manifest.revocation.status_list` IRI the same
/// way and asserts equality, so the in-the-clear IRI is bound). The index and
/// version enter as field elements directly.
// [OPUS-4.8] audit #12: status-list reference digest.
pub fn status_ref_digest(list_id: &Fr, index: u64, version: u64) -> Fr {
    const SIG_DOMAIN_STATUS_REF: u64 = 0x5a4b_5349_475f_5352; // "ZKSIG_SR"
    poseidon2::hash(&[
        Fr::from(SIG_DOMAIN_STATUS_REF),
        *list_id,
        Fr::from(index),
        Fr::from(version),
    ])
}

/// A hiding COMMITMENT to a credential's status-list index (sq-ayv): the
/// index-hiding analogue of the clear `index` in [`status_ref_digest`]. The
/// issuer signs a commitment to the index (via [`status_ref_commit_digest`]),
/// NOT the clear index, so a hidden-revocation presentation can withhold the
/// index entirely. `blinding` is a per-credential random field element (the
/// holder's secret) that makes the commitment hiding: two credentials at the
/// same index commit to different values, closing the linkability channel the
/// clear index opened.
///
/// # Cross-binding (load-bearing for soundness)
/// This commitment is recomputed IN-CIRCUIT by the `revoke_unset_d{depth}` member
/// from the SAME private `index` the Merkle bit-unset fold uses (sq-ayv) and a
/// private `blinding`, and exposed as a PUBLIC input. The verifier then requires
/// the proof's public commitment to byte-equal the ISSUER-SIGNED commitment (this
/// value, folded into the signed message). So the index proven UNSET is provably
/// the index the issuer committed to — a holder cannot obtain a signature over a
/// commitment to its REVOKED index and prove bit-unset for some OTHER index.
///
/// Domain-separated and binding under the Poseidon2 collision/hiding assumption
/// (the same hash the commitment + Merkle layers use). Issuer and verifier (and
/// the circuit) all compute it identically, so a drift cannot make a wrong
/// commitment verify (single source of truth).
// [OPUS-4.8] sq-ayv: hiding index commitment (issuer-signed in place of the clear index).
pub fn status_index_commitment(index: u64, blinding: &Fr) -> Fr {
    const SIG_DOMAIN_STATUS_IDX_COMMIT: u64 = 0x5a4b_5349_475f_4943; // "ZKSIG_IC"
    poseidon2::hash(&[
        Fr::from(SIG_DOMAIN_STATUS_IDX_COMMIT),
        Fr::from(index),
        *blinding,
    ])
}

/// A domain-separated digest of a credential's status-list reference that binds a
/// COMMITMENT to the index (sq-ayv) instead of the clear index — the
/// index-hiding analogue of [`status_ref_digest`]. The issuer folds
/// `(list_id, index_commitment, version)` so the credential's revocation
/// reference is issuer-attested WITHOUT the clear index ever entering the signed
/// object. A DISTINCT domain tag from [`status_ref_digest`] so a clear-index
/// digest can never be substituted for a committed-index one (and vice versa).
///
/// `index_commitment` is [`status_index_commitment`]`(index, blinding)`. The
/// verifier recomputes this digest from the disclosed `index_commitment` +
/// version + hashed list IRI and checks the issuer signature over it; the
/// hidden-revocation proof then cross-binds `index_commitment` to the proven-unset
/// index in-circuit (see [`status_index_commitment`]). So the index is disclosed
/// in NEITHER the signed object NOR any clear field, yet revocation is still
/// checked against the authoritative root.
// [OPUS-4.8] sq-ayv: committed-index status-reference digest.
pub fn status_ref_commit_digest(list_id: &Fr, index_commitment: &Fr, version: u64) -> Fr {
    const SIG_DOMAIN_STATUS_REF_COMMIT: u64 = 0x5a4b_5349_475f_5343; // "ZKSIG_SC"
    poseidon2::hash(&[
        Fr::from(SIG_DOMAIN_STATUS_REF_COMMIT),
        *list_id,
        *index_commitment,
        Fr::from(version),
    ])
}

/// Hash a status-list IRI string to a field element (domain-separated), so the
/// in-the-clear `manifest.revocation.status_list` IRI can be bound under the
/// issuer signature. Issuer and verifier both call this, so a drift cannot make
/// a wrong IRI verify (single source of truth).
// [OPUS-4.8] audit #12.
pub fn status_list_id_to_field(list_iri: &str) -> Fr {
    const SIG_DOMAIN_STATUS_LIST_IRI: u64 = 0x5a4b_5349_475f_4c49; // "ZKSIG_LI"
    // Fold the UTF-8 bytes 31 at a time (each chunk fits a BN254 field element
    // headroom-safe) through the sponge, prefixed by the domain tag and the
    // byte length so two IRIs differing only by trailing padding cannot collide.
    let bytes = list_iri.as_bytes();
    let mut acc: Vec<Fr> = Vec::with_capacity(2 + bytes.len() / 31 + 1);
    acc.push(Fr::from(SIG_DOMAIN_STATUS_LIST_IRI));
    acc.push(Fr::from(bytes.len() as u64));
    for chunk in bytes.chunks(31) {
        acc.push(Fr::from_be_bytes_mod_order(chunk));
    }
    poseidon2::hash(&acc)
}

/// The signed message for a per-graph commitment that binds BOTH the per-graph
/// RDFC10 salt (audit #9) AND the credential's status-list reference (audit
/// #12). The issuer signs `(C(G), salt_G, status_ref_digest)` under a distinct
/// domain tag, so the status reference (which list, which index, which version)
/// becomes ISSUER-ATTESTED.
///
/// # Why bind the status reference at all (the audit #12 fix)
/// Before this, revocation was entirely unchecked: a prover could OMIT
/// `manifest.revocation` (or point it at an empty/attacker list) and a
/// revoked/suspended credential would still verify, because nothing tied the
/// status reference to the credential. By folding the status-reference digest
/// into the SAME issuer signature that already binds `C(G)` and the salt, the
/// reference is mandatory and unforgeable: an omitted/forged/swapped reference
/// yields a different signed message and so has no valid issuer signature
/// (fail-closed — exactly the optional-field bypass that bit #3/#8/#9/#4). The
/// verifier then (a) recomputes this message from the disclosed reference and
/// checks the signature, (b) checks the disclosed status-list snapshot's
/// bit[index] is UNSET, and (c) checks the snapshot version is within a
/// freshness window. See `sparq_zk_compose::verifier`.
// [OPUS-4.8] audit #12: status-bound commitment message.
const SIG_DOMAIN_COMMITMENT_STATUS: u64 = 0x5a4b_5349_475f_4333; // "ZKSIG_C3"
pub fn commitment_message_with_status(commitment: &Fr, salt: &Fr, status_ref: &Fr) -> Fr {
    poseidon2::hash(&[
        Fr::from(SIG_DOMAIN_COMMITMENT_STATUS),
        *commitment,
        *salt,
        *status_ref,
    ])
}

/// The signed message for a HOLDER proof-of-possession (sq-cwq): a
/// domain-separated binding of the verifier's freshness `challenge` (its nonce,
/// the same field element that is public-input field 0 of every circuit member).
/// The HOLDER signs THIS value with its holder secret key; the verifier checks the
/// signature under the disclosed holder public key, proving the presenter
/// POSSESSES the holder secret AND did so freshly over the verifier's nonce (so a
/// captured manifest cannot be replayed by a party that does not hold the key —
/// the PoP is bound to the relying party's fresh challenge).
///
/// # Scope (honest)
/// This binds possession of the holder KEY to the verifier's challenge. It does
/// NOT, on its own, bind that key to a SPECIFIC credential — that requires an
/// issuer-attested holder binding (the issuer signing the holder key into the
/// credential), which is a documented deferral (see
/// `sparq_zk_compose::verifier::bind_holder_pop`). The relying party anchors the
/// holder key in an EXTERNAL holder registry (mirroring the issuer key-set `K`),
/// so an absent/untrusted/forged PoP fails closed.
// [OPUS-4.8] sq-cwq: holder proof-of-possession message (challenge-bound).
pub fn holder_pop_message(challenge: &Fr) -> Fr {
    const SIG_DOMAIN_HOLDER_POP: u64 = 0x5a4b_5349_475f_4850; // "ZKSIG_HP"
    poseidon2::hash(&[Fr::from(SIG_DOMAIN_HOLDER_POP), *challenge])
}

impl SecretKey {
    /// Produce a holder proof-of-possession over the verifier's `challenge`
    /// (sq-cwq), returning the signature hex (`compressed(R) ‖ s`). The signed
    /// message is [`holder_pop_message`], matching the verifier-side
    /// `bind_holder_pop` check. Same deterministic `(sk, m)` nonce discipline as
    /// [`Self::sign_commitment`] (no signing-time entropy, seed-reuse-proof).
    // [OPUS-4.8] sq-cwq: holder-side PoP signing.
    pub fn sign_holder_pop(&self, challenge: &Fr) -> String {
        let sig = sign_deterministic(self, &holder_pop_message(challenge));
        signature_to_hex(&sig)
    }
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

// --- in-circuit witness bridge (sq-z9l) -----------------------------------
// [OPUS-4.8] sq-z9l: the witness an in-circuit Schnorr verifier needs. The
// in-circuit gadget (`zk/compose/compose_core/src/issuer.nr`) verifies
// `s*G == R + e*pk` over Baby-JubJub with point coordinates in the BASE field
// (= Noir `Field` = `crate::field::Fr`) and the SCALAR `e` reduced mod the
// curve's scalar order `L`. This bridge produces exactly those field-element
// witnesses from a `(PublicKey, Signature, message)` triple so the host can
// render the circuit's `Prover.toml`.

impl PublicKey {
    /// The issuer key's affine coordinates as base-field [`Fr`] elements (= Noir
    /// `Field`). `None` for the identity (no affine coordinates) — which is never
    /// a usable key (rejected at parse/verify, see [`verify`]). The in-circuit
    /// gadget and the key-set Merkle leaf both work over these coordinates.
    // [OPUS-4.8] sq-z9l.
    pub fn coords(&self) -> Option<(Fr, Fr)> {
        self.0.xy()
    }
}

/// The key-set Merkle leaf for an issuer public key: `Poseidon2([pk.x, pk.y])`.
/// Mirrors `issuer.nr`'s `key_leaf` (`h2(pk.x, pk.y)`) bit-for-bit. `None` for
/// the identity key (no coordinates). The relying party commits its authoritative
/// KeySet K with this leaf, and the in-circuit membership proof folds it to the
/// public root.
// [OPUS-4.8] sq-z9l: key-set Merkle leaf (host mirror of issuer.nr::key_leaf).
pub fn key_set_leaf(pk: &PublicKey) -> Option<Fr> {
    let (x, y) = pk.coords()?;
    Some(poseidon2::hash(&[x, y]))
}

/// The Baby-JubJub scalar-field order `L` as a base-field element (`< q_base`,
/// so the embedding is injective). Mirrors `issuer.nr`'s `BJJ_L`.
fn scalar_order_in_base() -> Fr {
    // `JjScalar::MODULUS` big-endian, reduced into the base field. `L < q_base`,
    // so this is the exact value, not a reduction.
    Fr::from_be_bytes_mod_order(&JjScalar::MODULUS.to_bytes_be())
}

/// Map a Baby-JubJub scalar into the base field via its canonical big-endian
/// bytes. `L < q_base`, so a canonical scalar (`< L`) embeds injectively — the
/// resulting `Fr` is the same integer, suitable as a circuit `Field` witness.
fn jj_scalar_to_base(s: &JjScalar) -> Fr {
    Fr::from_be_bytes_mod_order(&s.into_bigint().to_bytes_be())
}

/// The in-circuit witness for one hidden issuer attestation: the affine point
/// coordinates of the issuer key and the signature's `R`, the signature scalar
/// `s`, and the challenge reduction `(e, e_k)` such that
/// `e_base = e + e_k * L` with `e < L` and `e_k < 8` (the soundness binding the
/// circuit re-checks). All as base-field [`Fr`] elements (= Noir `Field`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InCircuitSchnorrWitness {
    /// Issuer public-key affine coordinates.
    pub pk_x: Fr,
    pub pk_y: Fr,
    /// Signature nonce-commitment `R` affine coordinates.
    pub r_x: Fr,
    pub r_y: Fr,
    /// Signature scalar `s` (canonical, `< L`), embedded in the base field.
    pub s: Fr,
    /// Reduced challenge scalar `e = e_base mod L` (`< L`).
    pub e: Fr,
    /// Reduction quotient `k` such that `e_base = e + k * L` (`< 8`).
    pub e_k: Fr,
}

/// Build the [`InCircuitSchnorrWitness`] for `(pk, sig)` over the signed message
/// field element `m` (e.g. [`commitment_message_with_status`]'s output — the
/// SAME `Fr` the issuer signed and the circuit recomputes the challenge over).
///
/// Returns `None` if the inputs are degenerate (an identity key or a point with
/// no affine coordinates), which the in-circuit gadget would reject anyway. This
/// is an ISSUANCE/PROVER-SIDE helper: it does not itself verify the signature
/// (the circuit does), it only lays out the witness fields. Callers that want a
/// sanity check should also call [`verify`].
pub fn in_circuit_witness(pk: &PublicKey, m: &Fr, sig: &Signature) -> Option<InCircuitSchnorrWitness> {
    if pk.0.is_zero() {
        return None;
    }
    let (pk_x, pk_y) = pk.0.xy()?;
    let (r_x, r_y) = sig.r.xy()?;
    // The base-field Poseidon2 challenge digest, BEFORE the scalar-field
    // reduction (the in-circuit `e_base`). Recompute it from the SAME inputs
    // `challenge` uses, but keep the base-field value (challenge() returns the
    // reduced JjScalar; here we need both halves of the reduction).
    let e_base: Fr = poseidon2::hash(&[
        Fr::from(SIG_DOMAIN_COMMITMENT),
        r_x,
        r_y,
        pk_x,
        pk_y,
        *m,
    ]);
    // Reduce e_base into the scalar field, then re-embed the reduced value and
    // derive the quotient k = (e_base - e) / L (exact in the integers).
    let e_scalar = JjScalar::from_be_bytes_mod_order(&e_base.into_bigint().to_bytes_be());
    let e = jj_scalar_to_base(&e_scalar);
    let l = scalar_order_in_base();
    // e_k = (e_base - e) / L. e_base, e, L are all canonical base-field values
    // representing the true integers (e_base, e < q_base; e < L; e_base = e + kL
    // with k < 8), so the base-field subtraction/division gives the exact small
    // integer quotient (no wraparound: e_base >= e since e = e_base mod L).
    let e_k = (e_base - e) / l;
    Some(InCircuitSchnorrWitness {
        pk_x,
        pk_y,
        r_x,
        r_y,
        s: jj_scalar_to_base(&sig.s),
        e,
        e_k,
    })
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
    // [OPUS-4.8] sq-hbg7: stable-1.96 clippy `manual_is_multiple_of`.
    if !s.len().is_multiple_of(2) {
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

    // --- audit #12: status-bound commitment message ----------------------

    /// [OPUS-4.8] audit #12: a status-bound signature verifies over the SAME
    /// `(C(G), salt, status_ref)` and is REJECTED if any of the three differ —
    /// the property that makes an omitted/forged/swapped status reference (a
    /// different version, index, or list id) unverifiable.
    #[test]
    fn status_bound_signature_binds_the_reference() {
        let sk = SecretKey::from_seed(21);
        let pk = sk.public_key();
        let c = Fr::from(0xc0ffeeu64);
        let salt = Fr::from(0x5a17u64);
        let list_id = status_list_id_to_field("https://dmv.example/status/3");
        let sref = status_ref_digest(&list_id, 94, 7);
        let hex = sk.sign_commitment_with_status(&c, &salt, &sref);
        let sig = signature_from_hex(&hex).expect("round-trips");
        let msg = commitment_message_with_status(&c, &salt, &sref);
        assert!(verify(&pk, &msg, &sig), "honest status-bound signature verifies");

        // A DIFFERENT version (freshness rollover) ⇒ different message ⇒ fails.
        let sref_v8 = status_ref_digest(&list_id, 94, 8);
        let msg_v8 = commitment_message_with_status(&c, &salt, &sref_v8);
        assert!(!verify(&pk, &msg_v8, &sig), "wrong version must not verify");
        // A DIFFERENT index ⇒ fails (cannot point at another credential's bit).
        let sref_idx = status_ref_digest(&list_id, 95, 7);
        let msg_idx = commitment_message_with_status(&c, &salt, &sref_idx);
        assert!(!verify(&pk, &msg_idx, &sig), "wrong index must not verify");
        // A DIFFERENT list id ⇒ fails (cannot swap to an attacker-controlled list).
        let other_list = status_list_id_to_field("https://attacker.example/empty");
        let sref_other = status_ref_digest(&other_list, 94, 7);
        let msg_other = commitment_message_with_status(&c, &salt, &sref_other);
        assert!(!verify(&pk, &msg_other, &sig), "wrong list id must not verify");
        // The bare salt-only message (the prover OMITTING the status ref) ⇒
        // fails: a status-bound credential cannot be presented as salt-only.
        let salt_only = commitment_message_with_salt(&c, &salt);
        assert!(
            !verify(&pk, &salt_only, &sig),
            "salt-only (status-omitted) message must not verify against a status-bound sig"
        );
    }

    /// [OPUS-4.8] sq-ayv: `status_index_commitment` is bit-identical to the
    /// in-circuit `h3(SIG_DOMAIN_STATUS_IDX_COMMIT, index, blinding)` the
    /// `revoke_unset_d{depth}` member recomputes. The cross-binding (the index
    /// proven unset == the index the issuer committed to) rests on both computing
    /// the SAME value, so this pins the cross-vector against the nargo output
    /// (compose_core/tests.nr `revoke_idx_commit_cross_vector`).
    #[test]
    fn status_index_commitment_matches_circuit_cross_vector() {
        let got = status_index_commitment(0, &Fr::from(0xb1u64));
        assert_eq!(
            crate::field::field_to_hex(&got),
            "0x204fe7eb53d68b4f2c7b981619ed4fbd5f3b99879e0f387761bf489a21456118",
            "host index commitment must equal the in-circuit h3(DOMAIN, 0, 0xb1)"
        );
    }

    /// [OPUS-4.8] sq-ayv: a COMMITTED-index status-bound signature binds the index
    /// COMMITMENT (not the clear index), is hiding (a different blinding => a
    /// different commitment for the same index), and the committed-digest domain
    /// is distinct from the clear-index digest (no substitution).
    #[test]
    fn committed_index_status_signature_binds_the_commitment() {
        let sk = SecretKey::from_seed(31);
        let pk = sk.public_key();
        let c = Fr::from(0xc0ffeeu64);
        let salt = Fr::from(0x5a17u64);
        let list_id = status_list_id_to_field("https://dmv.example/status/3");
        let blinding = Fr::from(0xbeefu64);
        let idx_commit = status_index_commitment(94, &blinding);
        let sref = status_ref_commit_digest(&list_id, &idx_commit, 7);
        let hex = sk.sign_commitment_with_status(&c, &salt, &sref);
        let sig = signature_from_hex(&hex).expect("round-trips");
        let msg = commitment_message_with_status(&c, &salt, &sref);
        assert!(verify(&pk, &msg, &sig), "honest committed-index signature verifies");

        // Hiding: the SAME index under a DIFFERENT blinding commits differently,
        // so two presentations of the same credential are not linkable by the
        // commitment (the clear-index linkability channel is closed).
        let idx_commit2 = status_index_commitment(94, &Fr::from(0x1234u64));
        assert_ne!(idx_commit, idx_commit2, "different blinding => different commitment");

        // A DIFFERENT committed index => different digest => signature fails.
        let other_commit = status_index_commitment(95, &blinding);
        let sref_other = status_ref_commit_digest(&list_id, &other_commit, 7);
        let msg_other = commitment_message_with_status(&c, &salt, &sref_other);
        assert!(!verify(&pk, &msg_other, &sig), "a different index commitment must not verify");

        // The CLEAR-index digest (audit #12) is domain-separated from the
        // committed-index one: even with a numerically-equal field input it
        // cannot be substituted (distinct domain tags).
        let clear = status_ref_digest(&list_id, 94, 7);
        let committed = status_ref_commit_digest(&list_id, &idx_commit, 7);
        assert_ne!(clear, committed, "clear-index and committed-index digests are domain-separated");
    }

    /// [OPUS-4.8] audit #12: the list-id hash is collision-resistant on distinct
    /// IRIs and length-separated (a prefix cannot collide with a longer IRI).
    #[test]
    fn status_list_id_is_distinct_per_iri() {
        let a = status_list_id_to_field("https://dmv.example/status/3");
        let b = status_list_id_to_field("https://dmv.example/status/4");
        let c = status_list_id_to_field("https://dmv.example/status/3x");
        assert_ne!(a, b, "distinct lists hash distinctly");
        assert_ne!(a, c, "a prefix must not collide with a longer IRI");
    }

    // --- sq-z9l: in-circuit witness bridge --------------------------------

    use crate::field::field_to_hex;

    /// [OPUS-4.8] sq-z9l: the in-circuit witness reproduces the verify equation
    /// AND the challenge-reduction binding the circuit re-checks. This is the
    /// host/circuit contract: `s*G == R + e*pk` (in base-field coordinates) and
    /// `e_base == e + e_k*L` with `e < L`, `e_k < 8`.
    #[test]
    fn in_circuit_witness_is_internally_consistent() {
        let sk = SecretKey::from_seed(99);
        let pk = sk.public_key();
        let c = Fr::from(0xdecafu64);
        let m = commitment_message(&c);
        let sig = sign_deterministic(&sk, &m);
        assert!(verify(&pk, &m, &sig), "honest sig verifies (control)");

        let w = in_circuit_witness(&pk, &m, &sig).expect("witness");
        // The reduction binding the circuit asserts: e_base == e + e_k*L.
        let e_base: Fr = poseidon2::hash(&[
            Fr::from(SIG_DOMAIN_COMMITMENT),
            w.r_x,
            w.r_y,
            w.pk_x,
            w.pk_y,
            m,
        ]);
        let l = scalar_order_in_base();
        assert_eq!(w.e + w.e_k * l, e_base, "e_base == e + e_k*L");
        // e < L and e_k < 8 (3-bit).
        assert!(w.e < l, "e < L");
        assert!(w.e_k < Fr::from(8u64), "e_k < 8");

        // The witnessed e equals sig::challenge's reduced scalar, re-embedded.
        let e_ref = jj_scalar_to_base(&challenge(&sig.r, &pk.0, &m));
        assert_eq!(w.e, e_ref, "witnessed e == reduced challenge");

        // s*G == R + e*pk recomputed in the curve group from the witness scalars.
        // (Sanity that the base-field witness round-trips back to a valid eqn.)
        let g = EdwardsProjective::generator();
        let lhs = (g * sig.s).into_affine();
        let rhs = (sig.r.into_group() + pk.0 * challenge(&sig.r, &pk.0, &m)).into_affine();
        assert_eq!(lhs, rhs, "verify equation holds in-group (control)");
        assert_eq!(field_to_hex(&w.pk_x).len(), 66, "pk_x is a field hex");
    }

    /// [OPUS-4.8] sq-z9l: the witness bridge declines the identity key (the
    /// circuit rejects it; the bridge returns None rather than emitting a
    /// degenerate witness).
    #[test]
    fn in_circuit_witness_rejects_identity_key() {
        let id_pk = PublicKey(Affine::<EdwardsConfig>::zero());
        let m = commitment_message(&Fr::from(1u64));
        let s = JjScalar::from(7u64);
        let r = (EdwardsProjective::generator() * s).into_affine();
        let sig = Signature { r, s };
        assert!(in_circuit_witness(&id_pk, &m, &sig).is_none());
    }
}
