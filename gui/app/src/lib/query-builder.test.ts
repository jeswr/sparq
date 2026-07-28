// [OPUS-5] sq-ixc3.24 — unit tests for the visual query builder's PURE core.
//
// Covers the lowering (class triples, attribute filters + their FILTER forms, OPTIONAL branches,
// the AND-NOT / FILTER NOT EXISTS form, aggregates + GROUP BY, DISTINCT / ORDER BY / LIMIT,
// prefix abbreviation, variable uniquification, IRI + literal escaping) and the introspection
// parsers + the shapes ⊕ data suggestion merge. The canvas itself is covered by the Playwright
// mocked-IPC spec (gui/e2e-playwright/specs/query-builder.web.spec.ts).
//
// Run via:   npm run test:unit   (gui/app)
import { test } from "node:test";
import assert from "node:assert/strict";
import type { SparqlResults } from "@sparq/client";

import {
  asNumericLiteral,
  buildSparql,
  classesQuery,
  emptyModel,
  linkTargetsQuery,
  localName,
  mergeSuggestions,
  parseClassRows,
  parsePredicateRows,
  parseShapeRows,
  predicatesQuery,
  renderIri,
  renderLiteral,
  sanitizeVariable,
  shapesQuery,
  type AttributeFilter,
  type BuilderEdge,
  type BuilderModel,
  type BuilderNode,
  type PredicateStat,
  type PredicateSuggestion,
  type ShapeProperty,
} from "./query-builder.js";

const FOAF = "http://xmlns.com/foaf/0.1/";
const EX = "http://example.org/";

// ---------------------------------------------------------------------------
// builders
// ---------------------------------------------------------------------------

function node(partial: Partial<BuilderNode> & { id: string; variable: string }): BuilderNode {
  return {
    classIri: null,
    x: 0,
    y: 0,
    project: false,
    aggregate: null,
    filters: [],
    ...partial,
  };
}

function filter(partial: Partial<AttributeFilter> & { id: string; predicateIri: string }): AttributeFilter {
  return {
    variable: "value",
    op: "any",
    value: "",
    valueKind: "text",
    project: false,
    aggregate: null,
    ...partial,
  };
}

function edge(partial: Partial<BuilderEdge> & { id: string; from: string; to: string }): BuilderEdge {
  return { predicateIri: `${FOAF}knows`, mode: "required", ...partial };
}

function model(partial: Partial<BuilderModel>): BuilderModel {
  return { ...emptyModel(), limit: null, ...partial };
}

/** SPARQL text with runs of whitespace collapsed — for order-insensitive containment checks. */
const flat = (s: string) => s.replace(/\s+/g, " ").trim();

// ---------------------------------------------------------------------------
// term rendering
// ---------------------------------------------------------------------------

test("renderIri abbreviates well-known namespaces and records the prefix used", () => {
  const used = new Map<string, string>();
  assert.equal(renderIri(`${FOAF}Person`, used), "foaf:Person");
  assert.deepEqual([...used.keys()], ["foaf"]);
});

test("renderIri falls back to a full IRIREF for unknown namespaces and unsafe local parts", () => {
  assert.equal(renderIri("http://acme.test/thing"), "<http://acme.test/thing>");
  // A local part with a slash cannot be abbreviated safely.
  assert.equal(renderIri(`${FOAF}a/b`), `<${FOAF}a/b>`);
  // The bare namespace has an empty local part — never abbreviated.
  assert.equal(renderIri(FOAF), `<${FOAF}>`);
});

test("renderIri percent-encodes characters an IRIREF may not contain", () => {
  const out = renderIri("http://acme.test/a b<c>");
  assert.equal(out, "<http://acme.test/a%20b%3Cc%3E>");
  assert.ok(!/[ <>]/.test(out.slice(1, -1)));
});

test("renderLiteral escapes quotes, backslashes and newlines", () => {
  assert.equal(renderLiteral('a"b\\c\nd'), '"a\\"b\\\\c\\nd"');
});

test("sanitizeVariable keeps legal VARNAMEs and repairs the rest", () => {
  assert.equal(sanitizeVariable("?person"), "person");
  assert.equal(sanitizeVariable("first name"), "first_name");
  assert.equal(sanitizeVariable("1st"), "1st"); // a leading digit is legal in VARNAME
  assert.equal(sanitizeVariable("!!!", "fallback"), "fallback");
});

test("asNumericLiteral accepts SPARQL numeric forms and rejects the rest", () => {
  assert.equal(asNumericLiteral(" 42 "), "42");
  assert.equal(asNumericLiteral("-3.5"), "-3.5");
  assert.equal(asNumericLiteral("1e6"), "1e6");
  assert.equal(asNumericLiteral("twelve"), null);
});

test("localName takes the part after the last # or /", () => {
  assert.equal(localName(`${FOAF}name`), "name");
  assert.equal(localName(`${EX}a/b`), "b");
  assert.equal(localName("urn:x"), "urn:x");
});

// ---------------------------------------------------------------------------
// lowering
// ---------------------------------------------------------------------------

test("an empty canvas lowers to a valid, honest SELECT *", () => {
  const built = buildSparql(emptyModel());
  assert.equal(flat(built.sparql), "SELECT * WHERE { } LIMIT 100");
  assert.deepEqual(built.warnings, []);
});

test("a typed node with a projected attribute lowers to the expected SPARQL", () => {
  const built = buildSparql(
    model({
      nodes: [
        node({
          id: "n1",
          variable: "person",
          classIri: `${FOAF}Person`,
          project: true,
          filters: [
            filter({ id: "f1", predicateIri: `${FOAF}name`, variable: "name", project: true }),
          ],
        }),
      ],
    }),
  );
  assert.equal(
    built.sparql,
    [
      `PREFIX foaf: <${FOAF}>`,
      "",
      "SELECT ?person ?name",
      "WHERE {",
      "  ?person a foaf:Person .",
      "  ?person foaf:name ?name .",
      "}",
    ].join("\n"),
  );
  assert.deepEqual(built.projected, ["person", "name"]);
  assert.deepEqual(built.warnings, []);
});

test("only the prefixes the body actually uses are declared", () => {
  const built = buildSparql(
    model({ nodes: [node({ id: "n1", variable: "s", classIri: "http://acme.test/Widget" })] }),
  );
  assert.ok(!built.sparql.includes("PREFIX"));
  assert.ok(built.sparql.includes("<http://acme.test/Widget>"));
});

test("each comparison op lowers to its SPARQL form", () => {
  const cases: [AttributeFilter["op"], AttributeFilter["valueKind"], string, string][] = [
    ["eq", "text", "Alice", 'FILTER(?v = "Alice")'],
    ["ne", "text", "Alice", 'FILTER(?v != "Alice")'],
    ["gt", "number", "30", "FILTER(?v > 30)"],
    ["le", "number", "30", "FILTER(?v <= 30)"],
    ["eq", "iri", `${EX}alice`, "FILTER(?v = ex:alice)"],
    ["contains", "text", "ali", 'FILTER(CONTAINS(LCASE(STR(?v)), LCASE("ali")))'],
    ["starts", "text", "Al", 'FILTER(STRSTARTS(LCASE(STR(?v)), LCASE("Al")))'],
    ["ends", "text", "ce", 'FILTER(STRENDS(LCASE(STR(?v)), LCASE("ce")))'],
    ["regex", "text", "^A", 'FILTER(REGEX(STR(?v), "^A", "i"))'],
  ];
  for (const [op, valueKind, value, expected] of cases) {
    const built = buildSparql(
      model({
        nodes: [
          node({
            id: "n1",
            variable: "s",
            filters: [filter({ id: "f1", predicateIri: `${EX}p`, variable: "v", op, value, valueKind })],
          }),
        ],
      }),
    );
    assert.ok(built.sparql.includes(expected), `${op}/${valueKind} → ${built.sparql}`);
  }
});

test("op 'any' binds the value with no FILTER; an empty value warns instead of emitting a broken one", () => {
  const any = buildSparql(
    model({
      nodes: [
        node({
          id: "n1",
          variable: "s",
          filters: [filter({ id: "f1", predicateIri: `${EX}p`, variable: "v", op: "any" })],
        }),
      ],
    }),
  );
  assert.ok(any.sparql.includes("?s ex:p ?v ."));
  assert.ok(!any.sparql.includes("FILTER"));

  const blank = buildSparql(
    model({
      nodes: [
        node({
          id: "n1",
          variable: "s",
          filters: [filter({ id: "f1", predicateIri: `${EX}p`, variable: "v", op: "eq", value: "  " })],
        }),
      ],
    }),
  );
  assert.ok(!blank.sparql.includes("FILTER"));
  assert.equal(blank.warnings.length, 1);
  assert.match(blank.warnings[0], /no value/);
});

test("a non-numeric value in a number filter is compared as text AND warned about", () => {
  const built = buildSparql(
    model({
      nodes: [
        node({
          id: "n1",
          variable: "s",
          filters: [
            filter({
              id: "f1",
              predicateIri: `${EX}age`,
              variable: "age",
              op: "gt",
              value: "thirty",
              valueKind: "number",
            }),
          ],
        }),
      ],
    }),
  );
  assert.ok(built.sparql.includes('FILTER(?age > "thirty")'));
  assert.match(built.warnings.join(" "), /not a number/);
});

test("op 'absent' becomes FILTER NOT EXISTS, binds nothing, and refuses to be projected", () => {
  const built = buildSparql(
    model({
      nodes: [
        node({
          id: "n1",
          variable: "person",
          classIri: `${FOAF}Person`,
          project: true,
          filters: [
            filter({
              id: "f1",
              predicateIri: `${FOAF}mbox`,
              variable: "mbox",
              op: "absent",
              project: true,
            }),
          ],
        }),
      ],
    }),
  );
  assert.ok(built.sparql.includes("FILTER NOT EXISTS { ?person foaf:mbox ?mbox }"));
  assert.deepEqual(built.projected, ["person"]);
  assert.match(built.warnings.join(" "), /ABSENT/);
});

test("an optional link puts the leaf target's own patterns INSIDE the OPTIONAL group", () => {
  const built = buildSparql(
    model({
      nodes: [
        node({ id: "n1", variable: "person", classIri: `${FOAF}Person`, project: true }),
        node({ id: "n2", variable: "employer", classIri: `${EX}Company`, project: true }),
      ],
      edges: [
        edge({ id: "e1", from: "n1", to: "n2", predicateIri: `${EX}worksAt`, mode: "optional" }),
      ],
    }),
  );
  assert.equal(
    flat(built.sparql).includes("OPTIONAL { ?person ex:worksAt ?employer . ?employer a ex:Company . }"),
    true,
    built.sparql,
  );
  // The target's class triple must NOT also sit in the mandatory part.
  assert.equal(built.sparql.split("?employer a").length - 1, 1);
  assert.deepEqual(built.projected, ["person", "employer"]);
});

test("an AND-NOT link becomes FILTER NOT EXISTS and its leaf target is not projectable", () => {
  const built = buildSparql(
    model({
      nodes: [
        node({ id: "n1", variable: "person", classIri: `${FOAF}Person`, project: true }),
        node({ id: "n2", variable: "boss", classIri: `${EX}Manager`, project: true }),
      ],
      edges: [edge({ id: "e1", from: "n1", to: "n2", predicateIri: `${FOAF}knows`, mode: "not" })],
    }),
  );
  assert.ok(
    flat(built.sparql).includes("FILTER NOT EXISTS { ?person foaf:knows ?boss . ?boss a ex:Manager . }"),
    built.sparql,
  );
  assert.deepEqual(built.projected, ["person"]);
  assert.match(built.warnings.join(" "), /AND-NOT/);
});

test("a required link keeps both endpoints in the mandatory BGP", () => {
  const built = buildSparql(
    model({
      nodes: [
        node({ id: "n1", variable: "person", classIri: `${FOAF}Person`, project: true }),
        node({ id: "n2", variable: "friend", classIri: `${FOAF}Person`, project: true }),
      ],
      edges: [edge({ id: "e1", from: "n1", to: "n2" })],
    }),
  );
  assert.ok(!built.sparql.includes("OPTIONAL"));
  assert.ok(built.sparql.includes("?person foaf:knows ?friend ."));
  assert.ok(built.sparql.includes("?friend a foaf:Person ."));
});

test("aggregates produce a GROUP BY over exactly the non-aggregated projected items", () => {
  const built = buildSparql(
    model({
      nodes: [
        node({
          id: "n1",
          variable: "person",
          classIri: `${FOAF}Person`,
          project: true,
          aggregate: "count-distinct",
        }),
        node({ id: "n2", variable: "company", classIri: `${EX}Company`, project: true }),
      ],
      edges: [edge({ id: "e1", from: "n1", to: "n2", predicateIri: `${EX}worksAt` })],
      orderBy: { variable: "person_count", desc: true },
      limit: 10,
    }),
  );
  assert.ok(built.sparql.includes("(COUNT(DISTINCT ?person) AS ?person_count)"), built.sparql);
  assert.ok(built.sparql.includes("GROUP BY ?company"));
  assert.ok(built.sparql.includes("ORDER BY DESC(?person_count)"));
  assert.ok(built.sparql.endsWith("LIMIT 10"));
  assert.deepEqual(built.groupBy, ["company"]);
});

test("an all-aggregate projection emits no GROUP BY", () => {
  const built = buildSparql(
    model({
      nodes: [node({ id: "n1", variable: "s", classIri: `${FOAF}Person`, project: true, aggregate: "count" })],
    }),
  );
  assert.ok(built.sparql.includes("(COUNT(?s) AS ?s_count)"));
  assert.ok(!built.sparql.includes("GROUP BY"));
  assert.deepEqual(built.groupBy, []);
});

test("DISTINCT and an out-of-result ORDER BY are handled honestly", () => {
  const built = buildSparql(
    model({
      distinct: true,
      orderBy: { variable: "nope", desc: false },
      nodes: [node({ id: "n1", variable: "s", classIri: `${FOAF}Person`, project: true })],
    }),
  );
  assert.ok(built.sparql.startsWith("PREFIX"));
  assert.ok(built.sparql.includes("SELECT DISTINCT ?s"));
  assert.ok(!built.sparql.includes("ORDER BY"));
  assert.match(built.warnings.join(" "), /ORDER BY \?nope is not in the result/);
});

test("duplicate variable names are uniquified across nodes and attributes", () => {
  const built = buildSparql(
    model({
      nodes: [
        node({
          id: "n1",
          variable: "x",
          project: true,
          classIri: `${EX}A`,
          filters: [filter({ id: "f1", predicateIri: `${EX}p`, variable: "x", project: true })],
        }),
        node({ id: "n2", variable: "x", project: true, classIri: `${EX}B` }),
      ],
    }),
  );
  assert.deepEqual(built.projected, ["x", "x2", "x3"]);
  assert.equal(new Set(built.projected).size, 3);
});

test("an edge pointing at a deleted node is dropped with a warning, never silently", () => {
  const built = buildSparql(
    model({
      nodes: [node({ id: "n1", variable: "s", classIri: `${EX}A`, project: true })],
      edges: [edge({ id: "e1", from: "n1", to: "ghost" })],
    }),
  );
  assert.ok(!built.sparql.includes("ghost"));
  assert.match(built.warnings.join(" "), /no longer exists/);
});

test("a link whose predicate has not been chosen yet is skipped with a warning", () => {
  const built = buildSparql(
    model({
      nodes: [
        node({ id: "n1", variable: "a", classIri: `${EX}A`, project: true }),
        node({ id: "n2", variable: "b", classIri: `${EX}B`, project: true }),
      ],
      edges: [edge({ id: "e1", from: "n1", to: "n2", predicateIri: "" })],
    }),
  );
  // Never emit `?a <> ?b` — a relative IRI that would silently resolve against the base.
  assert.ok(!built.sparql.includes("<>"));
  assert.match(built.warnings.join(" "), /has no predicate yet/);
  // Both endpoints still stand on their own class patterns.
  assert.ok(built.sparql.includes("?a a ex:A ."));
  assert.ok(built.sparql.includes("?b a ex:B ."));
});

test("an attribute with no predicate binds nothing and is not projected", () => {
  const built = buildSparql(
    model({
      nodes: [
        node({
          id: "n1",
          variable: "s",
          classIri: `${EX}A`,
          project: true,
          filters: [filter({ id: "f1", predicateIri: "  ", variable: "v", project: true })],
        }),
      ],
    }),
  );
  assert.deepEqual(built.projected, ["s"]);
  assert.ok(!built.sparql.includes("?v"));
  assert.match(built.warnings.join(" "), /has no predicate yet/);
});

test("a node that binds nothing but is projected is called out", () => {
  const built = buildSparql(model({ nodes: [node({ id: "n1", variable: "lonely", project: true })] }));
  assert.match(built.warnings.join(" "), /nothing binds it/);
});

test("projecting nothing falls back to SELECT * with a warning", () => {
  const built = buildSparql(
    model({ nodes: [node({ id: "n1", variable: "s", classIri: `${FOAF}Person` })] }),
  );
  assert.ok(built.sparql.includes("SELECT *"));
  assert.match(built.warnings.join(" "), /Nothing is selected/);
});

// ---------------------------------------------------------------------------
// introspection queries + parsers
// ---------------------------------------------------------------------------

test("the introspection queries are plain SPARQL over the live store", () => {
  assert.match(classesQuery(5), /SELECT \?class \(COUNT\(DISTINCT \?s\) AS \?instances\)/);
  assert.ok(classesQuery(5).endsWith("LIMIT 5"));
  // A class filter is applied when one is chosen, and omitted for the untyped case. The IRI is
  // written in full: these queries carry no prefix header, so an abbreviation would not parse.
  assert.ok(predicatesQuery(`${FOAF}Person`).includes(`?s a <${FOAF}Person> .`));
  assert.ok(!predicatesQuery(`${FOAF}Person`).includes("foaf:"));
  assert.ok(!predicatesQuery(null).includes(" a "));
  // rdf:type is excluded from the characteristic set (it is the class itself).
  assert.ok(predicatesQuery(null).includes("22-rdf-syntax-ns#type"));
  assert.ok(linkTargetsQuery(null).includes("?o a ?objectClass ."));
  assert.ok(shapesQuery().includes("sh:targetClass"));
  // A blank-node (complex path) shape property is not guessed at.
  assert.ok(shapesQuery().includes("FILTER(isIRI(?path))"));
});

/** Minimal SPARQL-JSON builder for the parser tests. */
function results(vars: string[], rows: Record<string, { type: "uri" | "literal"; value: string }>[]): SparqlResults {
  return { head: { vars }, results: { bindings: rows } };
}

test("parseClassRows keeps IRI classes with their counts and drops the rest", () => {
  const parsed = parseClassRows(
    results(
      ["class", "instances"],
      [
        { class: { type: "uri", value: `${FOAF}Person` }, instances: { type: "literal", value: "4" } },
        { class: { type: "literal", value: "not-a-class" }, instances: { type: "literal", value: "9" } },
      ],
    ),
  );
  assert.deepEqual(parsed, [{ iri: `${FOAF}Person`, instances: 4 }]);
});

test("parsePredicateRows folds the link-target probe into the characteristic set", () => {
  const parsed = parsePredicateRows(
    results(
      ["p", "uses", "literals"],
      [
        {
          p: { type: "uri", value: `${FOAF}name` },
          uses: { type: "literal", value: "4" },
          literals: { type: "literal", value: "4" },
        },
        {
          p: { type: "uri", value: `${FOAF}knows` },
          uses: { type: "literal", value: "3" },
          literals: { type: "literal", value: "0" },
        },
      ],
    ),
    results(
      ["p", "objectClass", "uses"],
      [
        {
          p: { type: "uri", value: `${FOAF}knows` },
          objectClass: { type: "uri", value: `${FOAF}Person` },
          uses: { type: "literal", value: "3" },
        },
        // A duplicate object class must not be recorded twice.
        {
          p: { type: "uri", value: `${FOAF}knows` },
          objectClass: { type: "uri", value: `${FOAF}Person` },
          uses: { type: "literal", value: "1" },
        },
      ],
    ),
  );
  assert.deepEqual(parsed, [
    { iri: `${FOAF}name`, uses: 4, literalUses: 4, objectClasses: [] },
    { iri: `${FOAF}knows`, uses: 3, literalUses: 0, objectClasses: [`${FOAF}Person`] },
  ]);
});

test("parseShapeRows keeps one entry per (targetClass, path) and reads minCount as required", () => {
  const parsed = parseShapeRows(
    results(
      ["targetClass", "path", "datatype", "name", "minCount"],
      [
        {
          targetClass: { type: "uri", value: `${FOAF}Person` },
          path: { type: "uri", value: `${FOAF}name` },
          datatype: { type: "uri", value: "http://www.w3.org/2001/XMLSchema#string" },
          name: { type: "literal", value: "Full name" },
          minCount: { type: "literal", value: "1" },
        },
        {
          targetClass: { type: "uri", value: `${FOAF}Person` },
          path: { type: "uri", value: `${FOAF}name` },
          minCount: { type: "literal", value: "1" },
        },
      ],
    ),
  );
  assert.equal(parsed.length, 1);
  assert.equal(parsed[0].required, true);
  assert.equal(parsed[0].name, "Full name");
  assert.equal(parsed[0].datatype, "http://www.w3.org/2001/XMLSchema#string");
});

// ---------------------------------------------------------------------------
// suggestion merge — the shape-aware half
// ---------------------------------------------------------------------------

const dataStats: PredicateStat[] = [
  { iri: `${FOAF}name`, uses: 10, literalUses: 10, objectClasses: [] },
  { iri: `${FOAF}knows`, uses: 6, literalUses: 0, objectClasses: [`${FOAF}Person`] },
  { iri: `${EX}note`, uses: 4, literalUses: 2, objectClasses: [`${EX}Note`] },
];

const shapeProps: ShapeProperty[] = [
  {
    targetClass: `${FOAF}Person`,
    path: `${FOAF}name`,
    objectClass: null,
    datatype: "http://www.w3.org/2001/XMLSchema#string",
    name: "Full name",
    required: true,
  },
  {
    targetClass: `${FOAF}Person`,
    path: `${EX}homepage`,
    objectClass: `${EX}Page`,
    datatype: null,
    name: null,
    required: false,
  },
];

test("mergeSuggestions labels provenance and ranks shape-required first", () => {
  const merged = mergeSuggestions(dataStats, shapeProps);
  assert.deepEqual(
    merged.map((s) => [localName(s.iri), s.source, s.uses]),
    [
      ["name", "both", 10],
      // shape-declared but never used in the data — offered, honestly marked as unobserved
      ["homepage", "shape", null],
      ["knows", "data", 6],
      ["note", "data", 4],
    ],
  );
  assert.equal(merged[0].label, "Full name");
  assert.equal(merged[0].required, true);
});

test("mergeSuggestions classifies link / attribute / mixed from shapes then data", () => {
  const merged = mergeSuggestions(dataStats, shapeProps);
  const byName = new Map<string, PredicateSuggestion>(merged.map((s) => [localName(s.iri), s]));
  assert.equal(byName.get("name")!.kind, "attribute"); // sh:datatype
  assert.equal(byName.get("homepage")!.kind, "link"); // sh:class
  assert.equal(byName.get("knows")!.kind, "link"); // all-IRI objects
  assert.equal(byName.get("note")!.kind, "mixed"); // some literal, some not
  assert.deepEqual(byName.get("knows")!.objectClasses, [`${FOAF}Person`]);
});

test("mergeSuggestions works with no shapes at all (data-only pickers)", () => {
  const merged = mergeSuggestions(dataStats, []);
  assert.deepEqual(
    merged.map((s) => s.source),
    ["data", "data", "data"],
  );
  assert.deepEqual(merged.map((s) => s.uses), [10, 6, 4]);
});
