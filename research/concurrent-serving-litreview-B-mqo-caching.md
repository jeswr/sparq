# Concurrent serving — literature review part B: MQO/work sharing + result/semantic caching

> NOTE for the concurrent-serving research agent: completed by a parallel research subagent
> (2026-06-12) before your run started — companion to
> `concurrent-serving-litreview-A-mvcc-benchmarks.md`. Fold into `research/concurrent-serving.md`
> instead of re-researching these topics. Verdicts are inputs; your spikes confirm or refute.

## Topic A — Multi-Query Optimization and Work Sharing

### A.1 Classical MQO

**Sellis, "Multiple-Query Optimization," ACM TODS 13(1):23–52, 1988** ([TODS](https://dl.acm.org/doi/10.1145/42201.42203), [PDF](https://www.cs.cmu.edu/~natassa/courses/15-721/papers/sellis88.pdf)): given a *batch*, detect common (sub)expressions and compute each once. Sellis & Ghosh proved the problem **NP-hard**; original algorithms explore an exponential plan space. **Roy, Seshadri, Sudarshan, Bhobe, SIGMOD 2000** ([PDF](https://www.cse.iitb.ac.in/~sudarsha/Pubs-dir/mqo-sigmod00.pdf)): greedy heuristics over an AND-OR DAG made MQO practical-ish.

**Why MQO is rarely used online:** (1) needs a batch known simultaneously, but interactive queries arrive one at a time; (2) optimization cost super-linear in batch size, NP-hard in general — optimizer time can exceed saved execution time for cheap queries; (3) materializing shared intermediates costs memory/IO that may never pay off; (4) holding queries to form batches adds latency. MQO survives in batch ETL/reporting and in spirit in shared-scan engines, which share *at runtime in the execution engine*, sidestepping plan-combination NP-hardness.

### A.2 Shared/cooperative execution systems

| System | Venue | Mechanism | Headline result |
|---|---|---|---|
| **QPipe** | [SIGMOD 2005](http://www.cs.cmu.edu/~StagedDB/papers/qpipe.pdf) | On-demand simultaneous pipelining: operator micro-engines pipeline output to multiple parent queries | **2× over a commercial DBMS** on concurrent TPC-H |
| **Cooperative Scans** (Zukowski et al.) | [VLDB 2007](https://dl.acm.org/doi/abs/10.14778/2367502.2367515) | CScan + Active Buffer Manager; relevance policy replaces LRU; out-of-order delivery | Beats normal/attach/elevator on NSM/PAX and DSM. NOTE: the VLDB 2012 follow-up reports Vectorwise **productized a simpler predictive buffer manager instead** — ABM invasiveness vs benefit is a real-world verdict |
| **Crescando** | [VLDB 2009](http://www.vldb.org/pvldb/vol2/vldb09-323.pdf) | Clock Scan: continuous cyclic main-memory scans; batched query-data joins | Guaranteed latency/freshness regardless of predicate mix; trades best-case point latency for worst-case guarantees |
| **DataPath** | [SIGMOD 2010](https://dl.acm.org/doi/10.1145/1807167.1807224) | Push-based data-centric engine | ~70M tuples/s on 1TB instance |
| **IBM Blink / DB2 BLU** | ICDE 2008, [VLDB 2008](http://www.vldb.org/pvldb/vol1/1453924.pdf), [VLDB 2013](https://dl.acm.org/doi/10.14778/2536222.2536233) | Every query a scan over compressed denormalized table; scans batched/shared across cores | Shipped commercially (Blink → DB2 BLU) |
| **SharedDB** | [VLDB 2012](http://vldb.org/pvldb/vol5/p526_georgiosgiannikis_vldb2012.pdf) | One global query plan for the whole workload; shared joins evaluate hundreds of queries at once | See verified numbers below |
| **BatchDB** | [SIGMOD 2017](https://www-db.cs.tum.edu/~giceva/papers/SIGMOD_batchdb.pdf) | Hybrid OLTP+OLAP via per-workload replicas; OLAP batched on a dedicated replica | Competitive on TPC-C and TPC-H simultaneously; **isolation** (not sharing) across the OLTP/OLAP boundary |

**SharedDB verified numbers (TPC-W vs MySQL and commercial "SystemX", CPU-bound, caching disabled):**
- Browsing mix (scan/join-heavy): **2× SystemX, 8× MySQL** throughput (~1,500–2,100 WIPS @32 cores).
- **Ordering mix (point queries + updates): "SharedDB still wins… but the margins are lower. Most queries are point queries that can be executed highly efficiently with an index look-up… there is little benefit for sharing for such point queries."** For the lightweight search-item query SystemX executes batches *faster* — "the overhead of batching queries and updates is greater than the gains."
- Load-interaction: at fixed 400 light q/s, adding heavy queries collapses MySQL/SystemX; SharedDB throughput increases monotonically with concurrency.

### A.3 When sharing wins vs loses — the two verdict papers

**Johnson et al., "To Share or Not To Share?", VLDB 2007** ([PDF](http://pandis.net/resources/vldb07johnson.pdf)): aggressive work sharing is NOT always good on multicores. Sharing speeds up a uniprocessor up to **1.8×**, but a shared operator is a **serialization point**; as cores grow, independent parallel execution can beat sharing (sharing destroys parallelism). Contributes an analytical model; advocates deciding *at runtime per-opportunity*.

**Psaroudakis, Athanassoulis, Ailamaki, PVLDB 6(9):637, 2013** ([PDF](http://www.vldb.org/pvldb/vol6/p637-psaroudakis.pdf)) — integrates QPipe-style SP and SharedDB-style GQP in one engine; cleanest empirical answer:
- **High concurrency: sharing wins big** — shared circular scans cut response times **80–97%**; GQP dominates query-centric.
- **Low concurrency: sharing loses** — shared operators perform worse than query-centric (bookkeeping + lost intra-query parallelism).
- SP's serialization point is an artifact of push-based communication; pull-based **Shared Pages Lists** eliminate it (82–86% better at high concurrency vs push SP).
- Rules of thumb (their Table 1): low concurrency → query-centric + SP; high concurrency → GQP + SP; shared scans in the I/O layer in both — for ad-hoc scan-heavy OLAP over relatively static data.

**Synthesis (A):** sharing wins when queries are scan/join-heavy and similar, concurrency is high enough that contention (not parallelism) is the bottleneck, and SLAs tolerate batching. Sharing **loses** for point-query/OLTP work (SharedDB's own Ordering result), at low concurrency on multicores (Johnson 2007), and with little overlap. Production absorbed the cheap end (shared buffer-aware scans: BLU, SQL Server merry-go-round, Vectorwise predictive buffering), not full global-plan MQO.

## Topic B — Result / sub-plan / semantic caching and invalidation

### B.1 Relational foundations

- **Dar, Franklin, Jónsson, Srivastava, Tan, VLDB 1996** ([PDF](https://courses.cs.duke.edu/spring02/cps296.1/papers/DFJST-VLDB1996.pdf)): semantic regions (predicate constraints) describe the cache; incoming query split into cache-answerable part + **remainder query**; replacement by semantic distance. Cost: predicate containment/overlap reasoning.
- **Ivanova, Kersten, Nes, Gonçalves, SIGMOD 2009 "Recycler"** ([ACM](https://dl.acm.org/doi/10.1145/1559845.1559879)): MonetDB materializes every intermediate anyway → keep and reuse with cost-based admission/eviction. **~4× on SkyServer** (heavy inter-query overlap). Caveat: pipelined engines lack the free materialization — recycling hasn't generalized.
- Classic **view selection** (Harinarayan-Rajaraman-Ullman lattice): offline cousin; pays off for stable workloads + slow updates.

### B.2 SPARQL-specific caching

- **Martin, Unbehauen, Auer, ESWC 2010** ([Springer](https://link.springer.com/chapter/10.1007/978-3-642-13489-0_21)): app-side proxy cache; **invalidation by matching update triples against cached queries' graph patterns** (pattern-based, not TTL). Significant speedups for repeated queries; benefit shrinks as update rate rises.
- **Williams, Weaver, ISWC 2011** ([Springer](https://link.springer.com/chapter/10.1007/978-3-642-25073-6_48)): store indexes carry last-modified timestamps → endpoint computes per-query Last-Modified/ETag cheaply → standard HTTP caches do the rest. Invalidation becomes *validation*; no server-side tracking of cached queries.
- **Papailiou, Tsoumakos, Karras, Koziris, SIGMOD 2015** ([ACM](https://dl.acm.org/doi/10.1145/2723372.2723714)): the most sophisticated SPARQL cache — **canonical labelling indexes query graphs modulo isomorphism** + **generalization** (selective constants → variables so one cached sub-result serves many queries); adaptive promotion. **Up to two orders of magnitude** response-time reduction.
- **Salas, Hogan, ISWC 2018** (Best Student Paper; [extended PDF](https://aidanhogan.com/qcan/extended.pdf)) — verified numbers: over **768,618** unique SELECT strings from LSQ logs, syntactic normalization found 3,960 duplicates; QCan-Label (canonical variable labelling + pattern reordering) found **10,722 — 2.7× more**; full UCQ minimisation found NO additional duplicates on real logs. Cost: syntactic ≈0.1–0.3 ms; Label/Full 10–100 ms typical (mean ≈100 ms Full, worst real ≈2.5 s; doubly-exponential worst case only on synthetic queries). **Lesson: variable-renaming-level canonicalization is the cache-key sweet spot; semantic minimisation adds cost, no hits.**
- **Containment-based lookup is a dead end online:** NP-complete already for CQs and well-designed AND/OPT SPARQL (Letelier et al. PODS 2012/TODS 2013), **Π₂ᵖ-complete with UNION**, up to undecidable with projection ([Pichler-Skritek PODS 2014](https://www.researchgate.net/publication/266657751_Containment_and_equivalence_of_well-designed_SPARQL); [Kaminski-Kostylev ICDT 2016](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ICDT.2016.5)). Real caches use exact-match canonical keys or restricted generalization.
- **Triple Pattern Fragments** (Verborgh et al., ISWC 2014/JWS 2016): caching as *interface design* — single-triple-pattern responses are regular and highly HTTP-cacheable, at the cost of client work + round-trips.

### B.3 Production systems and repetition rates

- **Bonifati, Martens, Timm, PVLDB 11(2):149, 2017** ([PDF](http://www.vldb.org/pvldb/vol11/p149-bonifati.pdf)): of **180,653,910 logged queries** across DBpedia/Wikidata/British Museum etc., only 56,164,661 remain after string-level dedup — **~69% of endpoint traffic is exact duplicates**. Canonicalization raises it further (Salas-Hogan). WWW 2019 follow-up confirms for Wikidata.
- **Malyshev et al., ISWC 2018** ([PDF](https://iccl.inf.tu-dresden.de/w/images/5/5a/Malyshev-et-al-Wikidata-SPARQL-ISWC-2018.pdf)): of ~575M WDQS requests, organic queries ~0.6%; **~99.4% robotic**, templated and repetitive — ideal cache fodder but bursty/thrash-capable.
- **WDQS production** ([T126730](https://phabricator.wikimedia.org/T126730), [Wikitech](https://wikitech.wikimedia.org/wiki/Wikidata_query_service)): behind Varnish; per-entity invalidation REJECTED as infeasible ("nearly impossible without tracking all entities in results"); settled on **~60 s TTL Cache-Control** + client opt-outs. Freshness governed separately by **lag tolerance** (update lag feeds maxlag ×60, [T221774](https://phabricator.wikimedia.org/T221774)) — the canonical "generation/TTL + lag budget" production pattern. Blazegraph has no real result cache — hence HTTP-layer caching.
- **DBpedia/Virtuoso:** proxy result cache (exact-match, with invalidation) in front of Virtuoso + compiled-plan reuse; `MaxCacheExpiration` governs expiry.

### B.4 Invalidation strategies actually used

1. **TTL** (WDQS 60 s): correct-enough, bounded staleness, zero per-update work; wins when the app tolerates lag (Wikimedia engineered tolerance via maxlag).
2. **Validation/epoch tagging** (Williams-Weaver): per-index/partition last-modified generations; revalidate on hit. Cheap writes/reads, exact freshness; needs store cooperation.
3. **Predicate/pattern-partition invalidation** (Martin et al.): precise; cost grows with cache size × update rate; joins/OPTIONALs force conservative over-invalidation.
4. **Replica/batch decoupling** (BatchDB; WDQS updater): invalidation degenerates to replica refresh + lag bound.

### Honest summary — when each technique pays

- **Exact-match result caching with canonicalized keys** = highest-ROI for SPARQL endpoints: ~69% raw duplicate traffic, +2.7× hits from millisecond-cheap variable canonicalization. Pays almost unconditionally on read-mostly workloads; the design question is invalidation — production answers are TTL or epoch validation, NOT precise dependency tracking (WDQS rejected it).
- **Semantic/containment caching**: theoretically attractive, NP-hard-to-undecidable for SPARQL; restricted forms (Papailiou isomorphism + generalization) get most benefit (~100×) at engineering cost few systems paid.
- **Intermediate-result recycling**: needs an engine that materializes intermediates anyway + overlapping workload; poor fit for pipelined executors.
- **Execution-time work sharing**: wins for highly concurrent scan-heavy similar analytics (80–97% reductions; SharedDB 2–8×); in production as shared scans only. **Loses** for point-query mixes (batching overhead > index lookup — SharedDB's own data) and at low concurrency on multicores (serialization point).
- For sparq specifically: the proven cheap win is a **canonicalized exact-match result/sub-result cache with epoch- or TTL-based invalidation**; shared scans worth considering only under genuinely high concurrent analytical load.
