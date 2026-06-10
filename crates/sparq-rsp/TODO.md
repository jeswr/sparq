# sparq-rsp — follow-ups

Constraint honoured in v1: **no existing crate was modified**; nothing in the
workspace depends on this crate, and the wasm build is untouched.

## The big one: overlay evaluation instead of rebuild-per-window

v1 is honest but naive in R2R: every closed window materialises a **fresh**
dictionary-encoded `Graph` (`Dict::intern` per term + full index build) and the
SPARQL text is re-parsed and re-planned by `sparq_engine::query`. Overlapping
sliding windows (`STEP < RANGE`) re-intern and re-index the shared suffix on
every step — with `RANGE 10·STEP 1`, ~90 % of each build is redone work. The
measured plateau (~1.6 M triples/s, see README) is exactly this per-window
materialisation cost.

The fix is an **overlay / delta design**, in ascending order of core support
needed:

1. **Persistent stream dictionary** (no core change): keep ONE `Dict` for the
   life of the `ContinuousQuery`, intern each triple once at push time, and
   build per-window graphs from already-interned `[Id; 3]`s via
   `Graph::from_parts` — removes term hashing/allocation from the window loop.
   (Dictionary grows monotonically with the stream's vocabulary; needs either
   periodic compaction or acceptance.)
2. **Delta application** (existing core API): maintain one live `Graph` and
   per-slide `apply_delta(inserts = new step, deletes = evicted step)` instead
   of rebuilding — the eviction sets are exactly what `WindowedStream` already
   computes. Worth benchmarking against (1); `apply_delta`'s rebuild threshold
   may dominate for small windows.
3. **True overlay snapshot** (blocked on core): the cheap-snapshot API already
   recorded in the workspace `TODO.md` ("sparq-core: cheap graph snapshot API")
   would let the window loop keep one mutable graph and hand the engine an O(1)
   immutable snapshot per closed window.
4. **Parse/plan once**: cache the parsed `spargebra` algebra at `register` time
   and add a `sparq_engine` entry point taking pre-parsed algebra (today only
   the SPARQL string API is public). Parsing is microseconds — only worth it
   for very small windows at high window rates (the RANGE 100 line in the
   README).

## Smaller gaps

- **RSP-QL surface syntax**: `register` takes plain SPARQL + a programmatic
  `WindowSpec`; there is no `FROM NAMED WINDOW :w ON STREAM :s [RANGE PT10S
  STEP PT5S]` parser, and no named streams / multiple windows per query
  (RSP-QL allows joining several windows). One stream, one window, one query.
- **Window origin `t0` is fixed at 0** (documented divergence): RSP-QL
  parameterises it. Trivial to add to `WindowSpec::time` if needed.
- **ASK / CONSTRUCT continuous queries**: only SELECT is accepted. CONSTRUCT
  per window would give stream-to-stream transformation (the engine already
  has `construct`).
- **R2S hash collisions**: ISTREAM/DSTREAM diffs use 64-bit row hashes
  (documented, vanishingly unlikely to collide). Exact term-level multiset
  diffing would remove the caveat at the cost of hashing full term values.
- **Per-window budget**: `sparq_engine::query_with_budget` exists; exposing a
  `QueryBudget` per registered query would bound worst-case window evaluation
  in embedded deployments.
