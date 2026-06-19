# SPARQL 1.1 property-path support — assessment

Date: 2026-06-12 (original review). Scope: `crates/sparq-engine` (algebra from `spargebra`, execution in `exec.rs`).

> **Update (2026-06-12, commit `47d43294` "perf(engine): push bound endpoints into
> property-path evaluation"):** the three ranked performance improvements at the end
> of this document have since LANDED. Bound endpoints are now pushed into a
> single-source directed traversal (`directed_reach`), `?x p+ ?x` resolves via SCC
> (`cyclic_nodes`) rather than the all-pairs closure, the budget is checked *inside*
> the traversal, and the full-store scans for the zero-length node domain /
> `NegatedPropertySet` are avoided when an endpoint is bound — `p*`/`p?` emit only the
> bound node's reflexive pair, and `!(...)` skips excluded predicate blocks wholesale
> on the P-sorted permutation. **Sections 1–4 describe the PRE-optimisation code; their
> line numbers are stale** and they are retained as the design record. §5 summarises the
> behaviour as shipped. (sq-5kr, [OPUS-4.8])

## Verdict

**COMPLETE (semantics) / COMPLETE (bound-endpoint performance, since `47d43294`)** —
every SPARQL 1.1 path operator is evaluated natively and the W3C
`sparql11/property-path` suite passes 33/33. At the time of the original review,
evaluation always materialised the path's *entire* (start,end) relation before
applying bound endpoints, so bound-endpoint recursive paths (`:alice :knows+ ?x`) cost
the same as fully-unbound ones: a whole-graph transitive closure. That pathology has
since been fixed — see the update note above and §5.

## 1. Per-operator status

Dispatch: `GraphPattern::Path { subject, path, object }` → `eval_path` at `crates/sparq-engine/src/exec.rs:1594`; the recursive evaluator is `path_pairs` at `exec.rs:2037`. All operators are **evaluated natively** (set-semantics pair relations over dictionary ids); none are rewritten to BGPs and none are rejected.

| Operator | spargebra variant | Status | Where (exec.rs) | How |
|---|---|---|---|---|
| `iri` (PredicatePath) | `P::NamedNode` | Native | 2040, `predicate_pairs` 2091 | Range scan on `[None, Some(pid), None]` → P-leading permutation (PSO/POS); empty set if predicate not in dict |
| `^p` (InversePath) | `P::Reverse` | Native | 2041 | Recurse, swap every `(s,o)` → `(o,s)` |
| `p1/p2` (SequencePath) | `P::Sequence` | Native | 2042–2057 | Hash join of the two sub-relations on the midpoint (build `by_start` map on right side) |
| `p1\|p2` (AlternativePath) | `P::Alternative` | Native | 2058–2062 | Set union of sub-relations (`FxHashSet::extend`) |
| `p*` (ZeroOrMorePath) | `P::ZeroOrMore` | Native | 2064–2068 | `transitive_closure_pairs` + reflexive `(n,n)` for every node in `graph_nodes` (full-store scan, 2109) |
| `p+` (OneOrMorePath) | `P::OneOrMore` | Native | 2063, closure at 2122 | Full transitive closure of the sub-relation |
| `p?` (ZeroOrOnePath) | `P::ZeroOrOne` | Native | 2069–2073 | Sub-relation + reflexive pairs over `graph_nodes` |
| `!(...)` (NegatedPropertySet) | `P::NegatedPropertySet` | Native | 2074–2086 | Full-store scan `[None, None, None]`, drop triples whose predicate id is in the excluded set. Reverse-negated members arrive as `Reverse(NegatedPropertySet)` from spargebra normalisation and compose via the `Reverse` arm |

Endpoint handling in `eval_path` (1934–2034): variables and blank-node patterns become columns; concrete terms absent from the dictionary are unsatisfiable except for the zero-length solutions of `*`/`?`, which are produced even on an empty graph by interning the constant locally (1964, 2008–2032). Same-variable endpoints (`?x p+ ?x`) filter to `s == o` (1993). Solutions are deduplicated (set semantics) via a `seen` hash set (1987, 2003).

## 2. Recursive-path algorithm (`*` / `+`)

`transitive_closure_pairs`, exec.rs:2122–2147:

- **Algorithm**: materialise the base relation (one predicate-range scan per `NamedNode` leaf), build an adjacency map `Id -> Vec<Id>`, then a **DFS (stack-based, labelled BFS in the doc comment) from every start node** that appears as a subject in the base relation. Output is the full closure as an `FxHashSet<(Id, Id)>`.
- **Cycle handling**: per-start `seen: FxHashSet<Id>` visited set (2135–2138) — cycles terminate, and the global `seen` row-set in `eval_path` gives DISTINCT semantics, so cyclic graphs are handled correctly.
- **Bound endpoints are NOT pushed down.** `eval_path` computes `path_pairs` for the whole graph first, then filters rows against `s_bound` / `o_bound` (1991–1995). The doc comment at exec.rs:1928–1933 says this explicitly: "Correctness-first: the relation is materialised … a future optimisation can push a bound endpoint into a directed traversal." So there is no single-source BFS, no bidirectional search, no reverse traversal for a bound object.
- **Sorted permutations**: used only at the leaf level — `predicate_pairs` (2091) does a sorted range lookup `[None, Some(pid), None]` against the six built permutations (`sparq-core/src/store.rs:36`, `choose` at 578), so collecting a predicate's edges is a contiguous range, not a whole-store scan. The closure itself works on hash sets/maps and ignores sort order. `NegatedPropertySet`, `graph_nodes`, and the zero-length node domain do full `[None,None,None]` scans.
- **Both endpoints unbound**: identical code path — full all-pairs closure, plus (for `*`/`?`) reflexive pairs over every node found by a full-store scan. This is the spec-correct answer set, just computed eagerly.
- **Runaway guard**: a coarse, sticky budget check once per BFS start (`budget::exhausted(out.len())`, 2132; impl at exec.rs:119) breaks out under a row-count or deadline budget, so hosted/WASM callers with limits won't spin forever — but with no budget installed there is no cap.

## 3. Test and conformance coverage

- **Unit tests**: `mod path_tests` in exec.rs (~6963–6998) — a 4-node `:p` chain plus a `:q` edge covering `+`, `*` (incl. reflexive count), `/`, `|`, `?`, `^p`, `^p+`, and `!:p`, with bound-subject and fully-unbound variants. Zero-length constant-endpoint semantics are exercised through `eval_path`'s zero-row logic (asserted counts include the reflexive solutions).
- **W3C conformance**: `conformance-report.md` (sparq commit `9917404`, rdf-tests `f25dbc0`; 1229 tests run) reports **`sparql11/property-path`: 33 pass / 0 fail / 0 divergence / 0 skip = 100.0%**. The rdf-tests data is vendored at `tests/w3c/rdf-tests/sparql/sparql11/property-path/` (diamond/clique/loop fixtures present), run by `crates/sparq-conformance`.
- No dedicated large-graph or adversarial path benchmarks were found (nothing path-related in `bench/` or `research/`).

## 4. Likely cost pathology at scale

For `?x :knows+ ?y` — or, critically, even `:alice :knows+ ?x` — on a graph with `V` people and `E` knows-edges:

1. `predicate_pairs` materialises all `E` edges (fine).
2. `transitive_closure_pairs` runs a traversal **from every distinct subject**: `O(V·E)` time and up to `O(V²)` pairs of memory in dense/social graphs. On a small-world graph where most nodes reach most others, the closure is ~`V²` 16-byte pairs — 100k nodes ⇒ ~10^10 pairs, i.e. memory blow-up long before time-out.
3. Only *after* that does `eval_path` throw away everything not matching the bound endpoint — so a query whose answer is one node's ~k-hop neighbourhood (`O(E)` with a directed BFS) pays the full all-pairs price.
4. `p*`/`p?`/`!(…)` additionally do full-store scans for the node domain, and `ZeroOrMore` with two unbound endpoints adds `V` reflexive pairs on top of the closure.
5. Sequences of recursive steps (`p+/q+`) materialise each side's full closure before the hash join, multiplying the effect.

The budget guard (max-rows/deadline) converts the blow-up into an error/truncation when limits are installed, but does not make bound-endpoint queries fast.

## 5. Current behaviour (as shipped, commit `47d43294`)

All three improvements that the original review ranked below have LANDED; the
pathology in §4 no longer applies to bound-endpoint queries. Code anchors are
functions (not line numbers) so they stay valid as the file moves:

- **Bound endpoints pushed into a directed traversal** (was ranked #1). `eval_path`
  hands a `PathEnds { s, o }` hint into `path_pairs`, which for `+`/`*`/`?` (and,
  recursively, through `Reverse`/`Sequence`/`Alternative`) runs `directed_reach`: a
  single-source, budget-checked BFS whose frontier expansion is a sorted range scan
  (`[Some(s), Some(pid), None]` forward, `[None, Some(pid), Some(o)]` reverse for a
  bound object). The both-bound case is a reachability TEST that stops on hit. Turns
  the dominant `:s p+ ?x` case from `O(V·E)` all-pairs into `O(E_reachable)`. The
  contract on `PathEnds` makes the pushdown a pure optimisation: a sub-evaluation may
  ignore the hint and return extra pairs, and `eval_path` post-filters as the
  correctness backstop.
- **Early termination + per-SCC same-variable case + in-traversal budget** (was
  ranked #2). `?x p+ ?x` resolves via Kosaraju SCC (`cyclic_nodes`, `O(V+E)`) rather
  than the all-pairs closure; the zero-length operators' diagonal is exactly the node
  domain. The row/deadline budget is now checked *inside* `directed_reach` (every 1024
  pops), so a single runaway traversal respects it promptly — pinned by
  `budget_fires_inside_directed_traversal`.
- **No full-store scans for the zero-length node domain / `NegatedPropertySet`** (was
  ranked #3, this bead — sq-5kr). `path_pairs` emits only the bound node's reflexive
  pair for `p*`/`p?` (the `graph_nodes()` whole-store scan now runs *only* for a
  genuinely both-unbound `*`/`?`, where the answer truly IS the whole node domain, and
  for the `?x p*/p? ?x` diagonal). `negated_property_pairs` narrows to the bound
  endpoint's triples when one end is bound, and for the both-unbound case walks the
  P-leading permutation (`scan_sorted(&[None,None,None], 1)`) and skips each excluded
  predicate's contiguous block wholesale via `partition_point`, instead of a
  `[None,None,None]` scan that probes the excluded set per triple.

Regression coverage lives in `mod path_pushdown_tests` (and `mod path_tests`) in
`exec.rs`:
`pushdown_matches_full_closure_for_all_operators_and_binding_shapes` (every operator,
incl. `:e*`/`:e?`/`!:e`/`!(:e|:f)`, against the full-closure oracle for subject-bound,
object-bound, both-bound and same-variable shapes) and
`negated_property_set_matches_filter_oracle` (the block-skip scan vs a plain
scan + predicate `FILTER`).

## Remaining (out of scope here)

The both-unbound recursive case is still the spec-correct eager all-pairs closure
(§4 still applies when NEITHER endpoint is bound) — a streaming/lazy both-unbound
closure remains a possible future improvement, but is a separate, larger change.
