# Concurrent serving — literature review part C: scheduling + Rust server architecture + streaming protocols

> NOTE for the concurrent-serving research agent: completed by a parallel research subagent
> (2026-06-12) before your run started — companion to parts A (MVCC+benchmarks) and B (MQO+caching).
> Fold into `research/concurrent-serving.md` instead of re-researching. Verdicts are inputs;
> your spikes confirm or refute.

# Topic A — Query/request scheduling theory and practice

## SRPT/SJF optimality and starvation

- **Schrage (1968)**, *Operations Research* 16(3):687–690 ([INFORMS](https://pubsonline.informs.org/doi/10.1287/opre.16.3.687)): preemptive SRPT minimizes jobs-in-system at every instant (hence mean response time via Little's law) — a sample-path result, arbitrary arrival/service processes.
- **Bansal & Harchol-Balter, SIGMETRICS 2001** ([PDF](https://www.cs.cmu.edu/~harchol/Papers/Sigmetrics01.pdf)): the standard rebuttal to "SRPT starves big jobs" — under heavy-tailed sizes and load < 1, *every* size class has lower expected slowdown under SRPT than Processor-Sharing at moderate loads; even at high load the penalty to the largest jobs is small and bounded. Practical aging fixes: floor on decayed priority (Umbra's p_min), MLFQ periodic boost, SRPT-with-fairness hybrids ([catalog](https://www.cs.cmu.edu/~harchol/scheduling.html)).
- **MLFQ**: Corbató CTSS (1962); OSTEP chapter is the canonical treatment. New jobs enter top queue; allotment-consumers demoted (approximates SJF without knowing sizes); periodic boost prevents starvation; cumulative allotment accounting prevents gaming-by-yielding. Kleinrock & Muntz (JACM 19(3), 1972) for the queueing analysis.

## Dean & Barroso, "The Tail at Scale," CACM Feb 2013 ([PDF](https://www.barroso.org/publications/TheTailAtScale.pdf)) — verified numbers

- **Fan-out amplification**: 1-in-100 slow at one server → fanning to 100 servers makes **63%** of user requests slow; 1-in-10,000 at 2,000 servers → ~1 in 5 exceed a second.
- Measured Google service: one random leaf 1/5/10ms (50/95/99%ile); 95% of leaves 12/32/70ms; 100% of leaves 40/87/**140ms** — "waiting for the slowest 5% of requests is responsible for half of the total 99%-ile latency."
- **Hedged requests** (duplicate after 95th-pct latency outstanding → ~5% extra load): BigTable 1,000-key read across 100 servers, hedge at 10ms cut 99.9th from **1,800ms to 74ms** at **+2% requests**.
- **Tied requests** (enqueue at two servers, first-to-start cancels the other): idle cluster 99%ile 67→42ms (−37%); under concurrent terasort 108→67ms; disk overhead <1%; send-delay 2× avg network message delay (~1ms).
- Also: differentiated service classes + small queues, **time-slicing expensive queries (Google web search interleaves expensive queries exactly so they "don't add substantial latency to a large number of concurrent cheaper queries")**, micro-partitions (~20/machine, shed in ~5% steps), latency-induced probation, canary requests.

## Morsel-driven parallelism — Leis, Boncz, Kemper, Neumann, SIGMOD 2014 ([PDF](https://db.in.tum.de/~leis/papers/morsels.pdf))

- Pipelines dispatched as ~100k-tuple **morsels** to core-pinned workers, one OS thread per HW thread, never oversubscribed; NUMA-local dispatch. **>30× average speedup on 32 cores** (TPC-H/SSB).
- Key scheduling property: parallelism elastic at morsel granularity — a thread switches queries *between morsels*, enabling inter-query reprioritization **without preemption**.

## Umbra's scheduler — Wagner, Kohn, Neumann, SIGMOD 2021 ([PDF](https://15721.courses.cs.cmu.edu/spring2023/papers/07-scheduling/wagner-sigmod21.pdf)) — exact mechanism

**Structure.** Pipeline → task set (tasks = morsel groups carved at runtime); query's task sets in a *resource group* (RG). Global slot array bounded at **128 active RGs** (overflow → wait queue). One OS thread per core.

**Stride scheduling, lock-free thread-local.** Task i, priority p_i, stride S_i = 1/p_i; min-pass task runs a slice, P_i += S_i. Global stride/pass (S_G = 1/Σp_k; P_G += S_G per slice) timestamps arrivals (new task pass = P_G). Non-preemptive: consumed fraction f of slice → P_i += f·S_i. Umbra's twist: ALL scheduling state thread-local; task-set arrival/finish communicated via two atomic bitmasks per worker (change/return masks, fetch_or + exchange). Decision <1µs; total overhead ≈0.05% (0.02% at 120 cores; 4-socket Xeon E7-4870v2 measured).

**Adaptive morsels.** Target task duration **t_max = 2ms**. Startup: exponentially growing morsels from **16 tuples** (Cᵢ = 2Cᵢ₋₁) to learn throughput; default: one morsel of T·t_max tuples (EWMA α = 0.8); shutdown ("photo finish"): when remaining < W·t_max, shrink to max(t/W, t_min) to avoid stragglers. Fixed 60k morsels vary **>30×** in duration across TPC-H pipelines.

**Priority decay (the anti-head-of-line mechanism).** Every RG starts p₀ = 10⁴. After each 2ms quantum consumed: p_{i+1} = p_i if i < d_start, else max(p_min, λ·p_i), λ∈[0,1], **p_min = 100 > 0 so queries never starve**. "Similar to MLFQ": priority depends on CPU consumed, so short queries stay high-priority their whole life; long queries decay to a background share; same-arrival queries decay identically (base-latency order preserved — their principle (1); minimize mean relative slowdown — principle (2)). New arrivals enter at p₀ ≫ decayed priorities → immediately dominate the stride allocation instead of queueing behind heavies.

**Self-tuning.** Tracker logs 20s every 60s; one worker simulates the scheduler on the tracked workload (cost = mean relative slowdown), derivative-free search over d_start (5–35% of morsels pre-decay) and λ; takes 20–100ms on 20 threads, <0.01% of processing time.

**Results.** TPC-H SF3 (75%) + SF30 (25%), Poisson arrivals, load α∈[0.7,1.0]: short-query geomean latency degrades only **17%** α=0.8→1.0 (fair scheduling: 63%) — 2× better than fair; **>4.5× better mean slowdown** than Umbra-old for SF3; **>5× better than FIFO** for short queries (>10× at α≥0.95); tails **>10×** better. Vs MonetDB at α=0.96: mean slowdown 4.5× better, **84% more queries/s**; vs PostgreSQL: >65× slowdown, 10× throughput.

## Admission control, open vs closed loops, coordinated omission

- **Schroeder, Wierman, Harchol-Balter, NSDI 2006 "Open Versus Closed"** ([PDF](https://www.usenix.org/legacy/event/nsdi06/tech/full_papers/schroeder/schroeder.pdf)): closed-loop generators self-throttle — order-of-magnitude lower response times than open at the same nominal load; scheduling policy matters enormously in open systems, much less in closed. Benchmarking with a closed-loop tool silently measures a different system than production.
- **SEDA (Welsh & Culler, USITS 2003)**: per-stage adaptive admission control targeting a **90th-pct response time** SLO; token-bucket per stage; degrade before drop.
- **Coordinated omission (Gil Tene)**: a blocked closed-loop tester fails to issue the requests that should have been sent — stalls recorded once instead of N times. **wrk2** ([github](https://github.com/giltene/wrk2)): constant-throughput generation, latency measured **from scheduled send time**, HdrHistogram. **Hyperfoil** ([hyperfoil.io](https://hyperfoil.io/blog/news/2020-12-9-compensation/)): true open-model async driver.

# Topic B — High-throughput server architecture in Rust

## TechEmpower plateau numbers (project archived March 2026; R23 = final round, Feb 2025; Xeon Platinum 8375C 32c/64t)

- **Plaintext ceiling ~30M req/s in R23**, attributed to **wrk + large request headers**, not servers ([archival post-mortem](https://dev.to/kaliumhexacyanoferrat/techempower-framework-benchmarks-are-now-archived-whats-next-3l0a)). ASP.NET Core: **27.5M plaintext / 2.55M JSON** req/s (Fastify 1.18M/845k; Express 280k). Older Citrine (10GbE): **~7M cap**, diagnosed as network/load-generator bound ([TFB #3538](https://github.com/TechEmpower/FrameworkBenchmarks/issues/3538)). Top Rust entries R23: **ntex, may-minihttp, xitca-web** (all mimalloc); actix slightly behind, axum further back ([axum #1177](https://github.com/tokio-rs/axum/issues/1177)).
- **Why plaintext ≈10× JSON**: verified in TFB toolset — plaintext runs wrk with **HTTP/1.1 pipelining depth 16** (pipeline.lua) at 256–16,384 connections; JSON is one request per round trip. Pipelining amortizes one read()/write() pair over 16 requests — **syscalls/kernel networking are the limit, not parsing or routing**. Standard tricks: cached Date header (actix refreshes ~500ms), fixed Server/status byte slices, writev batching of pipelined responses.

## tokio (work-stealing) vs thread-per-core (monoio, glommio)

- **monoio benchmarks** ([benchmark.md](https://github.com/bytedance/monoio/blob/master/docs/en/benchmark.md), Xeon Gold 5118 + X710 10GbE): ≈tokio at 1 core; **~2× at 4 cores, ~3× at 16 cores**; glommio same shape, lower peak. Honest caveat from their own docs: at 1 core/few connections monoio has HIGHER latency and LOWER throughput than tokio (io_uring batching vs epoll wakeup). Tokio degrades with cores on echo-style loads: cross-core migration, cache-coherency, shared run-queue atomics.
- **Enberg, Rao, Tarkoma, ANCS 2019** ([PDF](https://penberg.org/papers/tpc-ancs19.pdf)): partitioned KV store with request steering, **p99 −71% vs Memcached**; steering costs are the limit.
- **ScyllaDB/Seastar**: shard-per-core production flagship; >19K reads/s/shard at p99=2ms ([shard-per-core](https://www.scylladb.com/product/technology/shard-per-core-architecture/)). Counterpoint: [without.boats thread-per-core](https://without.boats/blog/thread-per-core/) — work-stealing wins when load is NOT perfectly shardable (TPC suffers intra-shard HoL blocking).
- **SO_REUSEPORT** (Linux ≥3.9): per-worker listen sockets, kernel hashes connections. NGINX 1.9.1: **2–3× req/s on 36 cores** ([F5 blog](https://www.f5.com/company/blog/nginx/socket-sharding-nginx-release-1-9-1)); Cloudflare **+33% p99** ([blog](https://blog.cloudflare.com/the-sad-state-of-linux-socket-balancing/)). Caveat: balances connections not load; worker death drops connections.

## Lock-free read paths and the cost of Arc

- **Travis Downs, "A Concurrency Cost Hierarchy"** ([blog](https://travisdowns.github.io/blog/2020/07/06/concurrency-costs.html)): non-atomic op ~2ns; **uncontended atomic increment ~10ns; contended atomic 40–400ns** (cache-line bounce ~70 cycles min); syscall ~1µs; context switch ~10µs. Quantitative basis for: Arc::clone per request is ~free single-threaded but a 40–400ns serialization point when many cores hammer one refcount line → **sharded/per-core counters ~50× faster** despite 3× instructions. (= Bohm's zero-bookkeeping-reads lesson at the hardware level.)
- **arc-swap** ([perf notes](https://docs.rs/arc-swap/latest/arc_swap/docs/performance/index.html)): load() lock-free/usually wait-free, contention-free between readers (debt/hazard mechanism avoids the shared refcount); reads flat as readers grow while RwLock collapses; writes more expensive than uncontended mutex. No absolute published numbers by design; third-party ~100M loads/s aggregate indicative only.
- **left-right / evmap** (Gjengset): two copies, readers epoch-counters only, writer op-log + swap. Reads scale linearly without writes; dominates RwLock<HashMap>/chashmap on read side at 40 cores. Trade-offs: eventual consistency (readers see updates at publish()), 2× memory. **crossbeam-epoch** = the RCU-style reclamation underneath.

## HTTP/1.1 pipelining vs HTTP/2 multiplexing

- TFB: 16-deep H1 pipelining is what makes 27–30M req/s possible; same servers ~2.5M unpipelined JSON.
- **Vespa HTTP/2 study** ([blog](https://blog.vespa.ai/http2/)): one connection, 256 streams: **115k req/s vs 6.5k for serial H1**; best (4 conn × 256 streams) 225k, ~2× H1's best (128 connections). H2 beats *serial* H1 per connection; vs *pipelined* H1 the 9-byte DATA frame header + per-stream state + flow accounting cost raw throughput. H2 removes HTTP-layer HoL but keeps TCP-level (→ HTTP/3/QUIC).

# Topic C — Streaming result protocols

## Transport mechanics

- **Chunked (RFC 9112 §7.1)**: NO app-level flow control; backpressure is pure TCP (client stops reading → window closes → server write() blocks → propagates up into the result iterator). Consequences: a slow client holds query resources (snapshot, iterators, memory) open for the duration → pair streaming with execution/idle timeouts; status+headers committed BEFORE the body → a mid-stream failure cannot change the 200.
- **SSE (text/event-stream)**: data:/event:/id:/retry: fields over a long-lived chunked response. Built-in resumability: auto-reconnect with Last-Event-ID replay. Backpressure: same TCP-only story; EventSource cannot pause.
- **WebSocket (RFC 6455)**: NO flow control above TCP — no credit frames. Senders self-police via bufferedAmount or socket signals; fast-producer/slow-consumer OOMs the sender queue or stalls on TCP. WebSocketStream (Chrome) retrofits backpressure via WHATWG Streams. Per-message-deflate compounds memory.
- **HTTP/2 (RFC 9113 §5.2)**: explicit credit-based flow control on DATA frames, per-stream AND per-connection, initial window **65,535 B** (raisable to 2³¹−1 via SETTINGS), replenished by WINDOW_UPDATE. True per-result-stream backpressure when multiplexing several query streams on one connection; too-small window caps throughput at window/RTT (classic gRPC bug).

## SPARQL protocol + streaming the JSON results format

- **SPARQL 1.1 Protocol**: SELECT/ASK return "exactly one SPARQL Results Document"; nothing forbids chunked transfer — but the protocol has **no notion of partial/failed results** (errors are 4xx/5xx, impossible after a streamed 200 begins). Acknowledged gap: [w3c/sparql-dev#51](https://github.com/w3c/sparql-dev/issues/51), no standard mechanism.
- **Incremental SPARQL-JSON: yes.** `{"head":{"vars":[...]},"results":{"bindings":[...]}}` — head needs only the projection (known pre-evaluation); bindings emitted one at a time; close `]}}`. Wrinkle: nothing AFTER the array can carry an error/metadata trailer (CSV/TSV likewise) — true mid-stream error signaling needs HTTP trailers or a non-standard extension.

## Existing implementations

- **Oxigraph** (verified in source): lazy iterator evaluation; `cli/src/main.rs` `ReadForWrite` adapter (~lines 1873–1932, 4 query-response call sites) wraps incremental `sparesults` serializer state in `impl Read` — the HTTP layer pulls bindings on demand, chunked, never materialized. `sparesults` exposes streaming `WriterSolutionsSerializer` standalone.
- **Jena Fuseki**: ARQ "main query engine is a streaming engine"; streaming CONSTRUCT via JENA-329. Caveat: sort/group/aggregate are pipeline breakers; some pretty outputs buffer.
- **Virtuoso "Anytime Queries"** ([docs](https://docs.openlinksw.com/virtuoso/anytimequeries/)): partial evaluation under a deadline (&timeout= ms); on timeout returns **partial results with HTTP 200**, incompleteness flagged ONLY via X-SQL-State/X-SQL-Message headers. DBpedia runs this in production; clients not inspecting headers silently treat truncated results as complete — the streaming-error problem made policy.
- **QLever**: lazy operator evaluation (2024–25), "transmits the result to the client in small chunks"; streaming export of large result sets; cache handles lazy results (`cache-max` for lazy root operations). Historic Wikidata-endpoint OOMs ([#2174](https://github.com/ad-freiburg/qlever/issues/2174)) part of the motivation.

## Honest gaps

- Per-framework TFB R23 req/s for may-minihttp/actix/axum not extractable (JS results site; project archived). Verified anchors: ~30M plaintext cap, ASP.NET 27.5M/2.55M, ~7M on 10GbE Citrine, top Rust = ntex/may-minihttp/xitca-web.
- monoio publishes relative (2×/3×) not absolute QPS in the English doc.
- arc-swap/evmap publish no absolute ns by design; Travis Downs' 10ns/40–400ns are the best-grounded atomic costs.
- No published quantitative comparison of Umbra's scheduler vs tokio-style work-stealing; closest is their FIFO/fair/MonetDB/PostgreSQL comparison.
