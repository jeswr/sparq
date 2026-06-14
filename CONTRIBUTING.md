<!-- [OPUS-4.8] Governance: contributor guide (bead sq-rau7). Intentionally thin — AGENTS.md is the source of truth. -->
# Contributing to sparq

Thanks for your interest in `sparq`! This file is deliberately thin: the **single source
of truth for how to work on this repo is [`AGENTS.md`](./AGENTS.md)**. Read it first. It
covers what sparq is, how to use it (the `skills/` tree), and — for contributors — the
build/test/lint gate, the conformance ratchets, merge discipline, and our repository
conventions. This file only adds the human-facing bits.

## Before you start

- Read [`AGENTS.md`](./AGENTS.md), especially **"Working on this repo"** (the build/test/
  lint gate and merge discipline) and the **"Post-batch re-evaluation checklist"** (which
  gates a given change must re-run).
- `sparq` is **experimental and pre-1.0; the API is unstable.** Expect churn.

## The gate — what must be green to land a change

The full definition lives in `AGENTS.md`; in short, a change lands only when **all** of
these pass (CI enforces them — see [`docs/branch-protection.md`](./docs/branch-protection.md)):

- `cargo build --workspace` and `cargo test --workspace`.
- `cargo clippy --workspace --exclude sparq-py --all-targets -- -D warnings`
  (run over the **full workspace** — feature unification surfaces lints a single-crate
  check misses) and `cargo fmt --check`.
- The W3C SPARQL / SHACL / inference **conformance ratchets** and the performance/
  coverage ratchets, all green.
- If your change touches Cargo dependencies: `cargo audit` + `cargo deny check` (and the
  SBOM) once those gates are in place.

Use the [pull-request template](./.github/PULL_REQUEST_TEMPLATE.md) — its checklist is
tied directly to the post-batch re-evaluation table in `AGENTS.md`.

### The conformance-ratchet "never lower" rule

The committed conformance floors (`conformance-report.md`,
`inference-conformance-report.md`, the SHACL floors, `bench/perf-baseline.json`, the
coverage floors) are **ratchets: they only ever go UP.** Never lower a ratchet to make a
change pass — fix the regression instead. If a test newly diverges for a *documented,
spec-justified* reason, record the rationale in the report alongside the divergence
(don't just drop the count). Raising a floor when coverage genuinely improves is
encouraged.

### Changing a public API → update the matching skill

If you change any **public API** (a `pub` item, a CLI flag, an HTTP route, or a Python/JS
binding), update the corresponding `skills/<surface>/SKILL.md` **in the same change**.
This is a required, enforced convention — see the MAINTENANCE RULE in `AGENTS.md`.

## Tasks and TODOs live in beads, not markdown

This repo tracks work in **beads** (`bd`, a git-native dependency-graph issue tracker).
**Do not add `TODO` / `FIXME` markers to code or markdown, and do not create a `TODO.md`
or a checklist of pending work in a tracked doc.** Capture follow-up or discovered work as
a bead instead:

```sh
bd create "<imperative title>" -t <task|bug|feature|chore|spike> -p <0-4> \
  -l <area:crate,kind:...> -d "<what + why + where>"
```

Run `bd ready` to see unblocked work and `bd close <id>` to close. See the "Task
tracking — beads" section of `AGENTS.md` for the full rules (and never hand-edit
`.beads/issues.jsonl`).

## Reporting security issues

**Do not report security vulnerabilities through public issues or PRs.** Use the private
channels in [`SECURITY.md`](./SECURITY.md) (GitHub Security Advisories or
jesse@jeswr.org). Note in particular that the `sparq-zk*` and `sparq-mpc` crates are
research scaffolds with **no security guarantee** — see `SECURITY.md` for the caveats.

## Filing issues

Use the [issue templates](./.github/ISSUE_TEMPLATE/) — there's a form for bug reports and
one for feature requests. Security reports are redirected to the private channels above.

## License

By contributing, you agree that your contributions are licensed under the same terms as
the project (see [`LICENSE`](./LICENSE)).
