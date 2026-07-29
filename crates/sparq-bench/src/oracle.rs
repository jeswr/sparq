//! A **pluggable SPARQL oracle** abstraction, so the differential harness compares
//! *neutral normalised results* rather than one engine's own types.
//!
//! This is **node F** of the value-level multi-oracle differential-testing DAG designed in
//! `research/differential-testing-value-level.md` (bead `sq-qcnn.8`, epic `sq-qcnn`), and it
//! depends on node A (`sparq-difftest`, bead `sq-qcnn.4`) for the neutral result form.
//!
//! # Why a second oracle at all
//!
//! The harness today checks sparq against exactly one reference implementation (Oxigraph). That
//! catches a sparq-only bug, but it is *structurally blind* to a bug the two implementations
//! **share** — a misread of a spec clause both authors made, or an ambiguity both resolved the
//! same way. A second engine with **no shared lineage** is the mitigation: when sparq and
//! Oxigraph agree and the third engine does not, something is worth a human look.
//!
//! Stated honestly, and this bounds every claim below: **a second oracle is not an oracle of
//! truth.** Apache Jena and rdflib are *implementations*, not the specification. A disagreement
//! between two oracles is evidence of a spec ambiguity or of one engine's non-conformance — it is
//! **not** attributable to sparq, and this module never treats it as a sparq defect. Deciding
//! such a case is a human, reviewed act; the N-way triage + reviewed-allowlist machinery that
//! records those decisions is design record §5.2 and a **separate** follow-on bead, not this one.
//!
//! # What is here
//!
//! * [`OracleResult`] / [`OracleError`] — the neutral, engine-independent result and failure form.
//! * [`Oracle`] — the trait every oracle implements (`&dyn Oracle`-safe, so the harness holds a
//!   heterogeneous set without knowing which engine is which).
//! * [`OxigraphOracle`] — the existing in-process oracle, refactored behind the trait. Adds **no**
//!   dependency: `oxigraph` was already this crate's reference engine.
//! * [`SubprocessOracle`] — a generic adapter that shells out to an external harness which emits
//!   **SPARQL Results JSON** on stdout. The Apache Jena (Java) and rdflib (Python) harnesses in
//!   `crates/sparq-bench/oracles/` are configurations of this one adapter; see that directory's
//!   README for the wire protocol, build steps and the honest verification status.
//!
//! # Opt-in, and why by environment rather than a cargo feature
//!
//! Every non-Oxigraph oracle is **off by default**: absent [`ENV_CMD`], a `sparq-bench fuzz` run
//! performs exactly the comparisons it performed before, spawns no process, and needs no JVM and
//! no Python. (It does print one extra banner line naming the second oracle as disabled, so a log
//! always says which oracles a run actually consulted.) The opt-in is an environment variable
//! rather than a cargo feature *because a cargo feature would
//! buy nothing here*: the adapter is built on `std::process` alone and pulls in no dependency, so
//! there is no lean-build surface for a feature flag to protect. What actually needs gating is the
//! **runtime toolchain** (a JVM, a jar, a Python interpreter), which a compile-time feature cannot
//! express. This echoes `sparq-difftest`'s own reasoning for carrying no feature flag: where there
//! is no lean-build surface to protect, a flag adds a build configuration and buys nothing.

use std::cell::Cell;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use oxigraph::store::Store;
use sparq_difftest::json::Solution;
use sparq_difftest::{parse_results_json, QueryResults, Term};

/// The command line for the second oracle: program plus any fixed leading arguments, separated by
/// whitespace. **Unset or empty means no second oracle** (the default). The adapter appends two
/// arguments of its own — the data file and the query file — so a value of
/// `java -cp jena.jar:. SparqlOracle` is invoked as
/// `java -cp jena.jar:. SparqlOracle <data.ttl> <query.rq>`.
pub const ENV_CMD: &str = "SPARQ_FUZZ_ORACLE2_CMD";

/// Display name for the second oracle in the harness output (default: the program's file name).
/// Purely cosmetic — it selects nothing.
pub const ENV_NAME: &str = "SPARQ_FUZZ_ORACLE2_NAME";

/// Per-query wall-clock budget for the second oracle, in seconds (default
/// [`DEFAULT_TIMEOUT_SECS`]). A value that does not parse as a positive integer is ignored with a
/// warning and the default is used — a typo must not silently disable the timeout.
pub const ENV_TIMEOUT: &str = "SPARQ_FUZZ_ORACLE2_TIMEOUT_SECS";

/// Default per-query wall-clock budget for a subprocess oracle. Generous, because it has to
/// absorb JVM start-up (the dominant cost for the Jena harness), not just evaluation.
pub const DEFAULT_TIMEOUT_SECS: u64 = 60;

/// The exit code an external harness uses to say **"I cannot evaluate this query"** (a parse
/// error, an unimplemented function, a result form this adapter does not carry). It is a
/// SKIP-for-this-oracle, never a divergence — see [`OracleError::Unsupported`]. Any *other*
/// non-zero exit is a backend failure, which is deliberately a different bucket: "the oracle
/// declined" and "the oracle broke" must not be the same number.
pub const UNSUPPORTED_EXIT_CODE: i32 = 3;

/// How often the subprocess adapter polls for child exit while waiting out its timeout.
const POLL_INTERVAL: Duration = Duration::from_millis(5);

/// A neutral, engine-independent SPARQL result — the *only* thing oracles hand back, so the
/// comparator never sees an engine-specific type.
///
/// The variants mirror SPARQL's three result forms. Note what is deliberately **not** carried:
/// the `SELECT` header variable list. A [`Solution`] models a projected-but-unbound variable by
/// the variable's *absence*, and the comparison this feeds ([`sparq_difftest::multiset_equal`])
/// is over bound variables only, so the header would decide nothing. The cost is real and worth
/// naming: two engines that project **different variable lists** but bind the same terms compare
/// equal here. Catching that needs the header, and belongs with the wiring node that has a use
/// for it.
#[derive(Debug, Clone)]
pub enum OracleResult {
    /// A `SELECT` result: the solution **multiset** (SPARQL is bag semantics, so duplicates are
    /// significant and this is a `Vec`, not a set).
    Solutions(Vec<Solution>),
    /// A `CONSTRUCT` / `DESCRIBE` result, as `[subject, predicate, object]` triples in the neutral
    /// term model. Blank-node-bearing graphs are comparable only up to isomorphism
    /// ([`sparq_difftest::graph_isomorphic`]), never by label.
    Graph(Vec<[Term; 3]>),
    /// An `ASK` result.
    Boolean(bool),
}

/// Why an oracle produced no result. The three variants exist because they must be **counted
/// separately**: only one of them is a reason to look at sparq.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OracleError {
    /// The oracle declined the query — a parse error, an unimplemented feature, or a result form
    /// this adapter does not carry. A **skip for this oracle**, never a divergence: an engine that
    /// cannot answer a question has not answered it wrongly.
    Unsupported(String),
    /// The oracle rejected the **data** (a parse failure on the input graph). Distinct from
    /// `Unsupported` because both engines are handed the *same* serialisation, so one of them
    /// refusing it is itself a finding.
    Data(String),
    /// The oracle broke: spawn failure, timeout, a non-zero exit that is not
    /// [`UNSUPPORTED_EXIT_CODE`], or output that is not SPARQL Results JSON. A harness fault, not
    /// a statement about either engine's semantics.
    Backend(String),
}

impl fmt::Display for OracleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OracleError::Unsupported(m) => write!(f, "unsupported: {}", m),
            OracleError::Data(m) => write!(f, "data: {}", m),
            OracleError::Backend(m) => write!(f, "backend: {}", m),
        }
    }
}

impl std::error::Error for OracleError {}

/// An interchangeable SPARQL implementation the harness can consult.
///
/// Object-safe on purpose: the harness holds `&dyn Oracle` so adding an engine is a
/// configuration change, not a code change at every comparison site.
pub trait Oracle {
    /// The oracle's name, for output and for keying a triage record.
    fn name(&self) -> &str;

    /// Evaluate `query` over `data` (Turtle). `Err` is *not* a divergence — see [`OracleError`].
    fn eval(&self, data: &str, query: &str) -> Result<OracleResult, OracleError>;
}

/// Convert one Oxigraph term into the neutral term model.
///
/// Both engines in this harness materialise the same `oxrdf` term type, so this one function is
/// the single crossing point from an engine-specific model to the neutral one. It is a pure
/// re-shaping: it decides **no** equality (that is `sparq-difftest`'s job, deliberately, so the
/// engine under test never gets to define what counts as equal to itself).
pub fn neutral_term(t: &oxigraph::model::Term) -> Term {
    use oxigraph::model::Term as OxTerm;
    match t {
        OxTerm::NamedNode(n) => Term::Iri(n.as_str().to_string()),
        OxTerm::BlankNode(b) => Term::Blank(b.as_str().to_string()),
        OxTerm::Literal(l) => Term::Literal {
            lexical: l.value().to_string(),
            datatype: l.datatype().as_str().to_string(),
            lang: l.language().map(str::to_string),
        },
        OxTerm::Triple(inner) => Term::Triple(Box::new([
            neutral_term(&OxTerm::from(inner.subject.clone())),
            Term::Iri(inner.predicate.as_str().to_string()),
            neutral_term(&inner.object),
        ])),
    }
}

/// The **in-process** oracle: Oxigraph, behind the [`Oracle`] trait.
///
/// Adds no dependency — `oxigraph` is already this crate's reference engine — and no behaviour:
/// this is the loading + querying the fuzzer already did, moved behind the trait so a second
/// engine can stand beside it.
#[derive(Debug, Default, Clone, Copy)]
pub struct OxigraphOracle;

impl OxigraphOracle {
    /// A new Oxigraph oracle. Stateless: each evaluation gets its own store.
    pub fn new() -> Self {
        OxigraphOracle
    }

    /// Parse `data` (Turtle) into a fresh in-memory Oxigraph store.
    ///
    /// Exposed separately from [`Oracle::eval`] so the fuzzer can load a generated graph **once**
    /// and run several queries against it, rather than re-parsing per query: the trait's
    /// `eval(data, query)` shape is right for an interchangeable oracle but would otherwise turn
    /// the hot loop's one parse per graph into one parse per query.
    pub fn load(&self, data: &str) -> Result<Store, OracleError> {
        let store = Store::new()
            .map_err(|e| OracleError::Backend(format!("oxigraph store: {}", e)))?;
        store
            .load_from_reader(oxigraph::io::RdfFormat::Turtle, data.as_bytes())
            .map_err(|e| OracleError::Data(format!("oxigraph load error: {}", e)))?;
        Ok(store)
    }

    /// Evaluate `query` against an already-loaded store, projecting into the neutral form.
    // clippy: differential oracle pins oxigraph's legacy Store::query semantics
    #[allow(deprecated)]
    fn eval_on(&self, store: &Store, query: &str) -> Result<OracleResult, OracleError> {
        // Oxigraph does not separate "I cannot parse/support this" from "evaluation failed" in its
        // error type, and the fuzzer has always treated an Oxigraph query error as a skip
        // (`skipped_unsupported`, via `oxi_count`). Classifying it as `Unsupported` here keeps the
        // trait's meaning of the error consistent with the harness's existing reading of it.
        let results = store
            .query(query)
            .map_err(|e| OracleError::Unsupported(format!("oxigraph: {}", e)))?;
        match results {
            oxigraph::sparql::QueryResults::Boolean(b) => Ok(OracleResult::Boolean(b)),
            oxigraph::sparql::QueryResults::Solutions(iter) => {
                let mut out: Vec<Solution> = Vec::new();
                for sol in iter {
                    let sol = sol.map_err(|e| {
                        OracleError::Backend(format!("oxigraph solution: {}", e))
                    })?;
                    out.push(
                        sol.iter()
                            .map(|(v, t)| (v.as_str().to_string(), neutral_term(t)))
                            .collect(),
                    );
                }
                Ok(OracleResult::Solutions(out))
            }
            oxigraph::sparql::QueryResults::Graph(iter) => {
                let mut out: Vec<[Term; 3]> = Vec::new();
                for triple in iter {
                    let triple = triple
                        .map_err(|e| OracleError::Backend(format!("oxigraph triple: {}", e)))?;
                    out.push([
                        neutral_term(&oxigraph::model::Term::from(triple.subject)),
                        Term::Iri(triple.predicate.as_str().to_string()),
                        neutral_term(&triple.object),
                    ]);
                }
                Ok(OracleResult::Graph(out))
            }
        }
    }
}

impl Oracle for OxigraphOracle {
    fn name(&self) -> &str {
        "oxigraph"
    }

    fn eval(&self, data: &str, query: &str) -> Result<OracleResult, OracleError> {
        let store = self.load(data)?;
        self.eval_on(&store, query)
    }
}

/// An oracle that is an **external process** emitting SPARQL Results JSON on stdout.
///
/// One adapter serves every out-of-process engine: Apache Jena (the recommended second oracle,
/// via a small Java harness over jena-arq) and rdflib (optional, cheaper to provision but less
/// conformant, so it produces more to triage) differ only in the command line configured in
/// [`ENV_CMD`].
///
/// # Wire protocol
///
/// The adapter invokes `<program> <fixed args…> <data.ttl> <query.rq>` and expects:
///
/// * exit `0` — stdout is a SPARQL-Results-JSON document (`SELECT` bindings or an `ASK` boolean);
/// * exit [`UNSUPPORTED_EXIT_CODE`] — the harness declined the query ⇒ [`OracleError::Unsupported`];
/// * any other exit, a timeout, or unparseable stdout ⇒ [`OracleError::Backend`].
///
/// `CONSTRUCT` / `DESCRIBE` are **not** carried over this protocol: SPARQL Results JSON has no
/// graph form, so a harness must decline them with [`UNSUPPORTED_EXIT_CODE`]. Comparing graph
/// results across a subprocess oracle needs an N-Triples channel plus RDFC-1.0 isomorphism and is
/// left to the wiring node that needs it, rather than half-built here.
///
/// Data and query are passed as **files, not stdin**, and stdout/stderr are redirected to files
/// rather than pipes. That is deliberate: reading a pipe while also polling for exit either
/// deadlocks when the child fills the pipe buffer or needs a reader thread per stream, and this
/// adapter must be able to kill a child that hangs.
pub struct SubprocessOracle {
    name: String,
    program: String,
    args: Vec<String>,
    timeout: Duration,
    /// Serial number for temp-file names. A counter (not a clock or RNG) keeps the fuzzer's
    /// seed-only reproducibility intact — nothing here varies run to run.
    seq: Cell<u64>,
}

impl SubprocessOracle {
    /// Build an adapter from an explicit program + fixed arguments.
    pub fn new(
        name: impl Into<String>,
        program: impl Into<String>,
        args: Vec<String>,
        timeout: Duration,
    ) -> Self {
        SubprocessOracle {
            name: name.into(),
            program: program.into(),
            args,
            timeout,
            seq: Cell::new(0),
        }
    }

    /// Parse a whitespace-separated command line. `None` when it is empty (which is how "no
    /// second oracle" is spelled). Whitespace splitting means a path containing a space cannot be
    /// expressed — an acceptable limit for a harness knob, and a loud one: the spawn simply fails
    /// with the mis-split program name rather than silently running something else.
    pub fn from_command_line(cmd: &str, name: Option<&str>, timeout: Duration) -> Option<Self> {
        let mut parts = cmd.split_whitespace().map(str::to_string);
        let program = parts.next()?;
        let args: Vec<String> = parts.collect();
        let name = name
            .filter(|n| !n.trim().is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| {
                Path::new(&program)
                    .file_name()
                    .map(|f| f.to_string_lossy().into_owned())
                    .unwrap_or_else(|| program.clone())
            });
        Some(SubprocessOracle::new(name, program, args, timeout))
    }

    /// Build from already-read environment values. Split out from [`Self::from_env`] so the
    /// configuration rules — including the "a bad timeout falls back to the default rather than
    /// disabling the timeout" rule — are testable without mutating process environment.
    pub fn from_env_parts(
        cmd: Option<&str>,
        name: Option<&str>,
        timeout_secs: Option<&str>,
    ) -> Option<Self> {
        let cmd = cmd?;
        let timeout = match timeout_secs.map(str::trim).filter(|s| !s.is_empty()) {
            None => Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            Some(s) => match s.parse::<u64>() {
                Ok(n) if n > 0 => Duration::from_secs(n),
                _ => {
                    eprintln!(
                        "warning: {}={:?} is not a positive integer — using the {}s default",
                        ENV_TIMEOUT, s, DEFAULT_TIMEOUT_SECS
                    );
                    Duration::from_secs(DEFAULT_TIMEOUT_SECS)
                }
            },
        };
        SubprocessOracle::from_command_line(cmd, name, timeout)
    }

    /// Read the second-oracle configuration from the environment. `None` — the default — means
    /// the harness runs exactly as it did before this adapter existed.
    pub fn from_env() -> Option<Self> {
        let cmd = std::env::var(ENV_CMD).ok();
        let name = std::env::var(ENV_NAME).ok();
        let timeout = std::env::var(ENV_TIMEOUT).ok();
        SubprocessOracle::from_env_parts(cmd.as_deref(), name.as_deref(), timeout.as_deref())
    }

    /// A one-line description for the harness banner (name, command, timeout).
    pub fn describe(&self) -> String {
        format!(
            "{} [{} {}] timeout={}s",
            self.name,
            self.program,
            self.args.join(" "),
            self.timeout.as_secs()
        )
    }

    /// Run the child to completion or to the timeout, returning its exit code and stdout.
    fn run(&self, scratch: &Scratch) -> io::Result<Outcome> {
        let stdout = fs::File::create(&scratch.out)?;
        let stderr = fs::File::create(&scratch.err)?;
        let mut child = Command::new(&self.program)
            .args(&self.args)
            .arg(&scratch.data)
            .arg(&scratch.query)
            .stdin(Stdio::null())
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr))
            .spawn()?;

        let started = Instant::now();
        let status = loop {
            match child.try_wait()? {
                Some(status) => break status,
                None if started.elapsed() >= self.timeout => {
                    // Kill AND reap: a killed-but-unwaited child is a zombie, and the fuzzer runs
                    // thousands of cases per shard.
                    child.kill()?;
                    child.wait()?;
                    return Ok(Outcome::TimedOut);
                }
                None => std::thread::sleep(POLL_INTERVAL),
            }
        };
        Ok(Outcome::Exited {
            code: status.code(),
            stdout: fs::read_to_string(&scratch.out).unwrap_or_default(),
            stderr: fs::read_to_string(&scratch.err).unwrap_or_default(),
        })
    }
}

/// What a child process did.
enum Outcome {
    Exited {
        code: Option<i32>,
        stdout: String,
        stderr: String,
    },
    TimedOut,
}

/// The four temp files one evaluation needs, deleted on drop so a long fuzz shard does not fill
/// the temp directory even when a case fails early.
struct Scratch {
    data: PathBuf,
    query: PathBuf,
    out: PathBuf,
    err: PathBuf,
}

impl Scratch {
    fn new(seq: u64) -> Self {
        let dir = std::env::temp_dir();
        let stem = format!("sparq-oracle2-{}-{}", std::process::id(), seq);
        Scratch {
            data: dir.join(format!("{}.ttl", stem)),
            query: dir.join(format!("{}.rq", stem)),
            out: dir.join(format!("{}.out", stem)),
            err: dir.join(format!("{}.err", stem)),
        }
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        for p in [&self.data, &self.query, &self.out, &self.err] {
            let _ = fs::remove_file(p);
        }
    }
}

impl Oracle for SubprocessOracle {
    fn name(&self) -> &str {
        &self.name
    }

    fn eval(&self, data: &str, query: &str) -> Result<OracleResult, OracleError> {
        let seq = self.seq.get();
        self.seq.set(seq.wrapping_add(1));
        let scratch = Scratch::new(seq);

        fs::write(&scratch.data, data)
            .and_then(|()| fs::write(&scratch.query, query))
            .map_err(|e| OracleError::Backend(format!("writing oracle input: {}", e)))?;

        let outcome = self
            .run(&scratch)
            .map_err(|e| OracleError::Backend(format!("spawning {}: {}", self.program, e)))?;

        let (code, stdout, stderr) = match outcome {
            Outcome::TimedOut => {
                return Err(OracleError::Backend(format!(
                    "{} exceeded its {}s budget",
                    self.name,
                    self.timeout.as_secs()
                )))
            }
            Outcome::Exited {
                code,
                stdout,
                stderr,
            } => (code, stdout, stderr),
        };

        match code {
            Some(0) => parse_oracle_json(&stdout),
            Some(c) if c == UNSUPPORTED_EXIT_CODE => Err(OracleError::Unsupported(format!(
                "{} declined: {}",
                self.name,
                stderr.trim()
            ))),
            // A signal-terminated child reports `None`, which is a backend fault, not a decline.
            Some(c) => Err(OracleError::Backend(format!(
                "{} exited {}: {}",
                self.name,
                c,
                stderr.trim()
            ))),
            None => Err(OracleError::Backend(format!(
                "{} terminated by signal: {}",
                self.name,
                stderr.trim()
            ))),
        }
    }
}

/// Parse an external harness's stdout into the neutral result form.
///
/// Split out from [`Oracle::eval`] so the JSON contract is directly testable without spawning a
/// process. Unparseable output is [`OracleError::Backend`], never `Unsupported`: a harness that
/// exited `0` claimed to have answered, so garbage on stdout is a broken harness — classifying it
/// as a decline would let a silently broken oracle read as an innocuous skip.
pub fn parse_oracle_json(stdout: &str) -> Result<OracleResult, OracleError> {
    match parse_results_json(stdout) {
        Ok(QueryResults::Solutions { solutions, .. }) => Ok(OracleResult::Solutions(solutions)),
        Ok(QueryResults::Boolean(b)) => Ok(OracleResult::Boolean(b)),
        Err(e) => Err(OracleError::Backend(format!(
            "oracle exited 0 but its stdout is not SPARQL Results JSON: {}",
            e
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxigraph::model::{BlankNode, Literal, NamedNode, Term as OxTerm};

    const GRAPH: &str = r#"@prefix ex: <http://ex/> .
        ex:a ex:p 1 ; ex:q "x" .
        ex:b ex:p 2 .
    "#;

    /// A stub harness script, deleted when the test's binding goes out of scope.
    struct Stub(PathBuf);

    impl Drop for Stub {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }

    /// A stub external harness, written to a temp file, so the subprocess adapter is tested
    /// against a real child process (argv passing, stdout capture, exit-code classification)
    /// without needing a JVM or rdflib on the box.
    fn stub_with_timeout(body: &str, timeout: Duration) -> (Stub, SubprocessOracle) {
        static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "sparq-oracle-stub-{}-{}.py",
            std::process::id(),
            N.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        fs::write(&path, body).expect("write stub");
        let oracle = SubprocessOracle::new(
            "stub",
            "python3",
            vec![path.to_string_lossy().into_owned()],
            timeout,
        );
        (Stub(path), oracle)
    }

    fn stub(body: &str) -> (Stub, SubprocessOracle) {
        stub_with_timeout(body, Duration::from_secs(30))
    }

    #[test]
    fn neutral_term_maps_every_oxigraph_term_shape() {
        assert!(matches!(
            neutral_term(&OxTerm::NamedNode(NamedNode::new("http://ex/a").unwrap())),
            Term::Iri(i) if i == "http://ex/a"
        ));
        assert!(matches!(
            neutral_term(&OxTerm::BlankNode(BlankNode::new("b0").unwrap())),
            Term::Blank(l) if l == "b0"
        ));
        let lit = neutral_term(&OxTerm::Literal(Literal::new_typed_literal(
            "1",
            NamedNode::new("http://www.w3.org/2001/XMLSchema#integer").unwrap(),
        )));
        match lit {
            Term::Literal {
                lexical,
                datatype,
                lang,
            } => {
                assert_eq!(lexical, "1");
                assert_eq!(datatype, "http://www.w3.org/2001/XMLSchema#integer");
                assert_eq!(lang, None);
            }
            other => panic!("expected a literal, got {:?}", other),
        }
        let tagged = neutral_term(&OxTerm::Literal(
            Literal::new_language_tagged_literal("hi", "en").unwrap(),
        ));
        assert!(matches!(tagged, Term::Literal { lang: Some(l), .. } if l == "en"));
    }

    #[test]
    fn oxigraph_oracle_evaluates_select_ask_and_construct() {
        let oracle = OxigraphOracle::new();
        assert_eq!(oracle.name(), "oxigraph");

        let sel = oracle
            .eval(GRAPH, "SELECT ?s WHERE { ?s <http://ex/p> ?o }")
            .expect("select");
        match sel {
            OracleResult::Solutions(rows) => assert_eq!(rows.len(), 2),
            other => panic!("expected solutions, got {:?}", other),
        }

        let ask = oracle
            .eval(GRAPH, "ASK { <http://ex/a> <http://ex/q> \"x\" }")
            .expect("ask");
        assert!(matches!(ask, OracleResult::Boolean(true)));

        let con = oracle
            .eval(
                GRAPH,
                "CONSTRUCT { ?s <http://ex/r> ?o } WHERE { ?s <http://ex/p> ?o }",
            )
            .expect("construct");
        match con {
            OracleResult::Graph(triples) => {
                assert_eq!(triples.len(), 2);
                assert!(matches!(&triples[0][1], Term::Iri(i) if i == "http://ex/r"));
            }
            other => panic!("expected a graph, got {:?}", other),
        }
    }

    #[test]
    fn oxigraph_oracle_separates_bad_data_from_an_unsupported_query() {
        let oracle = OxigraphOracle::new();
        assert!(matches!(
            oracle.eval("this is not turtle @@@", "SELECT * WHERE { ?s ?p ?o }"),
            Err(OracleError::Data(_))
        ));
        assert!(matches!(
            oracle.eval(GRAPH, "SELEKT ?s WHERE {"),
            Err(OracleError::Unsupported(_))
        ));
    }

    #[test]
    fn oxigraph_oracle_load_is_reusable_across_queries() {
        let oracle = OxigraphOracle::new();
        let store = oracle.load(GRAPH).expect("load");
        // The whole point of exposing `load`: two evaluations, one parse.
        for q in [
            "SELECT ?s WHERE { ?s <http://ex/p> ?o }",
            "SELECT ?o WHERE { ?s <http://ex/q> ?o }",
        ] {
            assert!(matches!(
                oracle.eval_on(&store, q),
                Ok(OracleResult::Solutions(_))
            ));
        }
    }

    #[test]
    fn parse_oracle_json_reads_bindings_and_booleans_and_rejects_garbage() {
        let sol = parse_oracle_json(
            r#"{"head":{"vars":["s"]},"results":{"bindings":[{"s":{"type":"uri","value":"http://ex/a"}}]}}"#,
        )
        .expect("bindings");
        match sol {
            OracleResult::Solutions(rows) => {
                assert_eq!(rows.len(), 1);
                assert!(matches!(rows[0].get("s"), Some(Term::Iri(i)) if i == "http://ex/a"));
            }
            other => panic!("expected solutions, got {:?}", other),
        }
        assert!(matches!(
            parse_oracle_json(r#"{"head":{},"boolean":true}"#),
            Ok(OracleResult::Boolean(true))
        ));
        // Exited 0 but wrote nonsense: a BROKEN harness, not a decline.
        assert!(matches!(
            parse_oracle_json("not json at all"),
            Err(OracleError::Backend(_))
        ));
    }

    #[test]
    fn from_command_line_splits_program_and_args_and_defaults_the_name() {
        let o = SubprocessOracle::from_command_line(
            "/usr/bin/java -cp jena.jar SparqlOracle",
            None,
            Duration::from_secs(5),
        )
        .expect("configured");
        assert_eq!(o.program, "/usr/bin/java");
        assert_eq!(o.args, vec!["-cp", "jena.jar", "SparqlOracle"]);
        assert_eq!(o.name(), "java", "the name defaults to the program basename");

        let named =
            SubprocessOracle::from_command_line("java", Some("jena"), Duration::from_secs(5))
                .expect("configured");
        assert_eq!(named.name(), "jena");
        assert!(named.describe().contains("jena"));
    }

    #[test]
    fn from_env_parts_is_off_by_default_and_falls_back_on_a_bad_timeout() {
        assert!(
            SubprocessOracle::from_env_parts(None, None, None).is_none(),
            "no command configured must mean NO second oracle"
        );
        assert!(
            SubprocessOracle::from_env_parts(Some("   "), None, None).is_none(),
            "a whitespace-only command has no program to run"
        );
        let default = SubprocessOracle::from_env_parts(Some("python3"), None, None).unwrap();
        assert_eq!(default.timeout, Duration::from_secs(DEFAULT_TIMEOUT_SECS));
        let explicit =
            SubprocessOracle::from_env_parts(Some("python3"), None, Some("7")).unwrap();
        assert_eq!(explicit.timeout, Duration::from_secs(7));
        // A typo must NOT disable the timeout — that would let a hung oracle hang the shard.
        for bad in ["0", "-1", "banana"] {
            let o = SubprocessOracle::from_env_parts(Some("python3"), None, Some(bad)).unwrap();
            assert_eq!(
                o.timeout,
                Duration::from_secs(DEFAULT_TIMEOUT_SECS),
                "{:?} must fall back to the default, not to no timeout",
                bad
            );
        }
    }

    #[test]
    fn subprocess_oracle_reads_results_json_from_a_child_process() {
        let (_p, oracle) = stub(
            r#"import sys
data, query = sys.argv[1], sys.argv[2]
assert open(data).read().startswith("@prefix"), "data file not passed through"
assert "SELECT" in open(query).read(), "query file not passed through"
print('{"head":{"vars":["s"]},"results":{"bindings":[{"s":{"type":"uri","value":"http://ex/a"}}]}}')
"#,
        );
        match oracle.eval(GRAPH, "SELECT ?s WHERE { ?s ?p ?o }") {
            Ok(OracleResult::Solutions(rows)) => {
                assert_eq!(rows.len(), 1);
                assert!(matches!(rows[0].get("s"), Some(Term::Iri(i)) if i == "http://ex/a"));
            }
            other => panic!("expected one solution from the stub, got {:?}", other),
        }
    }

    #[test]
    fn subprocess_oracle_classifies_decline_break_and_hang_differently() {
        let (_p1, declines) = stub("import sys; sys.exit(3)\n");
        assert!(
            matches!(
                declines.eval(GRAPH, "SELECT * WHERE { ?s ?p ?o }"),
                Err(OracleError::Unsupported(_))
            ),
            "exit {} is the DECLINE code — a skip, not a divergence",
            UNSUPPORTED_EXIT_CODE
        );

        let (_p2, breaks) = stub("import sys\nsys.stderr.write('boom')\nsys.exit(1)\n");
        assert!(matches!(
            breaks.eval(GRAPH, "SELECT * WHERE { ?s ?p ?o }"),
            Err(OracleError::Backend(_))
        ));

        let (_p3, lies) = stub("print('definitely not json')\n");
        assert!(
            matches!(
                lies.eval(GRAPH, "SELECT * WHERE { ?s ?p ?o }"),
                Err(OracleError::Backend(_))
            ),
            "exit 0 with unparseable stdout is a BROKEN oracle, not a decline"
        );

        let (_p4, hangs) =
            stub_with_timeout("import time; time.sleep(30)\n", Duration::from_millis(200));
        let err = hangs
            .eval(GRAPH, "SELECT * WHERE { ?s ?p ?o }")
            .expect_err("a hung oracle must not hang the shard");
        assert!(
            matches!(&err, OracleError::Backend(m) if m.contains("budget")),
            "expected a timeout, got {:?}",
            err
        );
    }

    #[test]
    fn oracle_error_display_names_the_bucket() {
        assert_eq!(
            OracleError::Unsupported("f".into()).to_string(),
            "unsupported: f"
        );
        assert_eq!(OracleError::Data("f".into()).to_string(), "data: f");
        assert_eq!(OracleError::Backend("f".into()).to_string(), "backend: f");
    }
}
