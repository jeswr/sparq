<!-- [OPUS-4.8] sq-toze (cert-cis-audit) — CIS auditor findings. Adversarial re-verification of
     the cert-cis evidence (PR #231) against the ACTUAL repo. Authored while Fable unavailable —
     re-review when Fable returns. Engineer↔auditor loop (epic sq-toze). NON-CANONICAL timing. -->

# CIS slice — auditor findings (PR #231 / branch `cert-cis`)

**Auditor:** SPARQ agent (compliance-auditor persona), independent re-verification.
**Scope reviewed:** CIS Critical Security Controls v8 (artifact-applicable Safeguards) + CIS Docker
Benchmark v1.x §4. Evidence under `compliance/cis/{controls,evidence,gap-register,README}.md` on
`origin/cert-cis`, re-verified against the real `Dockerfile`, `.github/workflows/*`, `deny.toml`,
`supply-chain/*.toml`, `CODEOWNERS`, `.well-known/security.txt`, `.dockerignore`, and
`crates/sparq-server/src/{main,http}.rs` on the same ref.

**Verdict: `SIGN-OFF`** — zero open findings, with the standing caveat that the AUDIT-READY rows
(C-7.7 vuln-remediation-SLA record, C-16.13 external pentest) and any external-assessor certificate
remain external items the project cannot self-substitute. No new bead created (no codebase remediation
discovered; the two real gaps already carry beads `sq-toze.31` / `sq-toze.36`).

---

## What I independently checked (every PASS anchor re-run, not trusted)

All checks below were run against `origin/cert-cis` (commit `f5bca6c`) — I read the actual files, not
the engineer's transcription of them.

### Part A — Docker Benchmark §4 (verified line-by-line against the real `Dockerfile`)

- **D-4.1 (non-root):** runtime `FROM gcr.io/distroless/cc-debian12:nonroot@sha256:b0ae…`
  (`Dockerfile:76`); `git grep -nIE '^USER ' Dockerfile` → **no output** → no `USER root` override.
  The `:nonroot` distroless variant defaults to UID 65532. ENTRYPOINT (`:104`) inherits it. **PASS
  confirmed.**
- **D-4.2 / D-4.7 (digest-pinned, no floating `FROM` tag):** both `FROM`s carry `@sha256:…` digests
  (`:51` builder `rust:1.88-slim-bookworm@sha256:38bc…`, `:76` runtime). The `type=raw,value=latest`
  tag (`release.yml:223`) is a *published-image consumer tag*, not a `FROM` — the build never depends
  on a floating tag. **PASS confirmed.**
- **D-4.3 / D-4.8 (minimal, no setuid):** every `RUN`/install is in the discarded builder stage
  (`:64`, `:70`); the runtime stage only `COPY`s the single binary (`:84`). Distroless ships no
  apt/shell/coreutils → no setuid utilities, no `chmod u+s`. **PASS confirmed.**
- **D-4.9 / D-4.10 (COPY-not-ADD, no baked secrets):** `git grep -nIE '^ADD ' Dockerfile` → none;
  only `COPY` at `:57`, `:84`. No secret VALUE in the Dockerfile (only `SPARQ_ALLOW_REMOTE=1` posture
  env + header comments explaining `SPARQ_AUTH_TOKEN` is passed at `docker run -e`). `.dockerignore`
  excludes `.git/`, `.github/`, `.claude/`, `.vscode/`, `.idea/`, `target/`. **PASS confirmed.**
- **D-4.11 (locked/auditable build):** `Dockerfile:64` `cargo install --locked cargo-auditable`;
  `Dockerfile:70` `cargo auditable build --release --locked -p sparq-server`. `deny.toml` +
  `supply-chain/{config,audits}.toml` present (vet not vacuous). **PASS confirmed.**
- **D-4.5 (provenance + SBOM on push):** `release.yml:250-251` `provenance: mode=max` / `sbom: true`
  on the `build-push-action` docker job; archives + SBOM/VEX also `attest-build-provenance@v4.1.0`
  (`release.yml:108,145`). `scripts/gen-sbom-vex.sh` present. **PASS confirmed** (full SLSA-*level*
  claim correctly deferred to the `slsa` slice).

### Part B — CIS Controls v8 (artifact-applicable)

- **C-2.1 / C-2.4 (software inventory, automated):** CycloneDX SBOM in CI (`supply-chain.yml:106`
  `cargo cyclonedx --all --format json`) + per-release (`release.yml` sbom job → `gen-sbom-vex.sh`);
  `cargo auditable` embeds the manifest into the binary; Dependabot tracks 4 ecosystems
  (`.github/dependabot.yml`, 4 `package-ecosystem` entries). **PASS confirmed.**
- **C-2.6 / C-7.5 (allowlisting + dependency vuln scan, application side):** `supply-chain.yml`
  `deny` job runs `cargo deny check bans sources licenses` (`:52`, GATING) **and** `cargo deny check
  advisories` (`:66`, GATING) — I confirmed there is **no `continue-on-error`** on the deny job (the
  only `continue-on-error` occurrences are inside explanatory comments noting the *historical*
  GX-1/sq-q8de state that is now resolved). `vet` job `cargo vet --locked` (`:92`, GATING). Daily
  advisory watchdog `dependency-monitoring.yml` (`cron: 13 5 * * *`, `cargo deny check advisories`).
  CodeQL `analyze` (`codeql.yml`, on `pull_request`, feeds ci-summary). **PASS confirmed; the
  PARTIAL label on 7.5/7.6 is honest — only the image-OS half is the GX-12 gap.**
- **C-4.1 (secure-by-default artifact config):** verified in source, not just docs —
  `crates/sparq-server/src/main.rs` `--allow-remote` opt-in (fail-closed non-loopback bind, sq-o4qf);
  `http.rs:173-207` documents the no-auth-by-default posture and the `--allow-remote` /
  `auth_token`+`auth_token_read` refusal logic. **PASS confirmed.**
- **C-6.8 (CI access control):** `CODEOWNERS` has a real owner (`* @jeswr`, per-crate owners);
  `ci.yml:27-28` default `permissions: contents: read` with scoped per-job opt-ins; `release.yml:201`
  adds only `packages: write`. ci-summary `gate` aggregator (`ci-summary.yml`) auto-discovers all
  check-runs for the SHA (verified the aggregation logic at `:140-160`), so a future scan job *would*
  be gated. **PASS confirmed.**
- **C-7.1 (disclosure channel):** `SECURITY.md` present + `.well-known/security.txt` (RFC 9116,
  GX-3 closed). **PASS confirmed.**
- **C-16.x (SSDLC/SAST/threat-model/fuzz):** CodeQL gating (`codeql.yml`), clippy `-D warnings`,
  Miri (`miri.yml`), cargo-fuzz (`fuzz.yml`, `shacl-diff-fuzz.yml`), `research/threat-model.md`.
  Correctly cross-referenced to the `ssdf` slice. **PASS confirmed** (fuzz PASS / external pentest
  AUDIT-READY split is honest).

### The headline gaps — independently confirmed REAL

- **GX-12 (sq-toze.31) — no container-image CVE scan + no Dockerfile linter — CONFIRMED REAL.**
  `git grep -nIiE 'trivy|grype|dockle|hadolint|docker.?scout|anchore' origin/cert-cis -- '.github/'`
  returns **exactly two hits**, both genuine false positives: `ci-summary.yml:145` and
  `zk-toolchain.yml:31`, where the substring `anchore` appears inside the English word **"anchored"**
  in a comment. I also grepped `scripts/`, `*.sh`, `*.yml`, `*.yaml` workspace-wide → **no** Trivy/
  Grype/Dockle/Hadolint anywhere. The only container-touching CI is `docker-smoke`
  (`ci.yml:677-704`, `scripts/docker-smoke.sh`), which is a *functional boot+serve* gate, **not** a
  vuln scan — the engineer's distinction is accurate. Gap is real, P1, honestly disclosed.
- **GX-13 (sq-toze.36) — no `HEALTHCHECK` — CONFIRMED REAL.** `git grep -nIi HEALTHCHECK
  origin/cert-cis -- Dockerfile` → no output. The distroless-no-shell constraint the engineer notes
  (needs a static probe binary or a `--health-probe` subcommand) is correct. P3, honest.

### N/A-operator scoping — honest, not control-dodging

I specifically probed whether any "N/A (operator)" label hides a control sparq's *own code* touches:

- **C-6.x per-user auth labelled N/A(operator):** sparq does **not** dodge this — it *ships* an
  optional Bearer-token gate (`http.rs:190-207`, constant-time `constant_time_eq` compare, identical
  401 for missing-vs-wrong token, `WWW-Authenticate: Bearer`), which the slice explicitly credits as
  the artifact-side residual while correctly deferring per-*user* IAM to the operator per the
  documented B3 threat-model boundary. This is the honest split, not a dodge.
- **C-3.x data protection labelled N/A(operator):** correct for a data *engine* — the operator is the
  controller for loaded RDF; cross-referenced to the `privacy` slice.
- **Docker §5 runtime hardening / CIS §1/2/3/6/7 host-daemon / CIS v8 enterprise families
  (1/5/9/10/13/14/17/18):** these are genuinely properties of the operator's host/network/endpoint
  estate, not of a shipped Rust library + image. Each is *mapped* (listed in the control table /
  gap-register "explicitly NOT gaps") rather than silently dropped. Scoping is honest.

### ZK/MPC honesty tripwire — clean

No CIS row is satisfied by the ZK/MPC estate; the slice's honesty notes (`controls.md:12-13`,
`README.md` honesty note, `gap-register.md:294-300`) explicitly state CIS makes no crypto claim and
that the "v1 ZK verifier NOT sound" verdict is untouched. **No contradiction found.** No "verified
crypto control" claim exists anywhere in this slice.

---

## Coverage note

**Assessed (re-verified against the real repo):** Docker Benchmark §4 D-4.1…D-4.11 and the §5
N/A-operator mapping; CIS v8 C-2.1/2.4/2.6, C-4.1, C-6.8, C-7.1/7.3/7.5/7.6/7.7, C-16.1/16.11/16.12/
16.13/16.14, and the N/A-operator families (C-1/3/5/6-enduser/9/10/13/14/17/18). The headline GX-12
absence claim and GX-13 absence claim were both independently re-grepped.

**Could not fully assess (external / out-of-scope for a source audit):**
- **C-7.7 (remediation SLA met *over time*)** — AUDIT-READY; this is an organisational record an
  external assessor reviews, correctly labelled.
- **C-16.13 (external pentest engagement)** — AUDIT-READY; an org procurement, correctly labelled.
- **Live `docker run` runtime flags / Docker host & daemon config** — genuinely operator-side; not a
  property of the shipped artifact, so not auditable from source. Honestly N/A.
- I did **not** execute the Docker build or run Trivy myself (no claim depends on that; the gap is the
  *absence* of the lane, which a source grep proves). The `docker-smoke` *execution* needs a daemon;
  its *presence* as a gate I confirmed by source.

## Disposition

Every PASS evidence anchor resolved to the actual file/line/job claimed. The two gaps (GX-12 image
CVE-scan + Dockerfile linter; GX-13 HEALTHCHECK) are real, correctly severity-rated, beaded, and not
papered over. The Dockerfile hardening is genuinely strong (distroless nonroot, both stages
digest-pinned, locked auditable build, no shell/pkg-mgr/setuid, COPY-not-ADD, no baked secrets). The
N/A-operator scoping is honest and the ZK tripwire is clean.

**FINDINGS: 0 — SIGN-OFF** (external-assessor / external-pentest items remain external, as labelled).
