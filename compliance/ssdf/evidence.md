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
| `ci.yml#clippy` | clippy `-D warnings` (gating) + fmt (non-blocking today). **A lint, not a SAST** — it performs no taint or crypto-misuse analysis, so it is *partial* evidence for PW.7.2, never a substitute for the disabled CodeQL lane (GX-14). | PO.3.2, PW.5.1, PW.7.2 (partial) |
| `ci.yml#test` (sharded) | `cargo test --workspace` + doctests | PW.8.1, PW.8.2 |
| `ci.yml#conformance` (SPARQL/SHACL/inference ratchets) | spec-conformance floors (never lowered) | PO.4.1, PW.8.2 |
| `ci.yml#coverage` | per-crate coverage + test-presence ratchet | PO.4.1 |
| `ci.yml#unsafe-register` | unsafe-count ratchet (`scripts/unsafe-gate.py`, `bench/unsafe-snapshot.json`) | PO.3.2, PO.4.1, RV.3.4 |
| ~~`codeql.yml`~~ **DISABLED — NOT valid evidence** | ~~CodeQL `security-and-quality` SAST~~ — the workflow is retained on `main` with its triggers intact, but it has been **disabled at the Actions level (`disabled_manually`) since 2026-07-18** by separate maintainer direction (merge latency). GitHub schedules **no** run on **any** event (push, pull_request, merge_group, schedule): no `CodeQL analysis (rust)` check-run, no SARIF upload to code scanning, nothing fed to `ci-summary`, nothing gated. **It gates nothing and must not be cited as a met control.** No compensating SAST exists. See **GX-14** in [`../gap-register.md`](../gap-register.md), `ASSURANCE.md` §11, posture decision **#4620**. | ~~PO.3.2, PW.7.1, PW.7.2~~ — struck from PO.3.2 (which stands on its other four gating tools); PW.7.1 + PW.7.2 downgraded to **Partial** |
| `supply-chain.yml#audit` | `cargo deny check bans sources licenses` **and** `advisories` (all gating) | PO.3.2, PW.4.1, PW.4.4, RV.1.3 (sparq-local `RV.1.4`) |
| `supply-chain.yml#vet` | `cargo vet --locked` (per-dependency audit attestations, gating) | PW.4.1 |
| `supply-chain.yml#sbom` | CycloneDX SBOM artifact (`**/*.cdx.json`) | PO.3.3, PS.3.2 |
| `miri.yml#miri` | UB lane over `sparq-core` unsafe surface (nightly) | PW.8.2 |
| `fuzz.yml#fuzz` | coverage-guided fuzz (parsers + mmap loader) | PW.8.2, RV.3.4 |
| `scorecard.yml#analysis` | OpenSSF Scorecard → SARIF + public dashboard | PO.3.3 |
| `dependency-monitoring.yml` | daily advisory watchdog → tracking issue | RV.1.3 (sparq-local `RV.1.4`) |
| `ci-summary.yml#gate` | the single required aggregator (merge gate) | PO.2.1, PO.4.2, PO.3.3 |

> **`RV.1.4` is a sparq-local sub-task, not a standard SP 800-218 v1.1 task id.** RV.1 defines
> only RV.1.1 / RV.1.2 / RV.1.3; the continuous-monitoring obligation lives in RV.1.3's text.
> The watchdog/advisories-gate/Dependabot evidence above is mapped to standard **RV.1.3** and
> carries the `RV.1.4` label only for local traceability — see the footnote in `controls.md`.

## 3. Release & provenance artifacts (PS, PW.6)

| Artifact | Path / where produced | SSDF tasks |
|---|---|---|
| SLSA build-provenance attestation (Sigstore-signed) over archives | `release.yml` `actions/attest-build-provenance` step | PS.2.1, PS.3.2 |
| `SHA256SUMS` over release archives | `release.yml#release` | PS.2.1, PS.3.1 |
| Per-release CycloneDX SBOM **+ VEX** | `release.yml#sbom-vex` (provenance-attested) | PS.3.2, PW.4.4 |
| `cargo auditable` embedded dependency manifest in `sparq-cli` | `release.yml` (`cargo auditable build --release --locked`) | PS.3.2, PW.6.1 |
| Container SLSA provenance + embedded SBOM (`provenance: mode=max`, `sbom: true`) | `release.yml#container`, `Dockerfile` | PS.2.1, PS.3.2, PW.6.1 |
| Committed `Cargo.lock` (exact resolved tree) | `Cargo.lock` | PS.3.1, PW.6.1 |
| Dependency update automation (4 ecosystems) | `.github/dependabot.yml` | PO.1.3, PW.4.1, RV.1.3 (sparq-local `RV.1.4`) |

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

- **PW.7.1 / PW.7.2 — static analysis is NOT being performed (GX-14, P1; issue #4620).**
  [OPUS-5] `.github/workflows/codeql.yml` has been **disabled at the Actions level
  (`disabled_manually`) since 2026-07-18**, by separate maintainer direction (merge latency).
  The file and its triggers are retained on `main`, but GitHub schedules **no** run on **any**
  event (push, pull_request, merge_group, schedule) — so there is no `CodeQL analysis (rust)`
  check-run, no SARIF upload to code scanning, nothing contributed to `ci-summary`, and nothing
  gated. Two earlier claims in `controls.md` PW.7.2 were false as written and have been
  corrected: CodeQL does **not** "run on push/PR/merge_group + schedule", and code-scanning
  alerts are **not** "kept at zero" — **35 open `critical`
  `rust/hard-coded-cryptographic-value` alerts** remain. Those 35 were **triaged** under issue
  #4615 and found to be false positives of one query-model defect; **triaged is not covered**,
  and it says nothing about what an enabled scanner would find now. **There is no compensating
  SAST control.** clippy `-D warnings`, the unsafe-count ratchet, cargo-deny/cargo-vet, fuzz and
  Miri are all live and genuine — but **none** performs taint or crypto-misuse analysis, so this
  residual is real. PW.7.1 and PW.7.2 are consequently **Partial**, keeping only their
  code-review evidence. The durable posture (re-enable advisory-only / on a schedule / adopt
  another SAST / accept and document no SAST) is an **open maintainer decision (#4620)**. See
  **GX-14** in [`../gap-register.md`](../gap-register.md), `ASSURANCE.md` §11, and
  [`gap-register.md`](./gap-register.md) SSDF-G2.
- **PW.6.2 — reproducible builds (GX-8, bead sq-toze.9).** The release is `--locked` +
  provenance-attested, and the honest PW.6.2 reproducibility statement is now **documented**
  ([`../slsa/reproducible-build.md`](../slsa/reproducible-build.md)): a measured double-build of
  `sparq-cli` is **identical size + byte-identical apart from 22 bytes**, all from **one**
  non-determinism source (the `mimalloc` build-time `__DATE__`/`__TIME__` `.rodata` banner + the
  build-id it perturbs). The only *technical* SSDF gap that remains open is the **enforcement**
  step — `SOURCE_DATE_EPOCH`/feature-drop + a CI rebuild-and-diff ratchet for a byte-for-byte
  claim. See [`gap-register.md`](./gap-register.md).
- The **audit-ready** rows in `controls.md` (PO.1.x/PO.2.x/PO.5.x, several RV tasks) are
  *documented + automated* but their continuous operation / formal attestation is an
  organizational act; for a single-maintainer volunteer project they are asserted with the
  cited gate/doc, not externally certified. SSDF itself ships **no certificate** — it is an
  implementer-side practice framework — so "audit-ready" here means *evidenced and ready for
  an org to assert*, not "a cert is pending".
- The `sparq-zk*` / `sparq-mpc` estate is **not** counted as a met security control
  anywhere in this mapping; its no-production-guarantee status is a correctly-disclosed
  limitation. [OPUS-4.8] The v1 ZK verifier was originally found unsound
  (`research/zk-soundness-audit.md`, kept on record), then `sq-1s2` landed the binding layer and
  an internal re-audit (`research/zk-verifier-reaudit.md`, `sq-gbp4`) found the prior findings
  closed → "sound as landed for the assumed threat model" — but that is **internal/single-model
  self-review only, with external accredited-cryptographer sign-off still PENDING (`sq-qhy4`) and
  NO production guarantee** (`SECURITY.md`). Any future control claim that asserts an
  externally-validated or production ZK soundness/privacy guarantee is out of bounds until the
  external sign-off lands.
<!-- [OPUS-4.8] reconciled with post-remediation re-audit (sq-gbp4); see ZK-verdict cross-ref sweep -->
