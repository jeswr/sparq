import type { Metadata } from "next";
import { Database } from "lucide-react";

import { SurfaceContent } from "@/components/surface-content";
import { DataFormatsDemo } from "@/components/data-formats-demo";
import { withBasePath } from "@/lib/base-path";

export const metadata: Metadata = {
  title: "Data formats",
  description:
    "Parse and load RDF into a sparq Graph — Turtle, N-Triples, N-Quads, TriG, JSON-LD, compressed dumps, and the binary HDT archive format.",
};

// [OPUS-4.8] sq-vw3ax.8 — condensed to the scan-first template: demo first, one-line lead,
// ≤5 capabilities, single-sentence runs note + reproduce, caveats behind <details>, plus the
// universal README + SKILL.md links. The full loader/writer matrix lives in the SKILL.
export default function DataFormatsSurfacePage() {
  return (
    <SurfaceContent
      icon={Database}
      title="Data formats"
      statement="Getting RDF into a sparq Graph: the four text formats, compressed ingest, and HDT."
      tier="live"
      intro={
        <p>
          sparq parses the four standard RDF text formats (Turtle / N-Triples / N-Quads /
          TriG) and JSON-LD 1.1 in <code className="font-mono text-foreground">sparq-core</code>,
          with the binary HDT archive format in the opt-in{" "}
          <code className="font-mono text-foreground">sparq-hdt</code> crate; the picker
          above runs the same loaders the published wasm bundle ships.
        </p>
      }
      capabilities={[
        {
          title: "Turtle / N-Triples / N-Quads / TriG",
          body: "All four text formats, with named-graph preservation for the quad formats.",
        },
        {
          title: "JSON-LD 1.1",
          body: "JSON for linked data — parsed via the opt-in jsonld feature (oxjsonld), which the site bundle enables.",
        },
        {
          title: "Compressed & streaming ingest",
          body: "gzip / zstd / bzip2 dumps decode-and-load natively (fused decompress + parallel parse); gzip also decodes live in the browser.",
        },
        {
          title: "HDT archives",
          body: "Load .hdt and content-sniffed .hdt.gz/.zst/.bz2 via the opt-in sparq-hdt crate.",
        },
        {
          title: "RDF writer matrix",
          body: "Serialize back out to Turtle / TriG / N-Quads / JSON-LD (engine serialize-rdf feature; N-Triples always on).",
        },
      ]}
      runsNote="Live in your tab for the four text formats and JSON-LD — the demo runs the same loaders that ship in @jeswr/sparq, and the gzip panel decodes with the browser's native DecompressionStream before parsing, with no codec library and no server."
      reproduce="cargo test -p sparq-core"
      caveat={
        <p>
          Only <strong className="text-foreground">gzip</strong> decodes in the browser
          (via <code className="font-mono">DecompressionStream</code>);{" "}
          <strong className="text-foreground">zstd</strong> and{" "}
          <strong className="text-foreground">bzip2</strong> ingest, HDT loading, the
          mmap / external-memory path, and the fully-parallel fast loaders are
          native-only — in the browser the in-memory streaming loader is used.
        </p>
      }
      readmeHref="https://github.com/jeswr/sparq/tree/main/crates/sparq-core"
      skillHref="https://github.com/jeswr/sparq/blob/main/skills/data-formats/SKILL.md"
      links={[{ href: withBasePath("/app/"), label: "Open the SPARQL workbench", external: true }]}
    >
      <DataFormatsDemo />
    </SurfaceContent>
  );
}
