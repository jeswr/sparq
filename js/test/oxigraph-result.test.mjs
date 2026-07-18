// [OPUS-4.8] #1123 — the OXIGRAPH-shaped SELECT result accessor. Oxigraph's JS `Store.query`
// returns, for a SELECT, an array of plain `Map<string, Term>` keyed on the variable name (no
// `?`). `SparqStore.querySolutions` / `Bindings.toMap` reproduce that shape so Oxigraph-migration
// code ports unchanged, WITHOUT giving up the richer RDF/JS `Bindings` surface.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { SparqStore } from '../dist/index.js';

const TTL = `@prefix ex: <http://ex/> .
ex:alice ex:name "Alice" ; ex:knows ex:bob .
ex:bob ex:name "Bob" .`;

test('querySolutions returns an array of plain Map<string, Term> (Oxigraph shape)', async () => {
  const store = await SparqStore.fromString(TTL, 'turtle');
  const sols = store.querySolutions('PREFIX ex: <http://ex/> SELECT ?s ?n WHERE { ?s ex:name ?n } ORDER BY ?n');
  assert.equal(sols.length, 2);
  // Oxigraph drop-in: each solution is a real Map; .get("name") by string key; .value on the term.
  assert.ok(sols[0] instanceof Map, 'each solution is a native Map');
  assert.equal(sols[0].get('n').value, 'Alice');
  assert.equal(sols[1].get('n').value, 'Bob');
  // keys are bare variable names (no leading ?), exactly like Oxigraph
  assert.deepEqual([...sols[0].keys()].sort(), ['n', 's']);
  // iterating a solution yields [string, Term] pairs (Map semantics)
  for (const [name, term] of sols[0]) {
    assert.equal(typeof name, 'string');
    assert.ok(term.termType);
  }
  store.free();
});

test('the Oxigraph migration snippet ports verbatim', async () => {
  const store = await SparqStore.fromString(TTL, 'turtle');
  // Oxigraph: for (const binding of store.query("SELECT ...")) binding.get("s").value
  const names = [];
  for (const binding of store.querySolutions('PREFIX ex: <http://ex/> SELECT ?s ?n WHERE { ?s ex:name ?n }')) {
    names.push(binding.get('n').value);
  }
  assert.deepEqual(names.sort(), ['Alice', 'Bob']);
  store.free();
});

test('querySolutionsStream yields Oxigraph-shaped Maps lazily', async () => {
  const store = await SparqStore.fromString(TTL, 'turtle');
  const seen = [];
  for (const sol of store.querySolutionsStream('PREFIX ex: <http://ex/> SELECT ?n WHERE { ?s ex:name ?n }')) {
    assert.ok(sol instanceof Map);
    seen.push(sol.get('n').value);
  }
  assert.deepEqual(seen.sort(), ['Alice', 'Bob']);
  store.free();
});

test('Bindings.toMap bridges RDF/JS Bindings to the Oxigraph Map shape', async () => {
  const store = await SparqStore.fromString(TTL, 'turtle');
  const [b] = store.query('PREFIX ex: <http://ex/> SELECT ?n WHERE { ?s ex:name ?n } LIMIT 1');
  // RDF/JS Bindings: .get accepts a string already; iteration yields [Variable, Term]
  const [[variable]] = [...b];
  assert.equal(variable.termType, 'Variable', 'Bindings iteration keeps RDF/JS Variable keys');
  // toMap() is the Oxigraph bridge: bare-string keys
  const map = b.toMap();
  assert.ok(map instanceof Map);
  assert.equal(typeof [...map.keys()][0], 'string', 'toMap keys are bare strings');
  assert.equal(map.get('n').value, b.get('n').value, 'same term, both shapes');
  store.free();
});

test('Bindings.toMap and querySolutions agree term-for-term with materialised query()', async () => {
  const store = await SparqStore.fromString(TTL, 'turtle');
  const q = 'PREFIX ex: <http://ex/> SELECT ?s ?n WHERE { ?s ex:name ?n } ORDER BY ?n';
  const bindings = store.query(q);
  const sols = store.querySolutions(q);
  assert.equal(bindings.length, sols.length);
  for (let i = 0; i < bindings.length; i++) {
    for (const name of bindings[i].toMap().keys()) {
      assert.ok(bindings[i].get(name).equals(sols[i].get(name)), `row ${i} var ${name} matches`);
    }
  }
  store.free();
});
