// [OPUS-4.8] sq-ndaz — pure, framework-free data + helpers for the /surface/geosparql
// (GeoSPARQL) showcase. sparq-geo is an OPT-IN native crate: nothing in the engine
// workspace's lean wasm bundle depends on it (the wasm build carries ZERO geometry
// code), and `sparq-server` exposes it only behind its non-default `geo` cargo feature.
// The static GitHub-Pages site has no backend, so this surface is the honest tier-e
// "captured-output walkthrough" fallback the feature-showcase design names
// (research/feature-showcase-site-design.md §0, surface (e)).
//
// =====================================================================================
// HONESTY CONTRACT — read before editing. A sibling page (sq-rnwc) was caught fabricating
// "captured" output (dropped datatypes, invented rows); the fix on the next pages (sq-3was
// / sq-dwdm) pinned the serialization in a test so drift is impossible. We do the SAME
// here. Every payload on this page is split into two clearly-labelled halves, and the test
// (site/test/geosparql.test.mjs) pins the serialization of the live-captured half:
//
//   (A) GENUINELY LIVE-CAPTURED — every `GeoQuery` (the geof: SPARQL result tables: FILTER
//       geof:distance, the geof:sfWithin spatial join, the geof:buffer chain, and the
//       topology-PROPERTY-form geosparql_rewrite) and every `INDEX_RUN` row (the R-tree
//       GeoIndex nearest / within_distance / intersects metres). These were produced by
//       RUNNING THE REAL sparq-geo binary over the tiny DECLARED in-memory Turtle fixtures
//       below — the SAME fixtures the crate's OWN COMMITTED tests assert against
//       (crates/sparq-geo/tests/registry_sparql.rs + tests/e2e.rs + tests/query_rewrite.rs,
//       runnable today with `cargo test -p sparq-geo --features engine`, which corroborate
//       the spatial-join order, the ≈343.6 km London–Paris distance and the rewrite/index
//       matches) — and PASTED VERBATIM. The capture was driven by a small `--features
//       engine` example harness over those fixtures, re-run twice and diffed for byte
//       stability. The result cells are the engine's
//       exact `oxrdf::Term::Display` (N-Triples) serialization: an IRI is `<…>`, a typed
//       literal is `"…"^^<datatype-iri>`. The geof:distance value is bound as an
//       `xsd:double` literal, datatype INTACT. The GeoIndex distances are the index's exact
//       great-circle f64 metres (the page rounds them for display, but the captured value
//       is pinned). Everything in (A) is DETERMINISTIC (a fixed fixture, planar DE-9IM
//       relations, haversine distance, ties broken by the R-tree order) — the same binary
//       over the same fixture yields BYTE-IDENTICAL output (verified by re-running the
//       harness twice and diffing). Re-CAPTURE (do not hand-edit) if a fixture or the
//       pipeline changes — run the example again and paste.
//
//   (B) HONESTLY ILLUSTRATIVE / NON-CANONICAL — the small inline SVG map is a LEGIBILITY
//       AID, not captured output: the city/region coordinates it draws are exactly the WKT
//       longitudes/latitudes in the fixtures (so the dots ARE the real geometry), but the
//       map projection, the coastline-free abstract background, and the "≈ km" annotations
//       are illustrative chrome. No latency or throughput number is shown (hardware-
//       dependent, non-canonical). The metric-distance caveat is real: geof:distance with
//       metric units is exact haversine when an operand is a point, but a local
//       equirectangular approximation between two extended geometries — accurate locally,
//       distorted at continental scale or near the poles (see the crate's SKILL.md).
//
// Grounded in skills/geosparql/SKILL.md + crates/sparq-geo (README + the committed
// tests/registry_sparql.rs + tests/e2e.rs + tests/query_rewrite.rs that corroborate every
// captured value here).

/** Whether the geof: result tables / GeoIndex metres come from the REAL binary (they do —
 *  captured verbatim from the answer-exact engine over a declared fixture, not illustrative). */
export const IS_LIVE_CAPTURED = true as const;

/** The `geof:` function vocabulary namespace (OGC GeoSPARQL). */
export const GEOF_NS = "http://www.opengis.net/def/function/geosparql/" as const;

/** The `geo:` ontology namespace (geometry literals + topology properties). */
export const GEO_NS = "http://www.opengis.net/ont/geosparql#" as const;

/** The OGC `uom:` unit-of-measure namespace. */
export const UOM_NS = "http://www.opengis.net/def/uom/OGC/1.0/" as const;

// ── The declared fixtures (the EXACT Turtle the capture harness loaded) ─────────────────

/** A WKT point/polygon feature on the map, keyed by its short fixture name. */
export interface MapFeature {
  /** The fixture id (`london`, `france`, …). */
  id: string;
  /** A human label for the legend. */
  label: string;
  /** "point" | "polygon" — drives the marker shape. */
  kind: "point" | "polygon";
  /**
   * The WKT longitude/latitude coordinates, VERBATIM from the fixture. A point is a
   * single [lon, lat]; a polygon is its exterior ring as a list of [lon, lat].
   */
  coords: [number, number][];
}

/**
 * The cities + areas fixture (3 points, 2 polygons) the geof: SPARQL captures ran over.
 * VERBATIM from crates/sparq-geo/tests/registry_sparql.rs (and the capture harness):
 * the UK box contains London; the France box contains Paris and Lyon.
 */
export const CITIES: MapFeature[] = [
  { id: "london", label: "London", kind: "point", coords: [[-0.1278, 51.5074]] },
  { id: "paris", label: "Paris", kind: "point", coords: [[2.3522, 48.8566]] },
  { id: "lyon", label: "Lyon", kind: "point", coords: [[4.8357, 45.764]] },
  {
    id: "uk",
    label: "UK box",
    kind: "polygon",
    coords: [
      [-6, 50],
      [2, 50],
      [2, 59],
      [-6, 59],
      [-6, 50],
    ],
  },
  {
    id: "france",
    label: "France box",
    kind: "polygon",
    coords: [
      [-1, 42.5],
      [7, 42.5],
      [7, 51],
      [-1, 51],
      [-1, 42.5],
    ],
  },
];

// ── (A) GENUINELY LIVE-CAPTURED — the geof: SPARQL result tables ─────────────────────────

/** One captured geof: SPARQL run. */
export interface GeoQuery {
  /** A short tag for the chip/selector. */
  id: string;
  /** A human description of what the query asks. */
  caption: string;
  /** The exact SPARQL the harness ran (verbatim, pretty-printed). */
  sparql: string;
  /** Projected variable names, in order. */
  vars: string[];
  /**
   * Result rows, VERBATIM. Each cell is the engine's `oxrdf::Term::Display` (N-Triples)
   * string: an IRI `<…>` or a typed literal `"…"^^<…>`.
   */
  rows: string[][];
  /** Whether this query is driven by the opt-in geosparql_rewrite topology-property form. */
  rewrite?: boolean;
}

/**
 * The captured geof: runs (captured 2026-06-19, byte-identical across re-runs). Each `rows`
 * array is the VERBATIM engine output. NOTE the honest serialization detail the
 * fabrication-shape would have blurred: the geof:distance value comes back as a typed
 * `xsd:double` literal (datatype intact), NOT a bare number; the IRIs are full `<…>` forms.
 */
export const GEO_QUERIES: GeoQuery[] = [
  {
    id: "distance-400km",
    caption: "Cities within 400 km of London (geof:distance)",
    sparql: `PREFIX geo:  <http://www.opengis.net/ont/geosparql#>
PREFIX geof: <http://www.opengis.net/def/function/geosparql/>
PREFIX uom:  <http://www.opengis.net/def/uom/OGC/1.0/>
PREFIX ex:   <http://ex/>
SELECT ?city ?km WHERE {
  ex:london ex:loc ?here . ?city ex:loc ?there .
  BIND(geof:distance(?here, ?there, uom:kilometre) AS ?km)
  FILTER(?city != ex:london && ?km < 400)
} ORDER BY ?km`,
    vars: ["city", "km"],
    // London↔Paris ≈ 343.56 km < 400; Lyon (≈ 750 km) excluded. The ?km is an xsd:double.
    rows: [
      [
        "<http://ex/paris>",
        '"343.55653488088325"^^<http://www.w3.org/2001/XMLSchema#double>',
      ],
    ],
  },
  {
    id: "spatial-join",
    caption: "Which city is within which polygon (geof:sfWithin join)",
    sparql: `PREFIX geo:  <http://www.opengis.net/ont/geosparql#>
PREFIX geof: <http://www.opengis.net/def/function/geosparql/>
PREFIX ex:   <http://ex/>
SELECT ?city ?region WHERE {
  ?city ex:loc ?pt . ?region ex:area ?poly .
  FILTER(geof:sfWithin(?pt, ?poly))
} ORDER BY ?city`,
    vars: ["city", "region"],
    // The real spatial join over 3 points × 2 polygons: London in the UK box, Paris and
    // Lyon in the France box.
    rows: [
      ["<http://ex/london>", "<http://ex/uk>"],
      ["<http://ex/lyon>", "<http://ex/france>"],
      ["<http://ex/paris>", "<http://ex/france>"],
    ],
  },
  {
    id: "buffer-chain",
    caption: "Inside a 400 km buffer of London (geof:buffer ∘ geof:sfWithin)",
    sparql: `PREFIX geo:  <http://www.opengis.net/ont/geosparql#>
PREFIX geof: <http://www.opengis.net/def/function/geosparql/>
PREFIX uom:  <http://www.opengis.net/def/uom/OGC/1.0/>
PREFIX ex:   <http://ex/>
SELECT ?city WHERE {
  ex:london ex:loc ?here . ?city ex:loc ?there .
  FILTER(?city != ex:london &&
         geof:sfWithin(?there, geof:buffer(?here, 400, uom:kilometre)))
} ORDER BY ?city`,
    vars: ["city"],
    // geof:buffer returns a geo:wktLiteral that feeds straight into geof:sfWithin in the
    // same expression: only Paris falls inside the 400 km buffer.
    rows: [["<http://ex/paris>"]],
  },
  {
    id: "rewrite-sfwithin",
    caption: "Topology PROPERTY form — ?f geo:sfWithin ex:region (query rewrite)",
    sparql: `PREFIX geo: <http://www.opengis.net/ont/geosparql#>
PREFIX ex:  <http://example.org/>
SELECT ?f WHERE { ?f geo:sfWithin ex:region } ORDER BY ?f`,
    vars: ["f"],
    rewrite: true,
    // NO asserted geo:sfWithin triple exists. The geosparql_rewrite extension resolves each
    // feature's (hasDefaultGeometry|hasGeometry)/asWKT geometry and applies geof:sfWithin:
    // London and Big Ben (inside) match, and the region itself (reflexive under DE-9IM);
    // Paris (far outside) does not.
    rows: [
      ["<http://example.org/bigben>"],
      ["<http://example.org/london>"],
      ["<http://example.org/region>"],
    ],
  },
];

// ── (A) GENUINELY LIVE-CAPTURED — the R-tree GeoIndex runs ───────────────────────────────

/** One captured GeoIndex row: (entity term, great-circle metres or null for intersects). */
export interface IndexHit {
  /** The entity term, verbatim oxrdf::Term::Display (an `<iri>`). */
  term: string;
  /** Exact great-circle metres from the index, pasted verbatim; null for intersects. */
  metres: number | null;
}

/** One captured GeoIndex query (nearest / within_distance / intersects). */
export interface IndexRun {
  id: string;
  caption: string;
  /** The Rust call the harness ran (verbatim). */
  call: string;
  hits: IndexHit[];
}

/**
 * The features fixture (a region polygon + 3 points) the GeoIndex ran over. VERBATIM from
 * crates/sparq-geo/tests/query_rewrite.rs — the inner-London box, the London point inside
 * it, Big Ben just inside, and Paris far outside.
 */
export const FEATURES: MapFeature[] = [
  {
    id: "region",
    label: "Inner-London box",
    kind: "polygon",
    coords: [
      [-0.25, 51.45],
      [0.05, 51.45],
      [0.05, 51.57],
      [-0.25, 51.57],
      [-0.25, 51.45],
    ],
  },
  { id: "london", label: "London", kind: "point", coords: [[-0.13, 51.51]] },
  { id: "bigben", label: "Big Ben", kind: "point", coords: [[-0.1246, 51.5007]] },
  { id: "paris", label: "Paris", kind: "point", coords: [[2.3522, 48.8566]] },
];

/** The R-tree centre the nearest / within_distance queries used (central London). */
export const INDEX_CENTER = { lon: -0.13, lat: 51.51 } as const;

/**
 * The captured GeoIndex runs (captured 2026-06-19, byte-identical across re-runs). The
 * metres are the index's exact great-circle f64 (the page rounds for display). The index
 * built 4 entities, 0 skipped — entities resolve through hasGeometry/hasDefaultGeometry.
 */
export const INDEX_RUNS: IndexRun[] = [
  {
    id: "nearest",
    caption: "k-nearest to central London (R-tree, k = 4)",
    call: "index.nearest(Point(-0.13, 51.51), 4)",
    // london + region both contain/sit-on the centre (0 m), then Big Ben (~1.1 km), then
    // Paris (~344 km) — best-first.
    hits: [
      { term: "<http://example.org/london>", metres: 0 },
      { term: "<http://example.org/region>", metres: 0 },
      { term: "<http://example.org/bigben>", metres: 1099.5813838365123 },
      { term: "<http://example.org/paris>", metres: 343882.44355547347 },
    ],
  },
  {
    id: "within",
    caption: "Within 5 km of central London (radius query)",
    call: "index.within_distance(Point(-0.13, 51.51), 5_000.0, None)",
    // Only the London-area entities; Paris (~344 km) excluded.
    hits: [
      { term: "<http://example.org/london>", metres: 0 },
      { term: "<http://example.org/region>", metres: 0 },
      { term: "<http://example.org/bigben>", metres: 1099.5813838365123 },
    ],
  },
  {
    id: "intersects",
    caption: "Intersects a box over the Channel + Paris",
    call:
      'index.intersects_wkt("POLYGON((1 48, 3 48, 3 49.5, 1 49.5, 1 48))")',
    // The box covers Paris only (no distance is returned by intersects).
    hits: [{ term: "<http://example.org/paris>", metres: null }],
  },
];

/** The index build stats the harness reported (verbatim). */
export const INDEX_BUILD = { len: 4, skipped: 0 } as const;

// ── (B) HONESTLY ILLUSTRATIVE — the topology-relation families (vocabulary, not captured) ─

/**
 * The three GeoSPARQL topology-relation families sparq-geo implements, for the capability
 * note. These are the VOCABULARY (function names), not a captured run — each family is a
 * set of DE-9IM relations the crate evaluates; the sf* `geof:sfWithin` run above is the
 * captured proof that the family works.
 */
export const RELATION_FAMILIES: { prefix: string; name: string; note: string }[] = [
  {
    prefix: "sf*",
    name: "Simple Features",
    note: "sfEquals / sfDisjoint / sfIntersects / sfTouches / sfCrosses / sfWithin / sfContains / sfOverlaps",
  },
  {
    prefix: "eh*",
    name: "Egenhofer",
    note: "ehEquals / ehDisjoint / ehMeet / ehOverlap / ehCovers / ehCoveredBy / ehInside / ehContains",
  },
  {
    prefix: "rcc8*",
    name: "RCC8",
    note: "rcc8eq / rcc8dc / rcc8ec / rcc8po / rcc8tppi / rcc8tpp / rcc8ntpp / rcc8ntppi",
  },
];

// ── helpers ──────────────────────────────────────────────────────────────────────────────

/** A short label for an `<iri>` cell — strip the namespace for legibility. */
export function shortIri(cell: string): string {
  const m = cell.match(/^<.*[/#]([^/#>]+)>$/);
  return m ? m[1] : cell;
}

/** Lookup a captured geof: query by id (the selector key). */
export function geoQueryById(id: string): GeoQuery | undefined {
  return GEO_QUERIES.find((q) => q.id === id);
}

/** Lookup a captured GeoIndex run by id. */
export function indexRunById(id: string): IndexRun | undefined {
  return INDEX_RUNS.find((r) => r.id === id);
}
