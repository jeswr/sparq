//! [OPUS-4.8] sq-jxjgr: the server-side natural-language `ask` tool — behind the opt-in
//! `nlq` feature, OFF by default.
//!
//! This is the second of the two complementary grounding tools chosen on the 2026-06-23
//! design call (`shapes` is the no-LLM structured one): instead of handing the client a
//! shape to ground its OWN model, the `ask` tool runs the whole NL→SPARQL loop
//! **server-side** via [`sparq_nlq`] — introspect → generate → validate → execute — and
//! returns the executed SPARQL plus the result rows (and in-graph citations, when the
//! underlying graph carries provenance).
//!
//! ## Honest framing (the load-bearing contract)
//!
//! - **It embeds a configurable LLM call.** Cost, latency, and answer quality depend
//!   entirely on the model/endpoint the operator configures — this crate ships **no
//!   default model** and never phones home. The structured `shapes` / `introspect` tools
//!   are the no-LLM default; this trades a model call for the convenience of not writing
//!   SPARQL.
//! - **It is NOT a token-saving claim.** The project measured representation/token tricks
//!   as duds; this tool is an *ergonomics/grounding aid* pending measurement, nothing more.
//! - **It degrades cleanly.** With the `nlq` feature ON but **no** backend configured, the
//!   tool is not advertised and a direct call returns a clear *"not configured"* error —
//!   **never** a fabricated answer, never a panic.
//! - **The answer is grounded in real result rows.** What comes back is the model's
//!   *executed* query and the rows that query actually produced — not a free-form prose
//!   paragraph the model could hallucinate. The loop validates (spargebra) and executes
//!   (the engine, under a [`QueryBudget`]) before any answer is returned.
//!
//! ## Backends
//!
//! The `nlq` feature enables BOTH of sparq-nlq's configurable backends; [`ask`] picks
//! whichever the environment configures, preferring the OpenAI-compatible endpoint:
//! - `SPARQ_NLQ_ENDPOINT_URL` + `SPARQ_NLQ_ENDPOINT_MODEL` (+ optional
//!   `SPARQ_NLQ_ENDPOINT_KEY`) → a provider-agnostic OpenAI-compatible chat endpoint
//!   ([`sparq_nlq::endpoint::EndpointLlm`]);
//! - else `ANTHROPIC_API_KEY` → the Anthropic Messages API
//!   ([`sparq_nlq::live::AnthropicLlm`]).
//!
//! No test makes a live network call: the real loop logic is exercised in [`run_ask`]
//! with a scripted in-process [`sparq_nlq::Llm`] stub, and the "not configured" degrade
//! path is tested directly.

use serde_json::{json, Value};

use sparq_core::Graph;
use sparq_engine::QueryBudget;
use sparq_nlq::{Answer, Llm, Nlq, NlqConfig};

/// Names the backend the environment configures, if any. Mirrors the resolution order in
/// `resolve_backend` without constructing a client (so it is cheap enough to call from
/// `tools/list`). Returns `false` when neither backend is configured — the signal that the
/// `ask` tool must NOT be advertised and a direct call must fail closed.
pub fn backend_configured() -> bool {
    endpoint_configured() || anthropic_configured()
}

/// Whether the OpenAI-compatible endpoint backend is configured (URL + model both set).
fn endpoint_configured() -> bool {
    !env_empty(sparq_nlq::endpoint::ENV_BASE_URL) && !env_empty(sparq_nlq::endpoint::ENV_MODEL)
}

/// Whether the Anthropic backend is configured (`ANTHROPIC_API_KEY` set non-empty).
fn anthropic_configured() -> bool {
    !env_empty("ANTHROPIC_API_KEY")
}

/// `true` when an environment variable is unset or empty.
fn env_empty(var: &str) -> bool {
    std::env::var(var).map(|v| v.is_empty()).unwrap_or(true)
}

/// Construct the configured LLM backend, or an actionable "not configured" error. Endpoint
/// is preferred over Anthropic when both are set (the productized, provider-agnostic path).
fn resolve_backend() -> Result<Box<dyn Llm>, String> {
    if endpoint_configured() {
        let llm = sparq_nlq::endpoint::EndpointLlm::from_env()?;
        return Ok(Box::new(llm));
    }
    if anthropic_configured() {
        let llm = sparq_nlq::live::AnthropicLlm::from_env()?;
        return Ok(Box::new(llm));
    }
    Err(NOT_CONFIGURED.to_string())
}

/// The exact "not configured" message — degrade cleanly, never fabricate. Lists the
/// environment variables that select a backend so the operator can fix it.
const NOT_CONFIGURED: &str = "the `ask` tool is not configured: no LLM backend is set. \
    Configure one of:\n  - an OpenAI-compatible endpoint: SPARQ_NLQ_ENDPOINT_URL + \
    SPARQ_NLQ_ENDPOINT_MODEL (+ optional SPARQ_NLQ_ENDPOINT_KEY); or\n  - the Anthropic \
    Messages API: ANTHROPIC_API_KEY.\nThe server ships no default model. Until one is \
    configured, use the no-LLM `shapes` / `introspect` tools and write the query yourself.";

/// Run the `ask` tool: resolve the configured backend, run the NL→SPARQL loop, and render
/// the answer. Fails closed (the "not configured" error) when no backend is set — never a
/// fabricated answer, never a panic. The real loop logic lives in [`run_ask`] so it is
/// testable with a scripted backend (no network).
pub fn ask(graph: &Graph, question: &str, budget: &QueryBudget) -> Result<String, String> {
    let llm = resolve_backend()?;
    run_ask(graph, question, budget, llm)
}

/// The backend-agnostic core: run the sparq-nlq loop over `graph` with the supplied `llm`
/// and render the answer. Split out from [`ask`] so the loop is exercised in tests with an
/// in-process scripted backend (no network), and so the backend-resolution and loop
/// concerns stay separate.
pub fn run_ask(
    graph: &Graph,
    question: &str,
    budget: &QueryBudget,
    llm: Box<dyn Llm>,
) -> Result<String, String> {
    let config = config_from_budget(budget);
    let nlq = Nlq::with_config(graph, llm, config);
    match nlq.ask(question) {
        Ok(answer) => Ok(render_answer(graph, question, &answer)),
        // An NL-loop failure (the model produced no valid query, or the LLM call failed)
        // is a TOOL error string, not a panic — the caller wraps it as `isError: true`.
        Err(e) => Err(format!("could not answer the question: {}", e)),
    }
}

/// Map the MCP [`QueryBudget`] onto a [`NlqConfig`] so the `ask` tool inherits the
/// server's row cap and per-query deadline.
fn config_from_budget(budget: &QueryBudget) -> NlqConfig {
    let mut config = NlqConfig {
        max_rows: budget.max_rows,
        ..NlqConfig::default()
    };
    // Translate the budget's absolute deadline into a per-query duration (the loop builds
    // a fresh deadline at execution time). Use the remaining time; fall back to the
    // NlqConfig default if the deadline is already in the past (the loop will then trip
    // its own budget promptly).
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(deadline) = budget.deadline {
        let now = std::time::Instant::now();
        if deadline > now {
            config.exec_timeout = Some(deadline - now);
        }
    }
    config
}

/// Render the answer as a structured JSON object: the executed SPARQL, the result rows
/// (faithfully, never paraphrased into prose the model could distort), the repair count,
/// and — with `nlq/citations` (the `sparq-nlq/citations` feature) — in-graph citations.
///
/// `_graph` is the graph the answer was produced over; it is only consulted when the
/// `citations` feature is on (to resolve in-graph provenance).
fn render_answer(_graph: &Graph, question: &str, answer: &Answer) -> String {
    let result = json!({
        "question": question,
        "sparql": answer.sparql,
        "repairs": answer.repairs,
        "result": render_result(answer),
        "note": "Answer is grounded in the executed query's real result rows, not a \
                 generated paragraph. The `sparql` field is the exact query that ran.",
    });
    let result = with_citations(_graph, answer, result);
    serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string())
}

/// Render the [`sparq_nlq::QueryResult`] as `{ vars, rows }`: each row is a map from
/// variable name to the bound term's N-Triples form (`null` when unbound). ASK results
/// (zero vars, one empty row iff true) render as `{ "boolean": true|false }`.
fn render_result(answer: &Answer) -> Value {
    let r = &answer.result;
    if r.vars.is_empty() {
        // ASK encoding: one empty row iff true.
        return json!({ "boolean": !r.rows.is_empty() });
    }
    let var_names: Vec<String> = r.vars.iter().map(|v| v.as_str().to_string()).collect();
    let rows: Vec<Value> = r
        .rows
        .iter()
        .map(|row| {
            let mut obj = serde_json::Map::new();
            for (i, cell) in row.iter().enumerate() {
                let key = var_names.get(i).cloned().unwrap_or_else(|| i.to_string());
                let val = match cell {
                    Some(term) => Value::String(term.to_string()),
                    None => Value::Null,
                };
                obj.insert(key, val);
            }
            Value::Object(obj)
        })
        .collect();
    json!({ "vars": var_names, "rows": rows, "row_count": rows.len() })
}

/// Attach in-graph citations to the rendered answer when the `sparq-nlq/citations` feature
/// is enabled; a no-op (returns `result` unchanged) otherwise.
#[cfg(feature = "citations")]
fn with_citations(graph: &Graph, answer: &Answer, mut result: Value) -> Value {
    let cited = answer.citations(graph);
    // Render the human-facing footnote block; the renderer emits "no source recorded" for
    // un-sourced bindings (never a guessed source), so this stays honest.
    if let Some(obj) = result.as_object_mut() {
        obj.insert("citations".to_string(), json!(cited.footnotes()));
    }
    result
}

/// Citations are off when `sparq-nlq/citations` is not enabled — return the answer as-is.
#[cfg(not(feature = "citations"))]
fn with_citations(_graph: &Graph, _answer: &Answer, result: Value) -> Value {
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use sparq_engine::QueryBudget;

    /// Closure-backed test backend — the real `Llm` trait, no network.
    struct FnLlm<F: Fn(&str) -> Result<String, String>>(F);
    impl<F: Fn(&str) -> Result<String, String>> Llm for FnLlm<F> {
        fn complete(&self, prompt: &str) -> Result<String, String> {
            (self.0)(prompt)
        }
    }

    const TTL: &str = r#"@prefix ex: <http://ex/> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
ex:alice rdf:type ex:Person ; ex:name "Alice" .
ex:bob   rdf:type ex:Person ; ex:name "Bob" .
"#;

    fn graph() -> Graph {
        Graph::load_str(TTL, "turtle").expect("load turtle")
    }

    const COUNT_QUERY: &str =
        "PREFIX ex: <http://ex/> SELECT (COUNT(?s) AS ?n) WHERE { ?s a ex:Person }";

    #[test]
    fn run_ask_runs_the_real_loop_and_renders_the_executed_query() {
        // A scripted backend that returns a valid query — the REAL loop (validate +
        // execute via the engine) runs over the real graph; no mock bypasses it.
        let llm = Box::new(FnLlm(|_| Ok(format!("```sparql\n{}\n```", COUNT_QUERY))));
        let out = run_ask(&graph(), "how many people?", &QueryBudget::unlimited(), llm)
            .expect("the loop should answer");
        let v: Value = serde_json::from_str(&out).expect("ask output is JSON");
        // The executed query is reported verbatim.
        assert_eq!(v["sparql"], COUNT_QUERY);
        // The answer is grounded in the REAL result row (2 people).
        let rows = v["result"]["rows"].as_array().unwrap();
        assert_eq!(rows.len(), 1);
        let count_cell = rows[0].as_object().unwrap().values().next().unwrap();
        assert!(
            count_cell.as_str().unwrap().contains('2'),
            "the count must be the real engine result (2): {count_cell}"
        );
    }

    #[test]
    fn run_ask_surfaces_a_failed_loop_as_a_tool_error_not_a_panic() {
        // A backend that never produces SPARQL — the loop exhausts its repairs and errors.
        let llm = Box::new(FnLlm(|_| Ok("sorry, I cannot help".to_string())));
        let err = run_ask(&graph(), "anything", &QueryBudget::unlimited(), llm)
            .expect_err("a no-SPARQL loop must error, not panic");
        assert!(err.contains("could not answer"), "{err}");
    }

    #[test]
    fn ask_with_no_backend_configured_fails_closed() {
        // Snapshot + clear every backend-selecting env var so the test is hermetic
        // regardless of the host environment, then restore.
        let vars = [
            sparq_nlq::endpoint::ENV_BASE_URL,
            sparq_nlq::endpoint::ENV_MODEL,
            sparq_nlq::endpoint::ENV_API_KEY,
            "ANTHROPIC_API_KEY",
        ];
        let saved: Vec<(&str, Option<String>)> =
            vars.iter().map(|v| (*v, std::env::var(v).ok())).collect();
        for v in vars {
            std::env::remove_var(v);
        }

        let result = ask(&graph(), "how many people?", &QueryBudget::unlimited());

        // Restore BEFORE asserting so a failed assert does not leak env state.
        for (v, val) in saved {
            match val {
                Some(val) => std::env::set_var(v, val),
                None => std::env::remove_var(v),
            }
        }

        let err = result.expect_err("no backend ⇒ a clean 'not configured' error");
        assert!(err.contains("not configured"), "{err}");
        assert!(
            err.contains("SPARQ_NLQ_ENDPOINT_URL") && err.contains("ANTHROPIC_API_KEY"),
            "the error must name both backends so the operator can fix it: {err}"
        );
        // Crucially: it is an ERROR, NOT a fabricated answer.
        assert!(
            !err.contains("\"sparql\""),
            "an unconfigured ask must NEVER return a (fabricated) answer: {err}"
        );
    }

    #[test]
    fn ask_result_renders_a_real_count() {
        // End-to-end render shape over the real engine result.
        let llm = Box::new(FnLlm(|_| Ok(format!("```sparql\n{}\n```", COUNT_QUERY))));
        let out = run_ask(&graph(), "count", &QueryBudget::unlimited(), llm).unwrap();
        let v: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["question"], "count");
        assert_eq!(v["repairs"], 0);
        assert!(v["result"]["vars"].is_array());
    }
}
