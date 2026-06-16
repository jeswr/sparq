import type { Metadata } from "next";
import { Database } from "lucide-react";

import { SurfaceContent } from "@/components/surface-content";

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
          body: "gzip / zstd RDF dumps decode-and-load; in the browser via SparqStore.fromCompressed().",
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
            four text formats and gzip/zstd-compressed ingest — the same loaders
            that ship in <code className="font-mono">@jeswr/sparq</code>.
          </p>
        </>
      }
      caveat={
        <>
          <p>
            HDT loading, the mmap / external-memory ingest path, and the
            fully-parallel fast loaders are <strong className="text-foreground">native-only</strong>
            ; in the browser they fall back to the in-memory streaming loader.
          </p>
        </>
      }
      links={[{ href: "/try", label: "Load data in the REPL" }]}
    />
  );
}
