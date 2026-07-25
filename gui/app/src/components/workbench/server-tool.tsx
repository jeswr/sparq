"use client";

// [SONNET-4.6] sq-iemfq — the Server tool: SPARQL 1.1 Protocol endpoint client.
//
// Fills the bead-sq-iemfq panel: a Connect form with honest safety-warning UX (from the
// connection-safety classifier in @sparq/client/endpoint.ts), a live health badge from
// fetchServerHealth, and a SPARQL query panel that speaks the SPARQL 1.1 Protocol over
// `runEndpointQuery`. Follows the GUI workbench translation rule (research/gui-design.md
// §A.4/§A.5): marketing chrome CUT, live-result rendering only, honest error messaging.
//
// INVARIANTS enforced here:
//   - Bearer token is NEVER persisted (React useState only, cleared on disconnect).
//   - Fail-closed: an unreachable endpoint shows an error verbatim; no fabricated result.
//   - Connect is disabled if hasBlockingWarning(warnings) is true.

import * as React from "react";
import { Server, Loader2, RefreshCw, Unplug, Play } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";
import {
  connectionSafetyWarnings,
  hasBlockingWarning,
  runEndpointQuery,
  fetchServerHealth,
  extractTable,
  EndpointError,
  type EndpointConfig,
  type SafetyWarning,
  type EndpointResult,
  type ServerHealth,
} from "@sparq/client";

// [FABLE-5] sq-qgkwy.2 — the override lives in the sibling `.meta.ts` (eagerly bundled for the
// rail/tab honesty read path) so THIS panel module can stay behind a lazy dynamic import().
// Re-exported here so the panel file remains the tool's single import surface.
export { SERVER_TOOL_OVERRIDE } from "@/components/workbench/server-tool.meta";

// ---------------------------------------------------------------------------
// State shapes.
// ---------------------------------------------------------------------------

type HealthState =
  | { kind: "idle" }
  | { kind: "loading" }
  | { kind: "loaded"; health: ServerHealth }
  | { kind: "error"; message: string };

type QueryState =
  | { kind: "idle" }
  | { kind: "running" }
  | { kind: "result"; result: EndpointResult }
  | { kind: "error"; message: string };

const DEFAULT_QUERY = "SELECT * WHERE { ?s ?p ?o } LIMIT 10";

// ---------------------------------------------------------------------------
// Safety warning row — colour-coded by level.
// ---------------------------------------------------------------------------

function WarningRow({ w }: { w: SafetyWarning }) {
  const colorClass =
    w.level === "error"
      ? "text-destructive"
      : w.level === "warning"
        ? "text-[var(--warning)]"
        : "text-muted-foreground";
  const icon = w.level === "error" ? "✕" : w.level === "warning" ? "⚠" : "ℹ";
  return (
    <li className={cn("flex gap-1.5 text-[11px]", colorClass)}>
      <span aria-hidden className="shrink-0">
        {icon}
      </span>
      <span>{w.message}</span>
    </li>
  );
}

// ---------------------------------------------------------------------------
// Connect panel — shown when disconnected.
// ---------------------------------------------------------------------------

function ConnectPanel({
  url,
  setUrl,
  token,
  setToken,
  warnings,
  blocked,
  onConnect,
}: {
  url: string;
  setUrl: (v: string) => void;
  token: string;
  setToken: (v: string) => void;
  warnings: SafetyWarning[];
  blocked: boolean;
  onConnect: () => void;
}) {
  return (
    <div className="flex h-full flex-col items-center justify-center p-8">
      <div className="w-full max-w-md space-y-4">
        <div className="flex items-center gap-2">
          <Server className="size-5 text-muted-foreground" />
          <h2 className="text-base font-semibold">Connect to a SPARQL endpoint</h2>
        </div>

        {/* Endpoint URL */}
        <div className="space-y-1.5">
          <label
            htmlFor="server-url-input"
            className="block text-xs font-medium text-muted-foreground"
          >
            Endpoint URL
          </label>
          <input
            id="server-url-input"
            data-server-url=""
            type="url"
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            placeholder="http://127.0.0.1:3030/sparql"
            className="w-full rounded-md border bg-background px-3 py-1.5 font-mono text-sm outline-none focus:ring-2 focus:ring-ring"
            spellCheck={false}
          />
        </div>

        {/* Bearer token — never persisted */}
        <div className="space-y-1.5">
          <label
            htmlFor="server-token-input"
            className="block text-xs font-medium text-muted-foreground"
          >
            Bearer token{" "}
            <span className="font-normal text-muted-foreground/70">(optional — not persisted)</span>
          </label>
          <input
            id="server-token-input"
            data-server-token=""
            type="password"
            value={token}
            onChange={(e) => setToken(e.target.value)}
            placeholder="Enter bearer token…"
            className="w-full rounded-md border bg-background px-3 py-1.5 font-mono text-sm outline-none focus:ring-2 focus:ring-ring"
            autoComplete="off"
          />
        </div>

        {/* Safety warnings (pure — connectionSafetyWarnings is stateless + sync) */}
        {warnings.length > 0 && (
          <ul className="space-y-1 rounded-md border bg-card p-2.5">
            {warnings.map((w) => (
              <WarningRow key={w.code} w={w} />
            ))}
          </ul>
        )}

        <Button
          data-server-connect=""
          onClick={onConnect}
          disabled={blocked}
          className="w-full"
        >
          Connect
        </Button>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Health panel — shown after connecting.
// ---------------------------------------------------------------------------

function HealthPanel({
  health,
  onRefresh,
}: {
  health: HealthState;
  onRefresh: () => void;
}) {
  let statusNode: React.ReactNode;
  if (health.kind === "idle") {
    statusNode = <span className="text-muted-foreground">—</span>;
  } else if (health.kind === "loading") {
    statusNode = <Loader2 className="size-3.5 animate-spin text-muted-foreground" />;
  } else if (health.kind === "error") {
    statusNode = (
      <span className="text-destructive" data-server-health-status="">
        Error: {health.message}
      </span>
    );
  } else {
    const isLive = health.health.health.status === "ok";
    statusNode = isLive ? (
      <Badge variant="success" data-server-health-status="">
        Live
      </Badge>
    ) : (
      <Badge variant="muted" data-server-health-status="">
        Unreachable
      </Badge>
    );
  }

  return (
    <div className="flex items-center gap-2 border-b bg-card px-3 py-1.5 text-xs">
      <span className="font-medium text-muted-foreground">Health</span>
      <span className="flex items-center gap-1.5">{statusNode}</span>
      <Button
        size="icon-sm"
        variant="ghost"
        onClick={onRefresh}
        title="Refresh server health"
        data-server-health-refresh=""
        disabled={health.kind === "loading"}
        className="ml-auto"
      >
        <RefreshCw className={cn("size-3.5", health.kind === "loading" && "animate-spin")} />
      </Button>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Query panel — shown after connecting.
// ---------------------------------------------------------------------------

function QueryPanel({
  sparql,
  setSparql,
  queryState,
  onRun,
}: {
  sparql: string;
  setSparql: (v: string) => void;
  queryState: QueryState;
  onRun: () => void;
}) {
  return (
    <div className="flex min-h-0 flex-1 flex-col">
      {/* Editor row */}
      <div className="flex min-h-0 flex-[2] flex-col border-b">
        <div className="flex items-center gap-2 border-b bg-card px-3 py-1.5">
          <span className="text-xs font-medium text-muted-foreground">SPARQL query</span>
          <Button
            size="sm"
            onClick={onRun}
            disabled={queryState.kind === "running"}
            data-server-run-query=""
            className="ml-auto"
          >
            {queryState.kind === "running" ? (
              <Loader2 className="animate-spin" />
            ) : (
              <Play />
            )}
            Run query
          </Button>
        </div>
        <textarea
          data-server-sparql-editor=""
          value={sparql}
          onChange={(e) => setSparql(e.target.value)}
          spellCheck={false}
          className="min-h-0 flex-1 resize-none bg-background p-3 font-mono text-sm outline-none"
          aria-label="SPARQL query editor"
        />
      </div>

      {/* Results pane */}
      <div className="flex min-h-0 flex-1 flex-col overflow-auto">
        <QueryResult state={queryState} />
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Result rendering — fail-closed, no fabricated values.
// ---------------------------------------------------------------------------

function QueryResult({ state }: { state: QueryState }) {
  if (state.kind === "idle") {
    return (
      <p className="p-3 text-sm text-muted-foreground">Run a query to see results.</p>
    );
  }

  if (state.kind === "running") {
    return <p className="p-3 text-sm text-muted-foreground">Running…</p>;
  }

  if (state.kind === "error") {
    return (
      <pre
        data-server-result-error=""
        className="overflow-auto whitespace-pre-wrap p-3 font-mono text-xs text-destructive"
      >
        {state.message}
      </pre>
    );
  }

  const { result } = state;

  if (result.kind === "update") {
    return (
      <p data-server-result-update="" className="p-3 text-sm text-[var(--success)]">
        Update applied.
      </p>
    );
  }

  if (result.kind === "graph") {
    return (
      <pre
        data-server-result-graph=""
        className="overflow-auto whitespace-pre-wrap p-3 font-mono text-xs"
      >
        {result.ntriples || "(empty graph)"}
      </pre>
    );
  }

  if (result.kind === "boolean") {
    return (
      <p
        data-server-result-ask=""
        className={cn(
          "p-3 text-sm font-medium",
          result.value ? "text-[var(--success)]" : "text-muted-foreground",
        )}
      >
        {result.value ? "true" : "false"}
      </p>
    );
  }

  // SELECT — bindings table.
  const table = extractTable(result.results);

  if (table.vars.length === 0) {
    return <p className="p-3 text-sm text-muted-foreground">(empty result set)</p>;
  }

  return (
    <div className="overflow-auto">
      <table data-server-result-table="" className="w-full text-left text-xs">
        <thead className="border-b bg-card">
          <tr>
            {table.vars.map((v) => (
              <th key={v} className="px-3 py-1.5 font-medium text-muted-foreground">
                ?{v}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {table.rows.map((row, ri) => (
            <tr key={ri} className="border-b hover:bg-sidebar-accent/20">
              {row.map((cell, ci) => (
                <td key={ci} className="px-3 py-1 font-mono">
                  {cell}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Main panel — orchestrates connected vs disconnected state.
// ---------------------------------------------------------------------------

export function ServerTool() {
  const [url, setUrl] = React.useState("");
  // Bearer token — NEVER persisted to any storage; lives in React state only.
  const [token, setToken] = React.useState("");
  const [connected, setConnected] = React.useState(false);
  const [config, setConfig] = React.useState<EndpointConfig | null>(null);
  const [health, setHealth] = React.useState<HealthState>({ kind: "idle" });
  const [sparql, setSparql] = React.useState(DEFAULT_QUERY);
  const [queryState, setQueryState] = React.useState<QueryState>({ kind: "idle" });

  const warnings = connectionSafetyWarnings({ url, token });
  const blocked = hasBlockingWarning(warnings);

  const startHealthFetch = React.useCallback((cfg: EndpointConfig) => {
    setHealth({ kind: "loading" });
    void fetchServerHealth(cfg).then(
      (h) => setHealth({ kind: "loaded", health: h }),
      (err) =>
        setHealth({
          kind: "error",
          message: err instanceof Error ? err.message : String(err),
        }),
    );
  }, []);

  const onConnect = React.useCallback(() => {
    const cfg: EndpointConfig = { url, token: token.trim() || undefined };
    setConfig(cfg);
    setConnected(true);
    setQueryState({ kind: "idle" });
    startHealthFetch(cfg);
  }, [url, token, startHealthFetch]);

  const onDisconnect = React.useCallback(() => {
    setConnected(false);
    setConfig(null);
    setHealth({ kind: "idle" });
    setQueryState({ kind: "idle" });
    // Clear the token from memory on disconnect — never persisted.
    setToken("");
  }, []);

  const onRefreshHealth = React.useCallback(() => {
    if (config) startHealthFetch(config);
  }, [config, startHealthFetch]);

  const onRunQuery = React.useCallback(() => {
    if (!config) return;
    setQueryState({ kind: "running" });
    void runEndpointQuery(config, sparql).then(
      (result) => setQueryState({ kind: "result", result }),
      (err) => {
        const message =
          err instanceof EndpointError
            ? err.message
            : err instanceof Error
              ? err.message
              : String(err);
        setQueryState({ kind: "error", message });
      },
    );
  }, [config, sparql]);

  if (!connected) {
    return (
      <ConnectPanel
        url={url}
        setUrl={setUrl}
        token={token}
        setToken={setToken}
        warnings={warnings}
        blocked={blocked}
        onConnect={onConnect}
      />
    );
  }

  return (
    <div className="flex h-full flex-col">
      {/* Connected header with disconnect affordance */}
      <div className="flex items-center gap-2 border-b bg-card px-3 py-1.5 text-xs">
        <Server className="size-3.5 text-primary" />
        <span
          className="min-w-0 flex-1 truncate font-mono text-muted-foreground"
          title={config?.url}
        >
          {config?.url}
        </span>
        <Button
          size="sm"
          variant="ghost"
          onClick={onDisconnect}
          title="Disconnect from this endpoint"
        >
          <Unplug className="size-3.5" />
          Disconnect
        </Button>
      </div>

      {/* Health status row */}
      <HealthPanel health={health} onRefresh={onRefreshHealth} />

      {/* Query panel — available immediately after connect */}
      <QueryPanel
        sparql={sparql}
        setSparql={setSparql}
        queryState={queryState}
        onRun={onRunQuery}
      />
    </div>
  );
}
