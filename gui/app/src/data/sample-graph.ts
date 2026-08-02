// [OPUS-4.8] sq-ixc3.9 — a small, self-contained sample graph the workbench seeds the live
// store with on first warm. Real Turtle; loaded into the wasm Store and queried for real (no
// mocks). The foaf:name "Alice" binding is the deterministic fixture the tauri-driver e2e
// (gui/e2e/run-e2e.mjs) asserts on, so keep it present.
//
// [OPUS-5] sq-ixc3.22 — it also carries ONE RDF 1.2 reifier (`ex:knowsClaim`), so the graph
// view's annotation rendering has something real to draw out of the box. It is written in the
// explicit form — a reifier with an `rdf:reifies` triple term — rather than the `<< … >>`
// reified-triple sugar, because that is the RDF 1.2 core shape and reads unambiguously here.
// The graph view folds it onto the `ex:alice foaf:knows ex:bob` edge as a badge instead of
// drawing the reifier as plumbing nodes.
export const SAMPLE_TURTLE = `@prefix ex:   <http://example.org/> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
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

ex:knowsClaim rdf:reifies <<( ex:alice foaf:knows ex:bob )>> ;
  ex:since "2019"^^xsd:gYear ;
  ex:statedBy ex:carol .
`;

/** The engine format string the workbench loads the sample with (Turtle). */
export const SAMPLE_FORMAT = "turtle";

/** The default query the workbench opens with — a deterministic SELECT over the sample. */
export const DEFAULT_QUERY = `PREFIX foaf: <http://xmlns.com/foaf/0.1/>
SELECT ?name ?age WHERE {
  ?s foaf:name ?name ;
     foaf:age ?age .
}
ORDER BY ?name`;
