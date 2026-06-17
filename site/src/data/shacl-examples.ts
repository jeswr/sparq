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
];
