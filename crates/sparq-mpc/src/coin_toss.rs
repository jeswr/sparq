// [OPUS-5] sq-yyro follow-on (issue #3532): the distributed COIN-TOSS
// correlated-randomness SOURCE behind `crate::randomness::DistributedRandomness`
// — commit-then-share joint randomness summed over every party's OWN
// contribution. Setup-free and any-`n` (the fallback for the `n` PRSS refuses),
// interactive (ONE commit-open round per batch). Design:
// research/mpc-distributed-randomness-design.md §2.2 / §2.3 / §5 item 2.
//! **Distributed coin-toss** — commit-then-share joint randomness.
//!
//! The second dealer-less generator behind the [`crate::randomness`] seam, and
//! the design record's **general-`n` / setup-free fallback** to
//! [`crate::prss`]: it needs no replicated-seed setup at all, so it works for
//! every `n ≥ 2` — including the `n ≥ 10` PRSS refuses fail-closed
//! ([`crate::prss::MAX_PRSS_SEEDS`]) — at the cost of being **interactive**
//! (`research/mpc-distributed-randomness-design.md` §2.2 / §2.3 / §5 item 2).
//!
//! ## The construction
//!
//! Parties sit on the canonical Shamir points `x = 1..=n`; the honest-majority
//! threshold is `t = ⌊(n−1)/2⌋`, matching [`crate::shamir::ShamirBackend`].
//! One **commit-open round** produces a whole batch of elements:
//!
//! 1. **Commit.** Every party `i` draws its own contribution `r_i` (and a hiding
//!    nonce) from **its own** randomness and broadcasts
//!    `C_i = H(DOMAIN ‖ ctr ‖ i ‖ nonce_i ‖ r_i)`. Nothing about any `r_i` is
//!    revealed, and — the point of this phase — every contribution is *fixed*
//!    before any party has seen anything from the others.
//! 2. **Open / share.** Each party then shares the contribution it committed to
//!    on a fresh degree-`t` polynomial drawn from its own randomness, sending
//!    party `j` the point `x_j`.
//! 3. **Local sum (free).** Party `j` adds the `n` shares it received:
//!    `[r]_j = Σ_i [r_i]_j`. Sharing is linear, so that is a degree-`t` sharing
//!    of
//!
//!    ```text
//!    r  =  Σ_{i=1..n} r_i
//!    ```
//!
//! This is the design record's **VSS-summed** variant of §2.2 rather than the
//! open-in-the-clear variant: the trait's masks must stay *secret*, so the
//! contributions are secret-shared and summed, never opened. The commitment
//! round is what makes the contributions **binding before anything moves** —
//! the biasing-resistance §1 asks of a joint generator.
//!
//! ## Why no `≤ t` parties know `r`
//!
//! Take any set `T` of `≤ t` parties. `T` misses at least one honest party `j`,
//! and `T`'s whole view of `r_j` is `t` shares of a degree-`t` polynomial — which
//! is independent of `r_j`, information-theoretically (the Shamir threshold
//! property [`crate::shamir`] already rests on). So `r = Σ_i r_i` is uniform
//! given `T`'s view, because the summand `r_j` is. Unlike PRSS this argument
//! needs **no PRF assumption**: the protocol adds nothing beyond the local
//! CSPRNG the crate already relies on for `r_j` itself (sq-1vt), where PRSS's
//! secrecy additionally rests on the pseudorandomness of `PRF(k_A, ctr)`
//! (`prss` module docs, design §2.1 cons 2).
//!
//! ## The cost: interaction (this is the whole trade)
//!
//! PRSS pays a combinatorial setup and generates with **0 rounds**; the
//! coin-toss pays **no setup** and generates with **≥ 1 round per batch**
//! (design §2.2 / §2.3). Batch to amortise: [`CoinTossRandomness::shared_masks`]
//! produces `count` elements in ONE round, and [`CoinTossRandomness::rounds`]
//! reports the rounds actually executed, so the round cost is a measurable
//! number rather than a comment. On a WAN that number is the tax; on a LAN
//! either source works (design §6 Q2).
//!
//! ## What this module does and does NOT establish (read before claiming anything)
//!
//! - **Each party's contribution really is its own.** A party's `r_i`, its
//!   nonce, and its sharing polynomial all come from a `PartyState` carrying
//!   that party's own [`crate::shamir::ShamirDealer`] over its own CSPRNG; the
//!   commit phase touches only that state, so no contribution can be a function
//!   of another party's. `new` mints an independently OS-seeded generator per
//!   party (`new_seeded`, test/bench-gated, deliberately derives a *distinct*
//!   seed per party — one shared seed would make every "independent"
//!   contribution identical, which is the bug
//!   `each_party_draws_from_its_own_randomness` pins).
//! - **The commitment check is performed IN THE CLEAR — a simulation
//!   artefact.** `check_openings` recomputes each party's commitment from its
//!   opening, so a value that changed between the commit and open phases is
//!   caught fail-closed. That is a real check of the round *ordering* this code
//!   depends on, but a real deployment **cannot run it**: `r_i` is never opened
//!   publicly (it must stay secret), so binding the *sharing* to the commitment
//!   there needs a committed VSS (Feldman/Pedersen) — design §3 / §5 item 3,
//!   still a follow-on bead. Until then a *malicious* party can share a value
//!   other than the one it committed to and nothing detects it.
//! - **Honest-majority, semi-honest only.** Nothing here detects a party that
//!   emits a wrong share, aborts selectively after the barrier, or shares
//!   off-commitment; and the `r ≠ 0` guarantee below leans on a simulation
//!   artefact. **No malicious-tier property is claimed.**
//! - **The crate is an in-process SIMULATION.** All `n` parties are objects in
//!   one process, which therefore holds every opening. What is demonstrated is
//!   the *structure* of the protocol (which state a phase may touch, that the
//!   result is a correct degree-`t` sharing of the sum of all `n` contributions),
//!   never process, network, or scheduling isolation — in particular the
//!   "commit before anyone sees anything" barrier is enforced by this module's
//!   phase ordering, not by a network.
//! - **Not deployable, and this module does not change that.**
//!   [`RandomnessModel::HonestMajorityCoinToss`] reports
//!   [`deployable() == false`](RandomnessModel::deployable), so
//!   [`DistributedRandomness::require_deployable`] still refuses this source
//!   fail-closed. Acceptance must be tied to a *validated* construction behind
//!   the boundary of design-record §5 item 5, and the crate remains
//!   research-grade and **externally unaudited (`sq-qhy4` pending)**.
//!
//! ## `r ≠ 0` for the equality mask — the same honest gap as PRSS
//!
//! [`DistributedRandomness::shared_nonzero_mask`]'s structural contract is
//! `r ≠ 0` (a forced `r = 0` makes `secure_equal` report "equal" for every pair
//! — see the [`crate::randomness`] module note). The sum of uniform
//! contributions is uniform, so `r = 0` happens with probability `1/p ≈ 2^−61`
//! and the honest fix is to detect it and redraw in a fresh round. But **no
//! party can evaluate `r`** (that is the whole point), so a real deployment must
//! run a *distributed zero-test* on `[r]` — design §1 part 1 / §5 item 4, still
//! a follow-on bead. This module therefore does the rejection **centrally**,
//! using the simulation's access to every opening. That check is a **SIMULATION
//! ARTEFACT** and is why `shared_nonzero_mask` stays semi-honest-only here.

use crate::field::Fp;
use crate::partial::MpcError;
use crate::randomness::{accept_if_nonzero, DistributedRandomness, RandomnessModel};
use crate::shamir::{ShamirBackend, ShamirDealer, Share, ShareVec};
use sha2::{Digest, Sha512};
use zeroize::{Zeroize, Zeroizing};

/// Domain-separation tag for the contribution commitment. Versioned so a future
/// commitment change is a distinct, non-colliding derivation rather than a
/// silent re-interpretation of the same transcript.
const COMMIT_DOMAIN: &[u8] = b"sparq-mpc/cointoss/v1/commit";

/// A party's public commitment to one contribution: the first 32 bytes of the
/// domain-separated SHA-512 digest over `(ctr, party, nonce, value)`.
///
/// Public by construction (it is broadcast), so it is compared with a plain
/// `==`; there is no secret to leak through a timing difference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Commitment([u8; 32]);

/// The secret half of a commitment — what a party would reveal to open it.
///
/// Never leaves the simulation: in the shared variant `value` is secret-shared,
/// not broadcast (see the module note on [`check_openings`]).
struct Opening {
    /// Party `i`'s contribution `r_i`. **Secret** — it is one summand of `r`.
    value: Fp,
    /// The hiding randomiser, four uniform `F_p` draws (~244 bits of entropy)
    /// carried as their canonical little-endian words. Without it a commitment
    /// to a low-entropy field element would be brute-forceable.
    nonce: Zeroizing<[u64; 4]>,
}

impl Drop for Opening {
    fn drop(&mut self) {
        // The nonce is scrubbed by `Zeroizing`; scrub the contribution too — it
        // is the secret summand the whole construction protects.
        self.value.zeroize();
    }
}

/// Everything party `i` produced in the commit phase of one round: the public
/// commitments it broadcast, and the private openings it keeps.
struct PartyContribution {
    /// The Shamir point `x_i` this contribution belongs to.
    party: u64,
    /// One commitment per batch element, broadcast at the end of the commit
    /// phase.
    commitments: Vec<Commitment>,
    /// The matching openings, positionally aligned with
    /// [`Self::commitments`] — **private to the party**.
    openings: Vec<Opening>,
}

/// One party's entire state: its point, and its **own** live randomness.
///
/// The per-party [`ShamirDealer`] is what makes "each party contributes its own
/// randomness" structural rather than asserted — a party's contribution, nonce,
/// and sharing polynomial can only come from this object, and each party gets an
/// independently seeded one. (The dealer here deals exactly one value: *its own*
/// contribution. That is the dealer-less shape — no party deals on anyone else's
/// behalf, which is exactly what [`RandomnessModel::TrustedDealerSim`] *does*
/// do, and why that model is a simulation.)
struct PartyState {
    party: u64,
    dealer: ShamirDealer,
}

impl PartyState {
    /// **Commit phase.** Draw `count` fresh contributions from this party's own
    /// randomness and commit to each. Touches no other party's state, so a
    /// contribution cannot depend on another party's.
    fn commit(&mut self, base_ctr: u64, count: usize) -> PartyContribution {
        let mut commitments = Vec::with_capacity(count);
        let mut openings = Vec::with_capacity(count);
        for k in 0..count {
            let value = self.dealer.draw_fp();
            let nonce = Zeroizing::new([
                self.dealer.draw_fp().value(),
                self.dealer.draw_fp().value(),
                self.dealer.draw_fp().value(),
                self.dealer.draw_fp().value(),
            ]);
            commitments.push(commit_to(base_ctr + k as u64, self.party, &nonce, value));
            openings.push(Opening { value, nonce });
        }
        PartyContribution {
            party: self.party,
            commitments,
            openings,
        }
    }

    /// **Open / share phase.** Secret-share each committed contribution on a
    /// fresh degree-`t` polynomial whose free coefficients come from this
    /// party's own randomness. Returns one full `n`-share sharing per batch
    /// element.
    fn share_committed(&mut self, contribution: &PartyContribution) -> Vec<ShareVec> {
        contribution
            .openings
            .iter()
            .map(|opening| self.dealer.share(opening.value))
            .collect()
    }
}

/// `H(DOMAIN ‖ ctr ‖ party ‖ nonce ‖ value)`, truncated to 32 bytes.
///
/// Every input is fixed-length, so no two distinct `(ctr, party, nonce, value)`
/// tuples share a preimage by concatenation ambiguity. Including `ctr` and
/// `party` binds a commitment to the round and the committer, so one round's
/// transcript cannot be replayed as another's (or another party's).
fn commit_to(ctr: u64, party: u64, nonce: &[u64; 4], value: Fp) -> Commitment {
    let mut hasher = Sha512::new();
    hasher.update(COMMIT_DOMAIN);
    hasher.update(ctr.to_le_bytes());
    hasher.update(party.to_le_bytes());
    for word in nonce {
        hasher.update(word.to_le_bytes());
    }
    hasher.update(value.value().to_le_bytes());
    let mut digest = hasher.finalize();

    let mut out = [0u8; 32];
    out.copy_from_slice(&digest[..32]);
    // The tail of the digest is derived from secret material; scrub the buffer
    // we own. (`out` itself is the public commitment.)
    digest[..].zeroize();
    Commitment(out)
}

/// Check that party `contribution.party` is opening the values it **committed
/// to** at this round's counters — fail-closed on any mismatch.
///
/// **This runs in the clear, which is a SIMULATION ARTEFACT.** In the shared
/// variant `r_i` is never broadcast (it must stay secret), so a real federation
/// cannot recompute `C_i` from an opening; binding the *sharing* to the
/// commitment there requires a committed VSS (Feldman/Pedersen) — design record
/// §3 / §5 item 3, a follow-on bead. What this genuinely establishes *here* is
/// the round-ordering property the construction rests on: the value that enters
/// the open/share phase is the value fixed in the commit phase, so no
/// contribution can be re-drawn after the barrier.
fn check_openings(base_ctr: u64, contribution: &PartyContribution) -> Result<(), MpcError> {
    if contribution.commitments.len() != contribution.openings.len() {
        return Err(MpcError::Protocol(format!(
            "coin-toss party {} opened {} contribution(s) against {} commitment(s)",
            contribution.party,
            contribution.openings.len(),
            contribution.commitments.len()
        )));
    }
    for (k, (commitment, opening)) in contribution
        .commitments
        .iter()
        .zip(&contribution.openings)
        .enumerate()
    {
        let recomputed = commit_to(
            base_ctr + k as u64,
            contribution.party,
            &opening.nonce,
            opening.value,
        );
        if recomputed != *commitment {
            return Err(MpcError::Protocol(format!(
                "coin-toss commitment mismatch: party {} opened element {k} to a value it did \
                 not commit to — aborting the round rather than folding an unbound contribution \
                 into the joint randomness",
                contribution.party
            )));
        }
    }
    Ok(())
}

/// Party `party`'s **local** phase-3 step: add the shares it received, one from
/// each party's sharing.
///
/// Fail-closed on a share addressed to a different point — a party may only sum
/// what was sent to *it*, and a mis-addressed share would silently produce a
/// point off the summed polynomial (so every quorum would then reconstruct a
/// different value).
fn local_sum(party: u64, received: &[Share]) -> Result<Share, MpcError> {
    let mut y = Fp::zero();
    for share in received {
        if share.x != party {
            return Err(MpcError::Protocol(format!(
                "coin-toss party {party} received a share addressed to point {} — refusing to \
                 sum a share that is not on this party's evaluation point",
                share.x
            )));
        }
        y = y.add(share.y);
    }
    Ok(Share { x: party, y })
}

/// One completed commit-open round: the sharings it produced, and — for the
/// simulation only — the values they share.
struct CoinTossRound {
    /// `values[k] = Σ_i r_i^{(k)}`, the joint randomness of batch element `k`.
    ///
    /// **SIMULATION-ONLY oracle.** This is exactly what no `≤ t` parties can
    /// compute; it is available here only because this in-process simulation
    /// holds every party's opening. It backs the `r ≠ 0` rejection (see the
    /// module note on that gap) and the tests, and is deliberately not reachable
    /// from the public API.
    values: Vec<Fp>,
    /// `sharings[k]` is the full `n`-share degree-`t` sharing of `values[k]`.
    sharings: Vec<ShareVec>,
}

/// A **distributed coin-toss** correlated-randomness source: commit-then-share
/// joint randomness summed over all `n` parties' own contributions, reporting
/// [`RandomnessModel::HonestMajorityCoinToss`].
///
/// **Setup-free** — there is nothing to distribute before the first round, so
/// this works for every `n ≥ 2`, which is why the design record makes it the
/// fallback where PRSS's `C(n, t)` seed count is impractical. **Interactive** —
/// one commit-open round per batch; see [`Self::shared_masks`] to amortise and
/// [`Self::rounds`] to count.
///
/// See the module docs for the construction, the secrecy argument, and —
/// importantly — the things this does NOT establish (a *malicious*-tier
/// commitment binding, process isolation, deployability). This source is still
/// refused by [`require_deployable`](DistributedRandomness::require_deployable):
/// the crate is research-grade and externally unaudited (`sq-qhy4` pending).
///
/// Not `Clone`: cloning would duplicate every party's live CSPRNG state and
/// re-issue the same contributions under two apparently-independent sources.
pub struct CoinTossRandomness {
    n: usize,
    t: usize,
    /// One entry per party, on the canonical points `x = 1..=n` in order.
    parties: Vec<PartyState>,
    /// The commitment counter for the NEXT batch element. Strictly monotone: it
    /// domain-separates each element's commitment, so a round's transcript can
    /// never be replayed as a later round's.
    counter: u64,
    /// How many commit-open rounds this source has executed — the interaction
    /// cost, reported by [`Self::rounds`].
    rounds: u64,
}

impl std::fmt::Debug for CoinTossRandomness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never print party randomness state.
        f.debug_struct("CoinTossRandomness")
            .field("n", &self.n)
            .field("t", &self.t)
            .field(
                "parties",
                &format_args!("{} × <party-local dealer>", self.parties.len()),
            )
            .field("counter", &self.counter)
            .field("rounds", &self.rounds)
            .finish()
    }
}

impl CoinTossRandomness {
    /// Build a coin-toss source for `n` parties. **No setup runs** — that is the
    /// point of this source — so the only cost is minting each party's own
    /// OS-seeded ChaCha20 CSPRNG (via [`crate::shamir::ShamirBackend::dealer`],
    /// one independent generator per party).
    ///
    /// The threshold is the honest-majority `t = ⌊(n−1)/2⌋`, identical to
    /// [`crate::shamir::ShamirBackend`], so the sharings this produces
    /// reconstruct through that backend unchanged.
    ///
    /// # Errors
    ///
    /// [`MpcError::Protocol`] if `n < 2` (matching `ShamirBackend::new`). There
    /// is **no upper bound**: unlike [`crate::prss::PrssRandomness`] this source
    /// has no combinatorial setup to refuse, which is exactly the fallback role
    /// the design record assigns it (§2.2 / §2.3).
    ///
    /// Note the degenerate `n = 2` case: `t = 0`, so every sharing is the
    /// constant polynomial and every party's share equals `r`. That is the
    /// correct `t = 0` behaviour (a threshold of `0` promises secrecy against no
    /// one) and mirrors `ShamirDealer::share` at `t = 0`; it is not a privacy
    /// regime anyone should deploy.
    pub fn new(n: usize) -> Result<Self, MpcError> {
        let backend = ShamirBackend::new(n)?;
        let parties = (1..=n as u64)
            .map(|party| PartyState {
                party,
                // A FRESH OS-seeded CSPRNG per party — the parties' randomness
                // must be independent, so they must not share a generator.
                dealer: backend.dealer(),
            })
            .collect();
        Ok(CoinTossRandomness {
            n,
            t: backend.threshold(),
            parties,
            counter: 0,
            rounds: 0,
        })
    }

    /// Build a coin-toss source whose parties draw from the deterministic,
    /// seedable test PRNG — **for reproducible tests / benchmarks ONLY**, gated
    /// behind `#[cfg(any(test, feature = "insecure-test-rng"))]` exactly like
    /// `ShamirBackend::new_seeded`. Every contribution it produces is
    /// predictable, so every secrecy property of the module docs is void; the
    /// protocol itself is unchanged.
    ///
    /// Each party is seeded **distinctly** (derived from `seed` and the party
    /// point): one shared seed would make every party's supposedly independent
    /// contribution identical, collapsing `r = Σ r_i` to `n · r_1`.
    ///
    /// # Errors
    ///
    /// [`MpcError::Protocol`] if `n < 2`.
    #[cfg(any(test, feature = "insecure-test-rng"))]
    pub fn new_seeded(n: usize, seed: u64) -> Result<Self, MpcError> {
        // Validates `n >= 2` and fixes `t` before any party is built.
        let t = ShamirBackend::new_seeded(n, seed)?.threshold();
        let parties = (1..=n as u64)
            .map(|party| {
                Ok(PartyState {
                    party,
                    dealer: ShamirBackend::new_seeded(n, party_seed(seed, party))?.dealer(),
                })
            })
            .collect::<Result<Vec<_>, MpcError>>()?;
        Ok(CoinTossRandomness {
            n,
            t,
            parties,
            counter: 0,
            rounds: 0,
        })
    }

    /// Number of parties `n`.
    pub fn parties(&self) -> usize {
        self.n
    }

    /// The honest-majority privacy threshold `t = ⌊(n−1)/2⌋`.
    pub fn threshold(&self) -> usize {
        self.t
    }

    /// How many **commit-open rounds** this source has executed so far.
    ///
    /// This is the coin-toss's defining cost (design §2.2 / §2.3): PRSS
    /// generates with `0` rounds after its setup, this source pays one round per
    /// batch. Batching through [`Self::shared_masks`] is what amortises it —
    /// `k` elements in one call cost one round, `k` separate
    /// [`shared_mask`](DistributedRandomness::shared_mask) calls cost `k`.
    pub fn rounds(&self) -> u64 {
        self.rounds
    }

    /// Generate `count` fresh degree-`t` sharings in a **single** commit-open
    /// round — the batched form the interaction cost is amortised over.
    ///
    /// Each returned sharing is a full `n`-share degree-`t` sharing of a uniform
    /// value that no `≤ t` parties know (module docs); the sharings are mutually
    /// independent (every element gets its own contributions and its own masking
    /// polynomials). An empty batch is a no-op and consumes **no** round.
    ///
    /// # Errors
    ///
    /// - [`MpcError::Protocol`] if a party opens a contribution it did not
    ///   commit to (`check_openings`) or a share is mis-addressed (`local_sum`)
    ///   — the round aborts fail-closed rather than folding an unbound value
    ///   into the joint randomness.
    /// - [`MpcError::Protocol`] if the commitment counter would wrap.
    pub fn shared_masks(&mut self, count: usize) -> Result<Vec<ShareVec>, MpcError> {
        Ok(self.commit_open_round(count)?.sharings)
    }

    /// Run one commit-open round for `count` batch elements, returning the
    /// sharings together with the simulation-only value oracle.
    fn commit_open_round(&mut self, count: usize) -> Result<CoinTossRound, MpcError> {
        if count == 0 {
            // Nothing to agree on: no commitments, no round, no counter burned.
            return Ok(CoinTossRound {
                values: Vec::new(),
                sharings: Vec::new(),
            });
        }
        let base_ctr = self.reserve_counters(count)?;

        // ── Phase 1: COMMIT. Every party draws from its OWN randomness and
        //    broadcasts commitments. This loop touches one party's state at a
        //    time, so no contribution can be a function of another party's.
        let mut contributions = Vec::with_capacity(self.n);
        for party in &mut self.parties {
            contributions.push(party.commit(base_ctr, count));
        }

        // ── Barrier: every commitment is now public and binding. Only past this
        //    point may a value move.

        // ── Phase 2: OPEN / SHARE. Each party shares the contributions it
        //    committed to; a party opening something else aborts the round.
        let mut sharings = Vec::with_capacity(self.n);
        for (party, contribution) in self.parties.iter_mut().zip(&contributions) {
            check_openings(base_ctr, contribution)?;
            sharings.push(party.share_committed(contribution));
        }
        self.rounds += 1;

        // ── Phase 3: LOCAL SUM (free, zero rounds). Party `j` adds the `n`
        //    shares addressed to it, one per party's sharing.
        let summed = (0..count)
            .map(|k| {
                (1..=self.n as u64)
                    .map(|party| {
                        let received: Vec<Share> = sharings
                            .iter()
                            .map(|per_party| per_party[k][party as usize - 1])
                            .collect();
                        local_sum(party, &received)
                    })
                    .collect::<Result<ShareVec, MpcError>>()
            })
            .collect::<Result<Vec<ShareVec>, MpcError>>()?;

        // The simulation-only oracle: `r = Σ_i r_i`, over the openings only the
        // simulation holds. No party can compute this.
        let values = (0..count)
            .map(|k| {
                contributions
                    .iter()
                    .fold(Fp::zero(), |acc, c| acc.add(c.openings[k].value))
            })
            .collect();

        Ok(CoinTossRound {
            values,
            sharings: summed,
        })
    }

    /// Reserve `count` consecutive commitment counters, refusing fail-closed at
    /// exhaustion.
    ///
    /// Counters MUST be strictly monotone: a repeated counter would let one
    /// round's commitment transcript be replayed as another's. `u64` exhaustion
    /// is unreachable in practice, but wrapping would be a silent soundness
    /// break, so it is an error instead.
    fn reserve_counters(&mut self, count: usize) -> Result<u64, MpcError> {
        let base = self.counter;
        let advanced = u64::try_from(count)
            .ok()
            .and_then(|c| base.checked_add(c))
            .ok_or_else(|| {
                MpcError::Protocol(
                    "coin-toss commitment counter exhausted (2^64 elements) — refusing to wrap, \
                     since a reused counter would let one round's commitments be replayed as \
                     another's. Re-run with a fresh source."
                        .into(),
                )
            })?;
        self.counter = advanced;
        Ok(base)
    }
}

/// A distinct per-party seed for the test-only constructor, so the parties'
/// deterministic PRNGs are not all the same stream.
#[cfg(any(test, feature = "insecure-test-rng"))]
fn party_seed(seed: u64, party: u64) -> u64 {
    seed ^ party.wrapping_mul(0x9E37_79B9_7F4A_7C15)
}

/// The distributed coin-toss behind the dealer-less seam.
///
/// Reports [`RandomnessModel::HonestMajorityCoinToss`] — an honest *description*
/// of the regime, and still **not** a deployment credential:
/// `HonestMajorityCoinToss.deployable()` remains `false`, so
/// [`require_deployable`](DistributedRandomness::require_deployable) refuses this
/// source exactly as it refuses PRSS and the single-dealer simulation.
/// Acceptance is tied to a validated construction behind the boundary of
/// `research/mpc-distributed-randomness-design.md` §5 item 5, and the crate is
/// research-grade and externally unaudited (`sq-qhy4` pending).
impl DistributedRandomness for CoinTossRandomness {
    fn randomness_model(&self) -> RandomnessModel {
        RandomnessModel::HonestMajorityCoinToss
    }

    /// One fresh degree-`t` sharing of `r = Σ_i r_i`, costing **one commit-open
    /// round**. Use [`CoinTossRandomness::shared_masks`] to amortise that round
    /// over a batch. The simulation-only value oracle is discarded here.
    fn shared_mask(&mut self) -> Result<Vec<Share>, MpcError> {
        let mut round = self.commit_open_round(1)?;
        round
            .sharings
            .pop()
            .ok_or_else(|| MpcError::Protocol("coin-toss round produced no sharing".into()))
    }

    /// As [`shared_mask`](Self::shared_mask), redrawing in a fresh round on the
    /// `1/p ≈ 2^−61` chance that the joint value is `0` (the equality mask must
    /// be nonzero — [`crate::randomness`] module note).
    ///
    /// **The rejection test is a SIMULATION ARTEFACT.** It evaluates
    /// `r = Σ_i r_i` from every party's opening, which is possible only because
    /// this in-process simulation holds them all; no party can do it. A real
    /// deployment must replace it with a distributed zero-test on `[r]` whose
    /// opens are authenticated (design record §1 part 1, §5 item 4). Until then
    /// this method — like every `shared_nonzero_mask` — is **semi-honest only**
    /// and makes no active-adversary claim.
    fn shared_nonzero_mask(&mut self) -> Result<Vec<Share>, MpcError> {
        loop {
            let mut round = self.commit_open_round(1)?;
            let value = round
                .values
                .pop()
                .ok_or_else(|| MpcError::Protocol("coin-toss round produced no value".into()))?;
            let shares = round
                .sharings
                .pop()
                .ok_or_else(|| MpcError::Protocol("coin-toss round produced no sharing".into()))?;
            if let Some(accepted) = accept_if_nonzero(value, shares) {
                return Ok(accepted);
            }
        }
    }

    /// **Honestly refused.** The coin-toss is a *correlated-randomness*
    /// protocol: it produces a sharing of a value nobody chose, which is the
    /// wrong shape for "a holder shares the secret it already has". The sharing
    /// machinery is here, but the **verification** is not — dealer-less VSS
    /// needs a committed sharing (Feldman/Pedersen) or pairwise consistency
    /// checks so a malicious holder cannot distribute an inconsistent
    /// polynomial, composing with [`crate::robust`] on the output side (design
    /// record §3 / §5 item 3, a distinct follow-on bead). Returning an
    /// *unverified* plain sharing from a source labelled dealer-less would
    /// silently drop exactly the guarantee the caller would be reaching for, so
    /// this returns [`MpcError::NotYetImplemented`] instead.
    fn vss_own_input(&mut self, _secret: Fp) -> Result<Vec<Share>, MpcError> {
        Err(MpcError::not_yet(
            "dealer-less VSS input sharing (the coin-toss generates correlated randomness; it \
             has no verified input-sharing path)",
            "sq-yyro follow-on: research/mpc-distributed-randomness-design.md §3 / §5 item 3",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn coin_toss(n: usize) -> CoinTossRandomness {
        CoinTossRandomness::new_seeded(n, 0xC0FFEE).expect("n >= 2")
    }

    #[test]
    fn reports_the_coin_toss_model_but_is_still_not_deployable() {
        let source = coin_toss(5);
        let model = source.randomness_model();
        assert_eq!(model, RandomnessModel::HonestMajorityCoinToss);
        assert!(model.is_dealer_less(), "coin-toss is dealer-less BY DESIGN");
        // The honesty line: a real generator landing does NOT by itself make the
        // source deployment-acceptable. The label is descriptive; acceptance is
        // tied to a validated construction (design §5 item 5) and sq-qhy4 is
        // still pending, so the fail-closed gate must still refuse.
        assert!(!model.deployable());
        assert!(matches!(
            source.require_deployable(),
            Err(MpcError::NoBackendSatisfies { .. })
        ));
    }

    #[test]
    fn is_setup_free_for_every_n_including_the_ones_prss_refuses() {
        // The fallback role (design §2.2 / §2.3): PRSS refuses n = 10 because
        // C(10, 4) = 210 seeds is impractical; the coin-toss has no setup at all.
        assert!(crate::prss::PrssRandomness::with_seeded_setup(10, 1).is_err());
        for n in [2usize, 3, 4, 9, 10, 16, 33] {
            let source = CoinTossRandomness::new_seeded(n, 5).expect("no setup to refuse");
            assert_eq!(source.parties(), n);
            assert_eq!(source.threshold(), (n - 1) / 2);
            // No round has run yet — construction is free of interaction too.
            assert_eq!(source.rounds(), 0);
        }
        // n < 2 is refused like ShamirBackend::new.
        assert!(CoinTossRandomness::new_seeded(1, 1).is_err());
        assert!(CoinTossRandomness::new_seeded(0, 1).is_err());
    }

    #[test]
    fn generated_sharing_is_degree_t_and_reconstructs_to_the_sum_of_contributions() {
        for n in 2..=10usize {
            let mut source = coin_toss(n);
            let backend = ShamirBackend::new(n).expect("n >= 2");
            assert_eq!(backend.threshold(), source.threshold());
            let round = source.commit_open_round(4).expect("round");
            assert_eq!(round.sharings.len(), 4);
            for (value, shares) in round.values.iter().zip(&round.sharings) {
                // Full n-party sharing on the canonical points, in order.
                assert_eq!(shares.len(), n);
                for (i, s) in shares.iter().enumerate() {
                    assert_eq!(s.x, i as u64 + 1);
                }
                // Reconstruction (which for n > t+1 also RS-checks that all n
                // points lie on ONE degree-t polynomial) yields Σ_i r_i.
                assert_eq!(
                    backend.reconstruct(shares).expect("consistent degree-t sharing"),
                    *value,
                    "n = {n}"
                );
            }
        }
    }

    #[test]
    fn any_t_plus_one_shares_reconstruct_the_same_value() {
        // Degree-`t`-ness restated as the property that matters: every quorum
        // agrees. A round that emitted a higher-degree polynomial (e.g. by
        // summing sharings of different degrees) would let different quorums
        // reconstruct different values.
        let n = 7;
        let mut source = coin_toss(n);
        let backend = ShamirBackend::new(n).expect("n >= 2");
        let t = source.threshold();
        let round = source.commit_open_round(1).expect("round");
        let (value, shares) = (round.values[0], &round.sharings[0]);
        for start in 0..=(n - (t + 1)) {
            let quorum = &shares[start..start + t + 1];
            assert_eq!(backend.reconstruct(quorum).expect("quorum"), value);
        }
        let strided: Vec<Share> = shares.iter().step_by(2).copied().take(t + 1).collect();
        assert_eq!(strided.len(), t + 1);
        assert_eq!(backend.reconstruct(&strided).expect("strided quorum"), value);
    }

    #[test]
    fn r_is_the_sum_over_every_party_not_a_subset() {
        // The joint value must fold in ALL n contributions: a generator that
        // dropped one party (or let one party determine `r`) would still look
        // like a well-formed sharing, so this is checked against the openings.
        let n = 6;
        let mut source = coin_toss(n);
        let base = source.counter;
        let mut contributions = Vec::new();
        for party in &mut source.parties {
            contributions.push(party.commit(base, 1));
        }
        let total = contributions
            .iter()
            .fold(Fp::zero(), |acc, c| acc.add(c.openings[0].value));
        assert_eq!(contributions.len(), n);
        // Omitting ANY single party's contribution changes the joint value —
        // i.e. every party genuinely randomises `r` (and none is ignored).
        for skipped in 0..n {
            let partial = contributions
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != skipped)
                .fold(Fp::zero(), |acc, (_, c)| acc.add(c.openings[0].value));
            assert_ne!(
                partial, total,
                "party {} must contribute to r",
                skipped as u64 + 1
            );
        }
    }

    #[test]
    fn each_party_draws_from_its_own_randomness() {
        // Every party must have an INDEPENDENT generator. Seeding them all the
        // same way would make every r_i identical (r = n·r_1, and every party
        // would know r) — the exact bug `party_seed` exists to prevent.
        let n = 8;
        let mut source = coin_toss(n);
        let base = source.counter;
        let values: Vec<u64> = source
            .parties
            .iter_mut()
            .map(|p| p.commit(base, 1).openings[0].value.value())
            .collect();
        let distinct: BTreeSet<u64> = values.iter().copied().collect();
        assert_eq!(
            distinct.len(),
            n,
            "each party's contribution must come from its own randomness: {values:?}"
        );
    }

    #[test]
    fn one_commit_open_round_per_batch_and_none_for_an_empty_batch() {
        // The coin-toss's defining cost, made measurable (design §2.2/§2.3):
        // batching is what amortises the round, and PRSS's contrast is 0 rounds.
        let mut source = coin_toss(5);
        let batched = source.shared_masks(16).expect("batch");
        assert_eq!(batched.len(), 16);
        assert_eq!(source.rounds(), 1, "a whole batch costs ONE round");

        for _ in 0..16 {
            source.shared_mask().expect("mask");
        }
        assert_eq!(source.rounds(), 17, "unbatched masks cost one round each");

        // An empty batch is a no-op: no round, no counter burned.
        let counter_before = source.counter;
        assert!(source.shared_masks(0).expect("empty batch").is_empty());
        assert_eq!(source.rounds(), 17);
        assert_eq!(source.counter, counter_before);
    }

    #[test]
    fn commitment_binds_the_contribution() {
        // The round-ordering check: a value that changed between the commit and
        // the open phase must abort the round.
        let n = 4;
        let mut source = coin_toss(n);
        let base = source.counter;
        let mut contribution = source.parties[0].commit(base, 3);
        // The honest transcript verifies.
        check_openings(base, &contribution).expect("an untampered opening must verify");

        // (a) A changed VALUE is caught.
        let mut tampered = source.parties[1].commit(base, 3);
        tampered.openings[1].value = tampered.openings[1].value.add(Fp::one());
        assert!(matches!(
            check_openings(base, &tampered),
            Err(MpcError::Protocol(_))
        ));

        // (b) A changed NONCE is caught (the hiding randomiser is committed too).
        let mut renonced = source.parties[2].commit(base, 3);
        renonced.openings[0].nonce[0] ^= 1;
        assert!(matches!(
            check_openings(base, &renonced),
            Err(MpcError::Protocol(_))
        ));

        // (c) The commitment is bound to the ROUND: replaying this transcript at
        //     another counter must not verify.
        assert!(matches!(
            check_openings(base + 1, &contribution),
            Err(MpcError::Protocol(_))
        ));

        // (d) ...and to the COMMITTER: another party cannot claim it.
        contribution.party += 1;
        assert!(matches!(
            check_openings(base, &contribution),
            Err(MpcError::Protocol(_))
        ));

        // (e) A length mismatch is refused rather than silently truncated.
        let mut short = source.parties[3].commit(base, 3);
        short.openings.pop();
        assert!(matches!(
            check_openings(base, &short),
            Err(MpcError::Protocol(_))
        ));
    }

    #[test]
    fn local_sum_refuses_a_share_addressed_to_another_party() {
        let mine = Share {
            x: 2,
            y: Fp::new(11),
        };
        let theirs = Share {
            x: 3,
            y: Fp::new(13),
        };
        assert_eq!(
            local_sum(2, &[mine, mine]).expect("own shares").y,
            Fp::new(22)
        );
        assert!(matches!(
            local_sum(2, &[mine, theirs]),
            Err(MpcError::Protocol(_))
        ));
        // The empty sum is the additive identity at this party's point.
        assert_eq!(
            local_sum(2, &[]).expect("empty"),
            Share { x: 2, y: Fp::zero() }
        );
    }

    #[test]
    fn masks_do_not_repeat_across_rounds() {
        let n = 5;
        let mut source = coin_toss(n);
        let backend = ShamirBackend::new(n).expect("n >= 2");
        let mut seen = Vec::new();
        for _ in 0..32 {
            let shares = source.shared_mask().expect("mask");
            seen.push(backend.reconstruct(&shares).expect("reconstruct").value());
        }
        let distinct: BTreeSet<u64> = seen.iter().copied().collect();
        assert_eq!(distinct.len(), seen.len(), "masks must not repeat");
    }

    #[test]
    fn independent_sources_produce_independent_randomness() {
        // The production path must not be a fixed sequence.
        let mut a = CoinTossRandomness::new(4).expect("no setup");
        let mut b = CoinTossRandomness::new(4).expect("no setup");
        let backend = ShamirBackend::new(4).expect("n >= 2");
        let ra = backend.reconstruct(&a.shared_mask().expect("mask")).expect("r");
        let rb = backend.reconstruct(&b.shared_mask().expect("mask")).expect("r");
        assert_ne!(ra, rb);
    }

    #[test]
    fn shared_nonzero_mask_never_shares_zero() {
        let n = 5;
        let mut source = coin_toss(n);
        let backend = ShamirBackend::new(n).expect("n >= 2");
        for _ in 0..500 {
            let shares = source.shared_nonzero_mask().expect("nonzero mask");
            assert_ne!(
                backend.reconstruct(&shares).expect("reconstruct"),
                Fp::zero(),
                "the equality mask must never be r = 0"
            );
        }
    }

    #[test]
    fn counter_exhaustion_is_refused_rather_than_wrapped() {
        let mut source = coin_toss(4);
        source.counter = u64::MAX;
        assert!(matches!(source.shared_mask(), Err(MpcError::Protocol(_))));
        assert!(matches!(
            source.shared_nonzero_mask(),
            Err(MpcError::Protocol(_))
        ));
        assert!(matches!(
            source.shared_masks(2),
            Err(MpcError::Protocol(_))
        ));
        // The counter must not have wrapped to 0 (which would re-issue the
        // commitment domain of round #0).
        assert_eq!(source.counter, u64::MAX);
        // A refused round is not counted as executed.
        assert_eq!(source.rounds(), 0);
    }

    #[test]
    fn vss_own_input_is_honestly_refused() {
        // The coin-toss is not a verified input-sharing protocol; faking it with
        // an unverified plain sharing would drop the verification that is the
        // whole point of dealer-less VSS.
        let mut source = coin_toss(5);
        assert!(matches!(
            source.vss_own_input(Fp::new(4085)),
            Err(MpcError::NotYetImplemented { .. })
        ));
    }

    #[test]
    fn debug_never_prints_party_state() {
        let source = coin_toss(4);
        let rendered = format!("{source:?}");
        assert!(rendered.contains("<party-local dealer>"));
        assert!(!rendered.contains("ShamirDealer"));
    }

    #[test]
    fn commitment_is_domain_separated_by_round_party_nonce_and_value() {
        let nonce = [1u64, 2, 3, 4];
        let other = [1u64, 2, 3, 5];
        let base = commit_to(7, 1, &nonce, Fp::new(9));
        assert_ne!(base, commit_to(8, 1, &nonce, Fp::new(9)), "round differs");
        assert_ne!(base, commit_to(7, 2, &nonce, Fp::new(9)), "party differs");
        assert_ne!(base, commit_to(7, 1, &other, Fp::new(9)), "nonce differs");
        assert_ne!(base, commit_to(7, 1, &nonce, Fp::new(10)), "value differs");
        // Deterministic in its inputs — the check in `check_openings` depends
        // on recomputing exactly the same commitment.
        assert_eq!(base, commit_to(7, 1, &nonce, Fp::new(9)));
    }
}
