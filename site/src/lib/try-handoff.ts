// [OPUS-4.8] sq-vw3ax.11 — the HOME → /try handoff channel.
//
// The home page's HeroQueryRunner is a lightweight in-fold runner; "Open in workbench →" hands
// the current query + sample data off to the full /try workbench so the visitor continues where
// they left off. Two mechanisms, both client-only and static-export safe (no server):
//
//   1. sessionStorage['sparq:handoff'] — the primary path. The runner WRITES the payload, then
//      navigates to /try; the workbench CONSUMES (reads + clears) it on mount. sessionStorage is
//      same-tab and cleared when the tab closes, so it never leaks a stale query across sessions.
//   2. /try#q=<base64url> — a shareable-link fallback. The payload is encoded as base64url JSON
//      in the URL hash, so a copied /try#q=… link reopens the same query/data on any tab.
//
// Nothing here touches the network: the payload never leaves the browser. This is the site's
// "runs in your tab" posture — the handoff is tab-local plumbing, not a request.

/** The sessionStorage key the home runner writes and the /try workbench consumes. */
export const HANDOFF_STORAGE_KEY = "sparq:handoff";

/** A query (+ optional dataset) handed from the home runner to the /try workbench. */
export interface TryHandoff {
  /** The SPARQL query text to preload into the editor. */
  query: string;
  /** The dataset text to load into the in-tab store (Turtle etc.). Omitted = keep the default. */
  data?: string;
  /** The engine RDF format string for `data` (e.g. "turtle"). Defaults to "turtle". */
  format?: string;
}

/** Narrow an unknown parsed value to a {@link TryHandoff} (a non-empty query string is required). */
function asHandoff(value: unknown): TryHandoff | null {
  if (typeof value !== "object" || value === null) return null;
  const v = value as Record<string, unknown>;
  if (typeof v.query !== "string" || v.query.length === 0) return null;
  return {
    query: v.query,
    data: typeof v.data === "string" ? v.data : undefined,
    format: typeof v.format === "string" ? v.format : undefined,
  };
}

/**
 * Write a handoff payload into sessionStorage. No-op (never throws) outside a browser or when
 * storage is unavailable (private-mode quota) — the caller still navigates to /try, which simply
 * opens with its own default in that case.
 */
export function writeHandoff(payload: TryHandoff): void {
  if (typeof window === "undefined") return;
  try {
    window.sessionStorage.setItem(HANDOFF_STORAGE_KEY, JSON.stringify(payload));
  } catch {
    // Storage disabled/full — the handoff is best-effort; ignore.
  }
}

/**
 * Read AND clear the sessionStorage handoff (single-use). Returns the payload or `null` when
 * there is none / it is malformed. Clearing on read is what makes it a one-shot: a later reload
 * of /try does not re-consume a query the visitor has moved on from.
 */
export function consumeHandoff(): TryHandoff | null {
  if (typeof window === "undefined") return null;
  try {
    const raw = window.sessionStorage.getItem(HANDOFF_STORAGE_KEY);
    if (raw === null) return null;
    window.sessionStorage.removeItem(HANDOFF_STORAGE_KEY);
    return asHandoff(JSON.parse(raw));
  } catch {
    return null;
  }
}

// ── base64url JSON codec for the /try#q=<…> shareable-link fallback ────────────────────────────

/** UTF-8 → base64url (no padding), the URL-hash-safe alphabet. Browser-only. */
function toBase64Url(json: string): string {
  const bytes = new TextEncoder().encode(json);
  let binary = "";
  for (const b of bytes) binary += String.fromCharCode(b);
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

/** base64url (no padding) → UTF-8. Throws on malformed input (the caller catches). */
function fromBase64Url(b64url: string): string {
  const b64 = b64url.replace(/-/g, "+").replace(/_/g, "/");
  // [OPUS-4.8] restore `=` padding stripped by toBase64Url so atob() doesn't throw on lengths that
  // are not a multiple of 4 (e.g. a 10-char base64url → 10%4=2 → needs "==" appended).
  const padded = b64 + "=".repeat((4 - (b64.length % 4)) % 4);
  const binary = atob(padded);
  const bytes = Uint8Array.from(binary, (c) => c.charCodeAt(0));
  return new TextDecoder().decode(bytes);
}

/** Encode a handoff payload as the base64url JSON used in a `/try#q=<…>` shareable link. */
export function encodeQueryHash(payload: TryHandoff): string {
  return toBase64Url(JSON.stringify(payload));
}

/**
 * Decode a `#q=<base64url>` location hash into a handoff payload, or `null` when the hash is
 * absent / not a `#q=` fragment / malformed. Accepts the raw `location.hash` (with or without the
 * leading `#`).
 */
export function decodeQueryHash(hash: string): TryHandoff | null {
  const h = hash.startsWith("#") ? hash.slice(1) : hash;
  if (!h.startsWith("q=")) return null;
  try {
    return asHandoff(JSON.parse(fromBase64Url(h.slice(2))));
  } catch {
    return null;
  }
}
