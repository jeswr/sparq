//! [`GeoIndexProvider`]: a [`GeoIndex`] adapted to the engine's spatial-pushdown
//! seam (sq-mg9). Pass it to [`sparq_engine::with_spatial_index`] alongside the
//! [`geof_registry`](crate::geof_registry) so a pushable `geof:` FILTER is served by
//! the index's window/range scan instead of a per-row scan of every geometry:
//!
//! ```
//! use std::sync::Arc;
//! use sparq_core::Graph;
//! use sparq_geo::{geof_registry, GeoIndex, GeoIndexProvider};
//!
//! let g = Graph::load_str(r#"
//!     @prefix geo: <http://www.opengis.net/ont/geosparql#> .
//!     <http://ex/london> geo:asWKT "POINT(-0.1278 51.5074)"^^geo:wktLiteral .
//!     <http://ex/paris>  geo:asWKT "POINT(2.3522 48.8566)"^^geo:wktLiteral .
//! "#, "turtle").unwrap();
//!
//! let provider: Arc<dyn sparq_engine::SpatialProvider> =
//!     Arc::new(GeoIndexProvider::new(GeoIndex::build(&g)));
//! let reg = geof_registry();
//! let r = sparq_engine::with_spatial_index(provider, || {
//!     sparq_engine::with_functions(&reg, || sparq_engine::query(&g,
//!         "PREFIX geo:  <http://www.opengis.net/ont/geosparql#>
//!          PREFIX geof: <http://www.opengis.net/def/function/geosparql/>
//!          PREFIX uom:  <http://www.opengis.net/def/uom/OGC/1.0/>
//!          SELECT ?s WHERE {
//!            ?s geo:asWKT ?g .
//!            FILTER(geof:distance(?g, \"POINT(-0.12 51.5)\"^^geo:wktLiteral, uom:kilometre) < 50)
//!          }"))
//! }).unwrap();
//! assert_eq!(r.len(), 1); // only London is within 50 km
//! ```
//!
//! The pushdown is a candidate SUPERSET refined by the exact `geof:` FILTER (which
//! the engine still applies), so results are IDENTICAL to running without the index
//! — only fewer geometries are exact-checked.

use crate::geof::Unit;
use crate::GeoIndex;
use geo_types::{Geometry, Point};
use oxrdf::Term;
use rustc_hash::FxHashSet;
use sparq_engine::{SpatialProvider, SpatialQuery};

/// A [`GeoIndex`] wrapped as a [`SpatialProvider`] for the engine's spatial
/// FILTER pushdown. Cheap to share (`Arc` it once, reuse across queries/threads);
/// rebuild / [`GeoIndex::apply_delta`] the inner index on data change and wrap
/// afresh. [OPUS-4.8]
pub struct GeoIndexProvider {
    index: GeoIndex,
    /// The set of geometry LITERALS the index holds — the universe it is
    /// authoritative over. The engine consults this (via
    /// [`SpatialProvider::is_indexed`]) so a binding the index never saw (bound
    /// via a non-`geo:asWKT` predicate, a non-geographic CRS, …) is left for the
    /// exact `geof:` FILTER instead of being silently dropped. Built once.
    indexed: FxHashSet<Term>,
}

impl GeoIndexProvider {
    pub fn new(index: GeoIndex) -> Self {
        let indexed = index.entries().map(|e| e.literal.clone()).collect();
        Self { index, indexed }
    }

    /// Borrow the wrapped index (e.g. for the non-pushdown query methods).
    pub fn index(&self) -> &GeoIndex {
        &self.index
    }
}

/// The single CRS84 long/lat point a distance query measures from — `Some` only
/// when the constant geometry parses to a single `POINT` in a geographic CRS
/// (the only shape `GeoIndex::within_distance_literals` accepts as its centre).
fn center_point(wkt: &str) -> Option<Point<f64>> {
    let g = crate::parse_wkt_literal(wkt).ok()?;
    if !g.crs.is_geographic() {
        return None;
    }
    match g.geometry {
        Geometry::Point(p) => Some(p),
        _ => None,
    }
}

impl SpatialProvider for GeoIndexProvider {
    fn candidates(&self, query: &SpatialQuery) -> Option<Vec<Term>> {
        match query {
            SpatialQuery::DistanceWithin { point_wkt, radius, unit_iri, inclusive: _ } => {
                // Only METRIC distance (metre/km/mile) shares the index's great-circle
                // metric; degree/radian distance is euclidean coordinate distance — a
                // different metric the metre-window index cannot bound — so decline it
                // (it stays post-hoc). `inclusive` is irrelevant to a superset: the
                // index uses `<= meters` (the boundary ring is at worst a false
                // positive the residual FILTER removes).
                let scale = Unit::from_iri(unit_iri).ok()?.meters_scale()?;
                let center = center_point(point_wkt)?;
                let meters = radius * scale;
                Some(self.index.within_distance_literals(center, meters))
            }
            SpatialQuery::BboxIntersects { arg_wkt } => {
                let g = crate::parse_wkt_literal(arg_wkt).ok()?;
                // A non-geographic CONSTANT shares no metric with the index's geographic
                // entries: DECLINE (None) so the post-hoc path runs, rather than return an
                // empty candidate set that would wrongly drop same-CRS matches the exact
                // `geof:` check would keep. (is_indexed already protects non-geographic
                // VARIABLE bindings; this guards the constant side.)
                if !g.crs.is_geographic() {
                    return None;
                }
                Some(self.index.bbox_candidate_literals(&g))
            }
        }
    }

    fn is_indexed(&self, term: &Term) -> bool {
        self.indexed.contains(term)
    }
}
