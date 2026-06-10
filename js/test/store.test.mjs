import assert from 'node:assert/strict';
import { test } from 'node:test';
import { DataFactory as DF, SparqStore } from '../dist/index.js';

const XSD = 'http://www.w3.org/2001/XMLSchema#';
const DATA = `@prefix ex: <http://ex/> .
ex:alice ex:name "Alice" ; ex:age 30 ; ex:knows ex:bob .
ex:bob ex:name "Bob"@en ; ex:age 25 .
_:org ex:name "ACME" .`;

const load = () => SparqStore.fromString(DATA, 'turtle');

test('fromString parses and deduplicates', async () => {
  const store = await load();
  assert.equal(store.size, 6);
  assert.ok(store.heapBytes() > 0);
});

test('fromString compressed returns identical results', async () => {
  const raw = await load();
  const cmp = await SparqStore.fromString(DATA, 'turtle', { compressed: true });
  assert.equal(cmp.size, raw.size);
  assert.equal(
    cmp.queryJson('SELECT ?s ?p ?o WHERE { ?s ?p ?o }'),
    raw.queryJson('SELECT ?s ?p ?o WHERE { ?s ?p ?o }'),
  );
});

test('SELECT returns RDF/JS bindings with spec-compliant terms', async () => {
  const store = await load();
  const rows = store.queryBindings(
    'PREFIX ex: <http://ex/> SELECT ?s ?n ?a WHERE { ?s ex:name ?n . ?s ex:age ?a } ORDER BY ?a',
  );
  assert.equal(rows.length, 2);

  const [bob, alice] = rows;
  // named node
  const s = alice.get('s');
  assert.equal(s.termType, 'NamedNode');
  assert.equal(s.value, 'http://ex/alice');
  // plain literal: xsd:string datatype, empty language
  const n = alice.get('n');
  assert.equal(n.termType, 'Literal');
  assert.equal(n.value, 'Alice');
  assert.equal(n.language, '');
  assert.equal(n.datatype.value, `${XSD}string`);
  // typed literal
  const a = alice.get('a');
  assert.equal(a.value, '30');
  assert.equal(a.datatype.value, `${XSD}integer`);
  // language-tagged literal
  const bobName = bob.get('n');
  assert.equal(bobName.value, 'Bob');
  assert.equal(bobName.language, 'en');
  assert.equal(bobName.datatype.value, 'http://www.w3.org/1999/02/22-rdf-syntax-ns#langString');
});

test('bindings are Map-like per the RDF/JS query spec', async () => {
  const store = await load();
  const [row] = store.queryBindings(
    'PREFIX ex: <http://ex/> SELECT ?s ?n WHERE { ?s ex:name ?n . ?s ex:knows ?x }',
  );
  assert.equal(row.type, 'bindings');
  assert.equal(row.size, 2);
  // get accepts a Variable term or a bare string
  assert.equal(row.get(DF.variable('n')).value, 'Alice');
  assert.equal(row.get('n').value, 'Alice');
  assert.equal(row.get('missing'), undefined);
  assert.ok(row.has('s') && row.has(DF.variable('s')));
  // iteration yields [variable, term] pairs
  const names = [...row].map(([variable]) => variable.value).sort();
  assert.deepEqual(names, ['n', 's']);
  assert.deepEqual([...row.keys()].map(v => v.termType), ['Variable', 'Variable']);
  assert.equal([...row.values()].length, 2);
  // immutable set/delete/filter/map/merge
  const extended = row.set('extra', DF.literal('x'));
  assert.equal(extended.size, 3);
  assert.equal(row.size, 2);
  assert.equal(row.delete('n').size, 1);
  assert.equal(row.filter(t => t.termType === 'Literal').size, 1);
  assert.ok(row.map(() => DF.literal('y')).get('n').equals(DF.literal('y')));
  assert.ok(row.equals(row.set('n', DF.literal('Alice'))));
  assert.ok(!row.equals(row.delete('n')));
  assert.ok(row.merge(extended).equals(extended));
  assert.equal(row.merge(row.set('n', DF.literal('clash'))), undefined);
});

test('ASK queries (both ASK {…} and ASK WHERE {…})', async () => {
  const store = await load();
  assert.equal(store.queryBoolean('PREFIX ex: <http://ex/> ASK { ex:alice ex:knows ex:bob }'), true);
  assert.equal(store.queryBoolean('PREFIX ex: <http://ex/> ASK WHERE { ex:bob ex:knows ex:alice }'), false);
  // the form detector must not trip on ASK inside an IRI or literal
  assert.equal(
    store.queryBoolean('PREFIX ask: <http://ex/ASK#> # ASK comment\nASK { ?s ?p "ASK" }'),
    false,
  );
});

test('query() dispatches on the query form', async () => {
  const store = await load();
  assert.equal(store.query('ASK { ?s ?p ?o }'), true);
  const rows = store.query('SELECT ?s WHERE { ?s ?p ?o }');
  assert.ok(Array.isArray(rows) && rows.length === 6);
});

test('count() avoids materialisation but agrees with SELECT', async () => {
  const store = await load();
  assert.equal(store.count('SELECT ?s WHERE { ?s ?p ?o }'), 6);
});

test('match() with wildcard, constant and variable positions', async () => {
  const store = await load();
  const name = DF.namedNode('http://ex/name');

  assert.equal(store.match().length, 6);
  assert.equal(store.match(null, name, null).length, 3);
  // Variable positions are wildcards, per RDF/JS
  assert.equal(store.match(DF.variable('x'), name).length, 3);

  const quads = store.match(DF.namedNode('http://ex/alice'), name);
  assert.equal(quads.length, 1);
  assert.equal(quads[0].termType, 'Quad');
  assert.ok(quads[0].subject.equals(DF.namedNode('http://ex/alice')));
  assert.ok(quads[0].predicate.equals(name));
  assert.ok(quads[0].object.equals(DF.literal('Alice')));
  assert.equal(quads[0].graph.termType, 'DefaultGraph');

  // constant literal object
  assert.equal(store.match(null, null, DF.literal('Bob', 'en')).length, 1);
  assert.equal(store.match(null, null, DF.literal('Bob')).length, 0);

  // all-constant probe
  const age = DF.literal('30', DF.namedNode(`${XSD}integer`));
  assert.equal(store.match(DF.namedNode('http://ex/alice'), DF.namedNode('http://ex/age'), age).length, 1);
  assert.equal(store.match(DF.namedNode('http://ex/bob'), DF.namedNode('http://ex/age'), age).length, 0);

  // named graph argument matches nothing (triples live in the default graph)
  assert.equal(store.match(null, null, null, DF.namedNode('http://ex/g')).length, 0);
  assert.equal(store.match(null, null, null, DF.defaultGraph()).length, 6);
});

test('match() on a blank-node subject filters by label', async () => {
  const store = await load();
  const [orgQuad] = store.match(null, null, DF.literal('ACME'));
  assert.equal(orgQuad.subject.termType, 'BlankNode');
  const byBnode = store.match(orgQuad.subject, null, null);
  assert.equal(byBnode.length, 1);
  assert.ok(byBnode[0].object.equals(DF.literal('ACME')));
  assert.equal(store.match(DF.blankNode('no-such-label')).length, 0);
});

test('countQuads agrees with match', async () => {
  const store = await load();
  assert.equal(store.countQuads(), 6);
  assert.equal(store.countQuads(null, DF.namedNode('http://ex/name')), 3);
  assert.equal(store.countQuads(null, null, null, DF.namedNode('http://ex/g')), 0);
});

test('fromQuads round-trips RDF/JS terms', async () => {
  const ex = v => DF.namedNode(`http://ex/${v}`);
  const quads = [
    DF.quad(ex('s'), ex('p'), DF.literal('plain "quoted" \\ value\nline2')),
    DF.quad(ex('s'), ex('p'), DF.literal('hallo', 'de')),
    DF.quad(ex('s'), ex('p'), DF.literal('5', DF.namedNode(`${XSD}integer`))),
    DF.quad(DF.blankNode('b0'), ex('p'), ex('o')),
    DF.quad(ex('g-s'), ex('p'), ex('o'), ex('namedGraph')), // folded into the default graph
  ];
  const store = await SparqStore.fromQuads(quads);
  assert.equal(store.size, 5);
  for (const quad of quads) {
    const found = store.match(quad.subject.termType === 'BlankNode' ? null : quad.subject, quad.predicate, quad.object);
    assert.ok(found.length >= 1, `round-trip lost ${quad.object.value}`);
    const match = found.find(f => f.object.equals(quad.object));
    assert.ok(match, `object term changed for ${quad.object.value}`);
    if (quad.subject.termType !== 'BlankNode') assert.ok(match.subject.equals(quad.subject));
  }
});

test('SPARQL update: INSERT DATA / DELETE DATA / DELETE-INSERT WHERE / CLEAR', async () => {
  const store = await load();
  store.update('PREFIX ex: <http://ex/> INSERT DATA { ex:carol ex:name "Carol" }');
  assert.equal(store.size, 7);
  assert.equal(store.queryBoolean('PREFIX ex: <http://ex/> ASK { ex:carol ex:name "Carol" }'), true);

  store.update('PREFIX ex: <http://ex/> DELETE DATA { ex:carol ex:name "Carol" }');
  assert.equal(store.size, 6);

  store.update('PREFIX ex: <http://ex/> DELETE { ?s ex:age ?a } INSERT { ?s ex:years ?a } WHERE { ?s ex:age ?a }');
  assert.equal(store.match(null, DF.namedNode('http://ex/age')).length, 0);
  assert.equal(store.match(null, DF.namedNode('http://ex/years')).length, 2);

  store.update('CLEAR ALL');
  assert.equal(store.size, 0);
});

test('engine errors surface as JS exceptions', async () => {
  const store = await load();
  assert.throws(() => store.queryBindings('SELECT ?s WHERE { broken'), /error|expected/i);
  assert.throws(() => store.query('CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }'), /SELECT/);
});
