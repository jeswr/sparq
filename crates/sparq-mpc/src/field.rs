// [OPUS-4.8] Prime-field arithmetic for Shamir secret sharing (M3).
//! Arithmetic in the prime field `F_p` underpinning Shamir secret sharing.
//!
//! Architecture refs: §3.1 (secret-sharing families), §4.3 step 4 (the secure
//! computation runs over secret-shared values). Honest-majority Shamir requires
//! a field; this module is that field and nothing more — no networking, no
//! sharing logic (that is [`crate::shamir`]).
//!
//! ## Choice of `p`
//!
//! A 61-bit Mersenne prime `p = 2^61 - 1` (the largest Mersenne prime below
//! 2^63). Three reasons:
//! - **Products stay in `u128`.** Two field elements are `< 2^61`, so their
//!   product is `< 2^122`, comfortably inside `u128` — reduction is a couple of
//!   shifts-and-adds, no big-integer dependency. This keeps the crate
//!   dependency-free (no `num-bigint` / `ark-ff` reaching the build graph) which
//!   matters for the native-only / wasm-exclusion invariant.
//! - **Range covers the use case.** Salaries, counts, and 61-bit-truncated IRI
//!   fingerprints all fit under 2^61. The flatmate cumulative-salary aggregate
//!   (four × ~10^5) is six orders of magnitude clear of the modulus, so the
//!   secure sum never wraps and reconstructs to the true integer.
//! - **It is a real prime field**, so Lagrange interpolation (reconstruction)
//!   and the additive/multiplicative structure Shamir relies on are exact — this
//!   is NOT a toy modular-arithmetic stand-in.
//!
//! ## Honesty note
//!
//! This is textbook finite-field arithmetic (Shamir 1979; standard MPC field
//! ops). It is the *correct* substrate for honest-majority Shamir. It is not, by
//! itself, a security claim: security comes from the sharing scheme + threshold
//! ([`crate::shamir`]) under the honest-majority semi-honest model stated there.

/// The field modulus `p = 2^61 - 1` (a Mersenne prime).
pub const P: u64 = (1u64 << 61) - 1;

/// An element of `F_p`, always kept canonical in `[0, P)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Fp(u64);

// Inherent `add`/`sub`/`mul`/`neg` are deliberate: field arithmetic reads
// clearest as `a.add(b)` in the share/interpolation code, and `Fp` is an
// internal crate type (not a public numeric tower), so shadowing the std op
// trait names carries no confusion risk for callers. [OPUS-4.8]
#[allow(clippy::should_implement_trait)]
impl Fp {
    /// Embed a `u64` into the field (reducing mod `p`).
    #[inline]
    pub fn new(v: u64) -> Self {
        Fp(v % P)
    }

    /// The raw canonical representative in `[0, P)`.
    #[inline]
    pub fn value(self) -> u64 {
        self.0
    }

    /// Additive identity.
    #[inline]
    pub fn zero() -> Self {
        Fp(0)
    }

    /// Multiplicative identity.
    #[inline]
    pub fn one() -> Self {
        Fp(1)
    }

    /// Field addition.
    #[inline]
    pub fn add(self, other: Fp) -> Fp {
        // a, b < P < 2^61, so a + b < 2^62 — no u64 overflow before reduction.
        let s = self.0 + other.0;
        Fp(if s >= P { s - P } else { s })
    }

    /// Field subtraction.
    #[inline]
    pub fn sub(self, other: Fp) -> Fp {
        Fp(if self.0 >= other.0 {
            self.0 - other.0
        } else {
            // self - other + P, computed without underflow.
            self.0 + P - other.0
        })
    }

    /// Field negation.
    #[inline]
    pub fn neg(self) -> Fp {
        if self.0 == 0 {
            Fp(0)
        } else {
            Fp(P - self.0)
        }
    }

    /// Field multiplication. Reduces the `u128` product mod the Mersenne prime.
    #[inline]
    pub fn mul(self, other: Fp) -> Fp {
        let prod = (self.0 as u128) * (other.0 as u128);
        Fp(reduce_mersenne(prod))
    }

    /// Multiplicative inverse via Fermat's little theorem: `a^(p-2) mod p`.
    /// Panics if `self` is zero (zero has no inverse) — callers (Lagrange
    /// interpolation) only ever invert non-zero differences of distinct points.
    pub fn inv(self) -> Fp {
        assert!(self.0 != 0, "Fp::inv of zero");
        self.pow(P - 2)
    }

    /// Modular exponentiation by square-and-multiply.
    pub fn pow(self, mut exp: u64) -> Fp {
        let mut base = self;
        let mut acc = Fp::one();
        while exp > 0 {
            if exp & 1 == 1 {
                acc = acc.mul(base);
            }
            base = base.mul(base);
            exp >>= 1;
        }
        acc
    }
}

/// Reduce a 128-bit product modulo the Mersenne prime `p = 2^61 - 1` using the
/// identity `2^61 ≡ 1 (mod p)`: fold the high bits into the low bits. Two folds
/// then a conditional subtraction suffice for any `u128` product of two
/// sub-`2^61` operands.
#[inline]
fn reduce_mersenne(mut x: u128) -> u64 {
    const MASK: u128 = (1u128 << 61) - 1;
    // First fold: low 61 bits + (high bits × 2^61 ≡ 1, i.e. just added back).
    x = (x & MASK) + (x >> 61);
    // Second fold handles a carry the first fold may have pushed past 2^61.
    x = (x & MASK) + (x >> 61);
    let mut r = x as u64;
    if r >= P {
        r -= P;
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_sub_roundtrip() {
        let a = Fp::new(12345);
        let b = Fp::new(67890);
        assert_eq!(a.add(b).sub(b), a);
        assert_eq!(a.sub(b).add(b), a);
    }

    #[test]
    fn add_wraps_at_modulus() {
        let a = Fp::new(P - 1);
        assert_eq!(a.add(Fp::one()), Fp::zero());
    }

    #[test]
    fn neg_is_additive_inverse() {
        let a = Fp::new(999_999);
        assert_eq!(a.add(a.neg()), Fp::zero());
    }

    #[test]
    fn mul_reduces_near_modulus() {
        // Large operands near the modulus exercise the u128 reduction path.
        let a = Fp::new(P - 3);
        let b = Fp::new(P - 5);
        // (P-3)(P-5) = P^2 - 8P + 15 ≡ 15 (mod P).
        assert_eq!(a.mul(b), Fp::new(15));
    }

    #[test]
    fn inv_is_multiplicative_inverse() {
        for v in [1u64, 2, 3, 7, 12345, P - 1] {
            let a = Fp::new(v);
            assert_eq!(a.mul(a.inv()), Fp::one(), "inv of {v}");
        }
    }

    #[test]
    fn reduce_matches_reference_modulo() {
        // Cross-check the Mersenne fold against a plain u128 % P for a spread of
        // products — the fold must be exact, not approximate.
        for &(x, y) in &[(P - 1, P - 1), (1234567u64, 7654321u64), (P / 2, P / 2 + 1)] {
            let prod = (x as u128) * (y as u128);
            assert_eq!(reduce_mersenne(prod) as u128, prod % (P as u128));
        }
    }
}
