# sparq-rsp — follow-ups

Constraint honoured in v1 and v2: **no existing crate was modified**; nothing
in the workspace depends on this crate, and the wasm build is untouched.

## The big one: overlay evaluation instead of rebuild-per-window — DONE (v2)

v1 rebuilt a fresh dictionary-encoded `Graph` per closed window (the measured
~1.6 M triples/s plateau). v2 implements the first two rungs of the overlay
ladder as selectable [`EvalMode`]s, benchmarked head-to-head (README table,
1 M triples, Apple M1):

1. **Persistent stream dictionary** (`EvalMode::PersistentDict`) — DONE, and
   the **default**: ONE `Dict` per `ContinuousQuery` lifetime, terms interned
   once at push time, per-window graphs from already-interned `[Id; 3]`s via
   `Graph::from_parts`. Wins every benchmark scenario (1.2–5.3× over v1;
   biggest on sliding windows, where v1 re-interned the shared suffix every
   step). The dictionary grows monotonically with the stream's vocabulary —
   accepted and documented; periodic compaction remains a follow-up for
   unbounded-vocabulary streams (use `EvalMode::Rebuild` there meanwhile).
2. **Delta application** (`EvalMode::Delta`) — DONE, kept opt-in,
   **measured-and-not-defaulted**: one live `Graph` + per-slide
   `apply_delta(inserts, deletes)` (exact set-semantic diff between
   consecutive windows; multi-timestamp duplicates handled — a triple is only
   deleted when its LAST in-window occurrence leaves), compacting when churn
   outgrows the window. As the v1 TODO warned, the overlay cost dominates:
   `apply_delta` is term-level (intern + `id_of` per change) and overlay rows
   are re-sorted per scan, so it loses to PersistentDict everywhere
   (0.29–2.01 M; see README). Kept because its per-slide work is O(changes).
3. **True overlay snapshot** — DEFERRED, **blocked-on-engine-seam**: needs the
   cheap-snapshot API recorded in the workspace `TODO.md` ("sparq-core: cheap
   graph snapshot API") so the window loop can keep one mutable graph and hand
   the engine an O(1) immutable snapshot per closed window. That seam would
   also remove PersistentDict's remaining per-window cost (index build +
   O(dictionary) numeric/temporal cache rebuild in `Graph::from_parts`).
4. **Parse/plan once** — **DONE (both sides)**: engine side
   `sparq_engine::PreparedQuery` (parse / `From<spargebra::Query>`) with
   `query_prepared` / `ask_prepared` / `count_prepared` / `construct_prepared`
   (+ `_with_budget`) entry points; the string APIs are thin wrappers, so
   semantics are identical. Rsp side: all three continuous forms parse a
   `PreparedQuery` once at `register` time (where the form check already
   forced a parse) and execute the prepared algebra per window. Measured
   honestly (1 M triples, interleaved A/B, 5 reps): parsing the README's AVG
   query costs ~2.6 µs, so the saving only clears run-to-run noise at very
   small windows — RANGE 10 PersistentDict dropped ~11.8 → ~9.8 µs/window
   (median, ~17 %); at RANGE 100 the ~5 % saving is within noise, and above
   that it vanishes. Adopted anyway: strictly less work per window, no API
   change, and registration was already paying the parse.

## Smaller gaps

- **Window origin `t0`** — DONE: `WindowSpec::with_t0(t0)` (RSP-QL's
  parameterised origin); windows are `[t0 + k·step, t0 + k·step + range)`,
  pre-origin arrivals belong to no window (not late) but advance the
  watermark. Pinned by tests.
- **ASK / CONSTRUCT continuous queries** — DONE: `ContinuousConstruct`
  (stream-to-stream transformation; R2S as exact set diffs over the
  constructed graphs) and `ContinuousAsk` (one boolean per window, RSTREAM
  semantics), both validated at registration, both honouring `EvalMode`.
- **RSP-QL surface syntax** — DEFERRED, out-of-scope-by-design for now:
  `register` takes plain SPARQL + a programmatic `WindowSpec`; there is no
  `FROM NAMED WINDOW :w ON STREAM :s [RANGE PT10S STEP PT5S]` parser, and no
  named streams / multiple windows per query (RSP-QL allows joining several
  windows). One stream, one window, one query. Doing this properly is a
  parser + multi-window-join design (each window needs its own S2R state and
  the engine a dataset-per-window view) — a feature in its own right, not an
  increment on the current pipeline; revisit if a consumer actually speaks
  RSP-QL text.
- **R2S hash collisions**: ISTREAM/DSTREAM SELECT diffs use 64-bit row hashes
  (documented, vanishingly unlikely to collide). Exact term-level multiset
  diffing would remove the caveat at the cost of hashing full term values.
  (CONSTRUCT diffs are exact set differences — no caveat.)
- **Per-window budget**: `sparq_engine::query_with_budget` exists; exposing a
  `QueryBudget` per registered query would bound worst-case window evaluation
  in embedded deployments.
