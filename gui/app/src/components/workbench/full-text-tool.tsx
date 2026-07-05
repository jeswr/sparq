"use client";

// [SONNET-4.6] sq-9nwab — the Full-text tool: BM25 search panel over the sparq-text-wasm bundle.
//
// Translation rule (research/gui-design.md §A.4/§A.5): the website's /surface/full-text PAGE
// demonstrates BM25 search over a fixture document, wrapped in marketing chrome. Here that chrome
// is CUT. The tool searches the ACTIVE workspace's LIVE store (not a fixture) using the text:
// magic predicates exposed by the W-text wasm bundle.
//
// What runs: the live store is serialised to TriG (preserving every named graph) and passed to
// TextSearch.query() along with a SPARQL query using text:matches / text:score. The bundle is
// SEPARATE from the lean sparq-wasm triplestore engine — it is OPTIONAL. A build that did not
// sync it surfaces an honest "search unavailable" state rather than crashing or silently failing.
//
// This is a keyboard-first operational panel: Enter submits the search (not only the button).

import * as React from "react";
import { Search, Loader2 } from "lucide-react";

import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { useEngine } from "@/lib/engine-context";
import { loadTextSearch } from "@/lib/text-wasm";
import { TIER_META, toolById } from "@/data/tools";
import type { ToolOverride } from "@/data/tools";

/**
 * Optional honesty-metadata override merged over the base `ToolDef` (data/tools.ts) by the
 * tool-panel registry's `resolveTool` and by the stub itself. `undefined` = base metadata
 * unchanged. Omit fields you do not override.
 */
export const FULL_TEXT_TOOL_OVERRIDE: ToolOverride | undefined = {
  built: true,
  group: "working" as const,
};

interface HitRow {
  subject: string;
  property: string;
  literal: string;
  score: string;
}

type SearchState =
  | { kind: "idle" }
  | { kind: "running" }
  | { kind: "results"; hits: HitRow[]; ms: number; storeQuads: number }
  | { kind: "error"; message: string };

interface SparqlJsonResult {
  head: { vars: string[] };
  results: {
    bindings: Record<string, { type: string; value: string; datatype?: string }>[];
  };
}

/** Escape double quotes and backslashes in a term for SPARQL string literal safety. */
function buildSearchQuery(term: string): string {
  const escaped = term.replace(/\\/g, "\\\\").replace(/"/g, '\\"');
  return `PREFIX text: <http://sparq.dev/text#>
SELECT ?s ?prop ?lit ?score WHERE {
  ?s ?prop ?lit .
  ?lit text:matches "${escaped}" ; text:score ?score .
} ORDER BY DESC(?score) LIMIT 50`;
}

/** Compact a full IRI to a short form if it's long; strip angle brackets if present. */
function shortenIri(s: string): string {
  const inner = s.startsWith("<") && s.endsWith(">") ? s.slice(1, -1) : s;
  // Show only the last path/fragment segment for readability; full IRI on hover.
  const m = inner.match(/[#/]([^#/]+)\/?$/);
  return m ? m[1] : inner;
}

function HitItem({ hit }: { hit: HitRow }) {
  return (
    <div className="border-b px-3 py-2 text-xs" data-text-hit>
      <div className="flex flex-wrap items-center gap-2">
        <span className="font-mono text-[11px] text-muted-foreground" title={hit.subject}>
          {shortenIri(hit.subject)}
        </span>
        <span className="text-[11px] text-muted-foreground" title={hit.property}>
          · {shortenIri(hit.property)}
        </span>
        <span className="ml-auto text-[11px] tabular text-muted-foreground">
          score {parseFloat(hit.score).toFixed(3)}
        </span>
      </div>
      <p className="mt-0.5 break-words">{hit.literal}</p>
    </div>
  );
}

export function FullTextTool() {
  const { status, storeSize, serializeStore } = useEngine();
  const [term, setTerm] = React.useState("");
  const [state, setState] = React.useState<SearchState>({ kind: "idle" });
  const [bundleError, setBundleError] = React.useState<string | null>(null);

  const ready = status.kind === "ready";
  const tool = toolById("full-text");
  const tier = tool ? TIER_META[tool.tier] : null;

  // Eagerly probe the bundle on mount (once). This surfaces the unavailable state BEFORE the
  // user clicks Search, so the honest [data-text-unavailable] block appears immediately for a
  // build that did not sync the text wasm, and the test can branch correctly between the
  // "bundle available" and "bundle unavailable" paths without requiring a Search click first.
  React.useEffect(() => {
    let cancelled = false;
    loadTextSearch().then(
      () => {
        // Bundle loaded OK — no state change needed; search UI stays enabled.
      },
      (err: unknown) => {
        if (!cancelled) {
          setBundleError(err instanceof Error ? err.message : String(err));
        }
      },
    );
    return () => {
      cancelled = true;
    };
  }, []);

  const onSearch = React.useCallback(async () => {
    if (!term.trim()) return;
    setState({ kind: "running" });

    // Load the text-wasm bundle (cached after the mount probe succeeds).
    let textSearch;
    try {
      textSearch = await loadTextSearch();
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setBundleError(msg);
      setState({ kind: "idle" });
      return;
    }

    const data = serializeStore();
    if (data === null) {
      setState({
        kind: "error",
        message:
          "The live store is not ready (or this wasm bundle lacks the serialise binding), so it cannot be serialised for search.",
      });
      return;
    }

    const t0 = performance.now();
    try {
      const sparql = buildSearchQuery(term);
      const jsonStr = textSearch.query(data, "trig", sparql);
      const result = JSON.parse(jsonStr) as SparqlJsonResult;
      const hits: HitRow[] = result.results.bindings.map((b) => ({
        subject: b["s"]?.value ?? "",
        property: b["prop"]?.value ?? "",
        literal: b["lit"]?.value ?? "",
        score: b["score"]?.value ?? "0",
      }));
      setState({
        kind: "results",
        hits,
        ms: performance.now() - t0,
        storeQuads: storeSize,
      });
    } catch (err) {
      setState({
        kind: "error",
        message: err instanceof Error ? err.message : String(err),
      });
    }
  }, [term, serializeStore, storeSize]);

  const onKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      e.preventDefault();
      void onSearch();
    }
  };

  const searchDisabled =
    !ready || bundleError !== null || !term.trim() || state.kind === "running";

  return (
    <div className="flex h-full flex-col">
      {/* Header bar: icon + label + tier dot + search input + button */}
      <div className="flex items-center gap-2 border-b bg-card px-3 py-1.5">
        <Search className="size-3.5 shrink-0 text-muted-foreground" aria-hidden />
        <span className="text-xs font-medium">Full-text</span>
        {tier ? (
          <span className="flex items-center gap-1.5 text-[11px] text-muted-foreground">
            <span className={cn("size-2 rounded-full", tier.dot)} aria-hidden />
            {tier.label}
          </span>
        ) : null}
        <input
          data-text-search-input
          type="text"
          value={term}
          onChange={(e) => setTerm(e.target.value)}
          onKeyDown={onKeyDown}
          placeholder="Enter a search term…"
          disabled={!ready || bundleError !== null}
          className="ml-2 min-w-0 flex-1 rounded-md border bg-background px-2 py-1 text-xs outline-none focus:ring-1 focus:ring-primary disabled:opacity-50"
          aria-label="Full-text search term"
        />
        <Button
          data-text-search-btn
          size="sm"
          onClick={() => void onSearch()}
          disabled={searchDisabled}
        >
          {state.kind === "running" ? <Loader2 className="animate-spin" /> : <Search />}
          Search
        </Button>
      </div>

      {/* Bundle unavailable: honest error with rebuild instruction. Rendered once the
          mount-time probe resolves with a failure (set via the useEffect above). */}
      {bundleError !== null ? (
        <div
          data-text-unavailable
          className="m-3 rounded-md border border-[var(--warning)]/40 bg-[var(--warning)]/5 p-3 text-xs text-muted-foreground"
        >
          The full-text search bundle could not load: {bundleError}. It is a separate wasm
          bundle — rebuild it with{" "}
          <code className="rounded bg-muted px-1 py-0.5">npm run build:text-wasm</code> in{" "}
          <code className="rounded bg-muted px-1 py-0.5">js/</code>. Until then, search is
          unavailable.
        </div>
      ) : null}

      {/* Results pane */}
      <div data-text-results className="flex min-h-0 flex-1 flex-col overflow-auto">
        {bundleError !== null ? null : state.kind === "idle" ? (
          <p className="p-3 text-sm text-muted-foreground">
            {ready
              ? "Enter a term to search the live store."
              : "Waiting for the engine to warm…"}
          </p>
        ) : state.kind === "running" ? (
          <p className="p-3 text-sm text-muted-foreground">Searching…</p>
        ) : state.kind === "error" ? (
          <pre className="overflow-auto whitespace-pre-wrap p-3 font-mono text-xs text-destructive">
            {state.message}
          </pre>
        ) : state.hits.length === 0 ? (
          <p className="p-3 text-sm text-muted-foreground">
            No results found for &ldquo;{term}&rdquo; in{" "}
            {state.storeQuads.toLocaleString()} quads ({state.ms.toFixed(1)} ms).
          </p>
        ) : (
          <>
            <div className="flex items-center gap-2 border-b bg-card px-3 py-1 text-[11px] text-muted-foreground">
              <span>
                {state.hits.length} {state.hits.length === 1 ? "result" : "results"}
              </span>
              <span className="ml-auto tabular">
                {state.ms.toFixed(1)} ms over {state.storeQuads.toLocaleString()} quads
              </span>
            </div>
            <div>
              {state.hits.map((hit, i) => (
                <HitItem key={i} hit={hit} />
              ))}
            </div>
          </>
        )}
      </div>
    </div>
  );
}
