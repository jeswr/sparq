// [OPUS-4.8] sq-ndaz — unit tests for the /surface/geosparql (GeoSPARQL) walkthrough.
// This surface is tier-e (no backend behind the static site) so it replays REAL captured
// output from the sparq-geo binary (built with `--features engine`) over tiny declared
// in-memory Turtle fixtures — the SAME fixtures the crate's own committed tests assert
// against (crates/sparq-geo/tests/registry_sparql.rs + tests/e2e.rs + tests/query_rewrite.rs,
// runnable with `cargo test -p sparq-geo --features engine`). These tests pin the captured
// data's SHAPE *and SERIALIZATION* so the page can
// never silently drift into a fabrication — the exact failure mode the sibling http-server
// page (sq-rnwc) was caught in (dropped datatypes, invented rows). In particular: every
// result cell is the engine's verbatim oxrdf::Term::Display string; the geof:distance value
// carries its xsd:double datatype EXACTLY; the GeoIndex metres are pinned; and the map
// coordinates are the verbatim WKT lon/lat from the fixtures. Run via `npm run test:unit`.
import { test } from "node:test";
import assert from "node:assert/strict";

import {
  CITIES,
  FEATURES,
  GEO_NS,
  GEO_QUERIES,
  GEOF_NS,
  INDEX_BUILD,
  INDEX_CENTER,
  INDEX_RUNS,
  IS_LIVE_CAPTURED,
  RELATION_FAMILIES,
  UOM_NS,
  geoQueryById,
  indexRunById,
  shortIri,
} from "../src/lib/geosparql.ts";

test("the captures are flagged as live (real binary), and the namespaces are the OGC ones", () => {
  assert.equal(IS_LIVE_CAPTURED, true);
  assert.equal(GEOF_NS, "http://www.opengis.net/def/function/geosparql/");
  assert.equal(GEO_NS, "http://www.opengis.net/ont/geosparql#");
  assert.equal(UOM_NS, "http://www.opengis.net/def/uom/OGC/1.0/");
});

// ── The map fixtures are the verbatim WKT lon/lat ────────────────────────────────────────
test("every map feature carries WKT lon/lat coordinates (lon in [-180,180], lat in [-90,90])", () => {
  for (const f of [...CITIES, ...FEATURES]) {
    assert.ok(f.coords.length >= 1, `${f.id}: no coordinates`);
    if (f.kind === "point") {
      assert.equal(f.coords.length, 1, `${f.id}: a point has exactly one coordinate`);
    } else {
      // A polygon exterior ring is closed (first == last) and has ≥ 4 vertices.
      assert.ok(f.coords.length >= 4, `${f.id}: polygon ring too short`);
      assert.deepEqual(f.coords[0], f.coords[f.coords.length - 1], `${f.id}: ring not closed`);
    }
    for (const [lon, lat] of f.coords) {
      assert.ok(lon >= -180 && lon <= 180, `${f.id}: lon out of range: ${lon}`);
      assert.ok(lat >= -90 && lat <= 90, `${f.id}: lat out of range: ${lat}`);
    }
  }
});

test("the cities fixture is 3 points + 2 polygons (the geof: capture fixture)", () => {
  assert.equal(CITIES.filter((f) => f.kind === "point").length, 3);
  assert.equal(CITIES.filter((f) => f.kind === "polygon").length, 2);
  // London's coordinate is the verbatim WKT POINT(-0.1278 51.5074).
  const london = CITIES.find((f) => f.id === "london");
  assert.deepEqual(london.coords[0], [-0.1278, 51.5074]);
});

// ── The geof: SPARQL captures ────────────────────────────────────────────────────────────
test("every captured geof: query is real SPARQL with the geo:/geof: vocabulary", () => {
  for (const q of GEO_QUERIES) {
    // Each query references the geo: or geof: vocabulary.
    assert.match(
      q.sparql,
      /(geof:|geo:sf|geo:eh|geo:rcc8)/,
      `${q.id}: no geof:/geo: topology vocabulary`,
    );
    assert.ok(q.vars.length >= 1, `${q.id}: no projected variables`);
    for (const row of q.rows) {
      assert.equal(row.length, q.vars.length, `${q.id}: row arity != var count`);
    }
  }
});

// ── The honesty-regression guard: term serialization is VERBATIM oxrdf Display. ─────────
// The sibling page dropped a literal's datatype when a "captured" payload was hand-edited.
// Pin every result cell to its EXACT N-Triples serialization so a bare/mistyped value can
// never sneak in: an IRI is `<...>`, a plain literal is `"..."`, a typed literal is
// `"..."^^<datatype-iri>`.
test("every geof: result cell is a verbatim IRI or correctly-serialized literal", () => {
  const IRI = /^<[^>]+>$/;
  const PLAIN = /^"[^"]*"$/;
  const TYPED = /^".*"\^\^<[^>]+>$/s;
  for (const q of GEO_QUERIES) {
    for (const row of q.rows) {
      for (const cell of row) {
        const ok = IRI.test(cell) || PLAIN.test(cell) || TYPED.test(cell);
        assert.ok(ok, `${q.id}: cell is not a verbatim IRI/plain/typed literal: ${cell}`);
      }
    }
  }
});

test("geof:distance binds the distance as a verbatim xsd:double (datatype intact)", () => {
  const q = geoQueryById("distance-400km");
  // The exact captured output: Paris at 343.55653488088325 km, as an xsd:double literal.
  assert.deepEqual(q.rows, [
    [
      "<http://ex/paris>",
      '"343.55653488088325"^^<http://www.w3.org/2001/XMLSchema#double>',
    ],
  ]);
  const scoreCell = q.rows[0][1];
  assert.ok(
    scoreCell.endsWith("^^<http://www.w3.org/2001/XMLSchema#double>"),
    `distance missing xsd:double datatype: ${scoreCell}`,
  );
  // The bound value parses to ≈ 343.6 km and is strictly < 400 (the FILTER bound).
  const km = Number.parseFloat(scoreCell.slice(1));
  assert.ok(km < 400 && km > 340, `London–Paris ≈ 343.6 km, got ${km}`);
});

test("the spatial join returns the verbatim (city, region) pairs (pinned)", () => {
  const q = geoQueryById("spatial-join");
  assert.deepEqual(q.rows, [
    ["<http://ex/london>", "<http://ex/uk>"],
    ["<http://ex/lyon>", "<http://ex/france>"],
    ["<http://ex/paris>", "<http://ex/france>"],
  ]);
});

test("the geof:buffer chain keeps only Paris inside the 400 km buffer (pinned)", () => {
  const q = geoQueryById("buffer-chain");
  assert.deepEqual(q.rows, [["<http://ex/paris>"]]);
});

test("the topology PROPERTY form is the opt-in geosparql_rewrite, with the pinned matches", () => {
  const q = geoQueryById("rewrite-sfwithin");
  assert.equal(q.rewrite, true);
  // The rewrite has NO asserted geo:sfWithin triple — it resolves geometry and FILTERs.
  assert.match(q.sparql, /\?f geo:sfWithin ex:region/);
  assert.deepEqual(q.rows, [
    ["<http://example.org/bigben>"],
    ["<http://example.org/london>"],
    ["<http://example.org/region>"],
  ]);
});

// ── The R-tree GeoIndex captures ─────────────────────────────────────────────────────────
test("the GeoIndex built 4 entities, 0 skipped, centred on central London", () => {
  assert.equal(INDEX_BUILD.len, 4);
  assert.equal(INDEX_BUILD.skipped, 0);
  assert.deepEqual(INDEX_CENTER, { lon: -0.13, lat: 51.51 });
});

test("the GeoIndex metres are verbatim great-circle f64, best-first, pinned", () => {
  const nearest = indexRunById("nearest");
  // Pin the EXACT captured output (term + f64 metres) so a hand-edit can't invent a hit.
  assert.deepEqual(
    nearest.hits.map((h) => h.term),
    [
      "<http://example.org/london>",
      "<http://example.org/region>",
      "<http://example.org/bigben>",
      "<http://example.org/paris>",
    ],
  );
  assert.equal(nearest.hits[2].metres, 1099.5813838365123);
  assert.equal(nearest.hits[3].metres, 343882.44355547347);
  // nearest is best-first: metres are non-decreasing, every term is a verbatim IRI.
  let prev = -Infinity;
  for (const h of nearest.hits) {
    assert.match(h.term, /^<[^>]+>$/, `hit is not a verbatim IRI: ${h.term}`);
    assert.ok(h.metres >= prev, "nearest metres must be non-decreasing (best-first)");
    prev = h.metres;
  }
});

test("within_distance(5 km) drops Paris (~344 km away); intersects returns Paris with no metres", () => {
  const within = indexRunById("within");
  // Only the London-area entities are within 5 km; Paris must NOT appear.
  assert.ok(!within.hits.some((h) => h.term.includes("paris")), "Paris is ~344 km away, not within 5 km");
  for (const h of within.hits) {
    assert.ok(h.metres <= 5000, `within-5km hit exceeds radius: ${h.metres}`);
  }
  const intersects = indexRunById("intersects");
  assert.deepEqual(
    intersects.hits.map((h) => h.term),
    ["<http://example.org/paris>"],
  );
  // intersects returns no distance (metres is null) — the page must not invent one.
  assert.equal(intersects.hits[0].metres, null);
});

// ── The illustrative half is honestly labelled vocabulary ────────────────────────────────
test("the three topology-relation families are the sf*/eh*/rcc8* vocabulary", () => {
  assert.equal(RELATION_FAMILIES.length, 3);
  assert.deepEqual(
    RELATION_FAMILIES.map((r) => r.prefix),
    ["sf*", "eh*", "rcc8*"],
  );
  // The sf* family must name sfWithin (the captured proof above exercises it).
  const sf = RELATION_FAMILIES.find((r) => r.prefix === "sf*");
  assert.match(sf.note, /sfWithin/);
});

test("shortIri strips the namespace; lookups return undefined for unknown ids", () => {
  assert.equal(shortIri("<http://ex/paris>"), "paris");
  assert.equal(shortIri("<http://example.org/bigben>"), "bigben");
  assert.equal(geoQueryById("does-not-exist"), undefined);
  assert.equal(indexRunById("does-not-exist"), undefined);
});
