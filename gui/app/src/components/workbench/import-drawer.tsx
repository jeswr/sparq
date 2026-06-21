"use client";

// [OPUS-4.8] sq-ixc3.13 (epic sq-ixc3) — the IMPORT DRAWER: real disk / URL / paste ingest into
// the active workspace's live store via the NATIVE loader (research/gui-design.md §A.4).
//
// This is the operational counterpart to the site's /try dataset picker: NOT a demo dataset
// chooser, but the way a user gets THEIR OWN data into the workspace. Three tabs:
//   * FILE  — a native file-open dialog → the native loader (`load_path`): every format incl.
//             COMPRESSED (.gz/.bz2/.zst) + NATIVE-ONLY HDT, off the main thread, no wasm ceiling.
//             Desktop only (a browser cannot read an arbitrary disk path).
//   * URL   — fetch a remote document, then load it (native `load_text` on desktop, in-tab WASM
//             on the web). Recorded as a re-fetchable WorkspaceSourceMeta.
//   * PASTE — load a typed/pasted document (native `load_text` on desktop, in-tab WASM on web).
// Plus a format auto-detect (overridable), a named-graph-preserving toggle (the `preserve_graphs`
// loader arg), and a replace/add-to-current mode. On success it updates the datasets tree (via
// the engine context) AND records the import + a workspace snapshot (sq-atb0) via the workspace
// context. It is mounted ONCE in the workbench; the rail's "+ Import", the top bar, and Cmd-K all
// open it through `useImportDrawer`.
//
// HONESTY. The drawer states plainly when the native loader is unavailable (the web target): on
// the web, File import is disabled and paste/URL run in the in-tab WASM engine (no compressed-
// file / HDT path). No performance number is shown.

import * as React from "react";
import { Dialog as DialogPrimitive } from "radix-ui";
import {
  Upload,
  FileUp,
  Link2,
  ClipboardPaste,
  X,
  Loader2,
  FolderOpen,
  CheckCircle2,
  AlertTriangle,
} from "lucide-react";

import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { useEngine, type ImportKind, type ImportMode } from "@/lib/engine-context";
import { useWorkspace } from "@/lib/workspace-context";
import {
  FORMAT_OPTIONS,
  fileLabel,
  formatFromContentType,
  guessFormat,
  isCompressed,
  isHdt,
  urlLabel,
} from "@/lib/rdf-format";
import { hasNativeLoader, pickRdfFile } from "@/lib/tauri-ipc";
import type { WorkspaceSourceMeta } from "@sparq/client";

// ── Open-state context (so the rail / top bar / Cmd-K can open the drawer) ─────────────────────

const ImportDrawerContext = React.createContext<{
  open: boolean;
  setOpen: (open: boolean) => void;
} | null>(null);

/** Open / close the Import drawer from anywhere inside <ImportDrawerProvider>. */
export function useImportDrawer() {
  const ctx = React.useContext(ImportDrawerContext);
  if (!ctx) throw new Error("useImportDrawer must be used within <ImportDrawerProvider>");
  return ctx;
}

export function ImportDrawerProvider({ children }: { children: React.ReactNode }) {
  const [open, setOpen] = React.useState(false);
  const value = React.useMemo(() => ({ open, setOpen }), [open]);
  return (
    <ImportDrawerContext.Provider value={value}>
      {children}
      <ImportDrawer open={open} onOpenChange={setOpen} />
    </ImportDrawerContext.Provider>
  );
}

// ── The drawer body ────────────────────────────────────────────────────────────────────────────

type Tab = "file" | "url" | "paste";

interface FeedbackOk {
  kind: "ok";
  message: string;
}
interface FeedbackErr {
  kind: "error";
  message: string;
}
type Feedback = FeedbackOk | FeedbackErr | null;

function ImportDrawer({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const { importRdf, snapshotStore } = useEngine();
  const { recordImport } = useWorkspace();
  const native = hasNativeLoader();

  const [tab, setTab] = React.useState<Tab>(native ? "file" : "paste");
  const [busy, setBusy] = React.useState(false);
  const [feedback, setFeedback] = React.useState<Feedback>(null);

  // Shared controls.
  const [mode, setMode] = React.useState<ImportMode>("add");
  const [preserveGraphs, setPreserveGraphs] = React.useState(true);

  // File tab.
  const [filePath, setFilePath] = React.useState<string>("");
  // URL tab.
  const [url, setUrl] = React.useState<string>("");
  // Paste tab.
  const [pasteText, setPasteText] = React.useState<string>("");
  const [pasteFormat, setPasteFormat] = React.useState<string>("turtle");

  // Reset transient state whenever the drawer is (re)opened.
  React.useEffect(() => {
    if (open) {
      setFeedback(null);
      setBusy(false);
      setTab(native ? "file" : "paste");
    }
  }, [open, native]);

  const finishImport = React.useCallback(
    async (
      sourceKind: ImportKind,
      run: () => Promise<{ source: WorkspaceSourceMeta; storeSize: number; added: number }>,
    ) => {
      setBusy(true);
      setFeedback(null);
      try {
        const { source, storeSize, added } = await run();
        // Record the source + persist a fresh snapshot of the now-updated live store.
        await recordImport(source, snapshotStore());
        setFeedback({
          kind: "ok",
          message: `Imported ${added.toLocaleString()} quad${added === 1 ? "" : "s"} (${sourceKind}). Store now holds ${storeSize.toLocaleString()}.`,
        });
      } catch (err) {
        setFeedback({
          kind: "error",
          message: err instanceof Error ? err.message : String(err),
        });
      } finally {
        setBusy(false);
      }
    },
    [recordImport, snapshotStore],
  );

  // FILE — pick via the native dialog, then load via the native loader.
  const onPickFile = React.useCallback(async () => {
    const picked = await pickRdfFile();
    if (picked) setFilePath(picked);
  }, []);

  const onImportFile = React.useCallback(() => {
    if (!filePath) return;
    void finishImport("file", async () => {
      const label = fileLabel(filePath);
      const format = guessFormat(filePath);
      const result = await importRdf({
        kind: "file",
        mode,
        preserveGraphs,
        label,
        format,
        path: filePath,
      });
      const source: WorkspaceSourceMeta = {
        kind: "local",
        label,
        format: result.format,
        bytes: result.bytes,
        importedAt: Date.now(),
      };
      return { source, storeSize: result.storeSize, added: result.added };
    });
  }, [filePath, finishImport, importRdf, mode, preserveGraphs]);

  // URL — fetch the document, prefer the served Content-Type for the format, then load.
  const onImportUrl = React.useCallback(() => {
    const target = url.trim();
    if (!target) return;
    void finishImport("url", async () => {
      const res = await fetch(target);
      if (!res.ok) throw new Error(`Fetch failed: HTTP ${res.status} ${res.statusText}`);
      const body = await res.text();
      const format =
        formatFromContentType(res.headers.get("content-type")) ?? guessFormat(target);
      const result = await importRdf({
        kind: "url",
        mode,
        preserveGraphs,
        label: urlLabel(target),
        format,
        text: body,
        url: target,
      });
      const source: WorkspaceSourceMeta = {
        kind: "url",
        label: urlLabel(target),
        url: target,
        format: result.format,
        bytes: result.bytes,
        importedAt: Date.now(),
      };
      return { source, storeSize: result.storeSize, added: result.added };
    });
  }, [url, finishImport, importRdf, mode, preserveGraphs]);

  // PASTE — load the typed document with the chosen format.
  const onImportPaste = React.useCallback(() => {
    const text = pasteText.trim();
    if (!text) return;
    void finishImport("paste", async () => {
      const result = await importRdf({
        kind: "paste",
        mode,
        preserveGraphs,
        label: "pasted document",
        format: pasteFormat,
        text,
      });
      const source: WorkspaceSourceMeta = {
        kind: "local",
        label: "pasted document",
        format: result.format,
        bytes: result.bytes,
        importedAt: Date.now(),
      };
      return { source, storeSize: result.storeSize, added: result.added };
    });
  }, [pasteText, pasteFormat, finishImport, importRdf, mode, preserveGraphs]);

  return (
    <DialogPrimitive.Root open={open} onOpenChange={onOpenChange}>
      <DialogPrimitive.Portal>
        <DialogPrimitive.Overlay
          className={cn(
            "fixed inset-0 z-50 bg-black/40 backdrop-blur-sm",
            "data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0",
          )}
        />
        <DialogPrimitive.Content
          data-import-drawer
          className={cn(
            "bg-card text-card-foreground fixed right-0 top-0 z-50 flex h-full w-[min(28rem,calc(100vw-2rem))] flex-col",
            "border-l shadow-2xl",
            "data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:slide-out-to-right data-[state=open]:slide-in-from-right",
          )}
        >
          <div className="flex items-center gap-2 border-b px-4 py-3">
            <Upload className="size-4 text-primary" aria-hidden />
            <DialogPrimitive.Title className="flex-1 text-sm font-semibold">
              Import RDF into the workspace
            </DialogPrimitive.Title>
            <DialogPrimitive.Close asChild>
              <Button variant="ghost" size="icon-sm" aria-label="Close import drawer">
                <X className="size-4" />
              </Button>
            </DialogPrimitive.Close>
          </div>
          <DialogPrimitive.Description className="sr-only">
            Load RDF from a disk file (compressed and HDT supported on the desktop app), from a
            URL, or by pasting a document, into the active workspace&rsquo;s live store.
          </DialogPrimitive.Description>

          {/* Tab strip. */}
          <div className="flex border-b text-xs" role="tablist">
            <DrawerTab id="file" current={tab} onSelect={setTab} icon={FileUp} label="File" />
            <DrawerTab id="url" current={tab} onSelect={setTab} icon={Link2} label="URL" />
            <DrawerTab id="paste" current={tab} onSelect={setTab} icon={ClipboardPaste} label="Paste" />
          </div>

          <div className="flex-1 overflow-y-auto p-4">
            {tab === "file" && (
              <FilePane
                native={native}
                filePath={filePath}
                onPickFile={onPickFile}
                onClear={() => setFilePath("")}
              />
            )}
            {tab === "url" && <UrlPane url={url} onUrl={setUrl} />}
            {tab === "paste" && (
              <PastePane
                text={pasteText}
                onText={setPasteText}
                format={pasteFormat}
                onFormat={setPasteFormat}
              />
            )}
          </div>

          {/* Shared options + actions. */}
          <div className="space-y-3 border-t p-4">
            <OptionsRow
              mode={mode}
              onMode={setMode}
              preserveGraphs={preserveGraphs}
              onPreserveGraphs={setPreserveGraphs}
            />

            {feedback && (
              <p
                role="status"
                data-import-feedback={feedback.kind}
                className={cn(
                  "flex items-start gap-2 rounded-md border px-3 py-2 text-xs",
                  feedback.kind === "ok"
                    ? "border-[var(--success)]/40 text-[var(--success)]"
                    : "border-destructive/40 text-destructive",
                )}
              >
                {feedback.kind === "ok" ? (
                  <CheckCircle2 className="mt-0.5 size-3.5 shrink-0" />
                ) : (
                  <AlertTriangle className="mt-0.5 size-3.5 shrink-0" />
                )}
                <span>{feedback.message}</span>
              </p>
            )}

            <Button
              className="w-full"
              disabled={
                busy ||
                (tab === "file" && (!native || !filePath)) ||
                (tab === "url" && !url.trim()) ||
                (tab === "paste" && !pasteText.trim())
              }
              onClick={
                tab === "file" ? onImportFile : tab === "url" ? onImportUrl : onImportPaste
              }
            >
              {busy ? (
                <>
                  <Loader2 className="size-4 animate-spin" /> Importing…
                </>
              ) : (
                <>
                  <Upload className="size-4" /> Import {mode === "add" ? "(add to store)" : "(replace store)"}
                </>
              )}
            </Button>
          </div>
        </DialogPrimitive.Content>
      </DialogPrimitive.Portal>
    </DialogPrimitive.Root>
  );
}

// ── Panes ────────────────────────────────────────────────────────────────────────────────────

function DrawerTab({
  id,
  current,
  onSelect,
  icon: Icon,
  label,
}: {
  id: Tab;
  current: Tab;
  onSelect: (t: Tab) => void;
  icon: typeof FileUp;
  label: string;
}) {
  const active = id === current;
  return (
    <button
      role="tab"
      aria-selected={active}
      data-import-tab={id}
      onClick={() => onSelect(id)}
      className={cn(
        "flex flex-1 items-center justify-center gap-1.5 border-b-2 px-3 py-2 font-medium",
        active
          ? "border-primary text-foreground"
          : "border-transparent text-muted-foreground hover:text-foreground",
      )}
    >
      <Icon className="size-3.5" /> {label}
    </button>
  );
}

function FilePane({
  native,
  filePath,
  onPickFile,
  onClear,
}: {
  native: boolean;
  filePath: string;
  onPickFile: () => void;
  onClear: () => void;
}) {
  if (!native) {
    return (
      <div className="rounded-md border border-[var(--warning)]/40 bg-[var(--warning)]/5 p-3 text-xs text-muted-foreground">
        <p className="mb-1 flex items-center gap-1.5 font-medium text-foreground">
          <AlertTriangle className="size-3.5 text-[var(--warning)]" /> Desktop app only
        </p>
        Reading a file from disk — including <strong>compressed</strong> (.gz / .bz2 / .zst) and{" "}
        <strong>native-only HDT</strong> archives — needs the desktop app&rsquo;s native loader (no
        ~2&nbsp;GiB browser-tab ceiling). In this web build, use the <strong>URL</strong> or{" "}
        <strong>Paste</strong> tab instead.
      </div>
    );
  }
  return (
    <div className="space-y-3">
      <p className="text-xs text-muted-foreground">
        Load an RDF file through the native engine — Turtle / N-Triples / N-Quads / TriG / JSON-LD,
        including <strong>compressed</strong> (.gz / .bz2 / .zst) streams and{" "}
        <strong>native-only HDT</strong> (.hdt / .hdt.gz) archives. Named graphs are preserved.
      </p>
      <Button variant="outline" className="w-full" onClick={onPickFile}>
        <FolderOpen className="size-4" /> Choose a file…
      </Button>
      {filePath && (
        <div className="flex items-start gap-2 rounded-md border bg-background px-3 py-2 text-xs">
          <FileUp className="mt-0.5 size-3.5 shrink-0 text-muted-foreground" />
          <span className="min-w-0 flex-1 break-all font-mono">{filePath}</span>
          <button
            onClick={onClear}
            aria-label="Clear selected file"
            className="shrink-0 text-muted-foreground hover:text-foreground"
          >
            <X className="size-3.5" />
          </button>
        </div>
      )}
      {filePath && (
        <FormatNote
          label={fileLabel(filePath)}
          format={guessFormat(filePath)}
          compressed={isCompressed(filePath)}
          hdt={isHdt(filePath)}
        />
      )}
    </div>
  );
}

function UrlPane({ url, onUrl }: { url: string; onUrl: (v: string) => void }) {
  return (
    <div className="space-y-3">
      <p className="text-xs text-muted-foreground">
        Fetch an RDF document from a URL and load it into the store. The source is recorded as a
        re-fetchable workspace source. The format is auto-detected from the served{" "}
        <code>Content-Type</code> (or the URL extension).
      </p>
      <input
        type="url"
        inputMode="url"
        placeholder="https://example.org/data.ttl"
        value={url}
        onChange={(e) => onUrl(e.target.value)}
        className="w-full rounded-md border bg-background px-3 py-2 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring/50"
      />
      {url.trim() && (
        <FormatNote
          label={urlLabel(url)}
          format={guessFormat(url)}
          compressed={isCompressed(url)}
          hdt={isHdt(url)}
        />
      )}
    </div>
  );
}

function PastePane({
  text,
  onText,
  format,
  onFormat,
}: {
  text: string;
  onText: (v: string) => void;
  format: string;
  onFormat: (v: string) => void;
}) {
  // HDT is a BINARY archive format — it cannot be pasted as text, so it is never a paste option
  // (it is offered only on the File tab, where the native loader reads the binary off disk).
  const options = FORMAT_OPTIONS.filter((f) => f.value !== "hdt");
  return (
    <div className="space-y-3">
      <div className="flex items-center gap-2">
        <label htmlFor="import-paste-format" className="text-xs font-medium text-muted-foreground">
          Format
        </label>
        <select
          id="import-paste-format"
          value={format}
          onChange={(e) => onFormat(e.target.value)}
          className="rounded-md border bg-background px-2 py-1 text-xs outline-none focus-visible:ring-2 focus-visible:ring-ring/50"
        >
          {options.map((f) => (
            <option key={f.value} value={f.value}>
              {f.label}
            </option>
          ))}
        </select>
      </div>
      <textarea
        value={text}
        onChange={(e) => onText(e.target.value)}
        placeholder={"<http://example.org/a> <http://example.org/p> <http://example.org/b> ."}
        spellCheck={false}
        className="h-48 w-full resize-y rounded-md border bg-background px-3 py-2 font-mono text-xs outline-none focus-visible:ring-2 focus-visible:ring-ring/50"
      />
    </div>
  );
}

function FormatNote({
  label,
  format,
  compressed,
  hdt,
}: {
  label: string;
  format: string;
  compressed: boolean;
  hdt: boolean;
}) {
  return (
    <p className="text-[11px] text-muted-foreground">
      <span className="font-mono">{label}</span> → parsed as{" "}
      <span className="font-medium text-foreground">{format}</span>
      {compressed && !hdt && " (decompressed natively)"}
      {hdt && " (native-only HDT)"}.
    </p>
  );
}

function OptionsRow({
  mode,
  onMode,
  preserveGraphs,
  onPreserveGraphs,
}: {
  mode: ImportMode;
  onMode: (m: ImportMode) => void;
  preserveGraphs: boolean;
  onPreserveGraphs: (v: boolean) => void;
}) {
  return (
    <div className="space-y-2 text-xs">
      <div className="flex items-center gap-2">
        <span className="font-medium text-muted-foreground">Mode</span>
        <div className="inline-flex overflow-hidden rounded-md border">
          <ModeButton current={mode} value="add" label="Add to store" onSelect={onMode} />
          <ModeButton current={mode} value="replace" label="Replace store" onSelect={onMode} />
        </div>
      </div>
      <label className="flex cursor-pointer items-center gap-2">
        <input
          type="checkbox"
          checked={preserveGraphs}
          onChange={(e) => onPreserveGraphs(e.target.checked)}
          className="size-3.5 accent-[var(--primary)]"
        />
        <span className="text-muted-foreground">
          Preserve named graphs (route quad formats through the dataset loader)
        </span>
      </label>
    </div>
  );
}

function ModeButton({
  current,
  value,
  label,
  onSelect,
}: {
  current: ImportMode;
  value: ImportMode;
  label: string;
  onSelect: (m: ImportMode) => void;
}) {
  const active = current === value;
  return (
    <button
      onClick={() => onSelect(value)}
      className={cn(
        "px-2.5 py-1",
        active ? "bg-primary text-primary-foreground" : "bg-background hover:bg-accent",
      )}
    >
      {label}
    </button>
  );
}
