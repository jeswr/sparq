"use client";

// [OPUS-4.8] sq-ndaz — the /surface/geosparql (GeoSPARQL) showcase (tier-e). sparq-geo is
// an OPT-IN native crate (the engine's lean wasm bundle carries ZERO geometry code, and
// sparq-server exposes geof: only behind its non-default `geo` cargo feature), and the
// static Pages site has no backend — so, per the feature-showcase design's honest tier-e
// fallback, this REPLAYS captured output rather than running geof: live:
//
//   1. geof: SPARQL — pick a query (geof:distance < 400 km, the geof:sfWithin spatial
//      join, the geof:buffer chain, or the topology-PROPERTY-form geosparql_rewrite) and
//      see the REAL executed result table (verbatim oxrdf::Term serialization).
//   2. R-tree GeoIndex — nearest / within_distance / intersects metres, captured verbatim.
//   3. A small dependency-free SVG map draws the fixture geometry (the dots ARE the real
//      WKT lon/lat) so the spatial relations are legible — no Leaflet/MapLibre dep.
//
// HONESTY (see src/lib/geosparql.ts): the result rows / distances are REAL captured engine
// output (answer-exact, verbatim oxrdf serialization; pinned by site/test/geosparql.test.mjs
// so they cannot drift). The map projection + chrome are an illustrative legibility aid. The
// metric-distance caveat (haversine for points, local equirectangular for two extended
// geometries) is real. All data lives in src/lib/geosparql.ts (framework-free, unit-tested).

import * as React from "react";
import {
  Database,
  Globe2,
  MapPin,
  Play,
  Ruler,
  Search,
  Shapes,
} from "lucide-react";

import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { cn } from "@/lib/utils";
import {
  CITIES,
  FEATURES,
  GEO_QUERIES,
  INDEX_BUILD,
  INDEX_CENTER,
  INDEX_RUNS,
  RELATION_FAMILIES,
  type GeoQuery,
  type IndexRun,
  type MapFeature,
  shortIri,
} from "@/lib/geosparql";

// ── A tiny dependency-free SVG map (no Leaflet/MapLibre dep) ─────────────────────────────
// An equirectangular projection over the fixture's bounding box: the dots/rings ARE the
// real WKT lon/lat, so the spatial relations are faithful; the projection + abstract
// background are an illustrative legibility aid (labelled as such on the page).

const MAP_W = 360;
const MAP_H = 300;
const PAD = 22;

function bounds(features: MapFeature[]) {
  let minLon = Infinity,
    maxLon = -Infinity,
    minLat = Infinity,
    maxLat = -Infinity;
  for (const f of features) {
    for (const [lon, lat] of f.coords) {
      minLon = Math.min(minLon, lon);
      maxLon = Math.max(maxLon, lon);
      minLat = Math.min(minLat, lat);
      maxLat = Math.max(maxLat, lat);
    }
  }
  // Avoid a degenerate span.
  if (maxLon - minLon < 1e-6) {
    minLon -= 0.5;
    maxLon += 0.5;
  }
  if (maxLat - minLat < 1e-6) {
    minLat -= 0.5;
    maxLat += 0.5;
  }
  return { minLon, maxLon, minLat, maxLat };
}

function makeProject(features: MapFeature[]) {
  const { minLon, maxLon, minLat, maxLat } = bounds(features);
  const sx = (MAP_W - 2 * PAD) / (maxLon - minLon);
  const sy = (MAP_H - 2 * PAD) / (maxLat - minLat);
  const s = Math.min(sx, sy); // keep aspect ratio
  return (lon: number, lat: number): [number, number] => {
    const x = PAD + (lon - minLon) * s;
    const y = MAP_H - PAD - (lat - minLat) * s; // lat increases upward
    return [x, y];
  };
}

function FixtureMap({
  features,
  highlight,
}: {
  features: MapFeature[];
  highlight: Set<string>;
}) {
  const project = React.useMemo(() => makeProject(features), [features]);
  return (
    <svg
      viewBox={`0 0 ${MAP_W} ${MAP_H}`}
      className="w-full rounded-lg border bg-muted/30"
      role="img"
      aria-label="Map of the fixture geometry — points and polygons at their verbatim WKT longitude/latitude."
    >
      {/* subtle grid background (illustrative chrome, not data) */}
      {[0.25, 0.5, 0.75].map((t) => (
        <React.Fragment key={t}>
          <line
            x1={PAD + t * (MAP_W - 2 * PAD)}
            y1={PAD}
            x2={PAD + t * (MAP_W - 2 * PAD)}
            y2={MAP_H - PAD}
            className="stroke-foreground/5"
            strokeWidth={1}
          />
          <line
            x1={PAD}
            y1={PAD + t * (MAP_H - 2 * PAD)}
            x2={MAP_W - PAD}
            y2={PAD + t * (MAP_H - 2 * PAD)}
            className="stroke-foreground/5"
            strokeWidth={1}
          />
        </React.Fragment>
      ))}
      {/* polygons first (under the points) */}
      {features
        .filter((f) => f.kind === "polygon")
        .map((f) => {
          const on = highlight.has(f.id);
          const pts = f.coords.map(([lon, lat]) => project(lon, lat).join(",")).join(" ");
          return (
            <polygon
              key={f.id}
              points={pts}
              className={cn(
                on ? "fill-primary/15 stroke-primary/70" : "fill-foreground/5 stroke-foreground/25",
              )}
              strokeWidth={1.5}
            />
          );
        })}
      {/* points */}
      {features
        .filter((f) => f.kind === "point")
        .map((f) => {
          const [x, y] = project(f.coords[0][0], f.coords[0][1]);
          const on = highlight.has(f.id);
          return (
            <g key={f.id}>
              <circle
                cx={x}
                cy={y}
                r={on ? 6 : 4.5}
                className={cn(on ? "fill-primary stroke-background" : "fill-foreground/40 stroke-background")}
                strokeWidth={1.5}
              />
              <text
                x={x + 8}
                y={y + 4}
                className={cn(
                  "text-[10px]",
                  on ? "fill-primary font-semibold" : "fill-muted-foreground",
                )}
              >
                {f.label}
              </text>
            </g>
          );
        })}
    </svg>
  );
}

// ── result tables ────────────────────────────────────────────────────────────────────────

function ResultTable({ q }: { q: GeoQuery }) {
  return (
    <div className="overflow-x-auto rounded-lg border">
      <table className="w-full border-collapse font-mono text-[12px]">
        <thead>
          <tr className="border-b bg-muted/60">
            {q.vars.map((v) => (
              <th key={v} className="px-3 py-1.5 text-left font-semibold">
                ?{v}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {q.rows.map((row, i) => (
            <tr key={i} className="border-b last:border-0">
              {row.map((cell, j) => (
                <td key={j} className="px-3 py-1.5 align-top break-all">
                  {cell}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

/** Which fixture features a given query "lights up" on the map. */
function highlightForQuery(q: GeoQuery): Set<string> {
  const ids = new Set<string>();
  for (const row of q.rows) {
    for (const cell of row) {
      const id = shortIri(cell);
      if (CITIES.some((f) => f.id === id)) ids.add(id);
    }
  }
  // Always show London as the anchor for the city-fixture queries.
  if (!q.rewrite) ids.add("london");
  return ids;
}

function highlightForIndex(run: IndexRun): Set<string> {
  const ids = new Set<string>(run.hits.map((h) => shortIri(h.term)));
  return ids;
}

export function GeosparqlWalkthrough() {
  const [qSel, setQSel] = React.useState(0);
  const [ran, setRan] = React.useState(false);
  const [iSel, setISel] = React.useState(0);
  const q = GEO_QUERIES[qSel];
  const run = INDEX_RUNS[iSel];

  function pickQuery(i: number) {
    setQSel(i);
    setRan(false);
  }

  // The rewrite query uses the FEATURES fixture; the rest use CITIES.
  const queryFeatures = q.rewrite ? FEATURES : CITIES;
  const queryHighlight = ran ? highlightForQuery(q) : new Set<string>();

  return (
    <div className="space-y-10">
      {/* ── 1. geof: inside SPARQL ──────────────────────────────────────────────────── */}
      <section className="space-y-4">
        <div className="flex items-center gap-2">
          <Ruler className="size-5 text-primary" aria-hidden="true" />
          <h2 className="text-xl font-semibold">geof: spatial functions in SPARQL</h2>
          <Badge variant="success" className="text-[10px] uppercase">
            real captured output
          </Badge>
        </div>
        <p className="measure text-sm text-muted-foreground">
          With <code className="font-mono">sparq-geo</code>, the OGC{" "}
          <code className="font-mono">geof:</code> functions run inside plain SPARQL{" "}
          <code className="font-mono">FILTER</code> / <code className="font-mono">BIND</code> /{" "}
          <code className="font-mono">SELECT</code> — distance, the DE-9IM relation families,
          and the geometry-producing functions (
          <code className="font-mono">envelope</code> / <code className="font-mono">buffer</code>{" "}
          /&hellip;) that feed straight back into another <code className="font-mono">geof:</code>{" "}
          call. The bead&rsquo;s example — cities within{" "}
          <strong className="text-foreground">400 km of London</strong> — is the first chip.
          The last chip is the opt-in{" "}
          <strong className="text-foreground">topology PROPERTY form</strong>: write the
          relation as a triple (<code className="font-mono">?f geo:sfWithin ?region</code>) and
          the <code className="font-mono">geosparql_rewrite</code> extension resolves each
          feature&rsquo;s geometry and applies the matching{" "}
          <code className="font-mono">geof:</code> — with no asserted topology triple anywhere.
        </p>

        {/* Query chips. */}
        <div className="flex flex-wrap gap-2">
          {GEO_QUERIES.map((item, i) => (
            <button
              key={item.id}
              type="button"
              onClick={() => pickQuery(i)}
              className={cn(
                "rounded-full px-3 py-1.5 text-left text-[12.5px] ring-1 transition-colors",
                i === qSel
                  ? "bg-primary/10 text-primary ring-primary/30"
                  : "bg-muted/40 text-muted-foreground ring-foreground/10 hover:bg-muted",
              )}
            >
              {item.caption}
            </button>
          ))}
        </div>

        <div className="grid gap-4 lg:grid-cols-2">
          {/* The map. */}
          <Card>
            <CardHeader className="flex-row items-center gap-2 space-y-0">
              <Globe2 className="size-4 text-primary" aria-hidden="true" />
              <CardTitle className="text-sm">
                Fixture geometry {q.rewrite ? "(inner-London box + points)" : "(EU cities + boxes)"}
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-2">
              <FixtureMap features={queryFeatures} highlight={queryHighlight} />
              <p className="text-xs text-muted-foreground">
                The dots and rings are the verbatim WKT longitude/latitude from the fixture; the
                projection and background are an illustrative legibility aid (not captured
                output).{ran ? " Highlighted: the rows the query returned." : ""}
              </p>
            </CardContent>
          </Card>

          {/* The query + result. */}
          <Card>
            <CardHeader className="space-y-3">
              <div className="flex flex-wrap items-center justify-between gap-3">
                <CardTitle className="text-base">{q.caption}</CardTitle>
                <Button size="sm" onClick={() => setRan(true)} disabled={ran}>
                  <Play className="size-4" aria-hidden="true" />
                  {ran ? "Executed" : "Run"}
                </Button>
              </div>
              {q.rewrite && (
                <Badge variant="muted" className="w-fit text-[10px] uppercase">
                  opt-in geosparql_rewrite
                </Badge>
              )}
            </CardHeader>
            <CardContent className="space-y-4">
              <pre className="overflow-x-auto rounded-lg border bg-muted/40 p-3 font-mono text-[12px] leading-relaxed">
                {q.sparql}
              </pre>
              {ran && (
                <div className="space-y-1.5">
                  <div className="flex items-center gap-1.5 text-xs font-medium text-muted-foreground">
                    <Database className="size-3.5" aria-hidden="true" />
                    result
                    <Badge variant="success" className="text-[10px] uppercase">
                      real engine output
                    </Badge>
                  </div>
                  <ResultTable q={q} />
                  <p className="text-xs text-muted-foreground">
                    {q.rows.length} row{q.rows.length === 1 ? "" : "s"}, executed by the sparq
                    engine over the declared fixture — term serialization verbatim (the
                    <code className="font-mono"> geof:distance</code> value keeps its{" "}
                    <code className="font-mono">xsd:double</code> datatype).
                  </p>
                </div>
              )}
            </CardContent>
          </Card>
        </div>
      </section>

      {/* ── 2. R-tree GeoIndex ─────────────────────────────────────────────────────────── */}
      <section className="space-y-4">
        <div className="flex items-center gap-2">
          <Search className="size-5 text-primary" aria-hidden="true" />
          <h2 className="text-xl font-semibold">R-tree GeoIndex — nearest / radius / intersects</h2>
          <Badge variant="success" className="text-[10px] uppercase">
            real captured output
          </Badge>
        </div>
        <p className="measure text-sm text-muted-foreground">
          <code className="font-mono">GeoIndex::build</code> scans a graph&rsquo;s{" "}
          <code className="font-mono">geo:asWKT</code> / <code className="font-mono">geo:asGML</code>{" "}
          serializations (resolving the owning feature through{" "}
          <code className="font-mono">geo:hasGeometry</code> /{" "}
          <code className="font-mono">geo:hasDefaultGeometry</code>) into an{" "}
          <code className="font-mono">rstar</code> R-tree, then answers great-circle{" "}
          <code className="font-mono">nearest</code> (k-NN),{" "}
          <code className="font-mono">within_distance</code> (radius) and{" "}
          <code className="font-mono">intersects</code> queries. This index built{" "}
          <strong className="text-foreground">{INDEX_BUILD.len} entities, {INDEX_BUILD.skipped}{" "}
          skipped</strong>, centred on central London ({INDEX_CENTER.lon}, {INDEX_CENTER.lat}).
        </p>

        {/* Index-run chips. */}
        <div className="flex flex-wrap gap-2">
          {INDEX_RUNS.map((item, i) => (
            <button
              key={item.id}
              type="button"
              onClick={() => setISel(i)}
              className={cn(
                "rounded-full px-3 py-1.5 text-left text-[12.5px] ring-1 transition-colors",
                i === iSel
                  ? "bg-primary/10 text-primary ring-primary/30"
                  : "bg-muted/40 text-muted-foreground ring-foreground/10 hover:bg-muted",
              )}
            >
              {item.caption}
            </button>
          ))}
        </div>

        <div className="grid gap-4 lg:grid-cols-2">
          <Card>
            <CardHeader className="flex-row items-center gap-2 space-y-0">
              <MapPin className="size-4 text-primary" aria-hidden="true" />
              <CardTitle className="text-sm">Feature fixture</CardTitle>
            </CardHeader>
            <CardContent className="space-y-2">
              <FixtureMap features={FEATURES} highlight={highlightForIndex(run)} />
              <p className="text-xs text-muted-foreground">
                Highlighted: the entities this index query returned. Distances are great-circle
                metres from the index.
              </p>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="text-base">{run.caption}</CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
              <pre className="overflow-x-auto rounded-lg border bg-muted/40 p-3 font-mono text-[12px]">
                {run.call}
              </pre>
              <div className="overflow-x-auto rounded-lg border">
                <table className="w-full border-collapse font-mono text-[12px]">
                  <thead>
                    <tr className="border-b bg-muted/60">
                      <th className="px-3 py-1.5 text-left font-semibold">entity</th>
                      <th className="px-3 py-1.5 text-right font-semibold">metres</th>
                    </tr>
                  </thead>
                  <tbody>
                    {run.hits.map((h) => (
                      <tr key={h.term} className="border-b last:border-0">
                        <td className="px-3 py-1.5">{shortIri(h.term)}</td>
                        <td className="px-3 py-1.5 text-right tabular-nums text-muted-foreground">
                          {h.metres === null
                            ? "—"
                            : h.metres === 0
                              ? "0"
                              : h.metres.toLocaleString(undefined, {
                                  maximumFractionDigits: 1,
                                })}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
              <p className="text-xs text-muted-foreground">
                {run.hits.length} hit{run.hits.length === 1 ? "" : "s"} — entity terms verbatim,
                metres the index&rsquo;s exact great-circle f64 (rounded to 1 dp for display).
                <code className="font-mono"> intersects</code> returns no distance (shown as
                &ldquo;—&rdquo;).
              </p>
            </CardContent>
          </Card>
        </div>
      </section>

      {/* ── 3. Topology relation families (vocabulary) ─────────────────────────────────── */}
      <section className="space-y-3">
        <div className="flex items-center gap-2">
          <Shapes className="size-5 text-primary" aria-hidden="true" />
          <h2 className="text-xl font-semibold">Topology relation families</h2>
          <Badge variant="muted" className="text-[10px] uppercase">
            vocabulary
          </Badge>
        </div>
        <p className="measure text-sm text-muted-foreground">
          Each topology relation comes in three GeoSPARQL families — Simple Features, Egenhofer
          and RCC8 — every member a planar DE-9IM predicate the crate evaluates (the{" "}
          <code className="font-mono">geof:sfWithin</code> join above is the captured proof one
          works). Below are the function names; relations are{" "}
          <strong className="text-foreground">planar</strong> DE-9IM in coordinate space, not
          geodesic.
        </p>
        <div className="grid gap-3 sm:grid-cols-3">
          {RELATION_FAMILIES.map((r) => (
            <div key={r.prefix} className="rounded-xl bg-muted/40 p-4 ring-1 ring-foreground/10">
              <div className="flex items-baseline gap-2">
                <span className="font-mono text-sm font-semibold text-primary">{r.prefix}</span>
                <span className="text-sm font-semibold">{r.name}</span>
              </div>
              <div className="mt-1 font-mono text-[11px] leading-relaxed text-muted-foreground">
                {r.note}
              </div>
            </div>
          ))}
        </div>
      </section>
    </div>
  );
}
