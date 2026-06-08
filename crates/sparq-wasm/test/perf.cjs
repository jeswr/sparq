// Demonstrates that the engine's streaming/pushdown optimisations carry over to
// the WASM (browser) runtime, where memory and main-thread time are scarcest.
//   wasm-pack build --target nodejs --release --out-dir pkg-node
//   node test/perf.cjs
const assert = require("node:assert/strict");
const { Store } = require("../pkg-node/sparq_wasm.js");

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

function time(label, fn) {
  let best = Infinity, out;
  for (let r = 0; r < 5; r++) {
    const s = process.hrtime.bigint();
    out = fn();
    best = Math.min(best, Number(process.hrtime.bigint() - s) / 1e6);
  }
  console.log(`  ${label.padEnd(34)} ${best.toFixed(3)} ms`);
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
