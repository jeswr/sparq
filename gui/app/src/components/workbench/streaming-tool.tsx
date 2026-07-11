"use client";

// [SONNET-4.6] sq-kwb74 — the Streaming tool's RSP-QL tick view.
// [FABLE-5] sq-ixc3.16 — lit up as a full OPERATIONAL tool over the live workspace store:
//   * the window spec is CONFIGURABLE (editable continuous SPARQL, range/step — step < range
//     is sliding, step == range tumbling — max_delay, and the R2S operator: RSTREAM /
//     ISTREAM / DSTREAM), applied by re-registering the continuous query;
//   * a WORKSPACE FEED streams the live store's updates into the registered query: every
//     store-content change (import / INSERT / DELETE / restore) is diffed against the
//     previous snapshot (lib/rsp-feed.ts) and the ADDED triples are pushed as one logical
//     tick — so an RSP-QL window query runs over data imported into the workspace, not only
//     hand-typed demo events. Time is LOGICAL (one tick per update batch); the manual push
//     form remains for explicit-timestamp event injection.
// Loads the sparq-rsp-wasm bundle lazily via lib/rsp-wasm.ts (the main bundle stays
// wasm-free); degrades honestly to "unavailable" if the bundle is absent from this build.
// Every window result is real engine output — never a canned replay. STREAMING_TOOL_OVERRIDE
// flips the honesty metadata from "coming-soon stub" to "live-new-wasm / working" inside this
// file only, keeping it disjoint from parallel tool beads.

import * as React from "react";
import { Radio, Info, RefreshCw } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { useEngine } from "@/lib/engine-context";
import { loadRspModule, type WasmRspQuery, type WasmRsp } from "@/lib/rsp-wasm";
import { diffAddedTriples, snapshotKeys } from "@/lib/rsp-feed";
import { toolById, TIER_META } from "@/data/tools";
import type { ToolOverride } from "@/data/tools";

/**
 * Optional honesty-metadata override merged over the base `ToolDef` (data/tools.ts) by the
 * tool-panel registry's `resolveTool` and by the stub itself. `undefined` = base metadata
 * unchanged. Omit fields you do not override.
 */
export const STREAMING_TOOL_OVERRIDE: ToolOverride | undefined = {
  tier: "live-new-wasm",
  built: true,
  group: "working",
  // [FABLE-5] sq-ixc3.16 — the tool now runs window queries over live workspace updates.
  blurb:
    "RSP-QL window queries — tumbling/sliding, R/I/DSTREAM — over live workspace updates or manual pushes.",
};

// Default query and window parameters (tumbling 60-tick window, AVG per window).
const DEFAULT_SPARQL = "SELECT (AVG(?v) AS ?avg) WHERE { ?s <http://ex/reading> ?v }";
const DEFAULT_RANGE = 60;
const DEFAULT_STEP = 60;
const DEFAULT_MAX_DELAY = 0;
const DEFAULT_R2S = "rstream";

// [FABLE-5] sq-ixc3.16 — the per-update-batch cap on workspace-feed pushes. A huge import
// (100k+ quads) would otherwise stall the tab pushing every triple through the window engine
// in one effect. Excess triples are counted and reported honestly — never silently dropped.
const FEED_BATCH_CAP = 5_000;

/** A binding value in a SPARQL JSON Results row. */
interface BindingValue {
  type: string;
  value: string;
  datatype?: string;
  "xml:lang"?: string;
}

/** A single closed window returned by push() / flush(). */
interface ClosedWindow {
  start: number;
  end: number;
  results: {
    head: { vars: string[] };
    results: { bindings: Record<string, BindingValue>[] };
  };
}

type Status = "loading" | "ready" | "unavailable" | "error";

/** The window/query configuration a continuous query is registered with. */
interface RspConfig {
  sparql: string;
  range: number;
  step: number;
  maxDelay: number;
  r2s: string;
}

const DEFAULT_CONFIG: RspConfig = {
  sparql: DEFAULT_SPARQL,
  range: DEFAULT_RANGE,
  step: DEFAULT_STEP,
  maxDelay: DEFAULT_MAX_DELAY,
  r2s: DEFAULT_R2S,
};

/** Cumulative workspace-feed accounting shown in the feed status line (real counts). */
interface FeedInfo {
  /** The logical clock: ticks consumed so far (one per store-update batch). */
  tick: number;
  /** Triples streamed into the query so far. */
  pushed: number;
  /** Triples that failed to push (term the window engine rejected) — surfaced, not hidden. */
  skipped: number;
  /** Triples beyond {@link FEED_BATCH_CAP} in a single batch — dropped, counted honestly. */
  dropped: number;
}

const FEED_ZERO: FeedInfo = { tick: 0, pushed: 0, skipped: 0, dropped: 0 };

export function StreamingTool() {
  const { storeEpoch, snapshotStore } = useEngine();

  const [status, setStatus] = React.useState<Status>("loading");
  const [errorMsg, setErrorMsg] = React.useState<string>("");
  // A push/flush/apply rejection (e.g. a mistyped Turtle term or a bad window spec) is INLINE
  // and recoverable — it must never flip `status` away from "ready", or the whole panel
  // (incl. the form + Reset) unmounts and the tab dead-ends on a typo.
  const [actionError, setActionError] = React.useState<string>("");
  // The wasm module handle lives in a REF, not state: `Rsp` is a CLASS (a function), and
  // `setState(Rsp)` would invoke it as a functional updater — `Rsp(prev)` without `new`
  // throws and crashes the whole app. It is not render-relevant anyway.
  const rspModuleRef = React.useRef<WasmRsp | null>(null);
  const queryRef = React.useRef<WasmRspQuery | null>(null);
  const [windows, setWindows] = React.useState<ClosedWindow[]>([]);
  const [lateDropped, setLateDropped] = React.useState<number>(0);

  // [FABLE-5] sq-ixc3.16 — window/query configuration: the FORM state (editable) and the
  // APPLIED config the current continuous query was registered with (shown in the echo card).
  const [form, setForm] = React.useState({
    sparql: DEFAULT_SPARQL,
    range: String(DEFAULT_RANGE),
    step: String(DEFAULT_STEP),
    maxDelay: String(DEFAULT_MAX_DELAY),
    r2s: DEFAULT_R2S,
  });
  const [applied, setApplied] = React.useState<RspConfig>(DEFAULT_CONFIG);

  // [FABLE-5] sq-ixc3.16 — the workspace feed. The baseline key set + logical clock live in
  // refs (imperative bookkeeping the render never reads); the status line reads `feedInfo`.
  const [feedOn, setFeedOn] = React.useState(false);
  const feedKeysRef = React.useRef<Set<string>>(new Set());
  const feedTickRef = React.useRef(0);
  const feedEpochRef = React.useRef(0);
  const [feedInfo, setFeedInfo] = React.useState<FeedInfo>(FEED_ZERO);

  // Push form state (pre-filled defaults for the demo).
  const [pushS, setPushS] = React.useState("<http://ex/s1>");
  const [pushP, setPushP] = React.useState("<http://ex/reading>");
  const [pushO, setPushO] = React.useState("10");
  const [pushTs, setPushTs] = React.useState("0");

  const tool = toolById("streaming");
  const effectiveTier = STREAMING_TOOL_OVERRIDE?.tier ?? tool?.tier ?? "live-new-wasm";
  const tier = TIER_META[effectiveTier];

  // Load the wasm module on mount; create the initial query handle when ready.
  React.useEffect(() => {
    loadRspModule()
      .then((mod) => {
        rspModuleRef.current = mod;
        queryRef.current = mod.select(
          DEFAULT_SPARQL,
          DEFAULT_RANGE,
          DEFAULT_STEP,
          DEFAULT_MAX_DELAY,
          DEFAULT_R2S,
        );
        setStatus("ready");
      })
      .catch((err: unknown) => {
        const msg = err instanceof Error ? err.message : String(err);
        // Treat load/fetch errors as "unavailable" (missing bundle); surface the raw
        // message for genuine engine errors.
        if (
          msg.includes("404") ||
          msg.includes("Failed to fetch") ||
          msg.includes("Cannot find module") ||
          msg.includes("not find") ||
          msg.includes("NetworkError") ||
          msg.includes("Load failed")
        ) {
          setStatus("unavailable");
        } else {
          setStatus("error");
          setErrorMsg(msg);
        }
      });
  }, []);

  /**
   * Re-baseline the workspace feed at the CURRENT store content: only additions AFTER this
   * moment stream. Called when the feed is switched on and whenever the continuous query is
   * (re-)registered while the feed is on (a fresh query starts a fresh logical clock).
   */
  const rebaselineFeed = React.useCallback(() => {
    const snap = snapshotStore();
    feedKeysRef.current = snap === null ? new Set() : snapshotKeys(snap);
    feedTickRef.current = 0;
    feedEpochRef.current = storeEpoch;
    setFeedInfo(FEED_ZERO);
  }, [snapshotStore, storeEpoch]);

  /**
   * Register a continuous query for `cfg`, replacing the current handle and clearing the
   * accumulated tick output. Returns false (with an inline error) when the engine rejects the
   * spec (bad SPARQL / zero range/step) — the PREVIOUS query then keeps running untouched.
   */
  const registerQuery = React.useCallback(
    (cfg: RspConfig): boolean => {
      const mod = rspModuleRef.current;
      if (!mod) return false;
      let q: WasmRspQuery;
      try {
        q = mod.select(cfg.sparql, cfg.range, cfg.step, cfg.maxDelay, cfg.r2s);
      } catch (err: unknown) {
        setActionError(err instanceof Error ? err.message : String(err));
        return false;
      }
      queryRef.current = q;
      setApplied(cfg);
      setWindows([]);
      setLateDropped(0);
      setActionError("");
      if (feedOn) rebaselineFeed();
      return true;
    },
    [feedOn, rebaselineFeed],
  );

  /** Apply the config form: parse + validate the numeric fields, then re-register. */
  function handleApply() {
    const range = Number(form.range);
    const step = Number(form.step);
    const maxDelay = Number(form.maxDelay);
    for (const [label, v] of [
      ["range", range],
      ["step", step],
      ["max delay", maxDelay],
    ] as const) {
      if (!Number.isInteger(v) || v < 0) {
        setActionError(`${label} must be a non-negative whole number`);
        return;
      }
    }
    // Zero range/step is rejected by the engine too, but say it plainly here.
    if (range === 0 || step === 0) {
      setActionError("range and step must be greater than zero");
      return;
    }
    registerQuery({ sparql: form.sparql, range, step, maxDelay, r2s: form.r2s });
  }

  /** (Re-)create the query handle with the APPLIED config and clear accumulated output. */
  function handleReset() {
    registerQuery(applied);
  }

  /** Fold freshly-closed windows + the late-drop counter into the tick view. */
  const absorbClosed = React.useCallback((q: WasmRspQuery, closed: ClosedWindow[]) => {
    if (closed.length > 0) {
      setWindows((prev) => [...prev, ...closed]);
    }
    setLateDropped(q.lateDropped());
  }, []);

  function handlePush() {
    if (!queryRef.current) return;
    try {
      const ts = Number(pushTs);
      const json = queryRef.current.push(pushS, pushP, pushO, ts);
      absorbClosed(queryRef.current, JSON.parse(json) as ClosedWindow[]);
      setActionError("");
    } catch (err: unknown) {
      // Recoverable input error (bad Turtle term / timestamp): keep the panel READY and
      // surface the engine's message inline next to the form.
      setActionError(err instanceof Error ? err.message : String(err));
    }
  }

  function handleFlush() {
    if (!queryRef.current) return;
    try {
      const json = queryRef.current.flush();
      absorbClosed(queryRef.current, JSON.parse(json) as ClosedWindow[]);
      setActionError("");
    } catch (err: unknown) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  }

  /** Toggle the workspace feed; enabling captures the baseline (only NEW triples stream). */
  function handleFeedToggle() {
    // The rebaseline is a side effect — keep it OUT of the setState updater (StrictMode may
    // invoke updaters twice; the snapshot capture must run exactly once per toggle).
    if (!feedOn) rebaselineFeed();
    setFeedOn(!feedOn);
  }

  // [FABLE-5] sq-ixc3.16 — the workspace feed itself: on every store-content epoch bump while
  // the feed is on, diff the new snapshot against the previous one (lib/rsp-feed.ts) and push
  // the ADDED triples as ONE logical tick. The panel stays mounted while its tab is hidden
  // (the workbench hides, not unmounts, inactive tabs), so updates made from the Query tab or
  // the Import drawer stream through without the Streaming tab being visible.
  React.useEffect(() => {
    if (!feedOn || status !== "ready") return;
    if (storeEpoch === feedEpochRef.current) return;
    feedEpochRef.current = storeEpoch;
    const q = queryRef.current;
    if (!q) return;
    const snap = snapshotStore();
    if (snap === null) return;
    const { added, keys } = diffAddedTriples(feedKeysRef.current, snap);
    feedKeysRef.current = keys;
    if (added.length === 0) return;
    const batch = added.slice(0, FEED_BATCH_CAP);
    const droppedNow = added.length - batch.length;
    const ts = feedTickRef.current;
    feedTickRef.current = ts + 1;
    let pushedNow = 0;
    let skippedNow = 0;
    const closed: ClosedWindow[] = [];
    for (const [s, p, o] of batch) {
      try {
        closed.push(...(JSON.parse(q.push(s, p, o, ts)) as ClosedWindow[]));
        pushedNow++;
      } catch {
        // A term the window engine rejects (should not happen for engine-serialised
        // snapshots) is counted and reported — never a silent gap in the stream.
        skippedNow++;
      }
    }
    absorbClosed(q, closed);
    setFeedInfo((f) => ({
      tick: feedTickRef.current,
      pushed: f.pushed + pushedNow,
      skipped: f.skipped + skippedNow,
      dropped: f.dropped + droppedNow,
    }));
  }, [feedOn, status, storeEpoch, snapshotStore, absorbClosed]);

  const windowShape = applied.step === applied.range ? "Tumbling" : "Sliding";

  return (
    <div className="h-full overflow-auto">
      <div className="mx-auto max-w-2xl space-y-5 p-6">
        {/* Header */}
        <div className="space-y-1">
          <div className="flex items-center gap-2">
            <Radio className="size-5 text-primary" />
            <h2 className="text-lg font-semibold">Streaming</h2>
            {tier ? (
              <span className="flex items-center gap-1.5 text-xs text-muted-foreground">
                <span className={`size-2 rounded-full ${tier.dot}`} aria-hidden />
                {tier.label}
              </span>
            ) : null}
          </div>
          <p className="text-sm text-muted-foreground">
            RSP-QL windowed stream processing over the live workspace, powered by the{" "}
            <code className="rounded bg-muted px-1 py-0.5 text-[11px]">sparq-rsp-wasm</code>{" "}
            bundle. Every result is live engine output — not a replay.
          </p>
        </div>

        {/* Status + content — single element always carries data-rsp-status */}
        <div data-rsp-status={status}>
          {status === "loading" && (
            <p className="text-sm text-muted-foreground">Loading RSP engine…</p>
          )}

          {status === "unavailable" && (
            <div className="rounded-md border border-[var(--warning)]/40 bg-[var(--warning)]/5 p-3 text-xs text-muted-foreground">
              The RSP bundle is unavailable in this build. Rebuild with{" "}
              <code className="rounded bg-muted px-1 py-0.5">npm run build:rsp-wasm</code> in{" "}
              <code className="rounded bg-muted px-1 py-0.5">js/</code> to enable the live tick
              view.
            </div>
          )}

          {status === "error" && (
            <div className="rounded-md border border-destructive/40 bg-destructive/5 p-3 text-xs text-muted-foreground">
              RSP engine error: {errorMsg}
            </div>
          )}

          {status === "ready" && (
            <div className="space-y-4">
              {/* [FABLE-5] sq-ixc3.16 — window/query configuration */}
              <div className="rounded-lg border bg-background p-4 space-y-3">
                <h3 className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
                  Continuous query
                </h3>
                <div className="flex flex-col gap-1">
                  <label className="text-[11px] text-muted-foreground" htmlFor="rsp-cfg-sparql">
                    SPARQL (SELECT over the window)
                  </label>
                  <textarea
                    id="rsp-cfg-sparql"
                    data-rsp-config-sparql=""
                    rows={2}
                    className="rounded border bg-muted px-2 py-1 text-xs font-mono"
                    value={form.sparql}
                    onChange={(e) => setForm((f) => ({ ...f, sparql: e.target.value }))}
                  />
                </div>
                <div className="grid grid-cols-2 gap-2 sm:grid-cols-4">
                  <div className="flex flex-col gap-1">
                    <label className="text-[11px] text-muted-foreground">Range (ticks)</label>
                    <input
                      data-rsp-config-range=""
                      className="rounded border bg-muted px-2 py-1 text-xs font-mono"
                      value={form.range}
                      onChange={(e) => setForm((f) => ({ ...f, range: e.target.value }))}
                    />
                  </div>
                  <div className="flex flex-col gap-1">
                    <label className="text-[11px] text-muted-foreground">Step (ticks)</label>
                    <input
                      data-rsp-config-step=""
                      className="rounded border bg-muted px-2 py-1 text-xs font-mono"
                      value={form.step}
                      onChange={(e) => setForm((f) => ({ ...f, step: e.target.value }))}
                    />
                  </div>
                  <div className="flex flex-col gap-1">
                    <label className="text-[11px] text-muted-foreground">Max delay</label>
                    <input
                      data-rsp-config-delay=""
                      className="rounded border bg-muted px-2 py-1 text-xs font-mono"
                      value={form.maxDelay}
                      onChange={(e) => setForm((f) => ({ ...f, maxDelay: e.target.value }))}
                    />
                  </div>
                  <div className="flex flex-col gap-1">
                    <label className="text-[11px] text-muted-foreground">R2S</label>
                    <select
                      data-rsp-config-r2s=""
                      className="rounded border bg-muted px-2 py-1 text-xs font-mono"
                      value={form.r2s}
                      onChange={(e) => setForm((f) => ({ ...f, r2s: e.target.value }))}
                    >
                      <option value="rstream">RSTREAM</option>
                      <option value="istream">ISTREAM</option>
                      <option value="dstream">DSTREAM</option>
                    </select>
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  <button
                    data-rsp-apply=""
                    onClick={handleApply}
                    className="rounded bg-primary px-3 py-1 text-xs text-primary-foreground hover:bg-primary/90"
                  >
                    Register
                  </button>
                  <p data-rsp-applied="" className="text-[11px] text-muted-foreground">
                    {windowShape} window — range={applied.range}, step={applied.step}, max_delay=
                    {applied.maxDelay}, r2s={applied.r2s}
                  </p>
                </div>
              </div>

              {/* [FABLE-5] sq-ixc3.16 — workspace feed */}
              <div className="rounded-lg border bg-background p-4 space-y-2">
                <div className="flex items-center justify-between gap-2">
                  <h3 className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
                    Workspace feed
                  </h3>
                  <button
                    data-rsp-feed-toggle=""
                    aria-pressed={feedOn}
                    onClick={handleFeedToggle}
                    className={
                      feedOn
                        ? "rounded bg-primary px-3 py-1 text-xs text-primary-foreground hover:bg-primary/90"
                        : "rounded border px-3 py-1 text-xs hover:bg-muted"
                    }
                  >
                    {feedOn ? "Feed: on" : "Feed: off"}
                  </button>
                </div>
                <p className="text-[11px] text-muted-foreground">
                  Streams every triple ADDED to the live store (imports, INSERTs, merges) into
                  the registered query — one logical tick per update batch. Named graphs are
                  folded; only additions after the feed is enabled stream.
                </p>
                {feedOn ? (
                  <p data-rsp-feed-status="" className="text-[11px] text-muted-foreground">
                    tick {feedInfo.tick} · {feedInfo.pushed} triple
                    {feedInfo.pushed === 1 ? "" : "s"} streamed
                    {feedInfo.skipped > 0 ? ` · ${feedInfo.skipped} rejected` : ""}
                    {feedInfo.dropped > 0
                      ? ` · ${feedInfo.dropped} dropped (batch cap ${FEED_BATCH_CAP})`
                      : ""}
                  </p>
                ) : null}
              </div>

              {/* Push form */}
              <div className="rounded-lg border bg-background p-4 space-y-3">
                <h3 className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
                  Push event
                </h3>
                <div className="grid grid-cols-2 gap-2">
                  <div className="flex flex-col gap-1">
                    <label className="text-[11px] text-muted-foreground">Subject</label>
                    <input
                      data-rsp-push-s=""
                      className="rounded border bg-muted px-2 py-1 text-xs font-mono"
                      value={pushS}
                      onChange={(e) => setPushS(e.target.value)}
                    />
                  </div>
                  <div className="flex flex-col gap-1">
                    <label className="text-[11px] text-muted-foreground">Predicate</label>
                    <input
                      data-rsp-push-p=""
                      className="rounded border bg-muted px-2 py-1 text-xs font-mono"
                      value={pushP}
                      onChange={(e) => setPushP(e.target.value)}
                    />
                  </div>
                  <div className="flex flex-col gap-1">
                    <label className="text-[11px] text-muted-foreground">Object</label>
                    <input
                      data-rsp-push-o=""
                      className="rounded border bg-muted px-2 py-1 text-xs font-mono"
                      value={pushO}
                      onChange={(e) => setPushO(e.target.value)}
                    />
                  </div>
                  <div className="flex flex-col gap-1">
                    <label className="text-[11px] text-muted-foreground">Timestamp</label>
                    <input
                      data-rsp-push-ts=""
                      className="rounded border bg-muted px-2 py-1 text-xs font-mono"
                      value={pushTs}
                      onChange={(e) => setPushTs(e.target.value)}
                    />
                  </div>
                </div>
                <div className="flex gap-2 flex-wrap">
                  <button
                    data-rsp-push-button=""
                    onClick={handlePush}
                    className="rounded bg-primary px-3 py-1 text-xs text-primary-foreground hover:bg-primary/90"
                  >
                    Push
                  </button>
                  <button
                    data-rsp-flush-button=""
                    onClick={handleFlush}
                    className="rounded border px-3 py-1 text-xs hover:bg-muted"
                  >
                    Flush
                  </button>
                  <button
                    onClick={handleReset}
                    className="ml-auto flex items-center gap-1 rounded border px-3 py-1 text-xs hover:bg-muted"
                  >
                    <RefreshCw className="size-3" aria-hidden /> Reset
                  </button>
                </div>
                {actionError ? (
                  <p
                    data-rsp-push-error=""
                    className="rounded border border-destructive/40 bg-destructive/5 p-2 text-[11px] text-muted-foreground"
                  >
                    {actionError}
                  </p>
                ) : null}
                {lateDropped > 0 && (
                  <p className="text-[11px] text-muted-foreground">
                    Late-dropped arrivals: {lateDropped}
                  </p>
                )}
              </div>

              {/* Closed-window tick output */}
              <div>
                <h3 className="text-xs font-semibold uppercase tracking-wide text-muted-foreground mb-2">
                  Closed windows
                </h3>
                <div data-rsp-window-list="" className="space-y-2">
                  {windows.length === 0 ? (
                    <p className="text-xs text-muted-foreground">
                      No windows closed yet — push events to fill a window.
                    </p>
                  ) : (
                    windows.map((w, i) => (
                      <div
                        key={i}
                        data-rsp-window-item=""
                        className="rounded-lg border bg-background p-3 space-y-2"
                      >
                        <p className="text-xs font-semibold">
                          Window [{w.start}, {w.end})
                        </p>
                        {w.results.results.bindings.length === 0 ? (
                          <p className="text-xs text-muted-foreground">No results.</p>
                        ) : (
                          <table className="w-full text-xs">
                            <thead>
                              <tr>
                                {w.results.head.vars.map((v) => (
                                  <th
                                    key={v}
                                    className="text-left text-muted-foreground font-normal pb-1 pr-4"
                                  >
                                    ?{v}
                                  </th>
                                ))}
                              </tr>
                            </thead>
                            <tbody>
                              {w.results.results.bindings.map((binding, j) => (
                                <tr key={j}>
                                  {w.results.head.vars.map((v) => (
                                    <td key={v} className="font-mono pr-4">
                                      {binding[v]?.value ?? ""}
                                    </td>
                                  ))}
                                </tr>
                              ))}
                            </tbody>
                          </table>
                        )}
                      </div>
                    ))
                  )}
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Honest caveats */}
        <div className="space-y-1.5 rounded-md border bg-muted/30 p-3 text-xs text-muted-foreground">
          <div className="flex items-center gap-1.5 font-medium text-foreground">
            <Info className="size-3.5" aria-hidden /> How this works
          </div>
          <ul className="list-disc space-y-1 pl-5">
            <li>
              Real RSP-QL: every window result is computed by the live{" "}
              <code className="rounded bg-muted px-1 py-0.5">sparq-rsp-wasm</code> engine — not a
              canned replay.
            </li>
            <li>
              Time is LOGICAL: a window [t, t+range) closes when an arrival with ts ≥ end +
              max_delay lands (the workspace feed stamps one tick per update batch; manual
              pushes use their explicit ts). Flush closes all remaining open windows.
            </li>
            <li>
              R2S: RSTREAM emits the full result per window; ISTREAM the rows added vs the
              previous window; DSTREAM the rows dropped. step &lt; range slides, step = range
              tumbles.
            </li>
            <li>
              The workspace feed is append-only: a DELETE cannot retract an already-streamed
              arrival (stream semantics), and pre-feed store content does not replay.
            </li>
          </ul>
          <div className="pt-1">
            <Badge variant="outline">sq-kwb74</Badge> <Badge variant="outline">sq-ixc3.16</Badge>
          </div>
        </div>
      </div>
    </div>
  );
}
