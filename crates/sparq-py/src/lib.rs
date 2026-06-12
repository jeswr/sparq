//! sparq-py: the `sparq` Python package (pyo3 + maturin).
//!
//! A thin, allocation-conscious wrapper over the workspace's public APIs:
//!
//! * [`Graph`] wraps `sparq_core::Graph` (load / save / open / len).
//! * `Graph.query` / `Graph.query_json` / `Graph.ask` / `Graph.construct` /
//!   `Graph.describe` call `sparq_engine` (all four query forms are native:
//!   ASK early-exits, CONSTRUCT/DESCRIBE return term triples).
//! * `Graph.update` applies SPARQL Update; the engine returns a NEW graph
//!   (immutable store, rebuild semantics), which the wrapper swaps in place so
//!   Python sees an in-place mutation. Named graphs survive every operation
//!   (GRAPH-scoped data ops, graph templates, USING, CLEAR/DROP/ADD/COPY/MOVE).
//! * `Graph.reason` materializes the RDFS / OWL-RL closure via `sparq_reason`
//!   (in place, same swap; named graphs are carried across the rebuild);
//!   `Graph.inconsistencies` reports OWL 2 RL clashes; `Graph.load_n3` runs the
//!   Notation3 forward-chainer over an N3 document (facts + `{…} => {…}` rules)
//!   and `Graph.reason_n3_with(rules)` runs caller-supplied N3 rules over an
//!   already-loaded graph.
//!
//! Long-running engine calls release the GIL (`py.detach`) so other Python
//! threads keep running during parse / query / reasoning.

use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyString};
use sparq_core::dict::{Dict, Id};
use sparq_core::Graph as CoreGraph;
use spargebra::{Query, SparqlParser};

const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";

/// Map an engine `Err(String)` (parse/eval failure) to a Python `ValueError`.
fn engine_err(e: String) -> PyErr {
    PyValueError::new_err(e)
}

// ---------------------------------------------------------------------------
// Term
// ---------------------------------------------------------------------------

/// One RDF term in a query solution.
///
/// `kind` follows the SPARQL 1.1 JSON results convention (the same strings
/// `Graph.query_json` emits): `"uri"`, `"literal"`, or `"bnode"`.
/// `value` is the IRI / lexical form / blank-node label. For literals,
/// `language` is the language tag (if any) and `datatype` the datatype IRI
/// (always set — plain literals carry `xsd:string`, language-tagged ones
/// `rdf:langString`). Non-literals have `language = datatype = None`.
// `skip_from_py_object`: pyo3 0.29 makes the auto `FromPyObject` for Clone
// pyclasses opt-in; nothing here extracts a `Term` from Python, so skip it.
#[pyclass(frozen, eq, hash, skip_from_py_object, module = "sparq")]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Term {
    #[pyo3(get)]
    kind: String,
    #[pyo3(get)]
    value: String,
    #[pyo3(get)]
    language: Option<String>,
    #[pyo3(get)]
    datatype: Option<String>,
}

impl Term {
    fn from_oxrdf(t: &oxrdf::Term) -> Term {
        match t {
            oxrdf::Term::NamedNode(n) => Term {
                kind: "uri".into(),
                value: n.as_str().to_string(),
                language: None,
                datatype: None,
            },
            oxrdf::Term::BlankNode(b) => Term {
                kind: "bnode".into(),
                value: b.as_str().to_string(),
                language: None,
                datatype: None,
            },
            oxrdf::Term::Literal(l) => Term {
                kind: "literal".into(),
                value: l.value().to_string(),
                language: l.language().map(str::to_string),
                datatype: Some(l.datatype().as_str().to_string()),
            },
            // RDF-star quoted triple: keep the N-Triples rendering as the value.
            oxrdf::Term::Triple(t) => Term {
                kind: "triple".into(),
                value: t.to_string(),
                language: None,
                datatype: None,
            },
        }
    }

    /// N-Triples-style rendering (shared by `__repr__`).
    fn n3(&self) -> String {
        match self.kind.as_str() {
            "uri" => format!("<{}>", self.value),
            "bnode" => format!("_:{}", self.value),
            "triple" => self.value.clone(), // already rendered as << s p o >>

            _ => {
                let v = self.value.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
                if let Some(lang) = &self.language {
                    format!("\"{v}\"@{lang}")
                } else {
                    match self.datatype.as_deref() {
                        None | Some(XSD_STRING) => format!("\"{v}\""),
                        Some(dt) => format!("\"{v}\"^^<{dt}>"),
                    }
                }
            }
        }
    }
}

#[pymethods]
impl Term {
    fn __repr__(&self) -> String {
        format!("Term({})", self.n3())
    }

    /// The bare value (IRI / lexical form / label) — handy in f-strings.
    fn __str__(&self) -> String {
        self.value.clone()
    }
}

// ---------------------------------------------------------------------------
// QueryResult
// ---------------------------------------------------------------------------

/// A materialised SELECT result: `.vars` (projection order) and `.rows`
/// (a list of `{var: Term}` dicts; unbound variables are simply absent).
/// Supports `len(result)`, indexing, and iteration (via the sequence protocol).
#[pyclass(frozen, module = "sparq")]
struct QueryResult {
    #[pyo3(get)]
    vars: Vec<String>,
    rows: Py<PyList>,
}

#[pymethods]
impl QueryResult {
    #[getter]
    fn rows(&self, py: Python<'_>) -> Py<PyList> {
        self.rows.clone_ref(py)
    }

    fn __len__(&self, py: Python<'_>) -> usize {
        self.rows.bind(py).len()
    }

    /// Delegates to the underlying list, so negative indices and slices work,
    /// and Python's sequence-protocol fallback makes the result iterable.
    fn __getitem__<'py>(&self, py: Python<'py>, index: Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
        self.rows.bind(py).as_any().get_item(index)
    }

    fn __repr__(&self, py: Python<'_>) -> String {
        format!("QueryResult(vars={:?}, rows={})", self.vars, self.rows.bind(py).len())
    }
}

// ---------------------------------------------------------------------------
// Graph
// ---------------------------------------------------------------------------

/// Resolve `(text, format)` for `Graph.load`: `os.PathLike` is always a file;
/// a `str` is a file iff it names an existing file (no newline in it), else it
/// is parsed as RDF content. The format defaults from the file extension
/// (`.ttl` / `.nt` / `.nq` / `.trig`), falling back to `"turtle"`.
fn resolve_source(source: &Bound<'_, PyAny>, format: Option<&str>) -> PyResult<(String, String)> {
    let from_file = |path: &std::path::Path, format: Option<&str>| -> PyResult<(String, String)> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| PyIOError::new_err(format!("cannot read {}: {e}", path.display())))?;
        let fmt = match format {
            Some(f) => f.to_string(),
            None => match path.extension().and_then(|e| e.to_str()) {
                Some("nt") => "ntriples".into(),
                Some("nq") => "nquads".into(),
                Some("trig") => "trig".into(),
                _ => "turtle".into(),
            },
        };
        Ok((text, fmt))
    };

    if source.is_instance_of::<PyString>() {
        let s: String = source.extract()?;
        // A multi-line string is never a path; a single-line one is a path only
        // if that file actually exists — otherwise it is (single-line) RDF data.
        if !s.contains('\n') && std::path::Path::new(&s).is_file() {
            return from_file(std::path::Path::new(&s), format);
        }
        Ok((s, format.unwrap_or("turtle").to_string()))
    } else {
        // pathlib.Path / any os.PathLike.
        let path: std::path::PathBuf = source.extract().map_err(|_| {
            PyValueError::new_err("Graph.load expects a str (RDF data or file path) or an os.PathLike")
        })?;
        from_file(&path, format)
    }
}

/// All triples of the (default) graph as canonical `[s, p, o]` id rows.
fn all_triples(g: &CoreGraph) -> Vec<[Id; 3]> {
    let scan = g.store.scan(&[None, None, None]);
    scan.rows.iter().map(|r| scan.to_spo(r)).collect()
}

/// An engine `oxrdf::Triple` as a Python `(subject, predicate, object)` tuple
/// of [`Term`]s (the same term convention `Graph.query` rows use).
fn triple_terms(t: &oxrdf::Triple) -> (Term, Term, Term) {
    (
        Term::from_oxrdf(&oxrdf::Term::from(t.subject.clone())),
        Term::from_oxrdf(&oxrdf::Term::from(t.predicate.clone())),
        Term::from_oxrdf(&t.object),
    )
}

/// An immutable, dictionary-encoded RDF graph with SPARQL query / update and
/// opt-in reasoning. Build one with `Graph.load(...)`, `Graph.load_n3(...)`,
/// or `Graph.open(...)`.
#[pyclass(module = "sparq")]
struct Graph {
    inner: CoreGraph,
}

#[pymethods]
impl Graph {
    /// Load RDF from a string of data or from a file path.
    ///
    /// `source`: RDF content (e.g. a Turtle document), a path to an RDF file
    /// (`str` or `os.PathLike`), and `format` one of `"turtle"`, `"ntriples"`,
    /// `"nquads"`, `"trig"` (default: from the file extension, else `"turtle"`).
    /// N-Quads / TriG named graphs are preserved and queryable via `GRAPH`.
    #[staticmethod]
    #[pyo3(signature = (source, format=None))]
    fn load(py: Python<'_>, source: &Bound<'_, PyAny>, format: Option<&str>) -> PyResult<Graph> {
        let (text, fmt) = resolve_source(source, format)?;
        let inner = py
            .detach(|| CoreGraph::load_dataset(&text, &fmt))
            .map_err(engine_err)?;
        Ok(Graph { inner })
    }

    /// Parse a Notation3 document (facts + `{ premise } => { conclusion }` rules)
    /// and forward-chain the rules to fixpoint; the resulting graph contains the
    /// full ground closure. To apply N3 rules to an already-loaded graph instead,
    /// use `Graph.reason_n3_with(rules)`.
    #[staticmethod]
    fn load_n3(py: Python<'_>, text: &str) -> PyResult<Graph> {
        let inner = py
            .detach(|| {
                let mut dict = Dict::new();
                let triples = sparq_reason::reason_n3(&mut dict, text)?;
                Ok::<_, String>(CoreGraph::from_parts(dict, triples))
            })
            .map_err(engine_err)?;
        Ok(Graph { inner })
    }

    /// Open a graph previously persisted with `save(dir)` — the permutation
    /// indexes are memory-mapped (paged in on demand), so opening is near-instant
    /// even for datasets larger than RAM.
    #[staticmethod]
    fn open(py: Python<'_>, dir: std::path::PathBuf) -> PyResult<Graph> {
        let inner = py
            .detach(|| CoreGraph::open(&dir))
            .map_err(|e| PyIOError::new_err(format!("cannot open {}: {e}", dir.display())))?;
        Ok(Graph { inner })
    }

    /// Persist the graph (indexes + dictionary) into `dir` for later `Graph.open`.
    fn save(&self, py: Python<'_>, dir: std::path::PathBuf) -> PyResult<()> {
        py.detach(|| self.inner.save(&dir))
            .map_err(|e| PyIOError::new_err(format!("cannot save to {}: {e}", dir.display())))
    }

    /// Run a SPARQL SELECT, materialising the solutions as a `QueryResult`
    /// (`.vars` + `.rows`, where each row is a `{var: Term}` dict).
    fn query(&self, py: Python<'_>, sparql: &str) -> PyResult<QueryResult> {
        let res = py
            .detach(|| sparq_engine::query(&self.inner, sparql))
            .map_err(engine_err)?;
        let vars: Vec<String> = res.vars.iter().map(|v| v.as_str().to_string()).collect();
        let rows = PyList::empty(py);
        for row in &res.rows {
            let d = PyDict::new(py);
            for (var, cell) in vars.iter().zip(row) {
                if let Some(t) = cell {
                    d.set_item(var, Term::from_oxrdf(t))?;
                }
            }
            rows.append(d)?;
        }
        Ok(QueryResult { vars, rows: rows.unbind() })
    }

    /// Run a SPARQL SELECT and return the SPARQL 1.1 JSON results document as a
    /// `str` — the fast path: rows are serialised straight from the dictionary,
    /// skipping per-cell term materialisation.
    fn query_json(&self, py: Python<'_>, sparql: &str) -> PyResult<String> {
        py.detach(|| sparq_engine::query_json(&self.inner, sparql))
            .map_err(engine_err)
    }

    /// Answer an ASK query (a SELECT is also accepted: True iff it has rows).
    ///
    /// ASK runs on the engine's native entry point (evaluation early-exits at the
    /// first solution); a SELECT is answered by the engine's lazy solution count.
    fn ask(&self, py: Python<'_>, sparql: &str) -> PyResult<bool> {
        let parsed = SparqlParser::new().parse_query(sparql).map_err(|e| engine_err(e.to_string()))?;
        match parsed {
            Query::Ask { .. } => {
                let prepared = parsed.into();
                py.detach(|| sparq_engine::ask_prepared(&self.inner, &prepared)).map_err(engine_err)
            }
            Query::Select { .. } => {
                let prepared = parsed.into();
                let n = py
                    .detach(|| sparq_engine::count_prepared(&self.inner, &prepared))
                    .map_err(engine_err)?;
                Ok(n > 0)
            }
            _ => Err(PyValueError::new_err("ask() takes an ASK (or SELECT) query")),
        }
    }

    /// Run a SPARQL CONSTRUCT, returning the constructed graph as a list of
    /// `(subject, predicate, object)` `Term` triples — a deduplicated set in
    /// first-production order (per SPARQL 1.1 §16.2: template triples with an
    /// unbound variable or an illegal RDF position are silently dropped, and
    /// template blank nodes are fresh per solution).
    fn construct(&self, py: Python<'_>, sparql: &str) -> PyResult<Vec<(Term, Term, Term)>> {
        let triples = py
            .detach(|| sparq_engine::construct(&self.inner, sparql))
            .map_err(engine_err)?;
        Ok(triples.iter().map(triple_terms).collect())
    }

    /// Run a SPARQL DESCRIBE, returning the union of the concise bounded
    /// descriptions (CBD: every triple whose subject is the resource, recursing
    /// through blank-node objects) of each described resource, as a list of
    /// `(subject, predicate, object)` `Term` triples.
    fn describe(&self, py: Python<'_>, sparql: &str) -> PyResult<Vec<(Term, Term, Term)>> {
        let triples = py
            .detach(|| sparq_engine::describe(&self.inner, sparql))
            .map_err(engine_err)?;
        Ok(triples.iter().map(triple_terms).collect())
    }

    /// Apply a SPARQL Update (`INSERT DATA` / `DELETE DATA` / `DELETE/INSERT …
    /// WHERE` / `CLEAR` / `DROP` / `CREATE` / `ADD` / `COPY` / `MOVE`) in place.
    /// The engine's store is immutable, so the update produces a NEW graph which
    /// this wrapper swaps in; on error the graph is left unchanged. The full
    /// dataset is modelled: `GRAPH`-scoped data and templates, `USING (NAMED)`,
    /// and the graph-management operations all work on named graphs.
    fn update(&mut self, py: Python<'_>, sparql: &str) -> PyResult<()> {
        let inner = &self.inner;
        let new = py.detach(|| sparq_engine::update(inner, sparql)).map_err(engine_err)?;
        self.inner = new;
        Ok(())
    }

    /// Materialize the entailed closure for `profile` (`"rdfs"` or `"owl"`)
    /// in place, returning the number of NEW triples added. Idempotent.
    /// Reasoning runs over the default graph; named graphs are carried across
    /// the rebuild untouched.
    ///
    /// `"n3"` is not valid here — N3 rules live in an N3 document, so use
    /// `Graph.load_n3(text)` (rules + facts in one document) or
    /// `Graph.reason_n3_with(rules)` (rules applied to this graph) instead.
    fn reason(&mut self, py: Python<'_>, profile: &str) -> PyResult<usize> {
        if profile.eq_ignore_ascii_case("n3") {
            return Err(PyValueError::new_err(
                "N3 rules live in an N3 document; use Graph.load_n3(text) or Graph.reason_n3_with(rules) instead of reason(\"n3\")",
            ));
        }
        let prof = sparq_reason::Profile::parse(profile).ok_or_else(|| {
            PyValueError::new_err(format!("unknown reasoning profile {profile:?} (known: \"rdfs\", \"owl\")"))
        })?;
        // Take the graph apart through its public fields (no Clone on Dict/Graph):
        // move `dict` and `store` out, re-derive the canonical triples, materialize,
        // rebuild (re-attaching the named graphs, which reasoning does not touch).
        // A placeholder empty graph holds the slot during the closure.
        let g = std::mem::replace(&mut self.inner, CoreGraph::from_parts(Dict::new(), Vec::new()));
        let (inner, added) = py.detach(move || {
            let mut triples = all_triples(&g);
            let named = g.named;
            let mut dict = g.dict;
            let added = sparq_reason::materialize(prof, &mut dict, &mut triples);
            let mut rebuilt = CoreGraph::from_parts(dict, triples);
            rebuilt.named = named;
            (rebuilt, added)
        });
        self.inner = inner;
        Ok(added)
    }

    /// Forward-chain caller-supplied Notation3 `rules` (an N3 document: `{ premise }
    /// => { conclusion }` rules, plus any facts it contains) over THIS graph's
    /// default graph, in place, returning the number of NEW triples added.
    /// Named graphs are carried across the rebuild untouched; on error (e.g. a
    /// rules parse failure) the graph is left unchanged.
    ///
    /// The graph's triples join the document as ground facts (blank nodes keep
    /// their labels, so a label shared with the rules document denotes the same
    /// node — N3 merge semantics). RDF-star triple terms have no N3 form and are
    /// rejected.
    fn reason_n3_with(&mut self, py: Python<'_>, rules: &str) -> PyResult<usize> {
        use std::fmt::Write as _;
        let inner = &self.inner;
        let before = inner.len();
        // Render the default graph as N-Triples (a syntactic subset of N3) under
        // the rules document and run the same chainer as `load_n3` — exactly the
        // composition sparq-reason's own MaterializedN3Graph uses in fallback mode.
        let mut new = py
            .detach(|| {
                let rows = all_triples(inner);
                let mut src = String::with_capacity(rules.len() + 1 + 64 * rows.len());
                src.push_str(rules);
                src.push('\n');
                for [s, p, o] in rows {
                    for id in [s, p, o] {
                        let term = inner.dict.term(id);
                        if matches!(term, oxrdf::Term::Triple(_)) {
                            return Err("RDF-star triple terms cannot participate in N3 reasoning".into());
                        }
                        let _ = write!(src, "{term} ");
                    }
                    src.push_str(".\n");
                }
                let mut dict = Dict::new();
                let triples = sparq_reason::reason_n3(&mut dict, &src)?;
                Ok::<_, String>(CoreGraph::from_parts(dict, triples))
            })
            .map_err(engine_err)?;
        new.named = std::mem::take(&mut self.inner.named);
        let added = new.len().saturating_sub(before);
        self.inner = new;
        Ok(added)
    }

    /// Detect OWL 2 RL inconsistencies (clashes) in the default graph, returning
    /// one human-readable description per clash (empty list = no detected
    /// inconsistency). Covers the OWL 2 RL false rules: cax-dw (disjointWith) /
    /// cax-adc (AllDisjointClasses), cls-com (complementOf), cls-nothing
    /// (owl:Nothing instances), cls-maxc1 / cls-maxqc1/2 (cardinality-0
    /// violations), eq-diff1/2/3 (sameAs vs differentFrom / AllDifferent,
    /// including sameAs forced between distinct literal values), prp-asyp /
    /// prp-irp (asymmetric & irreflexive violations), prp-pdw / prp-adp
    /// (disjoint properties), and prp-npa1/2 (negative property assertions).
    ///
    /// Detection is over ASSERTED triples: run `reason("owl")` first to surface
    /// clashes that only follow by entailment.
    fn inconsistencies(&self, py: Python<'_>) -> PyResult<Vec<String>> {
        let inner = &self.inner;
        Ok(py.detach(|| sparq_reason::inconsistencies(&inner.dict, &all_triples(inner))))
    }

    /// Number of triples in the default graph.
    fn __len__(&self) -> usize {
        self.inner.len()
    }

    fn __repr__(&self) -> String {
        format!("Graph({} triples)", self.inner.len())
    }
}

// ---------------------------------------------------------------------------
// Module
// ---------------------------------------------------------------------------

/// The `sparq` Python module.
#[pymodule]
fn sparq(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Graph>()?;
    m.add_class::<QueryResult>()?;
    m.add_class::<Term>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
