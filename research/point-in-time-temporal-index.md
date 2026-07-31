# Point-in-time SPARQL over a persistent temporal index

> Status: design record for `sq-lsp7k.7` / issue 3039. **Design only — nothing here has
> been implemented.**
>
> [OPUS-5] This record surveys the machinery that actually exists today (verified against
> the tree at `45bc801c`), corrects two premises in the bead text, fixes the architecture
> boundary, and decomposes the work into independently landable child beads. It makes no
> performance claim and asserts no capability that is not in the tree.
>
> **Research limitation, stated up front:** this session had no network access, so the
> competitor claims in §2 are carried forward from this repo's own prior-art records
> (`research/competitive-feature-analysis-2026-07.md`,
> `research/feature-research-broad-sparql-vendors.md`) and are **not re-verified against
> vendor documentation here**. Anything in §2 that becomes load-bearing for a public claim
> must be re-checked against primary sources before it is published.

## 1. What exists today (verified, not assumed)

Four independent pieces are already in the tree. They are close to what the bead assumes,
but they do not compose into point-in-time query, and the reason is specific.

### 1.1 The generation ring — in-memory, timestamp-aware, not HTTP-reachable

`crates/sparq-serve/src/ring.rs` publishes one immutable `Generation<Graph>` per group
commit, each carrying `number: u64`, `epochs: PodEpochs`, and `published_at: SystemTime`
(`ring.rs:109-155`). The ring already exposes **both** lookups:

- `GenerationRing::at(number)` (`ring.rs:317`) — pin by generation number.
- `GenerationRing::as_of(t: SystemTime)` (`ring.rs:331`) — "the newest retained generation
  with `published_at <= t`", returning `None` rather than substituting a different instant.

Retention is `max(RingConfig::retain, TimeTravelConfig::max_generations)` older
generations, with an optional age bound that applies only to the time-travel extension
(`ring.rs:47-71`). Eviction is publish-driven; there is no background timer.

**The correction.** The bead is written as though the timestamp path exists and only needs
to reach further back. It does not exist *at the server surface at all*. `resolve_pin`
(`crates/sparq-server/src/http.rs:5569`) accepts exactly one token — `?generation=N`, URL
query or url-encoded POST body, body winning — and its own docstring says so explicitly:

> "callers that track timestamps resolve them via the library's `GenerationRing::as_of`."

So `as_of` is reachable only by embedders linking `sparq-serve`. Over HTTP there is no
`?at=`, no timestamp in the response, and `Sparq-Generation` (`http.rs:5629`) carries a
number with no wall-clock anchor. A client cannot even ask "what was the state at 09:00"
*inside* the retention window. That is a cheap, self-contained gap the bead does not
mention, and it should be the first slice.

### 1.2 The durable CDC change stream — the right substrate, the wrong access pattern

`crates/sparq-serve/src/change_stream.rs` is a segmented, fsync'd, append-only log. Each
`ChangeRecord` carries `seq`, `generation`, `timestamp_unix_nanos`, ordered
inserts-then-deletes as N-Quads lines, and a `rebase` gap flag (`change_stream.rs:150-173`).
Segments are named `changestream-<first-seq>.cdc`, records are length-framed with an FNV-1a
digest, and `RetentionPolicy` / `apply_retention` drop **whole old segments** under
size/age pressure bounded by a consumer-ack watermark (`change_stream.rs:231-250`,
`change_stream.rs:394`). The bead's "retention machinery for the segmented log already
landed" is accurate.

Two properties of this log decide the whole design:

**(a) There is no timestamp index and no bounded read.** The only reader is
`ChangeLog::poll(from_seq) -> Result<Vec<ChangeRecord>, BackupError>`
(`change_stream.rs:695`). It re-reads segments from `from_seq` to the head and returns
every record **materialised in one `Vec`**. There is no `as_of(timestamp) -> seq`, no
segment-level time bounds, no range read, no iterator. Reconstructing a past state over
today's API therefore costs a full scan and a full in-RAM copy of the retained history —
for every request. This, not storage, is the actual missing artifact. "Persistent temporal
index" should be read as *exactly this*: a seekable, range-readable index over the log that
already exists.

**(b) Recording is already O(full graph) per commit.** `diff_changes`
(`change_stream.rs:1075`) serialises **both** generations to N-Quads and takes a `BTreeSet`
difference — identical in shape to `backup_delta::export_delta`
(`backup_delta.rs:114-160`). The per-commit cost of having history at all is therefore
proportional to dataset size, not to change size. Nothing in this design makes that worse,
and nothing in this design fixes it; any "cheap history" framing would be dishonest while
that holds. It is called out as its own bead in §6 because it caps how attractive the whole
feature is under write-heavy load.

Supporting invariants worth keeping in front of the implementer:

- `generation == seq` for a single-writer ring, but both are recorded *precisely so they
  can be checked against each other* (`change_stream.rs:42-43`). A `rebase` record advances
  `generation` without a corresponding commit, so an index must key on both and must never
  assume the identity.
- The quad diff is only meaningful **within one writer lineage**, because surviving terms
  keep their blank-node identity across a fork (`change_stream.rs:27-37`). Diffing across
  lineages is explicitly unsupported.
- A `rebase` record is an **honest hole**: the span `(previous generation, generation]` was
  never captured (`change_stream.rs:562-601`).

### 1.3 Backup + delta artifacts — a working forward-replay path

`backup_delta::replay` (`backup_delta.rs:410`) applies an ordered delta chain onto a
`Graph`, fail-closed on discontinuity, via `Graph::apply_delta_nquads(inserts, deletes)`
(`crates/sparq-core/src/lib.rs:3243`). That call parses N-Quads, groups per graph slot, and
applies deletes before inserts.

This is the load-bearing convenience of the whole design: **the reconstruction primitive
already exists and is already tested.** Forward replay is `apply_delta_nquads(inserts,
deletes)`; *backward* replay — undoing a commit — is the same call with the arguments
swapped, `apply_delta_nquads(deletes, inserts)`, because a `ChangeRecord` is a complete
quad-level diff. No new graph mutation machinery is needed in either direction.

### 1.4 The `/streams` endpoint

`crates/sparq-server/src/streams.rs` serves a Neptune `GetRecords`-shaped feed with
`iteratorType` ∈ {`TRIM_HORIZON`, `AT_SEQUENCE_NUMBER`, `AFTER_SEQUENCE_NUMBER`, `LATEST`}
(`streams.rs:184-195`). There is no `AT_TIMESTAMP`. Once a timestamp index exists, adding
it is nearly free and is worth doing in the same programme.

## 2. Prior art and what it tells us to *not* build

Carried from this repo's records; **not re-verified in this session** (no network).

| System | Approach (as recorded) | Signal for sparq |
|---|---|---|
| GraphDB history/versioning plugin | Dedicated `DSPOCI` index; `FROM <at/timestamp>` graph-name convention | Nearest parity target. The *query surface* convention (a magic graph name) is a viable alternative to a query parameter — see §4.3. |
| Stardog | **Retired** its versioning feature | The strongest signal in the table. A bespoke versioned store is a long-lived maintenance liability; the bead itself flags this. Weighs directly against option C in §3. |
| MarkLogic | Bitemporal (valid-time + system-time), full audit trail | Out of scope. sparq's time travel is **system-time only**; `research/feature-research-broad-sparql-vendors.md:243` already records the valid-time gap as a separate concern. Do not conflate. |

The academic line here (snapshot / change-based / timestamp-based RDF archiving and the
hybrid designs that combine them) is directly relevant and `ring.rs:44` already names an
"OSTRICH-style delta-chain archive" as the intended follow-up. That literature is **not
cited in detail here** because it could not be verified this session; a child bead should
do a proper survey before option C (§3) is ever reconsidered.

## 3. Options

**A — Backward replay from a retained ring generation.** Take the oldest retained
generation, clone its `Graph`, apply inverted change records back to the target.
*Pros:* no new on-disk artifact beyond the index; exact; the primitive exists (§1.3); fails
closed naturally at the trim horizon and at any `rebase` record. *Cons:* cost grows with
distance from now, so the far past is the expensive case — exactly backwards from what
users want; a full `Graph` clone per query.

**B — Forward replay from a base backup.** Anchor on the newest base backup at or before
the target, replay forward. *Pros:* it is what `backup` + `backup_delta` already do; cost is
bounded by anchor spacing, which the operator controls. *Cons:* requires operators to keep
periodic base backups; introduces cross-artifact retention coupling — a base must not be
trimmed while the log ahead of it is still the only route to a reachable instant.

**C — A purpose-built versioned index (OSTRICH/DSPOCI-shaped).** Snapshots plus delta
chains in a timestamp-keyed store, queried *natively* with no materialisation.
*Pros:* the real parity answer; avoids per-query materialisation entirely. *Cons:* XL. It is
a second storage engine, and it needs engine-level integration — the executor would have to
scan a versioned index rather than a `Graph`, which is a far larger blast radius than
`sparq-server`. Stardog's retirement is the maintenance-cost signal against it.

### Recommendation

**Build the index, and make anchor selection a planner decision over A and B. Explicitly
defer C.**

Concretely: one new `TemporalIndex` over the existing change-log directory answers
`as_of(t) -> Option<(seq, generation)>` and reports an honest horizon; one
`materialise_at(generation)` planner picks the cheapest available anchor — a retained ring
generation (replay backward) or a base backup (replay forward) — and replays with
`apply_delta_nquads` in the corresponding direction; the ordinary SPARQL path then runs
against the materialised `Graph` with no executor changes at all.

Three reasons this is the right boundary:

1. **The public surface is already future-proof.** `ring.rs:35-45` argues, correctly, that
   a number/timestamp-based API lets the storage swap happen with no API change. A `?at=`
   parameter answered by materialisation today can be answered by a native versioned index
   later without breaking a single client.
2. **It reuses everything and adds one genuinely missing thing.** The diff format, the
   durability contract, the retention machinery, the replay primitive, and the fail-closed
   error discipline all exist. The index and the range reader do not.
3. **It keeps `sparq-engine` out of it.** Option C cannot avoid touching the executor.
   A/B cannot help but avoid it.

The honest cost of this recommendation: **materialisation is per-query and proportional to
dataset size plus replay distance.** That is not a good steady-state answer for a hot
historical workload, and the design must therefore ship an explicit cost bound and a small
cache (§4.4) rather than pretend the cost is not there. If real workloads show
materialisation dominating, that is the evidence that would justify revisiting C — and the
child bead in §6 that records it should say so in those terms.

## 4. Design

### 4.1 `TemporalIndex` — the persistent index (in `sparq-serve`)

A sidecar over the change-log directory holding one entry per segment:

```text
first_seq, last_seq, first_generation, last_generation,
min_timestamp_unix_nanos, max_timestamp_unix_nanos,
contains_rebase: bool
```

- **Built from the log, never trusted over it.** Rebuildable by a full scan on open; the
  persisted form is a cache, and a mismatch against the segment set on disk means rebuild,
  not fail. This keeps the index off the crash-safety critical path entirely — the log
  remains the only durable source of truth.
- **Updated by retention.** `apply_retention` already reports `first_retained_seq`
  (`change_stream.rs:251-266`); dropping segments drops index entries, and the trim horizon
  becomes a *timestamp* horizon the server can quote in an error message.
- **Resolution.** `as_of(t)` binary-searches segments on `max_timestamp`, then scans within
  the one candidate segment. Wall-clock timestamps are not monotonic under clock steps, so
  the within-segment scan must resolve ties the same way `GenerationRing::as_of` does —
  toward the newest qualifying record — and the two implementations must be tested against
  each other on the overlap where the ring and the log both cover an instant.
- **`contains_rebase` is the fail-closed flag.** If the span between the chosen anchor and
  the target crosses a `rebase` record, reconstruction is **refused**, not attempted. This
  is the single most important correctness rule in the design: a rebase marks a span that
  was never captured, so replaying across it silently produces a state the store was never
  in.

### 4.2 Bounded reads

`poll -> Vec` (§1.2a) must not be the replay path. Add a range/streaming read —
`read_range(from_seq, to_seq)` or an iterator — so replay is bounded by the requested span
rather than the whole retained log, in both time and memory. `poll` keeps its current
signature and semantics for existing consumers.

This is also a direct prerequisite for standing-query IVM, which needs every commit in
order and must not materialise history to get them — see §5.

### 4.3 Server surface

Extend `resolve_pin` (`http.rs:5569`) with `?at=<timestamp>`, same precedence rule as
`generation` (body wins over query string), and **mutually exclusive** with `?generation=N`
— supplying both is a `400`, never a silent precedence rule.

Accepted forms should be exactly two and both unambiguous: an `xsd:dateTime` /
RFC 3339 string with a required timezone offset, and integer Unix nanoseconds. A local
time with no offset must be rejected rather than assumed to be UTC.

Status semantics, extending the existing discipline rather than inventing one:

| Condition | Status | Rationale |
|---|---|---|
| Unparsable / naive timestamp / both `at` and `generation` | 400 | Client error, retry cannot help without a fix. |
| Timestamp after the current generation | 400 | Matches the existing "not yet published" rule (`http.rs:5590`). |
| Timestamp before the temporal horizon | 410 Gone | Matches the existing aged-out rule; the message must quote the *horizon timestamp*, not just a generation number. |
| Span crosses a `rebase` gap | 409 Conflict | Distinct from "trimmed": the history is intact on both sides but the span was never captured. A different operator remedy (restore from a backup), so a different code. |
| Reconstruction exceeds the configured budget | 400 with a typed body | See §4.4. Not 503 — it is a property of the request, not of server health. |

The response must carry the resolved generation in the existing `Sparq-Generation` header
plus a new timestamp header, so a client that asked by time learns the exact instant it
actually got. Without that round-trip the parameter is not honestly usable.

`/streams` gains `iteratorType=AT_TIMESTAMP` over the same index (§1.4).

Updates keep the existing rule: any pin on an update is a 400.

### 4.4 Cost bound and cache — not optional

Materialisation is a full graph copy plus a replay. Left unbounded, `?at=` is a
straightforward request-amplification vector on an endpoint whose read path has no authz
(`research/threat-model.md:471`). The feature must therefore ship with:

- a configured ceiling on reconstruction work (records replayed and/or anchor distance),
  refused with a typed 400 that states the measured cost and the ceiling;
- a small bounded cache of materialised historical snapshots, keyed by generation, so
  paginating through a historical result set does not re-materialise per request;
- a hard cap on concurrent in-flight materialisations.

### 4.5 Privacy — the load-bearing consequence

**A durable temporal index makes deleted data queryable again.** Any deployment that treats
SPARQL `DELETE` as redaction — including erasure-request handling — is silently broken by
this feature. The design's position must be explicit and must appear in the operator docs,
not only here:

- the feature is **opt-in and off by default**, like `time-travel` and `change-stream`
  already are;
- **trim is the erasure mechanism.** The advertised history horizon is exactly the window
  in which deleted data remains reachable, and shortening retention is the only way to
  make a deletion final;
- this interacts with `research/crypto-erase-at-rest.md` and must be reconciled with it
  before the feature is documented as production-suitable.

### 4.6 Trust programme boundary

The bead notes the pairing with provenance/attestation ("prove what the graph said at time
T") and says **do not couple**. Honoured: nothing here takes a dependency on `sparq-trust`
or `sparq-zk`, and no artifact in this design is a proof of anything. The change log's
digest is FNV-1a — a non-cryptographic integrity check that detects truncation and **not**
tampering (`change_stream.rs`, digest discussion). Any attestation story would need a
different digest and a different threat model, and belongs to that programme, not this one.

## 5. Sequencing against standing-query IVM (`sq-lsp7k.6`)

`research/standing-query-ivm-design.md` puts an opt-in `subscription-ivm` feature in
**`sparq-server`**, hard-dependent on `change-stream`, consuming ordered `ChangeRecord`s.
The bead's collision warning is real, and it is concretely in three places: `AppState`
wiring, the `sparq-server` Cargo feature graph, and `ChangeLog`'s reader API.

The resolution follows from where each phase lives:

- **Phases 1 and 2 (the index and the bounded reader) are pure `sparq-serve`.** They touch
  no `sparq-server` file. They can land **in parallel with, or before,** the IVM bead — and
  IVM actively wants Phase 2, since it must consume every commit in order without
  materialising the log. Landing Phase 2 first makes the IVM work smaller, not larger.
- **Phases 3–5 touch `http.rs` and `AppState` and should land after the IVM bead**, or at
  minimum not concurrently with it. `http.rs` is the known merge-conflict hot spot.

Recommended order: **Phase 0 → (Phase 1, Phase 2) → IVM → Phases 3–6.** Phase 0 is small
and touches `resolve_pin` only; if IVM is already in flight, Phase 0 waits behind it.

## 6. Phased plan (each phase = one child bead)

1. **`?at=` over the in-memory ring** — S, `sparq-server`. Wire `GenerationRing::as_of`
   into `resolve_pin`; add the response timestamp header; `at`/`generation` mutual
   exclusion; the 400/410 semantics of §4.3 restricted to the ring's window. Delivers
   honest arbitrary-timestamp query *within the retention window* and closes the
   round-trip gap of §1.1 with no new storage. **Acceptance:** a query at a timestamp
   between two publishes returns the earlier generation's data and reports that
   generation's number and timestamp; a timestamp below the ring's oldest retained
   generation is 410; both feature states behave identically where the window overlaps.
2. **`TemporalIndex`** — M, `sparq-serve`. Per-segment index of §4.1, rebuild-on-open,
   retention-aware, `contains_rebase`. **Acceptance:** `as_of` agrees with a brute-force
   scan over randomised logs including non-monotonic timestamps; the horizon moves exactly
   with `apply_retention`; a rebase-spanning resolution is refused.
3. **Bounded change-log reads** — M, `sparq-serve`. `read_range` / iterator per §4.2;
   `poll` unchanged. **Acceptance:** replaying a span allocates proportionally to the span,
   not the log; fail-closed behaviour below the trim horizon is preserved.
4. **Materialisation planner** — L, `sparq-serve`. Anchor selection across retained ring
   generations and base backups; forward and backward replay via `apply_delta_nquads`; the
   cost model and budget of §4.4. **Acceptance:** for every reachable generation, the
   materialised graph equals the graph that generation actually published — checked as a
   randomised differential against a run that retains every generation in the ring;
   over-budget requests are refused rather than served slowly.
5. **Server surface beyond the ring** — M, `sparq-server`. Extend `?at=` through the
   planner; the full §4.3 status table including 409-on-rebase; the snapshot cache and
   concurrency cap; `/streams` `AT_TIMESTAMP`. **Acceptance:** a timestamp older than the
   ring but inside the log's horizon returns correct historical results; the 410 message
   quotes a timestamp horizon; tests cover both feature states.
6. **Unified retention horizon** — M, `sparq-server` + `sparq-serve`. One operator-facing
   history-horizon knob composing ring retention, change-log retention, and base-backup
   retention, so the advertised horizon is a single honest number rather than the emergent
   minimum of three independent policies; surfaced in status output and in the 410 body.
   **Acceptance:** the advertised horizon is never later than any instant the server can
   actually serve, under randomised retention schedules.
7. **Docs, SKILL surface, and the privacy statement** — S. §4.5 written for operators, in
   the crate README and the relevant `SKILL.md`, including the explicit statement that
   deleted data stays queryable within the horizon and that trim is the erasure mechanism.
8. **Delta-cost investigation** — M, `sparq-serve`, *independent of this feature*. §1.2b:
   `diff_changes` is O(full graph) per commit. Measure it, decide whether the applier can
   hand the writer an exact change set instead, and record the answer. This is the single
   biggest determinant of whether durable history is affordable under write load, and it
   is currently unmeasured.

Item 8 is deliberately outside the critical path: this feature does not create that cost
and does not need it fixed to be correct, but no honest performance story about history can
be told until it is measured.

## 7. Open questions for the maintainer

1. **Surface convention.** `?at=<timestamp>` (this design) or GraphDB's `FROM <at/…>`
   magic-graph-name convention? The parameter composes with the existing `?generation=N`
   and needs no parser change; the graph-name form is closer to the competitor and works
   inside a federated query, but overloads dataset semantics. Recommend the parameter, but
   this is a compatibility judgement, not a technical one.
2. **Is per-query materialisation acceptable at all**, or is the only version of this
   feature worth shipping the native versioned index (option C)? The recommendation here
   assumes historical queries are rare relative to current-state queries. If that is wrong
   for the intended users, phases 4–5 are the wrong build.
3. **Base backups as a hard requirement.** Should the planner *require* a base-backup
   schedule to advertise a long horizon (bounding replay distance), or accept the
   unbounded-backward-replay cost when no backup exists? This is an operator-contract
   decision.
4. **Privacy posture (§4.5).** Does the erasure story need to be settled — including
   reconciliation with `research/crypto-erase-at-rest.md` — *before* phase 5 ships, or is
   an opt-in-plus-documented-caveat posture sufficient for the first release?
5. **Valid-time.** MarkLogic-style bitemporal is recorded as a separate gap. Confirm it
   stays out of scope for `sq-lsp7k.7` so the `?at=` surface is unambiguously system-time.

## 8. Uncertainties

- §2's competitor claims are second-hand within this repo and unverified this session.
- The relative cost of backward versus forward replay is unmeasured; the planner's cost
  model in phase 4 is a design placeholder until phase 4 measures it.
- Whether `Graph::apply_delta_nquads` with swapped arguments is an exact inverse of a
  recorded commit **in the presence of blank nodes** is argued from the lineage invariant
  (`change_stream.rs:27-37`) and the existing round-trip tests, but is not separately
  proven. Phase 4's randomised differential is what would establish it, and it should be
  treated as an obligation of that phase rather than an assumption of this document.
