// Demonstrates that the engine's streaming/pushdown optimisations carry over to
// the WASM (browser) runtime, where memory and main-thread time are scarcest.
//   wasm-pack build --target nodejs --release --out-dir pkg-node
//   node test/perf.cjs
//   node test/perf.cjs --json /tmp/wasm-perf.json   # strictly-additive emit
//
// [OPUS-4.8] (sq-k5qq) `--json <path>` writes the SAME per-workload timings STDOUT prints
// as a stable, hand-built JSON document (the JS analogue of the Rust harnesses'
// dependency-free `format!`-JSON; mpc_net_bench::cell_json). STDOUT is byte-for-byte
// unchanged whether or not the flag is present; every `ms` is best-effort, MEASURED on the
// running host — ADVISORY + NON-CANONICAL (stated in the emitted `note`) — nothing committed.
const assert = require("node:assert/strict");
const fs = require("node:fs");
const { Store } = require("../pkg-node/sparq_wasm.js");

// Extracts `--json <path>` from argv (a bare `--json` is a usage error), so the run is
// byte-identical when the flag is absent.
function takeJsonArg(argv) {
  const i = argv.indexOf("--json");
  if (i === -1) return null;
  if (i + 1 >= argv.length) {
    console.error("`--json` requires a path argument: --json <path>");
    process.exit(2);
  }
  return argv[i + 1];
}
// Hand-built results document. `rows` is [{label, ms}]; the assertion COUNTS in the body
// are the deterministic gate, the timings are advisory (the note says so). Serialised with
// JSON.stringify, which escapes the label strings; a self-test JSON.parse runs every time.
function resultsJson(meta, rows) {
  return JSON.stringify(
    {
      harness: "sparq-wasm perf",
      ...meta,
      note:
        "every `ms` is best-effort latency MEASURED on the running host — ADVISORY, " +
        "NON-CANONICAL (this dev box) — do not bake into committed files; the in-harness " +
        "assertion counts are the deterministic gate",
      rows,
    },
    null,
    2,
  );
}
const JSON_PATH = takeJsonArg(process.argv.slice(2));
const ROWS = [];

// Deterministic ~400k-triple graph (50k people, 8 triples each: type/name/age/city
// + 4 follows).
const N = 50_000;
let seed = 0x9e3779b9 >>> 0;
const rng = () => ((seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0) % N);
let ttl = "@prefix ex: <http://ex/> .\n";
const parts = [ttl];
for (let i = 0; i < N; i++) {
  parts.push(`ex:n${i} a ex:Person ; ex:name "name${i}" ; ex:age ${20 + (i % 80)} ; ex:city ex:c${i % 500} .\n`);
  for (let k = 0; k < 4; k++) parts.push(`ex:n${i} ex:follows ex:n${rng()} .\n`);
}
ttl = parts.join("");

const t0 = process.hrtime.bigint();
const store = Store.load(ttl, "turtle");
const loadMs = Number(process.hrtime.bigint() - t0) / 1e6;
console.log(`loaded ${store.size} triples in ${loadMs.toFixed(0)} ms\n`);
ROWS.push({ label: "load", ms: loadMs });

function time(label, fn) {
  let best = Infinity, out;
  for (let r = 0; r < 5; r++) {
    const s = process.hrtime.bigint();
    out = fn();
    best = Math.min(best, Number(process.hrtime.bigint() - s) / 1e6);
  }
  console.log(`  ${label.padEnd(34)} ${best.toFixed(3)} ms`);
  ROWS.push({ label, ms: best }); // [OPUS-4.8] (sq-k5qq) capture for the optional --json emit
  return out;
}

console.log("streaming/pushdown wins in the WASM runtime:");

// LIMIT early-termination: must NOT scan all 250k triples.
const lim = time("SELECT * ... LIMIT 10", () =>
  JSON.parse(store.query("SELECT * WHERE { ?s ?p ?o } LIMIT 10")));
assert.equal(lim.results.bindings.length, 10);

// Lazy count(): single pattern -> index range size, no materialisation.
const cPersons = time("count(?s a Person)", () =>
  store.count("PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s a ex:Person }"));
assert.equal(cPersons, N);

// Count-only two-pattern join.
const cJoin = time("count(follows . name)", () =>
  store.count("PREFIX ex: <http://ex/> SELECT * WHERE { ?s ex:follows ?o . ?o ex:name ?n }"));
assert.equal(cJoin, store.count("PREFIX ex: <http://ex/> SELECT * WHERE { ?s ex:follows ?o }"));

// Numeric FILTER pushdown (sequential scan, inline predicate).
const filt = time("FILTER(?a > 90) materialise", () =>
  JSON.parse(store.query("PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:age ?a . FILTER(?a > 90) }")));
assert.ok(filt.results.bindings.length > 0);

// OPTIONAL via sort-merge left join.
const opt = time("OPTIONAL materialise", () =>
  JSON.parse(store.query("PREFIX ex: <http://ex/> SELECT * WHERE { ?s ex:name ?n OPTIONAL { ?s ex:age ?a } }")));
assert.equal(opt.results.bindings.length, N);

// --- The native lazy-count wins carry over to the browser (shared core/engine). ---
// These count() the SAME queries that materialise above, but without building a row
// per solution — the win is the gap between each pair.

// N-pattern STAR count: Σ_s Π c_i(s), no 50k wide rows materialised.
const cStar = time("count(name·age·city star)", () =>
  store.count("PREFIX ex: <http://ex/> SELECT * WHERE { ?s ex:name ?n . ?s ex:age ?a . ?s ex:city ?c }"));
assert.equal(cStar, N);

// Filtered single-pattern count: binary-searched range size on the inline-integer
// column — compare to the FILTER materialise above (same predicate).
const cFilt = time("count(FILTER(?a > 90))", () =>
  store.count("PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:age ?a . FILTER(?a > 90) }"));
assert.equal(cFilt, filt.results.bindings.length);

// Lazy OPTIONAL (left-join) count: Σ_s c_left·max(1,c_right) — compare to the
// OPTIONAL materialise above.
const cOpt = time("count(name OPTIONAL age)", () =>
  store.count("PREFIX ex: <http://ex/> SELECT * WHERE { ?s ex:name ?n OPTIONAL { ?s ex:age ?a } }"));
assert.equal(cOpt, N);

console.log("\nall WASM streaming assertions passed ✓");

// [OPUS-4.8] (sq-k5qq) Build the results document and SELF-TEST it (round-trips through a
// real JSON.parse on every run — catches a malformed emit). Writing the file is
// strictly-additive: only when `--json <path>` was given; STDOUT above is unchanged.
const DOC = resultsJson({ triples: store.size }, ROWS);
const parsed = JSON.parse(DOC); // would throw on a malformed document
assert.equal(parsed.harness, "sparq-wasm perf");
assert.ok(parsed.note.includes("NON-CANONICAL"));
assert.ok(Array.isArray(parsed.rows) && parsed.rows.length === ROWS.length);
if (JSON_PATH) {
  fs.writeFileSync(JSON_PATH, DOC);
  console.error(`wrote ${ROWS.length} result rows to ${JSON_PATH}`);
}
