// [OPUS-4.8] sq-egy6 — built-in examples for the live /surface/shacl playground.
// The default is the `ex:age "thirty"` datatype-violation walkthrough from
// skills/shacl-validation/SKILL.md: a string where an xsd:integer is required.

export interface ShaclExample {
  id: string;
  label: string;
  description: string;
  /** True when this example's data is expected to conform (no violations). */
  conforms: boolean;
  data: string;
  shapes: string;
  /**
   * [OPUS-4.8] sq-pyn7 (#796) — the syntax of `shapes`. Defaults to "turtle"; an example with
   * "compact" carries its shapes graph as SHACL Compact Syntax, which the playground parses to
   * Turtle via the wasm `Store.parseShaclCompact` binding before validating.
   */
  shapesMode?: "turtle" | "compact";
}

const PERSON_SHAPE = `@prefix sh:  <http://www.w3.org/ns/shacl#> .
@prefix ex:  <http://example.org/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

ex:PersonShape a sh:NodeShape ;
  sh:targetClass ex:Person ;
  sh:property [
    sh:path ex:age ;
    sh:datatype xsd:integer ;
    sh:minCount 1 ;
    sh:message "ex:age must be exactly one xsd:integer" ;
  ] ;
  sh:property [
    sh:path ex:name ;
    sh:datatype xsd:string ;
    sh:minCount 1 ;
  ] .
`;

export const SHACL_EXAMPLES: ShaclExample[] = [
  {
    id: "datatype-violation",
    label: 'Datatype violation (ex:age "thirty")',
    description:
      'ex:alice has ex:age "thirty" — a string where the shape requires an xsd:integer. The report is non-conformant with one DatatypeConstraintComponent result.',
    conforms: false,
    data: `@prefix ex: <http://example.org/> .

# age is a string literal, not an integer — this violates sh:datatype xsd:integer.
ex:alice a ex:Person ;
  ex:name "Alice" ;
  ex:age "thirty" .
`,
    shapes: PERSON_SHAPE,
  },
  {
    id: "conforms",
    label: "Conforming data",
    description:
      "The same shape, but ex:age is a well-typed integer literal — the report conforms (sh:conforms = true, no results).",
    conforms: true,
    data: `@prefix ex: <http://example.org/> .

ex:alice a ex:Person ;
  ex:name "Alice" ;
  ex:age 30 .
`,
    shapes: PERSON_SHAPE,
  },
  {
    id: "min-count",
    label: "Missing required property",
    description:
      "ex:bob has an integer age but no ex:name — the sh:minCount 1 on ex:name fails, so the report carries a MinCountConstraintComponent result.",
    conforms: false,
    data: `@prefix ex: <http://example.org/> .

ex:bob a ex:Person ;
  ex:age 41 .
`,
    shapes: PERSON_SHAPE,
  },
  {
    // [OPUS-4.8] sq-pyn7 (#796) — the SCS *input* example: the same Person shape authored in
    // the terser SHACL Compact Syntax. The playground parses it to a Turtle shapes graph via
    // the wasm `Store.parseShaclCompact` binding before validating, so `ex:age "thirty"` (a
    // string) still fails the xsd:integer datatype constraint. `shapeClass` makes the shape
    // its own target class — the data's `ex:carol a ex:Person` is targeted.
    id: "compact-input",
    label: "Compact-syntax shapes (SCS input)",
    description:
      "The Person shape written in SHACL Compact Syntax, parsed to a shapes graph via Store.parseShaclCompact before validating. ex:carol's string age violates xsd:integer — the same report as the equivalent Turtle shapes.",
    conforms: false,
    shapesMode: "compact",
    data: `@prefix ex: <http://example.org/> .

# age is a string literal, not an integer — fails sh:datatype xsd:integer.
ex:carol a ex:Person ;
  ex:name "Carol" ;
  ex:age "thirty" .
`,
    shapes: `PREFIX ex: <http://example.org/>
PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>

shapeClass ex:Person {
\tex:name xsd:string [1..1] .
\tex:age xsd:integer [1..1] .
}
`,
  },
];
