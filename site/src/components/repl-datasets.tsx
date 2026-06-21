"use client";

// [OPUS-4.8] sq-repl-datasets — dataset source controls (built-in picker / file upload /
// URL load) and the triple viewer dialog for the live SPARQL REPL. These drive the store
// the REPL queries against; all parsing runs in the wasm Store (no mocks, no server).

import * as React from "react";
import {
  Database,
  Upload,
  Link2,
  Loader2,
  Table2,
  FileText,
} from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { cn } from "@/lib/utils";
import {
  formatTerm,
  type SparqlResults,
  type WasmStore,
} from "@/lib/sparq-wasm";
// [OPUS-4.8] sq-8uew — Turtle/N-Triples/N-Quads syntax highlighting for the text view.
// [OPUS-4.8] sq-gb4o (#805) — pretty/indented Turtle (+ raw N-Triples/N-Quads toggle) text view.
import { TurtleResult } from "@/components/rdf-highlight";
import {
  ALL_QUADS_BODY,
  FORMAT_OPTIONS,
  guessFormat,
  formatFromContentType,
} from "@/lib/repl-dataset";
// [OPUS-4.8] sq-spita — client-side gzip/zip decompression so a compressed dataset
// (e.g. a 10M-triple UMBC `.nt.gz` redirect, a TIB watdiv `.zip`) loads by upload OR URL.
import {
  sniffArchive,
  archiveCodecFromName,
  archiveCodecFromContentType,
  decompressArchive,
} from "@/lib/dataset-archive";
import { BUILTIN_DATASETS } from "@/data/sample-graph";

/**
 * Turns the raw bytes of an uploaded file / fetched URL into RDF text + the format to
 * parse it as, transparently decompressing a gzip (`.gz`) or zip (`.zip`) container in
 * the browser tab first. [OPUS-4.8] sq-spita.
 *
 * Detection order: an archive `Content-Type` / extension, else the payload's magic
 * number. Plain (uncompressed) RDF is decoded as UTF-8 directly. The RDF format is
 * derived from the inner (decompressed) member name, with the supplied `explicitFormat`
 * (an explicit picker choice) / `contentType` taking precedence when present.
 */
async function bytesToRdf(
  bytes: Uint8Array,
  sourceName: string,
  opts: { explicitFormat?: string; contentType?: string | null } = {},
): Promise<{ text: string; format: string }> {
  const { explicitFormat, contentType } = opts;
  const archiveCodec =
    archiveCodecFromContentType(contentType ?? null) ??
    archiveCodecFromName(sourceName) ??
    sniffArchive(bytes);

  if (archiveCodec) {
    const { text, innerName } = await decompressArchive(
      bytes,
      sourceName,
      archiveCodec,
    );
    // An explicit picker choice always wins; otherwise the INNER member name decides
    // (the archive's Content-Type only names the container, not the RDF format).
    const format =
      explicitFormat && explicitFormat !== "__auto__"
        ? explicitFormat
        : guessFormat(innerName ?? sourceName);
    return { text, format };
  }

  // Not compressed: decode the bytes as UTF-8 and detect the RDF format normally.
  const text = new TextDecoder().decode(bytes);
  const format =
    explicitFormat && explicitFormat !== "__auto__"
      ? explicitFormat
      : (formatFromContentType(contentType ?? null) ?? guessFormat(sourceName));
  return { text, format };
}

/** What's currently loaded into the REPL store, for the viewer's heading. */
export interface ActiveDataset {
  label: string;
  description: string;
}

// [OPUS-4.8] sq-17nw — enumerate the WHOLE dataset (default graph + every named graph)
// so the viewer shows named-graph content too; default-graph rows leave ?g unbound.
const ALL_QUADS_QUERY = `SELECT ?s ?p ?o ?g WHERE { ${ALL_QUADS_BODY} } ORDER BY ?g ?s ?p ?o`;

// ---------------------------------------------------------------------------
// Dataset source controls
// ---------------------------------------------------------------------------

export interface DatasetControlsProps {
  /** Currently selected built-in id, or null when a custom source is loaded. */
  activeBuiltinId: string | null;
  /** Replace the store from a built-in dataset. */
  onSelectBuiltin: (id: string) => void;
  /**
   * Replace OR add a custom RDF document (text + format). [OPUS-4.8] sq-atb0 — the optional
   * `origin` captures whether this came from a local file upload or a URL (with that URL), so
   * the workspace model can record the import as source metadata (and re-fetch a URL source).
   */
  onLoadText: (
    text: string,
    format: string,
    label: string,
    mode: "replace" | "add",
    origin?: { kind: "local" | "url"; url?: string },
  ) => Promise<void>;
  /** Disable controls while the engine is still cold. */
  disabled?: boolean;
}

/**
 * The dataset source row: a built-in picker plus "Upload" and "From URL" actions.
 * Each path parses through the wasm Store; failures surface a clear inline error.
 */
export function DatasetControls({
  activeBuiltinId,
  onSelectBuiltin,
  onLoadText,
  disabled,
}: DatasetControlsProps) {
  const [mode, setMode] = React.useState<"replace" | "add">("replace");
  const [urlOpen, setUrlOpen] = React.useState(false);
  const [fileError, setFileError] = React.useState<string | null>(null);
  const fileRef = React.useRef<HTMLInputElement | null>(null);

  const onFile = React.useCallback(
    async (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      // Reset the input so re-selecting the same file fires onChange again.
      e.target.value = "";
      if (!file) return;
      setFileError(null);
      try {
        // [OPUS-4.8] sq-spita — read the file as BYTES (not text) so a gzip/zip archive
        // can be detected by magic number and decompressed in-tab before parsing.
        const bytes = new Uint8Array(await file.arrayBuffer());
        const { text, format } = await bytesToRdf(bytes, file.name);
        // [OPUS-4.8] sq-atb0 — a local file import: record the origin so the workspace lists it
        // (it cannot be silently re-read across sessions — the snapshot is its durable copy).
        await onLoadText(text, format, file.name, mode, { kind: "local" });
      } catch (err) {
        setFileError(err instanceof Error ? err.message : String(err));
      }
    },
    [onLoadText, mode],
  );

  return (
    <div className="flex flex-wrap items-center gap-2 rounded-lg border bg-muted/30 p-2">
      <span className="flex items-center gap-1.5 text-xs font-medium text-muted-foreground">
        <Database className="size-3.5" /> Dataset
      </span>

      <label htmlFor="repl-dataset" className="sr-only">
        Built-in dataset
      </label>
      <select
        id="repl-dataset"
        value={activeBuiltinId ?? "__custom__"}
        disabled={disabled}
        onChange={(e) => onSelectBuiltin(e.target.value)}
        className="h-7 rounded-md border bg-background px-2 text-xs outline-none focus-visible:ring-3 focus-visible:ring-ring/40 disabled:opacity-50"
      >
        {BUILTIN_DATASETS.map((d) => (
          <option key={d.id} value={d.id}>
            {d.label}
          </option>
        ))}
        {activeBuiltinId === null && (
          <option value="__custom__">Custom (loaded)</option>
        )}
      </select>

      <span className="ml-auto flex items-center gap-2">
        <label
          htmlFor="repl-mode"
          className="text-[11px] text-muted-foreground"
        >
          On load
        </label>
        <select
          id="repl-mode"
          value={mode}
          disabled={disabled}
          onChange={(e) => setMode(e.target.value as "replace" | "add")}
          className="h-7 rounded-md border bg-background px-2 text-xs outline-none focus-visible:ring-3 focus-visible:ring-ring/40 disabled:opacity-50"
        >
          <option value="replace">Replace</option>
          <option value="add">Add to current</option>
        </select>

        <input
          ref={fileRef}
          type="file"
          // [OPUS-4.8] sq-spita — also accept gzip/zip archives (incl. compound suffixes
          // such as .ttl.gz / .nt.gz); they decompress in-tab before parsing.
          accept=".ttl,.turtle,.nt,.ntriples,.nq,.nquads,.trig,.jsonld,.gz,.zip,.ttl.gz,.nt.gz,.nq.gz,.trig.gz,text/turtle,application/n-triples,application/n-quads,application/trig,application/ld+json,application/gzip,application/zip"
          className="hidden"
          onChange={onFile}
        />
        <Button
          variant="outline"
          size="sm"
          disabled={disabled}
          onClick={() => fileRef.current?.click()}
        >
          <Upload className="size-3.5" /> Upload
        </Button>
        <Button
          variant="outline"
          size="sm"
          disabled={disabled}
          onClick={() => setUrlOpen(true)}
        >
          <Link2 className="size-3.5" /> From URL
        </Button>
      </span>

      {/* [OPUS-4.8] sq-spita — a failed upload (e.g. an unsupported zip member, or a
          parse error in the decompressed RDF) surfaces inline rather than silently. */}
      {fileError && (
        <p className="w-full rounded-lg border border-destructive/30 bg-destructive/5 p-2 text-xs text-destructive">
          {fileError}
        </p>
      )}

      <UrlLoadDialog
        open={urlOpen}
        onOpenChange={setUrlOpen}
        defaultMode={mode}
        onLoadText={onLoadText}
      />
    </div>
  );
}

// ---------------------------------------------------------------------------
// URL load dialog
// ---------------------------------------------------------------------------

function UrlLoadDialog({
  open,
  onOpenChange,
  defaultMode,
  onLoadText,
}: {
  open: boolean;
  onOpenChange: (o: boolean) => void;
  defaultMode: "replace" | "add";
  onLoadText: DatasetControlsProps["onLoadText"];
}) {
  const [url, setUrl] = React.useState("");
  // [OPUS-4.8] sq-spita — default to Auto-detect so a compressed URL (e.g. a UMBC
  // `.nt.gz` redirect or a TIB watdiv `.zip`) detects its container + inner RDF format
  // out of the box (served media type → URL extension → magic number).
  const [format, setFormat] = React.useState("__auto__");
  const [busy, setBusy] = React.useState(false);
  const [error, setError] = React.useState<string | null>(null);

  const submit = React.useCallback(async () => {
    const target = url.trim();
    if (!target) return;
    setBusy(true);
    setError(null);
    try {
      let res: Response;
      try {
        res = await fetch(target, { headers: { Accept: "text/turtle, application/n-triples, application/n-quads, application/trig, application/ld+json, */*" } });
      } catch {
        // A network/CORS rejection throws a TypeError with no useful detail in the
        // browser — be explicit about the most likely cause rather than failing silently.
        throw new Error(
          "Could not fetch the URL. The server likely does not send CORS headers " +
            "(Access-Control-Allow-Origin), so the browser blocks the cross-origin read. " +
            "Try a CORS-enabled host, or download the file and use Upload.",
        );
      }
      if (!res.ok) {
        throw new Error(`Fetch failed: HTTP ${res.status} ${res.statusText}`);
      }
      // [OPUS-4.8] sq-spita — fetch as BYTES so a gzipped (`.nt.gz`) or zipped (`.zip`)
      // dataset is decompressed in-tab; `bytesToRdf` also handles the auto-detect (served
      // media type → URL extension → magic number). An explicit picker choice still wins.
      const bytes = new Uint8Array(await res.arrayBuffer());
      const { text, format: fmt } = await bytesToRdf(bytes, target, {
        explicitFormat: format,
        contentType: res.headers.get("content-type"),
      });
      // [OPUS-4.8] sq-atb0 — a URL import: record the origin URL so a workspace can re-fetch it.
      await onLoadText(text, fmt, shortName(target), defaultMode, {
        kind: "url",
        url: target,
      });
      onOpenChange(false);
      setUrl("");
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }, [url, format, defaultMode, onLoadText, onOpenChange]);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(34rem,calc(100vw-2rem))]">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Link2 className="size-4 text-primary" /> Load RDF from a URL
          </DialogTitle>
          <DialogDescription>
            Fetched and parsed entirely in your browser tab. Gzip (.gz) and zip
            (.zip) archives decompress in-tab. The host must allow cross-origin
            reads (CORS).
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-3">
          <div className="space-y-1.5">
            <label htmlFor="repl-url" className="text-xs font-medium">
              URL
            </label>
            <input
              id="repl-url"
              type="url"
              value={url}
              placeholder="https://example.org/data.ttl"
              spellCheck={false}
              onChange={(e) => setUrl(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter" && !busy) void submit();
              }}
              className="w-full rounded-lg border bg-muted/40 px-3 py-2 font-mono text-[13px] outline-none focus-visible:ring-3 focus-visible:ring-ring/40"
            />
          </div>
          <div className="space-y-1.5">
            <label htmlFor="repl-url-format" className="text-xs font-medium">
              Format
            </label>
            <select
              id="repl-url-format"
              value={format}
              onChange={(e) => setFormat(e.target.value)}
              className="h-8 w-full rounded-lg border bg-background px-2 text-sm outline-none focus-visible:ring-3 focus-visible:ring-ring/40"
            >
              <option value="__auto__">
                Auto-detect (content type / extension; .gz/.zip ok)
              </option>
              {FORMAT_OPTIONS.map((f) => (
                <option key={f.value} value={f.value}>
                  {f.label}
                </option>
              ))}
            </select>
          </div>

          {error && (
            <p className="rounded-lg border border-destructive/30 bg-destructive/5 p-2.5 text-xs text-destructive">
              {error}
            </p>
          )}

          <div className="flex justify-end gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => onOpenChange(false)}
            >
              Cancel
            </Button>
            <Button size="sm" disabled={busy || !url.trim()} onClick={submit}>
              {busy ? (
                <Loader2 className="size-3.5 animate-spin" />
              ) : (
                <Link2 className="size-3.5" />
              )}
              Fetch & load
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}

function shortName(url: string): string {
  try {
    const u = new URL(url);
    const last = u.pathname.split("/").filter(Boolean).pop();
    return last || u.hostname;
  } catch {
    return url.slice(0, 40);
  }
}

// ---------------------------------------------------------------------------
// Dataset viewer dialog
// ---------------------------------------------------------------------------

export interface DatasetViewerProps {
  open: boolean;
  onOpenChange: (o: boolean) => void;
  store: WasmStore | null;
  size: number | null;
  active: ActiveDataset;
}

/**
 * The triple viewer: opened by the triple-count badge. Reads the actual triples out of
 * the live store (a `SELECT ?s ?p ?o` query — real data, not the source text) and shows
 * them as a table or as an N-Triples listing.
 */
export function DatasetViewer({
  open,
  onOpenChange,
  store,
  size,
  active,
}: DatasetViewerProps) {
  const [view, setView] = React.useState<"table" | "ntriples">("table");
  const [rows, setRows] = React.useState<SparqlResults["results"]["bindings"]>(
    [],
  );
  const [error, setError] = React.useState<string | null>(null);

  // Re-read the quads whenever the dialog opens against the current store.
  React.useEffect(() => {
    if (!open || !store) return;
    try {
      const parsed = JSON.parse(store.query(ALL_QUADS_QUERY)) as SparqlResults;
      setRows(parsed.results.bindings);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      setRows([]);
    }
  }, [open, store]);

  // Whether ANY row is in a named graph — drives the optional "graph" column / the
  // N-Quads (vs N-Triples) text view, so a plain triple dataset stays unchanged.
  const hasGraphs = React.useMemo(() => rows.some((r) => r.g), [rows]);

  // Default-graph rows render as triples; named-graph rows render as quads.
  const serialised = React.useMemo(
    () =>
      rows
        .map((r) => {
          const spo = `${formatTerm(r.s)} ${formatTerm(r.p)} ${formatTerm(r.o)}`;
          return r.g ? `${spo} ${formatTerm(r.g)} .` : `${spo} .`;
        })
        .join("\n"),
    [rows],
  );

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Database className="size-4 text-primary" /> {active.label}
          </DialogTitle>
          <DialogDescription>
            {active.description}
            {size !== null && ` · ${size} triples`}
          </DialogDescription>
        </DialogHeader>

        <div className="flex items-center gap-1.5">
          <Button
            variant={view === "table" ? "default" : "outline"}
            size="sm"
            onClick={() => setView("table")}
          >
            <Table2 className="size-3.5" /> Table
          </Button>
          <Button
            variant={view === "ntriples" ? "default" : "outline"}
            size="sm"
            onClick={() => setView("ntriples")}
          >
            <FileText className="size-3.5" /> {hasGraphs ? "TriG" : "Turtle"}
          </Button>
        </div>

        <div className="min-h-0 flex-1 overflow-auto rounded-lg border">
          {error ? (
            <pre className="p-3 text-xs text-destructive">{error}</pre>
          ) : rows.length === 0 ? (
            <p className="p-3 text-sm text-muted-foreground">
              The store is empty.
            </p>
          ) : view === "table" ? (
            <table className="w-full text-left text-sm">
              <thead className="sticky top-0 bg-muted">
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
                {rows.map((r, i) => (
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
                        {/* Default-graph rows leave ?g unbound — show a dash. */}
                        {r.g ? formatTerm(r.g) : "—"}
                      </td>
                    )}
                  </tr>
                ))}
              </tbody>
            </table>
          ) : (
            <TurtleResult
              text={serialised}
              rawLabel={hasGraphs ? "N-Quads" : "N-Triples"}
              className="overflow-x-auto p-3 text-[12px] leading-relaxed"
            />
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}
