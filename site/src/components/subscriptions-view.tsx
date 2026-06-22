"use client";

// [OPUS-4.8] sq-9ij6 — GUI Phase 2 item 7: the LIVE subscriptions view.
//
// In endpoint mode this view subscribes a SELECT to the connected sparq-server's
// `/subscriptions/sse` Server-Sent-Events stream and renders the streamed result DELTAS
// live as the dataset mutates — the standout demo a static Pages site fundamentally cannot
// do on its own (it needs a real, mutating server to push to it).
//
// It REUSES the sq-2mke endpoint-mode connection + bearer posture, never reinventing it:
//   * the SAME `EndpointConfig` (URL + optional bearer token) the Connect panel owns;
//   * the SAME honest `connectionSafetyWarnings` classifier — surfaced here too, so a live
//     stream over plaintext to a non-loopback host, or a token in the clear, is flagged
//     exactly as for a query (and a hard block, e.g. mixed-content, disables Subscribe);
//   * the bearer token is sent ONLY in the `Authorization: Bearer` header by the shared
//     `@sparq/client` `openSubscription` — the same channel the server's SSE read gate
//     (`--auth-token-read`) validates. This view never logs the token and never bypasses a
//     server gate.
//
// All wire-protocol + safety + diff logic lives in the framework-agnostic `@sparq/client`
// subscriptions module; this component is just the React host that draws the live table,
// the diff log, and the honest connect / streaming / disconnected / error lifecycle.

import * as React from "react";
import { Radio, WifiOff, Plus, Minus, CircleStop, Loader2, Activity } from "lucide-react";

import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { SparqlEditor } from "@/components/sparql-editor";
import {
  type EndpointConfig,
  type LiveResultSet,
  type SubscriptionHandle,
  applyNotification,
  connectionSafetyWarnings,
  emptyLiveResultSet,
  extractTable,
  hasBlockingWarning,
  liveResults,
  openSubscription,
} from "@sparq/client";

const DEFAULT_SUBSCRIPTION_QUERY =
  "SELECT ?s ?age WHERE { ?s <http://ex/age> ?age } ORDER BY ?age";

/** The live stream lifecycle, surfaced honestly in the UI. */
type StreamStatus =
  | { kind: "idle" }
  | { kind: "connecting" }
  | { kind: "live" }
  | { kind: "closed"; error?: string };

/** One entry in the diff log — the human-readable delta of a single notification. */
interface DiffLogEntry {
  /** The per-subscription SSE sequence (0 = initial snapshot, then 1, 2, …). */
  sequence: number;
  added: number;
  removed: number;
  /** Local wall-clock time the delta was applied (display only, not a benchmark). */
  at: string;
}

export interface SubscriptionsViewProps {
  /** The endpoint connection (URL + optional bearer token), shared with the REPL. */
  config: EndpointConfig;
  /** Whether endpoint mode is active — subscriptions only run against a real server. */
  active: boolean;
}

/**
 * [OPUS-4.8] sq-9ij6 — the live subscriptions view. Renders a SELECT editor, a
 * Subscribe/Stop control, the honest connection-safety findings (reused from sq-2mke), the
 * live accumulated result table, and a diff log of each streamed delta.
 *
 * The stream is held in a ref so a React re-render never re-opens it; the `useEffect`
 * cleanup aborts an in-flight stream on unmount or when endpoint mode is switched off, which
 * drops the server-side subscription (its slot is released on disconnect — no leak).
 */
export function SubscriptionsView({ config, active }: SubscriptionsViewProps) {
  const [query, setQuery] = React.useState(DEFAULT_SUBSCRIPTION_QUERY);
  const [status, setStatus] = React.useState<StreamStatus>({ kind: "idle" });
  const [resultSet, setResultSet] = React.useState<LiveResultSet>(emptyLiveResultSet);
  const [log, setLog] = React.useState<DiffLogEntry[]>([]);
  const handleRef = React.useRef<SubscriptionHandle | null>(null);
  // The current live set is mirrored in a ref so each streamed notification diffs against the
  // latest accumulated set deterministically — computing the diff ONCE per event at the top
  // level, never inside a state-updater (which React may re-invoke).
  const setRef = React.useRef<LiveResultSet>(emptyLiveResultSet());

  // Reuse the SAME honest classifier the Connect panel uses; a hard block (invalid URL /
  // mixed content) disables Subscribe exactly as it disables turning endpoint mode on.
  const warnings = React.useMemo(() => connectionSafetyWarnings(config), [config]);
  const blocked = hasBlockingWarning(warnings);

  const streaming = status.kind === "connecting" || status.kind === "live";

  const stop = React.useCallback(() => {
    handleRef.current?.close();
    handleRef.current = null;
  }, []);

  const subscribe = React.useCallback(() => {
    // Tear down any prior stream first (Subscribe is also "re-subscribe").
    handleRef.current?.close();
    setRef.current = emptyLiveResultSet();
    setResultSet(setRef.current);
    setLog([]);
    setStatus({ kind: "connecting" });

    handleRef.current = openSubscription(
      config,
      query,
      {
        onOpen: () => setStatus({ kind: "live" }),
        onEvent: (event) => {
          if (event.kind === "notification") {
            const note = event.notification;
            // Diff against the latest accumulated set (held in a ref) ONCE, then commit both
            // the new set and — when the delta was real — a log entry, at the top level.
            const { next, added, removed } = applyNotification(setRef.current, note);
            setRef.current = next;
            setResultSet(next);
            // The sequence-0 snapshot always counts; a later no-op diff the server suppresses.
            if (added > 0 || removed > 0 || note.sequence === 0) {
              setLog((entries) => [
                {
                  sequence: note.sequence,
                  added,
                  removed,
                  at: new Date().toLocaleTimeString(),
                },
                ...entries,
              ]);
            }
          } else if (event.kind === "error") {
            // A terminating server error (re-evaluation failed). The server ends the stream
            // right after, so onClose fires too; surface the message immediately.
            setStatus({ kind: "closed", error: event.error.message });
          }
          // `subscribed` ack + `unknown` frames need no view change.
        },
        onClose: (error) => {
          handleRef.current = null;
          // Preserve a server-error message already set by an `error` event.
          setStatus((prev) =>
            prev.kind === "closed" && prev.error
              ? prev
              : { kind: "closed", error },
          );
        },
      },
      { alias: "try-live" },
    );
  }, [config, query]);

  // Abort an in-flight stream on unmount, or when endpoint mode is switched off (the server
  // drops the subscription and releases its slot when the fetch is aborted).
  React.useEffect(() => {
    if (!active) {
      handleRef.current?.close();
      handleRef.current = null;
      setStatus({ kind: "idle" });
    }
    return () => {
      handleRef.current?.close();
      handleRef.current = null;
    };
  }, [active]);

  if (!active) {
    return (
      <div className="space-y-2 rounded-lg border bg-muted/30 p-3">
        <div className="flex items-center gap-2 text-xs font-medium text-muted-foreground">
          <Radio className="size-3.5" />
          Live subscriptions
          <span className="font-normal text-muted-foreground/80">
            — stream result deltas from the server as the dataset mutates
          </span>
        </div>
        <p className="text-[11.5px] leading-relaxed text-muted-foreground">
          Switch on <span className="font-medium">Endpoint mode</span> above and connect to a
          running sparq-server to subscribe a SELECT to its{" "}
          <code className="font-mono">/subscriptions/sse</code> stream. The view then renders
          the live added/removed result deltas each time a SPARQL UPDATE commits — something a
          static page can only do against a real, mutating server.
        </p>
      </div>
    );
  }

  const table = extractTable(liveResults(resultSet));

  return (
    <div className="space-y-3 rounded-lg border bg-muted/30 p-3">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex items-center gap-2 text-xs font-medium text-muted-foreground">
          <Radio
            className={cn("size-3.5", status.kind === "live" && "text-primary")}
          />
          Live subscriptions
          <span className="font-normal text-muted-foreground/80">
            — SSE stream of result deltas from <code className="font-mono">/subscriptions/sse</code>
          </span>
        </div>
        <StreamStatusBadge status={status} />
      </div>

      <div className="space-y-1.5">
        <label htmlFor="subscription-query" className="text-xs font-medium">
          Subscribed SELECT
        </label>
        <SparqlEditor
          id="subscription-query"
          ariaLabel="Subscription SELECT query"
          value={query}
          onChange={setQuery}
          rows={4}
        />
        <p className="text-[11px] leading-relaxed text-muted-foreground">
          Only a SELECT can be subscribed — the server diffs over solution bindings (a
          CONSTRUCT / ASK / UPDATE is refused with a 400). To watch it fire, run a SPARQL
          UPDATE against the same endpoint (the Run button above, in endpoint mode) and the
          stream pushes the added/removed rows.
        </p>
      </div>

      <div className="flex flex-wrap items-center gap-3">
        {streaming ? (
          <Button variant="outline" size="sm" onClick={stop} title="Close the SSE stream (unsubscribe)">
            <CircleStop className="size-3.5" />
            Stop
          </Button>
        ) : (
          <Button
            size="sm"
            onClick={subscribe}
            disabled={blocked}
            title={
              blocked
                ? "Fix the endpoint URL / transport warning before subscribing"
                : "Open an SSE subscription to the endpoint and stream result deltas"
            }
          >
            <Radio className="size-3.5" />
            Subscribe
          </Button>
        )}
        <StreamStatusLine status={status} rows={table.rows.length} />
      </div>

      <SafetyWarningList warnings={warnings} />

      <LiveTable table={table} status={status} />

      <DiffLog log={log} />
    </div>
  );
}

// [OPUS-4.8] sq-9ij6 — the live-stream status pill, reusing the Badge tokens.
function StreamStatusBadge({ status }: { status: StreamStatus }) {
  if (status.kind === "live") {
    return (
      <Badge variant="success" aria-live="polite">
        <Radio className="size-3" /> Streaming
      </Badge>
    );
  }
  if (status.kind === "connecting") {
    return (
      <Badge variant="muted" aria-live="polite">
        <Loader2 className="size-3 animate-spin" /> Connecting…
      </Badge>
    );
  }
  if (status.kind === "closed" && status.error) {
    return (
      <Badge variant="warning" aria-live="polite">
        <WifiOff className="size-3" /> Stream error
      </Badge>
    );
  }
  if (status.kind === "closed") {
    return (
      <Badge variant="muted" aria-live="polite">
        <WifiOff className="size-3" /> Disconnected
      </Badge>
    );
  }
  return (
    <Badge variant="muted" aria-live="polite">
      <WifiOff className="size-3" /> Idle
    </Badge>
  );
}

function StreamStatusLine({ status, rows }: { status: StreamStatus; rows: number }) {
  return (
    <p aria-live="polite" className="text-xs text-muted-foreground">
      {status.kind === "connecting" && "Opening the SSE stream…"}
      {status.kind === "live" && `Live · ${rows} ${rows === 1 ? "row" : "rows"} in the result set`}
      {status.kind === "closed" && !status.error && "Stream closed."}
      {status.kind === "closed" && status.error && (
        <span className="text-destructive">{status.error}</span>
      )}
    </p>
  );
}

// [OPUS-4.8] sq-9ij6 — render the SAME classified connection-safety findings the Connect
// panel uses. The subscription stream rides the same transport as a query, so the same
// posture applies (plaintext to a non-loopback host, token-in-the-clear, CORS, mixed
// content); we surface them here too so the user is not led to believe a live stream is any
// more private than a query.
function SafetyWarningList({
  warnings,
}: {
  warnings: ReturnType<typeof connectionSafetyWarnings>;
}) {
  // Only the transport-relevant findings matter for a stream; the SERVICE-allowlist info
  // note is about federation inside the SELECT, so it is still relevant and kept.
  if (warnings.length === 0) return null;
  return (
    <ul className="space-y-1.5">
      {warnings.map((w) => (
        <li
          key={w.code}
          data-safety-code={w.code}
          data-safety-level={w.level}
          className={cn(
            "flex items-start gap-2 rounded-md border p-2 text-[11.5px] leading-relaxed",
            w.level === "error" && "border-destructive/30 bg-destructive/5 text-destructive",
            w.level === "warning" &&
              "border-[color-mix(in_oklch,var(--warning)_35%,transparent)] bg-[color-mix(in_oklch,var(--warning)_10%,transparent)] text-[color-mix(in_oklch,var(--warning)_80%,var(--foreground))]",
            w.level === "info" && "border-border bg-muted/40 text-muted-foreground",
          )}
        >
          <span>{w.message}</span>
        </li>
      ))}
    </ul>
  );
}

// [OPUS-4.8] sq-9ij6 — the live accumulated result set, rendered through the SAME
// `extractTable` shaping the one-shot SELECT results panel uses, so a live row and a queried
// row render identically.
function LiveTable({
  table,
  status,
}: {
  table: ReturnType<typeof extractTable>;
  status: StreamStatus;
}) {
  if (table.vars.length === 0) {
    return (
      <p className="rounded-lg border bg-muted/30 p-3 text-sm text-muted-foreground">
        {status.kind === "live" || status.kind === "connecting"
          ? "Waiting for the first snapshot…"
          : "No live result yet — Subscribe to stream the current result set, then watch it update."}
      </p>
    );
  }
  return (
    <div className="max-h-80 overflow-auto rounded-lg border" data-result-kind="live-select">
      <table className="w-full text-left text-sm">
        <thead className="sticky top-0 bg-muted/80 backdrop-blur">
          <tr>
            {table.vars.map((v) => (
              <th key={v} className="px-3 py-2 font-medium">
                ?{v}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {table.rows.length === 0 ? (
            <tr>
              <td
                colSpan={table.vars.length}
                className="px-3 py-2 text-muted-foreground"
              >
                The result set is currently empty.
              </td>
            </tr>
          ) : (
            table.rows.map((row, i) => (
              <tr key={i} className="border-t">
                {row.map((cell, j) => (
                  <td key={table.vars[j]} className="px-3 py-1.5 font-mono text-[12.5px]">
                    {cell}
                  </td>
                ))}
              </tr>
            ))
          )}
        </tbody>
      </table>
    </div>
  );
}

// [OPUS-4.8] sq-9ij6 — the diff log: one line per streamed delta, newest first. Sequence 0 is
// the initial snapshot (full result as added); each later entry is the net added/removed of
// one re-evaluation the server pushed when a commit changed the result.
function DiffLog({ log }: { log: DiffLogEntry[] }) {
  if (log.length === 0) return null;
  return (
    <div className="space-y-1.5">
      <div className="flex items-center gap-1.5 text-xs font-medium text-muted-foreground">
        <Activity className="size-3.5" /> Delta log
      </div>
      <ul className="max-h-48 space-y-1 overflow-auto" data-testid="subscription-diff-log">
        {log.map((entry) => (
          <li
            key={entry.sequence}
            className="flex items-center gap-3 rounded-md border bg-background/60 px-2.5 py-1.5 text-[11.5px]"
          >
            <span className="tabular text-muted-foreground">
              {entry.sequence === 0 ? "snapshot" : `seq ${entry.sequence}`}
            </span>
            <span className="inline-flex items-center gap-1 text-[var(--success)]">
              <Plus className="size-3" />
              {entry.added}
            </span>
            <span className="inline-flex items-center gap-1 text-muted-foreground">
              <Minus className="size-3" />
              {entry.removed}
            </span>
            <span className="ml-auto tabular text-muted-foreground/70">{entry.at}</span>
          </li>
        ))}
      </ul>
    </div>
  );
}
