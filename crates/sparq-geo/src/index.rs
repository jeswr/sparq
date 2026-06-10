//! [`GeoIndex`]: an R-tree over the geometries of a sparq [`Graph`].
//!
//! ## Extraction (GeoSPARQL core RDF shape)
//!
//! [`GeoIndex::build`] scans the graph's `geo:asWKT` triples. For each
//! `(?g, geo:asWKT, "..."^^geo:wktLiteral)` the indexed ENTITY is every
//! feature `?f` with `?f geo:hasGeometry ?g` or `?f geo:hasDefaultGeometry
//! ?g`; when a geometry node has no owning feature the geometry node itself
//! is the entity (datasets often attach `geo:asWKT` straight to the thing).
//! Only the default graph is scanned. Literals that fail to parse, are EMPTY,
//! carry a non-geographic CRS, or are not `geo:wktLiteral`-typed are counted
//! in [`GeoIndex::skipped`] rather than aborting the build.
//!
//! ## Index design
//!
//! Entries live in a flat `Vec<(Term, GeoGeometry)>`; the R-tree
//! (`rstar::RTree`, bulk-loaded — packed STR build, O(n log n)) stores one
//! `{entry index, AABB}` leaf per geometry, in LONG/LAT DEGREE space. Queries
//! prune by bounding box in the tree and then refine against the actual
//! geometry:
//!
//! - [`within_distance`](GeoIndex::within_distance) — converts the metre
//!   radius into a pole-safe long/lat window, walks
//!   `locate_in_envelope_intersecting`, refines with the true great-circle
//!   distance ([`geof::point_to_geometry_meters`]).
//! - [`nearest`](GeoIndex::nearest) — expanding-radius search over
//!   `within_distance` (radius ×4 per round, seeded from the tree's own
//!   extent), exact under the same great-circle metric.
//! - [`intersects`](GeoIndex::intersects) — window query on the argument's
//!   AABB refined with `geo::Intersects` (degree space).
//!
//! Longitude windows are clamped to [-180, 180]; geometries or query balls
//! CROSSING THE ANTIMERIDIAN are not wrapped in v1 (see TODO.md).

use crate::literal::GeoGeometry;
use crate::{geof, vocab};
use geo::{BoundingRect, Intersects};
use geo_types::Point;
use oxrdf::{NamedNode, Term};
use rstar::{PointDistance, RTree, RTreeObject, AABB};
use rustc_hash::FxHashMap;
use sparq_core::dict::Id;
use sparq_core::Graph;

/// Mean-Earth-radius metres per degree of latitude (same sphere as `geo`'s haversine).
const METERS_PER_DEGREE: f64 = std::f64::consts::PI * 6_371_008.8 / 180.0;
/// Half the Earth's great circle: a radius that always covers everything.
const HALF_EARTH_METERS: f64 = std::f64::consts::PI * 6_371_008.8;

/// One indexed geometry: the entity it locates plus the parsed geometry.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    /// The entity term answers are reported as: the owning feature via
    /// `geo:hasGeometry` / `geo:hasDefaultGeometry`, else the `geo:asWKT`
    /// subject itself.
    pub entity: Term,
    pub geometry: GeoGeometry,
}

/// An R-tree leaf: the entry's index plus its precomputed AABB (degree space).
struct TreeItem {
    idx: u32,
    env: AABB<[f64; 2]>,
}

impl RTreeObject for TreeItem {
    type Envelope = AABB<[f64; 2]>;
    fn envelope(&self) -> AABB<[f64; 2]> {
        self.env
    }
}

impl PointDistance for TreeItem {
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        self.env.distance_2(point)
    }
}

/// A spatial index over the geometries of a sparq [`Graph`].
pub struct GeoIndex {
    entries: Vec<Entry>,
    tree: RTree<TreeItem>,
    skipped: usize,
}

impl GeoIndex {
    /// Extracts `(entity, geometry)` pairs from the graph (see module docs for
    /// the RDF shape) and bulk-loads the R-tree. A graph with no GeoSPARQL
    /// vocabulary yields an empty (but usable) index.
    pub fn build(graph: &Graph) -> GeoIndex {
        // geometry node id -> owning feature ids (geo:hasGeometry / geo:hasDefaultGeometry).
        let mut owners: FxHashMap<Id, Vec<Id>> = FxHashMap::default();
        for pred in [vocab::HAS_GEOMETRY, vocab::HAS_DEFAULT_GEOMETRY] {
            let pred = NamedNode::new_unchecked(pred);
            if let Some(pattern) = graph.pattern(None, Some(&pred), None) {
                let scan = graph.store.scan(&pattern);
                for row in scan.rows.iter() {
                    let [s, _, o] = scan.to_spo(row);
                    let features = owners.entry(o).or_default();
                    if !features.contains(&s) {
                        features.push(s);
                    }
                }
            }
        }

        let mut entries: Vec<Entry> = Vec::new();
        let mut skipped = 0usize;
        let as_wkt = NamedNode::new_unchecked(vocab::AS_WKT);
        if let Some(pattern) = graph.pattern(None, Some(&as_wkt), None) {
            let scan = graph.store.scan(&pattern);
            for row in scan.rows.iter() {
                let [s, _, o] = scan.to_spo(row);
                // Only geo:wktLiteral-typed literals; everything else is skipped.
                let lit = match graph.dict.term(o) {
                    Term::Literal(l) if l.datatype().as_str() == vocab::WKT_LITERAL => l,
                    _ => {
                        skipped += 1;
                        continue;
                    }
                };
                let geometry = match crate::parse_wkt_literal(lit.value()) {
                    // The index computes great-circle metres: non-geographic
                    // CRSs have no defined conversion, so they are skipped
                    // (still usable via the geof:: functions directly). EMPTY
                    // geometries have no bounding box (no location) — skipped.
                    Ok(g) if g.crs.is_geographic() && g.geometry.bounding_rect().is_some() => g,
                    _ => {
                        skipped += 1;
                        continue;
                    }
                };
                match owners.get(&s) {
                    Some(features) => {
                        for &f in features {
                            entries.push(Entry {
                                entity: graph.dict.term(f),
                                geometry: geometry.clone(),
                            });
                        }
                    }
                    None => entries.push(Entry { entity: graph.dict.term(s), geometry }),
                }
            }
        }

        let items: Vec<TreeItem> = entries
            .iter()
            .enumerate()
            .map(|(i, e)| {
                // Non-empty by construction (empties were skipped above).
                let rect = e.geometry.geometry.bounding_rect().expect("non-empty geometry");
                TreeItem {
                    idx: i as u32,
                    env: AABB::from_corners([rect.min().x, rect.min().y], [rect.max().x, rect.max().y]),
                }
            })
            .collect();
        GeoIndex { entries, tree: RTree::bulk_load(items), skipped }
    }

    /// All indexed entries (one per entity/geometry pair, in extraction order).
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Number of indexed entity/geometry pairs.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// `geo:asWKT` triples that were NOT indexed: non-wktLiteral objects,
    /// unparseable WKT, or non-geographic CRSs.
    pub fn skipped(&self) -> usize {
        self.skipped
    }

    /// A pole-safe long/lat window guaranteed to contain the great-circle
    /// ball of `meters` around `center` (longitude is NOT wrapped across the
    /// antimeridian — v1 limitation).
    fn ball_window(center: Point<f64>, meters: f64) -> AABB<[f64; 2]> {
        let dlat = meters / METERS_PER_DEGREE;
        let lat_min = (center.y() - dlat).max(-90.0);
        let lat_max = (center.y() + dlat).min(90.0);
        // Longitude degrees shrink with cos(lat): size the window for the
        // WORST latitude it spans so the window is a superset of the ball.
        let max_abs_lat = lat_min.abs().max(lat_max.abs()).to_radians();
        let (lon_min, lon_max) = if lat_min <= -90.0 + 1e-12
            || lat_max >= 90.0 - 1e-12
            || max_abs_lat.cos() * METERS_PER_DEGREE * 180.0 <= meters
        {
            (-180.0, 180.0)
        } else {
            let dlon = meters / (METERS_PER_DEGREE * max_abs_lat.cos());
            ((center.x() - dlon).max(-180.0), (center.x() + dlon).min(180.0))
        };
        AABB::from_corners([lon_min, lat_min], [lon_max, lat_max])
    }

    /// All entities whose geometry lies within `meters` great-circle metres of
    /// `center` (a CRS84 long/lat point), nearest first, truncated to `limit`
    /// (when given). Distance to an extended geometry is to its (spherical)
    /// closest point.
    pub fn within_distance(
        &self,
        center: Point<f64>,
        meters: f64,
        limit: Option<usize>,
    ) -> Vec<(&Term, f64)> {
        let window = Self::ball_window(center, meters);
        let mut hits: Vec<(&Term, f64)> = self
            .tree
            .locate_in_envelope_intersecting(&window)
            .filter_map(|item| {
                let e = &self.entries[item.idx as usize];
                let d = geof::point_to_geometry_meters(center, &e.geometry.geometry).ok()?;
                (d <= meters).then_some((&e.entity, d))
            })
            .collect();
        hits.sort_by(|a, b| a.1.total_cmp(&b.1));
        if let Some(k) = limit {
            hits.truncate(k);
        }
        hits
    }

    /// The `k` entities nearest to `center` (great-circle metres, nearest
    /// first). Exact under the same metric as
    /// [`within_distance`](Self::within_distance): an
    /// expanding-radius search (×4 per round) that stops once `k` results fit
    /// inside the current radius or the radius covers the whole Earth.
    pub fn nearest(&self, center: Point<f64>, k: usize) -> Vec<(&Term, f64)> {
        if k == 0 || self.tree.size() == 0 {
            return Vec::new();
        }
        // Seed the radius from the data extent: a k/n area share of the
        // bounding box diagonal (clamped) keeps the first window useful for
        // both dense city-scale and sparse global data.
        let root = self.tree.root().envelope();
        let diag_deg = {
            let lo = root.lower();
            let hi = root.upper();
            ((hi[0] - lo[0]).powi(2) + (hi[1] - lo[1]).powi(2)).sqrt()
        };
        let share = (k as f64 / self.entries.len().max(1) as f64).sqrt();
        let mut radius = (diag_deg * share * METERS_PER_DEGREE).clamp(1.0, HALF_EARTH_METERS);
        loop {
            let hits = self.within_distance(center, radius, Some(k));
            // Complete when k results fit strictly inside the radius (the
            // (k+1)-th nearest cannot be closer than the window bound), or
            // nothing can be further away.
            if hits.len() == k || radius >= HALF_EARTH_METERS {
                return hits;
            }
            radius = (radius * 4.0).min(HALF_EARTH_METERS);
        }
    }

    /// All entities whose geometry intersects `geometry` (simple-features
    /// `sfIntersects`, evaluated in long/lat degree space). The argument must
    /// be in a geographic CRS, matching the indexed geometries.
    pub fn intersects(&self, geometry: &GeoGeometry) -> Vec<&Term> {
        let rect = match geometry.geometry.bounding_rect() {
            Some(r) => r,
            None => return Vec::new(),
        };
        if !geometry.crs.is_geographic() {
            return Vec::new();
        }
        let window = AABB::from_corners([rect.min().x, rect.min().y], [rect.max().x, rect.max().y]);
        self.tree
            .locate_in_envelope_intersecting(&window)
            .filter_map(|item| {
                let e = &self.entries[item.idx as usize];
                e.geometry.geometry.intersects(&geometry.geometry).then_some(&e.entity)
            })
            .collect()
    }

    /// Convenience: [`intersects`](Self::intersects) from a wktLiteral lexical form.
    pub fn intersects_wkt(&self, wkt_literal: &str) -> Result<Vec<&Term>, crate::GeoError> {
        Ok(self.intersects(&crate::parse_wkt_literal(wkt_literal)?))
    }
}
