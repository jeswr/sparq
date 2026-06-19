# Paper-Factory Honesty-Gate Coverage — the `.typ` blind spot

<!-- [OPUS-4.8] sq-gum8 follow-up: design-for-review record extending the CI honesty
     gates to cover the paper-factory's authored surface (.typ sources + evidence JSON). -->

> 🤖 **SPARQ agent** design record. **DOC-ONLY** — no script/site/CI code is written here;
> the build comes after maintainer review. The phased plan in §6 enumerates each future bead.

## 0. Brief correction (read first)

The dispatching brief framed `sq-gum8` (the academic paper factory) as an **"un-started
epic"**. That premise is **wrong**, and saying so is the honest thing to do. The factory is
**built and as-built-documented**:

- Research + inventory + design records all exist: `research/paper-factory-research.md`
  (phase-1 prior-art survey), `research/paper-contributions-inventory.md` (phase-2 ranked
  inventory + re-runnable intake), `research/paper-factory-design.md` (phase-3 design,
  carrying an as-built correction banner).
- The pipeline is implemented: `site/scripts/build-papers.mjs` (the build step + the
  build-time honesty gate), `site/papers/_lib/bench.typ` (the data-load + `headline()`
  compile-panic accessor), two pilot papers `site/papers/filtered-ann.typ` +
  `site/papers/honest-benchmarking.typ`, the canonical evidence file
  `site/src/data/paper-evidence.json`, the registry `site/src/data/papers.ts`, the route
  `site/src/app/papers/{layout,page}.tsx` + `[slug]/page.tsx`, and
  `site/src/components/papers/{paper-html,paper-provenance}.tsx`.
- The methodology is encoded as a skill: `skills/academic-paper/SKILL.md`.
- CI builds it: `.github/workflows/pages.yml` installs a SHA-256-pinned Typst and runs
  `build-papers.mjs` via the site `prebuild`.
- Follow-ups are already beaded: `sq-3df4` (skill as-built — closed), `sq-gdhy`
  (auto-update trigger — closed), `sq-me8x` (A1 latency / Tier-B — open, blocked on the
  canonical runner / one IAM step), `sq-tvdl` (typst.ts upgrade path — open, P4 deferred).

So a fresh "design the paper-factory pipeline" pass would be **pure redundancy**. This record
therefore narrows to the **one genuine, un-beaded gap** found by reading the actual gate
scripts against the actual paper surface: **the CI honesty gates do not scan the paper
`.typ` sources or the evidence-JSON prose**, so the factory's headline empirical-honesty
controls have a coverage hole on exactly the surface that becomes a published paper.

## 1. Problem — where the honesty controls stop

The factory advertises that it "reuses the existing privacy-claims / no-perf-numbers gate
philosophy" and "enforces empirical honesty at the data layer, not only in prose". That is
true of the *intent* and true of the parts that are wired. But three mechanical facts,
verified against the live scripts, leave the authored paper text **outside** every
deterministic CI honesty gate:

1. **`scripts/check-privacy-claims.sh` scans only `*.md *.mdx *.tsx *.ts`.** Its file list is
   built with `git ls-files '*.md' '*.mdx' '*.tsx' '*.ts' …`. Paper sources are `*.typ` and
   the evidence file is `*.json` — **neither extension is in that list**. A ZK/MPC paper
   authored as a `.typ` could write an unqualified "the verifier is sound" / "privacy-
   preserving" headline and the privacy-claims gate would not see it. (This gate runs in
   `.github/workflows/docs-quality.yml`.)
2. **`scripts/check-no-perf-numbers.py` scans only `*.md`** (`SCAN_GLOBS = ["*.md"]`). A
   wall-clock / speedup / throughput literal typed directly into a `.typ` prose sentence or
   table — the exact thing the project's non-canonical-work-box rule exists to stop — trips
   nothing.
3. **The factory's own build-time gate guards only part of the `.typ` surface.**
   `build-papers.mjs::runHonestyGate()` validates the **evidence-JSON record schema** (each
   record has a valid `environment` ∈ {canonical, indicative}, a `source`, and a `value`) —
   it does *not* read paper prose at all. The `headline(key)` accessor in `bench.typ`
   panics the Typst compile if a *referenced* record is non-canonical — but it only fires
   for numbers pulled through `headline()`. A number or a soundness adjective **typed
   literally into the `.typ`** (not pulled through an accessor) is invisible to it. The two
   pilot `.typ` files already contain free-typed numeric literals in prose/tables
   (deterministic dataset sizes / parameters — legitimate today), which demonstrates that
   the `.typ` prose channel for raw numbers is wide open; nothing structurally distinguishes
   a legitimate "20 000 × 32 dataset" from an illegitimate hard-coded speedup.

**Net:** the published-paper text — the most outward, highest-stakes honesty surface the
project produces — is the one surface with **no deterministic CI honesty gate**. The
controls that exist are real but they protect the markdown/site-copy surface and the
*structured* evidence accessor path, not the free-form `.typ` prose that a reviewer or
venue actually reads. This is a coverage gap, not a wrong design — the design record's
honesty section is sound; the gates simply were never extended to the new file types the
factory introduced.

### What is NOT the gap (so we don't over-engineer)

- The `headline()` canonical/indicative split is sound and load-bearing — keep it; this work
  *complements* it, it does not replace it.
- The evidence-JSON schema gate is sound for what it checks. The gap is that it stops at the
  record envelope and never looks at the human `note`/`source` prose or the `.typ`.
- This is **not** about catching subtle semantic overclaims (no gate can). It is about
  closing the same *coarse, phrase/number* class the existing two gates already catch on
  `.md`, on the file types the factory added.

## 2. Goal + non-goals

**Goal.** Bring the paper-factory authored surface (`site/papers/**/*.typ`, the shared
`bench.typ`, and the prose fields of `site/src/data/paper-evidence.json`) under the *same*
coarse, deterministic honesty gates the rest of the repo already enforces — so a hard-coded
non-canonical number, or an unqualified ZK/MPC soundness/privacy claim, fails CI **before**
the paper is served, with the same inline-allow-marker escape hatch the existing gates use.

**Non-goals.** No NLP / semantic-claim classifier. No change to the `headline()` /
canonical-vs-indicative model (it stays). No new prose authored into the papers. No attempt
to gate the *generated* HTML/PDF outputs (they are git-ignored build artifacts derived from
the `.typ`; gating the source is both sufficient and the right layer).

## 3. Options

### Option A — Extend the two existing gate scripts to include `.typ` (+ JSON prose)

Add `*.typ` to `check-privacy-claims.sh`'s `git ls-files` glob list, and add `*.typ` to
`check-no-perf-numbers.py`'s `SCAN_GLOBS`. Optionally extend both to read the `note` (and
any free-text) fields of `paper-evidence.json` as additional scanned text.

- **Pros.** Maximal reuse — one phrase list, one allow-marker convention, one CI lane, one
  mental model. The `privacy-claims-allow:` / no-perf allow-line markers already exist and
  work in any text file (they are line comments, and Typst supports `//` line comments, so
  an inline `// privacy-claims-allow: <why>` reads naturally in a `.typ`). Smallest diff.
- **Cons.** The two scripts currently assume markdown-ish structure in a couple of spots
  (e.g. no-perf-numbers recognises fenced ` ``` ` code blocks and blockquoted fences to
  skip examples; the privacy gate's path-exclusion list is markdown-oriented). `.typ` has
  *different* comment/string syntax, so a naive add risks false positives (a `.typ` string
  literal that mentions a number) or false negatives (a `#raw` block). Needs a small
  per-extension tokenisation tweak, not just a glob addition. Must be **negative-tested**
  (a deliberately-overclaiming `.typ` fixture must fail; the honest pilots must stay green).

### Option B — A dedicated `.typ` honesty gate in `build-papers.mjs` (compile-time)

Fold the phrase/number scan **into the existing build step**, run alongside
`runHonestyGate()`, before compiling each `.typ`. It already reads every `.typ` and the
evidence JSON, already fails the build loudly, already runs in CI (`pages.yml`).

- **Pros.** Co-located with the factory; reuses the existing fail path and the existing CI
  lane (no new workflow). Sees the `.typ` *and* the JSON in one place. Naturally Typst-aware
  (can strip `//` comments / `headline(...)`-accessor spans before scanning, so an accessor-
  driven number is correctly *not* flagged while a literal one is).
- **Cons.** Now the *forbidden-phrase list lives in two places* (the shell gate + the mjs
  gate) unless factored to a shared data file — a drift risk and an honesty risk in itself
  (the two could diverge). It only runs where the site builds (the `pages.yml` deploy +
  local `prebuild`), not in the lightweight `docs-quality` lane, so a `.typ`-only PR that
  doesn't trigger a site build could merge un-scanned unless wired into the PR gate too.

### Option C — Hybrid: shared phrase list + thin per-surface scanners (recommended)

Factor the **forbidden-phrase patterns and the allow-marker convention into one shared
source of truth** (a small data file the two existing scripts already implicitly want), then
have **both** entry points consume it: (1) extend `check-privacy-claims.sh` +
`check-no-perf-numbers.py` to add `.typ` to their globs (Option A) with a minimal
Typst-comment-aware skip, and (2) keep a thin assertion inside `build-papers.mjs` that the
`.typ` it is about to compile has been gated (or re-runs the same shared scan) so the
factory's own build can never serve an un-scanned paper.

- **Pros.** One phrase list (no drift), gated in the cheap deterministic `docs-quality` lane
  *and* defended at the build boundary, Typst-aware where it matters. Single allow-marker
  convention across `.md` and `.typ`. This is the honest, no-blind-spot end state.
- **Cons.** Slightly more work than A alone (the refactor to a shared list). The refactor
  touches the two battle-tested gate scripts, so it must be landed carefully with the full
  negative-test matrix and a clean run on the current pilots.

## 4. Recommendation

**Adopt Option C, but stage it so the cheapest honest win lands first.**

1. **First, Option A as a contained step** — add `*.typ` to both gate scripts' scan globs
   with a minimal Typst-comment-aware skip, plus negative-test fixtures (one overclaiming
   `.typ`, one hard-coded-number `.typ`) that MUST fail and the two real pilots that MUST
   stay green. This closes the blind spot immediately with the smallest change to trusted
   code and lands in the existing `docs-quality` lane.
2. **Then the Option-C refactor** — factor the shared phrase list so the build-step
   assertion and the shell gate cannot drift, and add the `build-papers.mjs` build-boundary
   assertion so the factory itself refuses to compile an un-scanned `.typ`.

Rationale: Option A alone removes the honesty hole today with minimal risk to two
well-tested scripts; the Option-C refactor then removes the *drift* risk that two
independent phrase lists would create. Doing A inside C's plan (rather than as a throwaway)
avoids re-touching the gate scripts twice. Option B alone is rejected because a build-only
gate can be skipped by a `.typ`-only PR that does not trigger a site build, and because it
silently forks the phrase list.

**Honesty note on scope.** This work makes the *coarse* class of defect (forbidden phrase /
hard-coded number) mechanically caught on the paper surface. It does **not** and cannot
mechanically catch a subtle semantic overclaim — that remains the Stage-5 claims↔evidence
human/subagent review in `skills/academic-paper/SKILL.md`. The plan must say so plainly in
the doc updates (step 5) so the gate is not over-sold as a soundness guarantee — which would
itself violate the empirical-honesty mandate.

## 5. Interaction with the existing controls (must preserve)

- The `headline()` compile-panic and the canonical-vs-indicative `environment` split stay
  exactly as-is — this work is **additive**. A number pulled through `headline()` /`ev()`
  must NOT be flagged by the new `.typ` perf-number scan (it is already gated by the
  accessor); the scanner must therefore skip accessor-call spans, scanning only free-typed
  literals. This is the one piece of Typst-awareness the scanner genuinely needs.
- The `privacy-claims-allow: <why>` inline marker convention is preserved verbatim for
  `.typ` (Typst `//` line comments carry it). A legitimately-hedged ZK/MPC mention in a
  C-family paper uses the same marker the rest of the repo uses — no new convention.
- The ZK/MPC not-yet-sound posture is unchanged: `sq-qhy4` (external audit) remains the
  gate; the new `.typ` coverage simply makes the *mechanical* half of that posture apply to
  paper prose, where before only the human review did.

## 6. Phased plan (each item → a future bead)

Ordered. Items touching `site/` are serialised behind the one-site-branch discipline; the
gate-script steps do not touch `site/` and can run in parallel with site work.

1. **Add `.typ` to the two existing honesty gates (Option A).** Extend
   `scripts/check-privacy-claims.sh` (add `*.typ` to its `git ls-files` glob) and
   `scripts/check-no-perf-numbers.py` (add `*.typ` to `SCAN_GLOBS`), with a minimal
   Typst-comment / accessor-call skip so `headline()`/`ev()`-driven numbers are not
   false-flagged. Does **not** touch `site/`. *(no-site)*
2. **Negative-test fixtures + CI assertion.** Add a deliberately-overclaiming `.typ` and a
   hard-coded-non-canonical-number `.typ` fixture under a test path; assert both FAIL the
   respective gate and that the two real pilots + the shared `bench.typ` stay green. Wire
   the fixtures so a future regression (someone narrowing the glob back) is caught. *(no-site)*
3. **Scan the evidence-JSON prose fields.** Extend the gates (or `runHonestyGate()`) to also
   scan the human `note` / any free-text field of `site/src/data/paper-evidence.json` for the
   same forbidden phrases, so an overclaim hidden in a record `note` is caught. *(touches the
   evidence file's gate path; no React/site-route change)*
4. **Option-C refactor: one shared phrase list + build-boundary assertion.** Factor the
   forbidden-phrase patterns into a single shared source consumed by both the shell gate and
   `build-papers.mjs`; add a `build-papers.mjs` assertion that every `.typ` it compiles has
   been honesty-scanned (or re-run the shared scan there) so the factory cannot serve an
   un-scanned paper. Eliminates the drift risk. *(touches `site/scripts/`)*
5. **Doc + skill sync (honest scoping).** Update `skills/academic-paper/SKILL.md` (Stage 5)
   and `research/paper-factory-design.md` (§5) to state that the coarse honesty class is now
   *mechanically* gated on the `.typ`/JSON surface, while subtle semantic overclaims remain a
   human/subagent review responsibility — so the new gate is not over-sold. *(doc-only)*

**Surfaced follow-up (for the orchestrator to bead if it wants it tracked separately).** Once
the canonical runner lands (`sq-me8x`, blocked on one IAM step) and indicative-but-labelled
callouts begin appearing in papers, re-check that the `.typ` perf-number scan correctly
distinguishes an explicitly-labelled indicative callout (allowed, marker-carrying) from a
bare literal (forbidden) — i.e. the allow-marker path works end-to-end for a real indicative
callout, not only in fixtures.

## 7. Open question for the maintainer (genuinely needs a decision)

**Should the `.typ` perf-number scan treat free-typed deterministic *experiment parameters*
(dataset sizes, dimensions, k, mask selectivity — the legitimate literals already in the
pilot `.typ` tables) as allowed by default, or require each to carry an inline allow-marker?**
The privacy-claims gate fails-closed (a hit needs a marker); the no-perf-numbers gate is more
permissive (it allow-lists `bench/`+`research/` wholesale and skips code fences). For `.typ`
papers, *results* numbers must come through `headline()`/`ev()` (already gated), but *setup*
numbers (dataset size, k) are legitimately free-typed prose. Two honest options: (a)
**fail-closed** — every free literal in a `.typ` needs either an accessor or an allow-marker
(strongest, but noisier to author); (b) **scan only for result-shaped numerics** (units like
ms / µs / ×speedup / GB-s / triples-per-second / %faster) and leave bare counts alone
(lower friction, narrower net). Recommendation leans (b) for author ergonomics, but it is a
real call about how aggressive the net should be, and it is the maintainer's to make.

---

> 🤖 SPARQ agent — `sq-gum8` honesty-gate-coverage follow-up. Non-sycophantic by mandate.
> Corrects the "un-started epic" premise; designs the one genuine un-beaded gap (the `.typ`
> honesty blind spot). DOC-ONLY; build follows review per §6.
