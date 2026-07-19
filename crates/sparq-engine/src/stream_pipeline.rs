//! [FABLE-5] (sq-7d3dj.34.2.3) Streaming SELECT-JSON pipeline v1 (M1): classifier +
//! block driver over the emit sink — first solutions are emitted before the join
//! completes.
//!
//! 🤖 SPARQ agent. Design record: `research/lazy-pull-incremental-emission.md` §5 +
//! §5.1 AS AMENDED by decision `sq-7d3dj.34.2.7` — the original
//! byte-identity-for-bind-chains contract is RETIRED; admitted join chains emit in
//! the deterministic **seed-lex order** instead. This module is the evaluate-phase
//! counterpart of the #1745 serialize seam: `exec::eval_select_json_emit` calls
//! [`try_stream_json_emit`] after the single-pattern scan fast path; on `Ok(None)`
//! the buffered general path runs exactly as before (fail-closed, §8.3).
//!
//! ## Classifier (M1)
//!
//! ```text
//! Streamable ::= [Slice] [Distinct|Reduced] Project (BGP+filters | Union(Streamable-BGP…))
//! ```
//!
//! over conjunctive branches whose GOO join chain — re-derived with the SAME planner
//! calls the executor and the T22 EXPLAIN dry-run use (`goo_seed` / `goo_pick`) — is
//! **seed + bind_join-only**, the bind-join admission inequality evaluated on the
//! ESTIMATE exactly as the EXPLAIN dry-run does. Anything else returns `None`: any
//! predicted hash / merge / cross step, BIND / OPTIONAL / MINUS / property paths /
//! GROUP BY / VALUES / subqueries / named-graph forms / ORDER BY (v1), cyclic
//! (WCOJ) BGPs, RDF 1.2 quoted-triple decompositions, an armed zk-trace recorder
//! (the recorder's scan-completeness contract requires the recording `Bindings`
//! path — no zero-knowledge property is claimed or altered here; the v1 verifier's
//! external cryptographer sign-off remains pending, `sq-qhy4`), and active
//! EXPLAIN-ANALYZE instrumentation. A single-variable `DISTINCT` projection also
//! declines, so the #1747 distinct-projection skip-scan keeps its shapes
//! (complementarity: a shape it covers never reaches this pipeline).
//!
//! ## Driver (§5, amended)
//!
//! The seed scan is consumed in blocks with an adaptive ramp
//! ([`STREAM_BLOCK_FIRST`] doubling to [`STREAM_BLOCK_MAX`]; overridable via the
//! [`testing`] hooks so the harness can force tiny blocks). Each block runs through
//! the bind-join chain with the existing index kernels; **emission order is
//! seed-lex** (§5.1): within a block the per-join-value grouping is used ONLY as a
//! scan cache (one index scan per distinct join value per block, matches cached),
//! then the block's rows are walked IN ORDER, each row's matches appended
//! contiguously in match-scan order — NOT the buffered kernel's hash-bucket
//! group-loop order (`exec::bind_join`; counterexample in `sq-7d3dj.34.2.7`). The
//! order composes through the chain as the lexicographic
//! (seed row, match₁, match₂, …) order and preserves seed sortedness.
//!
//! Residual FILTERs run per block through the exact evaluator (`apply_filter`,
//! which compiles each expression once per block — the #1718 hoist); `DISTINCT`
//! keeps a persistent first-seen hash set across blocks and emits survivors
//! immediately; `REDUCED` streams WITHOUT dedup, mirroring the buffered
//! evaluator's `Reduced`-as-no-op (so its solutions stay multiset-equal to the
//! buffered path, which the §5.1 set-equality contract subsumes); `LIMIT`
//! early-exits the driver the moment it is satisfied; the JSON `head` is built
//! from the statically-known projection before the seed scan starts (it is handed
//! to `emit` at the normal flush boundaries, preserving the server's
//! small-result-stays-single-chunk `Content-Length` property from #1745);
//! `budget::check` runs per block and the sticky flag set by the coarse in-join
//! checks converts to the budget error at the next block boundary — a
//! post-first-byte trip surfaces as a truncated body, never a clean short 200
//! (§8.4). `ControlFlow::Break` from the sink aborts the driver (client gone).
//!
//! ## Contract (§8.1/§8.2 as amended)
//!
//! Admitted shapes emit deterministic seed-lex order with per-shape solution
//! equivalence vs the buffered path: set-equality under `DISTINCT`,
//! multiset-equality otherwise, and under `Slice` a sub-multiset of the UN-sliced
//! buffered result with the buffered sliced count. Everything not provably meeting
//! its contract is not admitted. The classification is independent of `flush`, so
//! every front door of the shared emit core (`query_json`, the chunked `Vec` API
//! and the live sink) stays byte-identical to each other for the same query.
//!
//! Declared as a CHILD module of `exec` (via `#[path]`, the file lives at
//! `src/stream_pipeline.rs` — the design record's reserved name, distinct from the
//! Phase-6 columnar `src/pipeline.rs`) so it reuses the private evaluation
//! machinery — `Bindings`, the GOO planner helpers, `scan_to_bindings`,
//! `apply_filter`, the query budget — without widening any item's visibility (the
//! `eqjoin` precedent). Always compiled: no new cargo feature, no new dependency,
//! sparq-core untouched (§8.5).

use super::*;

/// First seed block of the adaptive ramp (§5): small enough that the first
/// solutions cost microseconds-to-milliseconds of engine work.
pub(crate) const STREAM_BLOCK_FIRST: usize = 4096;

/// Steady-state seed block cap of the adaptive ramp (§5): large enough that the
/// per-block overhead (one budget check + one chain sequence point) amortises to
/// noise.
pub(crate) const STREAM_BLOCK_MAX: usize = 65_536;

/// Runtime toggle + telemetry for the streaming pipeline (the `sip_testing` /
/// `distinct_pushdown_testing` precedent, §8.5): the differential harness forces
/// both paths within one binary, overrides the block ramp to force multi-block
/// runs on small corpora, and reads which path ran. Surfaced publicly through
/// `sparq_engine::stream_pipeline_testing`.
pub(crate) mod testing {
    use std::cell::Cell;

    thread_local! {
        static ENABLED: Cell<bool> = const { Cell::new(true) };
        static RAMP: Cell<(usize, usize)> =
            const { Cell::new((super::STREAM_BLOCK_FIRST, super::STREAM_BLOCK_MAX)) };
        static FIRED: Cell<bool> = const { Cell::new(false) };
        static BLOCKS: Cell<usize> = const { Cell::new(0) };
        static SEED_PROCESSED: Cell<usize> = const { Cell::new(0) };
        static SEED_TOTAL: Cell<usize> = const { Cell::new(0) };
    }

    /// Whether the streaming pipeline is enabled on this thread (default on).
    #[inline]
    pub(crate) fn enabled() -> bool {
        ENABLED.with(|e| e.get())
    }

    /// Enables/disables the pipeline on this thread, returning the previous value.
    pub(crate) fn set_enabled(v: bool) -> bool {
        ENABLED.with(|e| e.replace(v))
    }

    /// Overrides the `(first, max)` seed-block ramp on this thread, returning the
    /// previous pair. Values are clamped to at least 1.
    pub(crate) fn set_block_ramp(first: usize, max: usize) -> (usize, usize) {
        RAMP.with(|r| r.replace((first.max(1), max.max(1))))
    }

    /// The active `(first, max)` seed-block ramp for this thread.
    #[inline]
    pub(crate) fn ramp() -> (usize, usize) {
        RAMP.with(|r| r.get())
    }

    /// Clears the per-query statistics (call before a measured run).
    pub(crate) fn reset_stats() {
        FIRED.with(|f| f.set(false));
        BLOCKS.with(|c| c.set(0));
        SEED_PROCESSED.with(|c| c.set(0));
        SEED_TOTAL.with(|c| c.set(0));
    }

    /// Records that the streamed path was taken for the current query.
    pub(crate) fn mark_fired() {
        FIRED.with(|f| f.set(true));
    }

    /// Records one fully-driven seed block.
    pub(crate) fn add_block() {
        BLOCKS.with(|c| c.set(c.get().saturating_add(1)));
    }

    /// Adds a branch's total seed-row count (recorded BEFORE its block loop runs, so
    /// a mid-drive observer can compare against [`add_seed_processed`]).
    pub(crate) fn add_seed_total(n: usize) {
        SEED_TOTAL.with(|c| c.set(c.get().saturating_add(n)));
    }

    /// Adds a completed block's seed-row count.
    pub(crate) fn add_seed_processed(n: usize) {
        SEED_PROCESSED.with(|c| c.set(c.get().saturating_add(n)));
    }

    /// `(fired, blocks_run, seed_rows_processed, seed_rows_total)` since the last
    /// [`reset_stats`]. Because evaluation is synchronous on the installing thread,
    /// an emit-sink callback can read these MID-STREAM — the
    /// first-emit-before-eval-complete probe observes
    /// `seed_rows_processed < seed_rows_total` at first-chunk time.
    pub(crate) fn stats() -> (bool, usize, usize, usize) {
        (
            FIRED.with(|f| f.get()),
            BLOCKS.with(|c| c.get()),
            SEED_PROCESSED.with(|c| c.get()),
            SEED_TOTAL.with(|c| c.get()),
        )
    }
}

/// One admitted chain step: an index-nested-loop (bind) join of the running rows
/// into pattern `pat` on exactly one shared variable.
struct BindStep {
    /// Index into the branch's `prepared` patterns.
    pat: usize,
    /// Column of the join variable in the running row layout.
    rk: usize,
    /// Canonical position (0=s, 1=p, 2=o) of the join variable in the pattern.
    pp: usize,
}

/// One classified UNION branch: a conjunctive BGP whose GOO chain is
/// seed + bind_join-only, plus its pushed-down and residual filters.
struct BranchPlan {
    prepared: Vec<Prepared>,
    pat_filters: Vec<Option<(usize, ScanCmp)>>,
    residual: Vec<Expression>,
    seed: usize,
    seed_sort_col: Option<usize>,
    steps: Vec<BindStep>,
    /// The running row layout after the whole chain (seed vars, then each step's
    /// non-join variables in canonical order — exactly the executor's layout).
    vars: Vec<Variable>,
    /// A constant term of the branch is absent from the dictionary: zero rows.
    unsat: bool,
}

/// A fully classified streamable plan.
struct StreamPlan {
    branches: Vec<BranchPlan>,
    distinct: bool,
    /// `(OFFSET, LIMIT)` of a top `Slice`.
    slice: Option<(usize, Option<usize>)>,
    /// The projection, minus synthetic blank-node variables — the JSON head.
    out_vars: Vec<Variable>,
}

/// The streaming-pipeline attempt over a SELECT pattern. `Ok(None)` = not admitted
/// (the caller falls back to the buffered path, unchanged); `Ok(Some(()))` = the
/// whole result was serialised through `emit`; `Err` = evaluation failed (a budget
/// trip or FILTER error — chunks may already have been emitted, exactly as the
/// buffered emit path documents). See the module docs for the admission rules.
pub(crate) fn try_stream_json_emit(
    graph: &Graph,
    pattern: &GraphPattern,
    flush: Option<usize>,
    emit: &mut dyn FnMut(String) -> ControlFlow<()>,
) -> Result<Option<()>, String> {
    if !testing::enabled() {
        return Ok(None);
    }
    // zk-trace armed: the recorder's scan-completeness contract needs the recording
    // Bindings path (result-equivalent; only the plan changes). Fail closed.
    #[cfg(feature = "zk")]
    if crate::zk::enabled() {
        return Ok(None);
    }
    // EXPLAIN ANALYZE instrumentation counts operators on the buffered path.
    if trace::enabled() {
        return Ok(None);
    }
    // Empty-default dataset view: the general path short-circuits at the BGP.
    if view::default_is_empty() {
        return Ok(None);
    }
    let Some(plan) = classify(graph, pattern) else {
        return Ok(None);
    };
    testing::mark_fired();
    drive(graph, &plan, flush, emit).map(Some)
}

/// Pattern-matches the top of the plan (`[Slice] [Distinct|Reduced] Project` over
/// conjunctive branches) and classifies every branch; `None` = not admitted.
fn classify(graph: &Graph, pattern: &GraphPattern) -> Option<StreamPlan> {
    let mut p = pattern;
    let mut slice = None;
    if let GraphPattern::Slice { inner, start, length } = p {
        slice = Some((*start, *length));
        p = inner;
    }
    let mut distinct = false;
    match p {
        GraphPattern::Distinct { inner } => {
            distinct = true;
            p = inner;
        }
        // REDUCED is a no-op on the buffered path (`eval_modified`); stream it the
        // same way so the solutions stay multiset-equal (see module docs).
        GraphPattern::Reduced { inner } => p = inner,
        _ => {}
    }
    let GraphPattern::Project { inner, variables } = p else {
        return None;
    };
    // Single-variable DISTINCT projections belong to the #1747 skip-scan
    // (try_distinct_pushdown) — decline so that path keeps its shapes.
    if distinct && variables.len() == 1 {
        return None;
    }
    let mut leaves: Vec<&GraphPattern> = Vec::new();
    collect_union_branches(inner, &mut leaves)?;
    let mut branches = Vec::with_capacity(leaves.len());
    for leaf in leaves {
        branches.push(classify_branch(graph, leaf)?);
    }
    let out_vars: Vec<Variable> = variables
        .iter()
        .filter(|v| !v.as_str().starts_with(BNODE_VAR_PREFIX))
        .cloned()
        .collect();
    Some(StreamPlan { branches, distinct, slice, out_vars })
}

/// Flattens (possibly nested) `Union`s into conjunctive leaves; `None` when any
/// leaf is not a flattenable BGP(+filters) conjunction.
fn collect_union_branches<'a>(p: &'a GraphPattern, out: &mut Vec<&'a GraphPattern>) -> Option<()> {
    match p {
        GraphPattern::Union { left, right } => {
            collect_union_branches(left, out)?;
            collect_union_branches(right, out)
        }
        leaf if is_conjunctive(leaf) => {
            out.push(leaf);
            Some(())
        }
        _ => None,
    }
}

/// Classifies one conjunctive branch: re-derives the GOO chain with the executor's
/// own planner calls and admits it only when every step is a bind join — the
/// admission inequality evaluated on the ESTIMATE, exactly as the T22 EXPLAIN
/// dry-run predicts the executor's run-time cutover. `None` = not admitted.
fn classify_branch(graph: &Graph, p: &GraphPattern) -> Option<BranchPlan> {
    let mut patterns = Vec::new();
    let mut filters = Vec::new();
    flatten_conjunction(p, &mut patterns, &mut filters);
    if patterns.is_empty() {
        return None; // the unit relation: nothing to stream
    }
    if !bgp_uses_binary(&patterns) {
        return None; // cyclic → WCOJ/LFTJ path
    }
    let (_, constraints) = extract_quoted_constraints(&patterns);
    if !constraints.is_empty() {
        return None; // RDF 1.2 triple-term decomposition: keep the full path
    }
    let (pat_filters, residual) = split_sargable(graph, &patterns, &filters);
    let pfilter = |i: usize| -> Option<(usize, ScanCmp)> { pat_filters.get(i).copied().flatten() };
    let prepared = prepare_bgp(graph, &patterns).ok()?;
    if prepared.iter().any(|pr| pr.unsatisfiable) {
        // A provably-absent constant: the branch has zero rows (the zk
        // empty-pattern recording only applies while a recorder is armed, which
        // already declined above).
        return Some(BranchPlan {
            prepared,
            pat_filters,
            residual,
            seed: 0,
            seed_sort_col: None,
            steps: Vec::new(),
            vars: collect_vars(&patterns),
            unsat: true,
        });
    }

    // GOO replay (shared verbatim with the executor + EXPLAIN dry-run).
    let mut cs_ctx = CsCtx::new(&prepared);
    let seed = goo_seed(&prepared);
    let seed_sort_col = goo_seed_sort(&prepared, seed, pfilter(seed).map(|(c, _)| c));
    // Accumulate the running row layout exactly as the executor builds it:
    // `scan_to_bindings` dedups a repeated variable position; `bind_join` appends
    // the pattern's non-join variables in canonical order.
    let mut vars: Vec<Variable> = Vec::new();
    for v in prepared[seed].pos_vars.iter().flatten() {
        if !vars.contains(v) {
            vars.push(v.clone());
        }
    }
    let mut cur_card = prepared[seed].est as f64;
    let mut var_ndv: FxHashMap<Variable, f64> = FxHashMap::default();
    let mut done = vec![false; prepared.len()];
    done[seed] = true;
    cs_ctx.note_done(seed);
    record_pattern_ndv(graph, &prepared, seed, cur_card, &mut var_ndv, &cs_ctx);

    let mut steps = Vec::with_capacity(prepared.len() - 1);
    for _ in 1..prepared.len() {
        let est_rows_before = cur_card.max(0.0).round() as usize;
        let (i, new_card, connected) = goo_pick(graph, &prepared, &done, &var_ndv, cur_card, &cs_ctx);
        if !connected {
            return None; // a cross-product step is never admitted
        }
        cur_card = new_card;
        done[i] = true;
        cs_ctx.note_done(i);
        record_pattern_ndv(graph, &prepared, i, cur_card, &mut var_ndv, &cs_ctx);
        // The executor's bind-join cutover, evaluated on the estimate (the EXPLAIN
        // dry-run's exact inequality): one connecting variable, distinct pattern
        // variables, running side much smaller than the pattern.
        let connecting: Vec<&Variable> = vars.iter().filter(|v| prepared[i].var_pos(v).is_some()).collect();
        if connecting.len() != 1
            || !distinct_pattern_vars(&prepared[i].pos_vars)
            || est_rows_before.saturating_mul(8) >= prepared[i].est
        {
            return None; // predicted merge / hash step: not admitted in M1
        }
        let jv = connecting[0].clone();
        let rk = vars.iter().position(|v| *v == jv).expect("connecting var is in the layout");
        let pp = prepared[i].var_pos(&jv).expect("connecting var is in the pattern");
        steps.push(BindStep { pat: i, rk, pp });
        for (pos, ov) in prepared[i].pos_vars.iter().enumerate() {
            if pos != pp {
                if let Some(v) = ov {
                    vars.push(v.clone());
                }
            }
        }
    }
    Some(BranchPlan {
        prepared,
        pat_filters,
        residual,
        seed,
        seed_sort_col,
        steps,
        vars,
        unsat: false,
    })
}

/// The block driver (§5, amended): consumes each branch's seed scan in ramped
/// blocks, runs the bind-join chain per block in seed-lex order, applies residual
/// filters / projection / first-seen DISTINCT / slice, and serialises surviving
/// solutions straight into the pending chunk.
fn drive(
    graph: &Graph,
    plan: &StreamPlan,
    flush: Option<usize>,
    emit: &mut dyn FnMut(String) -> ControlFlow<()>,
) -> Result<(), String> {
    // The head is statically known for classified shapes — built before the seed
    // scan starts (§5) and flushed at the normal chunk boundaries.
    let mut pending = String::from("{\"head\":{\"vars\":[");
    for (i, v) in plan.out_vars.iter().enumerate() {
        if i > 0 {
            pending.push(',');
        }
        pending.push('"');
        crate::json::escape_into(&mut pending, v.as_str());
        pending.push('"');
    }
    pending.push_str("]},\"results\":{\"bindings\":[");

    // Residual FILTERs never bind new row values, so rows carry store ids only
    // (serialised via `write_store_id_json`); the vocab exists solely for the
    // expression evaluator's signature.
    let local = LocalVocab::default();
    let mut seen: FxHashSet<Row> = FxHashSet::default();
    let (mut to_skip, mut remaining) = plan.slice.map_or((0, None), |(s, l)| (s, l));
    let mut written = 0usize;
    let (block_first, block_max) = testing::ramp();

    if remaining != Some(0) {
        'branches: for branch in &plan.branches {
            if branch.unsat {
                continue; // an absent constant term: zero rows from this branch
            }
            // Projection layout for THIS branch (a UNION branch may not bind every
            // projected variable — those cells serialise as unbound, i.e. omitted).
            let cols: Vec<Option<usize>> = plan
                .out_vars
                .iter()
                .map(|v| branch.vars.iter().position(|x| x == v))
                .collect();
            let seed_prep = &branch.prepared[branch.seed];
            let seed_rows = scan_to_bindings(
                graph,
                &seed_prep.id_pat,
                &seed_prep.pos_vars,
                branch.seed_sort_col,
                branch.pat_filters.get(branch.seed).copied().flatten(),
                None,
                #[cfg(feature = "semijoin-bitmap")]
                None,
            );
            testing::add_seed_total(seed_rows.rows.len());

            let mut start = 0usize;
            let mut block = block_first;
            while start < seed_rows.rows.len() {
                // Cooperative gate once per block: converts the sticky flag set by
                // the coarse in-join checks (or a deadline / cancellation) into the
                // budget error before more work is done.
                budget::check(written)?;
                let end = (start + block).min(seed_rows.rows.len());
                let mut rows: Vec<Row> = seed_rows.rows[start..end].to_vec();
                for step in &branch.steps {
                    if rows.is_empty() {
                        break;
                    }
                    let pr = &branch.prepared[step.pat];
                    rows = bind_join_seed_lex(
                        graph,
                        &rows,
                        &pr.id_pat,
                        &pr.pos_vars,
                        step.rk,
                        step.pp,
                        branch.pat_filters.get(step.pat).copied().flatten(),
                    );
                }
                if !rows.is_empty() && !branch.residual.is_empty() {
                    let mut b = Bindings::unsorted(branch.vars.clone(), rows);
                    for f in &branch.residual {
                        if b.rows.is_empty() {
                            break;
                        }
                        apply_filter(graph, &local, &mut b, f)?;
                    }
                    rows = b.rows;
                }
                for row in &rows {
                    let proj: Row = cols.iter().map(|c| c.map(|i| row[i]).unwrap_or(NO_ID)).collect();
                    if plan.distinct && !seen.insert(proj.clone()) {
                        continue; // first-seen hash DISTINCT, persistent across blocks
                    }
                    if to_skip > 0 {
                        to_skip -= 1; // OFFSET consumes surviving solutions
                        continue;
                    }
                    if written > 0 {
                        pending.push(',');
                    }
                    written += 1;
                    pending.push('{');
                    let mut first = true;
                    for (vi, &id) in proj.iter().enumerate() {
                        if id == NO_ID {
                            continue; // unbound (a UNION branch-local variable)
                        }
                        if !first {
                            pending.push(',');
                        }
                        first = false;
                        pending.push('"');
                        crate::json::escape_into(&mut pending, plan.out_vars[vi].as_str());
                        pending.push_str("\":");
                        write_store_id_json(graph, id, &mut pending);
                    }
                    pending.push('}');
                    if flush.is_some_and(|n| pending.len() >= n)
                        && emit(std::mem::take(&mut pending)).is_break()
                    {
                        return Ok(()); // consumer gone: abandon the rest
                    }
                    if let Some(rem) = remaining.as_mut() {
                        *rem -= 1;
                        if *rem == 0 {
                            break 'branches; // LIMIT satisfied: stop the driver
                        }
                    }
                }
                testing::add_seed_processed(end - start);
                testing::add_block();
                start = end;
                block = block.saturating_mul(2).min(block_max);
            }
        }
    }
    // Final sticky gate: a trip inside the last block must surface as the budget
    // error (a truncated stream), never a clean short document.
    budget::check(written)?;
    pending.push_str("]}}");
    let _ = emit(pending);
    Ok(())
}

/// The §5.1 seed-lex bind join over one block: the per-join-value grouping is a
/// SCAN CACHE only (one bound index scan per distinct join value in the block,
/// matches cached in match-scan order); the block's rows are then walked in order
/// with each row's matches appended contiguously — so the output is the
/// lexicographic (input row, match) order, unlike the buffered kernel's
/// hash-bucket group loop. Match semantics (bound scan + inline pushed-down
/// filter) are identical to `exec::bind_join`.
fn bind_join_seed_lex(
    graph: &Graph,
    rows: &[Row],
    id_pat: &IdPattern,
    pos_vars: &[Option<Variable>; 3],
    rk: usize,
    pp: usize,
    filt: Option<(usize, ScanCmp)>,
) -> Vec<Row> {
    // The pattern's NEW variable columns (admission guarantees the only shared
    // variable is the join variable, so the rest are new).
    let new_positions: Vec<usize> = (0..3).filter(|&p| p != pp && pos_vars[p].is_some()).collect();
    let mut cache: FxHashMap<Id, Vec<SmallVec<[Id; 2]>>> = FxHashMap::default();
    let mut out: Vec<Row> = Vec::new();
    for (i, row) in rows.iter().enumerate() {
        // Coarse cooperative check (the buffered kernel's per-group cadence): a
        // tripped budget sets the sticky flag; the driver's per-block
        // `budget::check` converts it into the error.
        if i & 1023 == 0 && budget::exhausted(out.len()) {
            break;
        }
        let val = row[rk];
        let matches = cache.entry(val).or_insert_with(|| {
            let mut bound = *id_pat;
            bound[pp] = Some(val);
            let scan = graph.store.scan(&bound);
            let mut ms: Vec<SmallVec<[Id; 2]>> = Vec::new();
            for prow in scan.rows.iter() {
                let pspo = scan.to_spo(prow);
                if let Some((fpos, cmp)) = filt {
                    if !cmp.test_id(graph, pspo[fpos]) {
                        continue;
                    }
                }
                ms.push(new_positions.iter().map(|&p| pspo[p]).collect());
            }
            ms
        });
        for m in matches.iter() {
            let mut r = row.clone();
            r.extend_from_slice(m);
            out.push(r);
        }
    }
    out
}
