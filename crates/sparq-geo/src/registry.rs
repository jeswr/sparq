//! [`geof_registry`]: the `geof:` functions packaged as a sparq-engine
//! [`FunctionRegistry`] (SPARQL 17.6 extensible value testing), so they run
//! inside SPARQL `FILTER` / `BIND` / `SELECT` expressions:
//!
//! ```
//! use sparq_core::Graph;
//! use sparq_geo::geof_registry;
//!
//! let g = Graph::load_str(r#"
//!     @prefix geo: <http://www.opengis.net/ont/geosparql#> .
//!     <http://ex/london> <http://ex/loc> "POINT(-0.1278 51.5074)"^^geo:wktLiteral .
//!     <http://ex/paris>  <http://ex/loc> "POINT(2.3522 48.8566)"^^geo:wktLiteral .
//! "#, "turtle").unwrap();
//! let r = sparq_engine::query_with_functions(&g,
//!     "PREFIX geof: <http://www.opengis.net/def/function/geosparql/>
//!      PREFIX uom:  <http://www.opengis.net/def/uom/OGC/1.0/>
//!      SELECT ?a ?b WHERE {
//!        ?a <http://ex/loc> ?ga . ?b <http://ex/loc> ?gb .
//!        FILTER(STR(?a) < STR(?b) && geof:distance(?ga, ?gb, uom:kilometre) < 400)
//!      }", &geof_registry()).unwrap();
//! assert_eq!(r.len(), 1); // London–Paris ≈ 343.6 km
//! ```
//!
//! Argument/result conventions (each function mirrors [`crate::geof::lex`]):
//!
//! * geometry arguments must be `geo:wktLiteral` literals (anything else — wrong
//!   datatype, an IRI, a plain string — is a SPARQL *expression* error: the row is
//!   filtered by a `FILTER`, left unbound by a `BIND`; never a hard query error);
//! * `geof:distance`'s third argument is a unit IRI ([`crate::geof::Unit`]),
//!   result `xsd:double`;
//! * the `geof:sf*` relations return `xsd:boolean`;
//! * `geof:envelope` / `geof:boundary` / `geof:convexHull` return `geo:wktLiteral`;
//! * every [`crate::GeoError`] (WKT parse failure, CRS mismatch, unknown unit, …)
//!   is the same expression error.

use crate::geof::lex;
use crate::vocab::{GEOF_NS, WKT_LITERAL};
use crate::GeoError;
use oxrdf::{Literal, NamedNode, Term};
use sparq_engine::FunctionRegistry;

/// `Err` unless exactly `n` arguments were passed.
fn arity(name: &str, args: &[Term], n: usize) -> Result<(), String> {
    if args.len() == n {
        Ok(())
    } else {
        Err(format!("geof:{name} expects {n} argument(s), got {}", args.len()))
    }
}

/// The lexical form of argument `i`, which must be a `geo:wktLiteral` literal.
fn wkt_lex<'a>(name: &str, args: &'a [Term], i: usize) -> Result<&'a str, String> {
    match &args[i] {
        Term::Literal(l) if l.datatype().as_str() == WKT_LITERAL => Ok(l.value()),
        other => Err(format!(
            "geof:{name}: argument {} must be a geo:wktLiteral literal, got {other}",
            i + 1
        )),
    }
}

/// A `geo:wktLiteral` term from a lexical form produced by [`crate::geof::lex`].
fn wkt_term(lex: String) -> Term {
    Term::Literal(Literal::new_typed_literal(lex, NamedNode::new_unchecked(WKT_LITERAL)))
}

/// The unit-of-measure IRI in argument `i` (must be a named node).
fn unit_iri<'a>(name: &str, args: &'a [Term], i: usize) -> Result<&'a str, String> {
    match &args[i] {
        Term::NamedNode(unit) => Ok(unit.as_str()),
        other => Err(format!(
            "geof:{name}: argument {} must be a unit-of-measure IRI, got {other}",
            i + 1
        )),
    }
}

/// The numeric value of argument `i` (any literal whose lexical form parses as f64 —
/// the engine hands numerics over as their typed literals).
fn num_arg(name: &str, args: &[Term], i: usize) -> Result<f64, String> {
    match &args[i] {
        Term::Literal(l) => l.value().parse::<f64>().map_err(|_| {
            format!("geof:{name}: argument {} must be numeric, got {l}", i + 1)
        }),
        other => Err(format!("geof:{name}: argument {} must be numeric, got {other}", i + 1)),
    }
}

/// A [`crate::geof::lex`] binary relation (`sf_*` / `eh_*` / `rcc8_*`).
type LexRelation = fn(&str, &str) -> Result<bool, GeoError>;
/// A [`crate::geof::lex`] unary geometry function (`envelope` / `boundary` / `convex_hull`).
type LexUnary = fn(&str) -> Result<String, GeoError>;
/// A [`crate::geof::lex`] binary set operation (`intersection` / `union` / …).
type LexSetOp = fn(&str, &str) -> Result<String, GeoError>;

/// The `geof:` extension functions as a sparq-engine [`FunctionRegistry`] — pass it
/// to [`sparq_engine::query_with_functions`] (or scope any other entry point with
/// [`sparq_engine::with_functions`]) to evaluate GeoSPARQL functions inside SPARQL.
///
/// Registered IRIs (all under `http://www.opengis.net/def/function/geosparql/`):
///
/// * `distance(g1, g2, unitIri)` -> `xsd:double`;
/// * the relation families, all `(g1, g2)` -> `xsd:boolean`: simple features
///   `sfEquals sfDisjoint sfIntersects sfTouches sfCrosses sfWithin sfContains
///   sfOverlaps`, Egenhofer `ehEquals ehDisjoint ehMeet ehOverlap ehCovers
///   ehCoveredBy ehInside ehContains`, RCC8 `rcc8eq rcc8dc rcc8ec rcc8po
///   rcc8tppi rcc8tpp rcc8ntpp rcc8ntppi`, plus the generic
///   `relate(g1, g2, de9imPattern)`;
/// * `envelope` / `boundary` / `convexHull` `(g)` -> `geo:wktLiteral`;
/// * `buffer(g, radius, unitIri)` -> `geo:wktLiteral` (a MULTIPOLYGON);
/// * `intersection` / `union` / `difference` / `symDifference` `(g1, g2)` ->
///   `geo:wktLiteral` (POLYGONAL operands only);
/// * `getSRID(g)` -> `xsd:anyURI` (the geometry's CRS IRI).
///
/// Build it once and reuse it: the registry is cheaply cloneable and `Send + Sync`,
/// so one instance can serve every query on every thread for the process lifetime.
pub fn geof_registry() -> FunctionRegistry {
    let mut reg = FunctionRegistry::new();

    // geof:distance(?g1, ?g2, ?unitIri) -> xsd:double.
    reg.register(format!("{GEOF_NS}distance"), |args: &[Term]| {
        arity("distance", args, 3)?;
        let a = wkt_lex("distance", args, 0)?;
        let b = wkt_lex("distance", args, 1)?;
        let unit = unit_iri("distance", args, 2)?;
        let d = lex::distance(a, b, unit).map_err(|e| e.to_string())?;
        Ok(Term::Literal(Literal::from(d)))
    });

    // The relation families: geof:sf* / geof:eh* / geof:rcc8*(?g1, ?g2) -> xsd:boolean.
    let relations: [(&'static str, LexRelation); 24] = [
        // Simple features (GeoSPARQL Req 22-24).
        ("sfEquals", lex::sf_equals),
        ("sfDisjoint", lex::sf_disjoint),
        ("sfIntersects", lex::sf_intersects),
        ("sfTouches", lex::sf_touches),
        ("sfCrosses", lex::sf_crosses),
        ("sfWithin", lex::sf_within),
        ("sfContains", lex::sf_contains),
        ("sfOverlaps", lex::sf_overlaps),
        // Egenhofer (Req 25).
        ("ehEquals", lex::eh_equals),
        ("ehDisjoint", lex::eh_disjoint),
        ("ehMeet", lex::eh_meet),
        ("ehOverlap", lex::eh_overlap),
        ("ehCovers", lex::eh_covers),
        ("ehCoveredBy", lex::eh_covered_by),
        ("ehInside", lex::eh_inside),
        ("ehContains", lex::eh_contains),
        // RCC8 (Req 26).
        ("rcc8eq", lex::rcc8_eq),
        ("rcc8dc", lex::rcc8_dc),
        ("rcc8ec", lex::rcc8_ec),
        ("rcc8po", lex::rcc8_po),
        ("rcc8tppi", lex::rcc8_tppi),
        ("rcc8tpp", lex::rcc8_tpp),
        ("rcc8ntpp", lex::rcc8_ntpp),
        ("rcc8ntppi", lex::rcc8_ntppi),
    ];
    for (name, f) in relations {
        reg.register(format!("{GEOF_NS}{name}"), move |args: &[Term]| {
            arity(name, args, 2)?;
            let v = f(wkt_lex(name, args, 0)?, wkt_lex(name, args, 1)?).map_err(|e| e.to_string())?;
            Ok(Term::Literal(Literal::from(v)))
        });
    }

    // geof:relate(?g1, ?g2, ?de9imPattern) -> xsd:boolean (generic DE-9IM test).
    reg.register(format!("{GEOF_NS}relate"), |args: &[Term]| {
        arity("relate", args, 3)?;
        let a = wkt_lex("relate", args, 0)?;
        let b = wkt_lex("relate", args, 1)?;
        let pattern = match &args[2] {
            Term::Literal(l) => l.value(),
            other => {
                return Err(format!(
                    "geof:relate: argument 3 must be a DE-9IM pattern string, got {other}"
                ))
            }
        };
        let v = lex::relate(a, b, pattern).map_err(|e| e.to_string())?;
        Ok(Term::Literal(Literal::from(v)))
    });

    // The unary geometry functions: geof:*(?g) -> geo:wktLiteral.
    let unary: [(&'static str, LexUnary); 3] = [
        ("envelope", lex::envelope),
        ("boundary", lex::boundary),
        ("convexHull", lex::convex_hull),
    ];
    for (name, f) in unary {
        reg.register(format!("{GEOF_NS}{name}"), move |args: &[Term]| {
            arity(name, args, 1)?;
            Ok(wkt_term(f(wkt_lex(name, args, 0)?).map_err(|e| e.to_string())?))
        });
    }

    // The set operations: geof:*(?g1, ?g2) -> geo:wktLiteral (polygonal operands only).
    let set_ops: [(&'static str, LexSetOp); 4] = [
        ("intersection", lex::intersection),
        ("union", lex::union),
        ("difference", lex::difference),
        ("symDifference", lex::sym_difference),
    ];
    for (name, f) in set_ops {
        reg.register(format!("{GEOF_NS}{name}"), move |args: &[Term]| {
            arity(name, args, 2)?;
            Ok(wkt_term(f(wkt_lex(name, args, 0)?, wkt_lex(name, args, 1)?).map_err(|e| e.to_string())?))
        });
    }

    // geof:buffer(?g, ?radius, ?unitIri) -> geo:wktLiteral (a MULTIPOLYGON).
    reg.register(format!("{GEOF_NS}buffer"), |args: &[Term]| {
        arity("buffer", args, 3)?;
        let g = wkt_lex("buffer", args, 0)?;
        let radius = num_arg("buffer", args, 1)?;
        let unit = unit_iri("buffer", args, 2)?;
        Ok(wkt_term(lex::buffer(g, radius, unit).map_err(|e| e.to_string())?))
    });

    // geof:getSRID(?g) -> xsd:anyURI (the geometry's CRS IRI).
    reg.register(format!("{GEOF_NS}getSRID"), |args: &[Term]| {
        arity("getSRID", args, 1)?;
        let iri = lex::get_srid(wkt_lex("getSRID", args, 0)?).map_err(|e| e.to_string())?;
        Ok(Term::Literal(Literal::new_typed_literal(
            iri,
            NamedNode::new_unchecked("http://www.w3.org/2001/XMLSchema#anyURI"),
        )))
    });

    reg
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wkt(lex: &str) -> Term {
        wkt_term(lex.to_string())
    }

    #[test]
    fn registry_contents() {
        let reg = geof_registry();
        assert_eq!(reg.len(), 35);
        for name in [
            "distance", "relate", "getSRID", "buffer",
            "sfEquals", "sfDisjoint", "sfIntersects", "sfTouches", "sfCrosses",
            "sfWithin", "sfContains", "sfOverlaps",
            "ehEquals", "ehDisjoint", "ehMeet", "ehOverlap", "ehCovers", "ehCoveredBy",
            "ehInside", "ehContains",
            "rcc8eq", "rcc8dc", "rcc8ec", "rcc8po", "rcc8tppi", "rcc8tpp", "rcc8ntpp",
            "rcc8ntppi",
            "envelope", "boundary", "convexHull",
            "intersection", "union", "difference", "symDifference",
        ] {
            assert!(reg.get(&format!("{GEOF_NS}{name}")).is_some(), "missing geof:{name}");
        }
    }

    #[test]
    fn get_srid_term_level() {
        let reg = geof_registry();
        let f = reg.get(&format!("{GEOF_NS}getSRID")).unwrap();
        let Term::Literal(l) = f(&[wkt("POINT(1 2)")]).unwrap() else { panic!("literal") };
        assert_eq!(l.value(), crate::vocab::CRS84);
        assert_eq!(l.datatype().as_str(), "http://www.w3.org/2001/XMLSchema#anyURI");
        let Term::Literal(l) =
            f(&[wkt("<http://www.opengis.net/def/crs/EPSG/0/27700> POINT(1 2)")]).unwrap()
        else {
            panic!("literal")
        };
        assert_eq!(l.value(), "http://www.opengis.net/def/crs/EPSG/0/27700");
    }

    #[test]
    fn relate_term_level() {
        let reg = geof_registry();
        let f = reg.get(&format!("{GEOF_NS}relate")).unwrap();
        let a = wkt("POLYGON((0 0, 4 0, 4 4, 0 4, 0 0))");
        let b = wkt("POINT(1 1)");
        let pat = |p: &str| Term::Literal(Literal::new_simple_literal(p));
        // "contains" pattern.
        assert_eq!(
            f(&[a.clone(), b.clone(), pat("T*****FF*")]).unwrap().to_string(),
            "\"true\"^^<http://www.w3.org/2001/XMLSchema#boolean>"
        );
        // "disjoint" pattern is false for a contained point.
        assert_eq!(
            f(&[a.clone(), b.clone(), pat("FF*FF****")]).unwrap().to_string(),
            "\"false\"^^<http://www.w3.org/2001/XMLSchema#boolean>"
        );
        // Malformed pattern / non-string pattern are expression errors.
        assert!(f(&[a.clone(), b.clone(), pat("TTT")]).is_err());
        assert!(f(&[a.clone(), b.clone(), Term::NamedNode(NamedNode::new_unchecked("http://x/"))]).is_err());
    }

    #[test]
    fn set_operations_and_buffer_term_level() {
        let reg = geof_registry();
        let a = wkt("POLYGON((0 0, 2 0, 2 2, 0 2, 0 0))");
        let b = wkt("POLYGON((1 1, 3 1, 3 3, 1 3, 1 1))");
        let inter = reg.get(&format!("{GEOF_NS}intersection")).unwrap();
        let Term::Literal(l) = inter(&[a.clone(), b.clone()]).unwrap() else { panic!("literal") };
        assert_eq!(l.datatype().as_str(), WKT_LITERAL);
        assert!(l.value().contains("MULTIPOLYGON"), "got {}", l.value());
        // Non-polygonal operands are expression errors.
        assert!(inter(&[wkt("POINT(0 0)"), b.clone()]).is_err());
        // buffer: polygon out, radius must be numeric, unit must be known.
        let buffer = reg.get(&format!("{GEOF_NS}buffer")).unwrap();
        let metre = Term::NamedNode(NamedNode::new_unchecked(format!("{}metre", crate::vocab::UOM_NS)));
        let radius = Term::Literal(Literal::from(100.0));
        let Term::Literal(l) = buffer(&[wkt("POINT(0 51)"), radius.clone(), metre.clone()]).unwrap()
        else {
            panic!("literal")
        };
        assert!(l.value().contains("MULTIPOLYGON"), "got {}", l.value());
        assert!(buffer(&[wkt("POINT(0 51)"), wkt("POINT(0 0)"), metre]).is_err());
        assert!(buffer(&[wkt("POINT(0 51)"), radius, Term::NamedNode(NamedNode::new_unchecked("http://ex/furlong"))]).is_err());
    }

    #[test]
    fn distance_term_level() {
        let reg = geof_registry();
        let f = reg.get(&format!("{GEOF_NS}distance")).unwrap();
        let unit = Term::NamedNode(NamedNode::new_unchecked(format!("{}kilometre", crate::vocab::UOM_NS)));
        let out = f(&[wkt("POINT(-0.1278 51.5074)"), wkt("POINT(2.3522 48.8566)"), unit]).unwrap();
        let Term::Literal(l) = out else { panic!("expected a literal") };
        assert_eq!(l.datatype().as_str(), "http://www.w3.org/2001/XMLSchema#double");
        let km: f64 = l.value().parse().unwrap();
        assert!((km - 343.6).abs() < 1.0, "London–Paris ≈ 343.6 km, got {km}");
    }

    #[test]
    fn relation_and_unary_term_level() {
        let reg = geof_registry();
        let within = reg.get(&format!("{GEOF_NS}sfWithin")).unwrap();
        let poly = wkt("POLYGON((0 0, 4 0, 4 4, 0 4, 0 0))");
        assert_eq!(
            within(&[wkt("POINT(1 1)"), poly.clone()]).unwrap().to_string(),
            "\"true\"^^<http://www.w3.org/2001/XMLSchema#boolean>"
        );
        assert_eq!(
            within(&[wkt("POINT(9 9)"), poly.clone()]).unwrap().to_string(),
            "\"false\"^^<http://www.w3.org/2001/XMLSchema#boolean>"
        );
        let envelope = reg.get(&format!("{GEOF_NS}envelope")).unwrap();
        let Term::Literal(l) = envelope(&[poly]).unwrap() else { panic!("expected a literal") };
        assert_eq!(l.datatype().as_str(), WKT_LITERAL);
        assert!(l.value().contains("POLYGON"), "got {}", l.value());
    }

    #[test]
    fn bad_arguments_are_errs() {
        let reg = geof_registry();
        let within = reg.get(&format!("{GEOF_NS}sfWithin")).unwrap();
        // Wrong arity.
        assert!(within(&[wkt("POINT(1 1)")]).is_err());
        // Not a wktLiteral (plain xsd:string).
        assert!(within(&[Term::Literal(Literal::new_simple_literal("POINT(1 1)")), wkt("POINT(1 1)")]).is_err());
        // Unparsable WKT.
        assert!(within(&[wkt("PONT(1 1)"), wkt("POINT(1 1)")]).is_err());
        // distance: unit must be an IRI, and a known one.
        let distance = reg.get(&format!("{GEOF_NS}distance")).unwrap();
        assert!(distance(&[wkt("POINT(0 0)"), wkt("POINT(1 1)"), wkt("POINT(0 0)")]).is_err());
        let bogus = Term::NamedNode(NamedNode::new_unchecked("http://ex/furlong"));
        assert!(distance(&[wkt("POINT(0 0)"), wkt("POINT(1 1)"), bogus]).is_err());
    }
}
