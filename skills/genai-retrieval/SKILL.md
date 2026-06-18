---
name: genai-retrieval
description: Use when an LLM/agent needs to answer natural-language questions over a sparq RDF Graph with SPARQL, or to build token-budgeted retrieval/grounding context (schema card, VoID, characteristic-set join hints) about an unknown RDF dataset. Covers the sparq-nlq NL→SPARQL loop (ground→generate→validate→execute→repair, offline record/replay, optional live Anthropic backend), its sparq_nlq::eval exec-accuracy harness (answer-set F1, oracle-vs-end-to-end, grounded-vs-ungrounded; live LLM test off by default), and sparq-introspect (schema/VoID/seed-scoped summaries + planner join hints).
---

# sparq genai-retrieval

The AI/agent surface of the sparq RDF+SPARQL engine, in two opt-in, network-free
crates that compose:

- **`sparq-introspect`** — mines the *effective schema* a graph actually uses
  (classes, per-class predicate usage, observed domain/range, characteristic sets,
  cross-class join hints, namespaces) by sorted scans over the store indexes — and
  renders it as a **token-budgeted text "schema card"**, **VoID** (N-Triples), or
  full **JSON** for LLM grounding / agent retrieval context.
- **`sparq-nlq`** — a deliberately lean **NL→SPARQL loop**: ground (with the
  introspect summary) → generate (an `Llm` behind a trait) → validate (`spargebra`
  parse) → execute (`sparq-engine` under a `QueryBudget`) → repair (≤ N rounds). The
  LLM sits behind a record/replay seam so CI is fully offline; a live Anthropic
  client is behind an opt-in `live` feature.

Both crates are read-only over `sparq-core`'s public API. Nothing in the workspace
depends on them; the default engine build compiles neither and carries zero GenAI
code.

## Quickstart

`Cargo.toml`:

```toml
[dependencies]
sparq-core       = "0.1"
sparq-introspect = "0.1"
sparq-nlq        = "0.1"   # add features = ["live"] for the Anthropic backend
```

Grounding context from a graph (no LLM, no network):

```rust
use sparq_core::Graph;
use sparq_introspect::Introspection;

let graph = Graph::load_str(turtle_or_ntriples, "turtle")?;   // or "ntriples"
let ix = Introspection::build(&graph);

let card = ix.to_text_summary(2500);          // prompt-ready schema card, ≤ 2500 chars
let json = ix.to_json();                       // full machine surface for an agent
let void = ix.to_void("http://ex.org/dataset");// W3C VoID, as N-Triples
# Ok::<(), String>(())
```

NL→SPARQL, offline via a recorded fixture (the CI path):

```rust
use sparq_core::Graph;
use sparq_nlq::{Nlq, ReplayLlm};

let graph  = Graph::load_str(text, "ntriples")?;
let replay = ReplayLlm::from_file("tests/fixtures/olympics_replay.json")?;
let nlq    = Nlq::new(&graph, Box::new(replay));  // builds the schema summary once

let answer = nlq.ask("How many athletes are on each team?")?;
println!("{}", answer.sparql);                    // the executed query (post-repair)
println!("{} rows, {} repair(s)", answer.result.len(), answer.repairs);
for row in &answer.result.rows {                  // Vec<Vec<Option<oxrdf::Term>>>
    // row[i] is None when var answer.result.vars[i] is unbound
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Key APIs

`sparq-introspect`:

```rust
// Build (one full SPO scan + one POS pass + one dict pass). Default tuning is
// sized for LLM grounding; tune with build_with.
Introspection::build(graph: &Graph) -> Introspection
Introspection::build_with(graph: &Graph, opts: &BuildOptions) -> Introspection

// Renderers.
Introspection::to_text_summary(&self, budget_chars: usize) -> String   // schema card
Introspection::to_json(&self) -> String                                // pretty JSON
Introspection::to_void(&self, dataset_iri: &str) -> String             // VoID N-Triples
Introspection::schema_summary_for(&self, seeds: &[&str], budget_chars: usize) -> String

// Result fields (all serde-Serialize):
//   .triples .subjects .entities : u64
//   .classes: Vec<ClassProfile>            // by instance count, per-class predicate usage + coverage
//   .predicates: Vec<PredicateProfile>     // global stats + inferred/declared domain & range
//   .characteristic_sets: CharacteristicSets
//   .join_hints: JoinHints                 // cross-class (C, p, D) edges + triple counts
//   .vocabularies: Vocabularies            // namespaces + well-known recognition

// Planner-facing: the EXACT characteristic-set table in dictionary-id space (no
// caps, no string resolution). Feeds sparq-engine's `cs-planner` CsTable.
sparq_introspect::characteristic_set_ids(graph: &Graph) -> Vec<CsIdSet>
//   CsIdSet { predicates: Box<[Id]>, subjects: u64, predicate_triples: Box<[u64]> }
```

`sparq-nlq`:

```rust
// The single seam to any model — synchronous, stateless (the whole prompt is in `prompt`).
pub trait Llm { fn complete(&self, prompt: &str) -> Result<String, String>; }

// Backends.
ReplayLlm::from_file(path) -> Result<ReplayLlm, String>   // exact-prompt replay (CI)
ReplayLlm::from_json(json) -> Result<ReplayLlm, String>
RecordingLlm::new(inner: L)                               // wraps any Llm; .save(path) writes a fixture
#[cfg(feature = "live")] live::AnthropicLlm::from_env() -> Result<AnthropicLlm, String>
#[cfg(feature = "live")] live::AnthropicLlm::with_model(model) -> Result<AnthropicLlm, String>

// The loop.
Nlq::new(graph: &Graph, llm: Box<dyn Llm>) -> Nlq
Nlq::with_config(graph: &Graph, llm: Box<dyn Llm>, config: NlqConfig) -> Nlq
Nlq::ask(&self, question: &str) -> Result<Answer, NlqError>
Nlq::prompt_for(&self, question: &str) -> String          // deterministic; for building fixtures
Nlq::repair_prompt_for(&self, question, failed_sparql, error) -> String

// Index-grounded entity/relation linking (sparq-sim) — opt-in via NlqConfig.link_entities.
// Resolves the question's proper nouns to concrete IRIs in THIS store's dictionary and
// surfaces them (with structurally-similar siblings from sparq_sim::Sim::most_similar) in
// the grounding prompt. Usable standalone:
sparq_nlq::link::EntityLinker::build(graph: &Graph, expand_k: usize, max_links: usize)
EntityLinker::link(&self, question: &str) -> Linking
// Linking { entities: Vec<LinkedEntity>, relations: Vec<LinkedRelation> }
//   LinkedEntity { mention, iri: NamedNode, label, exact: bool, similar: Vec<NamedNode> }
//   LinkedRelation { mention, iri: NamedNode, triples: u64 }
Linking::to_prompt_section(&self) -> Option<String>      // None when nothing linked

// Answer { sparql: String, result: QueryResult, repairs: usize, transcript: Vec<Turn> }
// NlqError { message: String, transcript: Vec<Turn> }  (impls Error/Display)
// QueryResult { vars: Vec<Variable>, rows: Vec<Vec<Option<Term>>> } — re-exported by sparq-nlq.
```

`NlqConfig` knobs (`Default` = the config the committed fixtures were recorded under):
`summary_budget_chars` (4000), `max_repair_rounds` (1), `exec_timeout`
(`Some(10s)`, native only), `max_rows` (`Some(1_000_000)`), `examples`
(two schema-agnostic few-shots), `ground` (`true` — the grounded loop the fixtures
were recorded under; set `false` for the ungrounded baseline the exec-accuracy
harness measures grounding against — see below), `link_entities` (`false` — opt-in
sparq-sim entity/relation linking appended to the grounded prompt; flipping it on a
recorded dataset means re-recording, since the prompt string changes), `link_expand_k`
(`3` — structurally-similar siblings per linked entity), `max_links` (`8`), and
`check_dictionary` (`false` — the **N2 dictionary constraint**, bead `sq-9yjp`;
see below).

**N2 dictionary constraint** (`sparq_nlq::constrain`, opt-in `check_dictionary`):
after a query parses, walk its algebra and check every predicate / `rdf:type` class
IRI against the live dictionary (`Graph::id_of`). An ungrounded IRI matches no triple,
so it becomes a targeted repair signal (`TurnOutcome::UngroundedTerms`) with
nearest-namespace did-you-mean suggestions pulled from the store. Off by default: an
empty-answer question (a valid-but-absent class) must not be "repaired", and the
exec-accuracy harness measures raw model accuracy with it off. Strict logit-level
**grammar-constrained decoding** is *not* implemented — it needs logit/grammar access
the Anthropic Messages API does not expose, so it is only feasible on a local backend
(`research/genai-nl-to-sparql.md` §11). Public surface:
`constrain::unknown_terms(graph, &spargebra::Query) -> Vec<UnknownTerm>`,
`constrain::dictionary_repair_message(&[UnknownTerm]) -> String`,
`UnknownTerm { iri, role: TermRole, suggestions }`.

`sparq_nlq::eval` — the **exec-accuracy harness** (the design doc's accuracy gate,
`research/genai-design.md` §4). It grades the *executed* query the QALD way — **answer-set
F1**, not query-string equality — and is **fully offline**: it drives `Nlq` over any
`Llm` (in CI a `ReplayLlm` fixture or a scripted in-memory backend); nothing here
touches the network.

```rust
use sparq_nlq::eval::{self, EvalCase, Linking, Comparison, Report};

// One case = a question + the GOLD SPARQL. The gold is executed on the SAME graph at
// scoring time (no checked-in answer blob to drift — research/genai-nl-to-sparql.md §4.3).
EvalCase::new(question: impl Into<String>, gold_sparql: impl Into<String>) -> EvalCase

// Score one (linking, grounding) configuration over the cases against `graph`.
// For Linking::Oracle the gold query is fed straight through validate→execute and the
// `llm` is NOT consulted (isolates the engine-side loop from model linking).
eval::run_config(graph: &Graph, cases: &[EvalCase], llm: Box<dyn Llm>,
                 config: NlqConfig, linking: Linking) -> Report

// Drives all four cells (end-to-end | oracle) × (grounded | ungrounded). `make_llm(ground)`
// is called once per end-to-end config (ReplayLlm is consumed by the loop, so hand out a
// fresh fixture each call; the grounded/ungrounded prompts are distinct strings).
eval::run_comparison(graph: &Graph, cases: &[EvalCase], base_config: &NlqConfig,
                     make_llm: impl FnMut(bool) -> Box<dyn Llm>) -> Comparison

// Linking ∈ { EndToEnd, Oracle } — reported SEPARATELY, per the design doc.
// Report { linking, grounded: bool, cases: Vec<CaseResult>, macro_f1, exact_match,
//          validity, total_repairs }  // macro_f1 = QALD headline metric
// CaseResult { question, outcome: Result<CaseScore, String> }  // Err = loop failure (F1=0 vs non-empty gold)
// CaseScore { f1: F1, sparql, repairs }
// F1 { precision, recall, f1, intersection, predicted, gold } ; F1::is_exact() -> bool
// AnswerSet::from_result(&QueryResult) -> AnswerSet  // order-independent multiset of rows; F1::score(pred, gold)
// Comparison { grounded_end_to_end, ungrounded_end_to_end, grounded_oracle, ungrounded_oracle }
// Comparison::headline_grounding_pays() -> bool  // grounded end-to-end macro-F1 > ungrounded ("grounding pays for itself")
// Comparison::summary() -> String                // a stderr table; printed at run time, no perf numbers baked into docs
```

## Common recipes

**1. One-call grounded retrieval context for an agent.** Hand the schema card plus
the seed-scoped detail into your own prompt:

```rust
let ix = Introspection::build(&graph);
let overview = ix.to_text_summary(3000);                  // dataset-wide card
let detail   = ix.schema_summary_for(                     // zoom into entities you care about
    &["http://xmlns.com/foaf/0.1/Person", "http://dbpedia.org/ontology/team"],
    1500,
);
let context = format!("{overview}\n\n{detail}");          // feed to your LLM
```

**2. Record a live session, then replay it forever.** Run live once to capture a
fixture; CI replays it with no network:

```rust
# #[cfg(feature = "live")] {
use std::rc::Rc;
use sparq_nlq::{live::AnthropicLlm, Nlq, NlqConfig, RecordingLlm};

let llm = Rc::new(RecordingLlm::new(AnthropicLlm::from_env()?));  // ANTHROPIC_API_KEY
let nlq = Nlq::with_config(&graph, Box::new(Rc::clone(&llm)), NlqConfig {
    max_repair_rounds: 3,                 // design-doc cap for live runs
    ..NlqConfig::default()
});
let _ = nlq.ask("Which team has the most athletes?")?;
llm.save("session.json")?;                // a ReplayLlm fixture for the regression set
# }
# Ok::<(), Box<dyn std::error::Error>>(())
```

**3. Inspect the provenance / debug a failure.** Every `ask` carries the full
transcript; `NlqError` carries it too:

```rust
match nlq.ask("a hard question") {
    Ok(a) => for t in &a.transcript {
        // t.prompt, t.completion, t.sparql: Option<String>, t.outcome: TurnOutcome
        // TurnOutcome ∈ { Ok{rows}, NoSparql, ParseError(s), ExecError(s) }
        println!("{:?}", t.outcome);
    },
    Err(e) => eprintln!("{e}\n{:#?}", e.transcript),  // Display = "<message> (after N turn(s))"
}
```

**4. Export VoID for a dataset catalog / DCAT entry.** Output is N-Triples (a Turtle
subset), so it re-parses with either parser:

```rust
let void_nt = Introspection::build(&graph).to_void("http://ex.org/dataset");
// void:Dataset with void:triples / void:entities / void:distinctSubjects /
// void:classes / void:properties, plus one classPartition per class and one
// propertyPartition per predicate. NOTE: void:distinctObjects is NOT emitted.
```

**5. Tune the introspection for a huge or noisy KG.** Bound the histograms and tables:

```rust
use sparq_introspect::{Introspection, BuildOptions};
let opts = BuildOptions { max_char_sets: 200, max_namespaces: 50, max_join_hints: 200,
                          ..BuildOptions::default() };
let ix = Introspection::build_with(&graph, &opts);   // tails aggregate into elided_* fields
```

**6. Feed exact characteristic sets to the engine's cardinality planner.** Requires
`sparq-engine` built with `--features cs-planner`:

```rust
# #[cfg(feature = "cs-planner")] {
use sparq_engine::cs::{CsSet, CsTable};
let table = CsTable::new(
    sparq_introspect::characteristic_set_ids(&graph).into_iter().map(|s| CsSet {
        predicates: s.predicates, subjects: s.subjects, predicate_triples: s.predicate_triples,
    }),
);
// ids are valid only against THIS graph's dictionary — rebuild when the graph rebuilds.
# }
```

**7. Measure NL→SPARQL exec-accuracy (answer-F1) offline.** Build a small set of
`(question, gold SPARQL)` cases, then run all four cells and assert that grounding pays:

```rust
use sparq_nlq::eval::{run_comparison, EvalCase};
use sparq_nlq::{NlqConfig, ReplayLlm};

let cases = vec![
    EvalCase::new("How many athletes are on each team?",
                  "SELECT ?t (COUNT(?a) AS ?n) WHERE { ?a :team ?t } GROUP BY ?t"),
    // … more (question, GOLD query) pairs
];
// `ground` flips per cell; the closure hands out a fresh fixture per end-to-end call.
let cmp = run_comparison(&graph, &cases, &NlqConfig::default(), |ground| {
    let f = if ground { "tests/fixtures/grounded.json" } else { "tests/fixtures/ungrounded.json" };
    Box::new(ReplayLlm::from_file(f).unwrap())
});
eprintln!("{}", cmp.summary());                 // printed at run time only
assert!(cmp.headline_grounding_pays());          // grounded end-to-end macro-F1 > ungrounded
# Ok::<(), Box<dyn std::error::Error>>(())
```

What this proves (and what it does NOT): with scripted/recorded completions it validates
the **harness** and the **mechanism** of the headline claim — answer-F1 scoring,
oracle-vs-end-to-end and grounded-vs-ungrounded reporting, the inequality — **not** a
real model's accuracy numbers. Those come from a `live_exec_accuracy` run (`--features
live` + `RecordingLlm`, which needs a key + network + dataset); that test is `#[ignore]`'d
and feature-gated **OFF**, so a model or network outage can never red the build. After a
live run the recorded sessions become the offline regression set — and the
record→`save`→`from_file`→identical-scores round-trip that makes them a *faithful*
regression set is itself CI-gated (`recorded_session_saves_and_replays_as_regression_set`
in `tests/exec_accuracy.rs`). (No claim is made here about NL→SPARQL quality beyond what
the fixtures encode.)

## Gotchas / feature flags / prerequisites

- **Opt-in crates.** Neither crate is in the default build; add `sparq-introspect` /
  `sparq-nlq` explicitly. They depend only on `sparq-core`'s public scan API, so they
  work against every storage mode (raw, mmap'd, compressed).
- **`live` feature (sparq-nlq).** OFF by default — without it there is no
  `live::AnthropicLlm` and no network code. With it, `AnthropicLlm` reads
  `ANTHROPIC_API_KEY`, POSTs `https://api.anthropic.com/v1/messages` (blocking, 120 s
  timeout), default model `claude-sonnet-4-6` (configurable via `with_model`). CI
  only compile-checks this path; no test calls the network.
- **`ReplayLlm` is exact-prompt match.** Prompts are deterministic functions of the
  graph + `NlqConfig` + examples, so a miss means the prompt template, the default
  `NlqConfig`, or the dataset drifted — re-record (e.g.
  `cargo run -p sparq-nlq --example record_olympics --release`). The miss error
  surfaces the question lines that *are* recorded.
- **Generated SPARQL is untrusted.** `ask` always executes under a `QueryBudget`
  (default 10 s wall clock + 1M rows). Budget trips surface as `TurnOutcome::ExecError`
  and are a repair signal. Opt out only by setting `exec_timeout` / `max_rows` to
  `None` explicitly. `exec_timeout` is native-only (no `Instant` on `wasm32`; the
  deadline is simply not set there).
- **Prompt constrains the model** to SELECT/ASK, schema-summary vocabulary only, no
  property paths, no federation, one ```sparql block. `extract_sparql` tolerates a
  ```sparql fence, a bare ``` fence, or bare query text starting `PREFIX`/`SELECT`/`ASK`.
- **ASK answers** use the engine's unit-row encoding: zero `vars`, one empty row iff
  true (`answer.result.vars.is_empty()`, `result.len() == 1` when true).
- **`eval` (exec-accuracy) is offline by construction and grades the answer set, not
  the query string.** It executes both the candidate and the **gold** query on the same
  graph and compares bind-row multisets (order-independent; empty-vs-empty scores 1, the
  QALD true-negative convention). The *live-model* accuracy run (`live_exec_accuracy`) is
  `#[cfg(feature = "live")]` **and** `#[ignore]`'d — it is never in the gate, so the
  scored numbers in CI come only from scripted/recorded completions. Treat them as a
  harness/mechanism check, not a model-quality benchmark.
- **Summary hints are hints.** In `to_text_summary`, per-class counts/coverage are
  exact, but the trailing `→ range, e.g. sample` come from the predicate's *global*
  profile. `schema_summary_for` is struct-level scoping (filters already-mined
  profiles by IRI) — it does NOT re-scan, so it won't chase a seed entity's instances.
- **Cost.** Introspection is `O(|G| + |dict|)` (sorted scans, no GROUP BY); measured
  ~0.1 s build on 1.78M triples. LLM-excluded `ask` latency is the engine's query
  time (p50 ~10 ms at olympics scale) — the LLM round trip dominates wall clock.
- **Graph scoping (sq-quuu).** `Introspection::build(&graph)` describes the **store of
  the `Graph` it is handed** — the **default graph** for the top-level `&graph`, or a
  **single named graph** when you pass that graph's sub-`Graph`. A schema card / VoID is
  always scoped to exactly one graph; there is **no cross-graph or union-of-all-graphs
  build**. On a quad dataset (N-Quads / TriG via `Graph::load_dataset`) each named graph
  is a self-contained `Graph`; fetch one by name with `graph.named_graph(&name)` and
  introspect it alone — `let card = Introspection::build(graph.named_graph(&g)?);`. Run it
  per graph (or over the default graph) rather than expecting it to merge the quads.

## See also

- `hdt-format` — load `.hdt` archives into a `Graph` before introspecting.
- `fused-decompress-parse`, `rust-parallel-parsing` — fast ingest into the `Graph`
  these crates read (`Graph::load_reader_parallel`).
- `mpc-protocols`, `noir-circuit-patterns`, `verifiable-credentials-zk`,
  `sparql-formal-semantics` — the verifiable/private query estate, orthogonal to this
  retrieval surface.
</skill_md>
<parameter name="key_apis">["Introspection::build(graph: &Graph) -> Introspection", "Introspection::build_with(graph: &Graph, opts: &BuildOptions) -> Introspection", "Introspection::to_text_summary(&self, budget_chars: usize) -> String", "Introspection::to_json(&self) -> String", "Introspection::to_void(&self, dataset_iri: &str) -> String", "Introspection::schema_summary_for(&self, seeds: &[&str], budget_chars: usize) -> String", "sparq_introspect::characteristic_set_ids(graph: &Graph) -> Vec<CsIdSet>", "trait Llm { fn complete(&self, prompt: &str) -> Result<String, String>; }", "ReplayLlm::from_file / from_json", "RecordingLlm::new(inner) + .save(path)", "live::AnthropicLlm::from_env() / with_model(model)  (feature = \"live\")", "Nlq::new(graph, Box<dyn Llm>) / with_config(graph, llm, NlqConfig)", "Nlq::ask(&self, question: &str) -> Result<Answer, NlqError>", "Nlq::prompt_for / repair_prompt_for (deterministic, for fixtures)", "Answer { sparql, result: QueryResult, repairs, transcript }", "NlqConfig { summary_budget_chars, max_repair_rounds, exec_timeout, max_rows, examples, ground, link_entities, link_expand_k, max_links, check_dictionary }", "sparq_nlq::link::EntityLinker::build(graph: &Graph, expand_k: usize, max_links: usize)", "EntityLinker::link(&self, question: &str) -> Linking", "sparq_nlq::link::Linking { entities: Vec<LinkedEntity>, relations: Vec<LinkedRelation> } ; Linking::to_prompt_section(&self) -> Option<String>", "sparq_nlq::constrain::unknown_terms(graph: &Graph, query: &spargebra::Query) -> Vec<UnknownTerm>", "sparq_nlq::constrain::dictionary_repair_message(unknowns: &[UnknownTerm]) -> String", "sparq_nlq::constrain::UnknownTerm { iri, role: TermRole, suggestions }", "sparq_nlq::eval::EvalCase::new(question, gold_sparql)", "sparq_nlq::eval::run_config(graph, cases, llm, config, Linking) -> Report", "sparq_nlq::eval::run_comparison(graph, cases, base_config, make_llm) -> Comparison", "Comparison::headline_grounding_pays() -> bool / summary() -> String", "F1::score(&AnswerSet, &AnswerSet) -> F1 / is_exact() -> bool ; AnswerSet::from_result(&QueryResult)", "sparq_engine::cs::{CsSet, CsTable}  (sparq-engine feature = \"cs-planner\")"]
