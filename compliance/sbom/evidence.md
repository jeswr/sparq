<!-- [OPUS-4.8] SBOM evidence pack — epic sq-toze / bead sq-toze.12. Reproducible commands + recorded
     probe of a real `cargo cyclonedx` run on branch cert-sbom. NON-CANONICAL timing. -->
# SBOM + supply-chain — evidence pack

This pack gives the SBOM auditor reproducible commands and the **recorded output** of a real SBOM
generation on this branch, so every "implemented & verified" row in `controls/sbom.md` is checkable
without re-deriving it. All paths are repo-relative; all CI job names match
`.github/workflows/*.yml` `jobs.<id>.name`.

## 1. The artifacts (checked-in source-of-truth)

| Artifact | Path | What it is |
|---|---|---|
| VEX (source of truth) | `supply-chain/vex.cdx.json` | CycloneDX 1.5 VEX; 2 vulnerabilities (`RUSTSEC-2024-0436`, `RUSTSEC-2025-0134`), each `not_affected`. |
| SBOM+VEX generator | `scripts/gen-sbom-vex.sh` | Produces per-binary SBOM + version-stamped VEX into `./sbom/` for the release. |
| Dependency policy | `deny.toml` | cargo-deny advisories/bans/sources/licenses; `[advisories].ignore` = the 2 RUSTSEC IDs the VEX mirrors. |
| cargo-vet config | `supply-chain/config.toml`, `supply-chain/audits.toml`, `supply-chain/imports.lock` | Trusted import sets + exemptions; gates new unaudited deps. |

## 2. The CI / release wiring (cite these job + step names)

| Control | Workflow#job | Step (verbatim) |
|---|---|---|
| Advisories/bans/sources/licenses gating | `supply-chain.yml#audit` ("cargo-deny (advisories + bans + sources + licenses)") | "cargo-deny check (bans + sources + licenses) — GATING" + "cargo-deny check (advisories) — GATING" |
| Per-dependency audit attestations | `supply-chain.yml#vet` ("cargo-vet … — GATING") | "cargo-vet check — GATING" (`cargo vet --locked`) |
| CI SBOM artifact (every push/PR) | `supply-chain.yml#sbom` ("generate CycloneDX SBOM") | "Generate SBOM …" + "Upload SBOM artifact" (`sbom-cyclonedx`) |
| Per-release SBOM + VEX | `release.yml#sbom` ("CycloneDX SBOM + VEX") | "Generate per-release SBOM + VEX" (`scripts/gen-sbom-vex.sh`) |
| SBOM + VEX SLSA attestation | `release.yml#sbom` | "Attest build provenance (SBOM + VEX)" (`actions/attest-build-provenance`, SHA-pinned `a2bbfa2…`) |
| cargo-auditable embedded manifest | `release.yml#package` + `Dockerfile:L70` | "Build …" `cargo auditable build --release --locked` |
| Release asset attach + checksums | `release.yml#release` ("create GitHub Release") | "Generate SHA256SUMS" + "Create release" (`softprops/action-gh-release`, SHA-pinned) |
| Image SBOM + provenance | `release.yml#docker` ("build + push container") | "Build and push" `provenance: mode=max` + `sbom: true` |
| Daily advisory watchdog (defence-in-depth) | `dependency-monitoring.yml#audit` ("cargo-deny advisories -> tracking issue") | cron `13 5 * * *`; opens/updates `security:dependency-vuln` issue |

## 3. Recorded probe — NTIA elements on a real SBOM (branch cert-sbom)

Reproduce:

```sh
cargo cyclonedx --all --format json
# then inspect crates/sparq-server/sparq-server.cdx.json
```

Recorded result for `crates/sparq-server/sparq-server.cdx.json` (cargo-cyclonedx 0.5.9, default
features, this branch — re-run **2026-06-15** to verify the `author`/`supplier` counts below):

| Field | Observed value | NTIA / CDX control |
|---|---|---|
| `bomFormat` / `specVersion` | `CycloneDX` / `1.3` | CDX-1 (valid); CDX-3 (spec version — see GS-4) |
| `serialNumber` | present | CDX-1 |
| `metadata.timestamp` | `2026-06-15T23:49Z` (varies per run) | N7 ✅ |
| `metadata.tools` | `{vendor: CycloneDX, name: cargo-cyclonedx, version: 0.5.9}` | N6 ✅ (SBOM-author = tool) |
| `metadata.authors` / `metadata.supplier` | **absent** | contributes to GS-1 |
| components count | **166** | — |
| `components[].name` | present 166/166 | N2 ✅ |
| `components[].version` | present 166/166 (e.g. `sparq-core@0.1.0`) | N3 ✅ |
| `components[].purl` | present 166/166 (`pkg:cargo/<name>@<ver>`) | N4 ✅ |
| `components[].licenses` | present 166/166 | CDX-2 ✅ |
| `components[].externalReferences` | present | CDX-4 ✅ |
| `components[].author` | **present 144/166** (e.g. `spargebra → "Tpt <thomas@pellissier-tanon.fr>"`, `aho-corasick → "Andrew Gallant <jamslam@gmail.com>"`); the 22 without it are the 3 first-party workspace crates `sparq-core`/`sparq-engine`/`sparq-serve` + 19 deps (`axum`, `axum-core`, `crossbeam-*`, `rayon`, `rayon-core`, `hashbrown`, `futures-*`, `either`, `equivalent`, `typenum`, `quick-xml`, `pin-project-lite`, `find-msvc-tools`) | N6/N1 — component-author identity present on the majority |
| `components[].supplier` / `.publisher` | **absent on all 166 (0/166)** | **GS-1** (N1 per-component *Supplier Name* slot empty) |
| `dependencies[]` graph | present, 167 nodes (incl. root) | N5 ✅ |

**Honest reading (corrected per the 2026-06-15 re-run):** cargo-cyclonedx 0.5.9 emits 6/7 NTIA
elements (N2,N3,N4,N5,N6,N7). The CycloneDX `author` field — the 1.3-era carrier of
component-originator identity and the closest field to NTIA *Supplier Name* — is populated on
**144/166** components from each crate's `authors` metadata; only the dedicated `supplier`/`publisher`
slot is empty (**0/166**). So **one** NTIA element — *Supplier Name* (`supplier`/`publisher`) — is
left **partial** (author-granularity present, supplier-granularity empty); this is the one genuine
NTIA-completeness gap (**N1/GS-1**). The generated SBOM is `specVersion 1.3` while the VEX is `1.5`.

> **Root/component refs are normalized — abs-path leak RESOLVED (F-6/GS-6, bead sq-toze.30, [OPUS-4.8]):**
> cargo-cyclonedx 0.5.9 *raw* output stamps the absolute build dir into every workspace/path-dependency
> `bom-ref` (`path+file:///<abs-path>/crates/sparq-server#0.1.0`) and `purl`
> (`pkg:cargo/sparq-server@0.1.0?download_url=file://.`), so an unprocessed published SBOM would carry
> the **CI runner's absolute path**. The **published** SBOMs are now post-processed by
> `scripts/sbom-normalize.jq` (invoked from `scripts/gen-sbom-vex.sh` for the two released-binary SBOMs,
> and from `supply-chain.yml#sbom` over every CI-uploaded `*.cdx.json`), which rewrites each such ref to
> the canonical, host-independent `pkg:cargo/<name>@<version>` form and strips the `download_url=file://…`
> purl query, while rewriting the dependency graph (root `bom-ref`, nested build-target sub-components,
> and every `dependencies[].ref` / `dependsOn[]` edge) in lock-step so all internal references still
> resolve. The transform is deterministic (pure function of the input — no time/host/RNG) and idempotent
> (a second pass is byte-identical). Both `gen-sbom-vex.sh` and `supply-chain.yml#sbom` additionally
> fail loudly if any `path+file://`, `download_url=file://`, or `/home/` survives. Verified locally
> against freshly-generated `sparq-cli`/`sparq-server` SBOMs: 0 host-path leaks, valid CycloneDX shell,
> all dependency refs resolve (167/175 respectively), idempotent. *Residual:* the abs-path leak is fixed
> only in the **post-processed publication path**; raw `cargo cyclonedx` still emits path refs (upstream
> behaviour, not changed here).

> The probe files (`**/*.cdx.json`) are **not committed** — `scripts/gen-sbom-vex.sh#L47` and the
> probe both delete them to keep the worktree clean. They are regenerable with the command above.

## 4. VEX ↔ deny.toml sync verification

```sh
# both lists must be exactly {RUSTSEC-2024-0436, RUSTSEC-2025-0134}
grep RUSTSEC deny.toml
python3 -c "import json;print([v['id'] for v in json.load(open('supply-chain/vex.cdx.json'))['vulnerabilities']])"
```

Recorded this branch: `deny.toml` `[advisories].ignore` = `{RUSTSEC-2024-0436, RUSTSEC-2025-0134}`;
VEX `vulnerabilities[].id` = `{RUSTSEC-2024-0436, RUSTSEC-2025-0134}`. **In sync.** (Automating this
drift-check is GS-5.)

## 5. Provenance verification (for a consumer)

```sh
gh attestation verify sparq-cli-<ver>.sbom.cdx.json --repo jeswr/sparq   # SLSA provenance on the SBOM
shasum -a 256 -c SHA256SUMS                                              # release-asset integrity
cargo audit bin sparq-server                                            # read the embedded manifest (cargo-auditable)
cosign verify-attestation ghcr.io/jeswr/sparq@<digest> ...              # image SBOM + provenance
```

These are the consumer-facing verification steps a downstream high-security integrator runs; they
correspond to SIG-1/SIG-2, DEP-5, and SIG-3 respectively.

> **Operating-effectiveness status (release-gated controls):** all four commands above are
> **unrunnable today** — `release.yml` triggers only on `push: tags: v*`, and the repo has **no
> `v*` tags, no GitHub Releases, and no Sigstore attestations yet** (verified `git tag -l 'v*'` → 0,
> `gh release list` → empty, `gh api repos/jeswr/sparq/attestations/...` → 404, 2026-06-15). The
> SIG-*/PUB-*/VEX-4/DEP-5 controls are therefore **verified at the configuration level (workflow
> wiring reviewed and correct), not at the operating level** — no attested SBOM/VEX artifact has ever
> been produced. An external auditor must re-verify these by cutting (or observing) the first `v*`
> release and re-running the commands above against its assets. The control rows in
> `controls/sbom.md` are labelled **"Audit-ready (config-verified; operating-verification pending
> first release)"** accordingly.
