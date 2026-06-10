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

/// A [`crate::geof::lex`] binary relation (`sf_*`).
type LexRelation = fn(&str, &str) -> Result<bool, GeoError>;
/// A [`crate::geof::lex`] unary geometry function (`envelope` / `boundary` / `convex_hull`).
type LexUnary = fn(&str) -> Result<String, GeoError>;

/// The `geof:` extension functions as a sparq-engine [`FunctionRegistry`] — pass it
/// to [`sparq_engine::query_with_functions`] (or scope any other entry point with
/// [`sparq_engine::with_functions`]) to evaluate GeoSPARQL functions inside SPARQL.
///
/// Registered IRIs (all under `http://www.opengis.net/def/function/geosparql/`):
/// `distance`, `sfEquals`, `sfDisjoint`, `sfIntersects`, `sfTouches`, `sfCrosses`,
/// `sfWithin`, `sfContains`, `sfOverlaps`, `envelope`, `boundary`, `convexHull`.
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
        let Term::NamedNode(unit) = &args[2] else {
            return Err(format!("geof:distance: argument 3 must be a unit-of-measure IRI, got {}", args[2]));
        };
        let d = lex::distance(a, b, unit.as_str()).map_err(|e| e.to_string())?;
        Ok(Term::Literal(Literal::from(d)))
    });

    // The eight simple-features relations: geof:sf*(?g1, ?g2) -> xsd:boolean.
    let relations: [(&'static str, LexRelation); 8] = [
        ("sfEquals", lex::sf_equals),
        ("sfDisjoint", lex::sf_disjoint),
        ("sfIntersects", lex::sf_intersects),
        ("sfTouches", lex::sf_touches),
        ("sfCrosses", lex::sf_crosses),
        ("sfWithin", lex::sf_within),
        ("sfContains", lex::sf_contains),
        ("sfOverlaps", lex::sf_overlaps),
    ];
    for (name, f) in relations {
        reg.register(format!("{GEOF_NS}{name}"), move |args: &[Term]| {
            arity(name, args, 2)?;
            let v = f(wkt_lex(name, args, 0)?, wkt_lex(name, args, 1)?).map_err(|e| e.to_string())?;
            Ok(Term::Literal(Literal::from(v)))
        });
    }

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
        assert_eq!(reg.len(), 12);
        for name in [
            "distance", "sfEquals", "sfDisjoint", "sfIntersects", "sfTouches", "sfCrosses",
            "sfWithin", "sfContains", "sfOverlaps", "envelope", "boundary", "convexHull",
        ] {
            assert!(reg.get(&format!("{GEOF_NS}{name}")).is_some(), "missing geof:{name}");
        }
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
