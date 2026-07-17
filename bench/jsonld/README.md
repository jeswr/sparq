<!-- [FABLE-5] sq-hmd7l.15 — the jsonld-bench comparative suite (epic sq-hmd7l). -->
# bench/jsonld — JSON-LD comparison (conformance table + gated throughput)

Two comparable dimensions for sparq's JSON-LD 1.1 processing vs **jsonld.js** and
**titanium-json-ld** (both registered in [`bench/competitors.json`](../competitors.json)):

1. **Conformance pass-rate table** — sparq's MEASURED W3C suite counts vs the peers'
   PUBLISHED results, pinned with provenance URL + date (never estimated):
   [`conformance-peers.json`](./conformance-peers.json), rendered with the
   comparability caveats in
   [`research/gap-jsonld-2026-07.md`](../../research/gap-jsonld-2026-07.md).
2. **Throughput** — expand / flatten / compact / toRdf docs/s + MB/s on the vendored
   fixtures (WebDataCommons-shaped schema.org docs + a ~100-term context-heavy doc;
   synthetic — real WDC snippets are not license-clean to vendor).

**INVARIANT (load-bearing): no throughput row without output-equality agreement.**
Every engine's output must pass, per (fixture, op), *before* its timing is emitted:
expand → deep-equality under the conformance comparator (`json_ld_equal` — the SAME
comparator the W3C ratchet trusts, `sparq_conformance::jsonld_bench`); flatten/toRdf →
canonical-RDF-dataset equality (blank-node-blind); compact → round-trip losslessness
(`reparse(compact(D, ctx)) ≡ D`). Failures are recorded exclusions, never silent drops.

## Run it

```sh
cargo build --release -p sparq-conformance --features jsonld-suite --example bench_jsonld
bash bench/jsonld/run.sh --smoke     # offline, sparq-only, self-asserting (exit 1 on drift)
bash bench/jsonld/run.sh --gather    # + peers (gather-only, never committed deps)
```

`--smoke` asserts: expand deep-equality vs the vendored jsonld.js-generated
expectations, flatten/toRdf dataset-equality, compact losslessness, the deterministic
anchors in [`expected.tsv`](./expected.tsv), and a NEGATIVE self-test (the comparator
must be able to say NO — the gate is proven non-vacuous on every run).

`--gather` (delegates to [`gather.py`](./gather.py)) needs: `npm install jsonld` in a
scratch dir (`--node-path <its node_modules>`), and optionally `TITANIUM_CP` pointing
at the five pinned jars (see [`scripts/bench-adapters/jsonld_adapter.py`](../../scripts/bench-adapters/jsonld_adapter.py)).
Envelopes land in `bench/competitor-results/` (git-ignored) with env metadata; all
timings are advisory wall-clock — work-box numbers are NON-canonical and never
transcribed into committed markdown.

## Honest caveats

- **compact compares different pipelines at the same task**: sparq = toRdf +
  RDF-writer compaction (`graph_to_jsonld_compact`); peers = the W3C document-level
  Compaction Algorithm. The shared gate is round-trip dataset losslessness; every
  emitted compact row carries this caveat in its envelope.
- **Expected outputs' provenance**: `fixtures/expected/*` were generated with
  jsonld.js 9.0.0 (npm, Node v20) on 2026-07-11 and cross-checked three ways at
  authoring time (sparq ≡ jsonld.js ≡ titanium-json-ld 1.7.0 on every fixture × op).
- Flattened expectations are compared dataset-level, not deep-equality: engines mint
  different (both spec-valid) blank-node labels for anonymous nodes.
- Conformance table caveats (denominator drift across suite snapshots; sparq's
  compact/frame use a round-trip oracle): see `conformance-peers.json` `_comment`.

## License

MIT (workspace license; fixture documents are synthetic and carry no upstream text).
