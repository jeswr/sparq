// Benchmark sketch: sparq (this package) vs oxigraph's npm package on a small
// synthetic workload — load N-Triples, full scan, predicate scan, a two-pattern
// join, and a COUNT aggregate. oxigraph is optional: `npm i --no-save oxigraph`
// to enable the comparison; the script skips it gracefully when absent.
//
// This is a *sketch*, not a rigorous benchmark (single process, no warm-up
// isolation, one dataset shape). Treat the numbers as directional only.

import { SparqStore } from '../dist/index.js';

const N = Number(process.env.BENCH_TRIPLES ?? 100_000);

function makeNTriples(n) {
  const people = Math.floor(n / 4);
  let out = '';
  for (let i = 0; i < people; i++) {
    out += `<http://ex/p${i}> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://ex/Person> .\n`;
    out += `<http://ex/p${i}> <http://ex/name> "Person ${i}" .\n`;
    out += `<http://ex/p${i}> <http://ex/age> "${20 + (i % 60)}"^^<http://www.w3.org/2001/XMLSchema#integer> .\n`;
    out += `<http://ex/p${i}> <http://ex/knows> <http://ex/p${(i + 1) % people}> .\n`;
  }
  return out;
}

const QUERIES = {
  'full scan (count rows)': 'SELECT ?s ?p ?o WHERE { ?s ?p ?o }',
  'predicate scan': 'SELECT ?s ?n WHERE { ?s <http://ex/name> ?n }',
  'join knows→name': 'SELECT ?a ?n WHERE { ?a <http://ex/knows> ?b . ?b <http://ex/name> ?n }',
  'filtered + limit': 'SELECT ?s ?a WHERE { ?s <http://ex/age> ?a FILTER(?a > 70) } LIMIT 100',
};

function time(fn) {
  const t0 = performance.now();
  const out = fn();
  return [performance.now() - t0, out];
}

function bench(label, fn, repeat = 3) {
  fn(); // warm-up
  let best = Infinity;
  let result;
  for (let i = 0; i < repeat; i++) {
    const [ms, out] = time(fn);
    if (ms < best) [best, result] = [ms, out];
  }
  console.log(`  ${label.padEnd(28)} ${best.toFixed(1).padStart(8)} ms  (${result} rows)`);
}

const nt = makeNTriples(N);
console.log(`dataset: ${N.toLocaleString()} triples (${(nt.length / 1e6).toFixed(1)} MB N-Triples)\n`);

// --- sparq ---------------------------------------------------------------------------------------
{
  console.log('sparq (wasm):');
  const t0 = performance.now();
  const store = await SparqStore.fromString(nt, 'ntriples');
  console.log(`  ${'load N-Triples'.padEnd(28)} ${(performance.now() - t0).toFixed(1).padStart(8)} ms  (${store.size} triples)`);
  for (const [label, q] of Object.entries(QUERIES)) {
    bench(label, () => store.query(q).length);
  }
  bench('count (no materialise)', () => store.count(QUERIES['full scan (count rows)']));
  console.log(`  heapBytes ≈ ${(store.heapBytes() / 1e6).toFixed(1)} MB\n`);
  store.free();
}

// --- oxigraph (optional) -------------------------------------------------------------------------
let oxigraph;
try {
  oxigraph = await import('oxigraph');
} catch {
  console.log('oxigraph: not installed — `npm i --no-save oxigraph` to compare. Skipping.');
}
if (oxigraph) {
  console.log('oxigraph (wasm):');
  const t0 = performance.now();
  const store = new oxigraph.Store();
  store.load(nt, { format: 'application/n-triples' });
  console.log(`  ${'load N-Triples'.padEnd(28)} ${(performance.now() - t0).toFixed(1).padStart(8)} ms  (${store.size} triples)`);
  for (const [label, q] of Object.entries(QUERIES)) {
    bench(label, () => store.query(q).length);
  }
}
