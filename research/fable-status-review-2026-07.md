# Fable program-level status + gap review — 2026-07 [FABLE-5]

> 🤖 **SPARQ agent** — program-level advisory authored by Claude Fable 5 at the maintainer's
> request. Inputs: a prose status digest of the estate, the LIVE `bd` database, `gh` PR state,
> and targeted grounding greps (dedup checks only, not a code audit). Where a claim rests on
> the digest rather than something I verified live, it is marked **[inferred]**.

## (a) STATUS ASSESSMENT

**The core is genuinely solid, and the estate is in a deliberate consolidation phase.**
The published core (sparq-core / engine / cli / server / parse / canon / reason / shacl /
geo / text / hdt / wasm + js + py bindings) is conformance-ratcheted, coverage- and
mutation-gated, and the original roadmap threads are audited as landed
(research/roadmap-completion-audit.md). The last ~30 merges are overwhelmingly
test-quality (sq-qcnn), CI structure/selection (sq-6vshe, sq-fmx4u), ingest perf
(sq-7d3dj), and site/test infrastructure — hardening, not new capability. That is the
*right* phase for a 45-crate estate, but it should be time-boxed (see advice §4).

**Maturity tiering is honest and well-labelled.** ~26 publish=false crates are correctly
internal scaffolding; ZK/MPC carry explicit research-grade/not-audited labels enforced by
the privacy-claims gate; sparq-gpu is explicitly parked. The honesty doctrine (nothing
crypto-facing is production-sound until the external audit sq-qhy4 closes) is intact and
correctly wired into docs and CI. **[verified via bead estate + digest]**

**The flagship new-capability program (sq-6tykl) is mid-decomposition, half-resolved
already.** The digest flagged the reasoner sub-epics and the federation/geo/RSP epics as
undecomposed shells. Live `bd` shows the reasoner workstream epics sq-pbz04.1–.6 (RL/RDFS,
EL CR6–CR9, QL, Direct Semantics, RIF-Core, D-entailment) now EXIST; federation
(sq-my8wd), geo/text (sq-lk3aw) and RSP (sq-2n1q3) are still childless shells, with the
decomposition PR #1411 (FABLE) open and in scope for exactly those. Direct Semantics
(sq-pbz04.4) remains honestly labelled GREENFIELD.

**Critical meta-finding: the estate moves faster than its own reporting, and the
git-visible backlog export is materially stale.** The status digest I was given flagged
the 2026-06-23 improvement-survey items (per-block Bloom filters, Elias-Fano codec,
exact-bitmap semi-join reducer, RDF/XML parsing, TSV abbreviation, streaming Turtle,
N3 log:semantics cycle-safety, MULTIPLICITY()) as "possibly unbeaded". **All of them were
beaded and most were BUILT AND CLOSED within days of the survey**: sq-wihld (A1 Bloom,
landed opt-in `block-bloom`), sq-gr8mb (A3 bitmap semi-join), sq-5zf8i (A4 Yannakakis
prepass), sq-townn (A7 streaming Turtle/TriG), sq-0fj4p (A10 log:semantics cycle
detection), sq-f47w1 (B1 RDF/XML parse, opt-in `rdfxml`), sq-u79ee (C1 lexical-form
preservation), sq-v411r (B2 multiplicity(), corrected as a vendor extension by sq-ygkhf).
Open remainders sit deliberately at P3: sq-96hp1 (A2 Elias-Fano, measurement-gated),
sq-gcs5q (C3 JSON-LD expand/flatten surfaces), sq-koshe (RDF/XML follow-ups). The digest
was misled because `.beads/issues.jsonl` — the only git-visible view of the backlog — is
missing every bead created since roughly 2026-06-16 (verified: sq-wihld, sq-v411r, all
sq-pbz04.* return zero hits in the working-tree jsonl while `bd show` finds them live).
This is both a reporting hazard and a repeat of the known priority-invisibility failure
mode. Beaded as **sq-0b0sh**.

**Bottom line:** goals and backlog are in much better alignment than the digest suggested.
The genuine gaps are (i) the reporting/export substrate, (ii) one engine-side reasoner
correctness follow-up (below), and (iii) finishing the sq-6tykl decomposition so the
greenlit capability programs can start drawing implementation waves.

## (b) MISSING FROM BEADS

Verified against the LIVE `bd` DB (not the stale jsonl), 2026-07-03. I created beads only
for the two items that are clearly real; everything else I checked was already tracked.

**Created:**

1. **sq-0b0sh** — `.beads/issues.jsonl` git export is stale/incomplete vs the live DB;
   automate export freshness (post-mutation hook or CI drift gate). *Why it matters: every
   agent, digest, or review that reads the jsonl gets a wrong backlog picture — this very
   review's input digest did.*
2. **sq-dcer9** — sparq-reason: stratified-N3 / NAF-aware counting for incremental
   materialization. research/roadmap-completion-audit.md lists `reason_n3_stratified` +
   NAF-aware counting as an open follow-up with no engine-side bead (sq-3jtd.3 tracks only
   the sparq-solid auth-view research record). *Why it matters: counting-based incremental
   maintenance is sound for monotone rules only; combining incremental + N3 NAF is a
   silent-unsoundness shape. First step is cheap: confirm current behavior and fail closed.*

**Checked and NOT missing** (recorded so the next review doesn't re-derive this):
survey A1–A10/B1–B4/C1–C5 items (closed or open at P3, ids above); certification
gap-register items (tracked under `[cert]` beads, e.g. sq-toze.13 SSDF, sq-f8tv/sq-iy3p/
sq-d43g CRA); crypto-erase-at-rest (sq-du24 CLOSED as an explicit operator-owned defer —
respect it); mobile builds (sq-v286.9); ZK-inference (sq-rsd3v has children); scheduler
(sq-sgu1); specs program (sq-rvgr2 has children); MPC audit posture (sq-9hrn, sq-toze.20,
sq-qhy4 + scope-add sq-tcz0k); hdt 0.7 decision (sq-wzm4 closed); QLever divergence
(sq-ai2wa); paper factory (sq-gum8, thin but tracked).

**Not beaded, deliberately:** federation/geo/RSP child decomposition — in flight as
PR #1411; duplicating it would collide. **Post-merge verification advised** that
sq-my8wd / sq-lk3aw / sq-2n1q3 actually receive implementable children **[inferred that
\#1411 will create them; the PR was still open at review time]**.

## (c) NEEDS DEEPER (FABLE/OPUS) REVIEW — ranked

1. **ZK verifier estate (sparq-zk / zk-compose / Noir circuits).** Both the v1 BROKEN
   audit and the post-remediation "sound as landed" re-audit (sq-gbp4) are single-model
   internal self-audits; sq-qhy4 (external accredited cryptographer, P0, credential-gated)
   is correctly the hard gate. Highest-value interim work: keep the auditor dossier
   (sq-qhy4.1) current, keep folding scope-adds (sq-tcz0k pattern), and route any further
   adversarial internal passes to Opus per the standing model-routing constraint. Nothing
   here should be re-litigated by cheap-fleet agents.
2. **MPC estate (sparq-mpc — the largest crate — + fedplan-mpc).** Semi-honest
   honest-majority only, novel oblivious-join/attestation algorithms, internally reviewed
   (sq-9hrn, sq-toze.20) but never externally. Keeping the malicious-security beads frozen
   (sq-t21/sq-bjl/sq-h99) until the ZK audit posture resolves is right; a scope decision —
   whether sq-qhy4's external engagement covers MPC or a second engagement is needed — is
   a maintainer call worth an explicit answer.
3. **Newly-landed opt-in engine fast paths with equivalence claims.** The June wave landed
   block-bloom (sq-wihld), bitmap semi-join (sq-gr8mb), Yannakakis prepass (sq-5zf8i),
   radix-sort permutations, plus DPccp wiring still open (sq-clhn1). Each claims
   output-identical semantics; the differential-oracle program (sq-qcnn.2 family) is the
   right harness but still gaining its second independent oracle. A focused adversarial
   review of the equivalence arguments for the opt-in features is cheap and high-leverage.
4. **sparq-core unsafe surface + proposed NEW unsafe fast paths (sq-7d3dj.21).** Boundary
   B5 is well-instrumented (Miri lane, geiger, justification register); hold the line that
   any new `from_utf8_unchecked`-style path ships with a per-site justification + Miri
   coverage in the same PR.
5. **Reasoner soundness through substrate migration.** OWL-RL's sound-but-silently-
   incomplete posture (sq-pbz04.1 "honest RL-completeness push"), EL CR6–CR9, QL CQ-gate
   broadening — plus the new sq-dcer9 NAF gap, which is the only item here with a
   potential *existing* silent-wrongness mode. Triage sq-dcer9's step-1 (fail-closed
   check) ahead of the larger builds.

## (d) ADVICE TO THE ORCHESTRATOR

1. **Land #1411, then verify the decomposition actually populated sq-my8wd / sq-lk3aw /
   sq-2n1q3 with implementable children.** That single PR closes the biggest structural
   gap in the flagship program; the reasoner half is already live.
2. **Fix the reporting substrate first (sq-0b0sh) and derive all future status digests
   from live `bd`, never from issues.jsonl.** This review nearly shipped several
   false gap findings because of it; the cost of the fix is trivial next to the cost of
   another misled program review.
3. **Don't seed new epics — drain.** Backlog coverage is comprehensive (every digest
   candidate gap but two was already tracked). Marginal value is in draining ready
   frontiers (sq-7d3dj, sq-qcnn, sq-ymr2e, sq-oy1f, sq-6vshe) and in starting reasoner/
   federation implementation waves once #1411 lands — not in more design records.
4. **Time-box the consolidation phase.** Test/CI/perf-infra has dominated merges for weeks.
   The greenlit capability programs are the maintainer's stated priority; once the
   decomposition lands, rebalance waves toward sq-pbz04.1 (RL substrate adoption) and the
   federation lane, keeping the curated disjoint-crate wave pattern.
5. **Clear the Dependabot queue in one batched wave** (arrow x3, hdt 0.7.3 — decision
   already made in sq-wzm4 — n3, cargo-minor group), respecting the CI congestion-collapse
   cap. Separately, the two logo PRs (#207, #823) need a maintainer pick-or-close; they are
   design taste, not agent work.
6. **Escalate to the maintainer only the true externals:** sq-qhy4 auditor engagement (and
   the MPC-scope question in (c)2), publish tokens (sq-v286.7 / release PR #1084),
   OS code-signing (sq-v286.8), and the AWS quota block on hardware validation
   (sq-1sa9r). Everything else proceeds under the standing proceed-and-document rule.
7. **Keep crypto surfaces off the cheap fleet and off Fable.** Route ZK/MPC/adversarial
   review to Opus (known model-routing constraint); keep Fable on decomposition, program
   review, and non-security design verdicts — as this review was.

---
*Review inputs: prose estate digest (2026-07), live bd DB (503 open/in-progress beads at
review time), gh PR state, targeted grounding greps. Not a line-level code audit; crate
maturity claims beyond the beads/PR evidence are carried from the digest.*
