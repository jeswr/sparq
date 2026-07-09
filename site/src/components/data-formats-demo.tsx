"use client";

// [OPUS-4.8] sq-j50v — the LIVE /surface/data-formats demo. Two real, in-tab panels, both
// driven by the shipped lean wasm Store (no mocks, no server):
//
//   1. FORMAT PICKER — pick Turtle / N-Triples / N-Quads / TriG, paste (or edit the seeded
//      sample), Parse → the engine's triple count + a sample of the parsed triples. Quad
//      formats route through `loadDataset` so the named graph survives and shows as a ?g
//      column; the TriG / N-Quads samples include a named-graph example.
//   2. GZIP INGEST — gzip the current document in-tab via the native `CompressionStream`,
//      show the size + ratio, then gunzip it back with `DecompressionStream` and parse the
//      decoded text → triple count. gzip is the one codec the browser decodes natively;
//      zstd / bzip2 ingest is native-only (see the page's honest caveat).

import * as React from "react";
import { Loader2, Play, FileArchive, Database, CheckCircle2 } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { cn } from "@/lib/utils";
import {
  loadSparq,
  loadIntoStore,
  prewarmSparqWhenIdle,
  datasetSize,
  formatTerm,
  type SparqlResults,
  type WasmStore,
} from "@/lib/sparq-wasm";
import {
  FORMAT_SAMPLES,
  sampleFor,
  gzipString,
  gunzipToString,
  formatBytes,
  type DataFormat,
} from "@/lib/data-formats";
// [OPUS-4.8] sq-8uew — Turtle/TriG/N-Triples/N-Quads syntax-highlighting input editor.
import { RdfEditor } from "@/components/rdf-editor";
// [OPUS-4.8] sq-ixc3.1 — JSON-LD syntax-highlighting input editor (the remaining picker format).
import { JsonLdEditor } from "@/components/jsonld-editor";

// The whole-dataset enumeration: default graph UNION every named graph, so a sample of an
// N-Quads / TriG document shows its named-graph rows too (?g bound). Same shape the REPL uses.
const SAMPLE_QUERY =
  "SELECT ?s ?p ?o ?g WHERE { { ?s ?p ?o } UNION { GRAPH ?g { ?s ?p ?o } } } LIMIT 8";

type EngineState = "cold" | "warming" | "ready" | "error";

type ParseState =
  | { kind: "idle" }
  | { kind: "running" }
  | { kind: "ok"; count: number; rows: SparqlResults["results"]["bindings"]; ms: number }
  | { kind: "error"; message: string };

type GzipState =
  | { kind: "idle" }
  | { kind: "running" }
  | {
      kind: "ok";
      rawBytes: number;
      gzBytes: number;
      count: number;
      ms: number;
    }
  | { kind: "error"; message: string };

export function DataFormatsDemo() {
  const [format, setFormat] = React.useState<DataFormat>("turtle");
  const [text, setText] = React.useState(sampleFor("turtle")?.sample ?? "");
  const [engine, setEngine] = React.useState<EngineState>("cold");
  const [parse, setParse] = React.useState<ParseState>({ kind: "idle" });
  const [gzip, setGzip] = React.useState<GzipState>({ kind: "idle" });

  // [OPUS-4.8] sq-4296 (#935 / #981) — pre-warm the wasm engine on the next browser-IDLE slot
  // (not synchronously during mount) so the ~300 kB+ engine never blocks the initial page
  // paint. Shares the single memoised loadSparq promise with the rest of the site, so the
  // first Parse joins the in-flight load rather than re-paying a cold start.
  React.useEffect(() => {
    let cancelled = false;
    setEngine("warming");
    const handle = prewarmSparqWhenIdle({
      onReady: () => {
        if (!cancelled) setEngine("ready");
      },
      onError: () => {
        if (!cancelled) setEngine("error");
      },
    });
    return () => {
      cancelled = true;
      handle.cancel();
    };
  }, []);

  // Switch the picked format AND reseed the textarea with that format's sample, so the
  // chosen syntax always matches the document. (Editing the textarea then keeps the edit.)
  const pickFormat = React.useCallback((f: DataFormat) => {
    setFormat(f);
    setText(sampleFor(f)?.sample ?? "");
    setParse({ kind: "idle" });
    setGzip({ kind: "idle" });
  }, []);

  // Parse the current text in the chosen format and read back the engine's triple count +
  // a small sample. loadIntoStore routes quad formats through loadDataset (named graphs).
  const runParse = React.useCallback(async () => {
    setParse({ kind: "running" });
    try {
      const Store = await loadSparq();
      setEngine("ready");
      const t0 = performance.now();
      const store: WasmStore = loadIntoStore(Store, text, format);
      const count = datasetSize(store);
      const rows = (
        JSON.parse(store.query(SAMPLE_QUERY)) as SparqlResults
      ).results.bindings;
      const ms = performance.now() - t0;
      setParse({ kind: "ok", count, rows, ms });
    } catch (e) {
      setParse({
        kind: "error",
        message: e instanceof Error ? e.message : String(e),
      });
    }
  }, [text, format]);

  // The gzip ingest path: gzip the current document in-tab, then gunzip it back and parse
  // the decoded text — the same JS-side decode a real `.ttl.gz` upload would take.
  const runGzip = React.useCallback(async () => {
    setGzip({ kind: "running" });
    try {
      const rawBytes = new TextEncoder().encode(text).byteLength;
      const gz = await gzipString(text);
      const decoded = await gunzipToString(gz);
      const Store = await loadSparq();
      setEngine("ready");
      const t0 = performance.now();
      const store = loadIntoStore(Store, decoded, format);
      const count = datasetSize(store);
      const ms = performance.now() - t0;
      setGzip({ kind: "ok", rawBytes, gzBytes: gz.byteLength, count, ms });
    } catch (e) {
      setGzip({
        kind: "error",
        message: e instanceof Error ? e.message : String(e),
      });
    }
  }, [text, format]);

  const busy = parse.kind === "running" || gzip.kind === "running";
  const disabled = engine === "cold" || engine === "warming";

  return (
    <section className="space-y-4" aria-labelledby="data-formats-demo-heading">
      <h2 id="data-formats-demo-heading" className="text-xl font-semibold">
        Try it: parse RDF in your tab
      </h2>

      <Card>
        <CardHeader className="flex-row items-center justify-between gap-2 space-y-0">
          <CardTitle className="flex items-center gap-2 text-base">
            <Database className="size-4 text-primary" aria-hidden="true" />
            Format picker
          </CardTitle>
          <EngineIndicator engine={engine} />
        </CardHeader>
        <CardContent className="space-y-3">
          <div
            role="tablist"
            aria-label="RDF text format"
            className="flex flex-wrap gap-1.5"
          >
            {FORMAT_SAMPLES.map((f) => (
              <Button
                key={f.format}
                role="tab"
                aria-selected={format === f.format}
                variant={format === f.format ? "default" : "outline"}
                size="sm"
                onClick={() => pickFormat(f.format)}
              >
                {f.label}
                {/* [FABLE-5] opacity-90 (was 70): the extension text on the selected teal button was 4.34:1, below WCAG AA 4.5:1 (color-contrast, sq-0rbfn) */}
                <span className="ml-1 font-mono text-[11px] opacity-90">
                  {f.ext}
                </span>
              </Button>
            ))}
          </div>

          <p className="text-xs text-muted-foreground">
            {sampleFor(format)?.blurb}
          </p>

          <label htmlFor="data-formats-input" className="sr-only">
            RDF document to parse
          </label>
          {/* [OPUS-4.8] sq-8uew / sq-ixc3.1 — syntax-highlighting input. JSON-LD is JSON (a
              different tokenizer), so the picker routes it to the JSON-LD editor; the four text
              formats share the Turtle-family editor. Both reuse the shared `.sq-tok-*` palette. */}
          {format === "jsonld" ? (
            <JsonLdEditor
              id="data-formats-input"
              ariaLabel="JSON-LD document to parse"
              value={text}
              onChange={setText}
              rows={10}
            />
          ) : (
            <RdfEditor
              id="data-formats-input"
              ariaLabel="RDF document to parse"
              value={text}
              onChange={setText}
              rows={10}
            />
          )}

          <div className="flex items-center gap-3">
            <Button onClick={runParse} disabled={busy || disabled}>
              {parse.kind === "running" ? (
                <Loader2 className="size-4 animate-spin" aria-hidden="true" />
              ) : (
                <Play className="size-4" aria-hidden="true" />
              )}
              Parse
            </Button>
            <p aria-live="polite" className="text-xs text-muted-foreground">
              {parse.kind === "ok" &&
                `${parse.count} triples · parsed in ${parse.ms.toFixed(1)} ms`}
              {parse.kind === "running" && "Parsing on the wasm engine…"}
              {parse.kind === "idle" &&
                engine === "warming" &&
                "Pre-warming the wasm engine…"}
            </p>
          </div>

          <ParseResult state={parse} />
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="flex-row items-center gap-2 space-y-0">
          <CardTitle className="flex items-center gap-2 text-base">
            <FileArchive className="size-4 text-primary" aria-hidden="true" />
            Compressed (gzip) ingest
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <p className="text-sm text-muted-foreground">
            gzip the document above entirely in your tab (the browser&apos;s native{" "}
            <code className="font-mono text-foreground">CompressionStream</code>),
            then decode it back with{" "}
            <code className="font-mono text-foreground">DecompressionStream</code>{" "}
            and parse the decoded text — the same JS-side decode a{" "}
            <code className="font-mono">.ttl.gz</code> upload takes before reaching{" "}
            <code className="font-mono">Store.load</code>.
          </p>

          <div className="flex items-center gap-3">
            <Button
              variant="outline"
              onClick={runGzip}
              disabled={busy || disabled}
            >
              {gzip.kind === "running" ? (
                <Loader2 className="size-4 animate-spin" aria-hidden="true" />
              ) : (
                <FileArchive className="size-4" aria-hidden="true" />
              )}
              gzip → decode → parse
            </Button>
            <p aria-live="polite" className="text-xs text-muted-foreground">
              {gzip.kind === "running" && "Compressing & decoding in your tab…"}
            </p>
          </div>

          <GzipResult state={gzip} />
        </CardContent>
      </Card>
    </section>
  );
}

// Subtle engine-readiness pill, mirroring the REPL's indicator.
function EngineIndicator({ engine }: { engine: EngineState }) {
  if (engine === "ready") {
    return (
      <Badge variant="success" aria-live="polite">
        <CheckCircle2 className="size-3" aria-hidden="true" /> Engine ready
      </Badge>
    );
  }
  if (engine === "error") {
    return (
      <Badge variant="warning" aria-live="polite">
        Engine failed — retries on parse
      </Badge>
    );
  }
  return (
    <Badge variant="muted" aria-live="polite">
      <Loader2 className="size-3 animate-spin" aria-hidden="true" /> Engine
      loading…
    </Badge>
  );
}

function ParseResult({ state }: { state: ParseState }) {
  if (state.kind === "error") {
    return (
      <pre className="overflow-x-auto rounded-lg border border-destructive/30 bg-destructive/5 p-3 text-xs text-destructive">
        {state.message}
      </pre>
    );
  }
  if (state.kind !== "ok") return null;

  const hasGraphs = state.rows.some((r) => r.g);
  return (
    <div className="space-y-2">
      <Badge variant="muted" className="tabular">
        <Database className="size-3" aria-hidden="true" /> {state.count} triples
      </Badge>
      {state.rows.length === 0 ? (
        <p className="rounded-lg border bg-muted/30 p-3 text-sm text-muted-foreground">
          Parsed, but the document has no triples.
        </p>
      ) : (
        <div className="overflow-x-auto rounded-lg border">
          <table className="w-full text-left text-sm">
            <thead className="bg-muted/50">
              <tr>
                {["subject", "predicate", "object"].map((h) => (
                  <th key={h} className="px-3 py-2 font-medium">
                    {h}
                  </th>
                ))}
                {hasGraphs && (
                  <th className="px-3 py-2 font-medium">graph</th>
                )}
              </tr>
            </thead>
            <tbody>
              {state.rows.map((r, i) => (
                <tr key={i} className="border-t">
                  {(["s", "p", "o"] as const).map((k) => (
                    <td
                      key={k}
                      className={cn(
                        "px-3 py-1.5 font-mono text-[12px]",
                        k === "o" && "break-all",
                      )}
                    >
                      {formatTerm(r[k])}
                    </td>
                  ))}
                  {hasGraphs && (
                    <td className="px-3 py-1.5 font-mono text-[12px] break-all text-muted-foreground">
                      {r.g ? formatTerm(r.g) : "—"}
                    </td>
                  )}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
      <p className="text-xs text-muted-foreground">
        A sample of up to 8 triples, read straight out of the live store.
      </p>
    </div>
  );
}

function GzipResult({ state }: { state: GzipState }) {
  if (state.kind === "error") {
    return (
      <pre className="overflow-x-auto rounded-lg border border-destructive/30 bg-destructive/5 p-3 text-xs text-destructive">
        {state.message}
      </pre>
    );
  }
  if (state.kind !== "ok") return null;

  const ratio = state.gzBytes > 0 ? state.rawBytes / state.gzBytes : 0;
  return (
    <div className="grid grid-cols-2 gap-2 sm:grid-cols-4">
      <Stat label="Original" value={formatBytes(state.rawBytes)} />
      <Stat label="gzipped" value={formatBytes(state.gzBytes)} />
      <Stat label="Ratio" value={`${ratio.toFixed(1)}×`} />
      <Stat label="Decoded → parsed" value={`${state.count} triples`} />
    </div>
  );
}

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-lg bg-muted/40 p-3 ring-1 ring-foreground/10">
      <div className="text-[11px] uppercase tracking-wide text-muted-foreground">
        {label}
      </div>
      <div className="tabular text-sm font-semibold">{value}</div>
    </div>
  );
}
