//! sparq-nlq: natural-language questions over a [`sparq_core::Graph`], answered with
//! SPARQL (GenAI phase 3 — `research/genai-design.md` §2, `research/genai-nl-to-sparql.md`).
//!
//! The loop is deliberately **lean** (the SPARQL-LLM finding: retrieve + repair beats
//! sprawling agents):
//!
//! ```text
//!            ┌──────────────────────────────────────────────────────┐
//!            │ GROUND   sparq_introspect::to_text_summary(budget)   │
//!            │          + few-shot examples + the question          │
//!            └───────────────────────────┬──────────────────────────┘
//!                                        ▼
//!            ┌──────────────────────────────────────────────────────┐
//!      ┌────▶│ GENERATE Llm::complete(prompt) -> completion         │
//!      │     └───────────────────────────┬──────────────────────────┘
//!      │                                 ▼
//!      │     ┌──────────────────────────────────────────────────────┐
//!      │     │ VALIDATE extract ```sparql block, spargebra parse    │
//!      │     └───────────┬──────────────────────────┬───────────────┘
//!      │           parse error                  parses
//!      │                 │                          ▼
//!      │     ┌───────────┴───────────┐  ┌──────────────────────────┐
//!      └─────┤ REPAIR (≤ N rounds:   │  │ EXECUTE sparq_engine::   │
//!      ▲     │ error + failed query  │  │ query_with_budget        │
//!      │     │ back to the LLM)      │  └────────────┬─────────────┘
//!      │     └───────────────────────┘     exec error│        ok
//!      └──────────────────────────────────◀──────────┘         ▼
//!                                              Answer { sparql, result,
//!                                                       repairs, transcript }
//! ```
//!
//! The LLM sits behind the [`Llm`] trait. CI never touches the network:
//! [`ReplayLlm`] serves recorded prompt→completion pairs from a JSON fixture, and
//! [`RecordingLlm`] wraps any backend to produce such fixtures. A thin Anthropic
//! Messages-API client (`AnthropicLlm`) is available behind
//! the non-default `live` feature.
#![forbid(unsafe_code)] // [OPUS-4.8] sq-emay: crate has zero `unsafe`

use std::cell::RefCell;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sparq_core::Graph;
use sparq_engine::query_with_budget;
use sparq_introspect::Introspection;

// Re-exported so callers configuring the loop need only this crate's namespace.
pub use sparq_engine::{QueryBudget, QueryResult};

#[cfg(feature = "live")]
pub mod live;

pub mod constrain;
pub mod eval;

// ---------------------------------------------------------------------------
// The LLM boundary
// ---------------------------------------------------------------------------

/// The single seam between this crate and any language model. Synchronous and
/// stateless by design: every call carries the full prompt (grounding, examples,
/// question, and — on repair rounds — the failed query and parser error), so
/// recording and replaying a session is just a list of `(prompt, completion)` pairs.
pub trait Llm {
    fn complete(&self, prompt: &str) -> Result<String, String>;
}

/// Shared backends: keep a handle outside the loop (e.g. an `Rc<RecordingLlm<_>>`
/// whose clone goes into [`Nlq::new`] while the original collects the exchanges).
impl<L: Llm + ?Sized> Llm for std::rc::Rc<L> {
    fn complete(&self, prompt: &str) -> Result<String, String> {
        (**self).complete(prompt)
    }
}

/// One recorded LLM call. The fixture format of [`ReplayLlm`] / [`RecordingLlm`]:
/// a JSON array of these.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Exchange {
    pub prompt: String,
    pub completion: String,
}

/// Replays recorded prompt→completion pairs from a fixture — **the CI path**.
/// Lookup is by exact prompt match: the prompt is fully deterministic (the schema
/// summary is a pure function of the graph, examples and templates are fixed), so a
/// fixture recorded once keeps replaying until the prompt construction or the
/// dataset changes — at which point re-record (see `examples/record_olympics.rs`).
pub struct ReplayLlm {
    exchanges: Vec<Exchange>,
}

impl ReplayLlm {
    pub fn new(exchanges: Vec<Exchange>) -> Self {
        Self { exchanges }
    }

    /// Loads a fixture produced by [`RecordingLlm::to_json`] / `save`.
    pub fn from_json(json: &str) -> Result<Self, String> {
        let exchanges: Vec<Exchange> =
            serde_json::from_str(json).map_err(|e| format!("invalid replay fixture: {e}"))?;
        Ok(Self { exchanges })
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let text = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            format!(
                "cannot read replay fixture {}: {e}",
                path.as_ref().display()
            )
        })?;
        Self::from_json(&text)
    }

    pub fn len(&self) -> usize {
        self.exchanges.len()
    }

    pub fn is_empty(&self) -> bool {
        self.exchanges.is_empty()
    }
}

impl Llm for ReplayLlm {
    fn complete(&self, prompt: &str) -> Result<String, String> {
        if let Some(e) = self.exchanges.iter().find(|e| e.prompt == prompt) {
            return Ok(e.completion.clone());
        }
        // A miss means the prompt construction (or the dataset) drifted from the
        // recording. Surface the question lines of what IS recorded to make the
        // mismatch diagnosable without dumping kilobytes of schema summary.
        let recorded: Vec<&str> = self
            .exchanges
            .iter()
            .filter_map(|e| e.prompt.lines().rev().find(|l| l.starts_with("Question:")))
            .collect();
        Err(format!(
            "replay miss: no recorded completion for this prompt (its question line: {:?}); \
             {} recorded exchanges with question lines {:?} — re-record the fixture",
            prompt.lines().rev().find(|l| l.starts_with("Question:")),
            self.exchanges.len(),
            recorded
        ))
    }
}

/// Wraps any [`Llm`] and records every `(prompt, completion)` pair, for producing
/// [`ReplayLlm`] fixtures. Failed calls are not recorded (a fixture replays the
/// happy path; errors should be re-derived live).
pub struct RecordingLlm<L: Llm> {
    inner: L,
    recorded: RefCell<Vec<Exchange>>,
}

impl<L: Llm> RecordingLlm<L> {
    pub fn new(inner: L) -> Self {
        Self {
            inner,
            recorded: RefCell::new(Vec::new()),
        }
    }

    pub fn exchanges(&self) -> Vec<Exchange> {
        self.recorded.borrow().clone()
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&*self.recorded.borrow()).expect("exchanges serialise to JSON")
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), String> {
        std::fs::write(path.as_ref(), self.to_json())
            .map_err(|e| format!("cannot write fixture {}: {e}", path.as_ref().display()))
    }
}

impl<L: Llm> Llm for RecordingLlm<L> {
    fn complete(&self, prompt: &str) -> Result<String, String> {
        let completion = self.inner.complete(prompt)?;
        self.recorded.borrow_mut().push(Exchange {
            prompt: prompt.to_string(),
            completion: completion.clone(),
        });
        Ok(completion)
    }
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// A few-shot example embedded in the grounding prompt.
#[derive(Debug, Clone)]
pub struct Example {
    pub question: String,
    pub sparql: String,
}

/// Knobs for the ask loop. `Default` is the configuration the committed fixtures
/// were recorded under — change a knob, re-record.
#[derive(Debug, Clone)]
pub struct NlqConfig {
    /// Character budget handed to [`Introspection::to_text_summary`] — the grounding
    /// deck. The introspect crate truncates greedily at line granularity, most
    /// important information first.
    pub summary_budget_chars: usize,
    /// Maximum REPAIR rounds after the initial generation (a parse or execution
    /// failure sends the error + failed query back to the LLM). The design doc caps
    /// this at 3; the default is the leanest loop that still self-corrects.
    pub max_repair_rounds: usize,
    /// Wall-clock cap on ONE query execution (turned into a [`QueryBudget`] deadline
    /// at execution time, so the config is reusable across `ask` calls). LLM-generated
    /// queries are untrusted input to the engine — the default is bounded (10 s);
    /// set `None` only to opt out explicitly. Native only (wasm has no `Instant`).
    pub exec_timeout: Option<std::time::Duration>,
    /// Cap on the rows of any materialised (intermediate or final) result — the
    /// other half of the budget. Default: 1,000,000. `None` opts out.
    pub max_rows: Option<usize>,
    /// Few-shot examples. The defaults are schema-agnostic (rdf:type / rdfs:label
    /// only) so one prompt template serves any dataset.
    pub examples: Vec<Example>,
    /// Whether to include the introspect schema summary in the prompt. `true`
    /// (default) is the grounded loop the committed fixtures were recorded under.
    /// Set `false` for the **ungrounded baseline** the design doc requires
    /// (`research/genai-design.md` §4, `sparq-nlq` row: exec-accuracy must beat
    /// the same LLM WITHOUT grounding — "the grounding must pay for itself").
    /// Construction still runs the introspection scan (it is cheap and the field
    /// is read at prompt time), so one [`Nlq`] can be reconfigured grounded vs
    /// ungrounded; the eval harness drives both off the same graph. [OPUS-4.8] sq-05rv
    pub ground: bool,
    /// N2 **dictionary-grounded constraint** ([`constrain`], `sq-9yjp`). When `true`,
    /// a query that parses is additionally checked against the **live dictionary**
    /// *before* execution: every predicate / `rdf:type` class IRI must be a term the
    /// store actually holds (via [`sparq_core::Graph::id_of`]). An ungrounded IRI
    /// matches no triple — a *valid-but-wrong* query — so it becomes a targeted repair
    /// signal ("predicate X not in the dictionary; did you mean Y?") with
    /// nearest-namespace candidates pulled from the dictionary, exactly as
    /// `research/genai-nl-to-sparql.md` §6 prescribes. Strict logit-level
    /// grammar-constrained *decoding* is not implementable against the Anthropic
    /// Messages API (no logit/grammar parameter); this is the design doc's API-backend
    /// fallback (§11).
    ///
    /// **Default `false`** — and deliberately so. The check is an *additive* constraint
    /// with a real false-positive: a query whose answer is legitimately empty (a class
    /// with no instances, e.g. `SELECT ?r WHERE { ?r a ex:Robot }` over a robot-free
    /// graph) uses a perfectly valid term that is simply *absent* from the dictionary,
    /// and must not be "repaired". So it stays opt-in, the same way [`ground`](Self::ground)
    /// is a knob: the exec-accuracy harness measures raw model accuracy with it off,
    /// and a caller who wants tighter grounding (accepting that empty-answer questions
    /// then need it off) sets it `true`. [OPUS-4.8] sq-9yjp
    pub check_dictionary: bool,
}

impl Default for NlqConfig {
    fn default() -> Self {
        Self {
            summary_budget_chars: 4000,
            max_repair_rounds: 1,
            exec_timeout: Some(std::time::Duration::from_secs(10)),
            max_rows: Some(1_000_000),
            examples: vec![
                Example {
                    question: "How many entities of each type are there?".into(),
                    sparql: "PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
                             SELECT ?type (COUNT(?s) AS ?count)\n\
                             WHERE { ?s rdf:type ?type }\n\
                             GROUP BY ?type\n\
                             ORDER BY DESC(?count)"
                        .into(),
                },
                Example {
                    question: "Show a few entities together with their labels.".into(),
                    sparql: "PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>\n\
                             SELECT ?s ?label\n\
                             WHERE { ?s rdfs:label ?label }\n\
                             LIMIT 10"
                        .into(),
                },
            ],
            ground: true,
            check_dictionary: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Answer + transcript
// ---------------------------------------------------------------------------

/// What happened to one LLM completion inside the loop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TurnOutcome {
    /// Parsed and executed; the query returned this many rows.
    Ok { rows: usize },
    /// The completion contained no extractable SPARQL.
    NoSparql,
    /// spargebra rejected the extracted query.
    ParseError(String),
    /// The engine rejected or aborted the (syntactically valid) query.
    ExecError(String),
    /// N2 dictionary constraint (`sq-9yjp`): the query parsed and executed, but used a
    /// predicate / class IRI absent from the live dictionary (so it matched no
    /// triples). Carries the ungrounded terms; the repair message lists them with
    /// nearest-namespace suggestions. [OPUS-4.8]
    UngroundedTerms(Vec<constrain::UnknownTerm>),
}

/// One generate→validate(→execute) round, kept verbatim for auditability — the
/// transcript IS the provenance of the answer.
#[derive(Debug, Clone)]
pub struct Turn {
    pub prompt: String,
    pub completion: String,
    /// The SPARQL extracted from the completion (`None` on [`TurnOutcome::NoSparql`]).
    pub sparql: Option<String>,
    pub outcome: TurnOutcome,
}

/// A successful run of the loop.
pub struct Answer {
    /// The query that produced `result` (post-repair, if any).
    pub sparql: String,
    /// Materialised solutions. ASK answers use the engine's unit-row encoding:
    /// zero variables, one empty row iff true.
    pub result: QueryResult,
    /// How many repair rounds were needed (0 = first completion was good).
    pub repairs: usize,
    /// Every round, including failed ones.
    pub transcript: Vec<Turn>,
}

// Manual impl: `QueryResult` (sparq-engine) does not derive `Debug`, and this crate
// does not modify core crates (workspace invariant) — summarise it instead.
impl std::fmt::Debug for Answer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Answer")
            .field("sparql", &self.sparql)
            .field("result_vars", &self.result.vars)
            .field("result_rows", &self.result.len())
            .field("repairs", &self.repairs)
            .field("transcript_turns", &self.transcript.len())
            .finish()
    }
}

/// A failed run — carries the full transcript so the failure is diagnosable.
#[derive(Debug)]
pub struct NlqError {
    pub message: String,
    pub transcript: Vec<Turn>,
}

impl std::fmt::Display for NlqError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} (after {} turn(s))",
            self.message,
            self.transcript.len()
        )
    }
}

impl std::error::Error for NlqError {}

// ---------------------------------------------------------------------------
// The loop
// ---------------------------------------------------------------------------

/// NL→SPARQL over one graph. Construction runs the introspection scan once and
/// renders the schema summary; `ask` is then grounding-free of further index work
/// until execution.
pub struct Nlq<'g> {
    graph: &'g Graph,
    schema_summary: String,
    llm: Box<dyn Llm>,
    config: NlqConfig,
}

impl<'g> Nlq<'g> {
    pub fn new(graph: &'g Graph, llm: Box<dyn Llm>) -> Self {
        Self::with_config(graph, llm, NlqConfig::default())
    }

    pub fn with_config(graph: &'g Graph, llm: Box<dyn Llm>, config: NlqConfig) -> Self {
        let schema_summary =
            Introspection::build(graph).to_text_summary(config.summary_budget_chars);
        Self {
            graph,
            schema_summary,
            llm,
            config,
        }
    }

    /// The grounding prompt for `question` — public (and deterministic) so that
    /// fixtures can be constructed and inspected outside the loop.
    ///
    /// When [`NlqConfig::ground`] is `false` the schema summary is omitted: this is
    /// the **ungrounded baseline** prompt (`research/genai-design.md` §4 — the same
    /// LLM with no schema deck). The two prompts are distinct strings, so a fixture
    /// recorded for one does not collide with the other under [`ReplayLlm`].
    pub fn prompt_for(&self, question: &str) -> String {
        let mut p = String::new();
        if self.config.ground {
            p.push_str(
                "You are a SPARQL query writer. Given the schema summary of an RDF dataset and a \
                 question, write ONE SPARQL 1.1 SELECT or ASK query that answers the question.\n\
                 \n\
                 Rules:\n\
                 - Use only classes and predicates that appear in the schema summary.\n\
                 - Declare every prefix you use with PREFIX lines (expand them from the summary's \
                 prefix glossary).\n\
                 - Do not use property paths or federation.\n\
                 - Output exactly one ```sparql code block and nothing else.\n\
                 \n",
            );
            p.push_str(&self.schema_summary);
            if !p.ends_with('\n') {
                p.push('\n');
            }
            p.push('\n');
        } else {
            // Ungrounded: no schema deck. The model must guess vocabulary — the
            // baseline that grounding has to beat.
            p.push_str(
                "You are a SPARQL query writer. Given a question about an RDF dataset, write ONE \
                 SPARQL 1.1 SELECT or ASK query that answers the question.\n\
                 \n\
                 Rules:\n\
                 - Declare every prefix you use with PREFIX lines.\n\
                 - Do not use property paths or federation.\n\
                 - Output exactly one ```sparql code block and nothing else.\n\
                 \n",
            );
        }
        for ex in &self.config.examples {
            p.push_str(&format!(
                "Question: {}\n```sparql\n{}\n```\n\n",
                ex.question, ex.sparql
            ));
        }
        p.push_str(&format!("Question: {question}"));
        p
    }

    /// The repair prompt: the failed query and the error, stacked onto the full
    /// grounding prompt (the trait is stateless — every call must carry everything).
    pub fn repair_prompt_for(&self, question: &str, failed_sparql: &str, error: &str) -> String {
        format!(
            "{}\n\nYour previous query failed. Here is what you wrote:\n```sparql\n{}\n```\n\n\
             Error: {}\n\nFix the query. Output exactly one ```sparql code block and nothing else.",
            self.prompt_for(question),
            failed_sparql,
            error
        )
    }

    /// The per-execution [`QueryBudget`]: built fresh at execution time so the
    /// configured timeout is a per-query duration, not a fixed absolute deadline.
    fn execution_budget(&self) -> QueryBudget {
        let mut b = QueryBudget::unlimited();
        b.max_rows = self.config.max_rows;
        #[cfg(not(target_arch = "wasm32"))]
        {
            b.deadline = self
                .config
                .exec_timeout
                .map(|d| std::time::Instant::now() + d);
        }
        b
    }

    /// Runs the loop: ground → generate → validate → execute, with up to
    /// `max_repair_rounds` repair rounds on parse or execution failure.
    pub fn ask(&self, question: &str) -> Result<Answer, NlqError> {
        let mut transcript: Vec<Turn> = Vec::new();
        let mut prompt = self.prompt_for(question);
        // Initial round + repair rounds.
        for round in 0..=self.config.max_repair_rounds {
            let completion = match self.llm.complete(&prompt) {
                Ok(c) => c,
                Err(e) => {
                    return Err(NlqError {
                        message: format!("LLM error: {e}"),
                        transcript,
                    })
                }
            };

            // VALIDATE: extract, then parse with spargebra (the parser error message
            // is the repair signal).
            let (sparql, failure): (Option<String>, Option<(String, TurnOutcome)>) =
                match extract_sparql(&completion) {
                    None => (
                        None,
                        Some((
                            "the completion contained no SPARQL code block".to_string(),
                            TurnOutcome::NoSparql,
                        )),
                    ),
                    Some(q) => match spargebra::SparqlParser::new().parse_query(&q) {
                        Err(e) => {
                            let msg = e.to_string();
                            (Some(q), Some((msg.clone(), TurnOutcome::ParseError(msg))))
                        }
                        Ok(parsed) => {
                            // N2 dictionary constraint (`sq-9yjp`): a query whose
                            // predicate/class IRIs are not in the live dictionary
                            // cannot match — turn it into a targeted repair signal
                            // BEFORE spending an execution on a guaranteed-empty query.
                            let unknowns = if self.config.check_dictionary {
                                constrain::unknown_terms(self.graph, &parsed)
                            } else {
                                Vec::new()
                            };
                            if !unknowns.is_empty() {
                                let msg = constrain::dictionary_repair_message(&unknowns);
                                (Some(q), Some((msg, TurnOutcome::UngroundedTerms(unknowns))))
                            } else {
                                // EXECUTE under the budget. Engine-side failures
                                // (unsupported form, budget trip) are repairable too.
                                match query_with_budget(self.graph, &q, &self.execution_budget()) {
                                    Err(e) => {
                                        (Some(q), Some((e.clone(), TurnOutcome::ExecError(e))))
                                    }
                                    Ok(result) => {
                                        transcript.push(Turn {
                                            prompt,
                                            completion,
                                            sparql: Some(q.clone()),
                                            outcome: TurnOutcome::Ok { rows: result.len() },
                                        });
                                        return Ok(Answer {
                                            sparql: q,
                                            result,
                                            repairs: round,
                                            transcript,
                                        });
                                    }
                                }
                            }
                        }
                    },
                };

            let (error, outcome) = failure.expect("non-success path always has a failure");
            transcript.push(Turn {
                prompt: prompt.clone(),
                completion,
                sparql: sparql.clone(),
                outcome,
            });
            if round == self.config.max_repair_rounds {
                return Err(NlqError {
                    message: format!("no valid query after {round} repair round(s): {error}"),
                    transcript,
                });
            }
            prompt = self.repair_prompt_for(
                question,
                sparql.as_deref().unwrap_or("(no query produced)"),
                &error,
            );
        }
        unreachable!("loop returns from its last round")
    }
}

/// Extracts the SPARQL query from a completion: the first ```sparql fenced block,
/// else the first anonymous ``` fenced block, else — if the text itself looks like a
/// bare query — the whole trimmed completion.
pub fn extract_sparql(completion: &str) -> Option<String> {
    if let Some(q) = extract_fenced(completion, "```sparql") {
        return Some(q);
    }
    if let Some(q) = extract_fenced(completion, "```") {
        return Some(q);
    }
    let t = completion.trim();
    let upper = t.to_ascii_uppercase();
    if upper.starts_with("PREFIX") || upper.starts_with("SELECT") || upper.starts_with("ASK") {
        return Some(t.to_string());
    }
    None
}

fn extract_fenced(text: &str, opener: &str) -> Option<String> {
    let start = text.find(opener)? + opener.len();
    // The fence opener runs to end-of-line (tolerates ```sparql\n and bare ```\n).
    let body_start = text[start..].find('\n').map(|i| start + i + 1)?;
    let body_end = text[body_start..].find("```").map(|i| body_start + i)?;
    let q = text[body_start..body_end].trim();
    (!q.is_empty()).then(|| q.to_string())
}

// ---------------------------------------------------------------------------
// Tests (offline: tiny in-memory graph, scripted/replayed LLMs)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Closure-backed test backend.
    struct FnLlm<F: Fn(&str) -> Result<String, String>>(F);
    impl<F: Fn(&str) -> Result<String, String>> Llm for FnLlm<F> {
        fn complete(&self, prompt: &str) -> Result<String, String> {
            (self.0)(prompt)
        }
    }

    fn tiny_graph() -> Graph {
        let ttl = r#"
            @prefix ex: <http://example.org/> .
            @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
            ex:alice a ex:Person ; rdfs:label "Alice" ; ex:knows ex:bob .
            ex:bob a ex:Person ; rdfs:label "Bob" .
            ex:acme a ex:Company ; rdfs:label "Acme" .
        "#;
        Graph::load_str(ttl, "turtle").expect("tiny graph parses")
    }

    const COUNT_PEOPLE: &str = "PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
         PREFIX ex: <http://example.org/>\n\
         SELECT (COUNT(?p) AS ?n) WHERE { ?p rdf:type ex:Person }";

    #[test]
    fn extract_prefers_sparql_fence() {
        let c = "Here you go:\n```sparql\nSELECT * WHERE { ?s ?p ?o }\n```\nDone.";
        assert_eq!(extract_sparql(c).unwrap(), "SELECT * WHERE { ?s ?p ?o }");
    }

    #[test]
    fn extract_falls_back_to_anonymous_fence_and_bare_text() {
        let c = "```\nASK { ?s ?p ?o }\n```";
        assert_eq!(extract_sparql(c).unwrap(), "ASK { ?s ?p ?o }");
        let bare = "  SELECT ?s WHERE { ?s ?p ?o }  ";
        assert_eq!(
            extract_sparql(bare).unwrap(),
            "SELECT ?s WHERE { ?s ?p ?o }"
        );
        assert_eq!(extract_sparql("I cannot answer that."), None);
    }

    #[test]
    fn prompt_contains_grounding_examples_and_question() {
        let g = tiny_graph();
        let nlq = Nlq::new(&g, Box::new(FnLlm(|_| Err("unused".into()))));
        let p = nlq.prompt_for("How many people are there?");
        assert!(p.starts_with("You are a SPARQL query writer."));
        assert!(p.contains("# Schema summary"));
        assert!(p.contains("ex:Person") || p.contains("Person")); // grounded in THIS graph
        assert!(p.contains("Question: How many entities of each type are there?")); // few-shot
        assert!(p.ends_with("Question: How many people are there?"));
    }

    #[test]
    fn ask_happy_path() {
        let g = tiny_graph();
        let nlq = Nlq::new(
            &g,
            Box::new(FnLlm(|_| Ok(format!("```sparql\n{COUNT_PEOPLE}\n```")))),
        );
        let a = nlq
            .ask("How many people are there?")
            .expect("loop succeeds");
        assert_eq!(a.repairs, 0);
        assert_eq!(a.result.len(), 1);
        assert_eq!(a.transcript.len(), 1);
        assert_eq!(a.transcript[0].outcome, TurnOutcome::Ok { rows: 1 });
        let n = a.result.rows[0][0].as_ref().expect("bound count");
        assert!(n.to_string().contains('2'), "two people, got {n}");
    }

    #[test]
    fn ask_repairs_a_parse_error_once() {
        let g = tiny_graph();
        // First completion has a syntax error (unclosed brace); the repair prompt
        // must carry the failed query + parser error; second completion is fixed.
        let nlq =
            Nlq::new(
                &g,
                Box::new(FnLlm(|prompt| {
                    if prompt.contains("Your previous query failed") {
                        assert!(prompt.contains("SELECT (COUNT(?p) AS ?n) WHERE { ?p"));
                        Ok(format!("```sparql\n{COUNT_PEOPLE}\n```"))
                    } else {
                        Ok("```sparql\nSELECT (COUNT(?p) AS ?n) WHERE { ?p rdf:type ex:Person\n```"
                        .to_string())
                    }
                })),
            );
        let a = nlq
            .ask("How many people are there?")
            .expect("repair succeeds");
        assert_eq!(a.repairs, 1);
        assert_eq!(a.transcript.len(), 2);
        assert!(matches!(
            a.transcript[0].outcome,
            TurnOutcome::ParseError(_)
        ));
        assert_eq!(a.transcript[1].outcome, TurnOutcome::Ok { rows: 1 });
    }

    #[test]
    fn ask_gives_up_after_the_repair_budget() {
        let g = tiny_graph();
        let nlq = Nlq::new(&g, Box::new(FnLlm(|_| Ok("no query here, sorry".into()))));
        let err = nlq.ask("How many people are there?").unwrap_err();
        assert!(err
            .message
            .contains("no valid query after 1 repair round(s)"));
        assert_eq!(err.transcript.len(), 2); // initial + one repair, both NoSparql
        assert!(err
            .transcript
            .iter()
            .all(|t| t.outcome == TurnOutcome::NoSparql));
    }

    #[test]
    fn record_then_replay_round_trips() {
        let g = tiny_graph();
        let recorder = RecordingLlm::new(FnLlm(|_| Ok(format!("```sparql\n{COUNT_PEOPLE}\n```"))));
        // Drive the recorder directly with the loop's own (public) prompt builder.
        let probe = Nlq::new(&g, Box::new(FnLlm(|_| Err("unused".into()))));
        let prompt = probe.prompt_for("How many people are there?");
        recorder.complete(&prompt).unwrap();
        let json = recorder.to_json();

        // ...replay it through the full loop.
        let replay = ReplayLlm::from_json(&json).unwrap();
        assert_eq!(replay.len(), 1);
        let nlq = Nlq::new(&g, Box::new(replay));
        let a = nlq
            .ask("How many people are there?")
            .expect("replay run succeeds");
        assert_eq!(a.result.len(), 1);

        // A different question is a replay MISS with a diagnosable error.
        let replay = ReplayLlm::from_json(&json).unwrap();
        let nlq = Nlq::new(&g, Box::new(replay));
        let err = nlq.ask("Something never recorded?").unwrap_err();
        assert!(err.message.contains("replay miss"), "{}", err.message);
    }

    #[test]
    fn budget_trips_surface_as_exec_errors() {
        let g = tiny_graph();
        let cfg = NlqConfig {
            max_rows: Some(1),
            max_repair_rounds: 0,
            ..NlqConfig::default()
        };
        let nlq = Nlq::with_config(
            &g,
            Box::new(FnLlm(|_| {
                Ok("```sparql\nSELECT ?s ?p ?o WHERE { ?s ?p ?o }\n```".into())
            })),
            cfg,
        );
        let err = nlq.ask("Everything, please").unwrap_err();
        assert!(
            matches!(&err.transcript[0].outcome, TurnOutcome::ExecError(e) if e.contains("budget")),
            "expected a budget trip, got {:?}",
            err.transcript[0].outcome
        );
    }

    /// N2 (`sq-9yjp`): an ungrounded predicate is a repair signal, and the repair
    /// prompt carries the dictionary feedback + suggestion. The model fixes it and the
    /// loop succeeds — the dictionary constraint paid for itself.
    #[test]
    fn ungrounded_predicate_triggers_dictionary_repair() {
        let g = tiny_graph();
        // First completion uses `ex:know` (not in the dictionary); the repair prompt
        // must surface the dictionary message + the `ex:knows` suggestion; second
        // completion is grounded.
        let cfg = NlqConfig {
            check_dictionary: true,
            ..NlqConfig::default()
        };
        let nlq = Nlq::with_config(
            &g,
            Box::new(FnLlm(|prompt| {
                if prompt.contains("not in the dataset's dictionary") {
                    assert!(prompt.contains("http://example.org/knows")); // suggestion
                    Ok("```sparql\nPREFIX ex: <http://example.org/>\n\
                        SELECT ?s WHERE { ?s ex:knows ?o }\n```"
                        .to_string())
                } else {
                    Ok("```sparql\nPREFIX ex: <http://example.org/>\n\
                        SELECT ?s WHERE { ?s ex:know ?o }\n```"
                        .to_string())
                }
            })),
            cfg,
        );
        let a = nlq.ask("Who knows whom?").expect("repair succeeds");
        assert_eq!(a.repairs, 1);
        assert_eq!(a.transcript.len(), 2);
        assert!(
            matches!(&a.transcript[0].outcome, TurnOutcome::UngroundedTerms(u)
                if u.len() == 1 && u[0].iri == "http://example.org/know"),
            "got {:?}",
            a.transcript[0].outcome
        );
        assert_eq!(a.transcript[1].outcome, TurnOutcome::Ok { rows: 1 });
    }

    /// Disabling the constraint accepts the (valid-but-empty) query as-is — the
    /// pre-N2 behaviour stays available.
    #[test]
    fn check_dictionary_false_accepts_ungrounded_query() {
        let g = tiny_graph();
        let cfg = NlqConfig {
            check_dictionary: false,
            max_repair_rounds: 0,
            ..NlqConfig::default()
        };
        let nlq = Nlq::with_config(
            &g,
            Box::new(FnLlm(|_| {
                Ok("```sparql\nPREFIX ex: <http://example.org/>\n\
                    SELECT ?s WHERE { ?s ex:know ?o }\n```"
                    .to_string())
            })),
            cfg,
        );
        let a = nlq
            .ask("Who knows whom?")
            .expect("accepted without the dict check");
        assert_eq!(a.repairs, 0);
        assert_eq!(a.result.len(), 0); // ungrounded predicate → empty, but accepted
    }
}
