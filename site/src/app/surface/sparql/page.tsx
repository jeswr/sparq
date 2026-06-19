import type { Metadata } from "next";
import { FileCode2 } from "lucide-react";

import { SurfaceContent } from "@/components/surface-content";

export const metadata: Metadata = {
  title: "SPARQL 1.1 / 1.2",
  description:
    "SELECT / ASK / CONSTRUCT / UPDATE, property paths, RDF 1.2 triple terms, aggregates, subqueries — the sparq SPARQL engine, live in your tab.",
};

export default function SparqlSurfacePage() {
  return (
    <SurfaceContent
      icon={FileCode2}
      title="SPARQL 1.1 / 1.2"
      statement="The full query language — SELECT, ASK, CONSTRUCT, UPDATE — over the sparq engine."
      tier="live"
      intro={
        <>
          <p>
            sparq evaluates SPARQL 1.1/1.2 over a{" "}
            <code className="font-mono text-foreground">sparq_core::Graph</code>{" "}
            using a worst-case-optimal join (WCOJ) engine. The same engine that
            powers the native crate is compiled to WebAssembly and ships as{" "}
            <code className="font-mono text-foreground">@jeswr/sparq</code>, so you
            can run real queries in this tab — nothing is sent to a server.
          </p>
          <p>
            The query surface covers the algebra you would expect — basic graph
            patterns, FILTER, OPTIONAL, UNION, MINUS, BIND, VALUES, aggregates and
            GROUP BY, sub-SELECT, and property paths — plus RDF 1.2 triple terms
            and named-graph dataset views.
          </p>
        </>
      }
      capabilities={[
        {
          title: "SELECT / ASK / CONSTRUCT / DESCRIBE",
          body: "All four query forms, returning bindings, a boolean, or a constructed graph.",
        },
        {
          title: "SPARQL Update",
          body: "INSERT / DELETE / DELETE…WHERE and graph management, mutating the store in place.",
        },
        {
          title: "Property paths",
          body: "Sequence, alternative, inverse, and the transitive *, +, ? operators.",
        },
        {
          title: "RDF 1.2 triple terms",
          body: "Triple terms in object position, written <<( s p o )>> and queried with the triple-term functions.",
        },
        {
          title: "Aggregates & subqueries",
          body: "COUNT / SUM / AVG / MIN / MAX / GROUP_CONCAT, GROUP BY / HAVING, and nested SELECT.",
        },
        {
          title: "Extension functions",
          body: "Register custom SPARQL functions via the engine's FunctionRegistry (native).",
        },
      ]}
      runsNote={
        <>
          <p>
            <strong className="text-foreground">Live in your browser tab.</strong>{" "}
            The lean wasm bundle carries the parser, triplestore and SPARQL engine.
            The live REPL runs your queries against the sample graph with no network
            round-trip — SELECT and ASK render a table / boolean, CONSTRUCT and
            DESCRIBE render the constructed N-Triples, and SPARQL Update
            (INSERT / DELETE) mutates the in-tab store.
          </p>
          <p>
            The same REPL has an{" "}
            <code className="font-mono text-foreground">EXPLAIN</code> /{" "}
            <code className="font-mono text-foreground">EXPLAIN ANALYZE</code> mode:
            EXPLAIN shows the planner&apos;s join order and cardinality estimates
            without executing, ANALYZE also runs the query and appends a per-operator
            row-count trace.
          </p>
        </>
      }
      caveat={
        <>
          <p>
            REGEX / REPLACE are compiled out of the lean wasm bundle to keep it
            small; they are available in the native build. The query-budget
            deadline (wall-clock timeout) is native-only — the wasm bundle enforces
            a row cap instead. In the wasm build,{" "}
            <code className="font-mono">EXPLAIN ANALYZE</code> reports exact
            per-operator row counts but zero wall-times (no monotonic clock under
            wasm32).
          </p>
        </>
      }
      links={[{ href: "/try", label: "Open the live REPL" }]}
    />
  );
}
