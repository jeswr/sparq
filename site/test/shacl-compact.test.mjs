// [OPUS-4.8] sq-pvg2 (#796) — unit tests for the SHACL Compact Syntax (SCS) serializer
// (src/lib/shacl-compact.ts). These cover the pure, framework-free shapes-graph -> SCS
// rendering against hand-built triples (no React, no wasm): golden output for a node +
// property shape, path expression rendering, the bare-literal numeric/boolean rules,
// prefix emission, and the HONEST "not expressible in compact syntax" reporting for
// logical constraints (sh:or). Run via `npm run test:unit`.
import { test } from "node:test";
import assert from "node:assert/strict";

import { shapesToCompact } from "../src/lib/shacl-compact.ts";

const SH = "http://www.w3.org/ns/shacl#";
const RDF = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
const XSD = "http://www.w3.org/2001/XMLSchema#";
const EX = "http://example.org/";

const iri = (v) => ({ termType: "NamedNode", value: v });
const bnode = (v) => ({ termType: "BlankNode", value: v });
const lit = (value, datatype) => ({ termType: "Literal", value, datatype });
const t = (s, p, o) => ({ subject: s, predicate: p, object: o });

const PREFIXES = [{ prefix: "ex", iri: EX }];

test("renders a node shape with targetClass and a property shape with count + datatype", () => {
  // ex:PersonShape -> ex:Person { ex:name [1..1] xsd:string . }
  const triples = [
    t(iri(`${EX}PersonShape`), iri(`${RDF}type`), iri(`${SH}NodeShape`)),
    t(iri(`${EX}PersonShape`), iri(`${SH}targetClass`), iri(`${EX}Person`)),
    t(iri(`${EX}PersonShape`), iri(`${SH}property`), bnode("p1")),
    t(bnode("p1"), iri(`${SH}path`), iri(`${EX}name`)),
    t(bnode("p1"), iri(`${SH}minCount`), lit("1", `${XSD}integer`)),
    t(bnode("p1"), iri(`${SH}maxCount`), lit("1", `${XSD}integer`)),
    t(bnode("p1"), iri(`${SH}datatype`), iri(`${XSD}string`)),
  ];

  const { text, unsupported } = shapesToCompact(triples, PREFIXES);

  assert.equal(unsupported.length, 0, "fully expressible in SCS");
  // prefix block emitted for the prefixes actually used
  assert.match(text, /^PREFIX ex: <http:\/\/example\.org\/>$/m);
  assert.match(text, /^PREFIX xsd: <http:\/\/www\.w3\.org\/2001\/XMLSchema#>$/m);
  // node-shape header with target arrow
  assert.match(text, /shape ex:PersonShape -> ex:Person \{/);
  // property line: path, count, datatype as inline param, terminated by " ."
  assert.match(text, /ex:name \[1\.\.1\] datatype=xsd:string \./);
});

test("renders inverse / sequence / alternative / star path expressions", () => {
  const seqHead = bnode("seqHead");
  const triples = [
    t(iri(`${EX}S`), iri(`${RDF}type`), iri(`${SH}NodeShape`)),
    // inverse path: ^ex:parent
    t(iri(`${EX}S`), iri(`${SH}property`), bnode("inv")),
    t(bnode("inv"), iri(`${SH}path`), bnode("invNode")),
    t(bnode("invNode"), iri(`${SH}inversePath`), iri(`${EX}parent`)),
    t(bnode("inv"), iri(`${SH}minCount`), lit("0", `${XSD}integer`)),
    // sequence path: ex:a / ex:b
    t(iri(`${EX}S`), iri(`${SH}property`), bnode("seq")),
    t(bnode("seq"), iri(`${SH}path`), seqHead),
    t(seqHead, iri(`${RDF}first`), iri(`${EX}a`)),
    t(seqHead, iri(`${RDF}rest`), bnode("seq2")),
    t(bnode("seq2"), iri(`${RDF}first`), iri(`${EX}b`)),
    t(bnode("seq2"), iri(`${RDF}rest`), iri(`${RDF}nil`)),
  ];

  const { text } = shapesToCompact(triples, PREFIXES);
  assert.match(text, /\^ex:parent \[0\.\.\*\]/);
  assert.match(text, /ex:a \/ ex:b/);
});

test("numeric/boolean literals render bare; closed=true is emitted", () => {
  const triples = [
    t(iri(`${EX}S`), iri(`${RDF}type`), iri(`${SH}NodeShape`)),
    t(iri(`${EX}S`), iri(`${SH}closed`), lit("true", `${XSD}boolean`)),
    t(iri(`${EX}S`), iri(`${SH}property`), bnode("age")),
    t(bnode("age"), iri(`${SH}path`), iri(`${EX}age`)),
    t(bnode("age"), iri(`${SH}minInclusive`), lit("0", `${XSD}integer`)),
  ];
  const { text } = shapesToCompact(triples, PREFIXES);
  assert.match(text, /closed=true \./);
  assert.match(text, /minInclusive=0/);
  // a bare integer, NOT a quoted/typed literal
  assert.doesNotMatch(text, /minInclusive="0"/);
});

test("logical constraints (sh:or) are reported as unsupported, never silently dropped", () => {
  const triples = [
    t(iri(`${EX}S`), iri(`${RDF}type`), iri(`${SH}NodeShape`)),
    t(iri(`${EX}S`), iri(`${SH}targetClass`), iri(`${EX}Thing`)),
    t(iri(`${EX}S`), iri(`${SH}or`), bnode("orlist")),
  ];
  const { unsupported } = shapesToCompact(triples, PREFIXES);
  assert.equal(unsupported.length, 1);
  assert.match(unsupported[0], /sh:or/);
  assert.match(unsupported[0], /compact syntax/i);
});

test("an empty / non-shape graph yields empty SCS, no crash", () => {
  const { text, unsupported } = shapesToCompact([], PREFIXES);
  assert.equal(text, "");
  assert.equal(unsupported.length, 0);
});

test("a full IRI is emitted in <> when no prefix covers it", () => {
  const triples = [
    t(iri("urn:shape:1"), iri(`${RDF}type`), iri(`${SH}NodeShape`)),
    t(iri("urn:shape:1"), iri(`${SH}targetClass`), iri("urn:class:Foo")),
  ];
  const { text } = shapesToCompact(triples, PREFIXES);
  assert.match(text, /shape <urn:shape:1> -> <urn:class:Foo> \{/);
});
