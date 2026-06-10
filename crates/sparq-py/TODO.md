# sparq-py — API gaps & follow-ups (T21)

Gaps in the wrapped crates' public APIs found while building the Python bindings.
Per the T21 scope, no existing crate source was touched; these are recorded for
their owning crates.

## Engine (sparq-engine)

- **No native ASK / CONSTRUCT / DESCRIBE.** `sparq_engine::{query, query_json, count}`
  accept only SELECT. `Graph.ask` therefore rewrites ASK → `SELECT *` over the same
  pattern via spargebra and answers `count > 0` — the same workaround
  `sparq-server::exec::prepare` uses. Exact, but it cannot short-circuit after the
  first solution (no `LIMIT 1`-style early exit through the lazy count path).
  CONSTRUCT / DESCRIBE raise `ValueError` from `Graph.query`.
- **Update drops named graphs.** `sparq_engine::update` collects only the default
  graph's triples and rebuilds; a dataset loaded from N-Quads/TriG loses its named
  graphs after `Graph.update`. (Named-graph targets inside the update itself are
  already rejected by the engine with an error.)

## Reasoning (sparq-reason)

- **`reason_n3` has no graph-input entry point.** `reason_n3(&mut Dict, src: &str)`
  parses facts AND rules from one N3 document; there is no way to combine an
  already-loaded graph with a separate rules document. So `graph.reason("n3")` is
  rejected with a pointer to `Graph.load_n3(text)` (document-level N3 closure).
  A `reason_n3_with(dict, triples, rules_src)` API would unlock
  `graph.reason("n3", rules=...)`.
- **Reasoning rebuilds, and drops named graphs** for the same reason as update:
  the closure is materialized over the default graph's triples and the graph is
  rebuilt with `from_parts` (which starts with no named graphs).
- `inconsistencies()` (OWL clash detection) is not yet surfaced in Python; it
  needs (dict, triples) access at the right moment — easy follow-up in
  `Graph.reason("owl")` if wanted (e.g. return or raise on clashes).

## Core (sparq-core)

- **`Dict`/`Graph` are not `Clone`,** so `Graph.copy()` cannot be offered cheaply;
  `reason()` works around it by moving the public `dict`/`store` fields out and
  re-deriving the triples with a full-store scan.
- **`Graph::save` panics on a sparse numeric cache** (`NumData::as_slice`
  unreachable). Not reachable through these bindings today (we never call
  `into_compressed`), but worth a `Result` upstream before exposing a compressed
  mode in Python.

## Packaging

- The workspace `release` profile is `panic = "abort"`, which would abort the whole
  Python interpreter on any Rust panic inside the extension. Added a
  `python-release` profile (`inherits = "release"`, `panic = "unwind"`); wheels must
  be built with `maturin build --profile python-release`. CI's quick check uses
  `maturin develop` (dev profile, unwinding) so it is unaffected.
- Wheels-matrix release wiring (manylinux/macos/windows × abi3, `maturin-action`)
  is a follow-up — see the note in `docs/release.md`.
- The PyPI name `sparq` must be checked for availability before first publish
  (same caveat as the crates.io names in `docs/release.md` §0).
