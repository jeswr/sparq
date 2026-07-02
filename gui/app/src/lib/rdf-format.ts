// [OPUS-4.8] sq-ixc3.13 (epic sq-ixc3) — RDF format auto-detection for the Import drawer.
//
// A small, pure, dependency-free helper (testable with no DOM): guess the engine format from a
// filename/URL, map a `Content-Type` to a format, and — load-bearing for the NATIVE loader —
// strip a compression suffix so a `data.ttl.gz` is parsed as `turtle` (the native loader decodes
// the `.gz` itself). Mirrors the site's `repl-dataset.ts` opinion on which extensions carry which
// format (site-local there; the GUI cannot import from `site/`, so this is the GUI's own copy of
// that tiny, stable table).

/** A selectable RDF serialisation in the drawer's format dropdown. */
export interface FormatOption {
  value: string;
  label: string;
  exts: string[];
}

/** The formats the engine ingests, with their file extensions. `hdt` is native-only (see drawer). */
export const FORMAT_OPTIONS: FormatOption[] = [
  { value: "turtle", label: "Turtle (.ttl)", exts: ["ttl", "turtle"] },
  { value: "ntriples", label: "N-Triples (.nt)", exts: ["nt", "ntriples"] },
  { value: "nquads", label: "N-Quads (.nq)", exts: ["nq", "nquads"] },
  { value: "trig", label: "TriG (.trig)", exts: ["trig"] },
  { value: "jsonld", label: "JSON-LD (.jsonld)", exts: ["jsonld", "json-ld", "json"] },
  { value: "hdt", label: "HDT (.hdt) — native only", exts: ["hdt"] },
];

/** The compression suffixes the native loader streams; stripped before the format guess. */
const COMPRESSION_EXTS = new Set(["gz", "bz2", "zst", "zstd"]);

/**
 * Strip the path/query off a name and the compression suffix (`.gz` / `.bz2` / `.zst[d]`) so the
 * INNER extension drives the format guess. `data.ttl.gz` → `data.ttl`; `x.hdt.gz` → `x.hdt`.
 */
export function stripCompression(name: string): string {
  const bare = name.split(/[?#]/)[0];
  const ext = bare.split(".").pop()?.toLowerCase() ?? "";
  return COMPRESSION_EXTS.has(ext) ? bare.slice(0, bare.length - ext.length - 1) : bare;
}

/** True when the name has a compression suffix the native loader will decode. */
export function isCompressed(name: string): boolean {
  const ext = name.split(/[?#]/)[0].split(".").pop()?.toLowerCase() ?? "";
  return COMPRESSION_EXTS.has(ext);
}

/** True when the name is an HDT archive (`.hdt` or `.hdt.gz`) — native-only ingest. */
export function isHdt(name: string): boolean {
  const inner = stripCompression(name).toLowerCase();
  return inner.endsWith(".hdt");
}

/**
 * Best-effort engine format guess from a filename / URL, decompression-aware. An `.hdt[.gz]`
 * reports `hdt`; otherwise the inner extension is matched against {@link FORMAT_OPTIONS},
 * falling back to Turtle.
 */
export function guessFormat(name: string): string {
  if (isHdt(name)) return "hdt";
  const inner = stripCompression(name);
  const ext = inner.split(".").pop()?.toLowerCase() ?? "";
  const hit = FORMAT_OPTIONS.find((f) => f.exts.includes(ext));
  return hit?.value ?? "turtle";
}

const CONTENT_TYPE_FORMATS: Record<string, string> = {
  "text/turtle": "turtle",
  "application/n-triples": "ntriples",
  "application/n-quads": "nquads",
  "application/trig": "trig",
  "application/ld+json": "jsonld",
};

/** Engine format for a response `Content-Type`, or `undefined` if unrecognised / absent. */
export function formatFromContentType(contentType: string | null): string | undefined {
  if (!contentType) return undefined;
  const mime = contentType.split(";")[0].trim().toLowerCase();
  return CONTENT_TYPE_FORMATS[mime];
}

// [OPUS-4.8] sq-xvj9 (epic sq-ixc3) — the store-EXPORT serialisations offered by the DatasetViewer's
// "Export data" action. This is the OUT direction (serialise the whole live store to a downloadable
// document), distinct from {@link FORMAT_OPTIONS} above (the IN direction the Import drawer parses).
// Only the three pretty, human-readable syntaxes the engine writer emits are offered — N-Triples /
// N-Quads (flat, line-oriented) are the snapshot/merge format, not a "readable export".

/** A store-export target serialisation. */
export type ExportFormat = "turtle" | "trig" | "jsonld";

/** Display + download metadata for one {@link ExportFormat}. */
export interface ExportFormatMeta {
  value: ExportFormat;
  /** The menu label. */
  label: string;
  /** The download filename extension (no leading dot). */
  ext: string;
  /** The download MIME type. */
  mime: string;
  /**
   * Which part of the store this serialisation carries. TriG + JSON-LD emit the WHOLE dataset
   * (default graph + every named graph); Turtle has no named-graph syntax, so it emits the
   * DEFAULT GRAPH only. Surfaced in the UI so the export scope is never silently misrepresented.
   */
  scope: "whole dataset" | "default graph";
}

/** The export serialisations, in menu order (Turtle · TriG · JSON-LD). */
export const EXPORT_FORMATS: readonly ExportFormatMeta[] = [
  { value: "turtle", label: "Turtle (.ttl)", ext: "ttl", mime: "text/turtle", scope: "default graph" },
  { value: "trig", label: "TriG (.trig)", ext: "trig", mime: "application/trig", scope: "whole dataset" },
  {
    value: "jsonld",
    label: "JSON-LD (.jsonld)",
    ext: "jsonld",
    mime: "application/ld+json",
    scope: "whole dataset",
  },
];

/**
 * Value-keyed view of {@link EXPORT_FORMATS} for O(1) metadata lookup. Typed as a total
 * `Record<ExportFormat, …>`, so every `ExportFormat` is guaranteed an entry (and a valid
 * `ext`) at compile time — no per-call `.find()` and no weak string fallback.
 */
export const EXPORT_FORMAT_META: Record<ExportFormat, ExportFormatMeta> = Object.fromEntries(
  EXPORT_FORMATS.map((f) => [f.value, f]),
) as Record<ExportFormat, ExportFormatMeta>;

/** The download filename for an exported dataset in `format` (e.g. `dataset.trig`). */
export function exportFilename(format: ExportFormat): string {
  return `dataset.${EXPORT_FORMAT_META[format].ext}`;
}

/** A short display label for a URL: the last non-empty path segment, or the host. */
export function urlLabel(url: string): string {
  try {
    const u = new URL(url);
    const tail = u.pathname.split("/").filter(Boolean).pop();
    return tail || u.host;
  } catch {
    return url.length > 40 ? `${url.slice(0, 39)}…` : url;
  }
}

/** A short display label for a disk path: the final path segment (filename). */
export function fileLabel(path: string): string {
  const seg = path.split(/[\\/]/).filter(Boolean).pop();
  return seg || path;
}
