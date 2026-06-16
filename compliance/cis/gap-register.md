<!-- [OPUS-4.8] sq-toze (cert-cis) — CIS gap register. Authored while Fable unavailable —
     re-review when Fable returns. Engineer↔auditor loop (epic sq-toze). -->

# CIS — gap register

Open gaps for the CIS slice (CIS Controls v8 + CIS Docker Benchmark §4). Each carries a severity, a
concrete remediation, a target, and the **`bd` bead** that tracks the fix (under epic `sq-toze`).
No gap is papered over; the closed cross-cutting gaps that this slice *depends on* are listed at the
bottom as context (owned by other slices).

Severity: **P0** blocks a defensible CIS posture; **P1** needed for a clean score; **P2/P3**
maturity/nice-to-have.

## Open gaps (owned by this slice)

| ID | Gap | Sev | CIS mapping | Remediation | Bead |
|---|---|---|---|---|---|
| **GX-13** | **No `HEALTHCHECK` instruction in the `Dockerfile`.** The server exposes `/health` (curled by `docker-smoke.sh`) but the image does not self-declare a healthcheck, so a bare `docker run` cannot report unhealthy-but-running. | **P3** (minor) | Docker Bench **§4.6**. | Add `HEALTHCHECK` against `127.0.0.1:3030/health`. **Constraint:** distroless has *no shell and no `wget`/`curl`*, so the check cannot be a shell command — it needs either a tiny static probe binary `COPY`d in or a `sparq-server --health-probe` subcommand invoked in exec-form. Track the small server addition in the bead. Orchestrators (k8s) typically use their own probes and ignore the image HEALTHCHECK, hence P3. | **sq-toze.36** |

## Explicitly NOT gaps (architectural decisions / operator responsibility)

These are recorded so the auditor sees they were *considered and scoped*, not missed:

- **Container runtime hardening** (`--read-only`, `--cap-drop=ALL`, `--pids-limit`, non-root
  `--user`, seccomp/`--security-opt`) — set at `docker run`, **operator responsibility**, documented
  in `Dockerfile:16–34` + `crates/sparq-server/README.md`. CIS Docker §5 = **N/A (operator)**, not a
  gap.
- **`sparq-server` has no per-user authentication** — the documented **B3** threat-model decision
  (`research/threat-model.md`); the image ships an optional Bearer-token gate and the guidance to
  front it with a gateway / sparq-solid. End-user IAM (CIS 6.x) = **N/A (operator)**, mapped in the
  `asvs` slice as an explicit decision, **not** a silent CIS gap.
- **The `latest` consumer tag** on the *published* image is a deliberate convenience tag *alongside*
  semver tags (`release.yml` metadata); the *build* never depends on a floating `FROM` (both `FROM`s
  are digest-pinned), so CIS Docker §4.7 is PASS, not a gap.
- **The CIS Docker §1/2/3/5/6/7 host/daemon/runtime/swarm families** — properties of the operator's
  Docker host, not of a shipped image. N/A (operator).
- **The CIS v8 enterprise families** (asset inventory C-1, account mgmt C-5/6 end-user, malware
  C-10, network monitoring C-13, awareness C-14, IR program C-17, pentest program C-18) —
  organisational/endpoint/network controls of the deploying org. N/A (operator).

## Dependencies (closed gaps in other slices this slice relies on as evidence)

These are **not** CIS-owned; they back CIS rows and are already closed/tracked elsewhere:

- **GX-1** (cargo-deny advisories PR-gate un-degraded) — backs C-7.5 application side. Closed
  (`sq-toze.2`, `supply-chain.yml#deny`).
- **GX-2** (per-release CycloneDX SBOM + VEX) — backs C-2.1/2.4 + the GX-12 allowlist's VEX
  alignment. Closed (`sq-toze.3`).
- **GX-3** (RFC 9116 `.well-known/security.txt`) — backs C-7.1. Closed (`sq-toze.4`).
- **GX-7** (cargo-auditable + cargo-vet) — backs D-4.11 + C-2.6/16.11. Closed (`sq-toze.8`).

## Addressed (closed by this slice)

- **GX-12** (container-image vuln scan + Dockerfile linter) — **ADDRESSED** (`sq-toze.31`,
  `.github/workflows/container-scan.yml`). The lane GATES via the `ci-summary` aggregator:
  `hadolint` lints the `Dockerfile` against the CIS Docker Benchmark (config `.hadolint.yaml`,
  `failure-threshold: warning`; the one finding — DL3059, info-level intentional RUN layering —
  is waived by rule code with a documented reason), and `trivy` builds the server image and scans
  its OS+library layers, failing on **fixable HIGH/CRITICAL** (`ignore-unfixed`), with a checked-in
  `.trivyignore` allowlist and a SARIF upload to code-scanning.
  Runs on PRs touching the image, on push-to-main, in the merge queue, and weekly to re-scan the
  unchanged base. Covers Docker Bench **§4.4** and the image-OS half of CIS v8 **7.5/7.6**.
  The `.trivyignore` allowlist carries the **unfixable** distroless-base OS CVEs (16 glibc/libssl
  findings, all LOW/MEDIUM with no Debian-12 fix and non-reachable in sparq-server's code path —
  each justified per-line in the file; `chore-codescanning-triage`, [OPUS-4.8]); these are the only
  Trivy code-scanning alerts and are now suppressed at scan time so they no longer report. NO
  fixable HIGH/CRITICAL is silenced — the honesty rule still holds.
  All third-party actions SHA-pinned. *(Trivy/Hadolint were not run end-to-end locally — no docker
  daemon in the authoring env — but hadolint ran clean against the real Dockerfile with the config,
  and actionlint passed; CI is the canonical run.)*

## Honesty statement

The CIS slice has **no remaining P1 technical gap** — GX-12 (image-CVE-scan + Dockerfile linter) is
now addressed (see above) — leaving one P3 minor gap (GX-13, HEALTHCHECK). The `Dockerfile`
hardening itself is verified PASS against the actual file; the closed gap was *automated
scanning/linting* coverage, not a hardening deficiency. Nothing here is satisfied by the ZK/MPC
estate, and no CIS claim contradicts the documented "v1 ZK verifier NOT sound" verdict.
