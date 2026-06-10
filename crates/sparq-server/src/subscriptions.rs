//! SEPA-style SPARQL subscriptions over WebSocket (T23).
//!
//! Spec lineage: the **SEPA** W3C member submission ("SPARQL 1.1 Subscribe Language",
//! <https://www.w3.org/submissions/2018/SUBM-sparql11-subscribe-20181016/>) — a client
//! registers a SELECT query and is notified with **added/removed bindings diffs**
//! whenever a committed update changes the query's result. We keep SEPA's notification
//! *shape* (both diffs are SPARQL JSON results objects) but speak plain JSON over a
//! plain WebSocket — no `sparql-se` URI scheme, no separate subscribe HTTP endpoint.
//! See `crates/sparq-server/SUBSCRIPTIONS.md` for the full protocol and divergences.
//!
//! # Architecture: re-evaluate + diff per committed update
//!
//! * **Commit hook** — [`crate::http::AppState::apply_update`] bumps a
//!   `tokio::sync::watch` "commit generation" *after* the atomic graph swap. Every
//!   `/subscriptions` connection task holds a `watch::Receiver`.
//! * **Re-evaluation** — when the generation changes, the connection task re-runs each of
//!   its active SELECTs against a fresh [`crate::http::AppState::snapshot`] on the blocking pool
//!   (`spawn_blocking`), under the server's [`sparq_engine::QueryBudget`] (query timeout +
//!   `--max-results` row cap), then diffs against the stored previous result.
//! * **Diff** — each solution row is canonicalised to its SPARQL-JSON binding object
//!   (variables sorted by `serde_json`'s map ordering) and the serialised string is the
//!   row's identity key in a `HashMap<key, binding>`. Added = keys in the new result only;
//!   removed = keys in the old result only. Set semantics: duplicate rows collapse to one
//!   key (documented divergence — SELECT is a bag, the diff is over distinct bindings).
//! * **Coalescing (dirty-flag pattern)** — the `watch` channel stores only the latest
//!   generation, and `Receiver::changed()` resolves once when the seen value is stale, no
//!   matter how many commits produced it. If commits land *while* a re-evaluation is
//!   running, the generation advances again, so the loop's next `changed()` fires
//!   immediately and one more re-evaluation — against the latest snapshot — covers the
//!   whole burst. Notifications are therefore per *re-evaluation*, not per commit, and a
//!   client may observe several commits as a single combined diff. Because the generation
//!   is bumped after the swap, a snapshot taken after `changed()` resolves is always at
//!   least as new as the commit that woke us; an "early" snapshot only leads to a later
//!   empty diff, which is suppressed.
//! * **Limits** — per-connection cap (`--max-subscriptions-per-conn`, default 16) checked
//!   against the connection's own table; global cap (`--max-subscriptions`, default 256)
//!   via an atomic acquire/release counter with a panic-safe release on connection drop.
//!   An initial evaluation that overflows `--max-results` (or times out) *refuses* the
//!   subscription, mirroring the HTTP 413/503 semantics; a later re-evaluation failure
//!   terminates the subscription with an `error` message.
//!
//! Known limitation: the engine relabels blank nodes between evaluations, so a result row
//! containing a blank node can show up as a remove+add pair across commits even when it is
//! semantically unchanged.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::Response;
use serde_json::{json, Value};

use sparq_core::Graph;
use sparq_engine::QueryBudget;

use crate::exec::{prepare, PrepareError, QueryForm};
use crate::http::{make_budget, AppState, ServerConfig, TIMEOUT_GRACE};

// ---------------------------------------------------------------------------
// Global bookkeeping
// ---------------------------------------------------------------------------

/// Server-global subscription bookkeeping, shared by every `/subscriptions` connection
/// through [`AppState`].
#[derive(Default)]
pub(crate) struct SubscriptionCounters {
    /// Active subscriptions across the whole server (bounded by `--max-subscriptions`).
    active: AtomicUsize,
    /// Monotonic subscription-id allocator (ids are unique across the server's lifetime).
    next_id: AtomicU64,
}

/// The slots a single connection holds against the global cap. Slots are acquired on
/// `subscribe` and released on `unsubscribe`/termination; `Drop` releases whatever is
/// still held, so a socket that disconnects (or a handler that panics) can never leak
/// global capacity.
struct ConnSlots {
    counters: Arc<SubscriptionCounters>,
    held: usize,
}

impl ConnSlots {
    fn new(counters: Arc<SubscriptionCounters>) -> Self {
        Self { counters, held: 0 }
    }

    /// Tries to take one global slot (CAS loop so concurrent connections never overshoot).
    fn try_acquire(&mut self, max_global: usize) -> bool {
        let ok = self
            .counters
            .active
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| (n < max_global).then(|| n + 1))
            .is_ok();
        if ok {
            self.held += 1;
        }
        ok
    }

    fn release_one(&mut self) {
        debug_assert!(self.held > 0);
        self.held -= 1;
        self.counters.active.fetch_sub(1, Ordering::AcqRel);
    }
}

impl Drop for ConnSlots {
    fn drop(&mut self) {
        if self.held > 0 {
            self.counters.active.fetch_sub(self.held, Ordering::AcqRel);
        }
    }
}

// ---------------------------------------------------------------------------
// WebSocket endpoint
// ---------------------------------------------------------------------------

/// `GET /subscriptions` — upgrades to the subscription WebSocket. Incoming text frames
/// are capped at the server's `--max-body-bytes` (same guard as HTTP request bodies).
pub async fn subscriptions_endpoint(State(state): State<AppState>, ws: WebSocketUpgrade) -> Response {
    let max_msg = state.config().max_body_bytes;
    ws.max_message_size(max_msg).on_upgrade(move |socket| handle_socket(socket, state))
}

/// One active subscription: the query, its identity, and the last seen result keyed by
/// canonical row encoding (the diff baseline).
struct Subscription {
    alias: Option<String>,
    query: String,
    /// Per-subscription notification sequence; 0 is the initial full result.
    sequence: u64,
    vars: Vec<String>,
    /// canonical row key (serialised binding object) → binding object.
    rows: HashMap<String, Value>,
}

/// The per-connection driver: a single task owns the socket and its subscription table,
/// `select!`-ing between client messages and the commit-generation watch channel. While a
/// re-evaluation batch runs, incoming frames simply queue on the socket — `unsubscribe`
/// is processed right after the batch.
async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let mut commits = state.subscribe_commits();
    // Drop any generations committed before this connection existed: the initial
    // notification (sent on subscribe) already reflects the current graph.
    commits.mark_unchanged();
    let mut subs: HashMap<u64, Subscription> = HashMap::new();
    let mut slots = ConnSlots::new(state.subs.clone());

    loop {
        tokio::select! {
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if !handle_client_message(&mut socket, &state, &mut subs, &mut slots, &text).await {
                            break;
                        }
                    }
                    Some(Ok(Message::Binary(_))) => {
                        if send(&mut socket, &error_msg("binary frames are not part of the protocol; send JSON text frames", None, None)).await.is_err() {
                            break;
                        }
                    }
                    // axum answers Ping automatically; Pong needs no action.
                    Some(Ok(Message::Ping(_) | Message::Pong(_))) => {}
                    Some(Ok(Message::Close(_))) | Some(Err(_)) | None => break,
                }
            }
            changed = commits.changed() => {
                if changed.is_err() {
                    break; // server shutting down
                }
                if !reevaluate_all(&mut socket, &state, &mut subs, &mut slots).await {
                    break;
                }
            }
        }
    }
    // `slots` drops here, releasing the connection's global subscription slots.
}

/// Handles one client text frame. Returns `false` when the socket is gone.
async fn handle_client_message(
    socket: &mut WebSocket,
    state: &AppState,
    subs: &mut HashMap<u64, Subscription>,
    slots: &mut ConnSlots,
    text: &str,
) -> bool {
    let parsed: Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(e) => {
            return send(socket, &error_msg(&format!("message is not valid JSON: {e}"), None, None))
                .await
                .is_ok();
        }
    };
    if let Some(req) = parsed.get("subscribe") {
        return handle_subscribe(socket, state, subs, slots, req).await;
    }
    if let Some(req) = parsed.get("unsubscribe") {
        return handle_unsubscribe(socket, subs, slots, req).await;
    }
    send(
        socket,
        &error_msg("unknown message; expected {\"subscribe\": {...}} or {\"unsubscribe\": {...}}", None, None),
    )
    .await
    .is_ok()
}

async fn handle_subscribe(
    socket: &mut WebSocket,
    state: &AppState,
    subs: &mut HashMap<u64, Subscription>,
    slots: &mut ConnSlots,
    req: &Value,
) -> bool {
    let alias = req.get("alias").and_then(Value::as_str).map(str::to_string);
    let refuse = |msg: String| error_msg(&msg, None, alias.as_deref());

    let Some(query) = req.get("query").and_then(Value::as_str) else {
        return send(socket, &refuse("subscribe requires a string 'query' field".into())).await.is_ok();
    };
    // Only SELECT is subscribable: the diff is defined over solution bindings.
    match prepare(query) {
        Ok(p) if p.form == QueryForm::Select => {}
        Ok(_) => {
            return send(socket, &refuse("only SELECT queries can be subscribed".into())).await.is_ok();
        }
        Err(PrepareError::Malformed(e)) => {
            return send(socket, &refuse(format!("malformed query: {e}"))).await.is_ok();
        }
    }

    // Limits: per-connection first (cheap), then the global slot.
    let config = state.config();
    if subs.len() >= config.max_subscriptions_per_conn {
        let max = config.max_subscriptions_per_conn;
        return send(socket, &refuse(format!("subscription limit reached for this connection ({max}); unsubscribe first or raise --max-subscriptions-per-conn"))).await.is_ok();
    }
    if !slots.try_acquire(config.max_subscriptions) {
        let max = config.max_subscriptions;
        return send(socket, &refuse(format!("server-wide subscription limit reached ({max}); retry later or raise --max-subscriptions"))).await.is_ok();
    }

    // Initial evaluation. Failure (parse-at-engine, timeout, max-rows overflow) refuses
    // the subscription — mirroring the HTTP endpoint's 400/503/413 semantics.
    let (vars, rows) = match evaluate(state, query.to_string()).await {
        Ok(r) => r,
        Err(e) => {
            slots.release_one();
            return send(socket, &refuse(format!("initial evaluation failed: {e}"))).await.is_ok();
        }
    };

    let id = state.subs.next_id.fetch_add(1, Ordering::Relaxed) + 1;
    let sub = Subscription { alias, query: query.to_string(), sequence: 0, vars, rows };

    // {"subscribed": ...} then the sequence-0 notification: full result as added, empty removed.
    let subscribed = with_alias(json!({ "subscribed": { "id": id } }), "subscribed", sub.alias.as_deref());
    if send(socket, &subscribed).await.is_err() {
        return false;
    }
    let added: Vec<&Value> = sorted_bindings(&sub.rows);
    let initial = notification(id, sub.alias.as_deref(), 0, &sub.vars, &added, &[]);
    if send(socket, &initial).await.is_err() {
        return false;
    }
    subs.insert(id, sub);
    true
}

async fn handle_unsubscribe(
    socket: &mut WebSocket,
    subs: &mut HashMap<u64, Subscription>,
    slots: &mut ConnSlots,
    req: &Value,
) -> bool {
    let Some(id) = req.get("id").and_then(Value::as_u64) else {
        return send(socket, &error_msg("unsubscribe requires a numeric 'id' field", None, None)).await.is_ok();
    };
    if subs.remove(&id).is_some() {
        slots.release_one();
        send(socket, &json!({ "unsubscribed": { "id": id } })).await.is_ok()
    } else {
        send(socket, &error_msg("no active subscription with that id on this connection", Some(id), None))
            .await
            .is_ok()
    }
}

/// Re-evaluates every subscription on this connection against the latest snapshot and
/// pushes non-empty diffs. A failed re-evaluation (timeout / max-rows overflow after the
/// data grew) terminates that subscription with an `error` message. Returns `false` when
/// the socket is gone.
async fn reevaluate_all(
    socket: &mut WebSocket,
    state: &AppState,
    subs: &mut HashMap<u64, Subscription>,
    slots: &mut ConnSlots,
) -> bool {
    let mut ids: Vec<u64> = subs.keys().copied().collect();
    ids.sort_unstable();
    for id in ids {
        let query = subs[&id].query.clone();
        match evaluate(state, query).await {
            Ok((vars, new_rows)) => {
                let sub = subs.get_mut(&id).expect("subscription vanished mid-batch");
                let added: Vec<&Value> =
                    sorted_pairs(new_rows.iter().filter(|(k, _)| !sub.rows.contains_key(*k)));
                let removed: Vec<&Value> =
                    sorted_pairs(sub.rows.iter().filter(|(k, _)| !new_rows.contains_key(*k)));
                if added.is_empty() && removed.is_empty() {
                    continue; // result unchanged — no notification (coalesced no-ops included)
                }
                sub.sequence += 1;
                let msg = notification(id, sub.alias.as_deref(), sub.sequence, &vars, &added, &removed);
                if send(socket, &msg).await.is_err() {
                    return false;
                }
                sub.vars = vars;
                sub.rows = new_rows;
            }
            Err(e) => {
                let alias = subs[&id].alias.clone();
                subs.remove(&id);
                slots.release_one();
                let msg = error_msg(
                    &format!("re-evaluation failed, subscription terminated: {e}"),
                    Some(id),
                    alias.as_deref(),
                );
                if send(socket, &msg).await.is_err() {
                    return false;
                }
            }
        }
    }
    true
}

// ---------------------------------------------------------------------------
// Evaluation + diff primitives
// ---------------------------------------------------------------------------

/// Runs the SELECT on the blocking pool under the server budget (deadline + row cap),
/// with the same hard await-cap as the HTTP path. Returns the head vars and the result
/// keyed by canonical row encoding.
async fn evaluate(state: &AppState, query: String) -> Result<(Vec<String>, HashMap<String, Value>), String> {
    let config = state.config().clone();
    let budget = make_budget(&config, true);
    let graph = state.snapshot();
    let task = tokio::task::spawn_blocking(move || eval_rows(&graph, &query, &budget));
    let joined = match config.query_timeout {
        Some(t) => tokio::time::timeout(t + TIMEOUT_GRACE, task)
            .await
            .map_err(|_| budget_error("query budget exceeded (timeout)", &config))?,
        None => task.await,
    };
    match joined {
        Ok(Ok(r)) => Ok(r),
        Ok(Err(e)) => Err(budget_error(&e, &config)),
        Err(_) => Err("query worker panicked".into()),
    }
}

/// Maps the engine's budget-violation strings onto the messages the HTTP layer uses
/// (so a refused subscription names the same limits as a 503/413 would).
fn budget_error(e: &str, config: &ServerConfig) -> String {
    if e.contains("query budget exceeded (timeout)") {
        let secs = config.query_timeout.map(|t| t.as_secs()).unwrap_or(0);
        return format!("query timed out (server limit: {secs}s)");
    }
    if e.contains("query budget exceeded (max-rows)") {
        let max = config.max_results.unwrap_or(0);
        return format!("result exceeds the server's max-results limit ({max} rows)");
    }
    e.to_string()
}

/// Evaluates the SELECT and canonicalises each solution row to its SPARQL-JSON binding
/// object; the serialised object is the row's identity key. `serde_json`'s map keeps a
/// deterministic key order, so equal rows always serialise identically. Duplicate rows
/// collapse (set semantics — see module docs).
fn eval_rows(graph: &Graph, query: &str, budget: &QueryBudget) -> Result<(Vec<String>, HashMap<String, Value>), String> {
    let r = sparq_engine::query_with_budget(graph, query, budget)?;
    let vars: Vec<String> = r.vars.iter().map(|v| v.as_str().to_string()).collect();
    let mut rows = HashMap::with_capacity(r.rows.len());
    for row in &r.rows {
        let mut obj = serde_json::Map::new();
        for (vi, term) in row.iter().enumerate() {
            if let Some(t) = term {
                obj.insert(vars[vi].clone(), term_json(t));
            }
        }
        let binding = Value::Object(obj);
        rows.insert(binding.to_string(), binding);
    }
    Ok((vars, rows))
}

/// SPARQL 1.1 JSON results encoding of an RDF term — same conventions as the engine's
/// `query_json` (no `datatype` member for plain `xsd:string`; the SPARQL 1.2
/// `{"type":"triple"}` encoding for RDF 1.2 triple terms).
fn term_json(t: &oxrdf::Term) -> Value {
    match t {
        oxrdf::Term::NamedNode(n) => json!({ "type": "uri", "value": n.as_str() }),
        oxrdf::Term::BlankNode(b) => json!({ "type": "bnode", "value": b.as_str() }),
        oxrdf::Term::Literal(l) => {
            let mut obj = serde_json::Map::new();
            obj.insert("type".into(), "literal".into());
            obj.insert("value".into(), l.value().into());
            if let Some(lang) = l.language() {
                obj.insert("xml:lang".into(), lang.into());
            } else {
                let dt = l.datatype();
                if dt.as_str() != "http://www.w3.org/2001/XMLSchema#string" {
                    obj.insert("datatype".into(), dt.as_str().into());
                }
            }
            Value::Object(obj)
        }
        oxrdf::Term::Triple(t) => {
            let subject = match &t.subject {
                oxrdf::NamedOrBlankNode::NamedNode(n) => term_json(&oxrdf::Term::NamedNode(n.clone())),
                oxrdf::NamedOrBlankNode::BlankNode(b) => term_json(&oxrdf::Term::BlankNode(b.clone())),
            };
            json!({
                "type": "triple",
                "value": {
                    "subject": subject,
                    "predicate": term_json(&oxrdf::Term::NamedNode(t.predicate.clone())),
                    "object": term_json(&t.object),
                }
            })
        }
    }
}

/// All bindings of a stored result, in deterministic (key) order.
fn sorted_bindings(rows: &HashMap<String, Value>) -> Vec<&Value> {
    sorted_pairs(rows.iter())
}

/// Sorts `(key, binding)` pairs by canonical key so notification output is deterministic
/// (`HashMap` iteration order is randomised).
fn sorted_pairs<'a>(iter: impl Iterator<Item = (&'a String, &'a Value)>) -> Vec<&'a Value> {
    let mut pairs: Vec<(&String, &Value)> = iter.collect();
    pairs.sort_unstable_by_key(|(k, _)| *k);
    pairs.into_iter().map(|(_, v)| v).collect()
}

// ---------------------------------------------------------------------------
// Message builders
// ---------------------------------------------------------------------------

/// A SPARQL JSON results object (`{"head":{"vars":[…]},"results":{"bindings":[…]}}`).
fn results_json(vars: &[String], bindings: &[&Value]) -> Value {
    json!({ "head": { "vars": vars }, "results": { "bindings": bindings } })
}

/// The SEPA-shaped notification: added/removed are both full SPARQL JSON results objects.
fn notification(id: u64, alias: Option<&str>, sequence: u64, vars: &[String], added: &[&Value], removed: &[&Value]) -> Value {
    let body = json!({
        "notification": {
            "id": id,
            "sequence": sequence,
            "addedResults": results_json(vars, added),
            "removedResults": results_json(vars, removed),
        }
    });
    with_alias(body, "notification", alias)
}

/// Echoes the client's alias inside the named envelope, when one was given (SEPA echoes
/// the alias so clients can correlate without tracking ids).
fn with_alias(mut msg: Value, envelope: &str, alias: Option<&str>) -> Value {
    if let Some(alias) = alias {
        msg[envelope]["alias"] = alias.into();
    }
    msg
}

fn error_msg(message: &str, id: Option<u64>, alias: Option<&str>) -> Value {
    let mut inner = serde_json::Map::new();
    inner.insert("message".into(), message.into());
    if let Some(id) = id {
        inner.insert("id".into(), id.into());
    }
    if let Some(alias) = alias {
        inner.insert("alias".into(), alias.into());
    }
    json!({ "error": Value::Object(inner) })
}

async fn send(socket: &mut WebSocket, msg: &Value) -> Result<(), axum::Error> {
    socket.send(Message::Text(msg.to_string())).await
}

// ---------------------------------------------------------------------------
// Unit tests (pure pieces; the protocol end-to-end lives in tests/subscriptions.rs)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn graph(ttl: &str) -> Graph {
        Graph::load_str(ttl, "turtle").unwrap()
    }

    #[test]
    fn eval_rows_canonical_keys_are_stable_across_evaluations() {
        let g = graph("@prefix ex: <http://ex/> . ex:a ex:p ex:b . ex:c ex:p \"x\"@en .");
        let q = "SELECT ?s ?o WHERE { ?s <http://ex/p> ?o }";
        let (vars1, rows1) = eval_rows(&g, q, &QueryBudget::unlimited()).unwrap();
        let (vars2, rows2) = eval_rows(&g, q, &QueryBudget::unlimited()).unwrap();
        assert_eq!(vars1, vec!["s", "o"]);
        assert_eq!(vars1, vars2);
        assert_eq!(rows1.keys().collect::<std::collections::BTreeSet<_>>(), rows2.keys().collect());
        assert_eq!(rows1.len(), 2);
    }

    #[test]
    fn diff_detects_added_and_removed() {
        let before = graph("@prefix ex: <http://ex/> . ex:a ex:p ex:b . ex:c ex:p ex:d .");
        let after = graph("@prefix ex: <http://ex/> . ex:a ex:p ex:b . ex:e ex:p ex:f .");
        let q = "SELECT ?s WHERE { ?s <http://ex/p> ?o }";
        let (_, old) = eval_rows(&before, q, &QueryBudget::unlimited()).unwrap();
        let (_, new) = eval_rows(&after, q, &QueryBudget::unlimited()).unwrap();
        let added = sorted_pairs(new.iter().filter(|(k, _)| !old.contains_key(*k)));
        let removed = sorted_pairs(old.iter().filter(|(k, _)| !new.contains_key(*k)));
        assert_eq!(added.len(), 1);
        assert_eq!(added[0]["s"]["value"], "http://ex/e");
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0]["s"]["value"], "http://ex/c");
    }

    #[test]
    fn term_json_matches_sparql_json_conventions() {
        use oxrdf::{Literal, NamedNode, Term};
        let uri = term_json(&Term::NamedNode(NamedNode::new("http://ex/a").unwrap()));
        assert_eq!(uri, json!({"type": "uri", "value": "http://ex/a"}));
        // Plain xsd:string carries no datatype member.
        let plain = term_json(&Term::Literal(Literal::new_simple_literal("hi")));
        assert_eq!(plain, json!({"type": "literal", "value": "hi"}));
        let lang = term_json(&Term::Literal(Literal::new_language_tagged_literal("hi", "en").unwrap()));
        assert_eq!(lang, json!({"type": "literal", "value": "hi", "xml:lang": "en"}));
        let typed = term_json(&Term::Literal(Literal::new_typed_literal(
            "1",
            NamedNode::new("http://www.w3.org/2001/XMLSchema#integer").unwrap(),
        )));
        assert_eq!(typed["datatype"], "http://www.w3.org/2001/XMLSchema#integer");
    }

    #[test]
    fn unbound_variables_are_omitted_from_the_binding() {
        let g = graph("@prefix ex: <http://ex/> . ex:a ex:p ex:b .");
        let q = "SELECT ?s ?missing WHERE { ?s <http://ex/p> ?o OPTIONAL { ?s <http://ex/q> ?missing } }";
        let (vars, rows) = eval_rows(&g, q, &QueryBudget::unlimited()).unwrap();
        assert_eq!(vars, vec!["s", "missing"]);
        let binding = rows.values().next().unwrap();
        assert!(binding.get("missing").is_none());
        assert_eq!(binding["s"]["type"], "uri");
    }

    #[test]
    fn max_rows_budget_refuses_oversized_results() {
        let g = graph("@prefix ex: <http://ex/> . ex:a ex:p ex:b . ex:c ex:p ex:d . ex:e ex:p ex:f .");
        let budget = QueryBudget { deadline: None, max_rows: Some(2) };
        let err = eval_rows(&g, "SELECT ?s WHERE { ?s ?p ?o }", &budget).unwrap_err();
        assert!(err.contains("max-rows"), "unexpected error: {err}");
    }

    #[test]
    fn notification_shape_is_sepa_like() {
        let vars = vec!["s".to_string()];
        let b = json!({"s": {"type": "uri", "value": "http://ex/a"}});
        let msg = notification(7, Some("watch"), 3, &vars, &[&b], &[]);
        assert_eq!(msg["notification"]["id"], 7);
        assert_eq!(msg["notification"]["alias"], "watch");
        assert_eq!(msg["notification"]["sequence"], 3);
        assert_eq!(msg["notification"]["addedResults"]["head"]["vars"], json!(["s"]));
        assert_eq!(msg["notification"]["addedResults"]["results"]["bindings"][0], b);
        assert_eq!(msg["notification"]["removedResults"]["results"]["bindings"], json!([]));
    }
}
