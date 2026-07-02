# Test-quality program plan — prioritized per-crate coverage + mutation decomposition (sq-qcnn)

> 🤖 SPARQ agent design record (architect stage, Claude Fable 5). Decomposition of epic
> **sq-qcnn** ("cargo-mutants ratchet + raise per-crate line-coverage floors to ≥90%
> core-logic") into a prioritized, **disjoint** (one crate per bead) implementation plan the
> fleet can drain in parallel. DESIGN + DECOMPOSE only — no tests are implemented here.
> Companion records: `research/test-coverage-decomposition.md` (the earlier sq-bif register),
> `research/differential-testing-value-level.md` (the sq-qcnn.2 diff-test lane — **disjoint**
> from this plan; its open children sq-qcnn.5–.10 are not duplicated here).

## Honesty framing (read first)

- A **line-coverage floor is not a correctness proof.** It proves a line *ran*, not that any
  test would notice if its behaviour changed. `bench/coverage-floor.json` floors are
  `floor(measured) − 2` — so the epic's "floor ≥ 90" target means **measured ≥ 92%**.
- The **mutation ratchet is the real test-quality signal** (`bench/mutants-baseline.json` +
  `scripts/mutants-gate.py`): a surviving mutant is code a test executed but never asserted
  on. The gate is the **surviving COUNT** (stable), not caught% (moving denominator).
- Both are still proxies. The strongest semantic oracle is the value-level differential lane
  (sq-qcnn.2/.4 done, .5–.10 open) — this plan complements it, it does not replace it.
- Prior finding (bit us at #1250): thin public wrappers reached only *indirectly* sit ~0%
  line-covered and drag a crate below floor. **Every bead here carries the
  direct-unit-test-per-public-fn discipline** (§ Discipline below).

## Measured state (2026-07-02; live sources of truth are the two `bench/*.json` ratchet files)

### Line floors below the ≥90 target, ranked by (distance × correctness-criticality)

| crate | floor | measured (ratchet note) | criticality | note |
|---|---|---|---|---|
| sparq-canon | 77 | 79.04 | **CORE** — RDFC-1.0 canonicalization; feeds diff-test bnode isomorphism (sq-qcnn.7), sparq-zk commitments, sparq-vc signing | mutation ceiling **STALE**: committed 9, real ≈ **98** survivors (sq-52su close-note) |
| sparq-substrate | 80 | 82.49 (`numeric.rs` **71.36**) | **CORE** — shared eval substrate: XSD numeric tower, 4 join kernels, term total order | worst mutation score in the baseline: **128 survivors, 60.98% caught** |
| sparq-engine | 83 (nightly 85) | ~85 test-only / 87.31 merged-nightly | **CORE** — the SPARQL engine itself | **absent from the mutation baseline entirely** (the historical gap the epic names) |
| sparq-server | 86 | 88.14 | PROTOCOL | 2nd pass after sq-4vao |
| sparq-geo | 87 | ~89 | CORE-eval — DE-9IM topology, geof: functions | unseeded in mutants |
| sparq-reason | 87 | 89.23 | **CORE** — RDFS/OWL-RL/N3 soundness | unseeded in mutants |
| sparq-rsp | 87 | ~89 | CORE-eval — deterministic window/R2S semantics | unseeded |
| sparq-zk | 88 | ~90 | SECURITY (route **opus**) | unaudited crate (sq-qhy4); tests claim no soundness |
| sparq-mpc | 88 | ~90 | SECURITY (route **opus**) | same |
| sparq-fedclient | 89 | 91.71 | PROTOCOL | micro-raise; nearly there |
| sparq-zk-compose | 76 | ~78 | SECURITY (route **opus**) | lowest floor in the file |

### High-floor crates whose mutation score exposes the vacuous-test gap (the epic's thesis)

| crate | line floor | surviving mutants | caught% |
|---|---|---|---|
| sparq-prov | 94 | 67 | 70.09 |
| sparq-introspect | 94 | 66 | 75.37 |
| sparq-parse | 91 | committed 14 → real ≈ **32** (stale) | 72.09 |

Line coverage says these crates are fine; mutation says a third of their logic could be
silently wrong. That asymmetry is exactly why the mutation ratchet, not the line floor, is
the program's primary success metric.

### Root infrastructure blocker

Only **6 of ~29 eligible crates** are seeded in `bench/mutants-baseline.json` and the lane is
still **advisory**, because the nightly `mutants-nightly` job is **cancelled after ~5-6
crates by the workflow concurrency group** (`cancel-in-progress` racing push-to-main on the
same ref) — sq-52su close-note. Until a full nightly artifact exists, ~20 crates (engine,
core, reason, geo, rsp, shacl, …) have *no* measured test-quality signal. Fixing this is the
single highest-leverage bead in the plan (TQ1). Crate beads do **not** block on it — each can
run `cargo mutants -p <crate>` locally to enumerate its own survivors.

### Explicit non-targets (so nobody re-litigates)

- At/above target: core 90, shacl 93, serve 92, wasm 93, nlq 91, vectors 91, fedplan 94,
  prov 94, introspect 94, sim 94, algos 97, text/hdt/policy 90 (prov/introspect get
  *mutation* beads only).
- `floor: 0` measurement-artifact crates (cli, conformance, gpu, *-wasm bundles) are
  presence-gated **by design** — leave them.
- sparq-solid 89: already had its raise pass (sq-2xdr); measured ≈ 91; diminishing returns.
- JSON-LD floors are owned by sq-oy1f.36. Un-floored **CORE** reasoners sparq-reason-el /
  sparq-reason-ql are a real gap → TQ13 wires both (bundled deliberately: gate-wiring
  touches the *shared* `scripts/coverage.sh` + ratchet JSON, so one bead avoids the only
  shared-file conflict in the plan).

## Discipline (goes verbatim into every bead brief)

1. **Enumerate before writing**: run `cargo mutants -p <crate>` (shard/timeout if large) and
   `scripts/coverage.sh` first; write tests that kill *named* survivors and cover *named*
   uncovered regions — never coverage-farm.
2. **One direct unit test per public fn** — wrappers/facades reached only indirectly sit ~0%
   covered (the #1250 trap). New tests assert *values*, not just "doesn't panic" (the
   mechanical-verify gate flips an expected value to check non-vacuity).
3. **Which mutants matter** (kill these classes first; they encode SPARQL semantics):
   comparison-operator flips (`<`→`<=`) in term-ordering/compare/window-bound code;
   arithmetic sign/rounding mutations in the numeric value tower (promotion, overflow,
   `xsd:decimal` rounding, INF/NaN); `&&`↔`||` flips in guard chains (fail-closed paths);
   "replace body with `Default::default()`" survivors on public fns (pure vacuous-test
   signal). **Provably-equivalent survivors** (defensive bounds, short-circuit guards) may be
   *documented* in the baseline note and left, as sparq-reason did — never silently ignored.
4. **Measure with the crate's whole-surface features** (see the `measure()` case in
   `scripts/coverage.sh`): substrate `numeric,join,compare,rows`; fedclient
   `fedclient,fedclient-adaptive`; policy `count-enforcement`; prov `reason`. Default build
   otherwise.
5. **Re-seed both ratchets** on completion:
   `scripts/coverage-gate.py --seed target/coverage/coverage-summary.json` (floors only rise)
   and `scripts/mutants-gate.py --seed target/mutants/<crate>/outcomes.json` (ceilings only
   fall). Stale ceilings (canon, parse) must be corrected to *reality* in the same PR that
   strengthens the tests — never commit a knowingly-false baseline.
6. Tests-only PRs: no public-API changes expected, so no SKILL.md churn; keep sparq-core/
   engine lean (no new deps); README untouched unless a note is factually stale.

## The beads (prioritized; one crate per bead; each independently dispatchable)

| # | bead | crate / surface | floor → target | why | tier |
|---|---|---|---|---|---|
| TQ1 | mutants-nightly CI fix + full seed | `.github/workflows` + baseline | — | unblocks measurement for ~20 crates; engine's first-ever mutation number | sonnet (sparq-ci-infra) |
| TQ2 | substrate raise + mutant-kill | sparq-substrate | 80 → 90 | correctness core; numeric.rs 71%; 128 survivors | sonnet |
| TQ3 | engine raise + first local mutants run | sparq-engine | 83 → 90 | the flagship; zero mutation signal today | sonnet |
| TQ4 | canon raise + kill real ~98 survivors + honest re-baseline | sparq-canon | 77 → 90 | lowest CORE floor; stale ceiling; feeds bnode-isomorphism + zk/vc | sonnet |
| TQ5 | parse mutation-kill + honest re-baseline | sparq-parse | 91 (floor OK) | vacuous-test archetype: real ~32 vs committed 14 | sonnet |
| TQ6 | reason raise + local mutants seed | sparq-reason | 87 → 90 | reasoner soundness; measured 89.23, close | sonnet |
| TQ7 | geo raise | sparq-geo | 87 → 90 | DE-9IM/geof eval semantics | sonnet |
| TQ8 | rsp raise | sparq-rsp | 87 → 90 | deterministic window semantics | sonnet |
| TQ9 | server raise (2nd pass) | sparq-server | 86 → 90 | protocol conformance envelope | sonnet |
| TQ10 | prov mutation-kill | sparq-prov | 94 (floor OK) | 67 survivors @ 70% caught | sonnet |
| TQ11 | introspect mutation-kill | sparq-introspect | 94 (floor OK) | 66 survivors @ 75% caught | sonnet |
| TQ12 | fedclient micro-raise | sparq-fedclient | 89 → 90 | measured 91.71; one small test-add | haiku |
| TQ13 | wire reason-el + reason-ql into the coverage gate | scripts/coverage.sh + floors | unfloored → seeded | CORE reasoners currently outside the ratchet | sonnet |
| TQ14 | zk-compose raise | sparq-zk-compose | 76 → 85 | lowest floor overall; audit-gated crate — tests make **no soundness claim** | **opus** |
| TQ15 | zk raise | sparq-zk | 88 → 90 | commitment/encoding determinism | **opus** |
| TQ16 | mpc raise | sparq-mpc | 88 → 90 | share/reconstruct algebra | **opus** |
| TQ17 | promote mutants lane advisory → GATING | ci.yml + baseline | — | end-state of the epic; **depends on TQ1 + TQ4 + TQ5** | sonnet (sparq-ci-infra) |

Only TQ17 has dependencies. TQ2–TQ16 touch disjoint crates (plus each crate's own line in
the two ratchet JSONs — different entries, near-zero conflict surface; rebase-don't-merge if
two land close together). SECURITY-tier beads (TQ14–16) route to **Opus** per standing
policy; everything else is mechanical-once-scoped fleet work.

### Per-crate module priorities (what "correctness-critical modules first" means)

- **sparq-substrate** (`--features numeric,join,compare,rows`): `numeric.rs` first — the XSD
  numeric value tower (type promotion, exact decimal arithmetic/rounding, overflow to
  error vs saturate, canonical lexical forms, INF/NaN propagation) is at 71.36% and carries
  most of the 128 survivors. Then `join.rs` (the 4 id-tuple kernels: empty-input, dup keys,
  bind-combine ordering, LFTJ trie edges), then `compare.rs` (SPARQL term total-order
  transitivity/antisymmetry across type boundaries — property-style direct tests). `rows.rs`
  is at 100.
- **sparq-engine**: aggregate semantics (GROUP BY empty-group, COUNT vs COUNT(DISTINCT),
  AVG/SUM over mixed numerics, error-in-group), FILTER three-valued logic (error ≠ false),
  OPTIONAL/MINUS scoping edges, ORDER BY across type boundaries, planner arms (greedy-GOO vs
  the opt-in DP planner must agree — a cheap in-crate differential), and the feature-gated
  surfaces (window-functions, result-cache invalidation, txn) in *both* feature states.
- **sparq-canon**: Hash-N-Degree-Quads recursion (automorphic bnode clusters — the classic
  RDFC-1.0 hard cases), poison-graph guard boundaries (the DoS bound: exactly-at vs
  one-past), issued-id map determinism, canonical N-Quads byte-exactness, the
  `rdf12-triple-terms` feature in both states, oxrdf-bridge error paths.
- **sparq-parse**: chunk-boundary correctness in the parallel gzip/zstd paths (record split
  across chunk edge, multi-member/multi-frame concatenation byte-equality vs single-stream),
  truncated/corrupt-stream error paths. Kill the ~32 real survivors; re-seed honestly.
- **sparq-reason**: remaining `n3/mod.rs` gaps (85.17 after sq-fodw5), builtin reverse-mode +
  fail-closed arms, incremental-closure retract/re-add, `explain` proof-tree feature state;
  document any *provably-equivalent* survivors in the baseline note as before.
- **sparq-geo**: DE-9IM predicate matrix vs known WKT fixture pairs (touching-not-overlapping,
  point-on-boundary), geof: function domain errors (empty geometry, mixed CRS), R-tree
  candidate-set == brute-force on small fixtures.
- **sparq-rsp**: window advance/close boundaries (tuple exactly on the edge — where `<`→`<=`
  mutants live), RStream/IStream/DStream deltas over identical inputs, out-of-order arrival
  determinism.
- **sparq-server**: content-negotiation + error-envelope paths not covered by sq-4vao
  (json_error/sanitized_error leak-envelope), GSP edge verbs, MVCC SI/OCC conflict paths —
  direct unit tests, not just loopback integration.
- **sparq-prov / sparq-introspect**: pure survivor-kill from the seeded outcomes lists; no
  floor movement needed. Assert lineage-graph *contents* (not counts) and introspection
  *values* (not presence).
- **zk / zk-compose / mpc** (Opus): deterministic encoding/round-trip and share/reconstruct
  algebra tests only; every PR body restates that tests do **not** constitute the sq-qhy4
  soundness audit.

## Success criteria for the epic (measurable, no vanity)

1. Every non-artifact crate has a **measured** mutation ceiling (absence ≠ zero survivors).
2. The mutants lane is **gating** (TQ17), with ceilings that are *true* (no stale canon/parse).
3. All CORE-crate line floors ≥ 90 (measured ≥ 92); protocol crates ≥ 89.
4. Surviving-mutant totals for the CORE set (substrate/engine/canon/reason) trend to a
   documented floor of provably-equivalent survivors — not to an undifferentiated number.

Beads carry ids `sq-qcnn.*` under the epic; this record is the plan of record for the wave.
