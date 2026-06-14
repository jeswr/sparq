# AGENTS.md — sparq

> A README for coding agents. If you are an AI agent working on or with this repo, read this first.

## What sparq is

sparq is a from-scratch **RDF triplestore and SPARQL 1.1 engine in Rust** — dictionary-encoded, six sorted permutation indexes, parallel + streaming execution, RDFS/OWL-RL/N3 inference, an out-of-core (mmap) mode with a compressed on-disk format, a WebAssembly build, and a W3C-conformant HTTP server. The engine is published across several surfaces:

- **Rust crates** (crates.io): `sparq-core`, `sparq-engine` (core), `sparq-cli`, `sparq-server`, plus opt-in capability crates (`sparq-reason`, `sparq-shacl`, `sparq-geo`, `sparq-text`, `sparq-rsp`, `sparq-hdt`, `sparq-solid`, ...).
- **npm**: `@jeswr/sparq` — RDF/JS-typed API over the wasm build, zero runtime deps.
- **PyPI**: `sparq` — pyo3/maturin bindings.

Status: experimental research engine; the API is unstable.

## Skills — how to USE sparq from your code

Usage instructions for each public surface are packaged as Agent Skills under [`skills/`](skills/) (the [agentskills.io](https://agentskills.io) open format — `name`/`description` frontmatter + Markdown). Read the one that matches the surface you are integrating:

Read [`skills/SKILL.md`](skills/SKILL.md) first — it is the router skill that lists every surface and points you at the right one. The main entry points:

- [`skills/sparql-query/SKILL.md`](skills/sparql-query/SKILL.md) — run SPARQL from Rust (`sparq-core` + `sparq-engine`).
- [`skills/data-formats/SKILL.md`](skills/data-formats/SKILL.md) — parse/load RDF (Turtle/N-Triples/N-Quads/TriG, HDT) into a Graph.
- [`skills/cli/SKILL.md`](skills/cli/SKILL.md) — the `sparq` CLI (query, mmap build/query, reason, bench).
- [`skills/http-server/SKILL.md`](skills/http-server/SKILL.md) — the SPARQL 1.1 Protocol HTTP server.
- [`skills/javascript-wasm/SKILL.md`](skills/javascript-wasm/SKILL.md) — the `@jeswr/sparq` npm package.
- [`skills/python/SKILL.md`](skills/python/SKILL.md) — the `sparq` Python package.

The capability surfaces (reasoning, SHACL, full-text, vector, GeoSPARQL, streaming RSP-QL, ZK query proofs, MPC, GenAI retrieval) each have their own `skills/<surface>/SKILL.md` — the router in [`skills/SKILL.md`](skills/SKILL.md) enumerates them.

If your agent runtime supports the Agent Skills standard, these load via progressive disclosure (name+description first, body on demand). If not, just read the SKILL.md files directly.

> Note: `.claude/skills/` (separate tree) holds INTERNAL skills for agents working *on* the engine's source (parsing perf, ZK circuits, etc.), not usage docs. Do not confuse the two.

## Working on this repo (contributor agents)

- Build: `cargo build --workspace`. Test: `cargo test --workspace`.
- Lint is enforcing (CI gates on it): `cargo clippy --workspace --exclude sparq-py --all-targets -- -D warnings` and `cargo fmt --check` must pass. Run clippy over the **full workspace**, not a single crate — feature unification surfaces lints that an isolated-crate check misses. (`sparq-py` is excluded because it needs the Python/maturin toolchain.)
- The core crates (`sparq-core`, `sparq-engine`) must stay dependency-free of the opt-in capability crates, and the wasm build must not regress — both are enforced in CI.
- Conformance: the W3C SPARQL + inference suites must stay green and are **ratcheted** (the committed floor only goes up) — see `conformance-report.md` / `inference-conformance-report.md` and the CI ratchet. Performance is gated the same way against a best-ever floor (`bench/perf-baseline.json`).
- **Merge discipline:** the gate for landing any change is *full-workspace clippy + `cargo test` + the conformance/perf ratchets*, all green. When work is done in parallel git worktrees, gate and merge **one branch at a time** with a full re-gate between merges; never edit `.beads/` files inside a worktree (it conflicts at merge — `bd export` regenerates the JSONL).

## MAINTENANCE RULE (REQUIRED — read before changing any public surface)

**When you change a public API, update the matching skill in the SAME change (same commit/PR).** A "public API" means any of:

- a `pub` item in a crate's public surface (a published crate's exported types, traits, functions, or their signatures);
- a CLI flag, subcommand, or its behavior in `sparq-cli`;
- an HTTP route, query/body parameter, or response shape in `sparq-server`;
- a Python binding (the `sparq` package) or a JS/RDF-JS binding (`@jeswr/sparq`).

Then edit the corresponding `skills/<surface>/SKILL.md` (sparql-query / data-formats / cli / http-server / python / javascript-wasm) so its instructions and examples still compile and run against the new surface. Do not split this across a follow-up PR — a skill that documents a removed flag or a changed signature is worse than no skill. If the change spans surfaces (e.g. a new query option exposed in both the CLI and the HTTP server), update every affected `SKILL.md`. Keep each `SKILL.md` body under ~500 lines; move long flag/route tables and runnable examples into that skill's `references/` and `scripts/`.

If you add a brand-new public surface, add a new `skills/<surface>/` (dir name == the skill's `name` frontmatter) and link it from the list above and from the README.

## Task tracking — beads, not markdown TODOs

This repo tracks work in **beads** (`bd`, a git-native dependency-graph issue tracker; the committed source-of-record is `.beads/issues.jsonl`). Rules for any agent working here:

- **Do NOT write TODO/FIXME into markdown or leave them in `TODO.md` files.** Capture future work as a bead instead.
- **When you identify follow-up/future work, create a bead for it** (from the repo root, with `bd` on your PATH):
  ```sh
  bd create "<imperative title>" -t <task|bug|feature|chore|spike> -p <0-4> -l <area:crate,kind:...> -d "<what + why + where>"
  ```
  This writes the shared Dolt DB (exclusive-lock-serialized — safe across parallel agents). For the rationale behind a *deferred* task, put it in the bead's `-d` description or `--design` field so the bead is self-contained. **Never edit `.beads/issues.jsonl` (or any `.beads/` file) by hand** — it causes merge conflicts; `bd export` regenerates it.
- Run `bd ready` to see unblocked work; close with `bd close <id>`.

**Beads session-context hook.** `.claude/settings.json` registers a `SessionStart` hook (`scripts/bd-session-context.sh`) that injects a concise bead snapshot — the `bd ready` list + open count — at the start of every Claude Code session, so a new or post-compaction session recovers the task state automatically. It's a graceful no-op when `bd` isn't installed or `.beads/` is absent. We deliberately do **not** use beads' own `bd setup claude` / `bd prime` injection: that path ships generic rules ("do not use TaskCreate / MEMORY.md") that conflict with this harness's task tracker and auto-memory, and it duplicates the beads guidance already in this file. The hook ships only the useful, non-conflicting part. (A committed `.claude/settings.json` hook takes effect on the *next* session start / `/hooks` reload, not the current session.)

## Orchestration — delegate to sub-agents + run a continuous bead loop

If you are an ORCHESTRATING agent (driving multi-step work on this repo), two standing rules:

1. **Delegate every substantive task to a sub-agent in an isolated git worktree** — implementation, research, test/triage, merge-conflict resolution, doc writing, AND the heavy verification/gating of a change. The orchestrator keeps only cheap glue: sequencing, `git merge`/`push`, worktree add/remove, bead bookkeeping (`bd close`/`export`), and reading one-line gate results. Do NOT run builds, toolchain installs, CI-log spelunking, or end-to-end gate runs in the main thread when an agent can — keep the orchestrator context small. Parallelise independent agents; serialise only CPU-heavy wall-clock *measurements* (those need a quiet box).
2. **Continuous loop:** iteratively `bd ready` → spin up sub-agents (parallel, worktree-isolated, smallest context-independent briefs) to address the ready beads → gate + merge one at a time → re-check `bd ready` → repeat. Don't wait to be prompted bead-by-bead. Sequence beads that touch the same files; respect dependency edges (`bd dep`).

Each sub-agent brief must: work in its own worktree, NOT push/merge (the orchestrator does), gate in-worktree (scope tests to affected crates; the orchestrator does the authoritative full-workspace gate at merge), create beads for any discovered work (`bd create`, never edit `.beads/`), and report a concise result. See the per-batch re-evaluation checklist below to decide which gates a given change must re-run.

## Post-batch re-evaluation checklist — what to re-run after a change

After a batch of changes, re-run only the evaluations whose inputs changed — on top of the base gate, which is always required. The base gate is full-workspace `clippy -D warnings` **plus full-workspace `cargo test`**: a sub-agent may scope its in-worktree test run to the affected crates for speed, but the orchestrator's authoritative pre-merge gate runs `cargo test` across the **whole workspace** (feature-unification and cross-crate regressions only surface workspace-wide). Map change → evaluation:

| If the change touches… | Re-run |
|---|---|
| a parser (turtle/nt/nq/trig, `sparq-core` parse, `spargebra`) | W3C SPARQL + rdf-turtle conformance; the chunked-vs-serial parser oracle; `sparq-bench fuzz` |
| query execution / operators (`sparq-engine` exec/optimizer) | full conformance ratchet; the operator-coverage bench; per-builtin error table |
| the reasoner (`sparq-reason`, rules, closure) | inference conformance ratchet; incremental==batch property tests; LUBM entailed tier |
| a public API (`pub` item / CLI flag / HTTP route / Py/JS binding) | update the matching `skills/<surface>/SKILL.md` (REQUIRED, same change); the surface's tests |
| `sparq-wasm` / the wasm graph | `scripts/wasm-deps-guard.sh`; `wasm-pack test --node`; the `wasm_bundle_bytes` size gate |
| Cargo dependencies (`Cargo.toml`/`Cargo.lock`) | `cargo audit` + `cargo deny check` + regenerate the SBOM (supply-chain gate) |
| the ZK verifier / circuits (`sparq-zk`, `sparq-zk-compose`) | `forge_gates` + `differential_fuzz`; the gate-count snapshot; re-open the soundness audit |
| SHACL (`sparq-shacl`) | the W3C SHACL conformance ratchet (core ≥98, sparql ≥5) |
| storage/encoding (`sparq-core` store/dict/compress, mmap, dict-spill) | the deterministic perf-gate metrics; byte-identity differentials; coverage with `--features dict-spill` |
| anything merged | the per-crate coverage ratchet + test-presence gate (`scripts/coverage*.py`) |

(Keep this table in sync as gates are added.)

## Documents must stay current — research records become architecture docs

A document must never describe the code as it ISN'T. Concretely:

- **No "not implemented" / "TODO" / "future work" statements left standing in a doc when they describe a real gap** — that is a disguised markdown TODO. Convert it to a **bead** and edit the doc to either delete the claim or replace it with a forward reference to the bead id. (If the feature IS now implemented, the statement is stale — fix the doc.)
- **A `research/` design record is provisional.** Once its design is implemented, it should graduate: either rewrite it into an **architecture document** describing what the code actually does (and where), or fold the durable parts into the relevant crate `README.md` / `skills/<surface>/SKILL.md` and delete the speculative design. A research doc that still says "we will…" or "X is not implemented" about shipped code is a bug in the docs.
- When you touch code, check the docs that describe it; if your change makes a doc statement false (in either direction), update the doc in the SAME change.

## Upstream blockers — roll your own, then contribute back

When a feature or a performance goal is **blocked by an upstream dependency** (a parser that rejects valid input, a missing API, a slow hot path), do NOT just mark it "unsupported/blocked-upstream" and stop. Instead:

1. **Vendor a local copy** of the upstream code (under `vendor/` or as a forked crate via `[patch.crates-io]`, as already done for `spargebra`), implement the feature/fix there, and ship it so sparq is unblocked.
2. **Open an issue + PR upstream** offering the change. Record the upstream issue/PR URL in the relevant bead and in the vendored copy's `*-PATCHES.md` (as `vendor/spargebra/SPARQ-PATCHES.md` does).
3. **Keep the PR live:** if you later change that vendored code, update the open PR; if the upstream PR was already merged/closed, open a new one for the delta.
4. A `blocked-upstream` bead is therefore a signal to roll-your-own + contribute, NOT a dead end. When the local implementation lands, the bead is unblocked.

## Proactively maintain this file (and the skills)

Do NOT wait to be told. Whenever you notice a **repeated behaviour, a standing rule, a convention, or a hard-won lesson** that future agents should follow, add it to this `AGENTS.md` (or the matching `skills/<surface>/SKILL.md`) as part of the same work — the same way you'd capture a follow-up as a bead. This file is the durable home for "how we work here"; keep it current without prompting.

## Orchestration cadence — background by default

When orchestrating, run agents and long shell commands (builds, gates, fetches, watches) **in the background** by default, and parallelise independent work. A foreground/blocking call stalls the loop and prevents picking up new instructions or other ready beads meanwhile. Reserve foreground for genuinely sequential, short glue. (Continues the delegation + continuous-loop rules above.)

**Reconcile finished-but-unnotified agents.** Completion notifications can occasionally not surface. Don't wait indefinitely on a background agent: if it's gone quiet, check the real state — `git worktree list`, the branch's last commit (`git log -1 <branch>`), and whether any of its processes are still alive (`pgrep -af cargo`). A committed branch + no live process = done; verify with your own gate and merge. (Trust ground truth over the notification stream.)

## Contribution workflow — PRs, reviews resolved, the `ci-summary` gate

Changes land on `main` via **pull requests**, not direct pushes (direct push is reserved for the rare hotfix the team agrees on). For every PR:

1. Branch → open a PR (`gh pr create`). Request review, **including GitHub Copilot** code review.
2. **Address and RESOLVE every review comment** before merge — especially Copilot's. "Resolve" means: make the change or reply with the reason it's declined, and mark the conversation resolved. An unresolved thread blocks merge.
3. CI must be green: a single **`ci-summary`** check aggregates every other workflow's result and passes ONLY when they all pass. It is the one required status check for branch protection.
4. Merge only when: `ci-summary` is green AND all review threads are resolved. Then squash/merge and delete the branch.

Branch protection (owner-set, out-of-repo) enforces this: require `ci-summary`, require conversation resolution, require the Copilot/CodeQL review. The repo documents the required set in `docs/branch-protection.md`.

**Security & quality gating:** new security/quality regressions must not merge. CodeQL (SAST) + `cargo clippy -D warnings` + `cargo-deny` + the coverage/conformance ratchets all feed `ci-summary`. Keep the GitHub **code-scanning** alert count at zero — SHA-pin every action (`uses: owner/action@<full-sha> # vX.Y.Z`), and resolve/triage Scorecard + CodeQL alerts as they appear.

## Automated review — roborev on every commit, including in worktrees

Every commit is auto-reviewed by **roborev**: a `.git/hooks/post-commit` hook enqueues a review job to the local roborev daemon. The reviewer agent is **codex — a deliberately non-Anthropic model**, so the engine is never reviewed by the same model family that wrote it. Git worktrees share the main repo's `.git/hooks`, so commits made by worktree sub-agents are reviewed too. Verify the loop is live with `roborev list` (recent jobs, all `done`) and `~/.roborev/post-commit.log` (the per-repo enqueue trail, incl. `sparq-wt-*` worktrees).

Findings must be **addressed, not merely gathered.** A finding is resolved in exactly one of two ways — **never** by merging the branch:

1. **the flagged code changed** — the lines the reviewer objected to were rewritten or removed, so the finding no longer describes anything that exists on `main` (verify against current HEAD, *not* the WIP SHA the review was filed on); or
2. **explicit triage** — it is fixed, beaded as real-but-deferred work (`bd create`), or closed with a written reason (`roborev close <id>`) when it's a false positive.

Squashing a branch into `main` does **not** address its findings: if the flagged code survived the squash it is still live — just orphaned onto a SHA that no longer appears in `git log`, which is *worse* than an open finding on a current commit because it's invisible. So before merging a branch, reconcile its roborev findings against current `main` HEAD and dispose of each (fix / bead / close-with-reason). Periodically run `roborev list --open` and drain the backlog; don't let unaddressed findings accumulate.

## Monitor CI after every push to main — and fix red

A local gate is necessary but NOT sufficient: CI runs on a clean checkout with different toolchains/targets/feature-unification than your incrementally-built box, so it catches things your local gate cannot (e.g. a `cargo test --workspace` that includes a crate your local gate `--exclude`s; a wasm-target-only lint; an action container with an older cargo). Therefore: **after pushing to main, watch the CI runs to completion and fix any failure immediately** — a red main is a stop-the-line condition. `gh run watch <id> --exit-status` (background) or `gh run list --branch main`; on failure `gh run view <id> --log-failed`. Roll the fix into the next push and re-watch. Do not pile more pushes onto a red main.

## Maximise parallelism — many agents + extra compute

Aim to keep **as many sub-agents running in parallel as possible** without (a) conflicting work or (b) competing for the same resource. Partition by **file/area ownership** so no two concurrent agents edit the same file (give each a distinct crate/dir; wire shared files — `ci-bench.sh`, `benchmarks.toml`, workflows, `AGENTS.md` — centrally afterward). The only things to serialise: two agents that must touch the same file, and CPU-heavy **wall-clock measurements** (benchmarks need a quiet box — read-only/light analysis parallelises freely). Don't sit at one or two agents when the `bd ready` set has more independent work — fan out.

**Extra compute:** you MAY launch additional **EC2 instances** to run work in parallel (e.g. the heavy full-scale benchmark tiers, large fuzzing, coverage). Follow the EC2 rules in memory: orphan-proof self-terminate is MANDATORY (instance-initiated-shutdown=terminate + a user-data watchdog), ~$5/day cap, tag `purpose=sparq-bench`/`sparq-dev`, **never touch the prod or dev boxes**, and a fresh session must orphan-check first. **If AWS SSO has expired** (`aws sts get-caller-identity --profile pss` fails), you cannot launch instances — raise a `needs:user` item (below) asking the user to re-auth, and keep surfacing it until they do.

## Inputs needed from the user — the `needs:user` bead queue

Anything blocked on a **human decision, credential, or out-of-repo action** (re-auth SSO, enable GitHub Pages, approve a destructive step, a product decision) is tracked as a **bead labelled `needs:user`**, with the exact ask in the description. This is the standard human-in-the-loop pattern (a dedicated review/blocked-on-human queue) mapped onto the existing tracker, so nothing waiting on the user is lost in the chat scroll. List them with `bd list -l needs:user`. An orchestrator should **surface the open `needs:user` items in its responses** (concise, at the end) until the user resolves them, so they can be actioned whenever the user is next available.

## No hard-coded performance numbers

Do not bake benchmark numbers (MB/s, ×-faster, recall, gate counts, latencies) into markdown. Reference the **generated structured data** instead (the benchmark harnesses emit JSON; CI publishes results). If you cite a number, cite where it was generated.

## Repository hygiene — where things live (READ THIS; it keeps the repo clean by default)

Everything you produce has exactly **one** correct home. Putting it anywhere else creates the cruft that forces periodic "clean-up runs" — so don't create it in the first place.

- **Tasks / TODOs / follow-ups / "future work" → a bead.** Never a `TODO`/`FIXME`/`XXX` marker in a markdown file, never a `TODO.md`, never a `- [ ]` checklist of pending work in a tracked doc. If you catch yourself writing "we should later…", run `bd create` (see the beads section above) and move on. Code-comment `TODO`s are discouraged too — prefer a bead and reference its id.
- **Durable knowledge → `AGENTS.md` / `CLAUDE.md`, a `skills/<surface>/SKILL.md`, a crate `README.md`, or a `research/` design record — whichever fits.** Workspace-wide conventions and contributor rules go here in `AGENTS.md` (Claude Code also auto-reads `CLAUDE.md`, which just points here). Usage knowledge goes in the matching skill. Per-crate caveats go in that crate's `README.md`. Design rationale and measured verdicts go in `research/` — or, for the rationale behind a specific deferred task, in that bead's description / `--design` field.
- **Do NOT commit narrative scratch docs.** No `HANDOVER*.md`, no `SESSION*.md`, no "current state" / "what I'm doing now" / progress-log markdown in the repo. Session and orchestration state belongs in beads (for work) or in your own un-tracked notes — never in a tracked file. The only living operational markdown allowed is **genuine reference** (a runbook, the benchmark catalog) and **generated reports** (the CI-published perf/conformance data) — not a story about a session.
- **No hard-coded performance numbers in markdown** (restated; see the section above): cite the generated structured data, not a baked-in figure.

Honour these homes and the repo never accumulates stale TODO lists or handover docs — no clean-up pass is ever needed.

## Public-API → SKILL.md maintenance rule

Important enough to state twice: see **MAINTENANCE RULE (REQUIRED)** near the top. In short — when you change any public API (`pub` item, CLI flag, HTTP route, Python/JS binding), update the corresponding `skills/<surface>/SKILL.md` in the SAME change. The surface→skill map is in [`skills/SKILL.md`](./skills/SKILL.md).
