<!-- [FABLE-5] sq-p3ssl — native in-process rdf-canon column for the RDFC-1.0
canonicalization panel. First-read only; work-box readings are NON-canonical by
construction and are never transcribed here. -->

# Gap record — native in-process rdf-canon column (canon panel, 2026-07)

**Axis:** RDFC-1.0 canonicalization, bridge-overhead sub-axis of the `canon-bench`
panel (epic `sq-hmd7l`; parent record `research/gap-canon-2026-07.md`).
**Status:** DELIVERED — `crates/sparq-canon/examples/canon_compare.rs`; smoke green.
**Bead:** `sq-p3ssl`.

## 1. The gap this closes

`bench/canon/run.sh` already carries a Rust `rdf-canon` column, but drives it
through a **gather-time scratch CLI subprocess**
(`scripts/bench-adapters/canon_adapter.sh --build`), so that column's wall
readings include process spawn and its `canon_us` crosses a process boundary
from a separately-built binary. There was no **native in-process** column that
runs sparq-canon and the `rdf-canon` crate over the *same parsed fixtures in
one process* — the cleanest possible isolation of what sparq adds on top of the
algorithm crate it delegates to.

`canon_compare` closes that: both implementations canonicalize the same
committed conformance graphs (the vendored W3C rdf-canon suite snapshot,
`crates/sparq-canon/tests/rdf-canon-testdata/`) in a single process:

| column | path exercised |
|---|---|
| `sparq` | public `sparq-canon` API — oxrdf-0.3 quads → **oxrdf-0.3↔0.2 bridge** → `rdf_canon` |
| `rdf-canon` | `rdf_canon::canonicalize_quads[_with::<Sha384>]` over oxrdf-0.2 quads parsed directly from the fixture bytes — **no bridge** |

The per-fixture delta between the columns is therefore the bridge +
guard-configuration overhead and nothing else — same process, same allocator
state, same parsed-input discipline (each fixture is parsed once per term
model before any timing; timing covers canonicalization only).

## 2. Honest scope (unchanged from the parent record)

- **NOT an independent-implementation comparison.** `sparq-canon` delegates its
  RDFC-1.0 algorithm to this same `rdf-canon` crate at the same lockfile pin;
  the independent cross-implementation check remains the JS `rdf-canonize`
  column of `bench/canon/run.sh`.
- **Bytes before stopwatch (the sq-p3ssl invariant).** No timing row is emitted
  unless BOTH implementations reproduce the vendored W3C expected canonical
  N-Quads **byte-for-byte on every sane eval fixture** (the exact-image
  RDFC-1.0 oracle; oracle equality on both sides implies pairwise equality). A
  single mismatch suppresses ALL timing rows and reds the run (exit 2).
- **Poison outcomes are results, never wins.** The suite's `poison – evil`
  evals and the negative 10-node clique run under a per-canonicalization soft
  wall-clock cap (`--cap-s` / `CANON_CAP_S`) on a worker thread; outcomes use
  the panel vocabulary (`ok` / `guard` / `capped` / `wrong` / `accepted`) and
  go to stderr + the envelope — a capped or guarded run never appears as a
  timing row. `wrong` and `accepted` red the run (soundness).
- **Work-box timings are NON-canonical** (`bench/CATALOG.md` QUIET-BOX); the
  example emits runtime-only numbers (TSV rows + a `CANON_COMPARE_JSON`
  envelope) and nothing is committed.

## 3. Supply chain

**No new dependency.** `rdf-canon` (and the oxrdf-0.2/oxttl-0.1 pair the
native column parses with) are already regular dependencies of `sparq-canon`
— the crate's own algorithm delegation — and `rdf-canon 0.15.3` is already
attested in `supply-chain/config.toml`. The bead's "attest the new dev-dep"
note turned out to be moot: the example only reuses existing deps. The
envelope records the `rdf-canon` pin by reading the crate manifest at runtime
(never hard-coded).

## 4. Run it

```sh
# acceptance (the sq-p3ssl gate): full sane-set equality gate for BOTH
# implementations + the poison/negative outcome panel; exit 0 = green
cargo run -p sparq-canon --release --example canon_compare -- --smoke

# full mode: min-of-N timing rows behind the gate + optional envelope file
cargo run -p sparq-canon --release --example canon_compare -- \
  --iters 5 --json-out bench/competitor-results/canon-compare.json
```

stdout contract: `<workload>\t<count>\t<us>` rows (`<testid>/<impl>` rows with
`count` = quads; `sane-total/<impl>` rows with `count` = fixtures), then a
single-line `CANON_COMPARE_JSON {…}` envelope (the `MEMBPT_JSON` precedent).

## 5. Follow-ups

- A canonical quiet-box gather (parent record §3) can now include the
  in-process bridge-overhead pair alongside the three subprocess columns; the
  `rdf-canon-rust` entry in `bench/competitors.json` (owned by `sq-hmd7l.16`/
  `.41`) stays the subprocess column of record until then.
- Upstream-drift mode (a *different* `rdf-canon` version in-process) is not
  possible in one process (one crate version per build); that comparison
  remains the scratch-CLI's `CANON_RDF_CANON_VERSION` override.
