import type { Metadata } from "next";
import { FileCode2 } from "lucide-react";

import { SurfaceContent } from "@/components/surface-content";
import { ReplLazy } from "@/components/repl-lazy";

export const metadata: Metadata = {
  title: "SPARQL 1.1 / 1.2",
  description:
    "SELECT / ASK / CONSTRUCT / UPDATE, property paths, RDF 1.2 triple terms, aggregates, subqueries — the sparq SPARQL engine, live in your tab.",
};

// [OPUS-4.8] sq-vw3ax.4 — scan-first deep page: statement + the live REPL demo
// first, then a tight capabilities list, a one-sentence "how this runs", caveats
// closed. The REPL mounts via ReplLazy so the heavy chunk loads after the shell.
export default function SparqlSurfacePage() {
  return (
    <SurfaceContent
      icon={FileCode2}
      title="SPARQL 1.1 / 1.2"
      statement="The full query language — SELECT, ASK, CONSTRUCT, UPDATE — over a worst-case-optimal-join engine, live in your tab."
      tier="live"
      capabilities={[
        {
          term: "All four query forms",
          body: "SELECT / ASK / CONSTRUCT / DESCRIBE, plus SPARQL Update (INSERT / DELETE / DELETE…WHERE) that mutates the in-tab store.",
        },
        {
          term: "The algebra you expect",
          body: "BGPs, FILTER, OPTIONAL, UNION, MINUS, BIND, VALUES, aggregates + GROUP BY / HAVING, and sub-SELECT.",
        },
        {
          term: "Property paths",
          body: "Sequence, alternative, inverse, and the transitive *, +, ? operators.",
        },
        {
          term: "RDF 1.2 triple terms",
          body: "Triple terms in object position, written <<( s p o )>>, queried with the triple-term functions.",
        },
        {
          term: "EXPLAIN / EXPLAIN ANALYZE",
          body: "Inspect the planner's join order and cardinality estimates; ANALYZE also runs and appends a per-operator row-count trace.",
        },
      ]}
      runsNote={
        <>
          Runs live in your tab via the lean wasm bundle — the same WCOJ engine the
          native crate ships, with no network round-trip. Reproduce:{" "}
          <code className="font-mono">cargo test -p sparq-engine</code>.
        </>
      }
      caveat={
        <p>
          REGEX / REPLACE are compiled out of the lean wasm bundle to keep it small
          (native-only). The wall-clock query-budget deadline is native-only too — the
          wasm build enforces a row cap instead, and{" "}
          <code className="font-mono">EXPLAIN ANALYZE</code> reports exact per-operator
          row counts but zero wall-times (no monotonic clock under wasm32).
        </p>
      }
      links={[{ href: "/try", label: "Open the full REPL" }]}
    >
      <ReplLazy />
    </SurfaceContent>
  );
}
