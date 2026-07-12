# bench/zk-compose

Benchmarks for the ZK proof-composition layer (`crates/sparq-zk-compose` +
the `zk/compose/` Noir circuit family).

Model: Opus 4.8 (Fable 5 unavailable — flag for re-review/upgrade when Fable
returns). Numbers below were measured by Opus 4.8.

## What is measured

1. **Gate counts** (`bb gates -s ultra_honk`, ground truth per the
   noir-optimisation skill) for every compiled circuit-family member, in the
   `sparq_ieee754` JSON convention (`circuit_size`) — the shape that originated
   in the ieee754 lineage, now the `sparq-org/noir_IEEE754` face repo.
2. **Wall-clock prove/verify** for the small credential-scale e2e cases.
3. **SPARQL feature → ZK gate-cost catalog** (`sparql_feature_catalog.json`) — a
   coverage map AND an optimisation-target list spanning SPARQL 1.1, joining each
   query's `circuit_size` from the gate-count snapshot. See "SPARQL feature
   catalog" below.

## Files

| file                              | contents |
|-----------------------------------|----------|
| `gate_counts_latest.json`         | per-member ultra_honk gate counts |
| `prove_verify_timing.json`        | bb prove/verify wall-clock + proof sizes (early, 2-member, darwin) |
| `family_cost_curve.json`          | sq-pn2 full-family (k,n,r,d) prove/verify/size curve |
| `family_curve/`                   | sq-pn2 standalone timing harness (own cargo project) |
| `sparql_feature_catalog.json`     | sq-1s2.1.2 SPARQL 1.1 feature → ZK coverage + gate-cost catalog |
| `scripts/gate_counts.sh`          | regenerate the gate-count JSON |
| `scripts/prove_verify.sh`         | time prove+verify for one member |
| `scripts/sparql_catalog.py`       | regenerate the SPARQL feature catalog (joins the snapshot) |

> The gate-count JSON is also the source the in-crate **regression gate**
> (`crates/sparq-zk-compose/tests/gate_count.rs`, sq-c5f) baselines against —
> re-run `scripts/gate_counts.sh` after an intentional circuit change and update
> both that JSON and `crates/sparq-zk-compose/tests/gate_count_snapshot.json`.

## sq-1s2.1.2: SPARQL feature → ZK gate-cost catalog

`sparql_feature_catalog.json` is a **comprehensive SPARQL 1.1 benchmark catalog**
that doubles as (i) a **ZK-coverage map** — for each SPARQL feature, which circuit
member(s) (if any) it compiles to today — and (ii) an **optimisation-target list**
— per-member `circuit_size` with the high-gate members flagged as reduction
targets. It implements the §6 catalog design of
`research/zk-field-native-encoding.md`.

The catalog spans the full feature surface: BGP (single / multi-pattern), every
numeric-FILTER datatype + operator (the complex filters: integer, signed integer,
decimal, double), boolean / string / dateTime FILTER, OPTIONAL, UNION, **property
paths including `+` / `*` / `?` / `/` / `|` / `^`**, aggregates / GROUP BY,
subqueries, BIND, VALUES, MINUS / negation, and the hidden-credential primitives
(revocation, issuer attestation, holder possession).

**Honesty is load-bearing.** Each query records its coverage status:

- `covered` — compiles to a real circuit member; its `circuit_size` is **joined
  from** `crates/sparq-zk-compose/tests/gate_count_snapshot.json` (the
  regression-gated source of truth), never hand-typed, so it can never drift.
- `partial (…)` — composed verifier-side or desugared to a covered primitive
  (e.g. UNION, OPTIONAL, alternative paths); `circuit_size: null`.
- `NO ZK CIRCUIT YET (gap)` — the feature has **no circuit today** (general
  property-path traversal `+`/`*`/`?`, aggregates, BIND expression eval, string
  predicates, dateTime compare, negation, boolean FILTER). These carry
  `circuit_size: null`. **A gate number is NEVER fabricated for a gap.**

High-gate covered members are auto-flagged: `HIGH_GATE_blake3_binding` marks the
numeric-FILTER family (the value-hook reduction target of §2.6 — the blake3
token-binding the encoding overhaul removes), and `HIGH_GATE_lattice` marks a
scan/join member that is large because of the (k,n,r)/(na,nb) lattice corner.

### Regenerate

```sh
bench/zk-compose/scripts/sparql_catalog.py > bench/zk-compose/sparql_feature_catalog.json
```

The generator needs **no** `nargo`/`bb` (it reads the committed snapshot), so it
is deterministic. The committed JSON is gated by
`crates/sparq-zk-compose/tests/sparql_catalog.rs`, which fails if a covered query
names a member absent from the snapshot, if a covered `circuit_size` disagrees
with the snapshot, or if a gap carries a non-null gate number. After an
**intentional** circuit change, re-run `scripts/gate_counts.sh` (re-baseline the
snapshot) and then re-run the catalog generator to re-join the new numbers.

> This file is the **DATA layer**. The website surface that renders it (the
> SPARQL → ZK coverage + gate-cost table on `/benchmarks/zk`) landed in sq-1s2.1.3
> (#777): `site/scripts/sync-zk-catalog.mjs` copies this JSON into
> `site/src/data/zk-sparql-catalog.generated.json`, which the
> `ZkSparqlCatalog` component (`site/src/components/benchmarks/zk-sparql-catalog.tsx`)
> renders — so the rendered numbers always trace back to this snapshot-joined data.

## sq-pn2: full-family prove/verify cost curve

`family_curve/` is a STANDALONE cargo project (own `[workspace]`, same isolation
pattern as `bench/zk`) that drives the `sparq-zk-compose` prover (nargo + bb
subprocesses) once per circuit-family member and emits the full **(k, n, r, d)
cost curve** — prove time, verify time, proof size, vk size. CI **builds** the
harness (a compile-only schema-drift guard, see the blockquote below)
but never **runs** it: each row is a real `bb prove` (~1-2 s) so the curve is
generated manually, not in CI. It is a plain timing harness, not criterion
(criterion's repeated sampling over ~1-2 s proofs would take hours).

`k` = committed graphs (disclosed-attribution width), `n` = slot bucket, `r` =
disclosed-row bucket (the scan family); `d` = digit count (the filter families).
The scan sizes are chosen to land on each compiled member; both filter families
sweep `d ∈ {1,2,4}`. As of sq-q7e/sq-tat the `filter_f64` member is **manifest-
composable** (it carries `d` in its `CircuitId::FilterF64 { d }`, has a real
`ProofInputs::FilterF64`, and renders its `Prover.toml` via `prover_toml_for`
from a canonical decimal-digit witness, exactly like `filter_int`), so the
harness drives it through that composable path rather than the old hand-written
raw `Prover.toml`. ([OPUS-4.8] sq-kep2 ported the harness to that schema.)

> **Schema-drift guard (sq-kep2).** Because `family_curve` is a STANDALONE cargo
> project (own `[workspace]`, see below) it is NOT part of the root workspace and
> so is **not** covered by the `cargo build --workspace` CI gate (ci.yml) — that
> is why a `CircuitId`/`ProofInputs`/`prover_toml_for` signature change in
> `sparq-zk-compose` (such as `FilterF64` going unit→struct) once broke it
> silently. It is now rebuilt on every ZK-touching PR by a lightweight
> `cargo build` step in `.github/workflows/zk-toolchain.yml` (a lane that already
> triggers on `crates/sparq-zk-compose/**`), so such drift is a visible red check.
> **When you change a `CircuitId`/`ProofInputs` variant or `prover_toml_for`,
> update this harness too.**
>
> The CI build is deliberately **not** `--locked` ([FABLE-5] sq-q134e): the
> harness path-deps on in-repo crates whose manifests bump dependencies on lanes
> that never run this guard, so the committed `family_curve/Cargo.lock` drifts
> silently on main and `--locked` then spuriously failed the *next* zk-touching
> PR (#1962, a main-side `hashbrown` bump). A plain `cargo build` keeps every
> still-compatible pin from the committed lock (cargo re-resolves minimally) and
> only absorbs the in-repo manifest bumps; if the lock did drift, CI emits an
> advisory notice + lock diff asking for a refresh commit instead of failing.

### Run

```sh
cd bench/zk-compose/family_curve
cargo run --release > ../family_cost_curve.json   # JSON to stdout, table to stderr
# average each member over N proves (reduces noise):
REPEATS=3 cargo run --release > ../family_cost_curve.json
```

Requires `nargo` + `bb` on PATH (it exits with an error if absent). Writes its bb
scratch under the system temp dir and cleans it per member.

### Full-family prove/verify/proof-size results

The per-member prove/verify wall-clock, proof bytes, and vk bytes are committed as
machine-readable JSON — `bench/zk-compose/family_cost_curve.json` (the consistent
full-family measurement, the authority for verify timing) — and regenerated by the
scripts below. Read the JSON for current figures rather than a baked table here.

Observations (from the committed measurements):
- **Prove time tracks gate count**: the scan members (largest gate counts) are
  the slowest; the `filter_*` members the fastest. The
  `filter_int_d{1,2,4}` members are gate-identical (17,416) and prove in the
  same ~1.06 s — `d` does not move cost (see the gate-count notes above).
- **proof size, vk size, and verify time are CONSTANT across the family**
  (14,656 B / 3,680 B / ~12 ms) — the ultra_honk succinctness property: a
  constant-size proof and a verify cost dominated by the fixed protocol, not the
  circuit. (This differs from the earlier 2-member `prove_verify_timing.json`,
  which reported a ~0.95 s scan verify on darwin/8-thread; on this aarch64 box
  verify is uniformly ~12 ms for every member. The earlier figure looks like a
  cold-vk / measurement artefact — `family_cost_curve.json` is the consistent
  full-family measurement and supersedes it for verify timing.)

## Reproduce

```sh
# gate counts (compiles the workspace, emits JSON to stdout)
bench/zk-compose/scripts/gate_counts.sh > bench/zk-compose/gate_counts_latest.json

# prove/verify timing for one member (needs a Prover.toml; the crate e2e tests
# leave real ones behind, or write one by hand)
bench/zk-compose/scripts/prove_verify.sh filter_int_d1
```

Toolchain: nargo 1.0.0-beta.21, bb 5.0.0-nightly.20260324.

## Gate counts and timing — committed JSON

### Gate counts (ultra_honk `circuit_size`)

Per-member gate counts are committed as `bench/zk-compose/gate_counts_latest.json`
(regenerated by `gate_counts.sh`, above) and regression-gated by
`crates/sparq-zk-compose/tests/gate_count_snapshot.json`. Read those for current
figures. The qualitative shape:

Notes:
- The scan members scale roughly linearly in `k * n` (the commitment-recompute
  sweep + the completeness double-loop dominate); `r` adds the row-soundness
  pass.
- `filter_int_d{1,2,4}` are **identical in gate count** (17,416): the blake3
  blackbox over the canonical token is the cost driver, and the token fits one
  64-byte blake3 block for all `d <= 19`, so digit count does not move the
  circuit size. The `d` family parameter exists only because the blackbox
  needs a comptime byte length; it leaks `ceil(log10(value))`, not gates.
- `filter_f64` was originally the cheapest member — a pure `sparq_ieee754`
  comparison, no string hashing — as a raw building block. As of sq-q7e/sq-tat it
  is **manifest-composable** (`filter_f64_d{D}`), binding the operand via the same
  canonical decimal-digit token as `filter_int`; read `gate_counts_latest.json`
  for the current per-`D` figures.

### Prove / verify wall-clock (small e2e)

The two-member e2e prove/verify timing is committed as
`bench/zk-compose/prove_verify_timing.json` (note: `family_cost_curve.json` above is
the fuller, more consistent measurement and supersedes it for verify timing).
`prove` includes `--write_vk`. Proof size is constant (ultra_honk) regardless of
circuit size; verify time grows with public-input count (scan carries the commitment
+ rows vectors, hence the higher verify cost).
