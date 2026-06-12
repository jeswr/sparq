# Inventory: Noir ZK + SPARQL estate (verified on disk, 2026-06-12)

Read-only survey; no repo was modified. Installed `nargo` is **1.0.0-beta.21**; the whole
estate targets beta.16/17, so any local compile today hits toolchain drift before real bugs.

## (a) Inventory table

| Path | Nargo/pkg name | Remote | Branch | Git state | Last commit | Size / purpose |
|---|---|---|---|---|---|---|
| `~/Documents/GitHub/jeswr/zkp-sparql-workspace` | (master workspace, not a Noir pkg) | `jeswr/zkp-sparql-workspace` | `proofs/div-f64-closure-workspace-side` | **ahead 3 unpushed**, 5 dirty (incl. untracked `questions/mul-f32-closure-agent-interrupted.md`) | 2026-05-13 | ISWC-2026 master repo: notes/decisions/questions/paper/Lean proofs + `circuits/` sub-checkouts + 21 worktrees |
| `…/zkp-sparql-workspace/circuits/sparql_noir` | TS `@jeswr/sparql-noir` + `noir/lib/{consts,types,utils,hashes,ebv,xpath,arith,algebra}` + `noir/bin/*` | `jeswr/sparql_noir` | `feat/algebra-round-2-graph-assertable-isiri-field` (synced) | 2 dirty | 2026-05-13 | The monolithic per-query circuit system; 573 tracked files on GitHub main |
| `~/Documents/GitHub/jeswr/sparql_noir` (standalone) | same | same | `feat/extended-featurs` — **no upstream** | **12 dirty** (deletes `ARCHITECTURE.md`, `noir/lib/arith/*`) | **2026-01-01** | Stale divergent copy, ~5 months behind GitHub main (pushed 2026-05-23) |
| `…/circuits/sparql_noir-sentinels-wiring` | same repo, 2nd checkout | same | `non-membership-sentinels-transform-wiring` — **upstream gone** | clean | 2026-05-04 | Dead checkout; remote branch deleted (sentinel non-membership work) |
| `…/circuits/sparql_noir_modular` | `filter_eq/ne/lt/gt/lang`, `bgp_match`, `binding_consistency` (one bin pkg each, dep: `noir-lang/poseidon v0.1.1` only) + TS `src/` | `jeswr/sparql_noir_modular` (main pushed 2026-05-13) | `v0.4-g5-filter-ne-lang` (synced; merge-to-main status unclear) | clean | 2026-05-13 | Property-decomposed prover/verifier; 3,066 TS lines + 7 Noir modules |
| `~/Documents/GitHub/jeswr/noir_IEEE754` | workspace: `ieee754`, `ieee754_unit_tests`, 44 `test_packages/*` | `jeswr/noir_IEEE754` (**public**, GitHub main 2026-05-10, branch pushes to 2026-06-03) | `copilot/ct-kernel-f16-storage` (synced — **not main**) | 6 untracked scratch items | 2026-05-19 | The PUSHED float lib: 130 src files, 16,773 lines |
| `…/circuits/noir_IEEE754` | same, 2nd checkout | same | `feat/claude-config` (merged as #93) | 1 dirty | 2026-05-10 | Workspace-side checkout, slightly stale |
| `~/Documents/GitHub/jeswr/test-lib` | **`test_lib`** | `jeswr/test-lib` (**private**, remote pushed 2026-05-19) | `main` — **ahead 76 unpushed** | 2 dirty (`src/ops/kernels.nr` modified, `scripts/__pycache__/`) | **2026-05-21** (newest float work anywhere) | The hidden better float lib: 7 src files, 1,929 lines, 79 commits over 4 days (2026-05-18→21) |
| `~/Documents/GitHub/jeswr/noir_XPath` | workspace: `xpath`, `xpath_unit_tests`, **241** `test_packages/*` | `jeswr/noir_XPath` | `main` (synced) | clean | 2026-01-29 | XPath 2.0 F&O for SPARQL FILTER; xpath lib = 24 files, 10,900 lines |
| `…/circuits/noir_XPath` | same + `qname_bench` + 20 float test pkgs | same | `refactor/new-ieee754-api` (synced) | 1 dirty | 2026-05-13 | PR-#39 migration branch to new Float API |
| `…/circuits/lampe-literate` | tooling (not Noir) | `jeswr/lampe-literate` | `fix/dedup-directives-by-kind-path` (synced) | clean | 2026-05-07 | Noir↔Lean literate-directive tool |
| `~/Documents/GitHub/jeswr/{noir-IEEE754, IEEE754-noir}` | — | — | — | **both EMPTY dirs** (created 12 Dec) | — | Stub noise; ignore |
| `~/Documents/desktop-content/zk_sparql_noir` | — | — | — | macOS **alias into the Trash** | — | Deleted folder; not live |

Early pipelines (`noir_sparql`, `noir_sparql_proof`, `noir_sparql_proof_rust`, `noir_json_parser`)
exist as the recon doc describes; not re-verified in depth.

Dependency edges: monolith `noir/lib/xpath` → `jeswr/noir_XPath@v0.1.0` (git tag); standalone
`noir_XPath/xpath` → `jeswr/noir_IEEE754@v0.3.1` (git tag) + `noir-lang/noir_json_parser@main`
(floating tag!); workspace `noir_XPath/xpath` → **relative path `../../../../noir_IEEE754/ieee754`**
(the standalone checkout); modular Noir modules → `noir-lang/poseidon@v0.1.1` only.
**Nothing depends on `test_lib` yet.**

## (b) The IEEE754 twins

- **Pushed one = `noir_IEEE754`** (public GitHub, the only float lib with a published tag, v0.3.1,
  consumed by noir_XPath). Big and complete: old free-function f32/f64 API at v0.3.1, plus the newer
  generic `Float<EXP_BITS, MANT_BITS, RM, E, M>` struct + `FloatStorage` trait (f8/f16/f32/f64
  storage impls), `std::ops` Add/Sub/Mul/Div/Neg, `sqrt_impl`, rounding-mode type parameter,
  44 FPgen/MPFR-oracle test packages, Lean obligations in the workspace.
- **Better one = `~/Documents/GitHub/jeswr/test-lib`, package name `test_lib`** — the "folder by a
  completely different name". Evidence: `src/codegen.nr` comptime-generates public `f16`/`f32`/
  `f64`/`f128` structs via `#[generate_float_type(N)]`; `FloatParts<E,M>` carrier;
  `src/ops/kernels.nr` (1,443 lines) with `add/sub/mul/div` (+ `_wide` u128 variants), uint→float
  conversions, classify/NaN canonicalisation, hint-and-verify helpers; GitHub description:
  "Generated Noir float library and gate benchmark experiments". The better abstractions: truly
  private fields/methods (enforced by `tests/private_fields/*`, lint scripts), generated global
  type names usable as `f64`, **f128 support**, committed gate baselines
  (`bench/float_ops_latest.json`) and a benchmark/regression harness.
- **Completeness gap (honest verdict):** test_lib is far less complete — no sqrt, **no comparison
  ops** (no Eq/Ord/lt anywhere), no rounding-mode parameter surfaced in the public API, ~22 tests
  vs noir_IEEE754's 44 oracle test packages, 1.9k vs 16.8k lines.
- **Git state:** remote `jeswr/test-lib` exists (private) but stale — local `main` is **76 commits
  ahead, unpushed**, plus uncommitted edits to `src/ops/kernels.nr`. The current best float work
  effectively exists only on this machine.

## (c) XPath library

Two checkouts, two API eras:
- **Standalone `noir_XPath` main** (clean, synced, 2026-01-29): float usage isolated to
  `xpath/src/numeric_types.nr` (1,399 lines), importing ~30 symbols of the **old** v0.3.1
  free-function API (`add_float32`, `float64_lt`, `IEEE754Float32`, rounding-mode consts, …).
- **Workspace branch `refactor/new-ieee754-api`** (deferred PR #39): migrated `XsdFloat`/`XsdDouble`
  onto the new `Float<E,M,RM>` API, wired 20 extra float test packages, but its Nargo dep is a
  TODO-marked **relative path to the standalone noir_IEEE754 checkout** — which currently sits on
  an unrelated `copilot/ct-kernel-f16-storage` branch. Blocked on the new API landing on
  noir_IEEE754 main.
- **Compile state:** `nargo check` (beta.21, /tmp copy of standalone main) fails with 26 errors —
  21 in xpath src (`sequence.nr` ×10, `numeric_types.nr` ×7, qname/types/date), rest inside deps.
  Repo pins **beta.16**, so largely toolchain drift, but **noir_XPath does not compile on the
  currently installed toolchain**. Its "Check for New Noir Versions" CI job fails daily
  (checked back to 2026-06-07).
- **Migration-to-test_lib estimate:** glue surface small — one file (`numeric_types.nr`) holds every
  IEEE754 reference. Real cost is in test_lib: it needs comparison predicates (eq/lt/le/gt/ge with
  NaN semantics), round/floor/ceiling-to-integral, float→int casts, and explicit rounding-mode
  entry points before XPath can use it. Rough size: a few hundred lines of new kernels + tests in
  test_lib, plus rewriting ~1.4k lines of numeric_types glue, plus regenerating float test
  packages. Competing half-finished path: PR-#39 migration to noir_IEEE754's new Float API is
  already ~done on the workspace branch.

## (d) The "trace" module — memory vs reality

**No module named `trace` exists in any of the Noir/ZKP repos** (verified: workspace-wide greps,
local + GitHub trees of `sparql_noir` incl. all 25 branches, `sparql_noir_modular` local + GitHub).
What exists:
1. **`sparq` engine** (`crates/sparq-engine/src/exec.rs`, `pub(crate) mod trace`, T22): thread-local
   EXPLAIN-ANALYZE operator trace — `Node {label, depth, rows, nanos}` — records operator row
   counts/timings, **not** proof-input sets.
2. The thing matching the description — executor trace (per-row matched leaf/slot indices, scan
   boundaries, executed join order) feeding trace-driven witness minimisation — is **Stage 2 of
   `research/zkp-query-proofs-plan.md`**: designed, **not implemented**.
3. In the modular repo, input identification is **`compileQuery`** (`src/compile.ts`, 609 lines):
   deterministic SPARQL→obligations translator, emitting ordered `triplePatterns` whose index =
   BGP row = `bindingIndex`, used to pull Merkle paths from `DatasetTree.proofs`.
   Deterministic-by-contract (shared prover/verifier TCB) but walks the query AST; not an
   execution trace over the dataset.

Verdict: the remembered trace module **does not exist yet as built**; the explain-analyze trace
and compileQuery are the two extant seams it grafts onto.

## (e) `sparql_noir_modular` architecture

- **Per-property circuits**: `noir/<module>/` — one tiny `bin` package per atomic property, each
  depending only on poseidon. "Property" = an **atomic query obligation**, finer than a triple
  pattern: one FILTER comparator applied to one binding row (`filter_eq/ne/lt/gt/lang`), one BGP
  triple-pattern Merkle inclusion (`bgp_match`, depth-8 Poseidon2), or cross-row variable
  consistency (`binding_consistency`). Each anchors public inputs to a disclosed-binding row +
  dataset commitment and emits a Poseidon-2 claim hash.
- **Composition = verifier-side manifest, no recursion.** `dispatch.ts` classifies each obligation
  **proof-vs-clear** (hidden operand → ZK proof; all-disclosed → plain-JS check; this selective
  split is the repo's stated architectural innovation), proves in parallel, emits
  `ProofManifest {disclosed, modules, edges}` (~161 KB demo). `verify.ts` (882 lines) re-runs
  `compileQuery` on the same query text, verifies each UltraHonk proof, recomputes public-input
  hashes from disclosed bindings, enforces complete-cover, and **re-classifies** to reject
  proof→clear downgrade attacks. Joins/UNION/OPTIONAL are manifest *edges* checked in plain JS.
  Recursive-aggregation meta-circuit over the manifest = explicitly future work.
- **State**: local branch `v0.4-g5-filter-ne-lang` (G5 closed, filter_ne + filter_lang shipped per
  `AUDIT.md`); GitHub main last pushed 2026-05-13. Coverage: BGP + `FILTER (?x op literal)` over
  =, ≠, <, >, lang; UNION/OPTIONAL/BIND/compound expressions = future work in compile.ts.
- **Inference support: none.** No rdfs/owl/entailment code anywhere. Extension surface: new Noir
  property module + `src/modules/*.ts` witness builder + a `compileQuery` classification branch —
  the README documents exactly this drop-in pattern.

## (f) Workspace health — what "out of control" actually means

Mostly **sprawl and divergence, not breakage**:
- **~27 working copies inside one repo**: 6 `circuits/` checkouts + 16 `.worktrees/` + 5 under
  `.worktrees/sub/` — plus duplicate standalone checkouts elsewhere in `~/Documents/GitHub/jeswr`.
  Sampled worktrees clean and synced.
- **Every checkout sits on a different non-main feature branch.** The workspace itself is 3 commits
  ahead of origin on a Lean-proof branch with 5 dirty paths, including an interrupted-agent
  question file — Wave-18 state frozen mid-flight since 2026-05-13.
- **Stale divergent copies**: standalone `sparql_noir` on an unpushed branch from 2026-01-01 with
  12 dirty files — 5 months behind its own GitHub main; sentinels-wiring checkout's upstream gone.
- **Fragile cross-repo path dep**: workspace noir_XPath builds against the standalone noir_IEEE754
  working copy via `../../../../`, whose checked-out branch is an unrelated Copilot branch.
- **Unpushed crown jewels**: test-lib's 76 unpushed commits + dirty kernels.nr.
- **Noise**: two empty IEEE754-named stub dirs; untracked scratch files; daily-failing
  version-check CI on noir_XPath.

## (g) Contradictions / additions vs the recon doc (zkp-noir-context.md)

1. **test-lib is invisible in the recon doc** — yet it is the newest float work (2026-05-18→21)
   and the answer to the IEEE754 quirk.
2. **The trace module is plan, not code.**
3. Recon's "Float<E,M,RM> API was landing" — confirmed on branches; publish-to-main blocker in
   PR #39's TODO still live; noir_IEEE754 saw branch pushes to 2026-06-03 not reflected locally.
4. sparql_noir GitHub continued past the local clone — confirmed (main pushed 2026-05-23;
   25 branches incl. prefix-tree, string-witness, babyjubjub signature work).
5. `desktop-content/zk_sparql_noir` is an alias to a Trashed folder — no hidden workspace there.
6. Name resolution: "zk-sparql-workspace" = `~/Documents/GitHub/jeswr/zkp-sparql-workspace`;
   "the sparql noir modular library" = `circuits/sparql_noir_modular`, package-per-module names
   `filter_eq`…`binding_consistency` (no package literally named `sparql_noir_modular`).
