// [OPUS-4.8] sq-lii76 (#981) — the `Dataset` RDF/JS `DatasetCore` surface (the named ESM
// entry the `<script type="module">` snippet imports), exercised against the built dist/.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { Dataset, DataFactory as DF } from '../dist/index.js';

const TTL = `@prefix ex: <http://ex/> .
ex:alice ex:name "Alice" ; ex:knows ex:bob .
ex:bob ex:name "Bob" .`;

test('Dataset.fromString loads quads; size spans the graph', async () => {
  const ds = await Dataset.fromString(TTL, 'turtle');
  assert.equal(ds.size, 3);
  ds.free();
});

test('Dataset.create is empty and accepts add/delete (DatasetCore)', async () => {
  const ds = await Dataset.create();
  assert.equal(ds.size, 0);
  const q = DF.quad(DF.namedNode('http://ex/s'), DF.namedNode('http://ex/p'), DF.literal('o'));
  const ret = ds.add(q);
  assert.equal(ret, ds, 'add() returns this');
  assert.equal(ds.size, 1);
  assert.ok(ds.has(q));
  ds.add(q); // idempotent — store is a set
  assert.equal(ds.size, 1);
  ds.delete(q);
  assert.equal(ds.size, 0);
  assert.ok(!ds.has(q));
  ds.free();
});

test('Dataset.match returns a new, independent DatasetCore', async () => {
  const ds = await Dataset.fromString(TTL, 'turtle');
  const names = ds.match(null, DF.namedNode('http://ex/name'), null);
  assert.equal(names.size, 2);
  // The original is untouched; the match result is a distinct dataset.
  assert.equal(ds.size, 3);
  assert.notEqual(names, ds);
  // Mutating the match result does not affect the source.
  names.delete(
    DF.quad(DF.namedNode('http://ex/alice'), DF.namedNode('http://ex/name'), DF.literal('Alice')),
  );
  assert.equal(names.size, 1);
  assert.equal(ds.size, 3);
  ds.free();
  names.free();
});

test('Dataset is iterable (Symbol.iterator yields every quad)', async () => {
  const ds = await Dataset.fromString(TTL, 'turtle');
  const quads = [...ds];
  assert.equal(quads.length, 3);
  for (const q of quads) {
    assert.ok(q.subject && q.predicate && q.object, 'each is a quad');
  }
  ds.free();
});

test('Dataset.fromQuads round-trips RDF/JS quads', async () => {
  const q = DF.quad(DF.namedNode('http://ex/s'), DF.namedNode('http://ex/p'), DF.namedNode('http://ex/o'));
  const ds = await Dataset.fromQuads([q]);
  assert.equal(ds.size, 1);
  assert.ok(ds.has(q));
  ds.free();
});

test('Dataset.store exposes the full SPARQL surface', async () => {
  const ds = await Dataset.fromString(TTL, 'turtle');
  const rows = ds.store.query('PREFIX ex: <http://ex/> SELECT ?n WHERE { ?s ex:name ?n }');
  assert.equal(rows.length, 2);
  ds.free();
});

test('Dataset preserves named graphs with { dataset: true }', async () => {
  const nq = '<http://ex/s> <http://ex/p> <http://ex/o> <http://ex/g> .';
  const ds = await Dataset.fromString(nq, 'nquads', { dataset: true });
  assert.equal(ds.size, 1);
  const inG = ds.match(null, null, null, DF.namedNode('http://ex/g'));
  assert.equal(inG.size, 1);
  ds.free();
  inG.free();
});
