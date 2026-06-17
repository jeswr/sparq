// [OPUS-4.8] sq-nzcb — headless wasm smoke test of the JS-facing sparq-rsp-wasm API.
//
// The crate's `#[cfg(test)] mod tests` in src/ runs on the NATIVE target only, so it
// exercises the sparq-rsp + serialiser functions the wrappers delegate to — NOT the actual
// `#[wasm_bindgen]` exports (`JsError::new` is a wasm-bindgen import that panics off-wasm).
// This module closes that gap: every test drives the REAL exported `Rsp` API compiled to and
// executed in a genuine wasm32 runtime via `wasm-pack test --node`, so the windowed
// continuous query + SPARQL-1.1-JSON serialisation across the JS boundary is asserted end to
// end — exactly the surface a `cargo build --target wasm32` never RUNS.
//
// Tiny, inline pushes. `wasm-pack test --node` runs each `#[wasm_bindgen_test]` in the Node
// executor (no browser, no DOM); Node is the default target, so no
// `wasm_bindgen_test_configure!(run_in_browser)` directive is needed.

#![cfg(target_arch = "wasm32")]

use sparq_rsp_wasm::Rsp;
use wasm_bindgen_test::*;

const READING: &str = "<http://ex/reading>";
const SENSOR: &str = "<http://ex/s1>";

fn avg_query() -> String {
    format!("SELECT (AVG(?v) AS ?avg) WHERE {{ ?s {READING} ?v }}")
}

/// The headline demo: AVG(?v) over a tumbling 60-tick window fires once per window close,
/// returning a `{start,end,results}` envelope carrying the SPARQL-1.1-JSON result table.
#[wasm_bindgen_test]
fn tumbling_avg_window_fires_on_close() {
    let mut q = Rsp::select(&avg_query(), 60, 60, 0, "rstream").expect("register");
    // Two readings in [0,60): AVG = (10 + 20) / 2 = 15.0.
    let none = q.push(SENSOR, READING, "10", 0).expect("push");
    assert_eq!(
        none, "[]",
        "no window closes before the window end is reached"
    );
    let none = q.push(SENSOR, READING, "20", 30).expect("push");
    assert_eq!(none, "[]");
    // A push at ts=65 advances the watermark to 65 ≥ 60, closing [0,60).
    let closed = q.push(SENSOR, READING, "5", 65).expect("push");
    assert!(closed.contains("\"start\":0"), "closed [0,60): {closed}");
    assert!(closed.contains("\"end\":60"), "{closed}");
    assert!(closed.contains("\"avg\""), "projects ?avg: {closed}");
    assert!(closed.contains("15.0"), "AVG(10,20) = 15.0: {closed}");
}

/// `flush` closes the final, still-open window at end-of-stream (ignoring max_delay).
#[wasm_bindgen_test]
fn flush_closes_the_tail_window() {
    let mut q = Rsp::select(&avg_query(), 10, 10, 0, "rstream").expect("register");
    q.push(SENSOR, READING, "4", 2).expect("push");
    // Nothing has advanced the watermark to 10, so [0,10) is still open.
    let tail = q.flush().expect("flush");
    assert!(tail.contains("\"start\":0"), "flush closes [0,10): {tail}");
    assert!(tail.contains("4.0"), "AVG(4) = 4.0: {tail}");
}

/// ISTREAM emits only rows ADDED relative to the previous window; a sensor present in both
/// of two consecutive identical windows is NOT re-emitted the second time. Uses a plain
/// projection (not an aggregate) so the diff is over stable rows.
#[wasm_bindgen_test]
fn istream_emits_only_added_rows() {
    let select = format!("SELECT ?s WHERE {{ ?s {READING} ?v }}");
    let mut q = Rsp::select(&select, 10, 10, 0, "istream").expect("register");
    // Window [0,10): s1 present.
    q.push(SENSOR, READING, "1", 1).expect("push");
    // Window [10,20): s1 present again. ts=22 closes both [0,10) and [10,20).
    q.push(SENSOR, READING, "2", 11).expect("push");
    let closed = q.push(SENSOR, READING, "3", 22).expect("push");
    // [0,10) ISTREAMs s1 (added vs. empty); [10,20) ISTREAMs nothing (s1 already present).
    // The second window's bindings array must be empty.
    let last_window = closed.rsplit("\"start\":10").next().unwrap_or("");
    assert!(
        last_window.contains("\"bindings\":[]"),
        "the second identical window ISTREAMs no new rows: {closed}"
    );
}

/// A non-SELECT query is rejected at registration, not at the first push.
#[wasm_bindgen_test]
fn non_select_rejected_at_register() {
    let ask = format!("ASK {{ ?s {READING} ?v }}");
    assert!(Rsp::select(&ask, 10, 10, 0, "rstream").is_err());
}

/// An unknown R2S operator name is a clean error.
#[wasm_bindgen_test]
fn unknown_r2s_errors() {
    assert!(Rsp::select(&avg_query(), 10, 10, 0, "nope").is_err());
}

/// A zero-width / non-advancing window is rejected (range/step must be > 0) — a clean error,
/// NOT the panic the underlying `WindowSpec::time` would raise on a zero argument.
#[wasm_bindgen_test]
fn zero_window_rejected() {
    assert!(Rsp::select(&avg_query(), 0, 10, 0, "rstream").is_err());
    assert!(Rsp::select(&avg_query(), 10, 0, 0, "rstream").is_err());
}

/// A malformed triple term is a clean error on push, not a panic.
#[wasm_bindgen_test]
fn malformed_triple_errors_on_push() {
    let mut q = Rsp::select(&avg_query(), 10, 10, 0, "rstream").expect("register");
    assert!(q.push("not a term", READING, "1", 0).is_err());
}

/// A late push whose every covering window already closed is dropped and COUNTED, not
/// silently swallowed — `lateDropped` reflects it. max_delay = 0, so a push behind a closed
/// window is too late.
#[wasm_bindgen_test]
fn late_push_is_counted() {
    let mut q = Rsp::select(&avg_query(), 10, 10, 0, "rstream").expect("register");
    q.push(SENSOR, READING, "1", 5).expect("push"); // [0,10)
    q.push(SENSOR, READING, "2", 25).expect("push"); // watermark 25 closes [0,10) and [10,20)
    assert_eq!(q.late_dropped(), 0, "no late drops yet");
    // ts=3 now lands behind the closed [0,10): too late.
    q.push(SENSOR, READING, "9", 3).expect("push");
    assert_eq!(q.late_dropped(), 1, "the ts=3 arrival is dropped as late");
}
