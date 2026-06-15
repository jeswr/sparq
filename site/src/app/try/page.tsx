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
          triplestore and SPARQL 1.1 engine (SELECT / ASK with BGP, FILTER,
          OPTIONAL, UNION, MINUS, BIND, VALUES, aggregates, property paths,
          sub-SELECT and RDF-star).
        </p>
      </header>
      <Repl />
    </div>
  );
}
