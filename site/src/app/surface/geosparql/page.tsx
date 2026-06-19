import type { Metadata } from "next";
import { MapPin } from "lucide-react";

import { SurfaceContent } from "@/components/surface-content";
import { GeosparqlWalkthrough } from "@/components/geosparql-walkthrough";

export const metadata: Metadata = {
  title: "GeoSPARQL",
  description:
    "sparq-geo — an opt-in GeoSPARQL 1.0/1.1 core: WKT / GML geometry literals, the OGC geof: spatial functions (distance, the sf*/eh*/rcc8* DE-9IM relation families, envelope / buffer / set ops) inside plain SPARQL, the topology-property query-rewrite extension, and an R-tree GeoIndex for nearest / radius / intersects queries. This page replays REAL captured engine output — the London–Paris distance < 400 km query, the within-polygon spatial join, and the GeoIndex metres — byte-for-byte.",
};

export default function GeosparqlSurfacePage() {
  return (
    <SurfaceContent
      icon={MapPin}
      title="GeoSPARQL"
      statement="WKT / GML geometry literals, the OGC geof: spatial functions and the sf*/eh*/rcc8* topology relations inside plain SPARQL, a topology-property query rewrite, and an R-tree GeoIndex for nearest / radius / intersects queries."
      tier="walkthrough"
      extraBadge="Opt-in crate · captured output"
      intro={
        <>
          <p>
            <code className="font-mono text-foreground">sparq-geo</code> is the opt-in
            GeoSPARQL 1.0/1.1 <strong className="text-foreground">core</strong> for the engine.
            It parses <code className="font-mono">geo:wktLiteral</code> and{" "}
            <code className="font-mono">geo:gmlLiteral</code> (the GML Simple-Features profile)
            geometries, evaluates the OGC <code className="font-mono">geof:</code> spatial
            functions, packages them as a{" "}
            <code className="font-mono">sparq_engine::FunctionRegistry</code> so they run inside
            real SPARQL <code className="font-mono">FILTER</code> /{" "}
            <code className="font-mono">BIND</code> / <code className="font-mono">SELECT</code>,
            and builds an <strong className="text-foreground">R-tree GeoIndex</strong> over a
            graph for distance / nearest / intersection queries.
          </p>
          <p>
            The function surface covers <strong className="text-foreground">distance</strong>{" "}
            (metric or angular units), the three DE-9IM{" "}
            <strong className="text-foreground">topology-relation families</strong> —{" "}
            <code className="font-mono">sf*</code> (Simple Features),{" "}
            <code className="font-mono">eh*</code> (Egenhofer),{" "}
            <code className="font-mono">rcc8*</code> (RCC8) — plus a generic 9-character DE-9IM{" "}
            <code className="font-mono">relate</code>, the geometry-producing functions (
            <code className="font-mono">envelope</code> / <code className="font-mono">boundary</code>{" "}
            / <code className="font-mono">convexHull</code> / <code className="font-mono">buffer</code>),
            and the four set operations (
            <code className="font-mono">intersection</code> / <code className="font-mono">union</code>{" "}
            / <code className="font-mono">difference</code> /{" "}
            <code className="font-mono">symDifference</code>). The RDF-native extra is the{" "}
            <strong className="text-foreground">topology-property query rewrite</strong>: write a
            relation as a triple (<code className="font-mono">?a geo:sfWithin ?b</code>) and an
            opt-in entry point expands it into the geometry-resolution joins plus the matching{" "}
            <code className="font-mono">geof:</code> FILTER — the engine and wasm bundle are
            unchanged.
          </p>
          <p>
            It is a <strong className="text-foreground">separate, opt-in crate</strong> so the
            core engine and the lean wasm bundle carry{" "}
            <strong className="text-foreground">zero geometry code</strong>; the server exposes
            it only behind its non-default <code className="font-mono">geo</code> cargo feature.
            This static Pages site has no backend and the crate is native-only, so &mdash;
            honestly &mdash; this is a{" "}
            <strong className="text-foreground">captured-output walkthrough</strong>: the
            London–Paris distance &lt; 400 km query, the within-polygon spatial join, the{" "}
            <code className="font-mono">geof:buffer</code> chain, the topology-property rewrite,
            and the R-tree <code className="font-mono">nearest</code> /{" "}
            <code className="font-mono">within_distance</code> /{" "}
            <code className="font-mono">intersects</code> metres are{" "}
            <em>real, verbatim engine output</em> &mdash; produced by running the real binary
            over tiny declared fixtures (the same ones the crate&rsquo;s tests use). The small
            map is a legibility aid drawn from the verbatim WKT coordinates, not captured output.
          </p>
        </>
      }
      capabilities={[
        {
          title: "WKT + GML geometry literals",
          body: "Parses geo:wktLiteral and geo:gmlLiteral (GML Simple-Features: Point / LineString / Polygon with rings / Multi*), an optional leading CRS IRI, EPSG:4326 axis-swap, and the OGC R9 geo: metadata properties (dimension / isEmpty / isSimple / …).",
        },
        {
          title: "geof: inside plain SPARQL",
          body: "geof_registry() packages the functions as a sparq-engine FunctionRegistry, so distance / the relation families / envelope / buffer / set ops run inside real FILTER / BIND / SELECT expressions — and chain (geof:buffer feeds straight into geof:sfWithin).",
        },
        {
          title: "Three DE-9IM relation families",
          body: "sf* (Simple Features), eh* (Egenhofer) and rcc8* (RCC8) topology predicates, plus a generic 9-char DE-9IM relate(). Relations are planar (coordinate-space), not geodesic.",
        },
        {
          title: "Topology-property query rewrite",
          body: "geosparql_rewrite expands ?a geo:sfWithin ?b triple patterns into (hasDefaultGeometry|hasGeometry)/asWKT resolution joins + the matching geof: FILTER — opt-in, so W3C conformance of the default entry points is unaffected.",
        },
        {
          title: "R-tree GeoIndex (nearest / radius / intersects)",
          body: "GeoIndex::build scans geo:asWKT / geo:asGML into an rstar R-tree; nearest (k-NN), within_distance (radius) and intersects answer great-circle metre queries, with incremental apply_delta to stay in sync with graph updates.",
        },
        {
          title: "Opt-in everywhere",
          body: "A separate crate (the core engine + lean wasm carry no geometry code); the server installs geof: only behind its non-default `geo` feature; the `reproject` module (a small curated EPSG table → CRS84) is itself opt-in.",
        },
      ]}
      runsNote={
        <>
          <p>
            <strong className="text-foreground">Captured-output replay.</strong> Every{" "}
            <code className="font-mono">geof:</code> result table and every R-tree{" "}
            <code className="font-mono">GeoIndex</code> metre on this page is the verbatim output
            of the real <code className="font-mono">sparq-geo</code> binary (built with{" "}
            <code className="font-mono">--features engine</code>) over tiny declared in-memory
            Turtle fixtures — the <em>same</em> fixtures the crate&rsquo;s own committed tests
            assert against (<code className="font-mono">tests/registry_sparql.rs</code> /{" "}
            <code className="font-mono">tests/e2e.rs</code> /{" "}
            <code className="font-mono">tests/query_rewrite.rs</code>), which corroborate the
            spatial-join order, the ≈ 343.6 km London–Paris distance and the rewrite / index
            matches. Everything is deterministic (a fixed fixture, planar DE-9IM, haversine
            distance), so a re-run yields byte-identical output; a pinning test (
            <code className="font-mono">site/test/geosparql.test.mjs</code>) asserts the
            serialization can&rsquo;t drift (datatypes intact, metres pinned). Reproduce the
            corroborating run yourself:
          </p>
          <pre className="overflow-x-auto rounded-lg border bg-muted/40 p-3 font-mono text-[12px] leading-relaxed">
            {`cargo test -p sparq-geo --features engine`}
          </pre>
        </>
      }
      caveat={
        <>
          <p>
            <strong className="text-foreground">The map is a legibility aid.</strong> The dots
            and rings are the verbatim WKT longitude/latitude from the fixtures (so they are the
            real geometry), but the projection and the abstract background are{" "}
            <em>illustrative chrome</em>, not captured output. No latency or throughput number is
            shown (hardware-dependent, non-canonical).
          </p>
          <p>
            <strong className="text-foreground">Metric distance is locally exact.</strong>{" "}
            <code className="font-mono">geof:distance</code> with metric units is exact haversine
            when an operand is a point, but a{" "}
            <strong className="text-foreground">local equirectangular approximation</strong>{" "}
            between two extended geometries (same for <code className="font-mono">geof:buffer</code>{" "}
            in metres) — accurate locally, distorted at continental scale or near the poles.
            Topology relations are <strong className="text-foreground">planar</strong> DE-9IM in
            coordinate space, not geodesic. This is the GeoSPARQL{" "}
            <em>core</em>: there is no RIF / <code className="font-mono">geor:</code> rule
            rewriting, and a handful of GML cases (<code className="font-mono">gml:Envelope</code>,
            arc segments, 3-D) are a clean <code className="font-mono">Unsupported</code> error —
            see the crate&rsquo;s open beads for the exact boundary.
          </p>
        </>
      }
      links={[
        {
          href: "https://github.com/jeswr/sparq/tree/main/crates/sparq-geo",
          label: "sparq-geo crate",
          external: true,
        },
        {
          href: "https://github.com/jeswr/sparq/blob/main/skills/geosparql/SKILL.md",
          label: "geosparql SKILL.md",
          external: true,
        },
      ]}
    >
      <GeosparqlWalkthrough />
    </SurfaceContent>
  );
}
