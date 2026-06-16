import type { Metadata } from "next";
import { Boxes } from "lucide-react";

import { SurfaceContent } from "@/components/surface-content";

export const metadata: Metadata = {
  title: "JavaScript / WASM",
  description:
    "Use the sparq engine from JavaScript/TypeScript via @jeswr/sparq — an idiomatic RDF/JS surface over the ~886 KB WebAssembly build, in Node or the browser.",
};

export default function JavascriptWasmSurfacePage() {
  return (
    <SurfaceContent
      icon={Boxes}
      title="JavaScript / WASM"
      statement="The @jeswr/sparq RDF/JS API — the Rust engine compiled to a single ~886 KB wasm artifact."
      tier="live"
      intro={
        <>
          <p>
            sparq is a Rust RDF triplestore + SPARQL engine compiled to a single{" "}
            <strong className="text-foreground">~886 KB</strong> (≈314 KB gzipped)
            WebAssembly artifact. The npm package{" "}
            <code className="font-mono text-foreground">@jeswr/sparq</code> wraps it
            in an idiomatic <a className="text-primary underline-offset-4 hover:underline" href="https://rdf.js.org/" target="_blank" rel="noopener noreferrer">RDF/JS</a>{" "}
            surface — <code className="font-mono">SparqStore</code>, Map-like{" "}
            <code className="font-mono">Bindings</code>, spec terms via{" "}
            <code className="font-mono">@rdfjs/types</code> — and runs unchanged in
            Node &ge; 18 and the browser.
          </p>
          <p>
            This very site uses it: the live REPL constructs a{" "}
            <code className="font-mono">Store</code>, loads Turtle, and runs your
            SPARQL — all in the tab. A thin raw <code className="font-mono">Store</code>{" "}
            class is available if you want SPARQL-JSON strings with no JS-side term
            materialisation.
          </p>
        </>
      }
      capabilities={[
        {
          title: "SparqStore (RDF/JS)",
          body: "fromString / fromCompressed, query() yielding Map-like Bindings, idiomatic terms.",
        },
        {
          title: "Streaming cursors",
          body: "Iterate large SELECT results without materialising the whole table.",
        },
        {
          title: "count() without materialising",
          body: "Get a result count without building every binding.",
        },
        {
          title: "RDF/JS match() / countQuads()",
          body: "The standard Source interface for pattern matching over the store.",
        },
        {
          title: "applyDelta / SPARQL Update",
          body: "Apply quad deltas or SPARQL Update to mutate the store.",
        },
        {
          title: "Raw Store class",
          body: "SPARQL-JSON strings + CONSTRUCT / DESCRIBE, skipping JS term materialisation.",
        },
      ]}
      runsNote={
        <>
          <p>
            <strong className="text-foreground">This is the engine.</strong> Every
            live demo on this site calls this API. In Node it loads the same wasm
            binary; in the browser it streams it on first use.
          </p>
        </>
      }
      caveat={
        <>
          <p>
            The wrapper does not yet expose CONSTRUCT / DESCRIBE — drop to the raw{" "}
            <code className="font-mono">Store</code> for those. The lean bundle
            omits REGEX/REPLACE and the wall-clock query budget (see the SPARQL
            surface), trading a smaller download for those native-only features.
          </p>
        </>
      }
      links={[
        { href: "/try", label: "Open the live REPL" },
        {
          href: "https://www.npmjs.com/package/@jeswr/sparq",
          label: "@jeswr/sparq on npm",
          external: true,
        },
      ]}
    />
  );
}
