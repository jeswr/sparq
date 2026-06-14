<!-- [OPUS-4.8] Governance: branch-protection doc-of-record (bead sq-41ey). -->
# Branch protection — `main`

This is the **doc-of-record** for the branch-protection ruleset on `main`. The settings
themselves are configured **out-of-repo** by the repository owner under
**Settings → Branches → Branch protection rules** (or a repository ruleset) on GitHub;
they cannot be expressed in a tracked file. This document records what those settings
should be, so the intended protection is reviewable and reproducible if the rule is ever
recreated.

## Protected branch

- **`main`** — the only long-lived branch. All changes land via pull request; direct
  pushes are disallowed (including for administrators — see "Other settings").

## Required status checks

PRs must be **up to date with `main`** before merging, and **all** of the checks below
must pass. These names are the **job names** from the CI workflows (an owner selects them
in the "Require status checks" list); keep this list in sync with
[`.github/workflows/ci.yml`](../.github/workflows/ci.yml) (and the auxiliary workflows) as
jobs are renamed or added.

From the **CI** workflow (`.github/workflows/ci.yml`):

| Required check (job name) | What it gates |
|---|---|
| `build + test (workspace)` | `cargo build --workspace --all-targets` + `cargo test --workspace`. |
| `clippy (gate) + fmt (informational)` | `cargo clippy --workspace --all-targets -- -D warnings` (the clippy gate; fmt is informational until the one-time reformat lands). |
| `W3C SPARQL conformance (ratchet >= 1229 pass+divergence)` | The W3C SPARQL conformance ratchet (never lower). |
| `W3C SHACL conformance (ratchet — core >= 98, sparql >= 5)` | The W3C SHACL core + SHACL-SPARQL ratchets. |
| `Inference conformance (ratchet >= 1967 pass+divergence)` | The RDFS/OWL-RL/N3/entailment + rdf-turtle inference ratchet. |
| `coverage ratchet + test-presence gate (per-crate)` | The per-crate line-coverage floor + the test-presence gate. |
| `wasm build (sparq-wasm)` | The `wasm32-unknown-unknown` build, the wasm-deps guard, and `wasm-pack test --node`. |

From the binding/packaging workflows (require when those surfaces are exercised):

| Required check (job name) | Workflow | What it gates |
|---|---|---|
| `maturin build + pytest` | `.github/workflows/python.yml` | The `sparq` PyPI binding build + pytest parity suite. |
| (js binding job) | `.github/workflows/js.yml` | The `@jeswr/sparq` npm build/tests. |

> Heavy benchmarks (`bench.yml`, `bench-ec2.yml`) and release/dist workflows are **not**
> required status checks — they are not part of the per-PR correctness gate.

### Checks to add once they land (supply-chain + SBOM)

The AGENTS.md post-batch checklist calls for a supply-chain gate when Cargo dependencies
change (`cargo audit` + `cargo deny check` + an SBOM). When those jobs are added to CI,
**add them to the required-checks list above** (and to this table). Until then they are
documented expectations, not enforced gates, and `deny.toml` has a CODEOWNERS entry in
anticipation.

## Required reviews

- **At least 1 approving review**, and **require review from Code Owners**
  (see [`CODEOWNERS`](../CODEOWNERS)). A change to a high-risk path
  (`sparq-zk*`, `sparq-mpc`, `sparq-core`, `sparq-server`, `.github/`, `deny.toml`,
  `SECURITY.md`) therefore needs the listed owner's approval.
- **Dismiss stale approvals** when new commits are pushed.

## History and push rules

- **Require linear history** — merges to `main` must not introduce merge commits
  (use squash or rebase merges). This matches the "gate and merge one branch at a time"
  discipline in `AGENTS.md`.
- **Block force pushes** to `main`.
- **Block branch deletion** for `main`.

## Other settings

- **Do not allow bypassing the above** — apply the rules to administrators too
  (include administrators / no bypass actors), so the gate is uniform.
- **Require conversation resolution before merging** (all PR review threads resolved).

## How this maps to the merge discipline

`AGENTS.md` defines the landing gate as *full-workspace clippy + `cargo test` + the
conformance/perf/coverage ratchets, all green*, with parallel worktrees gated and merged
**one branch at a time**. The required status checks above are the CI enforcement of that
gate; linear history + one CODEOWNERS approval + the up-to-date requirement enforce the
one-at-a-time merge discipline. When a new ratchet or gate is added to CI, update both
the CI workflow and this document, and add the new job to the required-checks list.
