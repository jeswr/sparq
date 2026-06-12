//! [`GeoIndex`]: an R-tree over the geometries of a sparq [`Graph`].
//!
//! ## Extraction (GeoSPARQL core RDF shape)
//!
//! [`GeoIndex::build`] scans the graph's `geo:asWKT` triples — in the DEFAULT
//! graph and in every NAMED graph ([`Graph::named`]). For each
//! `(?g, geo:asWKT, "..."^^geo:wktLiteral)` the indexed ENTITY is every
//! feature `?f` with `?f geo:hasGeometry ?g` or `?f geo:hasDefaultGeometry
//! ?g` (resolved within the same graph); when a geometry node has no owning
//! feature the geometry node itself is the entity (datasets often attach
//! `geo:asWKT` straight to the thing). Each [`Entry`] records which graph it
//! came from ([`Entry::graph`], `None` for the default graph) and its
//! `geo:asWKT` subject ([`Entry::node`]). Literals that fail to parse, are
//! EMPTY, carry a non-geographic CRS, or are not `geo:wktLiteral`-typed are
//! counted in [`GeoIndex::skipped`] rather than aborting the build.
//!
//! ## Index design
//!
//! Entries live in a slotted `Vec` (slots are tombstoned by incremental
//! deletes and reused by inserts); the R-tree (`rstar::RTree`, bulk-loaded —
//! packed STR build, O(n log n)) stores one `{entry slot, AABB}` leaf per
//! geometry, in LONG/LAT DEGREE space. Queries prune by bounding box in the
//! tree and then refine against the actual geometry:
//!
//! - [`within_distance`](GeoIndex::within_distance) — converts the metre
//!   radius into one or two pole-safe long/lat windows (a ball CROSSING THE
//!   ANTIMERIDIAN is split into two windows, one per side of ±180°), walks
//!   `locate_in_envelope_intersecting` per window, refines with the true
//!   great-circle distance ([`geof::point_to_geometry_meters`]).
//! - [`nearest`](GeoIndex::nearest) — expanding-radius search over
//!   `within_distance` (radius ×4 per round, seeded from the tree's own
//!   extent), exact under the same great-circle metric (and antimeridian-safe
//!   through `within_distance`).
//! - [`intersects`](GeoIndex::intersects) — window query on the argument's
//!   AABB refined with `geo::Intersects` (degree space). The argument's OWN
//!   coordinates are taken as written: a geometry whose WKT runs past ±180°
//!   is not wrapped (the refinement is planar in degree space either way).
//!
//! ## Incremental maintenance
//!
//! [`apply_delta`](GeoIndex::apply_delta) mirrors a [`Graph::apply_delta`]
//! batch into the index without a rebuild: the geometry nodes touched by the
//! batch are re-extracted from the (post-delta) graph, their old entries
//! removed from the R-tree (`rstar` incremental remove) and the new state
//! inserted (`rstar` incremental insert) — O(batch × log n) instead of the
//! O(n log n) rebuild. Deltas apply to the DEFAULT graph (sparq's
//! `Graph::apply_delta` semantics); named-graph entries are untouched.

use crate::literal::GeoGeometry;
use crate::{geof, vocab};
use geo::{BoundingRect, Intersects};
use geo_types::Point;
use oxrdf::{NamedNode, Term};
use rstar::{Envelope, PointDistance, RTree, RTreeObject, SelectionFunction, AABB};
use rustc_hash::{FxHashMap, FxHashSet};
use sparq_core::dict::Id;
use sparq_core::Graph;

/// Mean-Earth-radius metres per degree of latitude (same sphere as `geo`'s haversine).
const METERS_PER_DEGREE: f64 = std::f64::consts::PI * 6_371_008.8 / 180.0;
/// Half the Earth's great circle: a radius that always covers everything.
const HALF_EARTH_METERS: f64 = std::f64::consts::PI * 6_371_008.8;

/// One indexed geometry: the entity it locates plus the parsed geometry and
/// where it was extracted from.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    /// The entity term answers are reported as: the owning feature via
    /// `geo:hasGeometry` / `geo:hasDefaultGeometry`, else the `geo:asWKT`
    /// subject itself.
    pub entity: Term,
    /// The `geo:asWKT` subject this entry was extracted from (the geometry
    /// node; equal to `entity` when the feature carries `geo:asWKT` directly).
    pub node: Term,
    /// The named graph the entry came from; `None` for the default graph.
    pub graph: Option<Term>,
    pub geometry: GeoGeometry,
}

/// An R-tree leaf: the entry's slot plus its precomputed AABB (degree space).
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

/// Selects exactly the leaf with the given slot (descending only through
/// parents that can contain its envelope) — the removal cursor for
/// incremental maintenance.
struct SelectSlot {
    idx: u32,
    env: AABB<[f64; 2]>,
}

impl SelectionFunction<TreeItem> for SelectSlot {
    fn should_unpack_parent(&self, parent: &AABB<[f64; 2]>) -> bool {
        parent.contains_envelope(&self.env)
    }
    fn should_unpack_leaf(&self, leaf: &TreeItem) -> bool {
        leaf.idx == self.idx
    }
}

/// A spatial index over the geometries of a sparq [`Graph`].
pub struct GeoIndex {
    /// Entry slots; `None` slots were removed by a delta and may be reused.
    slots: Vec<Option<Entry>>,
    /// Reusable (tombstoned) slot indices.
    free: Vec<u32>,
    tree: RTree<TreeItem>,
    skipped: usize,
}

/// The `(entity, geometry)` rows extracted from ONE graph's `geo:asWKT`
/// scan, plus the number of skipped literals.
fn extract_graph(graph: &Graph, graph_name: Option<&Term>) -> (Vec<Entry>, usize) {
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
            let node = graph.dict.term(s);
            match owners.get(&s) {
                Some(features) => {
                    for &f in features {
                        entries.push(Entry {
                            entity: graph.dict.term(f),
                            node: node.clone(),
                            graph: graph_name.cloned(),
                            geometry: geometry.clone(),
                        });
                    }
                }
                None => entries.push(Entry {
                    entity: node.clone(),
                    node,
                    graph: graph_name.cloned(),
                    geometry,
                }),
            }
        }
    }
    (entries, skipped)
}

/// The degree-space AABB of a (non-empty) geometry.
fn geometry_env(g: &GeoGeometry) -> AABB<[f64; 2]> {
    // Non-empty by construction (empties are skipped during extraction).
    let rect = g.geometry.bounding_rect().expect("non-empty geometry");
    AABB::from_corners([rect.min().x, rect.min().y], [rect.max().x, rect.max().y])
}

impl GeoIndex {
    /// Extracts `(entity, geometry)` pairs from the graph — default graph plus
    /// every named graph (see module docs for the RDF shape) — and bulk-loads
    /// the R-tree. A graph with no GeoSPARQL vocabulary yields an empty (but
    /// usable) index.
    pub fn build(graph: &Graph) -> GeoIndex {
        let (mut entries, mut skipped) = extract_graph(graph, None);
        for (name, g) in &graph.named {
            let (more, more_skipped) = extract_graph(g, Some(name));
            entries.extend(more);
            skipped += more_skipped;
        }

        let items: Vec<TreeItem> = entries
            .iter()
            .enumerate()
            .map(|(i, e)| TreeItem { idx: i as u32, env: geometry_env(&e.geometry) })
            .collect();
        GeoIndex {
            slots: entries.into_iter().map(Some).collect(),
            free: Vec::new(),
            tree: RTree::bulk_load(items),
            skipped,
        }
    }

    /// All indexed entries (one per entity/geometry pair; extraction order on
    /// a fresh build, arbitrary after incremental updates).
    pub fn entries(&self) -> impl Iterator<Item = &Entry> {
        self.slots.iter().filter_map(|s| s.as_ref())
    }

    /// Number of indexed entity/geometry pairs.
    pub fn len(&self) -> usize {
        self.tree.size()
    }

    pub fn is_empty(&self) -> bool {
        self.tree.size() == 0
    }

    /// `geo:asWKT` triples that were NOT indexed (cumulative over the build
    /// and any [`apply_delta`](Self::apply_delta) calls): non-wktLiteral
    /// objects, unparseable WKT, or non-geographic CRSs.
    pub fn skipped(&self) -> usize {
        self.skipped
    }

    // ---- Incremental maintenance ---------------------------------------------------

    /// Mirrors a [`Graph::apply_delta`] batch into the index: call with the
    /// SAME graph (after its `apply_delta`) and the same insert/delete triple
    /// batches. Every geometry node the batch touches — as a `geo:asWKT`
    /// subject or a `geo:hasGeometry`/`geo:hasDefaultGeometry` object — is
    /// re-extracted from the graph's CURRENT state: its old entries leave the
    /// R-tree, its new ones enter it (both incremental; O(batch × log n)).
    /// Non-geo triples in the batch are ignored, so it is safe (just a no-op)
    /// to forward every update batch. Deltas affect default-graph entries
    /// only, matching `Graph::apply_delta`.
    pub fn apply_delta(&mut self, graph: &Graph, inserts: &[[Term; 3]], deletes: &[[Term; 3]]) {
        // 1. The geometry nodes whose extracted state may have changed.
        let mut nodes: Vec<Term> = Vec::new();
        let mut push_unique = |t: &Term| {
            if !nodes.contains(t) {
                nodes.push(t.clone());
            }
        };
        for [s, p, o] in inserts.iter().chain(deletes) {
            match p {
                Term::NamedNode(p) if p.as_str() == vocab::AS_WKT => push_unique(s),
                Term::NamedNode(p)
                    if p.as_str() == vocab::HAS_GEOMETRY
                        || p.as_str() == vocab::HAS_DEFAULT_GEOMETRY =>
                {
                    push_unique(o)
                }
                _ => {}
            }
        }
        if nodes.is_empty() {
            return;
        }

        // 2. Drop every default-graph entry derived from an affected node.
        for idx in 0..self.slots.len() as u32 {
            let matches = self.slots[idx as usize]
                .as_ref()
                .is_some_and(|e| e.graph.is_none() && nodes.contains(&e.node));
            if matches {
                let entry = self.slots[idx as usize].take().expect("checked Some");
                let removed = self
                    .tree
                    .remove_with_selection_function(SelectSlot {
                        idx,
                        env: geometry_env(&entry.geometry),
                    })
                    .is_some();
                debug_assert!(removed, "tree leaf must exist for a live slot");
                self.free.push(idx);
            }
        }

        // 3. Re-extract the affected nodes from the graph's current state.
        let as_wkt = NamedNode::new_unchecked(vocab::AS_WKT);
        for node in &nodes {
            // The node's owning features, post-delta.
            let mut owners: Vec<Term> = Vec::new();
            for pred in [vocab::HAS_GEOMETRY, vocab::HAS_DEFAULT_GEOMETRY] {
                let pred = NamedNode::new_unchecked(pred);
                if let Some(pattern) = graph.pattern(None, Some(&pred), Some(node)) {
                    let scan = graph.store.scan(&pattern);
                    for row in scan.rows.iter() {
                        let [s, _, _] = scan.to_spo(row);
                        let feature = graph.dict.term(s);
                        if !owners.contains(&feature) {
                            owners.push(feature);
                        }
                    }
                }
            }
            // The node's wktLiterals, post-delta (same skip rules as build).
            let Some(pattern) = graph.pattern(Some(node), Some(&as_wkt), None) else {
                continue;
            };
            let scan = graph.store.scan(&pattern);
            for row in scan.rows.iter() {
                let [_, _, o] = scan.to_spo(row);
                let lit = match graph.dict.term(o) {
                    Term::Literal(l) if l.datatype().as_str() == vocab::WKT_LITERAL => l,
                    _ => {
                        self.skipped += 1;
                        continue;
                    }
                };
                let geometry = match crate::parse_wkt_literal(lit.value()) {
                    Ok(g) if g.crs.is_geographic() && g.geometry.bounding_rect().is_some() => g,
                    _ => {
                        self.skipped += 1;
                        continue;
                    }
                };
                if owners.is_empty() {
                    self.insert_entry(Entry {
                        entity: node.clone(),
                        node: node.clone(),
                        graph: None,
                        geometry,
                    });
                } else {
                    for feature in &owners {
                        self.insert_entry(Entry {
                            entity: feature.clone(),
                            node: node.clone(),
                            graph: None,
                            geometry: geometry.clone(),
                        });
                    }
                }
            }
        }
    }

    /// Adds one entry to the slot array and the R-tree (incremental insert).
    fn insert_entry(&mut self, entry: Entry) {
        let env = geometry_env(&entry.geometry);
        let idx = match self.free.pop() {
            Some(idx) => {
                self.slots[idx as usize] = Some(entry);
                idx
            }
            None => {
                self.slots.push(Some(entry));
                (self.slots.len() - 1) as u32
            }
        };
        self.tree.insert(TreeItem { idx, env });
    }

    // ---- Queries --------------------------------------------------------------------

    /// The pole-safe long/lat window(s) guaranteed to contain the great-circle
    /// ball of `meters` around `center`. A ball crossing the ANTIMERIDIAN
    /// yields two windows (one per side of ±180°); otherwise one.
    fn ball_windows(center: Point<f64>, meters: f64) -> Vec<AABB<[f64; 2]>> {
        let dlat = meters / METERS_PER_DEGREE;
        let lat_min = (center.y() - dlat).max(-90.0);
        let lat_max = (center.y() + dlat).min(90.0);
        // Longitude degrees shrink with cos(lat): size the window for the
        // WORST latitude it spans so the window is a superset of the ball.
        let max_abs_lat = lat_min.abs().max(lat_max.abs()).to_radians();
        if lat_min <= -90.0 + 1e-12
            || lat_max >= 90.0 - 1e-12
            || max_abs_lat.cos() * METERS_PER_DEGREE * 180.0 <= meters
        {
            // The ball encloses a pole (or the whole longitude band): every
            // longitude is inside.
            return vec![AABB::from_corners([-180.0, lat_min], [180.0, lat_max])];
        }
        // Normalise the center into [-180, 180] so the wrap arithmetic below
        // is exact for callers passing e.g. 181°.
        let cx = (center.x() + 180.0).rem_euclid(360.0) - 180.0;
        let dlon = meters / (METERS_PER_DEGREE * max_abs_lat.cos());
        let (lon_min, lon_max) = (cx - dlon, cx + dlon);
        if lon_min < -180.0 {
            // Crosses the antimeridian westward: [-180, lon_max] + the wrapped
            // remainder on the +180 side.
            vec![
                AABB::from_corners([-180.0, lat_min], [lon_max, lat_max]),
                AABB::from_corners([lon_min + 360.0, lat_min], [180.0, lat_max]),
            ]
        } else if lon_max > 180.0 {
            // Crosses eastward: [lon_min, 180] + the wrapped remainder.
            vec![
                AABB::from_corners([lon_min, lat_min], [180.0, lat_max]),
                AABB::from_corners([-180.0, lat_min], [lon_max - 360.0, lat_max]),
            ]
        } else {
            vec![AABB::from_corners([lon_min, lat_min], [lon_max, lat_max])]
        }
    }

    /// All entities whose geometry lies within `meters` great-circle metres of
    /// `center` (a CRS84 long/lat point), nearest first, truncated to `limit`
    /// (when given). Distance to an extended geometry is to its (spherical)
    /// closest point. Balls crossing the antimeridian are handled (split into
    /// two windows internally).
    pub fn within_distance(
        &self,
        center: Point<f64>,
        meters: f64,
        limit: Option<usize>,
    ) -> Vec<(&Term, f64)> {
        let windows = Self::ball_windows(center, meters);
        // An entry's AABB can intersect both halves of a split window: dedupe.
        let mut seen: FxHashSet<u32> = FxHashSet::default();
        let mut hits: Vec<(&Term, f64)> = Vec::new();
        for window in &windows {
            for item in self.tree.locate_in_envelope_intersecting(window) {
                if windows.len() > 1 && !seen.insert(item.idx) {
                    continue;
                }
                let e = self.slots[item.idx as usize].as_ref().expect("live slot");
                let Ok(d) = geof::point_to_geometry_meters(center, &e.geometry.geometry) else {
                    continue;
                };
                if d <= meters {
                    hits.push((&e.entity, d));
                }
            }
        }
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
        let share = (k as f64 / self.tree.size().max(1) as f64).sqrt();
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
    /// be in a geographic CRS, matching the indexed geometries; its
    /// coordinates are taken as written (no antimeridian wrapping — the
    /// refinement itself is planar in degree space).
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
                let e = self.slots[item.idx as usize].as_ref().expect("live slot");
                e.geometry.geometry.intersects(&geometry.geometry).then_some(&e.entity)
            })
            .collect()
    }

    /// Convenience: [`intersects`](Self::intersects) from a wktLiteral lexical form.
    pub fn intersects_wkt(&self, wkt_literal: &str) -> Result<Vec<&Term>, crate::GeoError> {
        Ok(self.intersects(&crate::parse_wkt_literal(wkt_literal)?))
    }
}
