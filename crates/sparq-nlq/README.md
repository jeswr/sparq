<!-- [OPUS-4.8] sq-inzv: README brought to template. -->
# sparq-nlq

<p>
  <a href="https://crates.io/crates/sparq-nlq"><img src="https://img.shields.io/crates/v/sparq-nlq.svg" alt="crates.io"></a>
  <a href="https://docs.rs/sparq-nlq"><img src="https://docs.rs/sparq-nlq/badge.svg" alt="docs.rs"></a>
  <a href="../../LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
</p>

**Natural-language questions → SPARQL** over a sparq graph — an **opt-in** crate (GenAI
phase 3) built as a deliberately **lean loop** (retrieve + repair beats sprawling
agents). Nothing in the workspace depends on it; the default build does not compile it,
and the engine carries zero GenAI code.

The loop: **GROUND** (`sparq_introspect` schema summary + few-shot examples + question)
→ **GENERATE** (`Llm::complete`) → **VALIDATE** (extract the ` ```sparql ` block,
`spargebra` parse) → on error **REPAIR** (≤ N rounds) → **EXECUTE**
(`sparq_engine::query_with_budget`) → return `Answer { sparql, result, repairs,
transcript }`.

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

The live path is opt-in (network): `cargo add sparq-nlq --features live`, set
`ANTHROPIC_API_KEY`, and wrap `live::AnthropicLlm::from_env()?` in a `RecordingLlm` to
record a replayable fixture as you go.

## ✨ Features

- **Grounded generation** — the introspect crate's token-budgeted schema summary (exact
  counts from the store's permutation indexes, not guesses) plus two schema-agnostic
  few-shot examples.
- **Index-grounded entity/relation linking** ([`link`](src/link.rs), opt-in via
  `NlqConfig::link_entities`, default **off**) — resolves proper nouns to the IRIs
  present in *this* store (label index over `rdfs:label`/`skos:prefLabel`/… plus
  predicate local-names) and expands each with structurally-similar siblings via
  [`sparq-sim`](../sparq-sim) — no model, no network. Off by default because it changes
  the prompt string (re-record fixtures first). <!-- [OPUS-4.8] sq-uw40 -->
- **Validate before execute** — parses with `spargebra` *before* the engine sees the
  query; execution failures (unsupported forms, budget trips) are repair signals too.
- **Dictionary-grounded repair** ([`constrain`](src/constrain.rs), opt-in via
  `NlqConfig::check_dictionary`) — once a query parses, every predicate / `rdf:type`
  class IRI is checked against the **live dictionary** (`Graph::id_of`); an ungrounded
  IRI becomes a *targeted* did-you-mean repair signal. This is the design doc's
  API-backend fallback: strict logit-level grammar-constrained **decoding** needs
  logit/grammar access the Anthropic Messages API does not expose, so it is only
  feasible on a local backend — **not implemented here, and not claimed**
  (`research/genai-nl-to-sparql.md` §11). <!-- [OPUS-4.8] sq-9yjp -->
- **Budgeted execution** — LLM output is untrusted, so every query runs under a
  [`QueryBudget`] (default **bounded**: 10 s, 1M rows; opting out is explicit).
- **Full provenance** — `Answer` carries the final query, result, repair count, and the
  **full transcript**; `prompt_for` / `repair_prompt_for` are public + deterministic.
- **`Llm` trait — record/replay, offline CI** — `ReplayLlm` serves recorded pairs (the
  CI path); `RecordingLlm<L>` wraps any backend and `save()`s a fixture; `AnthropicLlm`
  is a thin blocking Messages-API client behind the non-default `live` feature.
- **Exec-accuracy harness** ([`eval`](src/eval.rs)) — grades executed SPARQL the QALD
  way (**answer-set F1**, not query-string equality) against a live-recomputed gold
  query, reporting linking (oracle vs end-to-end) and grounding (grounded vs ungrounded)
  separately.

## Honest status — what is and is not measured

- **Validated offline (the CI gate)**: the engine-side loop and the exec-accuracy
  harness itself, end-to-end on real data with scripted/recorded backends. The
  record→`save`→`from_file`→identical-scores round-trip is CI-gated, so a committed live
  session is guaranteed replayable.
- **NOT yet measured against a real model**: live-model exec-accuracy *numbers*, and
  whether linking lifts them. The offline scores demonstrate the *harness* and the
  *mechanism*, **not** a real model's accuracy; the live path (`#[cfg(feature = "live")]`
  **and** `#[ignore]`'d) needs a key, the olympics dump, and a **canonical** measurement
  host — open work in bead `sq-g0lw`. `AnthropicLlm` is compile-checked in CI but never
  called; no test touches the network. <!-- [OPUS-4.8] sq-05rv sq-g0lw -->
- **Partial scope**: grammar-constrained decoding stays unimplemented and unclaimed
  (see the dictionary-grounded-repair feature above, bead `sq-9yjp`).

## 📚 Learn more

- **How-to** — [`skills/genai-retrieval/SKILL.md`](../../skills/genai-retrieval/SKILL.md).
- **API reference** — [docs.rs/sparq-nlq](https://docs.rs/sparq-nlq).
- **Design** — [`research/genai-nl-to-sparql.md`](../../research/genai-nl-to-sparql.md)
  (§4.3) and [`research/genai-design.md`](../../research/genai-design.md) (§4).
- **Schema grounding** — [`sparq-introspect`](../sparq-introspect).
- **Performance** — the eval harnesses run network-free in CI; tracked figures live on
  the [benchmarks dashboard](https://jeswr.github.io/sparq/dev/bench), not in docs.
- **Contribute** — [`AGENTS.md`](../../AGENTS.md).

## License

[MIT](../../LICENSE). <!-- [OPUS-4.8] sq-lsxd -->
