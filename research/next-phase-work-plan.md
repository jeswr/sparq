# Proposed next-phase work plan for sparq — for maintainer steer [FABLE-5]

> **Status: PROPOSAL (2026-07-05). This is a plan for the maintainer to redirect, not a
> committed roadmap.** Authored by Claude Fable 5 at the maintainer's request, from the
> live estate after a large drain session. Every workstream traces to one or more of the
> maintainer's four repeatedly-stated attributes: **(P) performance, (C) correctness,
> (F) usefulness of features, (G) usefulness of the genAI features for the agent to
> self-improve.** Numbers are not fabricated; speculative items are marked *(spec.)*.
> Steer issue: see the "Proposed next-phase work plan — for maintainer steer" issue.

---

## Executive summary (60-second read)

The last session **closed or de-risked five programs**: the engine-split (2 seams shipped,
the rest parked on measured evidence — `engine-split-rfc.md` §10), the research-KB
**infrastructure** (connector → extractor → seeds → timestamps → tiers → per-merge ingest →
private dump repo, all merged), the GUI consolidation epic (15/15), the ZK correctness wave
(`KNOWN_FAILING` empty in-tree and in both published faces; the ieee754 unconstrained-hint
forge-map audit closed as sq-qhy4 prep), and CI structural speedup phase 2. Three of four
PSS integration asks shipped same-day.

That leaves one thing conspicuously **built but unused**: the whole KB apparatus exists and
**no live knowledge has ever flowed through it** — the pilot is the maintainer's own stated
core motivation (item G) and it is one arm away. The single highest-leverage next move is to
**close the self-improvement loop**.

**Recommended lead: build the KB pilot harness now (dry-run, fixture-tested — dispatchable
without you); you arm only the first live run.** Four more workstreams follow, three
dispatchable immediately, the biggest unlocks gated on decisions listed in §7.

| # | Workstream | Attr | Dispatchable now? | Gated on |
|---|-----------|------|-------------------|----------|
| WS‑1 | Close the KB self‑improvement loop | **G** F | Harness: **yes**. Live run: no | sq‑tzars.9 arm; sq‑tzars.5 §6.1/6.2/6.4 |
| WS‑2 | Correctness → external‑audit readiness | **C** G | Dossier: **yes**. Audit: no | sq‑qhy4 (schedule the audit) |
| WS‑3 | Performance — measure the real poles, then cut | **P** | 3a: **yes**. 3b: no | EC2 bench greenlight re‑confirm |
| WS‑4 | Reasoner + federation + RSP/geo build‑out | **F** | Substrate: **yes** | sq‑ohnj1 (drop‑in lane only) |
| WS‑5 | Mature the collaboration tier toward autonomy | F **G** | **yes** (as measured opt‑in) | — |

---

## 1. WS‑1 — Close the KB self‑improvement loop *(attributes G, F)*

**Why now.** The entire research-KB pipeline merged this session and has processed **zero
live records**. The maintainer's brief was explicit: build the KB *so the agent can
self-improve*, and *iterate on the ingestion workflow*. The apparatus is the means; the
**loop is the payoff**, and it has never been closed once.

**Crucial framing for dispatch:** sq-tzars.9's *acceptance criteria is the DRY-RUN harness*
— prereg-before-extraction ordering, dry-run gate (no KB mutation until a recorded audit
passes the pre-registered bar), hard caps fail-stop, append-only verbatim metrics — all
fixture-tested, **no live data and no key use**. That harness is dispatchable now. Only the
*first live run PR* is maintainer-armed (first real data entering the KB). So we can
front-load and de-risk the whole thing before you ever have to decide.

**First deliverables**
1. **sq-tzars.9 — the pilot harness** (bin `literature_pilot.rs`, dry-run, fixture-driven,
   maintainer-arm only for live). Dispatchable immediately.
2. **A "does KB grounding measurably help" A/B** *(new bead)*. We already have a real
   measurement precedent — `bench/pkg-dogfood/RESULTS.md` (the pkg-query N=30 study showed
   the Haiku NL tool cheaper at equal quality, sq-zbyo7). Extend it to the *outcome*
   question: on a set of held-out implementation tasks, does an agent given KB-grounded
   context produce better/cheaper/more-honest work than one without? This is the honest
   dogfooding test of item G — and it doubles as the evidence that decides whether to invest
   further in the KB at all.
3. **sq-tzars.5 decision package** — assemble the §6.1 (DQV-Note posture), §6.2
   (research-verdict enum), §6.4 (confidence calibration) option tables for a maintainer
   ruling; two of these are already effectively answered by shipped code (option tables are
   on #1111).

**Risk / dependency.** The live run needs your arm and (for the dump-automation half)
**#1552 KB_DUMP_TOKEN**. The A/B is the one genuinely novel measurement — design it
adversarially or it will flatter the KB.

**Size.** Harness: small (spec is tight, deps all merged). A/B: medium (a real experiment).
Decisions: your call, ~zero eng.

---

## 2. WS‑2 — Correctness → external‑audit readiness *(attributes C, G)*

**Why now.** **sq-qhy4 (external accredited-cryptographer audit of the ZK verifier + Noir
circuits) is the single biggest credibility unlock in the whole repo** — it gates *every*
production ZK security claim. It is maintainer-gated on actually commissioning the audit,
but the work that maximizes its value is dispatchable now, and this session produced a
template for it: the sq-l9ulg ieee754 forge-map audit (every reachable unconstrained-hint
site hand-traced to its binding constraints, latent gaps beaded) is exactly the kind of
input an external auditor should be handed rather than a cold codebase.

**First deliverables**
1. **Audit-readiness dossier** *(new bead)* — consolidate into one artifact the external
   cryptographer starts from: the forge-maps (sq-l9ulg + per-circuit), the mechanized-proof
   coverage matrix (Kani harnesses + the anti-vacuity domain self-checks landed this
   session), `research/threat-model.md`, and the honest soundness caveats. Turns a diffuse
   estate into a reviewable package and surfaces gaps *before* the paid clock starts.
2. **Next proof frontier** — anti-vacuity self-checks are in; the ripe next reach is the
   mechanized *differential* harness (sq-3x7dl.14.2: xpath circuits vs the trusted Rust
   XSD/XPath evaluator, wired into CI) and completing the bounded-Kani coverage matrix
   (sq-og8u8's follow-ups). Both are correctness-attribute wins independent of the audit.

**Risk / dependency.** The payoff (retiring "not externally audited" on the ZK surface) is
**gated on sq-qhy4** — a maintainer commissioning decision. The prep is not.

**Size.** Dossier: medium. Differential harness: medium-large.

---

## 3. WS‑3 — Performance: measure the real poles, then cut *(attribute P)*

**Why now.** The engine-split D2 measurement (sq-aqr2f) produced a **real and surprising
signal**: on a cold build the **GPU stack (naga/wgpu-hal/wgpu-core → sparq-gpu) dominates
the critical path**, not the engine. That reframes the cross-cutting optimization epic
(sq-7d3dj) and raises a concrete question worth answering. Separately, the maintainer
explicitly flagged that the benchmarks page is **missing competitor baselines**
(sq-vw3ax.12) — a credibility gap on the performance story itself.

**First deliverables**
1. **GPU-stack critical-path investigation** *(new bead)* — is the GPU stack needed on the
   *default / cold* build+run path at all, or can it be feature-gated out of the critical
   build graph? This is simultaneously a build-time win (removes the cold pole) and a
   runtime/bundle question. **Measure-first**, per the standing doctrine — the answer may be
   "it's load-bearing," in which case we say so and move on. *(spec.: the size of the win is
   unmeasured until this runs.)*
2. **EC2 competitor-baseline gather** (sq-vw3ax.12) — Oxigraph / QLever / Fuseki / Virtuoso
   on the *same canonical hardware* (work-box numbers are non-canonical; published paper
   numbers aren't same-hardware comparable). The orphan-proof EC2 bench infra already exists
   and is proven; a Wave-0 validation already ran.

**Risk / dependency.** WS-3b needs a **re-confirmed EC2 bench greenlight** ($ + the standing
orphan-proof self-terminate discipline). WS-3a is measure-first and honesty-bound — no
number ships to docs/tests from the work box.

**Size.** 3a: small-medium (mostly measurement). 3b: medium, EC2-gated.

---

## 4. WS‑4 — Reasoner + federation + RSP/geo build‑out *(attribute F)*

**Why now.** sq-6tykl is a **large, maintainer-greenlit epic that is barely started** — the
biggest feature-surface expansion available. DL conformance L5 landed this session
(sq-pbz04.4.5) but the *shared zero-overhead eval substrate* (sq-qonbz) — the foundation the
full reasoner suite (RL/EL/QL/Direct/RIF/D), SERVICE federation, and RSP/GeoSPARQL all
compose on — is the ripe first step, because everything downstream depends on it and the
"no perf regression" constraint (#1303) is already the enforced gate.

**First deliverables**
1. **Shared eval substrate foundation** (sq-qonbz) — the joins + numeric/term ops shared by
   the SPARQL engine *and* all reasoners, behind the existing perf-neutrality gate. This is
   the architecturally-load-bearing first move; sequence the reasoner profiles and the
   SERVICE/RSP/geo arms behind it.
2. **Vendor / drop-in parity** (#1576 epic sq-xqchl) — a maintainer-voiced ask; the eye-js
   n3reasoner shim (sq-ohnj1) is the first lane but is **blocked on you ratifying the API
   name / package surface**. The N3.js / RDF-JS parity work (sq-iwhl8) can proceed in
   parallel without that decision.

**Risk / dependency.** The substrate is the highest-value but also the most cross-cutting —
it touches the hot eval path, so it goes through the escalated review lane with the perf
gate as the proof. The drop-in *eye-js* lane is gated on **sq-ohnj1** (your API ratification).

**Size.** Large, multi-wave. Best run as a Fable-architect decomposition of sq-qonbz first.

---

## 5. WS‑5 — Mature the collaboration tier toward autonomy *(attributes F, G)*

**Why now.** This session was, in effect, a **live proof that the collaboration tier works
at scale**: an architect-decomposes → cheap-fleet-implements → Fable-adjudicates → arm-on-
verdict loop ran ~70 verdict-gated merges, caught **seven real soundness/safety defects in
review that green gates and passing tests missed**, and honored **two correct executor
stop-conditions** (engine A3, PSS materializer) that avoided improvising architecture. The
autonomous scheduler (sq-sgu1) would remove the orchestrator from per-agent dispatch — and
improving the agent's *own operating loop* is itself item G.

**Honest caveat — this is why it's WS-5, not WS-1.** Much of today's value came from
*human-in-the-loop judgment at the dispatch layer*: curating disjoint waves, choosing what
to escalate, and — critically — catching a **false "mutation-verified" self-report** on a ZK
conformance PR that a naive auto-arm would have merged. Removing that judgment naively would
regress quality. So the proposal is **not** a full replacement.

**First deliverables**
1. **Wire the existing collaboration-tier workflows as an opt-in accelerant** (sq-sgu1.2:
   `fable-architect-drain` + `fable-soundness-verdict` + `fable-lens-review` — the skills
   already exist) and run them on a **bounded, low-stakes bead frontier** as a *measured
   experiment*: does the automated loop hold the same quality bar (defect-catch rate,
   arm-on-verdict discipline) as the hand-driven one? Keep the human arm on anything
   touching a soundness/authorization/ZK surface.
2. **Only if the experiment holds**, widen the frontier. The measurement is the gate, not a
   calendar.

**Risk / dependency.** The failure mode is a quality regression that a metric wouldn't
immediately show. Mitigate by keeping the escalated-surface human arm and by making the
experiment's success criterion the *defect-catch rate*, not throughput.

**Size.** Small to start (the workflows exist); the value is in the measured rollout.

---

## 6. Recurring taxes worth clearing (small, dispatchable now)

Not workstreams, but real friction observed this session:

- **sq-umj0p** — the noir_XPath / noir_IEEE754 rulesets require a `copilot-review-posted`
  status context that **no producer workflow emits**; the manual-status workaround was used
  **six times today**. Either add the producer workflow or drop the context. A recurring tax
  on every face re-sync. *(maintainer call on which fix.)*
- **sq-s5tkx** — a pre-existing 8-test failure in `sparq-solid`'s `--all-features`
  `odrl_bridge` suite, invisible to CI because no lane runs that feature combination. Fix +
  add the combination to the matrix.
- **sq-nmx4l** — the deferred PSS incremental *materializer* (write-path O(store-size)),
  honestly kicked to an architect design record rather than improvised.

---

## 7. Prioritization, sequence, and the decisions I need from you

**Recommended sequence** (each line = why it's in this slot):

1. **WS‑1 harness** — highest leverage, dispatchable now, and it de-risks your only hard
   decision (the live-run arm) by making everything up to it fixture-tested first.
2. **WS‑2 dossier** — dispatchable now, and it turns the biggest credibility unlock
   (sq-qhy4) from "cold codebase" into "reviewable package" for whenever you schedule it.
3. **WS‑3a GPU-stack measurement** — dispatchable now, measure-first, resolves a real signal
   the session surfaced; cheap to answer.
4. **WS‑4 substrate** — dispatchable now but large; start with a Fable-architect
   decomposition of sq-qonbz so the downstream reasoner/federation/RSP arms are disjoint.
5. **WS‑5 opt-in experiment** — dispatchable now as a *bounded measured trial*, not a
   rollout.

**(a) Dispatchable now, no decision needed:** WS-1 harness, WS-2 dossier + differential
harness, WS-3a, WS-4 substrate (+ the N3.js parity lane), WS-5 experiment, and the §6 taxes
(modulo the sq-umj0p fix-choice).

**(b) Gated on a specific decision from you** (each is a single call):
- **sq-qhy4** — commission the external ZK audit (unlocks all production ZK security claims). *The biggest single unlock.*
- **sq-tzars.9 live-run arm** + **sq-tzars.5** §6.1/6.2/6.4 rulings (option tables on #1111).
- **#1552** — provide `KB_DUMP_TOKEN` (dump automation).
- **sq-ohnj1** — ratify the eye-js shim API name/surface (unblocks the #1576 drop-in lane).
- **EC2 bench greenlight** — re-confirm for WS-3b competitor baselines.
- **#1574** — ratify (or redirect) the engine-split A3 withdrawal, which deviated from the
  #1402 ratification on structural + measured grounds.
- **#1546** — the sparq-solid default-graph flip (independent of the above).
- Dependabot majors (#1419/#1283/#1282/#1281/#1280/#1278) and **glib/sq-u57m8** (needs a
  Tauri major).

**(c) Should wait:** Option B of the engine split (parked with explicit reopening
conditions in `engine-split-rfc.md` §10 — revisit only when per-job caching lands or the GPU
stack leaves the cold path); the MPC epic (sq-pwr) and the deeper ZK build-out (sq-1s2.*)
past what WS-2 readies — they compound on the external audit landing first.

---

## 8. Honesty notes

- This is a **proposal**; the maintainer's steer overrides any sequencing here.
- The GPU-stack win (WS-3a) and the collaboration-tier autonomy value (WS-5) are
  **unmeasured** until their first deliverables run — both are framed as measure-first, and
  either could come back "not worth it," which is a valid and honest outcome.
- No performance numbers are asserted in this document; the one real measurement referenced
  (D2 / sq-aqr2f) lives in its bead, non-canonical work-box.
- WS-2's payoff is explicitly **not** a soundness claim — the external audit (sq-qhy4)
  remains the sole gate on any production ZK security statement; the dossier only *prepares*
  for it.
