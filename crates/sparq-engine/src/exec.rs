//! Physical execution: BGP via greedy-ordered merge/hash joins over the
//! permutation indexes, plus OPTIONAL / UNION / MINUS / BIND / VALUES,
//! aggregation (GROUP BY / HAVING via Filter), ORDER BY, sub-SELECT and the
//! solution modifiers. All intermediate results are id-level (`Bindings`);
//! values computed at query time (BIND, aggregates) get ids in a per-query
//! `LocalVocab`, mirroring QLever's local vocabulary.

use crate::QueryResult;
use oxrdf::vocab::xsd;
use oxrdf::{BlankNode, Literal, NamedOrBlankNode, Term, Variable};
use rustc_hash::FxHashMap;
use sparq_core::dict::{self, Id, NO_ID};
use sparq_core::store::Pattern as IdPattern;
use sparq_core::temporal::{Temporal, Timeline};
use sparq_core::Graph;
use rustc_hash::FxHashSet;
use spargebra::algebra::{
    AggregateExpression, AggregateFunction, Expression, GraphPattern, OrderExpression, PropertyPathExpression,
};
use smallvec::SmallVec;
use spargebra::term::{GroundTerm, NamedNodePattern, TermPattern, TriplePattern};
use std::cmp::Ordering;

// ---- Cooperative query budget (T15 server hardening) -------------------------
//
// A thread-local, cooperatively-checked budget installed by the
// `*_with_budget` entry points in lib.rs. Checked at COARSE sites only: operator
// entry (`eval_graph_pattern`) and once per outer iteration / key group of the
// big row-producing loops — never in inner loops. Evaluation is synchronous on
// the installing thread (rayon offloads block the caller), so a thread-local
// suffices; the rayon-parallel branches use a captured `Limits` snapshot to cap
// their own work and the next on-thread check converts that into the error.
pub(crate) mod budget {
    use crate::QueryBudget;
    use std::cell::Cell;

    /// The installed limits, flattened for a cheap per-check read.
    #[derive(Clone, Copy)]
    pub(crate) struct Limits {
        on: bool,
        #[cfg(not(target_arch = "wasm32"))]
        deadline: Option<std::time::Instant>,
        max_rows: usize,
    }

    const OFF: Limits = Limits {
        on: false,
        #[cfg(not(target_arch = "wasm32"))]
        deadline: None,
        max_rows: usize::MAX,
    };

    impl Limits {
        /// Pure (no thread-local) exhaustion test for rayon closures, where the
        /// installing thread's sticky flag is out of reach: a worker that sees
        /// `hit` stops producing, and the caller's next on-thread check fires
        /// (the deadline is global time; a hit row cap leaves `rows > max_rows`).
        /// Only the rayon-parallel branches call this (and `snapshot`); the
        /// non-parallel (wasm) build compiles them out.
        #[cfg_attr(not(feature = "parallel"), allow(dead_code))]
        #[inline]
        pub(crate) fn hit(&self, rows: usize) -> bool {
            if !self.on {
                return false;
            }
            if rows > self.max_rows {
                return true;
            }
            #[cfg(not(target_arch = "wasm32"))]
            if self.deadline.is_some_and(|d| std::time::Instant::now() >= d) {
                return true;
            }
            false
        }
    }

    thread_local! {
        static ACTIVE: Cell<Limits> = const { Cell::new(OFF) };
        static EXCEEDED: Cell<Option<&'static str>> = const { Cell::new(None) };
    }

    /// Clears the budget when the `*_with_budget` entry point returns (also on
    /// error/unwind, so a poisoned thread never leaks a stale budget).
    pub(crate) struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            ACTIVE.with(|a| a.set(OFF));
            EXCEEDED.with(|e| e.set(None));
        }
    }

    pub(crate) fn install(b: &QueryBudget) -> Guard {
        #[cfg(not(target_arch = "wasm32"))]
        let on = b.deadline.is_some() || b.max_rows.is_some();
        #[cfg(target_arch = "wasm32")]
        let on = b.max_rows.is_some();
        ACTIVE.with(|a| {
            a.set(Limits {
                on,
                #[cfg(not(target_arch = "wasm32"))]
                deadline: b.deadline,
                max_rows: b.max_rows.unwrap_or(usize::MAX),
            })
        });
        EXCEEDED.with(|e| e.set(None));
        Guard
    }

    /// Snapshot of the installed limits, for the rayon-parallel branches.
    #[cfg_attr(not(feature = "parallel"), allow(dead_code))]
    #[inline]
    pub(crate) fn snapshot() -> Limits {
        ACTIVE.with(|a| a.get())
    }

    /// `true` while a deadline / row-cap budget is installed. [OPUS-4.8] roborev 1538:
    /// the streaming-JSON fast path uses this to take the cooperative SERIAL loop (which
    /// breaks every 1024 rows) instead of the parallel fan-out that materialises every
    /// matching fragment before any budget check — so `--max-results` / timeouts bound
    /// CPU and memory, not just the final response.
    #[cfg_attr(not(feature = "parallel"), allow(dead_code))]
    #[inline]
    pub(crate) fn active() -> bool {
        ACTIVE.with(|a| a.get().on)
    }

    /// `true` once the budget is exhausted (sticky) — row-producing loops break
    /// on it; `rows` is the loop's current output size.
    #[inline]
    pub(crate) fn exhausted(rows: usize) -> bool {
        let a = ACTIVE.with(|c| c.get());
        if !a.on {
            return false;
        }
        if EXCEEDED.with(|e| e.get()).is_some() {
            return true;
        }
        if rows > a.max_rows {
            EXCEEDED.with(|e| e.set(Some("max-rows")));
            return true;
        }
        #[cfg(not(target_arch = "wasm32"))]
        if a.deadline.is_some_and(|d| std::time::Instant::now() >= d) {
            EXCEEDED.with(|e| e.set(Some("timeout")));
            return true;
        }
        false
    }

    /// Propagates an exhausted budget as the query error.
    #[inline]
    pub(crate) fn check(rows: usize) -> Result<(), String> {
        if exhausted(rows) {
            let why = EXCEEDED.with(|e| e.get()).unwrap_or("timeout");
            return Err(format!("query budget exceeded ({why})"));
        }
        Ok(())
    }

    /// Caps a speculative `Vec` pre-allocation while a budget is active, so a
    /// budgeted cross-product cannot allocate its full (possibly astronomical)
    /// output up front before the first cooperative check fires.
    #[inline]
    pub(crate) fn cap_alloc(cap: usize) -> usize {
        let a = ACTIVE.with(|c| c.get());
        if !a.on {
            return cap;
        }
        cap.min(a.max_rows.saturating_add(1)).min(1 << 20)
    }
}

// ---- EXPLAIN ANALYZE operator trace (T22) -------------------------------------
//
// A thread-local trace installed only by `explain_analyze*`: every
// `eval_graph_pattern` operator entry records a node (label, depth, output rows,
// wall time). When no trace is installed the entire mechanism is one thread-local
// `Cell<bool>` read per operator entry — the same cost class as the budget check
// that already sits there, and nothing on any per-row path.
pub(crate) mod trace {
    use std::cell::{Cell, RefCell};

    /// One traced operator (pre-order position + depth reconstruct the tree).
    pub(crate) struct Node {
        pub(crate) label: String,
        pub(crate) depth: usize,
        pub(crate) rows: usize,
        pub(crate) nanos: u64,
    }

    thread_local! {
        static ENABLED: Cell<bool> = const { Cell::new(false) };
        static DEPTH: Cell<usize> = const { Cell::new(0) };
        static NODES: RefCell<Vec<Node>> = const { RefCell::new(Vec::new()) };
    }

    /// Disables tracing (and clears any partial trace) when the installing entry
    /// point returns — also on error/unwind, so a failed query never leaks a trace.
    pub(crate) struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            ENABLED.with(|e| e.set(false));
            DEPTH.with(|d| d.set(0));
            NODES.with(|n| n.borrow_mut().clear());
        }
    }

    pub(crate) fn install() -> Guard {
        ENABLED.with(|e| e.set(true));
        DEPTH.with(|d| d.set(0));
        NODES.with(|n| n.borrow_mut().clear());
        Guard
    }

    #[inline]
    pub(crate) fn enabled() -> bool {
        ENABLED.with(|e| e.get())
    }

    /// Opens a node, returning its index for [`exit`] to fill in.
    pub(crate) fn enter(label: String) -> usize {
        let depth = DEPTH.with(|d| {
            let v = d.get();
            d.set(v + 1);
            v
        });
        NODES.with(|n| {
            let mut n = n.borrow_mut();
            n.push(Node { label, depth, rows: 0, nanos: 0 });
            n.len() - 1
        })
    }

    pub(crate) fn exit(idx: usize, rows: usize, nanos: u64) {
        DEPTH.with(|d| d.set(d.get().saturating_sub(1)));
        NODES.with(|n| {
            if let Some(node) = n.borrow_mut().get_mut(idx) {
                node.rows = rows;
                node.nanos = nanos;
            }
        });
    }

    /// Drains the recorded nodes (pre-order). Call before the guard drops.
    pub(crate) fn take() -> Vec<Node> {
        NODES.with(|n| std::mem::take(&mut *n.borrow_mut()))
    }
}

// ---- Extension-function registry (SPARQL 17.6) --------------------------------
//
// A thread-local registry installed by the `*_with_functions` entry points /
// `with_functions` in lib.rs, consulted ONLY in `eval_function`'s `F::Custom` arm
// after the XSD constructor-cast check misses. The registry-free entry points
// never install one, so their behaviour and hot-path cost are exactly the
// pre-registry ones (the `F::Custom` arm gains one thread-local `Option` check,
// on a path that previously always errored). Like the budget, evaluation is
// synchronous on the installing thread; the three rayon-parallel branches that
// evaluate expressions off-thread (FILTER, BIND, aggregates) snapshot the
// registry and re-install it around each worker item via [`worker_install`].
pub(crate) mod functions {
    use crate::FunctionRegistry;
    use std::cell::RefCell;
    use std::sync::Arc;

    thread_local! {
        static ACTIVE: RefCell<Option<Arc<FunctionRegistry>>> = const { RefCell::new(None) };
    }

    /// Uninstalls the registry when the installing entry point returns (also on
    /// error/unwind, so a poisoned thread never leaks a stale registry).
    pub(crate) struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            ACTIVE.with(|a| a.borrow_mut().take());
        }
    }

    pub(crate) fn install(fns: &FunctionRegistry) -> Guard {
        ACTIVE.with(|a| *a.borrow_mut() = Some(Arc::new(fns.clone())));
        Guard
    }

    /// Snapshot of the installed registry for the rayon-parallel branches
    /// (`None` — the overwhelmingly common case — makes [`worker_install`] free).
    // [OPUS-4.8] Only the `parallel`-gated worker branches snapshot the registry (see
    // `worker_install`, gated the same way); match the sibling `limits::snapshot` so the
    // `-D warnings` clippy gate stays clean in no-parallel/wasm builds.
    #[cfg_attr(not(feature = "parallel"), allow(dead_code))]
    pub(crate) fn snapshot() -> Option<Arc<FunctionRegistry>> {
        ACTIVE.with(|a| a.borrow().clone())
    }

    /// Scoped re-install of a snapshot inside a rayon worker item. Restores the
    /// PREVIOUS thread-local value on drop: rayon runs some items on the
    /// installing thread itself, whose registry must survive the item.
    pub(crate) struct WorkerGuard(Option<Option<Arc<FunctionRegistry>>>);
    impl Drop for WorkerGuard {
        fn drop(&mut self) {
            if let Some(prev) = self.0.take() {
                ACTIVE.with(|a| *a.borrow_mut() = prev);
            }
        }
    }

    #[cfg_attr(not(feature = "parallel"), allow(dead_code))]
    pub(crate) fn worker_install(snap: &Option<Arc<FunctionRegistry>>) -> WorkerGuard {
        match snap {
            None => WorkerGuard(None),
            Some(fns) => WorkerGuard(Some(ACTIVE.with(|a| a.borrow_mut().replace(fns.clone())))),
        }
    }

    /// The extension function registered for `iri`, if a registry is installed
    /// and contains it.
    pub(crate) fn lookup(iri: &str) -> Option<crate::ExtFn> {
        ACTIVE.with(|a| a.borrow().as_ref().and_then(|fns| fns.get(iri).cloned()))
    }
}

// ---- Dataset view (L1) ---------------------------------------------------------
//
// A thread-local named-graph-subset view installed by the `*_view` entry points /
// `with_view` in lib.rs (see research/solid-access-control-design.md §5 in the
// sparq-solid worktree). Consulted at the only places a dataset is enumerated:
// `eval_graph_named` (named-graph visibility), the BGP/path entries
// (`DefaultGraphMode::Empty`) and `dataset::build_active` (FROM / FROM NAMED
// intersect with the view — restriction composes, never widens). A non-visible
// graph must be INDISTINGUISHABLE from an absent one; that is the security
// property. Like the budget/functions guards, evaluation is synchronous on the
// installing thread; the rayon-parallel expression branches (FILTER / BIND /
// aggregates — the only off-thread paths that can re-enter pattern evaluation,
// via EXISTS) snapshot the view and re-install it around each worker item.
pub(crate) mod view {
    use crate::{DatasetView, DefaultGraphMode};
    use oxrdf::Term;
    use rustc_hash::FxHashSet;
    use std::cell::RefCell;
    use std::sync::Arc;

    /// The installed view, plus the "inside GRAPH" suspend flag:
    /// `eval_graph_named` swaps evaluation to the named sub-`Graph`, whose inner
    /// patterns must NOT be empty-defaulted (only the TOP-LEVEL graph scope is).
    #[derive(Clone, Default)]
    pub(crate) struct State {
        named: Option<Arc<FxHashSet<Term>>>,
        default_empty: bool,
        suspended: bool,
    }

    thread_local! {
        static ACTIVE: RefCell<State> = RefCell::new(State::default());
    }

    /// Restores the pre-install state when the installing entry point returns
    /// (also on error/unwind, so a poisoned thread never leaks a stale view).
    pub(crate) struct Guard(State);
    impl Drop for Guard {
        fn drop(&mut self) {
            ACTIVE.with(|a| *a.borrow_mut() = std::mem::take(&mut self.0));
        }
    }

    pub(crate) fn install(v: &DatasetView) -> Guard {
        let new = State {
            named: Some(Arc::clone(&v.named)),
            default_empty: matches!(v.default, DefaultGraphMode::Empty),
            suspended: false,
        };
        Guard(ACTIVE.with(|a| std::mem::replace(&mut *a.borrow_mut(), new)))
    }

    /// Fully suspends the view (named filter AND empty default) for a scope —
    /// used by the entry points once `dataset::build_active` has folded the view
    /// into a dataset-clause ACTIVE graph: the restriction is already applied,
    /// and re-filtering would make a non-visible FROM NAMED graph behave
    /// differently from an absent one (both must be the EMPTY active graph).
    pub(crate) fn suspend_all() -> Guard {
        Guard(ACTIVE.with(|a| std::mem::take(&mut *a.borrow_mut())))
    }

    /// RAII suspension of the empty-default short-circuit only, for GRAPH scope
    /// (the named-graph visibility filter stays active). Restores the previous
    /// flag on drop, so nested scopes compose.
    pub(crate) struct GraphScope(bool);
    impl Drop for GraphScope {
        fn drop(&mut self) {
            ACTIVE.with(|a| a.borrow_mut().suspended = self.0);
        }
    }

    pub(crate) fn enter_graph() -> GraphScope {
        GraphScope(ACTIVE.with(|a| std::mem::replace(&mut a.borrow_mut().suspended, true)))
    }

    /// `true` when `name` is a visible named graph under the installed view
    /// (always true with no view installed).
    #[inline]
    pub(crate) fn allows(name: &Term) -> bool {
        ACTIVE.with(|a| a.borrow().named.as_ref().is_none_or(|s| s.contains(name)))
    }

    /// `true` when the view's default graph is EMPTY at the current scope —
    /// false with no view, under `StoreDefault`, or inside a GRAPH pattern.
    #[inline]
    pub(crate) fn default_is_empty() -> bool {
        ACTIVE.with(|a| {
            let s = a.borrow();
            s.default_empty && !s.suspended
        })
    }

    /// Snapshot of the installed view for the rayon-parallel expression branches
    /// (`None` — no view, the common case — makes [`worker_install`] free).
    #[cfg_attr(not(feature = "parallel"), allow(dead_code))]
    pub(crate) fn snapshot() -> Option<State> {
        ACTIVE.with(|a| {
            let s = a.borrow();
            (s.named.is_some() || s.default_empty).then(|| s.clone())
        })
    }

    /// Scoped re-install of a snapshot inside a rayon worker item. Restores the
    /// PREVIOUS thread-local value on drop: rayon runs some items on the
    /// installing thread itself, whose view must survive the item.
    pub(crate) struct WorkerGuard(Option<State>);
    impl Drop for WorkerGuard {
        fn drop(&mut self) {
            if let Some(prev) = self.0.take() {
                ACTIVE.with(|a| *a.borrow_mut() = prev);
            }
        }
    }

    #[cfg_attr(not(feature = "parallel"), allow(dead_code))]
    pub(crate) fn worker_install(snap: &Option<State>) -> WorkerGuard {
        match snap {
            None => WorkerGuard(None),
            Some(s) => WorkerGuard(Some(ACTIVE.with(|a| std::mem::replace(&mut *a.borrow_mut(), s.clone())))),
        }
    }
}

/// One solution row: the ids bound to each of a [`Bindings`]' variables. Inlined
/// up to 4 columns (the common case) so a join produces no heap allocation per
/// row — the dominant cost on large join results.
type Row = SmallVec<[Id; 4]>;

/// Result sizes at/above which the embarrassingly-parallel materialisation steps
/// (row building, term reconstruction) are worth handing to rayon; below it the
/// thread hand-off costs more than it saves. Native only (the wasm build has no
/// `parallel` feature, so these paths compile to the sequential loops).
#[cfg(feature = "parallel")]
const PAR_THRESHOLD: usize = 50_000;

/// A join / group key (the ids of the shared or grouping columns). Inlined up to
/// 2 columns — most joins are on one or two variables — so building a hash table
/// or probing it allocates nothing per key.
type Key = SmallVec<[Id; 2]>;

/// A hash-table posting list (row indices sharing a key). Inlined up to 2 — many
/// join keys (and almost all OPTIONAL keys) match only one or two rows — so the
/// build allocates nothing per bucket in the common case.
type Posting = SmallVec<[usize; 2]>;

/// Ids at or above this base index into the per-query [`LocalVocab`] instead of the graph
/// dictionary. It sits ABOVE the dictionary range `[1, INLINE_BASE)` and the inline-integer
/// range `[INLINE_BASE, INLINE_BASE + 2^30)`, i.e. at `INLINE_BASE + 2^30 = 3·2^30`, leaving
/// the local vocab `[3·2^30, 2^32)` (≈1.07B query-computed terms — far more than any query).
const LOCAL_BASE: Id = dict::INLINE_BASE + (1 << 30);

#[inline]
fn is_local(id: Id) -> bool {
    id >= LOCAL_BASE
}

/// Per-query vocabulary for terms produced during evaluation (BIND, aggregates)
/// that are not in the graph dictionary.
#[derive(Default)]
pub struct LocalVocab {
    terms: Vec<Term>,
    ids: FxHashMap<Term, Id>,
    /// Parallel to `terms`: the f64 value of each numeric local literal (NaN
    /// otherwise) — the local-vocab twin of the graph's `numerics` cache, so a
    /// FILTER/comparison over a BIND-computed numeric does not clone + re-parse
    /// the term per row.
    nums: Vec<f64>,
}

impl LocalVocab {
    /// Interns a term, returning a stable id: equal terms get the same id so
    /// DISTINCT, GROUP BY, joins and equality work on computed values.
    fn intern(&mut self, t: Term) -> Id {
        if let Some(&id) = self.ids.get(&t) {
            return id;
        }
        let id = LOCAL_BASE + self.terms.len() as Id;
        self.nums.push(match &t {
            Term::Literal(l) if is_numeric_dt(l) => l.value().parse::<f64>().unwrap_or(f64::NAN),
            _ => f64::NAN,
        });
        self.terms.push(t.clone());
        self.ids.insert(t, id);
        id
    }
    fn term(&self, id: Id) -> &Term {
        &self.terms[(id - LOCAL_BASE) as usize]
    }
    /// The cached numeric value of a local id (`None` for non-numeric terms) —
    /// exactly what `as_num` of the materialised term would return.
    #[inline]
    fn numeric(&self, id: Id) -> Option<f64> {
        let v = self.nums[(id - LOCAL_BASE) as usize];
        if v.is_nan() {
            None
        } else {
            Some(v)
        }
    }
}

/// Resolves an id to its term (graph dictionary or local vocab); `NO_ID` -> None.
fn term_of(graph: &Graph, local: &LocalVocab, id: Id) -> Option<Term> {
    if id == NO_ID {
        None
    } else if is_local(id) {
        Some(local.term(id).clone())
    } else {
        Some(graph.dict.term(id))
    }
}

/// Intermediate result: rows of one id per `vars[i]` (`NO_ID` = unbound).
/// `sorted_by` records the variable the rows are sorted on (if any), enabling a
/// merge join instead of a hash table.
struct Bindings {
    vars: Vec<Variable>,
    rows: Vec<Row>,
    sorted_by: Option<Variable>,
}

impl Bindings {
    fn col(&self, v: &Variable) -> Option<usize> {
        self.vars.iter().position(|x| x == v)
    }
    fn unsorted(vars: Vec<Variable>, rows: Vec<Row>) -> Self {
        Bindings { vars, rows, sorted_by: None }
    }
}

pub fn eval_select(graph: &Graph, pattern: &GraphPattern) -> Result<QueryResult, String> {
    let mut local = LocalVocab::default();
    let bindings = eval_modified(graph, &mut local, pattern)?;
    // Final budget gate: converts a row-capped/timed-out evaluation (including the
    // uninstrumented rayon branches) into the error before the expensive term
    // materialisation below.
    budget::check(bindings.rows.len())?;

    // SELECT * exposes only real variables, never synthetic blank-node variables.
    let out_vars: Vec<Variable> = bindings
        .vars
        .iter()
        .filter(|v| !v.as_str().starts_with(BNODE_VAR_PREFIX))
        .cloned()
        .collect();

    let col_of: Vec<Option<usize>> = out_vars.iter().map(|v| bindings.col(v)).collect();
    // Materialise each solution row's terms. This reconstructs an `oxrdf::Term` (an
    // IRI/string allocation) per cell and is the dominant cost of returning a large
    // result — but every row is independent, so do it in parallel on native (the wasm
    // build has no threads and keeps the sequential path). Order is preserved.
    let materialise = |row: &Row| -> Vec<Option<Term>> {
        col_of.iter().map(|c| c.and_then(|i| term_of(graph, &local, row[i]))).collect()
    };
    #[cfg(feature = "parallel")]
    let rows: Vec<Vec<Option<Term>>> = if bindings.rows.len() >= PAR_THRESHOLD {
        use rayon::prelude::*;
        bindings.rows.par_iter().map(materialise).collect()
    } else {
        bindings.rows.iter().map(materialise).collect()
    };
    #[cfg(not(feature = "parallel"))]
    let rows: Vec<Vec<Option<Term>>> = bindings.rows.iter().map(materialise).collect();
    Ok(QueryResult { vars: out_vars, rows })
}

/// Writes one binding's JSON value directly from its id — no intermediate `oxrdf::Term`
/// for the common (dictionary) case, which is the allocator-bound cost of materialising.
#[inline]
fn write_id_json(graph: &Graph, local: &LocalVocab, id: Id, s: &mut String) {
    if dict::is_inline(id) {
        crate::json::inline_int_json(s, id - dict::INLINE_BASE);
    } else if is_local(id) {
        // Computed terms (BIND / aggregates) are rare; reconstruct just these.
        crate::json::term_to_json(s, local.term(id));
    } else {
        write_store_id_json(graph, id, s);
    }
}

/// Writes one binding value's JSON directly from a STORE id (never a local-vocab id,
/// so no `LocalVocab` needed) — for the streaming single-pattern scan path. An RDF 1.2
/// triple-term id recurses through its component ids (which are always store/inline
/// ids), producing the SPARQL 1.2 `{"type":"triple","value":{…}}` JSON encoding.
fn write_store_id_json(graph: &Graph, id: Id, s: &mut String) {
    if dict::is_inline(id) {
        crate::json::inline_int_json(s, id - dict::INLINE_BASE);
    } else {
        match graph.dict.term_parts(id) {
            dict::TermParts::Triple([ts, tp, to]) => {
                s.push_str("{\"type\":\"triple\",\"value\":{\"subject\":");
                write_store_id_json(graph, ts, s);
                s.push_str(",\"predicate\":");
                write_store_id_json(graph, tp, s);
                s.push_str(",\"object\":");
                write_store_id_json(graph, to, s);
                s.push_str("}}");
            }
            parts => crate::json::parts_to_json(s, parts),
        }
    }
}

/// Appends a serialised fragment to the chunk list. Chunked mode (`flush = Some`)
/// MOVES the fragment in as its own chunk — never a second copy of result bytes;
/// single-string mode (`flush = None`) concatenates onto the tail (the pre-existing
/// behaviour of the parallel join). Concatenation order — and so the byte stream —
/// is identical either way.
// [OPUS-4.8] Both call sites are in `parallel`-gated blocks (the sequential/wasm paths
// `push` directly); gate the helper too so the `-D warnings` clippy gate stays clean in
// no-parallel/wasm builds.
#[cfg_attr(not(feature = "parallel"), allow(dead_code))]
fn emit_chunk(chunks: &mut Vec<String>, frag: String, flush: Option<usize>) {
    match chunks.last_mut() {
        Some(tail) if flush.is_none() => tail.push_str(&frag),
        _ => chunks.push(frag),
    }
}

/// Concatenates JSON chunks back into the single-string form (the non-streamed API).
fn join_chunks(mut chunks: Vec<String>) -> String {
    if chunks.len() == 1 {
        chunks.pop().expect("len checked")
    } else {
        chunks.concat()
    }
}

/// Streaming fast path: `SELECT ... WHERE { <one triple pattern> }` (optionally
/// projected) serialised straight from the index scan to SPARQL-JSON — no `Bindings`,
/// no per-row `Row`, no `Term`. Returns `None` if the query is not this shape (the
/// caller falls back to the general evaluator). The vector-at-a-time idea for the most
/// common (single-pattern) browser query. Output is a chunk sequence (see
/// [`eval_select_json_chunks`]); concatenated it is the exact JSON document.
fn single_pattern_scan_json(graph: &Graph, pattern: &GraphPattern, flush: Option<usize>) -> Option<Vec<String>> {
    // zk-trace: this streaming path serialises straight from the scan without
    // materialising Bindings, so it never hits the scan-recording hook. Fall
    // through to the Bindings path (which records) while a recorder is armed —
    // result-equivalent, only the plan changes.
    #[cfg(feature = "zk")]
    if crate::zk::enabled() {
        return None;
    }
    if view::default_is_empty() {
        return None; // empty-default view: the general path short-circuits at the BGP
    }
    let (proj, inner): (Option<&[Variable]>, &GraphPattern) = match pattern {
        GraphPattern::Project { inner, variables } => (Some(variables), inner),
        other => (None, other),
    };
    if !is_conjunctive(inner) {
        return None;
    }
    let mut patterns = Vec::new();
    let mut filters = Vec::new();
    flatten_conjunction(inner, &mut patterns, &mut filters);
    if patterns.len() != 1 {
        return None;
    }
    // Only pushed-down sargable numeric FILTER(s); a residual filter needs the general
    // expression evaluator.
    let (pat_filters, residual) = split_sargable(&patterns, &filters);
    if !residual.is_empty() {
        return None;
    }
    let filt: Option<(usize, ScanCmp)> = pat_filters[0];

    let (id_pat, pos_vars, unsat) = prepare_pattern(graph, &patterns[0]).ok()?;
    if !distinct_pattern_vars(&pos_vars) {
        return None; // a repeated variable needs the consistency check — use the general path
    }
    let out_vars: Vec<Variable> = match proj {
        Some(vs) => vs.iter().filter(|v| !v.as_str().starts_with(BNODE_VAR_PREFIX)).cloned().collect(),
        None => pos_vars.iter().flatten().filter(|v| !v.as_str().starts_with(BNODE_VAR_PREFIX)).cloned().collect(),
    };
    let cols: Vec<Option<usize>> = out_vars.iter().map(|v| pos_vars.iter().position(|x| x.as_ref() == Some(v))).collect();

    let mut head = String::from("{\"head\":{\"vars\":[");
    for (i, v) in out_vars.iter().enumerate() {
        if i > 0 {
            head.push(',');
        }
        head.push('"');
        crate::json::escape_into(&mut head, v.as_str());
        head.push('"');
    }
    head.push_str("]},\"results\":{\"bindings\":[");
    if unsat {
        head.push_str("]}}");
        return Some(vec![head]);
    }

    let scan = match filt {
        Some((c, _)) => graph.store.scan_sorted(&id_pat, c),
        None => graph.store.scan(&id_pat),
    };
    // Range-prune an all-inline filter column (identical to scan_to_bindings) so the
    // order matches the general path exactly (byte-identical output).
    let mut scan_rows: &[[Id; 3]] = scan.rows.as_ref();
    if let Some((fpos, cmp)) = filt {
        let actual_sort = scan.perm.order().into_iter().find(|&c| id_pat[c].is_none());
        if actual_sort == Some(fpos) && scan_rows.first().is_some_and(|r| dict::is_inline(scan.to_spo(r)[fpos])) {
            scan_rows = match inline_pass_values(cmp) {
                Some((lo, hi)) => {
                    let (lo_id, hi_id) = (dict::INLINE_BASE + lo, dict::INLINE_BASE + hi);
                    let start = scan_rows.partition_point(|r| scan.to_spo(r)[fpos] < lo_id);
                    let end = scan_rows.partition_point(|r| scan.to_spo(r)[fpos] <= hi_id);
                    &scan_rows[start..end]
                }
                None => &[],
            };
        }
    }
    // Per-row check: a no-op on a fully range-pruned slice; required for a mixed-datatype
    // column where pruning was skipped.
    let passes = |row: &[Id; 3]| -> bool {
        match filt {
            Some((fpos, cmp)) => cmp.test_id(graph, scan.to_spo(row)[fpos]),
            None => true,
        }
    };
    let write_row = |row: &[Id; 3], s: &mut String| {
        let spo = scan.to_spo(row);
        s.push('{');
        let mut first = true;
        for (vi, &col) in cols.iter().enumerate() {
            let Some(c) = col else { continue };
            if !first {
                s.push(',');
            }
            first = false;
            s.push('"');
            crate::json::escape_into(s, out_vars[vi].as_str());
            s.push_str("\":");
            write_store_id_json(graph, spo[c], s);
        }
        s.push('}');
    };

    let mut s = head;
    // [OPUS-4.8] roborev 1538: only take the parallel fan-out when NO budget is active. The
    // parallel path builds every matching JSON fragment before it can check the budget, so a
    // row cap / deadline would not bound CPU or memory — it would serialise the full result and
    // only then fail. With a budget installed we fall through to the cooperative serial loop
    // below, which checks `budget::exhausted` every 1024 rows and stops early.
    #[cfg(feature = "parallel")]
    if scan_rows.len() >= PAR_THRESHOLD && !budget::active() {
        use rayon::prelude::*;
        // One string per chunk (≈ per worker), not per row — avoids one heap allocation per
        // result cell. Chunks stay in order, so the bytes are identical to the serial path.
        let chunk = scan_rows.len().div_ceil(rayon::current_num_threads() * 4).max(1);
        let frags: Vec<(usize, String)> = scan_rows
            .par_chunks(chunk)
            .map(|rows| {
                let mut n = 0usize;
                let mut f = String::new();
                for row in rows {
                    if !passes(row) {
                        continue;
                    }
                    if !f.is_empty() {
                        f.push(',');
                    }
                    n += 1;
                    write_row(row, &mut f);
                }
                (n, f)
            })
            .collect();
        // Budget gate on the total row count (sets the sticky flag; the caller's
        // check converts it into the budget error).
        let _ = budget::exhausted(frags.iter().map(|(n, _)| n).sum());
        let mut chunks = vec![s];
        let mut wrote = false;
        for (_, f) in frags {
            if f.is_empty() {
                continue;
            }
            if wrote {
                chunks.last_mut().expect("chunks start non-empty").push(',');
            }
            wrote = true;
            emit_chunk(&mut chunks, f, flush);
        }
        chunks.last_mut().expect("chunks start non-empty").push_str("]}}");
        return Some(chunks);
    }
    let mut chunks: Vec<String> = Vec::new();
    let mut written = 0usize;
    for (i, row) in scan_rows.iter().enumerate() {
        // Coarse budget check every 1024 scanned rows; the caller's sticky check
        // turns an early stop into the budget error (never a truncated result).
        if i & 1023 == 0 && budget::exhausted(written) {
            break;
        }
        if !passes(row) {
            continue;
        }
        if written > 0 {
            s.push(',');
        }
        written += 1;
        write_row(row, &mut s);
        if flush.is_some_and(|n| s.len() >= n) {
            chunks.push(std::mem::take(&mut s));
        }
    }
    let _ = budget::exhausted(written); // final row-count gate (sticky)
    s.push_str("]}}");
    chunks.push(s);
    Some(chunks)
}

/// Evaluates a SELECT and serialises it straight to SPARQL-JSON, skipping the
/// `QueryResult` (and its per-cell `oxrdf::Term` allocation). On native the per-row
/// fragments are built in parallel; the wasm build is sequential.
pub fn eval_select_json(graph: &Graph, pattern: &GraphPattern) -> Result<String, String> {
    Ok(join_chunks(eval_select_json_chunks(graph, pattern, None)?))
}

/// [`eval_select_json`] as an ordered chunk sequence: the concatenation of the chunks
/// is byte-identical to the single-string result. `flush = Some(n)` starts a new chunk
/// roughly every `n` serialised bytes (and hands each parallel fragment over without
/// re-copying); `flush = None` produces the single-string layout (one chunk on the
/// sequential paths, head+fragments concatenated on the parallel path — exactly the
/// old behaviour and allocation profile).
pub fn eval_select_json_chunks(graph: &Graph, pattern: &GraphPattern, flush: Option<usize>) -> Result<Vec<String>, String> {
    // Streaming fast paths — no Bindings materialised at all.
    if let Some(json) = single_pattern_scan_json(graph, pattern, flush) {
        budget::check(0)?; // sticky: the streaming loop may have stopped mid-scan
        return Ok(json);
    }
    let mut local = LocalVocab::default();
    let bindings = eval_modified(graph, &mut local, pattern)?;
    budget::check(bindings.rows.len())?; // final gate (see eval_select)

    let out_vars: Vec<&Variable> = bindings.vars.iter().filter(|v| !v.as_str().starts_with(BNODE_VAR_PREFIX)).collect();
    let col_of: Vec<Option<usize>> = out_vars.iter().map(|v| bindings.col(v)).collect();

    let mut head = String::from("{\"head\":{\"vars\":[");
    for (i, v) in out_vars.iter().enumerate() {
        if i > 0 {
            head.push(',');
        }
        head.push('"');
        crate::json::escape_into(&mut head, v.as_str());
        head.push('"');
    }
    head.push_str("]},\"results\":{\"bindings\":[");

    let write_row = |row: &Row, s: &mut String| {
        s.push('{');
        let mut first = true;
        for (vi, &col) in col_of.iter().enumerate() {
            let Some(ci) = col else { continue };
            let id = row[ci];
            if id == NO_ID {
                continue; // unbound (e.g. OPTIONAL) — omitted from the binding
            }
            if !first {
                s.push(',');
            }
            first = false;
            s.push('"');
            crate::json::escape_into(s, out_vars[vi].as_str());
            s.push_str("\":");
            write_id_json(graph, &local, id, s);
        }
        s.push('}');
    };

    let mut s = head;
    #[cfg(feature = "parallel")]
    if bindings.rows.len() >= PAR_THRESHOLD {
        use rayon::prelude::*;
        // One string per chunk (≈ per worker), not per row. Chunks stay in order → identical bytes.
        let chunk = bindings.rows.len().div_ceil(rayon::current_num_threads() * 4).max(1);
        let frags: Vec<String> = bindings
            .rows
            .par_chunks(chunk)
            .map(|rows| {
                let mut f = String::new();
                for (k, row) in rows.iter().enumerate() {
                    if k > 0 {
                        f.push(',');
                    }
                    write_row(row, &mut f);
                }
                f
            })
            .collect();
        let mut chunks = vec![s];
        for (i, f) in frags.into_iter().enumerate() {
            if i > 0 {
                chunks.last_mut().expect("chunks start non-empty").push(',');
            }
            emit_chunk(&mut chunks, f, flush);
        }
        chunks.last_mut().expect("chunks start non-empty").push_str("]}}");
        return Ok(chunks);
    }
    let mut chunks: Vec<String> = Vec::new();
    for (i, row) in bindings.rows.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        write_row(row, &mut s);
        if flush.is_some_and(|n| s.len() >= n) {
            chunks.push(std::mem::take(&mut s));
        }
    }
    s.push_str("]}}");
    chunks.push(s);
    Ok(chunks)
}

/// ASK evaluation: `true` iff `pattern` has at least one solution. The pattern is
/// wrapped in a `LIMIT 1` slice so the early-terminating single-pattern scan path
/// stops at the first row; shapes without a streaming path evaluate normally (the
/// row count is irrelevant — only emptiness is observed).
pub fn eval_ask(graph: &Graph, pattern: &GraphPattern) -> Result<bool, String> {
    // Exact-count fast path first (a single-pattern BGP answers from the index).
    if let Some(n) = try_count(graph, pattern) {
        return Ok(n > 0);
    }
    let sliced = GraphPattern::Slice { inner: Box::new(pattern.clone()), start: 0, length: Some(1) };
    let mut local = LocalVocab::default();
    let b = eval_modified(graph, &mut local, &sliced)?;
    budget::check(b.rows.len())?;
    Ok(!b.rows.is_empty())
}

/// Evaluates a SELECT but returns only the solution count. When the count can be
/// derived without materialising the result (a single-pattern scan, possibly under
/// projection / LIMIT — like QLever's lazy count) it short-circuits; otherwise it
/// evaluates and counts the rows.
pub fn count_select(graph: &Graph, pattern: &GraphPattern) -> Result<usize, String> {
    if let Some(n) = try_count(graph, pattern) {
        return Ok(n);
    }
    let mut local = LocalVocab::default();
    let bindings = eval_modified(graph, &mut local, pattern)?;
    budget::check(bindings.rows.len())?; // final gate (see eval_select)
    Ok(bindings.rows.len())
}

/// The solution count without materialising, for shapes whose count is exact from
/// the index: a single-pattern BGP (range size) under projection / OFFSET-LIMIT.
fn try_count(graph: &Graph, p: &GraphPattern) -> Option<usize> {
    // zk-trace: a count/ASK answered from the index range consumes NO
    // attributable input triples, so an armed recorder would capture an
    // empty (insufficient) witness set. Disable the pushdown while recording
    // — the result is identical via the materialising path, only the plan
    // changes (zk module docs: "result-preserving plan changes").
    #[cfg(feature = "zk")]
    if crate::zk::enabled() {
        return None;
    }
    if view::default_is_empty() {
        return None; // empty-default view: the index ranges are not the active dataset
    }
    match p {
        GraphPattern::Project { inner, .. } | GraphPattern::Reduced { inner } => try_count(graph, inner),
        GraphPattern::Slice { inner, start, length } => try_count(graph, inner).map(|n| {
            let after_offset = n.saturating_sub(*start);
            length.map_or(after_offset, |l| after_offset.min(l))
        }),
        // OPTIONAL: count the left join without materialising (Σ over the join var).
        GraphPattern::LeftJoin { left, right, expression } => {
            count_leftjoin(graph, left, right, expression.as_ref())
        }
        // A single-pattern, filter-free BGP: the range size is the exact count.
        _ => count_pushdown(graph, p),
    }
}

/// The single triple pattern of a filter-free one-pattern BGP, else `None`.
fn single_pattern(p: &GraphPattern) -> Option<TriplePattern> {
    if !is_conjunctive(p) {
        return None;
    }
    let mut patterns = Vec::new();
    let mut filters = Vec::new();
    flatten_conjunction(p, &mut patterns, &mut filters);
    (patterns.len() == 1 && filters.is_empty()).then(|| patterns.pop().unwrap())
}

/// Exact solution count of `left OPTIONAL right` for the common shape — `left` and
/// `right` each a single filter-free pattern sharing exactly one variable `v`, with
/// no OPTIONAL filter — as `Σ_v c_left(v)·max(1, c_right(v))`, streamed from the
/// sorted indexes (each left binding survives, joined with its ≥1 right matches or
/// kept once with the right vars unbound). Returns `None` otherwise.
fn count_leftjoin(
    graph: &Graph,
    left: &GraphPattern,
    right: &GraphPattern,
    expression: Option<&Expression>,
) -> Option<usize> {
    if expression.is_some() {
        return None; // an OPTIONAL filter changes which right rows are compatible.
    }
    let (lp, rp) = (single_pattern(left)?, single_pattern(right)?);
    let (lip, lpv, lu) = prepare_pattern(graph, &lp).ok()?;
    if lu {
        return Some(0); // no left bindings at all.
    }
    if !distinct_pattern_vars(&lpv) {
        return None;
    }
    let (rip, rpv, ru) = prepare_pattern(graph, &rp).ok()?;
    if !distinct_pattern_vars(&rpv) {
        return None;
    }
    // Exactly one shared variable, at positions (lpos, rpos).
    let shared: Vec<(usize, usize)> = lpv
        .iter()
        .enumerate()
        .filter_map(|(i, v)| v.as_ref().and_then(|v| rpv.iter().position(|x| x.as_ref() == Some(v)).map(|j| (i, j))))
        .collect();
    let [(lpos, rpos)] = shared[..] else {
        return None;
    };
    // Σ_v c_left(v)·max(1, c_right(v)), streamed by merging the two sorted group-count
    // streams — left drives, right advances to match — so neither side is materialised.
    let mut left = GroupStream::new(graph, &lip, lpos);
    // Right unsatisfiable: every left binding is kept once with the right unbound.
    if ru {
        let mut total = 0usize;
        while let Some((_, cl)) = left.next() {
            total += cl;
        }
        return Some(total);
    }
    let mut right = GroupStream::new(graph, &rip, rpos);
    let mut rhead = right.next();
    let mut total = 0usize;
    while let Some((v, cl)) = left.next() {
        while let Some((rv, _)) = rhead {
            if rv < v {
                rhead = right.next();
            } else {
                break;
            }
        }
        let cr = match rhead {
            Some((rv, rc)) if rv == v => rc,
            _ => 0,
        };
        total += cl * cr.max(1);
    }
    Some(total)
}

// ---- Algebra dispatch ---------------------------------------------------------

fn eval_modified(graph: &Graph, local: &mut LocalVocab, p: &GraphPattern) -> Result<Bindings, String> {
    match p {
        GraphPattern::Project { inner, variables } => {
            let b = eval_modified(graph, local, inner)?;
            Ok(project_bindings(b, variables))
        }
        GraphPattern::Distinct { inner } => {
            // zk-trace: the enclosed pattern inputs are the PRE-DISTINCT
            // input sets (the reduction is verifier-side; zk module docs).
            #[cfg(feature = "zk")]
            let _zk = crate::zk::op_scope(crate::zk::Op::Distinct);
            let mut b = eval_modified(graph, local, inner)?;
            distinct_bindings(&mut b);
            Ok(b)
        }
        GraphPattern::Reduced { inner } => eval_modified(graph, local, inner),
        GraphPattern::Slice { inner, start, length } => {
            // LIMIT early-termination: a bare LIMIT over a single-pattern scan
            // (no ORDER BY / DISTINCT / aggregation / join, which all need the full
            // result) can stop after start+length rows instead of materialising the
            // whole relation — true streaming. LIMIT-without-ORDER-BY is
            // order-insensitive so any rows are valid.
            if let Some(len) = length {
                if let Some(cap) = start.checked_add(*len) {
                    if let Some(mut b) = try_capped(graph, inner, cap)? {
                        slice_bindings(&mut b, *start, *length);
                        return Ok(b);
                    }
                }
            }
            let mut b = eval_modified(graph, local, inner)?;
            slice_bindings(&mut b, *start, *length);
            Ok(b)
        }
        GraphPattern::OrderBy { inner, expression } => {
            let mut b = eval_modified(graph, local, inner)?;
            order_bindings(graph, local, &mut b, expression)?;
            Ok(b)
        }
        GraphPattern::Group { inner, variables, aggregates } => {
            // zk-trace: the enclosed pattern inputs are the PRE-AGGREGATION
            // input sets; the count pushdown below is disabled under an armed
            // recorder (inside `try_count`), so they are always captured.
            #[cfg(feature = "zk")]
            let _zk = crate::zk::op_scope(crate::zk::Op::Group);
            // COUNT(*) pushdown: a whole-dataset COUNT over a single pattern is the
            // scan range size — no need to materialise the solutions just to count
            // them (QLever counts lazily too; this is the q02-style win).
            if variables.is_empty() && aggregates.len() == 1 {
                if let (av, AggregateExpression::CountSolutions { distinct: false }) = (&aggregates[0].0, &aggregates[0].1) {
                    // COUNT(*) == the inner pattern's solution count. Use `try_count`, not
                    // just `count_pushdown`: it also covers OPTIONAL (lazy left-join count,
                    // Σ over the join var) and LIMIT/OFFSET — so `COUNT(*)` over an OPTIONAL
                    // no longer materialises the whole left-join just to count it.
                    if let Some(n) = try_count(graph, inner) {
                        let id = value_to_id(graph, local, &Value::Num(Num::Int(n as i64)));
                        let row: Row = std::iter::once(id).collect();
                        return Ok(Bindings { vars: vec![av.clone()], rows: vec![row], sorted_by: None });
                    }
                }
            }
            let b = eval_graph_pattern(graph, local, inner)?;
            group_aggregate(graph, local, b, variables, aggregates)
        }
        other => eval_graph_pattern(graph, local, other),
    }
}

/// Evaluates a pattern producing at most `cap` rows by stopping the scan early,
/// when that is safe: a single-pattern scan (optionally with a pushed-down
/// sargable numeric filter) under projection. Returns `None` for anything that
/// needs the full result first (joins, ORDER BY, DISTINCT, aggregation, residual
/// filters), so the caller falls back to full evaluation.
fn try_capped(graph: &Graph, inner: &GraphPattern, cap: usize) -> Result<Option<Bindings>, String> {
    // zk-trace: LIMIT early-termination would scan only the first `cap` rows,
    // recording a TRUNCATED input set — but the completeness witness (the
    // linear-sweep circuit) must see the whole scan range. Disable the cap
    // while recording; the full path is result-equivalent (LIMIT is
    // order-insensitive without ORDER BY).
    #[cfg(feature = "zk")]
    if crate::zk::enabled() {
        return Ok(None);
    }
    if view::default_is_empty() {
        return Ok(None); // empty-default view: the general path short-circuits at the BGP
    }
    match inner {
        GraphPattern::Project { inner, variables } => {
            Ok(try_capped(graph, inner, cap)?.map(|b| project_bindings(b, variables)))
        }
        GraphPattern::Reduced { inner } => try_capped(graph, inner, cap),
        p if is_conjunctive(p) => {
            let mut patterns = Vec::new();
            let mut filters = Vec::new();
            flatten_conjunction(p, &mut patterns, &mut filters);
            if patterns.len() != 1 {
                return Ok(None);
            }
            let (pat_filters, residual) = split_sargable(&patterns, &filters);
            if !residual.is_empty() {
                return Ok(None);
            }
            let (id_pat, pos_vars, unsat) = prepare_pattern(graph, &patterns[0])?;
            if unsat {
                return Ok(Some(Bindings::unsorted(collect_vars(&patterns), vec![])));
            }
            let filt = pat_filters[0];
            let sort_col = filt.map(|(c, _)| c);
            Ok(Some(scan_to_bindings(graph, &id_pat, &pos_vars, sort_col, filt, Some(cap))))
        }
        _ => Ok(None),
    }
}

/// Distinct (non-repeated) variable positions of a prepared pattern, or `None` if
/// a variable repeats (e.g. `?x p ?x`), which would make range counts over-count.
pub(crate) fn distinct_pattern_vars(pos_vars: &[Option<Variable>; 3]) -> bool {
    let vars: Vec<&Variable> = pos_vars.iter().flatten().collect();
    let mut sorted = vars.clone();
    sorted.sort();
    sorted.dedup();
    sorted.len() == vars.len()
}

/// A lazy stream of `(value, group-size)` pairs ascending by value, for the group
/// column `v_pos` of a pattern — the streaming form of [`group_counts`]. When the chosen
/// permutation already delivers `v_pos` order (the 6-perm case) it run-length-groups
/// directly over the borrowed scan rows, materialising NOTHING; otherwise it collects +
/// sorts the column once (reduced-permutation fallback). Lets a multi-pattern star COUNT
/// be summed by k-way merge with O(k) memory instead of one group vector per pattern.
struct GroupStream<'a> {
    scan: sparq_core::store::Scan<'a>,
    /// The STORED column (into the permutation's row layout) holding the group value —
    /// precomputed so the hot loop reads `row[col]` directly instead of rebuilding the
    /// canonical triple per row.
    col: usize,
    i: usize,
    sorted_vals: Option<Vec<Id>>,
}

impl<'a> GroupStream<'a> {
    fn new(graph: &'a Graph, id_pat: &IdPattern, v_pos: usize) -> Self {
        let scan = graph.store.scan_sorted(id_pat, v_pos);
        let order = scan.perm.order();
        // Canonical column `v_pos` is stored at this position in the permutation's rows.
        let col = order.iter().position(|&c| c == v_pos).unwrap();
        let sorted = order.into_iter().find(|&c| id_pat[c].is_none()) == Some(v_pos);
        let sorted_vals = (!sorted).then(|| {
            let mut v: Vec<Id> = scan.rows.iter().map(|r| r[col]).collect();
            v.sort_unstable();
            v
        });
        GroupStream { scan, col, i: 0, sorted_vals }
    }

    /// The next `(value, run-length)` in ascending value order, or `None` at the end.
    fn next(&mut self) -> Option<(Id, usize)> {
        let (slice, col): (&[[Id; 3]], usize) = match &self.sorted_vals {
            // The fallback stores bare values in column 0 of a 1-wide logical view; reuse
            // the same run-length code by treating the Vec as the source.
            Some(vals) => {
                if self.i >= vals.len() {
                    return None;
                }
                let v = vals[self.i];
                let mut c = 0;
                while self.i < vals.len() && vals[self.i] == v {
                    self.i += 1;
                    c += 1;
                }
                return Some((v, c));
            }
            None => (&self.scan.rows, self.col),
        };
        if self.i >= slice.len() {
            return None;
        }
        let v = slice[self.i][col];
        let mut c = 0;
        while self.i < slice.len() && slice[self.i][col] == v {
            self.i += 1;
            c += 1;
        }
        Some((v, c))
    }
}

/// Counts the solutions of a single triple pattern constrained by sargable numeric
/// FILTER(s), WITHOUT materialising a row per solution. Returns `None` (fall back) if
/// any filter is not a sargable numeric comparison on a variable of the pattern.
fn count_single_filtered(
    graph: &Graph,
    id_pat: &IdPattern,
    pos_vars: &[Option<Variable>; 3],
    filters: &[Expression],
) -> Option<usize> {
    // Resolve every filter to (canonical position, comparison); all must be sargable.
    let mut cmps: Vec<(usize, ScanCmp)> = Vec::with_capacity(filters.len());
    for f in filters {
        let (var, cmp) = extract_sargable(f)?;
        let pos = pos_vars.iter().position(|v| v.as_ref() == Some(&var))?;
        cmps.push((pos, cmp));
    }

    // Fast path: one filter on an all-inline column -> binary-searched range size
    // (the same value-sorted slice range-pruning uses, but we only need its length).
    // Requires the scan to be ACTUALLY sorted by the filter column (a reduced
    // permutation set may not deliver that order, in which case we fall through to the
    // count-scan below).
    if let [(fpos, cmp)] = cmps[..] {
        let scan = graph.store.scan_sorted(id_pat, fpos);
        let sorted_by_f = scan.perm.order().into_iter().find(|&c| id_pat[c].is_none()) == Some(fpos);
        if sorted_by_f && scan.rows.first().is_some_and(|r| dict::is_inline(scan.to_spo(r)[fpos])) {
            return Some(match inline_pass_values(cmp) {
                Some((lo, hi)) => {
                    let (lo_id, hi_id) = (dict::INLINE_BASE + lo, dict::INLINE_BASE + hi);
                    let start = scan.rows.partition_point(|r| scan.to_spo(r)[fpos] < lo_id);
                    let end = scan.rows.partition_point(|r| scan.to_spo(r)[fpos] <= hi_id);
                    end - start
                }
                None => 0,
            });
        }
    }

    // General: scan once, count rows passing ALL comparisons via the value caches.
    // No solution row is built. The single-comparison shapes are specialised so the
    // per-row test is a direct cache probe with no predicate-kind dispatch (this loop
    // is the COUNT-mode hot path for cached-numeric FILTERs).
    let scan = graph.store.scan(id_pat);
    let total = match cmps[..] {
        [(pos, ScanCmp::Num(cmp))] => scan
            .rows
            .iter()
            .filter(|row| graph.numeric_value(scan.to_spo(row)[pos]).is_some_and(|x| cmp.test(x)))
            .count(),
        [(pos, ScanCmp::Temp(op, t))] => scan
            .rows
            .iter()
            .filter(|row| {
                graph
                    .temporal_value(scan.to_spo(row)[pos])
                    .and_then(|v| Temporal::cmp_t(v, t))
                    .is_some_and(|o| op.eval(o))
            })
            .count(),
        _ => scan
            .rows
            .iter()
            .filter(|row| {
                let spo = scan.to_spo(row);
                cmps.iter().all(|(pos, cmp)| cmp.test_id(graph, spo[*pos]))
            })
            .count(),
    };
    Some(total)
}

/// Exact solution count of a filter-free conjunctive BGP without materialising the
/// result, for two shapes: a single pattern (index range size) and an N-pattern STAR
/// — every pattern sharing one common variable `v*`, with every other variable local
/// to a single pattern — counted as `Σ_v Π_i c_i(v)` over per-pattern group sizes,
/// streamed from the sorted indexes. Returns `None` (fall back to full evaluation)
/// for non-star shapes (e.g. 3+-pattern chains, where the product formula overcounts).
fn count_pushdown(graph: &Graph, inner: &GraphPattern) -> Option<usize> {
    if !is_conjunctive(inner) {
        return None;
    }
    let mut patterns = Vec::new();
    let mut filters = Vec::new();
    flatten_conjunction(inner, &mut patterns, &mut filters);

    if patterns.len() == 1 {
        let (id_pat, pos_vars, unsat) = prepare_pattern(graph, &patterns[0]).ok()?;
        if unsat {
            return Some(0);
        }
        if !distinct_pattern_vars(&pos_vars) {
            return None;
        }
        if filters.is_empty() {
            return Some(graph.store.estimate(&id_pat));
        }
        // Single pattern + sargable numeric FILTER(s): count the passing rows without
        // materialising — a binary-searched range size on an all-inline column, else a
        // count-scan (still no Row built per solution).
        return count_single_filtered(graph, &id_pat, &pos_vars, &filters);
    }

    // Multi-pattern star count is filter-free (a pushed filter changes the per-value
    // group counts; leave that to full evaluation).
    if !filters.is_empty() {
        return None;
    }

    // Prepare every pattern; each must have distinct in-pattern vars (no repeated var,
    // so a group-count per value is well defined).
    let mut prepared: Vec<(IdPattern, [Option<Variable>; 3])> = Vec::with_capacity(patterns.len());
    for p in &patterns {
        let (ip, pv, unsat) = prepare_pattern(graph, p).ok()?;
        if unsat {
            return Some(0);
        }
        if !distinct_pattern_vars(&pv) {
            return None;
        }
        prepared.push((ip, pv));
    }

    // Star test: find a variable in EVERY pattern; require every OTHER variable to
    // occur in exactly one pattern (so the only join is on the centre — otherwise the
    // product formula would overcount a second shared variable).
    let mut occ: FxHashMap<&Variable, usize> = FxHashMap::default();
    for (_, pv) in &prepared {
        for v in pv.iter().flatten() {
            *occ.entry(v).or_insert(0) += 1;
        }
    }
    let center = *occ.iter().find(|(_, &n)| n == prepared.len()).map(|(v, _)| v)?;
    if occ.iter().any(|(v, &n)| *v != center && n != 1) {
        return None;
    }

    // Σ_v Π_i c_i(v) over the centre value v, computed by k-way INTERSECTION MERGE of the
    // per-pattern group-count streams — only values present in EVERY pattern contribute.
    // O(k) memory: no per-pattern group vector is materialised (the dominant memory of a
    // star COUNT at scale), just one cursor per pattern.
    let mut streams: Vec<GroupStream> = Vec::with_capacity(prepared.len());
    for (ip, pv) in &prepared {
        let cpos = pv.iter().position(|v| v.as_ref() == Some(center))?;
        streams.push(GroupStream::new(graph, ip, cpos));
    }
    let mut heads: Vec<(Id, usize)> = Vec::with_capacity(streams.len());
    for s in &mut streams {
        match s.next() {
            Some(h) => heads.push(h),
            None => return Some(0), // a pattern with no rows → empty intersection.
        }
    }
    let mut total = 0usize;
    loop {
        let max_v = heads.iter().map(|(v, _)| *v).max().unwrap();
        // Advance every cursor up to `max_v`; track whether all reach exactly it.
        let mut all_equal = true;
        for (i, head) in heads.iter_mut().enumerate() {
            while head.0 < max_v {
                match streams[i].next() {
                    Some(h) => *head = h,
                    None => return Some(total), // this pattern exhausted → done.
                }
            }
            if head.0 != max_v {
                all_equal = false;
            }
        }
        if all_equal {
            let prod: usize = heads.iter().map(|(_, c)| *c).product();
            total += prod;
            for (i, head) in heads.iter_mut().enumerate() {
                match streams[i].next() {
                    Some(h) => *head = h,
                    None => return Some(total),
                }
            }
        }
    }
}

/// Evaluate `GRAPH <name> { inner }`. Each named graph is a self-contained sub-`Graph` with its own
/// dictionary, so we evaluate `inner` against it and then TRANSLATE the result ids into the outer
/// graph's id space (materialise each term in the sub-graph, re-intern in the outer dict / local
/// vocab) — so the bindings join and serialise correctly. `GRAPH ?g { … }` unions over every named
/// graph, prepending the graph-name binding.
fn eval_graph_named(
    graph: &Graph,
    local: &mut LocalVocab,
    name: &NamedNodePattern,
    inner: &GraphPattern,
) -> Result<Bindings, String> {
    fn eval_translated(
        graph: &Graph,
        local: &mut LocalVocab,
        sub: &Graph,
        #[cfg(feature = "zk")] gname: &Term,
        inner: &GraphPattern,
    ) -> Result<Bindings, String> {
        // Inside GRAPH the evaluation graph IS the named sub-graph: suspend a
        // view's empty-default short-circuit for the inner pattern (L1 view).
        let _scope = view::enter_graph();
        // zk-trace: tag the enclosed scans/filters with the named graph (the
        // sub-graph has its own dictionary; terms are materialized at record
        // time against it, so the tag is what attributes them). The `gname`
        // parameter is cfg'd out entirely when the feature is off, so the
        // default (wasm) build is byte-identical.
        #[cfg(feature = "zk")]
        let _zk = crate::zk::graph_scope(gname);
        let mut sub_local = LocalVocab::default();
        let b = eval_graph_pattern(sub, &mut sub_local, inner)?;
        let rows: Vec<Row> = b
            .rows
            .iter()
            .map(|r| {
                r.iter()
                    .map(|&id| match term_of(sub, &sub_local, id) {
                        Some(t) => value_to_id(graph, local, &Value::Term(t)),
                        None => NO_ID,
                    })
                    .collect()
            })
            .collect();
        Ok(Bindings::unsorted(b.vars, rows))
    }
    match name {
        NamedNodePattern::NamedNode(n) => {
            let target = Term::NamedNode(n.clone());
            // A graph outside an installed dataset view takes the absent-graph
            // branch below: non-visible must be INDISTINGUISHABLE from absent
            // (the L1 view's security property).
            let sub = if view::allows(&target) {
                graph.named.iter().find(|(t, _)| *t == target).map(|(_, sub)| sub)
            } else {
                None
            };
            match sub {
                Some(sub) => eval_translated(
                    graph,
                    local,
                    sub,
                    #[cfg(feature = "zk")]
                    &target,
                    inner,
                ),
                // The named graph is absent → ZERO solutions (even for `GRAPH <g> {}`,
                // which must NOT yield the unit row), but with `inner`'s variable
                // schema — evaluate against an empty graph for the columns, then drop
                // any rows (an empty group pattern would otherwise produce one).
                None => {
                    let _scope = view::enter_graph(); // schema eval matches the present-graph path
                    // zk-trace: an absent graph still records the operator
                    // boundary + (empty) pattern input sets under its name.
                    #[cfg(feature = "zk")]
                    let _zk = crate::zk::graph_scope(&target);
                    let empty = Graph::load_str("", "ntriples").map_err(|e| e.to_string())?;
                    let mut el = LocalVocab::default();
                    let mut b = eval_graph_pattern(&empty, &mut el, inner)?;
                    b.rows.clear();
                    Ok(b)
                }
            }
        }
        NamedNodePattern::Variable(v) => {
            let mut acc: Option<Bindings> = None;
            for (gname, sub) in &graph.named {
                if !view::allows(gname) {
                    continue; // not visible under the installed dataset view (L1)
                }
                // zk-trace: each iteration of `GRAPH ?g` tags the enclosed
                // scans/filters with the iteration's named graph — the scope
                // is installed INSIDE eval_translated (one place), so the
                // operator boundary stream is not double-nested.
                let mut b = eval_translated(
                    graph,
                    local,
                    sub,
                    #[cfg(feature = "zk")]
                    gname,
                    inner,
                )?;
                let gid = value_to_id(graph, local, &Value::Term(gname.clone()));
                match b.col(v) {
                    // The inner pattern itself binds the graph variable (e.g.
                    // `GRAPH ?g { ?g :p ?o }` or a VALUES/OPTIONAL inside): JOIN with
                    // the active graph name — keep rows already bound to this graph,
                    // fill unbound cells, drop conflicting rows.
                    Some(c) => {
                        b.rows.retain_mut(|row| {
                            if row[c] == NO_ID {
                                row[c] = gid;
                                true
                            } else {
                                row[c] == gid
                            }
                        });
                        b.sorted_by = None;
                    }
                    None => {
                        b.vars.insert(0, v.clone());
                        for row in &mut b.rows {
                            row.insert(0, gid);
                        }
                    }
                }
                acc = Some(match acc {
                    None => b,
                    Some(a) => union_bindings(a, b),
                });
            }
            Ok(acc.unwrap_or_else(|| Bindings::unsorted(vec![v.clone()], Vec::new())))
        }
    }
}

/// Operator-entry dispatcher. When an EXPLAIN ANALYZE trace is installed (T22) it
/// routes through the `#[cold]` timing wrapper; otherwise it is a direct call into
/// the evaluator — the only cost on the normal path is one thread-local flag read
/// per *operator* (the same class of check `budget::check` already does here).
fn eval_graph_pattern(graph: &Graph, local: &mut LocalVocab, p: &GraphPattern) -> Result<Bindings, String> {
    if trace::enabled() {
        return eval_graph_pattern_traced(graph, local, p);
    }
    eval_graph_pattern_inner(graph, local, p)
}

/// EXPLAIN ANALYZE wrapper: records one trace node per operator with its output
/// row count and wall time. `#[cold]` keeps it (and the `Instant` plumbing) off
/// the normal path entirely.
#[cold]
fn eval_graph_pattern_traced(graph: &Graph, local: &mut LocalVocab, p: &GraphPattern) -> Result<Bindings, String> {
    let idx = trace::enter(trace_label(p));
    #[cfg(not(target_arch = "wasm32"))]
    let start = std::time::Instant::now();
    let r = eval_graph_pattern_inner(graph, local, p);
    #[cfg(not(target_arch = "wasm32"))]
    let nanos = start.elapsed().as_nanos() as u64;
    // `Instant` is unusable on wasm32-unknown-unknown (it panics): rows only there.
    #[cfg(target_arch = "wasm32")]
    let nanos = 0u64;
    trace::exit(idx, r.as_ref().map_or(0, |b| b.rows.len()), nanos);
    r
}

/// The operator label an EXPLAIN ANALYZE trace node carries (only built while tracing).
fn trace_label(p: &GraphPattern) -> String {
    if is_conjunctive(p) {
        let mut patterns = Vec::new();
        let mut filters = Vec::new();
        flatten_conjunction(p, &mut patterns, &mut filters);
        let plan = if bgp_uses_binary(&patterns) { "binary GOO" } else { "worst-case-optimal (LFTJ)" };
        return format!("BGP [{plan}] ({} patterns, {} filters)", patterns.len(), filters.len());
    }
    match p {
        GraphPattern::Bgp { patterns } => format!("BGP ({} patterns)", patterns.len()),
        GraphPattern::Filter { .. } => "Filter".into(),
        GraphPattern::Join { .. } => "Join".into(),
        GraphPattern::LeftJoin { .. } => "LeftJoin (OPTIONAL)".into(),
        GraphPattern::Union { .. } => "Union".into(),
        GraphPattern::Extend { variable, .. } => format!("Extend (BIND ?{})", variable.as_str()),
        GraphPattern::Minus { .. } => "Minus".into(),
        GraphPattern::Values { .. } => "Values".into(),
        GraphPattern::Path { .. } => "PropertyPath".into(),
        GraphPattern::Graph { .. } => "Graph".into(),
        GraphPattern::Project { .. } => "Project (sub-select)".into(),
        GraphPattern::Distinct { .. } => "Distinct".into(),
        GraphPattern::Reduced { .. } => "Reduced".into(),
        GraphPattern::Slice { .. } => "Slice".into(),
        GraphPattern::OrderBy { .. } => "OrderBy".into(),
        GraphPattern::Group { .. } => "Group".into(),
        _ => "Other".into(),
    }
}

fn eval_graph_pattern_inner(graph: &Graph, local: &mut LocalVocab, p: &GraphPattern) -> Result<Bindings, String> {
    budget::check(0)?; // coarse cooperative cancellation: once per operator entry
    if is_conjunctive(p) {
        let mut patterns = Vec::new();
        let mut filters = Vec::new();
        flatten_conjunction(p, &mut patterns, &mut filters);
        // Push sargable numeric FILTERs down into the binary-plan scans (the WCOJ
        // path applies them afterwards); apply the rest normally.
        let (mut b, residual) = if bgp_uses_binary(&patterns) {
            let (pat_filters, residual) = split_sargable(&patterns, &filters);
            (eval_bgp_binary(graph, &patterns, &pat_filters)?, residual)
        } else {
            (eval_bgp(graph, &patterns)?, filters)
        };
        for f in &residual {
            apply_filter(graph, local, &mut b, f)?;
        }
        return Ok(b);
    }
    match p {
        GraphPattern::Bgp { patterns } => eval_bgp(graph, patterns),
        GraphPattern::Filter { expr, inner } => {
            let mut b = eval_graph_pattern(graph, local, inner)?;
            apply_filter(graph, local, &mut b, expr)?;
            Ok(b)
        }
        GraphPattern::Join { left, right } => {
            let l = eval_graph_pattern(graph, local, left)?;
            let r = eval_graph_pattern(graph, local, right)?;
            Ok(join_bindings(l, r))
        }
        GraphPattern::LeftJoin { left, right, expression } => {
            // zk-trace: operator boundary marker (one thread-local read; the
            // scope is a no-op when the recorder is disarmed). NOTE: the
            // embedded OPTIONAL condition is evaluated inside the join and is
            // NOT recorded as a FilterObligation (see zk module docs).
            #[cfg(feature = "zk")]
            let _zk = crate::zk::op_scope(crate::zk::Op::Optional);
            let l = eval_graph_pattern(graph, local, left)?;
            let r = eval_graph_pattern(graph, local, right)?;
            left_outer_join(graph, local, l, r, expression.as_ref())
        }
        GraphPattern::Union { left, right } => {
            #[cfg(feature = "zk")]
            let _zk = crate::zk::op_scope(crate::zk::Op::Union);
            let l = eval_graph_pattern(graph, local, left)?;
            let r = eval_graph_pattern(graph, local, right)?;
            Ok(union_bindings(l, r))
        }
        GraphPattern::Extend { inner, variable, expression } => {
            let b = eval_graph_pattern(graph, local, inner)?;
            extend_bindings(graph, local, b, variable, expression)
        }
        GraphPattern::Minus { left, right } => {
            #[cfg(feature = "zk")]
            let _zk = crate::zk::op_scope(crate::zk::Op::Minus);
            let l = eval_graph_pattern(graph, local, left)?;
            let r = eval_graph_pattern(graph, local, right)?;
            Ok(minus_bindings(l, r))
        }
        GraphPattern::Values { variables, bindings } => Ok(values_bindings(graph, local, variables, bindings)),
        GraphPattern::Path { subject, path, object } => {
            // zk-trace: property-path expansion scans the store without
            // per-pattern attribution — record an Op::Path marker so a
            // consumer fails closed (ZkTrace::first_uncaptured) rather than
            // building an insufficient witness set.
            #[cfg(feature = "zk")]
            let _zk = crate::zk::op_scope(crate::zk::Op::Path);
            eval_path(graph, local, subject, path, object)
        }
        GraphPattern::Graph { name, inner } => eval_graph_named(graph, local, name, inner),
        GraphPattern::Project { .. }
        | GraphPattern::Distinct { .. }
        | GraphPattern::Reduced { .. }
        | GraphPattern::Slice { .. }
        | GraphPattern::OrderBy { .. }
        | GraphPattern::Group { .. } => eval_modified(graph, local, p),
        other => Err(format!("unsupported graph pattern: {other:?}")),
    }
}

pub(crate) fn is_conjunctive(p: &GraphPattern) -> bool {
    match p {
        GraphPattern::Bgp { .. } => true,
        // A FILTER may only be flattened into the enclosing conjunction when every
        // variable it mentions is bound INSIDE its own group — otherwise hoisting it
        // changes scope (`{ :x :p ?v . { FILTER(?v = 1) } }` must see ?v UNBOUND).
        // EXISTS is conservatively never flattened (it evaluates against the group's
        // in-scope bindings).
        GraphPattern::Filter { inner, expr } => {
            if !is_conjunctive(inner) {
                return false;
            }
            let mut inner_vars: FxHashSet<Variable> = FxHashSet::default();
            collect_pattern_vars(inner, &mut inner_vars);
            filter_scope_ok(expr, &inner_vars)
        }
        GraphPattern::Join { left, right } => is_conjunctive(left) && is_conjunctive(right),
        _ => false,
    }
}

/// All variables bound by the triple patterns of a conjunctive subtree.
fn collect_pattern_vars(p: &GraphPattern, out: &mut FxHashSet<Variable>) {
    match p {
        GraphPattern::Bgp { patterns } => {
            for tp in patterns {
                for v in [tp_var(&tp.subject), nnp_var(&tp.predicate), tp_var(&tp.object)].into_iter().flatten() {
                    out.insert(v);
                }
            }
        }
        GraphPattern::Filter { inner, .. } => collect_pattern_vars(inner, out),
        GraphPattern::Join { left, right } => {
            collect_pattern_vars(left, out);
            collect_pattern_vars(right, out);
        }
        _ => {}
    }
}

/// `true` if a filter expression's variables are all in `bound` (and it has no
/// EXISTS), so applying it at the top of the flattened conjunction is equivalent.
fn filter_scope_ok(e: &Expression, bound: &FxHashSet<Variable>) -> bool {
    use Expression::*;
    match e {
        NamedNode(_) | Literal(_) => true,
        Variable(v) | Bound(v) => bound.contains(v),
        UnaryPlus(a) | UnaryMinus(a) | Not(a) => filter_scope_ok(a, bound),
        And(a, b) | Or(a, b) | Equal(a, b) | SameTerm(a, b) | Greater(a, b) | GreaterOrEqual(a, b) | Less(a, b)
        | LessOrEqual(a, b) | Add(a, b) | Subtract(a, b) | Multiply(a, b) | Divide(a, b) => {
            filter_scope_ok(a, bound) && filter_scope_ok(b, bound)
        }
        In(a, list) => filter_scope_ok(a, bound) && list.iter().all(|c| filter_scope_ok(c, bound)),
        If(c, t, f) => filter_scope_ok(c, bound) && filter_scope_ok(t, bound) && filter_scope_ok(f, bound),
        Coalesce(es) => es.iter().all(|c| filter_scope_ok(c, bound)),
        FunctionCall(_, args) => args.iter().all(|c| filter_scope_ok(c, bound)),
        Exists(_) => false,
    }
}

pub(crate) fn flatten_conjunction(p: &GraphPattern, patterns: &mut Vec<TriplePattern>, filters: &mut Vec<Expression>) {
    match p {
        GraphPattern::Bgp { patterns: tps } => patterns.extend(tps.iter().cloned()),
        GraphPattern::Join { left, right } => {
            flatten_conjunction(left, patterns, filters);
            flatten_conjunction(right, patterns, filters);
        }
        GraphPattern::Filter { expr, inner } => {
            flatten_conjunction(inner, patterns, filters);
            filters.push(expr.clone());
        }
        _ => unreachable!(),
    }
}

// ---- Sargable numeric filters (pushed into the scan) -------------------------

/// A numeric comparison `value OP threshold` that can be pushed down into a
/// pattern scan (FILTER predicate evaluated inline, in the column's sorted order,
/// so the numeric access is sequential rather than a random dictionary gather —
/// the layout fix the hardware research measured as an 8–15× win).
#[derive(Clone, Copy)]
pub(crate) enum NumCmp {
    Gt(f64),
    Ge(f64),
    Lt(f64),
    Le(f64),
    Eq(f64),
}

impl NumCmp {
    /// Human-readable comparison for EXPLAIN output, e.g. `> 28`.
    pub(crate) fn render(&self) -> String {
        match *self {
            NumCmp::Gt(t) => format!("> {t}"),
            NumCmp::Ge(t) => format!(">= {t}"),
            NumCmp::Lt(t) => format!("< {t}"),
            NumCmp::Le(t) => format!("<= {t}"),
            NumCmp::Eq(t) => format!("= {t}"),
        }
    }

    #[inline]
    fn test(&self, x: f64) -> bool {
        match *self {
            NumCmp::Gt(t) => x > t,
            NumCmp::Ge(t) => x >= t,
            NumCmp::Lt(t) => x < t,
            NumCmp::Le(t) => x <= t,
            NumCmp::Eq(t) => x == t,
        }
    }
}

/// A comparison operator, for the temporal pushed-down predicate.
#[derive(Clone, Copy)]
pub(crate) enum CmpOp {
    Gt,
    Ge,
    Lt,
    Le,
    Eq,
}

impl CmpOp {
    #[inline]
    fn eval(self, o: Ordering) -> bool {
        match self {
            CmpOp::Gt => o == Ordering::Greater,
            CmpOp::Ge => o != Ordering::Less,
            CmpOp::Lt => o == Ordering::Less,
            CmpOp::Le => o != Ordering::Greater,
            CmpOp::Eq => o == Ordering::Equal,
        }
    }

    fn render(self) -> &'static str {
        match self {
            CmpOp::Gt => ">",
            CmpOp::Ge => ">=",
            CmpOp::Lt => "<",
            CmpOp::Le => "<=",
            CmpOp::Eq => "=",
        }
    }
}

/// A sargable FILTER predicate pushed down into a pattern scan: numeric (via the f64
/// `numerics` cache) or temporal (via the `temporals` cache — dateTime/date vs a
/// temporal constant).
#[derive(Clone, Copy)]
pub(crate) enum ScanCmp {
    Num(NumCmp),
    /// `value OP temporal-constant`. A row passes when the comparison is DECIDABLE and
    /// satisfies the operator; an indeterminate (mixed-timezone window), cross-family
    /// (dateTime vs date) or non-temporal operand is a FILTER type error — the row is
    /// excluded, which `false` reproduces exactly. (For `=`, cross-family is "known
    /// different" rather than an error — also excluded, also `false`.)
    Temp(CmpOp, Temporal),
}

impl ScanCmp {
    /// Human-readable comparison for EXPLAIN output, e.g. `> 28`.
    pub(crate) fn render(&self) -> String {
        match *self {
            ScanCmp::Num(c) => c.render(),
            ScanCmp::Temp(op, t) => format!("{} temporal(instant {})", op.render(), t.instant),
        }
    }

    /// Evaluates the pushed-down predicate against one scanned column id, through the
    /// graph's numeric / temporal value cache — O(1), no term materialised.
    #[inline]
    fn test_id(&self, graph: &Graph, id: Id) -> bool {
        match *self {
            ScanCmp::Num(c) => graph.numeric_value(id).is_some_and(|x| c.test(x)),
            ScanCmp::Temp(op, t) => {
                graph.temporal_value(id).and_then(|v| Temporal::cmp_t(v, t)).is_some_and(|o| op.eval(o))
            }
        }
    }
}

/// The inclusive range of inline-integer *values* `[lo, hi]` (within `[0, INLINE_MAX]`)
/// that satisfy the comparison, or `None` if no integer can. Used to range-prune a
/// scan whose filter column holds inline integers (which sort by value). A TEMPORAL
/// predicate over an all-inline (integer) column is a type error on every row —
/// `None`, the empty range.
fn inline_pass_values(cmp: ScanCmp) -> Option<(u32, u32)> {
    let cmp = match cmp {
        ScanCmp::Num(c) => c,
        ScanCmp::Temp(..) => return None,
    };
    let max = (dict::INLINE_BASE - 1) as i64;
    let (lo, hi): (i64, i64) = match cmp {
        NumCmp::Gt(t) => (t.floor() as i64 + 1, max),
        NumCmp::Ge(t) => (t.ceil() as i64, max),
        NumCmp::Lt(t) => (0, t.ceil() as i64 - 1),
        NumCmp::Le(t) => (0, t.floor() as i64),
        NumCmp::Eq(t) => {
            if t.fract() != 0.0 || t < 0.0 || t > max as f64 {
                return None;
            }
            let v = t as i64;
            (v, v)
        }
    };
    let (lo, hi) = (lo.max(0), hi.min(max));
    (lo <= hi).then_some((lo as u32, hi as u32))
}

/// Recognises a FILTER of the form `?v OP constant` (or the symmetric
/// `constant OP ?v`) over a numeric or temporal constant, returning the variable
/// and the comparison to push down.
fn extract_sargable(e: &Expression) -> Option<(Variable, ScanCmp)> {
    fn lit_num(e: &Expression) -> Option<f64> {
        match e {
            Expression::Literal(l) if is_numeric_dt(l) => {
                let v: f64 = l.value().parse().ok()?;
                // A threshold f64 can't represent precisely (> 15 significant digits —
                // large integers or high-precision decimals) makes the sargable f64 scan
                // unsafe; decline so the filter takes the exact general comparison path.
                if sig_digits(l.value()) > 15 {
                    None
                } else {
                    Some(v)
                }
            }
            _ => None,
        }
    }
    // A well-formed dateTime/dateTimeStamp/date constant: its cached-comparable value.
    // (The runtime compare through the cache is bit-identical to the per-row parse, so
    // no precision guard is needed — unlike the f64 numeric threshold above.)
    fn lit_temp(e: &Expression) -> Option<Temporal> {
        match e {
            Expression::Literal(l) => temporal_of_lit(l),
            _ => None,
        }
    }
    fn var_of(e: &Expression) -> Option<Variable> {
        match e {
            Expression::Variable(v) => Some(v.clone()),
            _ => None,
        }
    }
    // (left, right, op-if-var-on-left, op-if-var-on-right)
    let (l, r, on_left, on_right): (&Expression, &Expression, CmpOp, CmpOp) = match e {
        Expression::Greater(l, r) => (l, r, CmpOp::Gt, CmpOp::Lt),
        Expression::GreaterOrEqual(l, r) => (l, r, CmpOp::Ge, CmpOp::Le),
        Expression::Less(l, r) => (l, r, CmpOp::Lt, CmpOp::Gt),
        Expression::LessOrEqual(l, r) => (l, r, CmpOp::Le, CmpOp::Ge),
        Expression::Equal(l, r) => (l, r, CmpOp::Eq, CmpOp::Eq),
        _ => return None,
    };
    let num_cmp = |op: CmpOp, t: f64| -> ScanCmp {
        ScanCmp::Num(match op {
            CmpOp::Gt => NumCmp::Gt(t),
            CmpOp::Ge => NumCmp::Ge(t),
            CmpOp::Lt => NumCmp::Lt(t),
            CmpOp::Le => NumCmp::Le(t),
            CmpOp::Eq => NumCmp::Eq(t),
        })
    };
    for (var, konst, op) in [(l, r, on_left), (r, l, on_right)] {
        let Some(v) = var_of(var) else { continue };
        if let Some(c) = lit_num(konst) {
            return Some((v, num_cmp(op, c)));
        }
        if let Some(t) = lit_temp(konst) {
            return Some((v, ScanCmp::Temp(op, t)));
        }
    }
    None
}

/// The canonical position (0=subject, 1=predicate, 2=object) of a variable in a
/// triple pattern, if it occurs there.
fn pattern_var_pos(tp: &TriplePattern, var: &Variable) -> Option<usize> {
    if matches!(&tp.subject, TermPattern::Variable(v) if v == var) {
        return Some(0);
    }
    if matches!(&tp.predicate, NamedNodePattern::Variable(v) if v == var) {
        return Some(1);
    }
    if matches!(&tp.object, TermPattern::Variable(v) if v == var) {
        return Some(2);
    }
    None
}

/// Splits FILTERs into per-pattern sargable numeric predicates (pushed into the
/// scan of the first pattern that binds the variable) and the residual filters
/// (applied normally afterwards).
pub(crate) fn split_sargable(patterns: &[TriplePattern], filters: &[Expression]) -> (Vec<Option<(usize, ScanCmp)>>, Vec<Expression>) {
    // zk-trace: a sargable FILTER pushed into the scan would make the scan
    // record only the POST-filter rows (the rows that PASSED), losing the
    // FILTER obligation and under-capturing the input set — the proof must
    // witness the operand of every filtered row and the rows it excluded.
    // Keep every filter residual while recording, so it flows through
    // apply_filter (one FilterObligation) over the FULL unfiltered scan set.
    // Result-equivalent; only the plan changes (zk module docs).
    #[cfg(feature = "zk")]
    if crate::zk::enabled() {
        return (vec![None; patterns.len()], filters.to_vec());
    }
    let mut pat_filters: Vec<Option<(usize, ScanCmp)>> = vec![None; patterns.len()];
    let mut residual = Vec::new();
    for f in filters {
        if let Some((var, cmp)) = extract_sargable(f) {
            if let Some((i, pos)) = patterns
                .iter()
                .enumerate()
                .find_map(|(i, tp)| pattern_var_pos(tp, &var).filter(|_| pat_filters[i].is_none()).map(|pos| (i, pos)))
            {
                pat_filters[i] = Some((pos, cmp));
                continue;
            }
        }
        residual.push(f.clone());
    }
    (pat_filters, residual)
}

// ---- BGP evaluation ----------------------------------------------------------

/// Evaluates a basic graph pattern, dispatching between a binary join plan and a
/// worst-case-optimal (Leapfrog Triejoin) plan. Cyclic BGPs — where binary plans
/// can blow up to an intermediate result far larger than the final answer (e.g.
/// triangles) — go to WCOJ, which runs in time `Õ(AGM bound)` and so is provably
/// optimal in the worst case. Acyclic (tree-shaped) BGPs use binary joins, which
/// are optimal for them and avoid LFTJ's per-tuple overhead. Both paths are
/// differentially tested to produce identical results.
/// Evaluate a SPARQL property path `subject <path> object` into bindings over its endpoint
/// variables. Computes the path's (start,end) id-pair relation recursively — transitive `+`/`*`
/// via BFS, zero-length `*`/`?` add identity over the graph's nodes — then constrains it by any
/// bound or repeated endpoint. Bound endpoints are PUSHED DOWN into the relation computation
/// ([`PathEnds`]): a bound subject turns `+`/`*` into a single-source directed BFS over sorted
/// range scans, a bound object traverses in reverse, and both-bound is a reachability test with
/// early exit — so `:s p+ ?x` costs `O(edges reachable)`, not the all-pairs closure. The
/// post-filter below stays as the correctness backstop (the pushdown contract allows supersets).
fn eval_path(
    graph: &Graph,
    local: &mut LocalVocab,
    subject: &TermPattern,
    path: &PropertyPathExpression,
    object: &TermPattern,
) -> Result<Bindings, String> {
    enum End {
        Var(Variable),
        Bound(Id),
        /// A concrete term ABSENT from the dictionary — unsatisfiable for ordinary
        /// paths, but still the start of a zero-length solution for `p*` / `p?`.
        Missing(Term),
    }
    let resolve = |t: &TermPattern| -> Result<End, String> {
        Ok(match t {
            TermPattern::Variable(v) => End::Var(v.clone()),
            TermPattern::BlankNode(b) => End::Var(bnode_var(b)),
            other => {
                let term = term_pattern_to_term(other)?;
                match graph.id_of(&term) {
                    Some(id) => End::Bound(id),
                    None => End::Missing(term),
                }
            }
        })
    };
    let (s_end, o_end) = (resolve(subject)?, resolve(object)?);
    // Paths that admit the ZERO-LENGTH solution connect every term to itself — even
    // terms that do not occur in the data (a constant endpoint on an empty graph).
    let zero_len = matches!(path, PropertyPathExpression::ZeroOrMore(_) | PropertyPathExpression::ZeroOrOne(_));

    let s_var = if let End::Var(v) = &s_end { Some(v.clone()) } else { None };
    let o_var = if let End::Var(v) = &o_end { Some(v.clone()) } else { None };
    let same_var = matches!((&s_var, &o_var), (Some(a), Some(b)) if a == b);
    let mut vars: Vec<Variable> = Vec::new();
    if let Some(v) = &s_var {
        vars.push(v.clone());
    }
    if let Some(v) = &o_var {
        if !same_var {
            vars.push(v.clone());
        }
    }
    // A concrete-but-absent endpoint makes the pattern unsatisfiable — unless the
    // path has a zero-length solution, handled below.
    if !zero_len && (matches!(s_end, End::Missing(_)) || matches!(o_end, End::Missing(_))) {
        return Ok(Bindings::unsorted(vars, Vec::new()));
    }
    let s_bound = if let End::Bound(id) = &s_end { Some(*id) } else { None };
    let o_bound = if let End::Bound(id) = &o_end { Some(*id) } else { None };

    let mut rows: Vec<Row> = Vec::new();
    let mut seen: FxHashSet<Row> = FxHashSet::default();
    // DefaultGraphMode::Empty (L1 dataset view): no data pairs at top-level graph
    // scope — exactly the empty-graph evaluation. The zero-length constant
    // solutions below still apply (`<s> p* <s>` holds even on an empty graph).
    if !(view::default_is_empty() || matches!(s_end, End::Missing(_)) || matches!(o_end, End::Missing(_))) {
        let ends = PathEnds { s: s_bound, o: o_bound };
        // `?x p ?x` (same variable at both ends — necessarily both unbound): only
        // diagonal pairs survive the filter, and for the recursive operators the
        // diagonal is computable WITHOUT the all-pairs closure: the zero-length
        // operators' diagonal is exactly the node domain, and `p+`'s diagonal is
        // the set of nodes on a directed cycle (SCC size >= 2, or a self-loop).
        let pairs: FxHashSet<(Id, Id)> = if same_var {
            match path {
                PropertyPathExpression::ZeroOrMore(_) | PropertyPathExpression::ZeroOrOne(_) => {
                    graph_nodes(graph).into_iter().map(|n| (n, n)).collect()
                }
                PropertyPathExpression::OneOrMore(a) => {
                    cyclic_nodes(graph, a)?.into_iter().map(|n| (n, n)).collect()
                }
                _ => path_pairs(graph, path, ends)?,
            }
        } else {
            path_pairs(graph, path, ends)?
        };
        for (s, o) in pairs {
            if s_bound.is_some_and(|b| s != b) || o_bound.is_some_and(|b| o != b) || (same_var && s != o) {
                continue;
            }
            let mut row: Row = SmallVec::new();
            if s_var.is_some() {
                row.push(s);
            }
            if o_var.is_some() && !same_var {
                row.push(o);
            }
            if seen.insert(row.clone()) {
                rows.push(row); // property-path solutions are a set (DISTINCT)
            }
        }
    }
    if zero_len {
        // The zero-length solution for a CONSTANT endpoint: `<s> p* ?x` yields
        // {?x -> <s>} even on the empty graph (interning the absent term locally);
        // `<s> p* <s>` yields the unit solution. (Variable–variable zero-length
        // solutions over the graph's nodes come from `path_pairs` above.)
        let const_id = |e: &End, local: &mut LocalVocab| match e {
            End::Bound(id) => Some(*id),
            End::Missing(t) => Some(local.intern(t.clone())),
            End::Var(_) => None,
        };
        let zrow: Option<Row> = match (&s_end, &o_end) {
            (End::Var(_), End::Var(_)) => None,
            (s_c, End::Var(_)) => const_id(s_c, local).map(|id| std::iter::once(id).collect()),
            (End::Var(_), o_c) => const_id(o_c, local).map(|id| std::iter::once(id).collect()),
            (s_c, o_c) => {
                let (a, b) = (const_id(s_c, local), const_id(o_c, local));
                (a == b).then(SmallVec::new)
            }
        };
        if let Some(row) = zrow {
            if seen.insert(row.clone()) {
                rows.push(row);
            }
        }
    }
    Ok(Bindings::unsorted(vars, rows))
}

/// Endpoint constraints pushed down into a path-relation computation (`None` =
/// that end is unbound). CONTRACT: `path_pairs(graph, path, ends)` returns a
/// SUBSET of the path's full (start,end) relation that contains EVERY pair
/// satisfying the bounds. A sub-evaluation is free to IGNORE the hint and
/// return extra relation pairs (callers always post-filter), but must never
/// invent pairs outside the relation — so the pushdown is purely an
/// optimisation and the post-filter in `eval_path` is the correctness backstop.
#[derive(Clone, Copy, Default)]
struct PathEnds {
    s: Option<Id>,
    o: Option<Id>,
}

impl PathEnds {
    const NONE: PathEnds = PathEnds { s: None, o: None };
    /// The constraint seen through `^path` (endpoints exchange roles).
    #[inline]
    fn swapped(self) -> PathEnds {
        PathEnds { s: self.o, o: self.s }
    }
}

/// When a `Sequence` has a bound outer endpoint, the midpoints reached by the
/// near hop are pushed one at a time into the far hop — but only while the
/// fan-out stays below this limit. Above it, the per-midpoint sub-evaluations
/// (each at least a binary search; a whole traversal for a recursive hop)
/// can exceed the single bulk evaluation they replace, so the far hop then
/// DELIBERATELY gets only the outer endpoint pushed and the midpoints meet in
/// the hash join instead.
const SEQ_MIDPOINT_FANOUT_LIMIT: usize = 1024;

/// All `(subject, object)` id pairs connected by a property path expression,
/// narrowed by any bound endpoints (see [`PathEnds`] for the exact contract).
/// Bound endpoints reach the leaves as range-scan prefixes and turn the
/// recursive operators into single-source directed traversals.
fn path_pairs(graph: &Graph, path: &PropertyPathExpression, ends: PathEnds) -> Result<FxHashSet<(Id, Id)>, String> {
    use PropertyPathExpression as P;
    Ok(match path {
        P::NamedNode(p) => predicate_pairs(graph, p, ends),
        P::Reverse(a) => path_pairs(graph, a, ends.swapped())?.into_iter().map(|(s, o)| (o, s)).collect(),
        P::Sequence(a, c) => {
            if let Some(s) = ends.s {
                // Bound start: evaluate the near hop from `s` only, then push each
                // reached midpoint into the far hop (which also receives the bound
                // object, enabling early exit deeper down).
                let av = path_pairs(graph, a, PathEnds { s: Some(s), o: None })?;
                let mids: FxHashSet<Id> = av.iter().filter(|&&(x, _)| x == s).map(|&(_, m)| m).collect();
                if mids.len() <= SEQ_MIDPOINT_FANOUT_LIMIT {
                    let mut out = FxHashSet::default();
                    for &m in &mids {
                        for (m2, o) in path_pairs(graph, c, PathEnds { s: Some(m), o: ends.o })? {
                            if m2 == m {
                                out.insert((s, o));
                            }
                        }
                    }
                    out
                } else {
                    // Fan-out too large for per-midpoint pushes: the far hop gets
                    // only the outer bound endpoint.
                    join_seq(av, path_pairs(graph, c, PathEnds { s: None, o: ends.o })?)
                }
            } else if let Some(o) = ends.o {
                // Bound object only: mirror image — far hop backwards from `o`,
                // midpoints pushed into the near hop as bound objects.
                let cv = path_pairs(graph, c, PathEnds { s: None, o: Some(o) })?;
                let mids: FxHashSet<Id> = cv.iter().filter(|&&(_, y)| y == o).map(|&(m, _)| m).collect();
                if mids.len() <= SEQ_MIDPOINT_FANOUT_LIMIT {
                    let mut out = FxHashSet::default();
                    for &m in &mids {
                        for (s, m2) in path_pairs(graph, a, PathEnds { s: None, o: Some(m) })? {
                            if m2 == m {
                                out.insert((s, o));
                            }
                        }
                    }
                    out
                } else {
                    join_seq(path_pairs(graph, a, PathEnds::NONE)?, cv)
                }
            } else {
                join_seq(path_pairs(graph, a, PathEnds::NONE)?, path_pairs(graph, c, PathEnds::NONE)?)
            }
        }
        // Endpoints push into BOTH branches of an alternative unchanged.
        P::Alternative(a, c) => {
            let mut s = path_pairs(graph, a, ends)?;
            s.extend(path_pairs(graph, c, ends)?);
            s
        }
        P::OneOrMore(a) => match (ends.s, ends.o) {
            (Some(s), _) => directed_reach(graph, a, s, ends.o, Dir::Fwd)?.into_iter().map(|r| (s, r)).collect(),
            (None, Some(o)) => directed_reach(graph, a, o, None, Dir::Rev)?.into_iter().map(|r| (r, o)).collect(),
            (None, None) => transitive_closure_pairs(path_pairs(graph, a, PathEnds::NONE)?),
        },
        P::ZeroOrMore(a) => match (ends.s, ends.o) {
            // A bound endpoint needs only ITS reflexive pair, not the whole node
            // domain (`<s> p* <s>` holds for any term, see the zero-length rules).
            (Some(s), _) => {
                let mut c: FxHashSet<(Id, Id)> =
                    directed_reach(graph, a, s, ends.o, Dir::Fwd)?.into_iter().map(|r| (s, r)).collect();
                c.insert((s, s));
                c
            }
            (None, Some(o)) => {
                let mut c: FxHashSet<(Id, Id)> =
                    directed_reach(graph, a, o, None, Dir::Rev)?.into_iter().map(|r| (r, o)).collect();
                c.insert((o, o));
                c
            }
            (None, None) => {
                let mut c = transitive_closure_pairs(path_pairs(graph, a, PathEnds::NONE)?);
                c.extend(graph_nodes(graph).into_iter().map(|n| (n, n)));
                c
            }
        },
        P::ZeroOrOne(a) => {
            let mut s = path_pairs(graph, a, ends)?;
            match (ends.s, ends.o) {
                // Bound endpoint: only its own reflexive pair (no full-store node scan).
                (Some(x), None) | (None, Some(x)) => {
                    s.insert((x, x));
                }
                (Some(x), Some(y)) => {
                    if x == y {
                        s.insert((x, x));
                    }
                }
                (None, None) => s.extend(graph_nodes(graph).into_iter().map(|n| (n, n))),
            }
            s
        }
        P::NegatedPropertySet(props) => negated_property_pairs(graph, props, ends),
    })
}

/// Hash join of two path relations on the shared midpoint (`a.end == c.start`).
fn join_seq(av: FxHashSet<(Id, Id)>, cv: FxHashSet<(Id, Id)>) -> FxHashSet<(Id, Id)> {
    let mut by_start: FxHashMap<Id, Vec<Id>> = FxHashMap::default();
    for (m, o) in cv {
        by_start.entry(m).or_default().push(o);
    }
    let mut out = FxHashSet::default();
    for (s, m) in av {
        if let Some(os) = by_start.get(&m) {
            for &o in os {
                out.insert((s, o));
            }
        }
    }
    out
}

/// Traversal direction for [`directed_reach`]: forward follows the sub-path
/// from a bound SUBJECT; reverse walks it backwards from a bound OBJECT (the
/// O-leading permutations answer the reversed leaf scans).
#[derive(Clone, Copy, PartialEq)]
enum Dir {
    Fwd,
    Rev,
}

/// Single-source reachability (one or more steps) from `start` under the
/// sub-path: a budget-checked BFS whose frontier expansion is a bounded
/// sub-evaluation — for a plain-predicate sub-path, one sorted range scan
/// (`[Some(n), Some(p), None]`, or `[None, Some(p), Some(n)]` reversed) per
/// node. With `target` bound the walk is a reachability TEST and stops as soon
/// as the target is reached.
fn directed_reach(
    graph: &Graph,
    sub: &PropertyPathExpression,
    start: Id,
    target: Option<Id>,
    dir: Dir,
) -> Result<FxHashSet<Id>, String> {
    // Plain-predicate fast path: successors come straight from a range scan
    // (no per-node relation set). `Some(None)` = predicate not in the
    // dictionary, hence no edges at all.
    let pid: Option<Option<Id>> = match sub {
        PropertyPathExpression::NamedNode(p) => Some(graph.id_of(&Term::NamedNode(p.clone()))),
        _ => None,
    };
    let step = |node: Id, out: &mut Vec<Id>| -> Result<(), String> {
        match pid {
            Some(None) => {}
            Some(Some(pid)) => {
                let pat: IdPattern = match dir {
                    Dir::Fwd => [Some(node), Some(pid), None],
                    Dir::Rev => [None, Some(pid), Some(node)],
                };
                let scan = graph.store.scan(&pat);
                let col = match dir {
                    Dir::Fwd => 2,
                    Dir::Rev => 0,
                };
                out.extend(scan.rows.iter().map(|r| scan.to_spo(r)[col]));
            }
            // Composite sub-path: one bounded sub-evaluation per node (its own
            // endpoints push recursively). The filter enforces the contract's
            // "may return extra relation pairs" clause.
            None => {
                let ends = match dir {
                    Dir::Fwd => PathEnds { s: Some(node), o: None },
                    Dir::Rev => PathEnds { s: None, o: Some(node) },
                };
                for (s, o) in path_pairs(graph, sub, ends)? {
                    match dir {
                        Dir::Fwd if s == node => out.push(o),
                        Dir::Rev if o == node => out.push(s),
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    };
    let mut seen: FxHashSet<Id> = FxHashSet::default();
    let mut stack: Vec<Id> = Vec::new();
    step(start, &mut stack)?;
    let mut pops = 0usize;
    while let Some(n) = stack.pop() {
        // Budget granularity: check INSIDE the walk (every 1024 pops), so one
        // runaway traversal respects the budget promptly.
        pops += 1;
        if pops & 0x3FF == 0 {
            budget::check(seen.len())?;
        }
        if seen.insert(n) {
            if target == Some(n) {
                break;
            }
            step(n, &mut stack)?;
        }
    }
    Ok(seen)
}

/// The nodes lying on a directed cycle of the sub-path's relation — exactly the
/// solutions of `?x p+ ?x` — via Kosaraju SCC over the base relation: every
/// member of an SCC of size >= 2, plus self-loop nodes. `O(V + E)` instead of
/// the all-pairs closure's `O(V·E)`.
fn cyclic_nodes(graph: &Graph, sub: &PropertyPathExpression) -> Result<FxHashSet<Id>, String> {
    let base = path_pairs(graph, sub, PathEnds::NONE)?;
    let mut adj: FxHashMap<Id, Vec<Id>> = FxHashMap::default();
    let mut radj: FxHashMap<Id, Vec<Id>> = FxHashMap::default();
    let mut cyclic: FxHashSet<Id> = FxHashSet::default();
    for &(s, o) in &base {
        if s == o {
            cyclic.insert(s); // self-loop: a 1-cycle
        }
        adj.entry(s).or_default().push(o);
        radj.entry(o).or_default().push(s);
    }
    // Pass 1: iterative DFS post-order (every node with an out-edge is a root
    // candidate; sinks are reached as children — a cycle member always has an
    // out-edge, so coverage is complete).
    let mut order: Vec<Id> = Vec::new();
    let mut visited: FxHashSet<Id> = FxHashSet::default();
    let mut steps = 0usize;
    let roots: Vec<Id> = adj.keys().copied().collect();
    for &root in &roots {
        if !visited.insert(root) {
            continue;
        }
        let mut stack: Vec<(Id, usize)> = vec![(root, 0)];
        while let Some(frame) = stack.last_mut() {
            steps += 1;
            if steps & 0x3FF == 0 {
                budget::check(order.len())?;
            }
            let (n, i) = *frame;
            match adj.get(&n).and_then(|v| v.get(i).copied()) {
                Some(m) => {
                    frame.1 += 1;
                    if visited.insert(m) {
                        stack.push((m, 0));
                    }
                }
                None => {
                    order.push(n);
                    stack.pop();
                }
            }
        }
    }
    // Pass 2: components of the TRANSPOSE graph, roots taken in reverse
    // post-order; a component of >= 2 members is a cycle through all of them.
    let mut assigned: FxHashSet<Id> = FxHashSet::default();
    for &root in order.iter().rev() {
        if !assigned.insert(root) {
            continue;
        }
        let mut members: Vec<Id> = vec![root];
        let mut stack: Vec<Id> = vec![root];
        while let Some(n) = stack.pop() {
            steps += 1;
            if steps & 0x3FF == 0 {
                budget::check(members.len())?;
            }
            if let Some(ps) = radj.get(&n) {
                for &m in ps {
                    if assigned.insert(m) {
                        members.push(m);
                        stack.push(m);
                    }
                }
            }
        }
        if members.len() >= 2 {
            cyclic.extend(members);
        }
    }
    Ok(cyclic)
}

/// All `(s, o)` for a single predicate IRI (empty if the predicate isn't in the
/// graph), narrowed by any bound endpoints — `[s?, p, o?]` is always a single
/// contiguous range in some built permutation.
fn predicate_pairs(graph: &Graph, p: &oxrdf::NamedNode, ends: PathEnds) -> FxHashSet<(Id, Id)> {
    match graph.id_of(&Term::NamedNode(p.clone())) {
        None => FxHashSet::default(),
        Some(pid) => {
            let pat: IdPattern = [ends.s, Some(pid), ends.o];
            let scan = graph.store.scan(&pat);
            scan.rows
                .iter()
                .map(|r| {
                    let t = scan.to_spo(r);
                    (t[0], t[2])
                })
                .collect()
        }
    }
}

/// `!(...)` — every edge whose predicate is NOT in the excluded set. A bound
/// endpoint narrows the scan to that one node's triples; the fully-unbound case
/// walks a P-leading permutation and skips each excluded predicate's contiguous
/// block wholesale (binary search to the block end — no per-triple set probe
/// over the excluded mass).
fn negated_property_pairs(graph: &Graph, props: &[oxrdf::NamedNode], ends: PathEnds) -> FxHashSet<(Id, Id)> {
    let excluded: FxHashSet<Id> = props.iter().filter_map(|p| graph.id_of(&Term::NamedNode(p.clone()))).collect();
    if ends.s.is_some() || ends.o.is_some() {
        let pat: IdPattern = [ends.s, None, ends.o];
        let scan = graph.store.scan(&pat);
        return scan
            .rows
            .iter()
            .filter_map(|r| {
                let t = scan.to_spo(r);
                (!excluded.contains(&t[1])).then_some((t[0], t[2]))
            })
            .collect();
    }
    // Both BUILT index sets contain a P-leading permutation (PSO full / POS
    // compact), so `scan_sorted` finds one; the fallback filter-scan defends
    // against a future index set where it doesn't.
    let scan = graph.store.scan_sorted(&[None, None, None], 1);
    let rows = &scan.rows[..];
    let mut out = FxHashSet::default();
    if scan.perm.order()[0] == 1 {
        let mut i = 0;
        while i < rows.len() {
            let p = rows[i][0];
            let j = i + rows[i..].partition_point(|r| r[0] == p);
            if !excluded.contains(&p) {
                for r in &rows[i..j] {
                    let t = scan.to_spo(r);
                    out.insert((t[0], t[2]));
                }
            }
            i = j;
        }
    } else {
        for r in rows {
            let t = scan.to_spo(r);
            if !excluded.contains(&t[1]) {
                out.insert((t[0], t[2]));
            }
        }
    }
    out
}

/// Every id that appears as a subject or object (the domain of zero-length path matches).
fn graph_nodes(graph: &Graph) -> FxHashSet<Id> {
    let pat: IdPattern = [None, None, None];
    let scan = graph.store.scan(&pat);
    let mut s = FxHashSet::default();
    for r in scan.rows.iter() {
        let t = scan.to_spo(r);
        s.insert(t[0]);
        s.insert(t[2]);
    }
    s
}

/// Transitive (NOT reflexive) closure of a pair relation — BFS of reachability from each start.
fn transitive_closure_pairs(pairs: FxHashSet<(Id, Id)>) -> FxHashSet<(Id, Id)> {
    let mut adj: FxHashMap<Id, Vec<Id>> = FxHashMap::default();
    for (s, o) in &pairs {
        adj.entry(*s).or_default().push(*o);
    }
    let mut out: FxHashSet<(Id, Id)> = FxHashSet::default();
    let starts: Vec<Id> = adj.keys().copied().collect();
    let mut pops = 0usize;
    for start in starts {
        // Budget check per BFS start node (sticky; the caller's next check
        // raises the error)…
        if budget::exhausted(out.len()) {
            break;
        }
        let mut seen: FxHashSet<Id> = FxHashSet::default();
        let mut stack: Vec<Id> = adj.get(&start).cloned().unwrap_or_default();
        while let Some(n) = stack.pop() {
            // …and every 1024 expansions INSIDE the walk, so one runaway start
            // node cannot overshoot the budget by a whole graph traversal.
            pops += 1;
            if pops & 0x3FF == 0 && budget::exhausted(out.len()) {
                return out;
            }
            if seen.insert(n) {
                out.insert((start, n));
                if let Some(nexts) = adj.get(&n) {
                    stack.extend(nexts.iter().copied());
                }
            }
        }
    }
    out
}

fn eval_bgp(graph: &Graph, patterns: &[TriplePattern]) -> Result<Bindings, String> {
    if patterns.is_empty() {
        return Ok(Bindings { vars: vec![], rows: vec![Row::new()], sorted_by: None });
    }
    // DefaultGraphMode::Empty (L1 dataset view): a non-empty BGP at top-level
    // graph scope has ZERO rows, with its normal variable schema (the empty BGP
    // above keeps its unit row; GRAPH scope suspends the flag).
    if view::default_is_empty() {
        return Ok(Bindings::unsorted(collect_vars(patterns), vec![]));
    }
    // RDF 1.2 quoted-triple patterns with variables decompose into synthetic-variable
    // slots + structural-unification relations, joined by the ordinary machinery (F14).
    let (rewritten, constraints) = extract_quoted_constraints(patterns);
    if !constraints.is_empty() {
        // zk-trace: structural-unification relations scan the store without
        // per-pattern attribution — mark Op::QuotedTriples so a consumer
        // fails closed (ZkTrace::first_uncaptured).
        #[cfg(feature = "zk")]
        let _zk = crate::zk::op_scope(crate::zk::Op::QuotedTriples);
        let mut b = eval_bgp(graph, &rewritten)?;
        for c in &constraints {
            b = join_bindings(b, quoted_relation(graph, c));
        }
        return Ok(b);
    }
    if patterns.len() >= 3 && bgp_is_cyclic(patterns) {
        return eval_bgp_wcoj(graph, patterns);
    }
    eval_bgp_binary(graph, patterns, &[])
}

// ---- RDF 1.2 quoted-triple patterns with variables (F14) ----------------------

/// Prefix for the synthetic variables standing in for a quoted-triple pattern slot
/// (like [`BNODE_VAR_PREFIX`], `#` cannot appear in a SPARQL VARNAME).
const QT_VAR_PREFIX: &str = "#qt#";

/// One BGP slot that held a quoted-triple pattern CONTAINING VARIABLES, replaced by a
/// synthetic variable. Its relation enumerates every stored triple term and unifies it
/// structurally against the quoted pattern, binding the inner variables.
struct QuotedConstraint {
    var: Variable,
    pattern: TriplePattern,
}

/// `true` when a quoted-triple pattern has a variable / blank node anywhere inside
/// (such a pattern cannot resolve to a single dictionary id).
fn quoted_has_var(t: &TriplePattern) -> bool {
    fn slot(tp: &TermPattern) -> bool {
        match tp {
            TermPattern::Variable(_) | TermPattern::BlankNode(_) => true,
            TermPattern::Triple(inner) => quoted_has_var(inner),
            _ => false,
        }
    }
    slot(&t.subject) || matches!(t.predicate, NamedNodePattern::Variable(_)) || slot(&t.object)
}

/// Rewrites the BGP: every subject/object slot holding a variable-carrying quoted-triple
/// pattern becomes a fresh synthetic variable, with the quoted pattern recorded as a
/// [`QuotedConstraint`]. Ground quoted triples are untouched (they resolve to one id).
fn extract_quoted_constraints(patterns: &[TriplePattern]) -> (Vec<TriplePattern>, Vec<QuotedConstraint>) {
    let mut out = Vec::with_capacity(patterns.len());
    let mut constraints: Vec<QuotedConstraint> = Vec::new();
    for tp in patterns {
        let mut tp = tp.clone();
        for s in [&mut tp.subject, &mut tp.object] {
            if let TermPattern::Triple(t) = s {
                if quoted_has_var(t) {
                    let var = Variable::new_unchecked(format!("{QT_VAR_PREFIX}{}", constraints.len()));
                    constraints.push(QuotedConstraint { var: var.clone(), pattern: (**t).clone() });
                    *s = TermPattern::Variable(var);
                }
            }
        }
        out.push(tp);
    }
    (out, constraints)
}

/// The variables of a quoted-triple pattern in first-occurrence order (blank nodes as
/// their synthetic existential variables), appended to `out` without duplicates.
fn collect_quoted_vars(t: &TriplePattern, out: &mut Vec<Variable>) {
    fn push(out: &mut Vec<Variable>, v: Variable) {
        if !out.contains(&v) {
            out.push(v);
        }
    }
    fn slot(tp: &TermPattern, out: &mut Vec<Variable>) {
        match tp {
            TermPattern::Variable(v) => push(out, v.clone()),
            TermPattern::BlankNode(b) => push(out, bnode_var(b)),
            TermPattern::Triple(inner) => collect_quoted_vars(inner, out),
            _ => {}
        }
    }
    slot(&t.subject, out);
    if let NamedNodePattern::Variable(v) = &t.predicate {
        push(out, v.clone());
    }
    slot(&t.object, out);
}

/// Builds the constraint's relation: one row per stored triple term that structurally
/// unifies with the quoted pattern — columns are the synthetic slot variable (bound to
/// the triple term's own id) followed by the quoted pattern's inner variables.
///
/// Enumeration scans the dictionary for `TermParts::Triple` records: triple terms are a
/// vanishing fraction of real dictionaries and the scan only runs for queries that quote
/// variables, so no ordinary query pays for it. (A persistent side index of triple-term
/// ids is the obvious upgrade if quoted-pattern workloads ever matter at scale.)
fn quoted_relation(graph: &Graph, c: &QuotedConstraint) -> Bindings {
    let mut vars = vec![c.var.clone()];
    collect_quoted_vars(&c.pattern, &mut vars);
    let mut rows: Vec<Row> = Vec::new();
    for id in 1..=graph.dict.len() as Id {
        let dict::TermParts::Triple(comps) = graph.dict.term_parts(id) else {
            continue;
        };
        let mut binds: Row = std::iter::repeat_n(NO_ID, vars.len()).collect();
        binds[0] = id;
        if unify_quoted(graph, &c.pattern, comps, &vars, &mut binds) {
            rows.push(binds);
        }
    }
    Bindings::unsorted(vars, rows)
}

/// Binds `v` to `id`, or checks consistency when the variable is already bound
/// (the same variable repeated inside a quoted pattern must match the same id).
fn bind_quoted_var(v: &Variable, id: Id, vars: &[Variable], binds: &mut [Id]) -> bool {
    let i = vars.iter().position(|x| x == v).expect("quoted var collected");
    if binds[i] == NO_ID {
        binds[i] = id;
        true
    } else {
        binds[i] == id
    }
}

/// Structurally unifies a quoted-triple pattern against a stored triple term's
/// component ids, recursing through nested quoted patterns.
fn unify_quoted(graph: &Graph, pat: &TriplePattern, comps: [Id; 3], vars: &[Variable], binds: &mut [Id]) -> bool {
    fn slot(graph: &Graph, tp: &TermPattern, id: Id, vars: &[Variable], binds: &mut [Id]) -> bool {
        match tp {
            TermPattern::Variable(v) => bind_quoted_var(v, id, vars, binds),
            TermPattern::BlankNode(b) => bind_quoted_var(&bnode_var(b), id, vars, binds),
            TermPattern::Triple(inner) => {
                if dict::is_inline(id) {
                    return false;
                }
                match graph.dict.term_parts(id) {
                    dict::TermParts::Triple(c) => unify_quoted(graph, inner, c, vars, binds),
                    _ => false,
                }
            }
            // A ground component: term-identity match (same as ordinary BGP slots).
            other => match term_pattern_to_term(other) {
                Ok(t) => graph.id_of(&t) == Some(id),
                Err(_) => false,
            },
        }
    }
    if !slot(graph, &pat.subject, comps[0], vars, binds) {
        return false;
    }
    match &pat.predicate {
        NamedNodePattern::NamedNode(n) => {
            if graph.id_of(&Term::NamedNode(n.clone())) != Some(comps[1]) {
                return false;
            }
        }
        NamedNodePattern::Variable(v) => {
            if !bind_quoted_var(v, comps[1], vars, binds) {
                return false;
            }
        }
    }
    slot(graph, &pat.object, comps[2], vars, binds)
}

/// Whether this conjunctive BGP would be routed to the worst-case-optimal plan
/// (so the caller knows whether sargable-filter pushdown into the binary scan
/// applies).
pub(crate) fn bgp_uses_binary(patterns: &[TriplePattern]) -> bool {
    !(patterns.len() >= 3 && bgp_is_cyclic(patterns))
}

/// Binary-join BGP plan: greedy cardinality ordering with sort-merge joins on the
/// current sort variable (falling back to hash, then cross product). `pat_filters`
/// holds an optional pushed-down numeric FILTER per pattern (by original index).
fn eval_bgp_binary(graph: &Graph, patterns: &[TriplePattern], pat_filters: &[Option<(usize, ScanCmp)>]) -> Result<Bindings, String> {
    if patterns.is_empty() {
        return Ok(Bindings { vars: vec![], rows: vec![Row::new()], sorted_by: None });
    }
    // L1 dataset view: the conjunctive-flattening path calls this directly
    // (bypassing eval_bgp), so the empty-default short-circuit must be here too.
    if view::default_is_empty() {
        return Ok(Bindings::unsorted(collect_vars(patterns), vec![]));
    }
    // Quoted-triple patterns with variables (F14): the conjunctive-flattening path calls
    // this directly (bypassing eval_bgp), so the decomposition must happen here too.
    // The rewrite preserves pattern count/order, so `pat_filters` indexes stay aligned.
    let (rewritten, constraints) = extract_quoted_constraints(patterns);
    if !constraints.is_empty() {
        #[cfg(feature = "zk")]
        let _zk = crate::zk::op_scope(crate::zk::Op::QuotedTriples);
        let mut b = eval_bgp_binary(graph, &rewritten, pat_filters)?;
        for c in &constraints {
            b = join_bindings(b, quoted_relation(graph, c));
        }
        return Ok(b);
    }
    let pfilter = |i: usize| -> Option<(usize, ScanCmp)> { pat_filters.get(i).copied().flatten() };

    let prepared = prepare_bgp(graph, patterns)?;
    if prepared.iter().any(|p| p.unsatisfiable) {
        // zk-trace: an unsatisfiable constant (a term absent from the
        // dictionary) is a PROVABLY-EMPTY input set — the per-property proof
        // must witness "no such triple exists". Record ONLY the patterns that
        // are provably empty; a satisfiable SIBLING was never consumed (the
        // join short-circuits), so claiming it empty would over-state the
        // trace.
        #[cfg(feature = "zk")]
        if crate::zk::enabled() {
            for (tp, prep) in patterns.iter().zip(&prepared) {
                if prep.unsatisfiable {
                    crate::zk::record_empty_pattern(crate::zk::key_of_algebra_pattern(tp));
                }
            }
        }
        return Ok(Bindings::unsorted(collect_vars(patterns), vec![]));
    }

    let var_pos = |i: usize, v: &Variable| -> Option<usize> { prepared[i].var_pos(v) };

    // Cost-based greedy (GOO): seed with the smallest single-pattern cardinality,
    // then repeatedly add the connected pattern that yields the smallest *estimated
    // join result*, using the per-predicate characteristic stats (distinct
    // subjects/objects) to estimate join selectivity. The join order only affects
    // performance (the result is identical for any order — differentially tested).
    // The decision logic lives in `goo_seed` / `goo_seed_sort` / `goo_pick` /
    // `record_pattern_ndv`, shared verbatim with the T22 EXPLAIN dry-run planner.
    let mut cs_ctx = CsCtx::new(&prepared);
    let seed = goo_seed(&prepared);
    let seed_sort_col = goo_seed_sort(&prepared, seed, pfilter(seed).map(|(c, _)| c));

    let mut result = scan_to_bindings(graph, &prepared[seed].id_pat, &prepared[seed].pos_vars, seed_sort_col, pfilter(seed), None);
    let mut done = vec![false; prepared.len()];
    done[seed] = true;
    cs_ctx.note_done(seed);

    // Running estimate of the result cardinality and the per-variable distinct
    // count (ndv), used to score the next join.
    let mut cur_card = prepared[seed].est as f64;
    let mut var_ndv: FxHashMap<Variable, f64> = FxHashMap::default();
    record_pattern_ndv(graph, &prepared, seed, cur_card, &mut var_ndv, &cs_ctx);

    for _ in 1..prepared.len() {
        // Pick the connected candidate with the smallest estimated output.
        let (i, new_card, _connected) = goo_pick(graph, &prepared, &done, &var_ndv, cur_card, &cs_ctx);
        cur_card = new_card;
        done[i] = true;
        cs_ctx.note_done(i);

        // Index-nested-loop (bind) join: when the running result is MUCH smaller than
        // the next pattern and exactly one variable connects them, look up each result
        // join value in the pattern's index (a bound scan) instead of scanning the whole
        // (large) relation and merge/hash-joining. This is the win on a selective join —
        // e.g. a chain whose far end is selective — where the merge would scan millions
        // of rows to match a few thousand. Same result, validated differentially.
        let connecting: Vec<Variable> = result.vars.iter().filter(|v| var_pos(i, v).is_some()).cloned().collect();
        if connecting.len() == 1
            && distinct_pattern_vars(&prepared[i].pos_vars)
            && result.rows.len().saturating_mul(8) < prepared[i].est
        {
            let jv = &connecting[0];
            let rk = result.col(jv).unwrap();
            let pp = var_pos(i, jv).unwrap();
            result = bind_join(graph, result, &prepared[i].id_pat, &prepared[i].pos_vars, rk, pp, pfilter(i));
            record_pattern_ndv(graph, &prepared, i, cur_card, &mut var_ndv, &cs_ctx);
            if result.rows.is_empty() {
                break;
            }
            continue;
        }

        // Execute: a pushed-down filter forces the scan into its own column order
        // (and filters inline); otherwise sort by the join variable for a merge.
        let filt = pfilter(i);
        let merge_var = result.sorted_by.clone().filter(|sv| var_pos(i, sv).is_some());
        let scan_sort = filt.map(|(c, _)| c).or_else(|| merge_var.as_ref().map(|jv| var_pos(i, jv).unwrap()));
        let rhs = scan_to_bindings(graph, &prepared[i].id_pat, &prepared[i].pos_vars, scan_sort, filt, None);
        let connected = prepared[i].pos_vars.iter().flatten().any(|v| result.vars.contains(v));
        // Merge only when both sides are sorted on the join variable (a filter may
        // have forced the scan into a different order).
        if let Some(jv) = merge_var.filter(|jv| rhs.sorted_by.as_ref() == Some(jv)) {
            result = merge_join(result, rhs, &jv);
        } else if connected {
            result = hash_join(result, rhs);
        } else {
            result = cross_product(result, rhs);
        }
        record_pattern_ndv(graph, &prepared, i, cur_card, &mut var_ndv, &cs_ctx);

        if result.rows.is_empty() {
            break;
        }
    }
    Ok(result)
}

// ---- Shared GOO planning decisions (executor + T22 EXPLAIN dry run) -----------
//
// The greedy-ordering decisions of `eval_bgp_binary` are factored into the small
// pure helpers below so EXPLAIN can REPLAY the planner without executing anything
// and without duplicating the logic (no drift): the executor calls them on its hot
// path (inlined, zero extra cost), the explain module replays them symbolically.

/// One BGP triple pattern prepared for planning: resolved constant ids, the
/// variable at each canonical position, and the index-range cardinality estimate.
pub(crate) struct Prepared {
    pub(crate) id_pat: IdPattern,
    pub(crate) pos_vars: [Option<Variable>; 3],
    pub(crate) est: usize,
    pub(crate) unsatisfiable: bool,
}

impl Prepared {
    /// Canonical position (0=s, 1=p, 2=o) of `v` in this pattern, if present.
    #[inline]
    pub(crate) fn var_pos(&self, v: &Variable) -> Option<usize> {
        self.pos_vars.iter().position(|pv| pv.as_ref() == Some(v))
    }
}

/// Prepares every pattern of a BGP for planning (constant resolution + estimates).
pub(crate) fn prepare_bgp(graph: &Graph, patterns: &[TriplePattern]) -> Result<Vec<Prepared>, String> {
    let mut prepared: Vec<Prepared> = Vec::with_capacity(patterns.len());
    for tp in patterns {
        let (id_pat, pos_vars, unsat) = prepare_pattern(graph, tp)?;
        let est = if unsat { 0 } else { graph.store.estimate(&id_pat) };
        prepared.push(Prepared { id_pat, pos_vars, est, unsatisfiable: unsat });
    }
    Ok(prepared)
}

/// GOO seed choice: the pattern with the smallest single-pattern cardinality.
pub(crate) fn goo_seed(prepared: &[Prepared]) -> usize {
    (0..prepared.len()).min_by_key(|&i| prepared[i].est).unwrap()
}

/// The seed scan's requested sort column: a pushed-down filter scans in its own
/// column's order (sequential numeric access); otherwise sort by the first seed
/// variable shared with another pattern, to enable a merge join.
pub(crate) fn goo_seed_sort(prepared: &[Prepared], seed: usize, filter_col: Option<usize>) -> Option<usize> {
    if filter_col.is_some() {
        return filter_col;
    }
    prepared[seed]
        .pos_vars
        .iter()
        .flatten()
        .find(|v| (0..prepared.len()).any(|j| j != seed && prepared[j].var_pos(v).is_some()))
        .and_then(|v| prepared[seed].var_pos(v))
}

/// The characteristic-set planning context (the opt-in `cs-planner` feature) the
/// GOO helpers consult for STAR joins: with a `crate::cs::CsTable` installed
/// (see [`crate::with_cs_table`]), candidate scoring and subject-variable ndv for
/// patterns of the star shape `?s <p> ?o` come from the CS table instead of the
/// per-predicate independence model. Without the feature this is a zero-sized
/// no-op; without an installed table every method is `None` and the `PredStat`
/// path runs unchanged. Either way, only JOIN ORDER is affected — never results.
pub(crate) struct CsCtx {
    #[cfg(feature = "cs-planner")]
    inner: Option<crate::cs::StarCtx>,
}

impl CsCtx {
    pub(crate) fn new(prepared: &[Prepared]) -> CsCtx {
        #[cfg(feature = "cs-planner")]
        {
            let inner = crate::cs::active().map(|table| {
                crate::cs::StarCtx::new(
                    table,
                    prepared.iter().map(|p| match (&p.pos_vars[0], p.id_pat) {
                        // The star shape: subject VARIABLE, bound predicate, unbound object.
                        (Some(v), [None, Some(pid), None]) => Some((v.clone(), pid)),
                        _ => None,
                    }),
                )
            });
            CsCtx { inner }
        }
        #[cfg(not(feature = "cs-planner"))]
        {
            let _ = prepared;
            CsCtx {}
        }
    }

    /// Marks pattern `i` joined, so later star estimates condition on it.
    #[inline]
    pub(crate) fn note_done(&mut self, i: usize) {
        #[cfg(feature = "cs-planner")]
        if let Some(s) = &mut self.inner {
            s.note_done(i);
        }
        #[cfg(not(feature = "cs-planner"))]
        let _ = i;
    }

    /// CS-based candidate output estimate (see `cs::StarCtx::pick_score`).
    #[inline]
    fn pick_score(&self, i: usize, cur_card: f64) -> Option<f64> {
        #[cfg(feature = "cs-planner")]
        {
            self.inner.as_ref().and_then(|s| s.pick_score(i, cur_card))
        }
        #[cfg(not(feature = "cs-planner"))]
        {
            let _ = (i, cur_card);
            None
        }
    }

    /// CS-based subject-variable ndv (see `cs::StarCtx::subject_ndv`).
    #[inline]
    fn subject_ndv(&self, i: usize) -> Option<f64> {
        #[cfg(feature = "cs-planner")]
        {
            self.inner.as_ref().and_then(|s| s.subject_ndv(i))
        }
        #[cfg(not(feature = "cs-planner"))]
        {
            let _ = i;
            None
        }
    }
}

/// Folds pattern `i`'s per-variable distinct-value estimates into the running
/// `var_ndv` map (each variable keeps its smallest — most selective — estimate,
/// capped by the running result cardinality). With a CS table installed, a star
/// pattern's SUBJECT variable uses the table's `Σ_{C ⊇ Q} count(C)` over the star
/// joined so far (call after `CsCtx::note_done`) instead of the `PredStat` marginal.
pub(crate) fn record_pattern_ndv(
    graph: &Graph,
    prepared: &[Prepared],
    i: usize,
    cur_card: f64,
    var_ndv: &mut FxHashMap<Variable, f64>,
    cs: &CsCtx,
) {
    let p = &prepared[i];
    for (pos, ov) in p.pos_vars.iter().enumerate() {
        if let Some(v) = ov {
            let raw = match pos {
                0 => cs.subject_ndv(i).unwrap_or_else(|| pattern_var_ndv(graph, &p.id_pat, pos, p.est)),
                _ => pattern_var_ndv(graph, &p.id_pat, pos, p.est),
            };
            let ndv = raw.min(cur_card.max(1.0));
            let e = var_ndv.entry(v.clone()).or_insert(ndv);
            *e = e.min(ndv);
        }
    }
}

/// One GOO step: the connected not-yet-joined pattern with the smallest estimated
/// join output (`|R| · |P| / max(ndv)` per shared variable), or — when nothing
/// connects — the smallest remaining pattern (cross product). Returns the chosen
/// pattern index, the updated result-cardinality estimate, and whether it was
/// connected. With a CS table installed, a star candidate's subject-variable
/// contribution is the table's conditional expansion `star(Q ∪ {p}) / star(Q)`
/// (predicate-correlation-aware) instead of the independence product; selectivity
/// over any OTHER shared variables keeps the independence model.
pub(crate) fn goo_pick(
    graph: &Graph,
    prepared: &[Prepared],
    done: &[bool],
    var_ndv: &FxHashMap<Variable, f64>,
    cur_card: f64,
    cs: &CsCtx,
) -> (usize, f64, bool) {
    let mut best: Option<(usize, f64)> = None;
    for i in 0..prepared.len() {
        if done[i] {
            continue;
        }
        let cs_score = cs.pick_score(i, cur_card);
        let mut sel = 1.0f64;
        let mut shared = cs_score.is_some();
        for (pos, ov) in prepared[i].pos_vars.iter().enumerate() {
            if let Some(v) = ov {
                // The subject variable of a CS-scored star candidate is already
                // accounted for by the conditional star estimate.
                if pos == 0 && cs_score.is_some() {
                    continue;
                }
                if let Some(&rndv) = var_ndv.get(v) {
                    shared = true;
                    let pndv = pattern_var_ndv(graph, &prepared[i].id_pat, pos, prepared[i].est);
                    sel /= rndv.max(pndv).max(1.0);
                }
            }
        }
        if !shared {
            continue;
        }
        let out = match cs_score {
            Some(star) => star * sel,
            None => cur_card * prepared[i].est as f64 * sel,
        };
        if best.is_none_or(|(_, bc)| out < bc) {
            best = Some((i, out));
        }
    }
    match best {
        Some((i, out)) => (i, out.max(0.0), true),
        None => {
            // Disconnected: smallest-cardinality remaining (cross product).
            let i = (0..prepared.len()).filter(|&j| !done[j]).min_by_key(|&j| prepared[j].est).unwrap();
            (i, cur_card * prepared[i].est as f64, false)
        }
    }
}

/// Estimated number of distinct values of the variable at canonical position
/// `pos` in a pattern, from the per-predicate characteristic stats. Falls back to
/// the pattern's cardinality (an upper bound) when the predicate is unbound or the
/// other terminal is bound (so the column is effectively keyed).
fn pattern_var_ndv(graph: &Graph, id_pat: &IdPattern, pos: usize, est: usize) -> f64 {
    let est = (est as f64).max(1.0);
    let stat = id_pat[1].and_then(|pid| graph.store.pred_stat(pid));
    match (pos, stat) {
        // subject var, predicate bound, object unbound -> distinct subjects of P
        (0, Some(s)) if id_pat[2].is_none() => (s.ndv_subj as f64).clamp(1.0, est),
        // object var, predicate bound, subject unbound -> distinct objects of P
        (2, Some(s)) if id_pat[0].is_none() => (s.ndv_obj as f64).clamp(1.0, est),
        _ => est,
    }
}

pub(crate) fn collect_vars(patterns: &[TriplePattern]) -> Vec<Variable> {
    let mut vars = Vec::new();
    for tp in patterns {
        for v in [tp_var(&tp.subject), nnp_var(&tp.predicate), tp_var(&tp.object)].into_iter().flatten() {
            if !vars.contains(&v) {
                vars.push(v);
            }
        }
    }
    vars
}

// ---- Worst-case-optimal join: Leapfrog Triejoin ------------------------------
//
// LFTJ (Veldhuizen 2014) evaluates a BGP one *variable* at a time in a fixed
// global order. At each variable it intersects, via the "leapfrog" galloping
// search, the sorted value streams of every pattern that mentions that variable,
// then recurses. Each pattern is a trie of its variable columns (projected from a
// permutation index, sorted in the global variable order). The total work is
// bounded by the AGM fractional-edge-cover bound on the BGP, so for cyclic
// queries it cannot produce the asymptotically-large intermediates a binary plan
// would. See research/ARCHITECTURE.md §4.

/// A pattern's relation projected onto its variables (in global order) as sorted,
/// deduplicated tuples — the trie that LFTJ navigates level by level.
struct Trie {
    tuples: Vec<Vec<Id>>,
}

/// One open level of a [`TrieIter`]: `hi` bounds the current key's subtree (rows
/// sharing all already-fixed columns), `cur` is the cursor within it.
struct Frame {
    hi: usize,
    cur: usize,
}

/// A cursor over a [`Trie`], using Veldhuizen's open-on-entry semantics: it starts
/// *above* the root, and `open()` descends one column, resetting the cursor to the
/// start of that subtree. This reset is what makes non-contiguous variable
/// participation correct — re-entering a level re-opens (rewinds) the iterator.
struct TrieIter<'a> {
    trie: &'a Trie,
    frames: Vec<Frame>,
}

impl<'a> TrieIter<'a> {
    fn new(trie: &'a Trie) -> Self {
        TrieIter { trie, frames: Vec::new() }
    }
    /// The column currently being iterated (valid once at least one `open`).
    #[inline]
    fn col(&self) -> usize {
        self.frames.len() - 1
    }
    #[inline]
    fn at_end(&self) -> bool {
        let f = self.frames.last().unwrap();
        f.cur >= f.hi
    }
    #[inline]
    fn key(&self) -> Id {
        let col = self.col();
        let f = self.frames.last().unwrap();
        self.trie.tuples[f.cur][col]
    }
    /// End (exclusive) of the run of rows in `[start, hi)` whose `col` equals the
    /// value at `start`. The slice is sorted, so this is a binary search — O(log n)
    /// rather than a linear scan of the (possibly large) run.
    #[inline]
    fn run_end(&self, col: usize, start: usize, hi: usize, val: Id) -> usize {
        start + self.trie.tuples[start..hi].partition_point(|row| row[col] <= val)
    }
    /// Advances to the next distinct value in the current column.
    fn next(&mut self) {
        let col = self.col();
        let (cur, hi) = {
            let f = self.frames.last().unwrap();
            (f.cur, f.hi)
        };
        let val = self.trie.tuples[cur][col];
        self.frames.last_mut().unwrap().cur = self.run_end(col, cur, hi, val);
    }
    /// Galloping seek: first value `>= x` in the current column.
    fn seek(&mut self, x: Id) {
        let col = self.col();
        let f = self.frames.last_mut().unwrap();
        let (mut a, mut b) = (f.cur, f.hi);
        while a < b {
            let m = a + (b - a) / 2;
            if self.trie.tuples[m][col] < x {
                a = m + 1;
            } else {
                b = m;
            }
        }
        f.cur = a;
    }
    /// Descends one column: into the subtree of the parent's current key (or, at
    /// the root, the whole relation), with the cursor reset to its start.
    fn open(&mut self) {
        match self.frames.last() {
            None => {
                self.frames.push(Frame { hi: self.trie.tuples.len(), cur: 0 });
            }
            Some(&Frame { cur: plo, hi: phi }) => {
                let pcol = self.frames.len() - 1;
                let val = self.trie.tuples[plo][pcol];
                let end = self.run_end(pcol, plo, phi, val);
                self.frames.push(Frame { hi: end, cur: plo });
            }
        }
    }
    fn up(&mut self) {
        self.frames.pop();
    }
}

/// Leapfrog intersection of the participating iterators at one level.
struct Leapfrog {
    order: Vec<usize>, // participant indices, kept in cyclic key order
    p: usize,
    ended: bool,
    key: Id,
}

impl Leapfrog {
    fn init(iters: &mut [TrieIter], parts: &[usize]) -> Self {
        let mut lf = Leapfrog { order: parts.to_vec(), p: 0, ended: false, key: 0 };
        if parts.iter().any(|&i| iters[i].at_end()) {
            lf.ended = true;
            return lf;
        }
        lf.order.sort_by_key(|&i| iters[i].key());
        lf.search(iters);
        lf
    }
    fn search(&mut self, iters: &mut [TrieIter]) {
        let k = self.order.len();
        loop {
            let max = iters[self.order[(self.p + k - 1) % k]].key();
            let min = iters[self.order[self.p]].key();
            if min == max {
                self.key = min;
                return;
            }
            iters[self.order[self.p]].seek(max);
            if iters[self.order[self.p]].at_end() {
                self.ended = true;
                return;
            }
            self.p = (self.p + 1) % k;
        }
    }
    fn next(&mut self, iters: &mut [TrieIter]) {
        let k = self.order.len();
        iters[self.order[self.p]].next();
        if iters[self.order[self.p]].at_end() {
            self.ended = true;
            return;
        }
        self.p = (self.p + 1) % k;
        self.search(iters);
    }
}

#[allow(clippy::too_many_arguments)]
fn lftj_recurse(
    iters: &mut [TrieIter],
    parts_at_level: &[Vec<usize>],
    level: usize,
    n_levels: usize,
    current: &mut [Id],
    out: &mut Vec<Row>,
) {
    if level == n_levels {
        out.push(Row::from_slice(current));
        return;
    }
    let parts = &parts_at_level[level];
    // Open-on-entry: descend each relevant iterator into this level (rewinding it).
    for &i in parts {
        iters[i].open();
    }
    let mut lf = Leapfrog::init(iters, parts);
    while !lf.ended {
        // Coarse budget check once per leapfrog key (sticky, so it also unwinds
        // the enclosing recursion levels).
        if budget::exhausted(out.len()) {
            break;
        }
        current[level] = lf.key;
        lftj_recurse(iters, parts_at_level, level + 1, n_levels, current, out);
        lf.next(iters);
    }
    for &i in parts {
        iters[i].up();
    }
}

/// Builds a pattern's trie of projected variable tuples, plus the global levels
/// of those variables (sorted ascending). Repeated-variable patterns keep only
/// rows where all positions of a variable agree.
fn build_trie(
    graph: &Graph,
    id_pat: &IdPattern,
    pos_vars: &[Option<Variable>; 3],
    var_levels: &FxHashMap<Variable, usize>,
) -> (Trie, Vec<usize>) {
    let mut var_positions: Vec<(Variable, Vec<usize>)> = Vec::new();
    for (pos, ov) in pos_vars.iter().enumerate() {
        if let Some(v) = ov {
            if let Some(e) = var_positions.iter_mut().find(|(x, _)| x == v) {
                e.1.push(pos);
            } else {
                var_positions.push((v.clone(), vec![pos]));
            }
        }
    }
    var_positions.sort_by_key(|(v, _)| var_levels[v]);
    let levels: Vec<usize> = var_positions.iter().map(|(v, _)| var_levels[v]).collect();

    let scan = graph.store.scan(id_pat);
    let mut tuples: Vec<Vec<Id>> = Vec::with_capacity(scan.rows.len());
    // zk-trace hook: the WCOJ path's per-pattern input set is the trie's
    // source scan (the consistency-passing rows), recorded pre-projection.
    #[cfg(feature = "zk")]
    let mut zk_matched: Vec<[Id; 3]> = Vec::new();
    for row in scan.rows.iter() {
        let spo = scan.to_spo(row);
        let mut tup = Vec::with_capacity(var_positions.len());
        let mut ok = true;
        for (_, positions) in &var_positions {
            let v0 = spo[positions[0]];
            if positions.iter().any(|&p| spo[p] != v0) {
                ok = false;
                break;
            }
            tup.push(v0);
        }
        if ok {
            #[cfg(feature = "zk")]
            if crate::zk::enabled() {
                zk_matched.push(spo);
            }
            tuples.push(tup);
        }
    }
    #[cfg(feature = "zk")]
    if crate::zk::enabled() {
        crate::zk::record_scan_ids(graph, id_pat, pos_vars, &zk_matched, false);
    }
    tuples.sort_unstable();
    tuples.dedup();
    (Trie { tuples }, levels)
}

fn eval_bgp_wcoj(graph: &Graph, patterns: &[TriplePattern]) -> Result<Bindings, String> {
    // Prepare patterns; an unsatisfiable constant makes the whole BGP empty.
    let mut prepared: Vec<(IdPattern, [Option<Variable>; 3])> = Vec::with_capacity(patterns.len());
    for tp in patterns {
        let (id_pat, pos_vars, unsat) = prepare_pattern(graph, tp)?;
        if unsat {
            // zk-trace: record ONLY this provably-empty pattern (see the
            // binary path) — siblings are not proven empty here.
            #[cfg(feature = "zk")]
            if crate::zk::enabled() {
                crate::zk::record_empty_pattern(crate::zk::key_of_algebra_pattern(tp));
            }
            return Ok(Bindings::unsorted(collect_vars(patterns), vec![]));
        }
        prepared.push((id_pat, pos_vars));
    }

    // Global variable order: most-constrained first (highest degree, then most
    // selective). Constant-only patterns contribute no variables but must match.
    // (Shared with the T22 EXPLAIN dry run.)
    let order_vars = wcoj_global_order(graph, patterns, &prepared);
    let n_levels = order_vars.len();

    let var_levels: FxHashMap<Variable, usize> =
        order_vars.iter().enumerate().map(|(i, v)| (v.clone(), i)).collect();

    // Build a trie per pattern that has variables; check constant-only patterns
    // for existence (empty range => empty BGP).
    let mut tries: Vec<Trie> = Vec::new();
    let mut trie_levels: Vec<Vec<usize>> = Vec::new();
    for (id_pat, pos_vars) in &prepared {
        if pos_vars.iter().all(|v| v.is_none()) {
            if graph.store.estimate(id_pat) == 0 {
                return Ok(Bindings::unsorted(order_vars, vec![]));
            }
            continue;
        }
        let (trie, levels) = build_trie(graph, id_pat, pos_vars, &var_levels);
        if trie.tuples.is_empty() {
            return Ok(Bindings::unsorted(order_vars, vec![]));
        }
        tries.push(trie);
        trie_levels.push(levels);
    }

    // No variables: BGP is a ground check that already succeeded.
    if n_levels == 0 {
        return Ok(Bindings { vars: vec![], rows: vec![Row::new()], sorted_by: None });
    }

    // Participating tries per global level.
    let mut parts_at_level: Vec<Vec<usize>> = vec![Vec::new(); n_levels];
    for (ti, levels) in trie_levels.iter().enumerate() {
        for &lvl in levels {
            parts_at_level[lvl].push(ti);
        }
    }

    let mut iters: Vec<TrieIter> = tries.iter().map(TrieIter::new).collect();
    let mut out: Vec<Row> = Vec::new();
    let mut current = vec![NO_ID; n_levels];
    lftj_recurse(&mut iters, &parts_at_level, 0, n_levels, &mut current, &mut out);

    // Output rows are produced in global-order lexicographic order.
    let sorted_by = order_vars.first().cloned();
    Ok(Bindings { vars: order_vars, rows: out, sorted_by })
}

/// The Leapfrog-Triejoin global variable order: most-constrained variables first
/// (highest pattern degree, ties broken by the smallest estimate among the
/// patterns mentioning the variable). Shared by `eval_bgp_wcoj` and EXPLAIN.
pub(crate) fn wcoj_global_order(
    graph: &Graph,
    patterns: &[TriplePattern],
    prepared: &[(IdPattern, [Option<Variable>; 3])],
) -> Vec<Variable> {
    let degree = |v: &Variable| prepared.iter().filter(|(_, pv)| pv.iter().flatten().any(|x| x == v)).count();
    let min_est = |v: &Variable| {
        prepared
            .iter()
            .filter(|(_, pv)| pv.iter().flatten().any(|x| x == v))
            .map(|(ip, _)| graph.store.estimate(ip))
            .min()
            .unwrap_or(usize::MAX)
    };
    let mut order_vars = collect_vars(patterns);
    order_vars.sort_by(|a, b| degree(b).cmp(&degree(a)).then(min_est(a).cmp(&min_est(b))));
    order_vars
}

/// α-acyclicity test via GYO reduction: repeatedly drop variables that occur in
/// only one pattern and patterns whose variable set is contained in another.
/// A BGP is cyclic iff anything remains. Cyclic BGPs benefit from WCOJ.
fn bgp_is_cyclic(patterns: &[TriplePattern]) -> bool {
    let mut next_id = 0u32;
    let mut ids: FxHashMap<Variable, u32> = FxHashMap::default();
    let mut edges: Vec<std::collections::HashSet<u32>> = Vec::new();
    for tp in patterns {
        let mut e = std::collections::HashSet::new();
        for v in [tp_var(&tp.subject), nnp_var(&tp.predicate), tp_var(&tp.object)].into_iter().flatten() {
            let id = *ids.entry(v).or_insert_with(|| {
                let id = next_id;
                next_id += 1;
                id
            });
            e.insert(id);
        }
        if !e.is_empty() {
            edges.push(e);
        }
    }

    loop {
        let mut changed = false;

        // Drop variables occurring in exactly one edge.
        let mut count: FxHashMap<u32, usize> = FxHashMap::default();
        for e in &edges {
            for &v in e {
                *count.entry(v).or_default() += 1;
            }
        }
        for e in edges.iter_mut() {
            let before = e.len();
            e.retain(|v| count[v] > 1);
            if e.len() != before {
                changed = true;
            }
        }
        let before = edges.len();
        edges.retain(|e| !e.is_empty());
        if edges.len() != before {
            changed = true;
        }

        // Drop an edge contained in a distinct edge (also removes duplicates).
        let mut removed = None;
        'outer: for i in 0..edges.len() {
            for j in 0..edges.len() {
                if i != j && edges[i].is_subset(&edges[j]) {
                    removed = Some(i);
                    break 'outer;
                }
            }
        }
        if let Some(i) = removed {
            edges.remove(i);
            changed = true;
        }

        if !changed {
            break;
        }
    }

    !edges.is_empty()
}

fn prepare_pattern(graph: &Graph, tp: &TriplePattern) -> Result<(IdPattern, [Option<Variable>; 3], bool), String> {
    let mut id_pat: IdPattern = [None, None, None];
    let mut pos_vars: [Option<Variable>; 3] = [None, None, None];
    let mut unsat = false;

    // Resolves one (subject/object) position: variable -> pos_vars; blank node ->
    // synthetic variable; concrete term -> dictionary id (absent term => unsat).
    let mut bind_term = |slot: usize, tp: &TermPattern| -> Result<(), String> {
        match tp {
            TermPattern::Variable(v) => pos_vars[slot] = Some(v.clone()),
            TermPattern::BlankNode(b) => pos_vars[slot] = Some(bnode_var(b)),
            other => match graph.id_of(&term_pattern_to_term(other)?) {
                Some(id) => id_pat[slot] = Some(id),
                None => unsat = true,
            },
        }
        Ok(())
    };
    bind_term(0, &tp.subject)?;
    bind_term(2, &tp.object)?;

    match &tp.predicate {
        NamedNodePattern::Variable(v) => pos_vars[1] = Some(v.clone()),
        NamedNodePattern::NamedNode(n) => match graph.id_of(&Term::NamedNode(n.clone())) {
            Some(id) => id_pat[1] = Some(id),
            None => unsat = true,
        },
    }
    Ok((id_pat, pos_vars, unsat))
}

fn scan_to_bindings(graph: &Graph, id_pat: &IdPattern, pos_vars: &[Option<Variable>; 3], sort_col: Option<usize>, filter: Option<(usize, ScanCmp)>, limit: Option<usize>) -> Bindings {
    let mut vars: Vec<Variable> = Vec::new();
    let mut var_positions: Vec<Vec<usize>> = Vec::new();
    for (pos, v) in pos_vars.iter().enumerate() {
        if let Some(v) = v {
            if let Some(idx) = vars.iter().position(|x| x == v) {
                var_positions[idx].push(pos);
            } else {
                vars.push(v.clone());
                var_positions.push(vec![pos]);
            }
        }
    }
    let scan = match sort_col {
        Some(c) => graph.store.scan_sorted(id_pat, c),
        None => graph.store.scan(id_pat),
    };
    // The TRUE sort column is the first unbound canonical column in the chosen
    // permutation's order — NOT necessarily the requested `sort_col`: with fewer than
    // six permutations the store may not have the requested order, in which case the
    // engine must report the real one so merge joins fall back to hash and range-
    // pruning is skipped (both keyed off the truthful `sorted_by` / `actual_sort`).
    let actual_sort = scan.perm.order().into_iter().find(|&c| id_pat[c].is_none());
    let sorted_by = actual_sort.and_then(|c| pos_vars[c].clone());

    // Range-pruning: when the pushed-down filter is on the scan's ACTUAL sort column and
    // that column holds inline integers (which sort by value), binary-search to the
    // passing value range instead of scanning + filtering the whole relation. Safe
    // only when EVERY value in the column is inline (so no dictionary-encoded
    // numeric in another datatype, scattered below INLINE_BASE, is skipped).
    let mut scan_rows: &[[Id; 3]] = scan.rows.as_ref();
    if let Some((fpos, cmp)) = filter {
        if actual_sort == Some(fpos) && scan_rows.first().is_some_and(|r| dict::is_inline(scan.to_spo(r)[fpos])) {
            scan_rows = match inline_pass_values(cmp) {
                Some((lo, hi)) => {
                    let (lo_id, hi_id) = (dict::INLINE_BASE + lo, dict::INLINE_BASE + hi);
                    let start = scan_rows.partition_point(|r| scan.to_spo(r)[fpos] < lo_id);
                    let end = scan_rows.partition_point(|r| scan.to_spo(r)[fpos] <= hi_id);
                    &scan_rows[start..end]
                }
                None => &[],
            };
        }
    }

    // Per-row builder: apply the pushed-down filter, then project (with the
    // repeated-variable consistency check); `None` drops the row.
    let build_row = |row: &[Id; 3]| -> Option<Row> {
        let spo = scan.to_spo(row);
        if let Some((fpos, cmp)) = filter {
            if !cmp.test_id(graph, spo[fpos]) {
                return None;
            }
        }
        let mut out = Row::with_capacity(vars.len());
        for positions in &var_positions {
            let v0 = spo[positions[0]];
            if positions.iter().any(|&p| spo[p] != v0) {
                return None;
            }
            out.push(v0);
        }
        Some(out)
    };

    // zk-trace hook (feature `zk`, armed recorder only): record the matched
    // triples of this pattern scan — the rows `build_row` keeps, BEFORE
    // projection (the witness needs whole triples, not just variable columns).
    // One `enabled()` check per scan; zero per-row cost when disarmed.
    #[cfg(feature = "zk")]
    if crate::zk::enabled() {
        let kept: Vec<[Id; 3]> = scan_rows
            .iter()
            .filter(|r| build_row(r).is_some())
            .map(|r| scan.to_spo(r))
            .collect();
        crate::zk::record_scan_ids(graph, id_pat, pos_vars, &kept, false);
    }

    // No LIMIT and a large relation: build the rows in parallel (order-preserving).
    #[cfg(feature = "parallel")]
    if limit.is_none() && scan_rows.len() >= PAR_THRESHOLD {
        use rayon::prelude::*;
        let rows: Vec<Row> = scan_rows.par_iter().filter_map(build_row).collect();
        return Bindings { vars, rows, sorted_by };
    }

    // Reserve only up to the LIMIT so a small LIMIT over a huge scan does not
    // allocate for the whole relation (the point of early termination).
    let cap = limit.map_or(scan_rows.len(), |n| n.min(scan_rows.len()));
    let mut rows: Vec<Row> = Vec::with_capacity(budget::cap_alloc(cap));
    for (i, row) in scan_rows.iter().enumerate() {
        // Coarse budget check every 4096 scanned rows.
        if i & 4095 == 0 && budget::exhausted(rows.len()) {
            break;
        }
        if let Some(out) = build_row(row) {
            rows.push(out);
            // LIMIT early-termination: stop scanning once we have enough rows.
            if let Some(n) = limit {
                if rows.len() >= n {
                    break;
                }
            }
        }
    }
    Bindings { vars, rows, sorted_by }
}

fn merge_join(left: Bindings, right: Bindings, jv: &Variable) -> Bindings {
    let lk = left.col(jv).unwrap();
    let rk = right.col(jv).unwrap();
    let mut out_vars = left.vars.clone();
    let mut right_only: Vec<usize> = Vec::new();
    let mut extra_shared: Vec<(usize, usize)> = Vec::new();
    for (ri, v) in right.vars.iter().enumerate() {
        match left.col(v) {
            Some(li) if v != jv => extra_shared.push((li, ri)),
            Some(_) => {}
            None => {
                out_vars.push(v.clone());
                right_only.push(ri);
            }
        }
    }
    let (l, r) = (&left.rows, &right.rows);
    let mut rows = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < l.len() && j < r.len() {
        // Coarse budget check once per key group.
        if budget::exhausted(rows.len()) {
            break;
        }
        let (lv, rv) = (l[i][lk], r[j][rk]);
        match lv.cmp(&rv) {
            Ordering::Less => i += 1,
            Ordering::Greater => j += 1,
            Ordering::Equal => {
                let mut i2 = i;
                while i2 < l.len() && l[i2][lk] == lv {
                    i2 += 1;
                }
                let mut j2 = j;
                while j2 < r.len() && r[j2][rk] == rv {
                    j2 += 1;
                }
                for lrow in l.iter().take(i2).skip(i) {
                    for rrow in r.iter().take(j2).skip(j) {
                        if extra_shared.iter().all(|&(lc, rc)| lrow[lc] == rrow[rc]) {
                            let mut row = lrow.clone();
                            for &rc in &right_only {
                                row.push(rrow[rc]);
                            }
                            rows.push(row);
                        }
                    }
                }
                i = i2;
                j = j2;
            }
        }
    }
    Bindings { vars: out_vars, rows, sorted_by: Some(jv.clone()) }
}

/// Layout for combining two bindings: shared (left col, right col) pairs and the
/// right-only columns appended after left's vars.
fn join_layout(left: &Bindings, right: &Bindings) -> (Vec<Variable>, Vec<(usize, usize)>, Vec<usize>) {
    let mut out_vars = left.vars.clone();
    let mut shared = Vec::new();
    let mut right_only = Vec::new();
    for (ri, v) in right.vars.iter().enumerate() {
        match left.col(v) {
            Some(li) => shared.push((li, ri)),
            None => {
                out_vars.push(v.clone());
                right_only.push(ri);
            }
        }
    }
    (out_vars, shared, right_only)
}

/// SPARQL solution compatibility on the shared columns: an unbound (`NO_ID`)
/// value never conflicts; two bound values must be equal.
fn compatible(lrow: &[Id], rrow: &[Id], shared: &[(usize, usize)]) -> bool {
    shared.iter().all(|&(lc, rc)| {
        let (a, b) = (lrow[lc], rrow[rc]);
        a == NO_ID || b == NO_ID || a == b
    })
}

/// Combines two compatible rows: left's row extended with the right-only columns,
/// filling any shared column that was unbound on the left from the right side.
fn merge_rows(lrow: &[Id], rrow: &[Id], shared: &[(usize, usize)], right_only: &[usize]) -> Row {
    let mut row = Row::from_slice(lrow);
    for &(lc, rc) in shared {
        if row[lc] == NO_ID {
            row[lc] = rrow[rc];
        }
    }
    for &rc in right_only {
        row.push(rrow[rc]);
    }
    row
}

/// Whether any row leaves any of the given columns unbound.
fn any_unbound(rows: &[Row], cols: &[usize]) -> bool {
    rows.iter().any(|r| cols.iter().any(|&c| r[c] == NO_ID))
}

/// Number of radix partitions for the parallel hash-join build. 64 spreads well at high thread
/// counts while keeping the per-partition tag-scan cheap.
const JOIN_PARTS: usize = 64;

/// The partition/lookup hash for a join key — build and probe must agree on it.
fn key_hash(key: &Key) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = rustc_hash::FxHasher::default();
    key.hash(&mut h);
    h.finish()
}

fn hash_join(left: Bindings, right: Bindings) -> Bindings {
    // Build the hash table on the smaller side.
    let (build, probe) = if left.rows.len() <= right.rows.len() {
        (left, right)
    } else {
        (right, left)
    };
    // Shared vars relative to (build, probe).
    let shared: Vec<(usize, usize)> = build
        .vars
        .iter()
        .enumerate()
        .filter_map(|(bi, v)| probe.col(v).map(|pi| (bi, pi)))
        .collect();
    let mut out_vars = build.vars.clone();
    let probe_only: Vec<usize> = probe
        .vars
        .iter()
        .enumerate()
        .filter(|(_, v)| !build.vars.contains(v))
        .map(|(i, v)| {
            out_vars.push(v.clone());
            i
        })
        .collect();
    // Build phase. Above PAR_THRESHOLD the build is radix-partitioned (Tier-1 #5 of
    // research/parallelism-scaling.md): rows are tagged with their key-hash partition in
    // parallel, then each partition builds its private map lock-free. Within a partition rows
    // are scanned in ascending index, so each posting list stays in ascending build-row order —
    // exactly the serial build — and the probe output is byte-identical.
    let build_key = |row: &Row| -> Key { shared.iter().map(|&(bi, _)| row[bi]).collect() };
    #[cfg(feature = "parallel")]
    let tables: Vec<FxHashMap<Key, Posting>> = if build.rows.len() >= PAR_THRESHOLD {
        use rayon::prelude::*;
        let parts: Vec<u8> = build
            .rows
            .par_iter()
            .map(|row| (key_hash(&build_key(row)) % JOIN_PARTS as u64) as u8)
            .collect();
        (0..JOIN_PARTS)
            .into_par_iter()
            .map(|p| {
                let mut t: FxHashMap<Key, Posting> = FxHashMap::default();
                for (ri, row) in build.rows.iter().enumerate() {
                    if parts[ri] as usize == p {
                        t.entry(build_key(row)).or_default().push(ri);
                    }
                }
                t
            })
            .collect()
    } else {
        let mut t: FxHashMap<Key, Posting> = FxHashMap::default();
        for (ri, row) in build.rows.iter().enumerate() {
            t.entry(build_key(row)).or_default().push(ri);
        }
        vec![t]
    };
    #[cfg(not(feature = "parallel"))]
    let tables: Vec<FxHashMap<Key, Posting>> = {
        let mut t: FxHashMap<Key, Posting> = FxHashMap::default();
        for (ri, row) in build.rows.iter().enumerate() {
            t.entry(build_key(row)).or_default().push(ri);
        }
        vec![t]
    };
    // Emit the output rows for one probe row (its matches, each combined with the
    // probe-only columns).
    let emit = |prow: &Row, out: &mut Vec<Row>| {
        let key: Key = shared.iter().map(|&(_, pi)| prow[pi]).collect();
        let table =
            if tables.len() == 1 { &tables[0] } else { &tables[(key_hash(&key) % JOIN_PARTS as u64) as usize] };
        if let Some(matches) = table.get(&key) {
            for &bi in matches {
                let mut combined = build.rows[bi].clone();
                for &pi in &probe_only {
                    combined.push(prow[pi]);
                }
                out.push(combined);
            }
        }
    };
    // The probe is read-only over the (partitioned) table, so for a large probe side build the
    // output in parallel on native.
    #[cfg(feature = "parallel")]
    if probe.rows.len() >= PAR_THRESHOLD {
        use rayon::prelude::*;
        // Budget snapshot for the workers (the installing thread's thread-local is
        // invisible to them): a worker that hits the limits stops adding to its own
        // accumulator; the caller's next on-thread check raises the actual error.
        let limits = budget::snapshot();
        let rows: Vec<Row> = probe
            .rows
            .par_iter()
            .fold(Vec::new, |mut acc, prow| {
                if !limits.hit(acc.len()) {
                    emit(prow, &mut acc);
                }
                acc
            })
            .reduce(Vec::new, |mut a, mut b| {
                a.append(&mut b);
                a
            });
        let _ = budget::exhausted(rows.len()); // sticky gate on the combined size
        return Bindings::unsorted(out_vars, rows);
    }
    let mut rows = Vec::new();
    for prow in &probe.rows {
        // Coarse budget check once per probe row.
        if budget::exhausted(rows.len()) {
            break;
        }
        emit(prow, &mut rows);
    }
    Bindings::unsorted(out_vars, rows)
}

/// Index-nested-loop join of a (small) `result` with a single triple pattern on one
/// shared variable: groups the result by the join value, and for each distinct value
/// looks up the pattern's matches with that variable BOUND (a binary-search range on a
/// permutation index) — so a large, selective pattern is never fully scanned. The
/// pattern must have distinct variables; a pushed-down sargable filter is applied inline.
fn bind_join(
    graph: &Graph,
    result: Bindings,
    id_pat: &IdPattern,
    pos_vars: &[Option<Variable>; 3],
    rk: usize,
    pp: usize,
    filt: Option<(usize, ScanCmp)>,
) -> Bindings {
    // The pattern's NEW variable columns (every variable position except the join one;
    // the only shared variable is the join variable, so the rest are new).
    let new_positions: Vec<usize> = (0..3).filter(|&p| p != pp && pos_vars[p].is_some()).collect();
    let mut out_vars = result.vars.clone();
    for &p in &new_positions {
        out_vars.push(pos_vars[p].clone().unwrap());
    }

    // Group result rows by the join value so each distinct value is looked up once.
    let mut groups: FxHashMap<Id, Vec<usize>> = FxHashMap::default();
    for (ri, row) in result.rows.iter().enumerate() {
        groups.entry(row[rk]).or_default().push(ri);
    }

    let mut out_rows: Vec<Row> = Vec::new();
    // zk-trace hook: accumulate the matched triples across all bound rescans
    // of this pattern (recorded once, under the pattern's ORIGINAL key, so
    // the input set merges with any full scans of the same pattern).
    #[cfg(feature = "zk")]
    let mut zk_matched: Vec<[Id; 3]> = Vec::new();
    for (val, ris) in groups {
        // Coarse budget check once per distinct join value.
        if budget::exhausted(out_rows.len()) {
            break;
        }
        let mut bound = *id_pat;
        bound[pp] = Some(val);
        let scan = graph.store.scan(&bound);
        for prow in scan.rows.iter() {
            let pspo = scan.to_spo(prow);
            if let Some((fpos, cmp)) = filt {
                if !cmp.test_id(graph, pspo[fpos]) {
                    continue;
                }
            }
            #[cfg(feature = "zk")]
            if crate::zk::enabled() {
                zk_matched.push(pspo);
            }
            let new_vals: SmallVec<[Id; 4]> = new_positions.iter().map(|&p| pspo[p]).collect();
            for &ri in &ris {
                let mut combined = result.rows[ri].clone();
                combined.extend(new_vals.iter().copied());
                out_rows.push(combined);
            }
        }
    }
    #[cfg(feature = "zk")]
    if crate::zk::enabled() {
        crate::zk::record_scan_ids(graph, id_pat, pos_vars, &zk_matched, true);
    }
    Bindings::unsorted(out_vars, out_rows)
}

fn cross_product(left: Bindings, right: Bindings) -> Bindings {
    let mut out_vars = left.vars.clone();
    out_vars.extend(right.vars.iter().cloned());
    let mut rows = Vec::with_capacity(budget::cap_alloc(left.rows.len().saturating_mul(right.rows.len())));
    for l in &left.rows {
        // Coarse budget check once per left row.
        if budget::exhausted(rows.len()) {
            break;
        }
        for r in &right.rows {
            let mut row = l.clone();
            row.extend_from_slice(r);
            rows.push(row);
        }
    }
    Bindings::unsorted(out_vars, rows)
}

/// Generic join used for Join of non-conjunctive sub-results. With fully-bound
/// shared columns it takes the fast path (merge if both are sorted on the join
/// var, else hash); when a shared column can be unbound (from OPTIONAL / UNION /
/// VALUES UNDEF), it falls back to a correct solution-compatibility nested loop.
fn join_bindings(left: Bindings, right: Bindings) -> Bindings {
    let (out_vars, shared, right_only) = join_layout(&left, &right);
    if shared.is_empty() {
        return cross_product(left, right);
    }
    let lcols: Vec<usize> = shared.iter().map(|&(lc, _)| lc).collect();
    let rcols: Vec<usize> = shared.iter().map(|&(_, rc)| rc).collect();
    if !any_unbound(&left.rows, &lcols) && !any_unbound(&right.rows, &rcols) {
        if let (Some(lv), Some(rv)) = (&left.sorted_by, &right.sorted_by) {
            if lv == rv && right.col(lv).is_some() {
                let jv = lv.clone();
                return merge_join(left, right, &jv);
            }
        }
        return hash_join(left, right);
    }
    let mut rows = Vec::new();
    for lrow in &left.rows {
        // Coarse budget check once per left row.
        if budget::exhausted(rows.len()) {
            break;
        }
        for rrow in &right.rows {
            if compatible(lrow, rrow, &shared) {
                rows.push(merge_rows(lrow, rrow, &shared, &right_only));
            }
        }
    }
    Bindings::unsorted(out_vars, rows)
}

// ---- OPTIONAL / UNION / MINUS / VALUES / BIND --------------------------------

fn left_outer_join(graph: &Graph, local: &mut LocalVocab, left: Bindings, right: Bindings, expr: Option<&Expression>) -> Result<Bindings, String> {
    let (out_vars, shared, right_only) = join_layout(&left, &right);
    let n_right_only = right_only.len();

    // Sort-merge left outer join for the common case — exactly one shared variable,
    // fully bound on both sides (the typical `?s p ?o OPTIONAL { ?s q ?r }`). It
    // avoids building a hash table over the entire right side (which dominates on
    // large OPTIONALs); QLever uses the same sorted-merge strategy here.
    if shared.len() == 1 {
        let (lk, rk) = shared[0];
        if !any_unbound(&left.rows, &[lk]) && !any_unbound(&right.rows, &[rk]) {
            return left_outer_merge(graph, local, left, right, lk, rk, &shared, &right_only, n_right_only, out_vars, expr);
        }
    }

    // Hash the right side by its shared-variable key when those columns are fully
    // bound; otherwise fall back to a compatibility scan (shared vars may be
    // unbound, in which case they act as wildcards).
    let lcols: Vec<usize> = shared.iter().map(|&(lc, _)| lc).collect();
    let rcols: Vec<usize> = shared.iter().map(|&(_, rc)| rc).collect();
    let table: Option<FxHashMap<Key, Posting>> =
        if !shared.is_empty() && !any_unbound(&left.rows, &lcols) && !any_unbound(&right.rows, &rcols) {
            let mut t: FxHashMap<Key, Posting> = FxHashMap::default();
            for (ri, row) in right.rows.iter().enumerate() {
                t.entry(rcols.iter().map(|&c| row[c]).collect()).or_default().push(ri);
            }
            Some(t)
        } else {
            None
        };

    let mut rows = Vec::new();
    for lrow in &left.rows {
        // Coarse budget check once per left row.
        if budget::exhausted(rows.len()) {
            break;
        }
        // Inline (no heap alloc per left row): an OPTIONAL key usually has 0–1
        // matches. The hashed branch copies the matching indices by value rather
        // than cloning the table's Vec.
        let candidates: SmallVec<[usize; 4]> = match &table {
            Some(t) => {
                let key: Key = lcols.iter().map(|&c| lrow[c]).collect();
                t.get(&key).map(|v| v.iter().copied().collect()).unwrap_or_default()
            }
            None => (0..right.rows.len()).filter(|&ri| compatible(lrow, &right.rows[ri], &shared)).collect(),
        };
        let mut matched = false;
        for ri in candidates {
            let combined = merge_rows(lrow, &right.rows[ri], &shared, &right_only);
            // OPTIONAL's filter is part of the join condition (evaluated on the
            // combined row); a row that fails it does not count as a match.
            let keep = match expr {
                None => true,
                Some(e) => {
                    let tmp = Bindings { vars: out_vars.clone(), rows: vec![], sorted_by: None };
                    effective_boolean(&eval_expr(graph, local, &tmp, &combined, e)?)
                }
            };
            if keep {
                rows.push(combined);
                matched = true;
            }
        }
        if !matched {
            let mut combined = lrow.clone();
            combined.extend(std::iter::repeat_n(NO_ID, n_right_only));
            rows.push(combined);
        }
    }
    Ok(Bindings::unsorted(out_vars, rows))
}

/// Sort-merge left outer join on a single shared variable (`lk`/`rk` columns).
#[allow(clippy::too_many_arguments)]
fn left_outer_merge(
    graph: &Graph,
    local: &mut LocalVocab,
    mut left: Bindings,
    mut right: Bindings,
    lk: usize,
    rk: usize,
    shared: &[(usize, usize)],
    right_only: &[usize],
    n_right_only: usize,
    out_vars: Vec<Variable>,
    expr: Option<&Expression>,
) -> Result<Bindings, String> {
    // Both sides come from index scans that are usually already key-sorted, so
    // these sorts are near-linear (pattern-defeating quicksort detects sortedness).
    left.rows.sort_unstable_by_key(|r| r[lk]);
    right.rows.sort_unstable_by_key(|r| r[rk]);
    let (l, r) = (&left.rows, &right.rows);
    let mut rows: Vec<Row> = Vec::with_capacity(l.len());
    let mut j = 0usize;
    let mut i = 0usize;
    while i < l.len() {
        // Coarse budget check once per key group.
        if budget::exhausted(rows.len()) {
            break;
        }
        let key = l[i][lk];
        let mut i2 = i + 1;
        while i2 < l.len() && l[i2][lk] == key {
            i2 += 1;
        }
        while j < r.len() && r[j][rk] < key {
            j += 1;
        }
        let mut j2 = j;
        while j2 < r.len() && r[j2][rk] == key {
            j2 += 1;
        }
        for lrow in l.iter().take(i2).skip(i) {
            let mut matched = false;
            for rrow in r.iter().take(j2).skip(j) {
                let combined = merge_rows(lrow, rrow, shared, right_only);
                let keep = match expr {
                    None => true,
                    Some(e) => {
                        let tmp = Bindings { vars: out_vars.clone(), rows: vec![], sorted_by: None };
                        effective_boolean(&eval_expr(graph, local, &tmp, &combined, e)?)
                    }
                };
                if keep {
                    rows.push(combined);
                    matched = true;
                }
            }
            if !matched {
                let mut combined = lrow.clone();
                combined.extend(std::iter::repeat_n(NO_ID, n_right_only));
                rows.push(combined);
            }
        }
        i = i2;
        j = j2;
    }
    // Output is ordered by the join variable (at column lk of out_vars).
    let sorted_by = out_vars.get(lk).cloned();
    Ok(Bindings { vars: out_vars, rows, sorted_by })
}

fn union_bindings(left: Bindings, right: Bindings) -> Bindings {
    let mut out_vars = left.vars.clone();
    for v in &right.vars {
        if !out_vars.contains(v) {
            out_vars.push(v.clone());
        }
    }
    let mut rows: Vec<Row> = Vec::with_capacity(left.rows.len() + right.rows.len());
    let map_row = |src_vars: &[Variable], row: &[Id], out_vars: &[Variable]| -> Row {
        out_vars
            .iter()
            .map(|v| src_vars.iter().position(|x| x == v).map(|i| row[i]).unwrap_or(NO_ID))
            .collect()
    };
    for row in &left.rows {
        rows.push(map_row(&left.vars, row, &out_vars));
    }
    for row in &right.rows {
        rows.push(map_row(&right.vars, row, &out_vars));
    }
    Bindings::unsorted(out_vars, rows)
}

fn minus_bindings(left: Bindings, right: Bindings) -> Bindings {
    // SPARQL MINUS: drop a left row iff some right row is *compatible* with it AND
    // their bound domains overlap on at least one shared variable. (Disjoint
    // domains never remove anything.)
    let shared: Vec<(usize, usize)> = left
        .vars
        .iter()
        .enumerate()
        .filter_map(|(li, v)| right.col(v).map(|ri| (li, ri)))
        .collect();
    if shared.is_empty() {
        return left; // disjoint domains -> MINUS removes nothing
    }
    let lcols: Vec<usize> = shared.iter().map(|&(lc, _)| lc).collect();
    let rcols: Vec<usize> = shared.iter().map(|&(_, rc)| rc).collect();

    // Fast path: shared columns fully bound -> compatibility is exact-key equality
    // and the domains always overlap, so membership in a hash set suffices.
    if !any_unbound(&left.rows, &lcols) && !any_unbound(&right.rows, &rcols) {
        let mut table: FxHashMap<Key, ()> = FxHashMap::default();
        for row in &right.rows {
            table.insert(rcols.iter().map(|&c| row[c]).collect(), ());
        }
        let rows: Vec<Row> = left
            .rows
            .into_iter()
            .filter(|row| !table.contains_key(&lcols.iter().map(|&c| row[c]).collect::<Key>()))
            .collect();
        return Bindings { vars: left.vars, rows, sorted_by: left.sorted_by };
    }

    // General path: per-row compatibility with a bound-domain-overlap check.
    let keep = |lrow: &Row| -> bool {
        !right.rows.iter().any(|rrow| {
            let mut overlap = false;
            for &(lc, rc) in &shared {
                let (a, b) = (lrow[lc], rrow[rc]);
                if a != NO_ID && b != NO_ID {
                    if a != b {
                        return false; // incompatible
                    }
                    overlap = true;
                }
            }
            overlap
        })
    };
    let rows: Vec<Row> = left.rows.iter().filter(|r| keep(r)).cloned().collect();
    Bindings { vars: left.vars, rows, sorted_by: left.sorted_by }
}

fn values_bindings(graph: &Graph, local: &mut LocalVocab, variables: &[Variable], bindings: &[Vec<Option<GroundTerm>>]) -> Bindings {
    let rows = bindings
        .iter()
        .map(|row| {
            row.iter()
                .map(|cell| match cell {
                    None => NO_ID,
                    Some(gt) => {
                        // Resolve against the graph dictionary so the id joins with
                        // BGP results; a term absent from the graph gets a local id
                        // (it can only match other VALUES, never the data).
                        let t = ground_to_term(gt);
                        graph.id_of(&t).unwrap_or_else(|| local.intern(t))
                    }
                })
                .collect()
        })
        .collect();
    Bindings::unsorted(variables.to_vec(), rows)
}

fn extend_bindings(graph: &Graph, local: &mut LocalVocab, mut b: Bindings, var: &Variable, expr: &Expression) -> Result<Bindings, String> {
    // BIND was fully serial because each row's computed value was interned immediately. Split it
    // (T1.0b): a PARALLEL pass evaluates the expression (read-only) and resolves the value to an
    // id read-only (inline / graph-dict / already-local); only genuinely new terms fall through to
    // the serial intern below, applied in row order → ids byte-identical to the serial path.
    // (Safe to parallelise: rows reference only ids created by EARLIER operators, never ids
    // created within this BIND loop.)
    // Row identity for BNODE(str)'s per-solution scoping (see ROW_SCOPE).
    let scope = b.rows.as_ptr() as usize;
    #[cfg(feature = "parallel")]
    let resolved: Vec<Result<Id, Term>> = if b.rows.len() >= PAR_THRESHOLD {
        use rayon::prelude::*;
        let lv: &LocalVocab = local;
        let bref = &b;
        // Thread-local extension-function registry and dataset view: snapshot +
        // per-item re-install (free when neither is installed) — see the FILTER
        // branch (the view matters here via EXISTS in the expression).
        let fns = functions::snapshot();
        let vw = view::snapshot();
        b.rows
            .par_iter()
            .enumerate()
            .map(|(i, row)| {
                let _fns = functions::worker_install(&fns);
                let _vw = view::worker_install(&vw);
                ROW_SCOPE.set((scope, i));
                let v = eval_expr(graph, lv, bref, row, expr)?;
                Ok(value_to_id_readonly(graph, lv, &v))
            })
            .collect::<Result<Vec<_>, String>>()?
    } else {
        let mut out = Vec::with_capacity(b.rows.len());
        for (i, row) in b.rows.iter().enumerate() {
            ROW_SCOPE.set((scope, i));
            let v = eval_expr(graph, local, &b, row, expr)?;
            out.push(value_to_id_readonly(graph, local, &v));
        }
        out
    };
    #[cfg(not(feature = "parallel"))]
    let resolved: Vec<Result<Id, Term>> = {
        let mut out = Vec::with_capacity(b.rows.len());
        for (i, row) in b.rows.iter().enumerate() {
            ROW_SCOPE.set((scope, i));
            let v = eval_expr(graph, local, &b, row, expr)?;
            out.push(value_to_id_readonly(graph, local, &v));
        }
        out
    };
    let col: Vec<Id> = resolved
        .into_iter()
        .map(|r| match r {
            Ok(id) => id,
            Err(term) => local.intern(term),
        })
        .collect();
    b.vars.push(var.clone());
    for (row, id) in b.rows.iter_mut().zip(col) {
        row.push(id);
    }
    b.sorted_by = None;
    Ok(b)
}

// ---- Aggregation --------------------------------------------------------------

/// Group row indexes by their group-key, preserving FIRST-SEEN order: `order[i]` is the i-th
/// distinct key in row order and `members[i]` its row indexes (ascending). Above PAR_THRESHOLD the
/// build is radix-partitioned (Tier-1 of research/parallelism-scaling.md): a parallel pass tags
/// each row with its key-hash partition, each partition then builds its private map lock-free
/// (within a partition rows are scanned in ascending index, so a group's first row IS its min),
/// and the global first-seen order is re-imposed by sorting groups on min row index — exactly the
/// serial first-seen order, so output stays byte-identical.
fn build_groups(b: &Bindings, key_cols: &[usize]) -> (Vec<Key>, Vec<Vec<usize>>) {
    #[cfg(feature = "parallel")]
    if b.rows.len() >= PAR_THRESHOLD {
        use rayon::prelude::*;
        use std::hash::{Hash, Hasher};
        const P: usize = 64;
        // Pass 1 (parallel): partition tag per row.
        let parts: Vec<u8> = b
            .rows
            .par_iter()
            .map(|row| {
                let mut h = rustc_hash::FxHasher::default();
                for &c in key_cols {
                    row[c].hash(&mut h);
                }
                (h.finish() % P as u64) as u8
            })
            .collect();
        // Pass 2 (parallel over partitions): private per-partition group builds. Each partition
        // scans the cheap tag vector and only constructs keys for its own rows.
        let per: Vec<Vec<(Key, usize, Vec<usize>)>> = (0..P)
            .into_par_iter()
            .map(|p| {
                let mut idx: FxHashMap<Key, usize> = FxHashMap::default();
                let mut out: Vec<(Key, usize, Vec<usize>)> = Vec::new(); // (key, min_ri, members)
                for (ri, row) in b.rows.iter().enumerate() {
                    if parts[ri] as usize != p {
                        continue;
                    }
                    let key: Key = key_cols.iter().map(|&c| row[c]).collect();
                    match idx.get(&key) {
                        Some(&i) => out[i].2.push(ri),
                        None => {
                            idx.insert(key.clone(), out.len());
                            out.push((key, ri, vec![ri]));
                        }
                    }
                }
                out
            })
            .collect();
        // Pass 3: merge + re-impose the global first-seen order via min row index.
        let mut all: Vec<(Key, usize, Vec<usize>)> = per.into_iter().flatten().collect();
        all.par_sort_unstable_by_key(|&(_, min_ri, _)| min_ri);
        return all.into_iter().map(|(k, _, m)| (k, m)).unzip();
    }
    let mut idx: FxHashMap<Key, usize> = FxHashMap::default();
    let mut order: Vec<Key> = Vec::new();
    let mut members: Vec<Vec<usize>> = Vec::new();
    for (ri, row) in b.rows.iter().enumerate() {
        let key: Key = key_cols.iter().map(|&c| row[c]).collect();
        match idx.get(&key) {
            Some(&i) => members[i].push(ri),
            None => {
                idx.insert(key.clone(), order.len());
                order.push(key);
                members.push(vec![ri]);
            }
        }
    }
    (order, members)
}

fn group_aggregate(
    graph: &Graph,
    local: &mut LocalVocab,
    b: Bindings,
    group_vars: &[Variable],
    aggregates: &[(Variable, AggregateExpression)],
) -> Result<Bindings, String> {
    let key_cols: Vec<usize> = group_vars.iter().map(|v| b.col(v).expect("group var present")).collect();

    // Group rows by the group-key id tuple, preserving first-seen order (parallel ≥ threshold).
    let (mut order, mut members) = build_groups(&b, &key_cols);
    // Whole-dataset aggregate with no GROUP BY: one (empty) group, even if input is empty.
    if group_vars.is_empty() && order.is_empty() {
        order.push(Key::new());
        members.push(Vec::new());
    }

    let mut out_vars: Vec<Variable> = group_vars.to_vec();
    for (v, _) in aggregates {
        out_vars.push(v.clone());
    }

    // Evaluate each group's aggregates (the expensive part — `eval_expr` over every member of
    // every group), then intern the result and build the output row, in first-seen `order`.
    // Interning needs `&mut LocalVocab` and so stays SERIAL and in order, making the ids
    // byte-identical regardless of how evaluation is scheduled.
    let mut rows: Vec<Row> = Vec::with_capacity(order.len());

    // [OPUS-4.8] roborev 1429 (Med): bound peak memory. The previous implementation
    // materialised the aggregate `Value`s (and then their resolved ids) for EVERY group at
    // once before building any row — for many-group / large GROUP_CONCAT queries that holds
    // all aggregate output in memory simultaneously. We now process in bounded BATCHES (and,
    // in the sequential path, one group at a time), interning and pushing each batch's rows
    // before evaluating the next, so only a bounded slice of `Value`s is live.
    #[cfg(feature = "parallel")]
    let parallel_eval = b.rows.len() >= PAR_THRESHOLD;
    #[cfg(not(feature = "parallel"))]
    let parallel_eval = false;

    #[cfg(feature = "parallel")]
    if parallel_eval {
        use rayon::prelude::*;
        // Thread-local extension-function registry and dataset view: snapshot +
        // per-item re-install (free when neither is installed) — see the FILTER branch.
        let fns = functions::snapshot();
        let vw = view::snapshot();
        // Process `members`/`order` in PAR_THRESHOLD-sized batches: evaluate + read-only
        // resolve each batch in parallel, then serially intern only that batch's genuinely
        // new terms and emit its rows. Peak `Value` footprint is one batch, not all groups.
        for (key_chunk, member_chunk) in order.chunks(PAR_THRESHOLD).zip(members.chunks(PAR_THRESHOLD)) {
            let lv: &LocalVocab = local; // immutable reborrow for the read-only parallel phase
            let bref = &b;
            let resolved: Vec<Vec<Result<Id, Term>>> = member_chunk
                .par_iter()
                .map(|members| {
                    let _fns = functions::worker_install(&fns);
                    let _vw = view::worker_install(&vw);
                    aggregates
                        .iter()
                        .map(|(_, agg)| eval_aggregate(graph, lv, bref, members, agg).map(|v| value_to_id_readonly(graph, lv, &v)))
                        .collect::<Result<Vec<_>, String>>()
                })
                .collect::<Result<Vec<_>, String>>()?;
            for (key, res) in key_chunk.iter().zip(resolved) {
                let mut row = Row::from_slice(key);
                for r in res {
                    row.push(match r {
                        Ok(id) => id,
                        Err(term) => local.intern(term),
                    });
                }
                rows.push(row);
            }
        }
    }

    if !parallel_eval {
        // Sequential STREAMING path: eval + intern + push per group, dropping each group's
        // `Value`s before moving to the next, so peak memory is a single group's aggregates.
        for (key, members) in order.iter().zip(&members) {
            let mut row = Row::from_slice(key);
            for (_, agg) in aggregates {
                let v = eval_aggregate(graph, local, &b, members, agg)?;
                let id = value_to_id(graph, local, &v);
                row.push(id);
            }
            rows.push(row);
        }
    }

    Ok(Bindings::unsorted(out_vars, rows))
}

fn eval_aggregate(graph: &Graph, local: &LocalVocab, b: &Bindings, members: &[usize], agg: &AggregateExpression) -> Result<Value, String> {
    match agg {
        AggregateExpression::CountSolutions { distinct } => {
            let n = if *distinct {
                let mut seen = std::collections::HashSet::new();
                members.iter().filter(|&&ri| seen.insert(b.rows[ri].clone())).count()
            } else {
                members.len()
            };
            Ok(Value::Num(Num::Int(n as i64)))
        }
        AggregateExpression::FunctionCall { name, expr, distinct } => {
            // MIN/MAX over an all-temporal column: fold at the ID level through the
            // temporals cache — no term materialised, no per-comparison lexical
            // re-parse. Falls through to the general path on any non-temporal member.
            if let AggregateFunction::Min | AggregateFunction::Max = name {
                let is_min = matches!(name, AggregateFunction::Min);
                if let Some(v) = minmax_temporal(graph, local, b, members, expr, *distinct, is_min) {
                    return Ok(v);
                }
            }
            // Collect the per-member values of `expr`. [OPUS-4.8] An UNBOUND member (a variable
            // with no binding in that row, e.g. an OPTIONAL one) is SKIPPED for every aggregate,
            // matching SPARQL "aggregate over the bound values": SUM/AVG over an OPTIONAL column
            // must still sum the rows that do have a value, not collapse to unbound. Only a
            // genuine expression ERROR (`Value::Error`, e.g. a type error inside `expr`) is fatal
            // for SUM/AVG. A BOUND but non-numeric member is pushed into `vals` and turns SUM/AVG
            // into a type error downstream (`as_numeric` -> None), per agg-err-01.
            let mut vals: Vec<Value> = Vec::with_capacity(members.len());
            let mut errored = false;
            for &ri in members {
                let v = eval_expr(graph, local, b, &b.rows[ri], expr)?;
                match v {
                    Value::Unbound => {}             // skip: aggregate ignores unbound rows
                    Value::Error => errored = true,  // fatal for SUM/AVG (-> unbound aggregate)
                    _ => vals.push(v),
                }
            }
            if *distinct {
                dedup_values(&mut vals);
            }
            match name {
                AggregateFunction::Count => Ok(Value::Num(Num::Int(vals.len() as i64))),
                // SUM/AVG with operand-type promotion: int+int stays integer, decimals
                // stay decimal (exact), floats/doubles promote. Any non-numeric or
                // errored member makes the whole aggregate a type error (-> unbound).
                AggregateFunction::Sum => Ok(sum_values(&vals, errored).map(Num::canonical_term).unwrap_or(Value::Error)),
                AggregateFunction::Avg => {
                    if vals.is_empty() && !errored {
                        return Ok(Value::Num(Num::Int(0))); // AVG({}) = 0 per SPARQL
                    }
                    Ok(sum_values(&vals, errored)
                        .and_then(|s| s.binop(Num::Int(vals.len() as i64), ArithOp::Div))
                        .map(Value::Num)
                        .unwrap_or(Value::Error))
                }
                // MIN/MAX over an all-numeric group return the typed VALUE (promoted,
                // canonically serialised — "2.0E-1"^^xsd:double); mixed groups keep the
                // lenient term path.
                AggregateFunction::Min => Ok(minmax_values(vals, Ordering::Less)),
                AggregateFunction::Max => Ok(minmax_values(vals, Ordering::Greater)),
                AggregateFunction::GroupConcat { separator } => {
                    let sep = separator.clone().unwrap_or_else(|| " ".to_string());
                    let joined = vals.iter().filter_map(value_str).collect::<Vec<_>>().join(&sep);
                    Ok(Value::Term(Term::Literal(Literal::new_simple_literal(joined))))
                }
                AggregateFunction::Sample => Ok(vals.into_iter().next().unwrap_or(Value::Unbound)),
                _ => Err("M2: unsupported aggregate".into()),
            }
        }
    }
}

/// Typed SUM with XPath promotion; `None` (a type error) if any member was non-numeric
/// or errored. The empty sum is `"0"^^xsd:integer`.
fn sum_values(vals: &[Value], errored: bool) -> Option<Num> {
    if errored {
        return None;
    }
    let mut acc = Num::Int(0);
    for v in vals {
        acc = acc.binop(as_numeric(v)?, ArithOp::Add)?;
    }
    Some(acc)
}

/// MIN/MAX: an all-numeric group compares by VALUE (exact for int/decimal) and returns
/// the typed value; any non-numeric member falls back to the lenient total-order term
/// comparison (which must order across types for the SPARQL MIN/MAX-over-anything case).
fn minmax_values(vals: Vec<Value>, keep: Ordering) -> Value {
    if vals.is_empty() {
        return Value::Unbound;
    }
    let nums: Option<Vec<Num>> = vals.iter().map(as_numeric).collect();
    match nums {
        Some(nums) => {
            let mut best = nums[0];
            for &n in &nums[1..] {
                if num_compare(n, best) == Some(keep) {
                    best = n;
                }
            }
            best.canonical_term()
        }
        None => {
            let cmp = |a: &Value, c: &Value| compare_values(a, c).unwrap_or(Ordering::Equal);
            match keep {
                Ordering::Less => vals.into_iter().min_by(cmp).unwrap(),
                _ => vals.into_iter().max_by(cmp).unwrap(),
            }
        }
    }
}

/// MIN/MAX over a variable whose group members are ALL well-formed temporal
/// (dateTime/date) graph terms, folded at the id level through the temporals cache.
/// `None` falls back to the general (materialise + compare_values) path: any unbound
/// member is skipped (as the general path skips it), but a local-vocab or
/// non-temporal member aborts the fast path entirely.
///
/// Tie semantics replicate `minmax_values` exactly: the comparator is
/// `compare_values(..).unwrap_or(Equal)` — same-family temporals by timeline, the
/// indeterminate/cross-family pairs by lexical form — with MIN keeping the FIRST of
/// equal members (`Iterator::min_by`) and MAX the LAST (`Iterator::max_by`); DISTINCT
/// drops later duplicate terms first (same term ⇔ same id for graph terms), which can
/// change which of two equal-VALUED but distinct terms MAX returns, exactly as the
/// general path's `dedup_values` does.
fn minmax_temporal(
    graph: &Graph,
    local: &LocalVocab,
    b: &Bindings,
    members: &[usize],
    expr: &Expression,
    distinct: bool,
    is_min: bool,
) -> Option<Value> {
    let Expression::Variable(v) = expr else { return None };
    let col = b.col(v)?;
    let lex = |id: Id| -> &str {
        match graph.dict.term_parts(id) {
            dict::TermParts::Lit { value, .. } => value,
            _ => "", // unreachable: cached temporal ids are literal dictionary ids
        }
    };
    let mut seen: FxHashSet<Id> = FxHashSet::default();
    let mut best: Option<(Temporal, Id)> = None;
    for &ri in members {
        let id = b.rows[ri][col];
        if id == NO_ID {
            continue; // unbound member: MIN/MAX skips it
        }
        if is_local(id) {
            return None; // computed term: general path
        }
        let t = graph.temporal_value(id)?; // non-temporal/ill-formed: general path
        if distinct && !seen.insert(id) {
            continue;
        }
        best = Some(match best {
            None => (t, id),
            Some((bt, bid)) => {
                let ord = Temporal::cmp_t(t, bt).unwrap_or_else(|| lex(id).cmp(lex(bid)));
                let replace = if is_min { ord == Ordering::Less } else { ord != Ordering::Less };
                if replace {
                    (t, id)
                } else {
                    (bt, bid)
                }
            }
        });
    }
    Some(match best {
        Some((_, id)) => Value::Term(term_of(graph, local, id).expect("aggregate member id resolves")),
        None => Value::Unbound, // no bound members
    })
}

/// Value comparison of two typed numerics: exact when both are int/decimal, f64 otherwise.
fn num_compare(a: Num, c: Num) -> Option<Ordering> {
    if let (Some(x), Some(y)) = (a.to_dec(), c.to_dec()) {
        if let Some(o) = x.cmp(y) {
            return Some(o);
        }
    }
    a.f64().partial_cmp(&c.f64())
}

fn dedup_values(vals: &mut Vec<Value>) {
    let mut seen = std::collections::HashSet::new();
    vals.retain(|v| seen.insert(value_key(v)));
}

fn value_key(v: &Value) -> String {
    match v {
        Value::Term(t) => format!("T{t}"),
        Value::Num(n) => format!("N{}^^{}", n.lexical(), n.datatype().as_str()),
        Value::Bool(b) => format!("B{b}"),
        Value::Unbound => "U".to_string(),
        Value::Error => "E".to_string(),
    }
}

// ---- Modifiers ----------------------------------------------------------------

fn project_bindings(b: Bindings, vars: &[Variable]) -> Bindings {
    let cols: Vec<Option<usize>> = vars.iter().map(|v| b.col(v)).collect();
    let rows = b
        .rows
        .iter()
        .map(|row| cols.iter().map(|c| c.map(|i| row[i]).unwrap_or(NO_ID)).collect())
        .collect();
    let sorted_by = b.sorted_by.filter(|sv| vars.contains(sv));
    Bindings { vars: vars.to_vec(), rows, sorted_by }
}

fn distinct_bindings(b: &mut Bindings) {
    let mut seen = std::collections::HashSet::new();
    b.rows.retain(|r| seen.insert(r.clone()));
}

fn slice_bindings(b: &mut Bindings, start: usize, length: Option<usize>) {
    if start > 0 {
        b.rows.drain(0..start.min(b.rows.len()));
    }
    if let Some(l) = length {
        b.rows.truncate(l);
    }
}

/// One precomputed ORDER BY key cell. A temporal (dateTime/date) graph term keeps only
/// its CACHED comparison value + id — no term materialised, no per-comparison lexical
/// re-parse (the q09-class fix: ORDER BY dateTime was re-parsing both lexicals on every
/// comparison of the sort). Everything else keeps the identity-preserving `Value`.
enum SortCell {
    Temp { t: Temporal, id: Id },
    Val(Value),
}

/// Compares two ORDER BY key cells under the lenient total order, reproducing
/// `compare_values` exactly: same-family temporals by timeline; the indeterminate /
/// cross-family temporal pairs fall back to the lexical-form comparison (both cells are
/// literals — class 3 — and non-numeric, so `compare_values` would reach its
/// `value_str` fallback); a temporal against any other key materialises the term
/// lazily (rare: only mixed-type columns) and defers to `compare_values` itself.
#[inline]
fn cmp_sort_cells(graph: &Graph, local: &LocalVocab, a: &SortCell, c: &SortCell) -> Ordering {
    match (a, c) {
        (SortCell::Temp { t: ta, id: ia }, SortCell::Temp { t: tb, id: ib }) => {
            match Temporal::cmp_t(*ta, *tb) {
                Some(o) => o,
                None => cmp_sort_cells_lex(graph, *ia, *ib),
            }
        }
        (SortCell::Temp { id, .. }, SortCell::Val(v)) => {
            compare_values(&sort_cell_term(graph, local, *id), v).unwrap_or(Ordering::Equal)
        }
        (SortCell::Val(v), SortCell::Temp { id, .. }) => {
            compare_values(v, &sort_cell_term(graph, local, *id)).unwrap_or(Ordering::Equal)
        }
        (SortCell::Val(av), SortCell::Val(cv)) => compare_values(av, cv).unwrap_or(Ordering::Equal),
    }
}

/// The lexical-form fallback for temporal sort cells `compare_values` cannot decide by
/// value (cross-family or the mixed-timezone window) — out of line: the hot comparator
/// stays branch + `partial_cmp`.
#[cold]
fn cmp_sort_cells_lex(graph: &Graph, a: Id, b: Id) -> Ordering {
    let lex = |id: Id| -> &str {
        match graph.dict.term_parts(id) {
            dict::TermParts::Lit { value, .. } => value,
            _ => "", // unreachable: cached temporal ids are literal dictionary ids
        }
    };
    lex(a).cmp(lex(b))
}

/// Materialises a temporal sort cell's term for the (rare) mixed-type-column
/// comparison against a non-temporal key.
#[cold]
fn sort_cell_term(graph: &Graph, local: &LocalVocab, id: Id) -> Value {
    Value::Term(term_of(graph, local, id).expect("sort key id resolves"))
}

fn order_bindings(graph: &Graph, local: &LocalVocab, b: &mut Bindings, exprs: &[OrderExpression]) -> Result<(), String> {
    // The sort key cell for one expression of one row. Numeric keys use the numerics
    // cache and temporal keys the temporals cache (no per-comparison reparse); other
    // expressions fall back to identity-preserving evaluation. The plain-variable case
    // is unpacked here so the column lookup and the cache probes happen exactly once.
    let cell_of = |row: &Row, e: &Expression| -> Result<SortCell, String> {
        if let Expression::Variable(v) = e {
            if let Some(c) = b.col(v) {
                let id = row[c];
                if id != NO_ID && !is_local(id) {
                    if let Some(n) = graph.numeric_value(id) {
                        return Ok(SortCell::Val(Value::Num(Num::Double(n))));
                    }
                    if let Some(t) = graph.temporal_value(id) {
                        return Ok(SortCell::Temp { t, id });
                    }
                    // Neither cache hit: materialise the term, exactly as
                    // `eval_expr(Variable)` would (no second cache probe).
                    return Ok(SortCell::Val(Value::Term(term_of(graph, local, id).expect("bound id resolves"))));
                }
            }
        }
        Ok(match eval_numeric(graph, local, b, row, e) {
            Some(n) => SortCell::Val(Value::Num(Num::Double(n))),
            None => SortCell::Val(eval_expr(graph, local, b, row, e)?),
        })
    };
    // The sort key (vector of (descending, SortCell)) for one row.
    let key_of = |row: &Row| -> Result<Vec<(bool, SortCell)>, String> {
        let mut key = Vec::with_capacity(exprs.len());
        for oe in exprs {
            let (desc, e) = match oe {
                OrderExpression::Asc(e) => (false, e),
                OrderExpression::Desc(e) => (true, e),
            };
            key.push((desc, cell_of(row, e)?));
        }
        Ok(key)
    };
    // Precompute the keys (independent, read-only) — in parallel for large result sets.
    #[cfg(feature = "parallel")]
    let mut keyed: Vec<(Vec<(bool, SortCell)>, Row)> = if b.rows.len() >= PAR_THRESHOLD {
        use rayon::prelude::*;
        b.rows.par_iter().map(|row| Ok((key_of(row)?, row.clone()))).collect::<Result<_, String>>()?
    } else {
        b.rows.iter().map(|row| Ok((key_of(row)?, row.clone()))).collect::<Result<_, String>>()?
    };
    #[cfg(not(feature = "parallel"))]
    let mut keyed: Vec<(Vec<(bool, SortCell)>, Row)> =
        b.rows.iter().map(|row| Ok((key_of(row)?, row.clone()))).collect::<Result<_, String>>()?;

    let cmp = |a: &(Vec<(bool, SortCell)>, Row), c: &(Vec<(bool, SortCell)>, Row)| {
        for ((desc, av), (_, cv)) in a.0.iter().zip(c.0.iter()) {
            let ord = cmp_sort_cells(graph, local, av, cv);
            let ord = if *desc { ord.reverse() } else { ord };
            if ord != Ordering::Equal {
                return ord;
            }
        }
        Ordering::Equal
    };
    #[cfg(feature = "parallel")]
    if keyed.len() >= PAR_THRESHOLD {
        use rayon::prelude::*;
        keyed.par_sort_by(cmp);
    } else {
        keyed.sort_by(cmp);
    }
    #[cfg(not(feature = "parallel"))]
    keyed.sort_by(cmp);

    b.rows = keyed.into_iter().map(|(_, r)| r).collect();
    b.sorted_by = None;
    Ok(())
}

// ---- FILTER + expression evaluation ------------------------------------------

fn apply_filter(graph: &Graph, local: &LocalVocab, b: &mut Bindings, expr: &Expression) -> Result<(), String> {
    // Per-row FILTER evaluation is independent and read-only over the graph/bindings, so a
    // large residual (non-pushed-down) filter is evaluated in parallel on native.
    // Row identity for BNODE(str)'s per-solution scoping (see ROW_SCOPE).
    let scope = b.rows.as_ptr() as usize;
    #[cfg(feature = "parallel")]
    let keep: Vec<bool> = if b.rows.len() >= PAR_THRESHOLD {
        use rayon::prelude::*;
        // The extension-function registry and the dataset view are thread-local:
        // snapshot them here and re-install per worker item (free when neither is
        // installed). The view matters because a FILTER can re-enter pattern
        // evaluation via EXISTS — without the re-install it would silently
        // evaluate UNRESTRICTED on a rayon worker.
        let fns = functions::snapshot();
        let vw = view::snapshot();
        b.rows
            .par_iter()
            .enumerate()
            .map(|(i, row)| {
                let _fns = functions::worker_install(&fns);
                let _vw = view::worker_install(&vw);
                ROW_SCOPE.set((scope, i));
                Ok(effective_boolean(&eval_expr(graph, local, b, row, expr)?))
            })
            .collect::<Result<Vec<bool>, String>>()?
    } else {
        let mut keep = Vec::with_capacity(b.rows.len());
        for (i, row) in b.rows.iter().enumerate() {
            ROW_SCOPE.set((scope, i));
            keep.push(effective_boolean(&eval_expr(graph, local, b, row, expr)?));
        }
        keep
    };
    #[cfg(not(feature = "parallel"))]
    let keep: Vec<bool> = {
        let mut keep = Vec::with_capacity(b.rows.len());
        for (i, row) in b.rows.iter().enumerate() {
            ROW_SCOPE.set((scope, i));
            keep.push(effective_boolean(&eval_expr(graph, local, b, row, expr)?));
        }
        keep
    };
    // zk-trace hook: record the FILTER obligation — expression, in-scope
    // variables, and per-row operand bindings with the verdict (the witness
    // builder needs the operands of every hidden-filter application). The
    // per-row path records OPERAND-TABLE INDICES (one memo probe per cell);
    // terms are materialized once per distinct operand id. Suppressed inside
    // EXISTS (per-row re-evaluation would flood the obligation list; EXISTS
    // is outside the stage-1 fragment).
    #[cfg(feature = "zk")]
    if crate::zk::enabled() && !crate::zk::in_exists() {
        let mut memo = crate::zk::OperandMemo::new();
        let rows: Vec<(Vec<u32>, bool)> = b
            .rows
            .iter()
            .zip(keep.iter())
            .map(|(row, &k)| {
                let cells = (0..b.vars.len())
                    .map(|c| memo.index(row[c], |id| term_of(graph, local, id)))
                    .collect();
                (cells, k)
            })
            .collect();
        crate::zk::record_filter(format!("{expr:?}"), &b.vars, memo.operands, rows);
    }
    let mut i = 0;
    b.rows.retain(|_| {
        let k = keep[i];
        i += 1;
        k
    });
    Ok(())
}

#[derive(Clone, Debug)]
enum Value {
    Bool(bool),
    Num(Num),
    Term(Term),
    Unbound,
    /// A SPARQL type error (e.g. an ordering comparison between incompatible
    /// types). Its effective boolean value is false, it propagates through the
    /// logical operators by the SPARQL 3-valued rules, and a BIND of it is unbound.
    Error,
}

/// A COMPUTED numeric value carrying its XSD type, implementing the SPARQL/XPath
/// operand-type-promotion tower: integer < decimal < float < double. Arithmetic
/// promotes both operands to the greater type; integer and decimal arithmetic is
/// EXACT (i64 / fixed-point [`Dec`]), falling back to double only on overflow.
/// Serialisation (see [`Num::lexical`]) is the XSD canonical form of the type.
#[derive(Clone, Copy, Debug)]
enum Num {
    Int(i64),
    Dec(Dec),
    Float(f32),
    Double(f64),
}

#[derive(Clone, Copy, PartialEq)]
enum ArithOp {
    Add,
    Sub,
    Mul,
    Div,
}

impl Num {
    /// Promotion rank in the XPath numeric tower.
    fn rank(self) -> u8 {
        match self {
            Num::Int(_) => 0,
            Num::Dec(_) => 1,
            Num::Float(_) => 2,
            Num::Double(_) => 3,
        }
    }

    fn f64(self) -> f64 {
        match self {
            Num::Int(i) => i as f64,
            Num::Dec(d) => d.f64(),
            Num::Float(f) => f as f64,
            Num::Double(d) => d,
        }
    }

    fn to_dec(self) -> Option<Dec> {
        match self {
            Num::Int(i) => Some(Dec { mant: i as i128, scale: 0 }),
            Num::Dec(d) => Some(d),
            _ => None,
        }
    }

    /// The typed numeric value of a literal, or `None` if the literal is not a
    /// well-formed numeric (an ill-formed numeric operand is a SPARQL type error).
    fn of_literal(l: &Literal) -> Option<Num> {
        if l.language().is_some() {
            return None;
        }
        let dt = l.datatype();
        let v = l.value().trim();
        if sparq_core::is_integer_datatype(dt.as_str()) {
            if let Ok(i) = v.parse::<i64>() {
                return Some(Num::Int(i));
            }
            // Integer beyond i64: exact i128 mantissa if it fits (scale 0 = integer
            // lexical), else not representable -> double.
            return match Dec::parse(v) {
                Some(d) if d.scale == 0 => Some(Num::Dec(d)),
                Some(_) => None, // "1.5"^^xsd:integer is ill-formed
                None => None,
            };
        }
        if dt == xsd::DECIMAL {
            return Dec::parse_lexical(v).map(Num::Dec);
        }
        if dt == xsd::FLOAT {
            return parse_xsd_f32(v).map(Num::Float);
        }
        if dt == xsd::DOUBLE {
            return parse_xsd_f64(v).map(Num::Double);
        }
        None
    }

    /// `op` under XPath operand promotion. `None` is a SPARQL type error (exact-type
    /// division by zero); exact-arithmetic overflow falls back to double, mirroring
    /// the engine's previous f64 behaviour.
    fn binop(self, o: Num, op: ArithOp) -> Option<Num> {
        let rank = self.rank().max(o.rank());
        if rank == 3 {
            return Some(Num::Double(apply_f64(self.f64(), o.f64(), op)));
        }
        if rank == 2 {
            let (a, b) = (self.f64() as f32, o.f64() as f32);
            return Some(Num::Float(apply_f64(a as f64, b as f64, op) as f32));
        }
        // Exact tier: integer / decimal.
        let (a, b) = (self.to_dec()?, o.to_dec()?);
        if op == ArithOp::Div {
            // xsd:integer / xsd:integer is DECIMAL division per SPARQL; exact-type
            // division by zero is a type error (not INF/NaN).
            if b.mant == 0 {
                return None;
            }
            return match a.checked_div(b) {
                Some(d) => Some(Num::Dec(d)),
                None => Some(Num::Double(self.f64() / o.f64())),
            };
        }
        if rank == 0 {
            // integer op integer -> integer (i64; on overflow fall back to double).
            let (x, y) = (match self { Num::Int(i) => i, _ => unreachable!() }, match o { Num::Int(i) => i, _ => unreachable!() });
            let r = match op {
                ArithOp::Add => x.checked_add(y),
                ArithOp::Sub => x.checked_sub(y),
                ArithOp::Mul => x.checked_mul(y),
                ArithOp::Div => unreachable!(),
            };
            return Some(match r {
                Some(i) => Num::Int(i),
                None => Num::Double(apply_f64(x as f64, y as f64, op)),
            });
        }
        let r = match op {
            ArithOp::Add => a.checked_add(b),
            ArithOp::Sub => a.checked_sub(b),
            ArithOp::Mul => a.checked_mul(b),
            ArithOp::Div => unreachable!(),
        };
        Some(match r {
            Some(d) => Num::Dec(d),
            None => Num::Double(apply_f64(self.f64(), o.f64(), op)),
        })
    }

    fn neg(self) -> Num {
        match self {
            Num::Int(i) => i.checked_neg().map(Num::Int).unwrap_or(Num::Double(-(i as f64))),
            Num::Dec(d) => d.mant.checked_neg().map(|m| Num::Dec(Dec { mant: m, scale: d.scale })).unwrap_or(Num::Double(-d.f64())),
            Num::Float(f) => Num::Float(-f),
            Num::Double(d) => Num::Double(-d),
        }
    }

    fn abs(self) -> Num {
        match self {
            Num::Int(i) => i.checked_abs().map(Num::Int).unwrap_or(Num::Double((i as f64).abs())),
            Num::Dec(d) => d.mant.checked_abs().map(|m| Num::Dec(Dec { mant: m, scale: d.scale })).unwrap_or(Num::Double(d.f64().abs())),
            Num::Float(f) => Num::Float(f.abs()),
            Num::Double(d) => Num::Double(d.abs()),
        }
    }

    fn ceil(self) -> Num {
        match self {
            Num::Int(_) => self,
            Num::Dec(d) => Num::Dec(d.round_to_int(RoundMode::Ceil)),
            Num::Float(f) => Num::Float(f.ceil()),
            Num::Double(d) => Num::Double(d.ceil()),
        }
    }

    fn floor(self) -> Num {
        match self {
            Num::Int(_) => self,
            Num::Dec(d) => Num::Dec(d.round_to_int(RoundMode::Floor)),
            Num::Float(f) => Num::Float(f.floor()),
            Num::Double(d) => Num::Double(d.floor()),
        }
    }

    /// XPath fn:round — round half towards POSITIVE INFINITY (so round(-2.5) = -2),
    /// preserving the argument's datatype.
    fn round(self) -> Num {
        match self {
            Num::Int(_) => self,
            Num::Dec(d) => Num::Dec(d.round_to_int(RoundMode::HalfUp)),
            Num::Float(f) => Num::Float((f + 0.5).floor()),
            Num::Double(d) => Num::Double((d + 0.5).floor()),
        }
    }

    fn datatype(self) -> oxrdf::NamedNodeRef<'static> {
        match self {
            Num::Int(_) => xsd::INTEGER,
            Num::Dec(_) => xsd::DECIMAL,
            Num::Float(_) => xsd::FLOAT,
            Num::Double(_) => xsd::DOUBLE,
        }
    }

    /// XSD CANONICAL lexical form of the value: integers as plain digits; decimals
    /// preserving the arithmetic scale ("3.0" for 1.0+2, "3" for CEIL(2.5)); float /
    /// double in mantissa-E-exponent form with a mandatory fractional digit ("3.21E4",
    /// "2.0E-1") and NaN / INF / -INF spelled per XSD.
    fn lexical(self) -> String {
        match self {
            Num::Int(i) => i.to_string(),
            Num::Dec(d) => d.lexical(),
            // f32 must be formatted as f32 (shortest round-trip); via f64 it would
            // grow spurious digits ("2.0000000298023224E-1" for 0.2f32).
            Num::Float(f) => {
                if f.is_nan() {
                    "NaN".to_string()
                } else if f == f32::INFINITY {
                    "INF".to_string()
                } else if f == f32::NEG_INFINITY {
                    "-INF".to_string()
                } else if f.fract() == 0.0 && f.abs() < 1e15 {
                    format!("{}", f as i64)
                } else {
                    let s = format!("{f:E}");
                    match s.split_once('E') {
                        Some((m, e)) if !m.contains('.') => format!("{m}.0E{e}"),
                        _ => s,
                    }
                }
            }
            Num::Double(d) => fmt_xsd_double(d),
        }
    }

    /// STRICT XSD-canonical lexical: float/double ALWAYS in mantissa-E-exponent form
    /// ("3.21E4", "1.0E2"), never plain. The W3C aggregate expected results use this
    /// for MIN/MAX/SUM, while arithmetic results use the plain-integral convention of
    /// [`Num::lexical`] — the suites were generated by different engines.
    fn canonical_lexical(self) -> String {
        match self {
            Num::Int(_) | Num::Dec(_) => self.lexical(),
            Num::Float(f) => {
                if f.is_nan() || f.is_infinite() {
                    self.lexical()
                } else {
                    let s = format!("{f:E}");
                    match s.split_once('E') {
                        Some((m, e)) if !m.contains('.') => format!("{m}.0E{e}"),
                        _ => s,
                    }
                }
            }
            Num::Double(d) => {
                if d.is_nan() || d.is_infinite() {
                    self.lexical()
                } else {
                    let s = format!("{d:E}");
                    match s.split_once('E') {
                        Some((m, e)) if !m.contains('.') => format!("{m}.0E{e}"),
                        _ => s,
                    }
                }
            }
        }
    }

    /// The value as a TERM in strict canonical form (see [`Num::canonical_lexical`]).
    fn canonical_term(self) -> Value {
        Value::Term(Term::Literal(Literal::new_typed_literal(self.canonical_lexical(), self.datatype())))
    }

    fn is_nan(self) -> bool {
        match self {
            Num::Float(f) => f.is_nan(),
            Num::Double(d) => d.is_nan(),
            _ => false,
        }
    }

    fn is_zero(self) -> bool {
        match self {
            Num::Int(i) => i == 0,
            Num::Dec(d) => d.mant == 0,
            Num::Float(f) => f == 0.0,
            Num::Double(d) => d == 0.0,
        }
    }
}

fn apply_f64(a: f64, b: f64, op: ArithOp) -> f64 {
    match op {
        ArithOp::Add => a + b,
        ArithOp::Sub => a - b,
        ArithOp::Mul => a * b,
        ArithOp::Div => a / b,
    }
}

/// Parse an xsd:float/xsd:double lexical: the XSD spellings of the specials, plus the
/// ordinary scientific notation Rust's parser shares with XSD. `None` = ill-formed.
fn parse_xsd_f64(v: &str) -> Option<f64> {
    match v {
        "NaN" => Some(f64::NAN),
        "INF" | "+INF" => Some(f64::INFINITY),
        "-INF" => Some(f64::NEG_INFINITY),
        // Rust accepts "inf"/"infinity"/"nan" spellings XSD does not; exclude them.
        _ if v.bytes().all(|c| c.is_ascii_digit() || matches!(c, b'+' | b'-' | b'.' | b'e' | b'E')) => v.parse::<f64>().ok(),
        _ => None,
    }
}

fn parse_xsd_f32(v: &str) -> Option<f32> {
    parse_xsd_f64(v).map(|d| d as f32)
}

/// Float/double serialisation: an INTEGRAL value prints as a plain integer ("6",
/// "1050" — matching the dominant convention across the W3C expected results, which
/// mix plain and scientific forms); anything else uses the XSD canonical
/// mantissa-E-exponent form with a mandatory fractional digit ("2.0E-1", "1.5E1").
fn fmt_xsd_double(v: f64) -> String {
    if v.is_nan() {
        return "NaN".to_string();
    }
    if v == f64::INFINITY {
        return "INF".to_string();
    }
    if v == f64::NEG_INFINITY {
        return "-INF".to_string();
    }
    if v.fract() == 0.0 && v.abs() < 1e15 {
        return format!("{}", v as i64);
    }
    let s = format!("{v:E}"); // shortest round-trip mantissa, e.g. "2E-1"
    match s.split_once('E') {
        Some((m, e)) if !m.contains('.') => format!("{m}.0E{e}"),
        _ => s,
    }
}

enum RoundMode {
    Ceil,
    Floor,
    HalfUp,
}

fn effective_boolean(v: &Value) -> bool {
    ebv(v) == Some(true)
}

/// SPARQL effective boolean value, three-valued: `None` is a TYPE ERROR (unbound,
/// non-literal terms, literals of unknown datatypes, ill-formed boolean / numeric
/// lexicals) — it matters because `!error` must stay an error, not become true.
fn ebv(v: &Value) -> Option<bool> {
    match v {
        Value::Bool(b) => Some(*b),
        Value::Num(n) => Some(!n.is_zero() && !n.is_nan()),
        Value::Unbound | Value::Error => None,
        Value::Term(Term::Literal(l)) => {
            if l.language().is_some() {
                // rdf:langString / rdf:dirLangString is NOT xsd:string: its EBV is a
                // type error per SPARQL (1.2 `expression/not-not` pins this down).
                return None;
            }
            let dt = l.datatype().as_str();
            if dt == xsd::BOOLEAN.as_str() {
                match l.value() {
                    "true" | "1" => Some(true),
                    "false" | "0" => Some(false),
                    _ => None,
                }
            } else if is_numeric_dt(l) {
                match l.value().parse::<f64>() {
                    Ok(n) => Some(n != 0.0 && !n.is_nan()),
                    Err(_) => None,
                }
            } else if dt == xsd::STRING.as_str() {
                Some(!l.value().is_empty())
            } else {
                None
            }
        }
        Value::Term(_) => None,
    }
}

fn eval_expr(graph: &Graph, local: &LocalVocab, b: &Bindings, row: &[Id], e: &Expression) -> Result<Value, String> {
    use Expression::*;
    match e {
        Variable(v) => match b.col(v) {
            // Always return the original term so term identity is preserved
            // (sameTerm, BIND passthrough, STR, etc.). The numeric fast path that
            // skips this materialisation lives in `eval_numeric`, used only by the
            // arithmetic / comparison operators where only the value matters.
            Some(c) if row[c] != NO_ID => Ok(Value::Term(term_of(graph, local, row[c]).unwrap())),
            _ => Ok(Value::Unbound),
        },
        NamedNode(n) => Ok(Value::Term(Term::NamedNode(n.clone()))),
        Literal(l) => Ok(Value::Term(Term::Literal(l.clone()))),
        And(a, c) => {
            // SPARQL 3-valued logic, short-circuiting: false dominates, so once the
            // left is false we return false WITHOUT evaluating the right (which may be
            // an error or an unsupported expression that would otherwise abort).
            let x = ebv3(&eval_expr(graph, local, b, row, a)?);
            if x == Some(false) {
                return Ok(Value::Bool(false));
            }
            let y = ebv3(&eval_expr(graph, local, b, row, c)?);
            Ok(and3(x, y))
        }
        Or(a, c) => {
            // SPARQL 3-valued logic, short-circuiting: true dominates.
            let x = ebv3(&eval_expr(graph, local, b, row, a)?);
            if x == Some(true) {
                return Ok(Value::Bool(true));
            }
            let y = ebv3(&eval_expr(graph, local, b, row, c)?);
            Ok(or3(x, y))
        }
        Not(a) => Ok(match ebv3(&eval_expr(graph, local, b, row, a)?) {
            Some(v) => Value::Bool(!v),
            None => Value::Error, // !error = error
        }),
        Equal(a, c) => equal_expr(graph, local, b, row, a, c),
        SameTerm(a, c) => {
            let (x, y) = (eval_expr(graph, local, b, row, a)?, eval_expr(graph, local, b, row, c)?);
            Ok(Value::Bool(matches!((&x, &y), (Value::Term(p), Value::Term(q)) if p == q)))
        }
        Greater(a, c) => cmp_expr(graph, local, b, row, a, c, |o| o == Ordering::Greater),
        GreaterOrEqual(a, c) => cmp_expr(graph, local, b, row, a, c, |o| o != Ordering::Less),
        Less(a, c) => cmp_expr(graph, local, b, row, a, c, |o| o == Ordering::Less),
        LessOrEqual(a, c) => cmp_expr(graph, local, b, row, a, c, |o| o != Ordering::Greater),
        Add(a, c) => arith(graph, local, b, row, a, c, ArithOp::Add),
        Subtract(a, c) => arith(graph, local, b, row, a, c, ArithOp::Sub),
        Multiply(a, c) => arith(graph, local, b, row, a, c, ArithOp::Mul),
        Divide(a, c) => arith(graph, local, b, row, a, c, ArithOp::Div),
        UnaryPlus(a) => eval_expr(graph, local, b, row, a),
        UnaryMinus(a) => {
            // Typed negation: the result keeps the argument's (promoted) numeric
            // datatype; a non-numeric operand is a type error.
            let v = eval_expr(graph, local, b, row, a)?;
            Ok(as_numeric(&v).map(|n| Value::Num(n.neg())).unwrap_or(Value::Error))
        }
        Bound(v) => Ok(Value::Bool(b.col(v).map(|c| row[c] != NO_ID).unwrap_or(false))),
        If(cond, t, f) => {
            // A type error in the condition propagates (it does NOT silently select
            // the else branch).
            match ebv3(&eval_expr(graph, local, b, row, cond)?) {
                Some(true) => eval_expr(graph, local, b, row, t),
                Some(false) => eval_expr(graph, local, b, row, f),
                None => Ok(Value::Error),
            }
        }
        Coalesce(es) => {
            // Returns the first argument that evaluates without error (unbound and
            // type errors are both skipped).
            for e in es {
                let v = eval_expr(graph, local, b, row, e)?;
                if !matches!(v, Value::Unbound | Value::Error) {
                    return Ok(v);
                }
            }
            Ok(Value::Unbound)
        }
        In(a, list) => {
            // `?x IN (..)` is the disjunction of `?x = e` under SPARQL `=` semantics:
            // true on the first match; otherwise a type error if ANY comparison
            // errored (e.g. an unbound operand), else false. Preserving the error
            // matters outside a plain FILTER (BIND, COALESCE).
            let x = eval_expr(graph, local, b, row, a)?;
            let mut errored = false;
            for c in list {
                let y = eval_expr(graph, local, b, row, c)?;
                match values_equal(&x, &y) {
                    Some(true) => return Ok(Value::Bool(true)),
                    Some(false) => {}
                    None => errored = true,
                }
            }
            Ok(if errored { Value::Error } else { Value::Bool(false) })
        }
        FunctionCall(f, args) => eval_function(graph, local, b, row, f, args),
        Exists(inner) => Ok(Value::Bool(eval_exists(graph, local, b, row, inner)?)),
    }
}

/// Correlated `EXISTS { inner }` for one outer solution row: evaluate `inner` and
/// test whether any of its solutions is join-compatible with the row on the
/// variables they share (same term, or unbound on the inner side). Working at the
/// id/term level — rather than substituting the row's terms into the pattern AST —
/// keeps blank-node-valued bindings expressible (spargebra has no ground blank-node
/// term pattern).
///
/// The inner pattern is evaluated against `graph`, which inside `GRAPH <g> { … }`
/// is the active named graph — so an EXISTS nested in a GRAPH pattern sees the same
/// dataset as its surrounding group, per the spec.
///
/// Note: the inner evaluation is re-run per outer row (the expression evaluator is
/// read-only, so there is no per-FILTER place to memoise the inner bindings). Fine
/// for correctness and small/mid results; a shared cache is a follow-up optimisation.
fn eval_exists(graph: &Graph, local: &LocalVocab, b: &Bindings, row: &[Id], inner: &GraphPattern) -> Result<bool, String> {
    // zk-trace: the inner pattern is re-run per outer row; tag its scans
    // `in_exists` and suppress their steps / filter obligations (EXISTS is
    // outside the stage-1 verifiable fragment — sparq-zk::verify rejects it;
    // the tag is for forensics, not proofs).
    #[cfg(feature = "zk")]
    let _zk = crate::zk::exists_scope();
    let mut inner_local = LocalVocab::default();
    let inner_b = eval_graph_pattern(graph, &mut inner_local, inner)?;
    budget::check(inner_b.rows.len())?;
    // Columns shared between the inner solutions and the (bound part of the) outer row.
    let shared: Vec<(usize, usize)> = inner_b
        .vars
        .iter()
        .enumerate()
        .filter_map(|(ic, v)| b.col(v).map(|oc| (oc, ic)))
        .filter(|&(oc, _)| row[oc] != NO_ID)
        .collect();
    Ok(inner_b.rows.iter().any(|irow| {
        shared
            .iter()
            .all(|&(oc, ic)| exists_compatible(graph, local, row[oc], &inner_local, irow[ic]))
    }))
}

/// Join-compatibility of an outer cell with an inner EXISTS cell, where the two rows
/// were produced against different local vocabs: unbound inner is compatible; ids in
/// the shared spaces (graph dictionary / inline integers) compare directly; anything
/// involving a local id falls back to term equality.
fn exists_compatible(graph: &Graph, outer_local: &LocalVocab, o: Id, inner_local: &LocalVocab, i: Id) -> bool {
    if i == NO_ID {
        return true;
    }
    if !is_local(o) && !is_local(i) {
        return o == i;
    }
    term_of(graph, outer_local, o) == term_of(graph, inner_local, i)
}


/// Fast numeric evaluation that never materialises a term: a numeric variable
/// resolves to its value via the dictionary cache, a numeric literal via one
/// parse, and arithmetic recurses. Returns `None` for anything non-numeric, so
/// the caller falls back to the full (identity-preserving) `eval_expr` path.
/// Used only where the value — not the term — matters (comparison / arithmetic).
fn eval_numeric(graph: &Graph, local: &LocalVocab, b: &Bindings, row: &[Id], e: &Expression) -> Option<f64> {
    use Expression::*;
    match e {
        Variable(v) => {
            let c = b.col(v)?;
            let id = row[c];
            if id == NO_ID {
                None
            } else if is_local(id) {
                // Computed values resolve through the local vocab's numeric cache
                // (no term clone, no per-row lexical re-parse).
                local.numeric(id)
            } else {
                graph.numeric_value(id)
            }
        }
        Literal(l) if is_numeric_dt(l) => l.value().parse::<f64>().ok(),
        Add(a, c) => Some(eval_numeric(graph, local, b, row, a)? + eval_numeric(graph, local, b, row, c)?),
        Subtract(a, c) => Some(eval_numeric(graph, local, b, row, a)? - eval_numeric(graph, local, b, row, c)?),
        Multiply(a, c) => Some(eval_numeric(graph, local, b, row, a)? * eval_numeric(graph, local, b, row, c)?),
        Divide(a, c) => Some(eval_numeric(graph, local, b, row, a)? / eval_numeric(graph, local, b, row, c)?),
        UnaryPlus(a) => eval_numeric(graph, local, b, row, a),
        UnaryMinus(a) => Some(-eval_numeric(graph, local, b, row, a)?),
        _ => None,
    }
}

/// Fast temporal (xsd:dateTime / xsd:dateTimeStamp / xsd:date) evaluation that never
/// materialises a term: a variable bound to a graph term resolves through the
/// load-time `temporals` cache (O(1), no lexical re-parse); a constant literal and a
/// BIND-computed (local-vocab) term parse once here. Returns `None` for anything
/// non-temporal or ill-formed, so the caller falls back to the general path (which
/// yields the exact type-error semantics). Used only where the VALUE matters
/// (comparison operators); term identity is never needed there.
fn eval_temporal(graph: &Graph, local: &LocalVocab, b: &Bindings, row: &[Id], e: &Expression) -> Option<Temporal> {
    use Expression::*;
    match e {
        Variable(v) => {
            let c = b.col(v)?;
            let id = row[c];
            if id == NO_ID {
                None
            } else if is_local(id) {
                // Computed values are rare; parse through the local vocab term.
                temporal_of_term(local.term(id))
            } else {
                graph.temporal_value(id)
            }
        }
        Literal(l) => temporal_of_lit(l),
        _ => None,
    }
}

/// The temporal value of a term, if it is a well-formed dateTime/date literal.
fn temporal_of_term(t: &Term) -> Option<Temporal> {
    match t {
        Term::Literal(l) => temporal_of_lit(l),
        _ => None,
    }
}

fn temporal_of_lit(l: &Literal) -> Option<Temporal> {
    if l.language().is_some() {
        return None;
    }
    Temporal::of_lit(l.value(), l.datatype().as_str())
}

/// The lexical form of an expression IF it is an exact-valued numeric operand (an
/// integer subtype or xsd:decimal — NOT float/double). Used to re-check comparisons that
/// the f64 fast path collapsed (integers > 2^53, high-precision decimals). Only reached
/// when the f64 comparison was equal, so the allocation is rare.
fn eval_exact_lexical(graph: &Graph, local: &LocalVocab, b: &Bindings, row: &[Id], e: &Expression) -> Option<String> {
    use Expression::*;
    match e {
        Variable(v) => {
            let id = row[b.col(v)?];
            if id == NO_ID {
                None
            } else if is_local(id) {
                exact_lexical_of_term(local.term(id))
            } else {
                graph.exact_numeric_lexical(id)
            }
        }
        Literal(_) => exact_lexical_of_term(&eval_expr(graph, local, b, row, e).ok().and_then(|v| match v {
            Value::Term(t) => Some(t),
            _ => None,
        })?),
        UnaryPlus(a) => eval_exact_lexical(graph, local, b, row, a),
        UnaryMinus(a) => eval_exact_lexical(graph, local, b, row, a).map(|s| match s.strip_prefix('-') {
            Some(r) => r.to_string(),
            None => format!("-{s}"),
        }),
        _ => None,
    }
}

fn exact_lexical_of_term(t: &Term) -> Option<String> {
    match t {
        Term::Literal(l)
            if l.language().is_none()
                && (sparq_core::is_integer_datatype(l.datatype().as_str()) || l.datatype() == xsd::DECIMAL) =>
        {
            Some(l.value().to_string())
        }
        _ => None,
    }
}

/// Exact comparison of two `xsd:decimal` / integer lexical forms (no f64). `None` if
/// either is not a well-formed decimal. Integers are decimals with an empty fraction.
fn cmp_decimal_str(a: &str, b: &str) -> Option<Ordering> {
    let (na, ia, fa) = split_decimal(a)?;
    let (nb, ib, fb) = split_decimal(b)?;
    let a_zero = ia.is_empty() && fa.is_empty();
    let b_zero = nb_is_zero(ib, fb);
    if a_zero && b_zero {
        return Some(Ordering::Equal);
    }
    let mag = ia
        .len()
        .cmp(&ib.len())
        .then_with(|| ia.cmp(ib))
        .then_with(|| {
            let n = fa.len().max(fb.len());
            // Compare fractional digits with implicit trailing-zero padding.
            (0..n)
                .map(|i| (fa.as_bytes().get(i).copied().unwrap_or(b'0'), fb.as_bytes().get(i).copied().unwrap_or(b'0')))
                .find_map(|(x, y)| (x != y).then(|| x.cmp(&y)))
                .unwrap_or(Ordering::Equal)
        });
    let neg_a = na && !a_zero;
    let neg_b = nb && !b_zero;
    Some(match (neg_a, neg_b) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        (false, false) => mag,
        (true, true) => mag.reverse(),
    })
}

fn nb_is_zero(int: &str, frac: &str) -> bool {
    int.is_empty() && frac.is_empty()
}

/// Splits a decimal lexical into (negative, integer-digits, fraction-digits), normalised
/// (no leading zeros on the integer part, no trailing zeros on the fraction). `None` if
/// the lexical is not digits with at most one `.`.
fn split_decimal(s: &str) -> Option<(bool, &str, &str)> {
    let s = s.trim();
    let (neg, s) = match s.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, s.strip_prefix('+').unwrap_or(s)),
    };
    let (int, frac) = s.split_once('.').unwrap_or((s, ""));
    if (int.is_empty() && frac.is_empty()) || !int.bytes().chain(frac.bytes()).all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some((neg, int.trim_start_matches('0'), frac.trim_end_matches('0')))
}

/// Significant decimal digits in a numeric lexical — used to decide whether the f64
/// sargable path is precision-safe (<= 15 digits round-trips through f64 unambiguously).
fn sig_digits(s: &str) -> usize {
    let (_, int, frac) = match split_decimal(s) {
        Some(p) => p,
        None => return usize::MAX,
    };
    if int.is_empty() {
        // 0.00123 -> significant digits start at the first non-zero fraction digit.
        frac.trim_start_matches('0').len()
    } else {
        int.len() + frac.len()
    }
}

/// An EXACT fixed-point decimal: `mant * 10^-scale`. Used to evaluate `+ - *` on
/// integer / `xsd:decimal` operands without f64 rounding (`0.1 + 0.2` is exactly `0.3`),
/// which the f64 arithmetic path gets wrong. Within `i128` range; overflow → `None` →
/// the caller falls back to f64. Division and `xsd:double`/`float` stay f64.
#[derive(Clone, Copy, Debug)]
struct Dec {
    mant: i128,
    scale: u32,
}

impl Dec {
    /// Parses an integer / decimal lexical (`[+-]?digits(.digits)?`), `None` otherwise.
    fn parse(s: &str) -> Option<Dec> {
        let (neg, int, frac) = split_decimal(s)?;
        let scale = frac.len() as u32;
        let mut mag: i128 = 0;
        for &ch in int.as_bytes().iter().chain(frac.as_bytes()) {
            mag = mag.checked_mul(10)?.checked_add((ch - b'0') as i128)?;
        }
        Some(Dec { mant: if neg { -mag } else { mag }, scale })
    }

    /// Both mantissas scaled to the common (max) scale, or `None` on overflow.
    fn align(self, o: Dec) -> Option<(i128, i128)> {
        let scale = self.scale.max(o.scale);
        let a = self.mant.checked_mul(10i128.checked_pow(scale - self.scale)?)?;
        let b = o.mant.checked_mul(10i128.checked_pow(scale - o.scale)?)?;
        Some((a, b))
    }

    fn checked_add(self, o: Dec) -> Option<Dec> {
        let (a, b) = self.align(o)?;
        Some(Dec { mant: a.checked_add(b)?, scale: self.scale.max(o.scale) })
    }
    fn checked_sub(self, o: Dec) -> Option<Dec> {
        let (a, b) = self.align(o)?;
        Some(Dec { mant: a.checked_sub(b)?, scale: self.scale.max(o.scale) })
    }
    fn checked_mul(self, o: Dec) -> Option<Dec> {
        Some(Dec { mant: self.mant.checked_mul(o.mant)?, scale: self.scale.checked_add(o.scale)? })
    }
    fn cmp(self, o: Dec) -> Option<Ordering> {
        let (a, b) = self.align(o)?;
        Some(a.cmp(&b))
    }

    /// Parses an integer / decimal lexical PRESERVING the written scale ("1.0" keeps
    /// scale 1), unlike [`Dec::parse`] which normalises trailing fraction zeros away.
    /// The scale is what XSD-canonical serialisation of decimal arithmetic preserves
    /// (`1.0 + 2` is `"3.0"`), so typed values must carry it.
    fn parse_lexical(s: &str) -> Option<Dec> {
        let s = s.trim();
        let (neg, s) = match s.strip_prefix('-') {
            Some(r) => (true, r),
            None => (false, s.strip_prefix('+').unwrap_or(s)),
        };
        let (int, frac) = s.split_once('.').unwrap_or((s, ""));
        if (int.is_empty() && frac.is_empty()) || !int.bytes().chain(frac.bytes()).all(|c| c.is_ascii_digit()) {
            return None;
        }
        let mut mag: i128 = 0;
        for &ch in int.as_bytes().iter().chain(frac.as_bytes()) {
            mag = mag.checked_mul(10)?.checked_add((ch - b'0') as i128)?;
        }
        Some(Dec { mant: if neg { -mag } else { mag }, scale: frac.len() as u32 })
    }

    /// EXACT decimal division. The result's scale is the SMALLEST `s >= 1` at which the
    /// quotient terminates (`0 / 2 = "0.0"`, `11.1 / 5 = "2.22"`); a non-terminating
    /// quotient is rounded half-up at scale 18. `None` on overflow (caller falls back
    /// to double); the caller must reject a zero divisor first (type error).
    fn checked_div(self, o: Dec) -> Option<Dec> {
        debug_assert!(o.mant != 0);
        let neg = (self.mant < 0) != (o.mant < 0);
        let n0 = self.mant.unsigned_abs();
        let d = o.mant.unsigned_abs();
        // mant(s) = n0 * 10^(s + o.scale - self.scale) / d
        let num_den = |s: u32| -> Option<(u128, u128)> {
            let e = s as i32 + o.scale as i32 - self.scale as i32;
            if e >= 0 {
                Some((n0.checked_mul(10u128.checked_pow(e as u32)?)?, d))
            } else {
                Some((n0, d.checked_mul(10u128.checked_pow((-e) as u32)?)?))
            }
        };
        const MAX_SCALE: u32 = 18;
        for s in 1..=MAX_SCALE {
            let (num, den) = num_den(s)?;
            if num % den == 0 {
                let mant = i128::try_from(num / den).ok()?;
                return Some(Dec { mant: if neg { -mant } else { mant }, scale: s });
            }
        }
        // Non-terminating: round half-up at the max scale.
        let (num, den) = num_den(MAX_SCALE)?;
        let q = num / den + u128::from(num % den * 2 >= den);
        let mant = i128::try_from(q).ok()?;
        Some(Dec { mant: if neg { -mant } else { mant }, scale: MAX_SCALE })
    }

    /// Rounds to an integer-valued decimal (scale 0), preserving the decimal TYPE
    /// (`CEIL("2.5"^^xsd:decimal)` is `"3"^^xsd:decimal`).
    fn round_to_int(self, mode: RoundMode) -> Dec {
        if self.scale == 0 || self.mant == 0 {
            return Dec { mant: self.mant, scale: 0 };
        }
        // [OPUS-4.8] `10i128.pow(self.scale)` overflows (debug panic / release wrap) for any
        // valid decimal whose scale is >= 39 (10^39 > i128::MAX). When the power exceeds i128
        // the integer part is necessarily 0 (|mant| < i128::MAX < 10^scale), so |value| < 1 and
        // also < 0.5 (2*i128::MAX < 10^39 <= 10^scale), making the rounded result obvious from
        // the sign alone — derive it directly instead of constructing the overflowing power.
        let mant = match 10i128.checked_pow(self.scale) {
            Some(p) => {
                let q = self.mant.div_euclid(p);
                let r = self.mant.rem_euclid(p); // 0..p
                match mode {
                    RoundMode::Floor => q,
                    RoundMode::Ceil => q + i128::from(r > 0),
                    RoundMode::HalfUp => q + i128::from(r * 2 >= p),
                }
            }
            None => match mode {
                // |value| < 1: floor of a tiny positive is 0, of a tiny negative is -1.
                RoundMode::Floor => -i128::from(self.mant < 0),
                // ceil of a tiny positive is 1, of a tiny negative is 0.
                RoundMode::Ceil => i128::from(self.mant > 0),
                // |value| < 0.5 always at this scale, so half-up rounds to 0.
                RoundMode::HalfUp => 0,
            },
        };
        Dec { mant, scale: 0 }
    }

    fn f64(self) -> f64 {
        self.mant as f64 / 10f64.powi(self.scale as i32)
    }

    /// The plain (never exponent) decimal lexical at this value's scale: scale 0 prints
    /// as an integer ("3"); otherwise exactly `scale` fraction digits ("3.0", "0.05").
    fn lexical(self) -> String {
        let mag = self.mant.unsigned_abs().to_string();
        let s = self.scale as usize;
        let mut out = String::with_capacity(mag.len() + s + 2);
        if self.mant < 0 {
            out.push('-');
        }
        if s == 0 {
            out.push_str(&mag);
            return out;
        }
        if mag.len() > s {
            out.push_str(&mag[..mag.len() - s]);
        } else {
            out.push('0');
        }
        out.push('.');
        for _ in mag.len()..s {
            out.push('0');
        }
        if mag.len() > s {
            out.push_str(&mag[mag.len() - s..]);
        } else {
            out.push_str(&mag);
        }
        out
    }
}

/// `true` if the expression performs arithmetic (`+ - *` / unary sign), so a comparison
/// over it must be evaluated EXACTLY rather than via f64 — the only case where f64 can
/// produce a wrong ordering for integer/decimal data (value comparison is monotonic;
/// arithmetic introduces flippable rounding error).
fn expr_has_arith(e: &Expression) -> bool {
    use Expression::*;
    match e {
        Add(..) | Subtract(..) | Multiply(..) => true,
        UnaryPlus(a) | UnaryMinus(a) => expr_has_arith(a),
        _ => false,
    }
}

/// Evaluates an expression EXACTLY as a fixed-point decimal, for integer/decimal operands
/// and `+ - *`. `None` for anything not exactly representable this way (division, doubles,
/// non-numeric operands, overflow) — the caller then uses the f64 path.
fn eval_dec(graph: &Graph, local: &LocalVocab, b: &Bindings, row: &[Id], e: &Expression) -> Option<Dec> {
    use Expression::*;
    match e {
        Variable(v) => {
            let id = row[b.col(v)?];
            if id == NO_ID {
                None
            } else if is_local(id) {
                exact_lexical_of_term(local.term(id)).and_then(|s| Dec::parse(&s))
            } else {
                Dec::parse(&graph.exact_numeric_lexical(id)?)
            }
        }
        Literal(l) if sparq_core::is_integer_datatype(l.datatype().as_str()) || l.datatype() == xsd::DECIMAL => {
            Dec::parse(l.value())
        }
        Add(a, c) => eval_dec(graph, local, b, row, a)?.checked_add(eval_dec(graph, local, b, row, c)?),
        Subtract(a, c) => eval_dec(graph, local, b, row, a)?.checked_sub(eval_dec(graph, local, b, row, c)?),
        Multiply(a, c) => eval_dec(graph, local, b, row, a)?.checked_mul(eval_dec(graph, local, b, row, c)?),
        UnaryPlus(a) => eval_dec(graph, local, b, row, a),
        UnaryMinus(a) => {
            let d = eval_dec(graph, local, b, row, a)?;
            Some(Dec { mant: d.mant.checked_neg()?, scale: d.scale })
        }
        _ => None,
    }
}

/// An ORDERING comparison (`<`, `>`, `<=`, `>=`). SPARQL only orders operands of
/// compatible types (numeric vs numeric, boolean vs boolean, or two literals of the
/// same datatype); anything else is a TYPE ERROR (`Value::Error`), which a FILTER
/// turns into "excluded". (Distinct from the lenient total order `compare_values`
/// used by ORDER BY / MIN / MAX, which must order across every type.)
fn cmp_expr(graph: &Graph, local: &LocalVocab, b: &Bindings, row: &[Id], a: &Expression, c: &Expression, f: impl Fn(Ordering) -> bool) -> Result<Value, String> {
    // EXACT path: integer/decimal arithmetic (`+ - *`) must not round through f64, which
    // can flip an ordering (`0.1 + 0.2` < `0.3` in f64). Only attempted when arithmetic is
    // present (the common, arithmetic-free comparison keeps the f64 fast path below).
    if expr_has_arith(a) || expr_has_arith(c) {
        if let (Some(da), Some(db)) = (eval_dec(graph, local, b, row, a), eval_dec(graph, local, b, row, c)) {
            if let Some(o) = da.cmp(db) {
                return Ok(Value::Bool(f(o)));
            }
        }
    }
    // Fast path: both sides numeric -> compare f64 directly, no term materialised.
    // Reaching here means BOTH operands are numeric, so a `None` partial_cmp is a
    // NaN value (op:numeric ordering of NaN is false) — NOT a cross-type error.
    if let (Some(x), Some(y)) = (eval_numeric(graph, local, b, row, a), eval_numeric(graph, local, b, row, c)) {
        // f64 rounding is monotonic — it only ever COLLAPSES distinct values to equal,
        // never flips an ordering. So re-check exactly ONLY when f64 says equal (catches
        // integers > 2^53 and high-precision decimals that share an f64).
        if x == y {
            if let (Some(la), Some(lb)) = (eval_exact_lexical(graph, local, b, row, a), eval_exact_lexical(graph, local, b, row, c)) {
                if let Some(ord) = cmp_decimal_str(&la, &lb) {
                    return Ok(Value::Bool(f(ord)));
                }
            }
        }
        return Ok(Value::Bool(x.partial_cmp(&y).map(&f).unwrap_or(false)));
    }
    // Fast path: both sides temporal -> compare cached/parsed timeline values, no term
    // materialised. `None` from `cmp_t` is exactly the strict path's type-error cases
    // (cross-family dateTime vs date, or mixed timezone presence inside the ±14h window).
    if let (Some(ta), Some(tb)) = (eval_temporal(graph, local, b, row, a), eval_temporal(graph, local, b, row, c)) {
        return Ok(match Temporal::cmp_t(ta, tb) {
            Some(o) => Value::Bool(f(o)),
            None => Value::Error,
        });
    }
    let (x, y) = (eval_expr(graph, local, b, row, a)?, eval_expr(graph, local, b, row, c)?);
    Ok(match value_compare_strict(&x, &y) {
        Some(o) => Value::Bool(f(o)),
        None => Value::Error,
    })
}

/// SPARQL `=` (and, negated, `!=`). See [`values_equal`].
fn equal_expr(graph: &Graph, local: &LocalVocab, b: &Bindings, row: &[Id], a: &Expression, c: &Expression) -> Result<Value, String> {
    // EXACT integer/decimal arithmetic equality (see `cmp_expr`) — `0.1 + 0.2 = 0.3`.
    if expr_has_arith(a) || expr_has_arith(c) {
        if let (Some(da), Some(db)) = (eval_dec(graph, local, b, row, a), eval_dec(graph, local, b, row, c)) {
            if let Some(o) = da.cmp(db) {
                return Ok(Value::Bool(o == Ordering::Equal));
            }
        }
    }
    // Fast path: both numeric (NaN == NaN is false, matching op:numeric-equal).
    if let (Some(x), Some(y)) = (eval_numeric(graph, local, b, row, a), eval_numeric(graph, local, b, row, c)) {
        // Re-check exactly when f64 says equal (see `cmp_expr`): distinct integers > 2^53
        // or high-precision decimals can share an f64 and must not be reported equal.
        if x == y {
            if let (Some(la), Some(lb)) = (eval_exact_lexical(graph, local, b, row, a), eval_exact_lexical(graph, local, b, row, c)) {
                if let Some(ord) = cmp_decimal_str(&la, &lb) {
                    return Ok(Value::Bool(ord == Ordering::Equal));
                }
            }
        }
        return Ok(Value::Bool(x == y));
    }
    // Fast path: both temporal. Same-family operands decide by timeline (`None` =
    // the indeterminate mixed-timezone window -> type error); dateTime and date are
    // DISJOINT value spaces -> known different (matching `values_equal`).
    if let (Some(ta), Some(tb)) = (eval_temporal(graph, local, b, row, a), eval_temporal(graph, local, b, row, c)) {
        if ta.kind != tb.kind {
            return Ok(Value::Bool(false));
        }
        return Ok(match Temporal::cmp_t(ta, tb) {
            Some(o) => Value::Bool(o == Ordering::Equal),
            None => Value::Error,
        });
    }
    let (x, y) = (eval_expr(graph, local, b, row, a)?, eval_expr(graph, local, b, row, c)?);
    Ok(match values_equal(&x, &y) {
        Some(eq) => Value::Bool(eq),
        None => Value::Error,
    })
}

/// Datatype family of a literal-valued operand, the basis for OPEN-WORLD `=` and
/// the ordering operators: only operands within a comparable family decide; unknown
/// datatypes and ill-formed lexicals are type errors unless the terms are identical.
enum LitKind<'a> {
    /// A numeric-datatype operand; `None` = ill-formed lexical.
    Num(Option<Num>),
    Str(&'a str),
    /// A boolean-datatype operand; `None` = ill-formed lexical.
    Bool(Option<bool>),
    /// xsd:dateTime / xsd:dateTimeStamp on the timeline; `None` = ill-formed.
    DateTime(Option<Timeline>),
    /// xsd:date on the timeline (midnight); `None` = ill-formed.
    Date(Option<Timeline>),
    /// Another XSD datatype (time, duration, gYear, …): (datatype IRI, lexical).
    OtherXsd(&'a str, &'a str),
    /// Language-tagged: (lowercased tag, value).
    Lang(String, &'a str),
    /// A literal of a NON-XSD (unknown) datatype: open-world, never decidable.
    Unknown,
    NotLiteral,
}

fn lit_kind(v: &Value) -> LitKind<'_> {
    match v {
        Value::Num(n) => LitKind::Num(Some(*n)),
        Value::Bool(b) => LitKind::Bool(Some(*b)),
        Value::Term(Term::Literal(l)) => {
            if let Some(tag) = l.language() {
                return LitKind::Lang(tag.to_ascii_lowercase(), l.value());
            }
            let dt = l.datatype();
            if is_numeric_dt(l) {
                LitKind::Num(Num::of_literal(l))
            } else if dt == xsd::STRING {
                LitKind::Str(l.value())
            } else if dt == xsd::BOOLEAN {
                LitKind::Bool(as_bool_val(v))
            } else if dt == xsd::DATE_TIME || dt == xsd::DATE_TIME_STAMP {
                LitKind::DateTime(Timeline::parse_datetime(l.value()))
            } else if dt == xsd::DATE {
                LitKind::Date(Timeline::parse_date(l.value()))
            } else if dt.as_str().starts_with("http://www.w3.org/2001/XMLSchema#") {
                LitKind::OtherXsd(dt.as_str(), l.value())
            } else {
                LitKind::Unknown
            }
        }
        _ => LitKind::NotLiteral,
    }
}

// `Timeline` (the parsed xsd:date/dateTime value) and its comparison rules live in
// `sparq_core::temporal`, shared with the graph's load-time `temporals` cache.

/// SPARQL `=` (and, negated, `!=`) as a three-valued result: `Some(true/false)` for a
/// decided comparison, `None` for a type error. OPEN-WORLD rules: identical terms are
/// equal regardless of datatype; non-literal terms decide by identity; within a
/// comparable literal family the VALUES decide; a language-tagged literal is KNOWN
/// different from any non-language literal; everything else — unknown datatypes,
/// ill-formed lexicals, cross-family pairs — is a TYPE ERROR (`"a"^^ex:dt != "b"^^ex:other`
/// filters the row out rather than evaluating to true).
fn values_equal(x: &Value, y: &Value) -> Option<bool> {
    if matches!(x, Value::Unbound | Value::Error) || matches!(y, Value::Unbound | Value::Error) {
        return None;
    }
    if let (Value::Term(p), Value::Term(q)) = (x, y) {
        if p == q {
            return Some(true); // sameTerm decides even for unknown datatypes
        }
        // RDF 1.2 triple terms compare componentwise, with VALUE equality on the
        // objects (`<<(:a :b 01)>> = <<(:a :b 1)>>` is true, errors propagate).
        if let (Term::Triple(a), Term::Triple(b)) = (p, q) {
            if a.subject != b.subject || a.predicate != b.predicate {
                return Some(false);
            }
            return values_equal(&Value::Term(a.object.clone()), &Value::Term(b.object.clone()));
        }
        if !matches!(p, Term::Literal(_)) || !matches!(q, Term::Literal(_)) {
            return Some(false); // IRI / bnode: identity decides
        }
    }
    let (ka, kb) = (lit_kind(x), lit_kind(y));
    if matches!(ka, LitKind::NotLiteral) || matches!(kb, LitKind::NotLiteral) {
        return Some(false); // computed literal vs non-literal term
    }
    use LitKind::*;
    match (ka, kb) {
        (Num(Some(a)), Num(Some(b))) => Some(num_compare(a, b) == Some(Ordering::Equal)),
        (Num(_), Num(_)) => None, // ill-formed numeric (and not sameTerm)
        (Str(a), Str(b)) => Some(a == b),
        (Bool(Some(a)), Bool(Some(b))) => Some(a == b),
        (Bool(_), Bool(_)) => None,
        (DateTime(Some(a)), DateTime(Some(b))) => Timeline::cmp_tl(a, b).map(|o| o == Ordering::Equal),
        (Date(Some(a)), Date(Some(b))) => Timeline::cmp_tl(a, b).map(|o| o == Ordering::Equal),
        (DateTime(_), DateTime(_)) | (Date(_), Date(_)) => None,
        // date and dateTime values are disjoint -> known different.
        (DateTime(_), Date(_)) | (Date(_), DateTime(_)) => Some(false),
        // A language-tagged literal equals only a literal with the same (ci) tag.
        (Lang(t1, v1), Lang(t2, v2)) => Some(t1 == t2 && v1 == v2),
        (Lang(..), _) | (_, Lang(..)) => Some(false),
        (OtherXsd(d1, l1), OtherXsd(d2, l2)) if d1 == d2 => Some(l1 == l2),
        // Cross-family, unknown datatypes, unknown XSD pairings: open world -> error.
        _ => None,
    }
}

/// Strict SPARQL value comparison for relational operators: `Some(ordering)` only
/// when the operands are value-comparable (same family per [`lit_kind`]), else
/// `None` (a type error).
fn value_compare_strict(x: &Value, y: &Value) -> Option<Ordering> {
    use LitKind::*;
    match (lit_kind(x), lit_kind(y)) {
        (Num(Some(a)), Num(Some(b))) => num_compare(a, b),
        (Str(a), Str(b)) => Some(a.cmp(b)),
        (Bool(Some(a)), Bool(Some(b))) => Some(a.cmp(&b)),
        (DateTime(Some(a)), DateTime(Some(b))) => Timeline::cmp_tl(a, b),
        (Date(Some(a)), Date(Some(b))) => Timeline::cmp_tl(a, b),
        // Same language tag: compare values (the suites' lenient extension).
        (Lang(t1, v1), Lang(t2, v2)) if t1 == t2 => Some(v1.cmp(v2)),
        // Same other-XSD datatype: lexical order (correct for time, gYear, …).
        (OtherXsd(d1, l1), OtherXsd(d2, l2)) if d1 == d2 => Some(l1.cmp(l2)),
        _ => None,
    }
}

fn as_bool_val(v: &Value) -> Option<bool> {
    match v {
        Value::Bool(b) => Some(*b),
        Value::Term(Term::Literal(l)) if l.datatype() == xsd::BOOLEAN => match l.value() {
            "true" | "1" => Some(true),
            "false" | "0" => Some(false),
            _ => None,
        },
        _ => None,
    }
}

/// Three-valued effective boolean: `None` is a SPARQL error (type error or unbound),
/// used by the logical operators to implement SPARQL's 3-valued `&&` / `||` / `!`.
fn ebv3(v: &Value) -> Option<bool> {
    ebv(v)
}

fn and3(x: Option<bool>, y: Option<bool>) -> Value {
    match (x, y) {
        (Some(false), _) | (_, Some(false)) => Value::Bool(false),
        (Some(true), Some(true)) => Value::Bool(true),
        _ => Value::Error,
    }
}

fn or3(x: Option<bool>, y: Option<bool>) -> Value {
    match (x, y) {
        (Some(true), _) | (_, Some(true)) => Value::Bool(true),
        (Some(false), Some(false)) => Value::Bool(false),
        _ => Value::Error,
    }
}

/// TYPED arithmetic with XPath operand promotion: the result carries the promoted
/// datatype (int+int→int, decimal involved→decimal, …, int/int→decimal) and exact
/// int/decimal value. Used for RESULT CONSTRUCTION (BIND / SELECT expressions /
/// aggregates) — the comparison operators keep their f64 fast path (`eval_numeric`),
/// where only the value matters.
fn arith(graph: &Graph, local: &LocalVocab, b: &Bindings, row: &[Id], a: &Expression, c: &Expression, op: ArithOp) -> Result<Value, String> {
    let (x, y) = (eval_expr(graph, local, b, row, a)?, eval_expr(graph, local, b, row, c)?);
    Ok(match (as_numeric(&x), as_numeric(&y)) {
        (Some(p), Some(q)) => p.binop(q, op).map(Value::Num).unwrap_or(Value::Error),
        _ => Value::Error,
    })
}

/// The TYPED numeric value of an evaluated operand: a computed numeric as-is, a
/// numeric literal parsed per its datatype. `None` for non-numerics AND for
/// ill-formed numeric literals (both are SPARQL type errors in arithmetic).
fn as_numeric(v: &Value) -> Option<Num> {
    match v {
        Value::Num(n) => Some(*n),
        Value::Term(Term::Literal(l)) => Num::of_literal(l),
        _ => None,
    }
}

fn as_num(v: &Value) -> Option<f64> {
    match v {
        Value::Num(n) => Some(n.f64()),
        Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
        Value::Term(Term::Literal(l)) if is_numeric_dt(l) => l.value().parse::<f64>().ok(),
        _ => None,
    }
}

fn is_numeric_dt(l: &Literal) -> bool {
    let dt = l.datatype().as_str();
    sparq_core::is_integer_datatype(dt)
        || dt == xsd::DECIMAL.as_str()
        || dt == xsd::DOUBLE.as_str()
        || dt == xsd::FLOAT.as_str()
}

/// LENIENT total order for ORDER BY (and the MIN/MAX fallback): SPARQL orders
/// unbound < blank nodes < IRIs < literals, then by value within each class
/// (numerics by value; everything else by string form, which keeps the order
/// deterministic across mixed literal types).
fn compare_values(x: &Value, y: &Value) -> Option<Ordering> {
    fn class(v: &Value) -> u8 {
        match v {
            Value::Unbound | Value::Error => 0,
            Value::Term(Term::BlankNode(_)) => 1,
            Value::Term(Term::NamedNode(_)) => 2,
            // SPARQL 1.2 total-order extension: triple terms sort AFTER literals.
            Value::Term(Term::Triple(_)) => 4,
            _ => 3, // literals, incl. computed numerics / booleans
        }
    }
    let (ca, cb) = (class(x), class(y));
    if ca != cb {
        return Some(ca.cmp(&cb));
    }
    match ca {
        0 => Some(Ordering::Equal),
        1 | 2 => Some(value_str(x)?.cmp(&value_str(y)?)),
        // Triple terms order componentwise (subject, predicate, then object under
        // this same total order, recursing through nesting).
        4 => {
            let (Value::Term(Term::Triple(a)), Value::Term(Term::Triple(b))) = (x, y) else {
                return None;
            };
            let nob = |n: &NamedOrBlankNode| {
                Value::Term(match n {
                    NamedOrBlankNode::NamedNode(n) => Term::NamedNode(n.clone()),
                    NamedOrBlankNode::BlankNode(b) => Term::BlankNode(b.clone()),
                })
            };
            let s = compare_values(&nob(&a.subject), &nob(&b.subject))?;
            if s != Ordering::Equal {
                return Some(s);
            }
            let p = a.predicate.as_str().cmp(b.predicate.as_str());
            if p != Ordering::Equal {
                return Some(p);
            }
            compare_values(&Value::Term(a.object.clone()), &Value::Term(b.object.clone()))
        }
        _ => {
            if let (Some(a), Some(b)) = (as_num(x), as_num(y)) {
                return a.partial_cmp(&b);
            }
            // dateTime/date order by timeline when comparable.
            if let Some(o) = value_compare_strict(x, y) {
                return Some(o);
            }
            Some(value_str(x)?.cmp(&value_str(y)?))
        }
    }
}

/// SPARQL built-in function calls (`STR`, `LANG`, `CONCAT`, `SUBSTR`, type tests, numeric, …).
/// Unsupported functions (hashes, dateTime, REGEX, BNODE/RAND/UUID) return a clear error rather
/// than a silent wrong answer. SPARQL type errors map to `Value::Error` (EBV false; unbound on BIND).
fn eval_function(
    graph: &Graph,
    local: &LocalVocab,
    b: &Bindings,
    row: &[Id],
    f: &spargebra::algebra::Function,
    args: &[Expression],
) -> Result<Value, String> {
    use spargebra::algebra::Function as F;
    let ev = |i: usize| eval_expr(graph, local, b, row, &args[i]);
    let simple = |s: String| Value::Term(Term::Literal(Literal::new_simple_literal(s)));
    // Both operands as ARGUMENT-COMPATIBLE string literals (second simple/xsd:string,
    // or same language tag as the first), else `Value::Error`.
    let str_compat2 = |a: &Value, c: &Value, g: &dyn Fn(&str, &str) -> bool| match (str_lit(a), str_lit(c)) {
        (Some((x, lx)), Some((y, ly))) if ly.is_none() || ly == lx => Value::Bool(g(&x, &y)),
        _ => Value::Error,
    };
    Ok(match f {
        F::Str => value_str(&ev(0)?).map(simple).unwrap_or(Value::Error),
        F::StrLen => value_str(&ev(0)?).map(|s| Value::Num(Num::Int(s.chars().count() as i64))).unwrap_or(Value::Error),
        // UCASE/LCASE/SUBSTR operate on string literals and preserve the language tag
        // of the argument (simple in, simple out; "bar"@en in, "BAR"@en out).
        F::UCase => match str_lit(&ev(0)?) {
            Some((s, lang)) => lit_with_lang(s.to_uppercase(), lang.as_deref()),
            None => Value::Error,
        },
        F::LCase => match str_lit(&ev(0)?) {
            Some((s, lang)) => lit_with_lang(s.to_lowercase(), lang.as_deref()),
            None => Value::Error,
        },
        F::Lang => match ev(0)? {
            Value::Term(Term::Literal(l)) => simple(l.language().unwrap_or("").to_string()),
            Value::Num(_) | Value::Bool(_) => simple(String::new()),
            _ => Value::Error,
        },
        F::Datatype => match ev(0)? {
            Value::Term(Term::Literal(l)) => Value::Term(Term::NamedNode(l.datatype().into_owned())),
            // A computed numeric knows its promoted XSD type (the type-promotion suite
            // checks `datatype(?l + ?r)` across the whole tower).
            Value::Num(n) => Value::Term(Term::NamedNode(n.datatype().into_owned())),
            Value::Bool(_) => Value::Term(Term::NamedNode(xsd::BOOLEAN.into_owned())),
            _ => Value::Error,
        },
        // CONCAT: all operands must be string literals. The result carries a language
        // tag only when every operand has the SAME tag; otherwise it is simple.
        F::Concat => {
            let mut s = String::new();
            let mut lang: Option<Option<String>> = None; // common-tag accumulator
            for i in 0..args.len() {
                match str_lit(&ev(i)?) {
                    Some((p, l)) => {
                        s.push_str(&p);
                        lang = Some(match lang {
                            None => l,
                            Some(prev) if prev == l => prev,
                            Some(_) => None,
                        });
                    }
                    None => return Ok(Value::Error),
                }
            }
            lit_with_lang(s, lang.flatten().as_deref())
        }
        F::Contains => str_compat2(&ev(0)?, &ev(1)?, &|a, c| a.contains(c)),
        F::StrStarts => str_compat2(&ev(0)?, &ev(1)?, &|a, c| a.starts_with(c)),
        F::StrEnds => str_compat2(&ev(0)?, &ev(1)?, &|a, c| a.ends_with(c)),
        // STRBEFORE/STRAFTER: arguments must be compatible (second simple/xsd:string,
        // or same language tag). On a match the result carries the FIRST argument's
        // language tag; no match gives the empty simple literal.
        F::StrBefore => match (str_lit(&ev(0)?), str_lit(&ev(1)?)) {
            (Some((a, la)), Some((c, lc))) if lc.is_none() || lc == la => match a.find(&c) {
                Some(i) => lit_with_lang(a[..i].to_string(), la.as_deref()),
                None => simple(String::new()),
            },
            _ => Value::Error,
        },
        F::StrAfter => match (str_lit(&ev(0)?), str_lit(&ev(1)?)) {
            (Some((a, la)), Some((c, lc))) if lc.is_none() || lc == la => match a.find(&c) {
                Some(i) => lit_with_lang(a[i + c.len()..].to_string(), la.as_deref()),
                None => simple(String::new()),
            },
            _ => Value::Error,
        },
        F::SubStr => {
            let (s, lang) = match str_lit(&ev(0)?) {
                Some(x) => x,
                None => return Ok(Value::Error),
            };
            let start = match as_num(&ev(1)?) {
                Some(n) => n as i64,
                None => return Ok(Value::Error),
            };
            let chars: Vec<char> = s.chars().collect();
            let from = (start.max(1) - 1) as usize; // SPARQL SUBSTR is 1-indexed by codepoint
            let out: String = if args.len() >= 3 {
                let len = match as_num(&ev(2)?) {
                    Some(n) => n.max(0.0) as usize,
                    None => return Ok(Value::Error),
                };
                chars.iter().skip(from).take(len).collect()
            } else {
                chars.iter().skip(from).collect()
            };
            lit_with_lang(out, lang.as_deref())
        }
        F::EncodeForUri => value_str(&ev(0)?).map(|s| simple(encode_for_uri(&s))).unwrap_or(Value::Error),
        F::Iri => match ev(0)? {
            // An IRI argument passes through unchanged.
            Value::Term(Term::NamedNode(n)) => Value::Term(Term::NamedNode(n)),
            v => match str_lit(&v) {
                // String literal: absolute IRIs pass; relative ones resolve against BASE.
                Some((s, None)) => resolve_iri(&s).map(|n| Value::Term(Term::NamedNode(n))).unwrap_or(Value::Error),
                _ => Value::Error,
            },
        },
        F::IsIri => Value::Bool(matches!(ev(0)?, Value::Term(Term::NamedNode(_)))),
        F::IsBlank => Value::Bool(matches!(ev(0)?, Value::Term(Term::BlankNode(_)))),
        F::IsLiteral => Value::Bool(matches!(ev(0)?, Value::Term(Term::Literal(_)) | Value::Num(_) | Value::Bool(_))),
        F::IsNumeric => Value::Bool(match ev(0)? {
            Value::Num(_) => true,
            Value::Term(Term::Literal(l)) => is_numeric_dt(&l),
            _ => false,
        }),
        // ABS/CEIL/FLOOR/ROUND preserve the argument's numeric DATATYPE
        // (CEIL("2.5"^^xsd:decimal) is "3"^^xsd:decimal, not xsd:integer).
        F::Abs => as_numeric(&ev(0)?).map(|n| Value::Num(n.abs())).unwrap_or(Value::Error),
        F::Ceil => as_numeric(&ev(0)?).map(|n| Value::Num(n.ceil())).unwrap_or(Value::Error),
        F::Floor => as_numeric(&ev(0)?).map(|n| Value::Num(n.floor())).unwrap_or(Value::Error),
        F::Round => as_numeric(&ev(0)?).map(|n| Value::Num(n.round())).unwrap_or(Value::Error),
        // STRDT(lexical, datatypeIRI) -> typed literal. The first argument must be a
        // SIMPLE literal (= xsd:string in RDF 1.1) — lang-tagged / typed input errors.
        F::StrDt => match (str_lit(&ev(0)?), ev(1)?) {
            (Some((lex, None)), Value::Term(Term::NamedNode(dt))) => {
                Value::Term(Term::Literal(Literal::new_typed_literal(lex, dt)))
            }
            _ => Value::Error,
        },
        // STRLANG(lexical, langTag) -> language-tagged literal; both arguments must be
        // simple literals.
        F::StrLang => match (str_lit(&ev(0)?), str_lit(&ev(1)?)) {
            (Some((lex, None)), Some((lang, None))) => match Literal::new_language_tagged_literal(lex, lang) {
                Ok(l) => Value::Term(Term::Literal(l)),
                Err(_) => Value::Error,
            },
            _ => Value::Error,
        },
        // LANGMATCHES(tag, range) — RFC 4647 basic filtering (`*` matches any non-empty tag).
        F::LangMatches => match (value_str(&ev(0)?), value_str(&ev(1)?)) {
            (Some(tag), Some(range)) => {
                let (tag, range) = (tag.to_ascii_lowercase(), range.to_ascii_lowercase());
                let m = if range == "*" {
                    !tag.is_empty()
                } else {
                    tag == range || tag.starts_with(&format!("{range}-"))
                };
                Value::Bool(m)
            }
            _ => Value::Error,
        },
        // Hash builtins: operand must be a simple literal / xsd:string; lowercase hex out.
        #[cfg(feature = "digest")]
        F::Md5 => digest_hex::<md5::Md5>(&ev(0)?),
        #[cfg(feature = "digest")]
        F::Sha1 => digest_hex::<sha1::Sha1>(&ev(0)?),
        #[cfg(feature = "digest")]
        F::Sha256 => digest_hex::<sha2::Sha256>(&ev(0)?),
        #[cfg(feature = "digest")]
        F::Sha384 => digest_hex::<sha2::Sha384>(&ev(0)?),
        #[cfg(feature = "digest")]
        F::Sha512 => digest_hex::<sha2::Sha512>(&ev(0)?),
        // TZ(xsd:dateTime) -> the timezone part of the lexical form as a simple
        // literal ("Z", "±hh:mm", or "" when absent).
        F::Tz => datetime_arg_tz(&ev(0)?).map(simple).unwrap_or(Value::Error),
        // TIMEZONE(xsd:dateTime) -> xsd:dayTimeDuration; no timezone is a type error.
        F::Timezone => match datetime_arg_tz(&ev(0)?).as_deref().and_then(tz_to_duration) {
            Some(d) => Value::Term(Term::Literal(Literal::new_typed_literal(d, xsd::DAY_TIME_DURATION))),
            None => Value::Error,
        },
        F::BNode => {
            if args.is_empty() {
                // BNODE(): a fresh blank node per call.
                static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
                let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                Value::Term(Term::BlankNode(BlankNode::new_unchecked(format!("fnb{n}"))))
            } else {
                // BNODE(str): the label is derived from (solution-row scope, argument),
                // so equal arguments within ONE solution map to the same blank node and
                // everything else stays distinct (see ROW_SCOPE).
                match string_literal(&ev(0)?) {
                    Some(s) => {
                        let (scope, idx) = ROW_SCOPE.get();
                        Value::Term(Term::BlankNode(BlankNode::new_unchecked(format!(
                            "fnb{scope:x}r{idx}h{:016x}",
                            fx64(&s)
                        ))))
                    }
                    None => Value::Error,
                }
            }
        }
        // NOW(): the current UTC instant as xsd:dateTime. RAND(): xsd:double in [0, 1)
        // (sourced from the same OS RNG as UUID, avoiding a new wasm-hostile dep) —
        // both native-only for the same reason as UUID()/STRUUID().
        #[cfg(not(target_arch = "wasm32"))]
        F::Now => Value::Term(Term::Literal(Literal::new_typed_literal(now_lexical(), xsd::DATE_TIME))),
        #[cfg(not(target_arch = "wasm32"))]
        F::Rand => {
            let bytes = *uuid::Uuid::new_v4().as_bytes();
            let x = u64::from_le_bytes(bytes[..8].try_into().expect("8 bytes"));
            Value::Num(Num::Double((x >> 11) as f64 / (1u64 << 53) as f64))
        }
        #[cfg(not(target_arch = "wasm32"))]
        F::Uuid => Value::Term(Term::NamedNode(oxrdf::NamedNode::new_unchecked(format!(
            "urn:uuid:{}",
            uuid::Uuid::new_v4()
        )))),
        #[cfg(not(target_arch = "wasm32"))]
        F::StrUuid => simple(uuid::Uuid::new_v4().to_string()),
        // xsd:dateTime accessors — parse the lexical form and return the numeric component.
        F::Year => datetime_field(&ev(0)?, 0),
        F::Month => datetime_field(&ev(0)?, 1),
        F::Day => datetime_field(&ev(0)?, 2),
        F::Hours => datetime_field(&ev(0)?, 3),
        F::Minutes => datetime_field(&ev(0)?, 4),
        F::Seconds => datetime_field(&ev(0)?, 5),
        // REGEX/REPLACE: the text operand must be a string literal (an IRI or non-string
        // literal is a type error); REPLACE's result keeps the text's language tag.
        #[cfg(feature = "regex")]
        F::Regex => {
            let (text, pat) = match (str_lit(&ev(0)?), value_str(&ev(1)?)) {
                (Some((t, _)), Some(p)) => (t, p),
                _ => return Ok(Value::Error),
            };
            let flags = if args.len() >= 3 { value_str(&ev(2)?).unwrap_or_default() } else { String::new() };
            match build_regex(&pat, &flags) {
                Some(re) => Value::Bool(re.is_match(&text)),
                None => Value::Error,
            }
        }
        #[cfg(feature = "regex")]
        F::Replace => {
            let (text, lang, pat, rep) = match (str_lit(&ev(0)?), value_str(&ev(1)?), value_str(&ev(2)?)) {
                (Some((t, lang)), Some(p), Some(r)) => (t, lang, p, r),
                _ => return Ok(Value::Error),
            };
            let flags = if args.len() >= 4 { value_str(&ev(3)?).unwrap_or_default() } else { String::new() };
            match build_regex(&pat, &flags) {
                Some(re) => lit_with_lang(re.replace_all(&text, rep.as_str()).into_owned(), lang.as_deref()),
                None => Value::Error,
            }
        }
        // ---- SPARQL 1.2 triple-term builtins ------------------------------------
        // TRIPLE(s, p, o): s must be an IRI / blank node, p an IRI, o any RDF term.
        F::Triple => {
            let subject = match ev(0)? {
                Value::Term(Term::NamedNode(n)) => NamedOrBlankNode::NamedNode(n),
                Value::Term(Term::BlankNode(b)) => NamedOrBlankNode::BlankNode(b),
                _ => return Ok(Value::Error),
            };
            let predicate = match ev(1)? {
                Value::Term(Term::NamedNode(n)) => n,
                _ => return Ok(Value::Error),
            };
            let Some(object) = value_as_term(&ev(2)?) else {
                return Ok(Value::Error);
            };
            Value::Term(Term::Triple(Box::new(oxrdf::Triple::new(subject, predicate, object))))
        }
        F::IsTriple => match ev(0)? {
            Value::Term(Term::Triple(_)) => Value::Bool(true),
            Value::Unbound | Value::Error => Value::Error,
            _ => Value::Bool(false),
        },
        F::Subject => match ev(0)? {
            Value::Term(Term::Triple(t)) => Value::Term(match t.subject {
                NamedOrBlankNode::NamedNode(n) => Term::NamedNode(n),
                NamedOrBlankNode::BlankNode(b) => Term::BlankNode(b),
            }),
            _ => Value::Error,
        },
        F::Predicate => match ev(0)? {
            Value::Term(Term::Triple(t)) => Value::Term(Term::NamedNode(t.predicate)),
            _ => Value::Error,
        },
        F::Object => match ev(0)? {
            Value::Term(Term::Triple(t)) => Value::Term(t.object),
            _ => Value::Error,
        },
        // ---- SPARQL 1.2 language / base-direction builtins -----------------------
        // hasLANG / hasLANGDIR: a boolean property of any RDF TERM (an IRI is simply
        // `false`); only unbound/error operands propagate the error.
        F::HasLang => match ev(0)? {
            Value::Term(Term::Literal(l)) => Value::Bool(l.language().is_some()),
            Value::Unbound | Value::Error => Value::Error,
            _ => Value::Bool(false),
        },
        F::HasLangDir => match ev(0)? {
            Value::Term(Term::Literal(l)) => Value::Bool(l.direction().is_some()),
            Value::Unbound | Value::Error => Value::Error,
            _ => Value::Bool(false),
        },
        // LANGDIR mirrors LANG: "" for a literal without a base direction, type error
        // on non-literals.
        F::LangDir => match ev(0)? {
            Value::Term(Term::Literal(l)) => simple(l.direction().map(|d| d.to_string()).unwrap_or_default()),
            Value::Num(_) | Value::Bool(_) => simple(String::new()),
            _ => Value::Error,
        },
        // STRLANGDIR(lexical, langTag, "ltr"|"rtl") -> directional language-tagged
        // literal; all three must be simple literals, the tag non-empty and valid,
        // the direction exactly lowercase "ltr"/"rtl".
        F::StrLangDir => match (str_lit(&ev(0)?), str_lit(&ev(1)?), str_lit(&ev(2)?)) {
            (Some((lex, None)), Some((lang, None)), Some((dir, None))) => {
                let dir = match dir.as_str() {
                    "ltr" => oxrdf::BaseDirection::Ltr,
                    "rtl" => oxrdf::BaseDirection::Rtl,
                    _ => return Ok(Value::Error),
                };
                match Literal::new_directional_language_tagged_literal(lex, lang, dir) {
                    Ok(l) => Value::Term(Term::Literal(l)),
                    Err(_) => Value::Error,
                }
            }
            _ => Value::Error,
        },
        // XSD constructor casts: xsd:integer(?x), xsd:decimal(?x), … (SPARQL 17.5),
        // then the installed extension-function registry (SPARQL 17.6; see
        // `query_with_functions` / `with_functions` in lib.rs). An IRI that is
        // neither stays the same hard query error as before the registry existed.
        F::Custom(nn) => {
            let mut vals = Vec::with_capacity(args.len());
            for i in 0..args.len() {
                vals.push(ev(i)?);
            }
            if vals.len() == 1 {
                if let Some(out) = eval_cast(nn.as_str(), &vals[0]) {
                    return Ok(out);
                }
            }
            if let Some(f) = functions::lookup(nn.as_str()) {
                // Arguments are materialised as concrete RDF terms; an unbound or
                // errored argument is an expression ERROR (row filtered / BIND
                // unbound), exactly like the builtins. The extension returning
                // `Err` (wrong arity, bad lexical, …) is the same expression
                // error — per-row, never a hard query error.
                let mut terms = Vec::with_capacity(vals.len());
                for v in &vals {
                    match value_as_term(v) {
                        Some(t) => terms.push(t),
                        None => return Ok(Value::Error),
                    }
                }
                return Ok(match f(&terms) {
                    Ok(t) => Value::Term(t),
                    Err(_) => Value::Error,
                });
            }
            return Err(format!("unsupported SPARQL function: Custom({})", nn.as_str()));
        }
        // With every default feature on, the arms above are exhaustive (the
        // `F::Custom` arm is no longer guarded on arity); this arm is reached
        // only when the feature-gated builtins (regex / digest / native-only
        // UUID) are compiled out — i.e. the wasm build.
        #[allow(unreachable_patterns)]
        other => return Err(format!("unsupported SPARQL function: {other:?}")),
    })
}

/// Plain decimal form of an f64 with at least one fraction digit ("0.0", "1.0",
/// "1.25") — the form the W3C cast expected-results use for float/double sources.
fn plain_min1(f: f64) -> String {
    if f.fract() == 0.0 && f.abs() < 1e15 {
        format!("{:.1}", f)
    } else {
        format!("{f}")
    }
}

/// A decimal lexical with trailing fraction zeros trimmed but AT LEAST one fraction
/// digit kept ("33.3300" -> "33.33", "0" -> "0.0") — the xsd:decimal cast convention.
fn dec_trim_min1(d: Dec) -> String {
    let mut d = d;
    while d.scale > 1 && d.mant % 10 == 0 {
        d.mant /= 10;
        d.scale -= 1;
    }
    if d.scale == 0 {
        d = Dec { mant: d.mant.saturating_mul(10), scale: 1 };
    }
    d.lexical()
}

/// Trim ALL trailing fraction zeros ("0.0" -> "0", "2.50" -> "2.5") — the xsd:string
/// cast convention for decimal sources.
fn dec_trim(d: Dec) -> String {
    let mut d = d;
    while d.scale > 0 && d.mant % 10 == 0 {
        d.mant /= 10;
        d.scale -= 1;
    }
    d.lexical()
}

/// The XSD constructor-cast table. `None` when `target` is not a recognised cast IRI
/// (the caller reports an unsupported function); `Some(Value::Error)` for a cast that
/// fails per XPath (invalid source lexical, wrong source type, NaN/INF to exact types).
/// The result LEXICAL forms follow the conventions of the W3C cast test expected
/// results (which track the reference implementations), varying by source type.
fn eval_cast(target: &str, v: &Value) -> Option<Value> {
    // The source as a STRING lexical only when it is a simple/xsd:string literal
    // (language-tagged literals and non-string types are NOT castable as strings).
    let src_str = || match v {
        Value::Term(Term::Literal(l)) if l.language().is_none() && l.datatype() == xsd::STRING => {
            Some(l.value().trim().to_string())
        }
        _ => None,
    };
    let typed = |lex: String, dt: oxrdf::NamedNodeRef<'_>| Value::Term(Term::Literal(Literal::new_typed_literal(lex, dt)));
    // Classify a numeric SOURCE by its tower type (computed value or literal).
    let src_num = || as_numeric(v).filter(|_| !matches!(v, Value::Bool(_)));
    if target == xsd::STRING.as_str() {
        // STR semantics with VALUE canonicalisation for typed sources: booleans print
        // true/false, decimals trim trailing zeros ("0.0" -> "0"), float/double print
        // plain when integral ("0E1" -> "0") — everything else keeps its lexical.
        if let Some(b) = as_bool_val(v) {
            return Some(typed(b.to_string(), xsd::STRING));
        }
        if let Some(n) = src_num() {
            let s = match n {
                Num::Int(i) => i.to_string(),
                Num::Dec(d) => dec_trim(d),
                // plain shortest decimal form, never scientific ("0E1" -> "0",
                // "1.25"^^xsd:float -> "1.25")
                Num::Float(f) if f.is_finite() => format!("{f}"),
                Num::Double(f) if f.is_finite() => format!("{f}"),
                other => other.lexical(),
            };
            return Some(typed(s, xsd::STRING));
        }
        return Some(match v {
            Value::Term(Term::BlankNode(_)) | Value::Term(Term::Triple(_)) | Value::Unbound | Value::Error => Value::Error,
            other => value_str(other).map(|s| typed(s, xsd::STRING)).unwrap_or(Value::Error),
        });
    }
    if target == xsd::BOOLEAN.as_str() {
        if let Some(b) = as_bool_val(v) {
            return Some(Value::Bool(b));
        }
        if let Some(n) = as_numeric(v) {
            return Some(Value::Bool(!n.is_zero() && !n.is_nan()));
        }
        return Some(match src_str().as_deref() {
            Some("true") | Some("1") => Value::Bool(true),
            Some("false") | Some("0") => Value::Bool(false),
            _ => Value::Error,
        });
    }
    if target == xsd::DATE_TIME.as_str() {
        return Some(match v {
            Value::Term(Term::Literal(l))
                if l.datatype() == xsd::DATE_TIME || l.datatype() == xsd::DATE_TIME_STAMP =>
            {
                typed(l.value().to_string(), xsd::DATE_TIME)
            }
            _ => match src_str() {
                Some(s) if parse_datetime(&s).is_some() => typed(s, xsd::DATE_TIME),
                _ => Value::Error,
            },
        });
    }
    let is_int = target == xsd::INTEGER.as_str();
    let is_dec = target == xsd::DECIMAL.as_str();
    let is_flt = target == xsd::FLOAT.as_str();
    let is_dbl = target == xsd::DOUBLE.as_str();
    if !(is_int || is_dec || is_flt || is_dbl) {
        return None; // not a cast IRI this engine knows
    }
    if is_int {
        // Truncate toward zero; strings must be valid xsd:integer lexicals.
        if let Some(b) = as_bool_val(v) {
            return Some(Value::Num(Num::Int(b as i64)));
        }
        if let Some(n) = src_num() {
            return Some(match n {
                Num::Int(i) => Value::Num(Num::Int(i)),
                Num::Dec(d) => Value::Num(Num::Int((d.mant / 10i128.pow(d.scale)) as i64)),
                Num::Float(_) | Num::Double(_) => {
                    let f = n.f64();
                    if f.is_finite() && f.abs() < 9.2e18 {
                        Value::Num(Num::Int(f.trunc() as i64))
                    } else {
                        Value::Error
                    }
                }
            });
        }
        return Some(src_str().and_then(|s| s.parse::<i64>().ok()).map(|i| Value::Num(Num::Int(i))).unwrap_or(Value::Error));
    }
    if is_dec {
        if let Some(b) = as_bool_val(v) {
            return Some(typed(if b { "1.0" } else { "0.0" }.to_string(), xsd::DECIMAL));
        }
        if let Some(n) = src_num() {
            return Some(match n {
                // integer -> N.0 (zero prints bare, per the reference results)
                Num::Int(0) => typed("0".to_string(), xsd::DECIMAL),
                Num::Int(i) => typed(format!("{i}.0"), xsd::DECIMAL),
                // decimal -> decimal keeps its lexical
                Num::Dec(d) => typed(d.lexical(), xsd::DECIMAL),
                Num::Float(_) | Num::Double(_) => {
                    let f = n.f64();
                    if f.is_finite() {
                        typed(plain_min1(f), xsd::DECIMAL)
                    } else {
                        Value::Error
                    }
                }
            });
        }
        // string -> parse (no exponent allowed), trim trailing zeros, keep >= 1
        // fraction digit ("+33.3300" -> "33.33", "0" -> "0.0").
        return Some(
            src_str()
                .and_then(|s| Dec::parse_lexical(&s))
                .map(|d| typed(dec_trim_min1(d), xsd::DECIMAL))
                .unwrap_or(Value::Error),
        );
    }
    // float / double
    let dt = if is_flt { xsd::FLOAT } else { xsd::DOUBLE };
    if let Some(b) = as_bool_val(v) {
        return Some(typed(if b { "1.0E0" } else { "0E0" }.to_string(), dt));
    }
    if let Some(n) = src_num() {
        return Some(match n {
            // integer -> N.0 (zero prints bare)
            Num::Int(0) => typed("0".to_string(), dt),
            Num::Int(i) => typed(format!("{i}.0"), dt),
            // decimal -> keeps its lexical
            Num::Dec(d) => typed(d.lexical(), dt),
            // float/double -> plain decimal form with >= 1 fraction digit
            Num::Float(f) => typed(plain_min1(f as f64), dt),
            Num::Double(f) => typed(plain_min1(f), dt),
        });
    }
    Some(match src_str() {
        Some(s) => {
            // A valid integer lexical keeps its form verbatim ("13" -> "13"^^xsd:double);
            // anything else parses and serialises in canonical scientific form.
            if s.parse::<i64>().is_ok() {
                typed(s, dt)
            } else if is_flt {
                match parse_xsd_f32(&s) {
                    Some(f) if !f.is_nan() || s == "NaN" => typed(if f.is_finite() { format!("{f:E}") } else { Num::Float(f).lexical() }, dt),
                    _ => Value::Error,
                }
            } else {
                match parse_xsd_f64(&s) {
                    Some(f) if !f.is_nan() || s == "NaN" => typed(if f.is_finite() { format!("{f:E}") } else { Num::Double(f).lexical() }, dt),
                    _ => Value::Error,
                }
            }
        }
        None => Value::Error,
    })
}

thread_local! {
    /// The query's BASE IRI (when declared), used by IRI()/URI() to resolve relative
    /// references. Set by the `lib.rs` query entry points after parsing.
    static QUERY_BASE: std::cell::RefCell<Option<oxiri::Iri<String>>> = const { std::cell::RefCell::new(None) };
}

/// Installs the active query's base IRI for expression evaluation (IRI()/URI()
/// relative-reference resolution). Called by the query entry points; `None` clears it.
pub(crate) fn set_query_base(base: Option<&str>) {
    QUERY_BASE.with(|b| *b.borrow_mut() = base.and_then(|s| oxiri::Iri::parse(s.to_string()).ok()));
}

/// `IRI(str)`: absolute IRIs pass through; relative references resolve against the
/// query's BASE (a relative reference without a base is a type error).
fn resolve_iri(s: &str) -> Option<oxrdf::NamedNode> {
    if let Ok(abs) = oxiri::Iri::parse(s.to_string()) {
        return Some(oxrdf::NamedNode::new_unchecked(abs.into_inner()));
    }
    QUERY_BASE.with(|b| {
        b.borrow()
            .as_ref()
            .and_then(|base| base.resolve(s).ok())
            .map(|iri| oxrdf::NamedNode::new_unchecked(iri.into_inner()))
    })
}

thread_local! {
    /// The identity of the solution row an expression is being evaluated for:
    /// (bindings identity — the rows buffer address —, row index). Set by the per-row
    /// evaluation loops (BIND / FILTER); BNODE(str) derives its label from it, so equal
    /// arguments within one solution share a blank node while distinct solutions get
    /// distinct ones. The buffer address is stable across consecutive Extends over the
    /// same Bindings (the SELECT-expression case the per-solution rule exists for).
    static ROW_SCOPE: std::cell::Cell<(usize, usize)> = const { std::cell::Cell::new((0, 0)) };
}

/// The current UTC instant as an `xsd:dateTime` lexical (civil-from-days conversion,
/// no time-crate dependency).
#[cfg(not(target_arch = "wasm32"))]
fn now_lexical() -> String {
    let d = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    let secs = d.as_secs() as i64;
    let days = secs.div_euclid(86400);
    let rem = secs.rem_euclid(86400);
    let (h, mi, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    // Howard Hinnant's civil_from_days.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { y + 1 } else { y };
    format!("{year:04}-{month:02}-{day:02}T{h:02}:{mi:02}:{s:02}Z")
}

/// 64-bit FxHash of a string (label material for BNODE(str)).
fn fx64(s: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = rustc_hash::FxHasher::default();
    s.hash(&mut h);
    h.finish()
}

/// The string value of a simple literal / xsd:string argument — the only operand type
/// the hash builtins and BNODE(str) accept; anything else is a type error (`None`).
fn string_literal(v: &Value) -> Option<String> {
    match v {
        Value::Term(Term::Literal(l)) if l.language().is_none() && l.datatype() == xsd::STRING => {
            Some(l.value().to_string())
        }
        _ => None,
    }
}

/// A STRING-LITERAL operand (simple/xsd:string or language-tagged): `(value, language)`.
/// IRIs, blank nodes, non-string literals and computed numerics/booleans are type
/// errors (`None`) — per the SPARQL string-function operand rules.
///
/// An RDF 1.2 base direction rides COMBINED into the language slot as `lang--dir`
/// (`--` cannot occur in a BCP47 tag): the string functions then preserve language AND
/// direction together, equality of the slot means "same language and same direction"
/// (exactly the SPARQL 1.2 CONCAT/compatibility rule), and [`lit_with_lang`] splits the
/// pair back out.
fn str_lit(v: &Value) -> Option<(String, Option<String>)> {
    match v {
        Value::Term(Term::Literal(l)) => {
            if let Some(lang) = l.language() {
                let tag = match l.direction() {
                    Some(d) => format!("{lang}--{d}"),
                    None => lang.to_string(),
                };
                Some((l.value().to_string(), Some(tag)))
            } else if l.datatype() == xsd::STRING {
                Some((l.value().to_string(), None))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// A simple or language-tagged literal value, per the (possibly `lang--dir` combined,
/// see [`str_lit`]) tag the operand carried.
fn lit_with_lang(s: String, lang: Option<&str>) -> Value {
    Value::Term(Term::Literal(match lang {
        Some(l) => match l.split_once("--") {
            Some((tag, dir)) => Literal::new_directional_language_tagged_literal_unchecked(
                s,
                tag,
                if dir == "rtl" { oxrdf::BaseDirection::Rtl } else { oxrdf::BaseDirection::Ltr },
            ),
            None => Literal::new_language_tagged_literal_unchecked(s, l),
        },
        None => Literal::new_simple_literal(s),
    }))
}

/// Lowercase-hex digest of a string-literal argument, or a type error.
#[cfg(feature = "digest")]
fn digest_hex<D: md5::Digest>(v: &Value) -> Value {
    use std::fmt::Write;
    match string_literal(v) {
        Some(s) => {
            let out = D::digest(s.as_bytes());
            let mut hex = String::with_capacity(out.len() * 2);
            for byte in out {
                let _ = write!(hex, "{byte:02x}");
            }
            Value::Term(Term::Literal(Literal::new_simple_literal(hex)))
        }
        None => Value::Error,
    }
}

/// The timezone part of an xsd:dateTime argument's lexical form: `"Z"`, `"±hh:mm"`, or
/// `""` when absent. `None` (type error) when the argument is not a valid xsd:dateTime.
fn datetime_arg_tz(v: &Value) -> Option<String> {
    let l = match v {
        Value::Term(Term::Literal(l))
            if l.datatype() == xsd::DATE_TIME || l.datatype() == xsd::DATE_TIME_STAMP =>
        {
            l
        }
        _ => return None,
    };
    let s = l.value();
    parse_datetime(s)?; // lexical shape check
    let (_, time) = s.split_once('T')?;
    Some(match time.find(['Z', '+', '-']) {
        Some(i) => time[i..].to_string(),
        None => String::new(),
    })
}

/// XSD-canonical `xsd:dayTimeDuration` for a timezone string (`Z` / `±hh:mm`); an empty
/// timezone is a type error (`None`).
fn tz_to_duration(tz: &str) -> Option<String> {
    if tz.is_empty() {
        return None;
    }
    if tz == "Z" {
        return Some("PT0S".to_string());
    }
    let (sign, hm) = tz.split_at(1);
    let (h, m) = hm.split_once(':')?;
    let (h, m): (u32, u32) = (h.parse().ok()?, m.parse().ok()?);
    if h == 0 && m == 0 {
        return Some("PT0S".to_string());
    }
    let mut out = String::new();
    if sign == "-" {
        out.push('-');
    }
    out.push_str("PT");
    if h > 0 {
        out.push_str(&format!("{h}H"));
    }
    if m > 0 {
        out.push_str(&format!("{m}M"));
    }
    Some(out)
}

/// Build a regex honouring the SPARQL flag string (`i` case-insensitive, `s` dot-all, `m`
/// multi-line, `x` extended/ignore-whitespace, `q` literal-pattern mode per XPath F&O —
/// every pattern character is matched literally, combinable with `i`).
/// Returns `None` on an invalid pattern or an unknown flag (→ type error).
#[cfg(feature = "regex")]
fn build_regex(pattern: &str, flags: &str) -> Option<regex::Regex> {
    if !flags.chars().all(|c| matches!(c, 'i' | 's' | 'm' | 'x' | 'q')) {
        return None;
    }
    let literal = flags.contains('q');
    let pattern = if literal { regex::escape(pattern) } else { pattern.to_string() };
    regex::RegexBuilder::new(&pattern)
        .case_insensitive(flags.contains('i'))
        // `q` suppresses the meaning of the OTHER flags' metacharacters too (per
        // XPath, only `i` keeps its effect alongside `q`).
        .dot_matches_new_line(!literal && flags.contains('s'))
        .multi_line(!literal && flags.contains('m'))
        .ignore_whitespace(!literal && flags.contains('x'))
        .build()
        .ok()
}

/// SPARQL `ENCODE_FOR_URI`: percent-encode everything except the unreserved set (RFC 3986
/// ALPHA / DIGIT / `-` `.` `_` `~`).
fn encode_for_uri(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for byte in s.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(byte as char);
        } else {
            out.push('%');
            out.push_str(&format!("{byte:02X}"));
        }
    }
    out
}

/// Extract a numeric `xsd:dateTime` component (0=year…5=seconds) from a value's lexical
/// form. YEAR…MINUTES return xsd:integer; SECONDS returns xsd:decimal (per SPARQL),
/// parsed from the lexical so fractional seconds stay exact.
fn datetime_field(v: &Value, idx: usize) -> Value {
    let s = match value_str(v) {
        Some(s) => s,
        None => return Value::Error,
    };
    let fields = match parse_datetime(&s) {
        Some(f) => f,
        None => return Value::Error,
    };
    if idx == 5 {
        // Re-extract the seconds lexical (timezone stripped) for an exact decimal.
        let lex = s
            .split_once('T')
            .map(|(_, t)| if let Some(i) = t.find(['Z', '+', '-']) { &t[..i] } else { t })
            .and_then(|t| t.rsplit_once(':').map(|(_, sec)| sec));
        return match lex.and_then(Dec::parse_lexical) {
            Some(d) => Value::Num(Num::Dec(d)),
            None => Value::Error,
        };
    }
    Value::Num(Num::Int(fields[idx] as i64))
}

/// Parse an `xsd:dateTime` lexical (`[-]YYYY-MM-DDThh:mm:ss[.frac][TZ]`) into
/// `[year, month, day, hours, minutes, seconds]`. Timezone is stripped (component accessors are on
/// the local time per SPARQL); seconds keeps any fractional part.
fn parse_datetime(s: &str) -> Option<[f64; 6]> {
    let (date, time) = s.split_once('T')?;
    let neg = date.starts_with('-');
    let mut d = date.strip_prefix('-').unwrap_or(date).split('-');
    let year: f64 = d.next()?.parse().ok()?;
    let year = if neg { -year } else { year };
    let month: f64 = d.next()?.parse().ok()?;
    let day: f64 = d.next()?.parse().ok()?;
    // Strip the timezone (Z, or +hh:mm / -hh:mm after the seconds — the time part itself has no '-').
    let time = if let Some(i) = time.find(['Z', '+', '-']) { &time[..i] } else { time };
    let mut t = time.split(':');
    let hours: f64 = t.next()?.parse().ok()?;
    let minutes: f64 = t.next()?.parse().ok()?;
    let seconds: f64 = t.next()?.parse().ok()?;
    Some([year, month, day, hours, minutes, seconds])
}

fn value_str(v: &Value) -> Option<String> {
    match v {
        Value::Term(Term::Literal(l)) => Some(l.value().to_string()),
        Value::Term(Term::NamedNode(n)) => Some(n.as_str().to_string()),
        Value::Term(Term::BlankNode(b)) => Some(b.as_str().to_string()),
        Value::Bool(b) => Some(b.to_string()),
        Value::Num(n) => Some(n.lexical()),
        Value::Unbound | Value::Error => None,
        Value::Term(_) => None,
    }
}

/// Resolves a computed NUMERIC value to an id without constructing a Term when
/// possible: an inline-range integer encodes straight into its id (no allocation at
/// all); any other numeric probes the dictionary by (lexical, datatype) parts.
/// `Ok(id)` on a hit; `Err(lexical)` carries the (already formatted) lexical form for
/// the caller's local-vocab miss path.
#[inline]
fn num_to_id(graph: &Graph, n: Num) -> Result<Id, String> {
    if let Num::Int(i) = n {
        if let Some(id) = dict::inline_id_of_int(i) {
            return Ok(id);
        }
    }
    let lex = n.lexical();
    match graph.dict.lookup_lit(&lex, n.datatype().as_str(), None) {
        NO_ID => Err(lex),
        id => Ok(id),
    }
}

/// Converts an evaluated value into an id. Computed terms are resolved against
/// the graph dictionary first (so they join and deduplicate against the data);
/// terms not already present get a per-query local id. Computed numerics skip
/// Term construction entirely when they resolve to an inline id or a dictionary
/// term (the BIND fast path).
fn value_to_id(graph: &Graph, local: &mut LocalVocab, v: &Value) -> Id {
    let term = match v {
        Value::Unbound | Value::Error => return NO_ID,
        Value::Num(n) => match num_to_id(graph, *n) {
            Ok(id) => return id,
            Err(lex) => Term::Literal(Literal::new_typed_literal(lex, n.datatype())),
        },
        Value::Bool(b) => {
            let lex = if *b { "true" } else { "false" };
            match graph.dict.lookup_lit(lex, xsd::BOOLEAN.as_str(), None) {
                NO_ID => Term::Literal(Literal::new_typed_literal(lex, xsd::BOOLEAN)),
                id => return id,
            }
        }
        Value::Term(t) => {
            if let Some(id) = graph.id_of(t) {
                return id;
            }
            t.clone()
        }
    };
    local.intern(term)
}

/// A computed value as a concrete RDF term (`None` for unbound / type error).
fn value_as_term(v: &Value) -> Option<Term> {
    Some(match v {
        Value::Unbound | Value::Error => return None,
        Value::Bool(b) => Term::Literal(Literal::new_typed_literal(b.to_string(), xsd::BOOLEAN)),
        Value::Num(n) => Term::Literal(Literal::new_typed_literal(n.lexical(), n.datatype())),
        Value::Term(t) => t.clone(),
    })
}

/// Read-only half of [`value_to_id`], for the parallel resolve pass (T1.0b). Most computed values
/// resolve WITHOUT touching the mutable vocab: small integers inline into the id, and terms
/// already in the graph dictionary (or already interned locally) are read-only lookups. Only a
/// genuinely new term returns `Err(term)` — carrying the constructed `Term` so the serial
/// intern-the-misses pass does no re-construction. Splitting this way removes the bulk of the B1
/// serialization point (research/parallelism-scaling.md) without sharded-vocab complexity, and
/// stays byte-identical: misses are interned in row order, exactly as the serial path would.
fn value_to_id_readonly(graph: &Graph, local: &LocalVocab, v: &Value) -> Result<Id, Term> {
    let term = match v {
        Value::Unbound | Value::Error => return Ok(NO_ID),
        // Computed numerics/booleans skip Term construction when they resolve to an
        // inline id or a dictionary term — the constructed-Term path only runs for
        // values that genuinely head to the local vocab (the BIND fast path).
        Value::Num(n) => match num_to_id(graph, *n) {
            Ok(id) => return Ok(id),
            Err(lex) => Term::Literal(Literal::new_typed_literal(lex, n.datatype())),
        },
        Value::Bool(b) => {
            let lex = if *b { "true" } else { "false" };
            match graph.dict.lookup_lit(lex, xsd::BOOLEAN.as_str(), None) {
                NO_ID => Term::Literal(Literal::new_typed_literal(lex, xsd::BOOLEAN)),
                id => return Ok(id),
            }
        }
        Value::Term(t) => {
            if let Some(id) = graph.id_of(t) {
                return Ok(id);
            }
            t.clone()
        }
    };
    if let Some(&id) = local.ids.get(&term) {
        return Ok(id);
    }
    Err(term)
}

// ---- spargebra term helpers --------------------------------------------------

fn ground_to_term(g: &GroundTerm) -> Term {
    match g {
        GroundTerm::NamedNode(n) => Term::NamedNode(n.clone()),
        GroundTerm::Literal(l) => Term::Literal(l.clone()),
        // A ground RDF 1.2 triple term (e.g. in VALUES): fully concrete, so it maps
        // straight to a structural `Term::Triple` (the object may nest another).
        GroundTerm::Triple(t) => Term::Triple(Box::new(oxrdf::Triple::new(
            t.subject.clone(),
            t.predicate.clone(),
            ground_to_term(&t.object),
        ))),
    }
}

fn term_pattern_to_term(tp: &TermPattern) -> Result<Term, String> {
    match tp {
        TermPattern::NamedNode(n) => Ok(Term::NamedNode(n.clone())),
        TermPattern::BlankNode(b) => Ok(Term::BlankNode(b.clone())),
        TermPattern::Literal(l) => Ok(Term::Literal(l.clone())),
        TermPattern::Variable(_) => Err("variable where a term was expected".into()),
        // RDF-star GROUND triple term `<<( s p o )>>` (RDF 1.2): build the structural
        // `Term::Triple`, which the dictionary interns/looks up by its component ids.
        // Variables INSIDE a quoted-triple pattern remain unsupported (they need pattern
        // decomposition over stored triple terms, not a single id — a clean error, no crash).
        TermPattern::Triple(t) => {
            let subject: oxrdf::NamedOrBlankNode = match term_pattern_to_term(&t.subject)? {
                Term::NamedNode(n) => n.into(),
                Term::BlankNode(b) => b.into(),
                _ => return Err("RDF-star triple-term subject must be an IRI or blank node".into()),
            };
            let predicate = match &t.predicate {
                NamedNodePattern::NamedNode(n) => n.clone(),
                NamedNodePattern::Variable(_) => {
                    return Err("variable inside a triple-term pattern is not yet supported (T6)".into())
                }
            };
            let object = term_pattern_to_term(&t.object)?;
            Ok(Term::Triple(Box::new(oxrdf::Triple::new(subject, predicate, object))))
        }
    }
}

/// Prefix for the synthetic variables that stand in for blank nodes (which are
/// existential variables in a query). `#` cannot appear in a SPARQL `VARNAME`,
/// so these can never collide with a user variable, and the `SELECT *` filter on
/// this prefix can never hide a real one.
const BNODE_VAR_PREFIX: &str = "#bn#";

fn bnode_var(b: &oxrdf::BlankNode) -> Variable {
    Variable::new_unchecked(format!("{BNODE_VAR_PREFIX}{}", b.as_str()))
}

fn tp_var(tp: &TermPattern) -> Option<Variable> {
    match tp {
        TermPattern::Variable(v) => Some(v.clone()),
        TermPattern::BlankNode(b) => Some(bnode_var(b)),
        _ => None,
    }
}

fn nnp_var(p: &NamedNodePattern) -> Option<Variable> {
    match p {
        NamedNodePattern::Variable(v) => Some(v.clone()),
        _ => None,
    }
}

#[cfg(test)]
mod exact_decimal_tests {
    use super::*;

    #[test]
    fn dec_arithmetic_is_exact() {
        use std::cmp::Ordering::*;
        let d = |s: &str| Dec::parse(s).unwrap();
        // The classic: 0.1 + 0.2 == 0.3 exactly (false in f64).
        assert_eq!(d("0.1").checked_add(d("0.2")).unwrap().cmp(d("0.3")), Some(Equal));
        // 0.3 - 0.1 == 0.2 exactly.
        assert_eq!(d("0.3").checked_sub(d("0.1")).unwrap().cmp(d("0.2")), Some(Equal));
        // Mixed scales + integer/decimal.
        assert_eq!(d("1").checked_add(d("0.5")).unwrap().cmp(d("1.5")), Some(Equal));
        assert_eq!(d("83").checked_mul(d("0.5")).unwrap().cmp(d("41.5")), Some(Equal));
        assert_eq!(d("0.1").checked_mul(d("0.1")).unwrap().cmp(d("0.01")), Some(Equal));
        // Ordering across scales and signs.
        assert_eq!(d("0.30000000000000001").cmp(d("0.3")), Some(Greater));
        assert_eq!(d("-2.5").checked_add(d("1")).unwrap().cmp(d("-1.5")), Some(Equal));
        // Large integers beyond 2^53 stay exact.
        assert_eq!(d("9007199254740992").checked_add(d("1")).unwrap().cmp(d("9007199254740993")), Some(Equal));
        // Non-decimal lexical (exponent) -> not parseable here -> None.
        assert!(Dec::parse("1e5").is_none());
    }

    #[test]
    fn cmp_decimal_str_is_exact() {
        use std::cmp::Ordering::*;
        // Integers beyond f64 precision, and high-precision decimals that share an f64.
        assert_eq!(cmp_decimal_str("9007199254740992", "9007199254740993"), Some(Less));
        assert_eq!(cmp_decimal_str("0.123456789012345678", "0.123456789012345679"), Some(Less));
        // Equality incl. non-canonical zeros / trailing zeros / signed zero.
        assert_eq!(cmp_decimal_str("1.50", "1.5"), Some(Equal));
        assert_eq!(cmp_decimal_str("007", "7"), Some(Equal));
        assert_eq!(cmp_decimal_str("-0", "0"), Some(Equal));
        assert_eq!(cmp_decimal_str("0.0", "-0.0"), Some(Equal));
        // Sign + magnitude.
        assert_eq!(cmp_decimal_str("-3", "2"), Some(Less));
        assert_eq!(cmp_decimal_str("-2", "-3"), Some(Greater));
        assert_eq!(cmp_decimal_str("10", "9"), Some(Greater));
        assert_eq!(cmp_decimal_str("1.1", "1.09"), Some(Greater));
        assert_eq!(cmp_decimal_str("0.1", "0.2"), Some(Less));
        // Integer vs decimal of equal value.
        assert_eq!(cmp_decimal_str("5", "5.0"), Some(Equal));
        assert_eq!(cmp_decimal_str("5", "5.0000000000000001"), Some(Less));
        // Malformed -> None (falls back to f64).
        assert_eq!(cmp_decimal_str("1.2.3", "1"), None);
        assert_eq!(cmp_decimal_str("1e5", "1"), None);
    }

    #[test]
    fn sig_digits_counts() {
        assert_eq!(sig_digits("120"), 3);
        assert_eq!(sig_digits("0.5"), 1);
        assert_eq!(sig_digits("9007199254740992"), 16);
        assert_eq!(sig_digits("0.123456789012345679"), 18);
        assert_eq!(sig_digits("0.00123"), 3); // leading fraction zeros are not significant
        assert_eq!(sig_digits("100.0"), 3);
    }
}

#[cfg(test)]
mod wcoj_tests {
    use super::*;
    use spargebra::SparqlParser;

    /// Extracts the BGP triple patterns from a simple `SELECT * WHERE { ... }`.
    fn bgp(sparql: &str) -> Vec<TriplePattern> {
        let q = SparqlParser::new().parse_query(sparql).unwrap();
        let spargebra::Query::Select { pattern, .. } = q else { panic!() };
        let mut patterns = Vec::new();
        let mut filters = Vec::new();
        // unwrap Project -> inner
        fn inner_of(p: &GraphPattern) -> &GraphPattern {
            match p {
                GraphPattern::Project { inner, .. }
                | GraphPattern::Distinct { inner }
                | GraphPattern::Slice { inner, .. } => inner_of(inner),
                other => other,
            }
        }
        flatten_conjunction(inner_of(&pattern), &mut patterns, &mut filters);
        patterns
    }

    /// Sorted set of result rows from a Bindings, for order-independent equality.
    fn rowset(b: &Bindings) -> Vec<Vec<(String, Id)>> {
        let mut rows: Vec<Vec<(String, Id)>> = b
            .rows
            .iter()
            .map(|r| {
                let mut kv: Vec<(String, Id)> =
                    b.vars.iter().zip(r).map(|(v, &id)| (v.as_str().to_string(), id)).collect();
                kv.sort();
                kv
            })
            .collect();
        rows.sort();
        rows
    }

    fn random_graph(seed0: u64, n_nodes: u32, n_edges: usize) -> Graph {
        let mut seed = seed0;
        let mut next = || {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (seed >> 33) as u32
        };
        let mut ttl = String::from("@prefix ex: <http://ex/> .\n");
        for _ in 0..n_edges {
            let a = next() % n_nodes;
            let b = next() % n_nodes;
            ttl.push_str(&format!("ex:n{a} ex:e ex:n{b} .\n"));
        }
        Graph::load_str(&ttl, "turtle").unwrap()
    }

    /// The binary and WCOJ BGP plans must return identical result sets.
    fn assert_plans_agree(sparql: &str, graph: &Graph) {
        let patterns = bgp(sparql);
        let binary = eval_bgp_binary(graph, &patterns, &[]).unwrap();
        let wcoj = eval_bgp_wcoj(graph, &patterns).unwrap();
        assert_eq!(
            rowset(&binary),
            rowset(&wcoj),
            "binary vs WCOJ disagree for `{sparql}` (binary {} rows, wcoj {} rows)",
            binary.rows.len(),
            wcoj.rows.len()
        );
    }

    #[test]
    fn cyclicity_classification() {
        assert!(!bgp_is_cyclic(&bgp("PREFIX ex: <http://ex/> SELECT * WHERE { ?a ex:e ?b . ?b ex:e ?c }")));
        assert!(bgp_is_cyclic(&bgp(
            "PREFIX ex: <http://ex/> SELECT * WHERE { ?a ex:e ?b . ?b ex:e ?c . ?c ex:e ?a }"
        )));
        assert!(bgp_is_cyclic(&bgp(
            "PREFIX ex: <http://ex/> SELECT * WHERE { ?a ex:e ?b . ?b ex:e ?c . ?c ex:e ?d . ?d ex:e ?a }"
        )));
        // star is acyclic
        assert!(!bgp_is_cyclic(&bgp(
            "PREFIX ex: <http://ex/> SELECT * WHERE { ?a ex:e ?b . ?a ex:e ?c . ?a ex:e ?d }"
        )));
    }

    #[test]
    fn wcoj_matches_binary_over_random_graphs() {
        for seed in [0x1111u64, 0xACE1, 0xDEADBEEF, 0x5EED] {
            let g = random_graph(seed, 25, 120);
            // chain (acyclic)
            assert_plans_agree("PREFIX ex: <http://ex/> SELECT * WHERE { ?a ex:e ?b . ?b ex:e ?c }", &g);
            // triangle (cyclic) — the canonical WCOJ win
            assert_plans_agree(
                "PREFIX ex: <http://ex/> SELECT * WHERE { ?a ex:e ?b . ?b ex:e ?c . ?c ex:e ?a }",
                &g,
            );
            // 4-cycle
            assert_plans_agree(
                "PREFIX ex: <http://ex/> SELECT * WHERE { ?a ex:e ?b . ?b ex:e ?c . ?c ex:e ?d . ?d ex:e ?a }",
                &g,
            );
            // square with a diagonal (denser cycle)
            assert_plans_agree(
                "PREFIX ex: <http://ex/> SELECT * WHERE { ?a ex:e ?b . ?b ex:e ?c . ?c ex:e ?a . ?a ex:e ?d . ?d ex:e ?c }",
                &g,
            );
        }
    }

    #[test]
    fn wcoj_repeated_variable_pattern() {
        // self-loops: ?x ex:e ?x  — repeated-variable handling in build_trie.
        let g = Graph::load_str(
            "@prefix ex: <http://ex/> . ex:a ex:e ex:a . ex:a ex:e ex:b . ex:b ex:e ex:b .",
            "turtle",
        )
        .unwrap();
        let patterns = bgp("PREFIX ex: <http://ex/> SELECT * WHERE { ?x ex:e ?x }");
        let wcoj = eval_bgp_wcoj(&g, &patterns).unwrap();
        assert_eq!(wcoj.rows.len(), 2); // a and b
    }
}

#[cfg(test)]
mod function_tests {
    use super::*;

    fn g() -> Graph {
        Graph::load_str(
            "@prefix : <http://ex/> .\n\
             :a :name \"Alice\" . :b :name \"bob\" . :c :name \"Carol123\" .\n\
             :a :age 30 . :b :age -5 .\n",
            "turtle",
        )
        .unwrap()
    }
    // Result-set size via the full materialising path (applies FILTER/BIND functions).
    fn n(sparql: &str) -> usize {
        crate::query(&g(), sparql).unwrap().len()
    }

    #[test]
    fn string_functions() {
        assert_eq!(n("SELECT ?n WHERE { ?s <http://ex/name> ?n FILTER(STRLEN(?n) > 3) }"), 2); // Alice, Carol123
        assert_eq!(n("SELECT ?n WHERE { ?s <http://ex/name> ?n FILTER(STRSTARTS(?n, \"A\")) }"), 1);
        assert_eq!(n("SELECT ?n WHERE { ?s <http://ex/name> ?n FILTER(STRENDS(?n, \"3\")) }"), 1);
        assert_eq!(n("SELECT ?n WHERE { ?s <http://ex/name> ?n FILTER(CONTAINS(LCASE(?n), \"o\")) }"), 2); // bob, carol
        assert_eq!(n("SELECT ?n WHERE { ?s <http://ex/name> ?n FILTER(UCASE(?n) = \"BOB\") }"), 1);
        // BIND never drops rows.
        assert_eq!(n("SELECT ?g WHERE { ?s <http://ex/name> ?n BIND(CONCAT(?n, \"!\") AS ?g) }"), 3);
        assert_eq!(n("SELECT ?g WHERE { ?s <http://ex/name> ?n BIND(SUBSTR(?n, 1, 2) AS ?g) }"), 3);
    }

    #[test]
    fn numeric_and_type_functions() {
        assert_eq!(n("SELECT ?a WHERE { ?s <http://ex/age> ?a FILTER(ABS(?a) > 10) }"), 1); // |30|
        assert_eq!(n("SELECT ?a WHERE { ?s <http://ex/age> ?a FILTER(isNumeric(?a)) }"), 2);
        assert_eq!(n("SELECT ?n WHERE { ?s <http://ex/name> ?n FILTER(isLiteral(?n)) }"), 3);
        assert_eq!(n("SELECT ?s WHERE { ?s <http://ex/name> ?n FILTER(isIRI(?s)) }"), 3);
        assert_eq!(n("SELECT ?a WHERE { ?s <http://ex/age> ?a FILTER(FLOOR(?a) = ?a) }"), 2);
    }

    #[test]
    fn lang_typed_and_sample() {
        let g = Graph::load_str(
            "@prefix : <http://ex/> . :a :name \"Alice\"@en . :b :name \"Bob\"@fr . :c :name \"X\" .",
            "turtle",
        )
        .unwrap();
        let q = |s: &str| crate::query(&g, s).unwrap().len();
        // LANGMATCHES: "en" matches the en literal; "*" matches any non-empty tag (en, fr; not X).
        assert_eq!(q("SELECT ?n WHERE { ?s <http://ex/name> ?n FILTER(LANGMATCHES(LANG(?n), \"en\")) }"), 1);
        assert_eq!(q("SELECT ?n WHERE { ?s <http://ex/name> ?n FILTER(LANGMATCHES(LANG(?n), \"*\")) }"), 2);
        // STRLANG / STRDT construct literals (exercised via BIND — all 3 rows).
        assert_eq!(q("SELECT ?x WHERE { ?s <http://ex/name> ?n BIND(STRLANG(STR(?n), \"de\") AS ?x) }"), 3);
        assert_eq!(
            q("SELECT ?x WHERE { ?s <http://ex/name> ?n BIND(STRDT(STR(?n), <http://www.w3.org/2001/XMLSchema#string>) AS ?x) }"),
            3
        );
        // SAMPLE: one row per group.
        let g2 = Graph::load_str("@prefix : <http://ex/> . :a :p :x . :a :p :y . :b :p :z .", "turtle").unwrap();
        assert_eq!(
            crate::query(&g2, "SELECT ?s (SAMPLE(?o) AS ?v) WHERE { ?s <http://ex/p> ?o } GROUP BY ?s").unwrap().len(),
            2
        );
    }

    #[test]
    fn datetime_accessors() {
        let g = Graph::load_str(
            "@prefix : <http://ex/> . @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
             :e :at \"2024-03-15T13:45:30\"^^xsd:dateTime .",
            "turtle",
        )
        .unwrap();
        let q = |s: &str| crate::query(&g, s).unwrap().len();
        assert_eq!(q("SELECT * WHERE { ?e <http://ex/at> ?d FILTER(YEAR(?d) = 2024 && MONTH(?d) = 3 && DAY(?d) = 15) }"), 1);
        assert_eq!(q("SELECT * WHERE { ?e <http://ex/at> ?d FILTER(HOURS(?d) = 13 && MINUTES(?d) = 45 && SECONDS(?d) = 30) }"), 1);
    }

    #[cfg(feature = "regex")]
    #[test]
    fn regex_functions() {
        assert_eq!(n("SELECT ?n WHERE { ?s <http://ex/name> ?n FILTER(REGEX(?n, \"^[A-Z]\")) }"), 2); // Alice, Carol123
        assert_eq!(n("SELECT ?n WHERE { ?s <http://ex/name> ?n FILTER(REGEX(?n, \"BOB\", \"i\")) }"), 1);
        assert_eq!(n("SELECT ?n WHERE { ?s <http://ex/name> ?n FILTER(REGEX(?n, \"[0-9]+\")) }"), 1); // Carol123
        assert_eq!(n("SELECT ?g WHERE { ?s <http://ex/name> ?n BIND(REPLACE(?n, \"[0-9]\", \"X\") AS ?g) }"), 3);
    }
}

#[cfg(test)]
mod path_tests {
    use super::*;

    // Chain a -p-> b -p-> c -p-> d, plus a -q-> x.
    fn g() -> Graph {
        Graph::load_str(
            "@prefix : <http://ex/> .\n:a :p :b . :b :p :c . :c :p :d . :a :q :x .\n",
            "turtle",
        )
        .unwrap()
    }
    fn n(sparql: &str) -> usize {
        crate::query(&g(), sparql).unwrap().len()
    }
    const PFX: &str = "PREFIX : <http://ex/> ";

    #[test]
    fn property_paths() {
        let n = |q: &str| n(&format!("{PFX}{q}"));
        // OneOrMore (transitive): all reachable pairs over the chain = C(4,2) = 6.
        assert_eq!(n("SELECT ?x ?y WHERE { ?x :p+ ?y }"), 6);
        assert_eq!(n("SELECT ?y WHERE { :a :p+ ?y }"), 3); // b,c,d
        // ZeroOrMore adds reflexive self-match: a,b,c,d.
        assert_eq!(n("SELECT ?y WHERE { :a :p* ?y }"), 4);
        // Sequence / Alternative / ZeroOrOne.
        assert_eq!(n("SELECT ?y WHERE { :a :p/:p ?y }"), 1); // c
        assert_eq!(n("SELECT ?y WHERE { :a :p|:q ?y }"), 2); // b, x
        assert_eq!(n("SELECT ?y WHERE { :a :p? ?y }"), 2); // a (zero), b (one)
        // Reverse (inverse), incl. inverse-transitive.
        assert_eq!(n("SELECT ?x WHERE { :d ^:p ?x }"), 1); // c
        assert_eq!(n("SELECT ?x WHERE { :d ^:p+ ?x }"), 3); // c,b,a
        // NegatedPropertySet: edges not via :p  ->  only a -q-> x.
        assert_eq!(n("SELECT ?x ?y WHERE { ?x !:p ?y }"), 1);
    }

    #[test]
    fn named_graphs() {
        // One default-graph triple + two named graphs (g1 has 2, g2 has 1).
        let nq = "<http://ex/a> <http://ex/p> <http://ex/x> .\n\
                  <http://ex/b> <http://ex/p> <http://ex/y> <http://ex/g1> .\n\
                  <http://ex/c> <http://ex/p> <http://ex/z> <http://ex/g1> .\n\
                  <http://ex/d> <http://ex/p> <http://ex/w> <http://ex/g2> .\n";
        let g = Graph::load_dataset(nq, "nquads").unwrap();
        let n = |q: &str| crate::query(&g, q).unwrap().len();
        // The default graph holds ONLY default triples (named graphs are not folded in).
        assert_eq!(n("SELECT * WHERE { ?s ?p ?o }"), 1);
        assert_eq!(n("SELECT * WHERE { GRAPH <http://ex/g1> { ?s ?p ?o } }"), 2);
        assert_eq!(n("SELECT * WHERE { GRAPH <http://ex/g2> { ?s ?p ?o } }"), 1);
        // GRAPH ?g ranges over both named graphs (3 triples), binding ?g; an absent graph -> 0.
        assert_eq!(n("SELECT ?g ?s WHERE { GRAPH ?g { ?s ?p ?o } }"), 3);
        assert_eq!(n("SELECT * WHERE { GRAPH <http://ex/absent> { ?s ?p ?o } }"), 0);
        // Result ids are translated to the outer dict, so a join across GRAPH works.
        assert_eq!(n("SELECT ?o WHERE { GRAPH ?g { <http://ex/b> <http://ex/p> ?o } }"), 1);
    }

    #[test]
    fn rdf_star_concrete_triple_terms() {
        // RDF 1.2 triple terms load and CONCRETE `<< … >>` patterns match via the
        // STRUCTURAL dictionary encoding (component ids). Variable-inside patterns are
        // still unsupported (clean error, tested below).
        let g = Graph::load_str(
            "PREFIX : <http://ex/>\n<< :alice :age 30 >> :certainty 0.9 .\n<< :bob :age 25 >> :certainty 0.5 .",
            "turtle",
        )
        .unwrap();
        let n = |q: &str| crate::query(&g, q).unwrap().len();
        assert_eq!(n("PREFIX : <http://ex/> SELECT ?c WHERE { << :alice :age 30 >> :certainty ?c }"), 1);
        assert_eq!(n("PREFIX : <http://ex/> SELECT ?c WHERE { << :carol :age 99 >> :certainty ?c }"), 0);
    }

    #[test]
    fn rdf_star_structural_roundtrip_output() {
        // Loading RDF-star Turtle and selecting the triple term materialises a structural
        // `Term::Triple` (oxrdf formats it as `<<( … )>>`), NOT the old canonical-string
        // literal stopgap.
        let g = Graph::load_str("PREFIX : <http://ex/>\n<< :alice :age 30 >> :certainty 0.9 .", "turtle").unwrap();
        let r = crate::query(
            &g,
            "SELECT ?t WHERE { ?r <http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies> ?t }",
        )
        .unwrap();
        assert_eq!(r.rows.len(), 1);
        let expected = Term::Triple(Box::new(oxrdf::Triple::new(
            oxrdf::NamedNode::new_unchecked("http://ex/alice"),
            oxrdf::NamedNode::new_unchecked("http://ex/age"),
            Term::Literal(Literal::new_typed_literal("30", xsd::INTEGER)),
        )));
        assert_eq!(r.rows[0][0], Some(expected));

        // SPARQL 1.2 JSON results encoding: {"type":"triple","value":{subject/predicate/object}}.
        let json = crate::query_json(
            &g,
            "SELECT ?t WHERE { ?r <http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies> ?t }",
        )
        .unwrap();
        assert!(
            json.contains(
                "{\"type\":\"triple\",\"value\":{\"subject\":{\"type\":\"uri\",\"value\":\"http://ex/alice\"},\
                 \"predicate\":{\"type\":\"uri\",\"value\":\"http://ex/age\"},\"object\":{\"type\":\"literal\",\
                 \"value\":\"30\",\"datatype\":\"http://www.w3.org/2001/XMLSchema#integer\"}}}"
            ),
            "got: {json}"
        );
    }

    #[test]
    fn rdf_star_nested_triple_terms() {
        // A triple term nests through the OBJECT position; it round-trips structurally.
        let g = Graph::load_str("PREFIX : <http://ex/>\n:x :p <<( :a :b <<( :c :d :e )>> )>> .", "turtle").unwrap();
        let r = crate::query(&g, "PREFIX : <http://ex/> SELECT ?o WHERE { :x :p ?o }").unwrap();
        assert_eq!(r.rows.len(), 1);
        let nn = oxrdf::NamedNode::new_unchecked;
        let inner = Term::Triple(Box::new(oxrdf::Triple::new(nn("http://ex/c"), nn("http://ex/d"), Term::NamedNode(nn("http://ex/e")))));
        let outer = Term::Triple(Box::new(oxrdf::Triple::new(nn("http://ex/a"), nn("http://ex/b"), inner)));
        assert_eq!(r.rows[0][0], Some(outer.clone()));
        // The concrete nested pattern matches (1) and a non-matching one misses (0).
        let n = |q: &str| crate::query(&g, q).unwrap().len();
        assert_eq!(n("PREFIX : <http://ex/> SELECT ?s WHERE { ?s :p <<( :a :b <<( :c :d :e )>> )>> }"), 1);
        assert_eq!(n("PREFIX : <http://ex/> SELECT ?s WHERE { ?s :p <<( :a :b <<( :c :d :x )>> )>> }"), 0);
    }

    #[test]
    fn rdf_star_values_ground_triple_term() {
        // A ground triple term in VALUES binds and joins against stored triple terms.
        let g = Graph::load_str("PREFIX : <http://ex/>\n<< :alice :age 30 >> :certainty 0.9 .", "turtle").unwrap();
        let n = |q: &str| crate::query(&g, q).unwrap().len();
        assert_eq!(
            n("PREFIX : <http://ex/> SELECT ?c WHERE { VALUES ?t { <<( :alice :age 30 )>> } \
               ?r <http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies> ?t . ?r :certainty ?c }"),
            1
        );
        assert_eq!(
            n("PREFIX : <http://ex/> SELECT ?c WHERE { VALUES ?t { <<( :alice :age 31 )>> } \
               ?r <http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies> ?t . ?r :certainty ?c }"),
            0
        );
    }

    #[test]
    fn rdf_star_variables_inside_quoted_patterns() {
        // F14: variables inside quoted-triple patterns MATCH structurally against the
        // stored triple terms, binding the inner variables.
        let g = Graph::load_str(
            "PREFIX : <http://ex/>\n<< :alice :age 30 >> :certainty 0.9 .\n<< :bob :age 25 >> :certainty 0.4 .\n:alice :name \"Alice\" .",
            "turtle",
        )
        .unwrap();
        let q = |s: &str| crate::query(&g, s).unwrap();
        // Var in the quoted subject slot.
        let r = q("PREFIX : <http://ex/> SELECT ?w ?c WHERE { << ?w :age 30 >> :certainty ?c }");
        assert_eq!(r.rows.len(), 1);
        assert!(r.rows[0][0].as_ref().unwrap().to_string().contains("alice"));
        // All slots variable: both reified statements match.
        assert_eq!(q("PREFIX : <http://ex/> SELECT * WHERE { << ?s ?p ?o >> :certainty ?c }").rows.len(), 2);
        // Inner variable JOINS with an outer pattern (alice has a name, bob does not).
        assert_eq!(
            q("PREFIX : <http://ex/> SELECT ?n WHERE { << ?w :age ?a >> :certainty ?c . ?w :name ?n }").rows.len(),
            1
        );
        // A ground component inside the quoted pattern constrains the match.
        assert_eq!(q("PREFIX : <http://ex/> SELECT ?c WHERE { << :bob :age ?a >> :certainty ?c }").rows.len(), 1);
        // No match: nothing reifies :alice :age 31.
        assert_eq!(q("PREFIX : <http://ex/> SELECT ?c WHERE { << :alice :age 31 >> :certainty ?c }").rows.len(), 0);
    }

    #[test]
    fn rdf_star_triple_builtins() {
        // F15: TRIPLE / isTRIPLE / SUBJECT / PREDICATE / OBJECT.
        let g = Graph::load_str("PREFIX : <http://ex/>\n<< :alice :age 30 >> :certainty 0.9 .", "turtle").unwrap();
        let one = |s: &str| {
            let r = crate::query(&g, s).unwrap();
            r.rows[0][0].as_ref().map(|t| t.to_string())
        };
        assert_eq!(
            one("PREFIX : <http://ex/> SELECT (TRIPLE(:a, :b, 1) AS ?t) {}").unwrap(),
            "<<( <http://ex/a> <http://ex/b> \"1\"^^<http://www.w3.org/2001/XMLSchema#integer> )>>"
        );
        assert_eq!(one("PREFIX : <http://ex/> SELECT (isTRIPLE(TRIPLE(:a, :b, :c)) AS ?x) {}").unwrap(), "\"true\"^^<http://www.w3.org/2001/XMLSchema#boolean>");
        assert_eq!(one("PREFIX : <http://ex/> SELECT (SUBJECT(TRIPLE(:a, :b, :c)) AS ?x) {}").unwrap(), "<http://ex/a>");
        assert_eq!(one("PREFIX : <http://ex/> SELECT (PREDICATE(TRIPLE(:a, :b, :c)) AS ?x) {}").unwrap(), "<http://ex/b>");
        assert_eq!(one("PREFIX : <http://ex/> SELECT (OBJECT(TRIPLE(:a, :b, :c)) AS ?x) {}").unwrap(), "<http://ex/c>");
        // A literal subject is a type error -> unbound.
        assert_eq!(one("PREFIX : <http://ex/> SELECT (TRIPLE(1, :b, :c) AS ?t) {}"), None);
    }
}

#[cfg(test)]
mod path_pushdown_tests {
    //! Differential tests for the bound-endpoint pushdown: for every operator ×
    //! every binding shape (subject bound / object bound / both / neither /
    //! same-variable), the pushed-down evaluation must agree with the
    //! full-relation evaluation filtered after the fact — over random graphs
    //! containing cycles, diamonds, self-loops and disconnected components.
    use super::*;

    /// Random multigraph over predicates `:e` / `:f`, plus deterministic
    /// adversarial shapes: a diamond (d0→{d1,d2}→d3), a directed 3-cycle
    /// (c0→c1→c2→c0), a self-loop (s0→s0) and a disconnected island (i0→i1).
    fn random_graph(seed0: u64, n_nodes: u32, n_edges: usize) -> Graph {
        let mut seed = seed0;
        let mut next = || {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (seed >> 33) as u32
        };
        let mut ttl = String::from("@prefix : <http://ex/> .\n");
        for _ in 0..n_edges {
            let (a, b) = (next() % n_nodes, next() % n_nodes);
            let p = if next() % 3 == 0 { "f" } else { "e" };
            ttl.push_str(&format!(":n{a} :{p} :n{b} .\n"));
        }
        ttl.push_str(":d0 :e :d1 . :d0 :e :d2 . :d1 :e :d3 . :d2 :e :d3 .\n");
        ttl.push_str(":c0 :e :c1 . :c1 :e :c2 . :c2 :e :c0 . :s0 :e :s0 .\n");
        ttl.push_str(":i0 :f :i1 .\n");
        Graph::load_str(&ttl, "turtle").unwrap()
    }

    /// Sorted row-strings of a result (paths have set semantics, order-free).
    fn rowset(r: &crate::QueryResult) -> Vec<String> {
        let mut v: Vec<String> = r
            .rows
            .iter()
            .map(|row| {
                row.iter()
                    .map(|t| t.as_ref().map(|t| t.to_string()).unwrap_or_default())
                    .collect::<Vec<_>>()
                    .join("\t")
            })
            .collect();
        v.sort();
        v.dedup();
        v
    }

    const PFX: &str = "PREFIX : <http://ex/> ";

    /// Every path operator and the compositions whose pushdown rules differ
    /// (inverse-of-recursive, recursive inside sequence on either side,
    /// alternative-of-recursive, negated sets, zero-length operators).
    const PATHS: &[&str] = &[
        ":e",
        "^:e",
        ":e+",
        ":e*",
        ":e?",
        "^:e+",
        "(^:e)+",
        ":e/:f",
        ":e/:f+",
        ":e+/:f",
        ":e+/:f+",
        ":e|:f",
        "(:e|:f)+",
        "(:e|^:f)*",
        "!:e",
        "!(:e|:f)",
        "^(:e/:f)",
        ":e*/:f?",
    ];

    /// Endpoints sampled from every adversarial shape (all exist in each graph).
    const NODES: &[&str] = &["n0", "n1", "n2", "d0", "d3", "c0", "c1", "s0", "i0", "i1"];

    #[test]
    fn pushdown_matches_full_closure_for_all_operators_and_binding_shapes() {
        for seed in [0x1111u64, 0xACE1, 0xDEADBEEF] {
            let g = random_graph(seed, 12, 40);
            for path in PATHS {
                // Reference: the full (both-unbound) relation, post-filtered here.
                let full = rowset(&crate::query(&g, &format!("{PFX}SELECT ?x ?y WHERE {{ ?x {path} ?y }}")).unwrap());
                let pairs: Vec<(String, String)> = full
                    .iter()
                    .map(|r| {
                        let (x, y) = r.split_once('\t').unwrap();
                        (x.to_string(), y.to_string())
                    })
                    .collect();
                // Same variable at both ends: the diagonal.
                let diag = rowset(&crate::query(&g, &format!("{PFX}SELECT ?x WHERE {{ ?x {path} ?x }}")).unwrap());
                let mut want_diag: Vec<String> =
                    pairs.iter().filter(|(x, y)| x == y).map(|(x, _)| x.clone()).collect();
                want_diag.sort();
                want_diag.dedup();
                assert_eq!(diag, want_diag, "same-var disagrees for `{path}` (seed {seed:#x})");
                for node in NODES {
                    let iri = format!("<http://ex/{node}>");
                    // Subject bound.
                    let got =
                        rowset(&crate::query(&g, &format!("{PFX}SELECT ?y WHERE {{ :{node} {path} ?y }}")).unwrap());
                    let mut want: Vec<String> =
                        pairs.iter().filter(|(x, _)| *x == iri).map(|(_, y)| y.clone()).collect();
                    want.sort();
                    want.dedup();
                    assert_eq!(got, want, "bound-subject :{node} disagrees for `{path}` (seed {seed:#x})");
                    // Object bound.
                    let got =
                        rowset(&crate::query(&g, &format!("{PFX}SELECT ?x WHERE {{ ?x {path} :{node} }}")).unwrap());
                    let mut want: Vec<String> =
                        pairs.iter().filter(|(_, y)| *y == iri).map(|(x, _)| x.clone()).collect();
                    want.sort();
                    want.dedup();
                    assert_eq!(got, want, "bound-object :{node} disagrees for `{path}` (seed {seed:#x})");
                }
                // Both bound: membership must match the full relation exactly.
                for (a, b) in
                    [("n0", "n1"), ("d0", "d3"), ("c0", "c0"), ("c0", "c2"), ("s0", "s0"), ("i0", "i1"), ("n0", "i0")]
                {
                    let row = format!("<http://ex/{a}>\t<http://ex/{b}>");
                    let got = crate::query(&g, &format!("{PFX}SELECT * WHERE {{ :{a} {path} :{b} }}")).unwrap();
                    assert_eq!(
                        !got.rows.is_empty(),
                        full.contains(&row),
                        "both-bound :{a}/:{b} disagrees for `{path}` (seed {seed:#x})"
                    );
                }
            }
        }
    }

    /// The negated-property-set block-skip scan must agree with an INDEPENDENT
    /// evaluation path: a plain scan + FILTER on the predicate.
    #[test]
    fn negated_property_set_matches_filter_oracle() {
        for seed in [0x5EEDu64, 0xBEEF] {
            let g = random_graph(seed, 10, 60);
            for (path, cond) in [("!:e", "?p != :e"), ("!(:e|:f)", "?p != :e && ?p != :f")] {
                let got = rowset(&crate::query(&g, &format!("{PFX}SELECT ?x ?y WHERE {{ ?x {path} ?y }}")).unwrap());
                let oracle = rowset(
                    &crate::query(
                        &g,
                        &format!("{PFX}SELECT DISTINCT ?x ?y WHERE {{ ?x ?p ?y FILTER({cond}) }}"),
                    )
                    .unwrap(),
                );
                assert_eq!(got, oracle, "`{path}` disagrees with filter oracle (seed {seed:#x})");
            }
        }
    }

    /// The row budget must fire INSIDE a single-source traversal (not only
    /// between traversal roots): one long chain, one start node, tiny budget.
    #[test]
    fn budget_fires_inside_directed_traversal() {
        let mut ttl = String::from("@prefix : <http://ex/> .\n");
        for k in 0..20_000u32 {
            ttl.push_str(&format!(":m{k} :e :m{} .\n", k + 1));
        }
        let g = Graph::load_str(&ttl, "turtle").unwrap();
        let budget = crate::QueryBudget { deadline: None, max_rows: Some(64) };
        let err = crate::query_with_budget(&g, "PREFIX : <http://ex/> SELECT ?y WHERE { :m0 :e+ ?y }", &budget)
            .unwrap_err();
        assert!(err.contains("budget"), "expected a budget error, got: {err}");
    }

    /// PAIRED micro-benchmark (same process, same graph) for bound-subject
    /// `knows+` at ~1M edges: the single-source pushdown vs the previous
    /// full-closure-then-filter algorithm. Clustered social graph — 100k
    /// people in 1000 clusters of 100, ten random in-cluster `knows` edges
    /// each — so the full closure is feasible to measure at all (~10M pairs;
    /// an unclustered 1M-edge graph would make "before" run for hours, which
    /// is the pathology being fixed). Run manually:
    ///   cargo test -p sparq-engine --release bench_bound_endpoint -- --ignored --nocapture
    /// Absolute times on a shared machine are contended; the RATIO is the result.
    #[test]
    #[ignore]
    fn bench_bound_endpoint_pushdown_1m_edges() {
        use std::time::Instant;
        const CLUSTERS: u32 = 1_000;
        const SIZE: u32 = 100;
        const OUT_DEG: u32 = 10;
        let mut seed = 0x5EEDu64;
        let mut next = || {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (seed >> 33) as u32
        };
        let mut ttl = String::with_capacity(48 << 20);
        ttl.push_str("@prefix : <http://ex/> .\n");
        for c in 0..CLUSTERS {
            for m in 0..SIZE {
                let a = c * SIZE + m;
                for _ in 0..OUT_DEG {
                    let b = c * SIZE + next() % SIZE;
                    ttl.push_str(&format!(":p{a} :knows :p{b} .\n"));
                }
            }
        }
        let t = Instant::now();
        let g = Graph::load_str(&ttl, "turtle").unwrap();
        eprintln!("loaded {} triples in {:.2?}", g.store.len(), t.elapsed());

        let knows = PropertyPathExpression::NamedNode(oxrdf::NamedNode::new("http://ex/knows").unwrap());
        let plus = PropertyPathExpression::OneOrMore(Box::new(knows.clone()));
        let id = |k: u32| g.id_of(&Term::NamedNode(oxrdf::NamedNode::new(format!("http://ex/p{k}")).unwrap())).unwrap();

        // AFTER: bound-subject pushdown, median over many random starts.
        let starts: Vec<Id> = (0..200).map(|_| id(next() % (CLUSTERS * SIZE))).collect();
        let mut times: Vec<f64> = Vec::new();
        let mut rows = 0usize;
        for &s in &starts {
            let t = Instant::now();
            let r = path_pairs(&g, &plus, PathEnds { s: Some(s), o: None }).unwrap();
            times.push(t.elapsed().as_secs_f64());
            rows += r.len();
        }
        times.sort_by(f64::total_cmp);
        let after = times[times.len() / 2];
        eprintln!("bound-subject pushdown: median {:.3?} over {} starts ({} total rows)",
            std::time::Duration::from_secs_f64(after), starts.len(), rows);

        // BEFORE: the previous algorithm — full all-pairs closure, then filter.
        let t = Instant::now();
        let full = transitive_closure_pairs(path_pairs(&g, &knows, PathEnds::NONE).unwrap());
        let before = t.elapsed().as_secs_f64();
        let s0 = starts[0];
        let filtered = full.iter().filter(|&&(x, _)| x == s0).count();
        eprintln!("full closure (old algorithm): {:.3?} ({} pairs; {} for one start)",
            std::time::Duration::from_secs_f64(before), full.len(), filtered);
        eprintln!("PAIRED RATIO full-closure / single-source = {:.0}x", before / after);

        // Both-bound reachability with early exit (hit and miss), and bound object.
        let (a, hit, miss) = (id(0), id(SIZE - 1), id(SIZE)); // same cluster / next cluster
        let t = Instant::now();
        for _ in 0..100 {
            let _ = path_pairs(&g, &plus, PathEnds { s: Some(a), o: Some(hit) }).unwrap();
        }
        eprintln!("both-bound (hit, early exit): {:.3?}/iter", t.elapsed() / 100);
        let t = Instant::now();
        for _ in 0..100 {
            let r = path_pairs(&g, &plus, PathEnds { s: Some(a), o: Some(miss) }).unwrap();
            assert!(!r.iter().any(|&(_, y)| y == miss));
        }
        eprintln!("both-bound (miss, cluster exhausted): {:.3?}/iter", t.elapsed() / 100);
        let t = Instant::now();
        for _ in 0..100 {
            let _ = path_pairs(&g, &plus, PathEnds { s: None, o: Some(a) }).unwrap();
        }
        eprintln!("bound-object reverse traversal: {:.3?}/iter", t.elapsed() / 100);
    }
}
