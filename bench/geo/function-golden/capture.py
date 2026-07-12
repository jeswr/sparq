#!/usr/bin/env python3
# [FABLE-5] sq-iwulr — OFFLINE capture of the GEOS golden truth-table for the
# GeoSPARQL geometry-function differential
# (crates/sparq-conformance/tests/geo_function_differential.rs).
#
# DO NOT RUN IN CI. This script is executed OFFLINE, by hand, against a locally
# installed shapely/GEOS (e.g. `python3 -m venv /tmp/geovenv && /tmp/geovenv/bin/pip
# install shapely && /tmp/geovenv/bin/python capture.py`), and its OUTPUT — the
# four *.tsv truth-tables next to it — is COMMITTED. CI only reads the committed
# TSVs; it never needs Python, shapely, GEOS, or the network. Re-run this script
# only to EXTEND the corpus (then re-review the diff like any oracle change).
#
# What is captured (all planar/Cartesian over the lon-lat coordinates, exactly
# the model both engines use for these functions — GEOS is planar, and sparq-geo's
# simple-features relations / degree-unit distance / degree-unit buffer are the
# planar coordinate-space operations):
#   geometries.tsv       gid <TAB> WKT                    — the committed corpus
#   relations.tsv        function <TAB> a <TAB> b <TAB> true|false
#                        (the 8 OGC simple-features predicates, full ordered
#                        cross-product of the corpus)
#   de9im.tsv            a <TAB> b <TAB> 9-char DE-9IM intersection matrix
#   distance.tsv         a <TAB> b <TAB> planar distance (coordinate units,
#                        full f64 precision via repr)
#   buffer_relation.tsv  geom <TAB> radius <TAB> probe-WKT <TAB> true|false
#                        (is the probe point WITHIN buffer(geom, radius)?)
#
# Buffer rows are MARGIN-GUARDED: the two engines discretise the buffer arc
# differently (quad segments), so every committed row requires
# |distance(probe, geom) - radius| >= MARGIN_FRACTION * radius — far outside any
# realistic arc-discretisation error on either side. The script REFUSES to emit
# a borderline row.
import sys
from datetime import date

import shapely
from shapely import wkt as shapely_wkt

MARGIN_FRACTION = 0.2

# The committed WKT corpus. Small, hand-picked to cover: point/line/polygon and
# multi* kinds, containment, boundary-touch (edge, corner, collinear edge),
# crossing, overlap, topological-equality-under-different-vertex-order, a
# polygon with a hole (plus a point inside the hole), and disjoint cases.
GEOMETRIES = [
    ("pt_a", "POINT(2 2)"),                     # inside sq_big / sq_small corner region
    ("pt_far", "POINT(40 40)"),                 # far from everything
    ("pt_edge", "POINT(0 5)"),                  # ON sq_big's left edge (boundary)
    ("pt_corner", "POINT(10 10)"),              # sq_big corner; inside poly_hole's HOLE
    ("line_diag", "LINESTRING(-5 -5, 5 5)"),
    ("line_horiz", "LINESTRING(-10 2, 10 2)"),
    ("line_vert", "LINESTRING(0 -5, 0 5)"),     # crosses line_horiz at (0 2)
    ("line_edge", "LINESTRING(0 0, 10 0)"),     # collinear with sq_big's bottom edge
    ("line_far", "LINESTRING(30 30, 40 40)"),
    ("sq_big", "POLYGON((0 0, 10 0, 10 10, 0 10, 0 0))"),
    ("sq_small", "POLYGON((2 2, 4 2, 4 4, 2 4, 2 2))"),        # within sq_big
    ("sq_overlap", "POLYGON((5 5, 15 5, 15 15, 5 15, 5 5))"),  # overlaps sq_big
    ("sq_touch", "POLYGON((10 0, 20 0, 20 10, 10 10, 10 0))"), # shares sq_big's right edge
    ("sq_corner", "POLYGON((10 10, 12 10, 12 12, 10 12, 10 10))"),  # single-point touch
    ("sq_far", "POLYGON((30 0, 35 0, 35 5, 30 5, 30 0))"),
    ("sq_equal", "POLYGON((10 0, 10 10, 0 10, 0 0, 10 0))"),   # sq_big, other start vertex
    (
        "poly_hole",
        "POLYGON((0 0, 20 0, 20 20, 0 20, 0 0), (5 5, 15 5, 15 15, 5 15, 5 5))",
    ),
    ("mpt", "MULTIPOINT((1 1), (11 11))"),
    ("mline", "MULTILINESTRING((-5 -5, 5 5), (20 20, 25 25))"),
    ("mpoly", "MULTIPOLYGON(((0 0, 3 0, 3 3, 0 3, 0 0)), ((5 5, 8 5, 8 8, 5 8, 5 5)))"),
]

# The 8 OGC simple-features predicates: (row name, shapely method).
SF_PREDICATES = [
    ("sfEquals", "equals"),
    ("sfDisjoint", "disjoint"),
    ("sfIntersects", "intersects"),
    ("sfTouches", "touches"),
    ("sfCrosses", "crosses"),
    ("sfWithin", "within"),
    ("sfContains", "contains"),
    ("sfOverlaps", "overlaps"),
]

# (geometry id, radius, probe WKT) — expected bool is CAPTURED, not hand-written.
BUFFER_CASES = [
    ("sq_big", 1.0, "POINT(-0.5 5)"),
    ("sq_big", 1.0, "POINT(-1.5 5)"),
    ("sq_big", 1.0, "POINT(10.5 10.5)"),
    ("sq_big", 1.0, "POINT(11 11)"),
    ("line_diag", 0.5, "POINT(0.3 0)"),
    ("line_diag", 0.5, "POINT(1 0)"),
    ("pt_a", 2.0, "POINT(3 3)"),
    ("pt_a", 2.0, "POINT(4.5 2)"),
    ("poly_hole", 1.0, "POINT(10 10)"),  # inside the hole; the buffered hole keeps it out
    ("mpoly", 0.5, "POINT(4 4)"),        # between the two components
]


def header(what: str) -> str:
    return (
        f"# [FABLE-5] sq-iwulr GOLDEN truth-table: {what}\n"
        f"# Captured OFFLINE by bench/geo/function-golden/capture.py — DO NOT regenerate in CI.\n"
        f"# Oracle: GEOS {shapely.geos_version_string} via shapely {shapely.__version__}"
        f" (planar/Cartesian over lon-lat coordinates, CRS84).\n"
        f"# Captured: {date.today().isoformat()}\n"
    )


def main() -> None:
    geoms = {gid: shapely_wkt.loads(w) for gid, w in GEOMETRIES}
    ids = [gid for gid, _ in GEOMETRIES]

    with open("geometries.tsv", "w", encoding="utf-8") as f:
        f.write(header("the WKT corpus (gid\\tWKT)"))
        for gid, w in GEOMETRIES:
            f.write(f"{gid}\t{w}\n")

    with open("relations.tsv", "w", encoding="utf-8") as f:
        f.write(header("simple-features predicates (function\\ta\\tb\\ttrue|false)"))
        for name, method in SF_PREDICATES:
            for a in ids:
                for b in ids:
                    v = getattr(geoms[a], method)(geoms[b])
                    f.write(f"{name}\t{a}\t{b}\t{'true' if v else 'false'}\n")

    with open("de9im.tsv", "w", encoding="utf-8") as f:
        f.write(header("DE-9IM intersection matrices (a\\tb\\tmatrix)"))
        for a in ids:
            for b in ids:
                f.write(f"{a}\t{b}\t{geoms[a].relate(geoms[b])}\n")

    with open("distance.tsv", "w", encoding="utf-8") as f:
        f.write(header("planar coordinate-space distances (a\\tb\\tdistance)"))
        for a in ids:
            for b in ids:
                f.write(f"{a}\t{b}\t{geoms[a].distance(geoms[b])!r}\n")

    with open("buffer_relation.tsv", "w", encoding="utf-8") as f:
        f.write(header("probe within buffer(geom, radius) (geom\\tradius\\tprobe\\ttrue|false)"))
        for gid, radius, probe_wkt in BUFFER_CASES:
            probe = shapely_wkt.loads(probe_wkt)
            d = probe.distance(geoms[gid])
            if abs(d - radius) < MARGIN_FRACTION * radius:
                sys.exit(
                    f"REFUSING borderline buffer row {gid} r={radius} probe={probe_wkt}: "
                    f"|{d} - {radius}| < {MARGIN_FRACTION} * {radius} — arc-discretisation "
                    f"differences between engines could flip it; pick a probe further from "
                    f"the buffer boundary."
                )
            v = probe.within(geoms[gid].buffer(radius))
            f.write(f"{gid}\t{radius!r}\t{probe_wkt}\t{'true' if v else 'false'}\n")

    n = len(ids)
    print(f"captured: {n} geometries, {8 * n * n} relation rows, {n * n} matrices, "
          f"{n * n} distances, {len(BUFFER_CASES)} buffer rows")


if __name__ == "__main__":
    main()
