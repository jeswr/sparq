import type { Metadata } from "next";
import { ShieldCheck } from "lucide-react";

import { SurfaceContent } from "@/components/surface-content";

export const metadata: Metadata = {
  title: "SHACL",
  description:
    "Validate RDF against SHACL shapes with the sparq engine — SHACL Core constraints, SHACL-SPARQL, custom constraint components, and the W3C validation report.",
};

export default function ShaclSurfacePage() {
  return (
    <SurfaceContent
      icon={ShieldCheck}
      title="SHACL"
      statement="Validate RDF data against shapes — SHACL Core, SHACL-SPARQL, and the W3C validation report."
      tier="live-new-wasm"
      extraBadge="Portability spike first"
      intro={
        <>
          <p>
            <code className="font-mono text-foreground">sparq-shacl</code> validates
            a <code className="font-mono">sparq_core::Graph</code> against SHACL
            shapes and produces a{" "}
            <strong className="text-foreground">W3C validation report</strong>{" "}
            (conformance plus per-violation results) as Turtle or human-readable
            text. It implements SHACL Core constraints, SHACL-SPARQL{" "}
            <code className="font-mono">sh:sparql</code> constraints, and custom
            SPARQL-based constraint components.
          </p>
          <p>
            Because it is pure Rust over <code className="font-mono">sparq-engine</code>
            , it is wasm-portable: a dedicated{" "}
            <code className="font-mono text-foreground">sparq-shacl-wasm</code>{" "}
            bundle can validate data + shapes in-tab, lazy-loaded only on this page.
          </p>
        </>
      }
      capabilities={[
        {
          title: "SHACL Core constraints",
          body: "class, datatype, cardinality, value ranges, node/property shapes, paths.",
        },
        {
          title: "Logical & qualified shapes",
          body: "and / or / not / xone, qualified value shapes, closed shapes, in / hasValue.",
        },
        {
          title: "SHACL-SPARQL (§5.2)",
          body: "sh:sparql constraints expressed as SPARQL SELECT over the engine.",
        },
        {
          title: "Custom constraint components (§6)",
          body: "Define new sh:ConstraintComponent backed by SPARQL.",
        },
        {
          title: "W3C validation report",
          body: "Conformance boolean + sh:result violations, as Turtle or human text.",
        },
        {
          title: "Path expressions",
          body: "Property paths in shape targets and constraints, reusing the engine's path evaluator.",
        },
      ]}
      runsNote={
        <>
          <p>
            Planned <strong className="text-foreground">live in your tab via a new
            wasm bundle</strong>. The validator is pure Rust over the engine, so it
            ports cleanly; the bundle is lazy-loaded on this page only to keep the
            landing page light.
          </p>
        </>
      }
      caveat={
        <>
          <p>
            SHACL-SPARQL constraints rely on the engine&rsquo;s REGEX, which is
            compiled out of the lean bundle — the SHACL bundle must include it (a
            size trade-off confirmed by a portability spike). Until that bundle
            ships, this surface is a captured-I/O walkthrough.
          </p>
        </>
      }
    />
  );
}
