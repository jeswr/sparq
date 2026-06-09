# State of the Art in RDF / OWL / Notation3 Inference Engines: Benchmarks, Published Results, and Optimization Techniques

**A reference document for the `sparq` Rust forward-chaining materialization reasoner**

Scope: materialization (forward-chaining) reasoners for RDFS, OWL 2 RL, and Notation3-style rules + builtins. Primary-source numbers throughout, extracted directly from the papers' result tables (not paraphrased). Hardware is stated for every benchmark because the numbers are meaningless without it.

---

## 0. Executive summary (the load-bearing facts)

1. **Semi-naive (delta) evaluation is the single non-negotiable baseline.** Every serious engine — RDFox, VLog, Nemo, Soufflé — uses it. `sparq`'s naive fixpoint re-derives the *entire* closure every round; semi-naive joins only against the *newly derived* delta. RDFox, VLog, and Nemo are all explicitly semi-naive (Nemo: *"materializes inferences by semi-naive bottom-up evaluation… storing the fresh results of each rule application in separate delta tables"*). This is your #1 win.
2. **owl:sameAs via rewriting (union-find canonicalization) is worth up to 7.8× memory, 31.1× single-thread time, and 85.5× fewer derivations** on real data (RDFox AAAI-2015 numbers, below). The naive axiomatic "copy all triples between equal resources" approach is quadratic and is what kills engines on UniProt/OpenCyc/UOBM.
3. **RDFox's speed comes from (a) semi-naive + (b) lock-free parallel insertion via compare-and-set (CAS) into a 6-index in-memory triple table + (c) sameAs rewriting.** Parallel speedup is **up to 13.9× on 16 physical cores, 19.3× on 32 hyperthreads**, with lock-free concurrency overhead of only ~0–10% (single-thread parallel-build vs sequential-build).
4. **EYE wins the *rule-heavy, fact-light* regime; VLog/Nemo win the *fact-heavy, rule-light* regime.** Measured: on DeepTaxonomy-1000 EYE-fw = 0.1 s vs VLog 1.6 s / Nemo 1.7 s / cwm 180 s; on LUBM-100 (13.4 M facts) EYE *throws an exception*, while VLog = 47.3 s and Nemo = 362 s. **This split is `sparq`'s opportunity: a single engine that is fast in *both* regimes beats all of them.**
5. **`sparq`'s representation (FxHashSet of `[u32;3]`, dict-encoded) is already the right substrate** — dictionary-encoded integer triples are exactly what RDFox/VLog/Nemo use. The gap is purely algorithmic: naive→semi-naive, no sameAs canonicalization, per-round index rebuild, single-threaded.

---

## 1. The engines (SOTA landscape)

### Classification (read this first)

| Engine | Paradigm | Logic / profile | Language | Parallel | Incremental |
|---|---|---|---|---|---|
| **RDFox** | Materialization (forward, datalog) | OWL 2 RL + SWRL + datalog; equality | C++ | **Yes (lock-free)** | **Yes (DRed / B-F / Counting)** |
| **VLog / Rulewerk** | Materialization (forward, chase) | Datalog + existential rules + stratified negation | C++ (Java wrapper) | Limited | No (stateless re-mat) |
| **Nemo** | Materialization (forward, chase) | Datalog + existential + stratified negation + aggregates + datatypes | **Rust** | Some | No |
| **EYE / eye-js** | **Forward + backward** along Euler paths | Notation3 (full: builtins, blank-node production, scoped negation) | Prolog (YAP/SWI), WASM | No | No |
| **Soufflé** | Materialization (forward, compiled) | Datalog (+ magic sets, choice) | Compiles to C++ | **Yes (OpenMP)** | Partial |
| **GraphDB** | Materialization (forward) | RDFS, OWL 2 RL/QL, OWL-Horst | Java | Yes | **Yes (both directions)** |
| **Stardog** | **Query rewriting** (no materialization) | OWL 2 RL/QL/EL subsets + rules | Java | n/a | n/a (query-time) |
| **Oxigraph** | RDF store, **no built-in reasoning** | (SPARQL only; reasoning external) | **Rust** | n/a | n/a |
| **ELK** | Consequence-based saturation | **OWL 2 EL** | Java | Yes | Yes |
| **Konclude** | Tableau (+ saturation) | OWL 2 DL | C++ | Yes | No |
| **HermiT / Pellet / Openllet** | Tableau (hypertableau) | OWL 2 DL | Java | No | No |

> **The dividing line:** RDFox/VLog/Nemo/Soufflé/GraphDB/`sparq` = **materialization** (compute the closure once). Stardog = **query rewriting** (rewrite the query, leave data alone). Konclude/HermiT/Pellet = **tableau** (model-construction for full OWL DL satisfiability — a different, harder problem). `sparq` is in the materialization camp; your direct competitors are **RDFox, VLog, Nemo, EYE**.

---

### 1.1 RDFox (Oxford Semantic Technologies) — the SOTA materialization engine

**Reasoning model:** Main-memory, centralized, multi-core RDF store. Materialization-based parallel datalog reasoning for OWL 2 RL + SWRL, with native equality (`owl:sameAs`) handling. Stores dictionary-encoded triples in RAM.

**Key papers (all primary sources, verified):**
- Motik, Nenov, Piro, Horrocks, Olteanu, **"Parallel Materialisation of Datalog Programs in Centralised, Main-Memory RDF Systems"**, AAAI 2014. <https://www.cs.ox.ac.uk/people/boris.motik/pubs/mnpho14parallel-materialisation-RDFox.pdf>
- Motik, Nenov, Piro, Horrocks, **"Handling owl:sameAs via Rewriting"**, AAAI 2015. <https://www.cs.ox.ac.uk/people/boris.motik/pubs/mnph15owl-sameAs-rewriting.pdf>
- Motik, Nenov, Piro, Horrocks, **"Incremental Update of Datalog Materialisation: the Backward/Forward Algorithm"**, AAAI 2015. <https://www.cs.ox.ac.uk/boris.motik/pubs/mnph15incremental-BF.pdf>
- Hu, Motik, Horrocks, **"Optimised Maintenance of Datalog Materialisations"**, AAAI 2018 / arXiv:1711.03987. <https://arxiv.org/pdf/1711.03987>
- Nenov, Piro, Motik, Horrocks, Wu, Banerjee, **"RDFox: A Highly-Scalable RDF Store"**, ISWC 2015. <https://www.cs.ox.ac.uk/people/boris.motik/pubs/npmhwb15RDFox-scalable.pdf>

**What makes it fast (the four pillars):**

**(a) Semi-naive datalog evaluation.** Baseline; only new facts trigger new joins.

**(b) Lock-free parallel materialization.** The 2014 paper's core contribution. Triples are stored in a **six-column triple table** with hash-table indexes (`Ispo`, `Isp`, etc.) connecting triples sharing a subject/predicate/object into linked lists (s-list, p-list, o-list). Insertion is **"mostly" lock-free via compare-and-set (CAS)**:
> *"Lock-freedom is usually achieved using compare-and-set: CAS(loc, exp, new)… we lock the bucket using CAS so only one thread can claim it… if the triple table is big enough this requires only an atomic increment and is thus lock-free."*

Because a single CAS cannot atomically update multiple index locations, they use **multiword-CAS / descriptors** (Harris-Fraser-Pratt) for the rare multi-location case, falling back to localized bucket locking only on hash collisions. Work is **dynamically** distributed to threads (static assignment fails because rules are recursive and data is skewed) — each thread pulls triples and matches them against all rules.

**(c) owl:sameAs rewriting** (see §4.2).

**(d) Incremental maintenance** (DRed / B-F / Counting hybrids, see §4.6).

**RDFox vendor-stated scale** (Oxford Semantic marketing, lower confidence than the papers): up to **9.2 billion triples**, **~36.9 bytes/triple**, import **~1 M triples/s**, reasoning **up to 6.1 M triples/s**, **2–3 M inferences/s**. <https://www.oxfordsemantic.tech/rdfox>

---

### 1.2 EYE / eye-js — the Notation3 reasoner

**Reasoning model:** Forward *and* backward chaining **along Euler paths**. Forward chaining for `=>` rules; backward chaining for `<=` rules (treated as user-defined builtins). The **Euler path** heuristic — *"don't step in your own steps"* (after Euler's 1736 Königsberg bridges) — prevents re-deriving along a derivation path already taken, giving loop-freedom and avoiding redundant work on deeply recursive rule chains. <https://github.com/eyereasoner/eye/blob/master/README.md>

**Implementation:** Written in Prolog, runs on an **Euler Abstract Machine** core compatible with YAP and SWI-Prolog. Generates **proofs** using the `swap/reason` vocabulary (toggle with `--nope`). Builtins are extensible via a plugin mechanism. <https://github.com/eyereasoner/eye>

**eye-js / eyeling:** `eye-js` ships EYE (SWI-Prolog) compiled to **WebAssembly** for browser/Node. Per-release benchmarks at <https://eyereasoner.github.io/eye-js/dev/bench/>. **eyeling** is a separate compact pure-JS N3 reasoner (forward+backward over Horn rules), checked against the community N3 test suite. <https://github.com/eyereasoner/eyeling>

**Performance characteristics:** Dominant when **rules ≫ facts** (deeply recursive taxonomies); the Euler-path backward chaining is essentially goal-directed and skips irrelevant derivations. **Weak / fails when facts ≫ rules** — on LUBM-100 it *threw an exception after reading the input facts* (see §3). N3-as-existential-rules paper: <https://iccl.inf.tu-dresden.de/w/images/4/49/RR23-N3Rules.pdf>

---

### 1.3 VLog / Rulewerk (VLog4j) — column-oriented, memory-frugal

**Reasoning model:** Forward-chaining datalog + Horn existential rules (the **chase**) + stratified negation. Rulewerk is the Java API wrapper. <https://iccl.inf.tu-dresden.de/web/VLog/en>

**Key technique — columnar IDB storage.** Urbani, Jacobs, Krötzsch, **"Column-Oriented Datalog Materialization for Large Knowledge Graphs"**, AAAI 2016 / arXiv:1511.08915. <https://arxiv.org/pdf/1511.08915>
- Each derived predicate's facts (`Δᵢₚ`) are stored as a **tuple of columns**, not rows. Columns are **hierarchically sorted** (first column fully sorted; subsequent columns sorted within each prefix-group) and **run-length-encoded (RLE)**.
- Rules with constants in their heads produce **constant columns occupying almost no memory**; EDB columns can be referenced rather than copied.
- Joins use **merge joins** between the new result and previous `Δᵢₚ` tables, often concatenating *only the column needed for the join*.
- VLog combines an **on-disk EDB layer + in-memory columnar IDB layer** → runs huge materializations on a 16 GB laptop.

**Headline result:** VLog uses **6–46% of RDFox's memory** at competitive runtime (see §3 table — on a 16 GB Macbook, RDFox-seq ran *out of memory* on most datasets that VLog completed).

---

### 1.4 Nemo — the Rust competitor (study this one closely)

**Reasoning model:** Rust, in-memory. Datalog + **existential rules** (1-parallel restricted chase) + **stratified negation** + **aggregates** (`#count`, `#sum`, …) + datatypes/arithmetic. RDF/SPARQL-compatible data model. Scales to **hundreds of millions of facts on a laptop, several billion on a server**. <https://github.com/knowsys/nemo> · <https://arxiv.org/abs/2308.15897>

**Architecture (the parts `sparq` should copy):**
- **Semi-naive bottom-up evaluation**, *"storing the fresh results of each rule application in separate delta tables."*
- Data stored as **hierarchically-sorted, column-based tables** (VLog-inspired), compressed with **RLE-with-increments**, constants **dictionary-encoded to integer IDs**.
- Tables accessed as **trie structures** (Fredkin tries); conjunctions evaluated with **leapfrog trie-join** (worst-case-optimal join). Projection/reordering fall back to row-based temp tables.
- Variable ordering (= column order) **heuristically chosen** as the query plan; union of many delta tables mitigated by **caching**.
- Papers: KR 2024 toolkit paper <https://proceedings.kr.org/2024/70/kr2024-0070-ivliev-et-al.pdf>; Datalog 2.0 paper <https://iccl.inf.tu-dresden.de/w/images/6/61/Ivliev-Datalog20-2024.pdf> (self-reported: *"competitive with other mature systems, often outperforming most… second place in these evaluations"*).

> **`sparq` takeaway:** Nemo is a Rust engine that already does semi-naive + columnar tries + leapfrog join. To *beat* Nemo you must match its data layout and join algorithm *and* add what it lacks (native owl:sameAs equality, EYE-style builtins, possibly better parallelism). Note `sparq`'s sibling research cache already contains the leapfrog-triejoin and worst-case-optimal-join literature.

---

### 1.5 Soufflé — compiled datalog (for technique transfer)

Synthesizes **parallel C++** (OpenMP) from datalog. Compiles datalog → **RAM (Relational Algebra Machine)** IR → C++. Uses **semi-naive evaluation + magic-set transformations**, and an **auto-scheduler / join optimizer** that finds join orders competitive with hand-tuned ones (LOPSTR 2022). Specialized concurrent data structures (B-trees, Bries). Not an RDF/OWL tool, but the **gold standard for datalog *execution* engineering**. <https://souffle-lang.github.io/lopstr22.html> · <https://souffle-lang.github.io/benchmarks>

---

### 1.6 GraphDB, Stardog, Oxigraph

- **GraphDB (Ontotext):** forward-chaining materialization; RDFS, OWL 2 RL/QL, OWL-Horst rulesets; **incremental in both directions** (insert → infer, delete → retract unsupported inferences). <https://graphdb.ontotext.com/documentation/10.7/reasoning.html>
- **Stardog:** **query rewriting**, not materialization — *"reasoning is performed at query time… it does not materialize inferences."* Pay-as-you-go; supports OWL 2 QL/RL/EL axiom subsets + rules via the "Blackout" reasoner. (Note: the older claim that Stardog materializes `owl:sameAs` eagerly while rewriting everything else was **not confirmed** in current docs — treat as unverified.) <https://docs.stardog.com/inference-engine/>
- **Oxigraph (Rust):** a SPARQL store with **no built-in reasoner**; reasoning must be done externally (e.g., apply rules and load results). Relevant to `sparq` only as a storage-layer comparison point. <https://github.com/oxigraph/oxigraph/discussions/401>

---

### 1.7 OWL DL reasoners (context only — different problem)

- **ELK** — consequence-based **OWL 2 EL** classifier; extremely fast on large EL ontologies (SNOMED CT, GO). Won the EL tracks of ORE 2015.
- **Konclude** — C++ tableau+saturation OWL 2 DL; **won 4 of 6 ORE-2015 tracks**.
- **HermiT** (hypertableau), **Pellet/Openllet** (tableau) — OWL 2 DL, Java; correctness reference but slow, largely unmaintained (HermiT *"not maintained at all"* per the 2023 survey). <https://pmc.ncbi.nlm.nih.gov/articles/PMC6044265/>

> These solve **OWL DL satisfiability/classification** (model construction), not bottom-up fact materialization. `sparq` does **not** compete here — RL/N3 materialization is a deliberately tractable fragment. Mentioned for completeness so you don't accidentally benchmark against the wrong target.

---

## 2. Benchmarks (datasets, queries, what they test)

### 2.1 LUBM — Lehigh University Benchmark (the standard)
- Guo, Pan, Heflin, *J. Web Semantics* 2005. <https://www.sciencedirect.com/science/article/abs/pii/S1570826805000132>
- University-domain OWL ontology + **synthetic, scalable, repeatable** ABox generator. Sizes: **LUBM-1, -8, -100, -1000, -8000** universities (LUBM-n). LUBM-100 ≈ 13 M triples; LUBM-1000 ≈ 130 M; LUBM-8000 ≈ 1 B.
- **14 SPARQL queries**, several of which require RDFS/OWL inference (subclass/subproperty transitivity, domain/range, `inverseOf`, etc.). Tests whether inferred answers are returned.
- **Caveat (from RDFox authors):** LUBM generates *multiple nearly-disconnected* university subgraphs, so it under-tests cross-graph join/inference hardness. Its ontology does **not** use `owl:sameAs`.

### 2.2 UOBM — University Ontology Benchmark
- Ma et al. 2006. Extends LUBM with **more complete OWL** (richer class axioms, `owl:sameAs`, more interconnection). Used in the RDFox papers as `UOBM_L` / `UOBM_U` (lower/upper-bound RL approximations). Harder for equality reasoning.

### 2.3 OWL2Bench
- Singh, Bhatia, Mutharaju, ISWC 2020. <https://dl.acm.org/doi/10.1007/978-3-030-62466-8_6> · <https://github.com/kracr/owl2bench>
- Extends UOBM. **TBox for each OWL 2 profile (EL, QL, RL, DL)** + scalable ABox generator + **22 reasoning SPARQL queries**. Tests profile support × ABox scalability × query performance.

### 2.4 LDBC Semantic Publishing Benchmark (SPB)
- BBC-derived; complex **queries under inference + continuous updates + failover**. The reference ruleset (RDFS-Plus-optimized) yields **~150 M implicit statements (1 : 1.6 expansion)** over the explicit triples. <https://ldbcouncil.org/benchmarks/spb/>

### 2.5 Real-world datasets (used in RDFox/VLog/incremental papers)
- **Claros** (cultural heritage; ~19 M base triples, ontology *not* in RL, has `sameAs`) — the canonical "hard recursive" dataset (`Claros-LE` adds hand-crafted hard rules).
- **DBpedia** (~113 M triples; `sameAs`-heavy).
- **UniProt** (proteins; ~123 M triples; very few merges but huge).
- **OpenCyc** (~2.4 M triples but **261 k rules, 3 781 sameAs-rules** — equality proliferates → worst case).
- **Reactome, ChEMBL** (biology/chemistry; used in the incremental-maintenance paper).

### 2.6 DeepTaxonomy (DT) — the rule-heavy stress test
- From the WellnessRules project; N3 at <http://eulersharp.sourceforge.net/2009/12dtb/>. **One single fact + a varying number of mutually-dependent subclass rules** (deeply nested RDFS subclass chain). Tests recursive rule-chaining depth, *not* fact volume. This is where EYE/backward-chaining shines and naive forward-chaining engines blow up.

### 2.7 ORE — OWL Reasoner Evaluation competition
- ORE 2013/2014/2015. Measured **classification, consistency, realisation** wall-clock + correctness across DL and EL profiles. **ORE-2015: Konclude won 4/6 tracks; ELK won the 2 EL tracks** (EL Consistency: ELK 425.1 s vs Konclude 1050.4 s). <https://pmc.ncbi.nlm.nih.gov/articles/PMC6044265/> — relevant as the OWL-DL context, *not* the RL-materialization target.

### 2.8 N3 / Notation3 test infrastructure (for `sparq`'s N3 mode)
- **N3 community test suite** (eyeling/eye are checked against it). <https://github.com/eyereasoner/eyeling>
- **N3 builtins spec** (W3C CG): six namespaces — **log: (18), math: (25), string: (16), list: (9), time: (7), crypto: (1)**. Argument modes are typed `++` (must be bound), `-` (output), `?` (bidirectional). <https://w3c-cg.github.io/n3Builtins/>
- Classic correctness tests: **socrates** (`type`/`subClassOf` → mortality), **graph/list** manipulation tests, **DeepTaxonomy**.

---

## 3. Published results (the numbers)

> **Read hardware carefully — the RDFox, VLog, and Nemo numbers are on *different machines* and are NOT directly comparable across tables.**

### 3.1 RDFox parallel materialization (AAAI 2014)
**Hardware:** Dell, **128 GB RAM, 2× Xeon E5-2650, 16 physical / 32 virtual cores**, Linux. (Comparison runs on 2× Xeon E5-2643, 8/16 cores, 100 GB cap.)

Materialization time (seconds) and **speedup vs 1 thread**, plus materialized triples & memory (from Table 2):

| Dataset (program) | 1 thr (s) | 8 thr | 16 thr | 32 thr | Max speedup | Triples | Memory |
|---|---|---|---|---|---|---|---|
| Claros_L 01K | 2477 | 333 (7.4×) | 179 (**13.9×**) | 127 (19.5×) | 19.5× | 95.5 M | 4.2 GB |
| Claros_LE 01K | 4989 | 773 | 415 (12.0×) | 285 (17.5×) | 17.5× | 555.1 M | 18.0 GB |
| DBpedia_L | 161 | 28 | 26 | 24 | 6.6× | 118.3 M | 6.1 GB |
| DBpedia_LE | 9075 | 1453 | 828 (11.0×) | 602 (15.1×) | 15.1× | 1529.7 M | 51.9 GB |
| LUBM_L 01K | 73 | 14 | 8 (8.7×) | 7 | 10.9× | 182.4 M | 9.3 GB |
| LUBM_LE 05K | 947 | 155 | 88 (10.8×) | 71 | 13.4× | 332.6 M | 13.8 GB |
| UOBM_U 010 | 4859 | 745 | — | — | — | 1661.0 M | 75.5 GB |

**Concurrency overhead:** comparing the **sequential build** vs the **1-thread parallel build**, the lock-free machinery adds only roughly **−5% to +10%** overhead (column "Par. imp." ranged −4.9% to +9.5% on most datasets; an outlier +37% on one UOBM case). Conclusion: *"parallelisation pays off even with just two threads"* (≈2× at 2 threads). Source: <https://www.cs.ox.ac.uk/people/boris.motik/pubs/mnpho14parallel-materialisation-RDFox.pdf>

### 3.2 owl:sameAs — Rewriting (REW) vs Axiomatization (AX) (AAAI 2015)
**Hardware:** Dell, **128 GB RAM, 2× Xeon E5-2643, 8 physical / 16 virtual cores**, Fedora 20.

**Total-work reduction (Table 2), AX → REW factor:**

| Dataset | Triples after AX | Triples after REW | **Triple factor** | Mem AX→REW | **Mem factor** | Rule appl. factor | **Derivations factor** | Merged resources |
|---|---|---|---|---|---|---|---|---|
| Claros | 102 M | 79.7 M | 1.28× | 4.5→3.6 GB | 1.28× | 5.8× | **85.5×** | 12 890 |
| DBpedia | 139 M | 136 M | 1.2× | 6.9→7.0 GB | 0.99× | 21.0× | 24.4× | 7 430 |
| **OpenCyc** | 1 176 M | 142 M | **7.8×** | 35.9→4.6 GB | **7.8×** | 25.3× | 45.9× | 361 386 |
| UniProt | 228 M | 228 M | 1.0× | 15.1→15.1 GB | 1.0× | 6.9× | 8.5× | 5 |
| UOBM | 36 M | 9.7 M | 3.2× | 1.2→0.4 GB | 3.2× | 9.9× | 3.8× | 686 |

**Materialization time (Table 3), single-thread:**

| Dataset | AX 1-thr (s) | REW 1-thr (s) | **REW speedup over AX** |
|---|---|---|---|
| Claros | 2042.9 | 65.8 | **31.1×** |
| UniProt | 370.6 | 143.4 | 2.6× |

Both modes parallelize ~6–7× on 8 physical cores. **Key insight:** the win tracks the number of merged resources — UniProt (5 merges) sees ~no triple reduction but still 8.5× fewer derivations; OpenCyc (361 k merges) sees the full 7.8× memory / 45.9× derivation collapse. Source: <https://www.cs.ox.ac.uk/people/boris.motik/pubs/mnph15owl-sameAs-rewriting.pdf>

### 3.3 VLog vs RDFox — memory (AAAI 2016)
**Hardware:** **Macbook Pro, 2.2 GHz Core i7, 16 GB RAM** (deliberately constrained). RDFox = the 2014 sequential release.

Materialization time (s) / peak memory (MB) (Table 2):

| Data/Rules | RDFox-seq time | RDFox-seq mem | VLog time | VLog mem | Derived (IDB) |
|---|---|---|---|---|---|
| LUBM-1K / L | 82 | 11 884 | 38 | **2 198** | 172 M |
| LUBM-1K / U | 148 | 14 593 | 80 | 2 418 | 197 M |
| LUBM-1K / LE | **oom** | oom | 2175 | 9 818 | 322 M |
| LUBM-5K / L | **oom** | oom | 196 | 8 280 | 815 M |
| LUBM-5K / U | **oom** | oom | 434 | 7 997 | 994 M |
| DBpedia / L | 177 | 7 917 | 91 | 532 | 33 M |
| Claros / L | **oom** | oom | 644 | 2 406 | 89 M |
| Claros-S / LE | 8.5 | 271 | 2.5 | 127 | 3.7 M |

**Takeaway:** on 16 GB, RDFox-seq runs out of memory on LUBM-5K/Claros while VLog completes — VLog's columnar+RLE layout is the memory enabler, often at *lower* runtime too. Source: <https://arxiv.org/pdf/1511.08915>

### 3.4 EYE vs cwm vs VLog vs Nemo (RR 2023 — the N3 paper, primary source)
**Hardware:** laptop, **11th-gen Intel Core i7-1165G7, 32 GB RAM**. Timeout = 10 min ("—").

| Dataset | #facts | #rules | cwm | EYE-fw | EYE-bw | VLog | Nemo |
|---|---|---|---|---|---|---|---|
| **DT 1000** | 1 | 3 001 | **180 s** | **0.1 s** | **0.001 s** | 1.6 s | 1.7 s |
| **DT 100000** | 1 | 30 001 | — (timeout) | 0.3 s | 0.003 s | — | — |
| **LUBM-001** | 100 543 | 136 | 117.4 s | 0.2 s | — | 2.4 s | 5.3 s |
| **LUBM-010** | 1 272 575 | 136 | — | 4.3 s | — | 31.2 s | 44.8 s |
| **LUBM-100** | 13 405 381 | 136 | — | **exception** | — | **47.3 s** | **362 s** |

**The decisive split (verbatim findings):**
- *"EYE performs much better than VLog and Nemo for the experiments with DT. Its reasoning time is off by one order of magnitude."* (Backward EYE is another ~100× faster than forward on DT.)
- *"VLog and Nemo could reason over all the LUBM datasets while EYE has thrown an exception after having read the input facts."*
- *"VLog [is] significantly lower than… EYE… Nemo… slower on LUBM."* cwm is uniformly slowest and times out from LUBM-010.

Source: <https://iccl.inf.tu-dresden.de/w/images/4/49/RR23-N3Rules.pdf> (= "Existential Notation3 Logic", arXiv:2308.07332 / TPLP).

### 3.5 Incremental maintenance (AAAI 2018, "Optimised Maintenance")
**Hardware:** Dell PowerEdge R720, **256 GB RAM, 2× Xeon E5-2670 2.6 GHz**, Fedora 24.
**Benchmarks:** UOBM, Reactome, UniProt, ChEMBL, Claros-LE, + synthetic SSPE (single-source path enumeration: 100 k-node, 1 M-edge DAG, builtin-heavy).
**Finding:** hybrid **DRed_c / B/F_c** (DRed/B-F + Counting) are *"usually significantly faster than existing approaches, sometimes by orders of magnitude,"* with **negligible counter-maintenance overhead**; pure DRed's overdeletion and pure B/F's "backward" rule evaluation are the bottlenecks the hybrids remove. The B/F algorithm alone was already *"several orders of magnitude more efficient than DRed on some inputs, and never significantly less efficient."* Sources: <https://arxiv.org/pdf/1711.03987> · <https://www.cs.ox.ac.uk/boris.motik/pubs/mnph15incremental-BF.pdf>

### 3.6 Numbers I could NOT pin down (stated honestly)
- **No single head-to-head table of RDFox vs VLog vs Nemo on identical hardware for LUBM-8000.** Each paper uses its own machine; cross-paper comparison is unsound.
- **RDFox LUBM-8000 (1 B triple) wall-clock** is not in the public papers (the 2014 paper tops out at LUBM-5K/UOBM-010); the vendor "6.1 M triples/s" figure is marketing, not peer-reviewed.
- **GraphDB / LDBC-SPB materialization times** are version-specific in Ontotext docs and not reported as a clean table here.

---

## 4. Optimization techniques (the actionable core)

### 4.1 Semi-naive evaluation — *THE* baseline (your #1 win)
**The technique.** Naive evaluation re-derives the *entire* closure every round: round *i+1* re-applies all rules to *all* facts, rediscovering everything already known. Semi-naive keeps a **delta Δᵢ** = facts newly derived in round *i*, and for each rule with body atoms `B₁…Bₙ` generates **n delta-rules**, each requiring at least one body atom to match Δ (the rest match the full relation). Thus round *i+1* only does joins that *involve at least one new fact* — every join touches the frontier, never the stale interior.

**Quantifying `sparq`'s naive penalty.** With a transitive/recursive program producing a closure of *N* facts over *R* rounds, naive evaluation does ~*O(R · cost(full join over all N))* work; semi-naive does ~*O(Σ cost(join over Δᵢ))*. For a long subclass/subproperty chain (LUBM, DeepTaxonomy) the naive cost is roughly **quadratic-to-cubic in closure size**, semi-naive is roughly **linear in the number of *derivations***. Published incremental/semi-naive speedups land at **1.6×–33×** (and on the DeepTaxonomy-style deep-recursion case, the gap is the difference between "completes in 0.1 s" and "times out"). **This is the highest-leverage single change for `sparq`.**

**How it maps to `sparq`.** Keep your `FxHashSet<[u32;3]>` as the cumulative store `I`. Maintain a second set `Δ` = triples derived *this* round. Each round: for every rule, for each body-atom position *k*, run the join with atom *k* bound to `Δ` and the others to `I`; collect new triples into `Δ_next`; `I ∪= Δ_next`; stop when `Δ_next` empty. This alone eliminates your per-round full re-derivation.
Refs: Soufflé semi-naive <https://souffle-lang.github.io/lopstr22.html>; Nemo delta tables <https://proceedings.kr.org/2024/70/kr2024-0070-ivliev-et-al.pdf>; survey <https://arxiv.org/pdf/1812.03975>.

### 4.2 owl:sameAs / equality rewriting (union-find canonicalization) — your #2 win
**The problem.** OWL 2 RL axiomatizes `owl:sameAs` as a congruence (reflexive/symmetric/transitive + replacement): for every pair of equal resources, *copy every triple* mentioning one to the other. A clique of *n* mutually-equal resources, each in triples with *nₛ × nₚ × nₒ* shapes, explodes to *nₛ·nₚ·nₒ* copies each re-derived many times. RDFox paper: a single tiny example derives the two `sameAs` triples **22 times** and the consequent triples grow **from ~60 to 6** after rewriting.

**The fix — representative selection + rewriting (RDFox's approach):**
1. Maintain a **union-find** over resources. When `⟨a, owl:sameAs, b⟩` is derived, union them and pick a **canonical representative** (RDFox picks deterministically, e.g., lowest ID).
2. **Rewrite both triples *and rules*** to the representative. RDFox stresses you must rewrite the *rules* too, not just the data, or query answers change — *"no existing system implements rule rewriting; certainly OWLIM SE and Oracle's RDF store do not, and so rewriting in these systems is not guaranteed to preserve query answers."*
3. Because `sameAs` facts are derived *during* materialization (not known up front), rewriting is **interleaved**: a thread can (a) apply a rewritten rule, (b) rewrite an "outdated" fact (one containing a non-representative), or (c) on a new `sameAs`, update the union-find and queue rewrites. **Lock-free** in RDFox.
4. Triples mentioning replaced resources are **marked, not deleted** (cheaper); marked triples are negligible in practice.

**Impact (from §3.2):** up to **7.8× less memory, 31.1× faster (single-thread), 45.9–85.5× fewer derivations.** The win scales with the number of merges — so it's nearly free when there are no equalities and decisive when there are many (UniProt-style large data, OpenCyc-style equality-dense ontologies, UOBM).

**How it maps to `sparq`.** Add a `union_find: Vec<u32>` over dictionary IDs. Intercept `owl:sameAs` derivations → `union(a,b)`. Define `canon(id) = find(id)`. Store triples in *canonical* form `[canon(s), canon(p), canon(o)]`; when a merge changes a representative, lazily re-canonicalize affected triples (iterate the index bucket for the merged ID). Rewrite rule constants through `canon` too. Emit the equivalence-class expansion only at *query* time, not in the store. This avoids the quadratic copy entirely. **Explicitly requested by the user and the second-biggest lever after semi-naive.**
Ref: <https://www.cs.ox.ac.uk/people/boris.motik/pubs/mnph15owl-sameAs-rewriting.pdf>

### 4.3 Magic sets / demand transformation (goal-directed materialization)
**The technique.** Rewrite the program w.r.t. a query goal so bottom-up evaluation only derives *query-relevant* facts — "simulate top-down with bottom-up." Adds `magic_p(…)` "demand" predicates that gate rule firing on whether a fact is actually needed. Standard for query-time reasoning; **Soufflé** applies it automatically. Extensions exist for stratified negation (*"Extended Magic for Negation"* <https://arxiv.org/pdf/1909.08246>) and recently DatalogMTL (AAAI 2025 <https://arxiv.org/abs/2412.07259>).

**Relevance to `sparq`.** For *full* materialization you derive everything, so magic sets don't help. But if `sparq` ever answers *specific* SPARQL/N3 queries under reasoning without full materialization (Stardog-style), magic sets give goal-directedness — and note **EYE's backward chaining is essentially a hand-rolled goal-directed evaluation**, which is *why* EYE crushes DeepTaxonomy. A hybrid (`sparq` materializes forward but switches to magic-set/backward for deeply-recursive, fact-light subprograms) is exactly how you'd beat *both* EYE and VLog in *both* regimes.

### 4.4 Rule indexing & rule-body join optimization (RETE / TREAT / join order)
**RETE** (Forgy): compile rules into a **discrimination network** (alpha nodes = single-atom filters; beta nodes = joins storing **partial-match tokens**); incrementally update partial matches as facts are inserted. Great for production-rule incremental matching; **memory-heavy** (stores all partial matches).
**TREAT** (Miranker): like RETE but **does not store beta partial matches** — recomputes joins on demand, trading time for far less memory; often wins when working memory changes a lot.
**For a datalog materializer**, the practical equivalents are:
- **Rule-to-fact index** — given a newly-derived fact, find the rules/positions it can fire. RDFox builds `rulesFor(F)` by generating the 8 wildcard generalizations of a triple and looking them up. `sparq` should index rules by their body-atom *patterns* (predicate + which positions are constants) so a new triple immediately yields candidate rule firings — instead of scanning all rules.
- **Join-order optimization within rule bodies.** The order you bind body atoms dominates cost (Soufflé's auto-scheduler finds expert-competitive orders). Heuristic: bind the **most selective atom first** (most-bound variables / smallest matching relation), and reuse bindings left-to-right. For `sparq`, estimate selectivity from per-predicate cardinalities (you already have the counts) and reorder body atoms once per program.
- **Worst-case-optimal joins (leapfrog triejoin).** Nemo uses leapfrog trie-join over sorted columnar tries — provably optimal for cyclic joins where binary joins blow up. Your research cache already holds this literature; relevant if `sparq` adds path/triangle-style rule bodies.
Refs: RETE <https://en.wikipedia.org/wiki/Rete_algorithm>; Soufflé join optimizer <https://souffle-lang.github.io/lopstr22.html>; Nemo leapfrog <https://proceedings.kr.org/2024/70/kr2024-0070-ivliev-et-al.pdf>.

### 4.5 Parallel materialization (RDFox's lock-free design)
**The technique.** Multi-core, **dynamic** work distribution (static fails: rules are recursive, data is skewed). Each thread repeatedly pulls a derived fact and matches it against all candidate rules. The shared store must support **concurrent insertion**:
- **6-index in-memory triple table**; insertion is **mostly lock-free via CAS**. Bucket claim = single CAS; appending to the triple table = atomic increment (lock-free if capacity suffices); the rare multi-index atomic update uses **multiword-CAS / descriptors**; localized locking only on hash collisions.
- **Data races avoided** because the only truly-atomic primitive is single-location CAS, so all multi-location index updates are either decomposed into independently-lock-free steps (linked-list insertion where *"triples already exist in the table"*) or guarded by mCAS/descriptors. False sharing across cores' caches is acknowledged as the residual cost.
- **Measured:** up to **13.9× on 16 cores, 19.3× on 32 HT**, ~2× at 2 threads, lock-free overhead ≈ 0–10%.

**How it maps to `sparq`.** Your `FxHashSet<[u32;3]>` is *not* concurrent. Options: (a) shard the store by `hash(s) % T` so each thread owns a partition (lock-free within partition, message-pass across); (b) use a concurrent hash set (`dashmap`/`flurry`) and parallelize the **per-round delta application** with `rayon` — within a semi-naive round, rule firings are independent and can fan out across cores, joining the round at the fixpoint barrier. Parallelism is a **multiplier on top of** semi-naive, so do semi-naive first.
Ref: <https://www.cs.ox.ac.uk/people/boris.motik/pubs/mnpho14parallel-materialisation-RDFox.pdf>

### 4.6 Incremental maintenance (DRed / B-F / Counting)
**DRed (Delete/Rederive):** on deletion, **over-delete** everything transitively depending on removed facts, then **re-derive** those with surviving alternative derivations. Simple but over-deletes aggressively; re-derivation evaluates rules "backwards."
**B/F (Backward/Forward):** for each candidate deletion, check via combined backward+forward chaining whether an alternative derivation exists — **exact**, avoids over-deletion; *"several orders of magnitude more efficient than DRed on some inputs."* But the "backward" rule evaluation (matching a rule head to a fact, evaluating the partial body as a query = a query with *m+1* atoms) is itself costly, especially with builtins.
**Counting:** keep a **derivation counter** per fact; on update, increment/decrement; a fact is gone when its count hits 0 — **no backward evaluation**, but **only sound for non-recursive** rules.
**Hybrids (DRed_c / B/F_c):** use Counting for the non-recursive part (eliminating backward evaluation there) and DRed/B-F only for recursive rules — *"pay-as-you-go,"* negligible counter overhead, *"sometimes orders of magnitude"* faster.

**How it maps to `sparq`.** If you support data updates, the cheapest correct-for-all-programs approach is **counting + DRed fallback**: keep a `u32` support-count per derived triple (incremented per distinct derivation); deletion decrements and removes at 0; only recursive strata need DRed-style over-delete/re-derive. Start with full re-materialization (correct, simple), add counting once base materialization is fast.
Refs: <https://arxiv.org/pdf/1711.03987> · <https://www.cs.ox.ac.uk/boris.motik/pubs/mnph15incremental-BF.pdf>

### 4.7 Stratified negation & well-founded semantics (for N3 scoped negation)
A program is **stratified** if no predicate negatively depends (transitively) on itself. Partition predicates into **strata**; evaluate stratum-by-stratum, lowest first; a negated atom is false iff not yet derived in a lower stratum. For stratified programs this coincides with **well-founded / stable / perfect** model semantics — a clean, universally-accepted answer.
**N3 specifics:** `log:notIncludes`, `log:notEqualTo`, `log:collectAllIn`, `log:forAllIn` implement **scoped negation-as-failure** — negation scoped to a named graph/blank-node, which the N3 spec characterizes as **monotonic** ("no other knowledge gained can influence the result") because the scope is closed. This means `sparq` can implement them by **stratifying on the negation scope**: fully materialize the scoped graph, *then* evaluate the `notIncludes` test. Refs: <https://w3c-cg.github.io/n3Builtins/> · stratified negation <https://www.ijcai.org/proceedings/2018/0259.pdf>.

### 4.8 Storage & indexing for rule matching
- **Which indexes:** RDFox's 6-permutation index (SPO, SOP, PSO, POS, OSP, OPS via linked lists) lets *any* triple pattern (1–2 bound positions) be matched without scanning. For rule bodies you typically need the indexes matching each atom's bound positions; a minimal set is often SPO + POS + OSP. `sparq`'s single `FxHashSet` supports only *membership*, not *pattern scan* — **you need at least one or two sorted/indexed permutations** to do body-atom joins efficiently. **This is your hidden bottleneck: per-round index rebuilds.**
- **Columnar vs row:** VLog/Nemo store IDB **column-wise, hierarchically sorted, RLE-compressed** → 6–46% of RDFox's memory and cache-friendly merge joins. RDFox is row-wise (the 6-column triple table) → faster point updates, more memory.
- **Avoid per-round index rebuilds:** with semi-naive you only *insert* the delta into persistent indexes; you never rebuild from scratch. Maintaining indexes incrementally (insert-only during materialization) is what makes the inner loop cheap.

### 4.9 Transitive-closure-specific & recursion
- For pure transitive properties (`owl:TransitiveProperty`, `rdfs:subClassOf+`), **specialized semi-naive TC** (or a dedicated reachability pass) beats general rule evaluation — fewer redundant joins, can use a frontier-based BFS. EYE's Euler-path dominance on DeepTaxonomy is essentially smart TC.
- **Stratify** the program: evaluate non-recursive strata once (no fixpoint), only iterate recursive strata. Nemo and the incremental paper both stratify; it cuts the number of facts entering the expensive fixpoint loop.

### 4.10 N3 / builtin-specific optimizations
- **Builtins are functional/relational filters**, not rules — bind their `++` (must-be-bound) args before evaluation, evaluate left-to-right; a builtin that can *generate* bindings (`-` mode, e.g. `list:member`) acts like a small relation. Order body atoms so builtins fire **after** their inputs are bound (the incremental paper notes builtins *restrict* body atom ordering — `t := exp` needs `exp`'s vars bound first).
- **Backward chaining for deep rule recursion** (EYE's lesson): for fact-light, rule-heavy subprograms, goal-directed/backward evaluation is orders of magnitude faster than forward materialization. A hybrid forward+backward `sparq` (forward by default, backward for flagged `<=` rules / deep taxonomies) directly targets EYE's only stronghold.
- **Proof generation** (EYE's `reason:` vocabulary) is optional but a differentiator; keep it toggleable (`--nope`-style) since it adds overhead.

---

## 5. Concrete recommendations for sparq (ranked by leverage)

Current state: naive fixpoint, `FxHashSet<[u32;3]>` dict-encoded, **per-round index rebuilds**, single-threaded, no equality handling.

| # | Optimization | Expected impact | Effort | Why |
|---|---|---|---|---|
| **1** | **Semi-naive evaluation** (delta tables) | **Large — likely 5–30×+ on recursive programs; turns timeouts into completions** (cf. EYE 0.1 s vs cwm 180 s gap is fundamentally this) | Medium | Naive re-derives the whole closure every round. This is the universal baseline every competitor has. **Do this first.** |
| **2** | **owl:sameAs union-find rewriting** | **Up to 7.8× memory, ~31× time, ~45–85× fewer derivations** on equality-heavy data (RDFox measured); ~free otherwise | Medium | Avoids the quadratic eq-copy blowup. Decisive on UniProt/UOBM/OpenCyc-class data. User-requested. |
| **3** | **Persistent multi-permutation indexes; stop per-round rebuilds** | Removes a hidden per-round *O(N)* cost; prerequisite for #1 to actually be fast | Medium | A single `FxHashSet` gives membership but not pattern-scan; rule-body joins need ≥2 sorted permutations maintained incrementally (insert-only). |
| **4** | **Rule-body join: indexing + selectivity-based reordering** | 2–10× on multi-atom rule bodies (Soufflé auto-scheduler territory) | Medium | Bind most-selective atom first; index rules by body pattern so a new triple maps directly to candidate firings (RDFox `rulesFor`). |
| **5** | **Stratification + dedicated transitive-closure pass** | Cuts fixpoint iterations; big on subclass/subproperty chains (the EYE/DeepTaxonomy regime) | Low–Med | Evaluate non-recursive strata once; use frontier-BFS for TC properties. |
| **6** | **Parallel materialization** (rayon over delta, or sharded concurrent store) | **Up to ~14× on 16 cores** (RDFox), ~2× at 2 threads, ~0–10% overhead | High | Multiplier on top of #1. Shard store by `hash(s)%T` or use a concurrent set; fan out independent rule firings within a round. |
| **7** | **Backward/goal-directed mode for deep, fact-light recursion** (magic sets or EYE-style) | Orders of magnitude on DeepTaxonomy-class inputs — the *only* place EYE currently beats VLog | High | A forward+backward hybrid lets one engine win **both** the rule-heavy (EYE) and fact-heavy (VLog) regimes — the explicit path to beating both. |
| **8** | **Columnar/RLE store for IDB** (VLog/Nemo style) | 2–16× less memory at competitive speed; enables larger datasets per machine | High | Only if memory becomes the binding constraint; row-wise is fine until then. |
| **9** | **Incremental maintenance** (counting + DRed fallback) | Enables fast updates without full re-materialization | High | Only if `sparq` needs live updates; defer until base materialization is fast. |

**Strategic framing — how `sparq` *beats* both EYE and RDFox:**
- **vs RDFox/VLog/Nemo (fact-heavy):** match their substrate — semi-naive (#1) + persistent indexes (#3) + sameAs rewriting (#2) + parallelism (#6). On the *same* hardware, a well-engineered Rust semi-naive parallel materializer with sameAs canonicalization is squarely competitive; Nemo already proves a Rust engine can reach "often outperforming most other tools."
- **vs EYE (rule-heavy):** EYE's *only* measured advantage is the DeepTaxonomy/deep-recursion regime, won by **backward/Euler-path goal-direction** (#7) — and EYE *throws an exception* on LUBM-100. A `sparq` that adds a backward/magic-set mode for flagged recursive subprograms covers EYE's stronghold while *also* scaling to large fact sets where EYE fails outright. **No existing single engine is fast in both regimes — that gap is the win condition.**

**Sequencing:** #1 → #3 → #2 → #4/#5 (correctness-preserving, big wins, moderate effort) before touching #6/#7/#8 (high effort, high ceiling). Benchmark continuously on **LUBM-1/8/100/1000** (fact-heavy), **DeepTaxonomy** (rule-heavy), **UOBM / UniProt** (equality-heavy), and the **N3 community test suite** (builtin correctness) so you can quote `sparq`-vs-EYE-vs-VLog-vs-Nemo on identical hardware — the one comparison the literature is *missing*.

---

## Sources (primary, with URLs)

**RDFox & equality / incremental:**
- Parallel materialization (AAAI 2014): <https://www.cs.ox.ac.uk/people/boris.motik/pubs/mnpho14parallel-materialisation-RDFox.pdf>
- owl:sameAs via Rewriting (AAAI 2015): <https://www.cs.ox.ac.uk/people/boris.motik/pubs/mnph15owl-sameAs-rewriting.pdf>
- Incremental B/F (AAAI 2015): <https://www.cs.ox.ac.uk/boris.motik/pubs/mnph15incremental-BF.pdf>
- Optimised Maintenance / Counting hybrids (AAAI 2018): <https://arxiv.org/pdf/1711.03987>
- RDFox: A Highly-Scalable RDF Store (ISWC 2015): <https://www.cs.ox.ac.uk/people/boris.motik/pubs/npmhwb15RDFox-scalable.pdf>
- Vendor page: <https://www.oxfordsemantic.tech/rdfox>

**VLog / Nemo:**
- VLog column-oriented (AAAI 2016): <https://arxiv.org/pdf/1511.08915>
- VLog system page: <https://iccl.inf.tu-dresden.de/web/VLog/en>
- Nemo toolkit (KR 2024): <https://proceedings.kr.org/2024/70/kr2024-0070-ivliev-et-al.pdf>
- Nemo (Datalog 2.0 2024): <https://iccl.inf.tu-dresden.de/w/images/6/61/Ivliev-Datalog20-2024.pdf>
- Nemo first glimpse (arXiv:2308.15897): <https://arxiv.org/abs/2308.15897>
- Nemo repo: <https://github.com/knowsys/nemo>

**EYE / N3:**
- EYE repo & README: <https://github.com/eyereasoner/eye> · <https://github.com/eyereasoner/eye/blob/master/README.md>
- eye-js (WASM): <https://github.com/eyereasoner/eye-js/> · benchmarks <https://eyereasoner.github.io/eye-js/dev/bench/>
- eyeling (pure JS): <https://github.com/eyereasoner/eyeling>
- N3 as Existential Rules / EYE-cwm-VLog-Nemo comparison (RR 2023): <https://iccl.inf.tu-dresden.de/w/images/4/49/RR23-N3Rules.pdf>
- Existential Notation3 Logic (arXiv:2308.07332 / TPLP): <https://arxiv.org/abs/2308.07332>
- N3 builtins spec: <https://w3c-cg.github.io/n3Builtins/>

**Benchmarks:**
- LUBM (J. Web Semantics 2005): <https://www.sciencedirect.com/science/article/abs/pii/S1570826805000132>
- OWL2Bench (ISWC 2020): <https://dl.acm.org/doi/10.1007/978-3-030-62466-8_6> · <https://github.com/kracr/owl2bench>
- LDBC SPB: <https://ldbcouncil.org/benchmarks/spb/>
- ORE 2015 report: <https://pmc.ncbi.nlm.nih.gov/articles/PMC6044265/>
- DeepTaxonomy N3: <http://eulersharp.sourceforge.net/2009/12dtb/>

**Optimization techniques / other engines:**
- Semi-naive survey: <https://arxiv.org/pdf/1812.03975>
- Magic sets for negation: <https://arxiv.org/pdf/1909.08246> · DatalogMTL magic sets: <https://arxiv.org/abs/2412.07259>
- Soufflé join optimizer (LOPSTR 2022): <https://souffle-lang.github.io/lopstr22.html> · benchmarks <https://souffle-lang.github.io/benchmarks>
- RETE: <https://en.wikipedia.org/wiki/Rete_algorithm>
- Stratified negation (IJCAI 2018): <https://www.ijcai.org/proceedings/2018/0259.pdf>
- GraphDB reasoning: <https://graphdb.ontotext.com/documentation/10.7/reasoning.html>
- Stardog inference: <https://docs.stardog.com/inference-engine/>
- Oxigraph reasoning discussion: <https://github.com/oxigraph/oxigraph/discussions/401>
- OWL Reasoners still useable in 2023 (arXiv:2309.06888): <https://arxiv.org/pdf/2309.06888>

---

**Research method note:** All numeric tables in §3 were extracted directly from the source PDFs via local text extraction (`pdftotext`) and read off the papers' own result tables, not paraphrased — the WebFetch summarizer could not parse the compressed PDFs, so I downloaded and parsed them locally to guarantee the figures are verbatim. Hardware is stated per-table because the RDFox (128 GB Xeon servers), VLog (16 GB Macbook), Nemo (server), and EYE/cwm comparison (32 GB i7 laptop) numbers are on different machines and must not be cross-compared.