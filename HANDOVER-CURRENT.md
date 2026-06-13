# Orchestrator handover — live state (updated 2026-06-13, USAGE FREEZE active)

## ⚠ USAGE FREEZE (user, 2026-06-13): ~80% of session budget consumed. NO new agent launches. If this session dies, resume protocol: (1) NO cloud resources are running (verify with the tag query below anyway); (2) check each worktree's STATUS.md + `git log main..HEAD` for agent state; (3) work the merge queue below one branch at a time with the gate.

## MERGE QUEUE (exact, in order; gate each, push on green)
1. zk-core: ALREADY MERGED on local main (unpushed); gate running at freeze time (`bash task bv9f3atvv`, expect ~712+/0 + wasm 1,643,095) → push jeswr main, remove ../sparq-zk-core, delete branch zk-core.
2. wikidata-8b-prep (../sparq-wd8b @ 50fe3b8, docs+scripts only) — merge, drop STATUS.md, gate (cheap), push, clean.
3. release-prep-v010 (../sparq-relprep @ HEAD, CHANGELOG fold + dist.yml runner + caveats + docs/release-notes-v0.1.0.md) — merge, gate, push, then: wait GitHub CI green on that SHA → `git tag v0.1.0 <sha> && git push jeswr v0.1.0` (release.yml creates the GH release + ghcr docker; do NOT gh release create) → `gh release edit v0.1.0 --notes-file docs/release-notes-v0.1.0.md`. crates.io/npm publishes BLOCKED on Jesse's `cargo login`/`npm login` (then docs/release.md §4 order; `--dry-run -p sparq-engine` checkpoint mid-chain).
4. Then in-flight agent branches as they complete (each: merge, drop STATUS.md, gate, push, clean worktree): dict-spill, zk-xpath-ieee754-migration, zk-compose, zk-trace-engine, serve-wave-a2, neon-intersect (re-verify benchmark numbers on a QUIET machine first), py-textindex.
5. After dict-spill lands: the 8B Wikidata EC2 run per bench/wikidata-8b/RUNBOOK.md (cap $30, ~$0.78 spent; babysitter pattern; SPARQ_SHA pin required).
6. After zk-compose + zk-trace land: write research/zk-test-bench-design.md (how ZK test suites/benchmarks are designed, what they derive from) → HAND TO JESSE FOR COMMENT (his explicit deliverable).
7. test-lib upstreaming (agent in ~/Documents/GitHub/jeswr/test-lib branch upstreaming-prep): when its parity report lands, review, then the swap plan in the IEEE754 audit (deprecated branch first, merge -s ours, re-pin noir_XPath to v0.9.0 BEFORE swap, keep test-summary check name). Swap needs Jesse-visible confirmation before the push.

If this file describes running agents/instances and you are a fresh session, verify each item below FIRST. Cloud resources take priority.

## Cloud resources
- NONE running. EC2 i-0b3e0be20affc86cf TERMINATED (verified, ~22:47Z 2026-06-12); 1B run completed, doc committed, cost ≈$0.78. NEVER touch i-090531b4ede8f2d3f (production).

## Git state (repo /Users/jesght/Documents/GitHub/rdfjs/sparq, remote `jeswr`)
- Pushed through 29ddd13. js-parity MERGED+PUSHED (gate 674/0; wasm baseline now **1,643,095**) — BINDINGS PARITY (goal clause 4) COMPLETE. Worktree sparq-jsbind removed, branch deleted.
- Gate command: `cargo test --workspace --exclude sparq-py --release --no-fail-fast 2>&1 | grep -aE "^test result" | awk '{s+=$4; f+=$6} END {print "passed:", s, "failed:", f}'` (-a REQUIRED) + wasm build + stat -f%z.

## Running agents (worktrees; agents NEVER push/merge; each keeps STATUS.md current)
- zk-compose (../sparq-zkcompose): ZK stage 2 — prover+verifier composition package (Noir circuit family w/ dynamic (k,n) sizing + crates/sparq-zk-compose orchestration, ProofManifest w/ entailmentRegime, tamper tests, gate/proving benches).
- zk-trace-engine (../sparq-zktrace): module B — per-operator exact input-set capture w/ graph attribution behind `zk` feature; owns sparq-engine zk paths + sparq-zk trace modules.
- serve-wave-a2 (../sparq-wavea2): sequenced writer + group-commit (+A3 epochs if clean); owns crates/sparq-serve only.
- neon-intersect (../sparq-neon): measure-first NEON/portable-SIMD intersection kernel; CAUTION at merge review — heavy concurrent builds on this M1 skew timings, re-verify headline numbers on a quiet machine before accepting.
- py-textindex (../sparq-pytext): TextIndex lifecycle on the py wrapper; owns sparq-py + minimal sparq-text additions.
- zk-core (../sparq-zk-core): resuming from 1c57a2d — rdf-canon suite, bnode guard, criterion benches, wasm-off check.
- DONE zk-ieee754-kernels: MERGED+PUSHED (2b66ba7, gate 674/0, wasm 1,643,095). Comparisons/rounding/sqrt/casts kernels + 52 gate-bench rows + 121/121 oracle vectors. Follow-up noted: sqrt_f16 circuit byte-identical to sqrt_f64 (correct per oracle; possibly missing narrow remainder bound).
- DONE zk-xpath: MERGED+PUSHED (504014a, gate 674/0, wasm 1,643,095). Vendor verified, zero beta.21 drift, all 241 packages run (63/71 REAL green, 8 pre-existing upstream gaps; 21/21 float pkgs = the migration oracle), VENDOR.md inventory written.
- zk-xpath-ieee754-migration (../sparq-zk-xpmig, from 504014a): USER SCOPE — migrate zk/xpath floats old API → sparq_ieee754; oracle = 21 float pkgs/283 tests; noir skills mandated; gate-count before/after table required.
- dict-spill (../sparq-dictspill, from 49a37cd): external/spillable build-time term dictionary per memtier requirements; audit 137fb39 consolidation spill machinery first; byte-identity preferred; wasm stays 1,643,095.

## Queue after agents return
1. Merge-gate each ZK branch (one at a time), push, clean worktrees.
2. NEW USER SCOPE (2026-06-12): migrate zk/xpath to the latest sparq_ieee754 float API (zk/ieee754) NOW, not stage 2 — launch a migration agent on the zk-xpath worktree after the current agent finishes. Reference: noir_XPath branch origin/copilot/update-noir-ieee745-package (partial old-pkg migration).
3. USER REQUIREMENT: every ZK agent brief must point at the vendored skills in .claude/skills/ (noir-optimisation, noir-circuit-patterns, sparql-formal-semantics, verifiable-credentials-zk — read by absolute path) AND the GLOBAL ~/.claude/skills noir-developer/noir-idioms/noir-js/noir-testing (invocable via Skill tool) — required before writing Noir code.
4. ZK stage 2: composition package modeled on sparql_noir_modular; e2e prove/verify + tamper tests; gate-count + proving-time benchmarks.
5. FINAL DELIVERABLE: research/zk-test-bench-design.md — how the ZK test suites/benchmarks are designed — handed to Jesse for comment.
6. DONE: Wikidata stage-1 doc committed (1B results + cost ledger). Follow-on engineering item: dictionary spill-to-disk (prerequisite for full truthy ~280-330GiB dict; slots into memtier track).

## Crash-resilience protocol (user mandate, revised)
The global plan meter is not accessible from inside a session and local token counts don't predict it — no usage polling. Instead: every agent brief mandates incremental commits + live STATUS.md; this file stays current each wave; cloud resources are first priority in any recovery.

## Authorizations granted 2026-06-13
- EC2 cap raised to $30 TOTAL (~$0.78 spent): 8B full-truthy Wikidata run AUTHORIZED + other instance needs. Sequencing: 8B low-resource run REQUIRES dict-spill merge first (dict ~280-330GiB unspillable otherwise). Prep (runbook/scripts, zero spend) proceeding in parallel.
- PUBLISHING AUTHORIZED: packages may be published under Jesse's accounts (npm @jeswr/sparq, crates.io, GitHub release v0.1.0 — T14/T20 were user-gated, now unblocked).

## IEEE754 upstreaming directive (Jesse, 2026-06-13)
test-lib content REPLACES the published noir IEEE754 repo (old main preserved, e.g. moved to a `deprecated` branch) ONCE test-lib has the same level of testing/benchmarking/CI as the old repo. Sequence: (1) audit agent — old repo's CI/test/bench inventory, test-lib state (76 unpushed commits + dirty kernels.nr), delta sparq zk/ieee754 vs test-lib (new kernels/benches to flow back); (2) gap-closure work in test-lib; (3) orchestrator executes swap: push `deprecated` branch first, verify, then replace main with test-lib history (+ sparq-side kernel additions). test-lib repo is now WRITABLE for this work (directive supersedes read-only-reference rule for test-lib only).

## Paused (user direction 2026-06-13)
- Horizontal scaling: PAUSED — to be worked in the prod-solid-server context, not sparq. research/adr-horizontal-scaling.md stays as reference; no implementation here.

## Awaiting Jesse
RDFox license decision; `cargo login` + `npm login` for registry publishes; PyPI rename decision (sparq taken → sparq-rdf free).
