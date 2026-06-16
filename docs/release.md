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
available; a 200 = taken). All 16 publishable crates plus the top-level `sparq` name were
checked; the npm and PyPI surface names are included for completeness. Re-run before the
first publish — registries change.

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
| `@jeswr/sparq` | npm | available |
| `sparq` | PyPI | **taken** — unrelated `shiventi/sparq` (SJSU degree-planning API client, latest 0.2.6) |
| `sparq-rdf` | PyPI | chosen distribution name (owner-approved; `pip install sparq-rdf`, `import sparq`) |

**Conclusion:** all crates.io names and the npm scope are clear to publish as-is. The PyPI
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

**16 crates publish.** The publish order follows the dependency DAG, leaf-first (a crate's
deps must exist on crates.io before it can be verified):

```text
sparq-core                      # no internal deps — first
  ├── sparq-introspect          # core only                ┐
  ├── sparq-reason              # core only                │ order among these
  ├── sparq-hdt                 # core only                │ six doesn't matter
  ├── sparq-shacl               # core only                │
  ├── sparq-sim                 # core only                │
  └── sparq-vectors             # core only                ┘
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
