<!-- [FABLE-5] Authored by Claude Fable 5 acting as the Fable-tier FRONT decomposition
stage (architect). Epic sq-hmd7l. This is the ONE design record for the
comparative-benchmarking-EVERYTHING program: it grounds the maintainer mandate against
the actual bench estate, fixes the canonical run protocol, gives every capability axis
an honest disposition (same-box comparison OR an explicit NOT-COMPARABLE verdict), and
lists the 28 disjoint child beads the implementation fleet executes. No implementation
here; no measured numbers here (numbers live in envelopes + per-axis gap records with
provenance). -->

# Comparative benchmarking EVERYTHING — program design record

**Status:** design record / program decomposition. **Date:** 2026-07-07.
**Epic:** `sq-hmd7l` (28 child beads, listed in §6). **Stage:** Fable architect —
decomposition only; the fleet implements.

**Mandate (maintainer, 2026-07-07, verbatim):** "set up to do comparative benchmarking
on _everything_ - not just sparql but _everything_ where you can do a comparison
(reasoning, full text search, geosparql etc.)". This extends the standing
performance-dominance mandate: order(s)-of-magnitude vs every open-source engine, beat
RDFox's published claims per axis, honest gap analysis on every axis (parity/behind =
gap + root cause + fix plan, never spin), and results propagate to the website and the
papers.

## 1. Problem

sparq has ~33 capability axes (full survey: the 2026-07-07 axis gather, summarized in
§5). Five are DONE — canonical same-box competitor envelopes exist for SPARQL query
latency, bulk ingest, HTTP serving, SHACL, and memory footprint, consolidated in
`research/perf-dominance-gap-2026-07.md`. The rest range from *registered gather-only,
never run* (full-text search vs jena-text; GeoSPARQL vs jena-geosparql; HDT vs hdt-cpp
— the cheapest wins in the whole program) through *sparq-only harness exists, no
competitor column* (reasoning, RSP, vector, N3 depth) to *no harness at all*
(serialization, JSON-LD throughput, Python bindings, GSP) to *genuinely no comparable
peer* (RIF, ZK/MPC, NL→SPARQL quality). The program's job is: every axis ends in either
a same-box comparison under one canonical protocol, or an explicit, documented
NOT-COMPARABLE verdict — never an unexamined blank.

## 2. Ground truth (verified against the repo, not taken from the epic framing)

What already exists and is load-bearing for the decomposition:

- **Registries.** `bench/competitors.json` (pinned versions/digests + per-entry honesty
  caveats; jena-text, jena-geosparql, hdt-cpp, lucene-anserini, PostGIS-loose and the
  five SPARQL engines are already registered), `bench/benchmarks.toml` +
  `scripts/check-new-bench-registered.py` (suite-id registration gate), and
  `bench/CATALOG.md` (scope + honesty notes, including the rsp-ql "no competitor perf
  column" note this record supersedes in §5.2).
- **Canonical-protocol precedents.** `scripts/bench/canonical-competitor-bench.sh` and
  `scripts/bench/canonical-http-gather-instance.sh` (quiet-EC2 gathers),
  `scripts/bench/shacl-same-box.sh` (the same-box template every new per-axis script
  copies: envelope JSON, `TIMEOUT_S`, `ONLY=` filtering, `canonical:false` off the
  quiet box), `scripts/gather-competitors.sh`, `scripts/ec2-bench.sh`,
  `scripts/orphan-check-bench.sh`, `scripts/bench/emit_envelope.py`.
- **Partially-done comparative harnesses.** `bench/pss-update-set/compare.py` (SPARQL
  UPDATE, QLever-only), `bench/parse/` (oxttl already an in-process competitor column),
  `bench/inference/eye-comparison.sh` (N3 vs EYE, ratio-reporting),
  `scripts/bench-adapters/vector_lib_adapter.py` (hnswlib/FAISS kernel adapter,
  matched-recall methodology already codified),
  `scripts/bench-adapters/beir_ir_adapter.py` (FTS IR-quality, wired but never run).
- **Prior records.** `research/perf-dominance-gap-2026-07.md` (+ HTTP addendum) — the
  master gap table with the fixed verdict vocabulary; `research/rdfox-claims-inventory.md`
  — RDFox published-claims normalization (per-core, never absolute-vs-absolute);
  `research/memory-per-triple-2026-07.md`.
- **Premise corrections found while grounding.** (a) `sq-w34fa` describes incremental
  reasoning as "a FEATURE gap first (batch closure only)" — stale: sparq-reason ships
  `incremental.rs` / `incremental_explain.rs` with two bench examples and
  `bench/inference/incremental-bench.md` (`full_rebuilds()==0` invariant). The
  remaining gap on that axis is a comparison story, not a feature (§5.1). (b) The five
  DONE axes already have re-measure/instrument beads (`sq-7d3dj.30.6`, `.32.2.3`,
  `.33.3`, `sq-mbg0k`, `sq-1s03r`, `sq-ohsvs`, `sq-vw3ax.12`) — this program links
  them and creates nothing duplicate for those axes.

## 3. Decomposition: options and the chosen shape

- **(A) One program mega-harness / one mega-bead.** Rejected: serializes the fleet,
  unreviewable diffs, and one failure blocks every axis.
- **(B) Fully independent per-axis beads, each registering its own competitors.**
  Rejected: `bench/competitors.json`, `bench/benchmarks.toml` and `bench/CATALOG.md`
  become an N-way merge conflict across ~20 parallel PRs — exactly the collision the
  disjointness rule exists to prevent.
- **(C) Chosen: single-owner seams + per-axis disjoint harness beads.** One registry
  bead (`sq-hmd7l.1`) owns all three shared registry files and pre-registers every new
  competitor + suite id up front; each per-axis bead then only **adds new files inside
  its own directory** (its harness script, its pinned query translations, its
  `research/gap-<axis>-2026-07.md` record); one bin-pack runner bead owns the EC2
  execution infrastructure; one execution bead appends canonical rows (sequenced by
  deps so the appends cannot collide); one consolidation bead owns the master gap
  table; one site bead owns the dashboard/paper propagation.

House rules every child bead inherits: **oracle before stopwatch** (count/result
cross-check vs an oracle before any timing row; disagreement recorded, never
adjusted); **no measured numbers in tracked files** (envelopes + gap records carry
numbers with provenance; registries and dashboards stay value-free in git, site data
flows from reviewed gather snapshots); **cheapest-first** (registered-never-run
harnesses before new-build harnesses); **absent tool ⇒ absent column**, never a
fabricated or estimated value.

## 4. Canonical run protocol + EC2 cost discipline

(Referenced by beads `sq-hmd7l.25` / `sq-hmd7l.26` as "design record sec 4".)

The established canonical protocol, applied program-wide:

1. **Same box, quiet box.** Canonical rows come only from a dedicated single-tenant
   EC2 instance (tag `sparq-bench`), never the shared work box or CI runners. One
   engine active at a time; min-of-N on a loaded/warmed store; identical corpus and
   query files per engine.
2. **Oracle first.** Every workload cross-checks counts/result-sets/closure sizes
   against an oracle (self-asserting expected files, engine-vs-engine agreement, or a
   pinned upstream reference) *before* timing. A cross-check failure is a recorded
   finding, never silently absorbed.
3. **Provenance lines.** Every envelope records instance id + type, git commit, UTC
   timestamp, `canonical:true|false`, and the cross-check status. First-read numbers
   gathered during harness development are allowed but flagged `canonical:false` and
   never enter the master table as canonical.
4. **Orphan-proofing (mandatory).** Every provisioned box sets
   `instance-initiated-shutdown-behavior=terminate`, installs a user-data watchdog,
   and self-terminates; `scripts/orphan-check-bench.sh` must come back clean after
   every run.
5. **Bin-packing + cost caps.** Small axes do not get one box each: the multi-axis
   runner (`sq-hmd7l.25`) provisions ONE Docker-capable quiet box and runs an ordered
   list of per-axis same-box scripts **serially** (protocol point 1 makes serial
   execution mandatory anyway), collects envelopes, then self-terminates. Concurrent
   boxes are capped per the standing fleet cost pattern; disk discipline between axes
   (df check, scratch cleanup, capped dataset sizes). GPU axes are explicitly outside
   the standard fleet (§5.1).
6. **Failed competitor = re-run action.** A competitor that fails to load/run is a
   missing column and a re-run bead, never a sparq win.

## 5. Per-axis disposition

(Referenced by bead `sq-hmd7l.1` as "design record sec 5".)

All 33 surveyed axes. "Done" = canonical competitor envelopes exist; links go to the
existing owner bead rather than a new one.

| # | Axis | State today | Disposition | Owner |
|---|------|-------------|-------------|-------|
| 1 | SPARQL query latency | DONE (5-engine canonical matrix) | re-measure after complex-shape fixes | sq-7d3dj.30.6 (existing); breadth sq-vw3ax.12 |
| 2 | Bulk load / ingest | DONE | scale-up + RDFox big-core instrument | sq-7d3dj.31 / sq-mbg0k (existing) |
| 3 | HTTP serving / TTFB / QPS | DONE (D9/D10 addendum) | maintained; no new bead | existing |
| 4 | SHACL validation | DONE (same-box harness) | canonical EC2 re-run | sq-7d3dj.33.3 (existing) |
| 5 | Memory bytes/triple | DONE (deterministic + canonical) | compressed-profile envelope | sq-7d3dj.32.2.3 (existing) |
| 6 | Materialization (RDFS/OWL-RL) | sparq-only harness | same-box vs Jena/VLog/Nemo on LUBM; RDFox published-claims column per-core only | **sq-hmd7l.7**; scale sq-1s03r |
| 7 | Incremental reasoning | sparq harness exists (premise-corrected, §2) | NOT-COMPARABLE same-box: no maintained OSS RDF peer does incremental materialization — per-delta vs full-rematerialization ratio + RDFox published claims | sq-w34fa (existing, reframed) |
| 8 | OWL 2 EL classification | sparq-only synthetic | vs ELK (+HermiT baseline) on GO/OpenGALEN/ORE, subsumption-count oracle | **sq-hmd7l.8** |
| 9 | OWL 2 QL rewriting | sparq-only synthetic | vs Ontop on NPD + Requiem: rewrite wall time AND UCQ size; rewriter-phase vs end-to-end labelled | **sq-hmd7l.9** |
| 10 | OWL DL tableau | NO harness | sparq-side ORE harness first (self-asserting verdicts), then HermiT/Openllet columns | **sq-hmd7l.10** |
| 11 | N3 rules | PARTIAL (EYE compared) | add cwm + jen3 columns to the existing harness | **sq-hmd7l.11** |
| 12 | RIF consumption | NO harness | **NOT-COMPARABLE** — no runnable peer; W3C RIF suite pass-rate runner backs the only-maintained-consumer claim; translated-rule perf rides axis 6 | **sq-hmd7l.24** |
| 13 | Full-text search | registered gather-only, never run | jena-text same-box + BEIR IR-quality run; pinned query translations | **sq-hmd7l.2** |
| 14 | GeoSPARQL | registered gather-only, never run | jena-geosparql same-box; result-set-size oracle (float geometry not bit-stable) | **sq-hmd7l.3** |
| 15 | RSP / streaming | sparq-only oracle harness | bounded count-matched replay vs RSP4J (§5.2) | **sq-hmd7l.20** |
| 16 | SPARQL UPDATE | PARTIAL (QLever parity) | extend to Fuseki + Oxigraph, wrap in envelope; post-workload store-state oracle | **sq-hmd7l.5** |
| 17 | Federation / SERVICE | planner micro only | FedShop-shaped same-box vs Comunica + Jena SERVICE over **local** member endpoints; request counts + source-selection precision alongside wall time | **sq-hmd7l.12** |
| 18 | Vector ANN | methodology registered, gather pending | execute SIFT1M/GloVe matched-recall Pareto vs hnswlib/FAISS (kernel-labelled); SPARQL-integrated surface stated uncontested | **sq-hmd7l.19** |
| 19 | RDF parsing | PARTIAL (oxttl in-harness) | add serd/rapper/Jena-riot external columns | **sq-hmd7l.6** |
| 20 | Serialization | NO harness | new round-trip-gated MB/s harness vs oxrdfio/riot/serd | **sq-hmd7l.14** |
| 21 | JSON-LD | conformance only | pass-rate table + corpus throughput vs jsonld.js/titanium-json-ld | **sq-hmd7l.15** |
| 22 | HDT load-and-decode | registered gather-only, never run | run the hdt-cpp decode-only gather (decode-to-native is the ONLY like-for-like axis, per CATALOG) | **sq-hmd7l.4** |
| 23 | GSP / LDP-CRUD serving | NO GSP-specific harness | same-box GSP panel vs Fuseki/Oxigraph (+CSS labelled loose) | **sq-hmd7l.21** |
| 24 | Graph analytics | sparq-only synthetic | LDBC Graphalytics validation-gated vs igraph/NetworKit; export cost reported honestly | **sq-hmd7l.13** |
| 25 | RDF canonicalization | conformance only | conformance parity + poison-graph DoS-resistance vs rdf-canonize/rdf-canon (capped wall-clock; cap-hits are results) | **sq-hmd7l.16** |
| 26 | Browser / WASM | size gate only | deterministic bundle-bytes vs oxigraph npm now; in-browser latency harness second | **sq-hmd7l.17** |
| 27 | Python bindings | NO harness | binding-overhead (primary) + absolutes vs pyoxigraph/rdflib | **sq-hmd7l.18** |
| 28 | KGE link prediction | sparq-only ablation | quality (MRR/Hits@k) at matched model+hyperparams vs pinned PyKEEN published numbers; same-box run upgrades to measured | **sq-hmd7l.23** |
| 29 | NL→SPARQL | sparq-only stub-LLM | **NOT-COMPARABLE** as an engine axis — quality is model-dominated; keep deterministic self-tracking; revisit only as fixed-model answerable-accuracy | documented here; no bead |
| 30 | Access control (WAC/ACP/ODRL) | sparq-only oracle | correctness-comparable more than perf-comparable; owned by the access-controlled-query benchmark paper program | sq-i6du2 (existing epic) |
| 31 | GPU kernels | self-relative | **NOT-COMPARABLE** as an engine axis — no runnable GPU RDF engine; kernel-level cuDF comparison only, needs non-standard (GPU) instances, outside this program's fleet | documented here; no bead |
| 32 | ZK / MPC query proofs | self-relative | **NOT-COMPARABLE** — uncontested surface; see §5.1 caveat | documented here; no bead |
| 33 | Conformance pass-rates | sparq scoreboard | cross-engine table vs peers' **published** EARL results (pinned sources, no estimated cells) | **sq-hmd7l.22** |

Cross-cutting spine beads: **sq-hmd7l.1** (registry single-owner), **.25** (multi-axis
EC2 runner), **.26** (canonical wave-1 execution), **.27** (gap-table v2
consolidation), **.28** (site + paper propagation).

### 5.1 NOT-COMPARABLE verdicts (documented, not spun)

- **RIF:** no live competitor consumes RIF (fuxi dead, Jena dropped it, RDFox consumes
  datalog). The honest claim is "only maintained open-source RIF consumer", backed by a
  W3C RIF suite pass-rate runner (`sq-hmd7l.24`) — failures listed, never rounded up.
  Performance of RIF-translated rules is compared on the materialization axis instead.
- **ZK / MPC:** no other engine proves SPARQL results in zero knowledge or runs
  MPC-federated SPARQL, so there is no comparative column; the estate stays
  self-relative vs the Noir/Barretenberg toolchain baselines. Honesty caveat, per the
  live privacy-claims gate: "uncontested" is a *market* observation, not a soundness
  claim — the v1 verifier is internally re-audited with **external
  accredited-cryptographer sign-off still pending** (`sq-qhy4`), and the MPC layer is
  **semi-honest-only**. No benchmark row may imply a proven cryptographic guarantee.
- **NL→SPARQL:** answer quality is dominated by the LLM, not the engine; an
  engine-comparison would measure the model. Deterministic stub-LLM self-tracking is
  preserved; low priority for the dominance mandate.
- **GPU:** kernel-level sub-component comparisons only (no runnable GPU RDF engine);
  requires GPU instances outside the standard `sparq-bench` fleet — excluded from this
  program, revisit separately if the mandate extends there.
- **Incremental reasoning:** no maintained open-source RDF engine ships incremental
  materialization, so the same-box form is empty. Honest output = per-delta latency vs
  full-rematerialization ratio (harness exists) + a published-claims-only RDFox column.
  Owned by the existing `sq-w34fa` with the §2 premise correction.

### 5.2 The bounded RSP comparability protocol (decision)

`bench/CATALOG.md` currently scopes rsp-ql as having *no competitor perf column*
(wall-clock service engines vs sparq's clock-free replay = time-model mismatch). That
blanket exclusion is too strong under the everything-mandate. **Decision:** a bounded,
honest comparison is possible and is adopted — drive RSP4J/YASPER with the *identical
timestamped replay*, require **per-window result-count agreement** with sparq's
deterministic oracle *first*, then report sustained triples/s side-by-side with a
**machine-attached time-model caveat on every emitted row** (the caveat travels in the
envelope, not just prose). Windows that cannot be count-matched are excluded and the
exclusion reported. `sq-hmd7l.1` replaces the CATALOG scope note with this protocol;
`sq-hmd7l.20` implements it.

### 5.3 Cross-axis comparability rules

- **RDFox** columns are published-claims-only, per-core/per-thread normalized via
  `research/rdfox-claims-inventory.md` — never absolute-vs-absolute.
- **Loose columns** (different architecture/surface: PostGIS, CSS, kernel libraries)
  are labelled loose/sub-component and never averaged with like-for-like columns.
- **Verdict vocabulary** is fixed: CLEARLY-AHEAD / AHEAD-BUT-NOT-OOM / PARITY /
  BEHIND / NOT-MEASURED / NOT-COMPARABLE. Every BEHIND or PARITY row files an
  immediate P1 profiling-first fix bead (`sq-hmd7l.27` enforces).
- **Licensing:** GraphDB Free and other proprietary-freeware candidates need an EULA
  check before any published number; engines failing it stay candidates.

## 6. Child-bead plan (28 disjoint beads)

Full spec (what/why/where, `crate`, `model_tier`, `FILES`, `INVARIANT`, `ACCEPTANCE`)
lives on each bead (`bd show sq-hmd7l.<n>`). Summary:

**Spine (P1).**

| Bead | Role | Tier | Files owned |
|------|------|------|-------------|
| sq-hmd7l.1 | registry pre-registration (single owner of all shared registry files) | sonnet | `bench/competitors.json`, `bench/benchmarks.toml`, `bench/CATALOG.md` |
| sq-hmd7l.25 | multi-axis bin-pack EC2 runner, orphan-proof | sonnet | `scripts/bench/multi-axis-box.sh`, `scripts/bench/README.md` |
| sq-hmd7l.26 | canonical wave-1 execution (tier-1 axes) | sonnet | canonical-row appends to the tier-1 gap records (sequenced by deps) |
| sq-hmd7l.27 | dominance gap-table v2 consolidation + fix-bead filing | opus | `research/perf-dominance-gap-2026-07.md` |

**Tier 1 — harness exists / cheapest-first (P1).**

| Bead | Axis | Tier | New files under |
|------|------|------|-----------------|
| sq-hmd7l.2 | full-text search | sonnet | `scripts/bench/fts-same-box.sh`, `bench/fts/queries-jena-text/` |
| sq-hmd7l.3 | GeoSPARQL | sonnet | `scripts/bench/geo-same-box.sh`, `bench/geo/queries-jena/` |
| sq-hmd7l.4 | HDT decode | haiku | `scripts/bench/hdt-same-box.sh` |
| sq-hmd7l.5 | SPARQL UPDATE | sonnet | `bench/pss-update-set/compare.py`, `scripts/bench/update-same-box.sh` |
| sq-hmd7l.6 | parsing | sonnet | `bench/parse/` |
| sq-hmd7l.7 | materialization | opus | `scripts/bench/materialize-same-box.sh`, `scripts/bench-adapters/{jena,vlog,nemo}` |

**Tier 2 — reasoning depth + new harnesses (P2).**
sq-hmd7l.8 (EL, sonnet), .9 (QL, sonnet), .11 (N3 columns, haiku), .12 (federation,
opus), .14 (serialization, sonnet), .16 (canonicalization, sonnet), .17 (WASM,
sonnet), .18 (Python, haiku), .19 (vector Pareto, sonnet), .20 (RSP protocol,
sonnet), .22 (conformance table, sonnet), .28 (site/paper propagation, sonnet —
owns `site/` + `bench/dashboard/`).

**Tier 3 — breadth (P3).**
sq-hmd7l.10 (DL/ORE, sonnet), .13 (Graphalytics, sonnet), .15 (JSON-LD, sonnet),
.21 (GSP, sonnet), .23 (KGE, sonnet), .24 (RIF verdict, sonnet).

Every per-axis bead also creates its own `research/gap-<axis>-2026-07.md`.

**Dependency edges (only where ordering is real).**

- `sq-hmd7l.1` → every bead introducing a **new** competitor or suite id: .6, .7, .8,
  .9, .10, .11, .12, .13, .14, .15, .16, .17, .18, .20, .21, .23, .24, .28. Tier-1
  beads .2/.3/.4/.5 and .19/.22/.25 are **not** gated: their competitors/adapters are
  already registered.
- `{.2, .3, .4, .5, .6, .25}` → `.26` (canonical wave-1) → `.27` (consolidation).
  Sequencing .26 after the tier-1 harness beads is what makes the gap-record appends
  collision-free.

**Disjointness assertion.** No two parallel beads touch the same file: the three
shared registry files have exactly one owner (.1); each axis bead only adds files in
its own directory; the master gap table has one owner (.27); `site/` + `bench/dashboard/`
have one owner (.28); the only multi-writer files (tier-1 gap records) are serialized
through the `.26` dep edges.

## 7. Results propagation (site + papers)

`sq-hmd7l.28` closes the mandate's propagation clause: dashboard competitor columns
for every new axis flow from **reviewed gather snapshots** through the existing
`site/scripts/sync-benchmarks.mjs` pipeline (no hand-typed numbers), and canonical
envelopes register as paper evidence (environment=canonical, with source provenance)
for the papers that cite the affected axes. Per-axis honesty caveats (loose columns,
time-model caveat, published-claims columns) must survive into the rendered site —
a caveat that exists only in a research record is not propagated.

## 8. Honesty and soundness posture

- Every axis ends in a verdict from the fixed vocabulary; NOT-COMPARABLE is a
  first-class, documented outcome (§5.1), not a silent omission.
- No measured numbers in this record or any tracked registry/dashboard source; numbers
  live in envelopes and gap records with full provenance, work-box readings are
  non-canonical by construction.
- ZK/MPC rows never state or imply a proven cryptographic guarantee (external audit
  pending `sq-qhy4`; MPC semi-honest-only).
- Linked, not duplicated: `sq-7d3dj.30.6` / `.32.2.3` / `.33.3` (DONE-axis
  re-measures), `sq-vw3ax.12` (competitor breadth), `sq-mbg0k` / `sq-1s03r` /
  `sq-ohsvs` / `sq-w34fa` (RDFox instrument rows), `sq-atjue` (substrate
  zero-overhead), `sq-i6du2` (access-control benchmark paper), `sq-6tykl` (reasoner +
  federation program).
