<!-- [OPUS-4.8] SBOM-publication policy TEMPLATE — epic sq-toze / bead sq-toze.12.
     TEMPLATE: needs org sign-off before adoption. Scoped to the SBOM framework. -->
# SBOM publication & maintenance policy (TEMPLATE)

> **Status: TEMPLATE — requires org sign-off before adoption.** This is the SBOM-framework-scoped
> policy. The broader vulnerability-management / coordinated-disclosure policy is owned by the `cra`
> and `privacy` worktrees under `compliance/policies/`; this template cross-references, not forks,
> that work. SECURITY.md is the live human-readable disclosure policy.

## 1. Purpose & scope

This policy governs how sparq produces, signs, and publishes a Software Bill of Materials (SBOM) and
an accompanying VEX for each released artifact (`sparq-cli`, `sparq-server`, the ghcr.io container).
It implements the SBOM-transparency obligations consumers in high-security settings rely on, and feeds
the NIST SSDF (PS.3), EU CRA (Annex I), and OpenSSF certification evidence.

## 2. Policy statements

1. **Per-release SBOM.** Every tagged release (`v*`) SHALL ship a CycloneDX SBOM per released binary,
   generated from the **default feature set** against the committed `Cargo.lock`
   (`scripts/gen-sbom-vex.sh`, `release.yml#sbom`).
2. **NTIA minimum elements.** Each SBOM SHALL carry the NTIA minimum elements (supplier, component
   name, version, unique identifier/PURL, dependency relationship, SBOM author, timestamp).
   Per-component supplier name (N1) is derived honestly by `scripts/sbom-normalize.jq` and asserted on
   every component by the GATING job `supply-chain.yml#sbom-supplier` (GS-1 RESOLVED, sq-toze.26).
3. **VEX.** Every advisory the dependency policy (`deny.toml [advisories].ignore`) chooses to tolerate
   SHALL have a 1:1 VEX entry stating its exploitability (`supply-chain/vex.cdx.json`). The VEX and the
   ignore list MUST NOT diverge.
4. **Signing / attestation.** Every published SBOM + VEX SHALL be SLSA build-provenance attested
   (`actions/attest-build-provenance`) and covered by the release `SHA256SUMS`.
5. **Container image.** The container image SHALL carry an embedded SBOM + max-mode SLSA provenance
   (`provenance: mode=max`, `sbom: true`).
6. **Dependency gating.** No release SHALL proceed past a failing cargo-deny (advisories, bans,
   sources, licenses) or cargo-vet gate; cargo-auditable SHALL embed the dependency manifest in
   shipped binaries.
7. **Maintenance.** When an ignored advisory gains a safe upgrade, the `deny.toml` ignore AND the
   matching VEX entry SHALL be removed together. The `.well-known/security.txt` `Expires` field SHALL
   be refreshed annually.

## 3. Roles

| Role | Responsibility |
|---|---|
| Maintainer / release owner | Runs the release pipeline; confirms SBOM + VEX attached + attested. |
| Security contact (SECURITY.md) | Maintains the VEX exploitability assessments; triages advisories. |
| Consumer (operator) | Verifies provenance (`gh attestation verify`), aggregates sparq's SBOM into their product SBOM, runs their own re-scan SLA against the published digests. |

## 4. Verification

The policy is automatically enforced by the CI/release wiring cited in `evidence.md §2`. A reviewer
can spot-check any release with the consumer-facing commands in `evidence.md §5`.

## 5. Open items (tracked)

GS-2/GX-8 (reproducible build) — the one open item. RESOLVED: GS-1 (per-component supplier, sq-toze.26),
GS-3 (JS SBOM), GS-4 (spec version), GS-5 (VEX drift-check CI), GS-6/GS-7 (purl canonicality) — see
`gap-register.md`.
