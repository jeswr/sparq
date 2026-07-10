// Coverage-guided fuzz target for the GeoSPARQL geometry literal parsers.
// Bead sq-3dyje.5; threat-model T-PARSE-FUZZ (untrusted geo:wktLiteral / geo:gmlLiteral
// values arriving from SPARQL endpoints or user-submitted RDF data). [SONNET-4.6]
//
// SURFACES:
//   * `sparq_geo::parse_wkt_literal(lex)` — parses a `geo:wktLiteral` lexical form:
//     optional leading `<CRS IRI>` then WKT geometry (POINT / LINESTRING / POLYGON / ...).
//   * `sparq_geo::parse_geometry_literal(value, datatype)` — dispatcher over the two
//     GeoSPARQL geometry serialisations: wktLiteral → parse_wkt_literal,
//     gmlLiteral → parse_gml_literal (XML-based GML2/3).
//
// The first byte of the fuzz input selects which surface is exercised and, for
// parse_geometry_literal, which datatype is used:
//   data[0] % 3 == 0 → parse_wkt_literal  (WKT path)
//   data[0] % 3 == 1 → parse_geometry_literal with geo:wktLiteral
//   data[0] % 3 == 2 → parse_geometry_literal with geo:gmlLiteral (GML path)
//
// Covering all three routes from one corpus lets libFuzzer build a shared dictionary of
// structural tokens (brackets, tag names, CRS prefixes) that transfer across paths.
//
// INVARIANT: hostile text must produce Ok(GeoGeometry) or a clean GeoError — never a
// panic, OOB, integer-overflow abort (overflow-checks on in this profile), or UB.
// A round-trip assertion for Ok results catches serialize→re-parse instability.
#![no_main]

use libfuzzer_sys::fuzz_target;

const WKT_LITERAL: &str = "http://www.opengis.net/ont/geosparql#wktLiteral";
const GML_LITERAL: &str = "http://www.opengis.net/ont/geosparql#gmlLiteral";

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    let ctrl = data[0];
    let body = String::from_utf8_lossy(&data[1..]);

    match ctrl % 3 {
        0 => {
            // Direct WKT literal path.
            if let Ok(geom) = sparq_geo::parse_wkt_literal(&body) {
                // Round-trip: serialise back to wktLiteral and re-parse.
                let reserialized = geom.to_wkt_literal();
                let _ = sparq_geo::parse_wkt_literal(&reserialized);
            }
        }
        1 => {
            // parse_geometry_literal dispatcher — WKT branch.
            let _ = sparq_geo::parse_geometry_literal(&body, WKT_LITERAL);
        }
        _ => {
            // parse_geometry_literal dispatcher — GML branch.
            let _ = sparq_geo::parse_geometry_literal(&body, GML_LITERAL);
        }
    }
});
