"use client";

// [OPUS-4.8] sq-xvj9 (epic sq-ixc3) — the DatasetViewer's EXPORT action: serialise the WHOLE live
// store to a pretty, prefix-abbreviated RDF document and hand it to the browser as a file download.
//
// This is a NEW capability, NOT a swap of the result-graph views: an explicit "export this dataset"
// affordance next to the rail's "+ Import data…" entry point. It calls the engine context's
// `exportStore` (which drives the wasm `serialize` binding with the site's COMMON_PREFIXES prefix
// map — sq-l5kr) and the shared `downloadText` DOM helper. Three formats, matching what the engine
// writer emits: Turtle (default graph — Turtle has no named-graph syntax), TriG and JSON-LD (the
// whole dataset). The scope of each is shown in the menu so it is never silently misrepresented.
//
// HONESTY. Nothing is fabricated: if the store is cold/empty the trigger is disabled; if a lean
// bundle lacks the serialise binding, `exportStore` returns null and a clear message is shown.

import * as React from "react";
import { DropdownMenu as DropdownMenuPrimitive } from "radix-ui";
import { Download, AlertTriangle } from "lucide-react";

import { cn } from "@/lib/utils";
import { useEngine } from "@/lib/engine-context";
import { downloadText } from "@/lib/download";
import { EXPORT_FORMATS, EXPORT_FORMAT_META, exportFilename, type ExportFormat } from "@/lib/rdf-format";

/**
 * The "Export data…" control for the left rail's datasets tree: a dashed teal trigger (sibling of
 * the "+ Import data…" affordance) that opens a small menu of the export serialisations. Selecting
 * one serialises the whole live store and downloads it. Disabled until the engine is ready with a
 * non-empty store.
 */
export function ExportDataMenu() {
  const { status, storeSize, exportStore } = useEngine();
  const [error, setError] = React.useState<string | null>(null);

  const disabled = status.kind !== "ready" || storeSize === 0;

  const onExport = React.useCallback(
    (format: ExportFormat) => {
      setError(null);
      const meta = EXPORT_FORMAT_META[format];
      try {
        const text = exportStore(format);
        if (text === null) {
          setError("The engine is not ready, or this build cannot serialise RDF.");
          return;
        }
        downloadText(exportFilename(format), text, meta.mime);
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      }
    },
    [exportStore],
  );

  return (
    <div className="mt-1">
      <DropdownMenuPrimitive.Root>
        <DropdownMenuPrimitive.Trigger asChild>
          <button
            type="button"
            disabled={disabled}
            data-export-trigger="rail"
            className={cn(
              "flex w-full items-center gap-1.5 rounded-md border border-dashed border-teal-line px-2 py-1.5 text-xs text-muted-foreground",
              "hover:bg-teal-faint hover:text-foreground",
              "disabled:cursor-not-allowed disabled:opacity-50 disabled:hover:bg-transparent disabled:hover:text-muted-foreground",
            )}
            title={
              disabled
                ? "Load some data first, then export the dataset"
                : "Export the dataset as pretty RDF: Turtle (default graph), TriG or JSON-LD (whole dataset)"
            }
          >
            <Download className="size-3" /> Export data…
          </button>
        </DropdownMenuPrimitive.Trigger>
        <DropdownMenuPrimitive.Portal>
          <DropdownMenuPrimitive.Content
            data-export-menu
            align="start"
            sideOffset={4}
            className={cn(
              "z-50 min-w-[13rem] overflow-hidden rounded-md border bg-popover p-1 text-popover-foreground shadow-md",
              "data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0",
            )}
          >
            <DropdownMenuPrimitive.Label className="px-2 py-1 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
              Export dataset as…
            </DropdownMenuPrimitive.Label>
            {EXPORT_FORMATS.map((f) => (
              <DropdownMenuPrimitive.Item
                key={f.value}
                data-export-format={f.value}
                onSelect={() => onExport(f.value)}
                className="flex cursor-pointer items-center justify-between gap-3 rounded-sm px-2 py-1.5 text-xs outline-none focus:bg-accent focus:text-accent-foreground"
              >
                <span className="font-medium">{f.label}</span>
                <span className="text-[10px] text-muted-foreground">{f.scope}</span>
              </DropdownMenuPrimitive.Item>
            ))}
            <p className="px-2 pb-1 pt-1.5 text-[10px] leading-snug text-muted-foreground">
              Pretty-printed, IRIs abbreviated with the common prefixes.
            </p>
          </DropdownMenuPrimitive.Content>
        </DropdownMenuPrimitive.Portal>
      </DropdownMenuPrimitive.Root>
      {error && (
        <p
          role="alert"
          aria-live="assertive"
          data-export-feedback="error"
          className="mt-1 flex items-start gap-1.5 px-1 text-[11px] text-destructive"
        >
          <AlertTriangle className="mt-0.5 size-3 shrink-0" />
          <span>{error}</span>
        </p>
      )}
    </div>
  );
}
