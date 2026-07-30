"use client";

// [OPUS-4.8] sq-vw3ax.11 — the HOME hero's in-browser SPARQL runner (the landing page's hero
// artifact). It replaces the heavy full REPL that used to be duplicated on the home page: a
// LIGHTWEIGHT runner that fits in-fold on the right of the split hero — a two-tab editor
// (Query | Data, both editable so the sample is inspectable), a big teal Run button bound to
// Ctrl/Cmd+Enter, and a typed results table below. The full workbench lives at /app; "Open in
// workbench →" is a HARD full-page navigation to it (sq-4hiqe: /try was removed, /app is the
// single workbench; /app is a separate overlaid Next app so a soft nav would fetch its RSC .txt).
//
// HONESTY (load-bearing). The idle state is an explicit, dimmed PREVIEW of the expected answer
// behind a "Preview — press Run to compute it live in your tab" pill — never a fake skeleton and
// never presented as a computed result. When the visitor presses Run, the REAL Rust engine
// (compiled to wasm) parses the sample Turtle and evaluates the join + SUM + ORDER BY in-tab; the
// footer's "N results · <t> ms · in-browser · 0 network requests" is measured, not illustrative.
// Nothing here is sent to a server (the engine runs entirely in the tab).
//
// WARM-UP (fixes the CTA→skeleton jank). The wasm engine pre-warms on this lazy chunk's
// hydration (prewarmSparqWhenIdle, an idle-slot fetch), and any pointerdown/focus inside the
// panel eagerly kicks loadSparq(). Run NEVER blocks on warmth — it awaits the (memoised) load, so
// a first Run before warm-up finishes simply shows a one-time "Starting engine…" substate.

import * as React from "react";
import dynamic from "next/dynamic";
import { Play, Loader2, ArrowRight, ShieldCheck } from "lucide-react";

import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { rovingTabIndex, useRovingTablist } from "@/lib/use-roving-tablist";
import { SparqlEditor } from "@/components/sparql-editor";
import { RdfEditor } from "@/components/rdf-editor";
import { ResultCell } from "@/components/repl-result-cells";
import { isGraphShaped } from "@/lib/result-graph-shape";
import {
  loadSparq,
  loadIntoStore,
  datasetSize,
  prewarmSparqWhenIdle,
  type SparqlResults,
} from "@/lib/sparq-wasm";
import {
  HERO_SAMPLE_TURTLE,
  HERO_SAMPLE_FORMAT,
  HERO_DEFAULT_QUERY,
  HERO_RESULT_VARS,
  HERO_PREVIEW_ROWS,
} from "@/data/hero-sample";
import { withBasePath } from "@/lib/base-path";
import { isNumericLiteral } from "@/lib/numeric-literal";

// [review #3601] The node-link SVG renderer AND the full node/edge derivation (deriveGraph, which
// repl-graph-view imports) load through a LITERAL dynamic import() only when the visitor actually
// switches to the Graph view — neither may sit in this (already-lazy) hero chunk for the majority
// who never open Graph (site policy: rarely-used net-new frontend code loads on the invocation
// path). The CHEAP eligibility check that decides whether to even OFFER the toggle stays
// synchronous — it is `isGraphShaped` (imported above), a cheap capped scan that exactly models
// deriveGraph's decline conditions (including the MAX_GRAPH_NODES cap) WITHOUT building the
// node/edge maps — so the Table | Graph toggle still appears the instant a result is graph-shaped,
// without eagerly running the full derivation for every result.
const ResultGraphView = dynamic(
  () => import("@/components/repl-graph-view").then((m) => m.ResultGraphView),
  {
    ssr: false,
    loading: () => (
      <div className="flex items-center justify-center gap-2 px-3 py-10 text-xs text-muted-foreground">
        <Loader2 className="size-3.5 animate-spin" aria-hidden />
        Loading graph view…
      </div>
    ),
  },
);

/** The first non-empty line of an engine error (keeps the compact strip to one line + any col). */
function firstErrorLine(err: unknown): string {
  const msg = err instanceof Error ? err.message : String(err);
  const line = msg.split("\n").find((l) => l.trim().length > 0);
  return (line ?? msg).trim();
}

type RunnerPhase = "idle" | "running" | "done" | "error";

/** A tiny typed results table: columns from `head.vars`, teal-mono IRIs + right-aligned numerics. */
function ResultsTable({
  results,
  dimmed,
}: {
  results: SparqlResults;
  dimmed?: boolean;
}) {
  const vars = React.useMemo(() => results.head?.vars ?? [], [results]);
  const rows = React.useMemo(() => results.results?.bindings ?? [], [results]);
  // A column is numeric when every bound cell in it is a numeric literal (so we right-align it).
  const numericVar = React.useMemo(() => {
    const m: Record<string, boolean> = {};
    for (const v of vars) {
      let sawBound = false;
      let allNumeric = true;
      for (const row of rows) {
        const t = row[v];
        if (!t) continue;
        sawBound = true;
        if (!isNumericLiteral(t)) allNumeric = false;
      }
      m[v] = sawBound && allNumeric;
    }
    return m;
  }, [vars, rows]);

  return (
    <table
      className={cn("w-full border-collapse text-[13px]", dimmed && "opacity-45")}
      data-hero-results-table
    >
      <thead>
        <tr className="border-b">
          {vars.map((v) => (
            <th
              key={v}
              scope="col"
              className={cn(
                "px-3 py-2 font-mono text-xs font-medium text-primary",
                numericVar[v] ? "text-right" : "text-left",
              )}
            >
              ?{v}
            </th>
          ))}
        </tr>
      </thead>
      <tbody>
        {rows.map((row, i) => (
          <tr key={i} className="border-b border-border/50 last:border-0">
            {vars.map((v) => (
              <td
                key={v}
                className={cn("px-3 py-1.5", numericVar[v] && "text-right tabular")}
              >
                <ResultCell term={row[v]} />
              </td>
            ))}
          </tr>
        ))}
      </tbody>
    </table>
  );
}

/** The PREVIEW table (idle state): the expected answer, honestly labelled — not computed.
 *
 * [SONNET-4.6] sq-ymr2e.13 — WCAG 2.1 AA fix: removed the `opacity-45` that was on the table.
 * At 45% opacity on a white background, text contrast is physically capped at ~3.3:1 (even black
 * text only reaches 3.3:1 blended with white at α=0.45), so NO token colour can clear the 4.5:1
 * AA floor with that opacity applied — axe flags it even for aria-hidden content that is visually
 * rendered. The "preview" state is communicated entirely by the pill overlay ("Preview — press Run
 * to compute it live in your tab") that sits on top of this table; the visual dimming via opacity
 * is redundant with the pill and has no semantic value on its own. With opacity removed, all token
 * colours are at full intensity and clear AA on the white card background. */
function PreviewTable() {
  return (
    <table className="w-full border-collapse text-[13px]" aria-hidden="true">
      <thead>
        <tr className="border-b">
          {HERO_RESULT_VARS.map((v, i) => (
            <th
              key={v}
              scope="col"
              className={cn(
                "px-3 py-2 font-mono text-xs font-medium text-primary",
                i === 0 ? "text-left" : "text-right",
              )}
            >
              ?{v}
            </th>
          ))}
        </tr>
      </thead>
      <tbody>
        {HERO_PREVIEW_ROWS.map((r) => (
          <tr key={r.name} className="border-b border-border/50 last:border-0">
            <td className="px-3 py-1.5">
              <span className="sq-tok-string">&quot;{r.name}&quot;</span>
            </td>
            <td className="px-3 py-1.5 text-right tabular">
              <span className="sq-tok-number">{r.linesOfRust}</span>
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

export function HeroQueryRunner() {
  const [tab, setTab] = React.useState<"query" | "data">("query");
  // [FABLE-5] sq-ymr2e.9 — APG tabs keyboard contract (arrow-key roving focus).
  const tablistKeys = useRovingTablist();
  const [query, setQuery] = React.useState(HERO_DEFAULT_QUERY);
  const [data, setData] = React.useState(HERO_SAMPLE_TURTLE);

  const [phase, setPhase] = React.useState<RunnerPhase>("idle");
  const [results, setResults] = React.useState<SparqlResults | null>(null);
  const [ms, setMs] = React.useState<number | null>(null);
  // [SONNET-4.6] sq-su1oe (#820) — the total triple count in the in-tab store after the most
  // recent run (default + named graphs via datasetSize). Shown in the proof footer alongside
  // the result count + timing so visitors can see the dataset size, not just the query output.
  const [triples, setTriples] = React.useState<number | null>(null);
  const [error, setError] = React.useState<string | null>(null);
  const [engineReady, setEngineReady] = React.useState(false);
  // [OPUS-4.8] sq-vw3ax.10 — result view: the typed Table (default) or the node-link Graph. The
  // Graph toggle only appears when the LIVE result is entity-relationship shaped (isGraphShaped is
  // true); a non-graph result silently falls back to the table, so a stale "graph" choice can
  // never render an empty picture.
  const [resultView, setResultView] = React.useState<"table" | "graph">("table");

  // WARM-UP 1: pre-warm on this lazy chunk's hydration, on the next browser-idle slot so it never
  // competes with paint. Cancelled on unmount if it has not fired yet.
  React.useEffect(() => {
    const handle = prewarmSparqWhenIdle({ onReady: () => setEngineReady(true) });
    return () => handle.cancel();
  }, []);

  // WARM-UP 2: any pointer/focus inside the panel eagerly kicks the (memoised) load — the visitor
  // is clearly about to interact, so warm now rather than wait for the idle slot.
  const eagerWarm = React.useCallback(() => {
    if (engineReady) return;
    void loadSparq().then(
      () => setEngineReady(true),
      () => {},
    );
  }, [engineReady]);

  const run = React.useCallback(async () => {
    setPhase("running");
    setError(null);
    const started = performance.now();
    try {
      // Run NEVER blocks on warmth — it awaits the memoised load, joining any in-flight prewarm.
      const Store = await loadSparq();
      setEngineReady(true);
      // Rebuild the store from the (possibly edited) Data tab each run, so editing data is live.
      const store = loadIntoStore(Store, data, HERO_SAMPLE_FORMAT);
      // [SONNET-4.6] sq-su1oe (#820) — capture the dataset size for the proof footer.
      // datasetSize counts the WHOLE dataset (default + named graphs) via a SELECT COUNT(*).
      setTriples(datasetSize(store));
      const json = store.query(query);
      const parsed = JSON.parse(json) as SparqlResults;
      setResults(parsed);
      setMs(performance.now() - started);
      setPhase("done");
    } catch (err) {
      // Keep the previous output visible (dimmed) — the error strip sits above it, not instead.
      setError(firstErrorLine(err));
      setPhase("error");
    }
  }, [data, query]);

  // Errors clear on edit (the strip is cleared and the phase reverts to whatever the last output
  // was) — editing the query/data is the natural "try again" gesture.
  const onQueryChange = React.useCallback(
    (next: string) => {
      setQuery(next);
      if (error !== null) {
        setError(null);
        setPhase(results ? "done" : "idle");
      }
    },
    [error, results],
  );
  const onDataChange = React.useCallback(
    (next: string) => {
      setData(next);
      if (error !== null) {
        setError(null);
        setPhase(results ? "done" : "idle");
      }
    },
    [error, results],
  );

  // Ctrl/Cmd+Enter runs from anywhere in the panel (the editors' textareas bubble the keydown).
  const onKeyDown = React.useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        if (phase !== "running") void run();
      }
    },
    [phase, run],
  );

  const openInWorkbench = React.useCallback(() => {
    // [OPUS-4.8] sq-4hiqe — /app is the single workbench (the in-tab /try REPL was removed). It is
    // a SEPARATE Next app overlaid at /app/, so hard-navigate the whole page rather than soft-push.
    window.location.assign(withBasePath("/app/"));
  }, []);

  const running = phase === "running";
  const rowCount = results?.results?.bindings?.length ?? 0;
  // The node-link Graph view is offered only when the settled result is genuinely graph-shaped. We
  // run only the CHEAP eligibility predicate here (isGraphShaped) — never the full node/edge
  // derivation — so the majority who never open Graph don't pay for it; deriveGraph + the SVG
  // renderer load lazily (ResultGraphView) the first time a visitor actually switches to Graph.
  const graphAvailable = React.useMemo(
    () => phase === "done" && results !== null && isGraphShaped(results),
    [phase, results],
  );
  const showGraph = resultView === "graph" && graphAvailable;

  return (
    <div
      onKeyDownCapture={onKeyDown}
      onPointerDown={eagerWarm}
      onFocusCapture={eagerWarm}
      className="overflow-hidden rounded-xl border border-primary/25 bg-card shadow-elevation-2"
      style={{ boxShadow: "var(--elevation-2), 0 0 0 1px var(--teal-glow)" }}
    >
      {/* Tabs: Query (default) | Data — both editable, so the sample is inspectable.
          [FABLE-5] sq-ymr2e.9 — APG tabs keyboard contract: roving tabindex + arrow-key
          focus movement via the shared hook (asserted by e2e/a11y-keyboard.spec.ts). */}
      <div
        role="tablist"
        aria-label="Runner editor"
        className="flex items-center gap-1 border-b bg-muted/25 px-2 py-1.5"
        {...tablistKeys}
      >
        {(["query", "data"] as const).map((t) => (
          <button
            key={t}
            id={`hero-tab-${t}`}
            type="button"
            role="tab"
            aria-selected={tab === t}
            aria-controls={`hero-panel-${t}`}
            tabIndex={rovingTabIndex(tab === t)}
            onClick={() => setTab(t)}
            className={cn(
              "rounded-md px-3 py-1 text-xs font-medium capitalize transition-colors outline-none focus-visible:ring-3 focus-visible:ring-ring/40",
              tab === t
                ? "bg-background text-foreground shadow-sm"
                : "text-muted-foreground hover:text-foreground",
            )}
          >
            {t === "query" ? "Query" : "Data"}
          </button>
        ))}
        <span className="ml-auto pr-1 font-mono text-[10px] uppercase tracking-[0.12em] text-muted-foreground">
          in-browser SPARQL
        </span>
      </div>

      {/* Editors. Both mounted; the inactive one is hidden (not unmounted) so edits persist. */}
      <div className="p-3">
        <div
          id="hero-panel-query"
          role="tabpanel"
          aria-labelledby="hero-tab-query"
          className={cn(tab === "query" ? "block" : "hidden")}
        >
          <label htmlFor="hero-query" className="sr-only">
            SPARQL query
          </label>
          <SparqlEditor
            id="hero-query"
            ariaLabel="SPARQL query"
            value={query}
            onChange={onQueryChange}
            rows={8}
          />
        </div>
        <div
          id="hero-panel-data"
          role="tabpanel"
          aria-labelledby="hero-tab-data"
          className={cn(tab === "data" ? "block" : "hidden")}
        >
          <label htmlFor="hero-data" className="sr-only">
            Sample data (Turtle)
          </label>
          <RdfEditor
            id="hero-data"
            ariaLabel="Sample data (Turtle)"
            value={data}
            onChange={onDataChange}
            rows={8}
          />
        </div>
      </div>

      {/* Bottom bar: privacy chip (left) + big teal Run button (right). */}
      <div className="flex flex-wrap items-center gap-3 border-t bg-muted/15 px-3 py-2.5">
        <span className="inline-flex items-center gap-1.5 text-xs text-muted-foreground">
          <ShieldCheck className="size-3.5 text-[var(--success)]" aria-hidden />
          runs in your browser · nothing is sent to a server
        </span>
        {/* [SONNET-4.6] sq-ymr2e.13 — WCAG 2.1 AA fix: style.backgroundColor gives axe a
            computable background-color. `bg-[var(--hero-grad)]` sets background-image (the visible
            gradient) but tailwind-merge drops the CVA default `bg-primary` background-color, so axe
            sees `background-color: transparent`, falls through to the near-white parent, and measures
            near-white text on near-white ≈ 1.1:1. The inline backgroundColor = --primary (teal,
            ~5.3:1 against the near-white primary-foreground) is covered by the gradient visually but
            is what axe reads as the button's effective background. */}
        <Button
          onClick={() => void run()}
          disabled={running}
          size="lg"
          className="ml-auto bg-[var(--hero-grad)] text-primary-foreground shadow-elevation-glow hover:opacity-95"
          style={{ backgroundColor: "var(--primary)" }}
        >
          {running ? (
            <Loader2 className="size-4 animate-spin" aria-hidden />
          ) : (
            <Play className="size-4" aria-hidden />
          )}
          Run
          <kbd className="ml-1 hidden rounded border border-primary-foreground/30 px-1 text-[10px] font-medium sm:inline-block">
            ⌘↵
          </kbd>
        </Button>
      </div>

      {/* Error strip — compact destructive token, between editor and results. */}
      {phase === "error" && error && (
        <div
          role="alert"
          data-testid="hero-error"
          className="flex items-start gap-2 border-t border-destructive/30 bg-destructive/10 px-3 py-2 text-xs text-destructive"
        >
          <span className="font-mono">{error}</span>
        </div>
      )}

      {/* View toggle: Table (default) | Graph — only when the live result is graph-shaped.
          [OPUS-4.8] sq-vw3ax.10 — the node-link view for non-aggregate SELECT results, the
          surviving home of the removed /try Graph view (the /app workbench is the other). */}
      {graphAvailable && (
        <div className="flex items-center gap-1 border-t bg-muted/15 px-3 py-1.5">
          <span className="mr-1 font-mono text-[10px] uppercase tracking-[0.12em] text-muted-foreground">
            view
          </span>
          {(["table", "graph"] as const).map((v) => (
            <button
              key={v}
              type="button"
              aria-pressed={resultView === v}
              onClick={() => setResultView(v)}
              className={cn(
                "rounded-md px-2.5 py-0.5 text-xs font-medium capitalize transition-colors outline-none focus-visible:ring-3 focus-visible:ring-ring/40",
                resultView === v
                  ? "bg-background text-foreground shadow-sm"
                  : "text-muted-foreground hover:text-foreground",
              )}
            >
              {v}
            </button>
          ))}
        </div>
      )}

      {/* Results area. */}
      <div className="relative border-t bg-background/40" data-hero-results>
        <div className="overflow-x-auto">
          {phase === "idle" ? (
            <PreviewTable />
          ) : results ? (
            showGraph ? (
              <ResultGraphView results={results} />
            ) : (
              <ResultsTable results={results} dimmed={running || phase === "error"} />
            )
          ) : (
            <PreviewTable />
          )}
        </div>

        {/* Idle overlay pill — honesty as the hook (never a skeleton, no timing footer). */}
        {phase === "idle" && (
          <div className="pointer-events-none absolute inset-0 flex items-center justify-center">
            <span className="rounded-full border border-border bg-card/90 px-3 py-1.5 text-xs font-medium text-muted-foreground shadow-sm">
              Preview — press Run to compute it live in your tab
            </span>
          </div>
        )}

        {/* Running overlay — spinner + first-run-only "Starting engine…" substate. */}
        {running && (
          <div className="pointer-events-none absolute inset-0 flex items-center justify-center bg-background/40">
            <span className="inline-flex items-center gap-2 rounded-full border border-border bg-card/90 px-3 py-1.5 text-xs font-medium text-muted-foreground shadow-sm">
              <Loader2 className="size-3.5 animate-spin" aria-hidden />
              {engineReady ? "Running…" : "Starting engine… (first run only)"}
            </span>
          </div>
        )}
      </div>

      {/* Proof footer — only in the results state (the real, measured proof line). */}
      {phase === "done" && ms !== null && (
        // [FABLE-5] sq-ymr2e.10 — data-vr-mask: the measured "N results · <t> ms" proof line is
        // real wall-clock data, so the visual-regression rig masks it (mask, don't chase).
        // [SONNET-4.6] sq-su1oe (#820) — triple count added: a live, measured figure from
        // datasetSize(store) (the whole dataset, not just the default graph). Labelled
        // "triples" not "rows" to distinguish from the SPARQL result-row count.
        <div
          data-vr-mask
          className="flex flex-wrap items-center gap-x-3 gap-y-1 border-t bg-muted/15 px-3 py-2 font-mono text-[11px] text-muted-foreground"
        >
          <span aria-live="polite">
            {triples !== null && (
              <>{triples.toLocaleString()} triple{triples === 1 ? "" : "s"}{" · "}</>
            )}
            {rowCount} result{rowCount === 1 ? "" : "s"} · {ms.toFixed(1)} ms · in-browser · 0
            network requests
          </span>
          <button
            type="button"
            onClick={openInWorkbench}
            className="ml-auto inline-flex items-center gap-1 rounded text-primary outline-none hover:underline focus-visible:ring-3 focus-visible:ring-ring/40"
          >
            Open in workbench <ArrowRight className="size-3" aria-hidden />
          </button>
        </div>
      )}
    </div>
  );
}
