# Competitor benchmark landscape for sparq

<!-- [OPUS-4.8] Authored by Opus 4.8 (Fable unavailable; flag for re-review when Fable returns). -->

**Status:** design record / research synthesis. **Date:** 2026-06-15.
**Question this answers (from @jeswr):** *"Are there other engines sparq should have
performance benchmarks against — e.g. Apache Solr and Jena Fuseki?"*

This record synthesises four research surveys (SPARQL/RDF-store landscape; benchmark-suite
baselines; per-surface comparators; feasibility + methodology) into one prioritised plan and
maps it onto the existing competitor seam (`bench/competitors.json` +
`scripts/gather-competitors.sh`). It introduces **no new policy** — it extends the QUIET-BOX /
gather-once discipline already codified in `bench/CATALOG.md` and the `competitors.json`
`_comment` block. No fabricated numbers appear here (per AGENTS.md: no hard-coded perf in
markdown); every quantitative claim is attributed to a cited third-party source.

---

## 0. TL;DR verdict on the owner's two examples

| Engine | Add as a SPARQL perf competitor? | One-line why |
|---|---|---|
| **Apache Jena Fuseki** | **YES — tier-1, add first.** | The canonical reference SPARQL 1.1 server; Apache-2.0 (publishable, no DeWitt clause), first-party Docker, runs every existing sparq suite unmodified. Its value is *correctness oracle + the mainstream baseline reviewers expect to see*, not a raw-speed rival (it lands mid/back-of-pack vs QLever/Virtuoso). |
| **Apache Solr** | **NO — not a SPARQL competitor.** Only a *loose kernel reference* for the `sparq-text` full-text sub-surface. | Solr is a Lucene-based document-search server with **no RDF, no triples, no SPARQL** — it cannot run SP2Bench/WatDiv/BSBM/LUBM/DBPSB. In the RDF world it appears only as a text *connector bolted beside* a triplestore (and RDF4J has **deprecated** its Solr backend). A fair head-to-head at the SPARQL-surface level is impossible. The only defensible Solr comparison is a *separate* full-text-search latency/recall harness against `sparq-text` — a different axis, lower priority, its own bead. |

The honest framing that runs through the whole record: the fair *SPARQL-surface* peers are
always the **triplestores that bolt the same capability onto SPARQL** (Jena/Fuseki, Virtuoso,
GraphDB, RDF4J). The specialist tools (Solr, FAISS/Qdrant, RDFox, hdt-cpp, PostGIS) do the
sub-capability *without* SPARQL/RDF — they are fair as **kernel references** (does sparq's
index/closure compete in isolation) and unfair as **surface benchmarks** (none does
the-capability-fused-into-SPARQL-over-RDF, which is the actual product).

---

## 1. The Fuseki verdict, in full

Fuseki + TDB2 is the de-facto reference RDF server in the SPARQL world. It has the most
complete and most-trusted SPARQL 1.1 implementation (plus early SPARQL-1.2/RDF-star work),
ships the canonical W3C test-suite pass, and appears as a baseline in essentially every
triplestore-benchmark paper: DBPSB, FEASIBLE, BSBM, WDBench, and the recent Sparqloscope
(ISWC 2025) all include Jena/Fuseki ([DBPSB][dbpsb], [FEASIBLE][feasible], [WDBench][wdbench],
[Sparqloscope][sparqloscope]).

So the value of benchmarking against Fuseki is two-fold and **neither half is "it will be hard
to beat"**:

1. **The standard yardstick reviewers expect to see.** Its absence from a comparison table is
   conspicuous; its presence is table-stakes credibility.
2. **A generous, easy-to-install correctness oracle** — same role Oxigraph already plays as the
   differential oracle, but as a JVM HTTP server over the public suites rather than an
   in-process Rust dep.

It is *not* the speed target. Fuseki is Java/JVM, single-machine, and consistently lands
mid-to-back of the pack on raw query/load speed against QLever/Virtuoso/RDFox. **sparq's real
speed target remains QLever** (the 2025 Patel-Schneider Wikidata study found QLever "the
fastest engine overall by a significant amount", loading the full Wikidata dump in ~4.5h
[[Patel-Schneider 2025][ps2025]]). Wire Fuseki as a **correctness + "standard-baseline"
column**, anchoring the mainstream end of the dashboard; QLever anchors the fast end and
Oxigraph the Rust-peer end.

**Feasibility (why it is the cheapest high-value add):** Apache-2.0 license → numbers are
publishable with no DeWitt/benchmark-publication restriction; first-party `jena-fuseki-docker`
tooling (Dockerfile + compose + helper scripts) plus community images (`stain/jena-fuseki`,
`secoresearch/fuseki`); `tdb2.tdbloader` bulk-loads NT/TTL and Fuseki serves SPARQL 1.1, so it
runs **all** existing sparq suites unmodified ([Fuseki Docker docs][fusekidocker]).

---

## 2. The Solr verdict, in full

**Solr is not an RDF triplestore and does not speak SPARQL.** It indexes documents → returns
document IDs; it has no triples, IRIs, BGP joins, or SPARQL algebra. It cannot run any of
sparq's SPARQL suites. The only way to make Solr "speak SPARQL" is the long-abandoned SolRDF
plugin.

How Solr actually shows up in the RDF ecosystem confirms the apples-to-oranges:

- In Jena (`jena-text`), GraphDB, and RDF4J, Lucene/Solr/Elasticsearch are **connectors/SAILs
  bolted *beside* the triplestore** — "Lucene indexes are additional information for accessing
  the RDF graph, not storage for the graph itself" ([jena-text][jenatext]).
- RDF4J has **deprecated its Solr `SearchIndex`** (as of 5.x) in favour of
  Lucene/Elasticsearch ([RDF4J Lucene SAIL docs][rdf4jlucene]) — even within the RDF world Solr
  is on the way out as the text backend.
- `sparq-text` is the *in-process equivalent of those connectors*: it does FTS over RDF
  literals and returns dictionary term-ids that join back to subjects/predicates inside a
  SPARQL BGP (the crate's model: "the dictionary term id of a string literal **is** its
  document id"). There is no query you can run on both Solr and `sparq-text` that means the
  same thing at the surface level.

A "fair" Solr comparison would require dumping RDF literals as flat documents, indexing them in
Solr, and timing raw term/phrase lookups — i.e. comparing the **BM25 inverted-index kernel** of
sparq's hand-rolled index against Lucene's mature one. That is a useful *sanity reference for
the sub-component*, but it tells you nothing about the surface's actual job (FTS fused into
SPARQL solution generation), and it flatters whichever side you cherry-pick (Solr/Lucene win on
raw index features and scale; sparq wins by avoiding the cross-process round-trip and join-back
materialisation). Solr is Apache-2.0 and easy to Docker ([official `_/solr` image][solrimage]),
so the comparison is *feasible* — but it requires a **separate IR-style harness** (index a
corpus, run BM25 queries, compare latency/recall), not a reuse of the SPARQL suites.

**Decision:** do **not** put Solr on the SPARQL query/load dashboard. Track the FTS comparison
as its own deferred bead against `sparq-text`, and prefer **Lucene** (the embedded library) over
**Solr** (the server) as the kernel reference, because `sparq-text` is an embedded library — a
Solr/ES server adds a network + distribution layer `sparq-text` deliberately doesn't have,
widening the gap.

---

## 3. Core SPARQL competitors (query + load) — prioritised

Scope: engines that can (a) bulk-load N-Triples/Turtle and (b) serve SPARQL 1.1, so they run
sparq's existing suites (SP2Bench, WatDiv, BSBM, LUBM-extensional, DBPSB) **unmodified** over an
HTTP-SPARQL adapter. The suites all consume standard NT/TTL + plain `.rq` queries
(`bench/benchmarks.toml` confirms: sp2b=turtle; dbpsb/watdiv/bsbm/lubm=ntriples), so any such
engine needs only an endpoint URL + the adapter.

**Already tracked by sparq** (`bench/competitors.json`): **QLever** (Docker; the speed target),
**Oxigraph** (embedded Rust dep; differential oracle + Rust peer), **EYE** (N3 reasoner — a
*reasoning* baseline, not a SPARQL-query store). RDFox is discussed in
`research/inference-sota.md` as the reasoning SOTA but is **not** in the registry (proprietary,
license-gated — see §4).

### Tier-1 — add first (open, dockerizable, publishable, runs our suites unmodified)

| Engine | License → publish? | Docker | Runs our SPARQL suites? | Role for sparq |
|---|---|---|---|---|
| **Apache Jena Fuseki / TDB2** | **Apache-2.0 — yes**, no DeWitt | **First-party** `jena-fuseki-docker` (+ `stain/jena-fuseki`) | **Yes** — `tdb2.tdbloader` + SPARQL 1.1 endpoint | Correctness oracle + the mainstream/standard baseline. *Directly answers the owner's question.* |
| **Virtuoso Open Source (VOS 7)** | **GPLv2 — yes**, no benchmark restriction | **First-party** `openlink/virtuoso-opensource-7` (active) | **Yes** — `isql ld_dir`/`rdf_loader_run` bulk-load + SPARQL | The historical production heavyweight; the single most-reported baseline across **every** suite (SP2Bench, WatDiv, BSBM, DBPSB, FEASIBLE, WDBench, Sparqloscope). Its absence is the most conspicuous gap. |

Both are the union of "appears in the most papers" and "cheapest to stand up." The most current
third-party study (Patel-Schneider, Wikidata Workshop 2025) benchmarked **Blazegraph /
MillenniumDB / QLever / Virtuoso**; the 2019 representativeness study and the 2023 Lam et al.
Wikidata study used **GraphDB / Jena Fuseki / Neptune / RDFox / Stardog** — so the union of
those seven plus Oxigraph (the Rust peer) is the comparator set reviewers expect. Of that union,
**Fuseki + Virtuoso are the two that are open, dockerizable, publishable, and run our suites
today** — hence tier-1.

### Tier-2 — add when justified (extra datapoint, scale tier, or reasoning column)

| Engine | License → publish? | Docker | Notes |
|---|---|---|---|
| **MillenniumDB** | OSS (research, permissive) | No official image (repos are benchmark scripts) | The modern academic peer that anchors **WDBench**; the C++ challenger to QLever. *But* SPARQL is **incomplete** (some constructs unsupported, timeouts on complex queries), and the WDBench scripts translate SPARQL→MDQL — so it needs a translation/build layer. Add **only when sparq runs WDBench/Wikidata at scale.** High gather cost. |
| **Eclipse RDF4J** | EDL/BSD — **yes** | **First-party** `eclipse/rdf4j-workbench` | Cheap to add, but partially **redundant with Fuseki** (same Java/Sesame lineage, similar speed band). Add only if a second JVM datapoint is wanted. |
| **GraphDB Free** | **NO — blocked.** Ontotext License Art. 15.3 is a **DeWitt clause** ("not allowed to publish evaluation results … without written permission"); 11.0+ also needs a license key | First-party `ontotext/graphdb` | Technically runs our suites and would be the *commercial-reasoner + LUBM-OWL-RL* column, but **publication-blocked**. needs:user — do not publish without written Ontotext permission. |

### Skip / defer for core SPARQL

| Engine | Why skip |
|---|---|
| **Blazegraph** | GPLv2 (publishable) and trivially comparable, **but abandoned since ~2019-2020** and consistently the slowest (in the 2025 study it took >10 days to load Wikidata vs QLever's ~4.5h [[ps2025]]). Add **only** as a single "what Wikidata is escaping" historical reference line, never as a serious modern rival. Community Docker only. |
| **Amazon Neptune** | Commercial, **managed-only (AWS)** — no on-prem binary → not reproducibly/cheaply benchmarkable on sparq's EC2 harness. Low priority. |
| **Stardog** | Commercial, closed; Docker "by request" only; free tier needs a license key; its **query-rewriting reasoner is not apples-to-apples** with sparq's materialisation. needs:user. |
| **RDFox** | Proprietary (license-key, free academic/eval). The right **reasoning** comparator (see §4 reasoning) but **gather-blocked** until a license is provisioned. Already needs:user in `research/inference-sota.md`. |
| **Apache Solr** | Not a SPARQL engine — see §2. FTS sub-surface only. |

---

## 4. Per-surface comparators (capability crates)

The crates `sparq-text`, `sparq-vectors`, `sparq-reason`, `sparq-hdt`, `sparq-geo` all exist in
`crates/`. For each, the comparison splits into a **kernel** reference (sparq's index/closure in
isolation — fair-ish) and an **end-to-end SPARQL** peer (the triplestore that bolts the same
capability onto SPARQL — the true like-for-like).

### Full-text — `sparq-text`
- **Like-for-like surface peer (recommended): Apache Jena Fuseki + `jena-text` (Lucene
  backend).** Same query shape (`text:query` ≈ sparq's `text:matches`), same join-back-to-graph
  semantics. Secondary RDF peers: GraphDB / RDF4J full-text connectors.
- **Kernel reference only: Lucene** (the embedded library, *not* Solr the server), clearly
  labelled "sub-component reference, not an RDF benchmark."
- **Caveat:** `sparq-text`'s index is deliberately minimal (in-house BM25, not a tantivy/Lucene
  port). A kernel comparison must be scoped to what sparq actually implements (token AND/OR,
  prefix, phrase, proximity/slop, BM25 scoring) or it is unfair to sparq. **Solr/ES: not a
  competitor** — see §2.

### Vector / ANN — `sparq-vectors`
- **Kernel comparators (the standard, fair way): the `ann-benchmarks` harness**, reporting the
  recall–QPS Pareto curve at *matched recall* (never a single latency number — ANN is a
  precision/speed trade-off, so two results are comparable only at similar precision
  [[ann-benchmarks][annbench]]).
  - **hnswlib** — canonical in-RAM HNSW reference → primary peer for sparq's `VectorIndex`.
  - **FAISS** — scalability/compression + GPU reference → maps to sparq's quantizers
    (scalar 4×, product/asymmetric-distance) and the brute-force `nearest_exact` ground truth.
  - **ScaNN** — recall/latency SOTA (anisotropic quantization) → the "can you beat the best" bar.
  - **DiskANN/Vamana** — the right peer for sparq's on-disk `DiskAnnIndex` (`.spqg`), which
    implements Vamana; the reference impl is the natural correctness+perf oracle.
- **Loose system-level reference only: Qdrant / Milvus / Weaviate** — full databases (server,
  persistence, filtering, sharding); benchmarking the *library* against them measures the whole
  DB stack (apples-to-oranges in the *opposite* direction from Solr — they have *more* than
  sparq). Use only to answer "is sparq's recall/latency in the same ballpark as a production
  vector DB's index," labelling network/persistence overhead explicitly.
- **Deeper caveat:** none of these do ANN-inside-SPARQL over dict-encoded RDF entity ids — the
  kernel benchmark validates "the index is competitive," but the *surface value* (graph-aware
  hybrid retrieval) has no direct competitor.

### Reasoning — `sparq-reason`
- **Already tracked: EYE** (N3 forward closure — `bench/inference/eye-comparison.md`). The right
  primary N3 peer. Caveat (already in-repo): it measures forward-closure throughput only, not
  EYE's full breadth; never cross-compare against EYE numbers from goal-directed papers.
- **RDFS / OWL 2 RL materialisation SOTA peers: RDFox (the bar), VLog, Nemo (the Rust
  competitor to beat)** — all semi-naive forward materialisers, exactly sparq's camp; the right
  targets for the LUBM/UOBM/DeepTaxonomy closure suites. RDFox is **license-gated** (needs:user);
  VLog/Nemo are open and are the **missing reasoning baselines** for LUBM-OWL-RL / Deep Taxonomy.
- **Apache Jena rules reasoner** — a *correctness oracle and JVM low/mid-perf baseline*, not a
  SOTA bar (Jena's forward rule reasoner OOMs where RDFox finishes in ms on large TBoxes
  [[RDFox-vs-Jena][rdfoxjena]]). Useful precisely because it is the JVM baseline most users run.
- **Do NOT benchmark against OWL-DL reasoners (Openllet/Pellet, HermiT, ELK, Konclude)** for
  closure/materialisation — different, harder problem (tableau / consequence-based
  classification). `research/inference-sota.md` says so explicitly. Mention only as "out of
  scope" so nobody wires the wrong target.

### HDT — `sparq-hdt`
- **Comparators: hdt-cpp and hdt-java** (the rdfhdt reference implementations). `sparq-hdt`
  wraps the Rust `hdt` crate, which decodes the standard HDT v1.0 layout produced by
  hdt-cpp/hdt-java. Fair benchmark: **load time + memory + triple-pattern resolution on the same
  `.hdt` archive** across the three decoders.
- **Caveats:** (1) mapped (`mapHDT`, mmap) vs loaded (`loadHDT`, full in-RAM) must be matched.
  (2) Central apples-to-oranges: `sparq-hdt` **decodes HDT into sparq's own Dict/Graph** (HDT as
  an *ingest format*) and then queries its own permutation indexes; hdt-cpp/java **query the
  compressed BitmapTriples structure in place** (HDT as a *queryable store*). Compare
  load-and-decode-to-native fairly; a "query over HDT" comparison is not like-for-like.
  (3) Encode-performance parity is a non-goal until the in-memory PFC+BitmapTriples encoder
  (bead **sq-ashy**) lands; round-trip correctness against an hdt-cpp/java archive (sparq already
  does this with vendored `snikmeta.hdt`) is the right *correctness* comparison.

### GeoSPARQL — `sparq-geo`
- **Compliance bar: GeoSPARQL-Jena (Fuseki-geosparql)** — the highest-compliance triplestore,
  the only one with full GML + WKT and all GeoSPARQL extensions ([spatial-RDF-store
  benchmark][spatialrdf]). Since `sparq-geo` implements both WKT and GML, Jena is the right
  like-for-like for **coverage/compliance**.
- **Performance peers: Virtuoso (perf bar), GraphDB (strong all-rounder)** — but both implement
  **only WKT** (no GML), so `sparq-geo` + Jena beat them on coverage; note the asymmetry.
- **Harness:** the existing `geosparql-benchmarking` framework (galbiston) + the GeoSPARQL
  Compliance Benchmark (Jovanovik et al. [[GeoSPARQL compliance][geocompliance]]); report
  compliance score alongside latency.
- **Loose lower bound only: PostGIS** — a relational spatial DB, not RDF/SPARQL (no `geof:`, no
  graph joins). The literature is blunt that relational solutions generally outperform
  semantic-web spatial approaches [[GeoSPARQL+][geosparqlplus]] — informative for the R-tree
  (`rstar`) sub-component, not a SPARQL competitor. Match CRS/operation semantics (sparq-geo's
  planar/equirectangular ops vs PostGIS geodesic/projected) or the comparison is unfair.

### Streaming — `sparq-rsp` (flagged for completeness, not requested)
No comparator wired. The natural RSP-QL peers (C-SPARQL, CQELS, RSP4J/YASPER) are
service/runtime engines with wall-clock windows, whereas `sparq-rsp` is a deterministic,
clock-free library — any throughput comparison is apples-to-oranges (different time model).
Conscious omission, not an oversight.

---

## 5. Gather methodology (extends the existing CATALOG discipline)

sparq's CI box is a **shared EC2 instance where wall-clock is non-canonical**
(`quiet_box_sensitive=true` in the registry). Competitor numbers must NOT be gathered per-CI.
The methodology below extends the exact machinery already proven for the QLever baselines in
`bench/qlever-baselines.md`.

1. **Gather ONCE, not per-CI.** A single controlled session on an *otherwise-idle* box — mirror
   the QLever-baselines precedent.
2. **Same machine + same suite + same dataset-scale, all in one session.** Run sparq and every
   competitor back-to-back on the identical NT/TTL corpus and identical `.rq` queries at the
   identical scale tier, so only the engine varies. Reuse the existing seed-pinned per-suite
   corpora in `bench/{sp2b,watdiv,bsbm,lubm,dbpsb}/`.
3. **Cold vs warm, min-of-N, fixed regime.** Match CATALOG: QLever-style comparisons are COLD
   (cache cleared each run), report min-of-K, keep the regime constant across engines. Prefer
   compute-only `COUNT(*)` mode for the headline (no serialisation noise); end-to-end pass
   secondary.
4. **Cross-check correctness, not just speed.** Assert every competitor returns the same
   COUNT/result size before trusting its timing (the registry's "COUNT VALUES + result SIZES
   cross-checked, not row==row" rule).
5. **Version + env pinning recorded with every datapoint.** Engine version (Fuseki:
   `fuseki --version`; Virtuoso: build string; or the resolved Docker image **digest**, the
   QLever pattern) + the env block already defined: `host_class, cpu_model, nproc, os, kernel,
   quiet_box, gathered_at_utc, git_commit`.
6. **Integrate via the existing seam, no hard-coded numbers in git.** Results land in
   git-ignored `bench/competitor-results/<engine>-<suite>-<UTC>.json`; a maintainer deliberately
   promotes a reviewed snapshot into the `engines`/`values` keys of `bench/competitors.json`
   (which ship **EMPTY** in git). This satisfies AGENTS.md's no-hard-coded-perf rule. Extend
   `scripts/gather-competitors.sh` with an **HTTP-SPARQL adapter** so a new engine = a registry
   entry + endpoint URL, reusing the existing `df` watchdog / `/tmp` cleanup / `--run --only`
   guards.

> Note on the two `competitors.json` files (don't update the wrong one): **`bench/competitors.json`**
> is the gather **registry** (the `competitors` array) plus the optional seam for injected competitor
> values, and is the source-of-record for `scripts/gather-competitors.sh`. **`bench/dashboard/competitors.json`**
> is the canonical, human-reviewable **static dashboard snapshot**, mirrored byte-for-meaning into
> `bench/dashboard/dashboard.js` as `COMPETITORS_DATA` (and read via `window.COMPETITORS`); its per-metric
> cells ship **empty by design** until gathered on a quiet box. New engines are registered in
> `bench/competitors.json`; the dashboard's static numbers live in `bench/dashboard/competitors.json`.

---

## 6. Sequenced plan

**Wave 1 — cheapest high-value, open + dockerizable, publishable (do first):**
1. **Apache Jena Fuseki / TDB2** — Apache-2.0, first-party Docker, runs all SPARQL suites
   unmodified, the expected mainstream baseline. *Gather first.*
2. **Virtuoso Open Source (VOS 7)** — GPLv2, first-party maintained Docker, the universal
   literature baseline; one-time `isql` ingest scripting is the only extra cost. *Gather second.*

Both reuse the existing suite corpora and the gather-once methodology; the only new code is the
HTTP-SPARQL adapter in `scripts/gather-competitors.sh` (write once, both engines share it).

**Wave 2 — extra datapoints / scale / reasoning, when justified:**
3. **Eclipse RDF4J** (EDL/BSD, first-party Docker) — optional second JVM datapoint; partly
   redundant with Fuseki.
4. **VLog + Nemo** — open OWL-2-RL/Datalog materialisers; the missing *reasoning* baselines for
   the LUBM-OWL-RL / Deep Taxonomy tier (alongside the already-tracked EYE).
5. **MillenniumDB** — only when sparq runs WDBench/Wikidata at scale (no official image +
   SPARQL→MDQL translation = high cost).
6. **Blazegraph** — a single historical reference line only, to frame the Wikidata migration
   narrative.

**Per-surface harnesses (separate beads, separate axes):**
7. **`sparq-text` vs jena-text / Lucene** (FTS harness).
8. **`sparq-vectors` via `ann-benchmarks`** (hnswlib/FAISS/ScaNN/DiskANN, recall–QPS).
9. **`sparq-geo` vs GeoSPARQL-Jena** (compliance + latency via geosparql-benchmarking).
10. **`sparq-hdt` vs hdt-cpp/hdt-java** (load/decode/triple-pattern on the same `.hdt`).

**Deferred / license-gated (do NOT publish without action):**
- **GraphDB Free** — Ontotext DeWitt clause (Art. 15.3) → written permission required + license
  key. needs:user.
- **RDFox** — proprietary; the reasoning SOTA bar but needs a license. needs:user.
- **Stardog** — license key + Docker-by-request + query-rewrite reasoning (not apples-to-apples).
- **Amazon Neptune** — managed-only (AWS), not reproducibly benchmarkable.

---

## Repo seam (paths)

- `bench/competitors.json` — gather **registry** (`competitors` array) + optional seam for injected
  competitor values; source-of-record for `scripts/gather-competitors.sh`. Tracks Oxigraph, QLever,
  EYE today. **Add Fuseki + Virtuoso here.**
- `bench/dashboard/competitors.json` — canonical, human-reviewable **static dashboard snapshot**
  (mirrored into `bench/dashboard/dashboard.js` as `COMPETITORS_DATA`); per-metric cells empty by
  design until gathered on a quiet box. **Update the dashboard's static competitor numbers here.**
- `scripts/gather-competitors.sh` — gather orchestrator (dry-run by default; `--run --only`).
  **Add the HTTP-SPARQL adapter here.**
- `bench/CATALOG.md` — QUIET-BOX note + cold/warm conventions (the discipline this plan extends).
- `bench/qlever-baselines.md` — the existing "gathered-once, pinned reference" precedent to mirror.
- `bench/benchmarks.toml` — machine-readable suite catalog (formats/dirs the adapter targets).
- `bench/inference/eye-comparison.md` — the EYE N3-closure head-to-head runner.
- `research/inference-sota.md` — RDFox/VLog/Nemo/EYE reasoning SOTA.

## Sources

- [Patel-Schneider, Wikidata Workshop 2025 — full-dump engine study][ps2025] (Blazegraph /
  MillenniumDB / QLever / Virtuoso)
- [WDBench (Angles et al., ISWC 2022)][wdbench] · [Sparqloscope (ISWC 2025, Freiburg)][sparqloscope]
- [DBPSB (Morsey et al.)][dbpsb] · [FEASIBLE (Saleem et al., ISWC 2015)][feasible]
- [Apache Jena Fuseki Docker][fusekidocker] · [jena-text][jenatext] · [RDFox vs Jena reasoning][rdfoxjena]
- [RDF4J Lucene SAIL (Solr deprecated)][rdf4jlucene] · [Apache Solr official image][solrimage]
- [ANN-Benchmarks][annbench]
- [Assessment & Benchmarking of Spatially-Enabled RDF Stores (MDPI)][spatialrdf] ·
  [GeoSPARQL Compliance Benchmark (MDPI)][geocompliance] · [GeoSPARQL+ (PostGIS comparison)][geosparqlplus]
- [Virtuoso (OpenLink)][virtuoso] · [Virtuoso OSS Docker][virtuosodocker] · [GraphDB Free license (DeWitt)][graphdblic]

[ps2025]: https://wikidataworkshop.github.io/2025/papers/paper3.pdf
[wdbench]: https://dl.acm.org/doi/10.1007/978-3-031-19433-7_41
[sparqloscope]: https://ad-publications.cs.uni-freiburg.de/ISWC_sparqloscope_BKTU_2025.pdf
[dbpsb]: https://jens-lehmann.org/files/2011/dbpsb.pdf
[feasible]: https://svn.aksw.org/papers/2015/ISWC_FEASIBLE/public.pdf
[fusekidocker]: https://jena.apache.org/documentation/fuseki2/fuseki-docker.html
[jenatext]: http://loopasam.github.io/jena-doc/documentation/query/text-query.html
[rdf4jlucene]: https://rdf4j.org/documentation/programming/lucene/
[solrimage]: https://hub.docker.com/_/solr
[annbench]: https://ann-benchmarks.com/
[rdfoxjena]: https://gist.github.com/justin2004/81951184f3dcb496e80eecdf09774b91?permalink_comment_id=4309078
[spatialrdf]: https://www.mdpi.com/2220-9964/8/7/310
[geocompliance]: https://www.mdpi.com/2220-9964/10/7/487
[geosparqlplus]: https://arxiv.org/pdf/2009.05032
[virtuoso]: https://virtuoso.openlinksw.com/
[virtuosodocker]: https://hub.docker.com/r/openlink/virtuoso-opensource-7
[graphdblic]: https://graphdb.ontotext.com/LICENSE-GraphDB-Free.txt
