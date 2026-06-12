# sparq-py — API gaps & follow-ups (T21)

Gaps in the wrapped crates' public APIs found while building the Python bindings.
Per the T21 scope, no existing crate source was touched; these are recorded for
their owning crates.

## Engine (sparq-engine)

- ~~**No native ASK / CONSTRUCT / DESCRIBE.**~~ **STALE — engine fixed (T16);
  bindings follow-up.** `sparq_engine` now has native `ask` (early-exiting),
  `construct`, `describe`, `construct_ntriples` (+ `_with_budget` and `_prepared`
  variants). The Python workaround is now the gap: rewire `Graph.ask` to
  `sparq_engine::ask` and route CONSTRUCT / DESCRIBE in `Graph.query` instead of
  raising `ValueError`. (Recorded for the bindings wave; the engine-seams wave does
  not touch sparq-py source.)
- ~~**Update drops named graphs.**~~ **STALE — engine fixed (update v2, F19).**
  `sparq_engine::update` / `update_in_place` model the full dataset: named graphs
  survive every operation, and GRAPH-scoped data ops, graph templates in
  DELETE/INSERT…WHERE, USING (NAMED), CLEAR/DROP/CREATE/ADD/COPY/MOVE and LOAD are
  implemented (engine tests `named_graph_updates`, `add_copy_move`). Bindings
  follow-up: drop the named-graph caveats from the Python docs/tests.

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
