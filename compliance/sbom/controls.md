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

**Headline:** 22 controls **implemented & verified**; 5 honest **gaps** (per-component supplier name,
reproducible build, SBOM spec version, VEX drift-check automation, JS-lockfile SBOM) — each recorded
with severity + a bead. No control is overclaimed.
