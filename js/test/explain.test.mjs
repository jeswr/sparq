// [OPUS-4.8] sq-ncvq.14: query-plan introspection (EXPLAIN / EXPLAIN ANALYZE)
// exposed on the raw wasm `Store`, closing the Rust+HTTP vs WASM/JS asymmetry.
// The raw `WasmStore` is the layer that carries the plan-introspection methods
// (the high-level `SparqStore` wrapper covers SELECT/ASK only, mirroring how
// CONSTRUCT/DESCRIBE live on the raw store too), so this exercises it directly.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { init, WasmStore } from '../dist/wasm.js';

const DATA = `@prefix ex: <http://ex/> .
ex:alice ex:name "Alice" ; ex:age 30 ; ex:knows ex:bob .
ex:bob ex:name "Bob"@en ; ex:age 25 .`;

const load = async () => {
  await init();
  return WasmStore.load(DATA, 'turtle');
};

test('explain() returns the planning-only plan text (no execution)', async () => {
  const store = await load();
  const plan = store.explain(
    'PREFIX ex: <http://ex/> SELECT ?n ?a WHERE { ?s ex:name ?n . ?s ex:age ?a }',
  );
  assert.equal(typeof plan, 'string');
  assert.match(plan, /EXPLAIN \(SELECT\)/);
  assert.match(plan, /Plan:/);
  // EXPLAIN accepts every query form, including the graph-valued ones.
  assert.match(
    store.explain('PREFIX ex: <http://ex/> CONSTRUCT { ?s ex:q ?o } WHERE { ?s ex:knows ?o }'),
    /EXPLAIN \(CONSTRUCT\)/,
  );
  // A malformed query surfaces as a thrown JS error.
  assert.throws(() => store.explain('SELECT WHERE {'));
});

test('explainAnalyze() runs SELECT/ASK and appends the operator trace', async () => {
  const store = await load();
  const trace = store.explainAnalyze('PREFIX ex: <http://ex/> SELECT ?n WHERE { ?s ex:name ?n }');
  assert.match(trace, /EXPLAIN ANALYZE \(SELECT\)/);
  assert.match(trace, /Plan:/);
  // CONSTRUCT/DESCRIBE are explain-only — explainAnalyze rejects them.
  assert.throws(
    () => store.explainAnalyze('CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }'),
    /EXPLAIN ANALYZE/,
  );
});
