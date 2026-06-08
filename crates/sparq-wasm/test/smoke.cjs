// End-to-end smoke test of the wasm build in a real runtime (Node).
//   wasm-pack build --target nodejs --release --out-dir pkg-node
//   node test/smoke.cjs
const assert = require("node:assert/strict");
const { Store } = require("../pkg-node/sparq_wasm.js");

const ttl = `@prefix ex: <http://ex/> .
ex:alice ex:name "Alice" ; ex:age 30 ; ex:knows ex:bob .
ex:bob ex:name "Bob"@en ; ex:age 25 ; ex:knows ex:carol .
ex:carol ex:name "Carol" ; ex:age 35 ; ex:knows ex:alice .`;

const store = Store.load(ttl, "turtle");
assert.equal(store.size, 9, "expected 9 triples");
assert.ok(store.heapBytes() > 0, "heapBytes should be positive");

// Triangle: cyclic BGP -> routes through the WCOJ (Leapfrog Triejoin) in wasm.
// alice->bob->carol->alice gives one triangle with 3 rotations.
const tri = JSON.parse(
  store.query(`PREFIX ex: <http://ex/> SELECT * WHERE { ?a ex:knows ?b . ?b ex:knows ?c . ?c ex:knows ?a }`)
);
assert.equal(tri.results.bindings.length, 3, "expected 3 triangle solutions");

// Lazy count(): single-pattern scan and two-pattern join, counted from the index.
assert.equal(store.count(`PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:age ?a }`), 3);
assert.equal(store.count(`PREFIX ex: <http://ex/> SELECT * WHERE { ?a ex:knows ?b . ?b ex:age ?x }`), 2);

// Aggregation: AVG(30,25,35) = 30.
const agg = JSON.parse(store.query(`PREFIX ex: <http://ex/> SELECT (AVG(?a) AS ?avg) WHERE { ?s ex:age ?a }`));
assert.deepEqual(agg.head.vars, ["avg"]);
assert.equal(agg.results.bindings[0].avg.value, "30", "expected avg age 30");

// Language tag + ordering + plain string (no xsd:string datatype) in SPARQL JSON.
const names = JSON.parse(
  store.query(`PREFIX ex: <http://ex/> SELECT ?n WHERE { ?s ex:name ?n } ORDER BY ?n`)
);
assert.deepEqual(names.head.vars, ["n"]);
assert.equal(names.results.bindings.length, 3);
assert.deepEqual(names.results.bindings[1].n, { type: "literal", value: "Bob", "xml:lang": "en" });
assert.deepEqual(names.results.bindings[0].n, { type: "literal", value: "Alice" });

// URI binding shape.
const knows = JSON.parse(store.query(`PREFIX ex: <http://ex/> SELECT ?o WHERE { ex:alice ex:knows ?o }`));
assert.deepEqual(knows.results.bindings[0].o, { type: "uri", value: "http://ex/bob" });

console.log("sparq-wasm smoke test: all assertions passed ✓");
