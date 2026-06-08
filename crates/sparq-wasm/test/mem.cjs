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
console.log(`${"triples".padStart(10)} ${"mode".padStart(12)} ${"load".padStart(8)} ${"throughput".padStart(11)} ${"store".padStart(8)} ${"B/triple".padStart(9)}   browser ceiling`);
console.log("-".repeat(92));

const TYPE_Q = "SELECT ?s WHERE { ?s <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://ex/Person> }";
let rawBpt = 0;
let cmpBpt = 0;
for (const entities of [25_000, 100_000, 300_000]) {
  const nt = genNT(entities);
  // Raw (default) vs block-compressed permutations (`loadCompressed`) — the latter is the
  // right default when the tab's RAM, not its CPU, is the constraint (fits a bigger graph).
  for (const mode of ["load", "loadCompressed"]) {
    const t0 = process.hrtime.bigint();
    const store = Store[mode](nt, "ntriples");
    const ms = Number(process.hrtime.bigint() - t0) / 1e6;
    const triples = store.size;
    const bpt = store.heapBytes() / triples;
    if (entities === 300_000) (mode === "load" ? (rawBpt = bpt) : (cmpBpt = bpt));
    const mps = triples / 1e6 / (ms / 1000);
    console.log(
      `${String(triples).padStart(10)} ${mode.padStart(12)} ${(ms.toFixed(0) + "ms").padStart(8)} ${(mps.toFixed(2) + " M/s").padStart(11)} ` +
      `${((store.heapBytes() / 1e6).toFixed(0) + "MB").padStart(8)} ${bpt.toFixed(0).padStart(9)}   ~${(2e9 / bpt / 1e6).toFixed(0)}M @2GB / ~${(4e9 / bpt / 1e6).toFixed(0)}M @4GB`,
    );
    // Sanity: results are identical regardless of storage mode.
    assert.equal(store.count(TYPE_Q), entities);
  }
}

console.log(
  `\nThe wasm build uses the 3-permutation compact index (~36 B/triple of permutations) +\n` +
  `the prefix-factored dictionary. \`loadCompressed\` block-compresses the permutations,\n` +
  `cutting the total from ~${rawBpt.toFixed(0)} to ~${cmpBpt.toFixed(0)} B/triple (-${(100 * (1 - cmpBpt / rawBpt)).toFixed(0)}%) for a small per-scan decode —\n` +
  `i.e. it fits ~${(rawBpt / cmpBpt).toFixed(1)}x more triples in the same tab. Results are identical. all assertions passed ✓`,
);
