import type { Metadata } from "next";
import { Radio } from "lucide-react";

import { SurfaceContent } from "@/components/surface-content";
import { RspPlayground } from "@/components/rsp-playground";

export const metadata: Metadata = {
  title: "Streaming RSP",
  description:
    "Continuous, windowed SPARQL over a live RDF stream with the sparq engine — RSP-QL sliding/tumbling time windows (RANGE/STEP), RSTREAM/ISTREAM/DSTREAM output, AVG-per-window — live in your tab via the W-rsp wasm bundle.",
};

export default function StreamingRspSurfacePage() {
  return (
    <SurfaceContent
      icon={Radio}
      title="Streaming RSP"
      statement="Continuous, windowed SPARQL over a live RDF stream — RSP-QL sliding/tumbling time windows, RSTREAM/ISTREAM/DSTREAM, an AVG per window close."
      tier="live-new-wasm"
      intro={
        <>
          <p>
            <code className="font-mono text-foreground">sparq-rsp</code> runs{" "}
            <strong className="text-foreground">windowed continuous SPARQL</strong>{" "}
            (RSP-QL-style RDF Stream Processing) over a stream of{" "}
            <code className="font-mono">(triple, timestamp)</code> elements. You push
            timestamped readings; it closes windows on a watermark and fires your query
            once per closed window — an <code className="font-mono">AVG(?v)</code>,{" "}
            <code className="font-mono">COUNT</code>, or any SELECT, evaluated over just
            that window&rsquo;s triples.
          </p>
          <p>
            It is a deterministic, synchronous library:{" "}
            <strong className="text-foreground">
              it reads no wall clock and runs no async runtime
            </strong>
            . Time advances only through the timestamps you push, so the whole pipeline
            is a pure, replayable function of the pushed sequence. That is exactly what
            makes it wasm-safe — and here the{" "}
            <strong className="text-foreground">browser tab is the clock</strong>: the
            buttons below advance the logical time, nothing in the bundle does.
          </p>
          <p>
            Streaming ships as a <em>separate</em>, lazy-loaded wasm bundle (the tier-b{" "}
            <code className="font-mono text-foreground">sparq-rsp-wasm</code>{" "}
            &ldquo;W-rsp&rdquo;), not folded into the lean triplestore bundle, so the
            landing page stays light and only this route downloads it. Pick an example,
            push readings, and the closed-window tables come back with no network
            round-trip.
          </p>
        </>
      }
      capabilities={[
        {
          term: "Tumbling & sliding time windows",
          body: "RANGE / STEP in your own logical time unit — step == range tumbles, step < range slides with overlap.",
        },
        {
          term: "Per-window aggregates",
          body: "AVG / COUNT / SUM / MIN / MAX over each window's triples; integer AVG comes back as xsd:decimal per SPARQL typing.",
        },
        {
          term: "RSTREAM / ISTREAM / DSTREAM",
          body: "Full result per window, only rows that newly appeared (ISTREAM), or only rows that disappeared (DSTREAM).",
        },
        {
          term: "Watermark + out-of-order tolerance",
          body: "A window closes when max_ts − maxDelay reaches its end; late arrivals past every covering window are counted, not silently dropped.",
        },
        {
          term: "Empty windows reported",
          body: "When the watermark jumps a gap, the window still fires (with no rows) — so DSTREAM can observe results disappear.",
        },
        {
          term: "Flush at end-of-stream",
          body: "Flush closes every remaining open window up to the last timestamp seen, ignoring maxDelay.",
        },
      ]}
      runsNote={
        <>
          <p>
            <strong className="text-foreground">Live in your browser tab.</strong> The
            stream processor is pure Rust over{" "}
            <code className="font-mono">sparq-engine</code>, compiled to{" "}
            <code className="font-mono">wasm32-unknown-unknown</code> as the separate{" "}
            <code className="font-mono text-foreground">sparq-rsp-wasm</code> bundle. The
            playground registers a continuous SELECT via{" "}
            <code className="font-mono text-foreground">Rsp.select(query, range, step, maxDelay, r2s)</code>
            , then drives the logical clock with{" "}
            <code className="font-mono text-foreground">push(s, p, o, ts)</code> and{" "}
            <code className="font-mono">flush()</code>. Each push returns the windows it
            just closed as standard SPARQL 1.1 JSON — nothing is sent to a server.
          </p>
        </>
      }
      caveat={
        <>
          <p>
            The in-tab bundle wraps the single-window{" "}
            <code className="font-mono">ContinuousQuery</code> SELECT form only. The
            native <code className="font-mono">sparq-rsp</code> crate also offers
            continuous CONSTRUCT / ASK, count (ROWS) windows, the RSP-QL surface syntax
            (<code className="font-mono">REGISTER STREAM … FROM NAMED WINDOW … ON …</code>),
            and multi-window joins (
            <code className="font-mono">WINDOW &lt;w1&gt; JOIN WINDOW &lt;w2&gt;</code>)
            — those stay native for now. Window semantics here are exactly the crate&rsquo;s,
            pinned by its tests; this is a thin, zero-<code className="font-mono">unsafe</code>{" "}
            wrapper that owns no parsing or semantics of its own.
          </p>
        </>
      }
      links={[
        {
          href: "https://github.com/jeswr/sparq/tree/main/crates/sparq-rsp",
          label: "sparq-rsp crate",
          external: true,
        },
        {
          href: "https://github.com/jeswr/sparq/tree/main/crates/sparq-rsp-wasm",
          label: "W-rsp wasm bundle",
          external: true,
        },
      ]}
    >
      <RspPlayground />
    </SurfaceContent>
  );
}
