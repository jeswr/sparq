// [FABLE-5] sq-vnh1v (epic sq-2ucrz) — shared BROWSER multi-file ingest for the epic's three
// upload consumers: RDF import (sq-eydh9), SHACL shapes (sq-txrui), N3 rules (sq-glo5r).
//
// Pure library, no React: pick many text files at once (File System Access
// `showOpenFilePicker` where available, a hidden `<input type="file" multiple>` everywhere
// else) and read files dropped onto a target (`DataTransfer`). Both paths resolve the SAME
// `IngestResult` shape so every consumer handles one contract.
//
// INVARIANTS (browser-parity floor, research/gui-consolidation-fix-plan.md §1.4):
//  * works in the static /app export with NO server and NO Tauri global — the
//    `input[multiple]` fallback is always available; File System Access is opportunistic
//    (and any picker failure other than a user cancel falls back to the input path);
//  * NEVER silently drops a selected/dropped file — every file resolves to an `accepted`
//    entry or a `rejected` entry with a human-readable reason (callers surface both);
//  * SSR/prerender-safe — importing this module touches no DOM; calling the picker outside
//    a browser resolves to an empty result instead of throwing.
//
// Format guessing intentionally stays in `rdf-format.ts` (`guessFormat`) — consumers call it
// per accepted file; this module only moves bytes.

/** One successfully read file: its name, full text, and on-disk size in bytes. */
export interface IngestedFile {
  name: string;
  text: string;
  bytes: number;
  /** Inner filename used for format detection after archive decompression. */
  effectiveName?: string;
}

/** One file that could NOT be ingested, with a human-readable reason the UI must surface. */
export interface RejectedFile {
  name: string;
  reason: string;
}

/**
 * The single result contract for both the picker and the drop reader. Every file the user
 * selected or dropped appears in exactly one of the two lists — no silent drops.
 */
export interface IngestResult {
  accepted: IngestedFile[];
  rejected: RejectedFile[];
}

/** Options shared by {@link pickTextFiles} and {@link readDroppedFiles}. */
export interface IngestOptions {
  /**
   * Dot-prefixed extensions to allow (e.g. `[".ttl", ".nt", ".shacl"]`), matched
   * case-insensitively against the filename tail. Absent/empty = accept any file.
   * Non-matching files are REJECTED with an explicit reason (never filtered silently).
   */
  accept?: string[];
  /**
   * Allow more than one file (default `true` — this library exists for bulk upload).
   * When `false`, the first acceptable file wins and the rest are rejected with a reason.
   */
  multiple?: boolean;
}

/** Options for {@link pickTextFiles}. */
export interface PickOptions extends IngestOptions {
  /** Dialog filter description shown by the File System Access picker (default "Supported files"). */
  description?: string;
}

/** True when `name` matches the `accept` extension list (or the list is absent/empty). */
export function matchesAccept(name: string, accept?: string[]): boolean {
  if (!accept?.length) return true;
  const lower = name.toLowerCase();
  return accept.some((ext) => lower.endsWith(ext.toLowerCase()));
}

/** True when a drag carries real files (vs dragged text/links), so drop targets can ignore the rest. */
export function hasDraggedFiles(dataTransfer: DataTransfer | null): boolean {
  return !!dataTransfer && Array.from(dataTransfer.types ?? []).includes("Files");
}

const emptyResult = (): IngestResult => ({ accepted: [], rejected: [] });

function errorMessage(err: unknown): string {
  return err instanceof Error ? err.message : String(err);
}

function isAbortError(err: unknown): boolean {
  return err instanceof DOMException && err.name === "AbortError";
}

function rejectedReasonUnsupported(accept: string[]): string {
  return `unsupported file type — expected ${accept.join(" / ")}`;
}

/**
 * Read a list of `File` objects into an {@link IngestResult}: extension filter first, the
 * single-file cap second, then `File.text()` — a per-file read failure becomes a rejection,
 * never an exception and never a silent drop.
 */
async function readFiles(files: File[], opts: IngestOptions): Promise<IngestResult> {
  const accepted: IngestedFile[] = [];
  const rejected: RejectedFile[] = [];
  for (const file of files) {
    if (!matchesAccept(file.name, opts.accept)) {
      rejected.push({ name: file.name, reason: rejectedReasonUnsupported(opts.accept ?? []) });
      continue;
    }
    if (opts.multiple === false && accepted.length >= 1) {
      rejected.push({ name: file.name, reason: "only one file can be used here" });
      continue;
    }
    try {
      const text = await file.text();
      accepted.push({ name: file.name, text, bytes: file.size });
    } catch (err) {
      // e.g. a directory masquerading as a File, or a file deleted between pick and read.
      rejected.push({ name: file.name, reason: `could not be read: ${errorMessage(err)}` });
    }
  }
  return { accepted, rejected };
}

// ── File System Access (opportunistic; typed locally — not yet in TS lib.dom) ─────────────────

interface PickerFileHandle {
  readonly name: string;
  getFile(): Promise<File>;
}

interface OpenFilePickerOptions {
  multiple?: boolean;
  excludeAcceptAllOption?: boolean;
  types?: { description?: string; accept: Record<string, string[]> }[];
}

type ShowOpenFilePicker = (options?: OpenFilePickerOptions) => Promise<PickerFileHandle[]>;

// ── The universal fallback: a hidden <input type="file" multiple> ─────────────────────────────

function pickViaInput(opts: PickOptions): Promise<IngestResult> {
  return new Promise((resolve) => {
    const input = document.createElement("input");
    input.type = "file";
    input.multiple = opts.multiple !== false;
    if (opts.accept?.length) input.accept = opts.accept.join(",");
    input.style.display = "none";
    // The input must be in the document for some engines (notably iOS Safari) to open it.
    document.body.appendChild(input);
    const finish = (result: Promise<IngestResult> | IngestResult) => {
      input.remove();
      resolve(result instanceof Promise ? result : Promise.resolve(result));
    };
    input.addEventListener(
      "change",
      () => finish(readFiles(Array.from(input.files ?? []), opts)),
      { once: true },
    );
    // Modern browsers fire `cancel` when the dialog is dismissed; nothing was selected,
    // so an empty result upholds the no-silent-drop invariant.
    input.addEventListener("cancel", () => finish(emptyResult()), { once: true });
    input.click();
  });
}

/**
 * Open a (multi-)file picker and read every selected file as text.
 *
 * Uses File System Access `showOpenFilePicker` when the browser provides it; otherwise —
 * and whenever the picker fails for any reason other than a user cancel (cross-origin
 * frame `SecurityError`, option `TypeError`, …) — falls back to a hidden
 * `<input type="file" multiple>`, which is the browser-parity floor. A user cancel
 * resolves to an empty result. Outside a browser (SSR/prerender) resolves empty, never throws.
 */
export async function pickTextFiles(opts: PickOptions = {}): Promise<IngestResult> {
  if (typeof window === "undefined" || typeof document === "undefined") return emptyResult();
  const picker = (window as { showOpenFilePicker?: ShowOpenFilePicker }).showOpenFilePicker;
  if (typeof picker !== "function") return pickViaInput(opts);
  try {
    const handles = await picker({
      multiple: opts.multiple !== false,
      // Keep "All files" available: the extension filter below REJECTS with a reason
      // rather than pretending off-list files were never selected.
      excludeAcceptAllOption: false,
      types: opts.accept?.length
        ? [
            {
              description: opts.description ?? "Supported files",
              // The MIME key only labels the filter group; the extension list drives it.
              accept: { "text/plain": opts.accept },
            },
          ]
        : undefined,
    });
    const preRejected: RejectedFile[] = [];
    const files: File[] = [];
    for (const handle of handles) {
      try {
        files.push(await handle.getFile());
      } catch (err) {
        preRejected.push({ name: handle.name, reason: `could not be read: ${errorMessage(err)}` });
      }
    }
    const read = await readFiles(files, opts);
    return { accepted: read.accepted, rejected: [...preRejected, ...read.rejected] };
  } catch (err) {
    if (isAbortError(err)) return emptyResult(); // user cancelled — nothing was selected
    return pickViaInput(opts); // any other picker failure → the fallback floor
  }
}

/**
 * Read every file in a drop's `DataTransfer` as text.
 *
 * MUST be called synchronously from the `drop` handler: the `File` objects are extracted
 * from `dataTransfer.items` before this function's first `await`, because browsers neuter
 * the item list once the drop event handler returns. Dropped directories and non-file drag
 * items that claim to be files are rejected with explicit reasons; dragged text/links
 * (`kind !== "file"`) are not files and are ignored.
 */
export async function readDroppedFiles(
  dataTransfer: DataTransfer | null,
  opts: IngestOptions = {},
): Promise<IngestResult> {
  if (!dataTransfer) return emptyResult();
  const files: File[] = [];
  const preRejected: RejectedFile[] = [];
  if (dataTransfer.items?.length) {
    // Synchronous extraction — no await before this loop completes (see doc comment).
    for (const item of Array.from(dataTransfer.items)) {
      if (item.kind !== "file") continue;
      const entry = typeof item.webkitGetAsEntry === "function" ? item.webkitGetAsEntry() : null;
      const file = item.getAsFile();
      if (entry?.isDirectory) {
        preRejected.push({
          name: entry.name || file?.name || "(directory)",
          reason: "directories cannot be ingested — drop the files inside it instead",
        });
        continue;
      }
      if (!file) {
        preRejected.push({
          name: entry?.name ?? "(unreadable item)",
          reason: "the browser could not expose this dropped item as a file",
        });
        continue;
      }
      files.push(file);
    }
  } else {
    // Engines without DataTransferItemList expose the flat FileList instead.
    files.push(...Array.from(dataTransfer.files ?? []));
  }
  const read = await readFiles(files, opts);
  return { accepted: read.accepted, rejected: [...preRejected, ...read.rejected] };
}
