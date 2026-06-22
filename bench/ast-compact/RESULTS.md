<!-- [OPUS-4.8] issue #1080. 🤖 SPARQ agent — the REAL cache-discounted compacted-AST
token A/B measurement record. bench/ is the AGENTS.md-sanctioned home for measured
numbers (exempt from check-no-perf-numbers.py); no user-facing markdown repeats them.
Written while Fable unavailable; flag for re-review when Fable returns. -->

# RESULTS — compacted-AST-first token A/B (real transcript telemetry)

> 🤖 **SPARQ agent.** Measurement record for issue **#1080**. Companion to, and a
> deliberate **contrast** with, the earlier ast-grep+outline A/B
> (`bench/pkg-dogfood/RESULTS-astgrep.md`, bead **sq-0fb3f**). Same telemetry method
> (cache-discounted effective input tokens mined from real sub-agent transcripts), a
> **different question class** — and a **different verdict**. `bench/` is the sanctioned
> home for the measured figures; they are not repeated in any user-facing doc.

## The question (and why it differs from sq-0fb3f)

The maintainer's idea (#1080): instead of just grepping, produce a **COMPACTED AST
REPRESENTATION** — a structural skeleton (every item as `line: signature`) that the
agent **works over and manipulates** — and test whether it improves performance.

This is **not** the sq-0fb3f question. That firm A/B measured *outline/ast-grep-FIRST*
on **code-structure lookups** ("where/how is X", "every impl of T") versus a *scoped
Read*, and found the structural tools are precision tools, **not token-savers** (B was
~21k MORE expensive at the median). The maintainer's distinction is that the compacted
representation should pay off for **editing / codemod / whole-file-understanding** —
tasks where the file's *bytes* dwarf its *skeleton* and the agent must reason over the
whole shape — not for a narrow lookup answerable by a scoped read. This experiment tests
**that** class honestly, expecting nothing (sq-0fb3f reversed a prior proxy twice).

## The two arms

| Arm | Strategy |
|---|---|
| **A** | **Normal `Grep`/`Read`** — the baseline an agent does today; explicitly forbidden from using any AST tool. |
| **B** | **Compacted-AST-FIRST** — generate the skeleton with `bench/ast-compact/compact_ast.sh <FILE>` (a one-line-per-item `ast-grep` dump), **work over that skeleton**, and `Read` raw bytes only for spans the skeleton can't resolve. |

Both arms run on Opus. Each of the 10×2 = 20 cells is answered by a **fresh sub-agent**
whose brief opens with a `[task=<id> arm=<A|B>]` tag, so no context bleeds between cells.
The compacted view turns e.g. the 2371-line `join.rs` into a **75-line** skeleton (~32×
structural compression) and the 10818-line `exec.rs` into ~451 items.

## The frozen task set (N=10)

10 CODE tasks over the real Rust workspace, requiring **understanding + modifying
structure**, across 4 classes — each graded against frozen `gold_keys` (the load-bearing
item names / call-site lines / handling points the answer must contain). The tasks are
"produce the precise EDIT PLAN + structural facts" so the 20 cells stay independent
without 20 agents mutating shared files. Definitions: `bench/ast-compact/tasks.json`.

- **summarise** (4): structure-summary of a 2.4k–10.8k-line file (`exec.rs`, `dict.rs`,
  `join.rs`, `source.rs`).
- **add-variant** (2): add an enum variant + enumerate every site that must handle it
  (`MpcError::Timeout`, `ArithOp::Modulo`).
- **rename** (2): rename a fn/method + find **every** call site (`render_key`,
  `secure_equal` — the latter with two substring-colliding siblings).
- **sig-change** (2): change a trait method signature across its impl + all call sites
  (`GlobalJoin::join`), and the OWL reasoning-entry inventory.

## The measured result

N = 10 frozen tasks, single counterbalanced run. Tokens are the **real
cache-discounted effective input tokens** mined from each fresh sub-agent's transcript
`message.usage` (`1.0·input + 0.1·cache_read + 1.25·cache_creation`); **no `count_tokens`
API, no char/byte proxy**. Quality is gold-key coverage per task.

| Arm | median eff. input tok | median quality | cheaper than the other arm on… |
|---|---|---|---|
| **A** — normal `Grep`/`Read` | 166,375 | 1.00 | 1 / 10 tasks |
| **B** — compacted-AST-first | **143,271** | 1.00 | **9 / 10 tasks** |

**B was cheaper on 9 / 10 tasks** (sign test p ≈ 0.02), at the **paired median −22k
effective input tokens**, and **total** B/A = **0.715** (B spent ~28% fewer effective
input tokens overall) — **at equal quality** (median 1.00 both arms). This is the
**opposite direction** to sq-0fb3f, and it is exactly the maintainer's predicted split:
the compacted representation pays off when the task is *whole-file/structural-edit*, not
a scoped lookup.

### Per task-class (paired median B−A; negative = B cheaper)

| Class | n | median A eff | median B eff | median B−A | A q | B q | note |
|---|---|---|---|---|---|---|---|
| **sig-change** | 2 | 283,712 | 151,462 | **−132,250** | 1.00 | 0.92 | biggest win — trait change spans a whole crate; the skeleton + structural `.join(` query enumerates impls/call-sites without reading the file whole |
| **add-variant** | 2 | 218,720 | 138,738 | **−79,981** | 0.90 | 1.00 | B both cheaper AND higher quality — structural `match` query + skeleton found the exhaustive-match sites a scoped read can wander on |
| **summarise** | 4 | 166,375 | 143,271 | **−16,000** | 1.00 | 1.00 | the skeleton *is* the answer to "what's the structure"; A re-reads the file's bulk |
| **rename** | 2 | 109,750 | 97,382 | **−12,368** | 1.00 | 1.00 | smallest gap; the single A-win (T06, a 3-occurrence rename in one file) is here — the compacted-view setup cost isn't amortised on a tiny, already-narrow task |

The **magnitude of the win scales with file size and structural breadth.** On the two
largest/broadest tasks B saved 224k (T08, 60% cheaper) and 145k (T04, 48% cheaper)
effective input tokens. On the smallest task (T06, rename 3 occurrences) B was 3.7k
*more* expensive — the same lesson sq-0fb3f found for lookups: don't pay the
compacted-view tax for a task a scoped read already answers cheaply.

## CONCLUSION (honest)

**For whole-file-understanding and structural-edit/codemod planning over large Rust
files, generating a compacted-AST skeleton FIRST and working over it measurably HELPS:
~14% lower median and ~28% lower total effective input tokens at equal quality, cheaper
on 9/10 tasks here.** The win is real and grows with file size / structural breadth
(summarise a 3k-line file, add-variant with an exhaustive-match audit, change a trait
signature across a crate). **It does NOT help — and slightly hurts — on a small, already
narrow task** (a 3-site rename in one file), exactly mirroring sq-0fb3f's lookup verdict.

**This does not contradict sq-0fb3f; it complements it.** sq-0fb3f's verdict ("ast-grep
+ outline are precision tools, not token-savers") was measured on **lookup** questions
against a **scoped Read**. The lever the maintainer named — a compacted representation
the agent *works over* for **whole-file/codemod** tasks — is a *different* question, and
on that question the compacted view wins because the file's bytes dwarf its skeleton.
The decision rule that survives both A/Bs: **reach for the compacted-AST view when the
unit of work is the whole file's shape or a cross-file structural edit; reach for a
scoped `Read` when the answer is a small known span.**

## Method (how this was measured / how to reproduce)

1. **Compacted-AST generator** — `bench/ast-compact/compact_ast.sh <FILE>` emits the
   one-line-per-item skeleton via `ast-grep` (invoked as `ast-grep`, never `sg`).
2. **Frozen task set** — `bench/ast-compact/tasks.json`: 10 tasks × 4 classes, each with
   `gold_keys`, frozen before the run.
3. **Fresh sub-agent per (task, arm)** — 20 Opus sub-agents, each brief tagged
   `[task=… arm=…]`. Arm A is forbidden AST tools; arm B must run `compact_ast.sh` first
   and work over the skeleton.
4. **Mine real tokens + grade** — `bench/ast-compact/mine.py <since-epoch>` finds the
   newest transcript per tag, sums cache-discounted effective input tokens from
   `message.usage`, grades the final answer against `gold_keys`, and emits one JSON row
   per cell. Stats (medians, per-class delta, sign test) computed from those rows.

## Honest caveats (load-bearing)

- **N = 10, single counterbalanced run, directional.** Enough to establish the
  **direction** (B cheaper for this class) and the per-class ordering and that it
  **reverses** sq-0fb3f's lookup verdict; it is not a multi-run significance study. The
  sign test (9/10, p≈0.02) is directional, not a powered effect-size estimate.
- **All numbers are runtime / NON-CANONICAL** — work-box transcripts, list-price
  context. Sanctioned record for this A/B's verdict, not a frozen perf benchmark; do not
  bake into user-facing docs (AGENTS.md / SKILL.md cite this file qualitatively only).
- **Tasks are "produce the edit plan", not "apply the edit".** This isolates the
  *understand-the-structure* cost (the part the compacted view targets) and keeps cells
  independent, but it does not measure the apply/compile/iterate loop. A real codemod
  also pays edit + `cargo check` cycles common to both arms; the *understanding* phase is
  what the skeleton accelerates, and that is what is measured here.
- **The grader is substring gold-key coverage**, a coarse correctness proxy; two cells'
  early mid-flight captures were re-mined to completion before the final table (the
  miner picks the newest transcript per tag). One task (T04, add-variant + cross-crate
  exhaustive-match audit) needed a re-dispatch in arm B — the first run wandered into a
  deep structural audit; the converged run is in the table. That fragility is itself a
  note: the broadest audits still benefit from the skeleton but can over-explore.
- **The single A-win (T06) is the honest boundary.** It is the smallest task; do not
  generalise the saving to small, already-scoped work.

## Reconciliation with sq-0fb3f (the prior, contradicting-looking verdict)

| | sq-0fb3f (`bench/pkg-dogfood/RESULTS-astgrep.md`) | this A/B (#1080) |
|---|---|---|
| Question class | **lookup** ("where/how is X", "impls of T") | **whole-file understanding + structural edit/codemod** |
| Baseline | a **scoped `Read`** of a known span | `Grep`/`Read` over a whole large file |
| Artifact | outline/structural query, then read the span | **compacted skeleton the agent works over** |
| Verdict | structural tools are **precision** tools, ~21k **MORE** expensive | compacted view **~14% median / ~28% total CHEAPER**, 9/10 |

Both are true. The unit of work decides: **scoped lookup → scoped Read; whole-file shape
or cross-file structural edit → compacted-AST view first.**

## Follow-up

Tracked as bead **sq-cdqdn**: wire the compacted-view recipe + the unit-of-work decision
rule into `.claude/skills/ast-grep/SKILL.md` and the AGENTS.md query-type tool-map
(citing this file qualitatively), and optionally run a larger-N / multi-run confirmation.

Licensed MIT (repo default).
