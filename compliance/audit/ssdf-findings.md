<!-- [OPUS-4.8] SSDF audit findings (framework `ssdf`, epic sq-toze, PR #228).
     Authored by the SPARQ SSDF compliance-auditor. NON-CANONICAL timing (EC2 work box). -->

# NIST SSDF (SP 800-218 v1.1) — audit findings (PR #228, branch `cert-ssdf`)

> 🤖 **SPARQ agent** — adversarial internal audit of the `compliance-engineer` SSDF slice
> (`compliance/ssdf/{controls,evidence,gap-register,README}.md`) against the **actual**
> `cert-ssdf` tree (`2a3778d`) and the live workflows under `.github/workflows/`. Independent
> verification: every cited gating job / file / script was opened or executed read-only; the
> engineer's files were **not** edited.

## Verdict

**FINDINGS: 2** (both medium/low — control-inventory **accuracy** defects; *no* substantive
overclaim and the ZK honesty tripwire is **clean**).

The substantive SSDF posture is sound: the 27→(actually 28) "implemented & verified" rows are
each backed by a real gating CI job / file / test that I confirmed substantiates the claim; the
13 "audit-ready" rows are honestly labelled (documented + automated, formal attestation = an
org/ISMS act, not silently upgraded); the single technical **gap** (PW.6.2 reproducible-build,
GX-8 / bead sq-toze.9) is the only real gap and is honestly recorded; and **no row launders the
`sparq-zk*`/`sparq-mpc` scaffold into a met cryptographic control** — the "v1 verifier is NOT
sound" verdict is intact and correctly treated as a disclosed limitation. The two findings below
are corrections a real SSDF assessor would raise when cross-footing the inventory and checking
task IDs against the publication; they must be fixed before a clean SIGN-OFF.

---

## Findings

### F1 — MEDIUM — Coverage-summary table does not cross-foot with its own rows

- **Control / artifact:** `compliance/ssdf/controls.md` "Coverage summary" table (lines ~106–112)
  + the same totals echoed in `README.md`, `evidence.md`, `gap-register.md`, and the engineer's
  PR description ("41 tasks → 27 implemented&verified / 13 audit-ready / 1 gap").
- **What I checked:**
  `grep -oE '\*\*(PO|PS|PW|RV)\.[0-9]\.[0-9]\*\*' controls.md | sort -u | wc -l` → **42** distinct
  task rows; and a per-group status tally over the actual table rows:

  | Group | **Actual rows** | Impl&Verified | Audit-ready | Gap | Summary table claims |
  |---|---|---|---|---|---|
  | PO | **13** | 7 | **6** | 0 | 12 / 7 / 5 / 0 |
  | PS | 4 | 4 | 0 | 0 | 4 / 4 / 0 / 0 ✓ |
  | PW | **15** | **11** | 3 | 1 | 14 / 10 / 3 / 1 |
  | RV | **10** | 6 | **4** | 0 | 11 / 6 / 5 / 0 |
  | **Total** | **42** | **28** | 13 | 1 | **41 / 27 / 13 / 1** |

- **Why it fails:** The control inventory mis-states its own totals. PO is 13 tasks not 12 (the
  audit-ready **PO.2.3** row pushes audit-ready to 6); PW is 15/11 not 14/10; RV is 10/4 not 11/5;
  the grand total is 42/28/13/1 not 41/27/13/1. An SSDF assessor cross-foots a control matrix as a
  first integrity check — a spine whose summary disagrees with its rows reads as an unreviewed
  inventory and undermines confidence in the (otherwise solid) evidence. This is *accounting*, not
  overclaiming: the individual rows are correct and honestly labelled; the roll-up is wrong.
- **Remediation:** Re-derive the coverage-summary table mechanically from the rows (PO 13 / PS 4 /
  PW 15 / RV 10 = 42; Impl&Verified 28 / Audit-ready 13 / Gap 1) and propagate the corrected
  totals to `README.md`, `evidence.md`, `gap-register.md` and the PR body. Tracked: bead **sq-ce97**
  (blocks epic `sq-toze`).

### F2 — LOW — `RV.1.4` is not a task id in SP 800-218 v1.1 (fabricated identifier)

- **Control:** `compliance/ssdf/controls.md` row **RV.1.4** ("Continuously monitor known-vulnerability
  sources for the software's components") + the `evidence.md` §2 row mapping `dependency-monitoring.yml`
  to "RV.1.4".
- **What I checked:** Enumerated the RV rows — RV.1.1, RV.1.2, RV.1.3, **RV.1.4**, RV.2.1, RV.2.2,
  RV.3.1–RV.3.4. SP 800-218 v1.1 defines RV.1 with exactly **RV.1.1 / RV.1.2 / RV.1.3** (RV total =
  9 tasks); there is **no RV.1.4**. The "monitor sources" obligation is part of RV.1's existing text
  (gather/monitor information about potential vulnerabilities), not a separate numbered task.
- **Why it fails:** The mapping introduces a task identifier that does not exist in the framework it
  claims to map. The **underlying control is real and well-evidenced** — the daily advisory watchdog
  (`dependency-monitoring.yml`, cron `13 5 * * *`, idempotent `security:dependency-vuln` tracking
  issue) plus the PR-time `cargo deny check advisories` **GATING** job (`supply-chain.yml#audit`) plus
  Dependabot — so this is **not** an overclaim of a non-existent capability. But citing it against a
  non-standard id is a mapping-accuracy defect an assessor checking IDs against the publication will
  flag, and it is the proximate cause of the RV miscount in F1 (it inflates RV to 11 in the summary).
- **Remediation:** Either fold the continuous-monitoring evidence under the correct standard task
  (RV.1.3 / RV.1's monitoring text) or keep the row but **footnote it explicitly** as a sparq-local
  sub-task that is *not* a standard SP 800-218 id (so the matrix never asserts a fabricated control
  number). Tracked: bead **sq-ce97**.

---

## What I verified holds (no finding)

These are the load-bearing claims I independently confirmed against the real tree/CI, so the
engineer gets credit and the next round need not re-litigate them:

- **cargo-deny advisories is GATING again (GX-1 closed).** `supply-chain.yml` `audit` job runs
  `cargo deny check bans sources licenses` **and** `cargo deny check advisories` with **no**
  `continue-on-error`; `deny.toml` is fail-closed (`yanked = "deny"`, advisories-v2 fails on any
  unignored advisory; the only two `ignore`s are justified `unmaintained` crates each with a bead).
  The job name `cargo-deny (advisories + bans + sources + licenses)` does **not** contain the whole
  word `advisory`, so the `ci-summary` `\b(advisory|informational)\b` exclusion does **not** drop it
  — it gates. **PW.4.4 / RV.1.x / PO.3.2 substantiated.**
- **cargo-vet is GATING.** `supply-chain.yml` `vet` job runs `cargo vet --locked`; name has no
  advisory/informational token → gates; `supply-chain/config.toml` imports the six trusted audit sets
  (Mozilla/Google/Bytecode-Alliance/ISRG/Embark/Zcash). **PW.4.1 substantiated.**
- **Unsafe-count ratchet is a real, fail-closed GATE — and is NOT cargo-geiger.** `ci.yml`
  `unsafe-register (count ratchet)` runs `python3 scripts/unsafe-gate.py --check` (a deterministic
  source scan vs `bench/unsafe-snapshot.json`); I ran it on `cert-ssdf`: PASS, live total 56 = snapshot
  56 across sparq-core(42)/vectors(9)/cli(2)/zk-compose(2)/bench(1). cargo-geiger is a **separate**
  `unsafe report (cargo-geiger, informational)` job with `|| true`, explicitly non-gating. The
  persona's "is the ratchet really just informational geiger?" trap is **avoided**. **PO.3.2 / RV.3.4
  substantiated.**
- **`#![forbid(unsafe_code)]` posture is real.** 20 of 23 crate `lib.rs` forbid unsafe; the five
  unsafe-bearing crates correctly do **not** forbid; the per-site register
  (`compliance/memsafety/unsafe-register.md`, 67 rows) covers the 56 sites. **PW.5.1 substantiated.**
- **Coverage ratchet GATES.** `ci.yml` `coverage` job runs `coverage-presence.py --check`
  (test-presence) + `coverage-gate.py --check-robust` (per-crate floor, fails if still below floor
  after K re-measures). **PO.4.1 substantiated.**
- **Release provenance / SBOM / VEX / cargo-auditable.** `release.yml` runs `cargo auditable build
  --release --locked`, `actions/attest-build-provenance` over **both** the archives **and** the
  SBOM+VEX, `scripts/gen-sbom-vex.sh` (+ checked-in `supply-chain/vex.cdx.json`), `SHA256SUMS`, and the
  container build sets `provenance: mode=max` + `sbom: true`. **PS.2.1 / PS.3.1 / PS.3.2 / PW.6.1
  substantiated.** (The SSDF slice correctly does **not** assert a numeric SLSA *level* — that is the
  SLSA framework's row — so no SLSA-level overclaim here.)
- **CodeQL / fuzz / Miri honestly scoped.** `codeql.yml` runs `security-and-quality` on
  push/PR/merge_group/schedule (gates). `fuzz.yml` runs PR-smoke + merge_group + nightly over 5 real
  targets (`fuzz/fuzz_targets/{parse_rdf_str,parse_sparql,load_reader_parallel,graph_open,validate_shacl}.rs`)
  — PW.8.2 negative testing substantiated. `miri.yml` is **schedule-only (nightly)**, intentionally
  with no PR/merge_group trigger — and the controls/evidence label it "nightly", so it is **not**
  overclaimed as a PR merge gate. **PW.7.x / PW.8.x substantiated and honestly bounded.**
- **`ci-summary / gate` aggregator is real and its advisory-exclusion rule is sound.** It polls all
  sibling check-runs on the head SHA, self-excludes by run-id, and fails iff a non-advisory check is
  non-passing — so the GATING jobs above genuinely block merge. **PO.2.1 / PO.3.3 / PO.4.2
  substantiated.** (Live branch-protection *selection* of `ci-summary / gate` is a GitHub
  server-side setting; `docs/branch-protection.md` is correctly labelled the doc-of-record and the
  mechanism is real — the inherent "can't read the ruleset from the tree" limit is honestly flagged,
  not overclaimed.)
- **ZK/MPC honesty tripwire — CLEAN.** `SECURITY.md` §"research scaffolds with NO security guarantee"
  and `research/zk-soundness-audit.md` both preserve the verdict: **the v1 ZK verifier (`verify_manifest`)
  is NOT sound** (six CRITICALs intact) and `sparq-mpc` provides no guarantee. `controls.md` explicitly
  scopes PW.4/PW.5/PW.8 to the **engine** (`sparq-core`/`-engine`/`-server`) and treats the crypto
  estate's disclaimer as a **correctly-disclosed limitation, not a met control**. No SSDF row claims a
  cryptographic guarantee. **This is the worst possible failure mode and it does not occur.**
- **Audit-ready labels are HONEST.** The 13 audit-ready rows (PO.1.1/.1.2/.2.2/.2.3/.5.1/.5.2,
  PW.2.1/.9.1/.9.2, RV.1.2/.2.1/.3.1/.3.2) are documented + automated but their continuous
  operation / formal attestation is an org/ISMS act SSDF leaves to the producing org; none is
  silently upgraded to "met cert." Correct.

## Coverage note

- **Assessed:** all 42 task rows in `controls.md`; all 14 CI jobs in `evidence.md` §2 (opened/ran
  each); the release/provenance artifacts in §3; the governance artifacts in §1
  (`SECURITY.md`, `.well-known/security.txt`, `CODEOWNERS`, `docs/branch-protection.md`, `deny.toml`,
  `supply-chain/config.toml`, `compliance/memsafety/unsafe-register.md`,
  `research/{threat-model,zk-soundness-audit}.md`); the single gap (PW.6.2 / GX-8 / sq-toze.9); and the
  ZK/MPC honesty tripwire.
- **Could not fully verify (inherent, not a finding):** (a) the *live* GitHub branch-protection ruleset
  (server-side; doc-of-record present, mechanism real); (b) actual published-release provenance
  attestations (`gh attestation verify` needs a tagged release — the workflow steps are present and
  correct); (c) the nightly Miri/fuzz *run history* (workflows correct; schedule cannot be executed
  in this audit). None of these alters a status label.
- **Standing external caveat:** SSDF ships no certificate (it is an implementer-side practice
  framework); "audit-ready" here means evidenced + ready for an org to assert, and external-auditor /
  external-cryptographer items remain external — consistent with the engineer's framing.

## Disposition

Address **F1** and **F2** (both → bead **sq-ce97**, blocks `sq-toze`). They are accounting/ID-accuracy
fixes, not control work — once `controls.md` cross-foots and `RV.1.4` is corrected/footnoted, this
slice is sign-off-ready (with the standing external caveat). The substantive control set, the
audit-ready labelling, the single honest gap, and the ZK-not-sound disclosure all pass.

**FINDINGS: 2**

---
*Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>*
