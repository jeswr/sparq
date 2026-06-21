import type { Metadata } from "next";
import { Database } from "lucide-react";

import { SurfaceContent } from "@/components/surface-content";
import { DataFormatsDemo } from "@/components/data-formats-demo";

export const metadata: Metadata = {
  title: "Data formats",
  description:
    "Parse and load RDF into a sparq Graph — Turtle, N-Triples, N-Quads, TriG, JSON-LD, compressed dumps, and the binary HDT archive format.",
};

// [OPUS-4.8] sq-vw3ax.4 — scan-first deep page: statement + the format-picker demo
// first, then a tight capabilities list, a one-sentence "how this runs", caveats
// closed.
export default function DataFormatsSurfacePage() {
  return (
    <SurfaceContent
      icon={Database}
      title="Data formats"
      statement="Getting RDF into a sparq Graph — the four text formats plus JSON-LD parse live in your tab; compressed ingest and HDT load natively."
      tier="live"
      capabilities={[
        {
          term: "Turtle / N-Triples / N-Quads / TriG",
          body: "All four text formats, with named-graph preservation for the quad formats.",
        },
        {
          term: "JSON-LD 1.1",
          body: "JSON for linked data — parsed via the opt-in jsonld feature (oxjsonld), which the site bundle enables.",
        },
        {
          term: "Compressed ingest",
          body: "gzip / zstd / bzip2 dumps decode-and-load natively (fused decompress + parallel parse); gzip also decodes live in the browser.",
        },
        {
          term: "HDT archives",
          body: "Load .hdt and content-sniffed .hdt.gz/.zst/.bz2 via the opt-in sparq-hdt crate (native).",
        },
        {
          term: "RDF writer matrix",
          body: "Serialize back out to Turtle / TriG / N-Quads / JSON-LD (engine serialize-rdf feature; the N-Triples writer is always on).",
        },
      ]}
      runsNote={
        <>
          Runs live in your tab for the four text formats and JSON-LD — the demo calls
          the same <code className="font-mono">Store.load</code> /{" "}
          <code className="font-mono">loadDataset</code> loaders that ship in{" "}
          <code className="font-mono">@jeswr/sparq</code>, and gzip decodes with the
          browser&apos;s native <code className="font-mono">DecompressionStream</code>,
          with no network round-trip. Reproduce:{" "}
          <code className="font-mono">cargo test -p sparq-core</code>.
        </>
      }
      caveat={
        <p>
          Only <strong className="text-foreground">gzip</strong> decodes in the browser
          (via <code className="font-mono">DecompressionStream</code>);{" "}
          <strong className="text-foreground">zstd</strong> and{" "}
          <strong className="text-foreground">bzip2</strong> ingest, HDT loading, the
          mmap / external-memory path, and the fully-parallel fast loaders are{" "}
          <strong className="text-foreground">native-only</strong> — in the browser the
          in-memory streaming loader is used.
        </p>
      }
      links={[{ href: "/try", label: "Open the full SPARQL REPL" }]}
    >
      <DataFormatsDemo />
    </SurfaceContent>
  );
}
