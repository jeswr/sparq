<!-- [OPUS-4.8] OpenSSF compliance audit — adversarial review of cert-openssf / PR #227
     against the ACTUAL repo. Epic sq-toze. Authored while Fable unavailable; re-review
     when Fable returns. Auditor: SPARQ agent. NON-CANONICAL timing. -->

# OpenSSF — independent audit findings (PR #227, branch `cert-openssf`)

> 🤖 SPARQ agent — adversarial compliance auditor. I independently re-read every cited
> workflow / doc / config on `cert-openssf` and probed for overclaiming, the ZK/MPC
> honesty tripwire, and a stale degraded-gate disclosure. This is a review, not a
> rubber-stamp.

**Verdict: FINDINGS: 1** (1 medium; plus 2 low advisories that do not block sign-off).

The OpenSSF posture is genuinely strong and the **single most important honesty check
passes cleanly**: the ZK/MPC research scaffolds are correctly and consistently excluded
from every `crypto_*` badge answer, and the "v1 verifier is NOT sound" verdict is intact
and cited. The one blocking finding is **stale evidence**: the engineer's docs disclose a
*degraded advisory PR-gate that no longer exists* — the actual repo has un-degraded it
(PR #210, bead sq-toze.2 **closed**). That contradiction would fail a docs-vs-repo
reconciliation and must be corrected before sign-off.

---

## Finding 1 — [MEDIUM] `Vulnerabilities` / `vulnerabilities_critical_fixed` / GX-1: docs disclose a degraded advisory PR-gate that the repo has already un-degraded (stale evidence, docs↔repo contradiction)

**Controls:** Scorecard `Vulnerabilities` (controls/openssf.md §A); Badge `vulnerabilities_critical_fixed` (evidence.md §Security); gap-register **GX-1**; Scorecard evidence narrative (evidence.md §Scorecard "Vulnerabilities").

**What I checked**
- `.github/workflows/supply-chain.yml` (cert-openssf): the `audit` job now runs **two
  gating steps** — `cargo deny check bans sources licenses` *and* a separate
  `cargo deny check advisories` (lines 51–66). There is **no `continue-on-error`** anywhere
  in the file (`grep -n continue-on-error` → only a code comment, line 54). The header and
  inline comments state plainly: "advisories is GATING again … the CVSS-4.0 parse blocker
  (sq-q8de) … is RESOLVED."
- `git log cert-openssf -- .github/workflows/supply-chain.yml` → top commit
  `d66726e ci(supply-chain): un-degrade cargo-deny gate … (sq-toze.2/.3/.8) (#210)`.
- `deny.toml` (cert-openssf): `[advisories] yanked = "deny"`, advisories v2 fail-closed,
  only **two** justified `unmaintained` ignores (RUSTSEC-2024-0436 `paste`,
  RUSTSEC-2025-0134 `rustls-pemfile`) — neither a vulnerability; each beaded.
- Bead lookup: the GX-1 fix bead **sq-toze.2** ("cargo-deny advisories PR-gate
  (un-degrade)") is **status: closed**.

**Why it fails**
The engineer's docs in **four places** still describe the *old, degraded* state:
- controls/openssf.md:41 — "PR-time *advisory* gating is degraded (cargo-deny CVSS-4.0
  parse bug, GX-1 / bead sq-q8de) — the daily watchdog is the real advisory gate."
- evidence.md:86 — "PR-time advisory gating is degraded by the cargo-deny CVSS-4.0 bug…
  the watchdog is the gate."
- evidence.md:127 — "(honest: PR-time advisory gating degraded — GX-1)."
- gap-register.md:24 — GX-1 listed as a current cross-framework gap, "the daily watchdog
  is the real advisory gate."

This is the **inverse** of the repo's actual state. `cargo deny check advisories` is now a
real PR/push/merge_group gate; the daily `dependency-monitoring.yml` watchdog is now
*defence-in-depth*, not "the real advisory gate." The `Vulnerabilities` IV evidence also
under-cites the gate — it lists only `bans sources licenses` as gating when `advisories`
is gating too. A relying party reconciling the OpenSSF evidence pack against the live CI
would find a direct contradiction; an external auditor would mark the evidence as not
matching the system. (Note: this is *not* a capability overclaim — the actual posture is
*better* than the docs claim — but stale/contradictory evidence is itself a defect, and the
persona's "is the degraded advisory PR-gate disclosed, not laundered?" check resolves to:
the gate is **not** degraded, so the disclosure is simply wrong.)

**Remediation (engineer)**
1. controls/openssf.md `Vulnerabilities` row → status stays IV, but cite
   `cargo deny check advisories` as **gating** (PR/push/merge_group) and reword the note:
   advisories un-degraded (sq-toze.2 closed, #210); the watchdog is now defence-in-depth.
2. evidence.md `vulnerabilities_critical_fixed` and the §Scorecard "Vulnerabilities" line →
   drop the "degraded" nuance; state the PR-time advisory gate is live + fail-closed
   (`yanked = "deny"`, two justified `unmaintained` ignores).
3. gap-register.md → **move GX-1 to the "Closed gaps" table** (closed by #210 / sq-toze.2),
   and remove it from the "cross-framework gaps that also touch OpenSSF" table, and from the
   "Honest bottom line" prose.
4. No new bead needed: the code fix already landed and is tracked by the **closed**
   sq-toze.2. This is a documentation-refresh only.

---

## Finding 2 — [LOW] Badge `warnings_strict` cites `cargo fmt --check` as part of the strict-warnings gate, but `rustfmt` is explicitly **non-gating** in CI

**Control:** Badge `warnings_strict` (evidence.md:75) and `warnings_fixed` (evidence.md:74).

**What I checked** — `.github/workflows/ci.yml`: header lines 5–6 — "rustfmt runs
**informationally** until the one-time `cargo fmt --all` reformat lands (deferred behind the
in-flight branches)"; line 301–303 confirm `cargo clippy --workspace --all-targets -- -D
warnings` **GATES** but `cargo fmt --all --check` is labelled "Informational until the
deferred `cargo fmt --all` reformat lands."

**Why it (mildly) fails** — evidence.md:75 lists `… + cargo fmt --check` alongside the
gating clippy as the "gold-level strict-warnings posture," implying fmt is enforced. It is
not gating yet. The *core* of `warnings_strict` (clippy `-D warnings`, full-workspace,
all-targets, hard gate) is genuinely met, so the criterion itself is honestly **Met** — but
the cited fmt evidence overstates what enforces it.

**Remediation** — In evidence.md `warnings_strict`/`warnings_fixed`, mark `cargo fmt
--check` as *informational (not yet gating — pending the deferred one-time reformat)*, so
the criterion rests only on the clippy hard-gate. Optionally beadable, but it is a one-line
doc accuracy fix, not codebase work.

---

## Finding 3 — [LOW] `Maintained` is double-mapped (Scorecard AR + Badge `maintained` Met) with no commit-cadence evidence beyond assertion

**Control:** Scorecard `Maintained` (controls/openssf.md:34, AR); Badge `maintained`
(evidence.md:39, Met).

**What I checked** — Both rows assert "active commit cadence on `main`" + "`SECURITY.md`
declares the posture." Scorecard `Maintained` reads last-90-days activity at scan time
(correctly labelled AR). The Badge `maintained` answer is asserted **Met** with the same
"active cadence" justification.

**Why it's a (low) concern** — The badge `maintained` answer is a self-cert that depends on
live activity the same way the Scorecard check does; asserting a flat "Met" is slightly
stronger than the AR posture the same evidence earns on the Scorecard side. This is not an
overclaim of a control sparq implements — it is an honesty-of-symmetry nit: the two views of
the *same* live signal carry different confidence labels.

**Remediation** — Either downgrade Badge `maintained` to "Met w/justification (live
cadence; same signal as Scorecard `Maintained`, AR)", or add the concrete cadence evidence
(e.g. last-release date + merged-PR count window) so both rows rest on the same checkable
basis. Cosmetic; does not block sign-off.

---

## Things I verified that PASSED (no finding)

**ZK/MPC honesty tripwire — PASS (the critical check).**
- `SECURITY.md` §"Scope and a critical caveat: research scaffolds with NO security
  guarantee" is intact: "`sparq-zk`/`sparq-zk-compose` — the v1 ZK verifier is **NOT
  sound**", `verify_manifest` "provides NO meaningful soundness guarantee", and `sparq-mpc`
  "cryptography deferred." `research/zk-soundness-audit.md` (present on cert-openssf)
  documents the 6 CRITICAL findings.
- evidence.md §Security: **every** `crypto_*` answer (`crypto_published`, `crypto_call`,
  `crypto_keylength`/`crypto_working`/`crypto_weaknesses`) is explicitly scoped to sparq's
  *own* delivery crypto (Sigstore/SLSA + TLS-at-operator) and **explicitly excludes** the
  ZK/MPC scaffolds, with the honesty anchor at the top of the file. `crypto_pfs` /
  `crypto_password_storage` / `crypto_random` → N/A (no auth/session crypto; boundary B3).
  **No `crypto_*` answer presents a research scaffold as a delivered guarantee.** This is
  the worst possible failure mode for this repo, and it is correctly avoided.

**Signed-Releases (IV) — PASS.** `.github/workflows/release.yml` uses
`actions/attest-build-provenance@a2bbfa2…` (Sigstore SLSA provenance) over **every** release
archive (`pkg/*.tar.gz`, `pkg/*.zip`), and a second attestation over `sbom/*.sbom.cdx.json`
+ `sbom/*.vex.cdx.json`. Container build sets `provenance: mode=max` + `sbom: true`.
`SHA256SUMS` covers all attached assets. The package/sbom/docker jobs carry the minimal
`id-token: write` + `attestations: write`; the `release` job is `contents: write` only.
(Nuance, not a finding: SHA256SUMS is *not itself* an attestation subject, but every asset
it covers is individually attested — the standard, correct pattern. The claim "Sigstore
provenance over every archive + SBOM + VEX" is accurate.)

**Pinned-Dependencies (IV) — PASS.** Every `uses:` across all 18 workflows is pinned by a
full 40-hex commit SHA (sweep found zero tag/branch refs). `Dockerfile` both stages
digest-pinned (`rust:1.88-slim-bookworm@sha256:38bc5a…`,
`gcr.io/distroless/cc-debian12:nonroot@sha256:b0ae8e…`).

**Token-Permissions (IV) / Dangerous-Workflow (IV) — PASS.** All 18 workflows declare a
top-level least-privilege `permissions:` block (read-only default); jobs opt into the
minimum write. No `pull_request_target` + untrusted-checkout; no untrusted `${{ }}` into
`run:` observed.

**SAST (IV) — PASS.** `codeql.yml` runs CodeQL `security-and-quality` over `rust`
(build-mode none) on push + PR + merge_group + weekly cron, SARIF → code-scanning. clippy
`-D warnings` gates in `ci.yml`.

**Fuzzing (IV) — PASS.** `fuzz.yml` runs cargo-fuzz (PR smoke 30s / push 120s / nightly
cron `23 4 * * *` 600s), run-all-then-fail (a crash blocks the merge), reproducer upload.
Plus `shacl-diff-fuzz.yml` (nightly differential).

**Security-Policy (IV) — PASS.** `SECURITY.md` (private channels + response targets +
scope caveats) and `.well-known/security.txt` (RFC 9116: dual `Contact`, `Expires:
2027-06-15` in the future, `Canonical`, `Policy`, `Preferred-Languages`). GX-3 genuinely
closed.

**CI-Tests (IV) — PASS.** `ci.yml` builds + `cargo nextest`/`cargo test --workspace`
(+ `--doc`) on every PR, aggregated by `ci-summary / gate`.

**Dependency-Update-Tool (IV) — PASS.** `.github/dependabot.yml` covers 4 ecosystems
(cargo, github-actions, npm, pip).

**License (IV) — PASS.** `LICENSE` = "MIT License" at root, SPDX-recognised.

**Binary-Artifacts (IV) — PASS.** No checked-in executables/archives/`.wasm` on
cert-openssf (tree sweep clean).

**dynamic_analysis_unsafe (Badge) — PASS.** `miri.yml` runs `cargo test -p sparq-core`
under Miri (`-Zmiri-tree-borrows`) over the pure-Rust unsafe surface and **honestly
documents** that the 16 mmap-backed sites structurally cannot run under Miri (covered by the
oracle + fuzz per `compliance/memsafety/unsafe-register.md`, present on cert-openssf).

**Live-state checks correctly labelled AR (not overclaimed).** Branch-Protection,
Code-Review, Maintained, Packaging, Webhooks are AR (score is whatever OpenSSF reads from
live GitHub state/history; `docs/branch-protection.md` is the doc-of-record). The
solo-maintainer Code-Review discount is disclosed (GX-OSSF-3). CII-Best-Practices is
honestly **GAP** (GX-4, badge not filed). `build_reproducible` is honestly **Unmet** (GX-8).
Registry-publish signing honestly **OPEN** (GX-OSSF-2). None of these is laundered.

---

## Coverage note

**Assessed (re-read the actual artifact on cert-openssf):** all 11 Scorecard checks the
engineer marks IV (Pinned-Dependencies, SAST, Token-Permissions, Dangerous-Workflow,
Dependency-Update-Tool, Fuzzing, Signed-Releases, Security-Policy, License, CI-Tests,
Binary-Artifacts) + Vulnerabilities; the 5 AR live-state checks (labels only — scores are
external); the 6 badge families incl. every `crypto_*` answer and `dynamic_analysis_*`;
SECURITY.md / security.txt / branch-protection.md / CONTRIBUTING secure-coding /
zk-soundness-audit.md / unsafe-register.md presence + content; deny.toml advisory policy;
the GX-1 / sq-toze.2 reconciliation.

**Could not fully assess (inherently external — standing caveat):**
- The **live Scorecard score** and the **live GitHub branch-protection ruleset** are read
  by OpenSSF/GitHub infrastructure at scan time and cannot be asserted from a file. The
  doc-of-record exists; the live state is the maintainer's to confirm (GX-OSSF-3). Correctly
  AR.
- The **bestpractices.dev badge filing** (GX-4) is a human-owned external step. Correctly
  GAP. I confirmed the drafted answers are evidence-grounded and the passing bar is met.
- I did **not** run a live `scorecard --repo=…` (no network/token from this worktree); the
  evidence narrative's local-repro instructions are reasonable.

## ZK exclusion verdict
**Correct.** ZK/MPC scaffolds are excluded from every `crypto_*` answer; the "v1 verifier
NOT sound" / "`sparq-mpc` no guarantee" verdict is intact and cited consistently across
SECURITY.md, evidence.md, and controls/openssf.md. No crypto control is overclaimed.

## Gap-register completeness verdict
**Honest but stale on one row.** GX-4 (badge filing), GX-OSSF-2 (registry signing),
GX-OSSF-3 (solo-maintainer review), GX-8 (reproducible build) are all correctly open and
honestly characterised; the closed gaps (GX-3, GX-5, GX-6) are correctly retired. The single
defect is **GX-1**, which is listed as an open/degraded gate but is in fact **closed**
(sq-toze.2 / #210) — see Finding 1. Fix that one row (move GX-1 to "Closed gaps") and the
register is complete and honest.

---

**FINDINGS: 1** (Finding 1 is blocking; Findings 2–3 are low and may be folded into the
same docs-refresh pass). Re-submit the corrected `compliance/openssf/{controls,evidence,
gap-register}` and I will re-audit for sign-off. Standing caveat: the live-Scorecard-score,
live-ruleset, and badge-filing items remain external (maintainer-owned) by their nature.
