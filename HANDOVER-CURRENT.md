# Orchestrator handover — live state (updated 2026-06-12 ~late evening UTC)

If this file describes running agents/instances and you are a fresh session, verify each item below FIRST. Cloud resources take priority.

## Cloud resources
- EC2 i-0b3e0be20affc86cf (r8g.large, eu-west-2, profile `pss`, tag purpose=sparq-hw-validation, launched 2026-06-12T18:53Z): Wikidata 1B benchmark. A babysitter agent is reattaching/finishing/TERMINATING it. If this file still lists it, verify:
  `aws ec2 describe-instances --profile pss --region eu-west-2 --filters Name=tag:purpose,Values=sparq-hw-validation`
  NEVER touch i-090531b4ede8f2d3f (production).

## Git state (repo /Users/jesght/Documents/GitHub/rdfjs/sparq, remote `jeswr`)
- Pushed through e1c33b9 (rsp-prepared merge). LOCAL UNPUSHED on main: 0f78cf6 (js-parity merge, wasm +50,028 accepted → new baseline ~1,643,095) + skills-vendor commit. Push blocked on the js gate (background task bcxxz1m7x): green = passed N/0 + wasm byte count; then `git push jeswr main`, `git worktree remove ../sparq-jsbind`, `git branch -d js-parity`.
- Gate command: `cargo test --workspace --exclude sparq-py --release --no-fail-fast 2>&1 | grep -aE "^test result" | awk '{s+=$4; f+=$6} END {print "passed:", s, "failed:", f}'` (-a REQUIRED) + wasm build + stat -f%z.

## Running agents (worktrees; agents NEVER push/merge; each keeps STATUS.md current)
- zk-core (../sparq-zk-core): resuming from 1c57a2d — rdf-canon suite, bnode guard, criterion benches, wasm-off check.
- zk-ieee754-kernels (../sparq-zk-kernels): resuming from 373d5cb — finish gate benches for new kernels, regression vs beta-21 baseline.
- zk-xpath (../sparq-zk-xpath): resuming from 81d08c5 — verify beta.21 claim, run/sample ~241 test packages, VENDOR.md + function inventory.
- Wikidata babysitter: see Cloud resources.

## Queue after agents return
1. Merge-gate each ZK branch (one at a time), push, clean worktrees.
2. NEW USER SCOPE (2026-06-12): migrate zk/xpath to the latest sparq_ieee754 float API (zk/ieee754) NOW, not stage 2 — launch a migration agent on the zk-xpath worktree after the current agent finishes. Reference: noir_XPath branch origin/copilot/update-noir-ieee745-package (partial old-pkg migration).
3. USER REQUIREMENT: every ZK agent brief must point at the vendored skills in .claude/skills/ — noir-optimisation, noir-circuit-patterns, sparql-formal-semantics, verifiable-credentials-zk — and require reading them before writing Noir code.
4. ZK stage 2: composition package modeled on sparql_noir_modular; e2e prove/verify + tamper tests; gate-count + proving-time benchmarks.
5. FINAL DELIVERABLE: research/zk-test-bench-design.md — how the ZK test suites/benchmarks are designed — handed to Jesse for comment.
6. Wikidata stage-1 doc: 1B results + cost ledger + RDFox comparison; commit research/wikidata-lowresource-stage1.md.

## Usage-aware shutdown protocol (user mandate)
Check `npx -y ccusage@latest blocks --json` between waves (reference reading: 140.6M tokens in the 20:00→01:00 UTC block at the time of writing). Near the limit: launch nothing new; agents already carry the STATUS.md/incremental-commit crash protocol.

## Awaiting Jesse
test-lib push (76 commits ahead), horizontal-scaling ADR answers, RDFox license decision, 8B-run budget sign-off.
