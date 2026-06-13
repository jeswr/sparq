<!-- [OPUS-4.8] authored by Opus 4.8 (Fable 5 unavailable) — re-review when Fable returns -->

# SPARQL 1.1 property paths — design & implementation plan

Date: 2026-06-13. Scope: `crates/sparq-engine` (algebra from vendored `spargebra`, execution in `exec.rs`), with reuse assessment against `crates/sparq-reason` and the `crates/sparq-zk` trace seam. This is a **DESIGN + PLAN** doc; a follow-up agent implements the remaining slices once `exec.rs` is free. Read-only analysis — no engine code was edited.

> **Status correction (read this first).** The companion file `research/property-paths-assessment.md` (2026-06-12) reviewed a *pre-pushdown* state and ranked three improvements. **All three have since landed** (commits `47d4329` "push bound endpoints into property-path evaluation", `2dbfca1` "differential + budget tests and paired bench for path pushdown"). The current `exec.rs` already implements bound-endpoint pushdown, single-source directed BFS (`directed_reach`), SCC-based `?x p+ ?x` (`cyclic_nodes`), index-aware negated sets, and per-loop budget checks. So **property paths are semantically complete and the major perf work is done.** This document is therefore a *spec-grounded audit + architecture record + a precise plan for the genuinely-remaining slices* — not a from-scratch implementation plan. Where the prompt's framing assumed paths were unsupported, this doc states the real ground truth and re-targets the wave plan accordingly.

---

## 0. Ground truth: what already exists

| Concern | Where | State |
|---|---|---|
| Algebra (8 path operators) | `vendor/spargebra/src/algebra.rs:7` (`PropertyPathExpression`), `:599` (`GraphPattern::Path { subject, path, object }`) | Complete; parser fully covers `iri ^ / \| * + ? !` |
| Parser-level translation | `vendor/spargebra/src/parser.rs:365` (`add_to_triple_or_path_patterns`) | Fixed-length `/` and top-level `^` are flattened to BGP triples at parse time (see §1) |
| Dispatch | `crates/sparq-engine/src/exec.rs:1594` (`eval_path` call site in `eval_graph_pattern_inner`) | Wired |
| Endpoint resolution + DISTINCT + zero-length | `exec.rs:1936` (`eval_path`) | Complete |
| Recursive evaluator | `exec.rs:2092` (`path_pairs`) | All 8 variants native, with bound-endpoint pushdown |
| Single-source BFS | `exec.rs:2225` (`directed_reach`, fwd/rev) | Complete |
| `?x p+ ?x` via SCC | `exec.rs:2298` (`cyclic_nodes`, Kosaraju) | Complete |
| All-pairs closure | `exec.rs:2455` (`transitive_closure_pairs`) | Complete, budgeted |
| Negated property set | `exec.rs:2397` (`negated_property_pairs`) | Index-aware block-skip |
| Budget guard | `exec.rs:33` (`mod budget`), checks inside BFS loops | Complete |
| zk trace fail-closed | `crates/sparq-zk/src/verify.rs:138` | `GraphPattern::Path { .. } => Err(UnsupportedFragment("property path"))` |
| W3C conformance | `tests/w3c/.../sparql11/property-path/` (fetched, gitignored), run by `crates/sparq-conformance` | **33/33 pass** at rdf-tests `f25dbc0` |

Key takeaway for the implementer: **do not re-implement; refine and harden.** The remaining work is correctness-edge hardening, an explicit cardinality/ordering hook in the planner, an adversarial test layer beyond the W3C suite, and a decision on whether to share the transitive kernel with `sparq-reason`.

---

## 1. The algebra split: what reaches `eval_path` vs. what becomes a BGP

This is the single most important architectural fact and it is easy to get wrong. **spargebra translates part of the path algebra at parse time**, in `add_to_triple_or_path_patterns` (`parser.rs:365`). When the engine encounters `?s <path> ?o`:

- **`PropertyPathExpression::NamedNode(p)`** → emitted as a plain `TriplePattern` (`parser.rs:376`). A length-1 path is just a triple; `eval_path` never sees it at top level.
- **`PropertyPathExpression::Reverse(p)`** → recurses with subject/object **swapped** (`parser.rs:379`). So a *top-level* `^:p` becomes a plain reversed triple `?o :p ?s`; `^` only survives as a `Path` node when nested inside a recursive/alternative path.
- **`PropertyPathExpression::Sequence(a, b)`** → a fresh **blank node** midpoint and two recursive triple-or-path patterns (`parser.rs:388–411`). This is exactly the SPARQL 1.1 spec's translation of fixed-length sequence to a BGP join. So `:a/:b` becomes `{ ?s :a _:m . _:m :b ?o }` — a BGP, never a `Path`.
- **Everything else** — `Alternative`, `ZeroOrMore`, `OneOrMore`, `ZeroOrOne`, `NegatedPropertySet` (and any `Reverse`/`Sequence` *nested inside* them) — falls through to the catch-all (`parser.rs:412`) and is emitted as `GraphPattern::Path { subject, path, object }`. **Only these reach `eval_path`.**

Consequence: the `Sequence` and `Reverse` arms in `path_pairs` (`exec.rs:2096`, `:2097`) fire **only** for sub-paths nested inside a surviving recursive/alternative/negated path (e.g. `(:a/:b)+`, `(^:p|:q)*`). They are not dead code — they are reached recursively, never at the top level. Any reasoning about path evaluation must distinguish "top-level fixed path → BGP (multiset/bag semantics, handled by the join machinery)" from "recursive/alternative path → `eval_path` (DISTINCT set semantics)". This split is exactly what SPARQL 1.1 §18.4–§18.5 prescribes (see §3).

> **Spec alignment.** SPARQL 1.1 Query, §18.5 "Translate Property Path Expressions" + §18.4.2.4 evaluation: fixed-length paths translate to basic graph patterns/joins (so they carry **multiset/bag** cardinality — duplicate routes count), whereas the arbitrary-length operators are evaluated by the **ALP** procedure (`ArbitraryLengthPath`, §18.4.2.5) which returns **distinct reachable nodes** (set semantics — routes are *not* counted). The spargebra-then-`eval_path` split realizes precisely this: `Sequence` → BGP/join (bag), `*`/`+`/`?` → `eval_path` (set, deduplicated by the `seen` row-set at `exec.rs:2024` and the `FxHashSet` pair relations throughout).

---

## 2. Operator-by-operator evaluation strategy (current design, audited)

Notation: `PathEnds { s, o }` (`exec.rs:2064`) carries bound endpoints down the recursion. CONTRACT (verbatim from the code, `exec.rs:2057`): `path_pairs(graph, path, ends)` returns a **subset** of the path's full `(start,end)` relation that contains **every** pair satisfying the bounds; a sub-evaluation may ignore the hint and return extra pairs (callers post-filter) but must **never invent** pairs outside the relation. So pushdown is a pure optimization and `eval_path`'s post-filter (`exec.rs:2014`) is the correctness backstop. This contract is the right one and should be preserved by any future change.

### 2.1 Fixed-length operators (set/bag via algebra + scans)

| Operator | Variant | Reaches `eval_path`? | Evaluation | Permutation / index |
|---|---|---|---|---|
| `iri` | `NamedNode(p)` | Only when nested | `predicate_pairs` (`exec.rs:2375`): scan `[ends.s, Some(pid), ends.o]` | Always one contiguous range in a built permutation; P-leading (PSO/POS) for unbound endpoints, S-/O-leading when an endpoint is bound (`store.rs` `choose`) |
| `^path` | `Reverse(a)` | Top-level → BGP; nested → here | `path_pairs(a, ends.swapped())` then swap each `(s,o)→(o,s)` (`exec.rs:2096`). Inverse = exchange S/O roles, which the **OSP/OPS/POS** permutations answer directly — no data copy, just a different leading column | O-leading permutation for the reversed leaf scan |
| `path1/path2` | `Sequence(a,c)` | Top-level → BGP; nested → here | Midpoint hash join (`join_seq`, `exec.rs:2194`) with endpoint pushdown: bound start → evaluate near hop from `s`, push each reached midpoint into the far hop while fan-out ≤ `SEQ_MIDPOINT_FANOUT_LIMIT` (1024, `exec.rs:2086`); above that, far hop gets only the outer bound and midpoints meet in the hash join. Bound object mirrors this | leaf scans per nested operator |
| `path1\|path2` | `Alternative(a,c)` | Yes | Set union of the two sub-relations; **endpoints push into both branches unchanged** (`exec.rs:2142`) | per-branch |
| `!iri` / `!(iri\|…)` | `NegatedPropertySet(props)` | Yes | `negated_property_pairs` (`exec.rs:2397`): bound endpoint → `[ends.s, None, ends.o]` scan, drop excluded predicate ids; fully-unbound → walk a **P-leading** permutation and `partition_point`-skip each excluded predicate's contiguous block wholesale (no per-triple set probe over the excluded mass) | P-leading (PSO full / POS compact) |
| `!^iri` / mixed | parser splits → `Alternative(NegatedPropertySet(direct), Reverse(NegatedPropertySet(inverse)))` | Yes | **Correctness-critical:** the negated-property-set-with-inverse case is normalized **at parse time** (`parser.rs:1932–1959`): forward members → `NegatedPropertySet([direct…])`, inverse members → `Reverse(NegatedPropertySet([inverse…]))`, combined by `Alternative`. This matches SPARQL 1.1 §18.4.2.4 exactly: `!(:a\|^:b)` = (any forward edge except `:a`) ∪ (any reverse edge except `:b`). The engine never special-cases it because the algebra already encodes it | composes via the `Reverse`+`Alternative` arms |

**Inverse-permutation note (prompt asked to confirm):** yes — inverse maps to swapping S/O, and the OSP/POS/OPS permutations already exist in `BUILT` (`store.rs:18`; compact wasm set = `{SPO, POS, OSP}` still has an O-leading index). `directed_reach` uses this for a bound *object* by scanning `[None, Some(pid), Some(node)]` (`exec.rs:2245`, `Dir::Rev`), reading the subject column. No new index is required.

### 2.2 Arbitrary-length operators (ALP, DISTINCT-node semantics)

The recursive operators implement SPARQL 1.1 §18.4.2.5 **ALP** semantics: the result is the set of **distinct** nodes reachable (not a count of paths), and the **zero-length** case (`*`, `?`) is reflexive over the relevant node domain. The engine realizes this with three direction-specialized strategies, chosen by which endpoints are bound:

| Case | `*` / `+` strategy | Where | Cost |
|---|---|---|---|
| **Bound start** `X p+ ?o`, `X p* ?o` | Single-source forward BFS from `X` over range scans; `*` adds the single reflexive pair `(X,X)` | `directed_reach(.., Dir::Fwd)` `exec.rs:2148`, `:2155` | `O(E_reachable)` — edges in `X`'s forward reach, **not** all-pairs |
| **Bound object** `?s p+ X`, `?s p* X` | Single-source **reverse** BFS from `X` (O-leading scans); `*` adds `(X,X)` | `directed_reach(.., Dir::Rev)` `exec.rs:2149`, `:2161` | `O(E_reachable backward)` |
| **Both bound** `A p+ B` | Reverse-or-forward BFS with **early exit** when the target is reached (`directed_reach`'s `target` arg, `exec.rs:2285`) | `exec.rs:2148`/`2149` with `ends.o`/`target` | reachability test, stops on hit |
| **Same var** `?x p+ ?x` | Nodes on a directed cycle = SCC members (size ≥ 2) ∪ self-loops, via Kosaraju (`cyclic_nodes`, `exec.rs:2298`) | `exec.rs:2005` | `O(V+E)` instead of all-pairs `O(V·E)` |
| **Both unbound** `?s p+ ?o` (the expensive one) | Full all-pairs closure: per-start BFS over the base relation's adjacency map | `transitive_closure_pairs` `exec.rs:2455` | `O(V·E)` time, up to `O(V²)` pairs of memory — **bounded only by the budget** (see §4) |

`?` (`ZeroOrOnePath`, `exec.rs:2173`) = base relation ∪ reflexive pairs, with the same endpoint-aware reflexive handling (a bound endpoint adds only **its own** `(x,x)`, not a whole-store `graph_nodes()` scan).

**Cycle handling (correctness).** Every traversal uses a per-source `seen: FxHashSet<Id>` visited set (`directed_reach` `exec.rs:2273`, `transitive_closure_pairs` `exec.rs:2469`), so cycles terminate. Combined with the set-valued pair relations and the `seen` row-set in `eval_path`, this gives exactly the DISTINCT-node semantics ALP requires: a node reachable by many routes (or via a cycle) appears **once**. This is the spec's `ALP` "visited" guard (§18.4.2.5, `eval(ALP(x, path))` accumulates a *set* `V` of nodes and recurses only into unvisited ones) — the implementation is structurally faithful.

**Zero-length domain (correctness subtlety).** For `*`/`?` with a **constant** endpoint absent from the data, the zero-length solution still holds (`<s> :p* <s>` is true on the empty graph). `eval_path` handles this by interning the absent constant locally and emitting the reflexive row even when the relation is empty (`exec.rs:1966`, `:2029–2052`). For variable–variable `*`/`?`, the reflexive domain is `graph_nodes()` (every subject or object id, `exec.rs:2442`) — matching the spec's restriction of zero-length to terms occurring in the data (subjects/objects), **not** all IRIs in the universe. This is correct per §18.4.2.5 (ZeroLengthPath ranges over `subjects(G) ∪ objects(G)`).

---

## 3. Spec citations for the subtle points (empirical honesty)

These are the rules most often implemented wrong; each is cited so the implementer can verify against the normative text rather than this prose.

1. **Fixed vs. arbitrary cardinality.** SPARQL 1.1 §18.4.2.4 (path evaluation) + §18.5 (translation). Fixed-length paths translate to BGPs → **bag** semantics (duplicate routes are distinct solutions). Arbitrary-length `*`/`+` use **ALP** → **set/DISTINCT** semantics. The W3C `pp` test suite encodes this difference explicitly (e.g. tests where `:a/:b` yields duplicates but `:p+` does not). **Verified in this engine** by the spargebra split (§1) + the dedup in `eval_path`.
2. **ALP visited-set / reachability not counting.** §18.4.2.5 `ALP(x, path)`: maintains a node set `V`, adds `x`, then for each `(x,n)` in `eval(path)` recurses on `n` **iff** `n ∉ V`. Output is the accumulated `V`. → reachability of distinct nodes, never path multiplicity, and cycles terminate. **Matches** `directed_reach` / `transitive_closure_pairs` visited sets.
3. **ZeroLengthPath domain.** §18.4.2.5: `ZeroLengthPath(X, Y)` over a graph binds `X=Y` to each term in `subjects(G) ∪ objects(G)` (plus the bound term if a constant). → reflexive over data nodes only. **Matches** `graph_nodes()` and the constant-intern path.
4. **Negated property set with inverse.** §18.4.2.4 / grammar `PathNegatedPropertySet`: `!(:a|^:b)` = forward edges with predicate ∉ {:a} **alternated with** reverse edges with predicate ∉ {:b}. **Matches** the parser normalization to `Alternative(NegatedPropertySet([:a]), Reverse(NegatedPropertySet([:b])))` (`parser.rs:1932`).
5. **`p*` includes the zero-length even with no `p` edges.** Already cited (3); the constant-endpoint case is the trap and is handled (`exec.rs:2029`).

**Flagged as the only residual semantic risk to re-verify under test (not a known bug):** the interaction of zero-length `*`/`?` with the **named-graph / dataset view** scope. `eval_path` gates the data-pair computation behind `!view::default_is_empty()` (`exec.rs:1993`) and still emits constant zero-length rows when the default graph is "empty" (L1 dataset view). This is subtle — the zero-length solution must come from the **active graph's** node domain, and a GRAPH-scoped path must use that graph's nodes. The W3C suite's path tests are default-graph; **GRAPH-scoped `*`/`?` zero-length over named graphs is under-tested** and should get a hand-written case (see §6, Wave C).

---

## 4. Planner & cardinality

### Current state
Path evaluation is **not** integrated into the binary-join GOO planner (`eval_bgp_binary`, `exec.rs:2675`) or the WCOJ router (`exec.rs` `eval_bgp_wcoj`). A `GraphPattern::Path` is its own algebra node evaluated by `eval_path`, producing a `Bindings`; it is joined to the rest of the query by the generic `Join`/`LeftJoin` operators **as an opaque relation** with no cardinality estimate fed back to the join order. spargebra's parse-time translation means *fixed* paths (the common case) **do** participate in BGP planning (they are triples by then), so the un-costed cases are exactly the recursive/alternative/negated ones.

### Costing a path step (design for the remaining slice)
A path node's output cardinality is bounded and estimable from per-predicate characteristic stats the planner already keeps (`exec.rs:2704` cardinality estimation uses distinct-subject/object counts per predicate):

- **`X p+ ?o` / `?s p+ X` (one bound end):** estimate = expected forward (resp. backward) reachable set size. Cheap proxy: `min(|nodes|, avg_out_degree(p)^d_eff)` is unstable; better to use the **directed BFS is already cheap** — for planning, treat a one-bound recursive path as cardinality ≈ a small multiple of `distinct_objects(p)` (it is bounded by `|nodes|`). Order it like a medium-selectivity scan.
- **`?s p+ ?o` (both unbound):** worst-case cardinality `O(V²)`; estimate as `distinct_subjects(p) * distinct_objects(p)` capped at `|nodes|²`. This should be ordered **last** among joinable patterns (it is the dominant cost), exactly as the prompt requests.
- **`X p+ Y` (both bound):** cardinality ≤ 1 (a boolean reachability test); order it like a maximally-selective filter — ideally evaluated as a semijoin/early-exit.

The hook: extend the per-pattern cost function (`goo_seed` / `goo_pick`, `exec.rs:2816`+) to recognize a `Path` pattern and assign it one of the three buckets above based on which endpoints are bound *at plan time* (a bound endpoint may itself be supplied by an earlier join — so the bound/unbound classification must be **dynamic**, recomputed as variables become bound during the greedy ordering, mirroring how bind-join eligibility is already computed). This is a planner change, not an evaluator change, and is the **only** genuinely new engine logic the remaining work needs.

### Bounding the unbounded-`*` risk (memory)
The spec **requires full reachability** (you cannot iterative-deepen and stop early for the both-unbound case without changing the answer set). So the worst case is genuinely `O(V²)` output. The engine bounds it by:
- **Visited-set per source** → time per source is `O(E)`, total `O(V·E)`; the *output* set is the true bound and equals the answer (no over-allocation).
- **Budget guard** (`exec.rs:2455` checks `budget::exhausted(out.len())` per start and every 1024 expansions inside the walk) → a hosted/WASM caller with `max_rows`/`deadline` installed gets a clean error/truncation instead of OOM. **With no budget installed there is no cap** — this is the documented, accepted worst case (the answer set is simply that large).
- **Recommendation:** the server/wasm entry points should install a default `max_rows` budget for queries containing a both-unbound recursive path, so an accidental `?s :related* ?o` on a dense graph fails fast rather than OOMs. This is a config/entry-point change, **not** an algorithm change. Document the worst case (`O(V²)` pairs) in the user-facing query-limits docs.

---

## 5. Reuse vs. dedicated BFS: the ALP-kernel decision

The prompt asks whether `*`/`+` should reuse `sparq-reason`'s transitive-closure machinery or use a dedicated per-seed BFS. **Recommendation: keep the dedicated BFS in `exec.rs`; do NOT couple property-path ALP to `sparq-reason`.** Reasoning, grounded in the actual code:

- `sparq-reason`'s reusable kernel is `transitive_closure(direct: &FxHashMap<Id, Vec<Id>>) -> FxHashMap<Id, Vec<Id>>` (`crates/sparq-reason/src/rdfs.rs:128`). It is `pub(crate)`, a clean standalone per-source DFS — **algorithmically identical** to the engine's `transitive_closure_pairs` (`exec.rs:2455`). Sharing it would mean making it `pub`, exporting it, and adding a `sparq-reason` dependency edge to `sparq-engine` for **zero algorithmic gain** (same DFS) while **losing** the engine's pushdown features that the reason kernel lacks: bound-endpoint single-source traversal, early-exit reachability tests, the budget hooks, and direction (`Dir::Rev`).
- The genuinely-faster reason path is the **linearized prp-trp** (`crates/sparq-reason/src/owl.rs:214`, the `O(N²)`-on-a-chain generator-edge form, 92× speedup per commit `579a0d4`). But it is **tightly embedded in the OWL fixpoint loop** (delta-driven, generator-membership tracking, semi-naive batching) and is **not reusable without extraction**. Its win is over the *nonlinear `R⋈R` join*, not over BFS — and the engine's both-unbound closure is *already* a BFS (linear per source), so the linearization win does not apply to the engine's structure.
- **Verdict:** the two transitive closures are siblings, not a shared dependency. Reuse here would be coupling for its own sake. If anything, the *direction* of any future de-dup should be the reverse: a standalone `sparq-core`-level `transitive_closure_pairs`/`directed_reach` utility that **both** `sparq-engine` and a refactored `sparq-reason` could call — but that is a separate refactor with its own risk budget and is **out of scope** for property paths. **Recommend: leave both in place, add a one-line cross-reference comment in each pointing at the other so a future maintainer knows they are siblings.**

The bind-join / WCOJ machinery is for *triple-pattern* joins; a path node is opaque to it. The only WCOJ/bind-join interaction is the planner-ordering hook in §4 (treat the path's output `Bindings` as a relation to bind-join against), which needs no change to the WCOJ kernel.

---

## 6. Correctness test plan

### 6.1 W3C SPARQL 1.1 property-path suite — already vendored and green
- **Vendored:** yes (fetched, not committed). `scripts/fetch-conformance.sh` clones `w3c/rdf-tests` at pinned commit `f25dbc0…` into `tests/w3c/rdf-tests/` (gitignored at `.gitignore`). The path group lives at `tests/w3c/rdf-tests/sparql/sparql11/property-path/manifest.ttl` and is auto-discovered via the `mf:include` chain from `sparql11/manifest-sparql11-query.ttl` (registered in `crates/sparq-conformance/src/main.rs` `GROUPS`).
- **Runner:** `cargo run --release -p sparq-conformance`; manifests parsed by `manifest.rs::collect` (follows `mf:include`), each `QueryEval` run via `sparq_engine::query()` on a 20s watchdog (`run.rs`), results compared by `compare.rs` with **bag** semantics by default and **sequence** semantics when `ORDER BY`/`rs:index` present, with blank-node bijection.
- **Current result:** `sparql11/property-path`: **33 pass / 0 fail / 0 divergence / 0 skip = 100%**. No skip-list excludes path tests; there is no path-specific xfail.
- **CI gate:** ratchet `PASS + DIVERGENCE ≥ 1229` in `.github/workflows/ci.yml`; a path regression fails CI.

### 6.2 Hand-written ALP edge cases (the gap)
The W3C suite is small (diamond/clique/loop fixtures) and does **not** stress the cases that this engine's optimizations could break. Add a dedicated test module (`crates/sparq-engine` `mod path_tests` already exists at ~`exec.rs:6963`; extend it, or add `crates/sparq-engine/tests/property_paths.rs` for larger graphs). Required cases:

- **Cycles & DISTINCT:** a 3-cycle `a→b→c→a`; assert `a :p+ ?o` = `{a,b,c}` (each once), `a :p* ?o` = `{a,b,c}` (a via zero-length **and** via the cycle — must appear once).
- **Self-loop:** `a :p a`; assert `?x :p+ ?x` includes `a` (1-cycle), and `a :p* ?o` = `{a}`.
- **Zero-length on empty/absent:** `<x> :p* <x>` on an empty graph → 1 solution; `<x> :p* ?o` on empty graph → `{?o=<x>}`; `?s :p? ?o` reflexive domain = data nodes only (not predicates, not unused IRIs).
- **The spec's explicit loop case:** the §18.4.2.5 example graph (`:a :p :b`, `:b :p :a`) — `:a :p* ?x` and `?x :p* ?y` counts — to pin the visited-set behavior.
- **Pushdown equivalence (differential):** for a battery of random small graphs, assert `eval(X p+ ?o)` equals `eval(?s p+ ?o)` filtered to `?s=X` — i.e. pushdown returns the **same set** as the all-pairs path filtered. (Commit `2dbfca1` added a differential test in this spirit; **extend it** to cover `Reverse`, nested `Sequence`-in-`+`, `Alternative`-in-`*`, and `NegatedPropertySet` under pushdown.)
- **Negated-inverse:** `?s !(:a|^:b) ?o` on a graph with `:a`, `:b`, `:c` edges — assert it = (forward non-`:a` edges) ∪ (reverse non-`:b` edges).
- **Same-var SCC correctness:** a graph with two disjoint SCCs and a DAG tail; assert `?x :p+ ?x` = exactly the SCC members + self-loops (validates `cyclic_nodes` Kosaraju against a brute-force `s==o` filter of the all-pairs closure).
- **GRAPH-scoped zero-length (the §3 flagged risk):** `SELECT * WHERE { GRAPH ?g { ?x :p* ?y } }` over two named graphs with disjoint node sets — assert each graph's zero-length domain is its **own** nodes. This is the one case most likely to be wrong; it is the first thing the implementer should verify.
- **Budget truncation:** with a tiny `max_rows` budget installed, a dense both-unbound `?s :p+ ?o` returns the documented error rather than OOM (and the error is the same shape as other budget errors).

### 6.3 Performance regression guard
There is a paired bench from commit `2dbfca1` (path pushdown). Extend `bench/` with one adversarial case: a star/social graph where `:alice :knows+ ?x` must stay `O(E_reachable)` (assert it does **not** materialize the all-pairs closure — measure peak rows or wall-time ratio vs. the unbound case).

---

## 7. Implementation wave plan (precise, gated, smallest-first)

The implementation agent executes this **after** `exec.rs` is free. Because the evaluator is already complete, the waves are *hardening + planner + tests*, ordered smallest-and-safest first, each independently shippable and gated by the conformance ratchet (which must never regress).

**Files in play:** `crates/sparq-engine/src/exec.rs` (evaluator + planner), `crates/sparq-engine/tests/property_paths.rs` (new, or extend in-file `mod path_tests`), `bench/` (one new case), `crates/sparq-conformance/*` (no code change — already wired), `crates/sparq-zk/src/verify.rs` (no change — §8). **No `spargebra` change needed** (algebra + parser are complete).

### Wave A — Test hardening (no engine code change; ship first)
- Add the §6.2 hand-written ALP edge cases as a new test module. This **characterizes** the current behavior and turns any later refactor regression into a red test.
- **Critical sub-step (do first):** the GRAPH-scoped zero-length test (§3/§6.2). If it fails, that is a real bug and Wave A becomes a fix, not just a test.
- **Gate:** `cargo test -p sparq-engine` green; conformance ratchet unchanged. Ship.

### Wave B — Differential & pushdown-equivalence battery (no engine code change)
- Extend the commit-`2dbfca1` differential test to cover `Reverse`, nested `Sequence`-in-recursive, `Alternative`-in-recursive, and `NegatedPropertySet` under all four endpoint-binding cases (bound-s, bound-o, both, neither), asserting pushdown ≡ all-pairs-then-filter.
- **Gate:** green + ratchet. Ship.

### Wave C — Planner cardinality hook (the one real engine change)
- Add `Path`-node recognition to the cost function (`goo_seed`/`goo_pick`, `exec.rs:2816`+): classify into the three buckets (§4) using **dynamic** bound/unbound endpoint analysis (a variable is "bound" if produced by an already-placed pattern). Order both-unbound recursive paths last; treat both-bound as a near-zero-cardinality semijoin.
- This **does not** change `eval_path` — it only changes join order, so the conformance suite (which tests result *sets*, order-insensitively for non-`ORDER BY`) is unaffected, while costed plans improve.
- **Gate:** ratchet unchanged (results identical) + a planner unit test asserting the chosen order for a query mixing a selective triple and a both-unbound path puts the path last. Ship.

### Wave D — Budget-by-default at entry points (config, not algorithm)
- At the server/wasm query entry points, install a default `max_rows` budget when the parsed algebra contains a both-unbound recursive `Path` (cheap algebra walk). Make it configurable/overridable. Document the `O(V²)` worst case in query-limits docs.
- **Gate:** the §6.2 budget-truncation test; ratchet unchanged (the suite's graphs are tiny, well under any sane default). Ship.

### Wave E (optional, lowest priority) — sibling-kernel cross-reference
- Add a one-line doc comment in `exec.rs:2455` and `crates/sparq-reason/src/rdfs.rs:128` noting the two are independent transitive-closure siblings (per §5), so a future maintainer doesn't accidentally couple them. **Do not** extract a shared kernel in this phase.
- **Gate:** docs only. Ship or defer.

**Order rationale:** Waves A/B are pure tests (zero regression risk, immediate safety net) and surface the GRAPH-scope risk early. Wave C is the only behavioral engine change and is result-preserving by construction. Wave D is operational safety. Wave E is hygiene. The expensive all-pairs case (`?s p* ?o`) is *already* implemented, so "all-pairs last" here means **planner-orders it last** (Wave C), not "implement it last."

---

## 8. ZK interaction

**Keep `Op::Path` fail-closed.** `crates/sparq-zk/src/verify.rs:138` already rejects `GraphPattern::Path { .. } => Err(UnsupportedFragment("property path"))`, and the test at `verify.rs:304–331` (`outside_fragment_fails_closed`) asserts both that a surviving `:a+` path is rejected as `"property path"` **and** that a parser-flattened `:a/:b` sequence is rejected via the **blank-node-in-pattern** guard (`verify.rs:312–315`) — because spargebra turns `:a/:b` into a BGP with a fresh blank node midpoint (§1), and the zk fragment rejects blank nodes in patterns. **Both rejection paths are correct and must stay.** A future ZK proof of property paths would need to prove a *traversal* (variable-length, data-dependent number of steps) inside the circuit — fundamentally harder than the stage-1 BGP fragment and **out of scope**. No change to `verify.rs` is part of this plan; the implementation waves above must **not** weaken either rejection. Add a test-comment cross-reference so a future ZK-path effort knows the blank-node guard is load-bearing for sequence paths.

---

## 9. Honest risks & unknowns

1. **GRAPH-scoped zero-length `*`/`?` (highest-confidence real risk).** The reflexive domain for a path inside `GRAPH ?g { … }` must be that graph's nodes, and `eval_path`'s `view::default_is_empty()` gating (`exec.rs:1993`) plus the constant-intern path are subtle. The W3C suite does not cover this. **Wave A must test it first**; it may be a latent bug. Flagged, not asserted — I did not run the engine.
2. **No execution was run.** This is a read-only code audit. The "33/33 W3C pass" figure comes from the existing `conformance-report.md` / `research/property-paths-assessment.md`, not a fresh run in this session. The implementer should run `./scripts/fetch-conformance.sh && cargo run -p sparq-conformance` to confirm before relying on it.
3. **Both-unbound `O(V²)` is unbounded without a budget.** Accepted and documented; mitigated by Wave D. On a dense graph with no budget installed, `?s :p* ?o` can OOM. This is spec-faithful (the answer set is that large) but operationally dangerous.
4. **Planner classification is dynamic, not static.** A path endpoint can become bound by an earlier join, so Wave C must recompute bound/unbound during greedy ordering, not once up front. Getting this wrong degrades plans (slower) but not correctness (results are order-insensitive except under `ORDER BY`, which is applied after).
5. **`SEQ_MIDPOINT_FANOUT_LIMIT = 1024` is a hand-tuned constant** (`exec.rs:2086`) with no adaptive basis. On graphs with midpoint fan-out just under/over 1024 the strategy flips; the threshold has not been swept. Low impact (both branches are correct), but a perf cliff exists. Not in the wave plan; note for future tuning.
6. **The `directed_reach` composite-sub-path arm** (`exec.rs:2257`, the `None` pid case for `(:a/:b)+` etc.) does one bounded sub-evaluation **per node** and filters — correct per the contract, but potentially quadratic for deeply-nested recursive sub-paths. Untested at scale; the §6.3 bench should include one nested case.
7. **Reason-kernel reuse was declined (§5).** If a future maintainer wants a single transitive-closure implementation across the codebase, the recommended direction (lift to `sparq-core`, have both call it) is a real refactor I have *not* designed here — only flagged as the correct direction if the duplication ever becomes a maintenance problem.

---

*Authored by Opus 4.8 (Fable 5 unavailable) — re-review when Fable returns. Read-only audit; no engine code modified; no builds or query runs executed in this session.*
