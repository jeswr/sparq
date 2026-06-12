# sparq-py — API gaps & follow-ups (T21)

Gaps in the wrapped crates' public APIs found while building the Python bindings.
Per the T21 scope, no existing crate source was touched; these are recorded for
their owning crates.

## Engine (sparq-engine)

- ~~**No native ASK / CONSTRUCT / DESCRIBE.**~~ **DONE (py-parity wave).**
  `Graph.ask` runs `sparq_engine::ask_prepared` (early-exiting; SELECT still
  accepted via the lazy count), and `Graph.construct` / `Graph.describe` return
  `(s, p, o)` `Term`-tuple lists.
- ~~**Update drops named graphs.**~~ **DONE (py-parity wave, on top of update v2/F19).**
  Named-graph caveats dropped from the Python docs/docstrings; GRAPH-scoped data
  ops, graph templates and DROP covered by pytest.
- **Full-text (`text:` magic predicates) is NOT reachable through `Graph.query`.**
  Verified: `sparq-text` is deliberately a standalone opt-in crate — no other crate
  (including sparq-engine and sparq-py) depends on it, and the `text:` rewrite only
  runs through `sparq_text::query_text(graph, &TextIndex, sparql)`. Exposing it in
  Python means adding the dependency plus a `TextIndex` lifecycle on the wrapper
  (build lazily, invalidate on every update/reason swap) — real new surface, left
  as a follow-up rather than wired in quietly.

## Reasoning (sparq-reason)

- ~~**`reason_n3` has no graph-input entry point.**~~ **DONE (py-parity wave).**
  `Graph.reason_n3_with(rules)` applies a caller-supplied N3 rules document to an
  already-loaded graph: the default graph is rendered as N-Triples (a syntactic
  subset of N3) under the rules document and run through `reason_n3` — the same
  composition `MaterializedN3Graph` uses in fallback mode, so list expansion and
  builtins behave identically to `load_n3`. (A native
  `reason_n3_with(dict, triples, rules_src)` seam in sparq-reason would skip the
  serialize/reparse round trip; worth it only if profiling says so.)
- ~~**Reasoning rebuilds, and drops named graphs.**~~ **DONE (py-parity wave).**
  `reason` / `reason_n3_with` carry `Graph.named` across the rebuild (reasoning
  itself still materializes over the default graph only).
- ~~`inconsistencies()` not surfaced in Python.~~ **DONE (py-parity wave).**
  `Graph.inconsistencies()` returns the clash descriptions (run `reason("owl")`
  first for entailed clashes). It stays a separate query rather than a raise
  inside `reason("owl")` — detection is over asserted triples, and callers may
  want the closure even when inconsistent.

## Core (sparq-core)

- **`Dict`/`Graph` are not `Clone`** — STILL OPEN (audited, deferred with reason):
  a naive `Clone` would deep-copy the dictionary arena + six permutations (O(n))
  and is wrong for the mmap-backed mode; the designed fix is the cheap-snapshot
  API in the workspace `TODO.md` (Arc-shared immutable base), deferred as a
  structural storage change. `Graph.copy()` stays blocked on it.
- ~~**`Graph::save` panics on a sparse numeric cache**~~ **STALE — already fixed.**
  `save`/`save_compressed` recompute a dense cache from the dictionary when the
  in-memory cache is sparse (`dense_numerics`' fallback arm); the engine-seams
  wave added the pinning regression test
  (`sparq-core::tests::save_after_into_compressed_sparse_numerics`). A compressed
  mode in Python is unblocked.

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
