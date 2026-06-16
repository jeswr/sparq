---
name: academic-paper
description: "Use when turning a sparq contribution into an academic paper — identifying a genuinely-novel contribution (the re-runnable intake), classifying it and picking a venue (ISWC/ESWC Resource for the engine, PVLDB/SIGMOD/EDBT for DB-systems, arXiv/workshop for not-yet-sound ZK/MPC), drafting a single-source Typst (.typ) paper whose eval section binds to live benchmark data via --input/sys.inputs, building both a PDF and an in-site HTML page from that one source, and reviewing claims↔evidence under sparq's empirical-honesty mandate (canonical vs indicative numbers; the ZK/MPC not-yet-sound disclaimer). The paper-factory for epic sq-gum8."
---

# academic-paper — the sparq paper-factory

This is the repeatable PROCESS for producing one paper from a sparq contribution. It is
**factory machinery**, not prose advice: it enforces sparq's empirical-honesty mandate at
the *data* layer, binds every number to live benchmark data, and emits a PDF + an in-site
HTML page from one Typst source. Design record: `research/paper-factory-design.md`
(consolidates the contribution inventory + the auto-gen/PDF research). Epic **sq-gum8**.

**Two non-negotiable constraints govern every step.** They are the point of the factory; do
not soften them.

1. **No ZK/MPC security property may be claimed as proven.** The single-prover ZK verifier
   has no external audit (bead **sq-qhy4**, open); the multi-prover path is re-open
   (**sq-9hrn**); the malicious-secure MPC layer is a stub. A ZK/MPC paper today is an
   **honest design / limitations / negative-result** contribution, arXiv/WIP only, and
   **must cite sq-qhy4**. Never write "secure" / "verifiable" / "private" as a *proven*
   property — only as a *design goal*.
2. **Work-box (EC2 `-aws`) wall-clock numbers are NON-CANONICAL.** Only deterministic
   integer metrics are canonical today (W3C/OGC conformance floors, byte-identity, recall
   floors, gate/round/byte counts, differential-fuzz pass). A speed/memory claim needs the
   **canonical runner** (`research/ci-ec2-design.md`). Indicative numbers are **never
   co-tabulated with canonical ones** and never feed an aggregate figure-of-merit.

The factory has six stages. Stages 0/1/3/5 are the methodology this skill encodes; 2/4/6 are
scripted (see `research/paper-factory-design.md` §1–§2 for the script/route layout).

---

## Stage 0 — Intake: identify a genuinely-novel contribution (re-runnable)

Run before claiming anything is publishable. This is the re-runnable identification process;
its current ranked output is `research/paper-contributions-inventory.md` — read it first and
diff against it rather than starting from scratch.

**Scan six sources:** crates (`crates/sparq-*/src`, READMEs — what is IMPLEMENTED vs
designed-only; `NotYetImplemented`/`#[ignore]` markers); `research/*.md` (claimed technique +
prior-art + the author's own honesty hedges + negative results); beads
(`.beads/issues.jsonl` — open audit/gating beads, esp. sq-qhy4/sq-9hrn/sq-1gir);
benchmark deltas (`research/BENCHMARKS.md`, `bench/` — which numbers exist, canonical vs
work-box); the conformance scoreboard (`crates/sparq-conformance/src/scoreboard.rs` — the
only fully-canonical evidence); skills (`skills/*/SKILL.md` — distilled durable findings).

**Score four criteria — a candidate must pass all four:**

1. **Novelty vs prior art** — a new algorithm/construction/finding, or a faithful
   re-implementation of cited prior art? Name the prior systems (ACORN, NaviX, QLever,
   RDFox, CostFed, SPDZ, coZK, …). Integration *can* be a contribution — but framed as a
   *systems/integration* paper, never dressed as a new algorithm. A measured negative result
   is also a contribution.
2. **Evidence available** — canonical evidence *today*, or only work-box numbers / a design?
3. **Generality** — does the claim generalise beyond sparq's exact code, or is it a local
   tuning artefact?
4. **Honesty / soundness** — for crypto: proven, or merely implemented + internally
   reviewed? For perf: canonical or not? **This is the veto against overclaiming.**

**Assign a readiness verdict:** PUBLISHABLE-NOW (sound claim + canonical evidence today, or a
design/methodology contribution needing no benchmark) | NEEDS-CANONICAL-BENCHMARKS (honest +
real, but the headline rests on work-box numbers — publishable once re-gathered on the
canonical runner) | NOT-YET-SOUND / NEEDS-EXTERNAL-AUDIT (ZK/MPC — honest design / negative
result only; cite sq-qhy4). **Rank by readiness × impact** (a PUBLISHABLE-NOW candidate with
a real audience outranks a high-impact unaudited crypto claim).

**Re-run triggers:** a new crate/feature lands; a benchmark improves or a canonical run
completes (re-evaluate every NEEDS-CANONICAL-BENCHMARKS candidate); a conformance floor rises;
an audit bead changes state (a sq-qhy4 *pass* moves the ZK candidate out of NOT-YET-SOUND —
until then it cannot move); a new `*-measured.md`/negative-result doc appears. The re-run is
cheap: re-scan, re-score, re-verdict, re-rank, diff the inventory, open/close paper beads.

---

## Stage 1 — Classify & venue-target

Map the contribution to its family, then to a venue + track + Typst template. **Verify every
page limit / deadline / blinding policy against the current-year CFP before submitting — they
drift yearly.** Maintainer precedent (jeswr, Oxford / ODI / W3C) centres on **ISWC/ESWC**;
the engine-as-artifact maps exactly to their **Resource track**.

| Family | Best venues | Track / notes |
| --- | --- | --- |
| **A — DB/engine performance** (indexes, parallel/streaming/mmap exec, ingest, compression) | **PVLDB** (rolling monthly, single-blind, EA&B track for measurement-heavy papers); **SIGMOD** (fixed rounds, double-blind); **ICDE**; **EDBT** (most accessible first DB paper) | systems/empirical; reproducibility/artifact tracks near-universal |
| **B — Semantic Web / RDF / SPARQL** (federation, SHACL, GeoSPARQL, RSP-QL, inference, GenAI-over-RDF, RDFC-1.0) | **ISWC** (home venue; **Resource track purpose-built for a reusable engine**); **ESWC** (European sibling; note its strict no-preprint window conflicts with arXiv-first); TheWebConf; SEMANTiCS; JWS (journal) | Research / **Resource** / In-Use + posters/demos/workshops |
| **C — Security/privacy (ZK/MPC), NOT-yet-sound** | **arXiv preprint + workshop NOW** (HotPETs / ZKProof / WPES / ISWC-ESWC privacy workshops); **PoPETs** = best real target *once sound* (journal, 4 rolling deadlines/yr); USENIX/CCS/S&P aspirational | preprint/WIP today; graduate to a real venue only when sq-qhy4 (and sq-9hrn) close |

**Template:** `charged-ieee` (IEEE 2-col) | `arkheion` (arXiv-style) | `para-lipics` (LIPIcs;
accepted papers need a Typst→LaTeX conversion for the publisher) | an acmart-style template.
**Double-blind venues** (SIGMOD/VLDB/ICDE/WWW/CCS/S&P) require the anonymized build (Stage 4);
ISWC/ESWC blinding varies by track. **Rolling cadence** (PVLDB monthly, PoPETs quarterly,
SIGMOD multi-round) → target submit-when-ready, not one annual crunch.

---

## Stage 2 — Canonical benchmark capture (the honesty boundary)

Run the eval on the pinned **canonical runner** (bare-metal / frequency-pinned —
`research/ci-ec2-design.md`). Emit result records into the `benchmark-data` branch (which
`site/scripts/sync-benchmarks.mjs` folds into `site/src/data/benchmarks.generated.json`),
each carrying:

```json
{ "key": "filtered_ann.recall_at_10", "value": 0.98, "unit": "recall",
  "n_reps": 30, "dispersion": { "ci95": [0.97, 0.99] },
  "environment": "canonical",
  "cpu": "...", "kernel": "...", "rustc": "...", "dataset_sha256": "...",
  "seed": 42, "commit": "<sha>" }
```

**The CI gate:** any record with `environment: "indicative"` is **BLOCKED** from a
paper-bound table — the build fails if a paper's `#bench.<key>` resolves to an indicative
record. Indicative and canonical numbers are never in the same table and never feed an
aggregate. Until the canonical runner exists (blocked on one IAM step), publish **only the
deterministic metrics** (conformance counts, recall floors, byte-identity, gate/round/byte
counts) — those are canonical today.

---

## Stage 3 — Draft: single-source Typst, bound to live data

**Writing methodology (the order matters):**

- **Contributions list FIRST** — it drives the whole paper ("the paper substantiates the
  claims"). Each bullet is **refutable** (names *what* is achieved, specific enough to be
  disproved) and **forward-references the section that delivers the evidence**. PJ's
  contrast: NO = "We describe the WizWoz system. It is really cool."; YES = "We give the
  syntax and semantics … (§3); we prove the type system sound … (§4); … half the length of
  the Java version (§5)."
- **Abstract = 4 sentences, written last:** (1) the problem; (2) why it is interesting;
  (3) what your solution achieves; (4) what follows.
- **Introduction ≤ 1 page**, in order: what is the problem / why important / why hard (why
  naive approaches fail) / why unsolved before (what distinguishes you) / key components +
  results → ending in the bulleted contributions list. The new contribution must be clear by
  **end of page 3**. Delete "the rest of this paper is organized as follows" — the
  contributions list does that job.
- **Real examples, never `foo`/`bar`; active voice; self-contained figure captions that tell
  the reader what to notice** (reviewers skim).
- **Related work late & charitable** — "credit is not like money": be generous; compare and
  contrast, don't list; label any inferior approach explicitly and up front; write it "as if
  telling the cited authors why they should care." Defer to near the end unless a concise
  early defensive stance is needed.
- **Conclusion** with concrete numbers; **no wishlist "future work"** ("no partial credit for
  neat things you wanted to do but didn't").

**The live-data recipe (the load-bearing factory mechanism).** Author one `.typ`; inject the
live JSON at build via `--input` / `sys.inputs`. At the top of the paper:

```typ
#let bench = json(bytes(sys.inputs.data))
#let anon  = sys.inputs.at("anon", default: "false") == "true"
```

Then the eval section references `#bench.<key>` — **never a hard-coded number**:

```typ
We observe recall@10 of #bench.filtered_ann.recall_at_10
over #bench.filtered_ann.n_queries queries (#bench.filtered_ann.n_reps reps,
95% CI #bench.filtered_ann.dispersion.ci95).
```

Run the self-check before handing off to review:

- **SIGPLAN-7 rubric** (red/yellow/green, supports judgment not a binary gate): (1) claims
  clearly stated + scoped, no implied generality; (2) suitable, fairly-configured baseline;
  (3) principled benchmark choice (established suites, justify subsets, no cherry-picking,
  don't test on the training set); (4) adequate analysis (enough trials; geometric mean for
  ratios, harmonic for rates, median under outliers; report variability/CIs); (5) relevant
  metrics (measure *all* important effects — index/compile time alongside runtime); (6) clear
  reproducible design (full platform spec, key parameters explored); (7) appropriate
  presentation (zero-based axes, right precision, distribution-reflecting summary).
- **Benchmarking-crimes self-check** (Heiser): full HW spec (CPU+µarch, cores, clock/turbo
  policy, cache levels, RAM, kernel release, storage); full SW spec (`rustc` + `Cargo.lock` +
  flags, baseline-system versions); dataset name+version+SHA-256+seed; report absolutes not
  ratios-only; never summarize 4/6/7/49% as "up to 49%"; never use a competitor's sub-optimal
  config (Heiser: that "probably constitutes scientific misconduct").

---

## Stage 4 — Build: one source → PDF + in-site HTML

```bash
# PDF (static asset served by the Next.js static export):
typst compile site/papers/<slug>/paper.typ site/public/papers/<slug>.pdf \
  --input data="$(cat site/src/data/benchmarks.generated.json)"

# anonymized build for a double-blind venue (strips jeswr/sparq identity):
typst compile site/papers/<slug>/paper.typ /tmp/<slug>-anon.pdf \
  --input data="$(cat site/src/data/benchmarks.generated.json)" --input anon=true
```

The **in-site HTML** page at `/papers/<slug>` renders the *same* `.typ` via **typst.ts**
(`@myriaddreamin/typst.react`), fed the same JSON — so HTML and PDF cannot show different
numbers. (Typst's own HTML export is still experimental; use typst.ts today.) The build step
(`site/scripts/build-papers.mjs`) is wired into `prebuild`, so `next build` regenerates every
paper's PDF against fresh data; a `benchmark-data` change re-triggers it (numbers
auto-update). For a venue that rejects Typst→LaTeX, the camera-ready fallback is Pandoc/LaTeX
via Tectonic against `acmart`/`IEEEtran`/`lipics`. **Do not** author the PDF with
`@react-pdf/renderer` — it duplicates the numbers (breaks single-source).

---

## Stage 5 — Review gate: claims ↔ evidence

For every claim in the intro/contributions list, identify its evidence (analysis / theorem /
measurement / case study) and confirm the forward-reference resolves. The gate (run as
subagent section-reviewers + one cross-cutting honesty/repro reviewer) **blocks** on:

- **Any indicative number in a claim** (the Stage-2 CI gate enforces this mechanically; the
  reviewer also checks no indicative/canonical co-tabulation).
- **A C-family (ZK/MPC) draft using "secure"/"verifiable"/"private" as a proven property** —
  must be a design goal, with the sq-qhy4 soundness-gap disclaimer present.
- **Overclaiming / implied generality, weak/unfair baselines, missing ablations, no error
  bars, irreproducibility** (the SIGPLAN-7 + benchmarking-crimes findings from Stage 3).

Resolve **all** findings before publish. Final check: the claims↔evidence loop is closed and
every headline number is canonical.

---

## Stage 6 — Publish & auto-update

Merge → `next build` serves `/papers/<slug>` + `/papers/<slug>.pdf` (static export, under the
`/sparq` basePath). A `benchmarks.generated.json` refresh re-runs the build → both artifacts
regenerate with current numbers. Each paper carries a **provenance stamp** (commit + canonical
runner fingerprint + dataset SHA-256). A venue camera-ready triggers the optional Typst→LaTeX
export. To **register a new paper**, add an entry to `site/src/data/papers.ts` and a
`site/papers/<slug>/paper.typ` — the index, the per-paper route, the nav, and the PDF build
are all data-driven off `papers.ts`.

---

## Empirical-honesty rules (the rubric, condensed)

- **Canonical vs indicative** — headline tables/figures use only `environment: canonical`
  records, reported with variability. Indicative (EC2 `-aws`) numbers live only in clearly
  labelled "indicative development measurement" callouts that name the instance type + `-aws`
  kernel, state they are not the basis of any claim, and are never co-tabulated with canonical
  numbers. Never quote a speedup blending the two.
- **ZK/MPC not-yet-sound disclaimer** — every C-family paper cites **sq-qhy4**, is marked
  arXiv/WIP-only, and frames any security/privacy/integrity/attestation property as a *design
  goal*, never proven. It graduates to a real venue only when sq-qhy4 (and, for the
  multi-prover path, sq-9hrn) close.
- **Honesty norm** — be scrupulously honest; don't over-sell, hide drawbacks, or disparage
  others' work; submit only if proud to attach your name as-is; anticipate scepticism and
  state how the approach could fail.

## Pilot

The factory's first pilot is **A1 + A2** together (both PUBLISHABLE-NOW, no canonical runner
or external audit needed): **A1** = RDF-native filtered-ANN ("Filter-as-Query" — exact
transitive-BGP filter over the engine's own dict-ids + answer-safety; canonical recall
evidence today; frame as integration, not an ANN-algorithm novelty); **A2** = the honest
same-box benchmark methodology (a methods/reproducibility contribution that strengthens A1's
eval). See `research/paper-contributions-inventory.md` (Part 2) and
`research/paper-factory-design.md` (§6).

## Notes

- This skill was **authored in-repo** (not installed). The strongest external pipeline
  (Imbad0202/academic-research-skills) is CC-BY-NC — incompatible with this permissively
  licensed repo; the MIT K-Dense scientific-writer is life-sciences-leaning and lacks
  sparq's venue map + non-canonical-benchmark handling + the live-data Typst pipeline. Those
  two sparq specifics ARE the contribution of the factory.
- This is a **site/process** surface, not a public crate API, so it is not in the
  `skills/SKILL.md` public-surface router (that router lists code-integration surfaces).
  Keep this skill in sync with `research/paper-factory-design.md` if the pipeline changes.
- Verify before relying on version-specific features: typst.ts's pinned upstream compiler vs
  Typst 0.15; the current-year CFP for any target venue.
