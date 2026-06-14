# sparq-nlq

**Natural-language questions → SPARQL** over a sparq graph — an opt-in crate (GenAI
phase 3, see [`research/genai-design.md`](../../research/genai-design.md)) built as a
deliberately **lean loop** (the SPARQL-LLM research finding the design adopts:
retrieve + repair beats sprawling agents). Nothing in the workspace depends on this
crate; the default build does not compile it, and the engine carries zero GenAI code.

## The loop

```text
         ┌──────────────────────────────────────────────────────┐
         │ GROUND   sparq_introspect::to_text_summary(budget)   │
         │          + few-shot examples + the question          │
         └───────────────────────────┬──────────────────────────┘
                                     ▼
         ┌──────────────────────────────────────────────────────┐
   ┌────▶│ GENERATE Llm::complete(prompt) -> completion         │
   │     └───────────────────────────┬──────────────────────────┘
   │                                 ▼
   │     ┌──────────────────────────────────────────────────────┐
   │     │ VALIDATE extract ```sparql block, spargebra parse    │
   │     └───────────┬──────────────────────────┬───────────────┘
   │           parse error                  parses
   │                 │                          ▼
   │     ┌───────────┴───────────┐  ┌──────────────────────────┐
   └─────┤ REPAIR (≤ N rounds:   │  │ EXECUTE sparq_engine::   │
   ▲     │ error + failed query  │  │ query_with_budget        │
   │     │ back to the LLM)      │  └────────────┬─────────────┘
   │     └───────────────────────┘     exec error│        ok
   └──────────────────────────────────◀──────────┘         ▼
                                           Answer { sparql, result,
                                                    repairs, transcript }
```

- **Grounding** is the introspect crate's token-budgeted schema summary — exact
  counts mined from the store's permutation indexes, not guesses — plus two
  schema-agnostic few-shot examples.
- **Validation** parses with `spargebra` *before* the engine sees the query, so the
  repair round gets a real parser error message. Execution failures (unsupported
  forms, `QueryBudget` trips) are repair signals too.
- **Execution** always runs under a [`QueryBudget`] — LLM-generated queries are
  untrusted input. The default is **bounded** (10 s wall clock, 1M materialised
  rows); opting out requires setting `exec_timeout` / `max_rows` to `None`
  explicitly.
- The returned `Answer` carries the final query, the materialised result, the repair
  count and the **full transcript** (every prompt/completion with its outcome) — the
  transcript is the provenance of the answer.

## The `Llm` trait — record/replay, offline CI

```rust
pub trait Llm {
    fn complete(&self, prompt: &str) -> Result<String, String>;
}
```

| Impl | Role |
|---|---|
| `ReplayLlm` | Serves recorded prompt→completion pairs from a JSON fixture — **the CI path**. Exact-prompt match (prompts are deterministic); misses produce a diagnosable error. |
| `RecordingLlm<L>` | Wraps any backend, records every exchange, `save()`s the fixture. |
| `AnthropicLlm` | Thin blocking client for the Anthropic Messages API (`POST /v1/messages`), behind the **non-default `live` feature**. Model id is configurable, default `claude-sonnet-4-6`; key from `ANTHROPIC_API_KEY`. |

## Usage — replay (offline, what the tests do)

```rust
use sparq_nlq::{Nlq, ReplayLlm};

let replay = ReplayLlm::from_file("tests/fixtures/olympics_replay.json")?;
let nlq = Nlq::new(&graph, Box::new(replay));
let answer = nlq.ask("How many athletes are on each team?")?;
println!("{}\n{} rows, {} repair(s)", answer.sparql, answer.result.len(), answer.repairs);
```

## Usage — live (opt-in, network)

```sh
cargo add sparq-nlq --features live   # or: features = ["live"] in Cargo.toml
export ANTHROPIC_API_KEY=sk-ant-...
```

```rust
use std::rc::Rc;
use sparq_nlq::{live::AnthropicLlm, Nlq, NlqConfig, RecordingLlm};

let llm = Rc::new(RecordingLlm::new(AnthropicLlm::from_env()?)); // record while you go
let nlq = Nlq::with_config(&graph, Box::new(Rc::clone(&llm)), NlqConfig {
    max_repair_rounds: 3,                       // design-doc cap for live runs
    ..NlqConfig::default()
});
let answer = nlq.ask("Which team has the most athletes?")?;
llm.save("my_session.json")?;                   // replayable fixture for later
```

`Nlq::prompt_for` / `repair_prompt_for` are public and deterministic, so fixtures can
be constructed and inspected outside the loop.

## Eval harness — measured results

`tests/olympics_eval.rs` is the design doc's accuracy-gate scaffold: it drives the
full loop against the real olympics dataset (1.78M triples,
`bench/qlever-olympics/olympics.nt`, skip-if-absent) with `ReplayLlm` serving the
committed fixture — 9 hand-written NL→SPARQL pairs over the olympics schema,
realistic completions (only vocabulary visible in the grounding prompt), including
**one deliberate parse failure that exercises the repair round**.

What the harness measures and gates (run it, or see the perf dashboard at
<https://jeswr.github.io/sparq/dev/bench>, for the absolute figures):

| Metric | What it checks |
|---|---|
| Parse success (after ≤1 repair) | every fixture pair parses |
| Execution success | every fixture pair executes |
| Result sanity | non-empty + exact row counts + spot-checked values |
| Repair rounds used | the one scripted malformed query exercises a repair |
| LLM-excluded `ask()` latency | gated low (the recorded p50/max print from the example below) |

Re-record the fixture whenever the prompt template, default `NlqConfig`, or dataset
changes:

```sh
cargo run -p sparq-nlq --example record_olympics --release
```

## Honest status

- **What is validated**: the engine-side loop — grounding, extraction, spargebra
  validation, budgeted execution, repair plumbing, record/replay — end-to-end against
  real data, with the design doc's LLM-excluded latency gate enforced in release.
- **What is NOT yet validated**: live-model exec-accuracy. The recorded completions
  are hand-written (as a competent LLM would answer given the grounding prompt); the
  design doc's full accuracy gate — QALD-10-style harness, exec-accuracy ≥ the same
  LLM without grounding ("the grounding must pay for itself") — is a documented
  follow-up that runs through this same harness with `--features live` +
  `RecordingLlm`, after which the recorded fixtures become the regression set.
- **Not yet wired**: entity/relation linking from `sparq-sim` (design §2 phase 3
  lists it as input to grounding; the schema summary alone is enough for the
  olympics-scale schema) — bead `sq-uw40`; and N2 grammar-constrained decoding
  against the live dictionary — bead `sq-9yjp`. <!-- [OPUS-4.8] -->
- `AnthropicLlm` is compile-checked in CI (`--features live`) but never called there;
  no test touches the network.
