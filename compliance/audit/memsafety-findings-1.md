<!-- [OPUS-4.8] sq-toze — memsafety AUDIT round 1. Adversarial internal auditor.
     Re-review when Fable returns. -->

# Memory-safety attestation — audit findings, round 1

Auditor stance: skeptical, evidence-or-it-didn't-happen. I re-ran the ratchet, re-read every
cited workflow job and file, and probed the gating logic of the `ci-summary` aggregator
against the actual `unsafe-register` job. The control set is **substantively sound** — the
register/snapshot/ratchet triangulate, the B5 coverage matrix is honest about what Miri can
and cannot reach, and no claim contradicts the ZK-not-sound verdict (the `sparq-zk-compose`
`flock` sites are correctly scoped as *memory*-safety FFI, not a crypto guarantee). The
findings below are real but bounded — one is a live documentation overclaim that must be
fixed in the register itself (the engineer's gap-register says "corrected" but the register
file still carries the overclaiming sentence).

## Findings

### F1 — MEDIUM — register still asserts a clippy lint that is not enabled (MS-5 / MS-G2)
**Control:** MS-5 (per-site `// SAFETY:` justification, "local enforcement").
**What I checked:** `grep -n undocumented_unsafe_blocks compliance/memsafety/unsafe-register.md`
→ line 45 still reads *"clippy `undocumented_unsafe_blocks` is the local enforcement."*
`grep -rn undocumented_unsafe_blocks crates/ Cargo.toml` → the lint is set **only** in
`vendor/spargebra/Cargo.toml`, NOT in any first-party crate.
**Why it fails:** the register makes a control claim (a clippy lint enforces the `// SAFETY:`
token) that the codebase does not back. `controls.md`/`evidence.md`/`gap-register.md` already
acknowledge this (MS-5 caveat + MS-G2), but the **authoritative register file itself** still
overclaims — an external reader of the register alone would be misled. Misrepresentation in
the spine document is worse than the underlying gap.
**Remediation:** edit `unsafe-register.md` line 45 to state the enforcement honestly — the
`// SAFETY:` requirement is enforced by **the register + review + the count ratchet**, with
`clippy::undocumented_unsafe_blocks` listed as a *recommended, not-yet-enabled* gap (MS-G2).
Do not claim the lint until it is actually added to the 5 unsafe crates.

### F2 — LOW — "every site has a `// SAFETY:` comment" is literally false (6/56) (MS-5)
**Control:** MS-5 / register §"How each site is bounded" + the NEEDS-REVIEW section's claim
"every one of the 56 sites has … a corresponding `// SAFETY:` comment in source."
**What I checked:** counted literal `SAFETY:` tokens per crate → 50; identified the 6 sites
(`dict.rs:483`, `dict.rs:2192-93`, `dictspill.rs:720-21`) that use an *adjacent block
comment* with a sound argument but not the literal token.
**Why it fails:** the substance is fine (all 6 are documented + sound), but the register's
absolute phrasing "every one of the 56 sites has … a `// SAFETY:` comment" is inaccurate.
**Remediation:** either (a) normalise the 6 to the literal `// SAFETY:` token (preferred —
also unblocks MS-G2's lint), or (b) soften the register phrasing to "every site has a
documented safety argument (50 via the `// SAFETY:` token, 6 via an adjacent justification
comment)". Pick one; do not leave the absolute claim standing.

### F3 — LOW — cross-doc unsafe-count drift, threat-model says 39 vs register 42 (MS-G5)
**Control:** MS-2 (the count is the attested figure) vs `research/threat-model.md`.
**What I checked:** `research/threat-model.md` line 21 → "Yes — 39 sites"; register +
`bench/unsafe-snapshot.json` + `scripts/unsafe-gate.py --check` → 42 in sparq-core.
**Why it fails:** two repo documents disagree on the headline number. The register is
authoritative (it is the ratchet source), so this is a *stale neighbouring doc*, not a defect
in the attestation — but a relying party reading the threat-model would get the wrong count.
**Remediation:** MS-G5 already tracks the one-line fix to `threat-model.md`. Acceptable to
leave as a tracked gap since `threat-model.md` is owned by a different surface, **provided**
`evidence.md`/`gap-register.md` flag the discrepancy explicitly (they do). No change required
in the memsafety deliverables beyond keeping MS-G5 open.

## Assessed and PASS (no finding)

- **MS-1** confined surface — re-ran the grep; 20 forbid + 5 unsafe = 25 crates. Correct.
- **MS-2/MS-3** register + ratchet — re-ran `--check` (PASS, 56==56) and `--list` (TOTAL=56);
  per-crate matches snapshot + register. Verified the `unsafe-register` job has no
  `continue-on-error`, its name lacks the `\b(advisory|informational)\b` exclusion tokens, and
  `ci.yml` triggers on `pull_request`+`merge_group` so `ci-summary / gate` discovers and gates
  it. The ratchet is genuinely merge-blocking. Strong PASS.
- **MS-4** geiger informational — confirmed `continue-on-error: true` + name-exclusion; honestly
  labelled non-gating. PASS.
- **MS-6** Miri — confirmed `miri.yml` runs `cargo miri test -p sparq-core` nightly/dispatch
  only, with no PR trigger, and the header honestly states the 16 mmap + 7 dict-spill sites are
  structurally out of Miri's reach. No overclaim. PASS.
- **MS-7** oracle — `crates/sparq-core/tests/mmap_corruption_oracle.rs` exists with the named
  sweep/survives functions. PASS.
- **MS-8/MS-9** fuzz + ASan — `fuzz/fuzz_targets/graph_open.rs` is the genuine B5 mmap-loader
  target; `fuzz.yml` builds nightly with `-Zsanitizer` on the gnu target. The MS-9 caveat (no
  standalone ASan lane) is honest. PASS.
- **MS-10** clippy `-D warnings` — gating, not `continue-on-error`. PASS.
- **MS-11** dependency memory-safety — `cargo deny check advisories bans sources licenses` all
  gating (GX-1 un-degraded). PASS.
- **MS-12/MS-13** edition-2024 env sites + no-unsafe-in-untrusted-text-path — verified against
  the register rows + the forbid list. PASS.
- **ZK honesty tripwire** — the `sparq-zk-compose` flock sites are scoped as memory-safety FFI
  only; nothing in the memsafety deliverables claims a cryptographic guarantee or contradicts
  the v1-ZK-NOT-sound verdict. PASS (tripwire clear).

## Coverage note

I assessed all 13 controls + the coverage matrix + the ZK tripwire. I did **not** execute the
Miri/fuzz/oracle suites (heavy; the EC2 box is NON-CANONICAL for timing) — I verified their
existence, wiring, triggers, and scope instead, which is the correct evidence level for an
attestation audit. I did re-run the ratchet (`--check`/`--list`) since it is hermetic + fast.

## Verdict

`FINDINGS: 3` (1 medium, 2 low). **No critical/high.** No sign-off this round: F1 is a live
overclaim in the authoritative register file and must be corrected before sign-off. F2 should
be resolved (normalise or soften). F3 is acceptable as a tracked cross-doc gap. The
*substantive* memory-safety posture is sound; the findings are documentation-accuracy issues,
not safety defects.
