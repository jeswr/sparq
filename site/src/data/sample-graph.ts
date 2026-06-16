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

// [OPUS-4.8] sq-repl-datasets — a small set of self-contained built-in datasets the
// REPL's dataset picker can switch between. Each is real RDF parsed by the wasm Store
// (no mocks). `format` is the engine format string ("turtle" | "ntriples" | "nquads" |
// "trig"); these built-ins are all Turtle.

export interface BuiltinDataset {
  /** Stable id used for the picker's selection state. */
  id: string;
  /** Human label shown in the picker. */
  label: string;
  /** One-line description of what's in the graph. */
  description: string;
  /** The RDF source text. */
  text: string;
  /** Engine format string for Store.load(). */
  format: string;
}

// A second sample: a tiny library catalogue (books, authors, subjects) — different
// vocabulary + shape from the social graph, so example queries exercise other joins.
export const LIBRARY_TURTLE = `@prefix ex:   <http://example.org/lib/> .
@prefix dc:   <http://purl.org/dc/terms/> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .

ex:book1 a ex:Book ;
  dc:title "The Pragmatic Programmer" ;
  ex:author ex:hunt, ex:thomas ;
  dc:subject "Software" ;
  ex:year 1999 .

ex:book2 a ex:Book ;
  dc:title "Designing Data-Intensive Applications" ;
  ex:author ex:kleppmann ;
  dc:subject "Databases" ;
  ex:year 2017 .

ex:book3 a ex:Book ;
  dc:title "Types and Programming Languages" ;
  ex:author ex:pierce ;
  dc:subject "Software" ;
  ex:year 2002 .

ex:hunt      ex:name "Andrew Hunt" .
ex:thomas    ex:name "David Thomas" .
ex:kleppmann ex:name "Martin Kleppmann" .
ex:pierce    ex:name "Benjamin C. Pierce" .
`;

// A third sample: a tiny geography graph (countries, capitals, population) — numeric
// FILTER / ORDER BY / aggregate friendly.
export const GEO_TURTLE = `@prefix ex:   <http://example.org/geo/> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .

ex:france   a ex:Country ; ex:name "France" ;
  ex:capital "Paris" ; ex:population 67000000 ; ex:continent "Europe" .
ex:japan    a ex:Country ; ex:name "Japan" ;
  ex:capital "Tokyo" ; ex:population 125000000 ; ex:continent "Asia" .
ex:brazil   a ex:Country ; ex:name "Brazil" ;
  ex:capital "Brasília" ; ex:population 214000000 ; ex:continent "Americas" .
ex:kenya    a ex:Country ; ex:name "Kenya" ;
  ex:capital "Nairobi" ; ex:population 54000000 ; ex:continent "Africa" .
ex:germany  a ex:Country ; ex:name "Germany" ;
  ex:capital "Berlin" ; ex:population 83000000 ; ex:continent "Europe" .
`;

export const BUILTIN_DATASETS: BuiltinDataset[] = [
  {
    id: "social",
    label: "Social graph (default)",
    description: "People, ages, cities and foaf:knows links — the REPL's default.",
    text: SAMPLE_TURTLE,
    format: "turtle",
  },
  {
    id: "library",
    label: "Library catalogue",
    description: "Books, authors and subjects (Dublin Core).",
    text: LIBRARY_TURTLE,
    format: "turtle",
  },
  {
    id: "geo",
    label: "World geography",
    description: "Countries, capitals, populations and continents.",
    text: GEO_TURTLE,
    format: "turtle",
  },
];

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
