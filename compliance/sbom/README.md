<!-- [OPUS-4.8] SBOM + supply-chain certification slice — epic sq-toze, bead sq-toze.12 (GX-1/2/7/8). -->
# SBOM + supply-chain transparency — certification slice

> 🤖 SPARQ agent. This is the **SBOM-framework** slice of sparq's certification estate
> (epic `sq-toze`, framework `sbom`). It is paired with an adversarial **SBOM auditor**;
> this PR is **draft** and awaits the auditor's zero-findings sign-off.

## What this framework covers

sparq is a Rust **library + binaries (`sparq-cli`, `sparq-server`) + container** shipped on
crates.io / npm (WASM) / ghcr.io and consumed as a **dependency in high-security settings**. For a
dependency, the dominant risk surface is **supply-chain integrity and transparency**, so the SBOM
framework is the highest-fit control family. This slice enumerates the SBOM-transparency controls and
maps each to concrete, verifiable repo evidence:

- **NTIA minimum elements** (the 7 baseline data fields a SBOM must carry).
- **CycloneDX completeness** — format, dependency-relationship graph, license data.
- **VEX** (Vulnerability Exploitability eXchange) — why a flagged advisory is not exploitable.
- **Signed / attested SBOM** — SLSA build-provenance over the SBOM artifacts.
- **Per-release publication** — the SBOM + VEX attached to every GitHub Release.
- **Dependency transparency & gating** — cargo-deny, cargo-vet, cargo-auditable.

The control spine is `controls/sbom.md`; the corroborating artifact/command evidence is `evidence.md`;
open gaps + remediation beads are `gap-register.md`.

## Scope — what is in / out for a library + server

| In scope (sparq's responsibility) | Out of scope (operator's responsibility) |
|---|---|
| The component inventory of what sparq **builds and ships** (the dependency tree of `sparq-cli` / `sparq-server` / the container image). | The SBOM of the **deployment environment** (base OS the operator runs the container on beyond the distroless base, sidecars, gateways). |
| Per-release publication + signing of sparq's own SBOM + VEX. | Aggregating sparq's SBOM into the operator's **product-level** SBOM / asset inventory. |
| Dependency-policy gating (advisories, bans, sources, licenses, audits) at PR + release time. | The operator's **own** vulnerability-response SLA against the published SBOM/VEX. |
| The VEX exploitability assessment for advisories in sparq's tree. | Continuous re-scanning of a **deployed** image (operator runs Trivy/Grype against ghcr digests — sparq publishes the digests + provenance to make that possible). |

The WASM/JS client surface (`crates/sparq-wasm`, `js/`) is a build artifact of the same workspace;
its npm-published dependency surface is covered by the same Cargo dependency tree plus the npm/JS
Dependabot ecosystem (`.github/dependabot.yml`). A dedicated JS-lockfile SBOM is recorded as a P2 gap
(see `gap-register.md`, GS-3).

## Honesty posture (one-paragraph summary)

The SBOM posture is **strong and mostly implemented & verified**: a per-release CycloneDX SBOM per
binary plus a VEX are generated (`scripts/gen-sbom-vex.sh`), attached to every Release, and
**SLSA build-provenance attested** (`release.yml`); cargo-deny gates advisories/bans/sources/licenses
at PR time; cargo-vet gates per-dependency audit attestations; cargo-auditable embeds the dependency
manifest into the shipped binaries. The honest **gaps** are: (GS-1) per-component **supplier/author**
NTIA fields are not emitted by `cargo-cyclonedx`, weakening two of the seven NTIA elements; (GS-2) no
**reproducible-build** evidence (GX-8, the SBOM-to-binary integrity link is asserted, not
independently reproducible); (GS-3) no separate **npm/JS-lockfile** SBOM for the WASM client. None of
these is papered over — each carries a severity and a tracking bead. No control here is overclaimed.
