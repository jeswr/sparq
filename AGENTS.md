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

- [`skills/rust-api/SKILL.md`](skills/rust-api/SKILL.md) — `sparq-core` + `sparq-engine` from Rust.
- [`skills/cli/SKILL.md`](skills/cli/SKILL.md) — the `sparq` CLI (query, mmap build/query, reason, bench).
- [`skills/http-server/SKILL.md`](skills/http-server/SKILL.md) — the SPARQL 1.1 Protocol HTTP server.
- [`skills/js/SKILL.md`](skills/js/SKILL.md) — the `@jeswr/sparq` npm package.
- [`skills/python/SKILL.md`](skills/python/SKILL.md) — the `sparq` Python package.

If your agent runtime supports the Agent Skills standard, these load via progressive disclosure (name+description first, body on demand). If not, just read the SKILL.md files directly.

> Note: `.claude/skills/` (separate tree) holds INTERNAL skills for agents working *on* the engine's source (parsing perf, ZK circuits, etc.), not usage docs. Do not confuse the two.

## Working on this repo (contributor agents)

- Build: `cargo build --workspace`. Test: `cargo test --workspace`. Lint is enforcing: `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --check` must pass.
- The core crates (`sparq-core`, `sparq-engine`) must stay dependency-free of the opt-in capability crates, and the wasm build must not regress — both are enforced in CI.
- Conformance: the W3C SPARQL suites must stay green (see `conformance-report.md`).

## MAINTENANCE RULE (REQUIRED — read before changing any public surface)

**When you change a public API, update the matching skill in the SAME change (same commit/PR).** A "public API" means any of:

- a `pub` item in a crate's public surface (a published crate's exported types, traits, functions, or their signatures);
- a CLI flag, subcommand, or its behavior in `sparq-cli`;
- an HTTP route, query/body parameter, or response shape in `sparq-server`;
- a Python binding (the `sparq` package) or a JS/RDF-JS binding (`@jeswr/sparq`).

Then edit the corresponding `skills/<surface>/SKILL.md` (rust-api / cli / http-server / python / js) so its instructions and examples still compile and run against the new surface. Do not split this across a follow-up PR — a skill that documents a removed flag or a changed signature is worse than no skill. If the change spans surfaces (e.g. a new query option exposed in both the CLI and the HTTP server), update every affected `SKILL.md`. Keep each `SKILL.md` body under ~500 lines; move long flag/route tables and runnable examples into that skill's `references/` and `scripts/`.

If you add a brand-new public surface, add a new `skills/<surface>/` (dir name == the skill's `name` frontmatter) and link it from the list above and from the README.


## Task tracking — beads, not markdown TODOs

This repo tracks work in **beads** (`bd`, a git-native dependency-graph issue tracker; the committed source-of-record is `.beads/issues.jsonl`). Rules for any agent working here:

- **Do NOT write TODO/FIXME into markdown or leave them in `TODO.md` files.** Capture future work as a bead instead.
- **When you identify follow-up/future work, create a bead for it:**
  ```sh
  cd /home/ubuntu/sparq && /home/ubuntu/.local/bin/bd create "<imperative title>" -t <task|bug|feature|chore|spike> -p <0-4> -l <area:crate,kind:...> -d "<what + why + where>"
  ```
  This writes the shared Dolt DB (exclusive-lock-serialized — safe across parallel agents). **Never edit `.beads/issues.jsonl` (or any `.beads/` file) by hand** — it causes merge conflicts; `bd export` regenerates it.
- Run `bd ready` to see unblocked work; close with `bd close <id>`.

## No hard-coded performance numbers

Do not bake benchmark numbers (MB/s, ×-faster, recall, gate counts, latencies) into markdown. Reference the **generated structured data** instead (the benchmark harnesses emit JSON; CI publishes results). If you cite a number, cite where it was generated.

## Public-API → SKILL.md maintenance rule

When you change a public API — any `pub` item in a crate's public surface, a CLI flag, an HTTP route, or the Python/JS bindings — **update the corresponding `skills/<surface>/SKILL.md` in the SAME change** so the usage docs never drift. The map is in [`skills/SKILL.md`](./skills/SKILL.md).
