<!-- [OPUS-4.8] Deep-research design-for-review record: data management, ops &
     production-readiness gaps for sparq. RESEARCH ONLY — no crates/ changes proposed
     as code; every item is a future bead for the maintainer to approve. NON-CANONICAL
     timing (this session runs on an EC2 work box); no measured numbers are baked here. -->

# Data management, ops & production-readiness — gap research

> Status: **research / design-for-review** (no implementation in this wave). Model: Opus 4.8
> (Fable unavailable — flag for re-review when Fable returns). `[OPUS-4.8]`
>
> Lens: operational / production features for sparq used **as a deployed engine** (the
> `sparq-server`/`sparq-serve` surface and the GUI's embedded engine), not only as a library.
> This complements — and does **not** duplicate — `research/production-certification-plan.md`
> (which is about *security certification*: SBOM/SLSA/CRA/memory-safety, not the operational
> data-management features below).

## 0. Honesty preface — what sparq ALREADY has (verified against the code, 2026-06-19)

The brief's prompts ("ACID transactions … some time-travel exists — verify … Prometheus exists
— gaps? … structured audit — exists — gaps?") are **mostly already satisfied**. Before any
proposal, the verified inventory, so nothing below re-proposes an existing control:

- **ACID / MVCC transactions** — `crates/sparq-engine/src/txn.rs`, opt-in `txn` cargo-feature
  (confirmed in `crates/sparq-engine/Cargo.toml`, line 115). **Snapshot isolation** with
  **first-committer-wins** optimistic write-write conflict detection (`TransactionManager`,
  `ReadTxn`, `WriteTxn`, `CommitError::Conflict`). Built on the engine's copy-on-write
  delta-overlay; `Graph::fork` makes snapshots `O(pending delta)`, not `O(triples)`.
  → **Implemented-and-verified.** (Gap: SI only — no serializable isolation; see Candidate 8.)
- **Durability (WAL)** — directory-backed `Graph` carries a per-graph write-ahead log
  (`crates/sparq-core/src/lib.rs`: `wal: Option<Wal>`; every `apply_delta` is appended + fsync'd
  *before* apply and replayed on `open`). A **journal frame** (sq-ycle) makes an atomic multi-op
  UPDATE crash-safe across the per-graph WALs. WAL **compaction/vacuum** is a CLI command
  (`sparq-cli compact`, sq-x32t) that physically purges erased/superseded bytes.
  → **Implemented-and-verified.** (Gap: this is on-box durability for a *persisted* graph; the
  in-memory `sparq-serve` writer documents `D = out of scope for v1` — see Candidate 1.)
- **Named-graph time-travel / versioning** — `crates/sparq-serve/src/ring.rs`: opt-in
  `TimeTravelConfig` retains generations beyond the concurrency bound so `GenerationRing::at` /
  `as_of` serve historical snapshots; memory-cost contract documented (`M × full graph`).
  → **Implemented-and-verified, but coarse** (full-graph generations, RAM-bound, no on-disk
  archive, no triple-level delta/diff query — see Candidate 5).
- **Observability** — `crates/sparq-server/src/metrics.rs` emits Prometheus text exposition
  (`sparq_http_requests_total`, `sparq_query_duration_seconds` histogram, `sparq_updates_total`,
  gauges). → **Implemented.** (Gap: no distributed **tracing** / OpenTelemetry spans, no
  per-query cost/cardinality metrics — see Candidate 7.)
- **Structured audit** — TWO tiers: `audit.rs` (`audit-log` feature — one flat `tracing` event
  per request) and `access_audit.rs` (`AuditSink` trait — typed actor/action/resource/decision
  records). → **Implemented-and-verified.** Genuinely covered; **not** re-proposed.
- **Change feed (in-process only)** — `crates/sparq-server/src/subscriptions.rs`: SEPA-style
  WebSocket SELECT subscriptions, re-evaluate-and-diff per committed generation. → **Implemented
  but in-process / ephemeral** — no durable, replayable, externally-consumable change stream
  (see Candidate 2).
- **DoS / admission limits** — `http.rs` `AppState` has `query_timeout`, `update_where_timeout`,
  `max_body_bytes` (413), `concurrency_limit` (429 load-shed), `header_read_timeout`,
  body-read timeout. → **Implemented.** (Gap: no *per-client/per-tenant* rate limiting or
  quotas — these are global — see Candidate 4.)
- **Horizontal scaling / replication** — `research/adr-horizontal-scaling.md` designs
  pod-sharded shards + read replicas off the deterministic single-writer log, **PROPOSED, not
  implemented**. The log it leans on (group-commit `Writer`) exists in `sparq-serve/src/writer.rs`
  but has **no serialized on-disk/wire form yet** (stage 2 of that ADR). → **Designed-only.**
- **Lineage** — `crates/sparq-prov` (PROV-O for CONSTRUCT/UPDATE/reasoner derivation),
  opt-in crate. → **Implemented** for *derivation* lineage. (Gap: no *ingest/dataset-level*
  lineage — which file/version/checksum produced which graph — see Candidate 9.)
- **Crypto-erase / at-rest encryption** — `research/crypto-erase-at-rest.md`, **designed-only**
  (opt-in `sparq-crypto-erase` recommended, not built). Per-tenant key-wrapping is sketched there.
- **AuthZ** — `sparq-solid` (WAC/ACP); `sparq-server` itself is documented **no-auth** beyond an
  optional Bearer gate on updates (threat-model boundary **B3**: "front with a gateway /
  sparq-solid"). → No OIDC/OAuth2/API-key/RBAC *in the server* (see Candidate 3).

**Correction to the brief's premise:** the brief lists transactions/isolation, time-travel,
Prometheus, and audit as things to "verify" — all four **exist and are verified above**, so the
real gaps are narrower and more specific than the brief implies. The genuine production holes are:
(1) **durable/recoverable state for the *serving* path** (online backup, PITR, snapshot
export/import — the `sparq-serve` writer is in-memory-only today); (2) a **durable, replayable,
externally-consumable change feed** (the Neptune-Streams shape) — the in-process WebSocket diff is
not that; (3) **first-party endpoint authn/authz** (OIDC/API-key/RBAC/graph-level) for operators
who can't put sparq behind Solid or a gateway; (4) **multi-tenancy quotas + per-principal rate
limiting**; plus finer items on time-travel archiving, tracing, isolation, and ingest lineage.

---

## 1. Problem framing

sparq's operational story is bifurcated:

- **The persisted library path** (`Graph::open` + WAL + `txn`) is genuinely production-grade for
  *durability and isolation on one box*: fsync'd WAL, crash replay, MVCC-SI transactions, offline
  vacuum.
- **The serving path** (`sparq-serve` generation ring + group-commit writer + `sparq-server`)
  deliberately chose an **in-memory, non-durable** model (writer.rs §6.5: `D = out of scope`),
  betting on the Solid-pod profile where state is reconstructable from the upstream resource store.

That bet is fine for the original seat, but it means an operator who wants to run sparq as a
*primary, authoritative* store (the GUI's persistent workspace; a standalone SPARQL endpoint over
a curated dataset; a knowledge-graph backend) has **no first-party answers** to the questions every
production datastore must answer: *How do I back this up without stopping it? How do I recover to a
point in time? How do I stream changes to a downstream system? Who is allowed to query what? How do
I stop one tenant starving another?* Competitors answer all of these:

| Capability | sparq today | GraphDB | Stardog | Neptune |
|---|---|---|---|---|
| MVCC/SI txns | **yes** (`txn`) | yes | yes | yes |
| On-box WAL durability | **yes** (persisted graph) | yes | yes | yes (managed) |
| Online backup / restore | **no** (CLI `save` is offline) | yes | yes (`server backup`) | yes (snapshots) |
| Point-in-time recovery | **no** | partial | partial | **yes** (PITR) |
| Durable change feed / CDC | **no** (in-proc WS only) | partial (SPARQL MONITOR) | yes (events) | **yes** (Streams) |
| Read replicas / HA | **designed-only** | yes (cluster) | yes (cluster) | yes (≤15 replicas) |
| Named-graph versioning | **coarse, RAM-only** | add-on | **yes** (git-like VCS) | no |
| OIDC/OAuth2 + RBAC | **no** (operator's job, B3) | **yes** | yes | IAM |
| Graph/triple-level security | **via Solid only** | yes (fine-grained) | yes | no |
| Per-tenant quota / rate limit | **no** (global limits only) | partial | partial | IAM/throttle |

The lean-core rule is the design constraint throughout: **every item below must be an opt-in crate
or cargo-feature** so `sparq-core`/`sparq-engine`/the wasm artifact stay lean and pay zero when the
operator doesn't deploy as a server. That is achievable for all of these — they all live at the
`sparq-serve`/`sparq-server` tier or in a new sibling crate, never in core.

---

## 2. Candidate features (with honest trade-offs)

Each candidate states: what it is, the concrete use-case, prior art (with refs), fit with the
maintainer's directions (Solid/decentralized, federation, ZK/MPC privacy, GenAI/vector, GUI
embedded-app), the opt-in shape, the honest cost/risk, and the specific decision-question.

### Candidate 1 — Online snapshot/backup + restore for the serving path (`sparq-serve`)

**What.** Make the in-memory `sparq-serve` store **export a consistent generation to disk while
serving** (no stop-the-world) and **restore from it on start**, plus an admin endpoint to trigger
it. The generation ring already publishes immutable snapshots; a backup is "pin generation N, write
its triples + dictionary + per-pod epochs to a single durable artifact." Restore re-hydrates the
ring's gen-0 from that artifact.

**Why / use-case.** The GUI's "persistent workspace" and any standalone-endpoint operator need
*online* backup. Today the only persistence is `sparq-cli save` (offline, rebuilds indexes) — you
cannot back up a *running* server without quiescing it. Restore-from-backup is also the bootstrap
primitive the horizontal-scaling ADR (stage 2) explicitly needs ("snapshot = a shipped generation;
node hydrates from the latest S3 snapshot + log tail") — so this unblocks that roadmap.

**Prior art.** Stardog `server backup`/`server restore` (backup IDs, S3 support, includes
permissions/named-graph metadata) — docs.stardog.com/operating-stardog. Neptune managed snapshots +
restore. The mechanism (consistent online snapshot of an MVCC store) is textbook PostgreSQL
`pg_basebackup` / RocksDB checkpoint.

**Fit.** Directly serves the **GUI embedded-app** (workspace save/load that survives a crash) and
is the prerequisite half of the **horizontal-scaling** stage-2 log/snapshot plane. Composes with
the existing on-disk compressed format (`sparq-core` save) as the artifact encoding.

**Opt-in.** Yes — a `sparq-serve` feature (`backup`) + a gated `sparq-server` admin route
(`POST /admin/backup`, `POST /admin/restore`). Off by default; zero cost when off.

**Effort.** **M.** The snapshot *content* (triples + dict) already round-trips via the core save
format; the new work is (a) serialize a generation's per-pod epoch vector + writer sequence number
alongside it, (b) the online-pin-while-serving flow (trivial given immutable generations), (c) the
admin endpoints + auth gate. Restore is "load gen-0 from artifact + set sequence."

**Decision-question.** Should online backup target a **single self-describing artifact** (simplest;
matches Stardog's backup-ID model) or **directly reuse the on-disk WAL-backed directory format**
(so a backup is also openable by `sparq-cli query-mmap`)? The latter is more powerful but couples
the serving snapshot to the persisted-graph on-disk layout.

---

### Candidate 2 — Durable, replayable change-data-capture stream (the Neptune-Streams shape)

**What.** A **durable, ordered, externally-consumable** change feed: every committed generation
emits a record `(seq, per-graph quad deltas: inserts/deletes, epoch bumps, commit timestamp)` to a
durable log that downstream consumers poll by sequence number (resume from any seq, exactly-once,
no gaps). This is the **serialized on-disk/wire form of the group-commit log** that
`writer.rs` already produces in memory and that the horizontal-scaling ADR's stage 2 calls for but
hasn't built.

**Why / use-case.** Three concrete consumers: (1) **replication** — a read replica tails the stream
and replays (the ADR's stage 3 HA story); (2) **downstream sync** — keep a full-text/vector index,
an OpenSearch mirror, or a materialized view in step with the graph (Neptune's primary advertised
use case); (3) **triggers/webhooks** — notify on changes matching a pattern. Today the *only* change
notification is the in-process WebSocket diff (`subscriptions.rs`): it is ephemeral (a consumer that
disconnects misses everything), in-process (no cross-service consumer), and per-query (re-evaluates
SELECTs, not a raw change log).

**Prior art.** **Amazon Neptune Streams** (docs.aws.amazon.com/neptune/.../streams.html): change
records written *synchronously within the committing transaction*, strict sequential order, no
duplicates, no gaps, complete-state-reconstructable, polled via REST `GetRecords`, 1–90 day
retention. **Debezium/Kafka-Connect** CDC patterns generally. The contract is the load-bearing
part: order + at-least-once-with-dedup-key + resumable offset.

**Fit.** This is the **keystone of the horizontal-scaling roadmap** (stages 2–3) — replication is
"tail this stream and replay." It also feeds the **federation** story (a downstream sparq node as a
materialized cache) and the **GenAI/vector** story (keep the embedding index live). Strong fit.

**Opt-in.** Yes — `sparq-serve` feature (`change-stream`) writing to a local append-only segment
file; a `sparq-server` feature exposing `GET /streams?after=<seq>`. Off by default.

**Effort.** **L.** The in-memory record already exists (the group-commit batch carries the quad
deltas + epoch bumps); the work is (a) a durable, segmented, fsync'd append log with a resumable
offset and retention/purge, (b) the per-graph quad-delta serialization format (stable, versioned),
(c) the read API + an `after=` cursor, (d) the **honesty boundary**: state plainly it is
at-least-once-with-dedup-key, single-writer-ordered, *not* a distributed log (no Kafka), and the
in-memory writer's `D = out of scope` caveat is *lifted* only for graphs that opt into the stream's
own durability. Pairs naturally with Candidate 1 (snapshot + log = full PITR — Candidate 6).

**Decision-question.** Is the right v1 **a local segmented log polled over HTTP** (self-contained,
matches Neptune's REST model, no broker dependency — my recommendation), or should it **emit to an
external broker** (Kafka/NATS) behind a sink trait (more "enterprise" but adds a heavy optional dep
and an integration-test surface)? Recommend local-first with a `ChangeSink` trait so a broker sink
is a later, separate opt-in.

---

### Candidate 3 — First-party endpoint authn/authz: OIDC/OAuth2 + API keys + RBAC (`sparq-authz`)

**What.** An **opt-in** authentication+authorization layer for `sparq-server` that is *not* Solid:
(a) **bearer/JWT verification** against an OIDC issuer (JWKS, `aud`/`iss`/`exp` checks); (b) static
**API keys** (hashed at rest); (c) a small **RBAC** model (roles → permissions: query / update /
graph-store-write / admin) mapped from a JWT claim or an API-key→role table; (d) optional
**named-graph-scoped** authorization (a role may read/write only an allow-listed set of graph IRIs).

**Why / use-case.** Boundary **B3** today says "front sparq with a gateway or sparq-solid." That is
the honest answer for the Solid seat, but it makes sparq a **non-starter as a standalone endpoint**
for operators who (i) don't run Solid, (ii) can't put a full API gateway in front (the GUI app, an
internal team endpoint, an edge deployment), or (iii) want *graph-level* security without WAC/ACP's
machinery. GraphDB and Stardog both ship this in-engine (OIDC, LDAP, Kerberos, RBAC roles,
fine-grained named-graph/statement security — GraphDB access-control docs). It is the single most
common "why can't I use this in prod" question for a SPARQL endpoint.

**Fit.** Complements (does not replace) `sparq-solid`: Solid stays the decentralized/WebID path;
`sparq-authz` is the **centralized-operator** path (classic OIDC tenant). Fits the GUI (per-user
workspace auth) and federation (a node that authenticates SERVICE callers). It must be designed to
*compose* with the privacy estate, not to be confused with it — RBAC is access control, **not** a
cryptographic privacy guarantee (the ZK/MPC honesty boundary stays intact: this is plaintext
authz, and the doc must say so).

**Opt-in.** Yes — a new `sparq-authz` crate + a `sparq-server` feature (`authz`). Off by default;
the no-auth B3 behaviour is unchanged when the feature is off.

**Effort.** **L** (OIDC/JWKS verification + key rotation + the graph-scoped enforcement hook into
the query/update path is real surface; the access-decision plumbing already exists via
`access_audit.rs`'s actor/decision model, which this would *produce* the decisions for).

**Decision-question.** Two sub-questions: (1) **Scope** — is graph-level (named-graph allow-list per
role) the right granularity ceiling for v1, explicitly **deferring** triple/pattern-level security
to "use sparq-solid or a future ABAC"? (2) **Boundary** — does adding first-party authz to
`sparq-server` change the threat-model B3 stance (from "always front it" to "optionally
self-protected"), and is that a posture the maintainer wants to own (it shifts auth bugs into
sparq's blast radius)?

---

### Candidate 4 — Multi-tenancy quotas + per-principal rate limiting (`sparq-serve` admission)

**What.** Extend the existing global admission limits (`concurrency_limit`/429, `query_timeout`,
`max_body_bytes`) to **per-principal / per-tenant** budgets: a token-bucket rate limit keyed on
(API key | WebID | source IP), a **per-tenant concurrency cap**, a **per-tenant result-byte / CPU-ms
budget**, and a **per-graph triple-count quota** (reject an UPDATE that would push a tenant's graph
over a configured ceiling).

**Why / use-case.** The horizontal-scaling and concurrent-serving research both target a
**high-tenancy** profile (one named graph per Solid resource, many independent owners). Today a
single abusive or runaway tenant can consume the whole global concurrency budget and starve every
other tenant — there is no isolation *within* one node. Quotas are what make a shared endpoint
safe to expose. (GraphDB/Stardog have partial throttling; this is a known production need across
all multi-tenant datastores.)

**Fit.** Directly serves the Solid-pod multi-tenant seat (pod = tenant) and the
horizontal-scaling design (per-shard fairness). Composes with Candidate 3 (the principal identity a
quota is keyed on comes from authz) — so these two are a natural pair, but each is independently
useful (quotas can key on source IP without authz).

**Opt-in.** Yes — a `sparq-serve` feature (`quota`) + config table; default disabled = today's
global-only behaviour.

**Effort.** **M.** Token-bucket + per-key counters are well-trodden; the subtle work is **cost
attribution** (charging CPU-ms / result-bytes accurately enough to be fair — the
`concurrent-serving.md` cost model gives the per-class estimates to seed this) and the
**per-graph triple-count quota** enforcement point in the writer.

**Decision-question.** What is the canonical **tenant key** — the pod IRI (Solid seat), the authz
principal (Candidate 3), or an opaque operator-supplied tenant header — and should v1 support just
*one* of these or a pluggable key extractor? Recommend a pluggable `TenantKey` extractor with the
pod-IRI default, so it works with and without Solid/authz.

---

### Candidate 5 — On-disk, triple-level named-graph archive + version/diff queries (`sparq-archive`)

**What.** Replace/augment the **RAM-bound, full-graph-per-generation** time-travel
(`TimeTravelConfig`) with an **on-disk, delta-compressed archive** that stores the change history
as snapshot + delta chains and answers the three RDF-archive query types — **version materialization**
(graph as-of version V), **delta materialization** (what changed between V1 and V2), and
**version queries** (when did triple T hold) — without pinning every historical generation in
memory.

**Why / use-case.** The current time-travel doc is explicit that `max_generations: 16` can pin
**~12 GB** for a 1M-triple graph — it does not scale past a handful of versions and dies on restart
(in-memory only). Real versioning use-cases (regulatory "what did the record say on date D",
dataset-evolution analytics, reproducible GenAI grounding "answer as-of the snapshot the model was
trained on") need *many* versions, *durably*, with *diff* queries — exactly what the RDF-archiving
literature solves.

**Prior art.** **OSTRICH** (Taelman et al., ISWC 2019 / Semantic Web Journal — "Triple storage for
random-access versioned querying of RDF archives"): snapshot + delta chain, offset-enabled, answers
VM/DM/VQ triple-pattern queries; **COBRA** (Taelman et al., SWJ 2022) — bidirectional delta chains.
**GLENDA** (ISWC 2023) — full SPARQL over RDF archives. **Stardog VCS** — git-like single-history
named-graph versioning with diff/tag/SPARQL-over-history (community.stardog.com). These give a
proven storage model and a proven query taxonomy.

**Fit.** Builds on the existing generation ring (each generation is already an immutable version);
the archive is "persist the deltas the writer already computes instead of pinning whole snapshots."
Strong fit with **GenAI** reproducibility and **Solid** (per-resource history). Pairs with the CDC
stream (Candidate 2) — the change stream *is* the delta source the archive ingests.

**Opt-in.** Yes — a new `sparq-archive` crate (the OSTRICH/COBRA storage) + a `sparq-serve` feature
wiring it to the writer. Off by default.

**Effort.** **XL.** This is a real storage-engine subproject (delta-chain encoding, offset indexes,
the VM/DM/VQ query algorithms, then optionally GLENDA-style full-SPARQL-over-archive). Phaseable:
phase 1 = durable delta-chain + VM (version-materialization), which already beats today's RAM model;
later phases add DM/VQ and SPARQL-over-history.

**Decision-question.** Is durable, scalable versioning **worth an XL storage subproject**, or is the
near-term need met by the *cheaper* combination of Candidate 1 (periodic snapshots) + Candidate 2
(change stream) giving "restore to a coarse checkpoint + replay deltas" without random-access
version queries? Recommend: do Candidates 1+2 first; greenlight `sparq-archive` only if random-access
version/diff *queries* (not just recovery) are a confirmed requirement.

---

### Candidate 6 — Point-in-time recovery (PITR) = Candidate 1 ⊕ Candidate 2 (a thin composition)

**What.** Not a new subsystem — a **runbook + thin tooling** that composes a base **snapshot**
(Candidate 1) with the **durable change stream** (Candidate 2) to recover the serving store to *any*
sequence number / timestamp: `restore(snapshot_N) ; replay(stream, from=N, to=target_seq)`. Add a
`sparq-cli pitr` command and the seq↔timestamp index needed to translate a wall-clock target to a
sequence number.

**Why / use-case.** PITR ("recover to 14:32 yesterday, just before the bad bulk load") is the single
most-asked-for operational guarantee that sparq cannot offer today. It is the canonical reason
Neptune/Postgres ship snapshot+WAL-archive. Because the pieces are Candidates 1 and 2, PITR is
*almost free* once those land — which is itself an argument for prioritizing 1 and 2.

**Prior art.** PostgreSQL base-backup + WAL archiving + `recovery_target_time`; Neptune PITR;
every serious RDBMS. The composition (snapshot + ordered log replay to a target) is standard.

**Fit.** Pure composition of two other candidates; no new architecture. Strong ops value, low
incremental cost. Fits the GUI (workspace undo-to-a-point) and any authoritative-store deployment.

**Opt-in.** Yes — inherits the opt-in features of Candidates 1+2; the CLI command is additive.

**Effort.** **S** *given* Candidates 1 and 2 (it is glue + a seq↔time index + a runbook). **Do not
scope this before 1 and 2.**

**Decision-question.** Should the recovery target be expressible as a **wall-clock timestamp**
(needs a durable seq↔timestamp index and an honest note that clock skew bounds precision) or
**sequence-number-only** (exact, simpler, but operator-unfriendly)? Recommend both, with the
timestamp path documented as "nearest commit at-or-before T."

---

### Candidate 7 — Distributed tracing (OpenTelemetry) + per-query cost metrics

**What.** Add **opt-in OpenTelemetry spans** to `sparq-server`/`sparq-serve`: a server span per
request that **honours an inbound W3C `traceparent`** and emits child spans for parse / plan /
execute / serialize, with DB semantic-convention attributes (operation class, result size, plan
cost estimate). Plus a handful of **new Prometheus metrics** the current exposition lacks: queue
depth per scheduler lane, group-commit batch size, cache hit/miss ratio, per-class query duration,
rejected-by-limit counters (429/413/timeout).

**Why / use-case.** Prometheus *counters/histograms* exist, but in any real deployment sparq sits
behind a gateway / inside a federation, and operators need **end-to-end traces** (where did the 2 s
go — parse, plan, a slow SERVICE leg, serialization?) correlated across services. W3C Trace Context
is the standard for this; without it sparq is a black box in a traced stack. The extra metrics
surface the scheduler/writer internals the concurrent-serving design already maintains but doesn't
export.

**Prior art.** **W3C Trace Context** (`traceparent`/`tracestate` Recommendation);
**OpenTelemetry semantic conventions for database client/server spans**; the Rust `tracing` +
`tracing-opentelemetry` ecosystem (sparq already uses `tracing` for audit, so the integration point
exists).

**Fit.** Pure ops/observability; composes with the existing `tracing`-based audit (same backbone).
Especially valuable for **federation** (propagate `traceparent` to `SERVICE` legs to trace a
distributed query) — a genuinely differentiated capability for a federated SPARQL engine.

**Opt-in.** Yes — a `sparq-server` feature (`otel`) pulling the otel deps only when enabled; the new
Prometheus metrics are cheap atomics, behind the existing metrics surface. Off by default.

**Effort.** **M.** Span instrumentation of the request path + `traceparent` extraction/injection is
mechanical given `tracing`; propagating context into the federation `SERVICE` HTTP calls is the
interesting part (and the highest-value one). New metrics are small.

**Decision-question.** Should the **federation client propagate `traceparent` to SERVICE endpoints
by default** when otel is on (best observability, but it leaks that sparq is the caller and adds a
header to outbound requests — a minor privacy/footprint consideration worth a conscious choice)?

---

### Candidate 8 — Serializable isolation option for the `txn` manager (close the SI gap)

**What.** Offer an **opt-in serializable** isolation level on top of today's snapshot-isolation
`TransactionManager`, via either (a) **SSI** (serializable snapshot isolation — track rw-antidependencies,
abort on dangerous structures; PostgreSQL's approach) or (b) the simpler **promotion to
serial commit** for write txns (the single-writer group-commit path already serializes writers, so
write-write is fine; the gap is read-skew / write-skew anomalies SI permits).

**Why / use-case.** SI is correct for the overwhelming majority of SPARQL workloads, but SI **admits
write-skew** — two concurrent txns each reading a constraint and each writing so the constraint is
jointly violated. For an engine that also runs **SHACL validation** as a consistency gate, write-skew
across two validating updates can commit a state neither update alone would (e.g., two updates that
each keep a cardinality constraint satisfied in isolation but violate it together). An opt-in
serializable level closes that for the rare workloads that need it.

**Prior art.** Cahill/Röhm/Fekete SSI (SIGMOD 2008 → PostgreSQL SERIALIZABLE); the project's own
`research/concurrent-serving-litreview-A-mvcc-benchmarks.md` already reviews the MVCC design space.

**Fit.** Narrow but real; opt-in on an already-opt-in feature. Composes with the SHACL story
(serializable + constraint-checking = the "ACID + integrity" deployment). Honest framing: this is a
*correctness ceiling raise for rare workloads*, not a headline feature — and it must not be oversold
(SI is the right default; serializable costs aborts/throughput).

**Opt-in.** Yes — a sub-mode of the existing `txn` feature (an `IsolationLevel` parameter); SI stays
the default.

**Effort.** **L** for SSI (anti-dependency tracking is genuinely tricky to get sound); **M** if the
v1 is the conservative "serialize write txns through the single writer + validate" path.

**Decision-question.** Is serializable isolation a **real, requested need** (does any target
deployment hit write-skew), or is SI + "run integrity checks in one txn" sufficient — i.e., should
this be **deferred** until a concrete write-skew case appears? Recommend deferring unless the
SHACL-as-integrity-gate deployment is on the near roadmap.

---

### Candidate 9 — Dataset/ingest lineage + schema registry (`sparq-prov` extension + catalog)

**What.** Two related but separable items:
(9a) **Ingest lineage** — extend `sparq-prov` to record, at *load* time, which **source artifact**
(file path/URL, checksum, format, byte size, parse timestamp, sparq version) produced which named
graph — the `prov:wasDerivedFrom` edge for *ingestion*, not just for CONSTRUCT/UPDATE derivation.
(9b) **A lightweight shapes/ontology registry** — a queryable catalog of the SHACL shapes /
ontologies governing each graph, with versions, so an operator can answer "what schema validates
this dataset, and which version."

**Why / use-case.** `sparq-prov` today covers *derived* data (CONSTRUCT/UPDATE/reasoner) but **not
ingest** — so a loaded graph has no machine-readable record of *where it came from*. CDMC and every
data-governance regime want dataset-level lineage (the `production-certification-plan.md` cdmc
worktree flags lineage/retention as operator-owned but *recommends* sparq improve it). The shapes
registry turns sparq's strong SHACL surface into a *governance* surface: validation that is
discoverable and versioned, not ad-hoc.

**Prior art.** W3C **PROV-O** (already sparq's lineage vocab); **DCAT** (dataset catalog vocab) and
**DQV** (data quality vocab) for the catalog layer; CDMC's lineage/cataloguing capabilities. The
shapes-registry idea mirrors schema registries in the streaming world (Confluent Schema Registry)
but RDF-native (shapes as RDF, queryable via SPARQL).

**Fit.** Extends an existing opt-in crate (`sparq-prov`) and an existing surface (SHACL). Strong fit
with **GenAI** (citable provenance for grounded answers — already a `prov-lineage` SKILL use-case)
and **governance/compliance**. Low architectural risk.

**Opt-in.** Yes — 9a is a `sparq-prov` addition (already off-by-default crate); 9b can be a thin
crate or a `sparq-introspect` extension (which already mines characteristic sets → SHACL).

**Effort.** **M** (9a is a focused `sparq-prov` addition wiring the loader's source metadata into a
PROV `Activity`; 9b is a small catalog model + SPARQL-queryable storage).

**Decision-question.** Is **ingest lineage (9a)** the higher-value half to do first (it closes the
CDMC gap and feeds GenAI citations), with the **shapes registry (9b)** deferred — or are they one
governance epic? Recommend 9a first as a standalone `sparq-prov` bead; 9b only if governance is a
named priority.

---

## 3. Recommendation

Prioritized, with the **dependency spine** made explicit (so the orchestrator can parallelize):

1. **Candidate 2 (durable change stream)** and **Candidate 1 (online backup/restore)** are the
   **keystones** — they unblock PITR (6), the horizontal-scaling roadmap (stages 2–3), downstream
   sync, and replication, and they convert sparq's serving path from "non-durable by design" to
   "recoverable." **Do these first, in parallel.** (2 = L, 1 = M.)
2. **Candidate 6 (PITR)** falls out almost for free once 1+2 land (**S**) — schedule it immediately
   after.
3. **Candidate 3 (endpoint authz)** + **Candidate 4 (quotas/rate-limit)** are the **"safe to expose
   as a standalone endpoint"** pair — the second-most-common production blocker after durability.
   They can proceed in parallel with the 1/2/6 spine since they touch the request-admission tier,
   not the storage tier. (3 = L, 4 = M.) Note 3 changes the B3 posture — **needs a maintainer
   decision before starting.**
4. **Candidate 7 (OTel tracing + metrics)** is independent, **M**, high ops value, low risk — good
   parallel fill, and uniquely valuable for federation tracing.
5. **Candidate 9a (ingest lineage)** is an independent **M** that closes a concrete CDMC/governance
   gap and feeds GenAI — good parallel fill.
6. **Candidate 5 (on-disk versioned archive)** is **XL** and should be **gated on a confirmed need**
   for random-access *version/diff queries* (vs. recovery, which 1+2+6 already give). **Defer.**
7. **Candidate 8 (serializable isolation)** is **deferred** unless a concrete write-skew workload
   (SHACL-as-integrity-gate) is on the near roadmap.

**Lean-core invariant holds for all:** every candidate is an opt-in crate or `sparq-serve`/`sparq-server`
cargo-feature; `sparq-core`/`sparq-engine`/wasm are untouched and pay zero when the feature is off.

**Honesty guardrails baked into the plan:** (a) the change stream and authz must be documented as
**plaintext/operational** mechanisms — they are *not* the ZK/MPC privacy estate and must never be
described as cryptographic privacy guarantees (the privacy-claims gate applies); (b) no measured
throughput/latency numbers enter any of these docs from this work-box; (c) the in-memory writer's
`D = out of scope` caveat is *narrowed* (not silently erased) — durability becomes available **for
graphs that opt into the stream/backup**, and the default in-memory mode stays non-durable and
labelled so.

---

## 4. Phased plan (each phase = a future bead for the orchestrator)

1. **Bead P1 — `sparq-serve` online snapshot export/import (Candidate 1, M).** Serialize a pinned
   generation (triples + dict + per-pod epochs + writer seq) to a durable artifact while serving;
   restore gen-0 from it; gated `sparq-server` admin routes. Acceptance: round-trip a running
   server's state through backup→restore with identical query results and preserved epoch/seq.
2. **Bead P2 — durable, resumable change stream (Candidate 2, L).** Segmented fsync'd append log of
   per-graph quad deltas + epoch bumps keyed by commit seq; versioned delta format; `ChangeSink`
   trait; `GET /streams?after=<seq>` with exactly-once-dedup + no-gap guarantees; retention/purge.
   Acceptance: a consumer resuming from any seq reconstructs the exact post-commit state; honesty
   doc on the single-writer-ordered, not-a-broker boundary.
3. **Bead P3 — PITR composition + CLI (Candidate 6, S; depends on P1+P2).** `restore(snapshot) ;
   replay(stream, to=target)`; seq↔timestamp index; `sparq-cli pitr --to <seq|time>`; runbook.
   Acceptance: recover a deliberately-corrupted store to the commit immediately before the bad write.
4. **Bead P4 — `sparq-authz` crate: OIDC/JWT + API keys + RBAC (Candidate 3, L).** JWKS-based bearer
   verification, hashed API keys, role→permission model, optional named-graph allow-list per role;
   produces the `access_audit` decision records. **Blocked on a maintainer decision** re: the B3
   posture change. Acceptance: deny/allow matrix tests; both feature states build clean.
5. **Bead P5 — per-principal quotas + rate limiting (Candidate 4, M; composes with P4).** Pluggable
   `TenantKey` extractor (pod-IRI default), token-bucket rate limit, per-tenant concurrency cap,
   per-tenant CPU-ms/result-byte budget, per-graph triple-count quota. Acceptance: one abusive
   tenant cannot starve others; quotas enforced at the writer for triple counts.
6. **Bead P6 — OpenTelemetry tracing + new ops metrics (Candidate 7, M).** W3C `traceparent`
   honouring + parse/plan/exec/serialize spans + DB semantic-convention attributes; propagate
   context to federation `SERVICE` legs; add lane-queue-depth / batch-size / cache-ratio /
   limit-rejection metrics. Acceptance: an end-to-end trace across a federated query renders in a
   collector; new metrics appear in `/metrics`.
7. **Bead P7 — `sparq-prov` ingest lineage (Candidate 9a, M).** Record source artifact
   (path/URL/checksum/format/size/time/version) → named-graph as a PROV `Activity`/`wasDerivedFrom`
   at load time. Acceptance: a loaded graph carries machine-readable, queryable origin lineage.
8. **Bead P8 — DEFERRED decision: `sparq-archive` versioned store (Candidate 5, XL)** + **`sparq-authz`
   triple/pattern-level security** + **serializable isolation (Candidate 8, L)** + **shapes registry
   (Candidate 9b)**. Each gated on a confirmed requirement per the recommendation; not scheduled
   until the maintainer greenlights the specific need.

---

## 5. Open questions that genuinely need the maintainer

1. **Durability posture.** Is sparq's serving path meant to stay **non-durable-by-design** (the Solid
   bet: state reconstructable upstream), with backup/CDC as *opt-in additions* — or should the
   serving path move toward **durable-by-default** for the standalone/GUI seats? This decides whether
   P1/P2 are niche features or a strategic direction.
2. **The B3 boundary (Candidate 3).** Does the maintainer want sparq to **self-protect** (first-party
   OIDC/RBAC in `sparq-server`), accepting that auth bugs then live in sparq's blast radius — or hold
   the "always front it with a gateway/Solid" line and reject endpoint authz outright? This is the
   single most consequential decision in this doc.
3. **CDC transport (Candidate 2).** Local segmented log polled over HTTP (Neptune-style,
   self-contained — my recommendation) vs. an external-broker sink (Kafka/NATS) — which is the v1?
4. **Versioning depth (Candidate 5).** Is the requirement *recovery* (met cheaply by P1+P2+P3) or
   *random-access version/diff queries* (the XL OSTRICH/COBRA archive)? Only the latter justifies P8.
5. **Tenant identity (Candidate 4).** Canonical tenant key — pod IRI, authz principal, or opaque
   header — and is a pluggable extractor acceptable as the v1 answer?
6. **Isolation (Candidate 8).** Is there any near-term deployment that actually hits SI write-skew
   (e.g., SHACL-as-integrity-gate), or is serializable isolation safely deferred?

---

## 6. Sources

- `crates/sparq-engine/src/txn.rs`, `crates/sparq-engine/Cargo.toml` (feature `txn`),
  `crates/sparq-core/src/lib.rs` (`wal`), `crates/sparq-serve/src/{writer,ring,applier}.rs`,
  `crates/sparq-server/src/{metrics,audit,access_audit,http,subscriptions}.rs`,
  `crates/sparq-prov/src/lib.rs` — sparq's verified current state (read 2026-06-19).
- `research/concurrent-serving.md`, `research/adr-horizontal-scaling.md`,
  `research/production-certification-plan.md`, `research/crypto-erase-at-rest.md` — prior sparq design.
- Amazon Neptune Streams — change-feed contract (ordered, no-dup, no-gap, REST poll, retention):
  https://docs.aws.amazon.com/neptune/latest/userguide/streams.html
- Stardog backup/restore + git-like versioning (VCS, diff/tag/SPARQL-over-history):
  https://docs.stardog.com/operating-stardog/database-administration/backup-and-restore ;
  https://docs.stardog.com/ (versioning).
- GraphDB access control (OIDC/Kerberos/X.509/LDAP, RBAC roles, fine-grained named-graph/statement
  security): https://graphdb.ontotext.com/documentation/11.2/access-control.html
- OSTRICH (Taelman et al., ISWC 2019 / Semantic Web Journal) — versioned RDF archive, VM/DM/VQ:
  https://rdfostrich.github.io/article-iswc2019-journal-ostrich/ ; COBRA bidirectional delta chains
  (SWJ 2022) https://rdfostrich.github.io/article-swj2020-cobra/ ; GLENDA (full SPARQL over archives,
  ISWC 2023) https://link.springer.com/chapter/10.1007/978-3-031-43458-7_14
- W3C Trace Context (`traceparent`/`tracestate`) + OpenTelemetry DB semantic conventions — tracing.
- Cahill/Röhm/Fekete, Serializable Snapshot Isolation (SIGMOD 2008) → PostgreSQL SERIALIZABLE.
- W3C PROV-O, DCAT, DQV — lineage/catalog vocabularies (Candidate 9).
