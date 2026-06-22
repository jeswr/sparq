// [OPUS-4.8] sq-dwdm — pure, framework-free data + helpers for the /surface/vector
// (Vector / ANN) showcase. sparq-vectors is an OPT-IN native crate: nothing in the
// workspace (or the lean wasm bundle) depends on it, the default engine build does not
// even compile it, and the `vec:` magic-predicate integration sits behind the non-default
// `vec-predicate` feature. The static GitHub-Pages site has no backend, so this surface is
// the honest tier-e "captured-output walkthrough" fallback the feature-showcase design
// names (research/feature-showcase-site-design.md §0, surface (e)).
//
// =====================================================================================
// HONESTY CONTRACT — read before editing. A sibling page (sq-rnwc) was caught fabricating
// "captured" output (dropped datatypes, invented rows); the fix on the next page (sq-3was)
// pinned the serialization in a test so drift is impossible. We do the SAME here. Every
// payload on this page is split into two clearly-labelled halves, and the test
// (site/test/vector.test.mjs) pins the serialization of the live-captured half:
//
//   (A) GENUINELY LIVE-CAPTURED — `BOLT` (the embed_labels + nearest_term_exact run) and
//       every `VecQuery` (the `vec:nearest` / `vec:search` SPARQL results). These were
//       produced by RUNNING THE REAL sparq-vectors binary
//       (crates/sparq-vectors/examples/capture_surface_vector.rs, built with
//       `--features vec-predicate`) over a tiny DECLARED in-memory fixture, with the
//       answer-EXACT backend (`nearest_exact` brute-force — no index build, no model), and
//       PASTED VERBATIM. The result cells are the engine's exact `oxrdf::Term::Display`
//       (N-Triples) serialization: an IRI is `<…>`, a plain literal is `"…"`, a typed
//       literal is `"…"^^<datatype-iri>`. The `vec:search` cosine score is bound by the
//       engine as an `xsd:double` literal, datatype INTACT. The Bolt cosines are the
//       engine's exact f32 (the page rounds them for display but the captured value is
//       pinned). Re-CAPTURE (do not hand-edit) if the fixture or pipeline changes — run the
//       example again and paste. Everything in (A) is DETERMINISTIC: the HashEmbedder is a
//       fixed lexical hash, the exact backend is answer-exact, ties break on ascending id —
//       so the same binary over the same fixture yields byte-identical output (verified by
//       re-running the harness).
//
//   (B) HONESTLY ILLUSTRATIVE / NON-CANONICAL — the ANN recall and latency characteristics
//       (HNSW / DiskANN / PQ) are NOT captured here and are NOT canonical. They are the
//       crate's own `cargo test` gates (tests/recall.rs, tests/diskann.rs, tests/quant.rs,
//       tests/throughput.rs) — run them yourself for the figures. We cite a couple of the
//       crate's documented recall FLOORS as representative, clearly labelled "measured by
//       the crate's tests, recall < 1.0, not a canonical guarantee", and we present NO
//       latency number. Approximate search is APPROXIMATE: recall < 1.0; only the exact
//       scan (`nearest_exact`) and the captured runs above are answer-exact. `HashEmbedder`
//       is TEST-ONLY (lexical n-gram hashing, no semantics — "car" and "automobile" are
//       unrelated); a real deployment supplies its own `Embedder`. We do NOT claim semantic
//       quality from these captures — they demonstrate the PIPELINE and the exact geometry,
//       not embedding quality.
//
// Grounded in crates/sparq-vectors (README + SKILL.md + src/rewrite.rs + tests/labels.rs +
// tests/vec_predicate.rs) and the capture harness examples/capture_surface_vector.rs.

/** Whether the embeddings on this page come from a REAL semantic model. They do NOT —
 *  the captures use the deterministic, test-only `HashEmbedder` (lexical, no semantics). */
export const IS_SEMANTIC_EMBEDDER = false as const;

/** Whether the captured search used the answer-EXACT backend (vs an approximate index). */
export const IS_EXACT_BACKEND = true as const;

/** The `vec:` predicate vocabulary namespace (crates/sparq-vectors/src/rewrite.rs). */
export const VEC_NS = "http://sparq.dev/vec#" as const;

// ── (A) GENUINELY LIVE-CAPTURED ─────────────────────────────────────────────────────────

/** One neighbour from the captured `nearest_term_exact` run: (term, exact f32 cosine). */
export interface Neighbour {
  /** The neighbour's term, verbatim oxrdf::Term::Display (here always an `<iri>`). */
  term: string;
  /** The engine's exact f32 cosine, pasted verbatim (the page rounds for display). */
  cosine: number;
}

/**
 * The Usain Bolt label-embedding nearest-neighbour run (the bead's requested example).
 * Captured VERBATIM from `embed_labels` (deterministic `HashEmbedder::new(64)`) +
 * `nearest_term_exact(store, graph, <bolt>, 4)` over a 5-entity in-memory graph. The seed
 * `<bolt>` is excluded; neighbours are best-first by cosine. HashEmbedder similarity is
 * LEXICAL (shared n-grams), not semantic — "Usain Bolt" is nearest "Usain Bolt Junior"
 * because the strings overlap, NOT because the model knows they are sprinters.
 */
export const BOLT = {
  /** The five rdfs:label / skos:prefLabel entities the fixture declares. */
  fixture: [
    { iri: "http://example.org/bolt", label: "Usain Bolt" },
    { iri: "http://example.org/bolt2", label: "Usain Bolt Junior" },
    { iri: "http://example.org/blake", label: "Yohan Blake" },
    { iri: "http://example.org/powell", label: "Asafa Powell" },
    { iri: "http://example.org/coubertin", label: "Pierre de Coubertin" },
  ],
  /** Entities embedded by `embed_labels` (all five carry a label literal). */
  embedded: 5,
  /** The query seed, excluded from its own neighbour list. */
  seed: "<http://example.org/bolt>",
  /** Verbatim captured neighbours, best-first (captured 2026-06-19). */
  neighbours: [
    { term: "<http://example.org/bolt2>", cosine: 0.8762895 },
    { term: "<http://example.org/blake>", cosine: 0.12010295 },
    { term: "<http://example.org/powell>", cosine: -0.022044001 },
    { term: "<http://example.org/coubertin>", cosine: -0.08783152 },
  ] as Neighbour[],
} as const;

/** One captured `vec:nearest` / `vec:search` SPARQL run over the unit-circle store. */
export interface VecQuery {
  /** A short tag for the chip/selector. */
  id: string;
  /** A human description of what the query asks. */
  caption: string;
  /** The exact SPARQL the harness ran (verbatim). */
  sparql: string;
  /** Projected variable names, in order. */
  vars: string[];
  /**
   * Result rows, VERBATIM. Each cell is the engine's `oxrdf::Term::Display` (N-Triples)
   * string: an IRI `<…>`, a plain literal `"…"`, or a typed literal `"…"^^<…>`.
   */
  rows: string[][];
}

/**
 * The unit-circle vector store the `vec:` captures run over (5 entities, dim 2). The
 * geometry is legible: `a` points along +x, `b` along +y, `c` near +x, `d` along −x,
 * `e` near +y. Same fixture as crates/sparq-vectors/tests/vec_predicate.rs.
 */
export const UNIT_CIRCLE = [
  { iri: "http://ex/a", label: "alpha", vec: [1.0, 0.0] },
  { iri: "http://ex/b", label: "beta", vec: [0.0, 1.0] },
  { iri: "http://ex/c", label: "gamma", vec: [0.9, 0.1] },
  { iri: "http://ex/d", label: "delta", vec: [-1.0, 0.0] },
  { iri: "http://ex/e", label: "epsilon", vec: [0.2, 0.98] },
] as const;

/**
 * The captured `vec:` magic-predicate runs (captured 2026-06-19). Each `rows` array is the
 * VERBATIM engine output from `query_vec`. NOTE the honest serialization detail the
 * fabrication-shape would have blurred: the BGP-joined labels come back as PLAIN string
 * literals (`"epsilon"`) because that is exactly how they are stored — no invented
 * datatype — and the cosine scores from `vec:search` carry their `xsd:double` datatype.
 */
export const VEC_QUERIES: VecQuery[] = [
  {
    id: "nearest-by-vector",
    caption: "Nearest neighbours of a query vector",
    sparql:
      'PREFIX vec: <http://sparq.dev/vec#>\nSELECT ?node WHERE { ?node vec:nearest ( "1,0" 2 ) }',
    vars: ["node"],
    // "1,0" → the two most +x-aligned entities: a (exact +x) then c (near +x).
    rows: [["<http://ex/a>"], ["<http://ex/c>"]],
  },
  {
    id: "nearest-by-seed",
    caption: "Nearest neighbour of a seed IRI (seed excluded)",
    sparql:
      "PREFIX vec: <http://sparq.dev/vec#>\nSELECT ?node WHERE { ?node vec:nearest ( <http://ex/a> 1 ) }",
    vars: ["node"],
    // Neighbours of <a> (its stored +x vector); a itself is excluded → c is nearest.
    rows: [["<http://ex/c>"]],
  },
  {
    id: "nearest-joined",
    caption: "Neighbours joined to ordinary triples (their labels)",
    sparql:
      'PREFIX vec: <http://sparq.dev/vec#>\nSELECT ?label WHERE {\n  ?node vec:nearest ( "0,1" 2 ) .\n  ?node <http://ex/label> ?label .\n}',
    vars: ["label"],
    // "0,1" → b (+y) and e (near +y); their labels, joined through the plain triple
    // pattern. Plain string literals — verbatim, no invented datatype.
    rows: [['"epsilon"'], ['"beta"']],
  },
  {
    id: "search-score",
    caption: "vec:search binds the cosine score (ORDER BY DESC)",
    sparql:
      'PREFIX vec: <http://sparq.dev/vec#>\nSELECT ?node ?score WHERE {\n  ( ?node ?score ) vec:search ( "1,0" 3 )\n} ORDER BY DESC(?score)',
    vars: ["node", "score"],
    // a (cosine 1.0) > c (~0.994) > e (~0.200); the score is an xsd:double literal,
    // datatype intact, ORDER BY DESC recovers best-first over the unordered VALUES table.
    rows: [
      ["<http://ex/a>", '"1"^^<http://www.w3.org/2001/XMLSchema#double>'],
      [
        "<http://ex/c>",
        '"0.9938837289810181"^^<http://www.w3.org/2001/XMLSchema#double>',
      ],
      [
        "<http://ex/e>",
        '"0.19996000826358795"^^<http://www.w3.org/2001/XMLSchema#double>',
      ],
    ],
  },
];

// ── (B) HONESTLY ILLUSTRATIVE / NON-CANONICAL ───────────────────────────────────────────

/**
 * Representative recall FLOORS the crate's own `cargo test`s assert (NOT canonical, NOT
 * measured on this box). Approximate search is APPROXIMATE — recall < 1.0; only the exact
 * scan is answer-exact. These are shown only to be honest about the exact-vs-approximate
 * trade-off, with the "run the test yourself" pointer. We present NO latency number (it is
 * hardware-dependent and non-canonical).
 */
export const RECALL_NOTES: { backend: string; note: string; test: string }[] = [
  {
    backend: "Exact brute-force scan",
    note: "answer-exact (recall 1.0) — the ground-truth baseline, no third-party ANN dependency",
    test: "the default; nearest_exact",
  },
  {
    backend: "On-disk DiskANN / Vamana",
    note: "approximate; the crate documents recall@10 ≈ 0.966 — representative, < 1.0, not canonical",
    test: "tests/diskann.rs",
  },
  {
    backend: "In-RAM HNSW (opt-in approx-ann)",
    note: "approximate; recall < 1.0 — the only third-party ANN dependency, off by default",
    test: "tests/recall.rs",
  },
];

/** Lookup a captured vec: query by id (the selector key). */
export function vecQueryById(id: string): VecQuery | undefined {
  return VEC_QUERIES.find((q) => q.id === id);
}
