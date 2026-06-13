# Dependency-Aware Task Tracking for AI Coding Agents — Beads (`bd`)

> Operational guide. Researched & **empirically verified on this box** (Linux aarch64, Node 20, npm 10, Go absent) on 2026-06-13. All commands below were run against `@beads/bd` v1.0.5. Forward-looking notes tagged `[OPUS-4.8]`.

## TL;DR (read this)

- **Use beads.** It is the right tool for *this* repo: a dependency graph of issues stored as git-committed JSONL, with `bd ready` computing the unblocked work-set offline in ms. That maps directly onto our merge-queue + roborev-backlog reality where "task X is blocked until PR Y merges" is the dominant relationship — something flat TODO.md and the agent's built-in todo tool cannot express.
- **Install is trivial here (no Go needed):** the npm package ships a postinstall that downloads the prebuilt Go binary for `linux/arm64`. Verified working: `npx @beads/bd@latest version` → `bd version 1.0.5`. See "Install".
- **How to adopt:** one `.beads/` DB at repo root, prefix `sq`. Model the 25 crates + cross-cutting concerns as **labels** (`area:sparq-core`, `area:zk`, …), big initiatives (MPC, ZK build-out, serve-wave) as **epics** (`-t epic` + `parent-child`), the merge queue as **`blocks` edges** ("re-run roborev job" `blocked` by "PR merges"), and roborev findings as tasks linked **`discovered-from`** their source. Commit `.beads/issues.jsonl`; the Dolt dir is gitignored.
- **One caveat, mandatory:** run `bd init` with `--skip-agents --skip-hooks` so it does **not** overwrite our `CLAUDE.md` or install git hooks (default behavior does both). See "Install step 2".
- **Honest downside:** beads is young (1.x, "API stability not guaranteed"), the backend is Dolt (an embedded SQL/git database — heavier than a flat file), and a 132 MB binary. If you want zero new infra and never parallelize agents, plain TODO.md is fine. Beads earns its keep the moment you have >1 agent and real cross-task blockers — which we do.

---

## 1. Install (verified for THIS box: Linux aarch64, no Go)

`go` is not installed; `node`/`npm` are. The npm package `@beads/bd` is published by `steveyegge` (maintainer email `steve.yegge@gmail.com`), latest **1.0.5**, MIT, **deps: none**. Its `postinstall.js` reads `os.platform()`/`os.arch()`, maps `linux`+`arm64` → the GitHub release asset **`beads_<ver>_linux_arm64.tar.gz`** (which exists for v1.0.4 and v1.0.5), downloads, untars, and chmods the native `bd` binary. `package.json` declares `os:[darwin,linux,win32,android]`, `cpu:[x64,arm64]` — so aarch64 Linux is a first-class target. **Verified: `bd version 1.0.5 (6a3f515ce)` runs.**

### Recommended: npm (no Go, no sudo)
`npm config get prefix` is `/usr` on this box, so a bare `npm i -g` needs root. Two clean options:

```bash
# Option A — project-local (preferred; pins the version, no global state)
cd /home/ubuntu/sparq
npm install @beads/bd            # installs node_modules/.bin/bd  (postinstall pulls linux_arm64 binary)
alias bd='/home/ubuntu/sparq/node_modules/.bin/bd'   # or add node_modules/.bin to PATH

# Option B — global without sudo (user prefix)
npm config set prefix "$HOME/.npm-global"
export PATH="$HOME/.npm-global/bin:$PATH"   # add to ~/.bashrc
npm install -g @beads/bd
bd version

# Option C — zero-install, ad-hoc
npx @beads/bd@latest <command>
```

### Fallback: direct prebuilt binary (also Go-free, if npm registry is unavailable)
```bash
cd /tmp
gh release download v1.0.5 -R steveyegge/beads -p 'beads_1.0.5_linux_arm64.tar.gz'   # or curl the asset URL
tar -xzf beads_1.0.5_linux_arm64.tar.gz
install -m755 bd "$HOME/.local/bin/bd"      # ensure ~/.local/bin on PATH
bd version
```
The official `curl … install.sh | bash` and `brew install beads` also exist, but the npm route is cleanest here and pins a version. **Do not** try the Rust port (`Dicklesworthstone/beads_rust`) — it's a separate third-party project, not the canonical tool.

> [OPUS-4.8] Pin the version (`@beads/bd@1.0.5`) in any agent brief; it's pre-1.0-stability and 67 versions have shipped. Re-verify `bd version` after upgrades before trusting flag names.

---

## 2. Initialize in our repo (DO NOT let it clobber CLAUDE.md)

Default `bd init` writes a `CLAUDE.md` (with beads integration), `.claude/settings.json`, installs a Beads skill, and installs **git hooks** (post-checkout, pre-push, prepare-commit-msg, post-merge, pre-commit). We already have a curated `CLAUDE.md`/memory setup and our own hooks discipline. Suppress all of it:

```bash
cd /home/ubuntu/sparq
bd init --prefix sq --skip-agents --skip-hooks --quiet
```

This creates `.beads/` with: `config.yaml`, `metadata.json`, `issues.jsonl` (the committed export), and `embeddeddolt/` (the Dolt DB). The auto-generated `.beads/.gitignore` already excludes `embeddeddolt/`, `dolt/`, sockets, locks, daemon files — so **only `.beads/issues.jsonl` + `config.yaml` get committed**. Add `.beads/issues.jsonl` to git; treat it as the source-of-record for review/merge.

---

## 3. Data model (verified via `bd show --json` / `bd export`)

**Issue fields:** `id` (hash-based, e.g. `sq-gq8` — collision-free across branches/agents), `title`, `status`, `priority` (int 0–4, 0=highest), `issue_type`, `created_at`, `updated_at`, plus on write: `description`, `design`, `notes`/`context`, `acceptance`, `assignee`, `labels[]`, `estimate`, `due`, `defer`, `external_ref`, `spec_id`, `metadata` (arbitrary JSON), `dependencies[]`. Counts surfaced: `dependency_count`, `dependent_count`, `comment_count`.

**Issue types** (`bd types`): `task` (default), `bug`, `feature`, `chore`, `epic`, `decision` (ADR), `spike`, `story`, `milestone`.

**Statuses** (`bd statuses`): `open` (active), `in_progress` (wip), `blocked` (wip), `deferred` (frozen), `closed` (done), `pinned` (frozen, never auto-decays), `hooked` (attached to an agent).

**Dependency edge types** (authoritative, from `bd link --help`): **`blocks` | `tracks` | `related` | `parent-child` | `discovered-from`** (default `blocks`). Semantics that matter:
- `bd link <A> <B>` (default) means **B blocks A** → A is blocked-by B. Mnemonic: second arg is the prerequisite. Verified: after `bd link <B> <A>`, `bd ready` listed only A and reported B as "← blocked by A".
- `parent-child` = epic/subtask hierarchy (also via `--parent`).
- `discovered-from` = "I found this while doing X" — the agent breadcrumb trail.
- `related` / `tracks` = soft links, no readiness effect.

**Ready computation:** `bd ready` returns open issues with **no active blockers**, excluding `in_progress`, `blocked`, `deferred`, `hooked`. It walks `blocks` edges transitively, offline, in ~ms (the FAQ claims ~10ms, no network). `--explain` prints the reasoning ("Reason: no blocking dependencies / Unblocks: N issues" vs "← blocked by <id>"). Verified empirically.

**Storage / git-native:** backend is **Dolt** (a versioned SQL DB with cell-level merge + branching), embedded under `.beads/embeddeddolt/` by default (single-writer file lock) or `.beads/dolt/` in server mode (multi-writer). `.beads/issues.jsonl` is the **export for git/interchange — explicitly NOT the source of truth or a full backup.** Each line is one issue with inline `dependencies[]` (verified shape: `{"_type":"issue","id":"sq-3dc",…,"dependencies":[{"depends_on_id":"sq-gq8","type":"blocks",…}]}`). Hash IDs mean two branches creating issues never collide; JSONL merges line-wise; Dolt merges concurrent field edits cell-by-cell. `bd export` / `bd import` move between DB and JSONL.

**Compaction / "molting":** there is no biological-style molt; it's archival hygiene. `bd compact` squashes old Dolt commits (keeps recent ones via cherry-pick). `bd admin compact --days N` does *semantic* compaction (summarizing closed issues to save context). `bd gc` runs DECAY (delete closed issues older than N days, default 90) → COMPACT → Dolt GC. `pinned` issues survive decay. Use `--dry-run` first.

**MCP / agent integration:** beads ships an MCP server ("dedicated MCP server with auto workspace detection") and `bd setup <claude|cursor|aider|codex|…>`; maturity is thin in docs. For us the CLI + `--json` is enough; skip MCP for now. `[OPUS-4.8]` revisit MCP only if we want the agent to call beads as a tool rather than via Bash.

---

## 4. `bd` cheat-sheet (the commands we'll actually use)

Assume `bd` on PATH (or the alias). Use `--json` in agent scripts; output is stable and `jq`-friendly.

```bash
# --- setup (once) ---
bd init --prefix sq --skip-agents --skip-hooks --quiet   # init without clobbering CLAUDE.md/hooks

# --- create work ---
bd create "Wire neon-intersect into engine" -t task -p 1 \
    -l area:sparq-engine,perf -d "…" --design-file - --external-ref gh-123   # full create
bd q "Quick task title"                                  # quick-capture, prints ID only
bd create "Re-run roborev job 4471" -p 2 \
    --deps blocks:sq-ab12 -l roborev                     # create already-blocked by sq-ab12
bd create "MPC M3" -t epic -p 1 -l area:sparq-mpc        # an epic

# --- structure / dependencies ---
bd link <A> <B>                       # B blocks A  (B is the prerequisite). default type=blocks
bd link <A> <B> --type related        # soft link
bd link <child> <parent> --type parent-child   # or use --parent on create
bd dep add <child> <parent>           # same as link
bd epic …                             # epic management
bd graph                              # render the dependency graph

# --- the agent loop ---
bd ready --json                       # claimable work (no active blockers)
bd ready --explain                    # human: why ready / why blocked
bd ready --assignee agentA --claim --json   # atomically claim first ready item for an agent
bd update <id> --claim                # claim + set in_progress
bd show <id> --json                   # full detail + audit trail
bd update <id> -p 0 --assignee me     # edit fields
bd note <id> "found edge case in …"   # append a note (breadcrumb)
bd close <id> --reason "merged in PR #321"   # complete

# --- views ---
bd list --status open -l area:sparq-zk        # filtered list
bd ready --exclude-type epic                  # leaf work only
bd blocked                                    # what's stuck and on what
bd status                                     # DB overview / stats
bd query "<expr>"                             # query language;  bd search "<text>"
bd label add <id> area:sparq-core             # labels;  bd label list-all

# --- data / hygiene ---
bd export > .beads/issues.jsonl       # DB → committed JSONL (also runs on hooks if enabled)
bd import .beads/issues.jsonl         # JSONL → DB (after a clone/pull)
bd gc --dry-run                       # preview decay+compact;  bd gc --older-than 90 to apply
```

---

## 5. The scheme we adopt for the sparq repo

**One DB, prefix `sq`, at repo root.** Commit `.beads/issues.jsonl` + `config.yaml`; never commit `embeddeddolt/`.

**Crates/areas → labels, not epics.** 25 crates is too many for epics. Use a flat label namespace, multi-label as needed:
- `area:<crate>` — e.g. `area:sparq-core`, `area:sparq-zk`, `area:sparq-mpc`, `area:sparq-hdt`, `area:sparq-serve`. (We have: sparq-bench, -cli, -conformance, -core, -engine, -geo, -gpu, -hdt, -introspect, -mpc, -nlq, -parse, -py, -reason, -rsp, -serve, -server, -shacl, -sim, -solid, -text, -vectors, -wasm, -zk-compose, -zk.)
- `kind:` cross-cutting: `kind:perf`, `kind:correctness`, `kind:docs`.
- `roborev` for review-backlog items; `quota-failed` for jobs that need re-running after quota reset.
- `js` for the JS/WASM port side.

**Big initiatives → epics (`-t epic`) with `parent-child` to leaf tasks.** Recommended epics: `MPC M3/M4`, `ZK build-out` (+ the 6 critical verifier-soundness findings as children), `serve-wave` (wave-b is done → close it), `HDT+Turtle parse perf`, `neon-intersect`. Use `bd create … --parent <epic-id>` or `bd link <child> <epic> --type parent-child`. Children inherit parent labels unless `--no-inherit-labels`.

**Merge queue → `blocks` edges.** Model "blocked until X merges" as: create the merge/PR as its own task (or `milestone`), and `bd link <downstream-task> <merge-task>` (default `blocks`). When the PR lands, `bd close <merge-task> --reason "merged #NNN"` and the downstream task pops into `bd ready` automatically. This is the single biggest reason to use beads here — it turns our informal "in-flight merge queue" into a queryable ready-set.

**roborev backlog → tasks linked to source.** Each failing-verdict finding becomes a `bug`/`task` labeled `roborev`, linked **`discovered-from`** the originating change/commit task. Quota-failed jobs: `task` labeled `roborev,quota-failed`, `-p 2`, no blocker (immediately ready to re-run). Use `bd note` to record the verdict text.

**Source-linking convention (pick consistently):**
- Code site → put `path/to/file.rs:LINE` in `--description` or `bd note`; tag `area:<crate>`.
- Commit/PR → `--external-ref gh-<num>` (the field is built for `gh-`/`jira-`/Linear refs).
- Design doc → `--spec-id <research/foo.md>` or `--design-file research/foo.md`. We have ~58 `research/*.md`; link the epic to its design doc this way rather than duplicating prose.
- Migrate the ~16 in-code TODO/FIXME markers and per-crate `TODO.md` items as leaf tasks with `discovered-from` the file; then the markdown TODOs can be retired (or kept as thin pointers) to avoid two sources of truth.

**Priority scale (0–4, 0=highest), our convention:**
- `P0` shipping-blocker / correctness regression / broken build.
- `P1` current-milestone work (active epics: MPC M3/M4, ZK soundness fixes).
- `P2` (default) normal backlog, roborev re-runs.
- `P3` nice-to-have / cleanup. `P4` someday/maybe (or use `--defer`).

**Dep-edge conventions:** `blocks` only for *real* ordering (must-merge-first, must-land-API-first). Everything else is priority + labels. Use `parent-child` strictly for epic→leaf. Use `discovered-from` for agent breadcrumbs. Use `related`/`tracks` sparingly — they don't affect readiness and tend to rot.

**Anti-rot rules:** (1) Close tasks the moment the PR merges — stale `blocks` edges silently hide ready work. (2) Don't encode a `blocks` edge you could express as priority; over-linking is the main failure mode. (3) Run `bd gc --dry-run` monthly; let closed issues decay after 90 days (epics/`pinned` survive). (4) One DB only — beads does not support cross-`.beads/` references, so resist per-crate databases.

---

## 6. Driving parallel agents off the ready-set

Each agent: `bd ready --assignee <agent> --claim --json` to atomically grab the first unblocked item (sets `in_progress`, prevents two agents taking the same task) → work → `bd note`/`bd create --deps discovered-from:<parent>` for spin-off work → `bd close --reason …`. Because closing unblocks downstream tasks, the ready-set is self-feeding: the orchestrator just keeps dispatching `bd ready` items until empty. Hash IDs + Dolt cell-merge make concurrent branches/worktrees safe to merge. This is exactly the "smallest context-independent deliverable" subagent pattern in our memory notes, but with the dependency graph deciding *order* instead of the orchestrator hand-sequencing.

---

## 7. Honest tradeoffs vs alternatives

| Option | When it wins | Why not here / downsides |
|---|---|---|
| **Beads (`bd`)** | Multiple agents, real cross-task blockers, want offline `ready` queries, git-native history, breadcrumb trails. **Our case.** | Young (1.x, no API-stability promise; 67 npm versions); Dolt backend is heavier than a flat file (132 MB binary, an embedded SQL/git DB under `.beads/`); learning curve on edge semantics; default `init` clobbers `CLAUDE.md`+hooks (mitigated by `--skip-*`); no cross-database refs. |
| **GitHub Issues** | Human collaborators, public visibility, PR auto-linking, mature/stable. | Requires network for *every* readiness query; dependency graph is weak (task-lists, not first-class `blocks`+`ready`); rate limits; agents pay latency; not git-local. Good as a *mirror* (`bd github …` exists) but poor as the agent's live work-graph. |
| **Plain markdown TODO.md** (what we have) | Zero infra, human-readable, trivially diffable, fine for a single linear worker. | No dependency semantics, no `ready` computation, no atomic claim — two agents collide; "blocked until X" is a prose comment a human must parse; rots fast across 25 crates. We already feel this pain (10+ TODO.md files, no cross-cutting view). |
| **Agent built-in todo tool** | Single task, single session scratchpad. | Ephemeral — dies with the session; no persistence, no graph, no cross-agent sharing. Wrong layer for a multi-day, multi-agent backlog. Keep it for in-session steps; beads for the durable backlog. |

**Verdict:** adopt beads as the durable, cross-agent backlog + merge-queue graph; keep the agent's built-in todo for in-session steps; optionally mirror selected epics to GitHub Issues for human visibility. Retire per-crate `TODO.md` once migrated (or leave one-line pointers to `bd`). The honest cost is a new young dependency and a Dolt directory; the honest benefit is that "what can an agent work on right now, given everything in flight?" becomes a single `bd ready` call instead of tribal knowledge.

---

## Sources

- Canonical repo (transferred org `gastownhall`, owner Steve Yegge): https://github.com/steveyegge/beads — README, `AGENTS.md`, `docs/FAQ.md`
- Docs site: https://steveyegge.github.io/beads/
- npm package: https://www.npmjs.com/package/@beads/bd (v1.0.5, verified installs `linux_arm64` binary)
- Releases / prebuilt binaries: https://github.com/steveyegge/beads/releases (assets `beads_<ver>_linux_arm64.tar.gz`, `..._linux_amd64.tar.gz`, darwin/windows/freebsd/android)
- Go module index (confirms canonical import path): https://pkg.go.dev/github.com/steveyegge/beads
- Third-party Rust port (NOT canonical, do not use): https://github.com/Dicklesworthstone/beads_rust
- Community writeup: https://ianbull.com/posts/beads/
- **Local empirical verification (2026-06-13, this aarch64 box):** `@beads/bd@1.0.5` installed via npm; `bd version` ✓; `bd init`, `bd q`, `bd link`, `bd ready`, `bd ready --explain`, `bd export`, `bd show --json`, `bd create --help`, `bd link --help`, `bd types`, `bd statuses`, `bd compact/gc --help` all run as documented above.
