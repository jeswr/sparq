//! SPARQL 1.1 Federated Query — `SERVICE` evaluation. [OPUS-4.8]
//!
//! `SERVICE <endpoint> { pattern }` evaluates `pattern` against a *remote* SPARQL
//! endpoint and joins the returned solutions back into the surrounding query, per
//! <https://www.w3.org/TR/sparql11-federated-query/>.
//!
//! ## How it works
//!
//! 1. The inner [`GraphPattern`] is wrapped as `SELECT * WHERE { <inner> }` using
//!    spargebra's `Display` impl (which round-trips algebra → SPARQL syntax), so the
//!    full pattern (BGPs, OPTIONAL, FILTER, sub-SELECT, …) is forwarded.
//!
//!    **Bind-join (`VALUES` pushdown).** When the SERVICE is the right side of a join
//!    (or `OPTIONAL`) whose join variables are already bound by the left side, the
//!    caller (`exec::try_bound_join_service`) pushes a *block* of those bindings into
//!    the wrapped query as a `VALUES` clause — `SELECT * WHERE { VALUES (?j…) { … }
//!    <inner> }` — so the remote returns only the rows that can survive the local join
//!    (the brTPF/FedX "bound join", bead sq-sjkj). This is ON by default and
//!    correctness-preserving; it FALLS BACK to forwarding the bare pattern verbatim
//!    when it does not apply (variable endpoint, no bound join var, a join key bound to
//!    a blank node, …). The block size is the one OPT-IN tuning knob
//!    ([`with_service_bound_join_block_size`] / `SPARQ_SERVICE_BIND_BLOCK`,
//!    default [`DEFAULT_BIND_BLOCK`]). The remote relation that is NOT bound-joined is
//!    still materialised and joined locally by the caller.
//! 2. The query is sent over HTTP (form-encoded POST, `Accept:
//!    application/sparql-results+json, application/sparql-results+xml;q=0.9` — JSON is
//!    preferred but XML is accepted as a fallback).
//! 3. The response is parsed into a [`ServiceRelation`] (variable list + rows of optional
//!    [`Term`]s). The body is content-sniffed (`parse_results`): a leading `{` is parsed
//!    as SPARQL-Results-JSON ([`parse_srj`]); a leading `<` as SPARQL-Results-XML
//!    ([`parse_srx`]). The XML path matters because some endpoints ignore `Accept` and
//!    always return XML — without it the whole SERVICE call would fail (bead sq-ycu).
//! 4. The caller (`exec::eval_service`) interns those terms into the local/graph
//!    dictionaries — exactly like `VALUES` — and joins them with the rest of the query.
//!
//! ## `SERVICE SILENT`
//!
//! Any error (DNS, connection, non-2xx status, malformed body) is swallowed when the
//! pattern is `SILENT`: evaluation yields the join identity (a single empty solution),
//! so the surrounding query keeps its existing bindings rather than failing. Without
//! `SILENT`, the error propagates and fails the query.
//!
//! ## Transport seam (testability)
//!
//! The HTTP call sits behind the [`Transport`] trait. Production uses [`HttpTransport`]
//! (ureq, a tiny blocking client — gated off wasm). Tests inject a canned-response or
//! local-loopback transport, so the SRJ parser and the algebra integration are
//! exercised without a public network dependency.
//!
//! ## Out of scope
//!
//! * `SERVICE ?var` (a *variable* endpoint): the endpoint IRI is only known once the
//!   surrounding bindings are produced, which requires a per-solution remote call. We
//!   reject it with a clear error (or, under `SILENT`, the empty result) rather than
//!   silently mis-evaluating — see [`eval_service`] in `exec.rs`.

use oxrdf::{BlankNode, Literal, NamedNode, NamedOrBlankNode, Term, Triple, Variable};

/// A materialised remote SELECT result: the projected variables and one row per
/// solution (`None` = the variable is unbound in that solution).
#[derive(Debug)]
pub(crate) struct ServiceRelation {
    pub vars: Vec<Variable>,
    pub rows: Vec<Vec<Option<Term>>>,
}

/// Abstracts the HTTP round-trip so tests can inject a fake endpoint. `query` is the
/// SPARQL query string; the return is the raw response body (expected to be
/// SPARQL-Results-JSON) or a transport error string.
pub(crate) trait Transport {
    fn fetch(&self, endpoint: &str, query: &str) -> Result<String, String>;
}

/// Evaluate one SERVICE call end-to-end: send `query` to `endpoint` via `transport`
/// and parse the response into a [`ServiceRelation`]. SILENT handling is the caller's
/// responsibility (it owns the join-identity fallback).
pub(crate) fn eval_remote(
    transport: &dyn Transport,
    endpoint: &str,
    query: &str,
) -> Result<ServiceRelation, String> {
    let body = transport.fetch(endpoint, query)?;
    parse_results(&body)
}

/// Parse a remote SELECT results document, content-sniffing JSON vs XML. [OPUS-4.8]
///
/// The SPARQL Protocol lets a client advertise an `Accept` preference, but a server MAY
/// ignore it; in practice some endpoints always emit SPARQL-Results-XML even when we ask
/// for JSON (bead sq-ycu). We therefore sniff the first non-whitespace byte rather than
/// trusting any `Content-Type` (which the `Transport` seam does not even surface): `{` ⇒
/// SPARQL-Results-JSON, `<` ⇒ SPARQL-Results-XML. Anything else is an error (or, under
/// `SILENT`, the caller's empty result).
#[cfg(feature = "service")]
pub(crate) fn parse_results(text: &str) -> Result<ServiceRelation, String> {
    match text.trim_start().as_bytes().first() {
        Some(b'<') => parse_srx(text),
        Some(b'{') => parse_srj(text),
        // An empty body or a leading byte that is neither `{` nor `<` is not a results
        // document we can parse; report it (SILENT turns this into an empty result).
        _ => Err(
            "SERVICE: endpoint response is neither SPARQL-Results-JSON nor -XML \
             (expected a leading '{' or '<')"
                .into(),
        ),
    }
}

// ---------------------------------------------------------------------------
// Bind-join (VALUES pushdown) into SERVICE — block size knob [OPUS-4.8] (sq-sjkj)
// ---------------------------------------------------------------------------
//
// When a SERVICE sub-query is the right side of a join whose join variables are
// already bound by the left side, we push a *block* of those bindings into the
// remote query as a `VALUES` clause (SPARQL 1.1 §10.2.1) — one remote request per
// block instead of materialising the whole remote relation and joining locally.
// This is the brTPF/FedX "bound join" (bead sq-sjkj, research candidate C1): for a
// selective join it slashes the data the endpoint returns and the rows we filter.
//
// The pushdown is a *correctness-preserving* optimisation of the existing SERVICE
// path: injecting `VALUES (?j) { (v1) (v2) … }` restricts the remote pattern to
// exactly the bound tuples, and the surrounding query joins the result back the
// same way it joins the unbound relation — so the answer is identical. It is ON by
// default (no new surface, zero downside on the applicable shape) and FALLS BACK to
// the verbatim path whenever it does not apply (variable endpoint, no bound join
// var, a join key bound to a blank node, an empty left side, …). The only TUNING
// KNOB — the block size — is OPT-IN via [`with_service_bound_join_block_size`] /
// the `SPARQ_SERVICE_BIND_BLOCK` env var; the default suits typical workloads.

/// Default bind-join block size: how many distinct binding tuples are pushed into
/// one remote `VALUES` request. ~50 mirrors FedX's default bound-join batch — large
/// enough to amortise the per-request round-trip, small enough to keep the injected
/// query (and the remote's VALUES-join fan-out) bounded.
pub(crate) const DEFAULT_BIND_BLOCK: usize = 50;

#[cfg(feature = "service")]
mod bind_block {
    use std::cell::Cell;

    thread_local! {
        // `None` => use the env / built-in default. A scope installs an override.
        static OVERRIDE: Cell<Option<usize>> = const { Cell::new(None) };
    }

    /// RAII override of the bind-join block size for the current scope.
    pub(crate) struct Guard(Option<usize>);
    impl Drop for Guard {
        fn drop(&mut self) {
            OVERRIDE.with(|o| o.set(self.0.take()));
        }
    }

    pub(crate) fn install(n: usize) -> Guard {
        // A zero block size would mean "never batch" — clamp to 1 so a tuple still
        // gets pushed one-per-request rather than silently disabling correctness.
        let n = n.max(1);
        Guard(OVERRIDE.with(|o| o.replace(Some(n))))
    }

    /// The active block size: an installed scope override wins; otherwise the
    /// `SPARQ_SERVICE_BIND_BLOCK` env var (parsed once is unnecessary — this is off
    /// the hot path, called once per bound-join); otherwise the built-in default.
    pub(crate) fn current() -> usize {
        if let Some(n) = OVERRIDE.with(|o| o.get()) {
            return n;
        }
        #[cfg(not(target_arch = "wasm32"))]
        if let Ok(s) = std::env::var("SPARQ_SERVICE_BIND_BLOCK") {
            if let Ok(n) = s.trim().parse::<usize>() {
                if n >= 1 {
                    return n;
                }
            }
        }
        super::DEFAULT_BIND_BLOCK
    }
}

/// The bind-join block size in force for the current scope. [OPUS-4.8] (sq-sjkj)
#[cfg(feature = "service")]
pub(crate) fn bind_block_size() -> usize {
    bind_block::current()
}

/// Runs `f` with `n` as the SERVICE bind-join block size (how many distinct binding
/// tuples are pushed into one remote `VALUES` request). OPT-IN tuning knob for the
/// bound-join pushdown (bead sq-sjkj): the default (~50, or `SPARQ_SERVICE_BIND_BLOCK`)
/// suits typical workloads, but a very selective or very fan-out join can benefit
/// from a larger or smaller block. `n` is clamped to at least 1. The override is
/// thread-local and restored on return/unwind, mirroring [`with_service_egress_allow`].
///
/// This knob does NOT change results — it only trades remote-request count against
/// per-request size; the bound-join is correctness-preserving at any block size.
///
/// ```no_run
/// # #[cfg(feature = "service")] {
/// sparq_engine::with_service_bound_join_block_size(200, || {
///     // ... run a federated query with large bound-join blocks
/// });
/// # }
/// ```
#[cfg(feature = "service")]
pub fn with_service_bound_join_block_size<R>(n: usize, f: impl FnOnce() -> R) -> R {
    let _guard = bind_block::install(n);
    f()
}

/// Renders a `VALUES` block that binds `vars` to each tuple in `tuples`, in the
/// SPARQL 1.1 syntax accepted inside a group graph pattern. [OPUS-4.8] (sq-sjkj)
///
/// Each term is emitted via its canonical N-Triples/Turtle form (oxrdf `Display`),
/// which is valid SPARQL term syntax for IRIs and literals (the only kinds the
/// caller pushes — see [`pushable_term`]). Single-variable blocks use the short
/// `VALUES ?v { v1 v2 }` form; multi-variable blocks use the parenthesised
/// `VALUES (?a ?b) { (a1 b1) (a2 b2) }` form. `UNDEF` is never emitted: the caller
/// only pushes fully-bound tuples (a tuple with an unbound join var falls back to
/// the verbatim path), so every cell is a concrete term.
#[cfg(feature = "service")]
pub(crate) fn render_values_block(vars: &[Variable], tuples: &[Vec<Term>]) -> String {
    use std::fmt::Write as _;
    let mut s = String::new();
    if vars.len() == 1 {
        let _ = write!(s, "VALUES {} {{", vars[0]);
        for t in tuples {
            let _ = write!(s, " {}", t[0]);
        }
        s.push_str(" }");
    } else {
        s.push_str("VALUES (");
        for (i, v) in vars.iter().enumerate() {
            if i > 0 {
                s.push(' ');
            }
            let _ = write!(s, "{v}");
        }
        s.push_str(") {");
        for tuple in tuples {
            s.push_str(" (");
            for (i, t) in tuple.iter().enumerate() {
                if i > 0 {
                    s.push(' ');
                }
                let _ = write!(s, "{t}");
            }
            s.push(')');
        }
        s.push_str(" }");
    }
    s
}

/// Whether a `Term` may be pushed as a `VALUES` data value into a remote query.
/// [OPUS-4.8] (sq-sjkj)
///
/// IRIs and literals are pushable (their `Display` is valid SPARQL term syntax and
/// their identity is global). A **blank node** is NOT: blank-node labels are scoped
/// to a single result document, so a local bnode label is meaningless to the remote
/// endpoint — pushing it would silently change semantics. A **triple term** (RDF 1.2
/// `<<( … )>>`) is conservatively excluded too: not every endpoint accepts it in
/// VALUES, and a join key is rarely a quoted triple. When a join-key tuple contains
/// any non-pushable term the caller abandons the bound-join for the verbatim path,
/// preserving exact semantics.
#[cfg(feature = "service")]
pub(crate) fn pushable_term(t: &Term) -> bool {
    matches!(t, Term::NamedNode(_) | Term::Literal(_))
}

// ---------------------------------------------------------------------------
// SPARQL Results JSON parser
// (https://www.w3.org/TR/sparql11-results-json/)
// ---------------------------------------------------------------------------

/// Parse a SELECT result document. ASK results (`{"boolean": …}`) are reported as an
/// error here — `SERVICE { … }` always wraps a SELECT in our forwarding, so a boolean
/// body indicates a misbehaving endpoint.
#[cfg(feature = "service")]
pub(crate) fn parse_srj(text: &str) -> Result<ServiceRelation, String> {
    let v: serde_json::Value =
        serde_json::from_str(text).map_err(|e| format!("SERVICE: invalid results JSON: {e}"))?;
    if v.get("boolean").is_some() {
        return Err("SERVICE: endpoint returned an ASK boolean, expected SELECT bindings".into());
    }
    let vars: Vec<Variable> = v
        .pointer("/head/vars")
        .and_then(|a| a.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|s| s.as_str())
                .map(|s| Variable::new(s).map_err(|e| format!("SERVICE: bad result variable {s:?}: {e}")))
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?
        .ok_or_else(|| "SERVICE: results JSON missing head.vars".to_string())?;

    let mut rows: Vec<Vec<Option<Term>>> = Vec::new();
    for sol in v
        .pointer("/results/bindings")
        .and_then(|a| a.as_array())
        .ok_or_else(|| "SERVICE: results JSON missing results.bindings".to_string())?
    {
        let obj = sol
            .as_object()
            .ok_or_else(|| "SERVICE: a solution binding is not a JSON object".to_string())?;
        // Build a row positionally over `vars`; a variable absent from the solution
        // object is UNBOUND (`None`) — identical to the VALUES UNDEF semantics.
        let mut row: Vec<Option<Term>> = Vec::with_capacity(vars.len());
        for var in &vars {
            match obj.get(var.as_str()) {
                Some(cell) => row.push(Some(srj_term(cell)?)),
                None => row.push(None),
            }
        }
        rows.push(row);
    }
    Ok(ServiceRelation { vars, rows })
}

/// Reconstruct one term from an SRJ binding value object. Mirrors the conformance
/// suite's `srj_term` (uri / bnode / literal / SPARQL-1.2 triple terms).
#[cfg(feature = "service")]
fn srj_term(val: &serde_json::Value) -> Result<Term, String> {
    let get = |k: &str| val.get(k).and_then(|s| s.as_str());
    match get("type") {
        Some("uri") => {
            let iri = get("value").unwrap_or_default();
            Ok(Term::NamedNode(
                NamedNode::new(iri).map_err(|e| format!("SERVICE: bad IRI {iri:?}: {e}"))?,
            ))
        }
        Some("bnode") => {
            let id = get("value").unwrap_or_default();
            Ok(Term::BlankNode(
                BlankNode::new(id).map_err(|e| format!("SERVICE: bad bnode {id:?}: {e}"))?,
            ))
        }
        // Both "literal" and the legacy "typed-literal" (pre-2013 endpoints) map here.
        Some("literal") | Some("typed-literal") | None => {
            let value = get("value")
                .ok_or_else(|| "SERVICE: literal binding without value".to_string())?
                .to_string();
            if let Some(lang) = get("xml:lang") {
                // [OPUS-4.8] sq-s955: the INBOUND counterpart to the outbound `its:dir`
                // emission (sq-bj7o, `json.rs::term_to_json`). A remote endpoint's SPARQL 1.2
                // results carry the RDF 1.2 base direction as a SEPARATE `its:dir` field
                // alongside the bare `xml:lang` tag; reading only `xml:lang` dropped the
                // direction, so a `dirLangString` arrived as a plain language-tagged literal
                // and silently lost its direction on the way in. We now reconstruct the
                // directional literal, validating the direction through the single source of
                // truth (`dict::parse_base_direction`, also used by the stored-slot,
                // materialised and outbound paths). An ABSENT or INVALID `its:dir` degrades
                // to a plain language-tagged literal — the SAME decision `split_lang_dir` /
                // `reconstruct_ref` make for a malformed stored slot — so all four paths AGREE
                // on `(lang, dir)`.
                match get("its:dir").and_then(sparq_core::dict::parse_base_direction) {
                    Some(dir) => Ok(Term::Literal(
                        Literal::new_directional_language_tagged_literal(value, lang, dir).map_err(
                            |e| format!("SERVICE: bad language tag {lang:?}: {e}"),
                        )?,
                    )),
                    None => Ok(Term::Literal(
                        Literal::new_language_tagged_literal(value, lang)
                            .map_err(|e| format!("SERVICE: bad language tag {lang:?}: {e}"))?,
                    )),
                }
            } else if let Some(dt) = get("datatype") {
                let dt = NamedNode::new(dt).map_err(|e| format!("SERVICE: bad datatype {dt:?}: {e}"))?;
                Ok(Term::Literal(Literal::new_typed_literal(value, dt)))
            } else {
                Ok(Term::Literal(Literal::new_simple_literal(value)))
            }
        }
        Some("triple") => {
            let v = val
                .get("value")
                .ok_or_else(|| "SERVICE: triple term without value".to_string())?;
            let part = |k: &str| -> Result<Term, String> {
                srj_term(v.get(k).ok_or_else(|| format!("SERVICE: triple term without {k}"))?)
            };
            let subject = match part("subject")? {
                Term::NamedNode(n) => NamedOrBlankNode::NamedNode(n),
                Term::BlankNode(b) => NamedOrBlankNode::BlankNode(b),
                other => return Err(format!("SERVICE: invalid triple-term subject: {other}")),
            };
            let predicate = match part("predicate")? {
                Term::NamedNode(n) => n,
                other => return Err(format!("SERVICE: invalid triple-term predicate: {other}")),
            };
            Ok(Term::Triple(Box::new(Triple {
                subject,
                predicate,
                object: part("object")?,
            })))
        }
        Some(other) => Err(format!("SERVICE: unknown binding type {other:?}")),
    }
}

// ---------------------------------------------------------------------------
// SPARQL-Results-XML (SRX) parsing [OPUS-4.8] (bead sq-ycu)
// ---------------------------------------------------------------------------
//
// The JSON path (`parse_srj`) is the preferred format, but some endpoints ignore the
// `Accept` header and only ever emit SPARQL-Results-XML
// (<https://www.w3.org/TR/rdf-sparql-XMLres/>, with the SPARQL 1.2 `<triple>` extension).
// This parser is the streaming-event analogue of the conformance suite's `parse_srx`
// (`sparq-conformance/src/results.rs`) — same quick-xml event handling, same predefined-
// entity decode discipline — but it (a) takes a `&str` body rather than a file path, and
// (b) projects each `<result>` *positionally* over the declared `<variable>` list (an
// absent binding ⇒ `None`), so it yields a `ServiceRelation` byte-for-byte compatible with
// `parse_srj`. An ASK `<boolean>` body is rejected, exactly like `parse_srj`, because
// SERVICE always wraps a SELECT.

/// Resolve one XML entity reference name (as quick-xml 0.40 hands it out in a
/// `Event::GeneralRef`, i.e. WITHOUT the surrounding `&`/`;`) to its replacement text:
/// the five predefined named entities (`amp`/`lt`/`gt`/`quot`/`apos`) and numeric
/// character references (`#38` decimal, `#x26` hex). [OPUS-4.8]
#[cfg(feature = "service")]
fn resolve_xml_entity(name: &str) -> Result<String, String> {
    if let Some(rep) = quick_xml::escape::resolve_predefined_entity(name) {
        return Ok(rep.to_string());
    }
    if let Some(rest) = name.strip_prefix('#') {
        let cp = if let Some(hex) = rest.strip_prefix(['x', 'X']) {
            u32::from_str_radix(hex, 16)
        } else {
            rest.parse::<u32>()
        }
        .map_err(|_| format!("SERVICE: bad numeric character reference &{name};"))?;
        return char::from_u32(cp)
            .map(|c| c.to_string())
            .ok_or_else(|| format!("SERVICE: numeric character reference &{name}; out of range"));
    }
    Err(format!("SERVICE: unknown XML entity &{name};"))
}

/// Reconstruct one term from an SRX value element's collected attributes/text. Mirrors the
/// conformance suite's `make_term` and the SRJ `srj_term` (uri / bnode / plain, language-
/// tagged — incl. the SPARQL 1.2 `its:dir` directional literal — and typed literals).
#[cfg(feature = "service")]
fn srx_term(
    kind: &str,
    lang: Option<String>,
    dir: Option<String>,
    dt: Option<String>,
    text: String,
) -> Result<Term, String> {
    match kind {
        "uri" => Ok(Term::NamedNode(
            NamedNode::new(&text).map_err(|e| format!("SERVICE: bad IRI {text:?}: {e}"))?,
        )),
        "bnode" => Ok(Term::BlankNode(
            BlankNode::new(&text).map_err(|e| format!("SERVICE: bad bnode {text:?}: {e}"))?,
        )),
        // "literal" (and any other leaf, defensively).
        _ => {
            if let Some(lang) = lang {
                // A `<literal xml:lang="…" its:dir="…">` is an RDF 1.2 dirLangString. As in
                // the SRJ path (sq-s955), an ABSENT or INVALID direction degrades to a plain
                // language-tagged literal so all parse paths agree on `(lang, dir)`.
                match dir
                    .as_deref()
                    .and_then(sparq_core::dict::parse_base_direction)
                {
                    Some(direction) => Ok(Term::Literal(
                        Literal::new_directional_language_tagged_literal(text, &lang, direction)
                            .map_err(|e| format!("SERVICE: bad language tag {lang:?}: {e}"))?,
                    )),
                    None => Ok(Term::Literal(
                        Literal::new_language_tagged_literal(text, &lang)
                            .map_err(|e| format!("SERVICE: bad language tag {lang:?}: {e}"))?,
                    )),
                }
            } else if let Some(dt) = dt {
                let dt =
                    NamedNode::new(&dt).map_err(|e| format!("SERVICE: bad datatype {dt:?}: {e}"))?;
                Ok(Term::Literal(Literal::new_typed_literal(text, dt)))
            } else {
                Ok(Term::Literal(Literal::new_simple_literal(text)))
            }
        }
    }
}

/// Parse a SPARQL-Results-XML SELECT document into a [`ServiceRelation`]. ASK `<boolean>`
/// bodies are rejected (SERVICE always wraps a SELECT in our forwarding).
#[cfg(feature = "service")]
pub(crate) fn parse_srx(text: &str) -> Result<ServiceRelation, String> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(text);
    reader.config_mut().trim_text(false);

    let mut vars: Vec<Variable> = Vec::new();
    // Per-row map of variable-name → term; projected positionally over `vars` at </result>.
    let mut cur_row: rustc_hash::FxHashMap<String, Term> = rustc_hash::FxHashMap::default();
    let mut rows: Vec<Vec<Option<Term>>> = Vec::new();
    let mut cur_var: Option<String> = None;
    // The open value element: (kind, xml:lang, its:dir, datatype, text).
    #[allow(clippy::type_complexity)]
    let mut cur_val: Option<(String, Option<String>, Option<String>, Option<String>, String)> =
        None;
    // SPARQL 1.2 `<triple>` nesting: each frame is (active slot, [s, p, o]).
    let mut triple_stack: Vec<(usize, [Option<Term>; 3])> = Vec::new();
    let mut in_boolean = false;
    let mut boolean: Option<bool> = None;

    // Route a finished term into the enclosing `<triple>` frame's active slot, or, at top
    // level, into the current row keyed by the open `<binding name=…>`.
    fn commit(
        term: Term,
        triple_stack: &mut [(usize, [Option<Term>; 3])],
        cur_row: &mut rustc_hash::FxHashMap<String, Term>,
        cur_var: &Option<String>,
    ) {
        if let Some((slot, parts)) = triple_stack.last_mut() {
            parts[*slot] = Some(term);
        } else if let Some(var) = cur_var.clone() {
            cur_row.insert(var, term);
        }
    }
    fn set_slot(triple_stack: &mut [(usize, [Option<Term>; 3])], slot: usize) {
        if let Some((s, _)) = triple_stack.last_mut() {
            *s = slot;
        }
    }

    loop {
        match reader
            .read_event()
            .map_err(|e| format!("SERVICE: invalid results XML: {e}"))?
        {
            Event::Eof => break,
            ev @ (Event::Start(_) | Event::Empty(_)) => {
                let is_empty = matches!(ev, Event::Empty(_));
                let e = match &ev {
                    Event::Start(e) | Event::Empty(e) => e,
                    _ => unreachable!(),
                };
                let name = e.local_name();
                let name = std::str::from_utf8(name.as_ref()).unwrap_or("").to_string();
                let attr = |key: &str| -> Option<String> {
                    e.attributes().filter_map(|a| a.ok()).find_map(|a| {
                        let k = std::str::from_utf8(a.key.as_ref()).ok()?;
                        // Matches a bare key or any namespaced `*:key` (e.g. `xml:lang`).
                        if k == key || k.ends_with(&format!(":{key}")) {
                            quick_xml::escape::unescape(std::str::from_utf8(&a.value).ok()?)
                                .ok()
                                .map(std::borrow::Cow::into_owned)
                        } else {
                            None
                        }
                    })
                };
                match name.as_str() {
                    "variable" => {
                        if let Some(v) = attr("name") {
                            vars.push(Variable::new(&v).map_err(|e| {
                                format!("SERVICE: bad result variable {v:?}: {e}")
                            })?);
                        }
                    }
                    "result" => cur_row.clear(),
                    "binding" => cur_var = attr("name"),
                    "uri" | "bnode" => cur_val = Some((name, None, None, None, String::new())),
                    "literal" => {
                        cur_val = Some((
                            name,
                            attr("lang"),
                            attr("dir"),
                            attr("datatype"),
                            String::new(),
                        ))
                    }
                    "triple" => triple_stack.push((0, [None, None, None])),
                    "subject" => set_slot(&mut triple_stack, 0),
                    "predicate" => set_slot(&mut triple_stack, 1),
                    "object" => set_slot(&mut triple_stack, 2),
                    "boolean" => in_boolean = true,
                    _ => {}
                }
                // Self-closing value elements (`<literal/>`, `<bnode/>`) get no End event:
                // commit the (empty-text) term right away.
                if is_empty {
                    if let Some((kind, lang, dir, dt, t)) = cur_val.take() {
                        commit(
                            srx_term(&kind, lang, dir, dt, t)?,
                            &mut triple_stack,
                            &mut cur_row,
                            &cur_var,
                        );
                    }
                }
            }
            Event::Text(t) => {
                // quick-xml 0.40 splits entity references out into `Event::GeneralRef`
                // (handled below), so the decoded text carries no `&…;` to unescape — it
                // is the verbatim character data.
                let s = t.decode().map_err(|e| e.to_string())?;
                if in_boolean {
                    boolean = Some(s.trim() == "true");
                } else if let Some(v) = cur_val.as_mut() {
                    v.4.push_str(&s);
                }
            }
            // A `&amp;` / `&lt;` / numeric `&#38;` / `&#x26;` reference inside the open
            // value element's text. (boolean bodies never carry references.) [OPUS-4.8]
            Event::GeneralRef(r) => {
                if let Some(v) = cur_val.as_mut() {
                    let name = r.decode().map_err(|e| e.to_string())?;
                    v.4.push_str(&resolve_xml_entity(&name)?);
                }
            }
            Event::CData(t) => {
                if let Some(v) = cur_val.as_mut() {
                    v.4.push_str(&String::from_utf8_lossy(&t));
                }
            }
            Event::End(e) => {
                let name = e.local_name();
                match name.as_ref() {
                    b"uri" | b"bnode" | b"literal" => {
                        if let Some((kind, lang, dir, dt, t)) = cur_val.take() {
                            commit(
                                srx_term(&kind, lang, dir, dt, t)?,
                                &mut triple_stack,
                                &mut cur_row,
                                &cur_var,
                            );
                        }
                    }
                    b"triple" => {
                        let Some((_, [s, p, o])) = triple_stack.pop() else {
                            return Err("SERVICE: stray </triple> in results XML".into());
                        };
                        let subject = match s {
                            Some(Term::NamedNode(n)) => NamedOrBlankNode::NamedNode(n),
                            Some(Term::BlankNode(b)) => NamedOrBlankNode::BlankNode(b),
                            other => {
                                return Err(format!(
                                    "SERVICE: invalid triple-term subject: {other:?}"
                                ))
                            }
                        };
                        let predicate = match p {
                            Some(Term::NamedNode(n)) => n,
                            other => {
                                return Err(format!(
                                    "SERVICE: invalid triple-term predicate: {other:?}"
                                ))
                            }
                        };
                        let object = o
                            .ok_or_else(|| "SERVICE: triple term without object".to_string())?;
                        commit(
                            Term::Triple(Box::new(Triple {
                                subject,
                                predicate,
                                object,
                            })),
                            &mut triple_stack,
                            &mut cur_row,
                            &cur_var,
                        );
                    }
                    b"binding" => cur_var = None,
                    b"result" => {
                        // Project the row positionally over the declared variables; an
                        // absent binding is UNBOUND (`None`) — the same VALUES-UNDEF
                        // semantics the SRJ path uses.
                        let row: Vec<Option<Term>> =
                            vars.iter().map(|v| cur_row.remove(v.as_str())).collect();
                        cur_row.clear();
                        rows.push(row);
                    }
                    b"boolean" => in_boolean = false,
                    _ => {}
                }
            }
            _ => {}
        }
    }

    if boolean.is_some() {
        return Err(
            "SERVICE: endpoint returned an ASK boolean, expected SELECT bindings".into(),
        );
    }
    Ok(ServiceRelation { vars, rows })
}

// ---------------------------------------------------------------------------
// SSRF egress policy (default-deny private / internal ranges) [OPUS-4.8]
// ---------------------------------------------------------------------------

/// [OPUS-4.8] (sq-iu0c) Stable marker substring embedded in EVERY engine error string
/// for a SERVICE egress refusal (a host blocked by the allowlist / default-deny SSRF
/// policy). It survives the transport-error wrapping in `HttpTransport::fetch`, so a
/// network-exposed host (e.g. `sparq-server`) can `contains()`-classify the refusal as a
/// **policy** decision — an honest `403`-style status — rather than a server-fault `500`.
/// This mirrors the existing `"query budget exceeded (timeout)"` → `503` marker pattern.
///
/// The marker is deliberately generic (it names no host) so it is safe to surface; the
/// host detail still travels in the surrounding (server-log-only) error text.
pub const SERVICE_EGRESS_REFUSED_MARKER: &str = "SERVICE egress refused";

//
// The `SERVICE` clause turns an attacker-controlled SPARQL string into an
// outbound HTTP request from the engine host (threat-model B4 / T-SERVICE-SSRF,
// bead sq-2v6f). With no egress filter that is a textbook SSRF primitive into
// the internal network — the worst case being the cloud-metadata endpoint
// 169.254.169.254, which on most clouds hands out credentials. The DEFAULT here
// is therefore DENY: a SERVICE endpoint that resolves to any non-global address
// is refused, and a deployer must explicitly opt a host/range back in via the
// allowlist (mirroring how `update.rs` gates `LOAD file://` behind
// `with_load_base`).
//
// DNS-rebinding safety: the check runs on the *resolved* IP, not the literal IRI
// host, and the production transport installs the policy as ureq's `Resolver`.
// ureq then connects only to the addresses the resolver returns, so the IP that
// is vetted is exactly the IP that is dialled — a hostile DNS answer that points
// at 127.0.0.1 / 169.254.169.254 is dropped before any socket is opened, and
// there is no resolve-then-reresolve TOCTOU window.

/// Classifies a resolved [`IpAddr`](std::net::IpAddr) as a forbidden (private /
/// internal / non-global) destination for SERVICE federation. Returns `true`
/// when the address is in a range the default-deny policy refuses.
///
/// Forbidden ranges:
/// * loopback — `127.0.0.0/8`, `::1`
/// * RFC1918 private — `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`
/// * link-local — `169.254.0.0/16` (incl. the `169.254.169.254` cloud-metadata
///   IP) and IPv6 `fe80::/10`
/// * unique-local IPv6 — `fc00::/7`
/// * unspecified — `0.0.0.0`, `::`
/// * IPv4-mapped IPv6 (`::ffff:a.b.c.d`) is unwrapped and re-classified as the
///   embedded IPv4 address, so a private v4 cannot be smuggled through a v6 host.
#[cfg(feature = "service")]
pub(crate) fn is_forbidden_ip(ip: std::net::IpAddr) -> bool {
    use std::net::IpAddr;
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()            // 127.0.0.0/8
                || v4.is_private()      // 10/8, 172.16/12, 192.168/16
                || v4.is_link_local()   // 169.254.0.0/16 (incl. 169.254.169.254)
                || v4.is_unspecified()  // 0.0.0.0
                // Defence-in-depth on ranges that are also internal but not
                // covered above: broadcast, shared CGNAT (100.64/10), benchmarking.
                || v4.is_broadcast()    // 255.255.255.255
                || matches!(v4.octets(), [100, b, ..] if (64..=127).contains(&b)) // 100.64/10 CGNAT
        }
        IpAddr::V6(v6) => {
            // Unwrap IPv4-mapped (::ffff:a.b.c.d) and re-check as IPv4 so a
            // private v4 can't ride in through a v6 literal.
            if let Some(v4) = v6.to_ipv4_mapped() {
                return is_forbidden_ip(IpAddr::V4(v4));
            }
            v6.is_loopback()            // ::1
                || v6.is_unspecified()  // ::
                || (v6.segments()[0] & 0xffc0) == 0xfe80 // fe80::/10 link-local
                || (v6.segments()[0] & 0xfe00) == 0xfc00 // fc00::/7 unique-local
        }
    }
}

/// SERVICE egress allowlist + policy mode. A host (DNS name or IP literal, exactly
/// as written in the SERVICE IRI authority) on this list is exempt from
/// [`is_forbidden_ip`] — its resolved addresses are permitted even when private.
///
/// Two modes (the [`Mode`] flag), both default-deny but at different strictnesses:
///
/// * **`Mode::DenyPrivate`** (the engine's standalone default, installed by
///   [`with_service_egress_allow`]): public IPs are reachable, private/internal IPs
///   are refused unless the host is on the allowlist. Allowlist entries only *add*
///   permission (re-open a private host).
/// * **`Mode::AllowlistOnly`** (the strict mode the *server* uses, installed by
///   [`with_service_egress_policy`]): ONLY hosts on the allowlist may be reached at
///   all — every other host is refused even if it resolves to a public address. An
///   empty allowlist in this mode is therefore "deny ALL SERVICE", which is the
///   server's safe default for the network-exposed surface.
///
/// Empty + `DenyPrivate` (the thread-local default before any scope installs a
/// policy) preserves the original behaviour: public allowed, private denied.
/// Installed for a scope via [`with_service_egress_allow`] /
/// [`with_service_egress_policy`], mirroring `update.rs`'s `with_load_base`
/// thread-local allowlist pattern. [OPUS-4.8] (sq-4w18)
#[cfg(feature = "service")]
pub(crate) mod egress_policy {
    use std::cell::RefCell;
    use std::collections::HashSet;

    /// How the allowlist is interpreted for hosts NOT on it. [OPUS-4.8] (sq-4w18)
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) enum Mode {
        /// Hosts off the allowlist are reachable iff they resolve to a *public*
        /// address (private/internal IPs are refused). The engine's standalone
        /// default and the semantics of [`super::with_service_egress_allow`].
        DenyPrivate,
        /// Hosts off the allowlist are refused unconditionally (even public IPs).
        /// The server installs this so federation is restricted to exactly the
        /// operator-listed hosts; an empty allowlist = deny ALL SERVICE.
        AllowlistOnly,
    }

    struct Policy {
        allow: HashSet<String>,
        mode: Mode,
    }

    thread_local! {
        static POLICY: RefCell<Policy> =
            RefCell::new(Policy { allow: HashSet::new(), mode: Mode::DenyPrivate });
    }

    /// Restores the previous policy when the installing scope returns (also on
    /// unwind, so a panicking SERVICE call never leaks a relaxed policy).
    pub(crate) struct Guard(Option<Policy>);
    impl Drop for Guard {
        fn drop(&mut self) {
            if let Some(prev) = self.0.take() {
                POLICY.with(|p| *p.borrow_mut() = prev);
            }
        }
    }

    /// Installs `hosts` (lower-cased) + `mode` as the active SERVICE egress policy
    /// for the duration of the returned guard; the previous policy is restored on
    /// drop.
    pub(crate) fn install(hosts: impl IntoIterator<Item = String>, mode: Mode) -> Guard {
        let allow: HashSet<String> = hosts.into_iter().map(|h| h.to_ascii_lowercase()).collect();
        let next = Policy { allow, mode };
        // Swap in the new policy and hand the previous one to the Guard for restore.
        POLICY.with(|p| Guard(Some(std::mem::replace(&mut *p.borrow_mut(), next))))
    }

    /// True if `host` (case-insensitive) is on the active allowlist. An entry is
    /// matched two ways: [OPUS-4.8] (sq-4w18)
    ///   * **exact** — the entry equals the host (`"sparql.example.org"`).
    ///   * **suffix wildcard** — an entry beginning with a dot (`".example.org"`)
    ///     matches any host ending in that suffix INCLUDING the bare apex
    ///     (`example.org`, `a.example.org`, `a.b.example.org`). This is the engine
    ///     representation of the server's `*.example.org` pattern. The leading-dot
    ///     boundary means `.example.org` does NOT match `notexample.org`.
    pub(crate) fn is_allowed(host: &str) -> bool {
        let h = host.to_ascii_lowercase();
        POLICY.with(|p| {
            let allow = &p.borrow().allow;
            if allow.contains(&h) {
                return true;
            }
            // Suffix-wildcard entries (".suffix"): match the apex and any subdomain.
            allow.iter().any(|e| {
                if let Some(suffix) = e.strip_prefix('.') {
                    h == suffix || h.ends_with(e.as_str())
                } else {
                    false
                }
            })
        })
    }

    /// The active policy mode.
    pub(crate) fn mode() -> Mode {
        POLICY.with(|p| p.borrow().mode)
    }
}

/// Runs `f` with `hosts` allowlisted for SERVICE federation: each host's resolved
/// addresses are permitted even if they fall in a private/internal range that the
/// default-deny SSRF policy would otherwise refuse. A host is matched
/// case-insensitively against the *authority* of the SERVICE IRI (DNS name or IP
/// literal, e.g. `"localhost"`, `"10.0.0.5"`, `"sparql.internal"`).
///
/// Without an installed allowlist, every SERVICE endpoint that resolves to a
/// loopback / RFC1918 / link-local / unique-local / unspecified address is
/// REJECTED — the secure default. This mirrors [`crate::with_load_base`], which
/// gates `LOAD file://` the same way. Only effective with the `service` feature.
///
/// ```no_run
/// # #[cfg(feature = "service")] {
/// // Permit federation to a trusted internal endpoint that resolves privately.
/// sparq_engine::with_service_egress_allow(["sparql.internal".to_string()], || {
///     // ... run a query containing `SERVICE <http://sparql.internal/> { ... }`
/// });
/// # }
/// ```
#[cfg(feature = "service")]
pub fn with_service_egress_allow<R>(
    hosts: impl IntoIterator<Item = String>,
    f: impl FnOnce() -> R,
) -> R {
    let _guard = egress_policy::install(hosts, egress_policy::Mode::DenyPrivate);
    f()
}

/// Runs `f` under a STRICT SERVICE egress policy: only the listed `hosts` may be
/// reached, and EVERY other host is refused — even one resolving to a public
/// address. This is the policy the network-exposed **server** installs (bead
/// sq-4w18): the SERVICE clause turns attacker-controlled SPARQL into outbound HTTP,
/// so federation is restricted to exactly the operator-configured endpoints.
///
/// `strict = false` is identical to [`with_service_egress_allow`] (default-deny
/// *private*: public hosts reachable, private/internal only if allowlisted). The
/// server wires this directly to its `--service-allow` config so the same call site
/// expresses both "no federation at all" (strict + empty list) and "an explicit
/// allowlist" (strict + hosts). Host matching is case-insensitive against the
/// SERVICE IRI *authority* (DNS name or IP literal), exactly like
/// [`with_service_egress_allow`].
///
/// ```no_run
/// # #[cfg(feature = "service")] {
/// // Restrict SERVICE to a single trusted endpoint; anything else is refused.
/// sparq_engine::with_service_egress_policy(true, ["sparql.example.org".to_string()], || {
///     // ... run a query that may contain `SERVICE <…> { ... }`
/// });
/// // Strict + empty list = federation fully disabled (deny ALL SERVICE).
/// sparq_engine::with_service_egress_policy(true, std::iter::empty(), || { /* ... */ });
/// # }
/// ```
#[cfg(feature = "service")]
pub fn with_service_egress_policy<R>(
    strict: bool,
    hosts: impl IntoIterator<Item = String>,
    f: impl FnOnce() -> R,
) -> R {
    let mode = if strict {
        egress_policy::Mode::AllowlistOnly
    } else {
        egress_policy::Mode::DenyPrivate
    };
    let _guard = egress_policy::install(hosts, mode);
    f()
}

// ---------------------------------------------------------------------------
// Production HTTP transport (ureq, blocking, native-only)
// ---------------------------------------------------------------------------

/// The real network transport: a blocking ureq POST with the SPARQL query
/// form-encoded in the body and an `Accept` header that prefers SPARQL-Results-JSON but
/// also accepts SPARQL-Results-XML (the response is content-sniffed by `parse_results`).
///
/// Gated to `cfg(not(wasm32))` AND the `service` feature so neither ureq nor any of
/// its TLS stack ever enters the wasm bundle.
#[cfg(all(feature = "service", not(target_arch = "wasm32")))]
pub(crate) struct HttpTransport {
    timeout: std::time::Duration,
}

#[cfg(all(feature = "service", not(target_arch = "wasm32")))]
impl HttpTransport {
    pub(crate) fn new() -> Self {
        // A finite default so an unreachable/slow endpoint cannot hang the engine
        // indefinitely; SILENT then turns this into an empty result.
        HttpTransport { timeout: std::time::Duration::from_secs(30) }
    }
}

/// ureq [`Resolver`](ureq::unversioned::resolver::Resolver) wrapper that enforces the SSRF egress policy
/// on the *resolved* addresses (DNS-rebinding-safe). [OPUS-4.8]
///
/// It resolves `netloc` with the standard system resolver, drops every address
/// the [`is_forbidden_ip`] policy refuses (unless the host is on the active
/// allowlist), and returns only the survivors — so ureq dials only vetted IPs.
/// If resolution yields nothing but forbidden addresses, it returns a
/// `PermissionDenied` error rather than an empty set, which surfaces to the
/// caller as a SERVICE failure (and an empty result under SILENT).
#[cfg(all(feature = "service", not(target_arch = "wasm32")))]
#[derive(Debug)]
struct EgressFilterResolver;

// [OPUS-4.8] sq-g2xs: the ureq-3 `Resolver` trait takes a parsed `&http::Uri` (+ `Config` +
// timeout) and returns an `ArrayVec<SocketAddr, 16>` rather than ureq-2's `&str` netloc → `Vec`.
// The SSRF logic is otherwise byte-for-byte the ureq-2 policy: derive `host:port` from the URI
// authority + scheme default port, key the allowlist by the bare host, resolve, drop every
// `is_forbidden_ip` address (unless allowlisted), and refuse with a `PermissionDenied` io error
// (carried as `ureq::Error::Io`, so the `SERVICE_EGRESS_REFUSED_MARKER` survives the wrapping
// and the kind stays `PermissionDenied` for the egress tests). A refusal is a HARD error (never
// an empty address set), so ureq cannot fall through to an unguarded dial.

/// `host:port` (for resolution) + bare host (for the allowlist key, IPv6 brackets stripped,
/// lowercased) from a ureq-3 request [`Uri`](ureq::http::Uri). `port` falls back to the scheme
/// default (443 for https, 80 otherwise). [OPUS-4.8] sq-g2xs.
#[cfg(all(feature = "service", not(target_arch = "wasm32")))]
fn uri_host_port(uri: &ureq::http::Uri) -> Option<(String, String)> {
    let authority = uri.authority()?;
    let host = authority.host();
    if host.is_empty() {
        return None;
    }
    let port = authority.port_u16().unwrap_or_else(|| {
        match uri.scheme_str() {
            Some("https") => 443,
            _ => 80,
        }
    });
    // The authority host keeps IPv6 brackets (`[::1]`); strip them for the allowlist key and
    // for `to_socket_addrs` (which wants the bare host + a separate port).
    let bare = host.trim_start_matches('[').trim_end_matches(']');
    Some((format!("{bare}:{port}"), bare.to_ascii_lowercase()))
}

/// Wrap a refusal reason as a `PermissionDenied` [`ureq::Error::Io`], preserving both the kind
/// (the egress tests assert on it) and the [`SERVICE_EGRESS_REFUSED_MARKER`] text. [OPUS-4.8].
#[cfg(all(feature = "service", not(target_arch = "wasm32")))]
fn egress_refused(reason: String) -> ureq::Error {
    ureq::Error::Io(std::io::Error::new(
        std::io::ErrorKind::PermissionDenied,
        reason,
    ))
}

#[cfg(all(feature = "service", not(target_arch = "wasm32")))]
impl ureq::unversioned::resolver::Resolver for EgressFilterResolver {
    fn resolve(
        &self,
        uri: &ureq::http::Uri,
        _config: &ureq::config::Config,
        _timeout: ureq::unversioned::transport::NextTimeout,
    ) -> Result<ureq::unversioned::resolver::ResolvedSocketAddrs, ureq::Error> {
        use std::net::ToSocketAddrs;
        let (host_port, host) = uri_host_port(uri).ok_or_else(|| {
            egress_refused(format!(
                "{SERVICE_EGRESS_REFUSED_MARKER}: request URI {uri} has no host authority to vet"
            ))
        })?;
        let allowed = egress_policy::is_allowed(&host);
        // [OPUS-4.8] (sq-4w18) STRICT (AllowlistOnly) mode — the server's policy — refuses any
        // host not on the allowlist BEFORE resolving DNS, so a host that is not explicitly
        // permitted never triggers even a lookup. An empty allowlist here = deny ALL SERVICE.
        if !allowed && egress_policy::mode() == egress_policy::Mode::AllowlistOnly {
            return Err(egress_refused(format!(
                "{SERVICE_EGRESS_REFUSED_MARKER}: host {host:?} is not on the SERVICE allowlist \
                 (strict allowlist-only policy; add it via --service-allow / SPARQ_SERVICE_ALLOW \
                 on the server, or with_service_egress_policy in an embedder)"
            )));
        }
        let resolved = host_port.to_socket_addrs().map_err(ureq::Error::Io)?;
        let mut permitted: ureq::unversioned::resolver::ResolvedSocketAddrs = arrayvec_default();
        // `ResolvedSocketAddrs` is a fixed-capacity `ArrayVec<_, 16>`; cap to its capacity (the
        // same `MAX_ADDRS` ureq's own resolver uses) so `push` never overruns the backing array.
        for sa in resolved
            .filter(|sa| allowed || !is_forbidden_ip(sa.ip()))
            .take(RESOLVED_ADDRS_CAP)
        {
            permitted.push(sa);
        }
        if permitted.is_empty() {
            return Err(egress_refused(format!(
                "{SERVICE_EGRESS_REFUSED_MARKER}: {host_port} resolves only to private/internal addresses \
                 (default-deny SSRF policy; allowlist the host via with_service_egress_allow)"
            )));
        }
        Ok(permitted)
    }
}

/// Capacity of ureq-3's [`ResolvedSocketAddrs`](ureq::unversioned::resolver::ResolvedSocketAddrs)
/// (`ArrayVec<SocketAddr, 16>`). Matches ureq's own `MAX_ADDRS`. [OPUS-4.8] sq-g2xs.
#[cfg(all(feature = "service", not(target_arch = "wasm32")))]
const RESOLVED_ADDRS_CAP: usize = 16;

/// An empty [`ResolvedSocketAddrs`](ureq::unversioned::resolver::ResolvedSocketAddrs) backing
/// store (a fixed-capacity `ArrayVec`). [OPUS-4.8] sq-g2xs.
#[cfg(all(feature = "service", not(target_arch = "wasm32")))]
fn arrayvec_default() -> ureq::unversioned::resolver::ResolvedSocketAddrs {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    // `ArrayVec::from_fn` fills the backing array, but the logical length starts at 0
    // (the same idiom ureq's `DefaultResolver::empty` uses), so it is genuinely empty.
    ureq::unversioned::resolver::ArrayVec::from_fn(|_| {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0)
    })
}

#[cfg(all(feature = "service", not(target_arch = "wasm32")))]
impl Transport for HttpTransport {
    fn fetch(&self, endpoint: &str, query: &str) -> Result<String, String> {
        // [OPUS-4.8] sq-g2xs: ureq-3 builds an `Agent` from a `Config` + a custom resolver via
        // `Agent::with_parts`; the resolver carries the default-deny SSRF policy exactly as in
        // ureq 2 (the resolved-and-vetted IP is the dialled IP — no DNS-rebinding re-resolve).
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(self.timeout))
            .user_agent(concat!("sparq-engine/", env!("CARGO_PKG_VERSION")))
            .build();
        let agent = ureq::Agent::with_parts(
            config,
            ureq::unversioned::transport::DefaultConnector::new(),
            EgressFilterResolver,
        );
        // POST with the query in an `application/x-www-form-urlencoded` `query=` field
        // (SPARQL Protocol §2.1.2 "query via POST with URL-encoded parameters") — the
        // most broadly supported method and not subject to URL-length limits.
        let resp = agent
            .post(endpoint)
            // Prefer JSON, but accept XML — some endpoints ignore `Accept` and only emit
            // SPARQL-Results-XML, which `parse_results` now handles (bead sq-ycu). [OPUS-4.8]
            .header(
                "Accept",
                "application/sparql-results+json, application/sparql-results+xml;q=0.9",
            )
            .send_form([("query", query)]);
        match resp {
            // ureq-3 caps `read_to_string` at 10 MiB by default; a federated SELECT result can
            // exceed that, so raise the limit generously (a finite cap still bounds memory).
            Ok(mut r) => r
                .body_mut()
                .with_config()
                .limit(SERVICE_MAX_BODY_BYTES)
                .read_to_string()
                .map_err(|e| format!("SERVICE: reading response from {endpoint}: {e}")),
            // ureq-3 surfaces non-2xx as `Error::StatusCode`; treat both transport and HTTP
            // errors uniformly (the caller decides SILENT vs propagate).
            Err(ureq::Error::StatusCode(code)) => {
                Err(format!("SERVICE: endpoint {endpoint} returned HTTP {code}"))
            }
            Err(e) => Err(format!("SERVICE: request to {endpoint} failed: {e}")),
        }
    }
}

/// Max bytes read from a SERVICE response body. ureq-3's default `read_to_string` cap is 10 MiB;
/// a federated SELECT can legitimately exceed that, so we raise it to a generous-but-finite bound
/// (memory is still bounded — a runaway endpoint cannot OOM the engine). [OPUS-4.8] sq-g2xs.
#[cfg(all(feature = "service", not(target_arch = "wasm32")))]
const SERVICE_MAX_BODY_BYTES: u64 = 1024 * 1024 * 1024;

// ---------------------------------------------------------------------------
// Tests (parser + transport seam; no public network)
// ---------------------------------------------------------------------------

#[cfg(all(test, feature = "service"))]
mod tests {
    use super::*;

    #[test]
    fn parses_uri_and_literal_bindings() {
        let body = r#"{
            "head": { "vars": ["s", "name"] },
            "results": { "bindings": [
                { "s": {"type":"uri","value":"http://ex/a"},
                  "name": {"type":"literal","value":"Alice"} },
                { "s": {"type":"uri","value":"http://ex/b"},
                  "name": {"type":"literal","value":"Bob","xml:lang":"en"} }
            ] }
        }"#;
        let rel = parse_srj(body).unwrap();
        assert_eq!(rel.vars.len(), 2);
        assert_eq!(rel.rows.len(), 2);
        assert_eq!(
            rel.rows[0][0],
            Some(Term::NamedNode(NamedNode::new("http://ex/a").unwrap()))
        );
        assert_eq!(
            rel.rows[1][1],
            Some(Term::Literal(
                Literal::new_language_tagged_literal("Bob", "en").unwrap()
            ))
        );
    }

    #[test]
    fn unbound_variable_becomes_none() {
        let body = r#"{
            "head": { "vars": ["a", "b"] },
            "results": { "bindings": [ { "a": {"type":"uri","value":"http://ex/x"} } ] }
        }"#;
        let rel = parse_srj(body).unwrap();
        assert_eq!(rel.rows[0][0], Some(Term::NamedNode(NamedNode::new("http://ex/x").unwrap())));
        assert_eq!(rel.rows[0][1], None);
    }

    #[test]
    fn typed_literal_roundtrips() {
        let body = r#"{
            "head": { "vars": ["n"] },
            "results": { "bindings": [
                { "n": {"type":"literal","value":"42",
                        "datatype":"http://www.w3.org/2001/XMLSchema#integer"} }
            ] }
        }"#;
        let rel = parse_srj(body).unwrap();
        let want = Literal::new_typed_literal(
            "42",
            NamedNode::new("http://www.w3.org/2001/XMLSchema#integer").unwrap(),
        );
        assert_eq!(rel.rows[0][0], Some(Term::Literal(want)));
    }

    /// [OPUS-4.8] sq-s955: inbound parity for the outbound `its:dir` emission. A SERVICE
    /// endpoint's results carry the RDF 1.2 base direction as a SEPARATE `its:dir` field
    /// next to the bare `xml:lang` tag; the parser must reconstruct the `dirLangString` so
    /// the direction survives INTO the local join (it was previously dropped). This mirrors
    /// the outbound `json.rs` parity test — direction round-trips both ways.
    #[test]
    fn dir_lang_string_direction_roundtrips_inbound() {
        for (dir, want) in [
            ("ltr", oxrdf::BaseDirection::Ltr),
            ("rtl", oxrdf::BaseDirection::Rtl),
        ] {
            let body = format!(
                r#"{{
                "head": {{ "vars": ["g"] }},
                "results": {{ "bindings": [
                    {{ "g": {{"type":"literal","value":"مرحبا","xml:lang":"ar","its:dir":"{dir}"}} }}
                ] }}
            }}"#
            );
            let rel = parse_srj(&body).unwrap();
            match &rel.rows[0][0] {
                Some(Term::Literal(l)) => {
                    assert_eq!(l.value(), "مرحبا");
                    assert_eq!(l.language(), Some("ar"));
                    // The direction round-trips inbound (the bug: it was None).
                    assert_eq!(l.direction(), Some(want), "its:dir={dir} must survive inbound");
                }
                other => panic!("expected a directional literal, got {other:?}"),
            }
        }
    }

    /// [OPUS-4.8] sq-s955: an ABSENT or INVALID `its:dir` degrades to a PLAIN language-tagged
    /// literal — the same decision `dict::split_lang_dir` / `reconstruct_ref` make for a
    /// malformed stored slot — so the inbound, stored-slot, materialised and outbound paths
    /// all AGREE on `(lang, dir)`. (An uppercase `LTR` or any other value is not a direction.)
    #[test]
    fn invalid_or_absent_its_dir_is_plain_language_literal() {
        let plain = Literal::new_language_tagged_literal("hi", "en").unwrap();
        for cell in [
            r#"{"type":"literal","value":"hi","xml:lang":"en"}"#, // absent its:dir
            r#"{"type":"literal","value":"hi","xml:lang":"en","its:dir":"LTR"}"#, // wrong case
            r#"{"type":"literal","value":"hi","xml:lang":"en","its:dir":"sideways"}"#, // bogus
            r#"{"type":"literal","value":"hi","xml:lang":"en","its:dir":""}"#, // empty
        ] {
            let body =
                format!(r#"{{"head":{{"vars":["g"]}},"results":{{"bindings":[{{"g":{cell}}}]}}}}"#);
            let rel = parse_srj(&body).unwrap();
            assert_eq!(
                rel.rows[0][0],
                Some(Term::Literal(plain.clone())),
                "invalid/absent its:dir in {cell} must degrade to a plain language-tagged literal"
            );
        }
    }

    #[test]
    fn empty_results_is_ok() {
        let body = r#"{"head":{"vars":["x"]},"results":{"bindings":[]}}"#;
        let rel = parse_srj(body).unwrap();
        assert!(rel.rows.is_empty());
        assert_eq!(rel.vars.len(), 1);
    }

    #[test]
    fn malformed_json_errors() {
        assert!(parse_srj("not json at all").is_err());
        assert!(parse_srj(r#"{"head":{}}"#).is_err()); // no vars
        assert!(parse_srj(r#"{"boolean":true}"#).is_err()); // ASK, not SELECT
    }

    // ---------------------------------------------------------------------
    // SPARQL-Results-XML (SRX) parsing [OPUS-4.8] (bead sq-ycu)
    // ---------------------------------------------------------------------

    const SRX_NS: &str = "http://www.w3.org/2005/sparql-results#";

    #[test]
    fn srx_parses_uri_and_literals() {
        let body = format!(
            r#"<?xml version="1.0"?>
            <sparql xmlns="{SRX_NS}">
              <head><variable name="s"/><variable name="name"/></head>
              <results>
                <result>
                  <binding name="s"><uri>http://ex/a</uri></binding>
                  <binding name="name"><literal>Alice</literal></binding>
                </result>
                <result>
                  <binding name="s"><uri>http://ex/b</uri></binding>
                  <binding name="name"><literal xml:lang="en">Bob</literal></binding>
                </result>
              </results>
            </sparql>"#
        );
        let rel = parse_srx(&body).unwrap();
        assert_eq!(rel.vars.len(), 2);
        assert_eq!(rel.rows.len(), 2);
        assert_eq!(
            rel.rows[0][0],
            Some(Term::NamedNode(NamedNode::new("http://ex/a").unwrap()))
        );
        assert_eq!(
            rel.rows[0][1],
            Some(Term::Literal(Literal::new_simple_literal("Alice")))
        );
        assert_eq!(
            rel.rows[1][1],
            Some(Term::Literal(
                Literal::new_language_tagged_literal("Bob", "en").unwrap()
            ))
        );
    }

    #[test]
    fn srx_unbound_variable_becomes_none() {
        // `b` is absent in the single solution and the bindings appear out of
        // declaration order — projection must be positional over <variable> order.
        let body = format!(
            r#"<sparql xmlns="{SRX_NS}">
              <head><variable name="a"/><variable name="b"/></head>
              <results>
                <result><binding name="a"><uri>http://ex/x</uri></binding></result>
              </results>
            </sparql>"#
        );
        let rel = parse_srx(&body).unwrap();
        assert_eq!(
            rel.rows[0][0],
            Some(Term::NamedNode(NamedNode::new("http://ex/x").unwrap()))
        );
        assert_eq!(rel.rows[0][1], None);
    }

    #[test]
    fn srx_typed_literal_and_bnode_and_entities() {
        let body = format!(
            r#"<sparql xmlns="{SRX_NS}">
              <head><variable name="n"/><variable name="b"/><variable name="t"/></head>
              <results>
                <result>
                  <binding name="n"><literal datatype="http://www.w3.org/2001/XMLSchema#integer">42</literal></binding>
                  <binding name="b"><bnode>b0</bnode></binding>
                  <binding name="t"><literal>a &amp; b &#38; c &#x3C; d</literal></binding>
                </result>
              </results>
            </sparql>"#
        );
        let rel = parse_srx(&body).unwrap();
        assert_eq!(
            rel.rows[0][0],
            Some(Term::Literal(Literal::new_typed_literal(
                "42",
                NamedNode::new("http://www.w3.org/2001/XMLSchema#integer").unwrap()
            )))
        );
        assert_eq!(
            rel.rows[0][1],
            Some(Term::BlankNode(oxrdf::BlankNode::new("b0").unwrap()))
        );
        // Predefined (&amp; -> &), decimal (&#38; -> &) and hex (&#x3C; -> <) refs all decode.
        assert_eq!(
            rel.rows[0][2],
            Some(Term::Literal(Literal::new_simple_literal("a & b & c < d")))
        );
    }

    #[test]
    fn srx_dir_lang_string_direction_roundtrips() {
        for (dir, want) in [
            ("ltr", oxrdf::BaseDirection::Ltr),
            ("rtl", oxrdf::BaseDirection::Rtl),
        ] {
            let body = format!(
                r#"<sparql xmlns="{SRX_NS}">
                  <head><variable name="g"/></head>
                  <results><result>
                    <binding name="g"><literal xml:lang="ar" its:dir="{dir}">مرحبا</literal></binding>
                  </result></results>
                </sparql>"#
            );
            let rel = parse_srx(&body).unwrap();
            match &rel.rows[0][0] {
                Some(Term::Literal(l)) => {
                    assert_eq!(l.value(), "مرحبا");
                    assert_eq!(l.language(), Some("ar"));
                    assert_eq!(l.direction(), Some(want), "its:dir={dir} must survive");
                }
                other => panic!("expected a directional literal, got {other:?}"),
            }
        }
    }

    #[test]
    fn srx_triple_term_parses() {
        // SPARQL 1.2 quoted-triple value: << <s> <p> "o" >>.
        let body = format!(
            r#"<sparql xmlns="{SRX_NS}">
              <head><variable name="t"/></head>
              <results><result>
                <binding name="t"><triple>
                  <subject><uri>http://ex/s</uri></subject>
                  <predicate><uri>http://ex/p</uri></predicate>
                  <object><literal>o</literal></object>
                </triple></binding>
              </result></results>
            </sparql>"#
        );
        let rel = parse_srx(&body).unwrap();
        let want = Term::Triple(Box::new(Triple {
            subject: NamedOrBlankNode::NamedNode(NamedNode::new("http://ex/s").unwrap()),
            predicate: NamedNode::new("http://ex/p").unwrap(),
            object: Term::Literal(Literal::new_simple_literal("o")),
        }));
        assert_eq!(rel.rows[0][0], Some(want));
    }

    #[test]
    fn srx_empty_results_is_ok() {
        let body = format!(
            r#"<sparql xmlns="{SRX_NS}"><head><variable name="x"/></head><results></results></sparql>"#
        );
        let rel = parse_srx(&body).unwrap();
        assert!(rel.rows.is_empty());
        assert_eq!(rel.vars.len(), 1);
    }

    #[test]
    fn srx_ask_boolean_is_rejected() {
        // SERVICE always wraps a SELECT, so an ASK boolean body is an error (mirrors SRJ).
        let body =
            format!(r#"<sparql xmlns="{SRX_NS}"><head/><boolean>true</boolean></sparql>"#);
        assert!(parse_srx(&body).is_err());
    }

    /// The core bead requirement: an endpoint that ignores `Accept` and returns XML must
    /// still be parsed (content-sniffed) by the end-to-end path, not just `parse_srj`.
    #[test]
    fn eval_remote_handles_xml_endpoint() {
        let body = r#"<?xml version="1.0"?>
            <sparql xmlns="http://www.w3.org/2005/sparql-results#">
              <head><variable name="x"/></head>
              <results><result><binding name="x"><uri>http://ex/1</uri></binding></result></results>
            </sparql>"#;
        let rel = eval_remote(&Canned(body), "http://unused/", "SELECT * WHERE {}").unwrap();
        assert_eq!(rel.rows.len(), 1);
        assert_eq!(
            rel.rows[0][0],
            Some(Term::NamedNode(NamedNode::new("http://ex/1").unwrap()))
        );
    }

    #[test]
    fn parse_results_rejects_non_json_non_xml() {
        // A leading byte that is neither `{` nor `<` (e.g. an HTML error page without a
        // doctype, or plain text) is reported rather than silently mis-parsed.
        assert!(parse_results("connection reset by peer").is_err());
        assert!(parse_results("").is_err());
        // Leading whitespace before the sniff byte is tolerated.
        assert!(parse_results("   {\"head\":{\"vars\":[\"x\"]},\"results\":{\"bindings\":[]}}").is_ok());
    }

    /// Canned-response transport: proves `eval_remote` wires the transport into the
    /// parser without touching the network.
    struct Canned(&'static str);
    impl Transport for Canned {
        fn fetch(&self, _endpoint: &str, _query: &str) -> Result<String, String> {
            Ok(self.0.to_string())
        }
    }

    #[test]
    fn eval_remote_uses_injected_transport() {
        let body = r#"{"head":{"vars":["x"]},
            "results":{"bindings":[{"x":{"type":"uri","value":"http://ex/1"}}]}}"#;
        let rel = eval_remote(&Canned(body), "http://unused/", "SELECT * WHERE {}").unwrap();
        assert_eq!(rel.rows.len(), 1);
    }

    struct Failing;
    impl Transport for Failing {
        fn fetch(&self, _e: &str, _q: &str) -> Result<String, String> {
            Err("connection refused".into())
        }
    }

    #[test]
    fn eval_remote_propagates_transport_error() {
        let err = eval_remote(&Failing, "http://unused/", "SELECT * WHERE {}").unwrap_err();
        assert!(err.contains("connection refused"));
    }

    // ---------------------------------------------------------------------
    // SSRF egress policy [OPUS-4.8] (bead sq-2v6f)
    // ---------------------------------------------------------------------

    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    fn v4(a: u8, b: u8, c: u8, d: u8) -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(a, b, c, d))
    }

    #[test]
    fn loopback_is_forbidden() {
        assert!(is_forbidden_ip(v4(127, 0, 0, 1)));
        assert!(is_forbidden_ip(v4(127, 255, 255, 254))); // anywhere in 127/8
        assert!(is_forbidden_ip(IpAddr::V6(Ipv6Addr::LOCALHOST))); // ::1
    }

    #[test]
    fn rfc1918_private_is_forbidden() {
        assert!(is_forbidden_ip(v4(10, 0, 0, 1))); // 10/8
        assert!(is_forbidden_ip(v4(10, 255, 255, 255)));
        assert!(is_forbidden_ip(v4(172, 16, 0, 1))); // 172.16/12 (low edge)
        assert!(is_forbidden_ip(v4(172, 31, 255, 255))); // 172.16/12 (high edge)
        assert!(is_forbidden_ip(v4(192, 168, 0, 1))); // 192.168/16
    }

    #[test]
    fn link_local_and_cloud_metadata_are_forbidden() {
        assert!(is_forbidden_ip(v4(169, 254, 0, 1))); // 169.254/16
        // The cloud-metadata endpoint — the highest-value SSRF target.
        assert!(is_forbidden_ip(v4(169, 254, 169, 254)));
        // IPv6 link-local fe80::/10.
        assert!(is_forbidden_ip(IpAddr::V6("fe80::1".parse().unwrap())));
        assert!(is_forbidden_ip(IpAddr::V6("febf::1".parse().unwrap()))); // top of /10
    }

    #[test]
    fn unique_local_v6_is_forbidden() {
        assert!(is_forbidden_ip(IpAddr::V6("fc00::1".parse().unwrap()))); // fc00::/7
        assert!(is_forbidden_ip(IpAddr::V6("fd12:3456::1".parse().unwrap())));
    }

    #[test]
    fn unspecified_is_forbidden() {
        assert!(is_forbidden_ip(v4(0, 0, 0, 0)));
        assert!(is_forbidden_ip(IpAddr::V6(Ipv6Addr::UNSPECIFIED))); // ::
    }

    #[test]
    fn cgnat_and_broadcast_are_forbidden() {
        assert!(is_forbidden_ip(v4(100, 64, 0, 1))); // 100.64/10 CGNAT (low)
        assert!(is_forbidden_ip(v4(100, 127, 255, 255))); // CGNAT (high)
        assert!(!is_forbidden_ip(v4(100, 63, 0, 1))); // just below CGNAT — public
        assert!(!is_forbidden_ip(v4(100, 128, 0, 1))); // just above CGNAT — public
        assert!(is_forbidden_ip(v4(255, 255, 255, 255))); // broadcast
    }

    #[test]
    fn ipv4_mapped_v6_is_unwrapped_and_classified() {
        // ::ffff:127.0.0.1 must be refused as the embedded private v4.
        assert!(is_forbidden_ip(IpAddr::V6("::ffff:127.0.0.1".parse().unwrap())));
        assert!(is_forbidden_ip(IpAddr::V6("::ffff:10.0.0.1".parse().unwrap())));
        assert!(is_forbidden_ip(IpAddr::V6("::ffff:169.254.169.254".parse().unwrap())));
        // A public v4 mapped into v6 is still allowed.
        assert!(!is_forbidden_ip(IpAddr::V6("::ffff:8.8.8.8".parse().unwrap())));
    }

    #[test]
    fn public_addresses_are_allowed() {
        assert!(!is_forbidden_ip(v4(8, 8, 8, 8))); // Google DNS
        assert!(!is_forbidden_ip(v4(1, 1, 1, 1))); // Cloudflare DNS
        assert!(!is_forbidden_ip(v4(93, 184, 216, 34))); // example.com (historical)
        assert!(!is_forbidden_ip(v4(172, 15, 0, 1))); // just below 172.16/12 — public
        assert!(!is_forbidden_ip(v4(172, 32, 0, 1))); // just above 172.16/12 — public
        assert!(!is_forbidden_ip(IpAddr::V6("2001:4860:4860::8888".parse().unwrap()))); // public v6
    }

    #[test]
    fn allowlist_plumbing_install_and_restore() {
        // Default: nothing is allowlisted.
        assert!(!egress_policy::is_allowed("localhost"));
        {
            let _g = egress_policy::install(
                ["localhost".to_string(), "10.0.0.5".to_string()],
                egress_policy::Mode::DenyPrivate,
            );
            assert!(egress_policy::is_allowed("localhost"));
            assert!(egress_policy::is_allowed("LOCALHOST")); // case-insensitive
            assert!(egress_policy::is_allowed("10.0.0.5"));
            assert!(!egress_policy::is_allowed("other.host"));
        }
        // Restored to empty on guard drop.
        assert!(!egress_policy::is_allowed("localhost"));
    }

    #[test]
    fn with_service_egress_allow_scopes_the_allowlist() {
        assert!(!egress_policy::is_allowed("sparql.internal"));
        let seen = with_service_egress_allow(["sparql.internal".to_string()], || {
            egress_policy::is_allowed("sparql.internal")
        });
        assert!(seen);
        // Allowlist is gone after the scope returns.
        assert!(!egress_policy::is_allowed("sparql.internal"));
    }

    #[test]
    fn strict_allowlist_only_mode_scopes_and_restores() {
        // [OPUS-4.8] (sq-4w18) Strict mode: only listed hosts are allowed; the mode
        // and allowlist both restore on scope exit.
        assert_eq!(egress_policy::mode(), egress_policy::Mode::DenyPrivate);
        assert!(!egress_policy::is_allowed("a.example"));
        with_service_egress_policy(true, ["a.example".to_string()], || {
            assert_eq!(egress_policy::mode(), egress_policy::Mode::AllowlistOnly);
            assert!(egress_policy::is_allowed("a.example"));
            assert!(egress_policy::is_allowed("A.EXAMPLE")); // case-insensitive
            assert!(!egress_policy::is_allowed("b.example"));
        });
        assert_eq!(egress_policy::mode(), egress_policy::Mode::DenyPrivate);
        assert!(!egress_policy::is_allowed("a.example"));
    }

    #[test]
    fn suffix_wildcard_allowlist_matches_apex_and_subdomains() {
        // [OPUS-4.8] (sq-4w18) A ".example.org" entry matches the apex and any
        // subdomain, but not a host that merely ends in the same letters.
        with_service_egress_policy(true, [".example.org".to_string()], || {
            assert!(egress_policy::is_allowed("example.org")); // apex
            assert!(egress_policy::is_allowed("sparql.example.org")); // subdomain
            assert!(egress_policy::is_allowed("a.b.example.org")); // deep subdomain
            assert!(egress_policy::is_allowed("SPARQL.EXAMPLE.ORG")); // case-insensitive
            assert!(!egress_policy::is_allowed("notexample.org")); // boundary respected
            assert!(!egress_policy::is_allowed("example.org.evil.com")); // suffix only
        });
    }

    #[test]
    fn non_strict_policy_matches_allow_helper() {
        // strict=false behaves exactly like with_service_egress_allow (DenyPrivate).
        with_service_egress_policy(false, ["c.example".to_string()], || {
            assert_eq!(egress_policy::mode(), egress_policy::Mode::DenyPrivate);
            assert!(egress_policy::is_allowed("c.example"));
        });
    }

    #[test]
    fn allowlist_restores_on_unwind() {
        // A panic inside the scope must still restore the previous (empty) policy —
        // a relaxed allowlist must never leak past the scope on unwind.
        let _ = std::panic::catch_unwind(|| {
            with_service_egress_allow(["leaky.host".to_string()], || {
                assert!(egress_policy::is_allowed("leaky.host"));
                panic!("boom");
            });
        });
        assert!(!egress_policy::is_allowed("leaky.host"));
    }

    // The resolver path is native-only (it wraps ureq's Resolver).
    #[cfg(not(target_arch = "wasm32"))]
    mod resolver {
        use super::*;

        /// [OPUS-4.8] sq-g2xs: invoke the ureq-3 `Resolver` for a `host:port` netloc by building
        /// the `http://<netloc>/` URI the resolver parses (default `Config` + no-deadline timeout).
        fn resolve_netloc(
            netloc: &str,
        ) -> Result<ureq::unversioned::resolver::ResolvedSocketAddrs, ureq::Error> {
            use ureq::unversioned::resolver::Resolver;
            let uri: ureq::http::Uri = format!("http://{netloc}/").parse().unwrap();
            let config = ureq::Agent::config_builder().build();
            let timeout = ureq::unversioned::transport::NextTimeout {
                after: ureq::unversioned::transport::time::Duration::NotHappening,
                reason: ureq::Timeout::Global,
            };
            EgressFilterResolver.resolve(&uri, &config, timeout)
        }

        /// `true` iff `e` is the egress-refusal `PermissionDenied` io error.
        fn is_permission_denied(e: &ureq::Error) -> bool {
            matches!(e, ureq::Error::Io(io) if io.kind() == std::io::ErrorKind::PermissionDenied)
        }

        #[test]
        fn resolver_refuses_loopback_endpoint() {
            // 127.0.0.1 resolves to itself; with no allowlist the policy must
            // refuse it with PermissionDenied — never returning a dial-able addr.
            let err = resolve_netloc("127.0.0.1:8080").unwrap_err();
            assert!(is_permission_denied(&err), "got {err:?}");
        }

        #[test]
        fn resolver_refuses_cloud_metadata_endpoint() {
            let err = resolve_netloc("169.254.169.254:80").unwrap_err();
            assert!(is_permission_denied(&err), "got {err:?}");
        }

        #[test]
        fn resolver_refuses_ipv6_loopback_endpoint() {
            // ureq passes IPv6 netlocs bracketed.
            let err = resolve_netloc("[::1]:80").unwrap_err();
            assert!(is_permission_denied(&err), "got {err:?}");
        }

        #[test]
        fn resolver_allows_public_endpoint() {
            // 8.8.8.8 is a literal so no DNS lookup happens; it is global, so it
            // passes the filter and comes back as a dial-able address.
            let addrs = resolve_netloc("8.8.8.8:443").unwrap();
            assert_eq!(addrs.len(), 1);
            assert_eq!(addrs[0].ip(), v4(8, 8, 8, 8));
        }

        #[test]
        fn resolver_permits_allowlisted_private_endpoint() {
            // With 127.0.0.1 on the allowlist, the loopback endpoint is dial-able.
            let addrs = with_service_egress_allow(["127.0.0.1".to_string()], || {
                resolve_netloc("127.0.0.1:8080")
            })
            .unwrap();
            assert_eq!(addrs.len(), 1);
            assert!(addrs[0].ip().is_loopback());
        }

        // [OPUS-4.8] (sq-4w18) Strict allowlist-only mode — the server's policy.

        #[test]
        fn strict_refuses_public_host_off_the_allowlist() {
            let err = with_service_egress_policy(true, std::iter::empty(), || {
                resolve_netloc("8.8.8.8:443")
            })
            .unwrap_err();
            assert!(is_permission_denied(&err), "got {err:?}");
        }

        #[test]
        fn strict_empty_allowlist_denies_all() {
            for netloc in ["8.8.8.8:443", "1.1.1.1:80", "127.0.0.1:8080"] {
                let err = with_service_egress_policy(true, std::iter::empty(), || {
                    resolve_netloc(netloc)
                })
                .unwrap_err();
                assert!(is_permission_denied(&err), "{netloc} must be refused, got {err:?}");
            }
        }

        #[test]
        fn strict_permits_allowlisted_host() {
            let addrs = with_service_egress_policy(true, ["8.8.8.8".to_string()], || {
                resolve_netloc("8.8.8.8:443")
            })
            .unwrap();
            assert_eq!(addrs.len(), 1);
            assert_eq!(addrs[0].ip(), v4(8, 8, 8, 8));

            let addrs = with_service_egress_policy(true, ["127.0.0.1".to_string()], || {
                resolve_netloc("127.0.0.1:8080")
            })
            .unwrap();
            assert_eq!(addrs.len(), 1);
            assert!(addrs[0].ip().is_loopback());
        }

        #[test]
        fn non_strict_resolver_allows_public_off_list() {
            let addrs = with_service_egress_policy(false, std::iter::empty(), || {
                resolve_netloc("8.8.8.8:443")
            })
            .unwrap();
            assert_eq!(addrs.len(), 1);
        }
    }
}
