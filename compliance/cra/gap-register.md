<!-- [OPUS-4.8] EU CRA gap register. Bead sq-toze.18 (epic sq-toze). -->
# EU CRA — gap register

Open gaps only. **Severity** = impact on a clean CRA conformity story (P0 blocks an essential
requirement; P1 a process/info requirement; P2 quality/assurance raise; P3 nice-to-have).
Every gap carries the `bd` bead (epic `sq-toze`) that tracks the fix. Gaps already closed by
other certification work are listed at the bottom as **resolved** so the auditor sees the full
picture and we do not re-propose them.

## CRA-specific open gaps

| ID | Gap | CRA req | Sev | Remediation | Bead |
|---|---|---|---|---|---|
| GX-CRA-1 | **No concrete support / EOL period statement.** Annex II A.6 expects the user to be told the period security support is provided. `SECURITY.md` describes support informally ("next release", "no LTS") with no concrete support-period/EOL date. | Annex II A.6; Annex I Part II.8 | P1 | Add a CRA-style support-period statement to `SECURITY.md` (or a `SUPPORT.md`): the supported window, minimum support period, EOL policy. **Audit-ready** — needs the maintainer's organizational decision on the actual period. | `sq-f8tv` |
| GX-CRA-3 | **No single named "cybersecurity policy" document.** CRA Art. 24 (open-source steward) requires a documented cybersecurity policy; Art. 13 (manufacturer) the documented risk assessment + processes. The substance is scattered (`SECURITY.md`, `CONTRIBUTING.md`, `research/threat-model.md`, `deny.toml`, the CI gates) but not consolidated into one adoptable artifact. | Art. 13 / Art. 24; controls.md CRA-CA.1/CRA-CA.6 | P2 | Author a consolidating policy template under `compliance/policies/` (CVD + secure-SDLC + dependency + release-signing). **Audit-ready** — template needs org sign-off. Cross-references the SSDF/privacy policy-template set. | `sq-d43g` |

## Cross-cutting gaps that bear on CRA (owned by other framework worktrees — cite, don't duplicate-fix)

| ID | Gap | CRA req | Sev | Owner worktree | Bead |
|---|---|---|---|---|---|
| GX-9 | **`dist.yml` release binaries lack a SLSA provenance attestation** (only `release.yml` archives/SBOM/image are attested). A binary distributed via the `dist` lane is not provenance-covered. | Annex I Part II.7 (secure distribution) | P2 | slsa / sbom | `sq-toze.23` |
| GX-10 | **No published-package provenance for crates.io / npm / PyPI.** The release archives + ghcr image are attested, but the registry-published packages (the form most consumers actually pull) carry no provenance. | Annex I Part II.7 | P2 | slsa / supply-chain | `sq-toze.24` |
| GX-12 | **No container-image vulnerability scan (Trivy/Grype) + Dockerfile-lint (Dockle/Hadolint) lane in CI.** The CIS-Docker posture (distroless/non-root/pinned) is strong but unscanned — a vulnerable base layer could ship undetected, weakening the I.2 "no known exploitable vulnerabilities" claim for the *container* artifact. | Annex I Part I (2); Part II.1/.3 | P2 | cis | `sq-toze.31` |

> **Honesty note on the formal layer.** The conformity-assessment / EU-declaration-of-conformity
> / CE-marking obligations (controls.md CRA-CA.2/CRA-CA.3) are **not** recorded as fixable gaps
> here: they are organizational/legal acts reserved to the manufacturer/steward and **cannot be
> self-certified by this project or an agent**. They are *audit-ready* (the technical evidence to
> support them exists) and intentionally left to the deploying/commercialising party. Recording
> them as "gaps with a bead" would imply sparq can close them in-tree, which it cannot.

## Resolved (closed by other certification work — do not re-propose)

| Former gap | Resolution | Evidence |
|---|---|---|
| GX-CRA-2 — no Article 14 ENISA/CSIRT reporting runbook | **Addressed** (sq-iy3p): an **adoptable operator runbook** operationalising the Article 14 early-warning (24h) / notification (72h) / final-report (14-day vuln / 1-month incident) timeline, ENISA single-reporting-platform + CSIRT routing, report-content checklist, and coordination with the `SECURITY.md` CVD flow + SBOM/VEX. Org-specific details are `<FILL-IN>` placeholders; the *act* of reporting stays an organisational/legal duty (not a cert claim). | [`incident-reporting-runbook.md`](./incident-reporting-runbook.md); controls.md CRA-CA.5 |
| GX-1 — advisories PR-gate degraded | **Resolved** (sq-toze.2): `cargo deny check advisories` is GATING again on PR (CVSS-4.0 blocker sq-q8de fixed). The "no known exploitable vulnerability" claim now rests on a real PR-time gate. | `.github/workflows/supply-chain.yml#audit` |
| GX-2 — no per-release SBOM + VEX | **Resolved** (sq-toze.3): per-release CycloneDX SBOM per binary + checked-in VEX, attached to the Release + SLSA-attested. | `scripts/gen-sbom-vex.sh`, `release.yml#sbom`, `supply-chain/vex.cdx.json` |
| GX-3 — no `.well-known/security.txt` | **Resolved** (sq-toze.4): RFC 9116 `security.txt` with Contact/Policy/Canonical/Expires. | `.well-known/security.txt` |
| GX-6 — no CONTRIBUTING secure-coding section | **Resolved** (sq-toze.7): secure-coding standard (unsafe policy, input validation, supply-chain). | `CONTRIBUTING.md` |
| GX-7 — no cargo-auditable / cargo-vet | **Resolved** (sq-toze.8): `cargo auditable build` on releases + container; `cargo vet --locked` GATING. | `release.yml#package`, `Dockerfile`, `supply-chain.yml#vet` |
