import type { Metadata } from "next";
import { Database } from "lucide-react";

import { SurfaceContent } from "@/components/surface-content";
import { DataFormatsDemo } from "@/components/data-formats-demo";

export const metadata: Metadata = {
  title: "Data formats",
  description:
    "Parse and load RDF into a sparq Graph — Turtle, N-Triples, N-Quads, TriG, compressed dumps, and the binary HDT archive format.",
};

export default function DataFormatsSurfacePage() {
  return (
    <SurfaceContent
      icon={Database}
      title="Data formats"
      statement="Getting RDF into a sparq Graph: the four text formats, compressed ingest, and HDT."
      tier="live"
      intro={
        <>
          <p>
            sparq parses the four standard RDF text formats —{" "}
            <strong className="text-foreground">Turtle, N-Triples, N-Quads</strong>{" "}
            and <strong className="text-foreground">TriG</strong> — in{" "}
            <code className="font-mono text-foreground">sparq-core</code>, with
            in-memory, streaming, parallel and external-memory loaders for large
            dumps. The binary{" "}
            <strong className="text-foreground">HDT</strong> archive format (and its
            content-sniffed <code className="font-mono">.hdt.gz</code> /{" "}
            <code className="font-mono">.hdt.zst</code> /{" "}
            <code className="font-mono">.hdt.bz2</code> variants) lives in the
            opt-in <code className="font-mono text-foreground">sparq-hdt</code>{" "}
            crate.
          </p>
          <p>
            These crates parse RDF <em>in</em>; to write RDF <em>out</em>, the
            engine ships a writer matrix (Turtle / TriG / N-Quads / JSON-LD 1.1)
            behind its <code className="font-mono">serialize-rdf</code> feature,
            with the N-Triples writer always on.
          </p>
        </>
      }
      capabilities={[
        {
          title: "Turtle / N-Triples / N-Quads / TriG",
          body: "All four text formats, with named-graph preservation for the quad formats.",
        },
        {
          title: "Compressed ingest",
          body: "gzip / zstd / bzip2 RDF dumps decode-and-load natively (fused decompress + parallel parse); gzip also decodes live in the browser.",
        },
        {
          title: "Streaming & parallel loaders",
          body: "Chunk-parallel parsing for large N-Triples dumps; external-memory ingest (native).",
        },
        {
          title: "HDT archives",
          body: "Load .hdt and content-sniffed .hdt.gz/.zst/.bz2 via the opt-in sparq-hdt crate.",
        },
        {
          title: "Copy-on-write snapshots",
          body: "Cheap immutable Graph snapshots for concurrent serving.",
        },
        {
          title: "RDF writer matrix",
          body: "Serialize back out to Turtle / TriG / N-Quads / JSON-LD (engine serialize-rdf feature).",
        },
      ]}
      runsNote={
        <>
          <p>
            <strong className="text-foreground">Live in your tab</strong> for the
            four text formats — the demo above runs the same{" "}
            <code className="font-mono">Store.load</code> /{" "}
            <code className="font-mono">loadDataset</code> loaders that ship in{" "}
            <code className="font-mono">@jeswr/sparq</code>, compiled to wasm. The
            gzip-ingest panel decodes with the browser&apos;s native{" "}
            <code className="font-mono">DecompressionStream</code> before parsing —
            no codec library, no server.
          </p>
        </>
      }
      caveat={
        <>
          <p>
            Only <strong className="text-foreground">gzip</strong> decodes in the
            browser (via <code className="font-mono">DecompressionStream</code>);{" "}
            <strong className="text-foreground">zstd</strong> and{" "}
            <strong className="text-foreground">bzip2</strong> ingest, HDT loading,
            the mmap / external-memory path, and the fully-parallel fast loaders are{" "}
            <strong className="text-foreground">native-only</strong> — in the
            browser the in-memory streaming loader is used.
          </p>
        </>
      }
      links={[{ href: "/try", label: "Open the full SPARQL REPL" }]}
    >
      <DataFormatsDemo />
    </SurfaceContent>
  );
}
