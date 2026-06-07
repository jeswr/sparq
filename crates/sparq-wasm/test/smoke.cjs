const { Store } = require("../pkg-node/sparq_wasm.js");
const ttl = `@prefix ex: <http://ex/> .
ex:alice ex:name "Alice" ; ex:age 30 ; ex:knows ex:bob .
ex:bob ex:name "Bob"@en ; ex:age 25 ; ex:knows ex:carol .
ex:carol ex:name "Carol" ; ex:age 35 ; ex:knows ex:alice .`;
const store = Store.load(ttl, "turtle");
console.log("triples:", store.size, "heapBytes:", store.heapBytes());
const tri = JSON.parse(store.query(`PREFIX ex: <http://ex/> SELECT * WHERE { ?a ex:knows ?b . ?b ex:knows ?c . ?c ex:knows ?a }`));
console.log("triangle solutions:", tri.results.bindings.length);
const agg = JSON.parse(store.query(`PREFIX ex: <http://ex/> SELECT (AVG(?a) AS ?avg) WHERE { ?s ex:age ?a }`));
console.log("avg age:", agg.results.bindings[0].avg.value);
const names = JSON.parse(store.query(`PREFIX ex: <http://ex/> SELECT ?n WHERE { ?s ex:name ?n } ORDER BY ?n`));
console.log("name[1] binding:", JSON.stringify(names.results.bindings[1].n));
console.log("vars:", JSON.stringify(names.head.vars));
