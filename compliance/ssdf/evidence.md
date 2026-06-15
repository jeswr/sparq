<!-- [OPUS-4.8] SSDF evidence index (cert framework `ssdf`, epic sq-toze, bead
     sq-toze.13). Re-review when Fable returns. NON-CANONICAL timing. -->

# NIST SSDF (SP 800-218) — evidence index

A by-artifact index of the repo evidence the [`controls.md`](./controls.md) rows cite, so
the auditor can resolve each claim to a concrete, checkable location. Paths are
repo-relative. "Gating" means the artifact is a required CI check folded into
`ci-summary / gate` (the single required status check — `docs/branch-protection.md`).

## 1. Governance & policy artifacts (PO, RV)

| Artifact | Path | SSDF tasks it evidences |
|---|---|---|
| Security policy + disclosure + SLAs + research-scaffold caveats | `SECURITY.md` | PO.1.1, RV.1.1, RV.1.2, RV.1.3, RV.2.1, RV.2.2, PW.9.2 |
| Machine-readable disclosure pointer (RFC 9116) | `.well-known/security.txt` | RV.1.1, RV.1.3 |
| Contributor secure-coding standard | `CONTRIBUTING.md` ("Secure coding" + "The gate") | PO.1.1, PO.2.1, PO.2.2, PW.1.3, PW.5.1, RV.3.3 |
| Code ownership / required-reviewer routing | `CODEOWNERS` | PO.1.2, PS.1.1, PW.2.1, PW.7.1 |
| Branch-protection doc-of-record | `docs/branch-protection.md` | PO.2.1, PS.1.1, PW.7.1 |
| PR template (per-change re-evaluation checklist) | `.github/PULL_REQUEST_TEMPLATE.md` | PO.2.1, PW.8.1 |
| Issue templates redirecting security reports | `.github/ISSUE_TEMPLATE/{security,config}.yml` | RV.1.1, RV.1.3 |
| Threat model (STRIDE, boundaries B1–B5) | `research/threat-model.md` | PW.1.1, PW.9.1 |
| ZK soundness audit (root-cause analysis of record) | `research/zk-soundness-audit.md` | PW.1.1, RV.3.1, PW.2.1 |
| Per-site `unsafe` justification register (B5 boundary) | `compliance/memsafety/unsafe-register.md` | PW.1.2, PW.8.2 |
| Dependency policy | `deny.toml` + `supply-chain/config.toml` | PO.3.1, PW.1.3, PW.4.1, PW.4.4 |

## 2. CI gates (the enforced controls)

| CI job (workflow#job) | What it gates | SSDF tasks |
|---|---|---|
| `ci.yml#clippy` | clippy `-D warnings` (gating) + fmt (non-blocking today) | PO.3.2, PW.5.1, PW.7.2 |
| `ci.yml#test` (sharded) | `cargo test --workspace` + doctests | PW.8.1, PW.8.2 |
| `ci.yml#conformance` (SPARQL/SHACL/inference ratchets) | spec-conformance floors (never lowered) | PO.4.1, PW.8.2 |
| `ci.yml#coverage` | per-crate coverage + test-presence ratchet | PO.4.1 |
| `ci.yml#unsafe-register` | unsafe-count ratchet (`scripts/unsafe-gate.py`, `bench/unsafe-snapshot.json`) | PO.3.2, PO.4.1, RV.3.4 |
| `codeql.yml` | CodeQL `security-and-quality` SAST | PO.3.2, PW.7.1, PW.7.2 |
| `supply-chain.yml#audit` | `cargo deny check bans sources licenses` **and** `advisories` (all gating) | PO.3.2, PW.4.1, PW.4.4, RV.1.4 |
| `supply-chain.yml#vet` | `cargo vet --locked` (per-dependency audit attestations, gating) | PW.4.1 |
| `supply-chain.yml#sbom` | CycloneDX SBOM artifact (`**/*.cdx.json`) | PO.3.3, PS.3.2 |
| `miri.yml#miri` | UB lane over `sparq-core` unsafe surface (nightly) | PW.8.2 |
| `fuzz.yml#fuzz` | coverage-guided fuzz (parsers + mmap loader) | PW.8.2, RV.3.4 |
| `scorecard.yml#analysis` | OpenSSF Scorecard → SARIF + public dashboard | PO.3.3 |
| `dependency-monitoring.yml` | daily advisory watchdog → tracking issue | RV.1.4 |
| `ci-summary.yml#gate` | the single required aggregator (merge gate) | PO.2.1, PO.4.2, PO.3.3 |

## 3. Release & provenance artifacts (PS, PW.6)

| Artifact | Path / where produced | SSDF tasks |
|---|---|---|
| SLSA build-provenance attestation (Sigstore-signed) over archives | `release.yml` `actions/attest-build-provenance` step | PS.2.1, PS.3.2 |
| `SHA256SUMS` over release archives | `release.yml#release` | PS.2.1, PS.3.1 |
| Per-release CycloneDX SBOM **+ VEX** | `release.yml#sbom-vex` (provenance-attested) | PS.3.2, PW.4.4 |
| `cargo auditable` embedded dependency manifest in `sparq-cli` | `release.yml` (`cargo auditable build --release --locked`) | PS.3.2, PW.6.1 |
| Container SLSA provenance + embedded SBOM (`provenance: mode=max`, `sbom: true`) | `release.yml#container`, `Dockerfile` | PS.2.1, PS.3.2, PW.6.1 |
| Committed `Cargo.lock` (exact resolved tree) | `Cargo.lock` | PS.3.1, PW.6.1 |
| Dependency update automation (4 ecosystems) | `.github/dependabot.yml` | PO.1.3, PW.4.1, RV.1.4 |

## 4. How to reproduce the evidence locally

```sh
# Dependency policy (the currently-gating subset + advisories)
cargo deny check bans sources licenses
cargo deny check advisories            # gating again post-GX-1 (CVSS-4.0 parse fixed)
cargo vet --locked                     # per-dependency audit attestations

# SBOM
cargo cyclonedx --all --format json    # CycloneDX SBOMs (**/*.cdx.json)

# Memory-safety / unsafe attestation
python3 scripts/unsafe-gate.py --check # the unsafe-count ratchet
cargo +nightly miri test -p sparq-core # UB lane (needs nightly + miri component)

# Provenance verification of a published release artifact
gh attestation verify <archive> --repo jeswr/sparq

# Read the embedded dependency manifest of a released binary
cargo audit bin <path-to-sparq-cli>
```

## 5. Honest gaps (do not read these as met)

- **PW.6.2 — reproducible builds (GX-8, bead sq-toze.9).** The release is `--locked` +
  provenance-attested, but there is no reproducible-build claim or evidence yet. See
  [`gap-register.md`](./gap-register.md). This is the only *technical* SSDF gap.
- The **audit-ready** rows in `controls.md` (PO.1.x/PO.2.x/PO.5.x, several RV tasks) are
  *documented + automated* but their continuous operation / formal attestation is an
  organizational act; for a single-maintainer volunteer project they are asserted with the
  cited gate/doc, not externally certified. SSDF itself ships **no certificate** — it is an
  implementer-side practice framework — so "audit-ready" here means *evidenced and ready for
  an org to assert*, not "a cert is pending".
- The `sparq-zk*` / `sparq-mpc` estate is **not** counted as a met security control
  anywhere in this mapping; its no-guarantee disclaimer (`SECURITY.md`,
  `research/zk-soundness-audit.md`) is a correctly-disclosed limitation. Any future control
  claim that contradicts the "v1 verifier is NOT sound" verdict is out of bounds.
