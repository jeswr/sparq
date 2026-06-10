# Release runbook

How to cut a sparq release. Everything below is **maintainer-triggered**: nothing publishes
until you push a `v*` tag (CI) or run `cargo publish` (crates.io) yourself.

## 0. One-time pre-release checklist (before the first 0.1.0 tag)

- [ ] **Add a `LICENSE` file** (MIT text) at the repo root. `license = "MIT"` in Cargo.toml
      satisfies crates.io, but the release archives and the Docker/ghcr page should carry the
      actual text; `release.yml` copies `LICENSE` into every archive *if present*.
- [ ] **Check crate-name availability** on crates.io for `sparq-core`, `sparq-engine`,
      `sparq-reason`, `sparq-cli`, `sparq-server` (https://crates.io/crates/<name>). Squatted
      names mean renaming before anything else.
- [ ] `cargo owner` / crates.io API token configured locally (`cargo login`).

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
in sync until they're unified. One known dist.yml fix to fold in: its `macos-13` runner
label is retired (GitHub removed it); `release.yml` already uses `macos-15-intel` for the
x64-darwin tier.

## 4. crates.io publication

**Publish order** follows the dependency DAG (a crate's deps must exist on crates.io before
it can be verified):

```
sparq-core                      # no internal deps — first
  ├── sparq-engine              # depends on core         ┐ order between these
  └── sparq-reason              # depends on core         ┘ two doesn't matter
        ├── sparq-cli           # depends on core + engine + reason — after all three
        └── sparq-server        # depends on core + engine — after engine
```

Exact commands, from the repo root, on the tagged commit:

```sh
cargo publish -p sparq-core
cargo publish -p sparq-engine
cargo publish -p sparq-reason
cargo publish -p sparq-cli
cargo publish -p sparq-server
```

- Modern cargo **waits for index propagation** after each publish, so the commands can be
  run back-to-back; if an older cargo complains a dependency isn't found, wait ~a minute
  and retry.
- Dry-run any crate first with `cargo publish --dry-run -p <crate>`
  (`--dry-run -p sparq-core` was verified for 0.1.0: packages + compiles cleanly).
- **Not published**: `sparq-bench` (internal benchmark harness) and `sparq-wasm` (ships via
  npm later) are `publish = false` in their manifests — `cargo publish` refuses them.
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
