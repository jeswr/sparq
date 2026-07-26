//! [FABLE-5] (sq-lsp7k.8, epic sq-lsp7k) The `tabular` subcommand end-to-end: CSV direct
//! mapping + the R2RML materializing subset over CSV logical tables, driven through the
//! *built* binary (`CARGO_BIN_EXE_sparq-cli`, the same mechanism as `cli_contract.rs`).
//!
//! The whole subcommand lives behind the opt-in `tabular` feature, so this file compiles
//! and runs only under it: `cargo test -p sparq-cli --features tabular`.
//!
//! The R2RML fixture suite under `tests/fixtures/r2rml/<case>/` is data-driven: each case
//! holds a `mapping.ttl`, its CSV logical table(s), and the exact expected output — an
//! `expected.nt` triple set, or (for a mapping that uses `rr:graphMap`/`rr:graph`) an
//! `expected.nq` QUAD set. Both are compared sorted + deduplicated, since R2RML output is a
//! graph/dataset, not a sequence. Cases named `tc….` are CSV-adaptations of the corresponding
//! W3C R2RML test cases (SQL logical tables replaced by CSV files, as the mapping headers
//! document); the rest pin the same constructs the W3C suite exercises (NULLs,
//! percent-encoding, constants, blank nodes, language tags, multiple tables, joins, named
//! graphs).

#![cfg(feature = "tabular")]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_sparq-cli")).args(args).output().expect("spawning sparq-cli")
}

/// (exit code, stdout, stderr) from the built binary; a signal death reports as -1.
fn run3(args: &[&str]) -> (i32, String, String) {
    let out = run(args);
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn scratch(tag: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("tabular-cli-{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

fn write(dir: &Path, name: &str, body: &str) -> PathBuf {
    let p = dir.join(name);
    std::fs::write(&p, body).expect("write fixture");
    p
}

fn s(p: &Path) -> &str {
    p.to_str().unwrap()
}

/// Sorted, deduplicated non-empty lines — triple SET comparison.
fn triple_set(nt: &str) -> Vec<String> {
    let mut v: Vec<String> = nt.lines().filter(|l| !l.trim().is_empty()).map(str::to_owned).collect();
    v.sort();
    v.dedup();
    v
}

const PEOPLE_CSV: &str = "name,age\nalice,34\nbob,28\n";

// ---------------------------------------------------------------------------
// Direct mapping
// ---------------------------------------------------------------------------

/// The acceptance one-shot: a plain CSV loads via direct mapping and is queried in the
/// same invocation — datatype inference makes `age` an xsd:integer, so a numeric FILTER
/// works without casts.
#[test]
fn direct_one_shot_query() {
    let dir = scratch("direct-query");
    let csv = write(&dir, "people.csv", PEOPLE_CSV);
    let (code, out, err) = run3(&[
        "tabular",
        s(&csv),
        "--query",
        "SELECT ?n WHERE { ?s <http://example.com/people#age> ?a ; <http://example.com/people#name> ?n . FILTER(?a > 30) }",
        "--format",
        "tsv",
    ]);
    assert_eq!(code, 0, "stderr: {err}");
    assert!(err.contains("loaded 6 triples"), "2 rows x (2 cols + rdf:type): {err}");
    assert!(out.contains("\"alice\""), "{out}");
    assert!(!out.contains("\"bob\""), "{out}");
}

/// `--out` streams the exact direct-mapping N-Triples (no graph build): full contract —
/// row-template subjects, per-column predicates, rdf:type, inference, NULL skip.
#[test]
fn direct_out_exact_triples() {
    let dir = scratch("direct-out");
    let csv = write(&dir, "emp.csv", "id,name,score\n1,Ann,4.5\n2,,true\n");
    let out_nt = dir.join("out.nt");
    let (code, _, err) = run3(&["tabular", s(&csv), "--out", s(&out_nt)]);
    assert_eq!(code, 0, "stderr: {err}");
    assert!(err.contains("wrote 7 triples"), "{err}");
    let got = triple_set(&std::fs::read_to_string(&out_nt).unwrap());
    let expected = triple_set(
        r#"<http://example.com/emp/row/1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://example.com/emp> .
<http://example.com/emp/row/1> <http://example.com/emp#id> "1"^^<http://www.w3.org/2001/XMLSchema#integer> .
<http://example.com/emp/row/1> <http://example.com/emp#name> "Ann" .
<http://example.com/emp/row/1> <http://example.com/emp#score> "4.5"^^<http://www.w3.org/2001/XMLSchema#decimal> .
<http://example.com/emp/row/2> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://example.com/emp> .
<http://example.com/emp/row/2> <http://example.com/emp#id> "2"^^<http://www.w3.org/2001/XMLSchema#integer> .
<http://example.com/emp/row/2> <http://example.com/emp#score> "true"^^<http://www.w3.org/2001/XMLSchema#boolean> ."#,
    );
    assert_eq!(got, expected);
}

/// `--template`/`--class none`/`--no-infer`/`--base` reshape the direct mapping.
#[test]
fn direct_flags_template_class_none_no_infer() {
    let dir = scratch("direct-flags");
    let csv = write(&dir, "emp.csv", "ID,age\n7,41\n");
    let out_nt = dir.join("out.nt");
    let (code, _, err) = run3(&[
        "tabular",
        s(&csv),
        "--base",
        "http://ex.org/",
        "--template",
        "http://ex.org/emp/{ID}",
        "--class",
        "none",
        "--no-infer",
        "--out",
        s(&out_nt),
    ]);
    assert_eq!(code, 0, "stderr: {err}");
    let got = triple_set(&std::fs::read_to_string(&out_nt).unwrap());
    let expected = triple_set(
        r#"<http://ex.org/emp/7> <http://ex.org/emp#ID> "7" .
<http://ex.org/emp/7> <http://ex.org/emp#age> "41" ."#,
    );
    assert_eq!(got, expected);
}

/// Compressed inputs stream through the same transparent decompression as every other
/// ingest path: `.csv.gz` and `.csv.zst` load identically to the plain file.
#[test]
fn direct_compressed_inputs_gz_and_zst() {
    let dir = scratch("direct-compressed");
    let gz = dir.join("people.csv.gz");
    let mut enc = flate2::write::GzEncoder::new(std::fs::File::create(&gz).unwrap(), flate2::Compression::default());
    enc.write_all(PEOPLE_CSV.as_bytes()).unwrap();
    enc.finish().unwrap();
    let zst = dir.join("people.csv.zst");
    let mut enc = zstd::stream::write::Encoder::new(std::fs::File::create(&zst).unwrap(), 0).unwrap();
    enc.write_all(PEOPLE_CSV.as_bytes()).unwrap();
    enc.finish().unwrap();
    for path in [&gz, &zst] {
        let (code, out, err) = run3(&[
            "tabular",
            s(path),
            "--query",
            "SELECT (COUNT(*) AS ?n) WHERE { ?s ?p ?o }",
            "--format",
            "tsv",
        ]);
        assert_eq!(code, 0, "{path:?} stderr: {err}");
        // table name = stem WITHOUT the compression + format extensions
        assert!(err.contains("loaded 6 triples"), "{path:?}: {err}");
        assert!(out.contains("6"), "{out}");
    }
}

/// `--out people.nt.gz` writes a valid gzip N-Triples archive (finished trailer, not a
/// truncated stream).
#[test]
fn direct_out_gz_roundtrip() {
    let dir = scratch("direct-out-gz");
    let csv = write(&dir, "t.csv", "a\n1\n");
    let out_gz = dir.join("t.nt.gz");
    let (code, _, err) = run3(&["tabular", s(&csv), "--class", "none", "--out", s(&out_gz)]);
    assert_eq!(code, 0, "stderr: {err}");
    let mut nt = String::new();
    let mut dec = flate2::read::MultiGzDecoder::new(std::fs::File::open(&out_gz).unwrap());
    std::io::Read::read_to_string(&mut dec, &mut nt).unwrap();
    assert_eq!(
        triple_set(&nt),
        triple_set(r#"<http://example.com/t/row/1> <http://example.com/t#a> "1"^^<http://www.w3.org/2001/XMLSchema#integer> ."#)
    );
}

// ---------------------------------------------------------------------------
// R2RML fixture suite
// ---------------------------------------------------------------------------

/// Data-driven R2RML suite: every `tests/fixtures/r2rml/<case>/` runs through the binary
/// (`--mapping … --out …`) and must produce EXACTLY the expected statement set — `expected.nq`
/// when the case emits named graphs, otherwise `expected.nt`.
#[test]
fn r2rml_fixture_suite() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/r2rml");
    let scratch = scratch("r2rml-suite");
    let mut cases: Vec<PathBuf> = std::fs::read_dir(&root)
        .expect("fixtures dir")
        .map(|e| e.unwrap().path())
        .filter(|p| p.is_dir())
        .collect();
    cases.sort();
    assert!(cases.len() >= 11, "expected the full fixture suite, found {}", cases.len());
    for case in cases {
        let name = case.file_name().unwrap().to_str().unwrap().to_owned();
        let mapping = case.join("mapping.ttl");
        // A case is a QUAD case iff it ships an expected.nq; exactly one of the two must exist.
        let nq = case.join("expected.nq");
        let expected = match std::fs::read_to_string(&nq) {
            Ok(s) => {
                assert!(!case.join("expected.nt").exists(), "case {name} has both expected.nt and expected.nq");
                s
            }
            Err(_) => std::fs::read_to_string(case.join("expected.nt"))
                .unwrap_or_else(|e| panic!("case {name}: no expected.nt / expected.nq ({e})")),
        };
        let mut csvs: Vec<PathBuf> = std::fs::read_dir(&case)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|e| e == "csv"))
            .collect();
        csvs.sort();
        let out_nt = scratch.join(format!("{name}.nt"));
        let mut args: Vec<String> = vec!["tabular".into()];
        args.extend(csvs.iter().map(|p| s(p).to_owned()));
        args.extend(["--mapping".into(), s(&mapping).to_owned(), "--out".into(), s(&out_nt).to_owned()]);
        let argrefs: Vec<&str> = args.iter().map(String::as_str).collect();
        let (code, _, err) = run3(&argrefs);
        assert_eq!(code, 0, "case {name}: stderr: {err}");
        let got = triple_set(&std::fs::read_to_string(&out_nt).unwrap());
        assert_eq!(got, triple_set(&expected), "case {name} diverged");
    }
}

/// The R2RML path also LOADS and queries in one shot (not just `--out`).
#[test]
fn r2rml_one_shot_query() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/r2rml/tc0002a-columns-typed");
    let (code, out, err) = run3(&[
        "tabular",
        s(&root.join("student.csv")),
        "--mapping",
        s(&root.join("mapping.ttl")),
        "--query",
        "SELECT ?n WHERE { ?s <http://xmlns.com/foaf/0.1/name> ?n }",
        "--format",
        "tsv",
    ]);
    assert_eq!(code, 0, "stderr: {err}");
    assert!(out.contains("\"Venus\""), "{out}");
}

/// Explicit `<name>=<path>` positional binding overrides the file-stem table name.
#[test]
fn r2rml_explicit_table_binding() {
    let dir = scratch("r2rml-binding");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/r2rml/tc0001a-template-class");
    // Copy the CSV under a NON-matching name; bind it explicitly to the `student` table.
    let renamed = dir.join("other-name.csv");
    std::fs::copy(root.join("student.csv"), &renamed).unwrap();
    let binding = format!("student={}", s(&renamed));
    let out_nt = dir.join("out.nt");
    let (code, _, err) =
        run3(&["tabular", &binding, "--mapping", s(&root.join("mapping.ttl")), "--out", s(&out_nt)]);
    assert_eq!(code, 0, "stderr: {err}");
    let got = triple_set(&std::fs::read_to_string(&out_nt).unwrap());
    assert_eq!(got.len(), 1, "{got:?}");
}

// ---------------------------------------------------------------------------
// [SONNET-4.6] (sq-u1z86) Joins, named graphs, row provenance
// ---------------------------------------------------------------------------

/// A join is queryable in the same shot: the child's `ex:worksIn` object IS the parent triples
/// map's subject, so a two-hop BGP resolves an employee to their department NAME.
#[test]
fn r2rml_join_one_shot_query() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/r2rml/join-parent-triples-map");
    let (code, out, err) = run3(&[
        "tabular",
        s(&root.join("dept.csv")),
        s(&root.join("emp.csv")),
        "--mapping",
        s(&root.join("mapping.ttl")),
        "--query",
        "SELECT ?d WHERE { <http://example.com/emp/1> <http://example.com/worksIn> ?dept . ?dept <http://example.com/name> ?d }",
        "--format",
        "tsv",
    ]);
    assert_eq!(code, 0, "stderr: {err}");
    assert!(out.contains("\"Engineering\""), "{out}");
}

/// A NULL join key never joins, so employee 3 (empty `dept` cell) gets no `ex:worksIn` triple
/// — the join is a filter, not an outer join that invents a term.
#[test]
fn r2rml_join_null_key_emits_nothing() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/r2rml/join-parent-triples-map");
    let (code, out, err) = run3(&[
        "tabular",
        s(&root.join("dept.csv")),
        s(&root.join("emp.csv")),
        "--mapping",
        s(&root.join("mapping.ttl")),
        "--query",
        "SELECT (COUNT(*) AS ?n) WHERE { <http://example.com/emp/3> <http://example.com/worksIn> ?d }",
        "--format",
        "tsv",
    ]);
    assert_eq!(code, 0, "stderr: {err}");
    assert!(out.contains('0'), "{out}");
}

/// With graph maps the in-process load takes the DATASET path, so `GRAPH ?g { … }` sees the
/// named graphs instead of everything folding into the default graph.
#[test]
fn r2rml_graph_map_is_queryable_as_a_dataset() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/r2rml/graph-map");
    let (code, out, err) = run3(&[
        "tabular",
        s(&root.join("record.csv")),
        "--mapping",
        s(&root.join("mapping.ttl")),
        "--query",
        "SELECT ?g WHERE { GRAPH ?g { <http://example.com/rec/1> <http://example.com/secret> ?v } } ORDER BY ?g",
        "--format",
        "tsv",
    ]);
    assert_eq!(code, 0, "stderr: {err}");
    assert!(err.contains("quads"), "the summary must say quads, not triples: {err}");
    assert!(out.contains("http://example.com/g/acme"), "{out}");
    assert!(out.contains("http://example.com/vault"), "{out}");
    // The other tenant's graph must NOT hold rec/1's secret.
    assert!(!out.contains("globex"), "{out}");
}

/// `--provenance` stamps each generated row subject with its 1-based row number and logical
/// table, under sparq's own vocabulary rather than `--base`.
#[test]
fn provenance_triples_in_both_modes() {
    let dir = scratch("provenance");
    let csv = write(&dir, "people.csv", "name\nalice\nbob\n");
    let out_nt = dir.join("direct.nt");
    let (code, _, err) =
        run3(&["tabular", s(&csv), "--class", "none", "--provenance", "--out", s(&out_nt)]);
    assert_eq!(code, 0, "stderr: {err}");
    assert_eq!(
        triple_set(&std::fs::read_to_string(&out_nt).unwrap()),
        triple_set(
            r#"<http://example.com/people/row/1> <http://example.com/people#name> "alice" .
<http://example.com/people/row/1> <https://sparq.dev/ns/tabular#row> "1"^^<http://www.w3.org/2001/XMLSchema#integer> .
<http://example.com/people/row/1> <https://sparq.dev/ns/tabular#table> "people" .
<http://example.com/people/row/2> <http://example.com/people#name> "bob" .
<http://example.com/people/row/2> <https://sparq.dev/ns/tabular#row> "2"^^<http://www.w3.org/2001/XMLSchema#integer> .
<http://example.com/people/row/2> <https://sparq.dev/ns/tabular#table> "people" ."#
        )
    );

    // R2RML has no row concept of its own, which is exactly where --provenance earns its keep.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/r2rml/tc0001a-template-class");
    let (code, out, err) = run3(&[
        "tabular",
        s(&root.join("student.csv")),
        "--mapping",
        s(&root.join("mapping.ttl")),
        "--provenance",
        "--query",
        "SELECT ?r ?t WHERE { ?s <https://sparq.dev/ns/tabular#row> ?r ; <https://sparq.dev/ns/tabular#table> ?t }",
        "--format",
        "tsv",
    ]);
    assert_eq!(code, 0, "stderr: {err}");
    assert!(out.contains('1') && out.contains("\"student\""), "{out}");
}

// ---------------------------------------------------------------------------
// Loud failures
// ---------------------------------------------------------------------------

#[test]
fn errors_are_loud_and_typed() {
    let dir = scratch("errors");
    let csv = write(&dir, "t.csv", "a,b\n1,2\n");

    // usage errors -> exit 2
    let (code, _, _) = run3(&["tabular"]);
    assert_eq!(code, 2, "no files is a usage error");
    let (code, _, err) = run3(&["tabular", s(&csv), "--bogus"]);
    assert_eq!(code, 2, "unknown flag: {err}");
    let mapping = write(&dir, "m.ttl", "@prefix rr: <http://www.w3.org/ns/r2rml#> .\n");
    let (code, _, err) = run3(&["tabular", s(&csv), "--mapping", s(&mapping), "--no-infer"]);
    assert_eq!(code, 2, "direct-mapping flag with --mapping: {err}");
    assert!(err.contains("direct-mapping"), "{err}");

    // data/mapping errors -> exit 1
    let ragged = write(&dir, "ragged.csv", "a,b\n1\n");
    let (code, _, err) = run3(&["tabular", s(&ragged), "--out", s(&dir.join("x.nt"))]);
    assert_eq!(code, 1, "{err}");
    assert!(err.contains("ragged"), "{err}");

    let bad_map = write(
        &dir,
        "bad.ttl",
        "@prefix rr: <http://www.w3.org/ns/r2rml#> .\n<#T> rr:logicalTable [ rr:tableName \"t\" ] ;\n rr:subjectMap [ rr:template \"http://e/{missing}\" ] .\n",
    );
    let (code, _, err) = run3(&["tabular", s(&csv), "--mapping", s(&bad_map), "--out", s(&dir.join("y.nt"))]);
    assert_eq!(code, 1, "{err}");
    assert!(err.contains("not found"), "{err}");

    let sql_map = write(
        &dir,
        "sql.ttl",
        "@prefix rr: <http://www.w3.org/ns/r2rml#> .\n<#T> rr:logicalTable [ rr:sqlQuery \"SELECT 1\" ] ;\n rr:subjectMap [ rr:template \"http://e/x\" ] .\n",
    );
    let (code, _, err) = run3(&["tabular", s(&csv), "--mapping", s(&sql_map), "--out", s(&dir.join("z.nt"))]);
    assert_eq!(code, 1, "{err}");
    assert!(err.contains("rr:sqlQuery"), "{err}");
}
