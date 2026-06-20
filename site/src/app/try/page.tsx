import type { Metadata } from "next";

// [OPUS-4.8] sq-4296 (#935 / #981) — code-split the REPL out of the /try route's first-load
// bundle (see src/components/repl-lazy.tsx) so the page shell renders before the heavy REPL
// chunk + its idle-deferred wasm engine load.
import { ReplLazy } from "@/components/repl-lazy";

export const metadata: Metadata = {
  title: "Live SPARQL REPL",
  description:
    "Run real SPARQL queries with the sparq engine compiled to WebAssembly — live in your browser tab by default — or connect the same editor to a running sparq-server over the SPARQL 1.1 Protocol.",
};

export default function TryPage() {
  return (
    <div className="space-y-6">
      <header className="space-y-2">
        <h1 className="text-2xl font-semibold">Live SPARQL REPL</h1>
        <p className="measure text-muted-foreground">
          The shared engine component. Edit the query, pick an example, and run —
          by default every query executes against the sample graph using the real sparq
          Rust engine compiled to WebAssembly, entirely in your browser tab with nothing
          sent to a server. The lean bundle ships the parser, triplestore and SPARQL
          1.1/1.2 engine: SELECT / ASK / CONSTRUCT / DESCRIBE and SPARQL Update
          (INSERT / DELETE, mutating the in-tab store), over BGP, FILTER, OPTIONAL, UNION,
          MINUS, BIND, VALUES, aggregates, property paths, sub-SELECT and RDF 1.2 triple terms. Switch
          to the EXPLAIN / ANALYZE mode to inspect the query plan.
        </p>
        <p className="measure text-muted-foreground">
          The <span className="font-medium text-foreground">Connect</span> panel runs the
          same editor against any running sparq-server over the SPARQL 1.1 Protocol —
          enter an endpoint URL and an optional bearer token. The token is sent only in the
          HTTP <code className="font-mono text-[0.9em]">Authorization</code> header; the
          panel surfaces honest connection-safety warnings (a token over plaintext to a
          non-loopback host, a cross-origin request the server must opt in via CORS, a
          SERVICE clause the server refuses by default) and never bypasses a server gate.
        </p>
      </header>
      <ReplLazy />
    </div>
  );
}
