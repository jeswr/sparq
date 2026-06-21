import type { Metadata } from "next";
import { ShieldCheck } from "lucide-react";

import { SurfaceContent } from "@/components/surface-content";
import { ShaclPlayground } from "@/components/shacl-playground";

export const metadata: Metadata = {
  title: "SHACL",
  description:
    "Validate RDF against SHACL shapes with the sparq engine — SHACL Core constraints, SHACL-SPARQL, custom constraint components, the W3C validation report, and a SHACL Compact Syntax view, live in your tab.",
};

// [OPUS-4.8] sq-vw3ax.4 — scan-first deep page: statement + the SHACL playground
// first, then a tight capabilities list, a one-sentence "how this runs", caveats
// closed.
export default function ShaclSurfacePage() {
  return (
    <SurfaceContent
      icon={ShieldCheck}
      title="SHACL"
      statement="Validate RDF against shapes — SHACL Core, SHACL-SPARQL, the W3C validation report and a live SHACL Compact Syntax view, in your tab."
      tier="live"
      capabilities={[
        {
          term: "SHACL Core constraints",
          body: "class, datatype, cardinality, value ranges, node/property shapes, paths, plus and / or / not / xone, qualified value shapes, closed shapes, in / hasValue.",
        },
        {
          term: "SHACL-SPARQL (§5.2)",
          body: "sh:sparql constraints expressed as SPARQL SELECT over the engine.",
        },
        {
          term: "Custom constraint components (§6)",
          body: "Define new sh:ConstraintComponent backed by SPARQL.",
        },
        {
          term: "W3C validation report",
          body: "Conformance boolean + sh:result violations, rendered as Turtle or human text.",
        },
        {
          term: "SHACL Compact Syntax (display)",
          body: "Render the shapes graph in the W3C compact notation — node/property shapes, paths, counts and value constraints — beside the report.",
        },
      ]}
      runsNote={
        <>
          Runs live in your tab via the SHACL-enabled wasm bundle (the published{" "}
          <code className="font-mono">@jeswr/sparq</code>) — it calls{" "}
          <code className="font-mono">Store.validate(data, shapes)</code> and renders the
          conformance flag plus the W3C report, with no network round-trip. Reproduce:{" "}
          <code className="font-mono">cargo test -p sparq-shacl</code>.
        </>
      }
      caveat={
        <>
          <p>
            The in-tab validator is sized for small documents (~10–100 triples);
            validate large graphs server-side via the{" "}
            <code className="font-mono">sparq-server</code> HTTP{" "}
            <code className="font-mono">validate</code> path.
          </p>
          <p>
            The <strong className="text-foreground">Compact syntax</strong> view is the{" "}
            <em>display</em> direction only (shapes &rarr; compact) and best-effort:
            SHACL Compact Syntax has no form for logical constraints (
            <code className="font-mono">sh:and</code> /{" "}
            <code className="font-mono">sh:or</code> /{" "}
            <code className="font-mono">sh:xone</code> /{" "}
            <code className="font-mono">sh:not</code>) or shape references (
            <code className="font-mono">sh:node</code>), which the view lists explicitly
            rather than dropping. The <em>parse</em> direction (compact text &rarr;
            shapes) belongs in the <code className="font-mono">sparq-shacl</code> engine
            and is tracked separately.
          </p>
        </>
      }
    >
      <ShaclPlayground />
    </SurfaceContent>
  );
}
