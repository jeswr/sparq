<!-- [OPUS-4.8] SLSA framework AUDIT findings (epic sq-toze, bead sq-toze.14, branch cert-slsa, PR #226). -->
# SLSA certification — auditor findings

> 🤖 SPARQ agent — independent compliance-auditor pass over the `cert-slsa` engineer evidence
> (`compliance/slsa/{README,controls,evidence,gap-register}.md`, PR #226, epic `sq-toze`).
> Every claim was re-verified against the **actual repo on `origin/main`** (cert-slsa is doc-only
> over the infra — it modifies *no* workflow / config file, so the cited controls must already exist
> on main, and they do). NON-CANONICAL timing.

**Verdict:** `FINDINGS: 3` (all **low** — evidence-precision corrections; **no control overclaim**,
**no blocking gap**). The substantive SLSA claim is sound and the gap register is complete and honest.
The certificate itself (accredited-assessor SLSA-level attestation) remains an external-body activity,
as the slice already labels.

---

## Headline assessment (the questions the brief asked)

- **Is the L2-not-L3 call correct? — YES, and it is well-reasoned.**
  - **L2 is genuinely met** for the `release.yml` archives and the ghcr.io container:
    - `release.yml#package` runs a scripted matrix build on **GitHub-hosted runners only**
      (`ubuntu-latest`/`macos-14`/`macos-15-intel`/`ubuntu-24.04-arm`/`windows-latest` — verified, no
      self-hosted runner anywhere in the release path) and emits signed in-toto provenance via
      `actions/attest-build-provenance@a2bbfa25…` (v4.1.0, Sigstore Fulcio + Rekor, OIDC-bound). ✓
      hosted platform + authenticated provenance = L2.
    - `release.yml#sbom` attests the per-release SBOM + VEX; `release.yml#docker` attaches buildkit
      `provenance: mode=max` + `sbom: true` on push. ✓
  - **L3 is correctly NOT claimed.** `attest-build-provenance` runs **as a step in the same job** as
    `cargo auditable build` (`release.yml#package`), so the OIDC/signing context is reachable by the
    user-controlled build steps. This is exactly the documented reason `attest-build-provenance` alone
    yields L2 and L3 needs an isolated trusted builder (`slsa-github-generator`). The engineer's
    `SL-B3-b`/`GX-11` reasoning matches the SLSA v1.0 build-track definition. **Not over- or
    under-claimed.**

- **Does `attest-build-provenance` run for BOTH archives AND container? — YES.**
  - Archives: `release.yml#package` *Attest build provenance (archives)* over `pkg/*.tar.gz` +
    `pkg/*.zip`. Container: `release.yml#docker` build-push with `provenance: mode=max` + `sbom: true`
    (with `id-token: write` + `attestations: write` on the job). SBOM/VEX additionally attested in
    `release.yml#sbom`. All three confirmed in the live workflow on `main`.

- **Is cargo-auditable really embedded? — YES (build-time control is wired correctly).**
  - `release.yml#package` installs `cargo-auditable` and builds with `cargo auditable build --release
    --locked -p sparq-cli`. The embed-time control is genuinely in place. (The *runtime* artifact —
    `cargo audit bin <file>` printing the manifest — could not be executed in this audit env, as it
    requires a tagged release build; the workflow wiring is verified, the produced-binary check is the
    consumer's `gh attestation verify` / `cargo audit bin` step the evidence documents.)

- **Is SBOM/VEX attested? — YES, and the VEX is genuinely 1:1 with policy.**
  - `release.yml#sbom` attests `*.sbom.cdx.json` + `*.vex.cdx.json`. `scripts/gen-sbom-vex.sh` is real
    and substantive (per-binary CycloneDX + version-stamped VEX). Cross-checked: the VEX
    (`supply-chain/vex.cdx.json`) lists exactly `RUSTSEC-2024-0436` + `RUSTSEC-2025-0134`, and
    `deny.toml [advisories].ignore` lists exactly those two — the claimed 1:1 sync holds.

- **Is the gap register complete + honest? — YES.**
  - GX-9 (dist.yml unattested), GX-10 (no published-package provenance), GX-8 (no reproducible-build
    evidence), GX-11 (L3 / in-band provenance) are all present, correctly severitied, and each maps to
    a **real, open** remediation bead: `sq-toze.23` (GX-9), `sq-toze.24` (GX-10), `sq-toze.9` (GX-8),
    `sq-toze.25` (GX-11) — all confirmed to exist in `.beads/issues.jsonl` with matching titles and
    `status: open`. The two "done" supply-chain inputs the L2 claim leans on (`sq-toze.3` GX-2 SBOM/VEX,
    `sq-toze.8` GX-7 cargo-auditable/vet) are both genuinely `status: closed`.
  - GX-9 is verified honest: `dist.yml` fires on `push: tags: [v*]` + `workflow_dispatch`, has
    top-level `permissions: contents: read` only (no `id-token`/`attestations`), no attest step, no
    cargo-auditable, and builds with bare `cargo build` (not even `--locked`). The "unattested binary in
    the release path" gap is accurate.

- **Honesty tripwire (ZK/MPC soundness): NOT triggered.** The SLSA slice stays in its lane (build
  provenance) and makes no cryptographic-soundness claim. It references `sparq-zk*`/`sparq-mpc` only as
  CODEOWNERS high-risk paths, which is accurate.

- **Source-track / least-privilege posture (the inputs L2 provenance binds to): all verified.**
  cargo-deny advisories GATING + cargo-vet GATING (no `continue-on-error` in `supply-chain.yml`; both
  job names avoid the `\b(advisory|informational)\b` exclusion so `ci-summary` requires them);
  deny.toml `[sources]` `unknown-registry/unknown-git = "deny"`; `Cargo.lock` tracked + `--locked`;
  every workflow carries top-level `permissions: contents: read`; `scorecard.yml` has
  `persist-credentials: false` + `publish_results: true`; **every** `uses:` across all 18 workflows is
  a 40-hex SHA pin; `Dockerfile` base images are SHA-pinned (`rust:1.88-slim-bookworm@sha256:38bc5a86…`,
  `gcr.io/distroless/cc-debian12:nonroot@sha256:b0ae8e98…`); `.well-known/security.txt` (RFC 9116)
  present. The `AR` labels for two-person review / protected-branch (`SL-S-b`/`SL-S-c`) are correctly
  *not* claimed as `IV` (out-of-repo GitHub ruleset; single-maintainer "different person" limitation
  recorded honestly).

---

## Findings

### F-1 (low) — GX-10 evidence mischaracterizes `js.yml` / `python.yml` as "publish paths"

- **Control:** `SL-B2-e` / GX-10 (published-package provenance), `controls.md` + `gap-register.md`.
- **What I checked:** `git show origin/main:.github/workflows/js.yml` and `python.yml`; grepped **all
  18** workflows for `cargo publish | npm publish | twine | pypi | maturin .* publish`.
- **Why it (mildly) fails:** the evidence says "npm (`js.yml`) and PyPI (`python.yml`) publish paths do
  not emit `attest-build-provenance`." But `js.yml`/`python.yml` contain **no publish step at all**
  (js.yml only runs the node test suite + checks `npm pack` stays publishable; python.yml only builds +
  tests), and there is **no publish workflow anywhere in the repo** (crates.io/npm/PyPI are evidently
  published by a manual/out-of-CI `cargo publish` etc.). The *gap is real and correctly recorded as a
  gap* — published packages carry no provenance — but the cited evidence names the wrong artifact: it
  describes these as "publish paths that don't attest" when they are test-only workflows.
- **Severity rationale:** low — the control is honestly a **gap**, not an overclaim; only the evidence
  *pointer* is inaccurate. An auditor following the citation finds no publish step where one is implied.
- **Remediation:** in `controls.md` SL-B2-e and `gap-register.md` GX-10, restate as: "There is **no
  publish workflow** for crates.io/npm/PyPI in `.github/workflows/` — these crates/bindings are
  published manually/out-of-CI, so the published artifact carries no provenance. npm `--provenance` and
  PyPI PEP-740 trusted-publishing would require *first adding* a CI publish workflow; crates.io has no
  upstream mechanism." (i.e. the fix is "add a provenance-emitting publish workflow," not "add
  attestation to an existing one.")

### F-2 (low) — stale "still OPEN in bd" note for `sq-toze.8` (GX-7) contradicts actual bead status

- **Control:** `gap-register.md` §"Dependency note (resolved inputs…)".
- **What I checked:** `.beads/issues.jsonl` for `sq-toze.8` → `title: [cert][gap GX-7] cargo-auditable
  + cargo-vet`, **`status: closed`**.
- **Why it fails:** the note states "**sq-toze.8 … the bead is still OPEN in `bd` and should be closed
  when this lands on `main`**." The bead is already **closed**. The narrative is stale.
- **Severity rationale:** low — the control evidence (cargo-auditable in `release.yml#package`,
  cargo-vet GATING in `supply-chain.yml#vet`) is correct; only the bead-status annotation is wrong, and
  it errs *conservatively* (claims more open work than exists).
- **Remediation:** update the note to "sq-toze.8 (GX-7) — **DONE/closed**; cargo-auditable +
  cargo-vet GATING landed on `main`."

### F-3 (low) — `evidence.md §4` implies `gh attestation verify` validates the buildkit `mode=max` image attestation

- **Control:** `SL-V-c` / container provenance (`evidence.md` §4, "Verify" line).
- **What I checked:** `release.yml#docker` uses `docker/build-push-action … provenance: mode=max,
  sbom: true` (buildkit-attached registry attestation), which is a distinct mechanism from the
  `actions/attest-build-provenance` Sigstore attestation used for the archives/SBOM.
- **Why it (mildly) fails:** §4's "Verify: `gh attestation verify oci://… --repo jeswr/sparq` **or**
  `cosign verify-attestation`" lists `gh attestation verify` first for the buildkit-provenance image.
  `gh attestation verify` is designed for `attest-build-provenance`-produced attestations; the buildkit
  `mode=max` provenance is a cosign-style registry attestation and is most reliably verified with
  `cosign verify-attestation`/the registry attestation API. `controls.md` correctly hedges with "or
  `cosign verify-attestation …`", so this is **not a false claim** — just an imprecise ordering in the
  evidence pack that could send an auditor down a tool that may not validate that specific attestation.
- **Severity rationale:** low — precision only; the control (provenance attached to the image) is met.
- **Remediation:** in `evidence.md §4`, lead the container "Verify" with `cosign verify-attestation
  --type slsaprovenance …` (or the registry attestation API), and note `gh attestation verify` applies
  to the `attest-build-provenance`-signed archives/SBOM. Optionally state which verifier each
  attestation mechanism requires.

---

## Coverage note

**Assessed (verified against `origin/main`):** SL-B1-a/b/c/d, SL-B2-a/b/c/d/e, SL-B3-a/b/c, SL-S-a…j,
SL-V-a/b/c; the per-artifact honest-level table; the gap register (GX-8/9/10/11) and every cited bead
(`sq-toze.3/.8/.9/.14/.23/.24/.25`); SBOM↔VEX↔deny.toml consistency; SHA-pin completeness across all 18
workflows; top-level token permissions per workflow; Dockerfile base-image pins; `.well-known/security.txt`.

**Could not fully execute (environment-bound — verified by wiring, not by produced artifact):**
- The *runtime* embedded-manifest read (`cargo audit bin <released-binary>`) and the live
  `gh attestation verify` / Rekor inclusion proof — these require a tagged release build + the public
  attestations store, out of reach in the audit env. The **workflow wiring** that produces them is
  verified; the produced-artifact verification is the documented consumer step.
- The **live GitHub branch-protection ruleset** behind the `AR` SL-S-b/SL-S-c rows — by definition an
  out-of-repo setting; `docs/branch-protection.md` is the reproducible record and the engineer correctly
  labels it `AR`, not `IV`. An external assessor confirms the live setting.

**Standing external caveats (correctly out of agent + auditor scope):** the SLSA-level *certificate*
(accredited assessor), crates.io published-package provenance (no upstream mechanism exists today), and
deploy-time admission enforcement (operator-owned). These are honestly recorded, not failures.

---

## Disposition

The three findings are **documentation-precision corrections to the engineer's evidence docs**, not
control overclaims and not new code gaps — so **no new remediation bead is warranted** (the underlying
GX-8/9/10/11 work is already beaded as `sq-toze.23/.24/.25/.9`). Once the engineer corrects F-1…F-3 in
`compliance/slsa/{controls,evidence,gap-register}.md`, this slice is **clean for sign-off** within its
explicitly-bounded scope: **"sparq's official release archives and container image reach SLSA Build
Level 2,"** with the L3 gap, the dist.yml/published-package gaps, and the reproducibility gap all
honestly recorded.

**FINDINGS: 3** (F-1, F-2, F-3 — all low).
