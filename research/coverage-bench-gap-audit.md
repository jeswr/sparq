# Coverage + Benchmark Gap Audit (sq-bif / sq-5o5)

> 🤖 SPARQ agent — read-only audit. Authored by Opus 4.8 (Fable unavailable; flag for
> re-review when Fable returns). `[OPUS-4.8]`

Scope: a READ-ONLY gap audit feeding the two coverage epics — **sq-bif** (comprehensive
correctness tests for all packages/features) and **sq-5o5** (full benchmark coverage).
Audited via the repo's own signals (the coverage ratchet, the test-presence gate, the
feature-matrix lane, and the benchmark registry) — NOT by running the heavy
`cargo llvm-cov` suite (this box is non-canonical; the % gate is slow). Empirically honest:
both epics are **already heavily executed** — most children are closed and the registry is
comprehensive. The headline is "substantially covered; one genuine new gap."

## Method — the signals I read (not re-ran)

- **Coverage ratchet** `bench/coverage-floor.json` (per-crate line-% floors, only-rises) +
  driver `scripts/coverage-gate.py` / `scripts/coverage.sh`.
- **Test-presence gate** `bench/coverage-presence.json` + `scripts/coverage-presence.py`
  (per-crate `#[test]` count floors + integration-dir presence; catches a crate losing
  its tests where the % gate would not).
- **Feature-matrix lane** `.github/workflows/feature-matrix.yml` (one CI leg per opt-in
  feature group — the ONLY place feature-gated code/tests are compiled, since the main
  `cargo nextest archive --workspace --all-targets` job runs with **no `--all-features`**).
- **Benchmark registry** `bench/benchmarks.toml` (every `[[benchmark]]`) + the G1 new-crate
  gate `scripts/gate-new-crate.py` (a new crate must register a bench or be `publish=false`).
- In-code `#[ignore]` / `TODO test` markers; `bd` epic/child status for sq-bif + sq-5o5.

## Per-crate / per-surface verdict

The coverage ratchet tracks **24 crates** with non-trivial floors (sparq-engine ≥83% over
238 tests, sparq-core ≥78/67, sparq-server ≥85/107, sparq-shacl ≥90/103, sparq-zk ≥88/79,
sparq-zk-compose ≥76/147, sparq-reason ≥81/127, etc.). `sparq-cli`/`sparq-conformance`/
`sparq-gpu` carry floor-0 with documented line-% artifacts and are **presence-gated** instead
(subprocess / test-driver / no-device). `sparq-bench`/`sparq-py` are harness/binding crates
(presence floor 0, by design). **Every crate that ships code is in at least one gate** — no
crate is silently untracked.

The benchmark registry is comprehensive: well-known suites SP2Bench, DBPSB/FEASIBLE, WatDiv,
BSBM, LUBM (extensional+entailed), the operator-coverage `.rq` suite, SHACL/FTS/vector/geo
capability suites, ZK/MPC, ingest/parse/compression/scaling, and W3C conformance are all
registered and (mostly) wired into `scripts/ci-bench.sh`. The G1 gate enforces a registered
bench (or `publish=false`) for every new crate.

**Most prior gaps are already beaded and closed.** sq-bif's children (per-builtin error-path
table, serializer oracle, UPDATE atomicity, SHACL property-path, HDT differential, MPC
adversarial, RDF1.2/rdf-star, the coverage gate itself, …) are nearly all `✓`. sq-5o5's
suites (WatDiv/BSBM/SP2Bench/DBPSB/LUBM, operator-coverage, dashboard, vectors-throughput,
py FFI micro-bench) are nearly all `✓`. The open children are already-scoped follow-ups
(below) — I did NOT re-bead them.

## Findings

### 1. GENUINE NEW GAP — `sparq-server` `audit-log` feature has tests, but CI never runs them

`sparq-server` declares an opt-in, default-OFF `audit-log` feature (compliance surface:
CDMC CD-2 / ISO 27001 A.8.15 / EU CRA logging). Its code is entirely behind the feature:
`crates/sparq-server/src/audit.rs` is `#[cfg(feature = "audit-log")]` (carries unit tests),
and `crates/sparq-server/tests/audit_log.rs` is `#![cfg(feature = "audit-log")]` (2
`#[tokio::test]` integration cases asserting allowed/denied audit records, no-leak of query
text / Bearer token).

But **nothing in CI enables `audit-log`**:

- The main test job runs `cargo nextest archive --workspace --all-targets` with no
  `--all-features` (`.github/workflows/ci.yml`) → the gated module + integration test are
  compiled out.
- `scripts/coverage.sh` only adds `--features mmap,dict-spill` (for sparq-core); sparq-server
  is measured with default features → the `audit-log` code is invisible to the % ratchet too.
- `.github/workflows/feature-matrix.yml` has legs for sparq-server `federation-descriptors,
  service,time-travel,geo,test-seams` — but **no `audit-log` leg** (only a header mention of
  unrelated `live`/`embeddings` exclusions). So the audit log can silently break.

This is the exact failure mode the feature-matrix lane exists to prevent, and it is a
compliance-relevant surface. **Priority 2.** (Companion to — but NOT covered by — the existing
`sq-wuft`, which only adds `filtered-ann` + `shacl-af` legs.)

### 2. ALREADY BEADED — `filtered-ann` + `shacl-af` feature-matrix legs (`sq-wuft`, open)

`sparq-vectors/filtered-ann` (dedicated `tests/filtered.rs`, `#![cfg(feature="filtered-ann")]`)
and `sparq-shacl/shacl-af` merged after the feature-matrix lane's branch point and are not yet
legs — same class of gap as #1, but **already captured by `sq-wuft`** (P2). No new bead; the
audit corroborates it and notes audit-log belongs in the same lane fix.

### 3. ALREADY BEADED — open sq-5o5 / sq-bif follow-ups (no new beads)

Verified open and correctly scoped; not re-beaded:
- `sq-goay` (sq-bif): GPU CPU-vs-GPU differential oracle (skipped when no device).
- `sq-cvwl`, `sq-5pnl`, `sq-i8jn`, `sq-zdv9` (sq-5o5): shacl-bench example, wasm query
  micro-bench, deterministic serve/server micro-metric, FedBench (blocked on SERVICE).
- `sq-v4if`: wire the four GenAI crates into the nightly perf gates.

### 4. NOT A GAP — stale empty scratch dirs `crates/sparq-fedplan`, `crates/sparq-prov`

Both directories exist on disk but are **empty and git-untracked** (no `Cargo.toml`, no `src/`,
`git ls-files` returns nothing). They are not workspace members and not real crates — local
scaffolding, not a coverage gap. The brief's other named "features" (tpf, odrl-bridge) do not
exist under those names; the real opt-in surfaces are the ones in the feature-matrix +
`filtered-ann`/`audit-log`/`shacl-af`. Worth a `git clean` of the worktree eventually, but no
bead.

### 5. NOT GAPS — verified deliberate exclusions

- `sparq-nlq/live` and `sparq-vectors/embeddings`: real-network features, **documented**
  feature-matrix exclusions (no offline test possible). `src/live.rs` having 0 `#[test]`s is by
  design.
- `#[ignore]`d tests (vectors throughput, zk-compose heavy gate/forge/e2e, engine flat-read
  bench, server updates, shacl diff_fuzz, …): intentional heavy/nightly/manual tiers, several
  already registered as benches (e.g. vectors-throughput → `sq-cu8c ✓`). Not gaps.

## Bottom line

Coverage + benchmark programs are **substantially complete**. One genuine NEW gap (audit-log
feature uncovered in CI). One adjacent gap already beaded (`sq-wuft`). Everything else is
either already-closed, already-beaded-and-open, deliberate, or stale scratch. No manufactured
gaps.
