// Spike benchmark: compares query/load time of a sparq-wasm build against itself
// across two builds (baseline vs +simd128). Run as:
//   node test/bench.cjs <pkgDir>
// Prints JSON lines so an outer script can diff two builds.
const path = require("node:path");
const pkgDir = process.argv[2] || "pkg-node";
const { Store } = require(path.join("..", pkgDir, "sparq_wasm.js"));

// Build a deterministic synthetic graph: N people, each with name/age/city and
// a `follows` edge to (i+1)%N and (i+7)%N, giving chains + cycles (WCOJ paths)
// + star attributes + a numeric column for FILTER.
function buildTurtle(n) {
  const cities = ["Paris", "Berlin", "Tokyo", "Lima", "Cairo"];
  const lines = ["@prefix ex: <http://ex/> ."];
  for (let i = 0; i < n; i++) {
    const age = 18 + (i % 60);
    lines.push(
      `ex:p${i} ex:name "Person ${i}" ; ex:age ${age} ; ex:city "${cities[i % cities.length]}" ; ` +
        `ex:follows ex:p${(i + 1) % n} ; ex:follows ex:p${(i + 7) % n} .`
    );
  }
  return lines.join("\n");
}

const N = 20000;
const ttl = buildTurtle(N);

const QUERIES = {
  "scan-type": `PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:name ?n }`,
  "star-3": `PREFIX ex: <http://ex/> SELECT ?s ?n ?a ?c WHERE { ?s ex:name ?n . ?s ex:age ?a . ?s ex:city ?c }`,
  "chain-2": `PREFIX ex: <http://ex/> SELECT ?a ?b ?c WHERE { ?a ex:follows ?b . ?b ex:follows ?c }`,
  "triangle-wcoj": `PREFIX ex: <http://ex/> SELECT ?a ?b ?c WHERE { ?a ex:follows ?b . ?b ex:follows ?c . ?c ex:follows ?a }`,
  "filter-age": `PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:age ?a . FILTER(?a > 50) }`,
  "filter-heavy": `PREFIX ex: <http://ex/> SELECT ?s ?a WHERE { ?s ex:age ?a . FILTER(?a > 30 && ?a < 70) }`,
};

function timeIt(fn, warmup, iters) {
  for (let i = 0; i < warmup; i++) fn();
  const ts = [];
  for (let i = 0; i < iters; i++) {
    const t0 = process.hrtime.bigint();
    fn();
    const t1 = process.hrtime.bigint();
    ts.push(Number(t1 - t0) / 1e6); // ms
  }
  ts.sort((a, b) => a - b);
  return { min: ts[0], median: ts[(ts.length / 2) | 0] };
}

// Load timing (rebuild store from scratch each iter).
const loadT = timeIt(() => Store.load(ttl, "turtle"), 2, 8);

const store = Store.load(ttl, "turtle");
const out = { pkg: pkgDir, triples: store.size, load: loadT, queries: {} };
for (const [name, q] of Object.entries(QUERIES)) {
  // sanity: get row count once
  const rows = JSON.parse(store.query(q)).results.bindings.length;
  out.queries[name] = { rows, ...timeIt(() => store.query(q), 3, 25) };
}
console.log(JSON.stringify(out));
