//! [OPUS-4.8] sq-yk6or (epic sq-pbz04, THE SUBSTRATE PAYOFF) — drive the SHARED
//! [`sparq_substrate::join`] hash-join kernels for the RDFS single-pass predicate join
//! (rdfs2 / rdfs3 / rdfs7), in place of the hand-rolled [`FxHashMap`](rustc_hash::FxHashMap)
//! adjacency probe in [`crate::rdfs::emit_consequences`].
//!
//! This is the end-to-end proof of the maintainer's "share join logic across the engine AND
//! the reasoners" goal (`research/shared-eval-substrate.md`, Phase 5): the SPARQL engine's
//! `eval_bgp_binary` and this reasoner now drive the *same* `build_table` + `probe_emit`
//! body, each supplying its OWN [`JoinKeys`](sparq_substrate::join::JoinKeys) key projection
//! and its OWN [`Budget`](sparq_substrate::join::Budget), monomorphically — there is no
//! `Box<dyn>` / `&dyn` between the probe loop and either the key projection or the budget
//! poll, so the compiler emits one specialised, inlinable body.
//!
//! # What is shared, and what is NOT
//!
//! The RDFS materialiser's per-assertion rule firing IS a relational join: an asserted triple
//! `(s, p, o)` joins, on its predicate `p`, against the already-saturated schema-closure
//! relations:
//!
//! | rule  | build relation (key = its `p` column) | conclusion           |
//! |-------|----------------------------------------|----------------------|
//! | rdfs7 | `sp_closure`: `(p, q)`  (super-property)| `(s, q, o)`         |
//! | rdfs2 | `dom_full`:   `(p, c)`  (domain class)  | `(s, type, c)`      |
//! | rdfs3 | `rng_full`:   `(p, c)`  (range class)   | `(o, type, c)`      |
//!
//! So we build ONE substrate hash table per schema relation (key = the predicate column) and
//! probe it with the ABox triples (key = the triple's predicate column). The kernel does the
//! keyed lookup + per-match row combine; this module reshapes the combined row into the rule's
//! conclusion triple (a fixed column permutation — exactly the thin layout adapter the engine
//! wraps the kernels with).
//!
//! The result is the SAME emitted multiset as [`crate::rdfs::emit_consequences`]'s hand-rolled
//! path — only the join machinery differs. Verified byte-for-byte by
//! [`crate::rdfs::tests`]' equivalence test over both paths.
//!
//! # Join shapes that genuinely cannot reuse the kernel (kept on the hand-rolled path)
//!
//! - **The `rdf:type` / subclass branch (`rdfs9`)** and the **`PropExpand` predicate-rewrite
//!   branch** (inverseOf / Symmetric / equivalentProperty orientation-swap). These are still a
//!   predicate-keyed lookup, but the orientation swap and the `type`-object key make the
//!   combine non-uniform per match; they stay on the direct adjacency path. Folding them in
//!   would not change *what* is emitted, only add reshape cost, so they are out of scope for
//!   this slice and noted here (and tracked for a follow-up).
//! - **The OWL-RL semi-naive fixpoint** (`owl.rs`): a delta-driven `Δ ⋈ full ∪ full ⋈ Δ` join
//!   with union-find `sameAs` canonicalisation. Genuinely a different (incremental, mutating)
//!   join shape than the substrate's static `&[Row]` build/probe kernel; adopting it is a
//!   separate, larger slice. Out of scope here; the RDFS path is the clean, isolated win.

use sparq_core::dict::Id;
use sparq_substrate::join::{self as sjoin, JoinKeys, NoBudget};
use sparq_substrate::rows::Row;

/// The reasoner's cooperative-cancellation hook for the shared substrate join kernels.
///
/// Materialisation runs the join to completion (the closure-level budget that bounds a
/// runaway fixpoint is a fixpoint concern installed AROUND the whole materialise call, not a
/// per-join cap — see `research/shared-eval-substrate.md` §6, the "materialisation avalanche"
/// note), so the reasoner supplies the unbounded [`NoBudget`]. Naming it through a reasoner
/// alias keeps the seam where a future closure-level budget would attach, and documents that
/// the reasoner — like the engine — owns its budget rather than the kernel imposing one.
pub(crate) type ReasonBudget = NoBudget;

/// One schema-closure build relation projected to substrate [`Row`]s `[p, target]` (predicate,
/// derived target), ready to feed [`sjoin::build_table`]. Keyed on column 0 (the predicate).
fn schema_rows(map: &rustc_hash::FxHashMap<Id, Vec<Id>>) -> Vec<Row> {
    let mut rows: Vec<Row> = Vec::new();
    for (&p, targets) in map {
        for &t in targets {
            rows.push(Row::from_slice(&[p, t]));
        }
    }
    rows
}

/// The [`JoinKeys`] for a predicate-keyed RDFS join: build column 0 (the schema relation's
/// predicate) equi-joins probe column 1 (the asserted triple's predicate). `right_only` is
/// empty because this module reshapes the combined row itself (the conclusion is a fixed
/// permutation, not a column append), matching the engine's layout-adapter pattern.
fn predicate_keys() -> JoinKeys {
    JoinKeys { key_cols: vec![(0, 1)], right_only: Vec::new() }
}

/// The substrate-driven equivalent of the rdfs2/3/7 portion of
/// [`crate::rdfs::emit_consequences`] over EVERY asserted triple. Builds one shared
/// [`sjoin::build_table`] per schema relation (sp/dom/rng), then drives [`sjoin::probe_emit`]
/// per matching pair via [`sjoin::hash_probe_serial`] with the reasoner's [`ReasonBudget`].
///
/// The `type` / subclass (`rdfs9`) branch and the `PropExpand` predicate-rewrite branch are
/// NOT computed here (see the module doc) — the caller runs those on the hand-rolled path.
/// This function emits EXACTLY the triples the hand-rolled `else`/non-`type` branch of
/// `emit_consequences` emits for the same inputs (the equivalence is asserted by a test).
pub(crate) fn sweep_predicate_join(
    asserted: &[[Id; 3]],
    ty: Id,
    sp_closure: &rustc_hash::FxHashMap<Id, Vec<Id>>,
    dom_full: &rustc_hash::FxHashMap<Id, Vec<Id>>,
    rng_full: &rustc_hash::FxHashMap<Id, Vec<Id>>,
    out: &mut Vec<[Id; 3]>,
) {
    let keys = predicate_keys();
    // The ABox probe rows: the asserted triples as `[s, p, o]` substrate rows. Column 1 (`p`)
    // is the probe key — exactly the column `emit_consequences` keys its `.get(&p)` lookups on.
    let probe: Vec<Row> = asserted.iter().map(|t| Row::from_slice(t)).collect();

    // rdfs7 — emit `(s, q, o)` for each super-property `q` of the asserted predicate.
    let sp_rows = schema_rows(sp_closure);
    let sp_tables = vec![sjoin::build_table(&sp_rows, &keys)];
    probe_into(&probe, &keys, &sp_rows, &sp_tables, out, |b, p| [p[0], b[1], p[2]]);

    // rdfs2 — domain typing: emit `(s, type, c)` for each domain class `c` of the predicate.
    let dom_rows = schema_rows(dom_full);
    let dom_tables = vec![sjoin::build_table(&dom_rows, &keys)];
    probe_into(&probe, &keys, &dom_rows, &dom_tables, out, |b, p| [p[0], ty, b[1]]);

    // rdfs3 — range typing: emit `(o, type, c)` for each range class `c` of the predicate.
    let rng_rows = schema_rows(rng_full);
    let rng_tables = vec![sjoin::build_table(&rng_rows, &keys)];
    probe_into(&probe, &keys, &rng_rows, &rng_tables, out, |b, p| [p[2], ty, b[1]]);
}

/// Drive [`sjoin::hash_probe_serial`] (the shared probe loop + [`ReasonBudget`] poll) over the
/// probe rows, collecting the build-side matches via the shared [`sjoin::probe_emit`], then
/// reshape each `build ++ probe` combined row into the rule's conclusion triple with `shape`.
///
/// `shape(build_cols, probe_cols)` receives the build row (`[p, target]`) and the probe row
/// (`[s, p, o]`) and returns the conclusion triple — a fixed permutation, the reasoner's thin
/// layout adapter over the generic kernel output.
fn probe_into(
    probe: &[Row],
    keys: &JoinKeys,
    build: &[Row],
    tables: &[rustc_hash::FxHashMap<sparq_substrate::rows::Key, sparq_substrate::rows::Posting>],
    out: &mut Vec<[Id; 3]>,
    shape: impl Fn(&[Id], &[Id]) -> [Id; 3],
) {
    // `probe_only` carries the whole probe row through, so the combined row is `build ++ probe`
    // and `shape` can read either side by fixed index. The engine wraps the kernels the same way.
    let probe_only: Vec<usize> = (0..3).collect();
    let mut combined: Vec<Row> = Vec::new();
    // `ReasonBudget` is the unbounded `NoBudget` (materialisation runs to completion); its value
    // constructor is `NoBudget`, monomorphising `exhausted` to a constant `false` the optimiser
    // deletes — byte-identical to the hand-rolled loop with no budget poll.
    let budget: ReasonBudget = NoBudget;
    sjoin::hash_probe_serial(probe, keys, build, tables, &probe_only, &budget, &mut combined);
    for row in &combined {
        // Combined row layout: build columns `[0..build_width)` then the probe columns. Build
        // width is 2 (`[p, target]`); the probe row follows at offset 2 (`[s, p, o]`).
        let (b, p) = row.split_at(2);
        out.push(shape(b, p));
    }
}
