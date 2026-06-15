// [OPUS-4.8] sq-py8h.1 — bounded property-path over DISCLOSED global-IRI keys.
//! Bounded-length SPARQL property paths over **disclosed global-IRI keys**
//! (the crypto-free, "headline federation" regime).
//!
//! ## What this is — and the regime boundary (read first)
//!
//! This module resolves **increment #1** of the bounded property-path design
//! (`research/mpc-bounded-property-path-design.md` §6 step 1, §2.1 DISCLOSED
//! regime). It evaluates a **bounded** path `?a (p){m,k} ?b` whose endpoints AND
//! intermediate join keys are **disclosed global IRIs** by *unrolling* the
//! bounded path — statically, on the PUBLIC bound `k` — into a finite set of
//! fixed-length BGP chains, evaluating each chain as a left-to-right fold of the
//! existing [`crate::join::DisclosedKeyJoin`], then UNION-ing and DEDUP-ing the
//! endpoint pairs.
//!
//! Because the join key on every hop is a disclosed global IRI, the whole
//! computation runs **OUTSIDE the cryptographic core** (architecture convention
//! #4; design §2.1 DISCLOSED regime): it is a plaintext fold of equi-joins, the
//! *exact* construction the `differential_three_holder_chain_equals_union` test
//! in [`crate::join`] already exercises for a single 3-pattern chain — here
//! generalised to a *bounded path* (a union of chains, one per length in
//! `m..=k`). **NO MPC primitive runs; NO secret sharing; NO MPC round occurs.**
//! It is crypto-free, and the differential tests assert that explicitly.
//!
//! ### Regime boundary (stated, not hidden) — disclosed-key ONLY
//!
//! This is the **DISCLOSED-KEY** regime: every endpoint and every intermediate
//! node `?z_i` on the chain is a disclosed global IRI, so nothing enters an MPC.
//! It does **NOT** support the HIDDEN-intermediate regime (where the `?z_i` are
//! secret and the per-hop match must stay secret-shared via `secure_equal` /
//! `degree_reduce` / `oblivious_set_output`). That is the separate, cryptographic
//! deliverable **sq-py8h.2** (design §2.1 HIDDEN regime, §6 step 2), which THIS
//! bead blocks. Do not read disclosed-key bounded-path support as hidden-path
//! support.
//!
//! ### Semantic boundary (the bound is a real SPARQL construct, not an approx)
//!
//! The operator implements the *bounded* path `{m,k}` exactly — a well-defined
//! SPARQL 1.1 construct. It is NOT the unbounded `(p)+`/`(p)*` closure; a pair
//! connected only by a chain of length `> k` is (correctly, by definition of the
//! bounded form) absent. The differential oracle is the clear-text engine's
//! `eval_path` over the *union* store evaluating the **same bounded form**
//! (`p{m,k}`), so the equality is exact, not an approximation (design §1.2).
//!
//! ## Supported path forms (the acceptance matrix)
//!
//! - **sequence** `p1/p2/.../pk` — one fixed BGP chain ([`PathForm::Sequence`]).
//! - **exact** `(p){k}` — one fixed `k`-hop chain ([`BoundedRepetition`] `m==k`).
//! - **range** `(p){m,k}` — UNION of the exactly-`ℓ` chains for `ℓ in m..=k`.
//! - **reflexive** `(p){0,k}` — the `{1,k}` union PLUS the length-0 identity pairs
//!   `(x,x)` over the node set in scope (design §2.3); the identity pairs appear
//!   exactly once after dedup.
//! - **alternation** `(p1|p2|...)` per hop — each hop position expands over its
//!   alternatives; a length-`ℓ` chain of an `a`-way alternation unrolls to `a^ℓ`
//!   fixed chains (design §2.4), all unioned and deduped.
//!
//! Composition: a [`BoundedRepetition`] is a repetition `{m,k}` of a single
//! [`PathStep`] (a named predicate or an alternation of named predicates); a
//! [`PathForm::Sequence`] is a fixed sequence of such forms each with its own
//! bound, distributed into the chain set. Endpoint-pair DEDUP is by the
//! `(a,b)` term pair (set semantics for path endpoints, design §2.2/§2.5) — the
//! realized length is NEVER part of the output (design §1.2 length-revealing
//! prohibition), so a pair reached by two different lengths appears once.

use crate::join::{DisclosedKeyJoin, GlobalJoin, JoinPlan};
use crate::partial::{HolderId, MpcError, PartialResult};
use oxrdf::{NamedNode, Term, Variable};
use std::collections::BTreeSet;

/// One hop of a bounded path: a single predicate, or an ALTERNATION of
/// predicates (`p1 | p2 | ...`). Each alternative is a disclosed global-IRI
/// predicate. A bare predicate is `alternatives.len() == 1`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathStep {
    /// The predicate IRIs this hop may traverse (one = a plain `p`, many = an
    /// alternation `p1|p2|...`). Must be non-empty.
    pub alternatives: Vec<NamedNode>,
}

impl PathStep {
    /// A single-predicate hop `p`.
    pub fn predicate(p: NamedNode) -> Self {
        PathStep {
            alternatives: vec![p],
        }
    }

    /// An alternation hop `p1 | p2 | ...`.
    pub fn alternation(alts: impl IntoIterator<Item = NamedNode>) -> Self {
        PathStep {
            alternatives: alts.into_iter().collect(),
        }
    }
}

/// A bounded repetition of one [`PathStep`]: `(step){m,k}` with PUBLIC, finite
/// `m <= k`. This is the core repetition unit; [`PathForm`] composes it.
///
/// - `(p){k}` is `m == k` (exactly-`k`).
/// - `(p){m,k}` is the range.
/// - `(p){0,k}` (`m == 0`) adds the length-0 reflexive identity pairs (design
///   §2.3). `(p?)` is the special case `{0,1}`.
///
/// The bound is PUBLIC and part of the query plan; it is a stated parameter, not
/// a secret (design §1.2/§4.1). An OPEN upper bound (`{m,}`, `+`, `*`) is NOT
/// representable here by construction — `max` is a concrete `usize` — which is
/// the point: only the bounded, tractable slice lives in this module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedRepetition {
    /// The repeated step.
    pub step: PathStep,
    /// Minimum hop count `m` (`0` enables the reflexive identity pairs).
    pub min: usize,
    /// Maximum hop count `k` (PUBLIC, finite, `>= min`).
    pub max: usize,
}

/// A bounded property-path FORM the unroller understands. Either a single
/// bounded repetition, or a fixed sequence of bounded repetitions (`/`).
///
/// A pure sequence of single-predicate steps `p1/p2/.../pk` is
/// `Sequence(vec![{p1}{1,1}, {p2}{1,1}, ...])`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathForm {
    /// `(step){m,k}` — a bounded repetition of one (possibly alternation) step.
    Repetition(BoundedRepetition),
    /// `f1 / f2 / ...` — a fixed sequence of bounded forms, joined head-to-tail.
    /// The endpoint pairs are the relational composition of the sub-forms'
    /// pair relations.
    Sequence(Vec<PathForm>),
}

impl PathForm {
    /// Exactly-`k` repetition `(p){k}` of a single predicate.
    pub fn exact(p: NamedNode, k: usize) -> Self {
        PathForm::Repetition(BoundedRepetition {
            step: PathStep::predicate(p),
            min: k,
            max: k,
        })
    }

    /// Range repetition `(p){m,k}` of a single predicate.
    pub fn range(p: NamedNode, m: usize, k: usize) -> Self {
        PathForm::Repetition(BoundedRepetition {
            step: PathStep::predicate(p),
            min: m,
            max: k,
        })
    }

    /// Range repetition `(step){m,k}` of an arbitrary (possibly alternation) step.
    pub fn repeat_step(step: PathStep, m: usize, k: usize) -> Self {
        PathForm::Repetition(BoundedRepetition {
            step,
            min: m,
            max: k,
        })
    }

    /// A plain sequence of single predicates `p1/p2/.../pn` (each exactly one hop).
    pub fn sequence(preds: impl IntoIterator<Item = NamedNode>) -> Self {
        PathForm::Sequence(preds.into_iter().map(|p| PathForm::exact(p, 1)).collect())
    }
}

/// Reserved subject-end variable name the unroller projects onto (`?__pp_a`).
const SUBJECT_VAR: &str = "__pp_a";
/// Reserved object-end variable name the unroller projects onto (`?__pp_b`).
const OBJECT_VAR: &str = "__pp_b";
/// Prefix for the fresh intermediate join variables `?__pp_z{i}` between hops.
const INTERMEDIATE_PREFIX: &str = "__pp_z";

/// The disclosed edge-relations the unroller joins over. ONE entry per
/// **predicate IRI** that appears anywhere in the path: the set of `(subject,
/// object)` global-IRI pairs each holder disclosed for that predicate, already
/// merged into a single disclosed `(?__pp_edge_s, ?__pp_edge_o)` partial per
/// predicate (the union over holders — the federated edge set for that
/// predicate). Because the keys are disclosed global IRIs, building this set is a
/// plaintext union; NO MPC is involved.
///
/// This is the disclosed-key analogue of "the graph": the unroller never sees
/// raw holder graphs, only these per-predicate disclosed edge relations
/// (minimise data sharing, architecture §4.2). The `(a,b)` pairs are the headline
/// federation case: endpoints are disclosed global IRIs.
#[derive(Debug, Clone)]
pub struct DisclosedEdges {
    /// `(predicate IRI string, disclosed (?__pp_edge_s, ?__pp_edge_o) edge partial)`.
    by_predicate: Vec<(String, PartialResult)>,
}

impl DisclosedEdges {
    /// Build the per-predicate disclosed edge relations from each holder's
    /// disclosed edge partials.
    ///
    /// `holder_edges` is one `(predicate, partial)` per holder-disclosed edge
    /// set, where `partial.vars == [?subject_var, ?object_var]` (in that order) —
    /// i.e. exactly what a holder gets from
    /// `evaluate_local("SELECT ?s ?o WHERE { ?s <p> ?o }")`. Rows for the same
    /// predicate from different holders are UNIONED (the federated edge set).
    ///
    /// Soundness: the subject/object variable names of each partial may differ
    /// per holder; only the COLUMN ORDER (subject, object) is contractual, so we
    /// re-key every edge partial onto the reserved `(?__pp_edge_s, ?__pp_edge_o)`
    /// schema.
    pub fn from_holder_edges(
        holder_edges: impl IntoIterator<Item = (NamedNode, PartialResult)>,
    ) -> Result<Self, MpcError> {
        let edge_s = Variable::new_unchecked("__pp_edge_s");
        let edge_o = Variable::new_unchecked("__pp_edge_o");
        let mut by_predicate: Vec<(String, PartialResult)> = Vec::new();
        for (pred, part) in holder_edges {
            if part.vars.len() != 2 {
                return Err(MpcError::Protocol(format!(
                    "disclosed edge partial for <{}> must have exactly 2 columns \
                     (subject, object); got {} columns",
                    pred.as_str(),
                    part.vars.len()
                )));
            }
            // Re-key onto the canonical (?__pp_edge_s, ?__pp_edge_o) schema so the
            // holder's chosen variable names cannot influence the join.
            let rekeyed = PartialResult {
                holder: part.holder.clone(),
                vars: vec![edge_s.clone(), edge_o.clone()],
                rows: part.rows.clone(),
            };
            let key = pred.as_str().to_string();
            match by_predicate.iter_mut().find(|(k, _)| *k == key) {
                Some((_, acc)) => acc.rows.extend(rekeyed.rows),
                None => by_predicate.push((key, rekeyed)),
            }
        }
        Ok(DisclosedEdges { by_predicate })
    }

    /// The disclosed edge relation for one predicate, projected onto a chosen
    /// `(subject_var, object_var)` schema (so a hop can be slotted into a chain
    /// at any position). Returns an EMPTY relation (no rows) for a predicate with
    /// no disclosed edges — an absent predicate simply contributes nothing, which
    /// is exactly how the clear-text engine treats it.
    fn relation_for(
        &self,
        pred: &NamedNode,
        subject: &Variable,
        object: &Variable,
    ) -> PartialResult {
        let rows = self
            .by_predicate
            .iter()
            .find(|(k, _)| k == pred.as_str())
            .map(|(_, p)| p.rows.clone())
            .unwrap_or_default();
        PartialResult {
            holder: HolderId::new("federation"),
            vars: vec![subject.clone(), object.clone()],
            rows,
        }
    }

    /// The set of all distinct node IRIs occurring as a subject or object in ANY
    /// disclosed edge — the term domain over which the reflexive (length-0)
    /// identity pairs `(x,x)` are added for `{0,k}` (design §2.3, mirroring the
    /// clear-text engine's `graph_nodes`).
    fn node_domain(&self) -> Vec<Term> {
        let mut set: BTreeSet<String> = BTreeSet::new();
        let mut nodes: Vec<Term> = Vec::new();
        for (_, part) in &self.by_predicate {
            for row in &part.rows {
                for slot in row.iter().flatten() {
                    let key = format!("{slot:?}");
                    if set.insert(key) {
                        nodes.push(slot.clone());
                    }
                }
            }
        }
        nodes
    }
}

/// Evaluate a bounded property-path `?a <form> ?b` over DISCLOSED edge relations
/// and return the deduped set of endpoint pairs `(a, b)` as a [`PartialResult`]
/// with `vars == [?__pp_a, ?__pp_b]`.
///
/// This is the increment-#1 deliverable: a CRYPTO-FREE unroll-and-fold. No MPC
/// primitive runs (no secret sharing, no `secure_equal`, no `CommCounter` is
/// touched) — every hop is a [`DisclosedKeyJoin`] over disclosed global IRIs,
/// exactly as the `differential_three_holder_chain_equals_union` test does for a
/// single chain. The realized hop length is never disclosed (design §1.2).
///
/// Regime: DISCLOSED-KEY ONLY (endpoints + intermediates are disclosed global
/// IRIs). The HIDDEN-intermediate regime is sq-py8h.2 and is NOT handled here.
pub fn eval_bounded_path_disclosed(
    edges: &DisclosedEdges,
    form: &PathForm,
) -> Result<PartialResult, MpcError> {
    let subject = Variable::new_unchecked(SUBJECT_VAR);
    let object = Variable::new_unchecked(OBJECT_VAR);

    // The pair RELATION over the canonical endpoint vars (?__pp_a, ?__pp_b),
    // computed by the unroll; then deduped to a set of endpoint pairs.
    let relation = eval_form_relation(edges, form, &subject, &object)?;
    Ok(dedup_endpoint_pairs(relation, &subject, &object))
}

/// Evaluate one [`PathForm`] into its `(subject, object)` pair relation
/// (NOT yet deduped — the caller dedups once at the top). All joins are
/// disclosed-key folds; the construction is the design §2 unroll.
fn eval_form_relation(
    edges: &DisclosedEdges,
    form: &PathForm,
    subject: &Variable,
    object: &Variable,
) -> Result<PartialResult, MpcError> {
    match form {
        PathForm::Repetition(rep) => eval_repetition(edges, rep, subject, object),
        PathForm::Sequence(parts) => eval_sequence(edges, parts, subject, object),
    }
}

/// `(step){m,k}` → UNION over `ℓ in m..=k` of the exactly-`ℓ` chains, plus the
/// reflexive identity pairs when `m == 0` (design §2.2/§2.3). Each exactly-`ℓ`
/// chain is a fold of [`DisclosedKeyJoin`] over `ℓ` copies of the step's edge
/// relation through fresh intermediate vars; alternation expands each hop over
/// its alternatives (design §2.4).
fn eval_repetition(
    edges: &DisclosedEdges,
    rep: &BoundedRepetition,
    subject: &Variable,
    object: &Variable,
) -> Result<PartialResult, MpcError> {
    if rep.min > rep.max {
        return Err(MpcError::Protocol(format!(
            "bounded path {{{},{}}} has min > max",
            rep.min, rep.max
        )));
    }
    if rep.step.alternatives.is_empty() {
        return Err(MpcError::Protocol(
            "bounded path step has no predicates (empty alternation)".into(),
        ));
    }

    // Accumulate the union of all chain lengths as raw (a,b) rows; dedup happens
    // once at the top.
    let mut union_rows: Vec<Vec<Option<Term>>> = Vec::new();

    for length in rep.min..=rep.max {
        if length == 0 {
            // Length-0: the reflexive identity pairs (x,x) over the node domain
            // (design §2.3). Data-independent given the node set.
            for node in edges.node_domain() {
                union_rows.push(vec![Some(node.clone()), Some(node)]);
            }
            continue;
        }
        // exactly-`length` chains: expand the alternation across the `length`
        // hop positions → `alternatives^length` fixed BGP chains.
        for chain in alternation_chains(&rep.step, length) {
            let chain_rel = eval_fixed_chain(edges, &chain, subject, object)?;
            union_rows.extend(chain_rel.rows);
        }
    }

    Ok(PartialResult {
        holder: HolderId::new("federation"),
        vars: vec![subject.clone(), object.clone()],
        rows: union_rows,
    })
}

/// Every fixed predicate sequence of `length` hops obtained by choosing, at each
/// position, one of the step's alternatives (the §2.4 distribution). For a
/// single-predicate step this is just the one `length`-long sequence.
fn alternation_chains(step: &PathStep, length: usize) -> Vec<Vec<NamedNode>> {
    let mut chains: Vec<Vec<NamedNode>> = vec![Vec::new()];
    for _ in 0..length {
        let mut next: Vec<Vec<NamedNode>> =
            Vec::with_capacity(chains.len() * step.alternatives.len());
        for prefix in &chains {
            for alt in &step.alternatives {
                let mut extended = prefix.clone();
                extended.push(alt.clone());
                next.push(extended);
            }
        }
        chains = next;
    }
    chains
}

/// A fixed predicate chain `p_1 / p_2 / ... / p_n` (n = chain.len()) →
/// `?a p_1 ?z1 . ?z1 p_2 ?z2 . ... ?z_{n-1} p_n ?b`, evaluated as a left-to-right
/// fold of [`DisclosedKeyJoin`] on the intermediate vars, projected onto
/// `(subject, object)`. This is the §2.1 core (DISCLOSED regime): a fold of
/// crypto-free equi-joins — the same shape as the existing 3-pattern chain test.
fn eval_fixed_chain(
    edges: &DisclosedEdges,
    chain: &[NamedNode],
    subject: &Variable,
    object: &Variable,
) -> Result<PartialResult, MpcError> {
    debug_assert!(
        !chain.is_empty(),
        "exactly-0 chains handled by reflexive arm"
    );
    let n = chain.len();

    // Hop endpoint vars: ?__pp_a, ?__pp_z1, ..., ?__pp_z{n-1}, ?__pp_b.
    let hop_var = |i: usize| -> Variable {
        if i == 0 {
            subject.clone()
        } else if i == n {
            object.clone()
        } else {
            Variable::new_unchecked(format!("{INTERMEDIATE_PREFIX}{i}"))
        }
    };

    // First hop's edge relation: (?__pp_a, ?z1).
    let mut acc = edges.relation_for(&chain[0], &hop_var(0), &hop_var(1));

    // Fold remaining hops via DisclosedKeyJoin on the shared intermediate var.
    let join = DisclosedKeyJoin::new();
    for (idx, pred) in chain.iter().enumerate().skip(1) {
        let next = edges.relation_for(pred, &hop_var(idx), &hop_var(idx + 1));
        let join_var = hop_var(idx); // the shared intermediate this hop joins on
        let plan = JoinPlan {
            join_var,
            key_disclosed: true,
        };
        acc = join.join(&[acc, next], &plan)?;
    }

    // Project onto (subject, object), dropping the intermediate columns.
    Ok(project_endpoints(&acc, subject, object))
}

/// `f1 / f2 / ... / fn` → relational composition of the sub-forms' pair
/// relations, folded left-to-right via [`DisclosedKeyJoin`] on a fresh
/// connecting variable. Each sub-form is evaluated onto `(prev_end, this_end)`
/// then joined on the shared middle var (design §2.1, generalised so a sequence
/// element may itself be a bounded repetition or alternation).
fn eval_sequence(
    edges: &DisclosedEdges,
    parts: &[PathForm],
    subject: &Variable,
    object: &Variable,
) -> Result<PartialResult, MpcError> {
    if parts.is_empty() {
        return Err(MpcError::Protocol("empty path sequence".into()));
    }
    if parts.len() == 1 {
        return eval_form_relation(edges, &parts[0], subject, object);
    }

    let mid_var = |i: usize| Variable::new_unchecked(format!("{INTERMEDIATE_PREFIX}seq{i}"));

    // First sub-form onto (?__pp_a, mid1).
    let first_end = mid_var(1);
    let mut acc = eval_form_relation(edges, &parts[0], subject, &first_end)?;
    let mut prev_end = first_end;

    let join = DisclosedKeyJoin::new();
    for (idx, part) in parts.iter().enumerate().skip(1) {
        let last = idx == parts.len() - 1;
        let this_end = if last {
            object.clone()
        } else {
            mid_var(idx + 1)
        };
        let part_rel = eval_form_relation(edges, part, &prev_end, &this_end)?;
        // Dedup each sub-form's relation before joining so a sub-form's internal
        // multiplicity (e.g. two lengths reaching the same mid pair) does not
        // inflate the composition — endpoint relations are sets.
        let part_rel = dedup_endpoint_pairs(part_rel, &prev_end, &this_end);
        let acc_dedup = dedup_endpoint_pairs(acc, subject, &prev_end);
        let plan = JoinPlan {
            join_var: prev_end.clone(),
            key_disclosed: true,
        };
        acc = join.join(&[acc_dedup, part_rel], &plan)?;
        acc = project_endpoints(&acc, subject, &this_end);
        prev_end = this_end;
    }

    Ok(project_endpoints(&acc, subject, object))
}

/// Project a partial onto exactly `[subject, object]`, dropping every other
/// (intermediate) column. The endpoint vars must be present.
fn project_endpoints(part: &PartialResult, subject: &Variable, object: &Variable) -> PartialResult {
    let s_idx = part.vars.iter().position(|v| v == subject);
    let o_idx = part.vars.iter().position(|v| v == object);
    let rows = match (s_idx, o_idx) {
        (Some(si), Some(oi)) => part
            .rows
            .iter()
            .map(|r| vec![r[si].clone(), r[oi].clone()])
            .collect(),
        // A degenerate single-endpoint relation (not expected on the disclosed
        // endpoint-pair path) — keep nothing rather than guess a schema.
        _ => Vec::new(),
    };
    PartialResult {
        holder: HolderId::new("federation"),
        vars: vec![subject.clone(), object.clone()],
        rows,
    }
}

/// DEDUP the endpoint pairs to a SET (design §2.2/§2.5): each distinct `(a,b)`
/// term pair appears exactly once, regardless of how many chain lengths or
/// alternation branches reached it. The realized length is not part of the row,
/// so length is never disclosed (design §1.2). Output is sorted canonically so
/// the disclosed multiset is order-independent.
fn dedup_endpoint_pairs(
    part: PartialResult,
    subject: &Variable,
    object: &Variable,
) -> PartialResult {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut rows: Vec<Vec<Option<Term>>> = Vec::new();
    for row in part.rows {
        let key = format!("{row:?}");
        if seen.insert(key) {
            rows.push(row);
        }
    }
    rows.sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));
    PartialResult {
        holder: HolderId::new("federation"),
        vars: vec![subject.clone(), object.clone()],
        rows,
    }
}

#[cfg(test)]
mod tests;
