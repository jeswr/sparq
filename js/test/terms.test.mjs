import assert from 'node:assert/strict';
import { test } from 'node:test';
import { DataFactory as DF, detectQueryForm, quadsToNQuads, termFromSparqlJson, termToNT } from '../dist/index.js';

test('DataFactory produces spec-compliant terms', () => {
  const n = DF.namedNode('http://ex/a');
  assert.equal(n.termType, 'NamedNode');
  assert.ok(n.equals(DF.namedNode('http://ex/a')));
  assert.ok(!n.equals(DF.namedNode('http://ex/b')));
  assert.ok(!n.equals(null));

  const plain = DF.literal('x');
  assert.equal(plain.language, '');
  assert.equal(plain.datatype.value, 'http://www.w3.org/2001/XMLSchema#string');
  const lang = DF.literal('x', 'EN');
  assert.equal(lang.language, 'en'); // normalised to lowercase
  assert.equal(lang.datatype.value, 'http://www.w3.org/1999/02/22-rdf-syntax-ns#langString');
  assert.ok(!plain.equals(lang));
  const typed = DF.literal('5', DF.namedNode('http://www.w3.org/2001/XMLSchema#integer'));
  assert.ok(typed.equals(DF.literal('5', DF.namedNode('http://www.w3.org/2001/XMLSchema#integer'))));

  assert.notEqual(DF.blankNode().value, DF.blankNode().value); // fresh labels
  assert.equal(DF.defaultGraph().value, '');
  assert.equal(DF.variable('v').termType, 'Variable');

  const q = DF.quad(n, DF.namedNode('http://ex/p'), plain);
  assert.equal(q.graph.termType, 'DefaultGraph');
  assert.ok(q.equals(DF.quad(n, DF.namedNode('http://ex/p'), DF.literal('x'))));
  assert.ok(!q.equals(DF.quad(n, DF.namedNode('http://ex/p'), lang)));

  // fromTerm/fromQuad deep-copy foreign implementations
  const foreign = { termType: 'Literal', value: 'x', language: 'en', datatype: { termType: 'NamedNode', value: 'http://www.w3.org/1999/02/22-rdf-syntax-ns#langString' } };
  assert.ok(DF.fromTerm(foreign).equals(lang));
  assert.ok(DF.fromQuad({ subject: n, predicate: n, object: foreign, graph: DF.defaultGraph(), termType: 'Quad', value: '' }).object.equals(lang));
});

test('termToNT and quadsToNQuads escape correctly', () => {
  assert.equal(termToNT(DF.namedNode('http://ex/a')), '<http://ex/a>');
  assert.equal(termToNT(DF.blankNode('b1')), '_:b1');
  assert.equal(termToNT(DF.literal('a"b\\c\nd')), '"a\\"b\\\\c\\nd"');
  assert.equal(termToNT(DF.literal('x', 'en')), '"x"@en');
  assert.equal(termToNT(DF.literal('5', DF.namedNode('http://www.w3.org/2001/XMLSchema#integer'))), '"5"^^<http://www.w3.org/2001/XMLSchema#integer>');

  const nq = quadsToNQuads([
    DF.quad(DF.namedNode('http://ex/s'), DF.namedNode('http://ex/p'), DF.literal('o')),
    DF.quad(DF.namedNode('http://ex/s'), DF.namedNode('http://ex/p'), DF.literal('o'), DF.namedNode('http://ex/g')),
  ]);
  assert.equal(nq, '<http://ex/s> <http://ex/p> "o" .\n<http://ex/s> <http://ex/p> "o" <http://ex/g> .\n');
});

test('termFromSparqlJson decodes the RDF 1.2 `its:dir` directional-literal channel', () => {
  // The engine's SPARQL JSON serialiser keeps `xml:lang` (bare tag) and
  // `its:dir` (base direction) apart for `"x"@en--ltr`; the parse side must
  // reassemble them into a directional language-tagged literal.
  const ltr = termFromSparqlJson({ type: 'literal', value: 'hello', 'xml:lang': 'en', 'its:dir': 'ltr' });
  assert.ok(ltr.equals(DF.literal('hello', { language: 'en', direction: 'ltr' })));
  assert.equal(ltr.direction, 'ltr');
  assert.equal(ltr.datatype.value, 'http://www.w3.org/1999/02/22-rdf-syntax-ns#dirLangString');

  const rtl = termFromSparqlJson({ type: 'literal', value: 'مرحبا', 'xml:lang': 'ar', 'its:dir': 'rtl' });
  assert.ok(rtl.equals(DF.literal('مرحبا', { language: 'ar', direction: 'rtl' })));

  // No `its:dir` → plain language-tagged literal, unchanged behaviour.
  const lang = termFromSparqlJson({ type: 'literal', value: 'hello', 'xml:lang': 'en' });
  assert.ok(lang.equals(DF.literal('hello', 'en')));
  assert.equal(lang.direction, '');
  assert.equal(lang.datatype.value, 'http://www.w3.org/1999/02/22-rdf-syntax-ns#langString');
});

test('query-form detection skips prologue, comments, IRIs and strings', () => {
  assert.equal(detectQueryForm('SELECT * WHERE { ?s ?p ?o }').form, 'SELECT');
  assert.equal(detectQueryForm('PREFIX ex: <http://ex/> ask { ?s ?p ?o }').form, 'ASK');
  assert.equal(detectQueryForm('BASE <http://ex/SELECT> # SELECT?\nDESCRIBE <http://ex/x>').form, 'DESCRIBE');
  assert.equal(detectQueryForm('PREFIX construct: <http://ex/> CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }').form, 'CONSTRUCT');
  assert.equal(detectQueryForm('PREFIX ex: <http://ex/>'), undefined);
});
