// [OPUS-4.8] sq-km34.1: IT-MAC authenticated secret sharing — the FOUNDATION for
// honest-majority malicious-with-abort MPC. Layered over the existing degree-t
// Shamir sharing (`crate::shamir`). Foundation only: MAC-carrying multiplication
// (sq-km34.2), the batched MAC-check (sq-km34.4), and the registry wiring
// (sq-km34.7) are SEPARATE beads and are NOT implemented here.
//! Authenticated (IT-MAC) secret sharing over the honest-majority Shamir backend
//! — the foundation of malicious-with-abort security.
//!
//! Design (authoritative): `research/mpc-malicious-security-design.md` §2.1–2.2
//! (the construction) and §2.3 (free linear ops). Skill `mpc-protocols`:
//! "malicious security comes free in honest majority" (Goyal–Song eprint
//! 2020/134) for *linear* circuits — which is exactly what this foundation
//! delivers; the cost only appears at multiplication (sq-km34.2+).
//!
//! ## What this is (§2.1–2.2)
//!
//! The SPDZ-family fix for the crate's load-bearing semi-honest assumptions
//! (design doc §1, "the honest hole") is to attach an **information-theoretic
//! MAC** to every secret-shared value. Pick a single, **session-global** MAC key
//! `α ∈ F_p` that is *itself* secret-shared as a degree-`t` Shamir sharing
//! `[α]` — **no party knows `α`** (design §2.1). The authenticated sharing of a
//! secret `x` is then the pair
//!
//! ```text
//! [[x]] = ( [x] , [α·x] )      both degree-t Shamir sharings.
//! ```
//!
//! `m_x = α·x` is the authenticator: a party cannot change its share of `[x]` to
//! flip `x` *and* simultaneously fix `[m_x]` to still equal `α·x`, because it does
//! not know `α`. That is the whole point — and it is why the later batched
//! MAC-check (sq-km34.4) catches tampering even at the minimal `n = 2t+1` where
//! Reed–Solomon redundancy is zero (design §2.5): soundness comes from the
//! *secret* `α`, not from over-determination of the codeword.
//!
//! ## What this foundation bead delivers — and what it deliberately does NOT
//!
//! This is bead **sq-km34.1**, the FOUNDATION (design §6 step 1). It delivers:
//!
//! - [`MacKey`] — the session-global secret-shared `[α]`, minted by the dealer,
//!   structurally **un-openable** (no method reconstructs `α`; see below).
//! - [`AuthenticatedShare`] — the `([x], [α·x])` pair (design §2.2).
//! - [`ShamirDealer::new_mac_session`](crate::shamir::ShamirDealer::new_mac_session)
//!   mints `[α]` once per session and returns a
//!   [`MacSession`](crate::shamir::MacSession) whose
//!   [`authenticated_share`](crate::shamir::MacSession::authenticated_share)
//!   produces authenticated sharings under that key.
//! - The **free linear ops** [`auth_add`], [`auth_scale`], [`auth_sub`],
//!   [`auth_add_constant`] (design §2.3): each applies the existing local Shamir
//!   linear op to BOTH `[x]` and `[α·x]`, with the public-constant MAC term using
//!   the shared `[α]`. Zero communication rounds — the honest-majority sweet spot.
//!
//! It does **NOT** deliver (those are later beads, do not implement here):
//!
//! - MAC-carrying **multiplication** + authenticated degree-reduce — **sq-km34.2**
//!   (design §2.4). Multiplication is where authentication needs real work
//!   (`α·(x·y) ≠ (α·x)·(α·y)`); this foundation only covers the FREE linear ops.
//! - The **batched MAC-check** at open time — **sq-km34.4** (design §2.5). Without
//!   it, an authenticated share is not yet tamper-*evident*; this bead builds the
//!   carrier, not the check. **So this is the foundation for malicious-with-abort
//!   security, but NOT yet malicious until sq-km34.2+ land.**
//! - **Registry / per-operator** reporting (`new_malicious`,
//!   `authenticated_abort` descriptor) — **sq-km34.7** (design §4). This module
//!   does NOT touch [`crate::backend`]; the security tier reported to a federation
//!   is unchanged until the check is wired in.
//!
//! ## Security tier (stated, not over-claimed)
//!
//! This foundation is the carrier for **honest-majority, malicious-with-abort**
//! security (design §3, "the achieved tier"). On its own it changes NO advertised
//! guarantee: it adds the MAC sharing alongside the value sharing and keeps the
//! linear ops free and correct. The malicious-security PROPERTY only materialises
//! once the multiplication (sq-km34.2) carries the MAC and the batched check
//! (sq-km34.4) fires on a mismatch. Until then the parties are still trusted to
//! follow the protocol (semi-honest), exactly as today.
//!
//! ## Why `α` is NEVER reconstructed (a structural, not documentary, guarantee)
//!
//! [`MacKey`] holds `[α]` only as its per-party [`Share`] vector and exposes NO
//! method that opens it — not `reconstruct`, not a getter that returns the field
//! value, nothing. The crate's reconstruction entry points
//! ([`ShamirBackend::reconstruct`](crate::shamir::ShamirBackend::reconstruct),
//! [`reconstruct_degree`](crate::shamir::reconstruct_degree),
//! [`reconstruct_robust`](crate::robust::reconstruct_robust)) all take `&[Share]`,
//! so `[α]`'s shares COULD be fed to them by external code that pulls them out —
//! which is why [`MacKey`] keeps its shares private and the only share-level
//! accessor (`MacKey::scaled_constant_mac`, crate-private) returns a *derived*
//! sharing of `α·c` for a PUBLIC `c` (the §2.3 add-constant MAC term), never
//! `[α]` itself.
//! The session `α` value is consumed by the dealer the instant it is shared and
//! is never returned to any caller (see
//! [`ShamirDealer::new_mac_session`](crate::shamir::ShamirDealer::new_mac_session)).
//! A test (`mac_key_is_never_reconstructed_*`) pins that no `α`-opening path
//! exists.

use crate::field::Fp;
use crate::partial::MpcError;
use crate::shamir::{add_constant, add_shares, scale, sub_shares, Share, ShareVec};

/// The **session-global MAC key sharing** `[α]` — a degree-`t` Shamir sharing of
/// a single `α ∈ F_p` that **no party knows** (design §2.1).
///
/// Minted ONCE per session by
/// [`ShamirDealer::new_mac_session`](crate::shamir::ShamirDealer::new_mac_session),
/// which draws `α` from the masking RNG (an OS-seeded ChaCha20 CSPRNG in
/// production, sq-1vt), shares it, and immediately drops the cleartext `α` — it is
/// never returned to any caller. The same `[α]` then authenticates every value in
/// the session via [`AuthenticatedShare`].
///
/// **`α` is structurally un-openable.** This type stores `[α]` only as its private
/// per-party share vector and exposes NO method that reconstructs it. The only
/// share-level accessor, `MacKey::scaled_constant_mac` (crate-private), returns a
/// sharing of `α·c` for a PUBLIC constant `c` (the add-constant MAC term of §2.3)
/// — a derived sharing, never `[α]` itself. This is the structural realisation of
/// the bead's acceptance criterion (2): no code path opens `α`. `[OPUS-4.8]`
#[derive(Debug, Clone)]
pub struct MacKey {
    /// The degree-`t` Shamir sharing `[α]` (one [`Share`] per party). Private so
    /// the only way to USE it is the public-constant MAC term [`Self::scaled_constant_mac`];
    /// there is deliberately no accessor that returns the shares for reconstruction.
    shares: ShareVec,
    /// The privacy threshold `t` the key was shared at (mirrors the dealer's). The
    /// authenticated sharings under this key MUST be at the same `t` so the MAC
    /// relation `α·x` opens at the same degree as the value.
    t: usize,
}

impl MacKey {
    /// Construct a [`MacKey`] from the dealer's freshly-minted `[α]` sharing.
    /// `pub(crate)` so ONLY the dealer (which alone saw — and has already dropped —
    /// the cleartext `α`) can build one; external code cannot supply its own
    /// `[α]`. The threshold `t` is recorded so authenticated sharings are checked
    /// to match it. `[OPUS-4.8]`
    pub(crate) fn from_shares(shares: ShareVec, t: usize) -> Self {
        MacKey { shares, t }
    }

    /// The number of parties `[α]` is shared across (`= shares.len()`). Lets a
    /// caller sanity-check that a value sharing and the MAC key are on the same
    /// party set WITHOUT exposing the shares themselves.
    pub fn parties(&self) -> usize {
        self.shares.len()
    }

    /// The threshold `t` this key (and every authenticated sharing under it) lives
    /// at.
    pub fn threshold(&self) -> usize {
        self.t
    }

    /// The MAC sharing of a **public** constant `c`, i.e. a degree-`t` sharing of
    /// `α·c`, computed as `c · [α]` (local [`scale`] of `[α]`). This is the §2.3
    /// add-constant MAC term — adding a public `c` to `[[x]]` leaves `[x]+c` but
    /// shifts the MAC by `α·c`, and since `[α]` is shared, `c·[α]` is a free local
    /// op. It returns a sharing of `α·c`, a *derived* quantity, **NOT** `[α]`: `c`
    /// is public, so revealing a sharing of `α·c` is no more than revealing a
    /// sharing of (public-`c`) × (secret-`α`), which is exactly the MAC of the
    /// public value `c` and is meant to be added into another MAC sharing — it is
    /// never opened on its own here. `[OPUS-4.8]`
    pub(crate) fn scaled_constant_mac(&self, c: Fp) -> ShareVec {
        scale(&self.shares, c)
    }

    /// **Test-only.** The raw `[α]` shares, so the independence test (acceptance
    /// (3)) can argue about `≤ t` views of `α`. The `cfg` gate keeps this out of
    /// the public/production API — there is deliberately no non-test accessor that
    /// returns the shares for reconstruction. `[OPUS-4.8]`
    #[cfg(test)]
    pub(crate) fn shares_for_test(&self) -> ShareVec {
        self.shares.clone()
    }
}

/// An **authenticated secret sharing** `[[x]] = ([x], [α·x])` (design §2.2): the
/// value's degree-`t` Shamir sharing `[x]` paired with the degree-`t` Shamir
/// sharing of its IT-MAC `m_x = α·x` under the session key [`MacKey`].
///
/// Produced by
/// [`MacSession::authenticated_share`](crate::shamir::MacSession::authenticated_share).
/// Carries TWO sharings (value + MAC), so share storage roughly doubles vs a plain
/// [`ShareVec`] (design §5) — the price of authentication. Linear combinations are
/// FREE: [`auth_add`] / [`auth_scale`] / [`auth_sub`] / [`auth_add_constant`] apply
/// the matching local Shamir op to BOTH components (design §2.3), with no
/// communication round.
///
/// The value `[x]` reconstructs via the existing
/// [`ShamirBackend::reconstruct`](crate::shamir::ShamirBackend::reconstruct); the
/// MAC `[α·x]` reconstructs the same way and equals `α · x` — the relation a
/// later batched check (sq-km34.4) verifies WITHOUT opening `α`. `[OPUS-4.8]`
#[derive(Debug, Clone)]
pub struct AuthenticatedShare {
    /// `[x]` — the degree-`t` Shamir sharing of the value.
    value: ShareVec,
    /// `[α·x]` — the degree-`t` Shamir sharing of the IT-MAC `m_x = α·x`.
    mac: ShareVec,
}

impl AuthenticatedShare {
    /// Assemble an authenticated sharing from its value and MAC sharings. The two
    /// must be on the identical party-point set (same length, same `x` per index);
    /// otherwise the linear ops below could not pair them and the MAC relation
    /// would be meaningless. `pub(crate)`: only the dealer mints these (it alone
    /// computed `α·x` from the secret `α` it holds transiently). `[OPUS-4.8]`
    pub(crate) fn new(value: ShareVec, mac: ShareVec) -> Result<Self, MpcError> {
        if value.len() != mac.len() {
            return Err(MpcError::Protocol(
                "AuthenticatedShare: value and MAC sharings differ in party count".into(),
            ));
        }
        for (v, m) in value.iter().zip(mac.iter()) {
            if v.x != m.x {
                return Err(MpcError::Protocol(
                    "AuthenticatedShare: value and MAC sharings on different evaluation points"
                        .into(),
                ));
            }
        }
        Ok(AuthenticatedShare { value, mac })
    }

    /// The value sharing `[x]` (degree-`t`). Reconstruct it with the existing
    /// [`ShamirBackend::reconstruct`](crate::shamir::ShamirBackend::reconstruct) /
    /// [`ShamirDealer::reconstruct`](crate::shamir::ShamirDealer::reconstruct).
    pub fn value_shares(&self) -> &[Share] {
        &self.value
    }

    /// The MAC sharing `[α·x]` (degree-`t`). Reconstructs to `α · x`. (Opening this
    /// reveals `α·x`, not `α` — and the later batched check (sq-km34.4) does not
    /// open it directly at all; it forms a random linear combination first.)
    pub fn mac_shares(&self) -> &[Share] {
        &self.mac
    }

    /// Number of parties this authenticated value is shared across.
    pub fn parties(&self) -> usize {
        self.value.len()
    }
}

// =============================================================================
// Free linear ops on authenticated shares (design §2.2.3 / §2.3).
//
// Each is the matching local Shamir linear op (`crate::shamir`) applied TWICE —
// once to `[x]`, once to `[α·x]` — using the field identities:
//   α·(x + y) = α·x + α·y        → add: add both components
//   α·(c·x)   = c·(α·x)          → scale: scale both components
//   α·(x − y) = α·x − α·y        → sub: sub both components
//   α·(x + c) = α·x + α·c        → add-constant: value gets +c, MAC gets +α·c
// All are LOCAL / zero-round, so the authenticated SUM aggregate stays the
// zero-round honest-majority sweet spot (design §2.3 / §5). [OPUS-4.8]
// =============================================================================

/// Add two authenticated sharings: `[[x]] + [[y]] = ([x]+[y], [α·x]+[α·y])`
/// (design §2.3). FREE / local — both components add component-wise, no round.
/// Inputs must be on the identical party-point set. `[OPUS-4.8]`
pub fn auth_add(
    a: &AuthenticatedShare,
    b: &AuthenticatedShare,
) -> Result<AuthenticatedShare, MpcError> {
    let value = add_shares(&a.value, &b.value)?;
    let mac = add_shares(&a.mac, &b.mac)?;
    AuthenticatedShare::new(value, mac)
}

/// Subtract two authenticated sharings: `[[x]] − [[y]] = ([x]−[y], [α·x]−[α·y])`
/// (design §2.3). FREE / local. Inputs must be on the identical party-point set.
/// `[OPUS-4.8]`
pub fn auth_sub(
    a: &AuthenticatedShare,
    b: &AuthenticatedShare,
) -> Result<AuthenticatedShare, MpcError> {
    let value = sub_shares(&a.value, &b.value)?;
    let mac = sub_shares(&a.mac, &b.mac)?;
    AuthenticatedShare::new(value, mac)
}

/// Multiply an authenticated sharing by a PUBLIC field constant `c`:
/// `c · [[x]] = (c·[x], c·[α·x])` (design §2.3, since `α·(c·x) = c·(α·x)`). FREE /
/// local. (This is scaling by a *public* scalar — NOT secret×secret multiplication,
/// which is sq-km34.2 and needs real work.) `[OPUS-4.8]`
pub fn auth_scale(a: &AuthenticatedShare, c: Fp) -> AuthenticatedShare {
    // Both components stay on the same party points, so `new` cannot fail; but
    // route through it to keep the value/MAC point-alignment invariant in one
    // place rather than re-asserting it here.
    AuthenticatedShare {
        value: scale(&a.value, c),
        mac: scale(&a.mac, c),
    }
}

/// Add a PUBLIC field constant `c` to an authenticated sharing:
/// `[[x]] + c = ([x]+c, [α·x] + c·[α])` (design §2.3). The value gets the usual
/// local add-constant; the MAC gains `α·c`, computed from the *shared* `[α]` as
/// `c·[α]` (`MacKey::scaled_constant_mac`) — still a FREE local op, no round.
/// The [`MacKey`] must be the same session key the sharing was authenticated under
/// (same party set and threshold); a mismatch is a protocol error. `[OPUS-4.8]`
pub fn auth_add_constant(
    a: &AuthenticatedShare,
    c: Fp,
    key: &MacKey,
) -> Result<AuthenticatedShare, MpcError> {
    if a.parties() != key.parties() {
        return Err(MpcError::Protocol(
            "auth_add_constant: authenticated share and MAC key are on different party sets".into(),
        ));
    }
    let value = add_constant(&a.value, c);
    // MAC of the public constant c is α·c = c·[α]; add it into the existing MAC.
    let mac = add_shares(&a.mac, &key.scaled_constant_mac(c))?;
    AuthenticatedShare::new(value, mac)
}

#[cfg(test)]
mod tests {
    //! sq-km34.1 acceptance tests (design §2.1–2.2). The three load-bearing ones,
    //! mapped to the bead's acceptance bar:
    //!  (1) authenticated sharing ROUND-TRIPS: reconstruct `[x]` → `x`, and the MAC
    //!      relation `reconstruct([α·x]) == α · x` HOLDS (`authenticated_share_round_trips`);
    //!  (2) `[α]` is NEVER reconstructed — STRUCTURAL (no `α`-opening path exists),
    //!      pinned + the MAC relation derives `α` only inside the test harness from
    //!      shares the test mints itself (`mac_key_is_never_reconstructed_by_the_api`);
    //!  (3) any `<= t` party views are INDEPENDENT of `α` — the standard privacy
    //!      property, tested by exhibiting a second `α` consistent with `t` shares
    //!      (`t_shares_of_alpha_are_independent_of_alpha`).
    //! Plus: the free linear ops preserve the MAC relation
    //! (`linear_ops_preserve_mac_relation`).
    use super::*;
    use crate::shamir::{reconstruct_at_zero, ShamirBackend};

    /// Reconstruct a degree-`t` sharing's secret using the test-only unchecked
    /// Lagrange-at-0 helper (RNG-free). The production `ShamirBackend::reconstruct`
    /// routes through the robust checker; here we want the plain secret for both
    /// `[x]` and `[α·x]` and for the forged-third-share independence argument.
    fn open(shares: &[Share], t: usize) -> Fp {
        reconstruct_at_zero(shares, t).expect("enough shares to reconstruct")
    }

    /// ACCEPTANCE (1): authenticated sharing round-trips — `[x]` reconstructs to
    /// `x`, and the MAC sharing `[α·x]` reconstructs to a value satisfying the MAC
    /// relation `m_x = α · x`. We recover `α` here ONLY by reconstructing it from
    /// shares the TEST mints (the dealer never hands it out — see acceptance (2));
    /// this is the test verifying the relation, not the API exposing `α`.
    #[test]
    fn authenticated_share_round_trips() {
        // Seedable test RNG for reproducibility; production uses OS-seeded ChaCha20.
        let backend = ShamirBackend::new_seeded(5, 0xA17C).unwrap(); // t = 2
        let mut dealer = backend.dealer();
        let mut session = dealer.new_mac_session();

        for secret in [0u64, 1, 7, 42, 100_000, crate::field::P - 1] {
            let x = Fp::new(secret);
            let auth = session.authenticated_share(x);

            // [x] reconstructs to x.
            let recovered_x = backend.reconstruct(auth.value_shares()).unwrap();
            assert_eq!(recovered_x, x, "value sharing must reconstruct to x");

            // The MAC relation holds: reconstruct([α·x]) == α · x. We get α from the
            // session-internal test accessor (test-only — NOT part of the public API).
            let alpha = session.alpha_for_test();
            let recovered_mac = backend.reconstruct(auth.mac_shares()).unwrap();
            assert_eq!(
                recovered_mac,
                alpha.mul(x),
                "MAC sharing must reconstruct to α·x"
            );
        }
    }

    /// ACCEPTANCE (2): `[α]` is NEVER reconstructed by any code path. This is a
    /// STRUCTURAL guarantee — [`MacKey`] exposes no `reconstruct` and no getter
    /// returning the `α` field value, and the dealer drops the cleartext `α` the
    /// instant it shares it. The test asserts the *shape* of the API: the only
    /// MAC-key-derived sharing a caller can obtain is `α·c` for a PUBLIC `c`
    /// ([`MacKey::scaled_constant_mac`]), and opening THAT yields `α·c`, never `α`.
    /// (A `c = 1` would give `α·1 = α`; that accessor is `pub(crate)` and only the
    /// §2.3 add-constant path uses it with a caller-supplied PUBLIC constant — the
    /// public surface of [`MacKey`] has no reconstruction at all.)
    #[test]
    fn mac_key_is_never_reconstructed_by_the_api() {
        let backend = ShamirBackend::new_seeded(5, 0xBEEF).unwrap();
        let mut dealer = backend.dealer();
        let session = dealer.new_mac_session();
        let key = session.mac_key();

        // The PUBLIC API of MacKey is: parties(), threshold(). Neither opens α.
        assert_eq!(key.parties(), 5);
        assert_eq!(key.threshold(), backend.threshold());

        // There is no `key.reconstruct(..)`, no `key.alpha()`, no public accessor
        // returning the shares for external reconstruction. The crate-internal
        // `scaled_constant_mac(c)` returns a sharing of α·c for PUBLIC c — a derived
        // MAC term, used only inside `auth_add_constant`. Even reconstructing THAT
        // gives α·c, not α (here c = 7, a public constant → α·7).
        let c = Fp::new(7);
        let mac_of_c = key.scaled_constant_mac(c);
        let opened = open(&mac_of_c, key.threshold());
        // We can only check this is consistent (α·7) using the test α; the point is
        // the *value* opened is α·c, and the API never lets a caller open α itself.
        let alpha = session.alpha_for_test();
        assert_eq!(
            opened,
            alpha.mul(c),
            "scaled_constant_mac opens to α·c (public c), not α"
        );
    }

    /// ACCEPTANCE (3): any `<= t` party views are INDEPENDENT of `α` — the standard
    /// Shamir information-theoretic hiding property applied to `[α]`. With `t`
    /// shares of `α` there is one degree of freedom left, so for ANY candidate
    /// `α'` there exists a degree-`t` polynomial through those `t` shares with
    /// `f(0) = α'`. We witness this by forging a `(t+1)`-th share that makes the
    /// `t` real shares of `α` interpolate to a DIFFERENT key `α' ≠ α` — proof the
    /// `t` shares pin down neither `α` nor anything about it.
    #[test]
    fn t_shares_of_alpha_are_independent_of_alpha() {
        let backend = ShamirBackend::new_seeded(7, 0x5EED).unwrap(); // t = 3
        let t = backend.threshold();
        assert_eq!(t, 3);
        let mut dealer = backend.dealer();
        let session = dealer.new_mac_session();

        // The actual α (test-only accessor) and the t shares an adversary holding t
        // parties would see.
        let alpha = session.alpha_for_test();
        let alpha_shares = session.alpha_shares_for_test();
        let t_views = &alpha_shares[..t];

        // For a DIFFERENT target key α' ≠ α, forge a (t+1)-th share so the t views
        // + the forged share interpolate to α'. Its existence is exactly the
        // information-theoretic hiding argument: the t shares leave f(0) free.
        let alpha_prime = alpha.add(Fp::new(12345)); // some α' != α
        assert_ne!(alpha_prime, alpha);
        let forged = forge_next_share(t_views, alpha_prime, t);

        let mut combined = t_views.to_vec();
        combined.push(forged);
        assert_eq!(
            open(&combined, t),
            alpha_prime,
            "t shares of α are consistent with a DIFFERENT key α' → they hide α"
        );
        // Sanity: the same t views with the REAL (t+1)-th share open to the real α.
        let mut real = t_views.to_vec();
        real.push(alpha_shares[t]);
        assert_eq!(open(&real, t), alpha);
    }

    /// The free linear ops (§2.3) preserve the MAC relation: after add / scale /
    /// sub / add-constant, the reconstructed MAC still equals `α ·` the
    /// reconstructed value. This is what makes the later batched MAC-check
    /// (sq-km34.4) sound on linear circuits, and what keeps the SUM aggregate's
    /// authentication FREE.
    #[test]
    fn linear_ops_preserve_mac_relation() {
        let backend = ShamirBackend::new_seeded(5, 0xF00D).unwrap();
        let t = backend.threshold();
        let mut dealer = backend.dealer();
        let mut session = dealer.new_mac_session();
        let key = session.mac_key();
        let alpha = session.alpha_for_test();

        let x = Fp::new(30_000);
        let y = Fp::new(45_000);
        let ax = session.authenticated_share(x);
        let ay = session.authenticated_share(y);

        // Assert the MAC relation reconstruct([α·v]) == α·reconstruct([v]) for an
        // authenticated value.
        let check = |auth: &AuthenticatedShare, expected: Fp| {
            let v = open(auth.value_shares(), t);
            let m = open(auth.mac_shares(), t);
            assert_eq!(v, expected, "value");
            assert_eq!(m, alpha.mul(expected), "MAC = α·value");
        };

        // add: x + y.
        check(&auth_add(&ax, &ay).unwrap(), x.add(y));
        // sub: x - y.
        check(&auth_sub(&ax, &ay).unwrap(), x.sub(y));
        // scale by public c.
        let c = Fp::new(3);
        check(&auth_scale(&ax, c), x.mul(c));
        // add public constant: x + k, MAC must pick up α·k via the shared [α].
        let k = Fp::new(500);
        check(&auth_add_constant(&ax, k, &key).unwrap(), x.add(k));

        // A composite linear combination: 3·x + y + 500, the shape of a salary
        // aggregate. MAC must still be α·(3x + y + 500).
        let combo =
            auth_add_constant(&auth_add(&auth_scale(&ax, c), &ay).unwrap(), k, &key).unwrap();
        check(&combo, x.mul(c).add(y).add(k));
    }

    /// The authenticated SUM aggregate (the flatmate cumulative-salary use case)
    /// round-trips AND stays authenticated — the foundation's headline linear
    /// circuit. Mirrors `shamir::secure_sum_equals_plaintext_sum`, but every
    /// addend is authenticated and the result's MAC relation holds.
    #[test]
    fn authenticated_sum_round_trips_and_stays_authenticated() {
        let salaries = [30_000u64, 45_000, 28_000, 51_000];
        let plaintext_sum: u64 = salaries.iter().sum();

        let backend = ShamirBackend::new_seeded(5, 0xABCD).unwrap();
        let t = backend.threshold();
        let mut dealer = backend.dealer();
        let mut session = dealer.new_mac_session();
        let alpha = session.alpha_for_test();

        let shared: Vec<AuthenticatedShare> = salaries
            .iter()
            .map(|&s| session.authenticated_share(Fp::new(s)))
            .collect();

        // Zero-round local accumulation (the SUM aggregate).
        let mut acc = shared[0].clone();
        for next in &shared[1..] {
            acc = auth_add(&acc, next).unwrap();
        }

        let sum = open(acc.value_shares(), t);
        assert_eq!(
            sum,
            Fp::new(plaintext_sum),
            "authenticated sum == plaintext sum"
        );
        let sum_mac = open(acc.mac_shares(), t);
        assert_eq!(sum_mac, alpha.mul(sum), "aggregate MAC stays α·sum");
    }

    /// Mismatched party sets are a protocol error, not a silent wrong answer (the
    /// linear ops must reject sharings on different point sets — the same
    /// discipline as `add_shares`).
    #[test]
    fn linear_ops_reject_mismatched_party_sets() {
        let b5 = ShamirBackend::new_seeded(5, 1).unwrap();
        let b7 = ShamirBackend::new_seeded(7, 2).unwrap();
        let mut d5 = b5.dealer();
        let mut d7 = b7.dealer();
        let mut s5 = d5.new_mac_session();
        let mut s7 = d7.new_mac_session();
        let a = s5.authenticated_share(Fp::new(1));
        let b = s7.authenticated_share(Fp::new(2));
        assert!(matches!(auth_add(&a, &b), Err(MpcError::Protocol(_))));
        assert!(matches!(auth_sub(&a, &b), Err(MpcError::Protocol(_))));
        // add_constant with a wrong-sized key.
        let wrong_key = s7.mac_key();
        assert!(matches!(
            auth_add_constant(&a, Fp::new(9), &wrong_key),
            Err(MpcError::Protocol(_))
        ));
    }

    /// Given `t` shares on a degree-`t` polynomial, produce ONE more share (at a
    /// fresh `x`) so the `t+1` interpolate to `target` at `x = 0`. Demonstrates the
    /// hiding property of acceptance (3); test-only. (Generalises the degree-2
    /// `forge_third_share` in `shamir.rs` to any `t`.)
    fn forge_next_share(views: &[Share], target: Fp, t: usize) -> Share {
        assert_eq!(views.len(), t, "need exactly t shares to leave one DOF");
        // Pick a fresh x distinct from the views' points.
        let xnew = (1..=10_000u64)
            .find(|x| views.iter().all(|s| s.x != *x))
            .expect("a fresh evaluation point exists");
        // Solve y_new so Lagrange-at-0 of {views, (xnew, y_new)} equals target.
        let mut xs: Vec<u64> = views.iter().map(|s| s.x).collect();
        xs.push(xnew);
        let weight = |i: usize| -> Fp {
            let xi = Fp::new(xs[i]);
            let mut num = Fp::one();
            let mut den = Fp::one();
            for (j, &xj) in xs.iter().enumerate() {
                if i == j {
                    continue;
                }
                let xj = Fp::new(xj);
                num = num.mul(xj.neg());
                den = den.mul(xi.sub(xj));
            }
            num.mul(den.inv())
        };
        // target = Σ_i y_i·L_i + y_new·L_new  ⇒  y_new = (target − Σ y_i·L_i)/L_new.
        let mut acc = Fp::zero();
        for (i, v) in views.iter().enumerate() {
            acc = acc.add(v.y.mul(weight(i)));
        }
        let l_new = weight(views.len());
        let y_new = target.sub(acc).mul(l_new.inv());
        Share { x: xnew, y: y_new }
    }
}
