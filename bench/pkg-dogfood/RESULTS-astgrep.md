<!-- [OPUS-4.8] sq-0fb3f. 🤖 SPARQ agent — the REAL cache-discounted ast-grep+outline
token A/B measurement record. This is the SANCTIONED home for these measured numbers
(bench/ is exempt from check-no-perf-numbers.py); no user-facing markdown repeats them.
AGENTS.md cites this record qualitatively. Written while Fable unavailable; flag for
re-review when Fable returns. -->

# RESULTS — ast-grep + outline-first token A/B (real transcript telemetry)

> 🤖 **SPARQ agent.** Measurement record for bead **sq-0fb3f**. Companion to
> [`RESULTS.md`](RESULTS.md) (the PKG-query 3-arm A/B) — same telemetry method, a
> different question. It **corrects** the earlier sq-lhwo.2 byte-proxy pilot (the
> directional "outline is the big lever" signal in the `ast-grep` SKILL §5), which
> **overstated** the saving — see the CRITICAL LESSON below. `bench/` is the
> AGENTS.md-sanctioned home for measured figures; the numbers below live HERE and are
> not repeated in any user-facing doc.

## The question

For an agent answering a **code-structure** question over this Rust workspace ("where /
how is X done", "every impl of trait T", "the signature of fn Y", "all call sites of
Z"), is it cheaper — at equal answer quality — to go **outline / `ast-grep`-FIRST**
(install the structural tool, outline the file or run a structural query, then `Read`
only the located span) than to just do a **scoped full `Read`** (locate-and-read with
`Grep` + `Read`)? The decision metric is **cache-discounted effective input tokens**
mined from the real sub-agent transcripts, with a paired answer-quality grade.

## The two arms

| Arm | Strategy |
|---|---|
| **A** | **Normal full `Read`** — `Grep`/`Read` to locate, then `Read` the relevant span (the baseline an agent does today). |
| **B** | **Outline / `ast-grep`-FIRST** — install + verify `ast-grep`, run the structural query / file outline (signatures only), then `Read` only the located span. |

Both arms run on **Opus**. The same N=16 frozen code-structure questions are answered by
fresh sub-agents per (task, arm), so no context bleeds between cells.

## The measured result

N = 16 frozen code-structure questions, single counterbalanced run. Tokens are the
**real cache-discounted effective input tokens** mined from each fresh sub-agent's
transcript `message.usage` (`1.0·input + 0.1·cache_read + 1.25·cache_creation`); **no
`count_tokens` API, no char/byte proxy**. Quality is gold-key coverage per task.

| Arm | median eff. input tok | quality | cheaper than the other arm on… |
|---|---|---|---|
| **A** — normal full `Read` | **67,254** | 0.98 | 11 / 16 tasks |
| **B** — outline / `ast-grep`-first | 75,558 | **1.00** | 5 / 16 tasks |

**B cost ≈ 21k MORE effective input tokens at the median** and was cheaper on only
**5 / 16** tasks. The outline/`ast-grep`-first install + structural queries +
verification reads cost **more** than a scoped `Read` — B is **NOT a token-saver
end-to-end**. Its only edge was a small **quality** nudge (A 0.98 → B 1.00).

### Per question-kind (B − A, effective input tokens; negative = B cheaper)

| Question kind | B vs A (eff. input tok) | note |
|---|---|---|
| **call-sites** ("every call site of fn Z") | **B +34.5k** (more expensive) | B's ONE quality edge: call-site **completeness** A 0.92 → B 1.00 |
| **structure** ("the shape / where-is-X of a file") | **B +21k** (more expensive) | |
| **signature** ("the signature of fn / type Y") | **B +11k** (more expensive) | |
| **trait-impls** ("every impl of trait T") | **B +3.4k** (more expensive) | smallest gap, still B-more-expensive |

B was **more expensive across the board** — every kind. The only place B *bought*
anything was call-site **completeness** (A 0.92 → B 1.00): the structural query
enumerated call sites a `Grep`/`Read` missed. That is the **completeness** value of the
structural tools, paid for with **more** tokens — not a token saving.

## CONCLUSION

**`ast-grep` + outline are PRECISION / completeness tools, measured to NOT save tokens
(slightly MORE expensive end-to-end on this question class).** Use them when
completeness is the point — enumerate **all** impls / **all** call sites where a
`Read`/`grep` might silently miss one, or express a shape a line-regex cannot — **not**
to cut tokens. **Narrow surviving exception:** outlining **only** the skeleton
(signatures) of a *very large single file* to locate a span still beats reading the
whole file. The broad "outline-first saves tokens" claim did not survive the firm A/B.

## Method (how this was measured / how to reproduce)

1. **Frozen task set** — 16 code-structure questions across 4 kinds (trait-impls /
   call-sites / signature / structure), each with `gold_keys`, frozen before the run.
2. **Fresh sub-agent per (task, arm)** — each of the 16×2 = 32 cells is answered by a
   brand-new Opus sub-agent whose brief opens with a `[task=<id> arm=<A|B>]` tag. Arm A
   sub-agents use only `Grep`/`Read`; arm B sub-agents install + verify `ast-grep`,
   outline / structural-query first, then `Read` the located span.
3. **Mine real tokens** — the transcript miner reads every `agent-*.jsonl` transcript,
   attributes it by its `[task=… arm=…]` tag, and sums the cache-discounted effective
   input + output tokens from `message.usage`. One JSON row per cell.
4. **Paired verdict** — grade each answer against its gold keys; report median effective
   input tokens, per-kind B−A delta, the per-task win count, and the paired quality
   pair per arm.

## Honest caveats (load-bearing)

- **N = 16, single counterbalanced run, directional.** Enough to establish the
  **direction** (B is more expensive, not a saver) and the per-kind ordering; it is not
  a multi-run significance study. Treat the deltas as directional magnitudes, not as a
  measured constant.
- **All numbers are runtime / NON-CANONICAL** — work-box transcripts, list-price
  context. They are the sanctioned record for this A/B's verdict, not a frozen perf
  benchmark; do not bake them into user-facing docs (AGENTS.md cites this file
  qualitatively, per the no-perf-numbers house rule).
- **B's value is completeness, not cost.** The one real win (call-site completeness
  A 0.92 → B 1.00) is a **quality/precision** signal, not a token signal. If
  completeness is critical (you must NOT miss a call site / impl), B is worth the extra
  tokens; if it is not, the scoped `Read` is both cheaper and quality-adequate here.
- **The narrow file-skeleton exception is not contradicted.** Outlining *only* the
  skeleton of one very large file to avoid reading it whole is a genuine per-file
  reduction; the firm A/B's "B more expensive" verdict is about the **end-to-end**
  outline/`ast-grep`-FIRST *strategy* (install + multiple structural queries +
  verification reads across the whole question class), not that single per-file lever.

## CRITICAL LESSON — the 2nd proxy reversal

This is the **second time** a byte/char proxy with a charitable scope assumption
**inverted** a verdict that real-transcript measurement then corrected:

1. **#1078 char-proxy** (PKG-query) looked competitive for "Opus reads the docs"; the
   real-transcript A/B in [`RESULTS.md`](RESULTS.md) showed read-docs was an order of
   magnitude more expensive.
2. **sq-lhwo.2 byte-proxy** (this A/B's predecessor) — a model-free file-read **byte**
   comparison over a few "where/how is X" tasks — produced the directional "outline is
   the big lever" signal recorded in the `ast-grep` SKILL §5. The byte proxy
   **overstated** it: counting outline-skeleton bytes vs whole-file bytes makes the
   outline look like a large saving, but it **omits** the install cost, the multiple
   structural queries, the re-reads, and the verification reads the real strategy pays
   — and it has no quality pair. The firm real-token A/B here shows the end-to-end
   outline/`ast-grep`-first strategy is **slightly more expensive**, not a saver.

**Takeaway (reinforced):** a byte/char proxy with a charitable read-scope assumption is
a direction-*finder*, not a substitute for mining the tokens the model actually
consumed. When the decision is load-bearing (adopt-as-standing-practice or not), the
proxy's verdict **must not** be trusted over the measurement. The proxy reversed twice;
measure the real transcripts before promoting a "this saves tokens" rule.

Licensed MIT (repo default).
