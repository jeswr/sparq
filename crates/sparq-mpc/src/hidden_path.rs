// [OPUS-4.8] sq-py8h.2 — HIDDEN-intermediate fixed exactly-`k` property-path chain
// (the cryptographic core of the bounded property-path operator).
//! Bounded property-path over a **secret graph**: the HIDDEN-intermediate fixed
//! exactly-`k` chain (`?a (p){k} ?b`) where the intermediate nodes `?z_1 … ?z_{k−1}`
//! are **never opened**.
//!
//! ## What this is (and how it differs from the disclosed-key sibling)
//!
//! This resolves **increment #2** of the bounded property-path design
//! (`research/mpc-bounded-property-path-design.md` §2.1 HIDDEN regime, §2.6, §6
//! step 2) — the **cryptographic core** the disclosed-key module
//! ([`crate::bounded_path`]) explicitly deferred. There the join keys are disclosed
//! global IRIs and the whole chain is a crypto-free fold of equi-joins. HERE the
//! intermediate node *values* are PRIVATE (secret-shared `F_p` keys): a chain
//! `a → z_1 → … → z_{k−1} → b` exists iff there is a sequence of `k` edges whose
//! join points line up, and each "lines up" test is decided as a **secret-shared
//! match bit that is never opened** — so no intermediate node `z_i`, and no per-hop
//! / per-pair match pattern, is ever reconstructed.
//!
//! ## The construction (design §2.1 / §2.6, realised against landed primitives)
//!
//! The federated edge set `E` is the union of every holder's `(subject, object)`
//! edges for the path predicate, each endpoint carried as `(private F_p key,
//! disclosed endpoint term)`. (The endpoint *terms* are the disclosed RESULT of the
//! path — convention #4: only the *intermediate join values* are hidden. The
//! candidate edge-tuple COUNT `|E|^k` is public, the standard MPC assumption.)
//!
//! An exactly-`k` chain is a `k`-tuple of edges `(e_1, …, e_k)` that is "connected":
//!
//! ```text
//! connected(e_1,…,e_k) = AND_{i=1..k-1}  [ e_i.object_key == e_{i+1}.subject_key ]
//! ```
//!
//! Each internal equality is computed with [`crate::compare::secure_equal_to_bit`]
//! — a bit-decomposition equality whose 0/1 verdict is a fresh degree-`t`
//! **secret-shared** sharing that is NEVER opened (so the L2 per-pair / per-hop
//! match-graph leak is closed *at the decision*, not merely at the output). The
//! `k − 1` internal-hop bits are AND-ed into a single per-tuple **chain bit** by
//! one [`crate::shamir::mul_shares_raw`] + one
//! [`crate::shamir::ShamirDealer::degree_reduce`] per AND — the BGW reshare-and-
//! recombine round (sq-dvuc, the keystone this bead depends on) that lets a
//! degree-`2t` product be reshared back to degree `t` so the next AND can chain.
//! For `k = 1` every edge is a trivially-connected length-1 chain, so the chain bit
//! is the constant shared `1` (no internal hop).
//!
//! Each tuple is a candidate whose endpoint pair `(a, b) = (e_1.subject,
//! e_k.object)` is the disclosed payload. **Set semantics (design §2.2/§2.5):** a
//! pair reachable by several distinct edge-tuples must appear ONCE, with the
//! realized chain *never* revealed. We therefore GROUP tuples by their disclosed
//! endpoint pair and **OR-fold** their chain bits into one connected-bit per
//! distinct `(a, b)` (`OR(x,y) = 1 − (1−x)(1−y)`, one more `mul_shares_raw` +
//! `degree_reduce`). Grouping is over the DISCLOSED endpoint terms — never over any
//! secret intermediate key — so it leaks nothing the result does not already
//! disclose.
//!
//! The one connected-bit per distinct endpoint pair becomes a
//! [`crate::oblivious_join::Candidate`] with a
//! [`crate::oblivious_join::MatchBit::SecretShared`] selector, fed verbatim into the
//! landed oblivious output path
//! ([`crate::oblivious_join::oblivious_set_output`], sq-jnkm): the secret bit drives
//! an oblivious select (one product, no open), the slots are shuffled, and exactly a
//! public padded bound `B` of slots is revealed. The per-pair match bit is never
//! opened anywhere; only the `B` shuffled output tags are opened (classifying a slot
//! as a real endpoint row or a dummy AFTER the permutation destroyed
//! position→candidate linkage).
//!
//! ## What stays hidden / what is disclosed (state it precisely)
//!
//! - **Hidden:** every intermediate node key `z_i` (never reconstructed); every
//!   per-hop and per-pair match bit (secret-shared, never opened); the realized
//!   chain that connected a pair (the OR-fold collapses multiplicity); the true
//!   result cardinality beyond the public `B` (L1 bounded to `B`).
//! - **Disclosed:** the public bound `k`; the public candidate-tuple count `|E|^k`
//!   (standard MPC assumption); the endpoint TERMS of the matched pairs (the result
//!   the recipient asked for); the public padded bound `B`.
//!
//! ## Security tier (HONESTY — privacy-claims gate, cite sq-qhy4)
//!
//! Honest-majority, semi-honest ONLY — inherits the [`crate::shamir::ShamirBackend`]
//! / [`crate::compare`] model and adds no new assumption. Every multiplication
//! routes through [`crate::shamir::ShamirDealer::degree_reduce`], whose reshare has
//! no in-protocol check that a deviating party reshared honestly, so this is **NOT**
//! maliciously secure (the same degree-`2t`-at-`n=2t+1` residual the rest of the
//! backend documents; sq-6d6g / sq-km34 IT-MAC is the named fix). The
//! per-intermediate / per-pair confidentiality this closes is a CONFIDENTIALITY
//! axis, orthogonal to malicious security; external soundness sign-off is still
//! pending (sq-qhy4). No guarantee beyond the documented semi-honest model is
//! claimed.
//!
//! ## Scope boundary (do not over-read)
//!
//! This is the **exactly-`k`** chain ONLY (`?a (p){k} ?b`, `k >= 1`). The bounded
//! `{1,k}` / `{0,k}` / `p?` union + reflexive diagonal + alternation, with the
//! cross-length OR-fold dedup, is the sibling increment **sq-py8h.3** (it builds the
//! union of exactly-`ℓ` chains ON TOP of this). DISTINCT over genuinely-secret
//! endpoint pairs (collapsing duplicate *secret* `(a,b)`) is the gated piece
//! **sq-py8h.4** (needs the secure-sort comparator). Planner guards / cost wiring is
//! **sq-py8h.5**. Here the endpoint dedup is over the DISCLOSED endpoint terms,
//! which is exactly the headline case (the endpoints are the disclosed result).

use crate::compare::secure_equal_to_bit;
use crate::field::Fp;
use crate::oblivious_join::{
    oblivious_join_output, oblivious_set_output, Candidate, MatchBit, ObliviousOutput,
};
use crate::partial::{MpcError, PartialResult};
use crate::shamir::{self, ShamirBackend, ShamirDealer, Share};
use oxrdf::{Term, Variable};
use std::collections::BTreeMap;

/// Reserved subject-end / object-end variable names the operator projects onto.
const SUBJECT_VAR: &str = "__pp_a";
const OBJECT_VAR: &str = "__pp_b";

/// Maximum number of edge-tuples (`|E|^k`) the operator will enumerate before
/// refusing the path with a [`MpcError::Protocol`]. The enumeration is
/// `|E|^k` — exponential in the PUBLIC bound `k` and polynomial-of-degree-`k` in
/// `|E|` — and each tuple costs `k − 1` secure equalities + AND/OR chaining, so an
/// unchecked unroll is a CPU/round denial-of-service on this surface. Because both
/// `|E|` and `k` are PUBLIC, the count is computable in closed form WITHOUT touching
/// any secret key, so we reject an over-cap path BEFORE any crypto runs. A protocol
/// guard, not a tuning knob: a path that exceeds it is abusive for this regime.
/// `[OPUS-4.8]`
pub const MAX_CHAIN_TUPLES: u64 = 1 << 18; // 262_144

/// One node of the secret graph: its PRIVATE join key (an `F_p` element — the
/// holder's injective encoding of the node, exactly the contract
/// [`crate::join::HiddenValueJoin`] documents) plus its DISCLOSED term (used ONLY
/// when the node is an ENDPOINT of a matched chain — never for an intermediate).
#[derive(Debug, Clone)]
pub struct HiddenNode {
    /// The private join key. Compared via secure equality; never reconstructed
    /// for an intermediate node.
    pub key: Fp,
    /// The disclosed term, emitted only if this node is a matched chain endpoint.
    pub term: Term,
}

/// One directed edge of the secret graph for the path predicate: `(subject,
/// object)`, each a [`HiddenNode`]. The edge set is the union of every holder's
/// disclosed edges for the predicate.
#[derive(Debug, Clone)]
pub struct HiddenEdge {
    /// The edge's subject node.
    pub subject: HiddenNode,
    /// The edge's object node.
    pub object: HiddenNode,
}

/// The federated secret edge relation the chain operator traverses — the union of
/// all holders' `(subject, object)` edges for one path predicate. The keys are
/// private; only the candidate-tuple COUNT (`|E|^k`) and matched endpoint TERMS are
/// ever disclosed.
#[derive(Debug, Clone)]
pub struct HiddenEdges {
    edges: Vec<HiddenEdge>,
}

impl HiddenEdges {
    /// Build the edge relation from a list of edges (the federated union).
    pub fn new(edges: Vec<HiddenEdge>) -> Self {
        HiddenEdges { edges }
    }

    /// Number of edges in the relation (`|E|` — public).
    pub fn len(&self) -> usize {
        self.edges.len()
    }

    /// Whether the relation is empty.
    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }
}

/// `|E|^k` (the edge-tuple count), checked against `u64` overflow so the caller can
/// reject the path with a controlled error instead of risking a huge allocation.
/// `[OPUS-4.8]`
fn projected_tuple_count(edge_count: usize, k: usize) -> Option<u64> {
    let e = edge_count as u64;
    let mut total: u64 = 1;
    for _ in 0..k {
        total = total.checked_mul(e)?;
    }
    Some(total)
}

/// A degree-`t` sharing of the PUBLIC constant `c` on the canonical points
/// `x = 1..=n`: the constant polynomial `f(x) = c` at each party. Reconstructs to
/// `c`, is degree `t` (degree 0 ⊆ degree `t`), and composes with secret sharings in
/// the multiplications below with no dealer/randomness. (Mirrors `compare.rs`'s
/// `const_sharing`, local here to avoid coupling across modules.)
fn const_sharing(n: usize, c: Fp) -> Vec<Share> {
    (1..=n as u64).map(|x| Share { x, y: c }).collect()
}

/// Secret AND of two secret-shared 0/1 bits: `[a ∧ b] = [a]·[b]`. One
/// [`shamir::mul_shares_raw`] (degree `2t`) followed by one
/// [`ShamirDealer::degree_reduce`] back to a fresh degree-`t` sharing — the BGW
/// multiplication-chain step (sq-dvuc) that lets the `k` hop bits chain. (Same
/// construction as `compare.rs`'s `secret_and`; local to keep file-lane discipline.)
fn secret_and(dealer: &mut ShamirDealer, a: &[Share], b: &[Share]) -> Result<Vec<Share>, MpcError> {
    let prod_2t = shamir::mul_shares_raw(a, b)?;
    dealer.degree_reduce(&prod_2t)
}

/// Secret OR of two secret-shared 0/1 bits via De Morgan: `[a ∨ b] = a + b − a·b`.
/// One secure multiplication (the `a·b` term); the rest is local affine ops. Used
/// to OR-fold the per-tuple chain bits of the chains that reach the same endpoint
/// pair into one connected-bit (design §2.5).
fn secret_or(dealer: &mut ShamirDealer, a: &[Share], b: &[Share]) -> Result<Vec<Share>, MpcError> {
    let and = secret_and(dealer, a, b)?;
    let sum = shamir::add_shares(a, b)?;
    shamir::sub_shares(&sum, &and)
}

/// The chain bit of ONE edge-tuple `(e_1, …, e_k)`: the secret-shared AND over the
/// `k − 1` internal hop equalities `[e_i.object_key == e_{i+1}.subject_key]`, each a
/// [`secure_equal_to_bit`] (secret-shared, never opened). For `k = 1` there is no
/// internal hop, so the bit is the constant shared `1` (every edge is a length-1
/// chain). The intermediate node KEYS are passed to the secure equality but never
/// reconstructed.
fn tuple_chain_bit(
    backend: &ShamirBackend,
    dealer: &mut ShamirDealer,
    tuple: &[&HiddenEdge],
) -> Result<Vec<Share>, MpcError> {
    let n = backend.parties();
    debug_assert!(!tuple.is_empty());
    // k = 1: trivially connected (no internal hop) → constant shared 1.
    let mut bit = const_sharing(n, Fp::one());
    for w in tuple.windows(2) {
        // The internal hop join: e_i.object must equal e_{i+1}.subject. Both keys
        // are PRIVATE intermediate node values — the equality verdict stays a
        // secret-shared bit, never opened (closes the per-hop L2 leak at source).
        let hop = secure_equal_to_bit(dealer, w[0].object.key, w[1].subject.key)?;
        bit = secret_and(dealer, &bit, &hop)?;
    }
    Ok(bit)
}

/// A disclosed endpoint pair key for grouping tuples (set semantics). Built from the
/// DISCLOSED endpoint terms — never any secret intermediate key.
fn endpoint_key(a: &Term, b: &Term) -> String {
    format!("{a:?}\u{0}{b:?}")
}

/// Build one [`Candidate`] per DISTINCT disclosed endpoint pair, OR-folding the
/// chain bits of every edge-tuple that reaches that pair into one secret-shared
/// connected-bit (design §2.5 — set semantics, realized length never revealed).
///
/// Iterates the `|E|^k` edge-tuples lexicographically, computes each tuple's chain
/// bit ([`tuple_chain_bit`]), and accumulates it into the connected-bit for the
/// tuple's endpoint pair `(e_1.subject, e_k.object)`. The grouping key is the
/// disclosed endpoint pair; the connected-bit and the per-tuple chain bits are all
/// secret-shared and never opened here.
fn build_candidates(
    backend: &ShamirBackend,
    dealer: &mut ShamirDealer,
    edges: &HiddenEdges,
    k: usize,
) -> Result<Vec<Candidate>, MpcError> {
    // Accumulator: disclosed endpoint key → (endpoint terms, OR-folded connected bit).
    let mut by_pair: BTreeMap<String, (Term, Term, Vec<Share>)> = BTreeMap::new();

    // Lexicographic enumeration of the |E|^k edge-tuples via a base-|E| odometer.
    let m = edges.len();
    if m == 0 {
        return Ok(Vec::new()); // no edges → no chains
    }
    let mut idx = vec![0usize; k];
    loop {
        // Materialise the current tuple as edge references.
        let tuple: Vec<&HiddenEdge> = idx.iter().map(|&i| &edges.edges[i]).collect();
        let chain_bit = tuple_chain_bit(backend, dealer, &tuple)?;
        let a_term = tuple[0].subject.term.clone();
        let b_term = tuple[k - 1].object.term.clone();
        let key = endpoint_key(&a_term, &b_term);
        match by_pair.get_mut(&key) {
            Some((_, _, acc)) => {
                // OR this tuple's chain bit into the running connected-bit.
                let folded = secret_or(dealer, acc, &chain_bit)?;
                *acc = folded;
            }
            None => {
                by_pair.insert(key, (a_term, b_term, chain_bit));
            }
        }

        // Advance the odometer (least-significant position is idx[k-1]).
        let mut pos = k;
        loop {
            if pos == 0 {
                // Wrapped all positions → enumeration complete.
                return Ok(by_pair
                    .into_values()
                    .map(|(a, b, bit)| Candidate {
                        payload: vec![Some(a), Some(b)],
                        matched: MatchBit::SecretShared(bit),
                    })
                    .collect());
            }
            pos -= 1;
            idx[pos] += 1;
            if idx[pos] < m {
                break;
            }
            idx[pos] = 0;
        }
    }
}

/// Fail-closed parameter checks shared by both entry points. `k >= 1`,
/// `n >= 2t+1` (the AND/OR chain's degree reduction needs it), and the projected
/// `|E|^k` tuple count must be within [`MAX_CHAIN_TUPLES`] — all computed from
/// PUBLIC parameters, before any crypto runs.
fn check_params(backend: &ShamirBackend, edges: &HiddenEdges, k: usize) -> Result<(), MpcError> {
    if k == 0 {
        return Err(MpcError::Protocol(
            "hidden exactly-k chain: k must be >= 1 (the reflexive length-0 arm is the {0,k} \
             increment sq-py8h.3, not the exactly-k chain operator)"
                .into(),
        ));
    }
    let t = backend.threshold();
    if backend.parties() < 2 * t + 1 {
        return Err(MpcError::Protocol(format!(
            "hidden exactly-k chain needs n >= 2t+1 for the AND/OR chain's degree reduction \
             (n={}, t={t})",
            backend.parties()
        )));
    }
    match projected_tuple_count(edges.len(), k) {
        Some(count) if count <= MAX_CHAIN_TUPLES => Ok(()),
        other => Err(MpcError::Protocol(format!(
            "hidden exactly-k chain over |E|={} edges with k={k} would enumerate {} edge-tuples, \
             exceeding the MAX_CHAIN_TUPLES cap of {} — refusing (CPU/round denial-of-service)",
            edges.len(),
            other
                .map(|c| c.to_string())
                .unwrap_or_else(|| "an amount that overflows u64".to_string()),
            MAX_CHAIN_TUPLES,
        ))),
    }
}

/// Evaluate the HIDDEN-intermediate exactly-`k` chain and return the disclosed
/// endpoint pairs as a [`PartialResult`] with `vars == [?__pp_a, ?__pp_b]` (real
/// rows only, dummies filtered, canonical order). The intermediate node keys are
/// never reconstructed; each per-pair connected-bit stays secret-shared until the
/// oblivious output's `B`-slot reveal.
///
/// `bound` is the public padded bound `B`; it must cover the distinct-endpoint-pair
/// candidate count (fail-closed in [`oblivious_set_output`] — never truncates a
/// match). See the module docs for the full construction and security tier.
pub fn eval_exact_k_chain_hidden(
    backend: &ShamirBackend,
    edges: &HiddenEdges,
    k: usize,
    bound: usize,
) -> Result<PartialResult, MpcError> {
    check_params(backend, edges, k)?;
    let mut dealer = backend.dealer();
    let candidates = build_candidates(backend, &mut dealer, edges, k)?;
    let out_vars = vec![
        Variable::new_unchecked(SUBJECT_VAR),
        Variable::new_unchecked(OBJECT_VAR),
    ];
    let (result, _cost) = oblivious_join_output(backend, &candidates, out_vars, bound)?;
    Ok(result)
}

/// Like [`eval_exact_k_chain_hidden`] but returns the raw oblivious output (the `B`
/// shuffled [`crate::oblivious_join::OutputSlot`]s + modelled cost) instead of the
/// dummy-filtered [`PartialResult`]. This is the transcript-level view: it
/// witnesses that the revealed slot count is exactly the public `B` (L1 bounded to
/// `B`, not the true match count). Used by the obliviousness tests.
pub fn eval_exact_k_chain_hidden_slots(
    backend: &ShamirBackend,
    edges: &HiddenEdges,
    k: usize,
    bound: usize,
) -> Result<ObliviousOutput, MpcError> {
    check_params(backend, edges, k)?;
    let mut dealer = backend.dealer();
    let candidates = build_candidates(backend, &mut dealer, edges, k)?;
    // Payload arity is 2 (the endpoint pair). Route through the slot-returning path.
    oblivious_set_output(backend, &candidates, 2, bound)
}

#[cfg(test)]
mod tests;
