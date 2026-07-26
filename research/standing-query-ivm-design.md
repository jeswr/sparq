# Standing-query incremental view maintenance

> Status: design record for `sq-lsp7k.6` / issue 2707.
>
> [SONNET-4.6] This record fixes the first implementation boundary and resolves the
> `/subscriptions` wire-contract question. It does not claim that the implementation has
> landed.

## Decision

`sparq-server` will add an opt-in `subscription-ivm` feature which maintains eligible
standing SELECT queries from the ordered quad additions and deletions of each committed
writer generation. The initial release supports positive basic graph patterns, deterministic
FILTER expressions, projection, DISTINCT, and simple grouped aggregates. Every other query
continues to use the existing snapshot re-evaluation path.

The existing protocol v1 remains byte-for-byte compatible. A client explicitly requests
protocol v2 when subscribing. V2 preserves `addedResults` and `removedResults` and adds
commit identity, maintenance mode, and a durable history cursor. This answers open question
6 in `research/competitive-feature-analysis-2026-07.md`: evolve the contract only through
explicit negotiation; do not silently change v1 coalescing or sequence semantics.

The first implementation belongs in `sparq-server`. It may reuse generic delta-join
machinery already shared by the engine and reasoners, but it must not make
`sparq-engine` depend on the server or CDC persistence.

## Existing seams

The current server already has three relevant pieces:

- `subscriptions.rs` owns registration, the sequence-zero snapshot, result canonicalisation,
  WS/SSE transport sharing, and snapshot re-evaluation plus set diff.
- The sequenced writer publishes one immutable graph generation per group commit. Its commit
  hook observes both the previous and new generations after durable update work succeeds and
  before readers are notified.
- With `change-stream` enabled, `ChangeLog` records one ordered `ChangeRecord` per published
  generation in a segmented, fsync'd log. Each record contains its sequence, generation,
  timestamp, and ordered quad additions/deletions. A `REBASE` record explicitly marks a gap.

The `tokio::sync::watch<u64>` notification used by v1 is deliberately lossy: it coalesces
generations. IVM must not consume that channel. It needs every commit, in order, including
commits whose net effect is later cancelled.

## Supported algebra fragment

Eligibility is decided once from the parsed `PreparedQuery`; matching source text is not
used. The classifier is fail-closed: an unknown algebra or expression variant is ineligible.

The initial IVM fragment is:

```text
Select
  Project?
    Distinct?
      Group?                 # optional, outermost relational operator
        Filter*
          positive BGP
```

The following are eligible:

- One or more triple patterns joined as a positive BGP.
- Default-graph patterns and `GRAPH <constant>` patterns. `GRAPH ?g`, dataset clauses which
  merge multiple default graphs, and protocol dataset overrides remain on re-evaluation
  until quad-to-active-dataset provenance is represented explicitly.
- FILTER over variables bound by the BGP, using deterministic scalar expressions whose
  engine semantics are available to the maintainer. Boolean connectives, comparisons,
  arithmetic, bound checks, and deterministic string/numeric functions are included.
- Projection and DISTINCT under the subscription protocol's existing distinct-binding
  output semantics.
- `GROUP BY` variables with `COUNT`, `SUM`, `AVG`, `MIN`, and `MAX`, without DISTINCT
  aggregate arguments. The ungrouped form is one global group.

The following force snapshot re-evaluation:

- OPTIONAL, UNION, MINUS, EXISTS/NOT EXISTS, SERVICE, subqueries, VALUES, BIND/Extend,
  property paths, variable graph names, federation, and lateral joins.
- ORDER BY, OFFSET, LIMIT, REDUCED, window functions, custom aggregates, aggregate DISTINCT,
  and nondeterministic functions such as random, UUID, or current-time evaluation.
- Query forms other than SELECT.

This boundary is intentionally conservative. Ineligibility is not a subscription error:
the registration succeeds in `reevaluate` mode and retains today's behavior and budgets.

## Maintained state

An eligible subscription compiles to an immutable `IvmPlan` plus mutable `IvmState`.

`IvmPlan` contains:

- normalised triple-pattern leaves and their graph scope;
- join variable layouts and a fixed join order;
- compiled FILTER and projection operations;
- optional group keys and aggregate descriptors; and
- a stable plan fingerprint derived from canonical algebra plus the IVM format version.

`IvmState` contains counted multisets rather than sets:

- one counted relation for every BGP leaf;
- counted intermediate join relations;
- a counted projected-result relation;
- per-group aggregate state; and
- the last applied CDC sequence and graph generation.

Counts are required even though the wire output has set semantics. Two derivations may
produce the same projected binding; deleting one must not emit a removal while another
derivation remains. A wire addition occurs only on a result count transition from zero to
positive, and a removal only on a transition from positive to zero.

Aggregate state is deletion-capable:

- `COUNT` stores the contributing row count.
- `SUM` stores count and numeric sum.
- `AVG` stores count and numeric sum and derives the result.
- `MIN` and `MAX` store a counted ordered multiset of values.

Before and after each touched group is updated, its canonical output binding is compared.
The old binding is removed and the new binding added when the aggregate value changes. An
empty group follows the engine's SPARQL aggregate semantics rather than a server-specific
special case.

State is bounded by the existing subscription result limit plus a separate internal-state
budget. The state budget accounts for leaf, intermediate, support-count, and aggregate
entries. Exceeding it transitions the subscription to `reevaluate` mode at the same committed
generation; it must never evict entries and continue incrementally.

## Per-commit delta algorithm

For commit `C`, processing is serial and atomic per subscription:

1. Verify that `C` is the immediate successor of `last_cdc_sequence` and that its generation
   advances the same lineage. A gap, `REBASE`, or lineage change triggers fallback.
2. Convert each changed quad into signed leaf-relation tuples for every matching triple
   pattern. A delete has weight `-1`; an insert has weight `+1`.
3. Propagate signed tuples through the BGP using differential join expansion. For relations
   `R` and `S`, a commit applies
   `Δ(R ⋈ S) = (ΔR ⋈ Sold) + (Rold ⋈ ΔS) + (ΔR ⋈ ΔS)`.
   For a longer BGP, the same rule is applied in the plan's fixed left-deep order, with
   each stage reading a consistent old-plus-current-commit view.
4. Apply FILTER with the engine's expression/error semantics, then projection and support
   counting.
5. Apply aggregate deltas to touched groups.
6. Derive wire additions/removals from zero-crossings or changed group bindings.
7. Append the history entry durably. Only after the append succeeds, publish the new
   in-memory state and advance its CDC cursor.

All arithmetic on support counts is checked. A negative count, overflow, missing delete,
evaluation error that cannot be represented consistently, or history append failure aborts
the incremental step. The server pins the committed graph generation and rebuilds the
subscription state by full evaluation; if that cannot be done under its budgets, the
subscription terminates using the existing error behavior.

Commits are queued on a bounded broadcast channel fed by the writer commit hook, not the
coalescing watch channel. A lagged receiver replays missing records from the segmented log.
If the records are unavailable, it rebuilds from the newest pinned snapshot and records a
history discontinuity.

## Bootstrap and concurrency

Registration must not race a commit between the initial snapshot and cursor capture:

1. Read the durable CDC tail cursor.
2. Pin the matching current graph generation.
3. Evaluate the full initial result and build all counted relations from that same snapshot.
4. Re-read the CDC tail. Replay every later record before exposing the subscription.
5. Emit sequence zero for the snapshot, followed by any replayed per-commit changes.

If a matching tail/generation pair cannot be established after a bounded retry, registration
uses `reevaluate` mode. There is no interval in which an eligible subscription silently
misses a commit.

Commit processing runs off the writer thread. The hook performs only the existing durable
record append and a bounded notification send. Per-subscription state is owned by one worker,
so commits for a subscription cannot reorder. Different subscriptions may process commits in
parallel.

## Durable change history

IVM answer history is a separate segmented log rooted below the configured change-stream
directory. Raw CDC and answer deltas have different retention and schema, and mixing them
would make `/streams` compatibility and compaction harder.

Each history record contains:

```text
format_version
subscription_key
plan_fingerprint
cdc_sequence
generation
commit_timestamp_nanos
added_results
removed_results
mode                 # "ivm" or "reevaluate"
continuity           # "continuous" or "reset"
```

`subscription_key` is a server-issued opaque random identifier, distinct from the
connection-local numeric `id`. It is returned only by protocol v2 and is the authorization
and lookup key for history. The persisted record never contains bearer credentials.

History retention is segment-based and independently configurable. A cursor identifies
`subscription_key`, CDC sequence, and record offset. Expired cursors return HTTP 410 with the
earliest available cursor; malformed or unknown cursors return HTTP 400/404 without revealing
other subscription keys.

The retrieval surface is:

```text
GET /subscriptions/history/{subscription_key}?after=<opaque-cursor>&limit=<n>
```

It uses the same read authentication gate as subscription registration. Deployments needing
per-principal isolation must place the server behind an authorization layer; possession of a
subscription key alone does not override the configured read gate.

On restart, history remains retrievable. Active subscriptions do not automatically resume:
a v2 client resubscribes with its query and may provide its prior history cursor. The server
continues only when the plan fingerprint matches and CDC coverage is continuous; otherwise it
returns a new snapshot with `continuity: "reset"`.

## Wire contract

Protocol v1 remains unchanged:

- WS subscribe frames without `version` use v1.
- The existing SSE endpoint without a version parameter uses v1.
- Notification `sequence` continues to count delivered, potentially coalesced
  re-evaluations.

Protocol v2 is requested with `"version": 2` in a WS subscribe object or `version=2` on SSE.
The `subscribed` response adds:

```json
{
  "subscribed": {
    "id": 1,
    "alias": "ages",
    "version": 2,
    "mode": "ivm",
    "subscriptionKey": "opaque",
    "historyCursor": "opaque"
  }
}
```

Each v2 notification keeps the v1 result fields and adds:

```json
{
  "notification": {
    "id": 1,
    "sequence": 1,
    "generation": 42,
    "commitSequence": 17,
    "mode": "ivm",
    "continuity": "continuous",
    "historyCursor": "opaque",
    "addedResults": {},
    "removedResults": {}
  }
}
```

For v2 `ivm` mode, there is one durable notification record per committed transaction which
changes the answer. `commitSequence` identifies the source CDC record; `sequence` remains
the per-subscription delivery sequence. Commits with an empty answer delta are represented
in the durable cursor progression but need not produce a transport notification.

For an ineligible query, `mode` is `reevaluate`. V2 may coalesce commits in that mode, so a
notification includes `fromCommitSequence` and `throughCommitSequence` instead of claiming a
single source commit. A mode or continuity change is explicit in the next notification.
Clients which require commit-exact deltas can reject a `reevaluate` subscription after the
`subscribed` response.

Unknown protocol versions are refused at registration. Servers never infer v2 from a header
or silently add v2 semantics to an existing connection.

## Failure and recovery rules

Correctness takes precedence over remaining incremental:

| Condition | Result |
| --- | --- |
| Unsupported algebra at registration | Subscribe in `reevaluate` mode |
| CDC gap, `REBASE`, or lineage change | Full rebuild; emit `continuity: "reset"` |
| Receiver lag with retained CDC records | Replay in order, remain `ivm` |
| Count invariant or expression-maintenance failure | Full rebuild from committed generation |
| IVM state budget exceeded | Full rebuild and remain in `reevaluate` mode |
| History append fails | Do not acknowledge incremental cursor; rebuild or terminate |
| Full rebuild exceeds query budgets | Existing terminal subscription error |

Fallback is fail-closed: no delta is emitted from partially applied state, no cursor advances
past an undurable history record, and no query is labelled `ivm` after it has switched to
re-evaluation.

## Verification boundary

The implementation is accepted only with differential tests that run the same eligible query
through IVM and through full evaluation after every commit, comparing counted internal
results and emitted set deltas. The corpus must cover:

- insert and delete on every BGP leaf, including multiple changed leaves in one commit;
- duplicate derivations and projection collisions;
- FILTER true, false, error, and unbound cases;
- named-graph constants;
- each supported aggregate, empty groups, group creation/removal, and repeated extrema;
- group commits containing cancelling operations;
- restart/replay, retention expiry, lag recovery, `REBASE`, and corrupted/truncated history;
- v1 compatibility and v2 WS/SSE parity; and
- randomized short commit traces checked against full re-evaluation after each generation.

Performance evidence is recorded in generated benchmark artifacts rather than hard-coded in
this design. The critical assertion is structural: after bootstrap, an eligible subscription
does not call the full query executor unless one of the documented fallback conditions occurs.

## Delivery slices

The work divides into independently reviewable slices: algebra eligibility and differential
test oracle; counted BGP/FILTER maintenance; projection and aggregate maintenance; ordered
commit delivery plus CDC replay; durable answer-history storage and retrieval; then v2
WS/SSE negotiation. V1 remains the default throughout, allowing each slice to land without
changing existing clients.
