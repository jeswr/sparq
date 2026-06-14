// [OPUS-4.8] sq-nuok: adversarial-share negative suite + a 'no fake crypto'
// stub gate for sparq-mpc. Written while Fable unavailable — re-review on return.
//! Adversarial / robustness negative suite for `sparq-mpc` (sq-nuok).
//!
//! The honest-path differential suite (in `shamir.rs` / `join.rs`) already pins
//! that the REAL parts compute the right answer. This module is its adversarial
//! complement, and it is deliberately *empirically honest*: it asserts only what
//! the code ACTUALLY guarantees, and where a guarantee is MISSING it PINS the
//! absence (so nobody mistakes the semi-honest scaffold for malicious security)
//! and the gap is filed as a bead.
//!
//! Four parts, mapping to the sq-nuok deliverable:
//!
//! 1. **Tampered-share behaviour** ([`tamper`]). Honest-majority Shamir is
//!    SEMI-HONEST: it has NO tamper detection (that is guarantee (D), malicious
//!    security — explicitly deferred, see `shamir.rs` "Security model"). These
//!    tests document the *actual* behaviour: a corrupted share silently changes
//!    the reconstructed value (it does NOT panic, and it does NOT silently
//!    return the TRUE secret — there is no notion of "correct" for it to claim).
//!    FINDING (bead **sq-uu0u**): no integrity check even when redundant shares
//!    are present — a single tampered share flips the result undetected. This is
//!    correct for the stated model but is pinned here so the absence is explicit.
//!
//! 2. **Threshold boundary** ([`threshold`]). The information-theoretic hiding
//!    property at EXACTLY the boundary: `t` shares cannot reconstruct (a Protocol
//!    error, AND the `t` shares are consistent with every candidate secret), and
//!    `t+1` shares do reconstruct — both directions, both sides of the boundary.
//!
//! 3. **Field-arithmetic fuzz** ([`field_fuzz`]). Random add / sub / mul / neg /
//!    inv over many seeds, cross-checked against a `u128`-modulo reference. With
//!    `p = 2^61 - 1` every product of two field elements is `< 2^122 < 2^128`, so
//!    `(a as u128 * b as u128) % (P as u128)` is an EXACT bignum-free reference
//!    for the prime (the crate is deliberately `num-bigint`-free; a `u128`
//!    reference is sound here, not a compromise — see `field.rs` "Choice of p").
//!
//! 4. **'No fake crypto' table** ([`no_fake_crypto`]). Modelled on
//!    `sparq-zk-compose/tests/forge_gates.rs`: a gate-indexed map enumerating
//!    every deferred / `NotYetImplemented` entry point and asserting each STAYS a
//!    typed error that names its gate — never `Ok` with a fabricated/plaintext
//!    value. This is what stops the scaffold being mistaken for finished crypto.

use crate::backend::{BackendInfo, MpcBackend, TrustModel};
use crate::field::{Fp, P};
use crate::holder::Holder;
use crate::join::{DisclosedKeyJoin, GlobalJoin, HiddenValueJoin, JoinPlan};
use crate::partial::{HolderId, MpcError, PartialResult};
// The proof/attestation boundary types live in `crate::proof`.
use crate::proof::{Attestation, AttestationShare, CollaborativeProof, Proof, ProofStatement};
use crate::shamir::{
    self, add_shares, mul_shares_raw, reconstruct_degree, scale, ShamirBackend, Share,
};
use oxrdf::{Term, Variable};

fn fp(v: u64) -> Fp {
    Fp::new(v)
}

// =====================================================================
// PART 1 — Tampered / corrupted shares (the semi-honest reality).
// =====================================================================
mod tamper {
    use super::*;

    /// Corrupt one share's `y` value by adding a nonzero delta (a malicious /
    /// faulty party feeding a wrong share). `delta != 0` guarantees the value
    /// actually changes in the field.
    fn corrupt_one(shares: &mut [Share], idx: usize, delta: u64) {
        let nonzero = if delta % P == 0 { 1 } else { delta };
        shares[idx].y = shares[idx].y.add(fp(nonzero));
    }

    /// At EXACTLY `t+1` shares (no redundancy), tampering one share changes the
    /// reconstructed secret. There is no possible detection here — `t+1` points
    /// determine SOME degree-`t` polynomial, just the wrong one. We pin that the
    /// scheme: (a) does NOT panic, (b) does NOT silently return the TRUE secret
    /// (no magic self-correction), i.e. the corruption propagates as designed.
    #[test]
    fn tampered_share_at_minimum_set_changes_secret_silently() {
        let backend = ShamirBackend::new_seeded(5, 0xABCD).unwrap();
        let t = backend.threshold(); // (5-1)/2 = 2
        let secret = fp(123_456);
        let shares = backend.dealer().share(secret);

        // Honest reconstruction from t+1 shares is correct.
        let honest = backend.reconstruct(&shares[..t + 1]).unwrap();
        assert_eq!(
            honest, secret,
            "sanity: honest t+1 reconstruction is correct"
        );

        // Tamper one of those t+1 shares.
        let mut tampered = shares[..t + 1].to_vec();
        corrupt_one(&mut tampered, 0, 9_999);

        // It still RETURNS a value (no panic), but a DIFFERENT one — and crucially
        // NOT the true secret. The scheme has no integrity check at this set size,
        // and that is the documented semi-honest behaviour.
        let got = backend
            .reconstruct(&tampered)
            .expect("Lagrange interpolation never errors on t+1 distinct points");
        assert_ne!(
            got, secret,
            "FINDING (sq-nuok pin → bead sq-uu0u): tampered share silently changes the reconstructed \
             secret — no tamper detection (semi-honest model; malicious security \
             deferred, guarantee (D))"
        );
    }

    /// Even with MAXIMAL redundancy (all `n` shares present, of which any `t+1`
    /// suffice), a single tampered share STILL silently corrupts the result. This
    /// sharpens the finding: the impl does NOT exploit the available redundancy to
    /// detect (let alone correct) the error — it just Lagrange-interpolates the
    /// whole over-determined set, which an inconsistent point poisons. A
    /// malicious-secure scheme (VSS / IT-MACs) WOULD catch this; v1 does not.
    #[test]
    fn tampered_share_with_full_redundancy_is_still_undetected() {
        let backend = ShamirBackend::new_seeded(7, 0x5EED).unwrap();
        let secret = fp(2_024);
        let mut shares = backend.dealer().share(secret); // n = 7 shares

        assert_eq!(
            backend.reconstruct(&shares).unwrap(),
            secret,
            "honest full set ok"
        );

        corrupt_one(&mut shares, 3, 1); // flip a single share by +1

        let got = backend.reconstruct(&shares).unwrap();
        assert_ne!(
            got, secret,
            "FINDING (sq-nuok pin → bead sq-uu0u): a single tampered share corrupts reconstruction \
             even with full n-share redundancy — the impl does no consistency / \
             error-detection across subsets (no malicious security)"
        );
    }

    /// The secure-equality primitive (`HiddenValueJoin::secure_equal`) opens
    /// `m = d·r` at degree `2t` and reports `m == 0` as the match bit. We
    /// reproduce that EXACT primitive from the public `shamir` ops and show that
    /// tampering one share of the opened product FLIPS the verdict undetected:
    /// a forged share can turn a true match into a false non-match (and a
    /// non-match into a spurious match). This is the join-layer face of the same
    /// missing guarantee — pinned, not implied.
    #[test]
    fn tampered_share_flips_secure_equality_verdict_undetected() {
        // n = 5, t = 2 → one multiplication opens at degree 2t = 4 (needs 5 pts).
        let backend = ShamirBackend::new_seeded(5, 0xEEEE).unwrap();
        let mut dealer = backend.dealer();
        let t = dealer.threshold();

        // Two EQUAL keys → honest verdict is "match" (m == 0).
        let key = fp(42);
        let sa = dealer.share(key);
        let sb = dealer.share(key);
        let mask = dealer.draw_nonzero_fp();
        let r = dealer.share(mask);
        let d = shamir::sub_shares(&sa, &sb).unwrap();
        let m_shares = mul_shares_raw(&d, &r).unwrap();

        // Honest open: equal keys ⇒ m == 0 ⇒ match.
        let m_honest = reconstruct_degree(&m_shares, 2 * t).unwrap();
        assert_eq!(
            m_honest,
            Fp::zero(),
            "sanity: equal keys open to m = 0 (match)"
        );

        // Tamper ONE product share: the opened m is now nonzero ⇒ verdict flips to
        // "no match", with NO signal that a share was corrupted.
        let mut tampered = m_shares.clone();
        tampered[0].y = tampered[0].y.add(fp(1));
        let m_tampered = reconstruct_degree(&tampered, 2 * t).unwrap();
        assert_ne!(
            m_tampered,
            Fp::zero(),
            "FINDING (sq-nuok pin → bead sq-uu0u): tampering one product share flips a true secure- \
             equality MATCH into a non-match undetected (semi-honest; no integrity)"
        );
    }

    /// Mismatched evaluation points (a party submitting a share on the WRONG `x`)
    /// in a LINEAR op (`add_shares`) IS caught — this is a real structural guard
    /// (not crypto integrity): the point-set check rejects it as a Protocol error.
    /// We assert the guard so the boundary between "checked structure" and
    /// "unchecked value integrity" is explicit.
    #[test]
    fn mismatched_points_in_linear_op_are_a_protocol_error() {
        let backend = ShamirBackend::new_seeded(4, 1).unwrap();
        let a = backend.dealer().share(fp(10));
        let mut b = backend.dealer().share(fp(20));
        // Corrupt b's evaluation point so the point-sets disagree.
        b[1].x += 1000;
        let err = add_shares(&a, &b).unwrap_err();
        assert!(
            matches!(err, MpcError::Protocol(_)),
            "a point-set mismatch must be a typed Protocol error, got {err:?}"
        );
    }

    /// A tampered share inside a SECURE SUM (`run_secure`) likewise propagates to
    /// a wrong reconstructed total — the aggregate is just as unprotected as a
    /// single reconstruction. We pin it to cover the `run_secure` path too.
    #[test]
    fn tampered_share_corrupts_secure_sum_silently() {
        let backend = ShamirBackend::new_seeded(4, 0x5151).unwrap();
        let mut dealer = backend.dealer();
        let inputs: Vec<Vec<Share>> = [10u64, 20, 30]
            .iter()
            .map(|&v| dealer.share(fp(v)))
            .collect();
        let summed = backend.run_secure(&inputs).unwrap();
        assert_eq!(
            backend.reconstruct(&summed[0]).unwrap(),
            fp(60),
            "honest sum"
        );

        // Corrupt one share of the summed sharing.
        let mut bad = summed.clone();
        bad[0][0].y = bad[0][0].y.add(fp(7));
        let got = backend.reconstruct(&bad[0]).unwrap();
        assert_ne!(
            got,
            fp(60),
            "FINDING (sq-nuok pin → bead sq-uu0u): a tampered share in run_secure's output corrupts \
             the reconstructed aggregate undetected"
        );
    }
}

// =====================================================================
// PART 2 — Threshold boundary, both sides (information-theoretic hiding).
// =====================================================================
mod threshold {
    use super::*;

    /// `t` shares (one BELOW the `t+1` reconstruction threshold) cannot
    /// reconstruct: it is a typed Protocol error, NOT a silent wrong/honest-
    /// looking value. This is the "fails closed below threshold" direction.
    #[test]
    fn exactly_threshold_shares_cannot_reconstruct() {
        // n = 7 → t = 3. Exactly t = 3 shares must be refused (need t+1 = 4).
        let backend = ShamirBackend::new_seeded(7, 11).unwrap();
        let t = backend.threshold();
        assert_eq!(t, 3);
        let shares = backend.dealer().share(fp(98_765));

        let err = backend.reconstruct(&shares[..t]).unwrap_err();
        match err {
            MpcError::Protocol(m) => {
                assert!(
                    m.contains("reconstruction needs"),
                    "should name the threshold: {m}"
                )
            }
            other => panic!("t shares must be a Protocol error, got {other:?}"),
        }
    }

    /// `t+1` shares (exactly AT the reconstruction threshold) DO reconstruct the
    /// true secret — the "succeeds at threshold" direction. Tested at several `n`.
    #[test]
    fn exactly_threshold_plus_one_shares_reconstruct() {
        for (n, seed) in [(3usize, 1u64), (5, 2), (7, 3), (9, 4)] {
            let backend = ShamirBackend::new_seeded(n, seed).unwrap();
            let t = backend.threshold();
            let secret = fp(31_337 + n as u64);
            let shares = backend.dealer().share(secret);
            // Take EXACTLY t+1 shares (the minimum that must succeed).
            let got = backend
                .reconstruct(&shares[..t + 1])
                .unwrap_or_else(|e| panic!("n={n}: t+1 shares must reconstruct, got {e:?}"));
            assert_eq!(got, secret, "n={n}: t+1 shares reconstruct the true secret");
        }
    }

    /// INFORMATION-THEORETIC HIDING at the boundary: a set of EXACTLY `t` shares
    /// is consistent with EVERY candidate secret. We prove it constructively —
    /// for two DIFFERENT target secrets we exhibit a `(t+1)`-th share that, with
    /// the same `t` shares, interpolates to each target. Hence the `t` shares
    /// alone determine nothing about the secret (the core privacy guarantee).
    #[test]
    fn t_shares_are_consistent_with_every_secret() {
        // n = 5 → t = 2. Take 2 shares; show they fit BOTH secret 1000 and 2000.
        let backend = ShamirBackend::new_seeded(5, 7).unwrap();
        let t = backend.threshold();
        assert_eq!(t, 2);
        let actual = fp(1000);
        let shares = backend.dealer().share(actual);
        let two = &shares[..t]; // exactly t shares

        for &target in &[fp(1000), fp(2000), fp(0), fp(P - 1)] {
            let forged = forge_completion(two, target);
            let mut combined = two.to_vec();
            combined.push(forged);
            // With the forged (t+1)-th share, the SAME t shares reconstruct to the
            // chosen target — so the t shares carry zero information about which.
            assert_eq!(
                reconstruct_degree(&combined, t).unwrap(),
                target,
                "t shares must be consistent with target {target:?} (hiding)"
            );
        }
    }

    /// Given `t` shares on a degree-`t` polynomial, produce a `(t+1)`-th share at
    /// a fresh `x` so the `t+1` points interpolate to `target` at `x = 0`. This is
    /// the constructive witness of information-theoretic hiding (the constant term
    /// is free until the `(t+1)`-th point is fixed). Test-only.
    fn forge_completion(shares: &[Share], target: Fp) -> Share {
        // Pick a fresh evaluation point distinct from the existing ones.
        let x_new = (1..=10_000u64)
            .find(|x| shares.iter().all(|s| s.x != *x))
            .expect("a fresh point exists");
        let mut xs: Vec<u64> = shares.iter().map(|s| s.x).collect();
        xs.push(x_new);
        // Lagrange weight at 0 for the i-th point of the (t+1)-point set.
        let weight = |i: usize| -> Fp {
            let xi = fp(xs[i]);
            let mut num = Fp::one();
            let mut den = Fp::one();
            for (j, &xj) in xs.iter().enumerate() {
                if i == j {
                    continue;
                }
                let xj = fp(xj);
                num = num.mul(xj.neg());
                den = den.mul(xi.sub(xj));
            }
            num.mul(den.inv())
        };
        // target = Σ_{i<t} y_i·L_i + y_new·L_new  ⇒  solve y_new.
        let last = shares.len(); // index of x_new in xs
        let mut acc = Fp::zero();
        for (i, s) in shares.iter().enumerate() {
            acc = acc.add(s.y.mul(weight(i)));
        }
        let l_new = weight(last);
        let y_new = target.sub(acc).mul(l_new.inv());
        Share { x: x_new, y: y_new }
    }

    /// Boundary symmetry across many `n`: `t` fails (typed error), `t+1` succeeds.
    /// One table covering both sides at once so the boundary is unambiguous.
    #[test]
    fn boundary_table_t_fails_t_plus_one_succeeds() {
        for (n, seed) in [(2usize, 10u64), (3, 11), (4, 12), (6, 13), (8, 14)] {
            let backend = ShamirBackend::new_seeded(n, seed).unwrap();
            let t = backend.threshold();
            let secret = fp(7 * n as u64 + 1);
            let shares = backend.dealer().share(secret);

            // BELOW threshold (t shares): refused. (For n=2, t=1 ⇒ 1 share fails.)
            assert!(
                backend.reconstruct(&shares[..t]).is_err(),
                "n={n}, t={t}: exactly t shares must NOT reconstruct"
            );
            // AT threshold (t+1 shares): succeeds with the true secret.
            assert_eq!(
                backend.reconstruct(&shares[..t + 1]).unwrap(),
                secret,
                "n={n}, t={t}: t+1 shares must reconstruct"
            );
        }
    }
}

// =====================================================================
// PART 3 — Field-arithmetic fuzz vs a u128-modulo reference.
// =====================================================================
mod field_fuzz {
    use super::*;

    /// Deterministic SplitMix64 (local to the fuzz; reproducible across runs).
    struct Lcg(u64);
    impl Lcg {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^ (z >> 31)
        }
        /// A canonical field element in [0, P) via rejection (no modulo bias).
        fn next_fp(&mut self) -> u64 {
            loop {
                let c = self.next() & ((1u64 << 61) - 1);
                if c < P {
                    return c;
                }
            }
        }
    }

    const PP: u128 = P as u128;

    /// add / sub / neg / mul over many random pairs and seeds, each checked
    /// against the EXACT `u128`-modulo reference. With p = 2^61-1 the product of
    /// two field elements is < 2^122 < 2^128, so `(a*b) % P` in u128 is exact —
    /// a sound bignum-free reference for this prime (field.rs "Choice of p").
    #[test]
    fn add_sub_neg_mul_match_u128_reference_over_many_seeds() {
        for seed in 0..64u64 {
            let mut rng = Lcg(seed.wrapping_mul(0x100_0001).wrapping_add(1));
            for _ in 0..2_000 {
                let a = rng.next_fp();
                let b = rng.next_fp();
                let (fa, fb) = (fp(a), fp(b));

                assert_eq!(fa.add(fb).value() as u128, (a as u128 + b as u128) % PP);
                assert_eq!(
                    fa.sub(fb).value() as u128,
                    ((a as u128 + PP) - b as u128) % PP
                );
                assert_eq!(fa.neg().value() as u128, (PP - a as u128) % PP);
                assert_eq!(fa.mul(fb).value() as u128, (a as u128 * b as u128) % PP);
            }
        }
    }

    /// Multiplicative inverse: `a · a⁻¹ == 1` AND `a⁻¹` equals the reference
    /// `a^(p-2) mod p` computed independently in u128 (so we are not just trusting
    /// `Fp::pow` to validate `Fp::inv`). Over many random NONZERO elements.
    #[test]
    fn inverse_matches_independent_u128_modpow_reference() {
        // Independent reference modular exponentiation in u128 (square-and-multiply).
        fn modpow_u128(mut base: u128, mut exp: u128, m: u128) -> u128 {
            let mut acc = 1u128;
            base %= m;
            while exp > 0 {
                if exp & 1 == 1 {
                    acc = acc * base % m;
                }
                base = base * base % m;
                exp >>= 1;
            }
            acc
        }
        for seed in 0..32u64 {
            let mut rng = Lcg(seed.wrapping_mul(2_654_435_761).wrapping_add(1));
            for _ in 0..1_000 {
                let mut a = rng.next_fp();
                if a == 0 {
                    a = 1; // inv(0) is undefined / panics — skip the single zero.
                }
                let fa = fp(a);
                let inv = fa.inv();
                // a · a⁻¹ = 1.
                assert_eq!(fa.mul(inv), Fp::one(), "a={a}: a·inv(a) must be 1");
                // a⁻¹ = a^(p-2) mod p, computed independently in u128.
                let reference = modpow_u128(a as u128, (P - 2) as u128, PP);
                assert_eq!(
                    inv.value() as u128,
                    reference,
                    "a={a}: inv mismatch vs u128 modpow"
                );
            }
        }
    }

    /// Field LAWS under fuzz: associativity/commutativity of + and ·, the
    /// distributive law, and additive/multiplicative identities — over random
    /// triples. A reduction bug (e.g. a bad Mersenne fold) breaks at least one.
    #[test]
    fn field_axioms_hold_under_fuzz() {
        let mut rng = Lcg(0xF1E1D);
        for _ in 0..50_000 {
            let a = fp(rng.next_fp());
            let b = fp(rng.next_fp());
            let c = fp(rng.next_fp());

            assert_eq!(a.add(b), b.add(a), "+ commutes");
            assert_eq!(a.mul(b), b.mul(a), "· commutes");
            assert_eq!(a.add(b).add(c), a.add(b.add(c)), "+ associates");
            assert_eq!(a.mul(b).mul(c), a.mul(b.mul(c)), "· associates");
            assert_eq!(
                a.mul(b.add(c)),
                a.mul(b).add(a.mul(c)),
                "· distributes over +"
            );
            assert_eq!(a.add(Fp::zero()), a, "0 is additive identity");
            assert_eq!(a.mul(Fp::one()), a, "1 is multiplicative identity");
            assert_eq!(a.add(a.neg()), Fp::zero(), "neg is additive inverse");
        }
    }

    /// Edge operands around the modulus: 0, 1, P-1, P-2, and small values — the
    /// reduction's conditional-subtraction branches. Checked vs u128 reference.
    #[test]
    fn boundary_operands_match_reference() {
        let edges = [0u64, 1, 2, 3, P - 1, P - 2, P - 3, 1 << 30, 1 << 60];
        for &a in &edges {
            for &b in &edges {
                let (fa, fb) = (fp(a), fp(b));
                assert_eq!(
                    fa.add(fb).value() as u128,
                    (a as u128 + b as u128) % PP,
                    "add {a}+{b}"
                );
                assert_eq!(
                    fa.mul(fb).value() as u128,
                    (a as u128 * b as u128) % PP,
                    "mul {a}*{b}"
                );
                assert_eq!(
                    fa.sub(fb).value() as u128,
                    ((a as u128 + PP) - b as u128) % PP,
                    "sub {a}-{b}"
                );
            }
        }
    }

    /// `scale` (public share op) is exactly field multiplication applied per
    /// share — fuzz it against the reference so the share-arithmetic helper, not
    /// just `Fp`, is covered.
    #[test]
    fn scale_share_op_matches_field_mul() {
        let backend = ShamirBackend::new_seeded(4, 0x5CA1E).unwrap();
        let mut rng = Lcg(0xC0FFEE);
        for _ in 0..500 {
            let secret = rng.next_fp();
            let factor = rng.next_fp();
            let shares = backend.dealer().share(fp(secret));
            let scaled = scale(&shares, fp(factor));
            let got = backend.reconstruct(&scaled).unwrap();
            let expect = (secret as u128 * factor as u128) % PP;
            assert_eq!(got.value() as u128, expect, "scale({secret})·{factor}");
        }
    }
}

// =====================================================================
// PART 4 — 'No fake crypto' table: every deferred entry point STAYS typed.
// Modelled on sparq-zk-compose/tests/forge_gates.rs (one gate per row).
// =====================================================================
mod no_fake_crypto {
    use super::*;

    /// Assert an `MpcError` is the deferred-stub channel AND names a gate. Every
    /// deferred path must carry its gate in the VALUE (not just a doc-comment) so
    /// the deferral is auditable at the call site (see `partial.rs` MpcError doc).
    fn assert_deferred(err: &MpcError, must_mention: &[&str]) {
        match err {
            MpcError::NotYetImplemented { what, gated_on } => {
                let hay = format!("{what} || {gated_on}");
                for needle in must_mention {
                    assert!(
                        hay.contains(needle),
                        "deferred error must name {needle:?}; got what={what:?} gated_on={gated_on:?}"
                    );
                }
            }
            other => panic!(
                "expected NotYetImplemented (the no-fake-crypto channel), got {other:?} \
                 — a deferred path returned a non-typed-stub result"
            ),
        }
    }

    // --- helpers reused across rows ---------------------------------------

    fn holder_with_salary() -> Holder {
        Holder::from_rdf(
            "h",
            "@prefix ex: <http://ex/> . @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
             ex:h ex:salary \"5\"^^xsd:integer .",
            "turtle",
        )
        .unwrap()
    }

    /// A do-nothing backend whose three trait methods are HONEST stubs — exactly
    /// the shape a future dishonest-majority backend (or any not-yet-built
    /// backend) presents before its crypto lands. Mirrors `proof.rs`'s StubBackend
    /// but lives here so the table is self-contained.
    struct StubBackend;
    impl MpcBackend for StubBackend {
        type Share = ();
        fn info(&self) -> BackendInfo {
            BackendInfo {
                name: "stub-deferred",
                trust_model: TrustModel::DishonestMajority,
                malicious_secure: false,
            }
        }
        fn share_private_input(&self, _h: &Holder) -> Result<Vec<()>, MpcError> {
            Err(MpcError::not_yet("secret-share private input", "M3 + Q2"))
        }
        fn run_secure(&self, _s: &[()]) -> Result<Vec<()>, MpcError> {
            Err(MpcError::not_yet("run secure computation", "M3 + Q2"))
        }
        fn reconstruct_disclosed(&self, _s: &[()]) -> Result<PartialResult, MpcError> {
            Err(MpcError::not_yet("reconstruct disclosed output", "M3 + Q2"))
        }
    }

    // === ROW 1: disclosed-key join routes the HIDDEN-VALUE regime away. =====
    // The crypto-free DisclosedKeyJoin must NOT fake a private-key join: a plan
    // with key_disclosed == false stays a typed NotYetImplemented naming M3/Q2/Q3.
    #[test]
    fn row1_disclosed_join_hidden_path_is_typed_stub() {
        let h = Holder::from_rdf(
            "a",
            "@prefix ex: <http://ex/> . ex:p1 ex:knows ex:x1 .",
            "turtle",
        )
        .unwrap();
        let pa = h
            .evaluate_local("PREFIX ex: <http://ex/> SELECT ?p ?x WHERE { ?p ex:knows ?x }")
            .unwrap();
        let plan = JoinPlan {
            join_var: Variable::new_unchecked("x"),
            key_disclosed: false,
        };
        let err = DisclosedKeyJoin.join(&[pa], &plan).unwrap_err();
        assert_deferred(&err, &["hidden-value", "M3", "Q2", "Q3"]);
    }

    // === ROW 2: collaborative-proof `prove` is deferred (Q1 + ZK foundation). ==
    #[test]
    fn row2_collaborative_prove_is_typed_stub() {
        struct StubProof;
        impl CollaborativeProof<StubBackend> for StubProof {
            fn prove(
                &self,
                _b: &StubBackend,
                _s: &ProofStatement,
                _j: &[JoinPlan],
            ) -> Result<Proof, MpcError> {
                Err(MpcError::not_yet(
                    "collaborative proof of correct + attested-source result",
                    "ZK foundation #3/#4/#5/#6/#8/#9/#12 + Q1 distributed-attestation spike (M4)",
                ))
            }
            fn verify(&self, _s: &ProofStatement, _p: &Proof) -> Result<bool, MpcError> {
                Err(MpcError::not_yet(
                    "verify collaborative proof",
                    "ZK foundation + M4",
                ))
            }
        }
        let stmt = ProofStatement {
            query: "SELECT ?x WHERE { ?x ?p ?o }".into(),
            disclosed_key_set: vec!["did:example:k#1".into()],
            challenge: vec![],
            disclosed_result: PartialResult {
                holder: HolderId::new("fed"),
                vars: vec![Variable::new_unchecked("x")],
                rows: vec![],
            },
        };
        let err = StubProof.prove(&StubBackend, &stmt, &[]).unwrap_err();
        // Must cite the issuer-signature gate (#3) AND the Q1 research spike — a
        // proof that "just works" without these would be fake crypto.
        assert_deferred(&err, &["#3", "Q1"]);
    }

    // === ROW 3: collaborative-proof `verify` is deferred too. ================
    #[test]
    fn row3_collaborative_verify_is_typed_stub() {
        struct StubProof;
        impl CollaborativeProof<StubBackend> for StubProof {
            fn prove(
                &self,
                _b: &StubBackend,
                _s: &ProofStatement,
                _j: &[JoinPlan],
            ) -> Result<Proof, MpcError> {
                Err(MpcError::not_yet("prove", "M4"))
            }
            fn verify(&self, _s: &ProofStatement, _p: &Proof) -> Result<bool, MpcError> {
                Err(MpcError::not_yet(
                    "verify collaborative proof",
                    "ZK foundation + M4",
                ))
            }
        }
        let stmt = ProofStatement {
            query: "SELECT * WHERE { ?s ?p ?o }".into(),
            disclosed_key_set: vec![],
            challenge: vec![],
            disclosed_result: PartialResult {
                holder: HolderId::new("fed"),
                vars: vec![],
                rows: vec![],
            },
        };
        let proof = Proof::test_stub();
        let err = StubProof.verify(&stmt, &proof).unwrap_err();
        // CRITICAL: verify must NOT return Ok(true) — a stub that "accepts" any
        // proof is the worst possible fake crypto. It must be the typed stub.
        assert!(
            !matches!(StubProof.verify(&stmt, &proof), Ok(true)),
            "FAKE-CRYPTO GUARD: deferred verify must never accept (Ok(true))"
        );
        assert_deferred(&err, &["verify", "M4"]);
    }

    // === ROW 4: distributed attestation (the federated #3) is deferred. ======
    #[test]
    fn row4_attestation_is_typed_stub() {
        struct StubAttest;
        impl Attestation for StubAttest {
            fn attest_source(
                &self,
                _h: &HolderId,
                _k: &[String],
            ) -> Result<AttestationShare, MpcError> {
                Err(MpcError::not_yet(
                    "distributed issuer-signature attestation over secret-shared witness",
                    "ZK foundation #3/#8/#9 + Q1",
                ))
            }
        }
        let err = StubAttest
            .attest_source(&HolderId::new("alice"), &[])
            .unwrap_err();
        assert_deferred(&err, &["attestation", "#3", "Q1"]);
    }

    // === ROW 5: a not-yet-built backend's share/run/reconstruct stay typed. ==
    // Documents the trust-model seam: a backend whose crypto is not implemented
    // (e.g. the future dishonest-majority one) must fail closed on ALL three
    // obligations — never return a plaintext / fabricated share or result.
    #[test]
    fn row5_deferred_backend_methods_are_typed_stubs() {
        let b = StubBackend;
        let h = holder_with_salary();
        assert_deferred(
            &b.share_private_input(&h).unwrap_err(),
            &["secret-share", "Q2"],
        );
        assert_deferred(
            &b.run_secure(&[]).unwrap_err(),
            &["secure computation", "Q2"],
        );
        assert_deferred(
            &b.reconstruct_disclosed(&[]).unwrap_err(),
            &["reconstruct", "Q2"],
        );
        // And its self-declared guarantees are honest: NOT malicious-secure.
        assert!(
            !b.info().malicious_secure,
            "stub backend must not claim malicious security"
        );
    }

    // === ROW 6: the REAL Shamir backend is NOT a stub on its real methods, but
    // it MUST still fail closed on protocol-precondition violations (typed
    // Protocol error, never a fabricated answer). This is the "real path is real,
    // and its error paths are typed too" control for the table. ================
    #[test]
    fn row6_real_backend_fails_closed_on_preconditions_not_with_fake_data() {
        // n < 2 is refused (typed Protocol error), not silently coerced.
        assert!(matches!(ShamirBackend::new(1), Err(MpcError::Protocol(_))));

        // run_secure with no inputs is a typed Protocol error, not an Ok(empty).
        let b = ShamirBackend::new_seeded(3, 1).unwrap();
        let empty: Vec<Vec<Share>> = vec![];
        assert!(matches!(b.run_secure(&empty), Err(MpcError::Protocol(_))));

        // reconstruct below threshold is a typed error, not a fabricated value
        // (overlaps the threshold suite; restated here as the no-fake-crypto row
        // for the REAL backend's reconstruction path).
        let shares = b.dealer().share(fp(5));
        let t = b.threshold();
        assert!(matches!(
            b.reconstruct(&shares[..t]),
            Err(MpcError::Protocol(_))
        ));

        // And the honest path still really works (the table doesn't break it).
        assert_eq!(b.reconstruct(&shares).unwrap(), fp(5));
        // The real backend declares its honest guarantees: honest-majority,
        // semi-honest (NOT malicious-secure).
        let info = b.info();
        assert_eq!(info.trust_model, TrustModel::HonestMajority);
        assert!(
            !info.malicious_secure,
            "v1 Shamir must NOT claim malicious security"
        );
    }

    // === ROW 7: HiddenValueJoin (the REAL M3 hidden join) is real — assert it
    // does NOT silently reconstruct/disclose the private KEY (only the payload).
    // A hidden join that leaked the key would be the join-layer's "fake privacy".
    #[test]
    fn row7_hidden_join_discloses_only_payload_not_the_private_key() {
        use crate::join::HiddenKeyedRows;
        let lit = |s: &str| Some(Term::Literal(oxrdf::Literal::new_simple_literal(s)));
        let left = HiddenKeyedRows {
            holder: HolderId::new("L"),
            payload_vars: vec![Variable::new_unchecked("name")],
            rows: vec![(fp(100), vec![lit("Alice")]), (fp(200), vec![lit("Bob")])],
        };
        let right = HiddenKeyedRows {
            holder: HolderId::new("R"),
            payload_vars: vec![Variable::new_unchecked("city")],
            rows: vec![(fp(200), vec![lit("Leeds")])],
        };
        let join = HiddenValueJoin::new(ShamirBackend::new_seeded(3, 0xBEEF).unwrap());
        let got = join.join(&left, &right).unwrap();
        // Output schema is payload-only: the private key var is NOT projected.
        assert_eq!(
            got.vars,
            vec![
                Variable::new_unchecked("name"),
                Variable::new_unchecked("city")
            ]
        );
        // And the disclosed rows carry NO field-encoded key (100/200) — only the
        // matched payload (Bob, Leeds).
        let rendered = format!("{:?}", got.rows);
        assert!(
            !rendered.contains("100") && !rendered.contains("200"),
            "key must not leak: {rendered}"
        );
        assert_eq!(got.rows.len(), 1, "exactly the matching (Bob,Leeds) row");
    }
}
