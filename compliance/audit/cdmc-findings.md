<!-- [OPUS-4.8] CDMC scorecard audit — adversarial review of compliance/cdmc/* on cert-cdmc / PR #234. -->
# CDMC scorecard audit — findings

> 🤖 SPARQ agent. Independent, adversarial audit of the CDMC capability-maturity scorecard
> (`compliance/cdmc/{scorecard,controls,evidence,gap-register,recommendations,README}.md`) on
> branch `cert-cdmc` / draft PR **#234**, epic `sq-toze`. NON-CANONICAL timing. I verified each of
> the 14 capability ratings against the cited repo evidence on `origin/cert-cdmc` (file/line/test
> inspection + a CI-workflow read), with extra skepticism on the four **4**s and the ZK/MPC
> exclusion. I write only under `compliance/audit/`; remediation is the engineer's.

## Verdict

**FINDINGS: 1** — one **medium** rating-inflation finding on capability **2.1** (catalogue rated
**4**; the evidence supports **3** by the scorecard's *own* maturity-scale definition). Every other
rating (the other three 4s, all the 3s, all the 2s) is evidence-backed and honest. The ZK/MPC
estate is correctly and consistently excluded from all protection-by-crypto maturity. The 7-gap set
(CD-1..CD-7) is the real set, and CD-1/CD-2/CD-4 are beaded.

This is **not** a sign-off: see the single finding below. It is a narrow, mechanical inflation, not
a systemic honesty failure — the scorecard's tone, scope-framing, and ZK posture are exemplary.

---

## Finding 1 — [MEDIUM] Capability 2.1 (catalogue) is rated **4** but the cited evidence is not CI-gated; by the scorecard's own scale it is a **3**

**Control violated:** CDMC 2.1 "Data catalogues are implemented, used & interoperable", and the
scorecard's own maturity scale (`scorecard.md` lines 33-34): **level 4 = "The capability is
enforced, gated and has regression/conformance coverage"** vs **level 3 = "A real, documented,
tested capability exists and is exercisable today."**

**Rating I would assign: 3** (not 4).

**What I checked (commands/files/lines, all on `origin/cert-cdmc`):**

1. The catalogue surface is real and tested — *but feature-gated and OFF by default*:
   - `crates/sparq-server/src/descriptors.rs` is the whole catalogue module and is compiled only
     behind `#[cfg(feature = "federation-descriptors")]` (`crates/sparq-server/src/lib.rs:28-29`).
   - The feature is declared at `crates/sparq-server/Cargo.toml:95`
     (`federation-descriptors = ["server", "dep:sparq-introspect"]`) and is **not** in the default
     feature set. Even with the feature compiled in, the endpoints are served only when the
     operator sets the flag; with the flag off (the default) `GET /.well-known/void` returns `404`
     (`descriptors.rs:14-23`, `http.rs:243-247`).
   - The module has **6** `#[test]`s (`descriptors.rs`, `grep -c '#\[test\]'` = 6) covering VoID
     and Service-Description generation + content negotiation. These tests are genuine.

2. **None of those 6 tests run in any CI gate, and the module is not even compiled in CI:**
   - The CI test runner builds the archive with `cargo nextest archive --workspace --all-targets`
     (`.github/workflows/ci.yml:82`) — **default features only**, no `--features
     federation-descriptors` and no `--all-features`. A `#[cfg(feature=…)]` module under default
     features is not compiled, so its `#[test]`s are not in `nextest.tar.zst` and never run in the
     sharded `cargo nextest run --archive-file …` jobs (`ci.yml:218-231`).
   - `cargo clippy --workspace --all-targets -- -D warnings` (`ci.yml:300`) and the MSRV
     `cargo check --workspace --exclude …` (`ci.yml:337,43`) likewise use default features — they
     do not compile the descriptor module either.
   - `git grep federation-descriptors -- .github/` returns **no** match; `release.yml`,
     `docker.yml`, `docs.yml` do not enable the feature. Confirmed: **no workflow on `cert-cdmc`
     compiles or exercises the catalogue capability.**

3. Contrast with the genuinely-gated 4s:
   - **6.1 (architecture, 4):** `crates/sparq-serve/tests/{ring.rs,time_travel.rs}` (7 + 7
     `#[test]`s) are **not** feature-gated and run in the default `nextest --workspace --all-targets`
     lane. 6.1 = 4 is **earned**.
   - **4.1 (security, 4):** every cited CI workflow (`miri.yml`, `fuzz.yml`, `codeql.yml`,
     `scorecard.yml`, `supply-chain.yml`) genuinely runs and gates. 4.1 = 4 is **earned**.

**Why it fails:** The scorecard reserves **4** for a capability that is "enforced, gated and has
regression/conformance coverage." 2.1's evidence is a real, documented, tested-on-demand capability
that **(a)** is OFF by default, **(b)** has zero CI regression coverage, and **(c)** is not even
compiled in any CI lane — so a regression in `descriptors.rs` or `sparq-introspect::to_void`
serialization would pass all gates silently. That is precisely the level-3 definition ("real,
documented, tested, exercisable today"), not level 4. Rating it 4 — and counting it as a "genuine
sparq strength" pushing Component 2's average to ~3.5 (`scorecard.md:72`, `README.md:387`) — is a
mild over-claim against the scorecard's own rubric.

Secondary (same finding): the scorecard, `controls.md` (row 2.1, "Implemented & verified"), and
`evidence.md` (E1) describe the catalogue without disclosing that it is **opt-in, default-OFF,
feature-gated** and returns `404` in a stock build. For an auditor reading "VoID + SD endpoints …
a genuine, tested machine-readable catalogue surface," the default-disabled posture is material and
should be stated.

**Specific remediation (pick one; either clears the finding):**

- **Preferred (keep the 4):** add the feature to a CI lane — e.g. extend the nextest archive or add
  a dedicated job that runs `cargo test -p sparq-server --features federation-descriptors`
  (and/or include `federation-descriptors` in a `--features` matrix for clippy/check), so the 6
  descriptor tests gate every PR. Then 2.1's "gated + regression coverage" basis is real and the 4
  stands. **And** add one sentence to `scorecard.md`/`controls.md`/`evidence.md` 2.1 noting the
  capability is opt-in (feature `federation-descriptors`, default OFF, `404` until enabled).
- **Alternative (re-rate to 3):** change 2.1 to **3** in `scorecard.md` (headline table + the
  Component-2 "~3.5"→ recompute, and `README.md`), and `controls.md` (Maturity column), with a note
  that the catalogue is exercisable today but not CI-gated. Component 2 average drops to ~3.0.

**Tracking:** the CI-gating gap is a real codebase deficiency → I filed it as a bead under epic
`sq-toze` (see "Beads filed" below). If the engineer takes the re-rate path instead, the bead can be
closed as won't-fix with a doc note.

---

## Ratings I assessed and AGREE with (no finding)

| Cap | Score | Verdict | Key evidence I confirmed |
|---|---|---|---|
| 1.1 | 3 | OK | `SECURITY.md`, `research/threat-model.md`, `CODEOWNERS`, the cert estate — engine governance real; org data-control honestly operator-owned. |
| 1.2 | 2 | OK (honestly low) | No business-data-owner concept; only the named-graph hook + `to_void`. Operator-owned, correctly capped. |
| 1.3 | 3 | OK | `crates/sparq-engine/src/service.rs` SERVICE federation + `tests/service_federation.rs`; B4 outbound boundary documented (`research/threat-model.md:97,381`). |
| 2.2 | 3 | OK | SHACL Core **98/98 (100%)** (`crates/sparq-shacl/README.md:24`) + `tests/{w3c_core.rs,diff_fuzz.rs}`; taxonomy honestly operator-owned. |
| 3.1 | 3 | OK | `sparq-solid` WAC+ACP fail-closed (`authindex.rs`); **genuine** constant-time token compare (`http.rs:567` XOR-accumulate + `black_box`, not string `==`); ODRL honestly roadmap-only. |
| 3.2 | 2 | OK (honestly low) | `metrics.rs` is aggregate-only (`{endpoint,status}`, no identity label); grep found **no** per-subject audit log. Low score is honest. |
| 4.1 | **4** | **Earned** | `forbid(unsafe_code)` in **20** crate `lib.rs` (scorecard says "20+", honest); Miri/fuzz/CodeQL/Scorecard/supply-chain CI lanes all genuinely gate; four-limit DoS guards real. |
| 4.2 | 2 | OK (honesty anchor) | No personal data of its own; ZK/MPC explicitly excluded as NOT sound. Correct. |
| 4.3 | 2 | OK (honestly low) | mmap `.spq`/dict plaintext (no cipher in `store.rs` save); no TLS dep in `sparq-server/Cargo.toml`. Honest. |
| 5.1 | 3 | OK | `crates/sparq-engine/src/update.rs` full UPDATE set + **real** WAL/fsync/`txn.log` commit frame + crash-replay `#[test]`s; policy operator-owned. |
| 5.2 | 3 | OK | SHACL + reasoning quality engine; no standing program (operator-owned). |
| 6.1 | **4** | **Earned** | `crates/sparq-serve/src/ring.rs` `ArcSwap` generation-ring MVCC + **genuine** time-travel (`at()`/`as_of()` return pinned past snapshots), tested in the default CI lane; `store.rs` permutation indexes + mmap. |
| 6.2 | 2 | OK (honestly low) | Only partial provenance (named-graph + WAL + reason `explain`); no W3C-PROV. CD-1 gap. Honest. |

### The four 4s — explicit verdict
- **2.1 catalogue — NOT genuinely earned as a 4** (Finding 1; it is a 3 by the scorecard's rubric
  because the capability is not CI-gated).
- **4.1 security — earned.** Cited CI lanes genuinely gate.
- **6.1 architecture — earned.** Ring/time-travel are real and CI-tested (default lane).
- (There are exactly three 4s in the headline table — 2.1, 4.1, 6.1; 6.1 was the third "4" the
  brief flagged. All three checked.)

### The 2s — honestly low?
Yes. 1.2 (ownership), 3.2 (access audit), 4.2 (privacy/crypto), 4.3 (encryption), 6.2 (lineage) are
all genuine operator-governance decisions or genuine gaps. I actively grepped for a hidden richer
audit log (3.2) and any at-rest cipher / TLS (4.3) and found none — the low scores are not secretly
higher.

---

## ZK/MPC exclusion — CONFIRMED consistent (the critical honesty tripwire)

- `research/zk-soundness-audit.md:8` is intact: "v1 verifier soundness is BROKEN … DO NOT present
  v1 as proving anything to a relying party … honest research scaffold."
- The CDMC docs honor it everywhere: `scorecard.md:51,81-84`, `controls.md:142,164-165`,
  `evidence.md:E5`, `gap-register.md` ("Explicitly NOT gaps"), `README.md:395` all state ZK/MPC
  contributes **zero** to 4.2/4.3 and that crediting it would be over-claiming.
- Code reality matches the exclusion: `crates/sparq-server/Cargo.toml` and
  `crates/sparq-engine/Cargo.toml` have **no** dependency on `sparq-zk`/`sparq-mpc`/
  `sparq-zk-compose` — the dependency runs the other way (`sparq-zk` depends on `sparq-engine`
  behind a `zk` feature). ZK is a leaf, opt-in research scaffold; nothing in the audited
  server/engine protection path leans on it. **No CDMC rating launders the scaffold into a
  verified crypto control.** This tripwire passes.

---

## Gap set (CD-1..CD-7) — complete and correctly beaded

The 7 gaps map to the real sub-4 capabilities and there is no inconvenient capability quietly
omitted (all 14 capabilities appear in `controls.md`). Beading status (checked in
`.beads/issues.jsonl` on the main checkout):
- **CD-1 (PROV lineage, 6.2)** → `sq-ntcg` (open, labels `cdmc,cert`). ✓
- **CD-2 (access audit, 3.2)** → `sq-0bxp` (open, labels `cdmc,cert`). ✓
- **CD-4 (ODRL usage control, 3.1)** → `sq-r06h` (sparq-policy ODRL evaluator) + `sq-aiut`
  (federated, gated on ZK). ✓
- CD-3/CD-5/CD-6/CD-7 are P1/P2 doc/mechanism gaps with ready bead intents in `gap-register.md`;
  the lead files them at consolidation. No objection.

The gap-register's note that "`bd` is not on PATH in the cert worktree" was accurate at write time;
the lead has since filed the three task-named beads, satisfying the contract.

---

## Informational notes (NOT findings)

- **4.1 cites cargo-deny, which has a *degraded* advisories PR-gate** (`continue-on-error`, GX-1 in
  `research/production-certification-plan.md`). This does **not** undercut 4.1=4: the rating rests
  on the broad estate (Miri/fuzz/CodeQL/Scorecard/SLSA/distroless + cargo-deny bans/sources/
  licenses), most of which gates firmly; the advisories degradation is owned by the `sbom`/`ssdf`/
  `cra` slices, not a CDMC over-claim. Noted for cross-framework consistency only.
- **"20+ crates" vs the plan's "23 crates"** for `forbid(unsafe_code)`: I count **20** `lib.rs`
  with the attribute on `cert-cdmc`. The CDMC docs consistently say "20+", which is accurate and
  *not* inflated (the plan's "23" is the higher number; CDMC chose the conservative phrasing —
  good).

---

## Coverage note

**Assessed (all 14 capabilities):** 1.1, 1.2, 1.3, 2.1, 2.2, 3.1, 3.2, 4.1, 4.2, 4.3, 5.1, 5.2,
6.1, 6.2 — each rating checked against its cited file/test/CI evidence on `origin/cert-cdmc` via
direct git inspection (`git show`/`git grep`) plus a focused read of `.github/workflows/ci.yml`,
`miri.yml`, and the cited crate `Cargo.toml`s. The four-limit DoS, constant-time token, WAL commit
frame, ring time-travel, SHACL 98/98, and the ZK dependency direction were each spot-verified at
file:line.

**Could not fully run (and why):**
- I could **not** run `cargo test -p sparq-server --features federation-descriptors` to observe the
  6 descriptor tests pass: the `federation-descriptors` feature **does not exist on `main`**
  (it lives only on `cert-cdmc`/the #219 branch), and `cert-cdmc` is checked out in another
  worktree (locked), so I could not build it in my isolated worktree. I verified the tests' presence
  and the feature declaration by source inspection instead. This non-execution does **not** weaken
  Finding 1 — the finding is precisely that those tests are *not* run by CI, which I confirmed from
  the workflow files, not from a local run.
- External-auditor / external-cryptographer items remain **external** by standing caveat (an
  accredited CDMC assessor would issue the actual capability certification; the ZK soundness
  re-review is an external-cryptographer item per `research/zk-soundness-audit.md`).

---

## Beads filed (discovered codebase work)

- **`sq-kzfi`** — "CDMC 2.1: CI-gate the `federation-descriptors` catalogue tests + disclose opt-in
  posture" (Finding 1 remediation, preferred path) — epic `sq-toze` (parent-child), P1, labels
  `cdmc,cert`. Filed via `bd create` in the main checkout (not hand-edited). Currently the 6
  descriptor tests are never compiled/run in any workflow.

---

## Verdict line

**FINDINGS: 1** (medium — capability 2.1 rated 4, evidence supports 3 by the scorecard's own
maturity scale; remediate by CI-gating the catalogue feature *or* re-rating to 3, and disclose the
opt-in/default-OFF posture). All other ratings honest; the four 4s checked (2.1 not earned, 4.1 and
6.1 earned); ZK/MPC correctly excluded from all crypto-protection maturity; the CD-1..CD-7 gap set
is complete with CD-1/CD-2/CD-4 beaded. Standing caveat: external-auditor / external-cryptographer
sign-off remains external.
