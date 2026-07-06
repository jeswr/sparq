"use client";

// [FABLE-5] sq-vnh1v (epic sq-2ucrz) — reusable drag-and-drop surface over `lib/file-ingest`.
//
// Three exports, one contract (`IngestResult` — accepted + rejected, no silent drops):
//   * `useFileDrop`   — the raw dragenter/over/leave/drop wiring as spreadable props, for
//                       consumers that render their own affordance (e.g. a whole-workbench
//                       overlay in sq-eydh9);
//   * `<DropTarget>`  — wraps arbitrary children as a drop target with a visible drag-over
//                       overlay. It adds NO keyboard path itself — pair it with a visible
//                       picker control (or use `<Dropzone>`);
//   * `<Dropzone>`    — a standalone dashed panel: drop affordance + a keyboard-accessible
//                       "Browse files" button that runs `pickTextFiles` (File System Access
//                       where available, `input[multiple]` fallback = the browser-parity floor).
//
// Consumers: RDF import (sq-eydh9), SHACL shapes (sq-txrui), N3 rules (sq-glo5r). Works in the
// static /app export — no server, no Tauri global; SSR-safe ("use client" + no module-scope DOM).

import * as React from "react";
import { FileUp, Loader2, Upload } from "lucide-react";

import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import {
  hasDraggedFiles,
  pickTextFiles,
  readDroppedFiles,
  type IngestResult,
} from "@/lib/file-ingest";

// ── useFileDrop — the shared drag/drop wiring ──────────────────────────────────────────────────

export interface FileDropOptions {
  /** Receives EVERY outcome — accepted files and per-file rejections — from a drop or pick. */
  onFiles: (result: IngestResult) => void;
  /** Dot-prefixed extensions to allow (off-list files are rejected with a reason). */
  accept?: string[];
  /** Allow more than one file (default true; extras beyond the first are rejected when false). */
  multiple?: boolean;
  /** Ignore drags and disable the picker. */
  disabled?: boolean;
}

/** The drag/drop handlers to spread onto a drop-target element. */
export interface FileDropTargetProps {
  onDragEnter: React.DragEventHandler<HTMLElement>;
  onDragOver: React.DragEventHandler<HTMLElement>;
  onDragLeave: React.DragEventHandler<HTMLElement>;
  onDrop: React.DragEventHandler<HTMLElement>;
}

/**
 * Drag-and-drop wiring for a files drop target: reacts only to drags that actually carry
 * files, tracks enter/leave depth (child elements re-fire both events), and resolves the
 * drop through `readDroppedFiles` into the consumer's `onFiles` callback.
 */
export function useFileDrop({ onFiles, accept, multiple, disabled }: FileDropOptions): {
  dragActive: boolean;
  targetProps: FileDropTargetProps;
} {
  const [dragActive, setDragActive] = React.useState(false);
  const depth = React.useRef(0);

  const targetProps: FileDropTargetProps = {
    onDragEnter: (e) => {
      if (disabled || !hasDraggedFiles(e.dataTransfer)) return;
      e.preventDefault();
      depth.current += 1;
      setDragActive(true);
    },
    onDragOver: (e) => {
      if (disabled || !hasDraggedFiles(e.dataTransfer)) return;
      e.preventDefault();
      e.dataTransfer.dropEffect = "copy";
    },
    onDragLeave: (e) => {
      if (disabled || !hasDraggedFiles(e.dataTransfer)) return;
      depth.current = Math.max(0, depth.current - 1);
      if (depth.current === 0) setDragActive(false);
    },
    onDrop: (e) => {
      if (disabled || !hasDraggedFiles(e.dataTransfer)) return;
      e.preventDefault();
      depth.current = 0;
      setDragActive(false);
      // readDroppedFiles extracts the File objects synchronously (before its first await) —
      // load-bearing: browsers neuter DataTransfer items once this handler returns.
      void readDroppedFiles(e.dataTransfer, { accept, multiple }).then(onFiles);
    },
  };

  return { dragActive, targetProps };
}

// ── <DropTarget> — wrap existing UI as a drop surface ──────────────────────────────────────────

export interface DropTargetProps extends FileDropOptions {
  /** Overlay affordance text while a files drag hovers the target. */
  label?: string;
  className?: string;
  children: React.ReactNode;
}

/**
 * Wraps `children` as a files drop target with a visible drag-over overlay. Provides no
 * keyboard path of its own — the consumer must keep a visible, focusable picker control
 * (e.g. an Import button or a `<Dropzone>`) so keyboard-only users reach the same files.
 */
export function DropTarget({
  onFiles,
  accept,
  multiple,
  disabled,
  label = "Drop files to add them",
  className,
  children,
}: DropTargetProps) {
  const { dragActive, targetProps } = useFileDrop({ onFiles, accept, multiple, disabled });
  return (
    <div data-slot="drop-target" className={cn("relative", className)} {...targetProps}>
      {children}
      {dragActive ? (
        <div
          aria-hidden
          className="pointer-events-none absolute inset-0 z-50 flex items-center justify-center rounded-md border-2 border-dashed border-primary bg-background/80"
        >
          <div className="flex items-center gap-2 text-sm font-medium">
            <FileUp className="size-5" aria-hidden />
            {label}
          </div>
        </div>
      ) : null}
      <span className="sr-only" role="status">
        {dragActive ? label : ""}
      </span>
    </div>
  );
}

// ── <Dropzone> — a standalone panel with the keyboard-accessible fallback ──────────────────────

export interface DropzoneProps extends FileDropOptions {
  /** Main affordance line (default "Drag & drop files here"). */
  label?: string;
  /** Secondary hint line; defaults to the `accept` extension list. */
  hint?: string;
  /** Label for the keyboard-accessible picker button (default "Browse files…"). */
  browseLabel?: string;
  className?: string;
}

/**
 * A standalone dashed drop panel: drop files onto it, or activate the (keyboard-accessible)
 * browse button to pick them — File System Access where available, `input[multiple]`
 * everywhere else, so the flow works in a plain browser with no Tauri global.
 */
export function Dropzone({
  onFiles,
  accept,
  multiple,
  disabled,
  label = "Drag & drop files here",
  hint,
  browseLabel = "Browse files…",
  className,
}: DropzoneProps) {
  const { dragActive, targetProps } = useFileDrop({ onFiles, accept, multiple, disabled });
  const [busy, setBusy] = React.useState(false);

  const browse = async () => {
    setBusy(true);
    try {
      onFiles(await pickTextFiles({ accept, multiple }));
    } finally {
      setBusy(false);
    }
  };

  const hintText = hint ?? (accept?.length ? accept.join(" · ") : undefined);

  return (
    <div
      data-slot="dropzone"
      {...targetProps}
      className={cn(
        "relative rounded-md border border-dashed p-5 text-center transition-colors",
        dragActive ? "border-primary bg-primary/5" : "border-border",
        disabled && "opacity-50",
        className,
      )}
    >
      <FileUp className="mx-auto size-6 text-muted-foreground" aria-hidden />
      <p className="mt-2 text-sm">{label}</p>
      {hintText ? <p className="mt-1 text-xs text-muted-foreground">{hintText}</p> : null}
      <Button
        type="button"
        variant="outline"
        size="sm"
        className="mt-3"
        onClick={browse}
        disabled={disabled || busy}
      >
        {busy ? <Loader2 className="animate-spin" /> : <Upload />}
        {browseLabel}
      </Button>
      <span className="sr-only" role="status">
        {dragActive ? "Release to drop the files" : ""}
      </span>
    </div>
  );
}
