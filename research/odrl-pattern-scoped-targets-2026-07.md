# Pattern-scoped ODRL targets — sub-named-graph (triple-pattern) result masking

Status: **design record + measured prototype** (spike `sq-lrtc3.3`, epic `sq-lrtc3`,
maintainer invariant "an ODRL policy over (requester, action, target=graph|**pattern**)
gates the query"). Companion records: [`solid-access-control-design.md`](solid-access-control-design.md)
(the WAC/ACP substrate + the DatasetView enforcement architecture this record must stay
consistent with), [`trust-graph-authorisation-2026-07.md`](trust-graph-authorisation-2026-07.md)
(the fail-closed enforcement law), `crates/sparq-solid/src/odrl_bridge.rs` (the
graph-granular ODRL→`<urn:sparq:auth>` bridge this extends).

<!-- [FABLE-5] 🤖 SPARQ agent. Design-first spike: the decision in §3 is prototyped
behind the OFF-by-default `pattern-scope` feature of sparq-solid; the bridge wiring is
decomposed into follow-up beads in §7, per the proceed-and-document rule. -->

> **Honest scope.** This is a *clear-path* authorisation mechanism (no cryptographic
> guarantee is claimed anywhere in it). The prototype is library-level: policy-decision
> plumbing from ODRL rules to pattern scopes is designed here (§5) but wired in
> follow-up beads. No performance numbers appear in this record; the measured envelope
> lives in `bench/pattern-scope/` (work-box numbers, non-canonical).

## 0. Problem

Enforcement granularity today is the **named graph**: `PodStore::view_for` builds a
`DatasetView` (an O(1) graph-NAME allowlist installed thread-locally in the engine —
`crates/sparq-engine/src/exec.rs` `mod view`), and the ODRL bridge materializes
`party auth:<mode> <graphIri>` grants consumed at that granularity
(`odrl_bridge.rs::materialize_policy` → `AuthIndex::accessible`). An ODRL target is a
bare asset IRI (`sparq_policy::model::Rule::target: Option<String>`), optionally
resolved through `odrl:AssetCollection` membership — still to a whole graph.

The maintainer invariant names `target = pattern` too: *"share my address book except
phone numbers"*, *"grant the researcher the trial results but not the participant
identifiers"* — a permission or prohibition whose target is a **subset of a graph
described by a triple pattern**. The result set the requester can observe must behave
**exactly as if the masked triples were physically absent** — under every construct.

### The leakage bar (what "fail-closed" means here)

A masked triple must not be observable through ANY of:

| Channel | Attack shape |
|---|---|
| SELECT rows | the triple's bindings appear |
| `EXISTS` / `NOT EXISTS` / `MINUS` | a boolean flips because the masked triple matched inside the sub-pattern (cf. the `sq-7d3dj.30.11` EXISTS re-entry lesson: EXISTS is evaluated on re-entrant paths that bypass naive result-side hooks) |
| `OPTIONAL` | a row binds vs stays unbound |
| Aggregates / `COUNT` | a count differs from the count over the physically-reduced dataset — a **counting side-channel** even when no masked binding is projected |
| ASK / result cardinality / `GRAPH ?g` enumeration | satisfiability or graph-name visibility differs from the physically-reduced dataset |

The only defensible definition is **oracle equivalence**: for every query Q,
`eval(masked(D), Q) = eval(D ∖ masked-triples, Q)`. Anything weaker invites a
construct-by-construct audit that has to be re-done for every new engine fast path.

## 1. Options considered

### A. In-line per-triple scan filter (engine-level DatasetView extension) — REJECTED for v1

Extend the engine's thread-local view state with per-graph triple-pattern sets consulted
during scans. Survey of the actual scan surface (2026-07-11, `origin/main`):

* The only universal row choke point is `TripleStore::scan_with` (`sparq-core/src/store.rs`);
  above it, `sparq-engine/src/exec.rs` has **~15 distinct id-level scan entry points**
  that bypass the common `scan_to_bindings` materializer (streaming JSON emit, DISTINCT
  skip-scans, anchor/semijoin probes, WCOJ trie build, bind-join, property paths).
* The COUNT/ASK fast paths (`count_pushdown`, `try_count`) answer from
  `TripleStore::estimate` — **pure index-range arithmetic that iterates no rows at
  all**. A row-level filter is silently ignored there; every such path must ALSO be
  detected and defeated (as `single_pattern_scan_json_emit` already does for the
  graph-level view and the `zk` recorder).

This is exactly the *"per-quad-pattern rules evaluated in-line"* (GraphDB-FGAC-shaped)
cost model that `solid-access-control-design.md` **deliberately rejected** — and it
re-taxes every future fast path with a soundness obligation: miss one entry point (or
one `estimate()` shortcut) and the miss is a **silent leak**, not a test failure. The
`sq-7d3dj.30.11` review history shows how subtle same-thread re-entry (EXISTS) makes
per-scan thread-local state. Kept on the table only as a **v2 performance path**
(§6) if the materialization cost measured in `bench/pattern-scope/` ever dominates a
real workload — it would then be built as an engine feature with a differential
oracle harness fuzzing every fast path.

### B. Algebra rewrite (inject guards into the query) — REJECTED

Rewrite the incoming query so every triple-pattern access to a scoped graph carries a
guard (a `FILTER NOT EXISTS`-style exclusion or an injected values-restriction).
Rejected because the rewrite must cover **every triple-touching construct** — property
paths (whose expansion is engine-internal), `EXISTS` sub-patterns, `GRAPH ?g`,
`SERVICE`-less federation later, and every construct SPARQL grows — and must survive
the optimizer's rewrites underneath it. A missed construct is again a silent leak.
Guards also do not compose with negative conditions (`NOT EXISTS`/`MINUS`): excluding
masked rows from the *outer* pattern does not stop a masked triple satisfying an
*inner* negation and flipping the outer row set. The prior art the estate already
follows (`query_as_rewrite`, the v1 `FROM NAMED` injection) rewrites only the
**dataset clause**, not patterns — that discipline ("the rewrite stays trivial because
policy ran at materialization time") is a load-bearing commitment of
`solid-access-control-design.md`.

### C. Post-eval row masking — REJECTED (unsound by construction)

Filter result rows after evaluation. `EXISTS`, `MINUS`, `OPTIONAL` and every aggregate
are computed **over the unmasked data before the filter runs**: a COUNT includes masked
rows, an EXISTS flips on masked triples. This fails the leakage bar definitionally —
listed only because it is the tempting-cheap option a future reader might reach for.

### D. Masked-subgraph materialization — **CHOSEN**

A pattern target denotes a **derived sub-graph asset**: the sub-graph of source graph
`g` visible under a scope (allow-patterns minus deny-patterns). Enforcement
materializes that sub-graph as a physical `Graph` replica and substitutes it for `g`
in the dataset the engine evaluates; the graph-granular machinery (the DatasetView
allowlist, the `∪ allow ∖ ∪ deny` walk, `wrap_read` default-graph semantics) is
**unchanged and unaware** that masking happened.

* **Sound by construction**: the evaluated dataset *is* the oracle dataset
  (`D ∖ masked-triples`) — OPTIONAL/EXISTS/MINUS/aggregates/COUNT/ASK equivalence is
  an identity, not an audit obligation. New engine fast paths inherit soundness for
  free. (The differential test in
  `crates/sparq-solid/tests/pattern_scope.rs` still exists — it pins the *pattern
  semantics* and the *assembly*, the two places a bug can now live.)
* **Preserves every prior design commitment**: graph-granularity enforcement,
  materialize-then-restrict, restriction-composes-never-widens, D1 triples-native
  grants (§5), fail-closed absence-of-grant.
* **Zero engine/core changes**: the prototype is a new fully-`#[cfg]`-gated module in
  `sparq-solid` using only public `sparq-core`/`sparq-engine` APIs. Feature-OFF builds
  of every crate are byte-identical (no always-compiled file is touched).
* **Cost model is explicit and measurable**: O(|source graph|) decode+rebuild per
  (scoped graph × distinct scope), paid at **scope-application time**, not per query
  row. The prototype's `ScopedDataset` is the cache unit: build once per
  (session-scope fingerprint), query many times. §6 gives the production caching path.

## 2. Semantics

### 2.1 Scope algebra

A **scope pattern** is `(s?, p?, o?)` — each component a concrete term or a wildcard
(no join variables in v1; a pattern is a per-triple predicate, so scope checking is
O(1) per triple and the masked sub-graph is well-defined without evaluation order).
A **graph scope** is `{allow: [pattern…], deny: [pattern…]}`; a triple is visible iff
it matches ≥1 allow pattern AND 0 deny patterns. **Deny overrides allow** at equal
rank, mirroring the ODRL conflict strategy the estate already implements. The empty
allow list yields the empty sub-graph (fail-closed: an empty scope grants nothing).

* Permission targeting a pattern asset ⇒ `allow = [patterns], deny = []`.
* Prohibition carving a hole in a granted graph ⇒ `allow = [any], deny = [patterns]`.
* Both rule kinds on one graph compose in one scope (allow from permissions, deny from
  prohibitions) — the same deny-overrides walk as the graph level, one level down.

### 2.2 Composition with the graph level (never widens, session path)

`PodStore::scoped_dataset(session, mode, scopes)` composes as **refinement only**: a
masked replica is included for graph `g` iff `g` is in the session's graph-level
accessible set AND `scopes` carries an entry for `g`; accessible graphs without a
scope entry are included whole; graphs outside the accessible set are ABSENT no matter
what `scopes` says. So the session path can only shrink what the graph level granted —
the "restriction composes, never widens" invariant holds *per layer*. The
policy-decision path that legitimately grants a pattern-scoped slice of an otherwise
un-granted graph (§5) uses the explicit-decision primitive `masked_dataset` instead,
where the caller (the future bridge — which has already run deny-overrides across ALL
rules) supplies the complete final visibility decision.

### 2.3 Indistinguishability of masked-empty and absent

A graph whose mask evaluates to the empty sub-graph is **omitted from the assembled
dataset entirely** (not included as an empty named graph): `GRAPH <g> {}`'s unit-row
semantics and `GRAPH ?g` enumeration must not reveal that a name *exists but is fully
masked* — the same property the graph-level view pins ("a non-visible graph is
indistinguishable from an absent one").

### 2.4 UPDATE surface — out of scope v1

Masking governs the READ path only. `PodStore::update_as` keeps its existing
graph-granular write enforcement; a pattern-scoped WRITE grant (row-level update
authority) is a different problem (blind writes into masked regions, delete-visibility
interactions) and is explicitly deferred (§7, follow-up bead). A scoped dataset is a
read-only replica; writes to the underlying store invalidate it (§6).

## 3. Prototype (merged with this record, feature `pattern-scope`, OFF by default)

`crates/sparq-solid/src/pattern_scope.rs`, `#[cfg(feature = "pattern-scope")]`:

| Surface | Contract |
|---|---|
| `ScopePattern` (`new`/`any`/`matches`) | per-triple predicate; `None` = wildcard |
| `GraphScope` (`allow_only`/`deny_within`/`visible`) | §2.1 algebra, fail-closed empty-allow |
| `masked_graph(&Graph, &GraphScope) -> Graph` | decode → filter → rebuild (fresh dict, public APIs only) |
| `masked_dataset(&Graph, &FxHashMap<Term, GraphScope>) -> Graph` | explicit-decision assembly (§2.2), empty default graph, omit-empty (§2.3) |
| `PodStore::scoped_dataset(&self, &Session, Mode, &FxHashMap<Term, GraphScope>) -> ScopedDataset` | refinement-only session composition (§2.2) |
| `ScopedDataset::{view, query, query_json, ask}` | same `wrap_read` rewrite + `DatasetView{default: Empty}` path as `query_as` — identical default-graph semantics |

Differential acceptance test (`tests/pattern_scope.rs`): a query battery — SELECT,
OPTIONAL, EXISTS, NOT EXISTS, MINUS, COUNT/GROUP BY aggregates, ASK, `GRAPH ?g`
enumeration, property path — asserting `scoped == oracle` (a `PodStore` loaded from
the source dataset with the masked lines physically deleted) row-for-row, AND
`scoped != unmasked` on the same battery (non-vacuity: a no-op mask flips the test
red). Fail-closed unit tests: empty-allow ⇒ absent graph; deny-overrides-allow;
scope on a non-accessible graph does not widen; fully-masked ⇒ indistinguishable from
absent.

## 4. Measured overhead envelope

`bench/pattern-scope/` (JSON, work-box, **non-canonical** — canonical numbers are
EC2-gated per the benchmarking discipline). Driver:
`cargo run -p sparq-solid --features pattern-scope --release --example pattern_scope_bench`.
Dimensions measured, each at several graph sizes:

1. **Scope-application cost** — `scoped_dataset` build time vs source size (the
   O(|graph|) rebuild this design pays instead of per-row filtering).
2. **Per-query overhead** — query latency over a `ScopedDataset` vs the same query
   via plain `query_as` on an equivalently-sized store (expected ≈0: after assembly
   the engine sees an ordinary dataset), vs the oracle store (expected =, it is the
   same dataset).
3. **Amortization** — queries needed for build-once/query-many to beat a
   hypothetical per-query rebuild (informs the §6 cache priority).

## 5. ODRL vocabulary for pattern targets (designed here, wired in follow-ups)

Grants must stay **triples-native** (design commitment D1). Two additions, both in the
sparq ODRL profile namespace, both consumed at materialization time:

```turtle
# The target asset of a permission/prohibition MAY be a pattern asset:
<urn:ex:asset:contacts-sans-phone> a sparq:PatternAsset ;
    sparq:sourceGraph <https://pod.ex/contacts> ;          # exactly one
    sparq:pattern [ sparq:predicate <https://ex.dev/ns#phone> ] .  # ≥1; absent component = wildcard
```

* `odrl:target <a PatternAsset>` on a **permission** ⇒ allow-patterns over
  `sourceGraph`; on a **prohibition** ⇒ deny-patterns carved out of whatever else is
  granted on `sourceGraph`. Deny overrides allow (§2.1) — same conflict strategy as
  the existing evaluator.
* Materialized form in `<urn:sparq:auth>` (the bridge's output, follow-up bead): a
  grant node `auth:PatternGrant` with `auth:agent`, `auth:mode`, `auth:graph`,
  and ≥1 `auth:allowPattern`/`auth:denyPattern` blank nodes carrying
  `auth:subject`/`auth:predicate`/`auth:object` — structurally parallel to the
  existing `auth:ConditionalGrant`, so `AuthIndex` grows one more parsed node kind
  whose product is a `GraphScope` per (principal, mode, graph).
* Fail-closed parse rule: a `PatternAsset` with zero parseable patterns, an ambiguous
  `sourceGraph`, or any component that is not a concrete term ⇒ the rule materializes
  **nothing** (absence-of-grant, never a whole-graph fallback).

`Rule::target` in `sparq-policy` stays a bare IRI — the pattern structure lives in the
policy *graph* and is resolved by the bridge, so the evaluator's matching semantics
(equality / collection membership) are untouched. This keeps the invasive change out
of the evaluator entirely.

## 6. Production path (beyond the spike)

* **Replica cache** — **SHIPPED** (`sq-nc3c6`, `crates/sparq-solid/src/scope_cache.rs`),
  with three deliberate deviations from the sketch above, all recorded here because they
  change the security argument, not just the code:
  * **Key.** Not a bare fingerprint: the map key is `(graph name, NORMALIZED GraphScope)`
    and the 64-bit hash is used ONLY to pick the lock stripe. Keying on a hash alone
    would make a collision serve one scope's replica for another's mask — an
    over-disclosure, i.e. fail-OPEN. Normalization (both pattern lists sorted +
    de-duplicated) is what makes two orderings of the same mask one scope class, and it
    cannot change visibility: `visible` is an `any` over `allow` and an `all` over
    `deny`, both invariant under reordering and duplication.
  * **Invalidation.** The store's existing generation bump (`reindex_with`) fires only
    when the AUTH VIEW is re-materialized, and an ordinary pod-document write does not
    re-materialize — so hanging invalidation off it alone would leave a stale replica
    after every data write. The cache is therefore dropped at BOTH seams: `reindex_with`
    (which every `materialize_*`/`put_acl`/`delete_acl`/bridge/trust path routes through)
    and the in-place data write in `update_inner`. Wholesale, not diffed: v1 buys
    soundness over precision, and a dropped replica is re-derived from the current graph.
    Staleness here is a freshness bug rather than a mask bypass — a stale replica holds
    only triples that were in the source and passed the same scope — but it would defeat
    a redaction performed by DELETING data, which is why it is not tolerated.
  * **Unmasked graphs are forked, never replicated.** An accessible graph with no scope
    entry (or a scope that masks nothing) now contributes a `Graph::fork` of the source —
    an `Arc`-sharing logical copy — instead of a decode → filter → rebuild that produced
    an identical graph. This is where most of the old per-call O(accessible dataset) cost
    lived, and it keeps the cache from holding a second full copy of the store.
* **Memory**: worst case one replica per (graph × scope class) of scoped triples, made
  finite by a hard `SHARDS * SHARD_CAP` cap with insertion-order eviction (a hit takes
  only a read lock, so it cannot record recency without serialising concurrent readers).
  The cap must exceed the number of scoped graphs one `scoped_dataset` pass touches or
  each pass evicts what the next needs; the shipped value was sized against the bench
  fixture's accessible-set size. Envelope in `bench/pattern-scope/` (`cold_build_ms` vs
  `warm_build_ms`; work-box, non-canonical).
* **v2 escape hatch** (only if measurement ever demands it): the in-line scan filter
  (§1-A) as an engine opt-in feature with a fuzzed differential oracle harness across
  every scan entry point — the bar it must clear is documented there.

## 7. Follow-up beads (created with this PR)

1. `sq-qnlj8` — `feat(solid)`: ODRL bridge wiring — `sparq:PatternAsset` parsing +
   `auth:PatternGrant` materialization + `AuthIndex` scope extraction (§5), gated on
   `pattern-scope` + `odrl-bridge`.
2. `sq-nc3c6` — `feat(solid)`: scoped-replica cache + write-path invalidation (§6).
   **SHIPPED** — see the §6 bullet for the three deviations from the original sketch.
3. `sq-fznmq` — `spike(solid)`: pattern-scoped UPDATE enforcement design (§2.4).
