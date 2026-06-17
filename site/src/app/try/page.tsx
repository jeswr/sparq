import type { Metadata } from "next";

import { Repl } from "@/components/repl";

export const metadata: Metadata = {
  title: "Live SPARQL REPL",
  description:
    "Run real SPARQL queries against a sample RDF graph using the sparq engine compiled to WebAssembly — live in your browser tab, nothing sent to a server.",
};

export default function TryPage() {
  return (
    <div className="space-y-6">
      <header className="space-y-2">
        <h1 className="text-2xl font-semibold">Live SPARQL REPL</h1>
        <p className="measure text-muted-foreground">
          The shared engine component. Edit the query, pick an example, and run —
          every query executes against the sample graph using the real sparq Rust
          engine compiled to WebAssembly. The lean bundle ships the parser,
          triplestore and SPARQL 1.1/1.2 engine: SELECT / ASK / CONSTRUCT / DESCRIBE
          and SPARQL Update (INSERT / DELETE, mutating the in-tab store), over BGP,
          FILTER, OPTIONAL, UNION, MINUS, BIND, VALUES, aggregates, property paths,
          sub-SELECT and RDF-star. Switch to the EXPLAIN / ANALYZE mode to inspect
          the query plan.
        </p>
      </header>
      <Repl />
    </div>
  );
}
