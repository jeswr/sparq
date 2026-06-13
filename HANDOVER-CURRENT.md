# Orchestrator handover — live state (updated 2026-06-13, SESSION-END at 97% usage, model=Opus 4.8)

## 🔴 SESSION-END HANDOFF (97% usage — this session may die any moment)
RESUMPTION OPTIONS: continue on this M1 in a fresh session, OR finish porting to the EC2 dev box (below).

### Pushed/clean
- main pushed up to (this commit) — includes zk-trace-engine merge (module B, agent-gated 734/0 in-worktree; orchestrator re-gate was KILLED due to load-39 thrash, NOT a failure — re-gate cleanly when load is low) + research/zk-soundness-audit.md.

### Unpushed/unmerged branches (committed, safe; merge when capacity allows — each: gate, push, clean worktree)
- proppaths-design @ 28a6dd4 — research/property-paths-design.md (paths ALREADY implemented, 33/33 W3C; doc is an audit + flags a likely latent bug: GRAPH-scoped zero-length `*`/`?`). DOC-only merge.
- zk-test-bench-design @ 3c91055 — research/zk-test-bench-design.md (the user's ZK test/bench deliverable + 6 open questions). DOC-only merge.
- upstreaming-prep @ cd5de8d in ~/Documents/GitHub/jeswr/test-lib — IEEE754 upstreaming PARITY COMPLETE (CI+test-summary, lint, README+LICENSE, clean secret scan, PARITY.md). NOT pushed. Awaits Jesse's go-ahead for the swap onto public noir_IEEE754 (deprecated-branch-first, merge -s ours, re-pin noir_XPath to v0.9.0 first — see research IEEE754 audit). v1.0.0 needs a feat!/BREAKING-CHANGE swap commit.

### RUNNING when session ended (may die at limit — recover via each worktree's STATUS.md + `git log main..HEAD`)
- serve-wave-b (../sparq-serveb): serving wave B scheduler (SRPT+aging, no-HoL). Building/testing.
- dict-spill (../sparq-dictspill): spillable dictionary finisher (gates 8B run). Benchmarking; ITS /private/tmp/dsp_bench (~53GB) is ACTIVE — reclaim only after it finishes.
- MPC research WORKFLOW run wf_f4fd3613-877 (mpc-zkp-research): deep research → research/mpc-zkp-research-and-architecture.md. ⚠ Workflow result is returned to the orchestrator; if the session dies before it completes, the synthesis is LOST — re-run the workflow (script saved at .../workflows/scripts/mpc-zkp-research-wf_f4fd3613-877.js). See memory [[mpc-zkp-project]].
- neon-intersect (../sparq-neon): PARKED (0 commits) — needs a quiet machine.

### TOP PRIORITY QUEUED WORK
1. ZK REMEDIATION (critical): research/zk-soundness-audit.md found the v1 verifier is UNSOUND (6 critical binding gaps). Fix order: public-input reconstruction + canonical vk (issues #1/#2) FIRST, then statement binding (#5/#8/#10/#11), then issuer-sig + replay (#3/#4). Use a fix→re-audit workflow. The zk-test-bench-design doc maps forge-and-verify tests to each issue.
2. EC2 PORT (in progress — see below).

### EC2 DEV-BOX PORT (prereqs DONE, launch is ONE command)
Goal: port the agent work to a fresh Claude Code session on a SEPARATE account on EC2 (no shared account/compute). Jesse logs in (I can't authenticate); /remote for desktop + SSH for terminal.
- DONE: keypair ~/.ssh/sparq-dev.pem (chmod 600); SG sg-0414c35a93f1c6557 (SSH from 188.29.24.150/32); AMI ami-03018a249a89a9ec1 (Ubuntu 24.04 arm64, eu-west-2); bootstrap script at /tmp/sparq-dev-userdata.sh (Node+Claude Code+Rust+wasm+gh+deps; noir toolchain = follow-up via noirup, needs nargo 1.0.0-beta.21 + bb 5.0.0-nightly.20260324).
- DECIDED: m7g.4xlarge, 300GB gp3, tag purpose=sparq-dev, profile pss, region eu-west-2.
- LAUNCH CMD (run after confirming /tmp/sparq-dev-userdata.sh exists): aws ec2 run-instances --profile pss --region eu-west-2 --image-id ami-03018a249a89a9ec1 --instance-type m7g.4xlarge --key-name sparq-dev --security-group-ids sg-0414c35a93f1c6557 --block-device-mappings '[{"DeviceName":"/dev/sda1","Ebs":{"VolumeSize":300,"VolumeType":"gp3"}}]' --user-data file:///tmp/sparq-dev-userdata.sh --tag-specifications 'ResourceType=instance,Tags=[{Key=purpose,Value=sparq-dev},{Key=Name,Value=sparq-dev-claude}]' --query 'Instances[0].InstanceId' --output text
- THEN: get public IP; ssh -i ~/.ssh/sparq-dev.pem ubuntu@<IP>; wait for ~/BOOTSTRAP_DONE; `gh auth login` (Jesse); clone jeswr/sparq + jeswr/* refs; copy HANDOVER-CURRENT.md + memory dir; start `claude` in tmux, relay login URL to Jesse, then /remote.
- NEVER touch prod EC2 i-090531b4ede8f2d3f. Dev box is a NEW instance.

---
# Orchestrator handover — live state (updated 2026-06-13, POST-CRASH RECOVERY, model=Opus 4.8)

## ⚠ MODEL = OPUS 4.8 (Fable 5 temporarily unavailable). Per Jesse: tag every commit with `Model: Opus 4.8 (...)` + `Co-Authored-By: Claude Opus 4.8` trailer, inline `// [OPUS-4.8]` markers on new code, same in every subagent brief — so Opus work can be re-reviewed when Fable returns. See memory feedback-opus-commit-tagging. When /model shows Fable again, revert to the Fable trailer.

## POST-CRASH STATE (2026-06-13): the prior session's 10 agents all died at the 4:20am session limit. Main = remote = 8ab4754 then py-textindex merged (dac9870, gate pending). Recovery done so far:
- zk-core, wikidata-8b-prep, release-prep-v010: ALL MERGED+PUSHED before the crash (main reached 8ab4754).
- py-textindex: MERGED+PUSHED (dac9870, gate 713/0 wasm 1,643,095). Task #44 bindings parity COMPLETE. Fable-authored branch, Opus merge. Worktree removed, branch deleted.
- dict-spill / serve-wave-a2 / zk-xpath-ieee754-migration: had committed+dirty work → OPUS FINISHER AGENTS RELAUNCHED (running now).
- zk-xpath-ieee754-migration: MERGED+PUSHED (f3bdb4c, gate 713/0). Opus finisher kept floor/ceil_float bit-twiddle on bb-gates evidence.
- zk-compose: MERGED+PUSHED (296f21f, gate 723/0). ZK stage-2 prover/verifier — dynamic (k,n) circuit family + crates/sparq-zk-compose, per-pattern BGP scan proofs + int FILTER + ProofManifest + nargo/bb prover + verifier w/ bnode re-check + tamper tests. Salvaged Fable scaffold (86496e7) validated + extended by Opus. Deferred: f64 FILTER compose, in-circuit sigs, multi-pattern joins, aggregation, revocation, inference, HolderPoP.
- serve-wave-a2: MERGED+PUSHED (e4e7745, gate 733/0). Sequenced writer + group-commit + commutativity batching + A3 RYW token. ~7.8x writer throughput at batch-16; batch-256 regresses (documented). Fable A2 + Opus A3/tests/benches.
- zk-trace-engine: RELAUNCHED from clean main, RUNNING (module B per-operator input-set capture; salvaged partial at /tmp/zk-salvage/zktrace-partial.patch was reference-only).
- neon-intersect: 0 commits, STILL PARKED — measure-first spike needs a QUIET machine; relaunch when dict-spill + zk-trace finish. Old worktree (../sparq-neon @ b436188) still exists, removable.
- EC2: could NOT verify (AWS SSO token expired — `aws sso login --profile pss` to re-check). Prior handover says nothing was ever launched; the 8B run is gated on dict-spill so it never started. LOW risk but UNVERIFIED.

## MODE: ultracode (this session, user /effort ultracode) — default to workflow orchestration + adversarial verification for substantive tasks; token cost not a constraint. Reverts when session ends.

## DISK (user mandate 2026-06-13, see memory feedback-disk-space): machine hit ~2 GiB free. Deleted bench/wikidata/truthy.nt.bz2 (40 GB) + qlever-{100m,synthetic,olympics} datasets/indexes (all git-ignored, regenerable) → 43 GB free. Persistent df watchdog running (Monitor task bs18m4j5v, alerts <12 GB). STILL RECLAIMABLE after dict-spill finishes: /private/tmp/dsp_bench + /private/tmp/dspbench (~30 GB, the agent's active 100M scratch — do NOT delete while it runs). EVERY future benchmark/dataset agent brief MUST: check free space first, scratch in /private/tmp, DELETE its datasets/indexes on completion, cap dataset size. Never delete tracked bench scripts/.gitignore/Qleverfile.

## IN FLIGHT NOW (Opus): dict-spill finisher (8B gate; RSS numbers reliable, wall-time may be CPU-contended → re-verify on quiet machine if borderline), zk-trace-engine (module B), readme-perf (README perf update + add parsing/serialisation, worktree ../sparq-readme), zk-soundness-audit WORKFLOW (run wf_8117e4d6-69a, read-only adversarial audit of zk-compose+sparq-zk prover/verifier → returns report to commit at research/zk-soundness-audit.md). neon-intersect still parked.

## RESUME PROTOCOL if THIS session dies: (1) re-auth AWS + verify no sparq-hw-validation instance running; (2) check each worktree STATUS.md + `git log main..HEAD`; (3) gate+push merge-ready branches one at a time; (4) relaunch any 0-commit branches.

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
