import type { Metadata } from "next";
import { Server } from "lucide-react";

import { SurfaceContent } from "@/components/surface-content";
import { HttpServerWalkthrough } from "@/components/http-server-walkthrough";

export const metadata: Metadata = {
  title: "HTTP server",
  description:
    "sparq-server — a W3C-conformant SPARQL 1.1 Protocol + Graph Store HTTP endpoint with content negotiation, EXPLAIN, Prometheus /metrics, and live WebSocket / SSE subscriptions. This page replays REAL captured curl + SSE-frame I/O, including a subscription firing on a committed UPDATE.",
};

export default function HttpServerSurfacePage() {
  return (
    <SurfaceContent
      icon={Server}
      title="HTTP server"
      statement="A W3C SPARQL 1.1 Protocol + Graph Store HTTP endpoint with content negotiation, EXPLAIN, Prometheus metrics, and live WebSocket / SSE subscriptions."
      tier="walkthrough"
      extraBadge="Native-only · captured I/O"
      intro={
        <>
          <p>
            <code className="font-mono text-foreground">sparq-server</code> is a
            W3C-conformant HTTP server (axum / tokio) that exposes the sparq query engine
            over a <code className="font-mono">sparq_core::Graph</code> — in-memory by
            default, or <strong className="text-foreground">durable on disk</strong> with{" "}
            <code className="font-mono">--persist&nbsp;DIR</code>. It implements the{" "}
            <strong className="text-foreground">SPARQL 1.1 Protocol</strong> (
            <code className="font-mono">query</code> + <code className="font-mono">update</code>{" "}
            at <code className="font-mono">/sparql</code>) and the{" "}
            <strong className="text-foreground">Graph Store HTTP Protocol</strong> (read{" "}
            <em>and</em> write), with <code className="font-mono">Accept</code>-driven content
            negotiation, hardening guards, Prometheus{" "}
            <code className="font-mono">/metrics</code>, and{" "}
            <strong className="text-foreground">WebSocket + SSE subscriptions</strong>.
          </p>
          <p>
            The headline below is the live <strong className="text-foreground">subscription</strong>:
            open an SSE (or WebSocket) stream for a <code className="font-mono">SELECT</code>,
            receive a full snapshot, then <code className="font-mono">POST</code> a SPARQL{" "}
            <code className="font-mono">UPDATE</code> and watch the <em>same</em> stream push an
            incremental diff — the result set updating itself the instant a write commits.
          </p>
          <p>
            The server stack is <strong className="text-foreground">native-only</strong> (axum /
            tokio / <code className="font-mono">std::net</code>) and deliberately absent from the
            lean wasm bundle, and this static site has no backend to host it. So — honestly — this
            page is a <strong className="text-foreground">captured-I/O walkthrough</strong>: every
            curl request and every SSE frame here is <em>real recorded output</em> from a running{" "}
            <code className="font-mono">sparq-server</code>, replayed in your tab. Run the same
            binary yourself and you get byte-for-byte the same responses.
          </p>
        </>
      }
      capabilities={[
        {
          title: "SPARQL 1.1 Protocol query + update",
          body: "GET (URL param) or POST (direct body / url-encoded form) at /sparql; updates are atomic — failure → 400 with no partial effect, success → 204.",
        },
        {
          title: "Accept-driven content negotiation",
          body: "SELECT/ASK as JSON / XML / CSV / TSV; CONSTRUCT/DESCRIBE + Graph-Store reads as N-Triples / prefix-compacting Turtle / RDF/XML — q-value aware.",
        },
        {
          title: "Graph Store HTTP Protocol (read + write)",
          body: "GET/HEAD read a graph; PUT replaces (201/204), POST merges, DELETE drops — bodies parsed by Content-Type, routed through the same atomic writer as UPDATE.",
        },
        {
          title: "Live WebSocket + SSE subscriptions",
          body: "Subscribe to a SELECT; get a sequence-0 snapshot then per-commit added/removed diffs. Both transports share one registry; close the stream to unsubscribe.",
        },
        {
          title: "EXPLAIN + Prometheus /metrics",
          body: "?explain=true returns the join plan with cardinality estimates (no execution); /metrics exposes triple count, applied-update counter and active-subscription gauge.",
        },
        {
          title: "Hardened by default",
          body: "Loopback-only bind, optional Bearer write/read gate, per-request timeouts, body / concurrency / decompress caps, slow-loris guards, and a fixed security-header set on every response.",
        },
      ]}
      runsNote={
        <>
          <p>
            <strong className="text-foreground">Captured I/O replay.</strong> The transcript and
            every curl recipe on this page were recorded verbatim from a real{" "}
            <code className="font-mono">sparq-server --format turtle</code> over a tiny seed
            graph. The static Pages site has no backend, so nothing here calls out to a network —
            the replay is deterministic and offline. To run it for yourself:
          </p>
          <pre className="overflow-x-auto rounded-lg border bg-muted/40 p-3 font-mono text-[12px] leading-relaxed">
            {`cargo run -p sparq-server -- --format turtle data.ttl
# or the published container (binds 0.0.0.0, NO auth by default):
docker run --rm -p 3030:3030 ghcr.io/jeswr/sparq-server`}
          </pre>
        </>
      }
      caveat={
        <>
          <p>
            <strong className="text-foreground">Not a live hosted endpoint.</strong> This page
            does not talk to a server — it replays captured frames, so you cannot run your own
            query against it here. The <strong className="text-foreground">default</strong> server
            (and the container image) bind without authentication; for any exposed deployment set{" "}
            <code className="font-mono">--auth-token</code> (write gate) plus{" "}
            <code className="font-mono">--auth-token-read</code>, deliver the token over TLS, and
            front it with a reverse proxy for real per-user authz.{" "}
            <code className="font-mono">SERVICE</code> federation, time-travel reads, federation
            descriptors, TPF/brTPF, and the SHACL endpoint are all <em>opt-in</em> cargo features
            and off in the default build.
          </p>
        </>
      }
      links={[
        {
          href: "https://github.com/jeswr/sparq/tree/main/crates/sparq-server",
          label: "sparq-server crate",
          external: true,
        },
        {
          href: "https://github.com/jeswr/sparq/blob/main/skills/http-server/SKILL.md",
          label: "http-server SKILL.md",
          external: true,
        },
      ]}
    >
      <HttpServerWalkthrough />
    </SurfaceContent>
  );
}
