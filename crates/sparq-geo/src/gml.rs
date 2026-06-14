//! `geo:gmlLiteral` lexical forms -> CRS-tagged geometries. [OPUS-4.8] sq-zy0
//!
//! GeoSPARQL (1.0 §8.5.2 / 1.1 §8.8) defines a second geometry serialization
//! datatype, `geo:gmlLiteral`, whose lexical form is a **GML geometry element**
//! (an XML fragment) in the OGC *GML Simple Features* profile. GeoSPARQL only
//! ever uses the geometry subset of GML (points, curves, surfaces and their
//! aggregates), so this module is a focused GML-SF *geometry* parser — NOT a
//! general GML processor.
//!
//! ```text
//! "<gml:Point srsName=\"http://www.opengis.net/def/crs/OGC/1.3/CRS84\">\
//!    <gml:pos>-83.4 34.3</gml:pos></gml:Point>"^^geo:gmlLiteral
//! ```
//!
//! Output is the SAME [`GeoGeometry`](crate::GeoGeometry) the WKT path produces
//! (a [`geo_types::Geometry<f64>`] tagged with a [`Crs`](crate::Crs)), so every
//! downstream `geof:` function, the [`GeoIndex`](crate::GeoIndex), and the CRS /
//! axis-order handling work UNCHANGED. As with WKT, coordinates are normalised
//! to x = longitude / y = latitude: an EPSG:4326 `srsName` (LAT/LONG axis order)
//! is axis-swapped on parse exactly as the WKT path swaps EPSG:4326 literals.
//!
//! ## Supported GML-SF geometry elements
//!
//! * `gml:Point` — coordinates via `gml:pos` (GML 3) or `gml:coordinates`
//!   (GML 2 legacy, still emitted by many tools);
//! * `gml:LineString` — `gml:posList`, repeated `gml:pos`, or `gml:coordinates`;
//! * `gml:Polygon` — one `gml:exterior` ring and zero or more `gml:interior`
//!   rings, each a `gml:LinearRing` (GML 3) or the GML 2
//!   `gml:outerBoundaryIs` / `gml:innerBoundaryIs` spellings;
//! * `gml:MultiPoint` — `gml:pointMember` / `gml:pointMembers`;
//! * `gml:MultiCurve` — `gml:curveMember(s)` of `gml:LineString` (the SF
//!   curve type) -> a [`geo_types::MultiLineString`];
//! * `gml:MultiSurface` — `gml:surfaceMember(s)` of `gml:Polygon` -> a
//!   [`geo_types::MultiPolygon`];
//! * the GML 2 aggregates `gml:MultiLineString` (`gml:lineStringMember`) and
//!   `gml:MultiPolygon` (`gml:polygonMember`), accepted as synonyms.
//!
//! Namespace prefixes are NOT assumed to be `gml:`: elements are matched on
//! their LOCAL name (the part after the last `:`), so a document binding the GML
//! namespace to a different prefix — or to the default namespace — parses too.
//! `srsName` is read from the OUTERMOST geometry element (GML allows it on inner
//! elements; the SF profile keeps it on the root, which is what we honour).
//!
//! Deferred (clean `GeoError::Unsupported`, tracked as beads): `gml:Envelope`,
//! the `gml:Curve`/`gml:Surface` arc-segment forms, `gml:srsDimension="3"`
//! 3-D coordinates, and `gml:Triangle`/`gml:PolygonPatch` tessellations.

use crate::literal::{swap_xy, Crs, GeoGeometry};
use crate::GeoError;
use geo_types::{
    Coord, Geometry, LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon,
};
use quick_xml::events::Event;
use quick_xml::reader::Reader;

// [OPUS-4.8] quick-xml 0.40 removed `BytesText::unescape()` and deprecated
// `Attribute::unescape_value()`. Both used to decode + resolve the XML
// predefined entities without attribute-value whitespace folding. We replicate
// that exact behaviour with `quick_xml::escape::unescape`
// (= `unescape_with(resolve_predefined_entity)`, the resolver the old methods
// used), rather than 0.40's `normalized_value`/`xml_content` which would add
// AVN whitespace folding / EOL normalization the old GML path never applied.
fn unescape_attr_value(raw: &[u8]) -> Result<String, GeoError> {
    let s =
        std::str::from_utf8(raw).map_err(|e| GeoError::Parse(format!("GML attr error: {e}")))?;
    quick_xml::escape::unescape(s)
        .map(|c| c.into_owned())
        .map_err(|e| GeoError::Parse(format!("GML attr error: {e}")))
}

/// Parses a `geo:gmlLiteral` LEXICAL FORM (the literal's string value, without
/// the `^^geo:gmlLiteral` datatype): a GML-SF geometry element.
///
/// The CRS comes from the root element's `srsName` attribute (GeoSPARQL Req 11);
/// absent, it defaults to CRS84 — the same default the WKT path uses.
pub fn parse_gml_literal(lex: &str) -> Result<GeoGeometry, GeoError> {
    let node = parse_root(lex)?;
    let crs = node.crs.clone();
    let geometry = build_geometry(&node)?;
    // EPSG:4326 is LAT/LONG; normalise to internal long/lat exactly as WKT does.
    let geometry = if crs == Crs::Epsg4326 { swap_xy(geometry) } else { geometry };
    Ok(GeoGeometry { crs, geometry })
}

// ---- Stage 1: XML -> a small geometry DOM ----------------------------------

/// A parsed GML element: its local name, its `srsName`-derived CRS, the
/// whitespace-collapsed text it directly contained, and its child elements.
#[derive(Debug)]
struct Node {
    /// Local name (namespace prefix stripped), lower-cased for matching.
    name: String,
    /// CRS resolved from the nearest `srsName` (root element's, in practice).
    crs: Crs,
    /// Concatenated character data directly inside this element.
    text: String,
    children: Vec<Node>,
}

impl Node {
    /// The first child whose local name equals `name` (case-insensitive).
    fn child(&self, name: &str) -> Option<&Node> {
        self.children.iter().find(|c| c.name == name)
    }
    /// All children whose local name equals `name`.
    fn children_named<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Node> {
        self.children.iter().filter(move |c| c.name == name)
    }
}

/// Local name (after the last `:`), lower-cased.
fn local_name(qname: &[u8]) -> String {
    let s = String::from_utf8_lossy(qname);
    let local = s.rsplit(':').next().unwrap_or(&s);
    local.to_ascii_lowercase()
}

/// Resolves a `srsName` attribute value to a [`Crs`]. EPSG codes appear in many
/// equivalent IRI/URN spellings; the common ones are normalised so axis-order
/// handling matches the WKT path.
fn crs_from_srs_name(srs: &str) -> Crs {
    let s = srs.trim();
    // urn:ogc:def:crs:EPSG::4326 / EPSG:4326 / .../EPSG/0/4326 all mean 4326.
    let is_4326 = s.ends_with(":4326")
        || s.ends_with("/4326")
        || s == crate::vocab::EPSG_4326;
    let is_crs84 = s == crate::vocab::CRS84
        || s.ends_with("CRS84")
        || s.ends_with(":CRS84");
    if is_crs84 {
        Crs::Crs84
    } else if is_4326 {
        Crs::Epsg4326
    } else {
        Crs::from_iri(s)
    }
}

/// Reads the whole lexical form into a single root [`Node`] tree, propagating
/// the root `srsName` down to children (so inner geometry builders see the CRS).
fn parse_root(lex: &str) -> Result<Node, GeoError> {
    let mut reader = Reader::from_str(lex.trim());
    reader.config_mut().trim_text(true);
    let mut stack: Vec<Node> = Vec::new();
    let mut root: Option<Node> = None;
    // The CRS in force: set by the first srsName seen, inherited downward.
    let mut inherited_crs = Crs::Crs84;

    loop {
        match reader.read_event() {
            Err(e) => return Err(GeoError::Parse(format!("GML XML error: {e}"))),
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                let name = local_name(e.name().as_ref());
                let mut crs = inherited_crs.clone();
                for attr in e.attributes().flatten() {
                    if local_name(attr.key.as_ref()) == "srsname" {
                        let val = unescape_attr_value(&attr.value)?;
                        crs = crs_from_srs_name(&val);
                        inherited_crs = crs.clone();
                    }
                }
                stack.push(Node { name, crs, text: String::new(), children: Vec::new() });
            }
            Ok(Event::Text(t)) => {
                if let Some(top) = stack.last_mut() {
                    // [OPUS-4.8] 0.40: decode (no EOL normalization, matching the old
                    // `unescape`) then resolve predefined entities.
                    let decoded = t
                        .decode()
                        .map_err(|err| GeoError::Parse(format!("GML text error: {err}")))?;
                    let txt = quick_xml::escape::unescape(&decoded)
                        .map_err(|err| GeoError::Parse(format!("GML text error: {err}")))?;
                    if !top.text.is_empty() {
                        top.text.push(' ');
                    }
                    top.text.push_str(txt.trim());
                }
            }
            Ok(Event::End(_)) => {
                let node = stack
                    .pop()
                    .ok_or_else(|| GeoError::Parse("GML: unbalanced end tag".to_string()))?;
                match stack.last_mut() {
                    Some(parent) => parent.children.push(node),
                    None => root = Some(node),
                }
            }
            // Empty self-closing element, e.g. <gml:pos/> (degenerate) — keep it.
            Ok(Event::Empty(e)) => {
                let name = local_name(e.name().as_ref());
                let mut crs = inherited_crs.clone();
                for attr in e.attributes().flatten() {
                    if local_name(attr.key.as_ref()) == "srsname" {
                        let val = unescape_attr_value(&attr.value)?;
                        crs = crs_from_srs_name(&val);
                        inherited_crs = crs.clone();
                    }
                }
                let node = Node { name, crs, text: String::new(), children: Vec::new() };
                match stack.last_mut() {
                    Some(parent) => parent.children.push(node),
                    None => root = Some(node),
                }
            }
            _ => {}
        }
    }
    if !stack.is_empty() {
        return Err(GeoError::Parse("GML: unclosed element(s)".to_string()));
    }
    root.ok_or_else(|| GeoError::Parse("GML: no geometry element found".to_string()))
}

// ---- Stage 2: the geometry DOM -> geo_types ---------------------------------

/// Dispatches the root element by local name to the matching geometry builder.
fn build_geometry(node: &Node) -> Result<Geometry<f64>, GeoError> {
    match node.name.as_str() {
        "point" => Ok(Geometry::Point(build_point(node)?)),
        "linestring" => Ok(Geometry::LineString(build_linestring(node)?)),
        "polygon" => Ok(Geometry::Polygon(build_polygon(node)?)),
        "multipoint" => Ok(Geometry::MultiPoint(build_multipoint(node)?)),
        // GML-SF curves are LineStrings; MultiCurve aggregates them.
        "multicurve" | "multilinestring" => {
            Ok(Geometry::MultiLineString(build_multicurve(node)?))
        }
        // GML-SF surfaces are Polygons; MultiSurface aggregates them.
        "multisurface" | "multipolygon" => {
            Ok(Geometry::MultiPolygon(build_multisurface(node)?))
        }
        "envelope" => Err(GeoError::Unsupported(
            "GML gml:Envelope not supported (tracked as a bead); use a Polygon".to_string(),
        )),
        "curve" | "surface" => Err(GeoError::Unsupported(format!(
            "GML gml:{} (arc/segment form) not in the SF profile (tracked as a bead)",
            if node.name == "curve" { "Curve" } else { "Surface" }
        ))),
        other => Err(GeoError::Parse(format!(
            "unrecognised GML geometry element <{other}>"
        ))),
    }
}

/// Parses a whitespace/comma-separated coordinate run into `(x, y)` pairs.
///
/// GML `gml:posList` / `gml:pos` use whitespace separators; the GML 2
/// `gml:coordinates` element uses `,` between the ordinates of a tuple and
/// (by default) a space between tuples. We accept both by treating `,` as a
/// separator too — adequate for the 2-D SF profile (default cs/ts).
fn parse_coords(text: &str) -> Result<Vec<Coord<f64>>, GeoError> {
    let nums: Vec<f64> = text
        .split([' ', '\t', '\n', '\r', ','])
        .filter(|t| !t.is_empty())
        .map(|t| {
            t.parse::<f64>()
                .map_err(|_| GeoError::Parse(format!("GML: non-numeric ordinate {t:?}")))
        })
        .collect::<Result<_, _>>()?;
    if nums.is_empty() {
        return Err(GeoError::Parse("GML: empty coordinate list".to_string()));
    }
    if !nums.len().is_multiple_of(2) {
        return Err(GeoError::Parse(format!(
            "GML: odd ordinate count {} (only 2-D SF coordinates supported)",
            nums.len()
        )));
    }
    Ok(nums.chunks_exact(2).map(|p| Coord { x: p[0], y: p[1] }).collect())
}

/// The coordinate run of a positional element, from any of the SF spellings:
/// `gml:posList`, one-or-more `gml:pos`, or the GML 2 `gml:coordinates`.
fn coords_of(node: &Node) -> Result<Vec<Coord<f64>>, GeoError> {
    if let Some(pl) = node.child("poslist") {
        return parse_coords(&pl.text);
    }
    if let Some(c) = node.child("coordinates") {
        return parse_coords(&c.text);
    }
    let pos: Vec<&Node> = node.children_named("pos").collect();
    if !pos.is_empty() {
        let mut out = Vec::new();
        for p in pos {
            out.extend(parse_coords(&p.text)?);
        }
        return Ok(out);
    }
    // A bare element carrying inline text (some encoders put it directly).
    if !node.text.is_empty() {
        return parse_coords(&node.text);
    }
    Err(GeoError::Parse(format!(
        "GML <{}>: no gml:pos / gml:posList / gml:coordinates",
        node.name
    )))
}

fn build_point(node: &Node) -> Result<Point<f64>, GeoError> {
    let coords = coords_of(node)?;
    if coords.len() != 1 {
        return Err(GeoError::Parse(format!(
            "GML gml:Point must have exactly one coordinate, got {}",
            coords.len()
        )));
    }
    Ok(Point::from(coords[0]))
}

fn build_linestring(node: &Node) -> Result<LineString<f64>, GeoError> {
    let coords = coords_of(node)?;
    if coords.len() < 2 {
        return Err(GeoError::Parse(
            "GML gml:LineString needs at least two coordinates".to_string(),
        ));
    }
    Ok(LineString::new(coords))
}

/// A `gml:LinearRing` (or the bare positional content of a boundary wrapper).
fn build_ring(node: &Node) -> Result<LineString<f64>, GeoError> {
    // The ring may be the node itself (a LinearRing) or wrapped in
    // exterior/interior/outerBoundaryIs/innerBoundaryIs holding a LinearRing.
    let ring_node = node.child("linearring").unwrap_or(node);
    let coords = coords_of(ring_node)?;
    if coords.len() < 4 {
        return Err(GeoError::Parse(
            "GML gml:LinearRing needs at least four coordinates (closed ring)".to_string(),
        ));
    }
    Ok(LineString::new(coords))
}

fn build_polygon(node: &Node) -> Result<Polygon<f64>, GeoError> {
    // Exterior: gml:exterior (GML 3) or gml:outerBoundaryIs (GML 2).
    let ext_wrap = node
        .child("exterior")
        .or_else(|| node.child("outerboundaryis"))
        .ok_or_else(|| {
            GeoError::Parse("GML gml:Polygon missing gml:exterior ring".to_string())
        })?;
    let exterior = build_ring(ext_wrap)?;

    // Interiors: gml:interior (GML 3) or gml:innerBoundaryIs (GML 2).
    let mut interiors = Vec::new();
    for wrap in node
        .children_named("interior")
        .chain(node.children_named("innerboundaryis"))
    {
        interiors.push(build_ring(wrap)?);
    }
    Ok(Polygon::new(exterior, interiors))
}

fn build_multipoint(node: &Node) -> Result<MultiPoint<f64>, GeoError> {
    let mut points = Vec::new();
    // gml:pointMember (one Point each) and gml:pointMembers (many Points).
    for member in node.children_named("pointmember") {
        let p = member.child("point").ok_or_else(|| {
            GeoError::Parse("GML gml:pointMember missing gml:Point".to_string())
        })?;
        points.push(build_point(p)?);
    }
    for members in node.children_named("pointmembers") {
        for p in members.children_named("point") {
            points.push(build_point(p)?);
        }
    }
    if points.is_empty() {
        return Err(GeoError::Parse(
            "GML gml:MultiPoint has no gml:pointMember(s)".to_string(),
        ));
    }
    Ok(MultiPoint::new(points))
}

fn build_multicurve(node: &Node) -> Result<MultiLineString<f64>, GeoError> {
    let mut lines = Vec::new();
    // gml:curveMember / gml:curveMembers (MultiCurve) and the GML 2
    // gml:lineStringMember (MultiLineString) each wrap a gml:LineString.
    for member in node
        .children_named("curvemember")
        .chain(node.children_named("linestringmember"))
    {
        let ls = member.child("linestring").ok_or_else(|| {
            GeoError::Unsupported(
                "GML curve member is not a gml:LineString (only SF curves supported)".to_string(),
            )
        })?;
        lines.push(build_linestring(ls)?);
    }
    for members in node.children_named("curvemembers") {
        for ls in members.children_named("linestring") {
            lines.push(build_linestring(ls)?);
        }
    }
    if lines.is_empty() {
        return Err(GeoError::Parse(
            "GML gml:MultiCurve has no gml:curveMember(s)".to_string(),
        ));
    }
    Ok(MultiLineString::new(lines))
}

fn build_multisurface(node: &Node) -> Result<MultiPolygon<f64>, GeoError> {
    let mut polys = Vec::new();
    // gml:surfaceMember / gml:surfaceMembers (MultiSurface) and the GML 2
    // gml:polygonMember (MultiPolygon) each wrap a gml:Polygon.
    for member in node
        .children_named("surfacemember")
        .chain(node.children_named("polygonmember"))
    {
        let poly = member.child("polygon").ok_or_else(|| {
            GeoError::Unsupported(
                "GML surface member is not a gml:Polygon (only SF surfaces supported)".to_string(),
            )
        })?;
        polys.push(build_polygon(poly)?);
    }
    for members in node.children_named("surfacemembers") {
        for poly in members.children_named("polygon") {
            polys.push(build_polygon(poly)?);
        }
    }
    if polys.is_empty() {
        return Err(GeoError::Parse(
            "GML gml:MultiSurface has no gml:surfaceMember(s)".to_string(),
        ));
    }
    Ok(MultiPolygon::new(polys))
}
