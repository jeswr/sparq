//! Physical execution: BGP via greedy-ordered merge/hash joins over the
//! permutation indexes, plus OPTIONAL / UNION / MINUS / BIND / VALUES,
//! aggregation (GROUP BY / HAVING via Filter), ORDER BY, sub-SELECT and the
//! solution modifiers. All intermediate results are id-level (`Bindings`);
//! values computed at query time (BIND, aggregates) get ids in a per-query
//! `LocalVocab`, mirroring QLever's local vocabulary.

use crate::QueryResult;
use oxrdf::vocab::xsd;
use oxrdf::{BlankNode, Literal, Term, Variable};
use rustc_hash::FxHashMap;
use sparq_core::dict::{self, Id, NO_ID};
use sparq_core::store::Pattern as IdPattern;
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
}

impl LocalVocab {
    /// Interns a term, returning a stable id: equal terms get the same id so
    /// DISTINCT, GROUP BY, joins and equality work on computed values.
    fn intern(&mut self, t: Term) -> Id {
        if let Some(&id) = self.ids.get(&t) {
            return id;
        }
        let id = LOCAL_BASE + self.terms.len() as Id;
        self.terms.push(t.clone());
        self.ids.insert(t, id);
        id
    }
    fn term(&self, id: Id) -> &Term {
        &self.terms[(id - LOCAL_BASE) as usize]
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
    let filt: Option<(usize, NumCmp)> = pat_filters[0];

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
            Some((fpos, cmp)) => graph.numeric_value(scan.to_spo(row)[fpos]).is_some_and(|x| cmp.test(x)),
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
    #[cfg(feature = "parallel")]
    if scan_rows.len() >= PAR_THRESHOLD {
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
fn distinct_pattern_vars(pos_vars: &[Option<Variable>; 3]) -> bool {
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
    let mut cmps: Vec<(usize, NumCmp)> = Vec::with_capacity(filters.len());
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

    // General: scan once, count rows passing ALL comparisons via the numeric cache.
    // No solution row is built.
    let scan = graph.store.scan(id_pat);
    let total = scan
        .rows
        .iter()
        .filter(|row| {
            let spo = scan.to_spo(row);
            cmps.iter().all(|(pos, cmp)| graph.numeric_value(spo[*pos]).map(|v| cmp.test(v)).unwrap_or(false))
        })
        .count();
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
    fn eval_translated(graph: &Graph, local: &mut LocalVocab, sub: &Graph, inner: &GraphPattern) -> Result<Bindings, String> {
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
            match graph.named.iter().find(|(t, _)| *t == target) {
                Some((_, sub)) => eval_translated(graph, local, sub, inner),
                // The named graph is absent → ZERO solutions (even for `GRAPH <g> {}`,
                // which must NOT yield the unit row), but with `inner`'s variable
                // schema — evaluate against an empty graph for the columns, then drop
                // any rows (an empty group pattern would otherwise produce one).
                None => {
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
                let mut b = eval_translated(graph, local, sub, inner)?;
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

fn eval_graph_pattern(graph: &Graph, local: &mut LocalVocab, p: &GraphPattern) -> Result<Bindings, String> {
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
            let l = eval_graph_pattern(graph, local, left)?;
            let r = eval_graph_pattern(graph, local, right)?;
            left_outer_join(graph, local, l, r, expression.as_ref())
        }
        GraphPattern::Union { left, right } => {
            let l = eval_graph_pattern(graph, local, left)?;
            let r = eval_graph_pattern(graph, local, right)?;
            Ok(union_bindings(l, r))
        }
        GraphPattern::Extend { inner, variable, expression } => {
            let b = eval_graph_pattern(graph, local, inner)?;
            extend_bindings(graph, local, b, variable, expression)
        }
        GraphPattern::Minus { left, right } => {
            let l = eval_graph_pattern(graph, local, left)?;
            let r = eval_graph_pattern(graph, local, right)?;
            Ok(minus_bindings(l, r))
        }
        GraphPattern::Values { variables, bindings } => Ok(values_bindings(graph, local, variables, bindings)),
        GraphPattern::Path { subject, path, object } => eval_path(graph, local, subject, path, object),
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

fn is_conjunctive(p: &GraphPattern) -> bool {
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

fn flatten_conjunction(p: &GraphPattern, patterns: &mut Vec<TriplePattern>, filters: &mut Vec<Expression>) {
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
enum NumCmp {
    Gt(f64),
    Ge(f64),
    Lt(f64),
    Le(f64),
    Eq(f64),
}

impl NumCmp {
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

/// The inclusive range of inline-integer *values* `[lo, hi]` (within `[0, INLINE_MAX]`)
/// that satisfy the comparison, or `None` if no integer can. Used to range-prune a
/// scan whose filter column holds inline integers (which sort by value).
fn inline_pass_values(cmp: NumCmp) -> Option<(u32, u32)> {
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

/// Recognises a FILTER of the form `?v OP numeric-constant` (or the symmetric
/// `constant OP ?v`), returning the variable and the comparison to push down.
fn extract_sargable(e: &Expression) -> Option<(Variable, NumCmp)> {
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
    fn var_of(e: &Expression) -> Option<Variable> {
        match e {
            Expression::Variable(v) => Some(v.clone()),
            _ => None,
        }
    }
    // (left, right, cmp-if-var-on-left, cmp-if-var-on-right)
    let (l, r, on_left, on_right): (&Expression, &Expression, fn(f64) -> NumCmp, fn(f64) -> NumCmp) = match e {
        Expression::Greater(l, r) => (l, r, NumCmp::Gt, NumCmp::Lt),
        Expression::GreaterOrEqual(l, r) => (l, r, NumCmp::Ge, NumCmp::Le),
        Expression::Less(l, r) => (l, r, NumCmp::Lt, NumCmp::Gt),
        Expression::LessOrEqual(l, r) => (l, r, NumCmp::Le, NumCmp::Ge),
        Expression::Equal(l, r) => (l, r, NumCmp::Eq, NumCmp::Eq),
        _ => return None,
    };
    if let (Some(v), Some(c)) = (var_of(l), lit_num(r)) {
        return Some((v, on_left(c)));
    }
    if let (Some(c), Some(v)) = (lit_num(l), var_of(r)) {
        return Some((v, on_right(c)));
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
fn split_sargable(patterns: &[TriplePattern], filters: &[Expression]) -> (Vec<Option<(usize, NumCmp)>>, Vec<Expression>) {
    let mut pat_filters: Vec<Option<(usize, NumCmp)>> = vec![None; patterns.len()];
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
/// bound or repeated endpoint. Correctness-first: the relation is materialised (the common
/// bound-endpoint case is cheap; a both-unbound `*` over a huge graph is expensive — a future
/// optimisation can push a bound endpoint into a directed traversal).
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
    if !(matches!(s_end, End::Missing(_)) || matches!(o_end, End::Missing(_))) {
        for (s, o) in path_pairs(graph, path)? {
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

/// All `(subject, object)` id pairs connected by a property path expression.
fn path_pairs(graph: &Graph, path: &PropertyPathExpression) -> Result<FxHashSet<(Id, Id)>, String> {
    use PropertyPathExpression as P;
    Ok(match path {
        P::NamedNode(p) => predicate_pairs(graph, p),
        P::Reverse(a) => path_pairs(graph, a)?.into_iter().map(|(s, o)| (o, s)).collect(),
        P::Sequence(a, c) => {
            let (av, cv) = (path_pairs(graph, a)?, path_pairs(graph, c)?);
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
        P::Alternative(a, c) => {
            let mut s = path_pairs(graph, a)?;
            s.extend(path_pairs(graph, c)?);
            s
        }
        P::OneOrMore(a) => transitive_closure_pairs(path_pairs(graph, a)?),
        P::ZeroOrMore(a) => {
            let mut c = transitive_closure_pairs(path_pairs(graph, a)?);
            c.extend(graph_nodes(graph).into_iter().map(|n| (n, n)));
            c
        }
        P::ZeroOrOne(a) => {
            let mut s = path_pairs(graph, a)?;
            s.extend(graph_nodes(graph).into_iter().map(|n| (n, n)));
            s
        }
        P::NegatedPropertySet(props) => {
            let excluded: FxHashSet<Id> =
                props.iter().filter_map(|p| graph.id_of(&Term::NamedNode(p.clone()))).collect();
            let pat: IdPattern = [None, None, None];
            let scan = graph.store.scan(&pat);
            scan.rows
                .iter()
                .filter_map(|r| {
                    let t = scan.to_spo(r);
                    (!excluded.contains(&t[1])).then_some((t[0], t[2]))
                })
                .collect()
        }
    })
}

/// All `(s, o)` for a single predicate IRI (empty if the predicate isn't in the graph).
fn predicate_pairs(graph: &Graph, p: &oxrdf::NamedNode) -> FxHashSet<(Id, Id)> {
    match graph.id_of(&Term::NamedNode(p.clone())) {
        None => FxHashSet::default(),
        Some(pid) => {
            let pat: IdPattern = [None, Some(pid), None];
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
    for start in starts {
        // Coarse budget check once per BFS start node (sticky; the caller's next
        // check raises the error).
        if budget::exhausted(out.len()) {
            break;
        }
        let mut seen: FxHashSet<Id> = FxHashSet::default();
        let mut stack: Vec<Id> = adj.get(&start).cloned().unwrap_or_default();
        while let Some(n) = stack.pop() {
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
    if patterns.len() >= 3 && bgp_is_cyclic(patterns) {
        return eval_bgp_wcoj(graph, patterns);
    }
    eval_bgp_binary(graph, patterns, &[])
}

/// Whether this conjunctive BGP would be routed to the worst-case-optimal plan
/// (so the caller knows whether sargable-filter pushdown into the binary scan
/// applies).
fn bgp_uses_binary(patterns: &[TriplePattern]) -> bool {
    !(patterns.len() >= 3 && bgp_is_cyclic(patterns))
}

/// Binary-join BGP plan: greedy cardinality ordering with sort-merge joins on the
/// current sort variable (falling back to hash, then cross product). `pat_filters`
/// holds an optional pushed-down numeric FILTER per pattern (by original index).
fn eval_bgp_binary(graph: &Graph, patterns: &[TriplePattern], pat_filters: &[Option<(usize, NumCmp)>]) -> Result<Bindings, String> {
    if patterns.is_empty() {
        return Ok(Bindings { vars: vec![], rows: vec![Row::new()], sorted_by: None });
    }
    let pfilter = |i: usize| -> Option<(usize, NumCmp)> { pat_filters.get(i).copied().flatten() };

    struct Prepared {
        id_pat: IdPattern,
        pos_vars: [Option<Variable>; 3],
        est: usize,
        unsatisfiable: bool,
    }
    let mut prepared: Vec<Prepared> = Vec::with_capacity(patterns.len());
    for tp in patterns {
        let (id_pat, pos_vars, unsat) = prepare_pattern(graph, tp)?;
        let est = if unsat { 0 } else { graph.store.estimate(&id_pat) };
        prepared.push(Prepared { id_pat, pos_vars, est, unsatisfiable: unsat });
    }
    if prepared.iter().any(|p| p.unsatisfiable) {
        return Ok(Bindings::unsorted(collect_vars(patterns), vec![]));
    }

    let var_pos = |i: usize, v: &Variable| -> Option<usize> {
        prepared[i].pos_vars.iter().position(|pv| pv.as_ref() == Some(v))
    };

    // Cost-based greedy (GOO): seed with the smallest single-pattern cardinality,
    // then repeatedly add the connected pattern that yields the smallest *estimated
    // join result*, using the per-predicate characteristic stats (distinct
    // subjects/objects) to estimate join selectivity. The join order only affects
    // performance (the result is identical for any order — differentially tested).
    let seed = (0..prepared.len()).min_by_key(|&i| prepared[i].est).unwrap();
    let seed_join_var = prepared[seed]
        .pos_vars
        .iter()
        .flatten()
        .find(|v| (0..prepared.len()).any(|j| j != seed && var_pos(j, v).is_some()))
        .cloned();
    // A pushed-down filter scans in its own column's order (sequential numeric
    // access); otherwise sort by the join variable to enable a merge join.
    let seed_sort_col = pfilter(seed).map(|(c, _)| c).or_else(|| seed_join_var.as_ref().and_then(|v| var_pos(seed, v)));

    let mut result = scan_to_bindings(graph, &prepared[seed].id_pat, &prepared[seed].pos_vars, seed_sort_col, pfilter(seed), None);
    let mut done = vec![false; prepared.len()];
    done[seed] = true;

    // Running estimate of the result cardinality and the per-variable distinct
    // count (ndv), used to score the next join.
    let mut cur_card = prepared[seed].est as f64;
    let mut var_ndv: FxHashMap<Variable, f64> = FxHashMap::default();
    let record_vars = |i: usize, cur_card: f64, var_ndv: &mut FxHashMap<Variable, f64>| {
        for (pos, ov) in prepared[i].pos_vars.iter().enumerate() {
            if let Some(v) = ov {
                let ndv = pattern_var_ndv(graph, &prepared[i].id_pat, pos, prepared[i].est).min(cur_card.max(1.0));
                let e = var_ndv.entry(v.clone()).or_insert(ndv);
                *e = e.min(ndv);
            }
        }
    };
    record_vars(seed, cur_card, &mut var_ndv);

    for _ in 1..prepared.len() {
        // Pick the connected candidate with the smallest estimated output.
        let mut best: Option<(usize, f64)> = None;
        for i in 0..prepared.len() {
            if done[i] {
                continue;
            }
            let mut sel = 1.0f64;
            let mut shared = false;
            for (pos, ov) in prepared[i].pos_vars.iter().enumerate() {
                if let Some(v) = ov {
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
            let out = cur_card * prepared[i].est as f64 * sel;
            if best.map_or(true, |(_, bc)| out < bc) {
                best = Some((i, out));
            }
        }
        let i = match best {
            Some((i, out)) => {
                cur_card = out.max(0.0);
                i
            }
            None => {
                // Disconnected: smallest-cardinality remaining (cross product).
                let i = (0..prepared.len()).filter(|&j| !done[j]).min_by_key(|&j| prepared[j].est).unwrap();
                cur_card *= prepared[i].est as f64;
                i
            }
        };
        done[i] = true;

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
            record_vars(i, cur_card, &mut var_ndv);
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
        record_vars(i, cur_card, &mut var_ndv);

        if result.rows.is_empty() {
            break;
        }
    }
    Ok(result)
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

fn collect_vars(patterns: &[TriplePattern]) -> Vec<Variable> {
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
            tuples.push(tup);
        }
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
            return Ok(Bindings::unsorted(collect_vars(patterns), vec![]));
        }
        prepared.push((id_pat, pos_vars));
    }

    // Global variable order: most-constrained first (highest degree, then most
    // selective). Constant-only patterns contribute no variables but must match.
    let all_vars = collect_vars(patterns);
    let degree = |v: &Variable| prepared.iter().filter(|(_, pv)| pv.iter().flatten().any(|x| x == v)).count();
    let min_est = |v: &Variable| {
        prepared
            .iter()
            .filter(|(_, pv)| pv.iter().flatten().any(|x| x == v))
            .map(|(ip, _)| graph.store.estimate(ip))
            .min()
            .unwrap_or(usize::MAX)
    };
    let mut order_vars = all_vars.clone();
    order_vars.sort_by(|a, b| degree(b).cmp(&degree(a)).then(min_est(a).cmp(&min_est(b))));
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

fn scan_to_bindings(graph: &Graph, id_pat: &IdPattern, pos_vars: &[Option<Variable>; 3], sort_col: Option<usize>, filter: Option<(usize, NumCmp)>, limit: Option<usize>) -> Bindings {
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
            if !graph.numeric_value(spo[fpos]).is_some_and(|x| cmp.test(x)) {
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
                for li in i..i2 {
                    for rj in j..j2 {
                        if extra_shared.iter().all(|&(lc, rc)| l[li][lc] == r[rj][rc]) {
                            let mut row = l[li].clone();
                            for &rc in &right_only {
                                row.push(r[rj][rc]);
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
    filt: Option<(usize, NumCmp)>,
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
                if !graph.numeric_value(pspo[fpos]).is_some_and(|x| cmp.test(x)) {
                    continue;
                }
            }
            let new_vals: SmallVec<[Id; 4]> = new_positions.iter().map(|&p| pspo[p]).collect();
            for &ri in &ris {
                let mut combined = result.rows[ri].clone();
                combined.extend(new_vals.iter().copied());
                out_rows.push(combined);
            }
        }
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
            combined.extend(std::iter::repeat(NO_ID).take(n_right_only));
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
        for li in i..i2 {
            let mut matched = false;
            for rj in j..j2 {
                let combined = merge_rows(&l[li], &r[rj], shared, right_only);
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
                let mut combined = l[li].clone();
                combined.extend(std::iter::repeat(NO_ID).take(n_right_only));
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
        b.rows
            .par_iter()
            .enumerate()
            .map(|(i, row)| {
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
    // every group) in parallel: it is read-only (`&LocalVocab`) and independent across groups.
    // Interning the results (`value_to_id`, needs `&mut LocalVocab`) stays serial and in `order`,
    // so the output is byte-identical to the serial path.
    #[cfg(feature = "parallel")]
    let agg_values: Vec<Vec<Value>> = if b.rows.len() >= PAR_THRESHOLD {
        use rayon::prelude::*;
        let lv: &LocalVocab = local; // immutable reborrow for the read-only parallel phase
        let bref = &b;
        members
            .par_iter()
            .map(|members| {
                aggregates
                    .iter()
                    .map(|(_, agg)| eval_aggregate(graph, lv, bref, members, agg))
                    .collect::<Result<Vec<_>, String>>()
            })
            .collect::<Result<Vec<_>, String>>()?
    } else {
        members
            .iter()
            .map(|members| {
                aggregates
                    .iter()
                    .map(|(_, agg)| eval_aggregate(graph, local, &b, members, agg))
                    .collect::<Result<Vec<_>, String>>()
            })
            .collect::<Result<Vec<_>, String>>()?
    };
    #[cfg(not(feature = "parallel"))]
    let agg_values: Vec<Vec<Value>> = members
        .iter()
        .map(|members| {
            aggregates
                .iter()
                .map(|(_, agg)| eval_aggregate(graph, local, &b, members, agg))
                .collect::<Result<Vec<_>, String>>()
        })
        .collect::<Result<Vec<_>, String>>()?;
    // Resolve the aggregate values to ids: a PARALLEL read-only pass (inline integers, graph-dict
    // and already-local terms — the vast majority) and then a serial intern of only the genuinely
    // new terms, in order, so ids are byte-identical to the fully-serial path (T1.0b).
    #[cfg(feature = "parallel")]
    let resolved: Vec<Vec<Result<Id, Term>>> = if b.rows.len() >= PAR_THRESHOLD {
        use rayon::prelude::*;
        let lv: &LocalVocab = local;
        agg_values
            .par_iter()
            .map(|vals| vals.iter().map(|v| value_to_id_readonly(graph, lv, v)).collect())
            .collect()
    } else {
        agg_values.iter().map(|vals| vals.iter().map(|v| value_to_id_readonly(graph, local, v)).collect()).collect()
    };
    #[cfg(not(feature = "parallel"))]
    let resolved: Vec<Vec<Result<Id, Term>>> =
        agg_values.iter().map(|vals| vals.iter().map(|v| value_to_id_readonly(graph, local, v)).collect()).collect();

    let mut rows: Vec<Row> = Vec::with_capacity(order.len());
    for (key, res) in order.iter().zip(resolved) {
        let mut row = Row::from_slice(key);
        for r in res {
            row.push(match r {
                Ok(id) => id,
                Err(term) => local.intern(term),
            });
        }
        rows.push(row);
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
            // Collect the per-member values of `expr`. An unbound / errored member is
            // recorded: SUM and AVG must ERROR on it (yielding an unbound aggregate),
            // while COUNT/MIN/MAX/SAMPLE/GROUP_CONCAT skip it.
            let mut vals: Vec<Value> = Vec::with_capacity(members.len());
            let mut errored = false;
            for &ri in members {
                let v = eval_expr(graph, local, b, &b.rows[ri], expr)?;
                if matches!(v, Value::Unbound | Value::Error) {
                    errored = true;
                } else {
                    vals.push(v);
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

fn order_bindings(graph: &Graph, local: &LocalVocab, b: &mut Bindings, exprs: &[OrderExpression]) -> Result<(), String> {
    // The sort key (vector of (descending, Value)) for one row. Numeric keys use the cache (no
    // per-comparison reparse); other expressions fall back to identity-preserving evaluation.
    let key_of = |row: &Row| -> Result<Vec<(bool, Value)>, String> {
        let mut key = Vec::with_capacity(exprs.len());
        for oe in exprs {
            let (desc, e) = match oe {
                OrderExpression::Asc(e) => (false, e),
                OrderExpression::Desc(e) => (true, e),
            };
            let v = match eval_numeric(graph, local, b, row, e) {
                Some(n) => Value::Num(Num::Double(n)),
                None => eval_expr(graph, local, b, row, e)?,
            };
            key.push((desc, v));
        }
        Ok(key)
    };
    // Precompute the keys (independent, read-only) — in parallel for large result sets.
    #[cfg(feature = "parallel")]
    let mut keyed: Vec<(Vec<(bool, Value)>, Row)> = if b.rows.len() >= PAR_THRESHOLD {
        use rayon::prelude::*;
        b.rows.par_iter().map(|row| Ok((key_of(row)?, row.clone()))).collect::<Result<_, String>>()?
    } else {
        b.rows.iter().map(|row| Ok((key_of(row)?, row.clone()))).collect::<Result<_, String>>()?
    };
    #[cfg(not(feature = "parallel"))]
    let mut keyed: Vec<(Vec<(bool, Value)>, Row)> =
        b.rows.iter().map(|row| Ok((key_of(row)?, row.clone()))).collect::<Result<_, String>>()?;

    let cmp = |a: &(Vec<(bool, Value)>, Row), c: &(Vec<(bool, Value)>, Row)| {
        for ((desc, av), (_, cv)) in a.0.iter().zip(c.0.iter()) {
            let ord = compare_values(av, cv).unwrap_or(Ordering::Equal);
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
        b.rows
            .par_iter()
            .enumerate()
            .map(|(i, row)| {
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
                return Some(!l.value().is_empty());
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
                // Computed values are rare; resolve through the local vocab term.
                as_num(&Value::Term(local.term(id).clone()))
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
                exact_lexical_of_term(&local.term(id))
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
        if self.scale == 0 {
            return self;
        }
        let p = 10i128.pow(self.scale);
        let q = self.mant.div_euclid(p);
        let r = self.mant.rem_euclid(p); // 0..p
        let mant = match mode {
            RoundMode::Floor => q,
            RoundMode::Ceil => q + i128::from(r > 0),
            RoundMode::HalfUp => q + i128::from(r * 2 >= p),
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
                exact_lexical_of_term(&local.term(id)).and_then(|s| Dec::parse(&s))
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

/// An xsd:date / xsd:dateTime VALUE: seconds-from-epoch of the local time, fractional
/// seconds, and the timezone offset when present. Comparison follows XSD: both-with-tz
/// and both-without compare directly; MIXED presence is only decidable outside the
/// ±14h window (inside it the comparison is indeterminate — a SPARQL type error).
#[derive(Clone, Copy, Debug)]
struct Timeline {
    secs: i64,
    frac: f64,
    tz: Option<i64>,
}

impl Timeline {
    fn parse_datetime(s: &str) -> Option<Timeline> {
        let (date, rest) = s.split_once('T')?;
        let (time, tz) = match rest.find(['Z', '+', '-']) {
            Some(i) => (&rest[..i], Some(parse_tz(&rest[i..])?)),
            None => (rest, None),
        };
        let days = parse_civil_date(date)?;
        let mut t = time.split(':');
        let h: i64 = t.next()?.parse().ok()?;
        let mi: i64 = t.next()?.parse().ok()?;
        let sec_lex = t.next()?;
        if t.next().is_some() {
            return None;
        }
        let sec: f64 = sec_lex.parse().ok()?;
        Some(Timeline {
            secs: days * 86_400 + h * 3600 + mi * 60 + sec.trunc() as i64,
            frac: sec.fract(),
            tz,
        })
    }

    fn parse_date(s: &str) -> Option<Timeline> {
        // The timezone suffix starts after the day: "...-23Z" / "...-23+05:00". A bare
        // date's own hyphens must not be mistaken for an offset sign, so require the
        // ":" of "±hh:mm" at the right position.
        let (date, tz) = if let Some(d) = s.strip_suffix('Z') {
            (d, Some(0))
        } else if s.len() > 10 && matches!(s.as_bytes()[s.len() - 6], b'+' | b'-') && s.as_bytes()[s.len() - 3] == b':' {
            (&s[..s.len() - 6], Some(parse_tz(&s[s.len() - 6..])?))
        } else {
            (s, None)
        };
        Some(Timeline { secs: parse_civil_date(date)? * 86_400, frac: 0.0, tz })
    }

    /// The absolute instant (treating an absent timezone as UTC) in seconds.
    fn instant(&self) -> f64 {
        (self.secs - self.tz.unwrap_or(0)) as f64 + self.frac
    }

    fn cmp_tl(a: Timeline, b: Timeline) -> Option<Ordering> {
        let (ai, bi) = (a.instant(), b.instant());
        match (a.tz, b.tz) {
            (Some(_), Some(_)) | (None, None) => ai.partial_cmp(&bi),
            // Mixed timezone presence: indeterminate inside the ±14h window.
            _ => {
                if (ai - bi).abs() > 14.0 * 3600.0 {
                    ai.partial_cmp(&bi)
                } else {
                    None
                }
            }
        }
    }
}

/// `"Z"` / `"±hh:mm"` -> offset seconds.
fn parse_tz(tz: &str) -> Option<i64> {
    if tz == "Z" {
        return Some(0);
    }
    let (sign, hm) = tz.split_at(1);
    let (h, m) = hm.split_once(':')?;
    let (h, m): (i64, i64) = (h.parse().ok()?, m.parse().ok()?);
    let off = h * 3600 + m * 60;
    Some(if sign == "-" { -off } else { off })
}

/// `[-]YYYY-MM-DD` -> days from the epoch (Howard Hinnant's days_from_civil).
fn parse_civil_date(date: &str) -> Option<i64> {
    let neg = date.starts_with('-');
    let mut p = date.strip_prefix('-').unwrap_or(date).split('-');
    let y: i64 = p.next()?.parse().ok()?;
    let y = if neg { -y } else { y };
    let m: i64 = p.next()?.parse().ok()?;
    let d: i64 = p.next()?.parse().ok()?;
    if p.next().is_some() || !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + doe - 719_468)
}

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
        if !matches!(p, Term::Literal(_)) || !matches!(q, Term::Literal(_)) {
            return Some(false); // IRI / bnode / triple term: identity decides
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
        (Str(a), Str(b)) => Some(a.cmp(&b)),
        (Bool(Some(a)), Bool(Some(b))) => Some(a.cmp(&b)),
        (DateTime(Some(a)), DateTime(Some(b))) => Timeline::cmp_tl(a, b),
        (Date(Some(a)), Date(Some(b))) => Timeline::cmp_tl(a, b),
        // Same language tag: compare values (the suites' lenient extension).
        (Lang(t1, v1), Lang(t2, v2)) if t1 == t2 => Some(v1.cmp(&v2)),
        // Same other-XSD datatype: lexical order (correct for time, gYear, …).
        (OtherXsd(d1, l1), OtherXsd(d2, l2)) if d1 == d2 => Some(l1.cmp(&l2)),
        _ => None,
    }
}

/// A numeric value for STRICT comparison — only genuine numeric literals (NOT
/// booleans, which are a separate, incomparable type in SPARQL relational ops).
fn strict_num(v: &Value) -> Option<f64> {
    match v {
        Value::Num(n) => Some(n.f64()),
        Value::Term(Term::Literal(l)) if is_numeric_dt(l) => l.value().parse::<f64>().ok(),
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
        // XSD constructor casts: xsd:integer(?x), xsd:decimal(?x), … (SPARQL 17.5).
        F::Custom(nn) if args.len() == 1 => match eval_cast(nn.as_str(), &ev(0)?) {
            Some(out) => out,
            None => return Err(format!("unsupported SPARQL function: Custom({})", nn.as_str())),
        },
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
fn str_lit(v: &Value) -> Option<(String, Option<String>)> {
    match v {
        Value::Term(Term::Literal(l)) => {
            if let Some(lang) = l.language() {
                Some((l.value().to_string(), Some(lang.to_string())))
            } else if l.datatype() == xsd::STRING {
                Some((l.value().to_string(), None))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// A simple or language-tagged literal value, per the tag the operand carried.
fn lit_with_lang(s: String, lang: Option<&str>) -> Value {
    Value::Term(Term::Literal(match lang {
        Some(l) => Literal::new_language_tagged_literal_unchecked(s, l),
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

/// Converts an evaluated value into an id. Computed terms are resolved against
/// the graph dictionary first (so they join and deduplicate against the data);
/// terms not already present get a per-query local id.
fn value_to_id(graph: &Graph, local: &mut LocalVocab, v: &Value) -> Id {
    let term = match v {
        Value::Unbound | Value::Error => return NO_ID,
        Value::Bool(b) => Term::Literal(Literal::new_typed_literal(b.to_string(), xsd::BOOLEAN)),
        Value::Num(n) => Term::Literal(Literal::new_typed_literal(n.lexical(), n.datatype())),
        Value::Term(t) => t.clone(),
    };
    graph.id_of(&term).unwrap_or_else(|| local.intern(term))
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
        Value::Bool(b) => Term::Literal(Literal::new_typed_literal(b.to_string(), xsd::BOOLEAN)),
        Value::Num(n) => Term::Literal(Literal::new_typed_literal(n.lexical(), n.datatype())),
        Value::Term(t) => t.clone(),
    };
    if let Some(id) = graph.id_of(&term) {
        return Ok(id);
    }
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
    fn rdf_star_variable_inside_quoted_pattern_is_clean_error() {
        // Variables inside a quoted-triple pattern are not yet supported: the query must
        // return a clean Err (no panic, no wrong answer).
        let g = Graph::load_str("PREFIX : <http://ex/>\n<< :alice :age 30 >> :certainty 0.9 .", "turtle").unwrap();
        let r = crate::query(&g, "PREFIX : <http://ex/> SELECT ?w WHERE { << ?w :age 30 >> :certainty ?c }");
        assert!(r.is_err(), "expected a clean error, got Ok with {} rows", r.map(|x| x.rows.len()).unwrap_or(0));
    }
}
