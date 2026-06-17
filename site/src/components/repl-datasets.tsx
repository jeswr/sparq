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
import {
  ALL_QUADS_BODY,
  FORMAT_OPTIONS,
  guessFormat,
  formatFromContentType,
} from "@/lib/repl-dataset";
import { BUILTIN_DATASETS } from "@/data/sample-graph";

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
  /** Replace OR add a custom RDF document (text + format). */
  onLoadText: (
    text: string,
    format: string,
    label: string,
    mode: "replace" | "add",
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
  const fileRef = React.useRef<HTMLInputElement | null>(null);

  const onFile = React.useCallback(
    async (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      // Reset the input so re-selecting the same file fires onChange again.
      e.target.value = "";
      if (!file) return;
      const text = await file.text();
      await onLoadText(text, guessFormat(file.name), file.name, mode);
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
          accept=".ttl,.turtle,.nt,.ntriples,.nq,.nquads,.trig,.jsonld,text/turtle,application/n-triples,application/n-quads,application/trig,application/ld+json"
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
  const [format, setFormat] = React.useState("turtle");
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
      const text = await res.text();
      // Auto-detect prefers the served media type (reliable for extension-less URLs),
      // then the URL extension. An explicit picker choice always wins.
      const fmt =
        format === "__auto__"
          ? (formatFromContentType(res.headers.get("content-type")) ?? guessFormat(target))
          : format;
      await onLoadText(text, fmt, shortName(target), defaultMode);
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
            Fetched and parsed entirely in your browser tab. The host must allow
            cross-origin reads (CORS).
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
              <option value="__auto__">Auto-detect (content type / extension)</option>
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
            <FileText className="size-3.5" />{" "}
            {hasGraphs ? "N-Quads" : "N-Triples"}
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
            <pre className="overflow-x-auto p-3 font-mono text-[12px] leading-relaxed">
              {serialised}
            </pre>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}
