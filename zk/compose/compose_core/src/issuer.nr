// [OPUS-4.8] sq-z9l: in-circuit issuer-signature verification + hidden-key set
// membership. Re-review when Fable returns.
//! In-circuit **Schnorr-over-Baby-JubJub** signature verification + **hidden-key
//! set membership** (plan: the privacy upgrade the `sparq_zk::sig` module docs
//! name -- "prove a commitment was signed by SOME key in K, without revealing
//! which").
//!
//! # The problem this closes
//! Today `sparq_zk_compose::verifier::bind_issuer_attestations` verifies each
//! commitment's issuer signature IN THE CLEAR against a disclosed key set K -- so
//! the verifier learns WHICH issuer (which authority) vouched for the holder, a
//! deanonymising linkability channel. This gadget proves, in zero knowledge:
//!
//!   "there exists an issuer public key `pk` that (1) is a member of the set K
//!    committed as the PUBLIC Poseidon2 Merkle root `key_set_root`, and (2)
//!    produced a valid Schnorr signature `(R, s)` over the PUBLIC commitment
//!    message `m`"
//!
//! revealing NEITHER `pk`, the signature, NOR the member index.
//!
//! # The scheme it mirrors (`crates/sparq-zk/src/sig.rs`, bit-for-bit)
//! Schnorr over Baby-JubJub (the BN254 *embedded twisted-Edwards* curve), with a
//! Poseidon2 challenge. The verify equation, exactly as `sig::verify`:
//!
//!   `e = Poseidon2([SIG_DOMAIN_COMMITMENT, R.x, R.y, pk.x, pk.y, m])  (mod L)`
//!   `s * G == R + e * pk`                                              (on curve)
//!
//! where `G` is the curve generator, `L` the curve's scalar-field order, and the
//! point coordinates live in Noir's `Field` (= Baby-JubJub's BASE field = BN254's
//! SCALAR field). That coordinate coincidence is WHY this curve was chosen: the
//! challenge hash is a Poseidon2 sponge over field coordinates, so the in-circuit
//! recomputation of `e` is the SAME `Poseidon2::hash` the scan-commitment layer
//! uses -- no foreign-field emulation for the hash.
//!
//! # Why NOT the `embedded_curve_*` black boxes (load-bearing correctness note)
//! Noir's `std` `embedded_curve_add` / `multi_scalar_mul` black boxes operate on
//! **Grumpkin** (the curve whose SCALAR field is BN254's BASE field), NOT
//! Baby-JubJub. `sig.rs` signs over Baby-JubJub (whose BASE field is BN254's
//! scalar field). They are DIFFERENT curves. Feeding Baby-JubJub coordinates to
//! the Grumpkin black box would verify a signature on the wrong curve -- a silent
//! soundness break. So the twisted-Edwards point arithmetic here is implemented
//! in explicit `Field` constraints, with the Baby-JubJub parameters (a = 1,
//! d = D below) embedded as constants. (Cost: see the module footer.)
//!
//! # Scalar reduction -- the soundness-critical binding (cf. noir-optimisation S3.4)
//! `sig::challenge` reduces the base-field Poseidon2 digest `e_base` (a `Field`,
//! so `e_base < q_base`) into the scalar field by `from_be_bytes_mod_order`, i.e.
//! `e = e_base mod L`. In-circuit we take `e` and a quotient `k` as PRIVATE
//! witnesses and BIND them: `e_base == e + k*L`, with `e < L` and `k < 8` (since
//! `q_base / L < 8`, so `e_base = e + k*L` has `k in {0..7}`). Without this
//! binding a prover could feed an `e' != e_base mod L` that satisfies the curve
//! equation for a forged `(R, s)` -- the classic "witnessed reduction not bound
//! to its preimage" hole. The scalar `s` is range-checked `s < L` (it is a signer
//! output, canonically `< L`); no reduction binding is needed for it.
//!
//! [OPUS-4.8] sq-ru0yx (internal re-audit finding M-1): `e_base == e + k*L` with
//! `e < L` and `k < 8` is a FIELD identity (mod `q_base`) and does NOT, on its own,
//! UNIQUELY pin `e`. Because `7*L < q_base < 8*L`, for `k == 7` it also admits the
//! wrap alternative `e' = e_base + q_base - 7*L` (which has `e' < L`, yet
//! `e' != e_base mod L`): `e' + 7*L` overflows past `q_base` and lands back on
//! `e_base`. `reduction_range_bind` therefore adds a NO-WRAP bound forbidding the
//! unreachable top window `e in [q_base - 7*L, L)` when `k == 7`. The honest
//! reduction always lands below that window, so this never rejects a valid witness.

use crate::hashes::h2;
use poseidon::poseidon2::Poseidon2;

// --- Baby-JubJub twisted-Edwards parameters (ark-ed-on-bn254 0.5.0) -----------
// Twisted Edwards: a*x^2 + y^2 = 1 + d*x^2*y^2, with a = 1.
// COEFF_D = 168696/168700 mod q (ark-ed-on-bn254 curves/mod.rs COEFF_D).
global BJJ_D: Field =
    9706598848417545097372247223557719406784115219466060233080913168975159366771;
// Generator G = (GENERATOR_X, GENERATOR_Y) (ark-ed-on-bn254 curves/mod.rs).
// `pub` so the holder-PoK member (`holder.nr`) reuses the SAME generator verbatim
// (design S3.4: holder_pok reuses issuer.nr's gadgets, no reimplementation).
pub global BJJ_GX: Field =
    19698561148652590122159747500897617769866003486955115824547446575314762165298;
pub global BJJ_GY: Field =
    19298250018296453272277890825869354524455968081175474282777126169995084727839;
// Scalar-field order L (ark-ed-on-bn254 fields/fr.rs modulus). ~2^251.
global BJJ_L: Field =
    2736030358979909402780800718157159386076813972158567259200215660948447373041;
// Number of bits to decompose a scalar into for double-and-add. `L < 2^251`, so
// 251 bits cover every canonical scalar `s, e in [0, L)`.
global SCALAR_BITS: u32 = 251;
// Reduction-quotient bound: q_base / L < 8, so `e_base = e + k*L` has `k in 0..7`.
global REDUCTION_K_BITS: u32 = 3;
// [OPUS-4.8] sq-ru0yx (internal re-audit finding M-1): the LARGEST honest
// reduction quotient is `k_max = floor((q_base - 1) / L) = 7` (`7*L < q_base <
// 8*L`). When `e_k == 7` the honest reduced scalar `e = e_base - 7*L` lies in
// `[0, q_base - 7*L)`; the window `[q_base - 7*L, L)` (~2^126 wide) is reached by
// NO honest `e_base < q_base`. The reduction identity `e + e_k*L == e_base` is a
// FIELD equation (mod q_base), so without forbidding that window a prover could
// witness the wrap alternative `(e', e_k') = (e_base + q_base - 7*L, 7)` -- which
// also satisfies the field identity, has `e' < L`, yet `e' != e_base mod L`. The
// no-wrap bound below closes it. `REDUCTION_TOP_BOUND = q_base - 7*L`.
global REDUCTION_K_MAX: u32 = 7;
global REDUCTION_TOP_BOUND: Field =
    2736030358979909402780800718157159386010666595306063529296694559936676884330;

// Domain separator for the commitment message challenge -- MUST equal
// `sig::SIG_DOMAIN_COMMITMENT` ("ZKSIG_C1" = 0x5a4b5349475f4331).
global SIG_DOMAIN_COMMITMENT: Field = 0x5a4b5349475f4331;

/// An affine twisted-Edwards point. The neutral element is `(0, 1)`.
pub struct Point {
    pub x: Field,
    pub y: Field,
}

/// Twisted-Edwards COMPLETE addition with a = 1 (the unified add/double law that
/// holds for ALL inputs on a curve with a square and d non-square -- Baby-JubJub
/// satisfies this, so no special-casing of doubling or the neutral is needed):
///   x3 = (x1*y2 + y1*x2) / (1 + d*x1*x2*y1*y2)
///   y3 = (y1*y2 - a*x1*x2) / (1 - d*x1*x2*y1*y2)         (a = 1)
/// The denominators are nonzero for all valid curve points (completeness), so the
/// field inversions are well-defined. (We do not re-derive completeness here; it
/// is the standard Hisil-Wong-Carter-Dawson result for a=1 Edwards curves with
/// d a non-square, which Baby-JubJub's d is.)
fn point_add(p: Point, q: Point) -> Point {
    let x1y2 = p.x * q.y;
    let y1x2 = p.y * q.x;
    let y1y2 = p.y * q.y;
    let x1x2 = p.x * q.x;
    let dxxyy = BJJ_D * x1x2 * y1y2;
    let x3 = (x1y2 + y1x2) / (1 + dxxyy);
    let y3 = (y1y2 - x1x2) / (1 - dxxyy);
    Point { x: x3, y: y3 }
}

/// Variable-base scalar multiplication `scalar * base` via double-and-add over
/// the LITTLE-ENDIAN bits of `scalar`. `scalar` MUST already be range-bound to
/// `< 2^SCALAR_BITS` by the caller (`to_le_bits::<SCALAR_BITS>` does this -- it
/// constrains every bit beyond the width to zero). Starts from the neutral
/// `(0, 1)` and, processing bits MSB-first, doubles then conditionally adds the
/// base -- the standard left-to-right double-and-add.
// `pub` so the holder-PoK member reuses this scalar-mul verbatim (design S3.4).
pub fn scalar_mul(scalar: Field, base: Point) -> Point {
    // to_le_bits constrains the decomposition to be canonical AND bounds the
    // scalar to < 2^SCALAR_BITS (bits beyond the width are forced to zero).
    let bits: [bool; SCALAR_BITS] = scalar.to_le_bits();
    let mut acc = Point { x: 0, y: 1 }; // neutral element
    for i in 0..SCALAR_BITS {
        // MSB first.
        let bit = bits[SCALAR_BITS - 1 - i];
        acc = point_add(acc, acc); // double
        // Conditionally add `base` iff this bit is set. The completeness of
        // point_add means adding the neutral is the (0,1) no-op below.
        let addend = Point {
            x: base.x * (bit as Field),
            y: 1 + (base.y - 1) * (bit as Field),
        };
        acc = point_add(acc, addend);
    }
    acc
}

/// Assert a point lies on Baby-JubJub: `x^2 + y^2 == 1 + d*x^2*y^2` (a = 1). The
/// caller passes PRIVATE `R` and `pk` through here so an OFF-curve witness (a twist
/// point, or an arbitrary `(x, y)` a forger might pick to satisfy the verification
/// equation off the curve) is rejected -- mirroring `sig::verify`'s `is_on_curve`
/// guard.
///
/// [OPUS-4.8] sq-l15mi (audit H-1): this on-curve guard does NOT exclude LOW-ORDER
/// / TORSION points. Baby-JubJub has cofactor 8, so its full group contains torsion
/// points of order 2/4/8 that ARE on the curve -- `x^2 + y^2 == 1 + d*x^2*y^2` holds
/// for them, so this assert ACCEPTS them. Prime-order-subgroup membership is a
/// SEPARATE constraint (`assert_in_prime_order_subgroup`). The earlier version of
/// this comment wrongly claimed the on-curve assert rejected "small-subgroup" points.
// `pub` so the holder-PoK member reuses this on-curve guard verbatim (design S3.4).
pub fn assert_on_curve(p: Point) {
    let xx = p.x * p.x;
    let yy = p.y * p.y;
    assert(xx + yy == 1 + BJJ_D * xx * yy, "point is not on Baby-JubJub");
}

/// Assert `p` lies in the PRIME-ORDER (order-`L`) subgroup of Baby-JubJub:
/// `[L]*p == O`, where `O` is the neutral `(0, 1)`. This is the in-circuit twin of
/// the host `Affine::is_in_correct_subgroup_assuming_on_curve` (`[MODULUS]*P == 0`)
/// -- the SAME cofactor/order check, no invented crypto. The caller MUST have
/// already run `assert_on_curve(p)` (the `[L]*p` computation is only meaningful for
/// an on-curve `p`).
///
/// # Why it is soundness-critical (sq-l15mi, audit H-1)
/// Baby-JubJub has cofactor 8, so the full curve group has order `8*L` and contains
/// torsion points of order 2/4/8 that `assert_on_curve` accepts. A torsion /
/// low-order issuer public key breaks Schnorr soundness on the hidden-issuer path:
/// for an order-2 `pk`, `e*pk` depends only on the parity of the challenge `e`, so a
/// prover holding NO secret key can forge a satisfying `(R, s)` (pick `s`, set
/// `R = s*G - e*pk`) with constant probability, then bind that torsion key into the
/// public `key_set_root`. Requiring `[L]*pk == O` keeps torsion keys out.
///
/// # Correctness of `[L]*p == O`
/// `L` is prime, so `ord(p) | L` iff `ord(p) in {1, L}`: `[L]*p == O` holds exactly
/// for the neutral and the order-`L` (prime-order-subgroup) points, and for NO
/// torsion point of order 2/4/8 (each of those needs a factor from the cofactor 8,
/// not `L`, to vanish). The neutral itself passes here but is rejected separately by
/// `schnorr_verify`'s identity guard, so the two together admit only genuine
/// order-`L` keys. `L < 2^SCALAR_BITS`, so `scalar_mul(BJJ_L, p)` decomposes `L`
/// exactly.
// [OPUS-4.8] sq-l15mi: the missing prime-order subgroup / torsion-key guard on the
// hidden-issuer path. `pub` so the in-crate Noir tests can isolate it.
pub fn assert_in_prime_order_subgroup(p: Point) {
    let lp = scalar_mul(BJJ_L, p);
    assert(
        (lp.x == 0) & (lp.y == 1),
        "public key not in prime-order subgroup (torsion / low-order key rejected)",
    );
}

/// Recompute the Schnorr challenge `e = Poseidon2([DOMAIN, R.x, R.y, pk.x, pk.y,
/// m]) mod L`, BINDING the witnessed scalar `e` (and quotient `k`) to the
/// base-field digest. Mirrors `sig::challenge` exactly: the Poseidon2 sponge over
/// the SAME six inputs, then the reduction into the scalar field. Returns the
/// bound scalar `e`.
fn challenge_scalar(r: Point, pk: Point, m: Field, e: Field, e_k: Field) -> Field {
    let e_base = Poseidon2::hash([SIG_DOMAIN_COMMITMENT, r.x, r.y, pk.x, pk.y, m], 6);
    // Bind e = e_base mod L: e_base == e + e_k*L, with e < L and e_k < 8.
    // (1)+(2)+(4) range-bind (e, e_k); (3) ties them to e_base. Split out so the
    // no-wrap range binding is unit-testable without a Poseidon2 preimage (M-1).
    reduction_range_bind(e, e_k);
    // (3) the reduction identity (a FIELD equation, mod q_base).
    assert(e + e_k * BJJ_L == e_base, "challenge reduction not bound to its preimage");
    e
}

/// The range-binding half of the challenge reduction: bind `(e, e_k)` so that the
/// reduction identity `e + e_k*L == e_base` (asserted by the caller) UNIQUELY pins
/// `e` to `e_base mod L`. Asserts:
///   (1) `e_k < 8` (a 3-bit quotient; `q_base / L < 8`),
///   (2) `e < L` (canonical scalar), and
///   (4) [OPUS-4.8] sq-ru0yx M-1: the NO-WRAP bound. (1)+(2)+(3) ALONE do NOT pin
///       `e`: when `e_k == 7` the FIELD identity (mod `q_base`) also admits the wrap
///       alternative `e' = e_base + q_base - 7*L` (`e' < L`) with `e' != e_base mod
///       L`, because `e' + 7*L` wraps past `q_base` back to `e_base`. We forbid the
///       unreachable top window `e in [q_base - 7*L, L)` by binding `e` to a
///       PER-BUCKET upper bound `bound = (e_k == 7) ? (q_base - 7*L) : L` and
///       asserting `e < bound`. For `e_k < 7` the sum `e + e_k*L <= 6*L + (L-1) =
///       7*L - 1 < q_base` cannot wrap, so `bound = L` (step (2)'s guarantee); for
///       `e_k == 7` the honest `e = e_base - 7*L` lies in `[0, q_base - 7*L)`, so
///       `bound = q_base - 7*L` never rejects an honest witness but excludes the
///       wrap window. `assert_lt_to` is the witnessed-difference range bind.
// [OPUS-4.8] sq-ru0yx M-1: reduction range binding incl. the no-wrap fix. `pub` so
// the in-crate Noir tests can exercise (4) with explicit (e, e_k) vectors.
pub fn reduction_range_bind(e: Field, e_k: Field) {
    e_k.assert_max_bit_size::<REDUCTION_K_BITS>();
    assert_lt_l(e);
    let is_top = (e_k == REDUCTION_K_MAX as Field) as Field;
    let bound = BJJ_L - is_top * (BJJ_L - REDUCTION_TOP_BOUND);
    assert_lt_to(e, bound);
}

/// Assert `value < bound` for a public-or-witnessed `bound <= L < 2^SCALAR_BITS`,
/// via the canonical "witnessed difference" range bind: `value` and
/// `bound - 1 - value` both decompose to `SCALAR_BITS` bits, so both lie in
/// `[0, 2^SCALAR_BITS)` and sum to `bound - 1`, forcing `value <= bound - 1 <
/// bound`. ([`assert_lt_l`] is the `bound == BJJ_L` specialisation.) The caller
/// MUST pass `bound <= L` (true for the two reduction buckets `L` and
/// `q_base - 7*L`), so `bound - 1 < 2^SCALAR_BITS` and the complement cannot wrap.
// [OPUS-4.8] sq-ru0yx M-1: per-bucket upper-bound range bind for the no-wrap fix.
fn assert_lt_to(value: Field, bound: Field) {
    value.assert_max_bit_size::<SCALAR_BITS>();
    let complement = bound - 1 - value;
    complement.assert_max_bit_size::<SCALAR_BITS>();
}

/// Assert `value < BJJ_L` (the scalar-field order). Implemented as the
/// canonical "witnessed difference" range bind: `value` and `L - 1 - value` both
/// decompose to `SCALAR_BITS` bits, so both are in `[0, 2^SCALAR_BITS)` and their
/// sum is `L - 1`, forcing `value <= L - 1 < L`. (`L < 2^SCALAR_BITS` so the
/// complement cannot wrap.)
// `pub` so the holder-PoK member reuses this `< L` range bind verbatim (design S3.4).
pub fn assert_lt_l(value: Field) {
    // value in [0, 2^SCALAR_BITS)
    value.assert_max_bit_size::<SCALAR_BITS>();
    // (L - 1 - value) in [0, 2^SCALAR_BITS) <=> value <= L - 1.
    let complement = BJJ_L - 1 - value;
    complement.assert_max_bit_size::<SCALAR_BITS>();
}

/// In-circuit Schnorr-over-Baby-JubJub verification of `(R, s)` over message `m`
/// under PRIVATE public key `pk`, mirroring `sig::verify`'s `s*G == R + e*pk`.
/// PRIVATE: `pk`, `r` (= signature R), `s`, `e`, `e_k`. PUBLIC (caller-bound):
/// `m`. Asserts:
///   1. `pk` and `R` are on the curve (off-curve forgery guard).
///   2. `pk` and `R` are in the prime-order subgroup, `[L]*P == O` (torsion /
///      low-order-key forgery guard, sq-l15mi -- on-curve alone does NOT exclude the
///      cofactor-8 torsion points; `sig::verify` mirrors this with
///      `is_in_correct_subgroup_assuming_on_curve`).
///   3. `pk` is not the neutral `(0, 1)` (identity-key universal forgery guard --
///      `sig::verify` rejects `pk.is_zero()`; the twisted-Edwards neutral is
///      `(0,1)`).
///   4. `s < L` (canonical scalar).
///   5. `e == Poseidon2(...) mod L` (challenge binding, see `challenge_scalar`).
///   6. `s*G == R + e*pk` (the verification equation).
pub fn schnorr_verify(pk: Point, r: Point, s: Field, e: Field, e_k: Field, m: Field) {
    // (1) on-curve guards for the private points.
    assert_on_curve(pk);
    assert_on_curve(r);
    // (2) [OPUS-4.8] sq-l15mi: prime-order subgroup membership. assert_on_curve does
    // NOT exclude the cofactor-8 torsion points; a torsion `pk` enables a no-secret
    // forgery on this hidden-issuer path (see assert_in_prime_order_subgroup). Guard
    // `pk` (the forgery vector) AND `R` (a free witness too -- defense in depth: an
    // honest R = k*G is already in the subgroup, so this never rejects a valid sig).
    assert_in_prime_order_subgroup(pk);
    assert_in_prime_order_subgroup(r);
    // (3) reject the identity public key (neutral (0,1)): e*pk = neutral for it,
    // so s*G == R is satisfiable for any (R = s*G), a universal forgery.
    assert(!((pk.x == 0) & (pk.y == 1)), "identity issuer key rejected");
    // (4) s canonical.
    assert_lt_l(s);
    // (5) bind e to the Poseidon2 challenge over (R, pk, m).
    let e_bound = challenge_scalar(r, pk, m, e, e_k);
    // (6) s*G == R + e*pk.
    let g = Point { x: BJJ_GX, y: BJJ_GY };
    let lhs = scalar_mul(s, g);
    let e_pk = scalar_mul(e_bound, pk);
    let rhs = point_add(r, e_pk);
    assert((lhs.x == rhs.x) & (lhs.y == rhs.y), "schnorr verification equation failed");
}

/// The Merkle leaf for an issuer public key in the key set K: `h2(pk.x, pk.y)`.
/// The Rust host mirror commits the same leaf (`poseidon2::hash(&[x, y])`), so
/// the set root is recomputable bit-for-bit verifier-side from the RP's
/// authoritative KeySet.
pub fn key_leaf(pk: Point) -> Field {
    h2(pk.x, pk.y)
}

/// Hidden-key set membership: prove the PRIVATE `pk` is a member of the set K
/// committed as the depth-`D` Poseidon2 Merkle root `key_set_root` (PUBLIC),
/// at a PRIVATE `index` along a PRIVATE authentication path `siblings`. Reuses
/// the revocation Merkle gadget pattern (`revoke::revoke_unset_check`): the leaf
/// is `key_leaf(pk)`, the directions are the bits of `index` (so a prover cannot
/// reuse another member's path under a different claimed index), and the fold
/// must recompute the public root.
pub fn key_set_membership<let D: u32>(
    pk: Point,
    key_set_root: Field,
    index: Field,
    siblings: [Field; D],
) {
    let leaf = key_leaf(pk);
    // Decompose index to exactly D bits in-circuit: directions = index bits, and
    // index < 2^D (in-range for the tree). Same discipline as revoke.nr.
    let dirs: [bool; D] = index.to_le_bits();
    let mut node = leaf;
    for k in 0..D {
        let sib = siblings[k];
        let is_right = dirs[k];
        let left = if is_right { sib } else { node };
        let right = if is_right { node } else { sib };
        node = h2(left, right);
    }
    assert(node == key_set_root, "issuer key is not a member of the committed key set");
}

/// The full hidden-issuer-attestation relation: the PRIVATE issuer key `pk`
/// (1) signed the PUBLIC commitment message `m` with the PRIVATE signature
/// `(R, s)` (Schnorr-over-Baby-JubJub), AND (2) is a member of the key set K
/// committed as the PUBLIC `key_set_root`. Proves "signed by SOME key in K"
/// without revealing which.
///
/// PUBLIC:  `m` (commitment message), `key_set_root`.
/// PRIVATE: `pk`, `r` (signature R), `s`, `e`, `e_k`, `index`, `siblings`.
pub fn hidden_issuer_attestation<let D: u32>(
    // -- public --
    m: Field,
    key_set_root: Field,
    // -- private --
    pk: Point,
    r: Point,
    s: Field,
    e: Field,
    e_k: Field,
    index: Field,
    siblings: [Field; D],
) {
    schnorr_verify(pk, r, s, e, e_k, m);
    key_set_membership::<D>(pk, key_set_root, index, siblings);
}

// Cost note (noir-optimisation): the dominant cost is the ~251-bit double-and-add
// scalar muls in schnorr_verify. The verification equation needs two (`s*G` and
// `e*pk`); the sq-l15mi prime-order-subgroup guards add two more (`[L]*pk` and
// `[L]*r`), so schnorr_verify now does FOUR scalar muls. Each bit does one double +
// one conditional add = two complete twisted-Edwards adds, each ~3 Field inversions
// + a handful of mults. ~251 * 4 * 2 ~= ~2000 point adds. The extra pair is the
// unavoidable price of torsion-key soundness (`is_in_correct_subgroup` on the host
// is one `[L]*P` per point too); the Merkle membership (one h2 per level) is tiny
// next to it. Measured `bb gates` for revoke_issuer_d10 is in the crate STATUS /
// the verifier docs. A follow-up MAY fold `[L]*pk` into `e*pk` via a shared ladder,
// but only after `bb gates` confirms the saving (noir-optimisation discipline).
