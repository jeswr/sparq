import type { Metadata } from "next";
import { Boxes } from "lucide-react";

import { SurfaceContent } from "@/components/surface-content";
import { JavascriptWasmDemo } from "@/components/javascript-wasm-demo";
import { EsmImportSnippet } from "./esm-import-snippet";

export const metadata: Metadata = {
  title: "JavaScript / WASM",
  description:
    "Use the sparq engine from JavaScript/TypeScript via @jeswr/sparq — an idiomatic RDF/JS surface (SparqStore + a named Dataset DatasetCore entry, importable from a <script type=module>) over the ~886 KB WebAssembly build, in Node or the browser.",
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
            materialisation. The panels below call the API directly against one seeded
            store, in your tab.
          </p>
          <p>
            For an RDF/JS{" "}
            <a className="text-primary underline-offset-4 hover:underline" href="https://rdf.js.org/dataset-spec/" target="_blank" rel="noopener noreferrer">
              <code className="font-mono">DatasetCore</code>
            </a>{" "}
            surface, the package exports a named{" "}
            <code className="font-mono">Dataset</code> — importable directly in a{" "}
            <code className="font-mono">&lt;script type=&quot;module&quot;&gt;</code> from
            any ESM CDN. Its async factories (
            <code className="font-mono">Dataset.fromString</code> /{" "}
            <code className="font-mono">.create</code> /{" "}
            <code className="font-mono">.fromQuads</code>) instantiate the wasm engine on
            first use, so the ~MB binary loads lazily — never on import (see the snippet
            below).
          </p>
        </>
      }
      capabilities={[
        {
          title: "SparqStore (RDF/JS)",
          body: "fromString / fromCompressed, query() yielding Map-like Bindings, idiomatic terms.",
        },
        {
          title: "Dataset — RDF/JS DatasetCore",
          body: "Named ESM entry: add / delete / has / match / size / iterate, lazily wasm-initialised; .store drops to the full SPARQL surface.",
        },
        {
          title: "Streaming cursors",
          body: "Iterate large SELECT results in batches without materialising the whole table.",
        },
        {
          title: "count() without materialising",
          body: "Get a solution count, read from the index, without building every binding.",
        },
        {
          title: "RDF/JS match() / countQuads()",
          body: "The standard Source interface for triple-pattern matching over the store.",
        },
        {
          title: "applyDelta / SPARQL Update",
          body: "Apply quad-level deltas or SPARQL Update to mutate the store in place.",
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
          <p>
            The demo below seeds a six-person Turtle graph, then exercises{" "}
            <code className="font-mono">queryCursor()</code>,{" "}
            <code className="font-mono">match()</code>/
            <code className="font-mono">countQuads()</code>,{" "}
            <code className="font-mono">count()</code> and{" "}
            <code className="font-mono">applyDelta()</code> against it — all in your tab.
          </p>
        </>
      }
      caveat={
        <>
          <p>
            The <code className="font-mono">SparqStore</code> wrapper&apos;s{" "}
            <code className="font-mono">query()</code> covers SELECT/ASK; CONSTRUCT /
            DESCRIBE are on the wrapper via{" "}
            <code className="font-mono">queryQuads()</code> (RDF/JS quads),{" "}
            <code className="font-mono">queryQuadsString()</code> (N-Triples), and{" "}
            <code className="font-mono">queryQuadsStream()</code>; drop to the raw{" "}
            <code className="font-mono">Store</code> only to skip term
            materialisation. The lean bundle omits REGEX/REPLACE and the
            wall-clock query budget (see the SPARQL surface), trading a smaller
            download for those native-only features.
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
    >
      <JavascriptWasmDemo />
      <EsmImportSnippet />
    </SurfaceContent>
  );
}
