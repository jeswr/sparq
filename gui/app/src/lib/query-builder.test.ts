// [SONNET-4.6] sq-ixc3.24 (#2700) — acceptance coverage for the visual query builder's pure core.
//
// The load-bearing contract: the diagram serialises to STANDARD SPARQL 1.1 (no private dialect),
// deterministically, and every honesty note the panel shows (cartesian product, unconstrained
// node, unbound NOT-EXISTS variable, nothing projected) is produced by `buildSparql`, not by the
// UI. Suggestions always carry their provenance and are never invented.

import assert from "node:assert/strict";
import test from "node:test";
import type { SparqlResults } from "@sparq/client";
import {
  addAttribute,
  addNode,
  addRelationship,
  buildSparql,
  buildSuggestionIndex,
  connectNodes,
  emptyModel,
  localName,
  nextId,
  partitionSuggestions,
  removeNode,
  sanitizeVariable,
  suggestionsFor,
  uniqueVariable,
  type BuilderModel,
} from "./query-builder.js";

const EX = "http://example.org/";
const FOAF_NAME = "http://xmlns.com/foaf/0.1/name";
const FOAF_KNOWS = "http://xmlns.com/foaf/0.1/knows";

function results(vars: string[], bindings: SparqlResults["results"]["bindings"]): SparqlResults {
  return { head: { vars }, results: { bindings } };
}

// ── serialisation ───────────────────────────────────────────────────────────────────────────

test("an empty canvas produces an honest placeholder, not a fabricated query", () => {
  const built = buildSparql(emptyModel());
  assert.match(built.sparql, /^# The canvas is empty/);
  assert.deepEqual(built.projection, []);
  assert.ok(built.warnings.some((w) => w.includes("empty")));
});

test("a typed node with an attribute serialises to standard SPARQL with only the used prefixes", () => {
  const withNode = addNode(emptyModel(), { classIri: `${EX}Person` });
  const withAttribute = addAttribute(withNode.model, withNode.nodeId, FOAF_NAME);
  const built = buildSparql(withAttribute.model);

  assert.equal(
    built.sparql,
    `PREFIX foaf: <http://xmlns.com/foaf/0.1/>

SELECT ?person ?name
WHERE {
  ?person a <http://example.org/Person> .
  ?person foaf:name ?name .
}
LIMIT 100`,
  );
  assert.deepEqual(built.projection, ["person", "name"]);
  assert.deepEqual(built.warnings, []);
  // The rdf/rdfs/owl/xsd defaults are never declared unless something used them.
  assert.ok(!built.sparql.includes("PREFIX rdf:"));
});

test("walking a relationship emits the join and names the target from the predicate", () => {
  const start = addNode(emptyModel(), { classIri: `${EX}Person` });
  const walked = addRelationship(start.model, start.nodeId, FOAF_KNOWS);
  const built = buildSparql(walked.model);

  assert.ok(built.sparql.includes("?person foaf:knows ?knows ."));
  assert.deepEqual(built.projection, ["person", "knows"]);
  assert.deepEqual(built.warnings, []);
});

test("a shape-declared target class types the walked-to node", () => {
  const start = addNode(emptyModel(), { classIri: `${EX}Person` });
  const walked = addRelationship(start.model, start.nodeId, FOAF_KNOWS, {
    targetClass: `${EX}Person`,
  });
  const built = buildSparql(walked.model);

  assert.ok(built.sparql.includes("?person2 a <http://example.org/Person> ."));
  assert.ok(built.sparql.includes("?person foaf:knows ?person2 ."));
});

test("OPTIONAL, exclusion (AND-NOT) and predicate alternatives emit the standard constructs", () => {
  const start = addNode(emptyModel(), { classIri: `${EX}Person` });
  const optional = addRelationship(start.model, start.nodeId, FOAF_KNOWS);
  const excluded = addRelationship(optional.model, start.nodeId, `${EX}blocks`);
  const model: BuilderModel = {
    ...excluded.model,
    edges: excluded.model.edges.map((edge, i) =>
      i === 0
        ? { ...edge, optional: true, alternates: [`${EX}follows`] }
        : { ...edge, negated: true },
    ),
  };
  const built = buildSparql(model);

  assert.ok(built.sparql.includes("OPTIONAL {"), "OPTIONAL block emitted");
  assert.ok(built.sparql.includes("UNION"), "alternative predicates emitted as a UNION");
  assert.ok(
    built.sparql.includes("FILTER NOT EXISTS { ?person <http://example.org/blocks> ?blocks . }"),
    built.sparql,
  );
});

test("a projected variable that only appears inside NOT EXISTS is flagged as unbindable", () => {
  const start = addNode(emptyModel(), { classIri: `${EX}Person` });
  const excluded = addRelationship(start.model, start.nodeId, `${EX}blocks`);
  const model: BuilderModel = {
    ...excluded.model,
    edges: excluded.model.edges.map((edge) => ({ ...edge, negated: true })),
  };
  const built = buildSparql(model);

  assert.ok(
    built.warnings.some((w) => w.includes("NOT EXISTS") && w.includes("?blocks")),
    built.warnings.join(" | "),
  );
});

test("disconnected node groups are reported as a cross product, not silently joined", () => {
  const first = addNode(emptyModel(), { classIri: `${EX}Person` });
  const second = addNode(first.model, { classIri: `${EX}Project` });
  const built = buildSparql(second.model);

  assert.ok(
    built.warnings.some((w) => w.includes("cross product")),
    built.warnings.join(" | "),
  );
});

test("an unconstrained node is called out", () => {
  const built = buildSparql(addNode(emptyModel()).model);
  assert.ok(built.warnings.some((w) => w.includes("unconstrained")), built.warnings.join(" | "));
});

test("nothing ticked projects SELECT * and says so", () => {
  const added = addNode(emptyModel(), { classIri: `${EX}Person` });
  const model: BuilderModel = {
    ...added.model,
    nodes: added.model.nodes.map((n) => ({ ...n, projected: false })),
  };
  const built = buildSparql(model);

  assert.ok(built.sparql.includes("SELECT *"));
  assert.deepEqual(built.projection, []);
  assert.ok(built.warnings.some((w) => w.includes("SELECT *")));
});

test("attribute filters serialise as SPARQL 1.1 expressions, with an optional filter kept inside OPTIONAL", () => {
  const start = addNode(emptyModel(), { classIri: `${EX}Person` });
  const withName = addAttribute(start.model, start.nodeId, FOAF_NAME);
  const withAge = addAttribute(withName.model, start.nodeId, `${EX}age`);
  const model: BuilderModel = {
    ...withAge.model,
    nodes: withAge.model.nodes.map((node) => ({
      ...node,
      attributes: node.attributes.map((attribute) =>
        attribute.predicate === FOAF_NAME
          ? { ...attribute, filter: { op: "contains" as const, value: 'A"B', kind: "literal" as const } }
          : {
              ...attribute,
              optional: true,
              filter: { op: "ge" as const, value: "18", kind: "number" as const },
            },
      ),
    })),
  };
  const built = buildSparql(model);

  assert.ok(built.sparql.includes('FILTER(CONTAINS(STR(?name), "A\\"B"))'), built.sparql);
  assert.ok(
    built.sparql.includes("OPTIONAL {\n    ?person <http://example.org/age> ?age .\n    FILTER(?age >= 18)\n  }"),
    built.sparql,
  );
});

test("a non-numeric operand on a numeric comparison degrades to a string literal and warns", () => {
  const start = addNode(emptyModel(), { classIri: `${EX}Person` });
  const withAge = addAttribute(start.model, start.nodeId, `${EX}age`);
  const model: BuilderModel = {
    ...withAge.model,
    nodes: withAge.model.nodes.map((node) => ({
      ...node,
      attributes: node.attributes.map((a) => ({
        ...a,
        filter: { op: "gt" as const, value: "old", kind: "number" as const },
      })),
    })),
  };
  const built = buildSparql(model);

  assert.ok(built.sparql.includes('FILTER(?age > "old")'), built.sparql);
  assert.ok(built.warnings.some((w) => w.includes("not a number")));
});

test("the aggregate panel emits GROUP BY with aliased aggregates", () => {
  const start = addNode(emptyModel(), { classIri: `${EX}Person` });
  const withCity = addAttribute(start.model, start.nodeId, `${EX}city`);
  const model: BuilderModel = {
    ...withCity.model,
    aggregates: [
      { id: "g1", fn: "COUNT", variable: "person", distinct: true, alias: "people" },
    ],
    groupBy: ["city"],
    limit: null,
  };
  const built = buildSparql(model);

  assert.equal(
    built.sparql,
    `SELECT ?city (COUNT(DISTINCT ?person) AS ?people)
WHERE {
  ?person a <http://example.org/Person> .
  ?person <http://example.org/city> ?city .
}
GROUP BY ?city`,
  );
  assert.deepEqual(built.projection, ["city", "people"]);
  assert.ok(
    built.warnings.some((w) => w.includes("ticked but not grouped")),
    built.warnings.join(" | "),
  );
});

test("an aggregate over a variable the pattern never binds is flagged", () => {
  const start = addNode(emptyModel(), { classIri: `${EX}Person` });
  const model: BuilderModel = {
    ...start.model,
    nodes: start.model.nodes.map((n) => ({ ...n, projected: false })),
    aggregates: [{ id: "g1", fn: "SUM", variable: "salary", distinct: false, alias: "total" }],
    groupBy: [],
  };
  const built = buildSparql(model);

  assert.ok(built.sparql.includes("(SUM(?salary) AS ?total)"));
  assert.ok(!built.sparql.includes("GROUP BY"));
  assert.ok(built.warnings.some((w) => w.includes("never bound")), built.warnings.join(" | "));
});

test("serialisation is deterministic — the same model always yields identical text", () => {
  const start = addNode(emptyModel(), { classIri: `${EX}Person` });
  const walked = addRelationship(start.model, start.nodeId, FOAF_KNOWS);
  assert.equal(buildSparql(walked.model).sparql, buildSparql(walked.model).sparql);
});

test("connecting two existing nodes closes a cycle instead of growing a tree", () => {
  const person = addNode(emptyModel(), { classIri: `${EX}Person` });
  const org = addNode(person.model, { classIri: `${EX}Organisation` });
  const employed = connectNodes(org.model, person.nodeId, org.nodeId, `${EX}worksAt`);
  const founded = connectNodes(employed.model, person.nodeId, org.nodeId, `${EX}founded`);
  const built = buildSparql(founded.model);

  assert.ok(built.sparql.includes("?person <http://example.org/worksAt> ?organisation ."));
  assert.ok(built.sparql.includes("?person <http://example.org/founded> ?organisation ."));
  // Both nodes are now joined, so there is no cross-product warning.
  assert.deepEqual(built.warnings, []);
  assert.notEqual(employed.edgeId, founded.edgeId);
});

test("removing a node drops its edges and its GROUP BY entries", () => {
  const start = addNode(emptyModel(), { classIri: `${EX}Person` });
  const walked = addRelationship(start.model, start.nodeId, FOAF_KNOWS);
  const grouped: BuilderModel = { ...walked.model, groupBy: ["knows"] };
  const pruned = removeNode(grouped, walked.nodeId);

  assert.equal(pruned.nodes.length, 1);
  assert.deepEqual(pruned.edges, []);
  assert.deepEqual(pruned.groupBy, []);
});

// ── names + ids ─────────────────────────────────────────────────────────────────────────────

test("variable names are sanitised, made unique and derived from IRI local names", () => {
  assert.equal(localName("http://example.org/ns#Person"), "Person");
  assert.equal(localName("http://example.org/Person/"), "Person");
  assert.equal(sanitizeVariable("2 legs!"), "_2legs");
  assert.equal(sanitizeVariable(""), "v");
  assert.equal(uniqueVariable("person", ["person", "person2"]), "person3");
  assert.equal(nextId("n", ["n1", "n7", "e2"]), "n8");
});

// ── suggestions ─────────────────────────────────────────────────────────────────────────────

test("suggestions carry provenance and merge shape declarations with observed data", () => {
  const index = buildSuggestionIndex({
    classes: results(
      ["class", "instances"],
      [
        {
          class: { type: "uri", value: `${EX}Person` },
          instances: { type: "literal", value: "12" },
        },
      ],
    ),
    characteristicSets: results(
      ["class", "p", "uses", "sample"],
      [
        {
          class: { type: "uri", value: `${EX}Person` },
          p: { type: "uri", value: FOAF_NAME },
          uses: { type: "literal", value: "12" },
          sample: { type: "literal", value: "Alice" },
        },
        {
          class: { type: "uri", value: `${EX}Person` },
          p: { type: "uri", value: `${EX}nickname` },
          uses: { type: "literal", value: "3" },
          sample: { type: "literal", value: "Ally" },
        },
      ],
    ),
    shapes: results(
      ["targetClass", "path", "datatype", "objectClass", "nodeKind", "minCount", "name"],
      [
        {
          targetClass: { type: "uri", value: `${EX}Person` },
          path: { type: "uri", value: FOAF_NAME },
          datatype: { type: "uri", value: "http://www.w3.org/2001/XMLSchema#string" },
          minCount: { type: "literal", value: "1" },
          name: { type: "literal", value: "Full name" },
        },
        {
          targetClass: { type: "uri", value: `${EX}Person` },
          path: { type: "uri", value: FOAF_KNOWS },
          objectClass: { type: "uri", value: `${EX}Person` },
        },
      ],
    ),
  });

  assert.equal(index.shapesPresent, true);
  assert.deepEqual(
    index.classes.map((c) => c.iri),
    [`${EX}Person`],
  );

  const offer = suggestionsFor(index, `${EX}Person`);
  assert.deepEqual(offer.map((s) => s.predicate), [FOAF_NAME, FOAF_KNOWS, `${EX}nickname`]);

  const name = offer[0];
  assert.equal(name.source, "both", "declared by a shape AND observed in the data");
  assert.equal(name.count, 12);
  assert.equal(name.required, true);
  assert.equal(name.label, "Full name");
  assert.equal(name.objectKind, "literal");

  const knows = offer[1];
  assert.equal(knows.source, "shape", "declared only by the shape — never claimed as observed");
  assert.equal(knows.count, null);
  assert.equal(knows.objectClass, `${EX}Person`);
  assert.equal(knows.objectKind, "iri");

  const { relationships, attributes } = partitionSuggestions(offer);
  assert.deepEqual(relationships.map((s) => s.predicate), [FOAF_KNOWS]);
  assert.deepEqual(attributes.map((s) => s.predicate), [FOAF_NAME, `${EX}nickname`]);
});

test("a shape targeting a class with no instances still offers that class", () => {
  const index = buildSuggestionIndex({
    shapes: results(
      ["targetClass", "path"],
      [
        {
          targetClass: { type: "uri", value: `${EX}Project` },
          path: { type: "uri", value: `${EX}title` },
        },
      ],
    ),
  });

  assert.deepEqual(index.classes, [{ iri: `${EX}Project`, instances: null }]);
});

test("with no introspection results the index is empty — no vocabulary is invented", () => {
  const index = buildSuggestionIndex({});
  assert.deepEqual(index.classes, []);
  assert.deepEqual(index.unscoped, []);
  assert.equal(index.shapesPresent, false);
  assert.deepEqual(suggestionsFor(index, `${EX}Person`), []);
});

test("an untyped node falls back to the unscoped predicate offer", () => {
  const index = buildSuggestionIndex({
    unscoped: results(
      ["p", "uses", "sample"],
      [
        {
          p: { type: "uri", value: FOAF_KNOWS },
          uses: { type: "literal", value: "9" },
          sample: { type: "uri", value: `${EX}bob` },
        },
      ],
    ),
  });

  const offer = suggestionsFor(index, null);
  assert.deepEqual(offer.map((s) => s.predicate), [FOAF_KNOWS]);
  assert.equal(offer[0].source, "data");
  assert.equal(offer[0].objectKind, "iri");
});
