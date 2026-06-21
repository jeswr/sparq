import type { Metadata } from "next";
import { Boxes } from "lucide-react";

import { SurfaceContent } from "@/components/surface-content";
import { JavascriptWasmDemo } from "@/components/javascript-wasm-demo";

export const metadata: Metadata = {
  title: "JavaScript / WASM",
  description:
    "Use the sparq engine from JavaScript/TypeScript via @jeswr/sparq — an idiomatic RDF/JS surface over the ~886 KB WebAssembly build, in Node or the browser.",
};

// [OPUS-4.8] sq-vw3ax.4 — scan-first deep page: statement + the RDF/JS API demo
// first, then a tight capabilities list, a one-sentence "how this runs", caveats
// closed.
export default function JavascriptWasmSurfacePage() {
  return (
    <SurfaceContent
      icon={Boxes}
      title="JavaScript / WASM"
      statement="The @jeswr/sparq RDF/JS API — the Rust engine compiled to a single ~886 KB wasm artifact, running unchanged in Node and the browser."
      tier="live"
      capabilities={[
        {
          term: "SparqStore (RDF/JS)",
          body: "fromString / fromCompressed, query() yielding Map-like Bindings, idiomatic spec terms via @rdfjs/types.",
        },
        {
          term: "Streaming cursors + count()",
          body: "Iterate large SELECT results in batches without materialising the whole table; count() reads from the index without building bindings.",
        },
        {
          term: "RDF/JS match() / countQuads()",
          body: "The standard Source interface for triple-pattern matching over the store.",
        },
        {
          term: "applyDelta / SPARQL Update",
          body: "Apply quad-level deltas or SPARQL Update to mutate the store in place.",
        },
        {
          term: "Raw Store class",
          body: "SPARQL-JSON strings + CONSTRUCT / DESCRIBE, skipping JS-side term materialisation.",
        },
      ]}
      runsNote={
        <>
          This is the engine — every live demo on this site calls this API. In Node it
          loads the same wasm binary; in the browser it streams it on first use, with no
          network round-trip beyond fetching the artifact. Reproduce:{" "}
          <code className="font-mono">npm install @jeswr/sparq</code>.
        </>
      }
      caveat={
        <p>
          The <code className="font-mono">SparqStore</code> wrapper&apos;s{" "}
          <code className="font-mono">query()</code> covers SELECT / ASK; CONSTRUCT /
          DESCRIBE are on the wrapper via{" "}
          <code className="font-mono">queryQuads()</code> (RDF/JS quads),{" "}
          <code className="font-mono">queryQuadsString()</code> (N-Triples) and{" "}
          <code className="font-mono">queryQuadsStream()</code> — drop to the raw{" "}
          <code className="font-mono">Store</code> only to skip term materialisation. The
          lean bundle omits REGEX / REPLACE and the wall-clock query budget (see the
          SPARQL surface), trading a smaller download for those native-only features.
        </p>
      }
      links={[
        { href: "/try", label: "Open the live REPL" },
        {
          href: "https://www.npmjs.com/package/@jeswr/sparq",
          label: "@jeswr/sparq on npm",
          external: true,
        },
      ]}
    >
      <JavascriptWasmDemo />
    </SurfaceContent>
  );
}
