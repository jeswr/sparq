"use client";

// (sq-txrui) [SONNET-4.6] — SHACL SHAPES SOURCES strip: bulk multi-file upload with per-source
// enable/disable toggle.
//
// Each uploaded .ttl shape file becomes a ShapeSource entry. The parent (shacl-tool.tsx)
// concatenates the editor doc + every ENABLED source text before passing it to sparqShaclValidate.
// Prefix redeclaration across Turtle docs is legal and handled by the WASM parser.
//
// Design note: the strip is intentionally compact — name truncated at 1fr, toggle and remove at
// fixed right. A disabled source has opacity-50 treatment on the name to signal it contributes
// nothing to the current validation run.
//
// File ingest: a PERSISTENT hidden <input type="file" multiple> (ref) is always in the DOM.
// The "Browse files…" button programmatically clicks it. The input also handles drag-and-drop
// via readDroppedFiles. This design allows Playwright to use setInputFiles() on the labelled
// input directly — `waitForEvent('filechooser')` + `showOpenFilePicker` can mis-fire in
// headless environments; a persistent input ref is the robust testable floor.
//
// INVARIANTS (bead sq-txrui):
//   * with zero sources the strip shows only the upload affordance (same validation input as
//     before — zero uploaded sources → tool behaves exactly as the old textarea);
//   * a disabled source contributes ZERO text to the concatenated shapes doc;
//   * every file that comes through the input fires the 'change' event (Playwright's
//     setInputFiles() path) or the 'drop' event (drag-and-drop path).
//
// Stable E2E hooks:
//   [data-sources-list]           — the <ul> of source rows (present iff sources.length > 0)
//   [data-source-name]            — the <span> with the file name (per row)
//   [data-source-toggle]          — the toggle button (aria-pressed="true/false" per row)
//   [data-source-remove]          — the remove button (per row)
//   [data-sources-file-input]     — the hidden <input type="file" multiple> (setInputFiles target)
//   button "Browse files…"        — the keyboard-accessible pick trigger (role="button")

import * as React from "react";
import { X, Eye, EyeOff, FileUp, Upload, Loader2 } from "lucide-react";

import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { useFileDrop } from "@/components/workbench/dropzone";
import { matchesAccept, type IngestResult } from "@/lib/file-ingest";

/** One uploaded shape source — carries a stable React key, its file name, Turtle text and
 * whether it is currently contributing to validation. */
export interface ShapeSource {
  id: string;
  name: string;
  text: string;
  enabled: boolean;
}

export interface ShaclSourcesProps {
  sources: ShapeSource[];
  /** Full replace — the parent owns the list. */
  onChange: (sources: ShapeSource[]) => void;
}

// ── internal helpers ────────────────────────────────────────────────────────────────────────────

const ACCEPTED_EXTS = [".ttl", ".shacl", ".n3"];

function makeId(name: string): string {
  return `${name}-${Date.now()}-${Math.random().toString(36).slice(2)}`;
}

function ingestToSources(result: IngestResult): ShapeSource[] {
  return result.accepted.map((f) => ({
    id: makeId(f.name),
    name: f.name,
    text: f.text,
    enabled: true,
  }));
}

/**
 * The SHAPES SOURCES strip: a persistent file input + drag-and-drop surface for bulk .ttl upload,
 * plus a list of accepted sources each with an enable/disable toggle and a remove button.
 *
 * The parent is responsible for concatenating `source.text` for every `source.enabled` entry into
 * the shapes document before calling `sparqShaclValidate`.
 */
export function ShaclSources({ sources, onChange }: ShaclSourcesProps) {
  // ── persistent file input ref ────────────────────────────────────────────────────────────────
  // A permanent DOM node so Playwright can call setInputFiles() on it at any time.
  const fileInputRef = React.useRef<HTMLInputElement>(null);
  const [busy, setBusy] = React.useState(false);

  // ── add incoming files ───────────────────────────────────────────────────────────────────────
  const appendSources = React.useCallback(
    (result: IngestResult) => {
      const incoming = ingestToSources(result);
      if (incoming.length > 0) onChange([...sources, ...incoming]);
    },
    [sources, onChange],
  );

  // ── input change handler (covers both manual pick AND Playwright setInputFiles) ──────────────
  const onInputChange = React.useCallback(
    async (e: React.ChangeEvent<HTMLInputElement>) => {
      const files = Array.from(e.target.files ?? []);
      // Reset the input value so the same file can be re-selected later.
      e.target.value = "";
      if (!files.length) return;
      setBusy(true);
      try {
        const accepted: { name: string; text: string; bytes: number }[] = [];
        const rejected: { name: string; reason: string }[] = [];
        for (const file of files) {
          if (!matchesAccept(file.name, ACCEPTED_EXTS)) {
            rejected.push({
              name: file.name,
              reason: `unsupported file type — expected ${ACCEPTED_EXTS.join(" / ")}`,
            });
            continue;
          }
          try {
            accepted.push({ name: file.name, text: await file.text(), bytes: file.size });
          } catch (err) {
            rejected.push({
              name: file.name,
              reason: `could not be read: ${err instanceof Error ? err.message : String(err)}`,
            });
          }
        }
        appendSources({ accepted, rejected });
      } finally {
        setBusy(false);
      }
    },
    [appendSources],
  );

  // ── drag-and-drop ────────────────────────────────────────────────────────────────────────────
  const { dragActive, targetProps } = useFileDrop({
    accept: ACCEPTED_EXTS,
    multiple: true,
    onFiles: appendSources,
  });

  // ── per-source actions ───────────────────────────────────────────────────────────────────────
  const toggleSource = React.useCallback(
    (id: string) => {
      onChange(sources.map((s) => (s.id === id ? { ...s, enabled: !s.enabled } : s)));
    },
    [sources, onChange],
  );

  const removeSource = React.useCallback(
    (id: string) => {
      onChange(sources.filter((s) => s.id !== id));
    },
    [sources, onChange],
  );

  const enabledCount = sources.filter((s) => s.enabled).length;

  // ── render ───────────────────────────────────────────────────────────────────────────────────
  return (
    <div className="flex shrink-0 flex-col border-b">
      {/* strip header */}
      <div className="flex shrink-0 items-center gap-2 border-b bg-card px-3 py-1.5">
        <span className="text-xs font-medium text-muted-foreground">Shape sources</span>
        {sources.length > 0 ? (
          <span className="text-[11px] text-muted-foreground">
            {enabledCount} / {sources.length} active
          </span>
        ) : null}
      </div>

      {/* source list — only rendered when at least one source has been uploaded */}
      {sources.length > 0 ? (
        <ul data-sources-list className="divide-y overflow-y-auto" style={{ maxHeight: "9rem" }}>
          {sources.map((src) => (
            <li key={src.id} className="flex items-center gap-2 px-3 py-1.5 text-xs">
              {/* toggle */}
              <button
                type="button"
                data-source-toggle
                aria-pressed={src.enabled}
                aria-label={src.enabled ? `Disable ${src.name}` : `Enable ${src.name}`}
                onClick={() => toggleSource(src.id)}
                className={cn(
                  "shrink-0 rounded p-0.5 text-muted-foreground hover:text-foreground",
                  !src.enabled && "opacity-50",
                )}
              >
                {src.enabled ? (
                  <Eye className="size-3.5" aria-hidden />
                ) : (
                  <EyeOff className="size-3.5" aria-hidden />
                )}
              </button>

              {/* file name */}
              <span
                data-source-name
                title={src.name}
                className={cn(
                  "min-w-0 flex-1 truncate font-mono",
                  !src.enabled && "opacity-50",
                )}
              >
                {src.name}
              </span>

              {/* remove */}
              <Button
                type="button"
                variant="ghost"
                size="sm"
                data-source-remove
                aria-label={`Remove ${src.name}`}
                onClick={() => removeSource(src.id)}
                className="h-5 w-5 shrink-0 p-0 text-muted-foreground hover:text-destructive"
              >
                <X className="size-3" aria-hidden />
              </Button>
            </li>
          ))}
        </ul>
      ) : null}

      {/* upload affordance: drag target + keyboard-accessible pick button.
          The hidden <input> is always in the DOM — required for Playwright setInputFiles(). */}
      <input
        ref={fileInputRef}
        type="file"
        multiple
        accept={ACCEPTED_EXTS.join(",")}
        data-sources-file-input
        aria-label="Select shape files"
        tabIndex={-1}
        className="sr-only"
        onChange={onInputChange}
      />

      <div
        data-slot="dropzone"
        {...targetProps}
        className={cn(
          "relative m-2 rounded-md border border-dashed p-4 text-center text-xs transition-colors",
          dragActive ? "border-primary bg-primary/5" : "border-border",
        )}
      >
        <FileUp className="mx-auto size-5 text-muted-foreground" aria-hidden />
        <p className="mt-1.5 text-sm text-muted-foreground">Drop .ttl shape files here</p>
        <p className="mt-0.5 text-[11px] text-muted-foreground">.ttl · .shacl · .n3</p>
        <Button
          type="button"
          variant="outline"
          size="sm"
          className="mt-2"
          onClick={() => fileInputRef.current?.click()}
          disabled={busy}
          aria-label="Browse files…"
        >
          {busy ? <Loader2 className="animate-spin" /> : <Upload />}
          Browse files…
        </Button>
        <span className="sr-only" role="status">
          {dragActive ? "Release to drop the files" : ""}
        </span>
      </div>
    </div>
  );
}
