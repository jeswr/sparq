// Browser memory-bound + ingestion test for the WASM build.
//   wasm-pack build --target nodejs --release --out-dir pkg-node
//   node test/mem.cjs
//
// The browser is the memory-constrained target (a wasm32 linear memory caps at 4 GB,
// and a real tab is happier under ~2 GB). This measures, with the SAME compact dict +
// byte-level N-Triples parser the native engine uses, the in-browser bytes/triple and
// load throughput, and extrapolates how many triples fit a 2 GB / 4 GB heap — the
// figures that decide what a browser deployment can hold.
const assert = require("node:assert/strict");
const { Store } = require("../pkg-node/sparq_wasm.js");

// N-Triples (so it exercises the custom byte-level parser in wasm), 8 triples/entity:
// type + name(string) + age(inline integer) + 4 follows edges.
function genNT(entities) {
  const T = "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>";
  const XSDINT = "<http://www.w3.org/2001/XMLSchema#integer>";
  const parts = [];
  for (let i = 0; i < entities; i++) {
    const s = `<http://ex/n${i}>`;
    parts.push(`${s} ${T} <http://ex/Person> .\n`);
    parts.push(`${s} <http://ex/name> "name${i}" .\n`);
    parts.push(`${s} <http://ex/age> "${20 + (i % 80)}"^^${XSDINT} .\n`);
    for (let k = 0; k < 4; k++) parts.push(`${s} <http://ex/follows> <http://ex/n${(i * 7 + k) % entities}> .\n`);
  }
  return parts.join("");
}

console.log("WASM memory-bound + ingestion (compact dict + byte-level N-Triples parser):\n");
console.log(`${"triples".padStart(10)} ${"load".padStart(8)} ${"throughput".padStart(11)} ${"store".padStart(8)} ${"B/triple".padStart(9)}   browser ceiling`);
console.log("-".repeat(78));

let bptLast = 0;
for (const entities of [25_000, 100_000, 300_000]) {
  const nt = genNT(entities);
  const t0 = process.hrtime.bigint();
  const store = Store.load(nt, "ntriples"); // <- custom byte parser, no oxrdf Term per term
  const ms = Number(process.hrtime.bigint() - t0) / 1e6;
  const triples = store.size;
  const heap = store.heapBytes();
  const bpt = heap / triples;
  bptLast = bpt;
  const mps = triples / 1e6 / (ms / 1000);
  const fit2 = 2e9 / bpt / 1e6;
  const fit4 = 4e9 / bpt / 1e6;
  console.log(
    `${String(triples).padStart(10)} ${(ms.toFixed(0) + "ms").padStart(8)} ${(mps.toFixed(2) + " M/s").padStart(11)} ` +
    `${((heap / 1e6).toFixed(0) + "MB").padStart(8)} ${bpt.toFixed(0).padStart(9)}   ~${fit2.toFixed(0)}M @2GB / ~${fit4.toFixed(0)}M @4GB`,
  );
  // Sanity: the lazy index queries still answer correctly at this size.
  assert.equal(store.count("SELECT ?s WHERE { ?s <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://ex/Person> }"), entities);
}

console.log(
  `\nThe store self-estimate (~${bptLast.toFixed(0)} B/triple here, dominated by the six u32 permutations\n` +
  `at 72 B/triple + the prefix-factored dictionary) sets the browser scale ceiling; the byte-level\n` +
  `parser keeps load throughput high without an oxrdf Term per term. all assertions passed ✓`,
);
