"use client";

// [SONNET-4.6] sq-kwb74 — the Streaming tool's RSP-QL tick view.
// Loads the sparq-rsp-wasm bundle lazily via lib/rsp-wasm.ts; degrades honestly to
// "unavailable" if the bundle is absent from this build. Every window result is real
// engine output — never a canned replay. STREAMING_TOOL_OVERRIDE flips the honesty
// metadata from "coming-soon stub" to "live-new-wasm / working" inside this file only,
// keeping it disjoint from parallel tool beads.

import * as React from "react";
import { Radio, Info, RefreshCw } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { loadRspModule, type WasmRspQuery, type WasmRsp } from "@/lib/rsp-wasm";
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
};

// Default query and window parameters (tumbling 60-tick window, AVG per window).
const DEFAULT_SPARQL = "SELECT (AVG(?v) AS ?avg) WHERE { ?s <http://ex/reading> ?v }";
const DEFAULT_RANGE = 60;
const DEFAULT_STEP = 60;
const DEFAULT_MAX_DELAY = 0;
const DEFAULT_R2S = "rstream";

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

export function StreamingTool() {
  const [status, setStatus] = React.useState<Status>("loading");
  const [errorMsg, setErrorMsg] = React.useState<string>("");
  // A push/flush rejection (e.g. a mistyped Turtle term) is INLINE and recoverable — it must
  // never flip `status` away from "ready", or the whole panel (incl. the form + Reset)
  // unmounts and the tab dead-ends on a typo.
  const [actionError, setActionError] = React.useState<string>("");
  // The wasm module handle lives in a REF, not state: `Rsp` is a CLASS (a function), and
  // `setState(Rsp)` would invoke it as a functional updater — `Rsp(prev)` without `new`
  // throws and crashes the whole app. It is not render-relevant anyway.
  const rspModuleRef = React.useRef<WasmRsp | null>(null);
  const queryRef = React.useRef<WasmRspQuery | null>(null);
  const [windows, setWindows] = React.useState<ClosedWindow[]>([]);
  const [lateDropped, setLateDropped] = React.useState<number>(0);

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

  /** (Re-)create the query handle and clear accumulated window output. */
  function handleReset() {
    if (!rspModuleRef.current) return;
    queryRef.current = rspModuleRef.current.select(
      DEFAULT_SPARQL,
      DEFAULT_RANGE,
      DEFAULT_STEP,
      DEFAULT_MAX_DELAY,
      DEFAULT_R2S,
    );
    setWindows([]);
    setLateDropped(0);
    setActionError("");
  }

  function handlePush() {
    if (!queryRef.current) return;
    try {
      const ts = Number(pushTs);
      const json = queryRef.current.push(pushS, pushP, pushO, ts);
      const closed = JSON.parse(json) as ClosedWindow[];
      if (closed.length > 0) {
        setWindows((prev) => [...prev, ...closed]);
      }
      setLateDropped(queryRef.current.lateDropped());
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
      const closed = JSON.parse(json) as ClosedWindow[];
      if (closed.length > 0) {
        setWindows((prev) => [...prev, ...closed]);
      }
      setLateDropped(queryRef.current.lateDropped());
      setActionError("");
    } catch (err: unknown) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  }

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
            RSP-QL windowed stream processing with tick view, powered by the{" "}
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
              {/* Query info card */}
              <div className="rounded-lg border bg-background p-3 space-y-1">
                <p className="text-[11px] uppercase tracking-wide text-muted-foreground">
                  Registered query
                </p>
                <code className="block text-xs font-mono break-all">{DEFAULT_SPARQL}</code>
                <p className="text-[11px] text-muted-foreground">
                  Tumbling window — range={DEFAULT_RANGE}, step={DEFAULT_STEP}, max_delay=
                  {DEFAULT_MAX_DELAY}, r2s={DEFAULT_R2S}
                </p>
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
              Tumbling window (range = step = 60): the [0, 60) window closes when a push with ts
              ≥ 60 + max_delay arrives. Flush closes all remaining open windows.
            </li>
            <li>
              RSTREAM: the full result set per window. ISTREAM / DSTREAM (incremental diff) are
              supported by the engine but not yet wired in this UI panel.
            </li>
          </ul>
          <div className="pt-1">
            <Badge variant="outline">sq-kwb74</Badge>
          </div>
        </div>
      </div>
    </div>
  );
}
