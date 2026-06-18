# sparq-nlq

**Natural-language questions → SPARQL** over a sparq graph — an **opt-in** crate
(GenAI phase 3) built as a deliberately **lean loop** (retrieve + repair beats sprawling
agents). Nothing in the workspace depends on it; the default build does not compile it,
and the engine carries zero GenAI code.

The loop: **GROUND** (`sparq_introspect::to_text_summary(budget)` + few-shot examples +
the question) → **GENERATE** (`Llm::complete`) → **VALIDATE** (extract the ` ```sparql `
block, `spargebra` parse) → on parse error **REPAIR** (error + failed query back to the
LLM, ≤ N rounds) → on parse success **EXECUTE** (`sparq_engine::query_with_budget`) → on
exec error repair, else return `Answer { sparql, result, repairs, transcript }`.

## 🚀 Quickstart

```rust,no_run
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# use sparq_core::Graph;
use sparq_nlq::{Nlq, ReplayLlm};

# let graph = Graph::load_str("", "turtle")?;
// Offline / CI path: serve recorded prompt→completion pairs from a fixture.
let replay = ReplayLlm::from_file("tests/fixtures/olympics_replay.json")?;
let nlq = Nlq::new(&graph, Box::new(replay));
let answer = nlq.ask("How many athletes are on each team?")?;
println!("{}\n{} rows, {} repair(s)", answer.sparql, answer.result.len(), answer.repairs);
# Ok(()) }
```

```sh
# Live path (opt-in, network): the non-default `live` feature + an API key.
cargo add sparq-nlq --features live
export ANTHROPIC_API_KEY=sk-ant-...
```

```rust,no_run
# // `live::AnthropicLlm` needs the non-default `live` feature (network); gated so the
# // doctest is a no-op under the default build.
# #[cfg(feature = "live")]
# fn demo() -> Result<(), Box<dyn std::error::Error>> {
# use sparq_core::Graph;
use std::rc::Rc;
use sparq_nlq::{live::AnthropicLlm, Nlq, NlqConfig, RecordingLlm};

# let graph = Graph::load_str("", "turtle")?;
let llm = Rc::new(RecordingLlm::new(AnthropicLlm::from_env()?)); // record while you go
let nlq = Nlq::with_config(&graph, Box::new(Rc::clone(&llm)),
    NlqConfig { max_repair_rounds: 3, ..NlqConfig::default() });
let answer = nlq.ask("Which team has the most athletes?")?;
llm.save("my_session.json")?;                   // replayable fixture for later
# let _ = answer;
# Ok(())
# }
# fn main() {}
```

## ✨ Features

- **Grounded generation** — the introspect crate's token-budgeted schema summary (exact
  counts from the store's permutation indexes, not guesses) plus two schema-agnostic
  few-shot examples.
- **Index-grounded entity/relation linking** ([`link`](src/link.rs), opt-in via
  `NlqConfig::link_entities`, default **off**) — resolves the question's proper nouns to
  the concrete IRIs present in *this* store (label index over `rdfs:label` / `skos:prefLabel`
  / `schema:name` / `foaf:name` / `dc:title`) and its predicate local-names, then expands
  each linked entity with its structurally-similar siblings via
  [`sparq-sim`](../sparq-sim)'s `Sim::most_similar` — the index-driven candidate generator,
  no model and no network. Those linked IRIs are appended to the grounding prompt so the
  model writes value/entity-bound queries against real IRIs instead of guesses
  (`research/genai-nl-to-sparql.md` §2.6/§8.3). Off by default because it changes the prompt
  string — re-record fixtures before enabling it on a recorded dataset. <!-- [OPUS-4.8] sq-uw40 -->
- **Validate before execute** — parses with `spargebra` *before* the engine sees the
  query, so the repair round gets a real parser error; execution failures (unsupported
  forms, `QueryBudget` trips) are repair signals too.
- **N2 dictionary constraint** ([`constrain`](src/constrain.rs), bead `sq-9yjp`,
  **opt-in** via `NlqConfig::check_dictionary`) — once a query parses, walk its algebra
  and check every predicate / `rdf:type` class IRI against the **live dictionary**
  (`Graph::id_of`). An ungrounded IRI matches no triple, so it becomes a *targeted*
  repair signal ("predicate `<…>` not in the dictionary; did you mean `<…>`?") with
  nearest-namespace edit-distance suggestions pulled from the store itself. This is the
  design doc's API-backend fallback (`research/genai-nl-to-sparql.md` §6, §11): strict
  logit-level grammar-constrained *decoding* needs logit/grammar access the Anthropic
  Messages API does not expose, so it is **only feasible on a local backend** — not
  implemented here, and not claimed. Off by default because an empty-answer question
  (a valid-but-absent class) must not be "repaired"; the exec-accuracy harness measures
  raw model accuracy with it off. <!-- [OPUS-4.8] sq-9yjp -->
- **Budgeted execution** — LLM-generated queries are untrusted input, so every query runs
  under a [`QueryBudget`]. The default is **bounded** (10 s wall clock, 1M materialised
  rows); opting out requires setting `exec_timeout` / `max_rows` to `None` explicitly.
- **Full provenance** — the returned `Answer` carries the final query, the materialised
  result, the repair count, and the **full transcript** (every prompt/completion with its
  outcome). `Nlq::prompt_for` / `repair_prompt_for` are public and deterministic, so
  fixtures can be built and inspected outside the loop.
- **`Llm` trait — record/replay, offline CI** — `fn complete(&self, prompt: &str) ->
  Result<String, String>`. `ReplayLlm` serves recorded pairs (exact-prompt match — the CI
  path); `RecordingLlm<L>` wraps any backend and `save()`s the fixture; `AnthropicLlm` is
  a thin blocking client for the Messages API behind the non-default `live` feature
  (model id configurable, default `claude-sonnet-4-6`; key from `ANTHROPIC_API_KEY`).
- **Exec-accuracy harness** ([`eval`](src/eval.rs)) — grades the executed SPARQL the QALD
  way (**answer-set F1**, not query-string equality) by running candidate and a **gold**
  query on the *same* graph; the gold answer is recomputed live (no checked-in answer blob
  to drift). Reports two axes separately: **linking** (oracle vs end-to-end) and
  **grounding** (grounded vs ungrounded baseline). `Comparison::headline_grounding_pays()`
  is the load-bearing check: grounded end-to-end macro-F1 **>** the *same LLM* ungrounded.

## Honest status — what is and is not measured

- **Validated offline (the CI gate)**: the engine-side loop — grounding, extraction,
  `spargebra` validation, budgeted execution, repair plumbing, record/replay — end-to-end
  against real data; **and** the exec-accuracy harness itself (answer-F1 scoring,
  oracle-vs-end-to-end / grounded-vs-ungrounded reporting, the "grounding pays" inequality),
  proven deterministically on an in-memory graph with scripted backends and on the olympics
  dataset when present (committed grounded fixture). The record→`save`→`from_file`→
  identical-scores round-trip is itself CI-gated, so a committed live session is guaranteed
  replayable.
- **NOT yet measured against a real model**: live-model exec-accuracy *numbers*. The
  in-memory and olympics scores use scripted / recorded completions — they demonstrate the
  *harness* and the *mechanism* of the headline claim, **not** a real model's accuracy. The
  live path (`live_exec_accuracy`, `#[cfg(feature = "live")]` **and** `#[ignore]`'d) needs a
  real key, the olympics dump, and a **canonical** measurement host (not a CI/sandbox box) —
  open work in bead `sq-g0lw`.
- **Wired, opt-in, not yet measured against a real model**: index-grounded
  entity/relation linking from `sparq-sim` (`NlqConfig::link_entities`, bead `sq-uw40`).
  The label/IRI matching and the structural-sibling expansion are CI-gated on an
  in-memory graph; whether it lifts live exec-accuracy is part of the open live
  measurement (`sq-g0lw`). The complementary lexical/exact literal-value tier is
  bead `sq-na0q`.
- **Partial (honest scope)**: N2 against the live dictionary (bead `sq-9yjp`). The
  **vocabulary** half — predicate / class IRIs checked against the dictionary, with
  did-you-mean repair — is wired (opt-in `check_dictionary`, see Features). Strict
  logit-level **grammar-constrained decoding** is *not* implemented: it needs
  logit/grammar access the Anthropic Messages API does not expose, so it is only
  feasible on a local/open backend (`research/genai-nl-to-sparql.md` §11). Not claimed.
  <!-- [OPUS-4.8] sq-9yjp -->
- `AnthropicLlm` is compile-checked in CI (`--features live`) but never called there; no
  test touches the network. <!-- [OPUS-4.8] sq-05rv sq-g0lw -->

## Measured results

The eval harnesses run in CI with no network and gate parse/execution success, result
sanity, repair-round use, and LLM-excluded `ask()` latency. Re-record fixtures whenever the
prompt template, default `NlqConfig`, or dataset changes; run the live measurement
explicitly with a key + dataset. Tracked figures live on the perf dashboard.

```sh
cargo run  -p sparq-nlq --example record_olympics --release
cargo test -p sparq-nlq --features live --release -- --ignored live_exec_accuracy  # network + key
```

## 📚 Learn more

- Design records: [`research/genai-design.md`](../../research/genai-design.md) (§4),
  [`research/genai-nl-to-sparql.md`](../../research/genai-nl-to-sparql.md) (§4.3)
- Schema grounding: [`sparq-introspect`](../sparq-introspect)
- Perf dashboard: <https://jeswr.github.io/sparq/dev/bench>
- Open work for this crate: `bd list -l area:sparq-nlq`

## License

MIT. [OPUS-4.8] sq-lsxd
