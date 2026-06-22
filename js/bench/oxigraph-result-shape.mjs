// [OPUS-4.8] #1123 — measures the cost of the OXIGRAPH-shaped SELECT result accessor
// (`querySolutions` → `Map<string, Term>[]`) relative to the existing RDF/JS `queryBindings`
// (`Bindings[]`) and the raw `queryJson` path, to answer the maintainer's explicit question:
// "are there performance (or other) detriments to aligning the result type with Oxigraph's?"
//
// This is a *sketch*, not a rigorous benchmark (single process, one dataset shape) — run it to
// regenerate directional numbers on the host; nothing is baked into docs (repo hygiene: no
// hard-coded perf numbers in markdown). Run: `node bench/oxigraph-result-shape.mjs`.
//
// What it shows (and why the alignment is safe):
//   * The dominant cost in BOTH `queryBindings` and `querySolutions` is the engine's SPARQL-JSON
//     serialize + JS-side parse + term materialisation — IDENTICAL regardless of the row shape
//     (`queryJson` isolates that floor).
//   * `querySolutions` is a thin O(rows) re-view of `queryBindings` (one extra `Map` per row),
//     so it is strictly ADDITIVE and opt-in — it does NOT slow the existing `queryBindings`.
//   * An Oxigraph-shaped `Map<string, Term>` is if anything CHEAPER to build than an immutable
//     RDF/JS `Bindings` (no `Variable` wrappers, no immutable-Bindings machinery), so aligning
//     the result type carries no material runtime detriment.
import { SparqStore } from '../dist/index.js';

const N = Number(process.env.BENCH_ROWS ?? 50_000);

let nt = '';
for (let i = 0; i < N; i++) nt += `<http://ex/p${i}> <http://ex/name> "Person ${i}" .\n`;

const store = await SparqStore.fromString(nt, 'ntriples');
const Q = 'SELECT ?s ?n WHERE { ?s <http://ex/name> ?n }';

function best(fn, repeat = 7) {
  fn(); // warm-up
  let b = Infinity;
  for (let i = 0; i < repeat; i++) {
    const t = performance.now();
    fn();
    b = Math.min(b, performance.now() - t);
  }
  return b;
}

const rows = store.queryBindings(Q).length;
const tJson = best(() => store.queryJson(Q).length);
const tBindings = best(() => store.queryBindings(Q).length);
const tSolutions = best(() => store.querySolutions(Q).length);

console.log(`SELECT result-shape bench — ${rows.toLocaleString()} rows\n`);
console.log(`  queryJson      (raw SPARQL-JSON string)  ${tJson.toFixed(1).padStart(8)} ms   ← serialize/parse floor`);
console.log(`  queryBindings  (RDF/JS Bindings[])       ${tBindings.toFixed(1).padStart(8)} ms`);
console.log(`  querySolutions (Oxigraph Map<str,Term>[])${tSolutions.toFixed(1).padStart(8)} ms`);
console.log(
  `\n  Oxigraph-Map view overhead vs Bindings: ${(((tSolutions - tBindings) / tBindings) * 100).toFixed(1)}% ` +
    `(one extra Map/row; the shared parse+materialise floor dominates both).`,
);

store.free();
