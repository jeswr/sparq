# STATUS — py-textindex (worktree sparq-pytext)

Recorded follow-up: the Python wrapper's full-text `TextIndex` lifecycle
(`crates/sparq-py/TODO.md`, Engine section).

## What the TODO specified

Add the `sparq-text` dependency plus a `TextIndex` lifecycle on the wrapper:
**build lazily, invalidate on every update/reason swap**. Verified context:
no other crate wires sparq-text (cli/server included — it is deliberately
standalone); the `text:` rewrite only runs through `sparq_text::query_text`.

## Done

- [x] `sparq-text` dependency on sparq-py (leaf-crate edge, sparq-reason
      precedent; engine/CLI/wasm dependency graphs stay text-free — opt-in
      consistent with native "engage by depending on the crate").
- [x] Lifecycle on `Graph`: `text: Option<TextIndex>` built lazily
      (GIL released) on first text call, cached; invalidated in `update`,
      `reason`, `reason_n3_with` (rebuild semantics — dict ids may change, so
      the native incremental `apply_delta` has no corresponding wrapper path).
- [x] `Graph.text_search(query, any=False, limit=None)` -> ranked
      `[(Term, score)]` BM25 hits; `Graph.query_text(sparql)` -> `text:`
      magic predicates (plain queries pass through);
      `Graph.build_text_index()` (eager, returns doc count);
      `Graph.drop_text_index()` (release cache; next call rebuilds).
- [x] 12 pytest cases (`tests/test_text.py`): AND/OR/prefix/limit ranking,
      language-tagged + typed-literal indexing rules, empty graph/query,
      `text:matches`/`matchesAny`/`score` joins + DESC ordering, error
      constraints, invalidation on update (insert AND delete) and on
      `reason_n3_with` (fresh dictionary — strongest staleness test),
      eager-build count + drop/lazy-rebuild equality.
- [x] CHANGELOG + README + TODO.md (item struck through, DONE).

## Gates

- pytest: test_basic.py 38 passed (baseline intact); full suite **50 passed**
  (maturin develop, dev profile, venv `.venv-py`, Python 3.14.5).
- `cargo test --workspace --exclude sparq-py --release`: see final report —
  only sparq-py (+ docs + Cargo.lock) changed; sparq-text source untouched.
- wasm untouched (no edge added anywhere near it).

## Not done / honest notes

- No `apply_delta` exposure on Python: the wrapper has no incremental update
  path (update() is a whole-graph swap), so an index `apply_delta` hook would
  have nothing to hang off — recorded rationale in lib.rs + TODO.md.
- Index covers the DEFAULT graph only (named graphs keep their own
  dictionaries) — same as native; documented in docstrings/README.
