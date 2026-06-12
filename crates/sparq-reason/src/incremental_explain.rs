//! `why()` provers for the incremental maintenance handles (the `explain` feature; child
//! module of `incremental` so it reaches the private counting state).
//!
//! # Design: reconstruct, don't double-book
//!
//! The counting engines already keep everything that DETERMINES every derivation: the
//! asserted base, the closed TBox the counts were computed against, and the exact,
//! deterministic per-triple emission function. So a support of a derived triple — (rule,
//! premises) — is *reconstructed* on demand by running the emission trace backwards from
//! small reverse indexes over the base, rather than stored per derivation (which would be a
//! second, parallel copy of the counting bookkeeping). The explanation state this adds is
//! only: the RAW TBox edge maps (the closures flatten multi-step chains away; proofs need
//! the steps back) and subject/object reverse indexes over the base.
//!
//! # Consistency model
//!
//! `why()` reads only the CURRENT maintenance state: after any `insert`/`delete`, a
//! returned proof's leaves are asserted in the current base — a retracted support is never
//! served; if an alternative support exists it is found (the search enumerates all
//! candidate emitters), and a triple that left the closure returns `None`.
//!
//! Explanations are **a witness, not all witnesses**: one deterministic derivation per
//! triple (all choice points iterate sorted).

use super::*;
use crate::explain::{
    bfs_path, id_triple_strings, sorted, ExplainOpts, ProofBuilder, ProofTree,
};

// ════════════════════════════════════════════════════════════════════════════════════════
// RDFS — MaterializedGraph
// ════════════════════════════════════════════════════════════════════════════════════════

/// Explanation state for [`MaterializedGraph`]: raw (unclosed) TBox edges + base reverse
/// indexes. Rebuilt wholesale on TBox mutations (alongside the counts), maintained
/// incrementally on ABox deltas.
#[derive(Default)]
pub(super) struct RdfsExplain {
    by_subj: FxHashMap<Id, Vec<[Id; 3]>>,
    by_obj: FxHashMap<Id, Vec<[Id; 3]>>,
    sc_raw: FxHashMap<Id, Vec<Id>>,
    sp_raw: FxHashMap<Id, Vec<Id>>,
    dom_raw: FxHashMap<Id, Vec<Id>>,
    rng_raw: FxHashMap<Id, Vec<Id>>,
}

impl RdfsExplain {
    pub(super) fn rebuild(
        &mut self,
        base: &FxHashSet<[Id; 3]>,
        sc: FxHashMap<Id, Vec<Id>>,
        sp: FxHashMap<Id, Vec<Id>>,
        dom: FxHashMap<Id, Vec<Id>>,
        rng: FxHashMap<Id, Vec<Id>>,
    ) {
        self.sc_raw = sc;
        self.sp_raw = sp;
        self.dom_raw = dom;
        self.rng_raw = rng;
        self.by_subj.clear();
        self.by_obj.clear();
        for &t in base {
            self.index(t);
        }
    }

    pub(super) fn add_triples(&mut self, added: &[[Id; 3]]) {
        for &t in added {
            self.index(t);
        }
    }

    pub(super) fn remove_triples(&mut self, removed: &[[Id; 3]]) {
        for t in removed {
            if let Some(v) = self.by_subj.get_mut(&t[0]) {
                if let Some(ix) = v.iter().position(|x| x == t) {
                    v.swap_remove(ix);
                }
            }
            if let Some(v) = self.by_obj.get_mut(&t[2]) {
                if let Some(ix) = v.iter().position(|x| x == t) {
                    v.swap_remove(ix);
                }
            }
        }
    }

    fn index(&mut self, t: [Id; 3]) {
        self.by_subj.entry(t[0]).or_default().push(t);
        self.by_obj.entry(t[2]).or_default().push(t);
    }

    /// Base triples whose emissions can have `id` as the conclusion's subject — i.e. every
    /// base triple with `id` in subject or object position (sorted + deduplicated, so the
    /// first successful trace is deterministic).
    fn candidates(&self, id: Id) -> Vec<[Id; 3]> {
        let mut c: Vec<[Id; 3]> = Vec::new();
        if let Some(v) = self.by_subj.get(&id) {
            c.extend_from_slice(v);
        }
        if let Some(v) = self.by_obj.get(&id) {
            c.extend_from_slice(v);
        }
        c.sort_unstable();
        c.dedup();
        c
    }
}

impl MaterializedGraph {
    /// One derivation of `t` from the current asserted base, or `None` if `t` is not in the
    /// closure (or a cap of [`ExplainOpts::default`] is exceeded). `dict` is only read (to
    /// render terms). See [`crate::explain`] for the proof shape and semantics.
    pub fn why(&self, dict: &Dict, t: [Id; 3]) -> Option<ProofTree> {
        self.why_with(dict, t, ExplainOpts::default())
    }

    /// [`why`](Self::why) with explicit depth/size caps.
    pub fn why_with(&self, dict: &Dict, t: [Id; 3], opts: ExplainOpts) -> Option<ProofTree> {
        let mut p = RdfsProver {
            g: self,
            dict,
            b: ProofBuilder::new(opts),
            memo: FxHashMap::default(),
        };
        let root = p.prove(t, 0)?;
        Some(p.b.finish(root))
    }
}

struct RdfsProver<'a> {
    g: &'a MaterializedGraph,
    dict: &'a Dict,
    b: ProofBuilder,
    memo: FxHashMap<[Id; 3], u32>,
}

impl RdfsProver<'_> {
    fn push(&mut self, t: [Id; 3], rule: &str, premises: Vec<u32>) -> Option<u32> {
        let ix = self.b.push(id_triple_strings(self.dict, t), rule, premises)?;
        self.memo.insert(t, ix);
        Some(ix)
    }

    fn prove(&mut self, t: [Id; 3], depth: usize) -> Option<u32> {
        if let Some(&ix) = self.memo.get(&t) {
            return Some(ix);
        }
        if depth > self.b.opts.max_depth {
            return None;
        }
        if self.g.base.contains(&t) {
            return self.push(t, "asserted", vec![]);
        }
        if self.g.schema_facts.contains(&t) {
            return self.prove_schema(t, depth);
        }
        if self.g.counts.contains_key(&t) {
            return self.prove_emitted(t, depth);
        }
        None
    }

    /// TBox-closure facts: a `subClassOf` (rdfs11) / `subPropertyOf` (rdfs5) chain over the
    /// raw (asserted) edges, folded left-associatively.
    fn prove_schema(&mut self, t: [Id; 3], depth: usize) -> Option<u32> {
        let v = &self.g.v;
        let (raw, rule) = if t[1] == v.sub_class {
            (&self.g.explain.sc_raw, "rdfs11")
        } else if t[1] == v.sub_prop {
            (&self.g.explain.sp_raw, "rdfs5")
        } else {
            return None;
        };
        let succ = |n: Id| raw.get(&n).map(|s| sorted(s)).unwrap_or_default();
        let path = if t[0] == t[2] {
            // Self-loop closure fact (c R* c via a cycle): step onto a successor first.
            sorted(&succ(t[0]))
                .into_iter()
                .find_map(|m| {
                    let mut p = vec![t[0]];
                    p.extend(bfs_path(m, t[2], succ)?);
                    Some(p)
                })?
        } else {
            bfs_path(t[0], t[2], succ)?
        };
        debug_assert!(path.len() >= 2, "a closure fact implies at least one edge");
        // First raw edge is an asserted leaf; each further edge extends the chain by one
        // rule application: (a R m), (m R n) ⊢ (a R n).
        let mut prev = self.prove([path[0], t[1], path[1]], depth + 1)?;
        for i in 2..path.len() {
            let edge = self.prove([path[i - 1], t[1], path[i]], depth + 1)?;
            let conclusion = [path[0], t[1], path[i]];
            prev = match self.memo.get(&conclusion) {
                Some(&ix) => ix,
                None => self.push(conclusion, rule, vec![prev, edge])?,
            };
        }
        Some(prev)
    }

    /// Counted (ABox-derived) facts: find an asserted emitter among the base triples
    /// touching the conclusion's subject and trace the emission back through the closed
    /// TBox, decomposing the flattened closures into single spec-rule steps.
    fn prove_emitted(&mut self, t: [Id; 3], depth: usize) -> Option<u32> {
        let v = &self.g.v;
        for src in self.g.explain.candidates(t[0]) {
            let [s, p, o] = src;
            if p == v.ty {
                // rdfs9: (o subClassOf* d), (s type o) ⊢ (s type d).
                if t[1] == v.ty && t[0] == s && contains(self.g.sc_closure.get(&o), t[2]) {
                    let schema = self.prove([o, v.sub_class, t[2]], depth + 1)?;
                    let src_n = self.prove(src, depth + 1)?;
                    return self.push(t, "rdfs9", vec![schema, src_n]);
                }
                continue;
            }
            // rdfs7: (p subPropertyOf* q), (s p o) ⊢ (s q o).
            if t[0] == s && t[2] == o && contains(self.g.sp_closure.get(&p), t[1]) {
                let schema = self.prove([p, v.sub_prop, t[1]], depth + 1)?;
                let src_n = self.prove(src, depth + 1)?;
                return self.push(t, "rdfs7", vec![schema, src_n]);
            }
            if t[1] != v.ty {
                continue;
            }
            // rdfs2 (+rdfs7/rdfs9): domain typing of the subject.
            if t[0] == s && contains(self.g.dom_full.get(&p), t[2]) {
                if let Some(ix) = self.trace_typing(t, src, false, depth) {
                    return Some(ix);
                }
            }
            // rdfs3 (+rdfs7/rdfs9): range typing of the object.
            if t[0] == o && contains(self.g.rng_full.get(&p), t[2]) {
                if let Some(ix) = self.trace_typing(t, src, true, depth) {
                    return Some(ix);
                }
            }
        }
        None
    }

    /// Decompose a flattened `dom_full`/`rng_full` emission `(s p o) ⊢ (x type c)` into
    /// spec-rule steps: optional rdfs7 up to the super-property `q` carrying the raw
    /// domain/range, the rdfs2/rdfs3 application, and an optional rdfs9 up the subclass
    /// chain. `range` selects rdfs3 (typing the object) over rdfs2 (typing the subject).
    fn trace_typing(&mut self, t: [Id; 3], src: [Id; 3], range: bool, depth: usize) -> Option<u32> {
        let v = &self.g.v;
        let [s, p, o] = src;
        let (raw, dr_pred, dr_rule) = if range {
            (&self.g.explain.rng_raw, v.range, "rdfs3")
        } else {
            (&self.g.explain.dom_raw, v.domain, "rdfs2")
        };
        let c = t[2];
        // Properties whose raw domain/range can type this triple: p and its super-properties.
        let mut props = vec![p];
        if let Some(qs) = self.g.sp_closure.get(&p) {
            props.extend(qs.iter().copied());
        }
        props.sort_unstable();
        props.dedup();
        for q in props {
            let Some(c0s) = raw.get(&q) else { continue };
            for &c0 in &sorted(c0s) {
                if c0 != c && !contains(self.g.sc_closure.get(&c0), c) {
                    continue;
                }
                // (s q o): the source itself, or one rdfs7 step up.
                let edge_triple = [s, q, o];
                if edge_triple == t {
                    continue; // pathological self-reference; try the next decomposition
                }
                let edge = if q == p {
                    self.prove(src, depth + 1)?
                } else if let Some(&ix) = self.memo.get(&edge_triple) {
                    ix
                } else {
                    let schema = self.prove([p, v.sub_prop, q], depth + 1)?;
                    let src_n = self.prove(src, depth + 1)?;
                    self.push(edge_triple, "rdfs7", vec![schema, src_n])?
                };
                // rdfs2/rdfs3: (q domain/range c0), (s q o) ⊢ (x type c0).
                let dr_leaf = self.prove([q, dr_pred, c0], depth + 1)?;
                let typed_triple = [if range { o } else { s }, v.ty, c0];
                if c0 == c {
                    return self.push(t, dr_rule, vec![dr_leaf, edge]);
                }
                let typed = if let Some(&ix) = self.memo.get(&typed_triple) {
                    ix
                } else {
                    self.push(typed_triple, dr_rule, vec![dr_leaf, edge])?
                };
                // rdfs9 up the subclass closure to c.
                let schema = self.prove([c0, v.sub_class, c], depth + 1)?;
                return self.push(t, "rdfs9", vec![schema, typed]);
            }
        }
        None
    }
}

/// Is `x` in the optional closure entry?
fn contains(entry: Option<&Vec<Id>>, x: Id) -> bool {
    entry.is_some_and(|v| v.contains(&x))
}
