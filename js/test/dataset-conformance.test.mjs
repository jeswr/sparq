// [OPUS-4.8] #1047 — conformance + cross-library differential for the RDF/JS `Dataset` surface.
//
// There is no packaged "official" RDF/JS *Dataset*-interface conformance runner that executes
// against an arbitrary third-party `Dataset` (the `rdf-test-suite` runner targets SPARQL/parser
// specs, and `@rdfjs/dataset` itself ships only the `DatasetCore` subset — not the `union` /
// `intersection` / `difference` algebra this PR adds, which is exactly the gap #1047 fills). So
// the strongest available check is a DIFFERENTIAL one: drive the SAME operations through the
// reference `@rdfjs/dataset` (a second, independent RDF/JS implementation) and assert agreement,
// and prove the interop set ops accept a SECOND foreign library (not just N3.js).
import assert from 'node:assert/strict';
import { test } from 'node:test';
import rdfDataset from '@rdfjs/dataset';
import { DataFactory as N3DF } from 'n3';
import { Dataset, DataFactory as DF } from '../dist/index.js';

const { namedNode, quad } = N3DF;
const T = (s, p, o) => quad(namedNode(`http://ex/${s}`), namedNode(`http://ex/${p}`), namedNode(`http://ex/${o}`));

const CORPUS = [T('a', 'p', 'b'), T('a', 'p', 'c'), T('b', 'p', 'c'), T('b', 'q', 'd')];

/** A reference @rdfjs/dataset DatasetCore over the given quads. */
function ref(quads) {
  return rdfDataset.dataset(quads);
}

test('DatasetCore conformance: match() agrees with @rdfjs/dataset (set of patterns)', async () => {
  const a = await Dataset.fromQuads(CORPUS);
  const r = ref(CORPUS);
  const patterns = [
    [namedNode('http://ex/a'), null, null],
    [null, namedNode('http://ex/p'), null],
    [null, null, namedNode('http://ex/c')],
    [namedNode('http://ex/b'), namedNode('http://ex/q'), null],
    [namedNode('http://ex/z'), null, null], // empty result
  ];
  for (const [s, p, o] of patterns) {
    const got = [...a.match(s, p, o)].map((q) => q.subject.value + ' ' + q.predicate.value + ' ' + q.object.value).sort();
    const exp = [...r.match(s, p, o)].map((q) => q.subject.value + ' ' + q.predicate.value + ' ' + q.object.value).sort();
    assert.deepEqual(got, exp, `match(${s?.value}, ${p?.value}, ${o?.value}) agrees with @rdfjs/dataset`);
  }
  a.free();
});

test('interop: set ops accept a @rdfjs/dataset operand (a 2nd foreign library)', async () => {
  const a = await Dataset.fromQuads([T('a', 'p', 'b'), T('a', 'p', 'c')]);
  const foreign = ref([T('a', 'p', 'b'), T('b', 'p', 'c')]); // @rdfjs/dataset, not sparq, not N3

  const u = a.union(foreign);
  assert.equal(u.size, 3, 'union with @rdfjs/dataset dedupes the shared quad');

  const i = a.intersection(foreign);
  assert.equal(i.size, 1);
  assert.ok(i.has(DF.quad(DF.namedNode('http://ex/a'), DF.namedNode('http://ex/p'), DF.namedNode('http://ex/b'))));

  const d = a.difference(foreign);
  assert.equal(d.size, 1);
  assert.ok(d.has(DF.quad(DF.namedNode('http://ex/a'), DF.namedNode('http://ex/p'), DF.namedNode('http://ex/c'))));

  assert.ok(a.contains(ref([T('a', 'p', 'b')])));
  assert.ok(a.equals(ref([T('a', 'p', 'b'), T('a', 'p', 'c')])));
  a.free(); u.free(); i.free(); d.free();
});

test('union/intersection/difference are symmetric with @rdfjs/dataset via DatasetCore semantics', async () => {
  // Build the expected sets manually with @rdfjs/dataset's DatasetCore (has/add), since the
  // reference lacks the algebra — this checks our algebra against first-principles set math.
  const left = [T('a', 'p', 'b'), T('a', 'p', 'c'), T('b', 'p', 'c')];
  const right = [T('a', 'p', 'c'), T('b', 'p', 'c'), T('b', 'q', 'd')];
  const a = await Dataset.fromQuads(left);
  const rRight = ref(right);

  // expected union = dedup(left ++ right)
  const expUnion = ref(left);
  for (const q of right) expUnion.add(q);
  assert.equal(a.union(rRight).size, expUnion.size);

  // expected intersection = left quads present in right
  const expInter = left.filter((q) => rRight.has(q)).length;
  assert.equal(a.intersection(rRight).size, expInter);

  // expected difference = left quads absent from right
  const expDiff = left.filter((q) => !rRight.has(q)).length;
  assert.equal(a.difference(rRight).size, expDiff);
  a.free();
});
