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
