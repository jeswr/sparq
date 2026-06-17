import type { Metadata } from "next";
import { ShieldCheck } from "lucide-react";

import { SurfaceContent } from "@/components/surface-content";
import { ShaclPlayground } from "@/components/shacl-playground";

export const metadata: Metadata = {
  title: "SHACL",
  description:
    "Validate RDF against SHACL shapes with the sparq engine — SHACL Core constraints, SHACL-SPARQL, custom constraint components, and the W3C validation report, live in your tab.",
};

export default function ShaclSurfacePage() {
  return (
    <SurfaceContent
      icon={ShieldCheck}
      title="SHACL"
      statement="Validate RDF data against shapes — SHACL Core, SHACL-SPARQL, and the W3C validation report."
      tier="live"
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
            , it is wasm-portable. The SHACL-enabled wasm bundle — the same one the
            published <code className="font-mono text-foreground">@jeswr/sparq</code>{" "}
            ships — runs the validator in this tab: paste data + shapes below and the
            conformance flag and per-violation report come back with no network
            round-trip.
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
            <strong className="text-foreground">Live in your browser tab.</strong>{" "}
            The validator is pure Rust over the engine, compiled to wasm in the
            SHACL-enabled bundle (the published{" "}
            <code className="font-mono">@jeswr/sparq</code> ships it). The playground
            calls the bundle&rsquo;s{" "}
            <code className="font-mono text-foreground">Store.validate(data, shapes)</code>{" "}
            binding, parses the two graphs, and renders the conformance flag plus the
            per-violation W3C report — nothing is sent to a server.
          </p>
        </>
      }
      caveat={
        <>
          <p>
            SHACL-SPARQL constraints rely on the engine&rsquo;s REGEX. The lean SPARQL
            REPL bundle compiles REGEX out, but the SHACL bundle keeps it, so{" "}
            <code className="font-mono">sh:sparql</code> constraints validate in-tab
            too. The in-tab validator is sized for small documents (~10–100 triples);
            validate large graphs server-side via the{" "}
            <code className="font-mono">sparq-server</code> HTTP{" "}
            <code className="font-mono">validate</code> path.
          </p>
        </>
      }
    >
      <ShaclPlayground />
    </SurfaceContent>
  );
}
