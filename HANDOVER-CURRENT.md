# Orchestrator handover — live state (updated 2026-06-12 ~late evening UTC)

If this file describes running agents/instances and you are a fresh session, verify each item below FIRST. Cloud resources take priority.

## Cloud resources
- NONE running. EC2 i-0b3e0be20affc86cf TERMINATED (verified, ~22:47Z 2026-06-12); 1B run completed, doc committed, cost ≈$0.78. NEVER touch i-090531b4ede8f2d3f (production).

## Git state (repo /Users/jesght/Documents/GitHub/rdfjs/sparq, remote `jeswr`)
- Pushed through 29ddd13. js-parity MERGED+PUSHED (gate 674/0; wasm baseline now **1,643,095**) — BINDINGS PARITY (goal clause 4) COMPLETE. Worktree sparq-jsbind removed, branch deleted.
- Gate command: `cargo test --workspace --exclude sparq-py --release --no-fail-fast 2>&1 | grep -aE "^test result" | awk '{s+=$4; f+=$6} END {print "passed:", s, "failed:", f}'` (-a REQUIRED) + wasm build + stat -f%z.

## Running agents (worktrees; agents NEVER push/merge; each keeps STATUS.md current)
- zk-core (../sparq-zk-core): resuming from 1c57a2d — rdf-canon suite, bnode guard, criterion benches, wasm-off check.
- DONE zk-ieee754-kernels: MERGED+PUSHED (2b66ba7, gate 674/0, wasm 1,643,095). Comparisons/rounding/sqrt/casts kernels + 52 gate-bench rows + 121/121 oracle vectors. Follow-up noted: sqrt_f16 circuit byte-identical to sqrt_f64 (correct per oracle; possibly missing narrow remainder bound).
- zk-xpath (../sparq-zk-xpath): resuming from 81d08c5 — verify beta.21 claim, run/sample ~241 test packages, VENDOR.md + function inventory.
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

## Awaiting Jesse
test-lib push (76 commits ahead), horizontal-scaling ADR answers, RDFox license decision.
