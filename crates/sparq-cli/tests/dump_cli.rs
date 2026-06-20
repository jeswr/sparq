//! [OPUS-4.8] (sq-oy1f.5) The `dump <file> <in> <out>` re-serializer end-to-end, with a
//! focus on the new `jsonld-compact` out-format (full W3C JSON-LD 1.1 Compaction against a
//! caller `@context` passed via `--context <file>`).
//!
//! Spawns the *built* `sparq-cli` binary (via `CARGO_BIN_EXE_sparq-cli`, the same mechanism
//! as `hdt_cli.rs` / `cli_contract.rs`). The `dump` subcommand lives behind the opt-in
//! `serialize-rdf` feature, so this whole file compiles and runs only under it:
//! `cargo test -p sparq-cli --features serialize-rdf`.
#![cfg(feature = "serialize-rdf")]

use std::path::{Path, PathBuf};
use std::process::Command;

/// A fresh scratch dir under the cargo per-test-target tmp dir.
fn scratch(tag: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("dump-cli-{tag}"));
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

/// (exit code, stdout, stderr) from the built binary.
fn run3(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_sparq-cli"))
        .args(args)
        .output()
        .expect("spawning sparq-cli");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// A small `schema.org` graph so an `@vocab` context can compact every predicate to a bare
/// term (which the prefix-only `jsonld-compacted` form cannot do).
const TTL: &str = "\
@prefix s: <http://schema.org/> .
s:alice s:name \"Alice\" ; s:knows s:bob .
";

/// Baseline: `dump … jsonld-expanded` emits a JSON-LD array (no `@context`) — the always-on
/// expanded path still works (a regression guard for the new match arm ordering).
#[test]
fn dump_jsonld_expanded_still_works() {
    let dir = scratch("expanded");
    let data = write(&dir, "g.ttl", TTL);
    let (code, out, err) = run3(&["dump", s(&data), "turtle", "jsonld-expanded"]);
    assert_eq!(code, 0, "stderr: {err}");
    // Expanded with no named graphs is a bare node-object array (full IRIs, no @context).
    assert!(out.trim_start().starts_with('['), "expanded doc is a bare array: {out}");
    assert!(out.contains("http://schema.org/name"), "full-IRI predicate present: {out}");
    assert!(!out.contains("\"@context\""), "expanded form has no @context: {out}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// `dump … jsonld-compact --context <ctx>` runs the FULL W3C Compaction Algorithm: an
/// `@vocab` context collapses every predicate to a bare `@vocab`-relative term and echoes the
/// `@context` — the capability the prefix-only `jsonld-compacted` form cannot express.
#[test]
fn dump_jsonld_compact_with_context() {
    let dir = scratch("compact");
    let data = write(&dir, "g.ttl", TTL);
    let ctx = write(&dir, "ctx.jsonld", r#"{"@vocab":"http://schema.org/"}"#);
    let (code, out, err) =
        run3(&["dump", s(&data), "turtle", "jsonld-compact", "--context", s(&ctx)]);
    assert_eq!(code, 0, "compaction must succeed; stderr: {err}");
    assert!(out.contains("\"@vocab\":\"http://schema.org/\""), "context echoed: {out}");
    // Bare `@vocab`-relative predicate keys (NOT `schema:name`) prove the algorithm ran.
    assert!(out.contains("\"name\":"), "predicate is a bare @vocab term: {out}");
    assert!(out.contains("\"knows\":"), "second predicate compacted too: {out}");
    // Minified by default — the pretty pass inserts newlines, so the default output has none.
    assert!(!out.contains('\n'), "minified by default (no pretty newlines): {out}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// The `-pretty` variant emits the same document indented (multi-line).
#[test]
fn dump_jsonld_compact_pretty() {
    let dir = scratch("compact-pretty");
    let data = write(&dir, "g.ttl", TTL);
    let ctx = write(&dir, "ctx.jsonld", r#"{"@vocab":"http://schema.org/"}"#);
    let (code, out, err) = run3(&[
        "dump", s(&data), "turtle", "jsonld-compact-pretty", "--context", s(&ctx),
    ]);
    assert_eq!(code, 0, "stderr: {err}");
    assert!(out.contains('\n'), "pretty compaction is multi-line: {out}");
    assert!(out.contains("\"name\":"), "still compacted: {out}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// `jsonld-compact` WITHOUT `--context` is a usage error (exit 2) with a clear message —
/// the `@context` is mandatory for the full algorithm.
#[test]
fn dump_jsonld_compact_missing_context_errors() {
    let dir = scratch("no-ctx");
    let data = write(&dir, "g.ttl", TTL);
    let (code, _out, err) = run3(&["dump", s(&data), "turtle", "jsonld-compact"]);
    assert_eq!(code, 2, "missing --context must exit 2: {err}");
    assert!(err.contains("--context"), "error mentions the flag: {err}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// A `--context` file that is not a JSON object (a JSON-LD `@context` must be one) is a
/// data error (exit 1), not a silently-wrong document.
#[test]
fn dump_jsonld_compact_non_object_context_errors() {
    let dir = scratch("bad-ctx");
    let data = write(&dir, "g.ttl", TTL);
    let ctx = write(&dir, "ctx.json", r#"["not an object"]"#);
    let (code, _out, err) =
        run3(&["dump", s(&data), "turtle", "jsonld-compact", "--context", s(&ctx)]);
    assert_eq!(code, 1, "a non-object @context must exit 1: {err}");
    assert!(err.contains("@context") || err.contains("JSON object"), "clear error: {err}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// `--context` with no following value is a usage error (exit 2).
#[test]
fn dump_context_flag_missing_value_errors() {
    let dir = scratch("ctx-no-val");
    let data = write(&dir, "g.ttl", TTL);
    let (code, _out, err) =
        run3(&["dump", s(&data), "turtle", "jsonld-compact", "--context"]);
    assert_eq!(code, 2, "--context with no value must exit 2: {err}");
    let _ = std::fs::remove_dir_all(&dir);
}
