// [OPUS-5] sq-ixc3.21 — acceptance coverage for the dataset-overview derivation: parsing the
// aggregate bindings, the subClassOf forest (including the cyclic case), the nested bubble pack
// geometry, the chord angles, the domain–range merge and the drill-down query builder.

import assert from "node:assert/strict";
import test from "node:test";
import type { SparqlResults, SparqlTerm } from "@sparq/client";
import {
  BUBBLE_MAX_RADIUS,
  BUBBLE_MIN_RADIUS,
  buildChordModel,
  buildClassHierarchy,
  buildDomainRangeRows,
  bubbleRadius,
  instanceListQuery,
  packHierarchy,
  parseClassRows,
  parseInstanceRows,
  parseLiteralRangeRows,
  parseRelationRows,
  parseSubClassEdges,
  predicatesBetween,
  subtreeInstances,
  type RelationRow,
} from "./dataset-overview.js";

const EX = "http://example.org/";
const FOAF = "http://xmlns.com/foaf/0.1/";
const XSD = "http://www.w3.org/2001/XMLSchema#";

const uri = (value: string): SparqlTerm => ({ type: "uri", value });
const lit = (value: string): SparqlTerm => ({ type: "literal", value });
const results = (
  vars: string[],
  bindings: Array<Record<string, SparqlTerm>>,
): SparqlResults => ({ head: { vars }, results: { bindings } });

test("class rows are parsed, abbreviated and ordered by instance count", () => {
  const rows = parseClassRows(
    results(
      ["class", "instances"],
      [
        { class: uri(`${EX}Doc`), instances: lit("2") },
        { class: uri(`${FOAF}Person`), instances: lit("4") },
        // A bnode class expression has no name to render — skipped, never guessed at.
        { class: { type: "bnode", value: "b0" }, instances: lit("9") },
        { class: uri(`${EX}Broken`) },
      ],
    ),
  );
  assert.deepEqual(
    rows.map((r) => [r.label, r.instances]),
    [
      ["foaf:Person", 4],
      ["ex:Doc", 2],
    ],
  );
});

test("subClassOf edges drop self-edges, non-IRI ends and duplicates", () => {
  const edges = parseSubClassEdges(
    results(
      ["sub", "super"],
      [
        { sub: uri(`${EX}Student`), super: uri(`${EX}Person`) },
        { sub: uri(`${EX}Student`), super: uri(`${EX}Person`) },
        { sub: uri(`${EX}Person`), super: uri(`${EX}Person`) },
        { sub: { type: "bnode", value: "b1" }, super: uri(`${EX}Person`) },
      ],
    ),
  );
  assert.deepEqual(edges, [{ sub: `${EX}Student`, super: `${EX}Person` }]);
});

test("relation and literal-range rows keep only fully-bound, non-zero counts", () => {
  const relations = parseRelationRows(
    results(
      ["source", "predicate", "target", "statements"],
      [
        {
          source: uri(`${EX}P`),
          predicate: uri(`${FOAF}knows`),
          target: uri(`${EX}P`),
          statements: lit("4"),
        },
        { source: uri(`${EX}P`), predicate: uri(`${FOAF}knows`), statements: lit("4") },
        {
          source: uri(`${EX}P`),
          predicate: uri(`${EX}zero`),
          target: uri(`${EX}D`),
          statements: lit("0"),
        },
      ],
    ),
  );
  assert.deepEqual(relations, [
    { source: `${EX}P`, predicate: `${FOAF}knows`, target: `${EX}P`, count: 4 },
  ]);

  const literals = parseLiteralRangeRows(
    results(
      ["source", "predicate", "datatype", "statements"],
      [
        {
          source: uri(`${EX}P`),
          predicate: uri(`${FOAF}name`),
          datatype: uri(`${XSD}string`),
          statements: lit("9"),
        },
        // DATATYPE(?o) left unbound: recorded as null, never invented.
        { source: uri(`${EX}P`), predicate: uri(`${EX}note`), statements: lit("2") },
      ],
    ),
  );
  assert.deepEqual(literals, [
    { source: `${EX}P`, predicate: `${FOAF}name`, datatype: `${XSD}string`, count: 9 },
    { source: `${EX}P`, predicate: `${EX}note`, datatype: null, count: 2 },
  ]);
});

test("the subClassOf forest nests classes and includes instance-free superclasses", () => {
  const roots = buildClassHierarchy(
    [
      { iri: `${EX}Person`, label: "Person", instances: 4 },
      { iri: `${EX}Student`, label: "Student", instances: 1 },
    ],
    [
      { sub: `${EX}Student`, super: `${EX}Person` },
      { sub: `${EX}Person`, super: `${EX}Agent` },
    ],
  );
  assert.equal(roots.length, 1);
  assert.equal(roots[0].iri, `${EX}Agent`);
  // Named only as a superclass: zero DIRECT instances is the honest figure.
  assert.equal(roots[0].instances, 0);
  assert.equal(roots[0].children[0].iri, `${EX}Person`);
  assert.equal(roots[0].children[0].children[0].iri, `${EX}Student`);
  assert.equal(subtreeInstances(roots[0]), 5);
});

test("a cyclic subClassOf set still terminates, with the closing edge reported", () => {
  const roots = buildClassHierarchy(
    [
      { iri: "urn:A", label: "A", instances: 1 },
      { iri: "urn:B", label: "B", instances: 1 },
    ],
    [
      { sub: "urn:A", super: "urn:B" },
      { sub: "urn:B", super: "urn:A" },
    ],
  );
  assert.equal(roots.length, 1);
  assert.equal(roots[0].iri, "urn:B");
  assert.equal(roots[0].children[0].iri, "urn:A");
  assert.deepEqual(roots[0].otherParents, ["urn:A"]);
});

test("a class with several superclasses nests under one and records the rest", () => {
  const roots = buildClassHierarchy([{ iri: "urn:C", label: "C", instances: 1 }], [
    { sub: "urn:C", super: "urn:B" },
    { sub: "urn:C", super: "urn:A" },
  ]);
  const c = roots.flatMap((r) => r.children).find((n) => n.iri === "urn:C");
  assert.ok(c, "C is nested under one of its superclasses");
  assert.deepEqual(c.otherParents, ["urn:B"]);
});

test("bubble radius grows with instance count, between the declared bounds", () => {
  assert.equal(bubbleRadius(10, 10), BUBBLE_MAX_RADIUS);
  assert.equal(bubbleRadius(0, 10), BUBBLE_MIN_RADIUS);
  assert.ok(bubbleRadius(9, 10) > bubbleRadius(1, 10));
  assert.ok(bubbleRadius(1, 10) >= BUBBLE_MIN_RADIUS);
});

test("packing nests child bubbles inside their parent and never overlaps siblings", () => {
  // The parent has FEWER direct instances than its children, so its bubble must grow to
  // enclose them — the case where a parent radius taken from its own count alone would fail.
  const pack = packHierarchy(
    buildClassHierarchy(
      [
        { iri: "urn:parent", label: "parent", instances: 1 },
        { iri: "urn:childA", label: "childA", instances: 10 },
        { iri: "urn:childB", label: "childB", instances: 7 },
        { iri: "urn:other", label: "other", instances: 4 },
        { iri: "urn:third", label: "third", instances: 2 },
      ],
      [
        { sub: "urn:childA", super: "urn:parent" },
        { sub: "urn:childB", super: "urn:parent" },
      ],
    ),
  );
  const at = (iri: string) => {
    const b = pack.bubbles.find((x) => x.iri === iri);
    assert.ok(b, `${iri} was packed`);
    return b;
  };
  const parent = at("urn:parent");
  const childA = at("urn:childA");
  const childB = at("urn:childB");
  assert.equal(childA.depth, 1);
  assert.equal(parent.totalInstances, 18);
  for (const child of [childA, childB]) {
    assert.ok(
      Math.hypot(child.x - parent.x, child.y - parent.y) + child.r <= parent.r + 1e-6,
      `${child.iri} is fully inside its parent`,
    );
  }
  assert.ok(
    Math.hypot(childA.x - childB.x, childA.y - childB.y) >= childA.r + childB.r - 1e-6,
    "sibling children do not overlap",
  );

  const roots = pack.bubbles.filter((b) => b.depth === 0);
  for (let i = 0; i < roots.length; i += 1) {
    for (let j = i + 1; j < roots.length; j += 1) {
      const a = roots[i];
      const b = roots[j];
      assert.ok(
        Math.hypot(a.x - b.x, a.y - b.y) >= a.r + b.r - 1e-6,
        `${a.iri} and ${b.iri} do not overlap`,
      );
    }
  }
  for (const b of pack.bubbles) {
    assert.ok(b.x - b.r >= -1e-6 && b.x + b.r <= pack.width + 1e-6, "inside the viewBox width");
    assert.ok(b.y - b.r >= -1e-6 && b.y + b.r <= pack.height + 1e-6, "inside the viewBox height");
  }
});

test("an empty hierarchy packs to an empty box", () => {
  assert.deepEqual(packHierarchy([]), { bubbles: [], width: 0, height: 0 });
});

const RELATIONS: RelationRow[] = [
  { source: `${EX}P`, predicate: `${FOAF}knows`, target: `${EX}P`, count: 4 },
  { source: `${EX}P`, predicate: `${EX}wrote`, target: `${EX}D`, count: 3 },
  { source: `${EX}P`, predicate: `${EX}read`, target: `${EX}D`, count: 1 },
];

test("chord arcs tile the circle and ribbon endpoints stay inside their arcs", () => {
  const chord = buildChordModel(RELATIONS);
  assert.equal(chord.arcs.length, 2);
  assert.equal(chord.ribbons.length, 2, "predicates between the same pair share one ribbon");
  assert.equal(chord.shownStatements, 8);
  assert.equal(chord.hiddenClasses, 0);
  assert.equal(chord.hiddenStatements, 0);

  const gaps = chord.arcs.length * 0.035;
  const covered = chord.arcs.reduce((sum, a) => sum + (a.endAngle - a.startAngle), 0);
  assert.ok(Math.abs(covered + gaps - 2 * Math.PI) < 1e-9, "arcs plus gaps cover the circle once");

  const arcOf = new Map(chord.arcs.map((a) => [a.iri, a] as const));
  // A self-relationship occupies BOTH an outgoing and an incoming endpoint on its own arc.
  assert.equal(arcOf.get(`${EX}P`)?.value, 12);
  assert.equal(arcOf.get(`${EX}D`)?.value, 4);

  for (const r of chord.ribbons) {
    const source = arcOf.get(r.source);
    const target = arcOf.get(r.target);
    assert.ok(source);
    assert.ok(target);
    assert.ok(r.sourceEndAngle > r.sourceStartAngle);
    assert.ok(r.targetEndAngle > r.targetStartAngle);
    assert.ok(r.sourceStartAngle >= source.startAngle - 1e-9);
    assert.ok(r.sourceEndAngle <= source.endAngle + 1e-9);
    assert.ok(r.targetStartAngle >= target.startAngle - 1e-9);
    assert.ok(r.targetEndAngle <= target.endAngle + 1e-9);
  }

  // Endpoints sharing an arc are laid side by side: no overlap, and together they fill the arc.
  for (const arc of chord.arcs) {
    const spans = chord.ribbons
      .flatMap((r) => [
        ...(r.source === arc.iri ? [[r.sourceStartAngle, r.sourceEndAngle]] : []),
        ...(r.target === arc.iri ? [[r.targetStartAngle, r.targetEndAngle]] : []),
      ])
      .sort((a, b) => a[0] - b[0]);
    for (let i = 1; i < spans.length; i += 1) {
      assert.ok(spans[i][0] >= spans[i - 1][1] - 1e-9, `endpoints on ${arc.iri} do not overlap`);
    }
    const filled = spans.reduce((sum, [start, end]) => sum + (end - start), 0);
    assert.ok(
      Math.abs(filled - (arc.endAngle - arc.startAngle)) < 1e-9,
      `endpoints fill the ${arc.iri} arc`,
    );
  }
});

test("the chord class cap reports what it left out instead of hiding it", () => {
  const capped = buildChordModel(RELATIONS, 1);
  assert.equal(capped.arcs.length, 1);
  assert.equal(capped.shownStatements, 4, "only the self-relationship survives the cap");
  assert.equal(capped.hiddenStatements, 4);
  assert.equal(capped.hiddenClasses, 1);
});

test("an empty store yields an empty chord, not a fabricated one", () => {
  assert.deepEqual(buildChordModel([]), {
    arcs: [],
    ribbons: [],
    shownStatements: 0,
    hiddenClasses: 0,
    hiddenStatements: 0,
  });
});

test("a ribbon drills down to its per-predicate breakdown, most frequent first", () => {
  assert.deepEqual(
    predicatesBetween(RELATIONS, `${EX}P`, `${EX}D`).map((r) => [r.predicate, r.count]),
    [
      [`${EX}wrote`, 3],
      [`${EX}read`, 1],
    ],
  );
  assert.deepEqual(predicatesBetween(RELATIONS, `${EX}D`, `${EX}P`), []);
});

test("domain–range merges class and datatype ranges, most frequent first", () => {
  const rows = buildDomainRangeRows(RELATIONS, [
    { source: `${EX}P`, predicate: `${FOAF}name`, datatype: `${XSD}string`, count: 9 },
    { source: `${EX}P`, predicate: `${EX}note`, datatype: null, count: 2 },
  ]);
  assert.equal(rows.length, 5);
  assert.deepEqual(
    [rows[0].predicateLabel, rows[0].rangeLabel, rows[0].rangeKind, rows[0].count],
    ["foaf:name", "xsd:string", "datatype", 9],
  );
  assert.equal(rows.filter((r) => r.rangeKind === "class").length, 3);
  const note = rows.find((r) => r.predicateLabel === "ex:note");
  assert.ok(note);
  assert.equal(note.rangeKind, "literal");
  assert.equal(note.range, null);
});

test("the instance drill-down query embeds only IRIs that are safe in an IRIREF", () => {
  assert.equal(
    instanceListQuery(`${EX}P`, 3),
    "SELECT ?instance WHERE { ?instance a <http://example.org/P> } LIMIT 3",
  );
  // A space or a `>` would break out of the IRIREF: decline rather than emit a broken query.
  assert.equal(instanceListQuery("http://example.org/a b", 3), null);
  assert.equal(instanceListQuery("http://example.org/a>", 3), null);
  assert.equal(instanceListQuery("", 3), null);
  assert.equal(instanceListQuery(`${EX}P`, 0), null);
});

test("instance rows keep IRIs and bnodes, skipping unbound rows", () => {
  assert.deepEqual(
    parseInstanceRows(results(["instance"], [{ instance: uri("urn:x") }, {}])),
    [{ kind: "uri", value: "urn:x" }],
  );
});
