<!-- [OPUS-4.8] -->
# metrics — the per-PR/task collection harness + baseline (sq-lhwo.1)

This directory is the **instrument-first backbone** that gates all adoption across
both the agent-effectiveness program
([`research/agent-effectiveness-program.md`](../research/agent-effectiveness-program.md))
and the sparq-PKG dogfooding track
([`research/dogfooding-sparq-knowledge-graph.md`](../research/dogfooding-sparq-knowledge-graph.md)).
Nothing gets adopted without **(1)** a baseline captured here and **(2)** an A/B run
against it using the shared `§5` protocol + verdict object in the dogfooding record.

The harness only **instruments**. It bakes in **no** improvement claim and is
**falsifiable**: a row/baseline can show that a change made things *worse*.

## Files

| Path | What | Tracked? |
|---|---|---|
| [`../scripts/agent-telemetry/metrics_row.py`](../scripts/agent-telemetry/metrics_row.py) | join harness — emits one row per PR/task | yes (code) |
| [`../scripts/agent-telemetry/capture_baseline.py`](../scripts/agent-telemetry/capture_baseline.py) | runs the harness over the last N merged PRs | yes (code) |
| [`../scripts/agent-telemetry/agent_telemetry.py`](../scripts/agent-telemetry/agent_telemetry.py) | the **canonical** token-accounting engine (reused, not re-derived) | yes (code) |
| `baseline.jsonl` | the committed "before" distribution (regenerable data) | yes (data) |
| `runtime/` | live append-only rows — session-local, non-canonical | **git-ignored** |

`baseline.jsonl` is **data, not a perf claim**: it is a `.jsonl` file, so the
`check-no-perf-numbers.py` markdown/Typst scan never touches it (that scan globs
`*.md` / `*.typ` only). Numbers in it are work-box/session-local and **non-canonical**
— used for before/after deltas and ranking, never copied into committed markdown.

## The metrics schema (one row per `(sha, pr, bead, session, arm)`)

Each row joins four already-existing stores. Every column is graded in the row's
`_field_quality` map so a consumer can see what is solid vs approximate:

### Cleanly measured (deterministic from GitHub / git ref-events)

- `ci_first_green`, `first_ci_attempt`, `ci_checks_failing`, `ci_advisory_failing` —
  from `statusCheckRollup`. **Advisory / non-blocking lanes do not fail the gate**
  (repo gate policy); they are tallied in `ci_advisory_failing` separately.
- `force_push_count` — `head_ref_force_pushed` events in the PR timeline.
- `post_first_push_commits` — commits whose `committedDate` is **after** PR open
  (ref-event-aligned, **not** a commit-message grep).
- `no_rework` — `(force_push_count == 0) AND (post_first_push_commits == 0)`. This
  closes the `git commit --amend` + force-push gaming hole: an amend leaves no new
  commit, but the force-push event is counted on its own.
- `churn_added` / `churn_deleted` / `changed_files`, `review_changes_requested`.
- `first_shot` (composite) — `first_ci_attempt AND no_rework AND
  roborev_blocking_zero AND review_changes_requested_zero`, with every sub-flag kept
  in `first_shot_subflags` so you can see **which** gate trips (you cannot game the
  composite by suppressing one signal).

### Approximate / partial (stated honestly, never fabricated)

- **Per-PR token attribution is hard.** Claude Code transcripts are per-**session**,
  not per-PR; one session can touch many PRs and one PR many sessions, and there is
  no ground-truth per-PR token ledger. So `tokens_in/out`, `cache_read/write`,
  `effective_input_tokens`, and `cache_hit_ratio` are populated **only** when a
  session transcript is supplied (`--telemetry-json` / `--transcript`); otherwise
  they are `null` and `_field_quality.tokens == "unattributed"`. The A/B side-steps
  this by running **one task per session** (`§5.1`), where session == task is exact.
  `effective_input_tokens = 1.0*fresh + 0.1*cache_read + 1.25*cache_write` (the
  `§5.1` cache-discount), so a "win" that is purely a cache artifact is visible.
- `roborev_findings{high,med,low}` / `roborev_verdict` — parsed from the codex
  reviewer's free-text output (`text_parsed`; stable format, but textual). Keyed on
  commit SHA across the PR's reviewed commits; `roborev_reviews_found` reports
  coverage. `first_shot` treats **High+Med** as the "blocking" proxy (the design's
  "forced a change" is not deterministically derivable from one review, so this is
  the conservative honest floor; `Low` is advisory).
- `usd_est` — `null` unless explicit `--price-*` flags are passed (prices drift; a
  baked price in a measurement tool is a lie).

### Quality pairs (schema slots the A/B / CI backfill fills)

Every efficiency metric has a quality pair so "fewer tokens, worse output" is
catchable. These are emitted as `null` placeholders here (graded `placeholder`) — the
harness does **not** fabricate them; the A/B run or a CI backfill populates them:
`coverage_delta`, `mutation_score_delta`, `conformance_floor_moved`,
`seeded_canary_find_rate`, `post_merge_revert`.

## How to add a row

```sh
# one merged PR -> append a row to the git-ignored runtime log
python3 scripts/agent-telemetry/metrics_row.py --pr <N> --bead <bead> --arm control

# inspect without appending
python3 scripts/agent-telemetry/metrics_row.py --pr <N> --dry-run

# attribute tokens for an A/B task (1 task == 1 session): supply the transcript
python3 scripts/agent-telemetry/metrics_row.py --pr <N> --arm treatment \
    --transcript ~/.claude/projects/<project>/<session>.jsonl

# opt-in $ estimate (you supply prices; never hard-coded)
python3 scripts/agent-telemetry/metrics_row.py --pr <N> \
    --price-input <N> --price-output <N>
```

Token attribution **reuses** `agent_telemetry.py`'s rollup (the canonical accounting
engine named by both designs) — it is never re-counted here.

## (Re)capturing the baseline

```sh
python3 scripts/agent-telemetry/capture_baseline.py --count 30 --out metrics/baseline.jsonl
# or pin explicit PRs for reproducibility:
python3 scripts/agent-telemetry/capture_baseline.py --prs 1057 1056 1055 --out metrics/baseline.jsonl
```

The baseline deliberately leaves token columns **unattributed** (per-PR tokens are
not derivable post-hoc); it captures the GitHub/git/roborev gate signals — first-shot,
rework, CI-first-pass, pushback — which are the "before" the A/B compares against. The
A/B fills token columns per task.

## How the A/B is run (the shared dogfooding `§5` protocol — do not reinvent)

The statistics, thresholds, kill-criteria, and verdict object are the **single shared
spec** in [`research/dogfooding-sparq-knowledge-graph.md`](../research/dogfooding-sparq-knowledge-graph.md)
`§5.1–§5.6`. This harness only emits the rows that spec consumes. In outline:

1. **Freeze a stratified task set** (`§5.5`): ≥4 query types (point-lookup, multi-hop,
   synthesis-across-docs, negative/out-of-KG), plus — for the effectiveness surface —
   stratify by surface (rust-feature / site / ci-infra / docs) and by size.
2. **Run two arms per task, counterbalanced within one session window** (`§5.1`):
   arm A (control: read-the-docs) vs arm B (treatment: the tool/PKG under test), with
   model behaviour pinned (record/replay) so drift cancels.
3. **Emit a row per task per arm** with `--arm control|treatment` and the task's
   `--transcript`, so tokens are exactly attributed (1 task == 1 session).
4. **Charge arm B all its costs** (`§5.1`): per-query round-trips, schema-grounding,
   every repair retry, the deferred-tool-definition tokens actually pulled in, plus
   an amortised slice of one-time ingestion `(ingest + embed) / N`.
5. **Judge on the verdict object** (`§5.6`):
   `{token_win, token_delta_median_pct, token_delta_ci, quality_delta{…},
   break_even_N, break_even_infinite, honest, recommend_adopt}`. `recommend_adopt`
   requires `token_win` **AND** quality non-regression (the paired quality columns)
   **AND** a finite `break_even_N`. Decide on the **object**, never one number.
6. **Kill-criteria** (`§5.4`, mechanical): a paired-median effective-input reduction
   below the pre-registered bar, a non-significant result, a saving that is entirely
   the cache-discount component, an infinite/over-horizon break-even, or any quality
   regression (more hallucination, lower accuracy, more pushback) → do not adopt.

The harness is **cheap and model-free** (deterministic scripts over already-emitted
JSON; no model-in-the-loop per row). It **ranks and gates; it never auto-adopts** —
adoption stays a maintainer judgment on the verdict object.

## Tests

Hermetic stdlib `unittest` (no `gh` / `roborev` / `bd` / network) over a checked-in
synthetic bundle (`../scripts/agent-telemetry/tests/fixture_metrics_bundle.json`):

```sh
python3 scripts/agent-telemetry/tests/test_metrics_row.py
python3 scripts/agent-telemetry/tests/test_agent_telemetry.py   # the engine it reuses
```
