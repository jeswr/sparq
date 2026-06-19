// [OPUS-4.8] sq-3was — pure, framework-free data + helpers for the /surface/genai
// (GenAI / NLQ) walkthrough. sparq-nlq is an OPT-IN native crate (it pulls a model
// behind a trait), it is NOT in the lean wasm bundle, and the static GitHub-Pages
// site has no backend, so this surface is the honest "captured-output walkthrough"
// fallback the feature-showcase design names for tier-e
// (research/feature-showcase-site-design.md §0, surface (e)).
//
// =====================================================================================
// HONESTY CONTRACT — read before editing. A sibling page (sq-rnwc) was caught
// fabricating "captured" output (dropped datatypes, invented triples). To make that
// impossible here, EVERYTHING on this page is split into two clearly-labelled halves,
// and the test (site/test/genai.test.mjs) pins the serialization of both:
//
//   (A) GENUINELY LIVE-CAPTURED — `SCHEMA_SUMMARY`, every `Question.sparql`,
//       `Question.vars`, `Question.headRows`, `Question.totalRows`, `Question.repairs`
//       and `Question.turnOutcomes`. These were produced by running the REAL
//       sparq-nlq loop (ground → generate → validate → execute → repair) with the
//       COMMITTED `ReplayLlm` fixture (crates/sparq-nlq/tests/fixtures/
//       olympics_replay.json) against the REAL 1,781,625-triple Olympics dataset
//       (github.com/wallscope/olympics-rdf), and PASTED VERBATIM. The schema summary
//       is the exact `Introspection::to_text_summary(4000)` grounding deck (pure
//       index introspection, no model). The result rows are the exact
//       `oxrdf::Term::Display` (N-Triples) serialization the engine emits — datatypes
//       and language tags INTACT (`xsd:integer` for COUNT, `xsd:int` for the source
//       `dbp:year`, `xsd:decimal` for AVG, `@en` langStrings for labels). This is
//       cross-checked offline+deterministically by the CI test
//       crates/sparq-nlq/tests/olympics_eval.rs (row counts, the one repair round).
//
//   (B) HONESTLY ILLUSTRATIVE — the *natural-language → SPARQL generation step* is
//       NOT a live model here. The committed fixture's completions are SCRIPTED:
//       realistic, schema-grounded queries a competent model would emit for this
//       prompt (one is deliberately malformed first to exercise the repair path),
//       NOT a real LLM call. So we DO NOT claim the *query text* is "what GPT/Claude
//       said"; we claim the loop's PLUMBING (grounding, extraction, spargebra
//       validation, budgeted execution, repair) and the EXECUTED RESULTS are real.
//       Live-model exec-accuracy NUMBERS are explicitly NOT measured here (open work,
//       sparq-nlq bead sq-g0lw; needs a key + a canonical host). `IS_LIVE_LLM = false`.
//
// NLQ exec-accuracy is NOT a correctness guarantee: a query can parse, execute and
// return rows while answering the wrong question. The page says so.
//
// Grounded in crates/sparq-nlq (README + src/lib.rs + the eval harness) and
// crates/sparq-introspect (to_text_summary).

/** Whether the NL→SPARQL generation step on this page was a LIVE model call. */
export const IS_LIVE_LLM = false as const;

/** The dataset the captured loop ran against. */
export const DATASET = {
  name: "Olympics (120 years)",
  source: "https://github.com/wallscope/olympics-rdf",
  triples: 1781625,
} as const;

/**
 * The verbatim grounding deck handed to the model: the exact output of
 * `sparq_introspect::Introspection::to_text_summary(4000)` over the dataset — schema
 * card / VoID-style introspection (class & predicate counts, characteristic sets,
 * prefix glossary) computed from the store's permutation indexes, NOT guessed. The
 * trailing `…` markers are the introspect crate's own budget-truncation at the 4000
 * char limit; this string IS the prompt the loop built.
 */
export const SCHEMA_SUMMARY =
  "# Schema summary — 1781625 triples, 406700 subjects, 8 classes, 16 predicates, 15 entity patterns\n## Prefixes\nfoaf: http://xmlns.com/foaf/0.1/ — FOAF (people & agents)\ndbo: http://dbpedia.org/ontology/ — DBpedia ontology\nns1: http://wallscope.co.uk/resource/olympics/team/ (1184 terms)\nrdf: http://www.w3.org/1999/02/22-rdf-syntax-ns# — RDF\nrdfs: http://www.w3.org/2000/01/rdf-schema# — RDF Schema\nskos: http://www.w3.org/2004/02/skos/core# — SKOS\nns2: http://wallscope.co.uk/resource/olympics/gender/\nxsd: http://www.w3.org/2001/XMLSchema# — XML Schema datatypes\nns3: http://wallscope.co.uk/resource/olympics/sport/\nns4: http://wallscope.co.uk/resource/olympics/city/\ndbp: http://dbpedia.org/property/ — DBpedia properties\nns5: http://wallscope.co.uk/resource/olympics/season/\nns6: http://wallscope.co.uk/ontology/olympics/\nns7: http://wallscope.co.uk/resource/olympics/athlete/ (134730 terms)\nns8: http://wallscope.co.uk/resource/olympics/event/ (765 terms)\nns9: http://wallscope.co.uk/resource/olympics/games/1896/\nns10: http://wallscope.co.uk/resource/olympics/medal/\n## Classes (by instance count)\n### foaf:Person — 134730 instances\n- dbo:team — 134730/134730 subjects (100%) → dbo:SportsTeam, e.g. ns1:30Februar\n- rdfs:label — 134730/134730 subjects (100%) → rdf:langString, e.g. \"  Gabrielle Marie \"Gabby\" Adcock (White-)\"@en\n- foaf:gender — 134730/134730 subjects (100%) → skos:Concept, e.g. ns2:F\n- foaf:age — 128429/134730 subjects (95%, avg 1.4/subj) → xsd:int, e.g. \"10\"\n- dbo:height — 101099/134730 subjects (75%) → xsd:int, e.g. \"127\"\n- dbo:weight — 100127/134730 subjects (74%) → xsd:double, e.g. \"100\"\n### dbo:SportsTeam — 1184 instances\n- rdfs:label — 1184/1184 subjects (100%) → rdf:langString, e.g. \"  Gabrielle Marie \"Gabby\" Adcock (White-)\"@en\n### dbo:SportsEvent — 765 instances\n- rdfs:label — 765/765 subjects (100%) → rdf:langString, e.g. \"  Gabrielle Marie \"Gabby\" Adcock (White-)\"@en\n- rdfs:subClassOf — 765/765 subjects (100%) → dbo:Sport, e.g. ns3:Aeronautics\n### dbo:Sport — 66 instances\n- rdfs:label — 66/66 subjects (100%) → rdf:langString, e.g. \"  Gabrielle Marie \"Gabby\" Adcock (White-)\"@en\n### dbo:Olympics — 51 instances\n- dbp:location — 51/51 subjects (100%) → dbo:City, e.g. ns4:Albertville\n- dbp:year — 51/51 subjects (100%) → xsd:int, e.g. \"1896\"\n- ns6:season — 51/51 subjects (100%) → dbo:TimePeriod, e.g. ns5:Summer\n### dbo:City — 42 instances\n- rdfs:label — 42/42 subjects (100%) → rdf:langString, e.g. \"  Gabrielle Marie \"Gabby\" Adcock (White-)\"@en\n### dbo:TimePeriod — 2 instances\n- rdfs:label — 2/2 subjects (100%) → rdf:langString, e.g. \"  Gabrielle Marie \"Gabby\" Adcock (White-)\"@en\n### skos:Concept — 2 instances\n- rdfs:label — 2/2 subjects (100%) → rdf:langString, e.g. \"  Gabrielle Marie \"Gabby\" Adcock (White-)\"@en\n## Entity patterns (characteristic predicate sets)\n- {ns6:athlete, ns6:event, ns6:games} × 229879 subjects\n- {dbo:height, dbo:team, dbo:weight, rdf:type, rdfs:label, foaf:age, foaf:gender} × 98547 subjects — mostly foaf:Person\n- {ns6:athlete, ns6:event, ns6:games, ns6:medal} × 39746 subjects\n- {dbo:team, rdf:type, rdfs:label, foaf:age, foaf:gender} × 27023 subjects — mostly foaf:Person\n- {dbo:team, rdf:type, rdfs:label, foaf:gender} × 5522 subjects — mostly foaf:Person\n- {dbo:height, dbo:team, rdf:type, rdfs:label, foaf:age, foaf:gender} × 1924 subjects — mostly foaf:Person\n- {rdf:type, rdfs:label} × 1296 subjects — mostly dbo:SportsTeam\n- {dbo:team, dbo:weight, rdf:type, rdfs:label, foaf:age, foaf:gender} × 935 subjects — mostly foaf:Person\n- {rdf:type, rdfs:label, rdfs:subClassOf} × 765 subjects — mostly dbo:SportsEvent\n- {dbo:height, dbo:team, dbo:weight, rdf:type, rdfs:label, foaf:gender} × 494 subjects — mostly foaf:Person\n- {dbo:ground, rdfs:label} × 230 subjects\n- {dbo:team, dbo:weight, rdf:type, rdfs:label, foaf:gender} × 151 subjects — mostly foaf:Person\n… and 3 rarer patterns\n## Predicates (global)\n- ns6:athlete — 269625 triples, 269625 subj, 134730 obj (IRIs, e.g. ns7:AAananthaS…)\n…\n";

/** One question's captured run through the real NL→SPARQL loop. */
export interface Question {
  /** The natural-language question. */
  question: string;
  /** The final SPARQL the loop executed (post-repair if any) — from the fixture. */
  sparql: string;
  /** Result variable names, in order ([] for an ASK). */
  vars: string[];
  /**
   * First few result rows, VERBATIM. Each cell is the engine's `oxrdf::Term::Display`
   * (N-Triples) string, or `null` for an unbound variable. An ASK's single unit row
   * is `[]` (zero variables, one empty row iff true).
   */
  headRows: (string | null)[][];
  /** Total rows the query returned over the full dataset. */
  totalRows: number;
  /** Repair rounds the loop needed (0 = the first completion parsed + executed). */
  repairs: number;
  /** Per-turn outcome tags, e.g. ["ParseError","Ok"] for the repair case. */
  turnOutcomes: string[];
  /** ASK query (unit-row encoding)? */
  isAsk: boolean;
}

// ── (A) GENUINELY LIVE-CAPTURED ─────────────────────────────────────────────────────
// Produced by the real sparq-nlq loop + ReplayLlm fixture over the real dataset, pasted
// verbatim. Re-capture (NOT hand-edit) if the prompt template, default NlqConfig, or
// dataset changes — see crates/sparq-nlq/examples/record_olympics.rs for the recorder.
export const QUESTIONS: Question[] = [
  {
    question: "How many athletes are on each team?",
    sparql: "PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\nPREFIX foaf: <http://xmlns.com/foaf/0.1/>\nPREFIX dbo: <http://dbpedia.org/ontology/>\nSELECT ?team (COUNT(?athlete) AS ?count)\nWHERE {\n  ?athlete rdf:type foaf:Person ;\n           dbo:team ?team .\n}\nGROUP BY ?team\nORDER BY DESC(?count)",
    vars: ["team", "count"],
    headRows: [["<http://wallscope.co.uk/resource/olympics/team/UnitedStates>", "\"9114\"^^<http://www.w3.org/2001/XMLSchema#integer>"], ["<http://wallscope.co.uk/resource/olympics/team/France>", "\"5777\"^^<http://www.w3.org/2001/XMLSchema#integer>"], ["<http://wallscope.co.uk/resource/olympics/team/GreatBritain>", "\"5758\"^^<http://www.w3.org/2001/XMLSchema#integer>"], ["<http://wallscope.co.uk/resource/olympics/team/Italy>", "\"4688\"^^<http://www.w3.org/2001/XMLSchema#integer>"], ["<http://wallscope.co.uk/resource/olympics/team/Germany>", "\"4569\"^^<http://www.w3.org/2001/XMLSchema#integer>"]],
    totalRows: 1184,
    repairs: 0,
    turnOutcomes: ["Ok"],
    isAsk: false,
  },
  {
    question: "How many athletes are in the dataset?",
    sparql: "PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\nPREFIX foaf: <http://xmlns.com/foaf/0.1/>\nSELECT (COUNT(?athlete) AS ?count)\nWHERE { ?athlete rdf:type foaf:Person }",
    vars: ["count"],
    headRows: [["\"134730\"^^<http://www.w3.org/2001/XMLSchema#integer>"]],
    totalRows: 1,
    repairs: 0,
    turnOutcomes: ["Ok"],
    isAsk: false,
  },
  {
    question: "List the year and host city of every Olympic games.",
    sparql: "PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\nPREFIX dbo: <http://dbpedia.org/ontology/>\nPREFIX dbp: <http://dbpedia.org/property/>\nSELECT ?games ?year ?city\nWHERE {\n  ?games rdf:type dbo:Olympics ;\n         dbp:year ?year ;\n         dbp:location ?city .\n}\nORDER BY ?year",
    vars: ["games", "year", "city"],
    headRows: [["<http://wallscope.co.uk/resource/olympics/games/1896/Summer>", "\"1896\"^^<http://www.w3.org/2001/XMLSchema#int>", "<http://wallscope.co.uk/resource/olympics/city/Athina>"], ["<http://wallscope.co.uk/resource/olympics/games/1900/Summer>", "\"1900\"^^<http://www.w3.org/2001/XMLSchema#int>", "<http://wallscope.co.uk/resource/olympics/city/Paris>"], ["<http://wallscope.co.uk/resource/olympics/games/1904/Summer>", "\"1904\"^^<http://www.w3.org/2001/XMLSchema#int>", "<http://wallscope.co.uk/resource/olympics/city/StLouis>"], ["<http://wallscope.co.uk/resource/olympics/games/1906/Summer>", "\"1906\"^^<http://www.w3.org/2001/XMLSchema#int>", "<http://wallscope.co.uk/resource/olympics/city/Athina>"], ["<http://wallscope.co.uk/resource/olympics/games/1908/Summer>", "\"1908\"^^<http://www.w3.org/2001/XMLSchema#int>", "<http://wallscope.co.uk/resource/olympics/city/London>"]],
    totalRows: 52,
    repairs: 0,
    turnOutcomes: ["Ok"],
    isAsk: false,
  },
  {
    question: "How many medals of each type were awarded?",
    sparql: "PREFIX ns6: <http://wallscope.co.uk/ontology/olympics/>\nPREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>\nSELECT ?medal (COUNT(?participation) AS ?count)\nWHERE {\n  ?participation ns6:medal ?m .\n  ?m rdfs:label ?medal .\n}\nGROUP BY ?medal\nORDER BY DESC(?count)",
    vars: ["medal", "count"],
    headRows: [["\"Gold\"@en", "\"13369\"^^<http://www.w3.org/2001/XMLSchema#integer>"], ["\"Bronze\"@en", "\"13293\"^^<http://www.w3.org/2001/XMLSchema#integer>"], ["\"Silver\"@en", "\"13106\"^^<http://www.w3.org/2001/XMLSchema#integer>"]],
    totalRows: 3,
    repairs: 0,
    turnOutcomes: ["Ok"],
    isAsk: false,
  },
  {
    question: "What is the average height of the athletes?",
    sparql: "PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\nPREFIX foaf: <http://xmlns.com/foaf/0.1/>\nPREFIX dbo: <http://dbpedia.org/ontology/>\nSELECT (AVG(?height) AS ?avgHeight)\nWHERE { ?athlete rdf:type foaf:Person ; dbo:height ?height }",
    vars: ["avgHeight"],
    headRows: [["\"176.316366892317380353\"^^<http://www.w3.org/2001/XMLSchema#decimal>"]],
    totalRows: 1,
    repairs: 0,
    turnOutcomes: ["Ok"],
    isAsk: false,
  },
  {
    question: "Which team has the most athletes?",
    sparql: "PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\nPREFIX foaf: <http://xmlns.com/foaf/0.1/>\nPREFIX dbo: <http://dbpedia.org/ontology/>\nSELECT ?team (COUNT(?athlete) AS ?count)\nWHERE { ?athlete rdf:type foaf:Person ; dbo:team ?team }\nGROUP BY ?team\nORDER BY DESC(?count)\nLIMIT 1",
    vars: ["team", "count"],
    headRows: [["<http://wallscope.co.uk/resource/olympics/team/UnitedStates>", "\"9114\"^^<http://www.w3.org/2001/XMLSchema#integer>"]],
    totalRows: 1,
    repairs: 0,
    turnOutcomes: ["Ok"],
    isAsk: false,
  },
  {
    question: "List all sports with their labels.",
    sparql: "PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\nPREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>\nPREFIX dbo: <http://dbpedia.org/ontology/>\nSELECT ?sport ?label\nWHERE { ?sport rdf:type dbo:Sport ; rdfs:label ?label }\nORDER BY ?label",
    vars: ["sport", "label"],
    headRows: [["<http://wallscope.co.uk/resource/olympics/sport/Aeronautics>", "\"Aeronautics\"@en"], ["<http://wallscope.co.uk/resource/olympics/sport/AlpineSkiing>", "\"Alpine Skiing\"@en"], ["<http://wallscope.co.uk/resource/olympics/sport/Alpinism>", "\"Alpinism\"@en"], ["<http://wallscope.co.uk/resource/olympics/sport/Archery>", "\"Archery\"@en"], ["<http://wallscope.co.uk/resource/olympics/sport/ArtCompetitions>", "\"Art Competitions\"@en"]],
    totalRows: 66,
    repairs: 0,
    turnOutcomes: ["Ok"],
    isAsk: false,
  },
  {
    // THE REPAIR CASE: the fixture's first completion drops the aggregate alias's
    // closing paren; spargebra rejects it, the loop feeds the parser error back, the
    // second completion parses and executes — turnOutcomes ["ParseError","Ok"].
    question: "How many events does each sport have?",
    sparql: "PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\nPREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>\nPREFIX dbo: <http://dbpedia.org/ontology/>\nSELECT ?sport (COUNT(?event) AS ?count)\nWHERE { ?event rdf:type dbo:SportsEvent ; rdfs:subClassOf ?sport }\nGROUP BY ?sport\nORDER BY DESC(?count)",
    vars: ["sport", "count"],
    headRows: [["<http://wallscope.co.uk/resource/olympics/sport/Athletics>", "\"83\"^^<http://www.w3.org/2001/XMLSchema#integer>"], ["<http://wallscope.co.uk/resource/olympics/sport/Shooting>", "\"83\"^^<http://www.w3.org/2001/XMLSchema#integer>"], ["<http://wallscope.co.uk/resource/olympics/sport/Swimming>", "\"55\"^^<http://www.w3.org/2001/XMLSchema#integer>"], ["<http://wallscope.co.uk/resource/olympics/sport/Cycling>", "\"44\"^^<http://www.w3.org/2001/XMLSchema#integer>"], ["<http://wallscope.co.uk/resource/olympics/sport/Sailing>", "\"38\"^^<http://www.w3.org/2001/XMLSchema#integer>"]],
    totalRows: 66,
    repairs: 1,
    turnOutcomes: ["ParseError", "Ok"],
    isAsk: false,
  },
  {
    question: "Are there any athletes taller than 200 centimetres?",
    sparql: "PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\nPREFIX foaf: <http://xmlns.com/foaf/0.1/>\nPREFIX dbo: <http://dbpedia.org/ontology/>\nASK {\n  ?athlete rdf:type foaf:Person ;\n           dbo:height ?height .\n  FILTER(?height > 200)\n}",
    vars: [],
    headRows: [[]],
    totalRows: 1,
    repairs: 0,
    turnOutcomes: ["Ok"],
    isAsk: true,
  },
];

/** Lookup by question text (the page's selection key). */
export function questionByText(text: string): Question | undefined {
  return QUESTIONS.find((q) => q.question === text);
}

/** The total repair rounds across the eval set — exactly one (the fixture exercises it). */
export function totalRepairs(): number {
  return QUESTIONS.reduce((n, q) => n + q.repairs, 0);
}
