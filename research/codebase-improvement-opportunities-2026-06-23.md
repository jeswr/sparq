# Codebase Improvement-Opportunity Survey — 2026-06-23

> 🤖 **SPARQ agent** — this is an autonomous, *grounded* improvement-opportunity survey.
> It synthesises 101 raw candidate opportunities from 13 parallel grounded probes into a
> de-duplicated, novelty-re-verified, honestly-scored design record. Every item below was
> checked against `origin/main`, `bd list`, and the existing `research/` corpus; items that
> are already implemented, already beaded, or that rested on a *false premise* are recorded in
> the "Dropped / corrected" section with the reason. **No performance numbers are baked in** —
> every perf item states a measurable hypothesis and the metric that would confirm it; every
> accuracy item cites a concrete wrong-or-missing behaviour. This is a design-for-review
> record; nothing here is implemented.

## TL;DR

The honest headline: **much of the obvious improvement surface is already covered.** Of 101 raw
candidates, the survey collapses to a much smaller set of *genuinely-open* wins once you remove
(a) duplicates across probes, (b) things already on `main`, (c) things already beaded — often at
an invisible P3/P4 priority — and (d) several items that rested on a factually-wrong premise
about the current codebase.

The single most concentrated cluster of real, high-leverage, buildable-now wins is in the
**storage / scan layer** (`sparq-core`), where the maintainer's own research
(`research/data-structures.md`, `research/optimization-techniques.md`) already says *adopt now*
but the code does **not** yet implement: per-block Bloom filters, Elias-Fano compressed-seek
columns, and the exact-bitmap semi-join reducer. These are designed-and-prioritised but
unbuilt. The second cluster is a small number of **standalone correctness/functionality gaps**
(RDF/XML parse, numeric/boolean TSV abbreviation, streaming Turtle serialisation, N3
`log:semantics` cycle-safety, the `MULTIPLICITY()` algebra-device vendor extension). *Two of
this cluster's original premises were corrected during the build phase — see §B2 and §C1.*

Several headline "opportunities" in the raw set were **false-premise** and are dropped:
SPARQL Protocol is *already* implemented in `sparq-server`; `$currentShape` is *already* bound
in `sparq-shacl`; per-predicate coverage and samples are *already* rendered in the
`sparq-introspect` text summary.

---

## How to read this

- **Current state** cites `file:line` on `origin/main` so a reviewer can confirm the gap.
- **Feasibility** is `buildable-now` (no external gate), `ec2-gated` (needs a quiet
  canonical host to measure), `maintainer-gated` (needs a design call), or `research-first`.
- **Measured by** names the metric that would confirm the payoff. Per the repo's honesty
  rules, work-box/EC2 timings are **non-canonical**; perf claims must be confirmed on the
  canonical perf-validation host (`sq-0g6g` gate) before any number is asserted anywhere.
- Items are grouped by axis (performance / functionality / accuracy), then a
  **Buildable-now shortlist** and a **Deferred** section rank across axes.

---

## A. Performance

### A1. Per-block Bloom filters on high-NDV (subject/object) permutation columns
- **Current state.** `crates/sparq-core/src/compress.rs` builds fixed-count delta+varint
  blocks with a sparse `(first-triple, byte-offset)` directory (see the module doc, lines
  1–6, and `CompressedPerm`, line 102). The directory's `first triple` gives an *implicit*
  min/max zone map that prunes **range** scans, but there is **no Bloom filter** — for an
  equality-bound constant whose id falls inside several overlapping blocks, nothing skips a
  block. `research/data-structures.md` §A2 (lines 111–123, 457+) flags this exact gap and
  says "prototype this first."
- **Opportunity.** Add an optional per-block Bloom bitset (a few bytes/block, 1–2 hashes);
  Bloom-check a block before `decode_block_at` on an equality-bound pattern. Zero false
  negatives by construction; the only cost of a false positive is one wasted block decode.
- **Approach.** Optional `Vec<Bloom>` (one per block) on the perm; populate during
  `from_triples_compressed`; gate the check in the `rows_in` scan path; only build filters
  for columns above a density threshold so dense predicates don't pay for filters that never
  skip.
- **Feasibility.** buildable-now. **Crate:** `sparq-core` (opt-in build-time). **Risk:** low
  (additive, correctness-neutral). **Measured by:** block-skip rate on selective
  equality-bound patterns (fraction of blocks skipped that the min/max map could not), and
  end-to-end latency on selective-filter queries (q06-shaped), confirmed on the canonical host.

### A2. Elias-Fano compressed-seek column codec (optional second codec)
- **Current state.** `compress.rs` perms are block-compressed (FoR+varint); a seek/range
  query must decode whole blocks even for a single row. `research/data-structures.md` §B1
  (lines 195–219) identifies Elias-Fano as "alone in this family supporting near-O(1)
  `NextGEQ(x)` directly on compressed data" and labels it "the one differentiated new bet"
  — but it is unimplemented.
- **Opportunity.** Implement an EF codec for the trailing column with a `next_geq(target)`
  method, route it as an alternate `PermData` variant, and call `next_geq` in the merge-join
  galloping / block-pruning hot paths instead of decode-then-gallop. EF is WASM-friendly
  (fewer bytes moved, scalar select).
- **Approach.** Standalone EF codec for u32 column deltas → expose `next_geq` → add a codec
  variant alongside the FoR+varint one → A/B vs ZSTD on scan throughput, seek-heavy join
  latency, and heap footprint, on native and wasm32. Keep ZSTD the default; pick EF per
  relation from build-time NDV/clustering stats.
- **Feasibility.** buildable-now (codec is self-contained) but **measurement-gated** — EF can
  *lose* latency on resident-cache streaming scans (pointer-chasing select/rank). **Crate:**
  `sparq-core` (opt-in codec choice). **Risk:** medium — must prototype plain-EF *and*
  Partitioned-EF and measure on multiple query shapes before adoption. **Measured by:**
  seek-heavy join latency, scan throughput, bytes-moved, and heap, A/B against ZSTD.

### A3. Exact-bitmap semi-join reducer on dense u32 ids
- **Current state.** `crates/sparq-engine/src/exec.rs` runs binary BGP joins with bind/merge
  strategies but has **no semi-join prefilter** that prunes a scan to ids reachable from the
  other side. `research/optimization-techniques.md` §1.1 / §2(a) (lines 63, 155+) rank exact
  bitmap semi-join (CIDR'26 "Not Yannakakis") the single best technique-architecture fit
  ("ADOPT NOW", small cost) — unimplemented in exec.rs.
- **Opportunity.** After materialising a join result, build a bitmap of reachable ids per
  join variable (one bit per id in the dense range; sparq's dict guarantees dense ids), pass
  it to the next scan, and skip rows whose join key is absent. Falls back to a Bloom filter
  for sparse/huge domains. This is a *scan filter*, distinct from per-row bind-join probing.
- **Approach.** Build bitmap inside `eval_bgp_binary`; teach `scan_to_bindings` to accept an
  optional pre-filter bitmap; gate behind a feature; measure on star/snowflake queries.
- **Feasibility.** buildable-now. **Crate:** `sparq-engine` (opt-in feature). **Risk:** low
  (filter only removes rows that would fail the join anyway). **Measured by:** intermediate
  row count reduction and end-to-end latency on star/snowflake workloads (canonical host).

### A4. Yannakakis full-semijoin prepass for acyclic BGPs (DAG-of-operators)
- **Current state.** `exec.rs` routes acyclic BGPs (`bgp_uses_binary`, ~line 3904) straight to
  binary join with no semi-join reduction. `research/optimization-techniques.md` §1.1 / §2(a2)
  (lines 64, 167+) prioritise Yannakakis via the "DAG of standard operators" recipe (arxiv
  2504.03279) — built from sparq's existing sorted-merge intersections, no bespoke reducer.
- **Opportunity.** A bottom-up semijoin prepass over the acyclic join DAG that filters each
  pattern to rows whose keys survive in descendants, cutting intermediate materialisation
  before the main join. This is the *generalisation* of A3 across an acyclic plan.
- **Approach.** `yannakakis_prefilter(patterns, graph)` building the DAG (reverse topo order)
  and computing semijoin intersections via `merge_join` on each variable; call before
  `eval_bgp_binary` when acyclic; default-off feature.
- **Feasibility.** buildable-now. **Crate:** `sparq-engine` (opt-in). **Risk:** medium — the
  prepass must be cheaper than the savings (pure overhead when intermediates are already
  small). **Measured by:** prepass cost vs intermediate-size reduction on chain/snowflake
  queries with large intermediates (canonical host gate). *Relationship:* A3 and A4 share the
  bitmap/merge-intersection machinery; A3 is the localised one-join version, A4 the plan-wide
  version. Build A3 first; A4 reuses it.

### A5. Index-Based Join Sampling (IJBS) for correlated-join cardinality
- **Current state.** GOO seed/scoring in `exec.rs` (`goo_seed` ~line 4068, `goo_pick`
  ~line 4232) use single-pattern estimates × an *independence* selectivity product. RDF
  predicates are correlated (subject→object distributions), so the independence assumption is
  wrong by orders of magnitude on correlated joins. `research/optimization-techniques.md` §1.3
  (line 92) ranks IJBS (CIDR'17) a small-cost win that *reuses the six permutation indexes*.
- **Opportunity.** Sample the *join outcome* directly (random index probes) for the seed and
  first few candidates, replacing the independence-product estimate where it is most wrong.
- **Approach.** Optional `index_sample_join` step in `prepare_bgp`; small k (e.g. ~1000);
  feature-gated. Note the **honest caveat from `sq-p6p6`**: the *local* planner's per-pattern
  estimates are already index-exact, so the win is specifically on the *multi-pattern
  correlated* estimate, not single-pattern cardinality. Frame and measure it as such.
- **Feasibility.** buildable-now. **Crate:** `sparq-engine` (opt-in). **Risk:** medium —
  sampling is overhead when correlation is rare; estimate errors only affect ordering, never
  correctness. **Measured by:** q-error of multi-join estimates (the engine already computes
  per-operator q-error in `explain_json.rs`) on correlated WatDiv/Wikidata queries, and join
  order quality.

### A6. Vector-at-a-time merge-join kernel
- **Current state.** `merge_join` in `exec.rs` is row-at-a-time (pointer advance + compare +
  append). `research/optimization-techniques.md` §1.2 (lines 77+) names vector-at-a-time the
  "highest-leverage architectural move," currently missing.
- **Opportunity.** Batch left rows, branchlessly scan the sorted right side, gather matches
  into a selection vector; SIMD (128-bit, WASM-compatible) over u32 ids as a follow-on.
- **Approach.** Scalar batched kernel first (measure), SIMD second; default-off feature;
  fallback is the current kernel byte-identical when disabled. Existing
  `test_asserts_plans_agree` catches output bugs.
- **Feasibility.** ec2-gated (kernel-level perf needs a quiet box). **Crate:** `sparq-engine`
  (opt-in). **Risk:** medium — new kernel = correctness risk caught by plan-agreement tests.
  **Measured by:** per-kernel and end-to-end latency on large joins (canonical host).

### A7. Streaming Turtle/TriG serialisation (`impl Write`, memory-bounded)
- **Current state.** `crates/sparq-engine/src/serialize.rs` `write_turtle` (line 274) /
  `write_trig` (line 423) build a full in-memory `String` (buffer-all-triples). No
  `Writer`-based path exists. CONSTRUCT/DESCRIBE over large graphs hold both the graph and the
  fully-rendered string in memory at once.
- **Opportunity.** A `write_turtle_streaming<W: Write>` that takes a triple iterator and a
  subject-buffer (emit on subject change), preserving Turtle predicate-object grouping
  without materialising the graph; enables HTTP chunked streaming responses.
- **Approach.** Extract the existing rendering into a `Write`-based function; pre-compute the
  used-prefix set in one pass; keep the buffered `write_turtle` for back-compat; new opt-in
  `streaming-serialization` feature.
- **Feasibility.** buildable-now (mostly refactor). **Crate:** `sparq-engine` (opt-in).
  **Risk:** low (logic exists; round-trip tests guard). **Measured by:** peak RSS during
  serialisation of a large CONSTRUCT result, and time-to-first-byte for streamed responses.

### A8. JSON-LD chunk-parallel ingest via document-structure prescan
- **Current state.** `crates/sparq-core/src/lib.rs` (~lines 722–728) parses JSON-LD serially
  via `oxjsonld` ("whole-document JSON parse, no parallel chunking applies"). N-Quads/Turtle
  already chunk-parallel.
- **Opportunity.** Prescan top-level `@graph` / node-object boundaries (JSON brace/bracket
  nesting, SIMD-able), partition, parse chunks via `oxjsonld::JsonLdParser::for_slice`,
  merge per-chunk quads dataset-wide (mirroring `load_nquads_parallel`).
- **Approach.** Document scanner for top-level boundaries → per-chunk parse + accumulate →
  graph-scoped merge; stays behind the existing `jsonld` feature; no new deps.
- **Feasibility.** buildable-now but **correctness-sensitive** — JSON-LD `@context` merging and
  relative-IRI base are stateful; over-eager chunking can break semantics. **Crate:**
  `sparq-core`. **Risk:** medium — validate every chunked output against the serial path.
  **Measured by:** ingest throughput (Mt/s) vs the serial path on JSON-LD corpora, with a
  serial-equivalence oracle gating correctness.

### A9. Per-operator cost-category annotations in EXPLAIN / slow-query ring
- **Current state.** `crates/sparq-engine/src/explain_json.rs` records per-operator wall nanos
  and BGP q-error, and keeps a `SlowQueryRing`, but operators carry only a free-form label —
  no machine-parseable cost category, and the slow-query ring is not stratified by dominant
  cost.
- **Opportunity.** A small `CostCategory` enum (BGP-estimate / filter-selectivity /
  join-cardinality / materialise / sort) emitted alongside the label; lets the slow-query ring
  be filtered by dominant cost and gives honest training data for a cost model.
- **Approach.** Optional `cost_category` field on `PlanNode`, assigned deterministically from
  operator type; emit in `to_json`; add a category filter to `SlowQueryRing::slowest`.
- **Feasibility.** buildable-now. **Crate:** `sparq-engine` (ships in existing `explain-json`
  feature). **Risk:** low (additive, optional JSON field). **Measured by:** this is an
  *observability* improvement — payoff is qualitative (faster slow-query root-cause); validate
  by reproducing a known slow query and confirming the dominant category is correctly attributed.

### A10. N3 `log:semantics` recursive-import cycle detection (safety)
- **Current state.** `crates/sparq-reason/src/n3/mod.rs` exposes `Resolver = dyn Fn(&str) ->
  Option<String>` (line 155) for `log:semantics`/`log:content` with **no visited-IRI set**.
  N3 is Turing-complete; an adversarial graph + a *live* resolver with a self/indirect import
  cycle can spin indefinitely. (The conformance harness uses an offline pinned resolver, so CI
  never hits this — but a production live resolver can.)
- **Opportunity.** Track visited document IRIs in the resolution context; on a re-visited IRI,
  deterministically fail or treat the semantics as empty (decide against cwm reference
  behaviour). Cheap, standard cycle-detection.
- **Approach.** Thread a `visited: &mut HashSet<String>` through the resolver call path;
  add/remove on entry/exit; cover direct (A→A), indirect (A→B→A), and diamond re-use; document
  the chosen semantics in the N3 how-to.
- **Feasibility.** buildable-now. **Crate:** `sparq-reason` (no feature flag — it is a safety
  fix). **Risk:** low. **Measured by:** termination on adversarial cyclic fixtures (a
  correctness/safety test, not a perf metric). *Classified `performance`/safety because the
  failure mode is non-termination, not a wrong answer.*

---

## B. Functionality

### B1. RDF/XML parsing in `sparq-core`
- **Current state.** `crates/sparq-core/src/lib.rs` `parse_to_triples` (line 673) dispatches
  N-Triples / N-Quads / Turtle / TriG / JSON-LD; **RDF/XML is absent** — the format tests at
  lines 6634 / 6680 explicitly *reject* `"rdfxml"`. `sparq-server` already uses
  `oxrdfxml::RdfXmlParser` for conformance + GSP bodies, so the dependency is in-tree and proven.
- **Opportunity.** Callers can *serialise* RDF/XML but cannot *parse* it into a sparq `Graph`.
  RDF/XML is a W3C standard concrete syntax; this is a real conformance/interop gap.
- **Approach.** Wire `oxrdfxml::RdfXmlParser` into `parse_to_triples` under a new
  `is_rdfxml_format` matcher (`"rdf-xml"`, `"rdfxml"`, `"application/rdf+xml"`); serial-only
  (XML is not line-delimited); add a serialise→parse round-trip test. No new deps.
- **Feasibility.** buildable-now. **Crate:** `sparq-core`. **Risk:** very low (proven
  dependency, trivial dispatch). **Measured by:** round-trip correctness and W3C RDF/XML test
  pass count.

### B2. `MULTIPLICITY()` aggregate device — a vendor extension, *not* a 1.2 conformance gap
- **Premise corrected by the build phase (PR #1257, sq-v411r).** This item originally claimed
  "SPARQL 1.2 adds the standard `multiplicity()` function" and treated it as a *conformance* gap.
  That was **wrong**: there is **no callable `multiplicity()` builtin** in the SPARQL 1.2 grammar
  / `BuiltInCall` production, and **no W3C conformance suite** for one (verified against the
  pinned `w3c/rdf-tests @ f25dbc0` — no `multiplicity` test directory). "multiplicity" appears
  **only as algebra/semantics NOTATION** — the `card[Ω](μ)` device renamed `multiplicity(μ|Ω)` in
  the §18.4 BGP-matching and Set-Function definitions (see `research/sparql12-engine.md` §1.4).
- **Current state.** No `multiplicity` in the engine (the `multiplicity` references in
  `crates/sparq-engine/src/cs.rs` are the *cardinality-estimation* characteristic-set model, a
  different concept).
- **What landed.** sparq ships `MULTIPLICITY()` as a **clearly-labelled VENDOR EXTENSION**
  (reserved IRI `urn:sparq:fn:multiplicity`) that exposes the 1.2 algebra device inside an
  aggregate argument (e.g. `SUM(?x * MULTIPLICITY())`), gated behind `sparql-12`. It is **not** a
  conformance feature; the boundary is documented in the engine README, the `sparql-query` SKILL,
  and `vendor/spargebra/SPARQ-PATCHES.md` §10. PR #1257 verified the existing
  aggregate/grouping/subquery conformance lanes are unchanged (no regression).
- **Feasibility.** delivered (additive, side-effect-free). **Crate:** `sparq-engine`. **Risk:**
  very low. **Measured by:** existing aggregate/grouping/subquery suites stay green; there is no
  `multiplicity` conformance suite to pass.

### B3. RDF 1.2 triple-term (quoted-triple) support in the RDF/JS surface
- **Current state.** The engine parses/queries RDF-star, but the JS RDF/JS surface does not
  expose it: `js/src/sparql.ts` `termFromSparqlJson` handles only `uri`/`literal`/`bnode`;
  `parseNTriples` has no `<< s p o >>` recognition; `crates/sparq-engine/src/json.rs` SPARQL-JSON
  serialisation has no triple-term case.
- **Opportunity.** A JS consumer cannot work with quoted-triple results even though the engine
  supports them. Close the surface gap end-to-end (JSON results + N-Triples-star parse +
  `termToNT`).
- **Approach.** Extend the RDF/JS term model with a `Triple` term type (private extension
  pending RDF/JS spec evolution); teach `termFromSparqlJson` the `type: "triple"` nested shape;
  teach `parseNTriples`/`termToNT` the `<< … >>` syntax; add the SPARQL-JSON triple-term case
  in `json.rs`.
- **Feasibility.** buildable-now. **Crate:** `sparq-wasm` + `js/` (+ a small `sparq-engine`
  JSON change). **Risk:** moderate — a term can now be a triple, so JS type-narrowing that
  assumes four term types breaks; guard with the full RDF/JS conformance suite. **Measured
  by:** quoted-triple round-trip through JSON results + N-Triples, RDF/JS suite green.

### B4. SPARQL entailment regimes wired into BGP matching (opt-in)
- **Current state.** `sparq-reason` / `sparq-reason-el` compute closures in isolation; the
  engine's `eval_bgp` matches the *asserted* graph only. `research/sparql12-engine.md` §1.4 and
  the W3C entailment-regimes draft define applying RDFS/OWL entailment during pattern matching;
  unimplemented in the evaluator.
- **Opportunity.** Optional `EntailmentRegime` parameter on query entry points; under
  `RDFS`/`OWL` (RL via `sparq-reason`, EL via `sparq-reason-el`), match against the
  asserted+inferred closure; `None` (default) preserves current behaviour and perf.
- **Approach.** Thread a regime parameter like the `dataset` override is threaded; lazy/cache
  the regime closure in a `ReasonedGraph` wrapper; RDFS first, OWL after; gate behind an
  `entailment-regime` feature; test against the W3C entailment suite (separate from the query
  suite).
- **Feasibility.** buildable-now but **maintainer-gated** (couples query exec to the reasoning
  layer; correctness/perf cascade). **Crate:** `sparq-engine` + `sparq-reason`(-`el`). **Risk:**
  medium — must default to no entailment for predictable perf. **Measured by:** W3C entailment
  regime suite pass count per regime, and the asserted-only path staying byte-identical.

---

## C. Accuracy / Conformance

### C1. Spec-conformant numeric/boolean TSV abbreviation (F21)
- **Premise corrected by the build phase (PR #1258, sq-u79ee).** This item originally claimed
  "the store normalises numeric lexical forms" and proposed a `sparq-core` Dict change. A proven
  repro on current `main` shows that is **false**: the N-Quads/Turtle load + projection path
  **already preserves** the original lexical form (`"1.0E6"^^xsd:double` → `1.0E6`,
  `"1.0e6"^^xsd:double` → `1.0e6`), and a computed double already serialises canonical. So **no
  `sparq-core` Dict change was needed** — a preserved-lexical-form field would have been dead
  storage, and the per-unique-literal memory cost flagged below is **not** incurred.
- **What landed.** The single real, narrow gap was the SPARQL-Results **TSV** writer
  (`sparq_server::results::term_to_tsv`), which quoted **every** typed literal instead of
  abbreviating `xsd:integer`/`xsd:decimal`/`xsd:double`/`xsd:boolean` to their bare Turtle token
  per the [W3C CSV/TSV results format](https://www.w3.org/TR/sparql11-results-csv-tsv/), matching
  the oxigraph `sparesults` reference. CSV already wrote bare values; JSON/XML carry the full
  term — all unaffected.
- **`tsv03` stays a DOCUMENTED_DIVERGENCE (conformance-neutral).** The W3C `tsv03` expected file
  writes `1.0e6` for the data term `"1.0E6"^^xsd:double` — a *different* RDF term under identity
  projection. The conformance harness compares **parsed expected terms vs in-memory
  `QueryResult` terms**, never the serialised TSV string, so this fix is conformance-neutral
  there; `tsv03` remains a tracked divergence (now better explained), not a fixable failure.
- **Feasibility.** delivered. **Crate:** `sparq-server` (serialise only — *not* `sparq-core`).
  **Risk:** low — additive, no hot-path impact. **Measured by:** `csv-tsv-res` suite unchanged
  (2 pass / 0 fail / 1 documented divergence), no SPARQL result-format regression.

### C2. Predicate-selectivity-aware cardinality in non-star federated joins
- **Current state.** `crates/sparq-fedplan/src/plan.rs` `independence_estimate` (line 412+)
  uses a coarse `out = l*r/max(|L|,|R|)` ndv approximation (line 423) and **ignores** the
  `distinct_subjects`/`distinct_objects` stats the `SourceDescriptor` already holds for
  non-star joins.
- **Opportunity.** When connected and the candidate's source carries predicate partitions,
  compute a weighted per-predicate selectivity from the retained sources' distinct-object/
  -subject counts instead of the max-leaf heuristic; fall back to max-leaf when stats are absent.
- **Approach.** Fold the descriptor's distinct counts into `independence_estimate`; opt-in
  `PlanOptions::use_predicate_selectivity` (default the better estimate, with a switch to recover
  the conservative behaviour for A/B).
- **Feasibility.** buildable-now. **Crate:** `sparq-fedplan` (opt-in knob). **Risk:** medium —
  changes cost ordering, so bind-vs-hash decisions can flip; the multiset result is unchanged,
  only the cost comparison. **Measured by:** join-order plan quality and q-error on
  multi-predicate federated patterns; regression test comparing old vs new cost on a suite.

### C3. JSON-LD `expand` + `flatten` algorithms (new gateable conformance surfaces)
- **Current state.** `crates/sparq-conformance/tests/jsonld_suite.rs` (lines 61–62) explicitly
  documents `expand`, `flatten`, `html`, `remote-doc` as the algorithm categories sparq does
  **not** yet ship as gateable surfaces. `toRdf`/`fromRdf`/`compact`/`frame` lanes are gated and
  rising (the scoreboard registers JSON-LD jobs).
- **Opportunity.** Implement `expand` and `flatten` as output-side transforms over the existing
  `fromRdf` writer, then gate them with floors. Closes a real conformance hole and enables
  document-to-document transforms.
- **Approach.** `graph_to_jsonld_expanded` and `graph_to_jsonld_flattened`; wire `run_expand`/
  `run_flatten` (modelled on `run_compact`/`run_frame`); add `EXPAND_FLOOR`/`FLATTEN_FLOOR` to
  the registry; round-trip-validate (expand→compact reconstructs the same RDF).
- **Feasibility.** buildable-now. **Crate:** `sparq-engine`/`serialize-rdf` (behind `jsonld`).
  **Risk:** moderate — `fromRdf` exists; the Oracle handles output-shape differences via
  RDF-equivalence. **Measured by:** W3C JSON-LD expand/flatten suite pass counts at a pinned
  revision; aligns with the `sq-oy1f` JSON-LD epic.

### C4. JSON-LD framing improvements — raise the documented frame floor
- **Current state.** `jsonld_suite.rs` (~line 161, scoreboard line 140) gates frame at 61/92
  with 28 *documented honest divergences* clustered as: `@value`-alternative value-pattern
  matching, `@explicit`/`@default` fill/prune, named-graph `@graph` framing.
- **Opportunity.** These are implementable Framing-Algorithm features, not spec ambiguities;
  fixing the three clusters raises the floor.
- **Approach.** Categorise the 28; extend value matching for `@value` alternative arrays; align
  `@explicit`/`@default` with the normative algorithm; iterate `@graph` framing; re-measure,
  raise the floor, document any residue.
- **Feasibility.** buildable-now. **Crate:** `sparq-engine`/`serialize-rdf`. **Risk:** moderate
  — the algorithm is 22 steps; the W3C RDF-equivalence oracle is strong. **Measured by:** frame
  suite pass count rising past 61; under `sq-oy1f`.

### C5. Per-regime breakdown of the SPARQL entailment conformance lane
- **Current state.** The inference ratchet is a single combined number (RDF-MT / OWL 2 RL / N3
  / entailment / rdf-turtle) — `crates/sparq-conformance/src/scoreboard.rs` line 163. There is
  no per-regime (RDF / RDFS / OWL-RL) visibility for the SPARQL entailment cases.
- **Opportunity.** Instrument the entailment runner to report and gate pass counts *per regime*,
  exposing which regimes are weak and driving focused fixes.
- **Approach.** Per-regime floors + separate scoreboard rows; pure instrumentation/reporting,
  no new algorithm. **Feasibility.** buildable-now. **Crate:** `sparq-conformance` (reporting).
  **Risk:** low (accounting only — parse regime IRIs from manifests correctly). **Measured by:**
  per-regime pass counts published; a regime at 0% becomes an actionable, visible gap.

---

## D. Buildable-now shortlist (ranked across axes)

Ranked by value × feasibility × confidence. Value weights hot-path/real-gap/accuracy impact;
buildable-now beats gated.

1. **A1 — Per-block Bloom filters** (`sparq-core`). Maintainer-research "prototype first";
   small, additive, correctness-neutral, attacks the measured selective-scan weak spot.
2. **A3 — Exact-bitmap semi-join reducer** (`sparq-engine`). Ranked the single best
   technique-fit; localised; reused by A4.
3. **B1 — RDF/XML parsing** (`sparq-core`). Real standard-syntax gap; proven in-tree dependency;
   very low risk.
4. **C1 — Numeric/boolean TSV abbreviation (F21)** (`sparq-server`). Narrow serialiser gap; the
   store already preserves data lexical forms, so *not* a `sparq-core` change (premise corrected,
   PR #1258).
5. **A7 — Streaming Turtle serialisation** (`sparq-engine`). Mostly refactor; real memory win
   for large CONSTRUCT/DESCRIBE; enables HTTP streaming.
6. **A10 — N3 `log:semantics` cycle detection** (`sparq-reason`). Genuine safety/DoS fix for
   live resolvers; cheap, standard.
7. **B2 — `MULTIPLICITY()` vendor extension** (`sparq-engine`). Exposes the 1.2 algebra device;
   *not* a conformance gap (no `multiplicity()` builtin exists in the spec — premise corrected,
   PR #1257).
8. **C2 — Predicate-selectivity in fedplan non-star joins** (`sparq-fedplan`). Real estimate
   gap using stats already present; correctness-neutral.
9. **A2 — Elias-Fano compressed-seek codec** (`sparq-core`). Highest *storage* upside but
   measurement-gated (can lose on resident-cache scans) — prototype + A/B before adoption.
10. **C3 — JSON-LD expand/flatten gateable surfaces** (`sparq-engine`/`serialize-rdf`). Closes a
    documented conformance hole; feeds the `sq-oy1f` epic.
11. **A4 — Yannakakis prepass** (`sparq-engine`). High pathological upside; build after A3.
12. **A8 — JSON-LD chunk-parallel ingest** (`sparq-core`). Throughput win, correctness-sensitive
    chunking; needs a serial-equivalence oracle.
13. **A9 / C5 — EXPLAIN cost categories + per-regime conformance breakdown** (observability;
    low risk, qualitative payoff).

## E. Deferred (EC2 / maintainer / research-first)

- **A5 — Index-Based Join Sampling** — buildable but the win is *only* on correlated multi-join
  estimates (single-pattern estimates are already index-exact per `sq-p6p6`); measure the q-error
  delta on a quiet host before adopting. *(ec2-gated measurement.)*
- **A6 — Vector-at-a-time merge-join kernel** — kernel-level perf needs a canonical quiet box;
  highest architectural ceiling, larger effort. *(ec2-gated.)*
- **B4 — SPARQL entailment regimes in BGP matching** — couples query exec to reasoning; needs a
  maintainer design call on default behaviour and closure caching. *(maintainer-gated.)*
- **OWL EL concurrent saturation (E4) / incremental classification (E5) / OWL QL rewriter (Q1)**
  — these are *roadmap phases already laid out* in `research/owl2-el-ql-reasoning-spike.md`, not
  fresh discoveries; the spike's honest verdict is EL-depth-first and QL-behind-EL, and Q1's
  PerfectRef applicability check is the #1 unsoundness trap. **Research-first / already-tracked
  by the spike**; surfaced here only for visibility, not as new buildable beads.
- **Fedplan EWMA tuning** — already open as `sq-1z7g` (P4); needs a canonical federated
  workload. *(ec2-gated, beaded.)*
- **Benchmark/observability harnesses** (join-strategy micro-bench, cost-model q-error gate,
  flamegraph-diff recipe, dict-resolve micro-bench, codec-specific parse gate, wasm query
  micro-bench) — valuable measurement infrastructure but **non-canonical on the work box**;
  the wasm query micro-bench is already `sq-5pnl` (P4). Bundle into the `sq-5o5` benchmark epic
  rather than spawning many tiny beads.

## F. Dropped / corrected (false-premise, already-done, or already-beaded)

- **"Add SPARQL Protocol support to sparq-server"** — **FALSE PREMISE.** `crates/sparq-server/
  src/http.rs` (module doc lines 3–9, route doc ~lines 348–562) already implements the SPARQL
  1.1 Protocol `query` operation (GET + both POST forms), content negotiation, `default-graph-uri`
  /`named-graph-uri`, Service Description, and SPARQL Update. Not a gap.
- **"Missing `$currentShape` pre-binding in sh:sparql"** — **ALREADY DONE.** `crates/sparq-shacl/
  src/sparql.rs` (module doc lines 25–26) binds `$currentShape` to the source shape's node.
- **"Per-predicate coverage metadata not rendered" + "object-sample text not rendered" in
  `sparq-introspect`** — **FALSE PREMISE.** `to_text_summary` already renders coverage
  (`cp.coverage * 100`), avg-per-subject multiplicity, and per-predicate samples
  (`cp.samples.first()`, "e.g. {sample}") — `crates/sparq-introspect/src/lib.rs` summary body.
  (The *importance-weighted budgeting* variant is a minor refinement, not a missing-render gap;
  too marginal to bead.)
- **Adaptive bind-join / runtime-cardinality re-optimisation / divergence-triggered reorder**
  (multiple raw items) — **ALREADY BEADED** as `sq-6i40` (P3, OPEN), which is explicitly the
  redesign follow-up after `sq-p6p6` landed *neutral* (local estimates are index-exact, so a
  reorder only flips multiplicative arms — no work reduction). The honest path is *pruning/
  semi-join reduction* (captured here as A3/A4), not arm-reordering. Folded into A3–A5; the
  reorder framing is dropped as a known negative result.
- **"Raise SHACL-SPARQL floor from 5 to full suite"** — **PREMISE OVERSTATED.** `crates/sparq-
  shacl/tests/w3c_sparql.rs` asserts *every* node+property `sh:sparql` entry passes (line 71);
  the floor `5` reflects the count of cases in that sub-suite, not "5 of 20+." No under-coverage
  bug; reframing to "import additional `sh:sparql` sub-suites" is marginal — dropped.
- **Live exec-accuracy for sparq-nlq / dynamic few-shot / schema-slice retrieval / conversational
  schema caching** — **ALREADY BEADED / DESIGNED.** `sq-g0lw` (live numbers, CLOSED via fixtures),
  `sq-05rv` (harness, CLOSED), and `research/genai-nl-to-sparql.md` §3/§5/§8.5 own the dynamic
  few-shot, retrieval, and cross-turn caching designs (M1–M3 scope). Active programs `sq-lzvwg`/
  `sq-z6l7e` cover the QA-accuracy track. Dropped as duplicative.
- **Fedclient result cache / `SERVICE ?var`** — **ALREADY IDENTIFIED** under the closed
  federation epics `sq-dnko`/`sq-3183` (items C6/C2). Not fresh discoveries; defer to those.
- **HDT CRC-skip (`load_unchecked`)** — overlaps `sq-fkj` (decode-only HDT entry, P4 OPEN);
  the CRC-skip is a thin variant of the same decode-fast-path direction. Folded into `sq-fkj`.
- **Brotli wasm bundle pre-compression** — **DEPLOYMENT-LEVEL / likely already handled** by the
  CDN content-encoding; pure infra, no code change, no measurable engine win. Dropped.
- **WASM packed-results binary wire format** — **breaks the W3C SPARQL-JSON results contract**;
  any consumer parsing raw JSON breaks. High risk for a marginal boundary-size win; dropped.
- **SPARQL arithmetic decimal-division scale / LANGDIR-family completeness** — **ALREADY
  RESOLVED + DOCUMENTED** in `crates/sparq-conformance/FINDINGS.md` (F4 round-2/3, F15/F17/F18)
  as spec-divergent suite expectations or implemented functions; no engine bug. Dropped.
- **RDF/JS conformance ratchet wiring** — the implementation is **in-progress** (`sq-iwhl8`,
  IN_PROGRESS); the ratchet-floor wiring is a natural tail of that bead, not a separate
  discovery. Defer to `sq-iwhl8`.
- **Numerous `sparq-vectors` micro-tuners** (batch k-NN frontier sharing, deferred PQ re-rank,
  adaptive metric/codec/degree selectors, distortion-adaptive taxonomy geometry, delta-compaction
  heuristic, predictive mask caching) — each is a *measurement-gated micro-optimisation* whose
  payoff "depends on query clustering / distribution" and is only realised on specific workloads;
  most are tuning knobs on the `sq-0wo9e` structure-aware-vectorisation epic. Individually too
  marginal/speculative to bead now; better captured as ablations under `sq-0wo9e`. Dropped from
  the actionable set.
- **`sparq-core` dict micro-opts** (MRU/Bloom-guard hot-term cache, prefix frequency-sort,
  triple-term batch interning, lazy datatype table, compact inline-integer range) — all
  *measure-first* "is hashbrown already good enough?" probes with explicitly modest expected
  upside (sub-MB footprint, unproven probe-depth problem). Speculative without a profile;
  dropped pending a measured hot-spot.
- **`sparq-shacl` micro-opts** (Term-clone reduction, closed-shape/subclass-closure caching,
  qualifiedValueShape short-circuit, lazy path values, sparql pre-bind cache, component index,
  memo `>=`→`>`) — small localised perf refinements behind real benchmarks; the memo-invalidation
  `>=`→`>` and the ByTypes closed-shape audit (C-ish accuracy) are the only ones with correctness
  relevance, but both are sub-bead-sized. Roll into the `sq-bif`/benchmark programs rather than
  beading individually.
- **Streaming JSON-LD serialisation, SSE result streaming, wasm sub-bundle feature splitting,
  cursor `Symbol.dispose`, batch delta apply, lazy engine iteration** — each is a real but
  *lower-priority* ergonomics/perf item; several (sub-bundle splitting, SSE) carry non-trivial
  build/infra risk for a niche payoff. Not in the actionable shortlist; revisit if a concrete
  user need surfaces.

---

> 🤖 **SPARQ agent** — Conclusion: the genuinely-open, high-value, buildable-now wins are
> concentrated and few. Build the storage-layer trio the maintainer's own research already
> blessed (**A1 Bloom, A3 bitmap semi-join, A2 Elias-Fano**), close the small standalone
> correctness/functionality gaps (**B1 RDF/XML, C1 TSV abbreviation, B2 `MULTIPLICITY()`
> vendor extension, A10 N3 cycle-safety, A7 streaming Turtle**), and fold the rest into existing
> epics rather than spawning duplicate beads. Every perf claim here is a hypothesis to be confirmed on the
> canonical perf host (`sq-0g6g`); no numbers are asserted.
