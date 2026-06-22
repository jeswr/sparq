# Test-coverage decomposition register (sq-bif) — [OPUS-4.8]

> 🤖 SPARQ agent register. Purpose: record the **atomic, crate-scoped, mutually-independent**
> beads the test-coverage epic **sq-bif** was decomposed into, so the autonomous scheduler
> has clean parallel material and the decomposition is not redone. This is a register, not a
> plan — the per-crate analysis lives in the beads themselves and in
> `research/coverage-and-benchmark-plan.md` / `research/coverage-bench-gap-audit.md`.

## Premise correction (honesty)

The brief framed sq-bif as "too broad for one agent — turn it into per-crate beads." Verified
against reality, that premise was **partly stale**:

- **sq-bif already has 20 children, all CLOSED** (an earlier decomposition was completed in
  waves — e.g. `sq-hbg7` stood up the gate, `sq-bif.1` seeded floors/presence for 7 untracked
  crates, `sq-bif.2/.3/.4` filled fedclient/fedplan/prov). The epic itself is still open.
- A **separate open epic `sq-qcnn`** owns "cargo-mutants ratchet + raise per-crate line floors
  to >=90%". To avoid collision, the new beads below are framed as **specific, bounded test
  deliverables** (named modules / feature-states / oracles), **not** blanket floor-raising —
  the floor rising is a side effect, re-seeded via `scripts/coverage-gate.py --seed`.

So the *real* remaining gap surface is **narrower** than "every crate": it is (a) the 3 crates
**untracked by both coverage gates**, and (b) specific **both-feature-state** / under-tested-glue
surfaces. 11 beads created (`sq-bif.6`–`sq-bif.16`), all P1–P3, all parallel-safe.

## The key parallel-safety property

Each bead touches **exactly one crate's `tests/`** (plus, where noted, that crate's own line
in `scripts/coverage.sh` / the gate JSONs / one `feature-matrix.yml` leg). No two beads edit the
same file, so any wave can run them concurrently without conflict. The `sparq-server` bead
explicitly forbids touching `src/` (the hot `http.rs` auth merge path) — tests only. **No `bd dep`
edges** were added: there is no genuine ordering between them.

## Register

| Crate | Coverage read (verified) | Gap class | Bead | Pri |
|---|---|---|---|---|
| sparq-zk-compose | floor **76** (lowest). `build.rs`/`driver.rs`/`toml.rs` = **~1621 LOC, 0 unit tests**; only the slow nargo/bb e2e suite exercises them | untested **glue** (circuit-id derivation, subprocess error, witness TOML) — NOT in-circuit soundness | `sq-bif.6` | P1 |
| sparq-policy | **untracked by both gates** + absent from `coverage.sh`; ~138 tests incl. feature-gated `count-enforcement` suite | gate-wiring + both-feature-state verification | `sq-bif.7` | P2 |
| sparq-algos | **untracked by both gates**; 22 good inline unit tests (incl. hand-computed PageRank oracle) but **no `tests/` dir** | integration suite (dangling-node PR, disconnected/chain, singleton communities) + gate-wiring | `sq-bif.8` | P2 |
| sparq-canon | **untracked by both gates**; but breadth is strong — full **86-entry W3C RDFC-1.0** suite | gate-wiring (primary) + 2 isolated unit cases | `sq-bif.9` | P2 |
| sparq-shacl | floor 90; `rules.rs` (shacl-af) not verified default-OFF; `eval.rs` dispatch partly dark (sq-qap0) | both-feature-state + named dark branches | `sq-bif.10` | P2 |
| sparq-reason | floor 82; `explain` not verified default-OFF; `incremental_explain.rs` (~1188 LOC) **0 isolated unit tests** | both-feature-state + isolated module tests | `sq-bif.11` | P2 |
| sparq-engine | floor 83; `vectorized` + `result-cache` have **no feature-matrix leg**; result-cache write-invalidation untested | feature e2e + cache-invalidation | `sq-bif.12` | P2 |
| sparq-core | floor 90; no parallel-vs-serial byte-identical differential; dict-spill spill-activation/reload edges untested | both-feature-state differential + spill edges | `sq-bif.13` | P2 |
| sparq-server | floor 86; features tested only in isolation; `test-seams` only one failure mode | feature-**composition** + multi-step failure injection (tests-only) | `sq-bif.14` | P2 |
| sparq-mpc | floor 88; `transport.rs` codec, `proof.rs` NotYetImplemented arm, `oblivious_join` determinism untested | **glue** tests (semi-honest only — NOT malicious-security) | `sq-bif.15` | P2 |
| sparq-text | floor 90; well-tested; `engine` feature not verified in pure-index state | one both-feature-state test (honesty: document if pure-index isn't a supported standalone state) | `sq-bif.16` | P3 |

## Crates judged already-covered (deliberately NOT beaded)

Assessed and found at-standard for the sq-bif mandate (a focused floor-raise, if wanted, is
**sq-qcnn**'s job, not a new sq-bif bead):

- **sparq-parse** — compression/parallel-parse paths thoroughly tested incl. mutation-pinned
  deadlock regressions.
- **sparq-solid** — WAC/ACP grant **and** deny directions tested; ODRL-bridge + count covered.
  Residual edge cases (conditional-grant refresh in isolation, multi-dimensional session binding)
  are minor; left to sq-qcnn / future follow-up rather than a dedicated sq-bif bead.
- **sparq-text** content (BM25 oracle, analyzer edges, rewrite) — only the one feature-state
  gap was beaded (`sq-bif.16`).
- The 7 crates seeded by the closed **`sq-bif.1`** (fedclient, fedplan, prov, 4 `-wasm` bundles)
  and the crates with closed sibling beads (geo `sq-9h1r`, hdt `sq-fj7a`, etc.).
- Crates correctly floor-0 / presence-gated by design (cli subprocess artifact, conformance
  test-driver, gpu device-gated, `-wasm` native-llvm-cov artifacts) — see
  `bench/coverage-floor.json` notes.

## Assessment-depth honesty

- **Deep (read src + tests directly, this session):** zk-compose glue (confirmed 0 `#[cfg(test)]`
  in build/driver/toml), algos (read all 22 test names + the d=0.85 oracle), canon (confirmed the
  86-vector assert), policy (confirmed `#![cfg(feature="count-enforcement")]` gating + gate-absence),
  the coverage gate machinery (`scripts/coverage.sh`, both gate JSONs).
- **Medium (delegated Explore agents, evidence-based, spot-checked):** engine/core feature-state
  gaps (verified `feature-matrix.yml` legs directly), shacl/reason module coverage, solid/server/mpc
  gaps. Module line-counts cited are agent-reported; treat as approximate. The `eval.rs ~58%` figure
  is from the prior `sq-qap0` audit note, not a fresh measurement here.
- **Not run:** no `cargo`/`llvm-cov` executed (too slow for a read-mostly decomposition; and any
  number measured on this work box is **non-canonical** — the CI ratchet is the record).

## Boundaries / out-of-scope (gated elsewhere)

- **In-circuit ZK/MPC soundness** testing is **out** (gated under `sq-qhy4`: external
  accredited-cryptographer sign-off PENDING; the v1 verifier is remediated + internally
  re-audited only; MPC is semi-honest-only). The zk-compose (`sq-bif.6`) and mpc (`sq-bif.15`)
  beads target **non-cryptographic glue** only and must keep any privacy/soundness mention
  caveated (the LIVE privacy-claims CI gate fails an unqualified claim).
- **Blanket floor-raise to >=90% + mutants ratchet** is **sq-qcnn**, not sq-bif.
