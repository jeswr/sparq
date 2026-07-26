# Release runbook

How to cut a sparq release. Everything below is **maintainer-triggered**: nothing publishes
until you push a `v*` tag (CI) or run `cargo publish` (crates.io) yourself.

## 0. One-time pre-release steps (before the first 0.1.0 tag)

These are tracked as beads (`bd list -l area:release`); the procedure is documented
here for the runbook:

- **Add a `LICENSE` file** (MIT text) at the repo root. `license = "MIT"` in Cargo.toml
  satisfies crates.io, but the release archives and the Docker/ghcr page should carry the
  actual text; `release.yml` copies `LICENSE` into every archive *if present*.
- See **§0a crate-name availability** below — checked; all crates.io names are clear, the
  npm scope `@jeswr/sparq` is clear, and the PyPI **distribution** name `sparq` is taken, so
  the Python wheel publishes as **`sparq-rdf`** (owner-approved; the import name stays
  `sparq` — done, see the Python wheels section).
- `cargo owner` / crates.io API token configured locally (`cargo login`).

## 0a. Crate-name availability

Availability snapshot as of **2026-06-14** (crates.io API: a 404 / "does not exist" =
available; a 200 = taken). The original snapshot covered 16 publishable crates plus the
top-level `sparq` name; the npm and PyPI surface names are included for completeness. The
publishable set is **17** (sparq-algos was missed in the 2026-06-14 snapshot — see the row
flagged *not yet re-checked* below). Re-run before the first publish — registries change.

For the **PyPI** row, that re-run is one command (`scripts/check-pypi-name.py --check`,
sq-ed5): it reads the distribution name from `crates/sparq-py/pyproject.toml` (`sparq-rdf`)
and queries the PyPI JSON API with the same 404-available / 200-taken convention, exiting 0
(available) / 1 (taken) / 2 (indeterminate). `--expect available|taken` asserts the expected
state for a `&&` chain before `maturin upload`; `--name <n>` checks an explicit name. (Live
mode needs network; the hermetic decision-table + name-reader self-test runs in CI's
`ci-scripts` lane.) crates.io has no separate pre-flight script — `cargo publish` aborts on a
taken name itself.

| Name | Registry | Status |
|---|---|---|
| `sparq` | crates.io | available |
| `sparq-core` | crates.io | available |
| `sparq-engine` | crates.io | available |
| `sparq-cli` | crates.io | available |
| `sparq-server` | crates.io | available |
| `sparq-reason` | crates.io | available |
| `sparq-introspect` | crates.io | available |
| `sparq-hdt` | crates.io | available |
| `sparq-shacl` | crates.io | available |
| `sparq-sim` | crates.io | available |
| `sparq-vectors` | crates.io | available |
| `sparq-geo` | crates.io | available |
| `sparq-serve` | crates.io | available |
| `sparq-text` | crates.io | available |
| `sparq-rsp` | crates.io | available |
| `sparq-solid` | crates.io | available |
| `sparq-nlq` | crates.io | available |
| `sparq-algos` | crates.io | not yet re-checked (added to the publishable set after the 2026-06-14 snapshot — sq-v286.1) |
| `@jeswr/sparq` | npm | available |
| `sparq` | PyPI | **taken** — unrelated `shiventi/sparq` (SJSU degree-planning API client, latest 0.2.6) |
| `sparq-rdf` | PyPI | chosen distribution name (owner-approved; `pip install sparq-rdf`, `import sparq`) |

**Conclusion:** every crates.io name checked in the 2026-06-14 snapshot and the npm scope are
clear to publish as-is; `sparq-algos` (the 17th publishable crate) still needs its one-off
availability check before the first publish (the re-run noted above covers it). The PyPI
**distribution** name `sparq` is taken by a real, unrelated, actively-maintained package, so
the Python wheel ships as **`sparq-rdf`** (owner-approved; `sq-8slf`). This is the only name
that changed — the **import** name stays `sparq` (`import sparq`), and no crates.io / npm
name is affected. Done in `crates/sparq-py/pyproject.toml` (`[project] name = "sparq-rdf"` +
`[tool.maturin] module-name = "sparq"`); see the Python wheels section below.

## 1. Version bump

The version lives in **one place**: `[workspace.package] version` in the root `Cargo.toml`
(every crate inherits it via `version.workspace = true`). Additionally, the internal
`path` dependencies carry an explicit `version = "X.Y.Z"` requirement (the standard
workspace-publish pattern — crates.io strips `path`, so the version is what consumers
resolve). On a bump, update **both**:

1. `[workspace.package] version` in `/Cargo.toml`.
2. The `version = "…"` next to each `sparq-* = { path = … }` dependency in
   `crates/sparq-engine`, `crates/sparq-reason`, `crates/sparq-cli`, `crates/sparq-server`.
3. `cargo check -p sparq-cli -p sparq-server` to refresh `Cargo.lock`; commit it.

(`cargo release` or `release-plz` can automate 1–3 later; not adopted yet.)

## 2. Changelog

Add a `## [X.Y.Z] - YYYY-MM-DD` section to `CHANGELOG.md` (Keep-a-Changelog format:
Added / Changed / Fixed / Removed). Performance claims go in only with a pointer to the
measurement (e.g. `bench/qlever-baselines.md`). Add the compare/tag link at the bottom.

## 3. Tag push → what CI does

```sh
git tag vX.Y.Z
git push jeswr vX.Y.Z
```

Pushing the tag triggers `.github/workflows/release.yml`:

1. **package** — builds `sparq-cli` for every hardware tier (same matrix as `dist.yml`:
   arm64/x64 darwin, x86-64 baseline/v2/v3/v4 + arm64 Linux, x64/arm64 Windows) and packages
   each as `sparq-cli-vX.Y.Z-<tier>.tar.gz` (`.zip` on Windows) containing the binary,
   `README.md`, `LICENSE` (if present), and `CHANGELOG.md`.
2. **release** — generates `SHA256SUMS` over all archives and creates the GitHub Release
   with every archive + the checksum manifest attached (plus auto-generated notes linking
   to `CHANGELOG.md`). Verify with `shasum -a 256 -c SHA256SUMS`.
3. **docker** — builds the `Dockerfile` and pushes
   `ghcr.io/jeswr/sparq-server:{X.Y.Z, X.Y, latest}` using the workflow's own
   `GITHUB_TOKEN` (ghcr needs no extra secrets). **To disable container publishing**,
   delete the `docker` job from `release.yml` (or gate it with `if: false`) — the other
   jobs are independent of it.

Note: `dist.yml` *also* fires on `v*` tags (bare per-tier binaries as workflow artifacts).
`release.yml` deliberately duplicates its matrix rather than `workflow_call`-ing it —
`dist.yml` has no `workflow_call` trigger and is owned by another work-thread; see the
header comment in `release.yml`. Once both are merged, drop the `tags:` trigger from
`dist.yml` (keep `workflow_dispatch`) to stop double-building, and keep the two matrices
in sync until they're unified. (The retired `macos-13` runner label has been replaced with
`macos-15-intel` in both workflows' x64-darwin tier.)

## 4. crates.io publication

<!-- [OPUS-4.8] sq-v286.1: reconciled 16 → 17 publishable crates. sparq-algos has full
crates.io metadata and no `publish = false`, so publish.yml's `crates` job already packages
+ attests it; it is added below as a core-only leaf (depends on sparq-core only — verified
in crates/sparq-algos/Cargo.toml — and nothing in the workspace depends on it). -->
**17 crates publish.** The publish order follows the dependency DAG, leaf-first (a crate's
deps must exist on crates.io before it can be verified):

```text
sparq-core                      # no internal deps — first
  ├── sparq-introspect          # core only                ┐
  ├── sparq-reason              # core only                │ order among these
  ├── sparq-hdt                 # core only                │ seven doesn't matter
  ├── sparq-shacl               # core only                │
  ├── sparq-sim                 # core only                │
  ├── sparq-vectors             # core only                │
  └── sparq-algos               # core only                ┘
sparq-engine                    # core + introspect — after both
  ├── sparq-geo                 # core + engine            ┐
  ├── sparq-serve               # core + engine            │ order among these
  ├── sparq-text                # core + engine            │ six doesn't matter
  ├── sparq-rsp                 # core + engine            │
  ├── sparq-solid               # core + engine + reason   │
  └── sparq-nlq                 # core + engine + introspect ┘
sparq-cli                       # core + engine + reason + hdt
sparq-server                    # core + engine + geo + serve — last
```

Exact commands, from the repo root, on the tagged commit:

```sh
cargo publish -p sparq-core
cargo publish -p sparq-introspect
cargo publish -p sparq-reason
cargo publish -p sparq-hdt
cargo publish -p sparq-shacl
cargo publish -p sparq-sim
cargo publish -p sparq-vectors
cargo publish -p sparq-algos        # [OPUS-4.8] sq-v286.1 — core-only leaf (17th publishable crate)

# CHECKPOINT before sparq-engine: the crates.io package resolves UPSTREAM spargebra 0.4.6,
# not the vendored copy (the [patch]/path override is stripped on publish). Dry-run it
# against upstream first — it must package + compile cleanly:
#   cargo publish --dry-run -p sparq-engine
cargo publish -p sparq-engine

cargo publish -p sparq-geo
cargo publish -p sparq-serve
cargo publish -p sparq-text
cargo publish -p sparq-rsp
cargo publish -p sparq-solid
cargo publish -p sparq-nlq
cargo publish -p sparq-cli
cargo publish -p sparq-server
```

- Modern cargo **waits for index propagation** after each publish, so the commands can be
  run back-to-back; if an older cargo complains a dependency isn't found, wait ~a minute
  and retry.
- Dry-run any crate first with `cargo publish --dry-run -p <crate>`
  (`--dry-run -p sparq-core` was verified for 0.1.0: packages + compiles cleanly).
- **Not published** (`publish = false` in their manifests — `cargo publish` refuses them):
  `sparq-bench`, `sparq-conformance`, `sparq-gpu`, `sparq-parse`, `sparq-py` (ships as
  PyPI wheels later, see below), and `sparq-wasm` (ships via npm).
- Publishing is **permanent** (versions can only be yanked, not deleted/reused).

> [OPUS-4.8] **Registry-publish signing (sq-jgt3 / GX-OSSF-2).** Scorecard's `Signed-Releases`
> is satisfied by the Sigstore SLSA build-provenance over the GitHub-Release archives
> (`release.yml`) — but that does **not** sign the bytes a consumer installs from a *package
> registry*. Honest per-registry status:
>
> - **crates.io — no first-party signing/provenance scheme exists upstream** (no equivalent of
>   npm `--provenance` or PyPI PEP-740 attestations). The tractable equivalent we DO ship is an
>   out-of-band attestation: [`publish.yml`](../.github/workflows/publish.yml)'s `crates` job runs
>   `cargo package` and attests the resulting `.crate` bytes (identical to what `cargo publish`
>   uploads) with `actions/attest-build-provenance`. This puts **no** provenance link on the
>   crates.io page — that needs upstream support — but a consumer who downloads the `.crate` can
>   `gh attestation verify <file> --repo jeswr/sparq`. The crates.io-native sub-gap stays **OPEN**
>   (external — see `compliance/openssf/gap-register.md` GX-OSSF-2 / `compliance/gap-register.md`
>   GX-10). Do **not** describe a crates.io publish as "signed".
> - **npm `@jeswr/sparq` — native Sigstore provenance** via [`publish.yml`](../.github/workflows/publish.yml)'s
>   `npm` job. [OPUS-4.8] sq-v286.11: the job now authenticates with **OIDC trusted publishing**
>   (no `NPM_TOKEN`) — npm exchanges the GitHub Actions OIDC token for a short-lived publish
>   credential and records the Sigstore-signed provenance automatically; consumers verify with
>   `npm audit signatures`. See §8 for the one-time Trusted-Publisher registration.
> - **PyPI `sparq-rdf` — native PEP-740 attestations** via Trusted Publishing (see the
>   "Python wheels" section below + §8) once the maintainer registers the Trusted Publisher.
>
> [OPUS-4.8] sq-v286.11: **crates.io Trusted Publishing** (GA 2025-07, RFC 3691) is now available
> as an *authentication* mechanism (a short-lived OIDC token via `rust-lang/crates-io-auth-action`,
> in place of a long-lived `CARGO_REGISTRY_TOKEN`). It is **not** a provenance/signing scheme — it
> does **not** put a provenance link on the crates.io page — so the "do not describe a crates.io
> publish as signed" caveat above is unchanged. The CI side of the auth flip is pre-wired (§8).

## 5. Docker (manual / local)

CI does this on tag; to build locally:

```sh
docker build -t sparq-server .                      # add --build-arg CARGO_FLAGS="-j 2" to cap parallelism
docker run --rm -p 3030:3030 -v "$PWD/data:/data:ro" sparq-server --format turtle /data/file.ttl
curl 'http://localhost:3030/sparql?query=SELECT%20*%20WHERE%20%7B%3Fs%20%3Fp%20%3Fo%7D%20LIMIT%201'
```

Image: multi-stage (`rust:1.87-slim-bookworm` builder → distroless `cc-debian12:nonroot`
runtime), `--locked` release build, binds `0.0.0.0:3030` (flags appended as CMD args
override the entrypoint defaults — the server's arg parser is last-wins).

## 6. Homebrew

`packaging/homebrew/sparq.rb` is a **formula template**; Homebrew installs from a tap, which
is a separate repo decision. To ship it:

1. Create the tap repo `jeswr/homebrew-sparq` (one-time).
2. After the GitHub Release exists, copy the template to `Formula/sparq.rb` in the tap,
   set `version`, and replace each `REPLACE_WITH_SHA256_<tier>` with the matching line from
   the release's `SHA256SUMS` asset (tiers used: `arm64-darwin`, `x64-darwin`,
   `arm64-linux`, `x64-v2`).
3. `brew install jeswr/sparq/sparq` then `brew test sparq` to verify.

Users get `sparq-cli` plus a `sparq` symlink on PATH.

## 7. Post-release

- Check the release page artifacts + `SHA256SUMS`, `docker run ghcr.io/jeswr/sparq-server:X.Y.Z`,
  and the crates.io pages render the README.
- Bump `[workspace.package] version` to the next `-dev` cycle if desired, and start a new
  `## [Unreleased]` section in `CHANGELOG.md`.

<!-- [OPUS-4.8] release publishing tracked: bead sq-7re (wheels matrix). PyPI name resolved by sq-8slf (was sq-ed5). -->
## Python wheels (PyPI) — release publishing not yet wired (bead `sq-7re`)

`crates/sparq-py` packages the engine as the PyPI distribution **`sparq-rdf`** with the
import name **`sparq`** (`pip install sparq-rdf`, then `import sparq`) — pyo3 + maturin,
`abi3-py39` so one wheel per platform covers CPython ≥ 3.9. CI builds and tests it
informationally on pushes to main (`.github/workflows/python.yml`); release publishing is
**not** wired. To ship wheels with a release later:

1. PyPI **distribution** name — **done** (`sq-8slf`, owner-approved). The bare `sparq`
   distribution name is **taken** by an unrelated package (see §0a), so the
   `[project] name` in `crates/sparq-py/pyproject.toml` — i.e. the **PyPI project /
   distribution** name, the `pip install <dist>` name — is **`sparq-rdf`**. The
   **import/module** name is kept as `sparq` (it comes from the cdylib `[lib] name` + the
   `#[pymodule] fn sparq`); because the distribution and module names differ,
   `[tool.maturin] module-name = "sparq"` pins the import name (otherwise maturin would
   normalise the distribution name to `sparq_rdf`). Net: users `pip install sparq-rdf` but
   still `import sparq`. (Only the PyPI distribution name was contested — not the import
   module, not any crate name.)
2. Add a wheels job to `release.yml` using `PyO3/maturin-action` with a platform
   matrix (manylinux x86_64/aarch64, macOS arm64/x64, Windows x64), each step:
   `command: build`, `args: --manifest-path crates/sparq-py/Cargo.toml --profile
   python-release --out dist` (the `python-release` profile keeps unwinding panics —
   the default `release` profile's `panic = "abort"` would hard-abort the interpreter
   on any Rust panic).
3. Publish with `maturin upload` (or `pypa/gh-action-pypi-publish` + trusted
   publishing) gated on the `v*` tag, mirroring the cargo/crates.io flow above.

> [OPUS-4.8] **Update (sq-toze.37 / GX-10): the PyPI publish lane is now WIRED** in
> [`.github/workflows/publish.yml`](../.github/workflows/publish.yml) — jobs `pypi-build`
> (maturin matrix: manylinux x86_64/aarch64, macOS arm64/x64, Windows x64), `pypi-sdist`,
> and `pypi-publish` (uploads via PyPI **Trusted Publishing** with native **PEP-740
> attestations** — `pypa/gh-action-pypi-publish` `attestations: true` + OIDC `id-token: write`,
> GitHub `environment: pypi`). It fires on `release: published` or `workflow_dispatch` with
> `publish_pypi: true`. **One maintainer prerequisite remains** (it cannot be a repo file): on
> the `sparq-rdf` PyPI project, register a **Trusted Publisher** — owner `jeswr`, repo `sparq`,
> workflow `publish.yml`, environment `pypi` (PyPI → project → *Publishing* → *Add a pending /
> published publisher*). Until that one-time PyPI-account step is done the upload step correctly
> fails to mint an OIDC token (no static API token is stored). Once registered, every published
> release emits native PyPI provenance (the "Provenance" panel on the release-files page;
> consumers verify with `pypi-attestations verify` / `gh attestation verify`).

<!-- [OPUS-4.8] sq-v286.11 (maintainer #758): CI publishing via OIDC trusted publishing. -->
## 8. CI publishing — OIDC trusted publishers (the one-time `needs:user` registry-side steps)

[OPUS-4.8] sq-v286.11 (maintainer #758): "publishing via CI with semantic release, using trusted
OIDC publishing for npm, and whatever the best practices are for the python ecosystem." The repo
holds only the **workflows**; each registry's **trust config is a one-time maintainer action** in
that registry's web UI (it cannot live in a repo file — that is the whole point of OIDC: the
registry, not a stored secret, decides which workflow it trusts). All three legs use the GitHub
Actions OIDC token (`id-token: write`); **no long-lived `NPM_TOKEN` / `CARGO_REGISTRY_TOKEN` is
stored**. Honest bootstrap note: **npm and crates.io require the package/crate to already exist**
(register the trusted publisher *after* one manual bootstrap publish); **PyPI supports a *pending*
publisher** so the very first publish can be trusted-publishing too.

### 8a. npm `@jeswr/sparq` — WIRED (`publish.yml` `npm` job)

The `npm` job authenticates entirely via OIDC trusted publishing (no `NODE_AUTH_TOKEN`). It pins
`npm@^11.5.1` (the trusted-publishing CLI floor — Node 22's bundled npm can be older) and keeps
`--provenance --access public`.

**needs:user (npmjs.com):** `@jeswr/sparq` must already exist on npm, so:
1. ONE bootstrap publish with a short-lived **granular** access token (delete the token after).
2. npmjs.com → `@jeswr/sparq` → **Settings → Trusted Publisher** → *GitHub Actions*:
   - Organization or user: **`jeswr`**
   - Repository: **`sparq`**
   - Workflow filename: **`publish.yml`** (filename only, with extension — **not** a path)
   - Environment: **leave blank** (the `npm` job uses no GitHub Environment)
   - Allowed actions: **`npm publish`**

Every subsequent CI publish is tokenless and provenance-bearing (`npm audit signatures`).

### 8b. PyPI `sparq-rdf` — WIRED (`publish.yml` `pypi-publish` job)

Already implemented to current best practice: `pypa/gh-action-pypi-publish` with `attestations: true`
+ OIDC `id-token: write` + GitHub `environment: pypi`, no API token. Emits native PEP-740 provenance.

**needs:user (pypi.org):** PyPI → (project `sparq-rdf` if it exists, else *Your projects → Publishing*
for a **pending** publisher) → **Add a new publisher** → *GitHub*:
   - PyPI Project Name: **`sparq-rdf`**
   - Owner: **`jeswr`**, Repository: **`sparq`**
   - Workflow name: **`publish.yml`**
   - Environment: **`pypi`** (matches the `pypi-publish` job's `environment:`)

Because PyPI allows a *pending* publisher, no manual bootstrap upload is required.

### 8c. crates.io (17 crates) — CI side PRE-WIRED, flip is one config change (`release-plz.yml` + `release-plz.toml`)

crates.io Trusted Publishing (GA 2025-07, RFC 3691) supplies a short-lived OIDC token via
`rust-lang/crates-io-auth-action` — no `CARGO_REGISTRY_TOKEN`. The CI side is pre-wired as a
commented block on `release-plz.yml`'s `release-plz-release` job; `release-plz.toml` keeps
`publish = false` until the trust config exists (so a `publish=true` with no credential can't break
tag-cutting). This is the "config-flip" the design record (§6 item 4) calls "the point of adoption".

**needs:user (crates.io), per the 17 publishable crates (`docs/release.md` §4 DAG), leaf-first:**
1. ONE bootstrap `cargo publish` per crate (crates.io requires each crate to already exist).
2. For **each** crate: crates.io → crate → **Settings → Trusted Publishing → Add** → *GitHub*:
   - Repository owner: **`jeswr`**, Repository name: **`sparq`**
   - Workflow filename: **`release-plz.yml`**
   - Environment: **leave blank** (the `release-plz-release` job uses no GitHub Environment)

**Then flip (three coordinated edits):**
- `release-plz.yml`: uncomment `id-token: write`, the `rust-lang/crates-io-auth-action` step
  (SHA-pinned `c6f97d4…` # v1.0.5), and the `CARGO_REGISTRY_TOKEN: ${{ steps.cratesio-auth.outputs.token }}` env.
- `release-plz.toml`: set `publish = true`.
- `publish.yml`'s `crates` job then reverts to attest-only over the `.crate` bytes (release-plz
  becomes the publisher; the out-of-band attestation stays as the verifiable-bytes evidence).

> Provenance honesty: crates.io trusted publishing is an **auth** mechanism only — it does **not**
> put a provenance link on the crates.io page (no upstream scheme exists, unlike npm/PyPI). The
> "do not describe a crates.io publish as signed" caveat in §4 is unchanged.

### 8d. Publish-rate protections (issue #1135) — what stops a runaway release

**A crates.io version can never be unpublished.** Three protections stand between an
automated pipeline and the registry. All three are already in place; none of them is what
you flip.

1. **The Release PR can never be armed.** `scripts/release_pr_guard.py` is the single
   predicate every arming/merging path consults — `auto-arm.py`, `rearm-sweeper.py`, the
   `check-pr-arm-base.py` PreToolUse hook (which is where agent-typed `gh pr merge --auto`
   goes), `batch-merge.py`, `pr-backlog.py`. It keys on **head branch, author and title —
   never a label**, because anything holding `pull-requests: write` can add or remove a
   label. Adding `review:pass` to the Release PR does not make it armable. It fails closed:
   an unknown head branch refuses rather than admits. The Release PR is merged by a
   maintainer, by hand, deliberately.
2. **A minimum release interval.** `scripts/release-interval-guard.py --enforce` runs in
   `release-plz.yml`'s `release-plz-release` job **before** the tag/publish step.
   `MIN_RELEASE_INTERVAL` is 24 hours, measured from `max(newest v* tag date, newest
   crates.io publication)`. It refuses on any indeterminacy — shallow checkout, unreadable
   tag list, unparseable date, unreachable crates.io, a future-dated last release. A
   definitive crates.io 404 is the only accepted "never published" answer. There is no
   override flag: publishing inside the window is done by hand, consciously.
3. **Version-group coverage.** The same guard refuses when a crate cargo *would* publish is
   absent from `release-plz.toml`'s `version_group` — release-plz would version it
   independently of the locked workspace version and publish it anyway, so what ships would
   not be what the config describes. While `publish = false` this is a loud warning; it
   becomes a hard refusal on the flip.

See what would be published, without touching anything:

```sh
python3 scripts/release-interval-guard.py --dry-run
```

It prints the publishable crate list, each version, the dependency-first publish order and
the cadence verdict it *would* return. It only ever runs `git`, never `cargo`.

> **Open item blocking the flip.** As measured by `--dry-run` on 2026-07-26, **26** crates
> are cargo-publishable (no `publish = false`) while only **17** are in the `version_group`.
> The nine outside it are `sparq-arrow`, `sparq-fedplan`, `sparq-forms`, `sparq-mcp`,
> `sparq-reason-el`, `sparq-reason-ql`, `sparq-shaclc`, `sparq-substrate`, `sparq-wrapper`.
> Each needs a decision: add a `[[package]]` entry with `version_group = "sparq"` (it ships),
> or set `publish = false` in its `Cargo.toml` (it does not). The "17 crates" figure in §8c
> and §0a describes the version_group, not cargo's publishable set.
