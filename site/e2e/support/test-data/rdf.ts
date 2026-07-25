// [OPUS-4.8] sq-ymr2e.1 — small, stable RDF + SPARQL test-data the sibling journey suites build
// on. Kept deterministic and tiny: fixed IRIs, fixed literals, DESC-orderable numerics so a
// journey can assert CONCRETE result values (the "an answer a static site could not credibly
// fake" evidence, §2.1) without depending on the shipped sample graph, which may evolve. These
// are fixtures, not the product's bundled sample.
// [OPUS-4.8] sq-4hiqe — the /try REPL journeys that consumed these were removed; the module stays
// as a shared support fixture for the home + future runner journeys.

/** A minimal two-person graph: names + integer ages, ORDER-BY-able. */
export const SAMPLE_TURTLE = `@prefix ex: <http://example.org/> .

ex:alice a ex:Person ; ex:name "Alice" ; ex:age 30 .
ex:bob   a ex:Person ; ex:name "Bob"   ; ex:age 24 .
ex:carol a ex:Person ; ex:name "Carol" ; ex:age 41 .
`;

/** The trivial all-triples probe — the rawest editor→run→table path. */
export const SELECT_ALL = "SELECT * WHERE { ?s ?p ?o }";

/** Names ordered ascending — a deterministic, value-assertable projection over SAMPLE_TURTLE. */
export const SELECT_NAMES_ASC = `PREFIX ex: <http://example.org/>
SELECT ?name WHERE { ?person ex:name ?name } ORDER BY ASC(?name)`;

/** Ages ordered descending — DESC so a fake static echo of the query would not match. */
export const SELECT_AGES_DESC = `PREFIX ex: <http://example.org/>
SELECT ?name ?age WHERE { ?person ex:name ?name ; ex:age ?age } ORDER BY DESC(?age)`;

/** The name values SELECT_NAMES_ASC must return, in order — for a journey value assertion. */
export const EXPECTED_NAMES_ASC = ["Alice", "Bob", "Carol"] as const;

/** A syntactically invalid query for the error-state journeys (unterminated WHERE). */
export const INVALID_QUERY = "SELECT * WHERE { ?s ?p ?o";
