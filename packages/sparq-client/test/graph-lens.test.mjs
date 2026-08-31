// [OPUS-5] sq-ixc3.22 — unit tests for the programmable graph-viz LENS core + the RDF 1.2
// reifier-annotation folding.
//
// CI coverage: this suite runs in the GATING gui.yml `shared TS client typecheck` job
// (`npm test` in packages/sparq-client), the same lane as test/decompress.test.mjs.
import assert from "node:assert/strict";
import { test } from "node:test";

import {
  DEFAULT_LENS,
  DEFAULT_LENS_ID,
  GRAPH_LENS_SLOTS,
  RDF_REIFIES,
  bindFocusNode,
  edgeKeyOf,
  edgeKeyOfStatement,
  edgeStyleIndex,
  foldReifiedAnnotations,
  graphLensSlotFormError,
  importGraphLens,
  mergeNTriples,
  newGraphLens,
  nodeDetailRows,
  nodeKeyOfRdfTerm,
  nodeKeyOfSparqlTerm,
  nodeStyleIndex,
  parseGraphLens,
  parseGraphLenses,
  parseNTriples,
  rankRadius,
  serializeGraphLens,
  termToNTriples,
  typeColorIndex,
} from "../src/index.ts";

const select = (vars, bindings) => ({ head: { vars }, results: { bindings } });
const uri = (value) => ({ type: "uri", value });
const lit = (value, extra = {}) => ({ type: "literal", value, ...extra });

// ---------------------------------------------------------------------------
// The lens model: parse / serialise / import.
// ---------------------------------------------------------------------------

test("the built-in default lens fills the slots it documents", () => {
  assert.equal(DEFAULT_LENS.id, DEFAULT_LENS_ID);
  assert.equal(DEFAULT_LENS.builtin, true);
  // Every slot key the lens fills must be a declared slot (no typo'd key silently ignored).
  const declared = new Set(GRAPH_LENS_SLOTS.map((s) => s.slot));
  for (const key of Object.keys(DEFAULT_LENS.queries)) assert.ok(declared.has(key), key);
  // The focus-bound slots must actually mention ?node, or clicking a node would do nothing.
  for (const spec of GRAPH_LENS_SLOTS) {
    const q = DEFAULT_LENS.queries[spec.slot];
    if (spec.bindsFocus && q) assert.match(q, /\?node\b/);
  }
});

test("parseGraphLens drops malformed records and unknown slot keys", () => {
  assert.equal(parseGraphLens(null), null);
  assert.equal(parseGraphLens({ name: "no id" }), null);
  assert.equal(parseGraphLens({ id: "x", name: "   " }), null);
  const lens = parseGraphLens({
    id: "lens_1",
    name: "  Trimmed  ",
    queries: { start: "SELECT ?node {}", nope: "SELECT *", expand: "   " },
    updatedAt: 7,
  });
  assert.equal(lens.name, "Trimmed");
  assert.deepEqual(Object.keys(lens.queries), ["start"]);
  assert.equal(lens.updatedAt, 7);
});

test("parseGraphLenses drops duplicates and anything colliding with the built-in id", () => {
  const lenses = parseGraphLenses([
    { id: "a", name: "A", queries: {} },
    { id: "a", name: "A again", queries: {} },
    { id: DEFAULT_LENS_ID, name: "impostor", queries: {} },
    "not a lens",
    { id: "b", name: "B", queries: {} },
  ]);
  assert.deepEqual(
    lenses.map((l) => l.id),
    ["a", "b"],
  );
  assert.deepEqual(parseGraphLenses("nope"), []);
});

test("a shared lens round-trips through JSON but always imports under a fresh id", () => {
  const lens = newGraphLens("Shared");
  const restored = importGraphLens(serializeGraphLens(lens));
  assert.equal(restored.name, "Shared");
  assert.deepEqual(restored.queries, lens.queries);
  assert.notEqual(restored.id, lens.id, "an import must never overwrite a local lens by id");
  assert.equal(importGraphLens("{ not json"), null);
  assert.equal(importGraphLens('{"id":"x"}'), null);
});

// ---------------------------------------------------------------------------
// The slot/form contract — a lens RENDERS, it never MODIFIES.
// ---------------------------------------------------------------------------

test("every slot refuses a SPARQL UPDATE, whichever slot it is smuggled into", () => {
  // The shape a hostile "shared lens" takes: an UPDATE parked in a slot the UI describes as a
  // read-only query. `parseGraphLens` accepts it (it validates shape, not SPARQL), so the form
  // check is what stands between an imported lens and the store.
  for (const { slot } of GRAPH_LENS_SLOTS) {
    const message = graphLensSlotFormError(slot, "update");
    assert.ok(message, `the ${slot} slot must refuse an UPDATE`);
    assert.match(message, /UPDATE/);
    assert.match(message, /not run/);
  }
});

test("each slot accepts exactly its documented form", () => {
  for (const spec of GRAPH_LENS_SLOTS) {
    if (spec.form === "SELECT") {
      assert.equal(graphLensSlotFormError(spec.slot, "select"), null);
      // A graph form in a SELECT slot is a mismatch, not a mutation — refused all the same.
      assert.ok(graphLensSlotFormError(spec.slot, "construct"));
      assert.ok(graphLensSlotFormError(spec.slot, "describe"));
    } else {
      // Both graph forms answer with a graph, so both drive an expansion.
      assert.equal(graphLensSlotFormError(spec.slot, "construct"), null);
      assert.equal(graphLensSlotFormError(spec.slot, "describe"), null);
      assert.ok(graphLensSlotFormError(spec.slot, "select"));
    }
    assert.ok(graphLensSlotFormError(spec.slot, "ask"), "no slot reads an ASK");
  }
});

test("an unknown slot name is refused rather than defaulted", () => {
  assert.ok(graphLensSlotFormError("notASlot", "select"));
});

test("the built-in lens satisfies its own slot/form contract", () => {
  // Non-vacuous the other way: the contract the guard enforces is the one the shipped lens meets,
  // so the guard cannot be passing only because nothing real is ever checked against it.
  const forms = { select: "select", construct: "construct" };
  for (const spec of GRAPH_LENS_SLOTS) {
    const query = DEFAULT_LENS.queries[spec.slot];
    if (!query) continue;
    assert.equal(
      graphLensSlotFormError(spec.slot, forms[spec.form.toLowerCase()]),
      null,
      `the built-in lens's ${spec.slot} slot is ${spec.form}`,
    );
    assert.match(query, new RegExp(spec.form, "i"));
  }
});

// ---------------------------------------------------------------------------
// Binding the focus node.
// ---------------------------------------------------------------------------

test("bindFocusNode substitutes ?node and $node in pattern positions", () => {
  assert.equal(
    bindFocusNode("CONSTRUCT { ?node ?p ?o } WHERE { ?node ?p ?o }", "<http://ex/a>"),
    "CONSTRUCT { <http://ex/a> ?p ?o } WHERE { <http://ex/a> ?p ?o }",
  );
  assert.equal(bindFocusNode("{ $node ?p ?o }", "_:b1"), "{ _:b1 ?p ?o }");
});

test("bindFocusNode leaves longer variable names alone", () => {
  const q = "{ ?nodes ?p ?node_id . ?node ?p ?o }";
  assert.equal(bindFocusNode(q, "<http://ex/a>"), "{ ?nodes ?p ?node_id . <http://ex/a> ?p ?o }");
});

test("bindFocusNode never rewrites inside a literal, an IRI or a comment", () => {
  const q = [
    "# ?node in a comment",
    'FILTER(?x = "?node")',
    "FILTER(?y = <http://ex/?node>)",
    "FILTER(?z = '''multi\n?node line''')",
    "?node ?p ?o",
  ].join("\n");
  const out = bindFocusNode(q, "<http://ex/a>");
  assert.match(out, /# \?node in a comment/);
  assert.match(out, /"\?node"/);
  assert.match(out, /<http:\/\/ex\/\?node>/);
  assert.match(out, /multi\n\?node line/);
  assert.match(out, /<http:\/\/ex\/a> \?p \?o/);
  // Exactly one substitution happened.
  assert.equal(out.split("<http://ex/a>").length - 1, 1);
});

test("bindFocusNode survives a less-than operator (not an IRI) and an unterminated literal", () => {
  assert.equal(
    bindFocusNode("FILTER(?a < 3) ?node ?p ?o", "<http://ex/a>"),
    "FILTER(?a < 3) <http://ex/a> ?p ?o",
  );
  assert.equal(bindFocusNode('"oops\n?node ?p ?o', "<http://ex/a>"), '"oops\n<http://ex/a> ?p ?o');
});

test("bindFocusNode is a no-op when the slot ignores the focus", () => {
  const q = "SELECT ?s WHERE { ?s ?p ?o }";
  assert.equal(bindFocusNode(q, "<http://ex/a>"), q);
});

// ---------------------------------------------------------------------------
// Node identity across the two term representations.
// ---------------------------------------------------------------------------

test("an RDF term and the SPARQL binding of the same term share one key", () => {
  const { statements } = parseNTriples(
    '<http://ex/a> <http://ex/p> "hi"@en .\n' +
      '<http://ex/a> <http://ex/q> "7"^^<http://www.w3.org/2001/XMLSchema#integer> .\n' +
      '_:b0 <http://ex/p> "plain" .\n',
  );
  assert.equal(nodeKeyOfRdfTerm(statements[0].s), nodeKeyOfSparqlTerm(uri("http://ex/a")));
  assert.equal(
    nodeKeyOfRdfTerm(statements[0].o),
    nodeKeyOfSparqlTerm(lit("hi", { "xml:lang": "en" })),
  );
  assert.equal(
    nodeKeyOfRdfTerm(statements[1].o),
    nodeKeyOfSparqlTerm(lit("7", { datatype: "http://www.w3.org/2001/XMLSchema#integer" })),
  );
  assert.equal(nodeKeyOfRdfTerm(statements[2].s), nodeKeyOfSparqlTerm({ type: "bnode", value: "b0" }));
  // An xsd:string literal keys identically to a plain one (N-Triples drops the datatype).
  assert.equal(
    nodeKeyOfRdfTerm(statements[2].o),
    nodeKeyOfSparqlTerm(lit("plain", { datatype: "http://www.w3.org/2001/XMLSchema#string" })),
  );
  assert.equal(nodeKeyOfSparqlTerm(undefined), null);
});

test("termToNTriples round-trips a binding back into the spelling bindFocusNode substitutes", () => {
  assert.equal(termToNTriples(uri("http://ex/a")), "<http://ex/a>");
  assert.equal(termToNTriples({ type: "bnode", value: "b0" }), "_:b0");
  assert.equal(termToNTriples(lit("plain")), '"plain"');
  assert.equal(termToNTriples(lit("hi", { "xml:lang": "en" })), '"hi"@en');
  assert.equal(
    termToNTriples(lit("7", { datatype: "http://www.w3.org/2001/XMLSchema#integer" })),
    '"7"^^<http://www.w3.org/2001/XMLSchema#integer>',
  );
  assert.equal(termToNTriples(lit('say "hi"\n')), '"say \\"hi\\"\\n"');
  assert.equal(termToNTriples(undefined), null);
  // An RDF 1.2 triple term: SPARQL 1.2 nests a term triple in `value` instead of a lexical
  // string, so the literal branch would call `value.replace` on an object and throw.
  // `nodeKeyOfRdfTerm` already has a "triple" case, so this side must not throw on one.
  const tripleTerm = {
    type: "triple",
    value: {
      subject: uri("http://ex/a"),
      predicate: uri("http://ex/p"),
      object: uri("http://ex/b"),
    },
  };
  assert.equal(
    termToNTriples(tripleTerm),
    "<<( <http://ex/a> <http://ex/p> <http://ex/b> )>>",
  );
  assert.equal(nodeKeyOfSparqlTerm(tripleTerm), "t:<<( <http://ex/a> <http://ex/p> <http://ex/b> )>>");
  // The spelling it produces re-parses to the SAME node key — the join the view depends on.
  const nt = termToNTriples(lit("hi", { "xml:lang": "en" }));
  const { statements } = parseNTriples(`<http://ex/a> <http://ex/p> ${nt} .\n`);
  assert.equal(
    nodeKeyOfRdfTerm(statements[0].o),
    nodeKeyOfSparqlTerm(lit("hi", { "xml:lang": "en" })),
  );
});

// ---------------------------------------------------------------------------
// Reading the styling slots.
// ---------------------------------------------------------------------------

test("nodeStyleIndex takes the first type/label and the maximum rank per node", () => {
  const index = nodeStyleIndex(
    select(
      ["node", "type", "label", "rank"],
      [
        { node: uri("http://ex/a"), type: uri("http://ex/Person"), label: lit("Alice"), rank: lit("3") },
        { node: uri("http://ex/a"), type: uri("http://ex/Agent"), rank: lit("9") },
        { node: uri("http://ex/b") },
        { type: uri("http://ex/Person") },
      ],
    ),
  );
  assert.equal(index.size, 2);
  assert.deepEqual(index.get(nodeKeyOfSparqlTerm(uri("http://ex/a"))), {
    type: "http://ex/Person",
    label: "Alice",
    rank: 9,
  });
  assert.deepEqual(index.get(nodeKeyOfSparqlTerm(uri("http://ex/b"))), {});
});

test("a non-numeric rank is ignored rather than sizing a node to NaN", () => {
  const index = nodeStyleIndex(
    select(["node", "rank"], [{ node: uri("http://ex/a"), rank: lit("not a number") }]),
  );
  assert.equal(index.get(nodeKeyOfSparqlTerm(uri("http://ex/a"))).rank, undefined);
});

test("edgeStyleIndex keys on the whole edge and keeps the first row per edge", () => {
  const index = edgeStyleIndex(
    select(
      ["s", "p", "o", "label"],
      [
        { s: uri("http://ex/a"), p: uri("http://ex/p"), o: uri("http://ex/b"), label: lit("first") },
        { s: uri("http://ex/a"), p: uri("http://ex/p"), o: uri("http://ex/b"), label: lit("second") },
        { s: uri("http://ex/a"), p: uri("http://ex/p") },
      ],
    ),
  );
  assert.equal(index.size, 1);
  const key = edgeKeyOf(
    nodeKeyOfSparqlTerm(uri("http://ex/a")),
    nodeKeyOfSparqlTerm(uri("http://ex/p")),
    nodeKeyOfSparqlTerm(uri("http://ex/b")),
  );
  assert.equal(index.get(key).label, "first");
});

test("nodeDetailRows keeps result order and skips half-bound rows", () => {
  assert.deepEqual(
    nodeDetailRows(
      select(
        ["key", "value"],
        [
          { key: uri("http://ex/name"), value: lit("Alice") },
          { key: uri("http://ex/city") },
          { key: uri("http://ex/age"), value: lit("30") },
        ],
      ),
    ),
    [
      { key: "http://ex/name", value: "Alice" },
      { key: "http://ex/age", value: "30" },
    ],
  );
});

test("typeColorIndex is deterministic and in range", () => {
  assert.equal(typeColorIndex("http://ex/Person"), typeColorIndex("http://ex/Person"));
  for (const t of ["", "a", "http://ex/Person", "http://ex/Place", "x".repeat(500)]) {
    const i = typeColorIndex(t);
    assert.ok(Number.isInteger(i) && i >= 0 && i < 8, `${t} → ${i}`);
  }
});

test("rankRadius interpolates and falls back to the midpoint on a degenerate range", () => {
  assert.equal(rankRadius(0, 0, 10, 4, 12), 4);
  assert.equal(rankRadius(10, 0, 10, 4, 12), 12);
  assert.equal(rankRadius(5, 0, 10, 4, 12), 8);
  assert.equal(rankRadius(undefined, 0, 10, 4, 12), 8, "no rank → midpoint");
  assert.equal(rankRadius(3, 5, 5, 4, 12), 8, "every node equal → midpoint");
  assert.equal(rankRadius(99, 0, 10, 4, 12), 12, "clamped, never larger than max");
  assert.equal(rankRadius(Number.NaN, 0, 10, 4, 12), 8);
});

// ---------------------------------------------------------------------------
// RDF 1.2 reifier annotations.
// ---------------------------------------------------------------------------

const REIFIED_DOC =
  `<http://ex/alice> <http://ex/knows> <http://ex/bob> .\n` +
  `<http://ex/claim> <${RDF_REIFIES}> <<( <http://ex/alice> <http://ex/knows> <http://ex/bob> )>> .\n` +
  `<http://ex/claim> <http://ex/since> "2019" .\n` +
  `<http://ex/claim> <http://ex/source> <http://ex/directory> .\n`;

test("the spec reifier form folds onto its edge and leaves the drawn graph clean", () => {
  const { statements } = parseNTriples(REIFIED_DOC);
  const folded = foldReifiedAnnotations(statements);
  // Only the asserted triple is drawn — no rdf:reifies plumbing, no reifier node fan-out.
  assert.equal(folded.base.length, 1);
  assert.equal(folded.base[0].p.value, "http://ex/knows");
  const key = edgeKeyOfStatement(folded.base[0]);
  const anns = folded.annotations.get(key);
  assert.equal(anns.length, 2);
  assert.deepEqual(
    anns.map((a) => a.p.value),
    ["http://ex/since", "http://ex/source"],
  );
  assert.equal(anns[0].o.value, "2019");
  assert.equal(anns[0].reifier.value, "http://ex/claim");
  assert.equal(folded.unattached, 0);
});

test("a reifier declared AFTER its annotations still folds (two-pass)", () => {
  const reordered =
    `<http://ex/alice> <http://ex/knows> <http://ex/bob> .\n` +
    `<http://ex/claim> <http://ex/since> "2019" .\n` +
    `<http://ex/claim> <${RDF_REIFIES}> <<( <http://ex/alice> <http://ex/knows> <http://ex/bob> )>> .\n`;
  const folded = foldReifiedAnnotations(parseNTriples(reordered).statements);
  assert.equal(folded.base.length, 1);
  assert.equal(folded.annotations.get(edgeKeyOfStatement(folded.base[0])).length, 1);
});

test("the direct triple-term-subject form folds the same way", () => {
  const doc =
    `<http://ex/alice> <http://ex/knows> <http://ex/bob> .\n` +
    `<<( <http://ex/alice> <http://ex/knows> <http://ex/bob> )>> <http://ex/since> "2019" .\n`;
  const folded = foldReifiedAnnotations(parseNTriples(doc).statements);
  assert.equal(folded.base.length, 1);
  const anns = folded.annotations.get(edgeKeyOfStatement(folded.base[0]));
  assert.equal(anns.length, 1);
  assert.equal(anns[0].reifier, null);
});

test("an annotation whose triple is NOT asserted is counted, never invented as an edge", () => {
  // RDF 1.2 reification does not assert the reified triple: dropping the asserted line must
  // leave the graph empty and the annotation reported as unattached.
  const doc = REIFIED_DOC.split("\n").slice(1).join("\n");
  const folded = foldReifiedAnnotations(parseNTriples(doc).statements);
  assert.equal(folded.base.length, 0);
  assert.equal(folded.annotations.size, 0);
  assert.equal(folded.unattached, 2);
});

test("a graph with no reification is returned untouched", () => {
  const { statements } = parseNTriples(
    '<http://ex/a> <http://ex/p> <http://ex/b> .\n<http://ex/b> <http://ex/q> "x" .\n',
  );
  const folded = foldReifiedAnnotations(statements);
  assert.equal(folded.base.length, 2);
  assert.equal(folded.annotations.size, 0);
  assert.equal(folded.unattached, 0);
});

test("a triple POINTING AT a reifier stays in the drawn graph", () => {
  // Only statements whose SUBJECT is the reifier are annotations; an incoming reference is data.
  const doc =
    REIFIED_DOC + `<http://ex/report> <http://ex/cites> <http://ex/claim> .\n`;
  const folded = foldReifiedAnnotations(parseNTriples(doc).statements);
  assert.equal(folded.base.length, 2);
  assert.ok(folded.base.some((st) => st.p.value === "http://ex/cites"));
});

// ---------------------------------------------------------------------------
// Merging expansion results.
// ---------------------------------------------------------------------------

test("mergeNTriples unions documents, preserving first-seen order without duplicates", () => {
  const a = "<http://ex/a> <http://ex/p> <http://ex/b> .\n";
  const b = "<http://ex/a> <http://ex/p> <http://ex/b> .\n<http://ex/b> <http://ex/p> <http://ex/c> .\n";
  assert.equal(
    mergeNTriples(a, b),
    "<http://ex/a> <http://ex/p> <http://ex/b> .\n<http://ex/b> <http://ex/p> <http://ex/c> .\n",
  );
  assert.equal(mergeNTriples("", "  \n\n"), "");
  assert.equal(mergeNTriples(), "");
});
