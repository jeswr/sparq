# RIF conformance — W3C RIF suite pass-rate runner (`rif-conformance`)

> 🤖 SPARQ agent. [FABLE-5] sq-hmd7l.24 (epic sq-hmd7l).

**The RIF axis verdict is NOT-COMPARABLE** (codified in
[`research/gap-rif-2026-07.md`](../../research/gap-rif-2026-07.md)): no runnable
open-source RIF peer exists to race — fuxi is dead, Jena dropped RIF, RIFle is
unmaintained, and RDFox consumes datalog, not RIF. This axis is therefore
**conformance-only**: it emits a W3C RIF test-suite pass-rate (pass/fail/skip
counts) and **no wall-clock**. Performance of RIF-translated rules is measured on
the materialization axis (sq-hmd7l.7), never here. The pass-rate runner is what
BACKS (rather than asserts) the "only maintained open-source RIF consumer" claim.

## Invoke

```sh
bench/rif/run.sh --smoke   # self-asserting vendored subset; exit 1 on any drift
bench/rif/run.sh           # full per-dialect pass-rate over the FETCHED W3C archive
```

Both wrap `cargo run -p sparq-reason --release --example rif_conformance
--features rif-xml` — the feature flag is required (RIF is opt-in; the default
build carries zero RIF code).

## The two lanes

- **`--smoke`** runs the **vendored subset** in `suite/` and asserts every case's
  verdict EXACTLY against the pinned [`expected.tsv`](./expected.tsv) — pinned
  SKIPs included (they pin the vacuity guards). **The vendored cases are
  sparq-AUTHORED**, in the W3C archive's directory taxonomy
  (`<TestType>/<id>/<id>-{premise,conclusion,nonconclusion,input}.rif`): the real
  W3C RIF files carry no redistribution license — even a subset is a prohibited
  derivative (license gate in `scripts/fetch-inference-suites.sh`).
- **Full mode** walks the fetched archive under `tests/w3c/rif-core/`
  (`scripts/fetch-inference-suites.sh`; gitignored) and prints a **per-dialect**
  breakdown (`TOTAL rif-core pass N fail N skip N of N`), the named skip taxonomy
  (the denominator's honesty) and an itemised fail listing — failures are listed,
  never rounded up. Absent archive ⇒ graceful skip, exit 0. Output is teed to the
  gitignored `results.tsv`.

## Oracle

A condensed restatement of the `rif-wg-core` lane
(`crates/sparq-conformance/tests/rif_wg_core_suite.rs` — the authoritative,
ratchet-carrying copy): import → validate → closure → conclusion satisfaction;
the NET vacuity rule (an un-importable premise is a SKIP, never a vacuous pass);
and the reject-polarity guard (a rejection merely because a construct is
unsupported never counts as "correctly rejected"). That test lane owns the Core
pass-count RATCHET (`RIF_WG_CORE_FLOOR`); this harness owns the pass-RATE
emission and the registry surface.
