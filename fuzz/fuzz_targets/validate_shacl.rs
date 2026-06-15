// Coverage-guided fuzz target for SHACL validation.
// Bead sq-o4pi (atop the sq-ovnf `fuzz/` harness); threat-model T-PARSE-FUZZ. [OPUS-4.8]
//
// SURFACE: `sparq_shacl::validate(data: &Graph, shapes: &Graph) -> ValidationReport` —
// the SHACL Core + SHACL-SPARQL entry point. It runs `ShapesModel::parse(shapes)` (the
// shapes-graph parser: target discovery, constraint-component extraction, `sh:path`
// parsing, `sh:and`/`sh:or`/`sh:node`/`sh:property` shape graph construction) and then
// `eval::validate_graph` (the focus-node traversal, the (focus, shape) recursion guard
// that keeps cyclic `sh:node`/`sh:property` references from overflowing the stack, the
// constraint evaluators, and the `sh:select`/`sh:ask` routing through sparq-engine).
//
// INPUT SPLIT: the fuzzer must drive BOTH the shape-parser and the validator traversal,
// and `validate` only runs the traversal once a shapes graph parses, so we feed it two
// ways from one corpus (the low bit of the first byte picks which, so libFuzzer explores
// both):
//
//   * SPLIT mode — carve the remaining bytes into a shapes chunk + a data chunk at a
//     fuzz-chosen split point, parse EACH as Turtle, and validate only the pairings where
//     BOTH parse. This exercises arbitrary (shapes, data) combinations the way two
//     separately-supplied files would.
//   * SELF mode — parse the WHOLE remaining buffer as Turtle ONCE and use that graph as
//     BOTH the shapes graph and the data graph. A SHACL shapes document is itself valid
//     RDF, so as the corpus grows toward real SHACL Turtle this reaches the deepest
//     shape-parse + traversal code WITHOUT depending on a lucky split (the shapes parse,
//     and their own triples become focus nodes the traversal walks). `validate` borrows
//     both graphs immutably, so a single parsed graph is shared as both arguments — one
//     parse, two `&` borrows; no clone (`Graph` is not `Clone`) and no redundant re-parse.
//
// We feed bytes as `&str` via `from_utf8_lossy` (the public `Graph::load_str` takes
// `&str`) and Turtle as the format (the SHACL-idiomatic serialisation that exercises the
// shape parser most). No full-size `to_vec()`: `from_utf8_lossy` borrows for valid UTF-8,
// and we slice the BORROWED `data` for the split — a large mutated input never gets a
// harness-side copy before a byte reaches the parser.
//
// INVARIANT: hostile (data, shapes) bytes must produce a clean `Err` from the parser OR a
// BOUNDED `ValidationReport` — NEVER a panic, a recursion/stack-overflow abort, an
// integer-overflow abort (overflow-checks are on in this profile), an allocator OOM-abort,
// or UB. libFuzzer treats any of those as a crash and saves the reproducing input; a
// finding here is a real sparq-shacl robustness bug (the validator must degrade to `Err`
// or a bounded report on any input, never abort).
#![no_main]

use libfuzzer_sys::fuzz_target;
use sparq_core::Graph;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    // First byte is the control byte; the rest is the document payload.
    let ctrl = data[0];
    let body = &data[1..];
    if body.is_empty() {
        return;
    }

    if ctrl & 1 == 0 {
        // SPLIT mode: carve a shapes chunk + a data chunk at a fuzz-chosen split point.
        // The remaining 7 control bits index a position in `0..=body.len()` so the two
        // chunks vary in size across the corpus AND both extremes are reachable: split==0
        // (empty shapes) and split==body.len() (empty data). Modulo by `body.len() + 1`
        // (never zero — body is non-empty here) keeps this overflow-safe under
        // overflow-checks, unlike a `(ctrl >> 1) * body.len()` multiply which could
        // overflow `usize` for fuzz-sized inputs and abort as a false-positive crash. [OPUS-4.8]
        let split = (ctrl as usize >> 1) % (body.len() + 1);
        let (shapes_bytes, data_bytes) = body.split_at(split);

        let shapes = Graph::load_str(&String::from_utf8_lossy(shapes_bytes), "turtle");
        let data = Graph::load_str(&String::from_utf8_lossy(data_bytes), "turtle");
        // validate() runs the traversal only once BOTH graphs parse; an Err on either
        // side is a clean outcome, not a finding.
        if let (Ok(data), Ok(shapes)) = (data, shapes) {
            let _ = sparq_shacl::validate(&data, &shapes);
        }
    } else {
        // SELF mode: parse the whole body once and validate the document against its own
        // shapes — the deepest reach into shape-parse + traversal with no split-luck
        // dependence. `validate(&Graph, &Graph)` borrows both args immutably and does not
        // mutate, so the SAME graph serves as both the data and the shapes graph: one
        // parse, two shared `&` borrows (no clone — Graph is not Clone — and no redundant
        // second parse of identical bytes). [OPUS-4.8]
        let text = String::from_utf8_lossy(body);
        if let Ok(graph) = Graph::load_str(&text, "turtle") {
            let _ = sparq_shacl::validate(&graph, &graph);
        }
    }
});
