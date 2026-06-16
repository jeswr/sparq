<!-- [OPUS-4.8] SLSA framework intro (epic sq-toze / bead sq-toze.14, branch cert-slsa). -->
# SLSA build provenance — sparq certification slice

**Framework:** [SLSA](https://slsa.dev) (Supply-chain Levels for Software Artifacts) v1.0 —
Build track + Source/Provenance/Verification dimensions.
**Bead:** `sq-toze.14` (epic `sq-toze`). **Branch:** `cert-slsa`.

## Honest headline

> **sparq's official release archives (`sparq-cli` tar.gz/zip) and the ghcr.io `sparq-server`
> container image reach SLSA Build Level 2.** They carry signed, hosted-platform-generated
> provenance (`actions/attest-build-provenance` / buildkit `provenance: mode=max`). sparq does
> **not** claim Build L3 (provenance is generated in-band with the build, not by an isolated
> trusted builder), and one tag-time build path (`dist.yml`) plus the crates.io/npm/PyPI
> published packages are currently **unattested** — see `gap-register.md`.

This is a bounded, evidence-backed claim. We never publish "SLSA L3" or "all artifacts attested"
because the provenance does not back it.

## Why SLSA is a high-fit framework for sparq

sparq is consumed as a **dependency** (crates.io / npm-WASM / PyPI / ghcr). For a dependency the
dominant supply-chain question is *"was this artifact built from the source it claims, by a build
I can verify?"* — which is exactly what SLSA's provenance answers. sparq already emits signed
provenance on release; this slice declares the **honest level**, maps each control to concrete
repo evidence, and records the gaps to a higher/complete posture.

## Scope — what's in and out for a library/server

**In scope (sparq's build pipeline):** the `release.yml` archive build + the `release.yml#docker`
container build (where provenance lives); the source-track integrity controls SLSA's provenance
binds to (version control, two-person review, pinned+locked deps, cargo-vet/deny, least-privilege
tokens); consumer-side verification documentation.

**Out of scope / operator-owned:**
- **Deploy-time admission enforcement** — requiring `gh attestation verify` / a cosign policy
  before running an artifact is the *operator's* control. sparq ships verifiable provenance + the
  documented verify command (`controls.md` SL-V-b).
- **The SLSA-level *certificate*** — an accredited-assessor attestation is an external-body
  activity. This slice makes the controls + evidence **audit-ready**; the certificate is external.
- **crates.io published-package provenance** — no upstream mechanism exists yet (folded into
  GX-10 as an external sub-gap).

## Files

| File | Contents |
|---|---|
| `controls.md` | The control spine: SLSA Build L1→L3 + Source/Verification controls, each → status (IV/AR/GAP) → evidence (file/CI-job) → owner. The honest per-artifact level table. |
| `evidence.md` | The verifiable evidence pack — exact workflow snippets + the `gh attestation verify` / `cargo audit bin` / `cargo vet` commands an auditor runs. |
| `gap-register.md` | Open gaps (GX-8/9/10/11), severity, remediation, target, the `bd` bead each tracks. |

## Status summary (for `compliance/README.md`)

| Posture | Detail |
|---|---|
| **Implemented & verified** | Build L2 for release archives + container **+ the `dist.yml` tiered binaries** (signed provenance, hosted runner, cargo-auditable, attested SBOM/VEX; GX-9 closed via sq-toze.23); source-track integrity (pinned+locked deps, cargo-vet + cargo-deny GATING, least-privilege tokens, security.txt). |
| **Audit-ready** | Two-person review + protected-branch ruleset (configured out-of-repo, recorded in `docs/branch-protection.md`); consumer verification policy (operator-enforced); the SLSA-level certificate (external assessor). |
| **Gap** | no published-package provenance — crates.io/npm/PyPI (GX-10/sq-toze.24); no reproducible-build evidence (GX-8/sq-toze.9); Build L3 not met — in-band provenance (GX-11/sq-toze.25). *(GX-9 dist.yml binaries — CLOSED, now SLSA Build L2 / sq-toze.23.)* |

## Do-not-re-propose (already in the posture — cite, don't re-add)

`actions/attest-build-provenance` + buildkit `provenance: mode=max` (release.yml), cargo-auditable
(release.yml), cargo-vet GATING (supply-chain.yml), cargo-deny sources/advisories GATING, SHA-pinned
actions + base images, Cargo.lock + `--locked`, ci-summary required gate, CODEOWNERS +
branch-protection doc, `.well-known/security.txt`, per-release SBOM+VEX. These are evidence, not
new work.
