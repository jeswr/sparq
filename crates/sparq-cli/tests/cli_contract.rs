//! [OPUS-4.8] Per-subcommand golden-output + exit-code CONTRACT table for sparq-cli.
//!
//! Spawns the *built* `sparq-cli` binary (via `CARGO_BIN_EXE_sparq-cli`, the same
//! mechanism as `hdt_cli.rs`) and drives it through an invocation table —
//! `(args) -> (expected exit code, stdout/stderr shape)`. Modelled on the server's
//! `protocol.rs` per-input negotiation table, adapted to a CLI process table.
//!
//! Why a *subprocess* test and not unit tests: every subcommand's logic lives behind
//! `fn main()`'s positional dispatch and reports via `println!`/`std::process::exit`,
//! so the only way to test the real exit-code + stdout contract is to run the binary.
//! (This is also why the crate's llvm-cov reads ~0% — the covered code runs in a child
//! process the coverage instrumentation of the test binary never sees.)
//!
//! These tests are intentionally datasets-tiny and self-contained: each builds its own
//! fixtures under `CARGO_TARGET_TMPDIR` and cleans them up.
//!
//! NOTE on the `query` output contract (verified empirically, see beads from sq-q50l).
//!
//! `query` routes SELECT **and** ASK **and** CONSTRUCT through `sparq_engine::query`,
//! which only handles SELECT/ASK. So:
//!
//! - SELECT -> prints "<n> solutions in <ms>ms"
//! - ASK -> prints "1 solutions" (true) / "0 solutions" (false)  [NOT a boolean]
//! - CONSTRUCT/DESCRIBE -> "query error: only SELECT and ASK queries are supported",
//!   and exits 1.
//!
//! i.e. the `query` subcommand has **no result-format / row-emitting output** — it is a
//! solution *counter*. The CONSTRUCT-as-error and the missing rows/boolean/format output
//! are tracked as enhancement beads; this table asserts the *actual* contract so a future
//! change to it is a deliberate, test-visible decision.
//!
//! The CONSTRUCT path that *does* work today is `bench` (run_query_suite routes graph
//! forms through construct_or_describe) — exercised below.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

// ---------------------------------------------------------------------------
// Harness
// ---------------------------------------------------------------------------

/// A fresh, unique scratch dir under the cargo per-test-target tmp dir.
fn scratch(tag: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("cli-contract-{tag}"));
    let _ = std::fs::remove_dir_all(&dir); // clean any prior run
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

/// The canonical 3-triple N-Triples fixture used across the happy-path cases.
const NT: &str = "\
<http://ex/alice> <http://ex/knows> <http://ex/bob> .
<http://ex/alice> <http://ex/age> \"30\" .
<http://ex/bob> <http://ex/age> \"25\" .
";

/// The same graph as Turtle, to exercise the buffered (non-NT) load path + the `ttl` alias.
const TTL: &str = "\
@prefix ex: <http://ex/> .
ex:alice ex:knows ex:bob ; ex:age 30 .
ex:bob ex:age 25 .
";

/// Run the built binary with `args`, returning the captured Output.
fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_sparq-cli"))
        .args(args)
        .output()
        .expect("spawning sparq-cli")
}

/// Convenience: (exit code, stdout, stderr). A missing code (signal) is reported as -1.
fn run3(args: &[&str]) -> (i32, String, String) {
    let out = run(args);
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn write(dir: &Path, name: &str, body: &str) -> PathBuf {
    let p = dir.join(name);
    std::fs::write(&p, body).expect("write fixture");
    p
}

fn s(p: &Path) -> &str {
    p.to_str().unwrap()
}

// ---------------------------------------------------------------------------
// 1. Happy paths — `query`
// ---------------------------------------------------------------------------

#[test]
fn query_select_counts_solutions_nt() {
    let dir = scratch("select-nt");
    let data = write(&dir, "data.nt", NT);
    let (code, stdout, stderr) = run3(&["query", s(&data), "ntriples", "SELECT ?s ?p ?o WHERE { ?s ?p ?o }"]);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(stdout.contains("3 solutions"), "stdout: {stdout}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn query_select_turtle_and_ttl_alias_load_identically() {
    let dir = scratch("select-ttl");
    let data = write(&dir, "data.ttl", TTL);
    // canonical name
    let (c1, o1, e1) = run3(&["query", s(&data), "turtle", "SELECT ?s ?p ?o WHERE { ?s ?p ?o }"]);
    assert_eq!(c1, 0, "stderr: {e1}");
    assert!(o1.contains("3 solutions"), "stdout: {o1}");
    // alias
    let (c2, o2, _) = run3(&["query", s(&data), "ttl", "SELECT ?s ?p ?o WHERE { ?s ?p ?o }"]);
    assert_eq!(c2, 0);
    assert!(o2.contains("3 solutions"), "stdout: {o2}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn query_ask_true_and_false_are_distinguishable() {
    let dir = scratch("ask");
    let data = write(&dir, "data.nt", NT);
    // ASK currently surfaces as a solution count: 1 == true, 0 == false (see module note).
    let (ct, ot, et) = run3(&["query", s(&data), "ntriples", "ASK { ?s <http://ex/knows> ?o }"]);
    assert_eq!(ct, 0, "stderr: {et}");
    assert!(ot.contains("1 solutions"), "ASK-true stdout: {ot}");

    let (cf, of, _) = run3(&["query", s(&data), "ntriples", "ASK { <http://ex/nobody> ?p ?o }"]);
    assert_eq!(cf, 0);
    assert!(of.contains("0 solutions"), "ASK-false stdout: {of}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn query_with_reason_flag_loads() {
    // `--reason rdfs` on a graph with no schema is a no-op closure, but it must
    // route through the reasoning load path and still answer the query.
    let dir = scratch("reason-flag");
    let data = write(&dir, "data.nt", NT);
    let (code, stdout, stderr) = run3(&[
        "query",
        s(&data),
        "ntriples",
        "SELECT ?s ?p ?o WHERE { ?s ?p ?o }",
        "--reason",
        "rdfs",
    ]);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(stdout.contains("solutions"), "stdout: {stdout}");
    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// 2. Happy paths — compressed-input auto-detection (gzip / zstd / bzip2)
//    All three codecs of the *same* N-Triples bytes must load identically.
// ---------------------------------------------------------------------------

#[test]
fn compressed_input_autodetect_gzip_zstd_bzip2_load_identically() {
    use std::io::Write;
    let dir = scratch("compressed");
    let raw = write(&dir, "data.nt", NT);

    // gzip
    let gz = dir.join("data.nt.gz");
    {
        let mut enc = flate2::write::GzEncoder::new(
            std::fs::File::create(&gz).unwrap(),
            flate2::Compression::fast(),
        );
        enc.write_all(NT.as_bytes()).unwrap();
        enc.finish().unwrap();
    }
    // bzip2
    let bz = dir.join("data.nt.bz2");
    {
        let mut enc = bzip2::write::BzEncoder::new(
            std::fs::File::create(&bz).unwrap(),
            bzip2::Compression::fast(),
        );
        enc.write_all(NT.as_bytes()).unwrap();
        enc.finish().unwrap();
    }
    // zstd
    let zst = dir.join("data.nt.zst");
    std::fs::write(&zst, zstd::stream::encode_all(NT.as_bytes(), 3).unwrap()).unwrap();

    let q = "SELECT ?s ?p ?o WHERE { ?s ?p ?o }";
    let mut counts = Vec::new();
    for f in [&raw, &gz, &bz, &zst] {
        let (code, stdout, stderr) = run3(&["query", s(f), "ntriples", q]);
        assert_eq!(code, 0, "{}: stderr {stderr}", f.display());
        assert!(stdout.contains("3 solutions"), "{}: stdout {stdout}", f.display());
        // capture the solution count token to assert byte-identical results across codecs
        counts.push(
            stdout
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_string(),
        );
    }
    assert!(
        counts.iter().all(|c| c == "3"),
        "compressed inputs disagreed on solution count: {counts:?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// 3. Happy paths — index round-trip: save -> query-mmap ; build -> query-mmap
// ---------------------------------------------------------------------------

#[test]
fn save_then_query_mmap_round_trip() {
    let dir = scratch("save-roundtrip");
    let data = write(&dir, "data.nt", NT);
    let idx = dir.join("idx");

    let (sc, _so, se) = run3(&["save", s(&data), "ntriples", s(&idx)]);
    assert_eq!(sc, 0, "save stderr: {se}");
    assert!(idx.is_dir(), "save did not create the index dir");

    let (qc, qo, qe) = run3(&["query-mmap", s(&idx), "SELECT ?s ?p ?o WHERE { ?s ?p ?o }"]);
    assert_eq!(qc, 0, "query-mmap stderr: {qe}");
    assert!(qo.contains("3 solutions"), "query-mmap stdout: {qo}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn save_compressed_then_query_mmap_round_trip() {
    let dir = scratch("save-compressed");
    let data = write(&dir, "data.nt", NT);
    let idx = dir.join("cidx");

    let (sc, _so, se) = run3(&["save", s(&data), "ntriples", s(&idx), "compressed"]);
    assert_eq!(sc, 0, "save stderr: {se}");

    let (qc, qo, qe) = run3(&["query-mmap", s(&idx), "SELECT ?s ?p ?o WHERE { ?s ?p ?o }"]);
    assert_eq!(qc, 0, "query-mmap stderr: {qe}");
    assert!(qo.contains("3 solutions"), "query-mmap stdout: {qo}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn build_external_then_query_mmap_round_trip() {
    let dir = scratch("build-roundtrip");
    let data = write(&dir, "data.nt", NT);
    let idx = dir.join("bidx");

    let (bc, _bo, be) = run3(&["build", s(&data), "ntriples", s(&idx)]);
    assert_eq!(bc, 0, "build stderr: {be}");
    assert!(idx.is_dir(), "build did not create the index dir");

    let (qc, qo, qe) = run3(&["query-mmap", s(&idx), "SELECT ?s ?p ?o WHERE { ?s ?p ?o }"]);
    assert_eq!(qc, 0, "query-mmap stderr: {qe}");
    assert!(qo.contains("3 solutions"), "query-mmap stdout: {qo}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn recompress_raw_index_then_query() {
    let dir = scratch("recompress");
    let data = write(&dir, "data.nt", NT);
    let raw = dir.join("raw");
    let cmp = dir.join("cmp");

    assert_eq!(run3(&["save", s(&data), "ntriples", s(&raw)]).0, 0);
    let (rc, _ro, re) = run3(&["recompress", s(&raw), s(&cmp)]);
    assert_eq!(rc, 0, "recompress stderr: {re}");

    let (qc, qo, _) = run3(&["query-mmap", s(&cmp), "SELECT ?s ?p ?o WHERE { ?s ?p ?o }"]);
    assert_eq!(qc, 0);
    assert!(qo.contains("3 solutions"), "stdout: {qo}");
    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// 4. Happy paths — ingest / reason / bench (the TSV + summary shapes)
// ---------------------------------------------------------------------------

#[test]
fn ingest_full_reports_triple_and_term_counts() {
    let dir = scratch("ingest");
    let data = write(&dir, "data.nt", NT);
    let (code, stdout, stderr) = run3(&["ingest", s(&data), "full"]);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(stdout.contains("triples ingested : 3"), "stdout: {stdout}");
    assert!(stdout.contains("distinct terms   : 6"), "stdout: {stdout}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn reason_rdfs_reports_triple_count() {
    let dir = scratch("reason");
    let data = write(&dir, "data.nt", NT);
    let (code, stdout, stderr) = run3(&["reason", s(&data), "ntriples", "rdfs"]);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(stdout.contains("triples after rdfs reasoning"), "stdout: {stdout}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn bench_emits_one_tsv_line_per_query_including_construct() {
    let dir = scratch("bench");
    let data = write(&dir, "data.nt", NT);
    let qd = dir.join("q");
    std::fs::create_dir_all(&qd).unwrap();
    write(&qd, "a_all.rq", "SELECT ?s ?p ?o WHERE { ?s ?p ?o }");
    write(&qd, "b_ask.rq", "ASK { ?s <http://ex/knows> ?o }");
    // CONSTRUCT works via `bench` (graph forms route through construct_or_describe),
    // even though it errors via the `query` subcommand — assert that contract here.
    write(&qd, "c_con.rq", "CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }");

    let (code, stdout, stderr) = run3(&["bench", s(&data), "ntriples", s(&qd), "1"]);
    assert_eq!(code, 0, "stderr: {stderr}");
    // One TSV line per query, sorted by name: name<TAB>rows<TAB>micros
    let lines: Vec<&str> = stdout.lines().filter(|l| l.contains('\t')).collect();
    assert_eq!(lines.len(), 3, "expected 3 TSV lines, got: {stdout}");
    assert!(lines[0].starts_with("a_all\t3\t"), "line0: {}", lines[0]);
    assert!(lines[1].starts_with("b_ask\t1\t"), "line1: {}", lines[1]);
    assert!(lines[2].starts_with("c_con\t3\t"), "line2 (CONSTRUCT): {}", lines[2]);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn compare_compress_reports_footprint() {
    let dir = scratch("compare-compress");
    let data = write(&dir, "data.nt", NT);
    let (code, stdout, stderr) = run3(&["compare-compress", s(&data), "ntriples"]);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(stdout.contains("index footprint"), "stdout: {stdout}");
    assert!(stdout.contains("triples"), "stdout: {stdout}");
    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// 5. Error / exit-code contract — the previously-untested surface.
//    The contract: bad usage -> exit 2 ; runtime failure -> exit 1 ; success -> 0.
// ---------------------------------------------------------------------------

#[test]
fn malformed_query_exits_nonzero_with_useful_stderr() {
    let dir = scratch("err-badquery");
    let data = write(&dir, "data.nt", NT);
    let (code, _stdout, stderr) = run3(&["query", s(&data), "ntriples", "SELECT ?s WHERE { this is not sparql"]);
    assert_ne!(code, 0, "a malformed query must NOT exit 0");
    assert_eq!(code, 1, "parse failure is a runtime error (exit 1); stderr: {stderr}");
    assert!(
        stderr.to_lowercase().contains("error"),
        "stderr should explain the failure: {stderr}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn missing_input_file_exits_nonzero() {
    let dir = scratch("err-missing");
    let missing = dir.join("does-not-exist.nt");
    let (code, _stdout, stderr) = run3(&["query", s(&missing), "ntriples", "SELECT ?s ?p ?o WHERE { ?s ?p ?o }"]);
    assert_ne!(code, 0, "a missing input file must NOT exit 0");
    assert_eq!(code, 1, "missing file is a runtime error (exit 1); stderr: {stderr}");
    assert!(!stderr.is_empty(), "stderr should report the missing file");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn unsupported_format_exits_nonzero() {
    // [OPUS-4.8] Regression guard for the fixed bug: before the load_quiet format
    // guard, an unknown format silently parsed the input as Turtle and exited 0.
    let dir = scratch("err-format");
    let data = write(&dir, "data.nt", NT);
    let (code, _stdout, stderr) = run3(&["query", s(&data), "bogusfmt", "SELECT ?s ?p ?o WHERE { ?s ?p ?o }"]);
    assert_ne!(code, 0, "an unsupported format must NOT exit 0 (it silently parsed as Turtle before the fix)");
    assert_eq!(code, 2, "unsupported format is a usage error (exit 2); stderr: {stderr}");
    assert!(
        stderr.contains("unknown format"),
        "stderr should name the unknown format: {stderr}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn build_unsupported_format_exits_nonzero() {
    // [OPUS-4.8] `build` calls Graph::build_external directly (not load_quiet); guard against
    // the same silent-Turtle-fallback bug there too (sq-q50l).
    let dir = scratch("err-buildformat");
    let data = write(&dir, "data.nt", NT);
    let idx = dir.join("idx");
    let (code, _stdout, stderr) = run3(&["build", s(&data), "bogusfmt", s(&idx)]);
    assert_eq!(code, 2, "build with unsupported format must exit 2; stderr: {stderr}");
    assert!(stderr.contains("unknown format"), "stderr: {stderr}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn construct_via_query_subcommand_errors_today() {
    // Documents the CURRENT contract (tracked for change via a bead): the `query`
    // subcommand only supports SELECT/ASK, so CONSTRUCT is a runtime error, not rows.
    let dir = scratch("err-construct");
    let data = write(&dir, "data.nt", NT);
    let (code, _stdout, stderr) = run3(&["query", s(&data), "ntriples", "CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }"]);
    assert_eq!(code, 1, "CONSTRUCT via `query` is currently a runtime error; stderr: {stderr}");
    assert!(
        stderr.contains("only SELECT and ASK"),
        "stderr should explain the unsupported form: {stderr}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn no_subcommand_prints_usage_and_exits_2() {
    let (code, _stdout, stderr) = run3(&[]);
    assert_eq!(code, 2, "no subcommand must exit 2 (usage error)");
    assert!(stderr.contains("usage:"), "stderr: {stderr}");
}

#[test]
fn unknown_subcommand_prints_usage_and_exits_2() {
    let (code, _stdout, stderr) = run3(&["frobnicate"]);
    assert_eq!(code, 2, "unknown subcommand must exit 2 (usage error)");
    assert!(stderr.contains("usage:"), "stderr: {stderr}");
}

#[test]
fn query_too_few_args_exits_2() {
    let dir = scratch("err-args");
    let data = write(&dir, "data.nt", NT);
    let (code, _stdout, stderr) = run3(&["query", s(&data)]); // missing format + sparql
    assert_eq!(code, 2, "too few args is a usage error (exit 2)");
    assert!(stderr.contains("usage: sparq-cli query"), "stderr: {stderr}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn query_mmap_missing_dir_exits_1() {
    let dir = scratch("err-mmapdir");
    let missing = dir.join("no-such-index");
    let (code, _stdout, stderr) = run3(&["query-mmap", s(&missing), "SELECT ?s ?p ?o WHERE { ?s ?p ?o }"]);
    assert_eq!(code, 1, "opening a non-existent index dir is a runtime error (exit 1)");
    assert!(stderr.contains("open error"), "stderr: {stderr}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn bench_unknown_mode_exits_2() {
    let dir = scratch("err-benchmode");
    let data = write(&dir, "data.nt", NT);
    let qd = dir.join("q");
    std::fs::create_dir_all(&qd).unwrap();
    write(&qd, "a.rq", "ASK { ?s ?p ?o }");
    let (code, _stdout, stderr) = run3(&["bench", s(&data), "ntriples", s(&qd), "1", "badmode"]);
    assert_eq!(code, 2, "an unknown bench mode is a usage error (exit 2); stderr: {stderr}");
    assert!(stderr.contains("unknown mode"), "stderr: {stderr}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn reason_unknown_profile_exits_2() {
    let dir = scratch("err-profile");
    let data = write(&dir, "data.nt", NT);
    let (code, _stdout, stderr) = run3(&["reason", s(&data), "ntriples", "bogusprofile"]);
    assert_eq!(code, 2, "an unknown reasoning profile is a usage error (exit 2); stderr: {stderr}");
    assert!(
        stderr.to_lowercase().contains("unknown reasoning profile"),
        "stderr: {stderr}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn recompress_same_src_dst_exits_2() {
    let dir = scratch("err-recompress");
    let same = dir.join("same");
    let (code, _stdout, stderr) = run3(&["recompress", s(&same), s(&same)]);
    assert_eq!(code, 2, "src == dst is a usage error (exit 2)");
    assert!(stderr.contains("dirs must differ"), "stderr: {stderr}");
    let _ = std::fs::remove_dir_all(&dir);
}
