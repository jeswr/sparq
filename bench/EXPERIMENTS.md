# Agent-effectiveness & dogfooding experiments — index

🚀 A single front door to the measured experiments run while building the PKG
(Project Knowledge Graph) and tuning how the agent works on this repo. Each row
links to its detailed record; the numbers live in the linked `bench/.../RESULTS*.md`
(this `bench/` tree is the sanctioned home for measured figures).

✨ **Two cross-cutting findings the experiments keep reproducing:**

1. **Real measurement beats proxies — proxies inverted the verdict twice.** A
   char/byte *proxy* said "don't adopt PKG-query" and "outline-before-read is the
   big lever"; the **real cache-discounted transcript measurement reversed both**.
   Re-measure work-levers with real tokens before codifying them.
2. **For an LLM agent, fluency beats formal superiority.** The agent-facing win
   came from what the model wields fluently (a Haiku NL-tool; `schema.org` terms),
   not from the formally-richest option (`ast-grep`/outline; academic `gUFO`).

## The experiments

| Experiment | Question | N | Verdict | Record |
|---|---|---|---|---|
| **PKG token A/B (3-arm)** | Does querying the PKG cost fewer tokens than reading docs? | 30 | Opus `pkg-query` ~halves effective tokens vs reading docs (29/30, equal/better quality); a **Haiku NL-tool is ~30× cheaper than reading docs at equal quality**. Reversed an earlier char-proxy that said "don't adopt." | [`pkg-dogfood/RESULTS.md`](pkg-dogfood/RESULTS.md) |
| **NL-tool boundary** | When to delegate to the NL-tool vs read docs; does it hallucinate off-PKG? | 20 | In-PKG win replicates (~56×). On 10/10 genuinely off-PKG questions the tool **abstained (`NOT_IN_PKG`), zero hallucinations** → safe to adopt with a read-docs fallback. | [`pkg-dogfood/RESULTS.md`](pkg-dogfood/RESULTS.md) |
| **ast-grep + outline A/B** | Does outline/`ast-grep`-first save tokens on code-structure questions? | 16 | **No** — outline/ast-grep-first cost *more* effective tokens than a normal scoped `Read`; it's a **precision/completeness tool, not a token-saver**. Reversed the byte-proxy that called outline "the big lever." | [`pkg-dogfood/RESULTS-astgrep.md`](pkg-dogfood/RESULTS-astgrep.md) |
| **FO-KM Metric 1** | Does any foundational ontology beat gUFO / no-FO for the agent's KM tasks? | 16 | **schema.org-as-top wins (0.84)** ≫ DOLCE-DUL (0.64) > no-FO (0.58) > **gUFO (0.54, *below* no-FO)**. Driver = **LLM fluency**, not formal richness. | `bench/fo-km/RESULTS.md` (PRs #1107/#1108, landing) |
| **bd → sparq bridge eval** | Can sparq replace the bead tracker? | — | **Bridge, don't replace** (0/4 replacement criteria met). The real value is the research↔beads JOIN (links designs to their covering beads; surfaces dormant designs). | `crates/sparq-kb` README + PR #1076 |
| **gUFO closure-prior (KGE)** | Does gUFO closure firm up the link-prediction MRR lift? | multi-seed | **Not robust** — the lift is sign-unstable across synthetic slices (within per-seed spread). A real schema-bearing KG run is needed. | PR #1094 / `crates/sparq-vectors` eval |
| **sparq-terse query-authoring A/B** | Does the terse dialect (`K:` keywords / `V()`) save query-authoring tokens vs plain SPARQL, at equal correctness? | 30 | **Split, per lever.** Lever 1 (`K:` keyword) clears the token bar **and ties** plain SPARQL on quality → **conditional adopt** (proxy fidelity; pending the transcript fan-out). Lever 3 (`V()`) **does-not-adopt on quality** — it loud-fails (correctly) on a punctuation-heavy prefLabel, so `resolution_correctness < 1.0`. A `V()` is a clean-label convenience, not a drop-in for an explicit IRI. | [`terse/RESULTS.md`](terse/RESULTS.md) |

## Designs (context for the above)

- [`research/dogfooding-sparq-knowledge-graph.md`](../research/dogfooding-sparq-knowledge-graph.md) — the PKG dogfooding plan + reconciled outcomes.
- [`research/agent-effectiveness-program.md`](../research/agent-effectiveness-program.md) — the measurement program + shared protocol.
- `research/foundational-ontology-km-benchmark.md` — the FO-vs-gUFO benchmark design (PR #1106).
- `research/llm-ergonomic-sparql-surface.md` — the terse/`V()` query-surface design (PR #1074).

## Honest caveats (read before citing)

- These are **small-N, single-run** measurements (N = 16–30) with deterministic but
  **heuristic grading**. They are *directional*; large + consistent gaps (e.g. the
  ~30× NL-tool win, schema.org 0.84 vs gUFO 0.54) are robust to grading noise, small
  ones are not.
- The FO-KM result is **Metric 1 only (the agent's NL-tool)**. **Metric 2 — the KGE
  closure-prior MRR (machine reasoning, not the agent)** — is **compute-deferred**
  (needs a canonical/quiet box; tracked as a bead). A formal FO could rank differently
  there, because that metric does not depend on the agent's fluency.
- All token figures are **cache-discounted effective input tokens**
  (`1.0·fresh + 0.1·cache_read + 1.25·cache_write`) mined from real agent transcript
  `message.usage` — not a tokenizer proxy and not the `count_tokens` API.

## Reproducing

The harnesses live beside their records: [`pkg-dogfood/`](pkg-dogfood/) (token A/B +
real-transcript miner + analyzer), [`terse/`](terse/) (the `sparq-terse` per-lever
adoption gate — frozen tasks + reference queries + real-transpiler grader + verdict
object; `run.sh`), and `bench/fo-km/` (FO overlays + discriminating tasks + `analyze.py`;
PRs #1107/#1108, landing). Each `RESULTS*.md` documents its method and the exact arms.

## License

MIT — see the repository [`LICENSE`](../LICENSE).
