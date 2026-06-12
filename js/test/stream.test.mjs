import assert from 'node:assert/strict';
import { test } from 'node:test';
import { SparqlJsonRowsParser, SparqStore } from '../dist/index.js';

/** N-Triples with `n` subjects × 3 predicates — big enough for several 64 KiB chunks. */
function bigNT(n) {
  let out = '';
  for (let i = 0; i < n; i++) {
    out += `<http://ex/s${i}> <http://ex/name> "subject number ${i} with some padding text" .\n`;
    out += `<http://ex/s${i}> <http://ex/index> "${i}"^^<http://www.w3.org/2001/XMLSchema#integer> .\n`;
    if (i % 2 === 0) out += `<http://ex/s${i}> <http://ex/even> "true"^^<http://www.w3.org/2001/XMLSchema#boolean> .\n`;
  }
  return out;
}

const N = 4000;
const load = () => SparqStore.fromString(bigNT(N), 'ntriples');

test('queryJsonChunks concatenates byte-identically to queryJson, in >1 chunk', async () => {
  const store = await load();
  const sparql = 'SELECT ?s ?p ?o WHERE { ?s ?p ?o }';
  const chunks = [...store.queryJsonChunks(sparql)];
  assert.ok(chunks.length > 1, `expected several chunks, got ${chunks.length}`);
  assert.ok(chunks.slice(0, -1).every(c => c.length >= 64 * 1024), 'non-final chunks flush at ~64 KiB');
  assert.equal(chunks.join(''), store.queryJson(sparql));
});

test('queryBindingsStream yields the same solutions as queryBindings', async () => {
  const store = await load();
  const sparql = 'SELECT ?s ?name ?i WHERE { ?s <http://ex/name> ?name ; <http://ex/index> ?i } ORDER BY ?s';
  const materialised = store.queryBindings(sparql);
  assert.equal(materialised.length, N);

  let i = 0;
  for (const row of store.queryBindingsStream(sparql)) {
    assert.ok(row.equals(materialised[i]), `row ${i} differs`);
    i++;
  }
  assert.equal(i, N);
});

test('queryBindingsStream works with for await…of and OPTIONAL rows', async () => {
  const store = await load();
  const sparql =
    'SELECT ?s ?even WHERE { ?s <http://ex/name> ?n OPTIONAL { ?s <http://ex/even> ?even } }';
  let total = 0;
  let bound = 0;
  for await (const row of store.queryBindingsStream(sparql)) {
    total++;
    if (row.has('even')) bound++;
  }
  assert.equal(total, N);
  assert.equal(bound, N / 2);
});

test('queryBindingsStream: empty results, early break, ASK rejection', async () => {
  const store = await load();
  assert.deepEqual([...store.queryBindingsStream('SELECT ?s WHERE { ?s <http://ex/nope> ?o }')], []);

  // Early break abandons (and frees) the wasm cursor; the store stays usable.
  let seen = 0;
  for (const _ of store.queryBindingsStream('SELECT ?s ?p ?o WHERE { ?s ?p ?o }')) {
    if (++seen === 3) break;
  }
  assert.equal(seen, 3);
  assert.equal(store.count('SELECT ?s WHERE { ?s <http://ex/index> ?o }'), N);

  assert.throws(() => [...store.queryBindingsStream('ASK { ?s ?p ?o }')], /SELECT/);
});

test('SparqlJsonRowsParser is chunk-boundary-agnostic (split mid-row, mid-head)', () => {
  const doc =
    '{"head":{"vars":["s","o"]},"results":{"bindings":[' +
    '{"s":{"type":"uri","value":"http://ex/a"},"o":{"type":"literal","value":"has \\"quotes\\" and {braces}"}},' +
    '{"s":{"type":"bnode","value":"b0"},"o":{"type":"literal","value":"x","xml:lang":"en"}}]}}';
  for (const splitEvery of [1, 3, 7, 50, doc.length]) {
    const parser = new SparqlJsonRowsParser();
    const rows = [];
    for (let i = 0; i < doc.length; i += splitEvery) rows.push(...parser.push(doc.slice(i, i + splitEvery)));
    assert.equal(rows.length, 2, `split=${splitEvery}`);
    assert.equal(rows[0].o.value, 'has "quotes" and {braces}');
    assert.equal(rows[1].s.type, 'bnode');
    assert.equal(parser.boolean, undefined);
  }
  // ASK boolean form sets .boolean and yields no rows
  const ask = new SparqlJsonRowsParser();
  assert.deepEqual(ask.push('{"head":{},"bool'), []);
  assert.deepEqual(ask.push('ean":true}'), []);
  assert.equal(ask.boolean, true);
});
