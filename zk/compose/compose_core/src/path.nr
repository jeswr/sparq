//! Bounded-depth path-reachability proof (`path_reach_d{D}_k{K}_n{N}` family,
//! sq-3kd2g.2 / #1591; design record `research/zksparql-fragment-extension.md`
//! sec 4 -- the normative statement and its soundness requirements 1-6).
//!
//! [FABLE-5] Statement, for a SPARQL property-path operator `p+` / `p*` over
//! the union of `K` committed graphs, with the depth bound `D` a COMPILE-TIME
//! constant of the circuit member (mechanism (a), maintainer-confirmed on
//! #1591: distinct `D` => distinct deterministic verification key, so the
//! verifier learns the bound from WHICH circuit member it accepts; the
//! `depth_bound` public input re-states `D` in-circuit, belt-and-braces):
//!
//! > There exists a chain of committed triples `(t_1, ..., t_l)` with
//! > `1 <= l <= D` (`0 <= l <= D` when `allow_zero`), each `t_i` of the form
//! > `(n_{i-1}, pred_enc, n_i)` and a member of some committed graph in the
//! > attested source set, with `n_0 = src_enc` and `n_l = dst_enc`.
//!
//! This is EXISTENCE ONLY (record sec 4 req 2): the relation never asserts that
//! longer paths do not exist, nor that the reachable set is complete. Failure
//! to produce a proof at depth `D` proves NOTHING. Every bounded witness is a
//! genuine SPARQL `p+`/`p*` solution (membership preserved, req 3); consumers
//! MUST be shown `D` -- rendering "path exists" without the bound
//! misrepresents the claim (req 1; the verifier obligations land with the
//! fragment-gate/manifest beads sq-3kd2g.3/.6).
//!
//! Constraint walk (numbered like `scan.nr`):
//!
//! 1.  **Commitment recompute** -- identical to `scan_check` step 1: the
//!     witnessed per-slot term encodings re-hash (leaf `h3`, `commit_fold`
//!     over the first `counts[g]` leaves) to the PUBLIC `commitments[g]`.
//! 1b. **Distinct-graph strict ordering** -- same family guard as `scan_check`
//!     step 1b (plan S2.5): public commitments strictly increasing, so the
//!     `K` committed graphs are pairwise distinct.
//! 2.  **Depth-bound disclosure**: `depth_bound == D`, constraining the public
//!     input to the compile-time bound (a manifest cannot claim a different
//!     bound than the member's VK encodes).
//! 3.  **Chain soundness with inert padding** (req 4 -- padding rows are THE
//!     forgery surface, and the record's padding-forgery negatives are
//!     normative). The witness discloses `path_len = l` (PRIVATE -- the exact
//!     length within the bound stays hidden) and a node array `nodes[D]`. A
//!     running endpoint starts at `cur = src_enc`; step `s` is ACTIVE iff
//!     `s < path_len`. Every ACTIVE step must witness the triple
//!     `(cur, pred_enc, nodes[s])` in some active slot of some committed
//!     graph (the same union row-membership relation the scan circuits use);
//!     `cur` then advances to `nodes[s]`. INACTIVE steps are pass-throughs:
//!     they are exempt from membership but CANNOT advance `cur` -- so a
//!     shorter real chain cannot use fabricated padding rows to "reach"
//!     `dst_enc` (forge-negative: padding_cannot_extend), and a longer claim
//!     cannot activate an uncommitted step (forge-negative: fabricated
//!     active step trips membership).
//! 4.  **Endpoint**: after all `D` steps, `cur == dst_enc`. For `l = 0` the
//!     endpoint never advanced, so this enforces `src_enc == dst_enc` -- the
//!     zero-length equality of req 5 -- with no extra constraint.
//! 5.  **Zero-length occurrence witness** (req 5): a zero-length path
//!     additionally requires `allow_zero` (the operator admits it: `p*`/`p?`,
//!     never `p+`) AND that the endpoint term occurs as a SUBJECT or OBJECT
//!     of an active slot in some committed graph -- SPARQL's zero-length-path
//!     term universe is the graph's nodes, so bare `src == dst` equality is
//!     NOT enough, and a predicate-only occurrence does not count.
//! 6.  **Source attribution**: `attribution[g]` (PUBLIC) must equal "graph
//!     `g` contributes to the witnessed chain" -- it holds an active slot
//!     equal to some ACTIVE chain step (or, for the zero-length case, an
//!     active slot whose subject/object is the endpoint term). Unlike
//!     `scan_check`'s attribution (a deterministic function of the public
//!     pattern and the committed graphs), path attribution is
//!     CHAIN-RELATIVE: it attests the source graphs of the EXISTENTIAL
//!     WITNESS the prover chose, not of all possible chains -- a different
//!     witness chain between the same endpoints may legitimately carry a
//!     different attribution set. Within the chosen witness it is exact:
//!     a triple present in several graphs sets EVERY such graph's bit, so
//!     the prover can neither understate nor overstate.
//!
//! `challenge` (in the member `main`s) is a public input bound by the proof
//! but unconstrained in the relation -- replay binding is the verifier's
//! nonce check, exactly as in every other family member.
//!
//! Cycles are permitted (req 6): SPARQL 1.1 path semantics is existence-based
//! set semantics, so the chain is not required to be simple.
//!
//! ## NOT a soundness claim (load-bearing honesty)
//!
//! This module BUILDS circuits; it does not make the composition verifier
//! sound. The verifier stack is internally re-audited but NOT externally
//! audited (`sq-qhy4` pending); nothing here is a proven cryptographic
//! guarantee. Verifier-side acceptance of path syntax and the manifest/
//! dispatch wiring are follow-up beads (`sq-3kd2g.3` / `sq-3kd2g.6`).

use crate::hashes::{commit_fold, h3};

/// The generic bounded-depth reachability relation; each
/// `path_reach_d{D}_k{K}_n{N}` bin package is one monomorphisation. See the
/// module docs for the statement and the numbered constraint walk.
pub fn path_reach_check<let D: u32, let K: u32, let N: u32>(
    commitments: [Field; K],
    pred_enc: Field,
    src_enc: Field,
    dst_enc: Field,
    allow_zero: bool,
    depth_bound: u32,
    attribution: [bool; K],
    path_len: u32,
    nodes: [Field; D],
    counts: [u32; K],
    enc: [[[Field; 3]; N]; K],
) {
    // 2. Depth-bound disclosure: the public input re-states the compile-time
    //    bound (the VK already IS the bound -- mechanism (a); this pins the
    //    manifest-visible number to it).
    assert(depth_bound == D, "depth bound does not match the circuit member");

    assert(path_len <= D, "path length exceeds circuit depth bound");
    let zero = path_len == 0;
    // `p+` (allow_zero = false) requires at least one step.
    assert(allow_zero | !zero, "zero-length path not allowed for this operator");

    // 1. Commitment recompute (binds every active slot's encodings).
    for g in 0..K {
        assert(counts[g] <= N, "graph size exceeds circuit bucket");
        let mut leaves: [Field; N] = [0; N];
        for i in 0..N {
            leaves[i] = h3(enc[g][i][0], enc[g][i][1], enc[g][i][2]);
        }
        assert_eq(commit_fold(leaves, counts[g]), commitments[g], "graph commitment mismatch");
    }

    // 1b. Distinct-graph strict ordering (same guard as scan_check step 1b).
    for g in 1..K {
        assert(
            commitments[g - 1].lt(commitments[g]),
            "per-graph commitments not strictly increasing (duplicate-inclusion / COUNT forgery)",
        );
    }

    // 3. Chain soundness with inert padding. `contrib[g]` accumulates the
    //    per-graph source-attribution bits alongside the same sweep.
    let mut contrib: [bool; K] = [false; K];
    let mut cur = src_enc;
    for s in 0..D {
        let active = s < path_len;
        let mut present = false;
        for g in 0..K {
            let mut present_g = false;
            for i in 0..N {
                let slot_active = i < counts[g];
                let eq =
                    (enc[g][i][0] == cur) & (enc[g][i][1] == pred_enc) & (enc[g][i][2] == nodes[s]);
                present_g |= slot_active & eq;
            }
            // Only ACTIVE steps attribute: an inactive padding row equal to
            // some graph's content must not drag that graph into the
            // attested source set.
            contrib[g] |= active & present_g;
            present |= present_g;
        }
        assert(!active | present, "chain step not in any committed graph");
        // Pass-through padding: inactive steps CANNOT advance the endpoint.
        cur = if active { nodes[s] } else { cur };
    }

    // 4. Endpoint. For path_len = 0 this is the src == dst equality (req 5).
    assert(cur == dst_enc, "chain does not reach the destination");

    // 5. Zero-length occurrence witness (req 5): the endpoint term must be a
    //    NODE of the committed union -- subject or object of an active slot;
    //    predicate-position occurrence does not count.
    let mut occurs_any = false;
    for g in 0..K {
        let mut occurs_g = false;
        for i in 0..N {
            let slot_active = i < counts[g];
            occurs_g |= slot_active & ((enc[g][i][0] == src_enc) | (enc[g][i][2] == src_enc));
        }
        // The zero-length case attributes every graph the endpoint occurs in
        // (deterministic given the endpoint, unlike the chain-relative bits).
        contrib[g] |= zero & occurs_g;
        occurs_any |= occurs_g;
    }
    assert(!zero | occurs_any, "zero-length endpoint not a node of any committed graph");

    // 6. Source attribution: exact w.r.t. the witnessed chain (chain-relative;
    //    see module docs), in BOTH directions.
    for g in 0..K {
        assert(
            attribution[g] == contrib[g],
            "attribution does not match the witnessed chain's source graphs",
        );
    }
}
