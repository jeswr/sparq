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
//! Argument/result conventions (each function has the same semantics as its
//! [`crate::geof::lex`] mirror):
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
//!
//! # Parsed-geometry caching (sq-lkrgi) [FABLE-5]
//!
//! The registry evaluates through the TYPED [`crate::geof`] API over a small
//! per-thread cache of parsed geometries keyed by (datatype, full lexical form)
//! — see `geom_cache` — instead of the string-level `geof::lex` mirrors, which
//! re-parse every argument on every row. A CONSTANT geometry in a `FILTER`
//! (Geographica's large-constant selections pass a ~100 KB polygon literal per
//! row) is therefore parsed once per thread, not once per row. This is a pure
//! evaluation-strategy change: a lexical form parses to the same geometry every
//! time (no graph state is involved), so results are identical to fresh parses;
//! the differential unit tests below pin that. Parse FAILURES are not cached —
//! an erroneous literal re-errors per row, exactly as before.

use crate::geof::{self, Unit};
use crate::literal::GeoGeometry;
use crate::vocab::{GEOF_NS, GML_LITERAL, WKT_LITERAL};
use crate::GeoError;
use oxrdf::{Literal, NamedNode, Term};
use sparq_engine::FunctionRegistry;
use std::rc::Rc;

/// `Err` unless exactly `n` arguments were passed.
fn arity(name: &str, args: &[Term], n: usize) -> Result<(), String> {
    if args.len() == n {
        Ok(())
    } else {
        Err(format!("geof:{name} expects {n} argument(s), got {}", args.len()))
    }
}

/// Bounded per-thread cache of parsed geometry literals (sq-lkrgi). [FABLE-5]
///
/// # Why serving a cached parse is SOUND (the load-bearing property)
///
/// The cached value is `parse_wkt_literal(lex)` / `parse_gml_literal(lex)` for
/// the EXACT lexical form `lex` — a deterministic pure function of the string
/// alone (no graph, dictionary, or session state feeds the parse), so an entry
/// can never go stale and no invalidation is needed. The two parsers are keyed
/// separately ([`LexKind`]) so a lexical form cached under one datatype can
/// never satisfy a lookup for the other, and parse FAILURES are never cached
/// (each erroneous row re-errors, preserving per-row expression-error
/// semantics). The cache is per-thread (`thread_local!`, matching the
/// `sparq-vectors` mask-cache idiom) so the registry closures stay
/// `Send + Sync` with no locking, and it is bounded both by entry count and by
/// total key bytes so a long-lived process cannot grow it without bound.
mod geom_cache {
    use super::{GeoError, GeoGeometry};
    use std::cell::RefCell;
    use std::rc::Rc;

    /// Which parser a cache entry belongs to. WKT and GML lexical spaces are
    /// keyed separately: the same string means different things to the two
    /// parsers, so a hit must never cross datatypes.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(super) enum LexKind {
        /// `geo:wktLiteral` — `crate::literal::parse_wkt_literal`.
        Wkt,
        /// `geo:gmlLiteral` — `crate::gml::parse_gml_literal`.
        Gml,
    }

    /// One cached parse: the key (parser + full lexical form) and the shared
    /// parsed geometry.
    struct CachedGeom {
        kind: LexKind,
        lex: String,
        geom: Rc<GeoGeometry>,
    }

    /// Entry cap. The hot shape is ONE constant + a stream of per-row
    /// geometries; a handful of slots keeps every constant of a realistic
    /// expression resident while per-row strings churn through the tail.
    pub(super) const MAX_ENTRIES: usize = 16;
    /// Total-key-bytes cap (the parsed geometry's size is roughly proportional
    /// to its lexical form, so key bytes are the memory proxy). Bounds the
    /// per-thread footprint even when every entry is a large polygon.
    pub(super) const MAX_KEY_BYTES: usize = 1 << 20;

    thread_local! {
        /// MRU-ordered (front = most recently used) cache. A `Vec` scanned
        /// linearly: at ≤ `MAX_ENTRIES` entries a scan is a few length checks
        /// plus one confirming memcmp on the hit, far below hash-map noise —
        /// and orders of magnitude below re-parsing a large WKT body.
        static CACHE: RefCell<Vec<CachedGeom>> = const { RefCell::new(Vec::new()) };
    }

    #[cfg(test)]
    thread_local! {
        /// Test-only count of parse ATTEMPTS (misses), successful or not.
        /// Lets tests assert a constant really was parsed once, not per row.
        static PARSE_ATTEMPTS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    }

    /// The parsed geometry for `lex` under `kind`: a cache hit when this exact
    /// (kind, lexical form) was parsed on this thread recently, a fresh parse
    /// otherwise. Errors pass through uncached.
    pub(super) fn parse(kind: LexKind, lex: &str) -> Result<Rc<GeoGeometry>, GeoError> {
        // Lookup phase (borrow released before any parsing runs).
        let hit = CACHE.with(|cell| {
            let mut cache = cell.borrow_mut();
            cache.iter().position(|e| e.kind == kind && e.lex == lex).map(|i| {
                if i > 0 {
                    let entry = cache.remove(i);
                    cache.insert(0, entry);
                }
                cache[0].geom.clone()
            })
        });
        if let Some(geom) = hit {
            return Ok(geom);
        }
        #[cfg(test)]
        PARSE_ATTEMPTS.with(|c| c.set(c.get() + 1));
        let geom = Rc::new(match kind {
            LexKind::Wkt => crate::literal::parse_wkt_literal(lex)?,
            LexKind::Gml => crate::gml::parse_gml_literal(lex)?,
        });
        CACHE.with(|cell| {
            let mut cache = cell.borrow_mut();
            cache.insert(0, CachedGeom { kind, lex: lex.to_string(), geom: geom.clone() });
            // Evict from the LRU tail past either cap — but never the
            // just-inserted MRU entry, so even a single over-budget constant
            // stays cached (evicting it would re-parse it every row, the exact
            // pathology this cache removes).
            while cache.len() > 1
                && (cache.len() > MAX_ENTRIES
                    || cache.iter().map(|e| e.lex.len()).sum::<usize>() > MAX_KEY_BYTES)
            {
                cache.pop();
            }
        });
        Ok(geom)
    }

    /// Test-only: drop all entries and zero the attempt counter so a test
    /// starts from a known-cold cache.
    #[cfg(test)]
    pub(super) fn reset() {
        CACHE.with(|c| c.borrow_mut().clear());
        PARSE_ATTEMPTS.with(|c| c.set(0));
    }

    /// Test-only: number of live entries.
    #[cfg(test)]
    pub(super) fn len() -> usize {
        CACHE.with(|c| c.borrow().len())
    }

    /// Test-only: parse attempts since the last [`reset`].
    #[cfg(test)]
    pub(super) fn parse_attempts() -> usize {
        PARSE_ATTEMPTS.with(|c| c.get())
    }
}

/// The PARSED geometry of argument `i`, through the per-thread `geom_cache`
/// (so a constant geometry repeated across rows is parsed once per thread,
/// not once per row — sq-lkrgi). Both GeoSPARQL serializations are accepted
/// (§8.5): a `geo:wktLiteral` or a `geo:gmlLiteral`, each keyed under its own
/// parser. Any other term is a per-row expression error. [FABLE-5]
fn geom_arg(name: &str, args: &[Term], i: usize) -> Result<Rc<GeoGeometry>, String> {
    match &args[i] {
        Term::Literal(l) if l.datatype().as_str() == WKT_LITERAL => {
            geom_cache::parse(geom_cache::LexKind::Wkt, l.value()).map_err(|e| e.to_string())
        }
        Term::Literal(l) if l.datatype().as_str() == GML_LITERAL => {
            geom_cache::parse(geom_cache::LexKind::Gml, l.value()).map_err(|e| e.to_string())
        }
        other => Err(format!(
            "geof:{name}: argument {} must be a geo:wktLiteral or geo:gmlLiteral literal, got {other}",
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

/// A [`crate::geof`] binary relation (`sf_*` / `eh_*` / `rcc8_*`).
type GeomRelation = fn(&GeoGeometry, &GeoGeometry) -> Result<bool, GeoError>;
/// A [`crate::geof`] unary geometry function (`envelope` / `boundary` / `convex_hull`).
type GeomUnary = fn(&GeoGeometry) -> Result<GeoGeometry, GeoError>;
/// A [`crate::geof`] binary set operation (`intersection` / `union` / …).
type GeomSetOp = fn(&GeoGeometry, &GeoGeometry) -> Result<GeoGeometry, GeoError>;

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
///   `geo:wktLiteral` (point-set ops: polygon overlay plus the well-defined
///   line/point cases — see `geof` for the supported matrix);
/// * `getSRID(g)` -> `xsd:anyURI` (the geometry's CRS IRI).
///
/// Build it once and reuse it: the registry is cheaply cloneable and `Send + Sync`,
/// so one instance can serve every query on every thread for the process lifetime.
///
/// Geometry arguments are parsed through a small bounded PER-THREAD cache keyed
/// by the literal's exact lexical form (sq-lkrgi), so a constant geometry in a
/// `FILTER` — even a ~100 KB polygon — is parsed once per thread rather than
/// once per row. Results are identical to fresh parses (the parse is a pure
/// function of the lexical form); parse failures are never cached. [FABLE-5]
pub fn geof_registry() -> FunctionRegistry {
    let mut reg = FunctionRegistry::new();

    // geof:distance(?g1, ?g2, ?unitIri) -> xsd:double.
    reg.register(format!("{GEOF_NS}distance"), |args: &[Term]| {
        arity("distance", args, 3)?;
        let a = geom_arg("distance", args, 0)?;
        let b = geom_arg("distance", args, 1)?;
        let unit = Unit::from_iri(unit_iri("distance", args, 2)?).map_err(|e| e.to_string())?;
        let d = geof::distance(&a, &b, unit).map_err(|e| e.to_string())?;
        Ok(Term::Literal(Literal::from(d)))
    });

    // The relation families: geof:sf* / geof:eh* / geof:rcc8*(?g1, ?g2) -> xsd:boolean.
    let relations: [(&'static str, GeomRelation); 24] = [
        // Simple features (GeoSPARQL Req 22-24).
        ("sfEquals", geof::sf_equals),
        ("sfDisjoint", geof::sf_disjoint),
        ("sfIntersects", geof::sf_intersects),
        ("sfTouches", geof::sf_touches),
        ("sfCrosses", geof::sf_crosses),
        ("sfWithin", geof::sf_within),
        ("sfContains", geof::sf_contains),
        ("sfOverlaps", geof::sf_overlaps),
        // Egenhofer (Req 25).
        ("ehEquals", geof::eh_equals),
        ("ehDisjoint", geof::eh_disjoint),
        ("ehMeet", geof::eh_meet),
        ("ehOverlap", geof::eh_overlap),
        ("ehCovers", geof::eh_covers),
        ("ehCoveredBy", geof::eh_covered_by),
        ("ehInside", geof::eh_inside),
        ("ehContains", geof::eh_contains),
        // RCC8 (Req 26).
        ("rcc8eq", geof::rcc8_eq),
        ("rcc8dc", geof::rcc8_dc),
        ("rcc8ec", geof::rcc8_ec),
        ("rcc8po", geof::rcc8_po),
        ("rcc8tppi", geof::rcc8_tppi),
        ("rcc8tpp", geof::rcc8_tpp),
        ("rcc8ntpp", geof::rcc8_ntpp),
        ("rcc8ntppi", geof::rcc8_ntppi),
    ];
    for (name, f) in relations {
        reg.register(format!("{GEOF_NS}{name}"), move |args: &[Term]| {
            arity(name, args, 2)?;
            let a = geom_arg(name, args, 0)?;
            let b = geom_arg(name, args, 1)?;
            let v = f(&a, &b).map_err(|e| e.to_string())?;
            Ok(Term::Literal(Literal::from(v)))
        });
    }

    // geof:relate(?g1, ?g2, ?de9imPattern) -> xsd:boolean (generic DE-9IM test).
    reg.register(format!("{GEOF_NS}relate"), |args: &[Term]| {
        arity("relate", args, 3)?;
        let a = geom_arg("relate", args, 0)?;
        let b = geom_arg("relate", args, 1)?;
        let pattern = match &args[2] {
            Term::Literal(l) => l.value(),
            other => {
                return Err(format!(
                    "geof:relate: argument 3 must be a DE-9IM pattern string, got {other}"
                ))
            }
        };
        let v = geof::relate(&a, &b, pattern).map_err(|e| e.to_string())?;
        Ok(Term::Literal(Literal::from(v)))
    });

    // The unary geometry functions: geof:*(?g) -> geo:wktLiteral.
    let unary: [(&'static str, GeomUnary); 3] = [
        ("envelope", geof::envelope),
        ("boundary", geof::boundary),
        ("convexHull", geof::convex_hull),
    ];
    for (name, f) in unary {
        reg.register(format!("{GEOF_NS}{name}"), move |args: &[Term]| {
            arity(name, args, 1)?;
            let g = geom_arg(name, args, 0)?;
            Ok(wkt_term(f(&g).map_err(|e| e.to_string())?.to_wkt_literal()))
        });
    }

    // The set operations: geof:*(?g1, ?g2) -> geo:wktLiteral (point-set ops over
    // polygon/line/point operands; see geof for the supported matrix).
    let set_ops: [(&'static str, GeomSetOp); 4] = [
        ("intersection", geof::intersection),
        ("union", geof::union),
        ("difference", geof::difference),
        ("symDifference", geof::sym_difference),
    ];
    for (name, f) in set_ops {
        reg.register(format!("{GEOF_NS}{name}"), move |args: &[Term]| {
            arity(name, args, 2)?;
            let a = geom_arg(name, args, 0)?;
            let b = geom_arg(name, args, 1)?;
            Ok(wkt_term(f(&a, &b).map_err(|e| e.to_string())?.to_wkt_literal()))
        });
    }

    // geof:buffer(?g, ?radius, ?unitIri) -> geo:wktLiteral (a MULTIPOLYGON).
    reg.register(format!("{GEOF_NS}buffer"), |args: &[Term]| {
        arity("buffer", args, 3)?;
        let g = geom_arg("buffer", args, 0)?;
        let radius = num_arg("buffer", args, 1)?;
        let unit = Unit::from_iri(unit_iri("buffer", args, 2)?).map_err(|e| e.to_string())?;
        Ok(wkt_term(geof::buffer(&g, radius, unit).map_err(|e| e.to_string())?.to_wkt_literal()))
    });

    // geof:getSRID(?g) -> xsd:anyURI (the geometry's CRS IRI).
    reg.register(format!("{GEOF_NS}getSRID"), |args: &[Term]| {
        arity("getSRID", args, 1)?;
        let iri = geom_arg("getSRID", args, 0)?.crs.iri().to_string();
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
        // Line/point operands are now supported (sq-gn3): a point inside the
        // polygon survives; a point outside clips to an empty MULTIPOINT.
        let Term::Literal(inside) = inter(&[wkt("POINT(2 2)"), b.clone()]).unwrap() else {
            panic!("literal")
        };
        assert!(inside.value().contains('2'), "got {}", inside.value());
        let Term::Literal(outside) = inter(&[wkt("POINT(0 0)"), b.clone()]).unwrap() else {
            panic!("literal")
        };
        assert!(
            outside.value().contains("EMPTY") || outside.value().contains("()"),
            "got {}",
            outside.value()
        );
        // 1-D set subtraction is now supported (sq-fxv3): line − line removes
        // the collinear-overlapping span, leaving LINESTRING(0 0, 1 0). [OPUS-4.8]
        let diff = reg.get(&format!("{GEOF_NS}difference")).unwrap();
        let Term::Literal(d) =
            diff(&[wkt("LINESTRING(0 0, 2 0)"), wkt("LINESTRING(1 0, 3 0)")]).unwrap()
        else {
            panic!("literal")
        };
        assert_eq!(d.datatype().as_str(), WKT_LITERAL);
        assert!(d.value().contains("LINESTRING") && d.value().contains('1'), "got {}", d.value());
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

    // ---- the parsed-geometry cache (sq-lkrgi) [FABLE-5] -----------------------------
    //
    // Each test runs on its own thread (the default test harness), so the
    // thread-local cache and attempt counter are naturally isolated; `reset()`
    // makes the cold start explicit.

    /// A syntactically valid ring POLYGON with `vertices` distinct vertices
    /// (radius-2 circle about the origin) — a scaled-down stand-in for the
    /// Geographica large-constant polygon.
    fn ring_polygon(vertices: usize) -> String {
        let mut s = String::from("POLYGON((");
        for i in 0..=vertices {
            let theta = (i % vertices) as f64 / vertices as f64 * std::f64::consts::TAU;
            if i > 0 {
                s.push_str(", ");
            }
            s.push_str(&format!("{} {}", 2.0 * theta.cos(), 2.0 * theta.sin()));
        }
        s.push_str("))");
        s
    }

    /// A syntactically valid LINESTRING whose lexical form exceeds `min_len` bytes.
    fn big_linestring(min_len: usize) -> String {
        let mut s = String::with_capacity(min_len + 32);
        s.push_str("LINESTRING(0 0");
        let mut i = 1usize;
        while s.len() < min_len {
            s.push_str(&format!(", {} {}", i, i % 7));
            i += 1;
        }
        s.push(')');
        s
    }

    #[test]
    fn constant_geometry_is_parsed_once_across_rows() {
        geom_cache::reset();
        let reg = geof_registry();
        let within = reg.get(&format!("{GEOF_NS}sfWithin")).unwrap();
        // One constant polygon FILTERed against a stream of distinct per-row
        // points — the Geographica q09/q13/q16/q17 shape.
        let constant = wkt(&ring_polygon(512));
        let rows = 50;
        for i in 0..rows {
            let p = wkt(&format!("POINT({} 0.0)", i as f64 * 0.001));
            assert_eq!(
                within(&[p, constant.clone()]).unwrap().to_string(),
                "\"true\"^^<http://www.w3.org/2001/XMLSchema#boolean>"
            );
        }
        // `rows` distinct per-row points + ONE parse of the constant. Without
        // the cache this is `2 * rows` (the constant re-parsed every row).
        assert_eq!(geom_cache::parse_attempts(), rows + 1);
    }

    #[test]
    fn cached_parse_equals_fresh_parse() {
        // The load-bearing equivalence: a cache HIT returns exactly what a
        // fresh parse returns (incl. CRS handling and EPSG:4326 axis swap).
        geom_cache::reset();
        for lex in [
            "POINT(-0.1278 51.5074)",
            "<http://www.opengis.net/def/crs/EPSG/0/4326> POINT(51.5074 -0.1278)",
            "<http://www.opengis.net/def/crs/EPSG/0/27700> LINESTRING(530000 180000, 531000 181000)",
            "POLYGON((0 0, 4 0, 4 4, 0 4, 0 0))",
        ] {
            let cold = geom_cache::parse(geom_cache::LexKind::Wkt, lex).unwrap();
            let warm = geom_cache::parse(geom_cache::LexKind::Wkt, lex).unwrap();
            let fresh = crate::literal::parse_wkt_literal(lex).unwrap();
            assert_eq!(*cold, fresh, "cold parse must equal a fresh parse for {}", lex);
            assert!(Rc::ptr_eq(&cold, &warm), "second lookup must be a cache hit for {}", lex);
            assert_eq!(*warm, fresh, "cached parse must equal a fresh parse for {}", lex);
        }
        // Same equivalence for the GML parser under its own key space.
        let gml = "<gml:Point><gml:pos>-83.38 33.95</gml:pos></gml:Point>";
        let cold = geom_cache::parse(geom_cache::LexKind::Gml, gml).unwrap();
        let warm = geom_cache::parse(geom_cache::LexKind::Gml, gml).unwrap();
        let fresh = crate::gml::parse_gml_literal(gml).unwrap();
        assert_eq!(*cold, fresh);
        assert!(Rc::ptr_eq(&cold, &warm));
    }

    #[test]
    fn wkt_and_gml_cache_keys_do_not_collide() {
        geom_cache::reset();
        // Cache a string under the WKT parser…
        let kept = geom_cache::parse(geom_cache::LexKind::Wkt, "POINT(1 2)").unwrap();
        // …then look the SAME string up under the GML kind: it must NOT serve
        // the WKT entry — it must attempt (and fail) a GML parse of non-XML.
        assert!(geom_cache::parse(geom_cache::LexKind::Gml, "POINT(1 2)").is_err());
        assert_eq!(geom_cache::parse_attempts(), 2);
        drop(kept);
    }

    #[test]
    fn cache_entry_count_is_bounded_and_mru_survives() {
        geom_cache::reset();
        for i in 0..(geom_cache::MAX_ENTRIES + 8) {
            geom_cache::parse(geom_cache::LexKind::Wkt, &format!("POINT({} 0)", i)).unwrap();
        }
        assert!(geom_cache::len() <= geom_cache::MAX_ENTRIES);
        // The most recently used entry is still resident (no new attempt)…
        let attempts = geom_cache::parse_attempts();
        let last = format!("POINT({} 0)", geom_cache::MAX_ENTRIES + 7);
        geom_cache::parse(geom_cache::LexKind::Wkt, &last).unwrap();
        assert_eq!(geom_cache::parse_attempts(), attempts);
        // …while the oldest was evicted (a lookup re-parses it).
        geom_cache::parse(geom_cache::LexKind::Wkt, "POINT(0 0)").unwrap();
        assert_eq!(geom_cache::parse_attempts(), attempts + 1);
    }

    #[test]
    fn byte_cap_evicts_tail_but_never_the_mru_entry() {
        geom_cache::reset();
        // A single constant LARGER than the whole byte budget still caches —
        // the MRU entry is never evicted (evicting it would re-parse it every
        // row, the exact pathology the cache removes).
        let big = big_linestring(geom_cache::MAX_KEY_BYTES + 1024);
        geom_cache::parse(geom_cache::LexKind::Wkt, &big).unwrap();
        assert_eq!(geom_cache::len(), 1);
        geom_cache::parse(geom_cache::LexKind::Wkt, &big).unwrap();
        assert_eq!(geom_cache::parse_attempts(), 1, "over-budget constant must still cache");
        // A subsequent small insert pushes the over-budget giant out of the tail…
        geom_cache::parse(geom_cache::LexKind::Wkt, "POINT(3 4)").unwrap();
        assert_eq!(geom_cache::len(), 1, "byte cap must evict the giant tail entry");
        // …and the small MRU entry is the survivor.
        geom_cache::parse(geom_cache::LexKind::Wkt, "POINT(3 4)").unwrap();
        assert_eq!(geom_cache::parse_attempts(), 2);
    }

    #[test]
    fn parse_failures_are_not_cached() {
        geom_cache::reset();
        assert!(geom_cache::parse(geom_cache::LexKind::Wkt, "PONT(1 1)").is_err());
        assert!(geom_cache::parse(geom_cache::LexKind::Wkt, "PONT(1 1)").is_err());
        // Each erroneous row re-attempts (and re-errors); nothing is stored.
        assert_eq!(geom_cache::parse_attempts(), 2);
        assert_eq!(geom_cache::len(), 0);
    }
}
