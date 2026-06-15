<!-- [OPUS-4.8] SBOM gap register — epic sq-toze / bead sq-toze.12. Every open gap carries a severity,
     a remediation plan, a target, and the tracking bead. No gap is papered over. -->
# SBOM + supply-chain — gap register

Severity: **P0** blocks a perfect score on a high-value framework; **P1** needed for a perfect score;
**P2** raises maturity / completeness. Each gap is honestly recorded with a remediation plan and a
`bd` bead under epic `sq-toze`. None is silently passed in the control table.

## Open gaps

| ID | Gap | Sev | Control(s) | Remediation plan | Target | Bead |
|---|---|---|---|---|---|---|
| **GS-1** | **Per-component supplier name (NTIA N1) not emitted.** `cargo-cyclonedx` 0.5.9 leaves `components[].supplier`/`.author` empty on all 166 components; the literal NTIA "Supplier Name" element is therefore unpopulated per component. (PURL + crates.io transitively identify the supplier-of-record, and the top-level supplier is in the VEX — so this is a *completeness*, not an *integrity*, gap.) | P2 | N1 | Either bump cargo-cyclonedx and emit CycloneDX 1.5/1.6 with `metadata.supplier` + per-component `publisher` from crates.io metadata (preferred), or post-process `scripts/gen-sbom-vex.sh` to inject the crates.io publisher per PURL. Add a probe assertion that `metadata.supplier` is non-empty. | next SBOM tooling bump | `sq-toze.26` |
| **GS-2** | **No reproducible-build evidence (GX-8).** The SBOM↔binary link is asserted via SLSA provenance (SIG-1) + the embedded cargo-auditable manifest (DEP-5), but an auditor cannot independently rebuild a bit-identical binary from the SBOM. Higher SLSA levels + CRA integrity want this (or an honest "not reproducible because…"). | P2 | INT-3 | Produce reproducible-build evidence (pinned toolchain + `--locked` + deterministic flags) OR a documented honest statement of which inputs (timestamps, `RUSTFLAGS=-Ctarget-cpu`, build-path) prevent reproducibility. Shared with the SLSA + CRA worktrees. | aligned with `slsa` worktree | `sq-toze.9` (GX-8) |
| **GS-3** | **No dedicated npm/JS-lockfile SBOM** for the WASM client (`crates/sparq-wasm`, `js/`). The Cargo tree + npm-ecosystem Dependabot cover the deps, but there is no CycloneDX SBOM for the published npm package surface specifically. | P2 | scope (WASM client) | Add a per-release CycloneDX SBOM for the npm package (e.g. `@cyclonedx/cyclonedx-npm`) attached to the JS release. | with next JS release wiring | `sq-toze.27` |
| **GS-4** | **Generated SBOM is CycloneDX 1.3** (cargo-cyclonedx default) while the VEX is 1.5 — mixed spec versions are valid but 1.5/1.6 carries richer supplier/author slots (relevant to GS-1). | P2 | CDX-3 | Emit CycloneDX 1.5/1.6 from the SBOM generator (couples with GS-1). | with GS-1 | `sq-toze.28` |
| **GS-5** | **VEX ↔ deny.toml sync is enforced by comment + manual inspection, not CI.** A future advisory added to `deny.toml [advisories].ignore` without a matching VEX entry (or vice-versa) would not be caught automatically. (Currently **in sync** — verified this branch.) | P2 | VEX-3 | Add a CI check (in `supply-chain.yml`) that fails if the RUSTSEC id set in `deny.toml [advisories].ignore` ≠ the VEX `vulnerabilities[].id` set. Lands test-first. | next supply-chain.yml edit | `sq-toze.29` |

## Notable NON-gaps (recorded so the auditor does not re-open them)

These were candidate gaps in the original cross-cutting register that are now **closed** — cite as
evidence, do not re-propose:

| Was | Now | Evidence |
|---|---|---|
| **GX-1** — degraded advisories PR-gate (`continue-on-error`) | **Closed** (sq-toze.2). Advisories check is **GATING**; CVSS-4.0 blocker (sq-q8de) resolved. | `supply-chain.yml#audit` "cargo-deny check (advisories) — GATING"; `deny.toml` fail-closed. |
| **GX-2** — no checked-in / per-release SBOM with VEX | **Closed** (sq-toze.3). Per-release SBOM + version-stamped VEX, attested + attached. | `scripts/gen-sbom-vex.sh`; `supply-chain/vex.cdx.json`; `release.yml#sbom`, `#release`. |
| **GX-3** — no `.well-known/security.txt` (RFC 9116) | **Closed** (sq-toze.4). | `.well-known/security.txt` (Contact + Policy + Expires 2027-06-15). |
| **GX-7** — no cargo-auditable / cargo-vet | **Implemented & verified; bead OPEN to close-out.** Both are **wired**: cargo-vet is a GATING CI job, cargo-auditable is used in release + Docker builds. The bead `sq-toze.8` remains open only as an administrative close-out, NOT because the control is missing. | `supply-chain.yml#vet`; `release.yml#package` + `Dockerfile:L70`. **Recommend the close-out is confirmed by the SBOM auditor and `sq-toze.8` closed.** |

> **Honesty note for the auditor:** the SBOM posture is strong — 22 controls implemented & verified.
> The five open gaps are all **P2 completeness/maturity** items (per-component supplier field,
> reproducibility, JS SBOM, spec version, drift automation); **none** is a P0/P1 integrity or
> transparency hole, and **none** is overclaimed in `controls/sbom.md`. The one item that could look
> like an open gap — GX-7 (cargo-auditable/cargo-vet) — is in fact **implemented & verified**; the bead
> is open only as a close-out, which this register flags explicitly rather than silently treating the
> bead's open state as a missing control.
