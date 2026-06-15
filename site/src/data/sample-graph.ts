// [OPUS-4.8] sq-8thu — a small, self-contained sample graph for the live REPL.
// Real Turtle; loaded into the wasm Store and queried for real (no mocks).

export const SAMPLE_TURTLE = `@prefix ex:   <http://example.org/> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .

ex:alice a foaf:Person ;
  foaf:name "Alice" ;
  foaf:age 30 ;
  ex:city "London" ;
  foaf:knows ex:bob, ex:carol .

ex:bob a foaf:Person ;
  foaf:name "Bob"@en ;
  foaf:age 25 ;
  ex:city "Bristol" ;
  foaf:knows ex:alice .

ex:carol a foaf:Person ;
  foaf:name "Carol" ;
  foaf:age 41 ;
  ex:city "London" ;
  foaf:knows ex:alice, ex:dan .

ex:dan a foaf:Person ;
  foaf:name "Dan" ;
  foaf:age 19 ;
  ex:city "Cardiff" .
`;

export interface ExampleQuery {
  label: string;
  sparql: string;
}

export const EXAMPLE_QUERIES: ExampleQuery[] = [
  {
    label: "All people & ages",
    sparql: `PREFIX foaf: <http://xmlns.com/foaf/0.1/>
SELECT ?name ?age WHERE {
  ?s foaf:name ?name ;
     foaf:age  ?age .
} ORDER BY DESC(?age)`,
  },
  {
    label: "Adults (FILTER ≥ 25)",
    sparql: `PREFIX foaf: <http://xmlns.com/foaf/0.1/>
SELECT ?name ?age WHERE {
  ?s foaf:name ?name ; foaf:age ?age .
  FILTER(?age >= 25)
} ORDER BY ?age`,
  },
  {
    label: "Who knows whom (join)",
    sparql: `PREFIX foaf: <http://xmlns.com/foaf/0.1/>
SELECT ?a ?b WHERE {
  ?x foaf:name ?a ; foaf:knows ?y .
  ?y foaf:name ?b .
} ORDER BY ?a ?b`,
  },
  {
    label: "Count per city (GROUP BY)",
    sparql: `PREFIX ex: <http://example.org/>
SELECT ?city (COUNT(?s) AS ?people) WHERE {
  ?s ex:city ?city .
} GROUP BY ?city ORDER BY DESC(?people)`,
  },
  {
    label: "ASK: anyone over 40?",
    sparql: `PREFIX foaf: <http://xmlns.com/foaf/0.1/>
ASK { ?s foaf:age ?age . FILTER(?age > 40) }`,
  },
];
