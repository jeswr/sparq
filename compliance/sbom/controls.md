<!-- [OPUS-4.8] SBOM control table — epic sq-toze / bead sq-toze.12. -->
# SBOM + supply-chain — controls

The full control spine (control → status → evidence → owner, one row per applicable control) lives at
[`controls/sbom.md`](controls/sbom.md) — that is the table the SBOM auditor checks. It is grouped:

- **A. NTIA minimum elements** (the 7 baseline SBOM data fields) — N1..N7.
- **B. CycloneDX completeness** — CDX-1..4.
- **C. VEX** — VEX-1..4.
- **D. Signed / attested SBOM** — SIG-1..3.
- **E. Per-release publication** — PUB-1..4.
- **F. Dependency transparency & gating** — DEP-1..6.
- **G. SBOM-to-artifact integrity** — INT-1..3.

Corroborating commands + the recorded `cargo cyclonedx` probe are in [`evidence.md`](evidence.md);
open gaps + remediation beads are in [`gap-register.md`](gap-register.md).

**Headline:** **31 control rows — 23 implemented & verified, 6 audit-ready, 2 gap.** The 6
**audit-ready** rows (SIG-1/2/3, PUB-1/2, VEX-4) are release-gated and **config-verified, not
operating-verified** — no `v*` tag / Release / attestation exists yet (verified 2026-06-15), so the
attested artifacts have never been produced. The 2 **gap** rows are N1/GS-1 (per-component Supplier
Name slot — partial; the `author` field is present on 144/166 components) and INT-3/GS-2 (reproducible
build). CDX-3/GS-4 (spec version) is now **RESOLVED** ([OPUS-4.8], sq-toze.28 — SBOM emitted as
CycloneDX **1.5**, matching the VEX, with `metadata.lifecycles` populated). Separately, **3 tracked
open gap *items*** (GS-1,2,3) plus the RESOLVED GS-6/sq-toze.30, GS-4/sq-toze.28, and GS-5/sq-toze.29
are recorded in [`gap-register.md`](gap-register.md), each with a severity + a bead — note GS-3
(JS-lockfile SBOM) is a scope item with no control row. GS-5 (VEX↔deny drift-check automation) is now
**RESOLVED** ([OPUS-4.8], sq-toze.29 — the GATING CI job `supply-chain.yml#vex-deny-sync`), so VEX-3 is
fully Implemented & verified with no residual sub-gap. So the gap *items* ≠ the gap *rows*. No control
is overclaimed.
