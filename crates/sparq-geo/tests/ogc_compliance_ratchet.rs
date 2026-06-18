//! [OPUS-4.8] sq-9h1r — OGC GeoSPARQL compliance ratchet.
//!
//! A fixture-gated, pass-count **ratchet** over a corpus of OGC GeoSPARQL
//! conformance-style assertions: each fixture is a geometry pair, a `geof:`
//! topology relation, and the expected boolean per the OGC GeoSPARQL
//! specification (1.0 Req 22/25/26, 1.1 §9–10). The harness runs every
//! assertion, counts passes, and enforces TWO invariants:
//!
//! 1. zero regressions — every fixture currently asserted MUST pass
//!    (`assert_eq!(fail, 0)`), so a relation that starts disagreeing with the
//!    hand-checked OGC truth value is a hard failure; and
//! 2. a pass-count FLOOR ([`OGC_RATCHET_FLOOR`]) that may only RISE — mirrors
//!    the SHACL W3C ratchet (`crates/sparq-shacl/tests/w3c_sparql.rs`) and the
//!    conformance-suite ratchets, so deleting coverage below the recorded floor
//!    fails the build.
//!
//! This is a HAND-CURATED conformance set, NOT the official OGC TEAM Engine /
//! CITE test suite (that suite is a Java/HTTP harness over a live endpoint and
//! is not vendored here). The fixtures are modelled on the relation semantics
//! the OGC Simple Features / GeoSPARQL DE-9IM patterns define; the expected
//! booleans are derived by hand from those matrices.
//!
//! ## OGC sub-conformance classes EXERCISED here
//!
//! * **Topology vocabulary extension — Simple Features (`geof:sf*`)**: all eight
//!   relations (`sfEquals` `sfDisjoint` `sfIntersects` `sfTouches` `sfCrosses`
//!   `sfWithin` `sfContains` `sfOverlaps`) over point / line / polygon operands,
//!   in BOTH operand orders where the relation is order-sensitive.
//! * **Topology vocabulary extension — Egenhofer (`geof:eh*`)**: all eight
//!   (`ehEquals` `ehDisjoint` `ehMeet` `ehOverlap` `ehCovers` `ehCoveredBy`
//!   `ehInside` `ehContains`) over REGION (polygon) pairs.
//! * **Topology vocabulary extension — RCC8 (`geof:rcc8*`)**: all eight
//!   (`rcc8eq` `rcc8dc` `rcc8ec` `rcc8po` `rcc8tpp` `rcc8tppi` `rcc8ntpp`
//!   `rcc8ntppi`) over region pairs.
//! * **Geometry serialization equivalence (GeoSPARQL §8.5)**: a subset of the
//!   sf fixtures is ALSO run with one operand supplied as a `geo:gmlLiteral`
//!   (GML-SF), asserting the WKT and GML serializations yield identical
//!   topology results (the `gml_equivalence` test).
//!
//! ## OGC sub-conformance classes NOT exercised here (tracked as beads)
//!
//! * The official OGC CITE / TEAM-Engine conformance suite itself (live-endpoint
//!   HTTP harness) — not vendored.
//! * The RDFS/OWL **Geometry / Feature class** entailment requirements
//!   (`geo:Feature`, `geo:Geometry`, the `geo:hasGeometry` reasoning rules) —
//!   now covered by `tests/entailment.rs` (sq-5ts8), which drives the generic
//!   `sparq-reason` RDFS/OWL-RL closure over the GeoSPARQL ontology axioms.
//! * The **query rewrite extension** (the `geo:sfWithin`-style triple-pattern
//!   property forms, as opposed to the `geof:` filter functions tested here) —
//!   NOT implemented; the gap is pinned by `tests/entailment.rs` (sq-5ts8).
//! * Non-topological `geof:` functions (`distance`, `buffer`, set operations,
//!   `boundary`/`envelope`/`convexHull`) — covered by `tests/relations.rs`, not
//!   this topology ratchet.

use sparq_geo::geof::lex;
use sparq_geo::GeoError;

/// [OPUS-4.8] Pass-count FLOOR for the OGC topology compliance fixtures
/// (sq-9h1r). A ratchet: it may only RISE. Set to the number of assertions in
/// [`FIXTURES`] (expanded across both operand orders) at commit time; raising it
/// is a deliberate act when fixtures are added.
const OGC_RATCHET_FLOOR: usize = 119;

/// One topology relation under test, named by its `geof:` local name and bound
/// to its lexical-level implementation.
type Rel = fn(&str, &str) -> Result<bool, GeoError>;

/// The full set of topology relations the ratchet drives, grouped by family.
fn all_relations() -> Vec<(&'static str, Rel)> {
    vec![
        // Simple Features (GeoSPARQL Req 22 / OGC SFA DE-9IM).
        ("sfEquals", lex::sf_equals as Rel),
        ("sfDisjoint", lex::sf_disjoint),
        ("sfIntersects", lex::sf_intersects),
        ("sfTouches", lex::sf_touches),
        ("sfCrosses", lex::sf_crosses),
        ("sfWithin", lex::sf_within),
        ("sfContains", lex::sf_contains),
        ("sfOverlaps", lex::sf_overlaps),
        // Egenhofer (GeoSPARQL Req 25).
        ("ehEquals", lex::eh_equals),
        ("ehDisjoint", lex::eh_disjoint),
        ("ehMeet", lex::eh_meet),
        ("ehOverlap", lex::eh_overlap),
        ("ehCovers", lex::eh_covers),
        ("ehCoveredBy", lex::eh_covered_by),
        ("ehInside", lex::eh_inside),
        ("ehContains", lex::eh_contains),
        // RCC8 (GeoSPARQL Req 26).
        ("rcc8eq", lex::rcc8_eq),
        ("rcc8dc", lex::rcc8_dc),
        ("rcc8ec", lex::rcc8_ec),
        ("rcc8po", lex::rcc8_po),
        ("rcc8tpp", lex::rcc8_tpp),
        ("rcc8tppi", lex::rcc8_tppi),
        ("rcc8ntpp", lex::rcc8_ntpp),
        ("rcc8ntppi", lex::rcc8_ntppi),
    ]
}

/// A single OGC conformance assertion: `relation(a, b) == expected`.
struct Fixture {
    a: &'static str,
    b: &'static str,
    relation: &'static str,
    expected: bool,
}

const fn fx(a: &'static str, b: &'static str, relation: &'static str, expected: bool) -> Fixture {
    Fixture { a, b, relation, expected }
}

// Reusable geometries (named so the assertion list reads like the OGC matrices).
const BIG: &str = "POLYGON((0 0, 4 0, 4 4, 0 4, 0 0))";
const BIG_REORDERED: &str = "POLYGON((4 0, 4 4, 0 4, 0 0, 4 0))";
const SMALL_INSIDE: &str = "POLYGON((1 1, 2 1, 2 2, 1 2, 1 1))";
const CORNER_TANGENT: &str = "POLYGON((0 0, 1 0, 1 1, 0 1, 0 0))";
const EDGE_ADJACENT: &str = "POLYGON((4 0, 6 0, 6 2, 4 2, 4 0))";
const FAR: &str = "POLYGON((9 9, 10 9, 10 10, 9 10, 9 9))";
const OVERLAP: &str = "POLYGON((3 3, 6 3, 6 6, 3 6, 3 3))";

/// The OGC conformance corpus. Each entry's expected boolean is hand-derived
/// from the relation's DE-9IM matrix per the OGC GeoSPARQL specification.
const FIXTURES: &[Fixture] = &[
    // ---- Simple Features: polygon vs polygon -------------------------------
    // Identical regions: equal, intersect, within, contains; not disjoint /
    // touches / crosses / overlaps.
    fx(BIG, BIG, "sfEquals", true),
    fx(BIG, BIG, "sfDisjoint", false),
    fx(BIG, BIG, "sfIntersects", true),
    fx(BIG, BIG, "sfTouches", false),
    fx(BIG, BIG, "sfCrosses", false),
    fx(BIG, BIG, "sfWithin", true),
    fx(BIG, BIG, "sfContains", true),
    fx(BIG, BIG, "sfOverlaps", false),
    // Same point set, different ring start vertex: still topologically equal.
    fx(BIG, BIG_REORDERED, "sfEquals", true),
    fx(BIG, BIG_REORDERED, "sfWithin", true),
    fx(BIG, BIG_REORDERED, "sfContains", true),
    // Small region strictly inside the big one.
    fx(SMALL_INSIDE, BIG, "sfWithin", true),
    fx(SMALL_INSIDE, BIG, "sfContains", false),
    fx(SMALL_INSIDE, BIG, "sfIntersects", true),
    fx(SMALL_INSIDE, BIG, "sfDisjoint", false),
    fx(SMALL_INSIDE, BIG, "sfTouches", false),
    fx(SMALL_INSIDE, BIG, "sfEquals", false),
    fx(SMALL_INSIDE, BIG, "sfOverlaps", false),
    // …and the reverse direction is contains.
    fx(BIG, SMALL_INSIDE, "sfContains", true),
    fx(BIG, SMALL_INSIDE, "sfWithin", false),
    // Far-apart regions: disjoint.
    fx(BIG, FAR, "sfDisjoint", true),
    fx(BIG, FAR, "sfIntersects", false),
    fx(BIG, FAR, "sfTouches", false),
    fx(BIG, FAR, "sfEquals", false),
    fx(BIG, FAR, "sfWithin", false),
    fx(BIG, FAR, "sfContains", false),
    // Edge-adjacent regions: touch (boundaries meet, interiors do not).
    fx(BIG, EDGE_ADJACENT, "sfTouches", true),
    fx(BIG, EDGE_ADJACENT, "sfIntersects", true),
    fx(BIG, EDGE_ADJACENT, "sfDisjoint", false),
    fx(BIG, EDGE_ADJACENT, "sfOverlaps", false),
    fx(BIG, EDGE_ADJACENT, "sfContains", false),
    fx(BIG, EDGE_ADJACENT, "sfWithin", false),
    // Partially overlapping regions: overlap.
    fx(BIG, OVERLAP, "sfOverlaps", true),
    fx(BIG, OVERLAP, "sfIntersects", true),
    fx(BIG, OVERLAP, "sfDisjoint", false),
    fx(BIG, OVERLAP, "sfTouches", false),
    fx(BIG, OVERLAP, "sfContains", false),
    fx(BIG, OVERLAP, "sfWithin", false),
    fx(BIG, OVERLAP, "sfEquals", false),
    // ---- Simple Features: line / point operands ----------------------------
    // Line crossing a polygon interior (dim-1 vs dim-2): crosses.
    fx("LINESTRING(-1 1, 3 1)", "POLYGON((0 0, 2 0, 2 2, 0 2, 0 0))", "sfCrosses", true),
    fx("LINESTRING(-1 1, 3 1)", "POLYGON((0 0, 2 0, 2 2, 0 2, 0 0))", "sfIntersects", true),
    fx("LINESTRING(-1 1, 3 1)", "POLYGON((0 0, 2 0, 2 2, 0 2, 0 0))", "sfWithin", false),
    fx("LINESTRING(-1 1, 3 1)", "POLYGON((0 0, 2 0, 2 2, 0 2, 0 0))", "sfTouches", false),
    // Two lines crossing at one interior point: crosses.
    fx("LINESTRING(0 0, 2 2)", "LINESTRING(0 2, 2 0)", "sfCrosses", true),
    fx("LINESTRING(0 0, 2 2)", "LINESTRING(0 2, 2 0)", "sfIntersects", true),
    fx("LINESTRING(0 0, 2 2)", "LINESTRING(0 2, 2 0)", "sfTouches", false),
    fx("LINESTRING(0 0, 2 2)", "LINESTRING(0 2, 2 0)", "sfOverlaps", false),
    // Two lines sharing only an endpoint: touch.
    fx("LINESTRING(0 0, 1 1)", "LINESTRING(1 1, 2 0)", "sfTouches", true),
    fx("LINESTRING(0 0, 1 1)", "LINESTRING(1 1, 2 0)", "sfCrosses", false),
    fx("LINESTRING(0 0, 1 1)", "LINESTRING(1 1, 2 0)", "sfIntersects", true),
    // Collinear partially-overlapping lines: overlap (same dim, partial share).
    fx("LINESTRING(0 0, 2 0)", "LINESTRING(1 0, 3 0)", "sfOverlaps", true),
    fx("LINESTRING(0 0, 2 0)", "LINESTRING(1 0, 3 0)", "sfCrosses", false),
    fx("LINESTRING(0 0, 2 0)", "LINESTRING(1 0, 3 0)", "sfIntersects", true),
    // Line strictly within a polygon.
    fx("LINESTRING(0.5 0.5, 1.5 1.5)", "POLYGON((0 0, 2 0, 2 2, 0 2, 0 0))", "sfWithin", true),
    fx("LINESTRING(0.5 0.5, 1.5 1.5)", "POLYGON((0 0, 2 0, 2 2, 0 2, 0 0))", "sfCrosses", false),
    // Point inside a polygon: within (interior).
    fx("POINT(1 1)", "POLYGON((0 0, 2 0, 2 2, 0 2, 0 0))", "sfWithin", true),
    fx("POINT(1 1)", "POLYGON((0 0, 2 0, 2 2, 0 2, 0 0))", "sfIntersects", true),
    fx("POINT(1 1)", "POLYGON((0 0, 2 0, 2 2, 0 2, 0 0))", "sfTouches", false),
    fx("POINT(1 1)", "POLYGON((0 0, 2 0, 2 2, 0 2, 0 0))", "sfDisjoint", false),
    // Point ON a polygon boundary: touches, NOT within (interior rule).
    fx("POINT(0 1)", "POLYGON((0 0, 2 0, 2 2, 0 2, 0 0))", "sfTouches", true),
    fx("POINT(0 1)", "POLYGON((0 0, 2 0, 2 2, 0 2, 0 0))", "sfWithin", false),
    fx("POINT(0 1)", "POLYGON((0 0, 2 0, 2 2, 0 2, 0 0))", "sfIntersects", true),
    // Point vs same / different point.
    fx("POINT(3 4)", "POINT(3 4)", "sfEquals", true),
    fx("POINT(3 4)", "POINT(3 4)", "sfIntersects", true),
    fx("POINT(3 4)", "POINT(3 4)", "sfDisjoint", false),
    fx("POINT(3 4)", "POINT(3 5)", "sfDisjoint", true),
    fx("POINT(3 4)", "POINT(3 5)", "sfIntersects", false),
    fx("POINT(3 4)", "POINT(3 5)", "sfEquals", false),
    // ---- Egenhofer: region pairs (Req 25 — exactly one holds per pair) ------
    fx(BIG, BIG, "ehEquals", true),
    fx(BIG, BIG, "ehDisjoint", false),
    fx(BIG, BIG, "ehMeet", false),
    fx(BIG, BIG, "ehOverlap", false),
    fx(BIG, BIG, "ehCovers", false),
    fx(BIG, BIG, "ehCoveredBy", false),
    fx(BIG, BIG, "ehInside", false),
    fx(BIG, BIG, "ehContains", false),
    fx(BIG, FAR, "ehDisjoint", true),
    fx(BIG, FAR, "ehEquals", false),
    fx(BIG, FAR, "ehMeet", false),
    fx(BIG, EDGE_ADJACENT, "ehMeet", true),
    fx(BIG, EDGE_ADJACENT, "ehDisjoint", false),
    fx(BIG, EDGE_ADJACENT, "ehOverlap", false),
    fx(BIG, OVERLAP, "ehOverlap", true),
    fx(BIG, OVERLAP, "ehMeet", false),
    fx(BIG, OVERLAP, "ehEquals", false),
    fx(SMALL_INSIDE, BIG, "ehInside", true),
    fx(SMALL_INSIDE, BIG, "ehContains", false),
    fx(SMALL_INSIDE, BIG, "ehCoveredBy", false),
    fx(BIG, SMALL_INSIDE, "ehContains", true),
    fx(BIG, SMALL_INSIDE, "ehInside", false),
    fx(CORNER_TANGENT, BIG, "ehCoveredBy", true),
    fx(CORNER_TANGENT, BIG, "ehInside", false),
    fx(CORNER_TANGENT, BIG, "ehCovers", false),
    fx(BIG, CORNER_TANGENT, "ehCovers", true),
    fx(BIG, CORNER_TANGENT, "ehContains", false),
    fx(BIG, CORNER_TANGENT, "ehCoveredBy", false),
    // ---- RCC8: region pairs (Req 26 — exactly one holds per pair) -----------
    fx(BIG, BIG, "rcc8eq", true),
    fx(BIG, BIG, "rcc8dc", false),
    fx(BIG, BIG, "rcc8ec", false),
    fx(BIG, BIG, "rcc8po", false),
    fx(BIG, FAR, "rcc8dc", true),
    fx(BIG, FAR, "rcc8ec", false),
    fx(BIG, FAR, "rcc8eq", false),
    fx(BIG, EDGE_ADJACENT, "rcc8ec", true),
    fx(BIG, EDGE_ADJACENT, "rcc8dc", false),
    fx(BIG, EDGE_ADJACENT, "rcc8po", false),
    fx(BIG, OVERLAP, "rcc8po", true),
    fx(BIG, OVERLAP, "rcc8ec", false),
    fx(BIG, OVERLAP, "rcc8eq", false),
    fx(SMALL_INSIDE, BIG, "rcc8ntpp", true),
    fx(SMALL_INSIDE, BIG, "rcc8tpp", false),
    fx(SMALL_INSIDE, BIG, "rcc8ntppi", false),
    fx(BIG, SMALL_INSIDE, "rcc8ntppi", true),
    fx(BIG, SMALL_INSIDE, "rcc8ntpp", false),
    fx(CORNER_TANGENT, BIG, "rcc8tpp", true),
    fx(CORNER_TANGENT, BIG, "rcc8ntpp", false),
    fx(CORNER_TANGENT, BIG, "rcc8tppi", false),
    fx(BIG, CORNER_TANGENT, "rcc8tppi", true),
    fx(BIG, CORNER_TANGENT, "rcc8tpp", false),
];

fn run_fixtures(fixtures: &[Fixture]) -> (usize, usize, Vec<String>) {
    let relations = all_relations();
    let lookup = |name: &str| -> Rel {
        relations
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, f)| *f)
            .unwrap_or_else(|| panic!("fixture references an unknown relation: {name:?}"))
    };
    let mut pass = 0usize;
    let mut fail = 0usize;
    let mut failures = Vec::new();
    for f in fixtures {
        let rel = lookup(f.relation);
        match rel(f.a, f.b) {
            Ok(got) if got == f.expected => pass += 1,
            Ok(got) => {
                fail += 1;
                failures.push(format!(
                    "geof:{}({}, {}) = {got}, expected {}",
                    f.relation, f.a, f.b, f.expected
                ));
            }
            Err(e) => {
                fail += 1;
                failures.push(format!("geof:{}({}, {}) errored: {e}", f.relation, f.a, f.b));
            }
        }
    }
    (pass, fail, failures)
}

#[test]
fn ogc_topology_compliance_ratchet() {
    let (pass, fail, failures) = run_fixtures(FIXTURES);

    println!("\nOGC GeoSPARQL topology compliance scoreboard");
    for why in &failures {
        println!("  FAIL {why}");
    }
    println!("pass {pass} / fail {fail} (floor {OGC_RATCHET_FLOOR})");

    assert_eq!(fail, 0, "OGC topology compliance regressions:\n{}", failures.join("\n"));
    assert!(
        pass >= OGC_RATCHET_FLOOR,
        "OGC compliance pass count regressed: {pass} < floor {OGC_RATCHET_FLOOR} — \
         if fixtures were removed deliberately, lower the floor in the same commit"
    );
}

/// GeoSPARQL §8.5: the WKT and GML serializations of the same geometry are
/// equivalent. A subset of the sf fixtures is re-run with operand `b` supplied
/// as a `geo:gmlLiteral` (GML-SF); the topology result MUST be identical to the
/// all-WKT run. This exercises the geometry-serialization-equivalence
/// sub-conformance class against the GML parser.
#[test]
fn gml_equivalence_for_topology() {
    // (a-WKT, b-WKT, b-as-GML, relation, expected) — b is the GML-supplied side.
    struct GmlFixture {
        a: &'static str,
        b_gml: &'static str,
        relation: &'static str,
        expected: bool,
    }
    // A 2x2 square at the origin, in GML-SF. The gml: prefix is declared on the
    // root element so the fixture is well-formed XML. [OPUS-4.8]
    const SQUARE_GML: &str = "<gml:Polygon xmlns:gml=\"http://www.opengis.net/gml\">\
<gml:exterior><gml:LinearRing>\
<gml:posList>0 0 2 0 2 2 0 2 0 0</gml:posList>\
</gml:LinearRing></gml:exterior></gml:Polygon>";
    // A point, in GML-SF (gml: prefix declared on the root element). [OPUS-4.8]
    const POINT_GML: &str =
        "<gml:Point xmlns:gml=\"http://www.opengis.net/gml\"><gml:pos>1 1</gml:pos></gml:Point>";

    let fixtures = [
        // Point inside the GML square: within / intersects.
        GmlFixture { a: "POINT(1 1)", b_gml: SQUARE_GML, relation: "sfWithin", expected: true },
        GmlFixture { a: "POINT(1 1)", b_gml: SQUARE_GML, relation: "sfIntersects", expected: true },
        GmlFixture {
            a: "POINT(9 9)",
            b_gml: SQUARE_GML,
            relation: "sfDisjoint",
            expected: true,
        },
        // The GML point sits inside the WKT square.
        GmlFixture {
            a: "POLYGON((0 0, 2 0, 2 2, 0 2, 0 0))",
            b_gml: POINT_GML,
            relation: "sfContains",
            expected: true,
        },
    ];

    let relations = all_relations();
    let lookup = |name: &str| -> Rel {
        relations.iter().find(|(n, _)| *n == name).map(|(_, f)| *f).unwrap()
    };

    for f in &fixtures {
        let rel = lookup(f.relation);
        // GML literals carry their datatype via parse_geometry_literal, but the
        // lex:: relations parse WKT; so cross-check the GML path through the
        // crate's combined parser and the relation through the geometry API.
        let a = sparq_geo::parse_wkt_literal(f.a).expect("WKT a parses");
        let b = sparq_geo::parse_gml_literal(f.b_gml).expect("GML b parses");
        let got_gml = relate_via_name(f.relation, &a, &b);
        // The same pair, with b re-expressed as WKT, must agree.
        let b_wkt = b.to_wkt_literal();
        let got_wkt = rel(f.a, &b_wkt).expect("WKT relation evaluates");
        assert_eq!(
            got_gml, f.expected,
            "GML path geof:{}({}, <gml>) = {got_gml}, expected {}",
            f.relation, f.a, f.expected
        );
        assert_eq!(
            got_gml, got_wkt,
            "WKT/GML serialization disagreement for geof:{} on {} vs {b_wkt}",
            f.relation, f.a
        );
    }
}

/// Dispatch a relation by `geof:` local name over already-parsed geometries
/// (used by the GML-equivalence test, whose `b` operand is parsed from GML).
fn relate_via_name(
    name: &str,
    a: &sparq_geo::GeoGeometry,
    b: &sparq_geo::GeoGeometry,
) -> bool {
    use sparq_geo::geof;
    match name {
        "sfEquals" => geof::sf_equals(a, b),
        "sfDisjoint" => geof::sf_disjoint(a, b),
        "sfIntersects" => geof::sf_intersects(a, b),
        "sfTouches" => geof::sf_touches(a, b),
        "sfCrosses" => geof::sf_crosses(a, b),
        "sfWithin" => geof::sf_within(a, b),
        "sfContains" => geof::sf_contains(a, b),
        "sfOverlaps" => geof::sf_overlaps(a, b),
        other => panic!("relate_via_name: unsupported relation {other:?}"),
    }
    .expect("relation evaluates")
}

/// The recorded floor must never exceed what the corpus can actually deliver —
/// catches a stale floor left above the fixture count after a deletion.
#[test]
fn floor_is_consistent_with_corpus() {
    let (pass, fail, _) = run_fixtures(FIXTURES);
    assert_eq!(fail, 0);
    assert!(
        OGC_RATCHET_FLOOR <= pass,
        "OGC_RATCHET_FLOOR ({OGC_RATCHET_FLOOR}) exceeds achievable passes ({pass}) — \
         the floor is unreachable; lower it or add fixtures"
    );
}
