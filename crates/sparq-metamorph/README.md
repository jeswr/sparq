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
Each config carries a `PresetEvidence` recording what backs its URL/method/negotiation
conventions, so a misconfigured endpoint cannot masquerade as a found bug. Presets are
documentation-derived (`UpstreamDocs`) — building one contacts nothing; only
`confirmed_live()`, called after a real exchange with that very endpoint, records
`LiveInstance`, which is what the opt-in, off-CI `tests/preset_live_conformance.rs`
probe does.

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
- **`DISTINCT` variant** (`distinct`): the same partition recombined under **set**
  semantics — `D(π(Ω))` is the set union of the deduplicated branches, not their multiset
  union, because two solutions in different branches may project to the same row. With
  `SELECT DISTINCT *` the branches are provably disjoint and the stronger multiset law is
  checked as well. Exercises the duplicate-elimination path the base oracle never touches.
- **Aggregate partitioning** (`aggregate`): `COUNT(*)`/`COUNT(e)`/`SUM(e)` over the three
  branches must recombine to the base — an additive law plus an **error-status** law
  (the base aggregate is unbound exactly when some branch's is), derived to be invariant
  under both readings of SPARQL's aggregate error semantics. `SUM` carries an exactness
  precondition (integer-valued expression) because a promoted floating-point fold is not
  associative; a promoted cell is reported fail-closed as a harness failure.
- **`ORDER BY` differential mode** (`differential::check_differential_ordered`): compares
  ordered results up to permutation within each sort-key equivalence class
  (`sparq_difftest::order_by_equal`), since SPARQL §15.1 specifies the result sequence only
  partially. Catches a reordering the unordered oracle is structurally blind to, and does
  *not* flag the reordering the spec permits.
- **`EXISTS` stays excluded** from every oracle: its substitution semantics is a known
  SPARQL 1.1 defect under revision for SPARQL 1.2, so a "violation" involving it would
  measure the standard rather than an engine. Revisit when SPARQL 1.2 settles it.
- **Strict verdicts** (`verdict`): wrong-result violations vs engine failures are never
  conflated; every oracle fails closed on an engine error.
- **Seeded generator** (`generate`): in-crate SplitMix64, no wall clock or OS
  randomness — a ledger seed reproduces its case bit-for-bit per generator version
  (entries also carry the reduced query + data inline).
- **Found-bug ledger** (`ledger`): JSONL entries **require** an upstream issue URL and
  a developer-confirmation status; unfiled observations are not entries.
- **Non-vacuity self-tests**: a deliberately-injected wrong-result mutant
  (`FilterDropsRow`) is flagged by every oracle against the *real* sparq engine. Because
  it perturbs cardinality only, each extension carries a mutant aimed at the law it adds
  (an off-by-one aggregate cell, a silently-unbound aggregate, a reversed result order).
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
