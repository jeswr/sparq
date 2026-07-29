// [OPUS-5] sq-ixc3.24 — unit tests for the visual query builder's pure core.
//
// Covers the two things that MUST hold for the tool to be honest:
//   1. the serializer emits standard SPARQL 1.1 for what the canvas shows — nothing hidden,
//      nothing invented (prefix header derived from actual use; OPTIONAL filters scoped
//      inside the OPTIONAL; NOT-EXISTS carrying its leaf node's patterns);
//   2. validation catches the models whose SPARQL would be invalid or would mean something
//      other than the diagram (duplicate variables, projecting a negated variable, projecting
//      a non-grouped variable alongside an aggregate).
//
// Plus the schema-introspection parsers, whose job is to report ONLY what the store returned
// (shape-declared vs data-observed kept distinct, unbound/odd rows dropped rather than guessed).
//
// The rendered canvas is covered by the Playwright spec
// (gui/e2e-playwright/specs/query-builder.web.spec.ts).
//
// Run via:   npm run test:unit   (gui/app)
import { test } from "node:test";
import assert from "node:assert/strict";
import type { SparqlResults, SparqlTerm } from "@sparq/client";

import {
  abbreviateIri,
  buildQuery,
  buildSchemaIndex,
  emptyModel,
  escapeLiteral,
  iriLabel,
  mergeSuggestions,
  parseAnyPredicateRows,
  parseCharacteristicRows,
  parseClassRows,
  parseShapeRows,
  projectedVariables,
  renderAggregate,
  renderFilterExpr,
  renderIri,
  renderPredicate,
  renderValue,
  requiredPrefixLines,
  serializeModel,
  suggestionsFor,
  uniqueVariable,
  validateModel,
  variableStem,
  type BuilderAttribute,
  type BuilderModel,
  type BuilderNode,
} from "./query-builder.js";

const FOAF = "http://xmlns.com/foaf/0.1/";

// ---------------------------------------------------------------------------
// Builders for terse fixtures.
// ---------------------------------------------------------------------------

function attr(partial: Partial<BuilderAttribute> & { variable: string }): BuilderAttribute {
  return {
    id: `a-${partial.variable}`,
    predicate: `${FOAF}name`,
    projected: false,
    optional: false,
    filter: null,
    ...partial,
  };
}

function node(partial: Partial<BuilderNode> & { variable: string }): BuilderNode {
  return {
    id: `n-${partial.variable}`,
    classIri: null,
    attributes: [],
    projected: false,
    x: 0,
    y: 0,
    ...partial,
  };
}

function model(partial: Partial<BuilderModel>): BuilderModel {
  return { ...emptyModel(), limit: null, ...partial };
}

/** A person node with a projected name attribute — the canonical starting diagram. */
function personModel(): BuilderModel {
  return model({
    nodes: [
      node({
        variable: "person",
        classIri: `${FOAF}Person`,
        projected: true,
        attributes: [attr({ variable: "personName", projected: true })],
      }),
    ],
  });
}

// ---------------------------------------------------------------------------
// Term rendering.
// ---------------------------------------------------------------------------

test("renderIri – abbreviates against a common prefix when the local part is safe", () => {
  assert.equal(renderIri(`${FOAF}name`), "foaf:name");
  assert.equal(renderIri("http://www.w3.org/2000/01/rdf-schema#label"), "rdfs:label");
});

test("renderIri – angle-brackets an IRI no common prefix covers", () => {
  assert.equal(renderIri("http://data.example.com/vocab/weight"), "<http://data.example.com/vocab/weight>");
});

test("renderIri – an unsafe local part stays a full IRI rather than an invalid prefixed name", () => {
  // A slash in the local part cannot appear in a PN_LOCAL — abbreviating would be a syntax error.
  assert.equal(renderIri(`${FOAF}deep/path`), `<${FOAF}deep/path>`);
});

test("renderIri – passes a prefixed name and an explicit <…> through untouched", () => {
  assert.equal(renderIri("foaf:name"), "foaf:name");
  assert.equal(renderIri("<http://example.org/p>"), "<http://example.org/p>");
});

test("renderPredicate – keeps the `a` keyword, never rewriting it to an IRI", () => {
  assert.equal(renderPredicate("a"), "a");
  assert.equal(renderPredicate(`${FOAF}knows`), "foaf:knows");
});

test("escapeLiteral – escapes quotes, backslashes and the control whitespace", () => {
  assert.equal(escapeLiteral('say "hi"'), 'say \\"hi\\"');
  assert.equal(escapeLiteral("a\\b"), "a\\\\b");
  assert.equal(escapeLiteral("one\ntwo\ttab"), "one\\ntwo\\ttab");
});

test("renderValue – each value kind renders as its SPARQL term form", () => {
  assert.equal(renderValue("Alice", "string"), '"Alice"');
  assert.equal(renderValue("42", "number"), "42");
  assert.equal(renderValue("TRUE", "boolean"), "true");
  assert.equal(renderValue("no", "boolean"), "false");
  assert.equal(renderValue("2020-01-01", "date"), '"2020-01-01"^^xsd:date');
  assert.equal(renderValue(`${FOAF}Person`, "iri"), "foaf:Person");
});

test("renderFilterExpr – comparison ops compare the variable directly", () => {
  assert.equal(
    renderFilterExpr("age", { op: ">=", value: "18", kind: "number" }),
    "?age >= 18",
  );
});

test("renderFilterExpr – string ops go through STR() and quote the pattern", () => {
  assert.equal(
    renderFilterExpr("name", { op: "contains", value: "Ali", kind: "string" }),
    'CONTAINS(STR(?name), "Ali")',
  );
  assert.equal(
    renderFilterExpr("name", { op: "starts", value: "A", kind: "string" }),
    'STRSTARTS(STR(?name), "A")',
  );
  assert.equal(
    renderFilterExpr("name", { op: "ends", value: "z", kind: "string" }),
    'STRENDS(STR(?name), "z")',
  );
});

test("renderFilterExpr – the regex `i` flag is emitted only when asked for", () => {
  assert.equal(
    renderFilterExpr("n", { op: "regex", value: "^a", kind: "string" }),
    'REGEX(STR(?n), "^a")',
  );
  assert.equal(
    renderFilterExpr("n", { op: "regex", value: "^a", kind: "string", caseInsensitive: true }),
    'REGEX(STR(?n), "^a", "i")',
  );
});

// ---------------------------------------------------------------------------
// Prefix header — declared iff used.
// ---------------------------------------------------------------------------

test("requiredPrefixLines – declares exactly the prefixes the body uses", () => {
  const header = requiredPrefixLines("SELECT * WHERE { ?s a foaf:Person ; rdfs:label ?l }");
  assert.equal(
    header,
    "PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>\nPREFIX foaf: <http://xmlns.com/foaf/0.1/>",
  );
});

test("requiredPrefixLines – a colon inside a <…> IRI or a string literal is not a prefix", () => {
  // `<http://…>` is already absolute, and "a:b" is a string value — declaring either would be
  // an invented dependency on a prefix the query does not use.
  assert.equal(requiredPrefixLines('SELECT * WHERE { ?s <http://xmlns.com/foaf/0.1/x> "foaf:y" }'), "");
});

test("requiredPrefixLines – picks up a datatype prefix after ^^", () => {
  const header = requiredPrefixLines('FILTER(?d > "2020-01-01"^^xsd:date)');
  assert.equal(header, "PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>");
});

// ---------------------------------------------------------------------------
// Serialization.
// ---------------------------------------------------------------------------

test("buildQuery – the canonical class + attribute diagram", () => {
  const built = buildQuery(personModel());
  assert.equal(
    built.sparql,
    [
      "PREFIX foaf: <http://xmlns.com/foaf/0.1/>",
      "",
      "SELECT ?person ?personName",
      "WHERE {",
      "  ?person a foaf:Person .",
      "  ?person foaf:name ?personName .",
      "}",
    ].join("\n"),
  );
  assert.equal(built.runnable, true);
});

test("serializeModel – an empty projection is SELECT *, not a guessed variable list", () => {
  const m = model({ nodes: [node({ variable: "s", classIri: `${FOAF}Person` })] });
  assert.match(serializeModel(m), /^SELECT \*\n/);
});

test("serializeModel – DISTINCT and LIMIT are emitted only when set", () => {
  const plain = serializeModel(personModel());
  assert.doesNotMatch(plain, /DISTINCT/);
  assert.doesNotMatch(plain, /LIMIT/);
  const m = { ...personModel(), distinct: true, limit: 25 };
  const out = serializeModel(m);
  assert.match(out, /^SELECT DISTINCT \?person \?personName$/m);
  assert.match(out, /^LIMIT 25$/m);
});

test("serializeModel – an OPTIONAL attribute's filter stays INSIDE the OPTIONAL", () => {
  // Outside, the filter would drop rows where the attribute is simply absent — which is the
  // opposite of what "optional attribute, filtered" shows on the canvas.
  const m = model({
    nodes: [
      node({
        variable: "person",
        projected: true,
        attributes: [
          attr({
            variable: "mbox",
            predicate: `${FOAF}mbox`,
            optional: true,
            filter: { op: "contains", value: "@example.org", kind: "string" },
          }),
        ],
      }),
    ],
  });
  assert.equal(
    serializeModel(m),
    [
      "SELECT ?person",
      "WHERE {",
      "  OPTIONAL {",
      "    ?person foaf:mbox ?mbox .",
      '    FILTER(CONTAINS(STR(?mbox), "@example.org"))',
      "  }",
      "}",
    ].join("\n"),
  );
});

test("serializeModel – a required attribute's filter follows its triple at group level", () => {
  const m = model({
    nodes: [
      node({
        variable: "person",
        projected: true,
        attributes: [
          attr({ variable: "age", predicate: `${FOAF}age`, filter: { op: ">=", value: "18", kind: "number" } }),
        ],
      }),
    ],
  });
  assert.equal(
    serializeModel(m),
    [
      "SELECT ?person",
      "WHERE {",
      "  ?person foaf:age ?age .",
      "  FILTER(?age >= 18)",
      "}",
    ].join("\n"),
  );
});

test("serializeModel – an edge joins two nodes; OPTIONAL wraps the edge", () => {
  const m = model({
    nodes: [
      node({ variable: "person", classIri: `${FOAF}Person`, projected: true }),
      node({ variable: "friend", classIri: `${FOAF}Person`, projected: true }),
    ],
    edges: [
      {
        id: "e1",
        from: "n-person",
        to: "n-friend",
        predicate: `${FOAF}knows`,
        alternatives: [],
        mode: "optional",
      },
    ],
  });
  assert.equal(
    serializeModel(m),
    [
      "SELECT ?person ?friend",
      "WHERE {",
      "  ?person a foaf:Person .",
      "  ?friend a foaf:Person .",
      "  OPTIONAL { ?person foaf:knows ?friend . }",
      "}",
    ].join("\n"),
  );
});

test("serializeModel – alternative predicates become a UNION (the v1 UNION affordance)", () => {
  const m = model({
    nodes: [
      node({ variable: "person", projected: true }),
      node({ variable: "work", projected: true }),
    ],
    edges: [
      {
        id: "e1",
        from: "n-person",
        to: "n-work",
        predicate: `${FOAF}made`,
        alternatives: ["http://purl.org/dc/terms/creator"],
        mode: "required",
      },
    ],
  });
  assert.match(
    serializeModel(m),
    /\{ \?person foaf:made \?work \} UNION \{ \?person dc:creator \?work \}/,
  );
});

test("serializeModel – a NOT-EXISTS edge carries its leaf node's own patterns inside", () => {
  const m = model({
    nodes: [
      node({ variable: "person", classIri: `${FOAF}Person`, projected: true }),
      node({
        variable: "blocked",
        attributes: [
          attr({
            variable: "blockedName",
            filter: { op: "=", value: "Bob", kind: "string" },
          }),
        ],
      }),
    ],
    edges: [
      {
        id: "e1",
        from: "n-person",
        to: "n-blocked",
        predicate: `${FOAF}knows`,
        alternatives: [],
        mode: "not-exists",
      },
    ],
  });
  assert.equal(
    serializeModel(m),
    [
      "SELECT ?person",
      "WHERE {",
      "  ?person a foaf:Person .",
      "  FILTER NOT EXISTS {",
      "    ?person foaf:knows ?blocked .",
      "    ?blocked foaf:name ?blockedName .",
      '    FILTER(?blockedName = "Bob")',
      "  }",
      "}",
    ].join("\n"),
  );
});

test("serializeModel – GROUP BY + aggregates + ORDER BY", () => {
  const m = model({
    nodes: [
      node({ variable: "person", classIri: `${FOAF}Person`, projected: true }),
      node({ variable: "friend" }),
    ],
    edges: [
      { id: "e1", from: "n-person", to: "n-friend", predicate: `${FOAF}knows`, alternatives: [], mode: "required" },
    ],
    aggregates: [
      { id: "g1", fn: "COUNT", target: "friend", distinct: true, alias: "friends" },
    ],
    groupBy: ["person"],
    orderBy: [{ variable: "friends", desc: true }],
    limit: 10,
  });
  assert.equal(
    serializeModel(m),
    [
      "SELECT ?person (COUNT(DISTINCT ?friend) AS ?friends)",
      "WHERE {",
      "  ?person a foaf:Person .",
      "  ?person foaf:knows ?friend .",
      "}",
      "GROUP BY ?person",
      "ORDER BY DESC(?friends)",
      "LIMIT 10",
    ].join("\n"),
  );
});

test("renderAggregate – COUNT(*) never takes DISTINCT; GROUP_CONCAT carries its separator", () => {
  assert.equal(
    renderAggregate({ id: "g", fn: "COUNT", target: "*", distinct: true, alias: "n" }),
    "(COUNT(*) AS ?n)",
  );
  assert.equal(
    renderAggregate({ id: "g", fn: "GROUP_CONCAT", target: "x", distinct: false, alias: "xs", separator: ", " }),
    '(GROUP_CONCAT(?x; SEPARATOR=", ") AS ?xs)',
  );
});

test("projectedVariables – canvas order, node before its own attributes", () => {
  const m = model({
    nodes: [
      node({
        variable: "person",
        projected: true,
        attributes: [attr({ variable: "personName", projected: true }), attr({ variable: "skip" })],
      }),
      node({ variable: "other", projected: true, attributes: [] }),
    ],
  });
  assert.deepEqual(projectedVariables(m), ["person", "personName", "other"]);
});

// ---------------------------------------------------------------------------
// Validation.
// ---------------------------------------------------------------------------

/** All error messages, for terse assertions. */
function errors(m: BuilderModel): string[] {
  return validateModel(m)
    .filter((i) => i.severity === "error")
    .map((i) => i.message);
}

test("validateModel – a well-formed diagram has no issues", () => {
  assert.deepEqual(validateModel(personModel()), []);
});

test("validateModel – an empty canvas is an error, not a silently-empty query", () => {
  assert.match(errors(emptyModel())[0], /canvas is empty/);
});

test("validateModel – the same variable bound twice is an error (it would silently join)", () => {
  const m = model({
    nodes: [node({ variable: "x" , classIri: `${FOAF}Person` }), node({ variable: "x", classIri: `${FOAF}Agent` })],
  });
  assert.ok(errors(m).some((e) => /bound 2 times/.test(e)));
});

test("validateModel – an invalid variable name is an error", () => {
  const m = model({ nodes: [node({ variable: "no-dashes", classIri: `${FOAF}Person` })] });
  assert.ok(errors(m).some((e) => /not a valid variable name/.test(e)));
});

test("validateModel – a node with no class, attribute or edge matches nothing", () => {
  assert.ok(errors(model({ nodes: [node({ variable: "lonely" })] })).some((e) => /matches nothing/.test(e)));
});

test("validateModel – projecting a variable bound only inside NOT EXISTS is an error", () => {
  const m = model({
    nodes: [
      node({ variable: "person", classIri: `${FOAF}Person`, projected: true }),
      node({ variable: "blocked", projected: true, attributes: [attr({ variable: "bn" })] }),
    ],
    edges: [
      { id: "e1", from: "n-person", to: "n-blocked", predicate: `${FOAF}knows`, alternatives: [], mode: "not-exists" },
    ],
  });
  assert.ok(errors(m).some((e) => /only inside a NOT-EXISTS branch/.test(e)));
});

test("validateModel – a NOT-EXISTS branch must be a leaf in v1", () => {
  const m = model({
    nodes: [
      node({ variable: "person", classIri: `${FOAF}Person`, projected: true }),
      node({ variable: "blocked", attributes: [attr({ variable: "bn" })] }),
      node({ variable: "third", attributes: [attr({ variable: "tn" })] }),
    ],
    edges: [
      { id: "e1", from: "n-person", to: "n-blocked", predicate: `${FOAF}knows`, alternatives: [], mode: "not-exists" },
      { id: "e2", from: "n-blocked", to: "n-third", predicate: `${FOAF}knows`, alternatives: [], mode: "required" },
    ],
  });
  assert.ok(errors(m).some((e) => /must be a leaf in v1/.test(e)));
});

test("validateModel – a projected non-grouped variable alongside an aggregate is an error", () => {
  const m = model({
    nodes: [
      node({
        variable: "person",
        classIri: `${FOAF}Person`,
        projected: true,
        attributes: [attr({ variable: "personName", projected: true })],
      }),
    ],
    aggregates: [{ id: "g1", fn: "COUNT", target: "person", distinct: false, alias: "n" }],
    groupBy: ["person"],
  });
  // ?person is grouped, ?personName is not — SPARQL rejects that, so the builder must too.
  const errs = errors(m);
  assert.ok(errs.some((e) => /\?personName/.test(e) && /GROUP BY/.test(e)));
  assert.ok(!errs.some((e) => /\?person\b(?!Name)/.test(e) && /not in GROUP BY/.test(e)));
});

test("validateModel – a string function on a non-string value kind is an error", () => {
  const m = model({
    nodes: [
      node({
        variable: "p",
        projected: true,
        attributes: [
          attr({ variable: "age", filter: { op: "contains", value: "1", kind: "number" } }),
        ],
      }),
    ],
  });
  assert.ok(errors(m).some((e) => /string function/.test(e)));
});

test("validateModel – a numeric filter whose value is not a number is an error", () => {
  const m = model({
    nodes: [
      node({
        variable: "p",
        projected: true,
        attributes: [attr({ variable: "age", filter: { op: ">", value: "old", kind: "number" } })],
      }),
    ],
  });
  assert.ok(errors(m).some((e) => /not a number/.test(e)));
});

test("validateModel – an undeclared prefix is a WARNING, and the query still builds", () => {
  const m = model({ nodes: [node({ variable: "p", classIri: "acme:Widget", projected: true })] });
  const issues = validateModel(m);
  assert.equal(issues.filter((i) => i.severity === "error").length, 0);
  assert.ok(issues.some((i) => i.severity === "warning" && /acme:/.test(i.message)));
  assert.equal(buildQuery(m).runnable, true);
});

test("buildQuery – an invalid model STILL produces readable SPARQL, just not runnable", () => {
  const built = buildQuery(model({ nodes: [node({ variable: "lonely" })] }));
  assert.equal(built.runnable, false);
  assert.match(built.sparql, /SELECT/);
});

test("validateModel – LIMIT must be a positive whole number", () => {
  assert.ok(errors({ ...personModel(), limit: 0 }).some((e) => /positive whole number/.test(e)));
  assert.ok(errors({ ...personModel(), limit: 2.5 }).some((e) => /positive whole number/.test(e)));
});

test("validateModel – alternative predicates are rejected on a non-required edge", () => {
  const m = model({
    nodes: [node({ variable: "a", classIri: `${FOAF}Person` }), node({ variable: "b", classIri: `${FOAF}Person` })],
    edges: [
      { id: "e1", from: "n-a", to: "n-b", predicate: `${FOAF}knows`, alternatives: [`${FOAF}made`], mode: "optional" },
    ],
  });
  assert.ok(errors(m).some((e) => /only available on a required edge/.test(e)));
});

// ---------------------------------------------------------------------------
// Schema introspection parsers.
// ---------------------------------------------------------------------------

function results(vars: string[], bindings: Record<string, SparqlTerm>[]): SparqlResults {
  return { head: { vars }, results: { bindings } };
}

const uri = (value: string): SparqlTerm => ({ type: "uri", value });
const lit = (value: string): SparqlTerm => ({ type: "literal", value });

test("parseClassRows – reads class + instance count, dropping rows without a class IRI", () => {
  const parsed = parseClassRows(
    results(
      ["class", "instances"],
      [
        { class: uri(`${FOAF}Person`), instances: lit("3") },
        { instances: lit("9") },
        { class: lit("not-an-iri"), instances: lit("1") },
      ],
    ),
  );
  assert.deepEqual(parsed, [{ classIri: `${FOAF}Person`, instances: 3, source: "data" }]);
});

test("parseCharacteristicRows – groups observed predicates by class and reads the sample kind", () => {
  const parsed = parseCharacteristicRows(
    results(
      ["class", "p", "uses", "sample"],
      [
        { class: uri(`${FOAF}Person`), p: uri(`${FOAF}name`), uses: lit("7"), sample: lit("Alice") },
        { class: uri(`${FOAF}Person`), p: uri(`${FOAF}knows`), uses: lit("2"), sample: uri("http://e.org/b") },
      ],
    ),
  );
  const list = parsed.get(`${FOAF}Person`);
  assert.equal(list?.length, 2);
  assert.deepEqual(list?.[0], {
    predicate: `${FOAF}name`,
    source: "data",
    uses: 7,
    objectKind: "literal",
    valueType: null,
  });
  assert.equal(list?.[1].objectKind, "iri");
});

test("parseAnyPredicateRows – the untyped-node fallback list", () => {
  const parsed = parseAnyPredicateRows(
    results(["p", "uses", "sample"], [{ p: uri(`${FOAF}name`), uses: lit("4"), sample: lit("x") }]),
  );
  assert.deepEqual(parsed, [
    { predicate: `${FOAF}name`, source: "data", uses: 4, objectKind: "literal", valueType: null },
  ]);
});

test("parseShapeRows – reads sh:datatype / sh:class / sh:nodeKind into the object kind", () => {
  const parsed = parseShapeRows(
    results(
      ["class", "path", "datatype", "nodeKind", "valueClass"],
      [
        {
          class: uri(`${FOAF}Person`),
          path: uri(`${FOAF}name`),
          datatype: uri("http://www.w3.org/2001/XMLSchema#string"),
        },
        { class: uri(`${FOAF}Person`), path: uri(`${FOAF}knows`), valueClass: uri(`${FOAF}Person`) },
        {
          class: uri(`${FOAF}Person`),
          path: uri(`${FOAF}homepage`),
          nodeKind: uri("http://www.w3.org/ns/shacl#IRI"),
        },
      ],
    ),
  );
  const list = parsed.get(`${FOAF}Person`) ?? [];
  assert.deepEqual(
    list.map((s) => [s.predicate, s.objectKind, s.valueType, s.source, s.uses]),
    [
      [`${FOAF}name`, "literal", "http://www.w3.org/2001/XMLSchema#string", "shape", null],
      [`${FOAF}knows`, "iri", `${FOAF}Person`, "shape", null],
      [`${FOAF}homepage`, "iri", null, "shape", null],
    ],
  );
});

test("parseShapeRows – no shapes in the store yields no shape suggestions (never invented)", () => {
  assert.equal(parseShapeRows(results(["class", "path"], [])).size, 0);
  assert.equal(parseShapeRows(null).size, 0);
});

test("mergeSuggestions – shape-declared lead; a predicate in both keeps both facts", () => {
  const merged = mergeSuggestions(
    [{ predicate: `${FOAF}name`, source: "shape", uses: null, objectKind: "literal", valueType: "xsd:string" }],
    [
      { predicate: `${FOAF}name`, source: "data", uses: 12, objectKind: "literal", valueType: null },
      { predicate: `${FOAF}nick`, source: "data", uses: 3, objectKind: "literal", valueType: null },
    ],
  );
  assert.equal(merged.length, 2);
  // The shape entry keeps its declared type AND gains the observed count.
  assert.deepEqual(merged[0], {
    predicate: `${FOAF}name`,
    source: "shape",
    uses: 12,
    objectKind: "literal",
    valueType: "xsd:string",
  });
  assert.equal(merged[1].source, "data");
});

test("buildSchemaIndex – shape-only classes are offered with a NULL instance count", () => {
  const index = buildSchemaIndex({
    classes: results(["class", "instances"], [{ class: uri(`${FOAF}Person`), instances: lit("2") }]),
    characteristics: results(
      ["class", "p", "uses", "sample"],
      [{ class: uri(`${FOAF}Person`), p: uri(`${FOAF}name`), uses: lit("2"), sample: lit("a") }],
    ),
    anyPredicates: results(["p", "uses", "sample"], [{ p: uri(`${FOAF}name`), uses: lit("2"), sample: lit("a") }]),
    shapes: results(
      ["class", "path"],
      [{ class: uri("http://example.org/Widget"), path: uri("http://example.org/sku") }],
    ),
  });
  assert.equal(index.hasShapes, true);
  const widget = index.classes.find((c) => c.classIri === "http://example.org/Widget");
  assert.deepEqual(widget, { classIri: "http://example.org/Widget", instances: null, source: "shape" });
  assert.equal(index.classes.find((c) => c.classIri === `${FOAF}Person`)?.instances, 2);
});

test("buildSchemaIndex – no shapes anywhere leaves hasShapes false", () => {
  const index = buildSchemaIndex({
    classes: null,
    characteristics: null,
    anyPredicates: null,
    shapes: results(["class", "path"], []),
  });
  assert.equal(index.hasShapes, false);
  assert.deepEqual(index.classes, []);
});

test("suggestionsFor – an untyped node falls back to the store-wide predicate list", () => {
  const index = buildSchemaIndex({
    classes: null,
    characteristics: results(
      ["class", "p", "uses", "sample"],
      [{ class: uri(`${FOAF}Person`), p: uri(`${FOAF}name`), uses: lit("1"), sample: lit("a") }],
    ),
    anyPredicates: results(["p", "uses", "sample"], [{ p: uri(`${FOAF}seeAlso`), uses: lit("5"), sample: uri("http://e.org/x") }]),
    shapes: null,
  });
  assert.deepEqual(
    suggestionsFor(index, null).map((s) => s.predicate),
    [`${FOAF}seeAlso`],
  );
  assert.deepEqual(
    suggestionsFor(index, `${FOAF}Person`).map((s) => s.predicate),
    [`${FOAF}name`],
  );
  // An unknown class falls back rather than showing an empty picker.
  assert.deepEqual(
    suggestionsFor(index, "http://example.org/Unknown").map((s) => s.predicate),
    [`${FOAF}seeAlso`],
  );
});

// ---------------------------------------------------------------------------
// Naming helpers.
// ---------------------------------------------------------------------------

test("variableStem – last segment, lower-camelled and stripped to variable-safe characters", () => {
  assert.equal(variableStem(`${FOAF}Person`), "person");
  assert.equal(variableStem("http://example.org/vocab#hasPart"), "hasPart");
  assert.equal(variableStem("foaf:name"), "name");
  // A trailing separator carries no segment — fall back to the authority, not the whole IRI.
  assert.equal(variableStem("http://example.org/"), "exampleorg");
  assert.equal(variableStem("///"), "node");
});

test("uniqueVariable – suffixes until free", () => {
  assert.equal(uniqueVariable("person", []), "person");
  assert.equal(uniqueVariable("person", ["person"]), "person2");
  assert.equal(uniqueVariable("person", ["person", "person2"]), "person3");
});

test("iriLabel / abbreviateIri – prefixed form when possible, else the last segment", () => {
  assert.equal(abbreviateIri(`${FOAF}name`), "foaf:name");
  assert.equal(abbreviateIri("http://data.example.com/x"), null);
  assert.equal(iriLabel(`${FOAF}name`), "foaf:name");
  assert.equal(iriLabel("http://data.example.com/weight"), "weight");
});
