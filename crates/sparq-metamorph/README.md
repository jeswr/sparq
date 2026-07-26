# sparq-metamorph

Metamorphic + differential **logic-bug harness for SPARQL engines** — TLP and NoREC
re-derived for SPARQL's three-valued (true / false / **error**) effective-boolean-value
semantics, a cross-engine differential oracle over
[`sparq-difftest`](../sparq-difftest/README.md), a seeded deterministic case generator,
SPARQL-protocol drivers, and a machine-checked found-bug ledger. Bead `sq-gum8.6`
(paper P2a); the research core is the `tlp` module's error-semantics case analysis.

**Internal tooling — not published** (`publish = false`): nothing in the shipping graph
depends on it, so the lean core/engine/wasm builds never link it. The bug-hunting
campaign runs **off-CI** against live endpoints; CI runs network-free self-tests only.

## 🚀 Quickstart

```rust
use sparq_metamorph::{check_tlp, check_norec, generate_case, InProcessSparq};

let case = generate_case(42); // seed in, identical case out, forever
let engine = InProcessSparq::from_ntriples("sparq", &case.data_ntriples)?;
let verdict = check_tlp(&engine, &case.pattern, &case.predicate);
assert!(verdict.is_pass()); // Pass | Violation (wrong result) | EngineFailure
# Ok::<(), sparq_metamorph::EngineFailure>(())
```

External engines via the `protocol-drivers` cargo feature (off by default; pulls
`ureq`): build an `EndpointConfig` preset — `fuseki`, `oxigraph`, `virtuoso`,
`blazegraph`, `graphdb`, `qlever`, `millenniumdb`, or `generic` for anything else —
wrap it in an `HttpSparqlEngine`, and pass it to `check_differential` alongside sparq.
Each preset carries a `PresetEvidence` recording whether its URL/method/negotiation
conventions were confirmed against a running instance or encoded from upstream
documentation, so a misconfigured endpoint cannot masquerade as a found bug;
`tests/preset_live_conformance.rs` is the opt-in, off-CI probe that checks one against
a live endpoint.

## ✨ Features

- **TLP for SPARQL** (`tlp`): partitions `SELECT * { P }` on `FILTER(c)` /
  `FILTER(!c)` / `FILTER(COALESCE(IF(c, false, false), true))` — the third branch
  *reifies* the evaluation-**error** outcome (type errors, unbound variables), which
  SPARQL, unlike SQL's `NULL`, cannot test as a value. Multiset union must equal the
  base. Full spec-cited case analysis in the module docs.
- **NoREC for SPARQL** (`norec`): moves the predicate into projection position
  (`SELECT (IF(c, true, false) AS ?flag)`), off the filter-pushdown/index paths, and
  cross-checks cardinalities harness-side.
- **Differential oracle** (`differential`): same query on ≥2 engines, compared through
  sparq-difftest's engine-independent multiset comparators (dependency only).
- **Strict verdicts** (`verdict`): wrong-result violations vs engine failures are never
  conflated; every oracle fails closed on an engine error.
- **Seeded generator** (`generate`): in-crate SplitMix64, no wall clock or OS
  randomness — a ledger seed reproduces its case bit-for-bit per generator version
  (entries also carry the reduced query + data inline).
- **Found-bug ledger** (`ledger`): JSONL entries **require** an upstream issue URL and
  a developer-confirmation status; unfiled observations are not entries.
- **Non-vacuity self-tests**: a deliberately-injected wrong-result mutant
  (`FilterDropsRow`) is flagged by all three oracles against the *real* sparq engine.
- **Nightly CI driver** (`harness` + the `metamorph-driver` binary): seeded window ->
  TLP + NoREC verdicts per seed, every verdict counted (fail-closed), deterministic
  repro on failure. Driven by `.github/workflows/metamorph.yml` (advancing nightly
  window + fixed smoke window, auto-filed findings). Red path on demand:
  `metamorph-driver <start> <count> --inject-filter-drops-row`.

## 📚 Learn more

- Rustdoc module docs — the TLP three-valued case analysis (`tlp`), the NoREC
  honesty note on "non-optimizable", and the ledger schema table (`ledger`).
- `skills/academic-paper/SKILL.md` — the paper factory this instrument feeds
  (paper draft bead `sq-gum8.7`).
- Prior art: SQLancer's TLP (OOPSLA 2020) and NoREC (ESEC/FSE 2020) for SQL;
  GDsmith/Gamera for Cypher/Gremlin. No dedicated SPARQL logic-bug harness predates
  this crate (closest: SparqLog's incidental wrong-result reports, 2023).

## License

[MIT](../../LICENSE).
