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

## 0b. v0.1.0 ships ahead of the external ZK review (issue #2552)

**Decision (maintainer, issue #2552, 2026-07-26): v0.1.0 goes out without waiting for the
external accredited-cryptographer review of the ZK estate (bead `sq-qhy4`), carrying
experimental warnings instead.** Recorded here because it is a release-scope decision and
nothing in CI encodes it: there is no audit gate in `release.yml`, `release-plz.yml`,
`release-plz.toml` or either release guard, and there never was — the review is a P0 task,
not a release blocker.

What the release ships instead of the review, and what must stay true of every future
release while `sq-qhy4` is open:

- The GitHub Release body carries an **Experimental** paragraph naming `sparq-zk` and
  `sparq-mpc` as research scaffolds, stating that no external accredited cryptographer has
  reviewed them and that the release makes no soundness/security/privacy claim for either,
  and linking `SECURITY.md`. It is pinned by
  `scripts/tests/test_release_publish_guard.py::TestReleaseCarriesTheExperimentalZkCaveat`
  — the release notes are the one surface a downloader who never opens the repo still
  reads, so it is a test, not a convention.
- `SECURITY.md` (§ *Scope and a critical caveat*), `README.md`, `crates/sparq-zk/README.md`,
  `crates/sparq-mpc/README.md`, `skills/zk-query-proofs/SKILL.md` and `skills/mpc/SKILL.md`
  already carry the matching "research scaffold / not externally audited / semi-honest
  only" language, and `scripts/check-privacy-claims.sh` gates against any unqualified
  soundness or privacy claim creeping back in.

The one thing this decision does **not** license: presenting a ZK "verified" result or an
MPC run as a production-grade guarantee anywhere. That stays false until `sq-qhy4` closes.

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

0. **publish-cadence guard** — the `setup` job runs
   `scripts/release-interval-guard.py --enforce --released-tag vX.Y.Z` before anything is
   built. If the previous release was less than 24h ago the job fails and **every** job
   below it is blocked (they all depend on `setup`). See §8d — this is the tag-push half
   of the at-most-one-release-per-day policy. The tag stays pushed; delete it
   (`git push --delete <remote> vX.Y.Z`) and re-push once the window has passed.
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

Note: [GPT-5] `dist.yml` is manual-dispatch only; it does not run on `v*` tags. Both
`dist.yml` and `release.yml` call the reusable `build-matrix.yml`, which is the single
source of truth for the hardware-tier matrix and build steps. `dist.yml` selects
`mode: binary` for bare per-tier workflow artifacts, while `release.yml` selects
`mode: archive` for the versioned release archives described above.

## 3a. The LWS / Solid server binary — container only, no `dist`/`release` archive

<!-- [OPUS-5] sq-gg0qq.11 (issue #2741): the bead asks for a decision, recorded here so the
runbook is the single place a releaser looks. Related: research/lws-3-crate-split.md §3
(blast radius) and its §7 Q2 (the published image name is still a maintainer question). -->

**Decision: the Solid/LDP server binary is NOT added to `dist.yml` or to the `release.yml`
archives. It ships only as a container image, and it is labelled EXPERIMENTAL.**

`crates/sparq-lws-core` builds an implicit binary from `src/main.rs` (there is no `[[bin]]`
stanza and the crate is `publish = false`). A **second** workflow fires on a `v*` tag alongside
`release.yml` (`dist.yml` is dispatch-only, per the note above) —
`.github/workflows/lws-container.yml` — which smoke-tests
(`crates/sparq-lws-core/tests/container-smoke.sh`) and Trivy-scans a native amd64 image, then
pushes a multi-arch `linux/amd64,linux/arm64` index to
`ghcr.io/sparq-org/sparq-lws-core:{X.Y.Z, X.Y, latest}` with SBOM and max-mode build
provenance. It is also `workflow_dispatch`-able, which tags only `:latest` — that is the form
the demo and the `deploy/` templates reference. Nothing else about the LWS binary is published.

Why not a `dist`/`release` archive:

- **The crate's own description says `EXPERIMENTAL`**, and its whole distribution surface is a
  long-lived server process configured entirely through `SOLID_SERVER_*` / `PSS_*` environment
  variables. A bare `solid-server` archive next to `sparq-cli-vX.Y.Z-<tier>.tar.gz` reads as a
  supported product; a container carries the runtime contract with it and is the honest shape.
- **The crate boundary is still moving.** `sparq-lws` and `sparq-solid-server` do not exist yet
  and the partition that would create them is an open maintainer decision
  (research/lws-3-crate-split.md §5/§7); the container/deploy cutover is deliberately sequenced
  last in that record's §6 phase 4, because it is the only phase that can break a running demo.
  Adding hardware-tiered release archives now would pin a binary NAME (`sparq-lws-core` today,
  `solid-server` after the cutover) that the split is expected to change, and
  `.github/workflows/deploy-lint.yml` already asserts the current image string literally.
- **Cost.** `build-matrix.yml` builds ten hardware tiers. Doubling it for a binary whose
  supported deployment is a container buys nothing a consumer has asked for.

**Revisit when** (any one is enough): the `sq-gg0qq` split lands and the bin has a stable
crate + name; or a consumer needs a non-container deployment. The change is then small — add a
package/bin selector to `build-matrix.yml` — and it should ship as a clearly EXPERIMENTAL-
labelled OPT-IN artifact, not silently alongside the `sparq-cli` archives.

## 4. crates.io publication

<!-- [OPUS-4.8] sq-v286.1: reconciled 16 → 17 publishable crates. sparq-algos has full
crates.io metadata and no `publish = false`, so publish.yml's `crates` job already packages
+ attests it; it is added below as a core-only leaf (depends on sparq-core only — verified
in crates/sparq-algos/Cargo.toml — and nothing in the workspace depends on it). -->
**26 crates publish.** The publish order follows the dependency DAG, leaf-first (a crate's
deps must exist on crates.io before it can be verified):

```text
sparq-core, sparq-fedplan       # no published internal deps — first
sparq-algos, sparq-hdt, sparq-introspect, sparq-sim,
sparq-substrate, sparq-wrapper  # core-only leaves
sparq-engine                    # core + introspect
sparq-reason                    # core + substrate
sparq-reason-el                 # core + optional substrate
sparq-arrow, sparq-geo, sparq-nlq, sparq-rsp,
sparq-serve, sparq-shacl,
sparq-solid, sparq-text         # engine/reason dependants
sparq-forms                     # core + shacl
sparq-server                    # core + engine + geo + serve
sparq-vectors                   # core + optional engine
sparq-cli                       # core + engine + reason + hdt + server
sparq-mcp                       # broad optional integrations — last

sparq-reason-ql and sparq-shaclc have no published runtime dependencies and may be
published in any earlier group; they appear after their related capability crates below.
```

Exact commands, from the repo root, on the tagged commit:

```sh
cargo publish -p sparq-core
cargo publish -p sparq-fedplan
cargo publish -p sparq-introspect
cargo publish -p sparq-hdt
cargo publish -p sparq-sim
cargo publish -p sparq-algos        # [OPUS-4.8] sq-v286.1 — core-only leaf (17th publishable crate)
cargo publish -p sparq-substrate
cargo publish -p sparq-wrapper

# CHECKPOINT before sparq-engine: the crates.io package resolves UPSTREAM spargebra 0.4.6,
# not the vendored copy (the [patch]/path override is stripped on publish). Dry-run it
# against upstream first — it must package + compile cleanly:
#   cargo publish --dry-run -p sparq-engine
cargo publish -p sparq-engine

cargo publish -p sparq-reason
cargo publish -p sparq-reason-el
cargo publish -p sparq-arrow
cargo publish -p sparq-geo
cargo publish -p sparq-serve
cargo publish -p sparq-text
cargo publish -p sparq-rsp
cargo publish -p sparq-solid
cargo publish -p sparq-nlq
cargo publish -p sparq-shacl
cargo publish -p sparq-forms
cargo publish -p sparq-reason-ql
cargo publish -p sparq-shaclc
cargo publish -p sparq-server
cargo publish -p sparq-vectors
cargo publish -p sparq-cli
cargo publish -p sparq-mcp
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
- **Confirm the isolated-builder provenance verified** (issue #4571 / GX-11 / SL-B3-b). After the
  Release is cut, `release.yml`'s `verify-provenance` job calls
  `.github/workflows/release-verify.yml`, which re-downloads every published
  asset and checks, fail-closed, that: both `.intoto.jsonl` bundles are attached and listed in
  `SHA256SUMS`; `SHA256SUMS` matches the published bytes; and `slsa-verifier verify-artifact`
  accepts **every** asset against one of the two bundles (an asset covered by neither reds the
  run). It is part of the release run — look for the `verify published provenance` job on the same
  workflow run that cut the tag; its uploaded `provenance-verification.log` is the evidence record.

  > It is driven from `release.yml` deliberately, and `release-verify.yml`'s `release: published`
  > trigger is only an out-of-band net for a Release published **by hand**. GitHub does not start
  > a workflow run from an event generated by a workflow's own `GITHUB_TOKEN`, and that is the
  > token `release.yml` creates the Release with — so a normal `v*` release emits no
  > run-starting `release` event at all. `scripts/tests/test_verify_release_provenance.sh` pins
  > the caller structurally so this cannot silently regress to trigger-only.

  To run the identical checks by hand — e.g. to re-verify an older tag, or from a consumer's
  machine:

  ```bash
  go install github.com/slsa-framework/slsa-verifier/v2/cli/slsa-verifier@v2.7.1
  scripts/verify-release-provenance.sh --tag vX.Y.Z          # `gh release download` + verify
  scripts/verify-release-provenance.sh --tag vX.Y.Z --dir ./assets   # already-downloaded assets
  ```

  A **red** run means a *published* release does not carry the provenance the compliance estate
  claims for it: treat it as a supply-chain incident and fix forward — never relax the checks and
  never drop `release`'s `needs: [provenance, provenance-artifacts]`, which is what stops a
  Release being cut when a trusted builder fails in the first place.
- **Then, and only then, update the compliance estate.** A green verification run is the evidence
  `compliance/slsa/controls.md` SL-B3-b needs to move from **AR** to **IV** *for the artifacts that
  verified*. Record the run URL in `compliance/slsa/evidence.md` and narrow GX-11 in both gap
  registers. Keep the wording bounded: the verified property is *unforgeable provenance*, not a
  hardened build (the generic generator signs digests our build jobs reported), and the ghcr.io
  container image is still in-band **L2** — so GX-11 narrows, it does not close.
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

### 8c. crates.io (26 crates) — CI side PRE-WIRED, flip is one config change (`release-plz.yml` + `release-plz.toml`)

crates.io Trusted Publishing (GA 2025-07, RFC 3691) supplies a short-lived OIDC token via
`rust-lang/crates-io-auth-action` — no `CARGO_REGISTRY_TOKEN`. The CI side is pre-wired as a
commented block on `release-plz.yml`'s `release-plz-release` job; `release-plz.toml` keeps
`publish = false` until the trust config exists (so a `publish=true` with no credential can't break
tag-cutting). This is the "config-flip" the design record (§6 item 4) calls "the point of adoption".

**needs:user (crates.io), per the 26 publishable crates (`docs/release.md` §4 DAG), leaf-first:**
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

### 8d. Publish-rate protections (issues #1135, #2552) — what stops a runaway release

**The policy: at most one release per day.** `MIN_RELEASE_INTERVAL` in
`scripts/release-interval-guard.py` is the single constant that states it (24h), and it is
enforced at **both** points a release can start — the Release-PR path (`release-plz.yml`)
and the `v*` tag push (`release.yml`). There is no override flag: releasing inside the
window is done by hand, deliberately.

**A crates.io version can never be unpublished.** Four protections stand between an
automated pipeline and the registry. All four are already in place; none of them is what
you flip.

1. **The Release PR can never be armed.** `scripts/release_pr_guard.py` is the single
   predicate every arming/merging path consults — `auto-arm.py`, `rearm-sweeper.py`, the
   `check-pr-arm-base.py` PreToolUse hook (which is where agent-typed `gh pr merge --auto`
   goes), `batch-merge.py`, `pr-backlog.py`. It keys on **head branch, author and title —
   never a label**, because anything holding `pull-requests: write` can add or remove a
   label. Adding `review:pass` to the Release PR does not make it armable. It fails closed:
   an unknown head branch refuses rather than admits. The Release PR is merged by a
   maintainer, by hand, deliberately.

   **Read "armed" literally.** The PreToolUse hook recognises `gh pr merge` **with**
   `--auto`. A direct `gh pr merge <n> --squash` (no `--auto`), `--admin`, a
   `gh api graphql … enablePullRequestAutoMerge` mutation, a REST
   `PUT …/pulls/<n>/merge`, a backslash line-continuation, or shell-variable indirection
   all reach `gh` unblocked — verified by executing the hook against a fake `gh`. That is a
   deliberate scope (the hook governs *arming*), not an oversight, and it is why
   protection 2 exists: the interval guard runs inside `release-plz.yml` itself and does
   not care how the merge happened. If the guard script itself cannot run,
   `.claude/settings.json`'s wrapper **denies** any `gh pr merge` rather than allowing it.
2. **A minimum release interval.** `scripts/release-interval-guard.py --enforce` runs in
   `release-plz.yml`'s `release-plz-release` job **before** the tag/publish step.
   `MIN_RELEASE_INTERVAL` is 24 hours, measured from `max(newest v* tag date, newest
   crates.io publication)`. It refuses on any indeterminacy — shallow checkout, unreadable
   tag list, unparseable date, unreachable crates.io, a future-dated last release. A
   definitive crates.io 404 is the only accepted "never published" answer. There is no
   override flag: publishing inside the window is done by hand, consciously.
3. **The same interval, on the tag-push path** (issue #2552). Protection 2 only covers
   releases that go through the Release PR. §3's canonical instruction — push a `vX.Y.Z`
   tag — fires `release.yml` **directly**, which was previously uncadenced: a hand-pushed
   tag (or a script pushing one) could cut a release minutes after the last. The guard now
   also runs in `release.yml`'s `setup` job, the job every other job there depends on, so a
   refusal stops the archives, the SBOM/VEX, the GitHub Release and the ghcr image.

   It runs there with `--released-tag vX.Y.Z`, and that flag is load-bearing: on a tag push
   `v<workspace version>` is already in the tag list, so without it the guard takes its
   "already tagged, nothing to release" branch and allows unconditionally. `--released-tag`
   excludes the tag being cut and suppresses that branch, so the interval is measured
   against the *previous* release. It refuses if handed anything that is not a `vX.Y.Z`
   tag.

   It is **unconditional** — `workflow_dispatch` builds are guarded too. They look like a
   developer/test path (pre-release, `dev-`-prefixed image tags), but the Release they
   create fires `release: published`, and `publish.yml`'s `npm` job runs on *any* `release`
   event, so a dispatch reaches a registry as well. Two consequences: `inputs.tag` on a
   dispatch **must** be a `vX.Y.Z` tag (a `-dev` suffix is fine), and the unbounded way to
   exercise the pipeline is a local build, not a dispatch.
   `scripts/tests/test_release_publish_guard.py` pins the step's `run:`, that it carries
   no `if:` and no `continue-on-error`, the `fetch-depth: 0` checkout it depends on, and
   that no job in `release.yml` escapes `setup`.
4. **Version-group coverage.** The same guard refuses when a crate cargo *would* publish is
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

### 8e. release-plz forge token — one `needs:user` secret unblocks the Release-PR (issue #3273)

`release-plz.yml` cannot open the Release-PR with the workflow's own `GITHUB_TOKEN`: the
repo/org setting **Settings → Actions → General → "Allow GitHub Actions to create and approve
pull requests"** is disabled, so every push to `main` ends in
`Failed to open PR … 403 Forbidden … /pulls`. The job is advisory and the known 403 is
contained loudly (signature + live probe), so nothing goes red — but no Release-PR is opened,
which is what the version/changelog automation and the GUI-download release depend on.

Both jobs now pick their forge token in this order, so **no workflow edit is needed** — only
the secret:

```text
App token (ORCHESTRATOR_APP_ID + ORCHESTRATOR_APP_PRIVATE_KEY)  ← preferred
  || RELEASE_PLZ_TOKEN     (fine-grained PAT: contents:write + pull_requests:write)
  || GITHUB_TOKEN          (fallback; cannot open PRs while the setting is disabled)
```

**needs:user — do exactly one:**
1. Provision `ORCHESTRATOR_APP_ID` + `ORCHESTRATOR_APP_PRIVATE_KEY` (the App used by
   `batch-merge.yml`; install it on this repo with contents + pull-requests write), **or**
2. add a `RELEASE_PLZ_TOKEN` repo secret (fine-grained PAT, same two permissions), **or**
3. enable the repo setting above.

Options 1–2 also fix a second, independent problem: GitHub suppresses workflow triggers for
events created with `GITHUB_TOKEN`, so a `v<version>` tag pushed by the `release-plz / tag` job
would **not** fire `release.yml` (`push: tags: ["v*"]`) — the tag→`release.yml` handoff §3
describes. A minted App token / PAT is a normal actor and does fire it. Option 3 alone does not.

Once a privileged token is configured the containment step no longer tolerates a 403 at all: a
provisioned token that still 403s is a real failure (App not installed, PAT expired or
under-scoped) and fails the job. Promotion: after a main run is observed opening/updating the
Release-PR, drop the `(advisory)` token from the job name, delete the containment step and
remove the entry from `.github/advisory-registry.json`.

**You do not have to watch for that moment.** The `release-plz-pr` job self-reports: on every
successful run it re-runs the same side-effect-free PR-creation probe (`POST /pulls` with
`head == base`, which can never create a PR), and the moment that probe returns **422** instead
of **403** — authorization passed, only the body was rejected, i.e. PR-creation is unblocked —
the run emits a `release-plz Release-PR promotion unblocked (sq-lonae)` warning naming the three
edits above. The probe runs even on success because `release-plz release-pr` also exits 0 when
there is simply nothing to release, which says nothing about whether PR-creation is allowed.
A probe that returns anything else (or does not complete) is reported as inconclusive and never
claims readiness, and the step can never fail the release path.
