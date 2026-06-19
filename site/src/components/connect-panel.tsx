"use client";

// [OPUS-4.8] sq-2mke — GUI Phase 2 item 6: the "Connect" panel for endpoint mode.
//
// This panel lets the SAME SPARQL editor run against any running SPARQL 1.1 Protocol
// endpoint (a `sparq-server`, or any conformant endpoint) instead of the in-tab WASM
// store. It CONSUMES the existing server API (`crates/sparq-server/src/http.rs`) — it
// changes nothing server-side — and honours the server's security posture
// (`crates/sparq-server/README.md`) as honest connection-safety UX, never bypassing a
// gate. The bearer token is sent only in the `Authorization` header by the shared client;
// this panel never logs it.
//
// All wire-protocol + safety logic lives in the framework-agnostic `@sparq/client`
// endpoint module; this component is just the React host that draws the form, the
// classified warnings, and the connection-test result.

import * as React from "react";
import {
  Loader2,
  PlugZap,
  Plug,
  ShieldAlert,
  ShieldCheck,
  Info,
  CircleCheck,
} from "lucide-react";

import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import {
  type EndpointConfig,
  type SafetyWarning,
  connectionSafetyWarnings,
  hasBlockingWarning,
  parseEndpointUrl,
  runEndpointQuery,
} from "@sparq/client";

/** The connection-test lifecycle, surfaced inline under the form. */
type TestState =
  | { kind: "idle" }
  | { kind: "testing" }
  | { kind: "ok"; ms: number }
  | { kind: "fail"; message: string };

export interface ConnectPanelProps {
  /** The current endpoint config (URL + optional token), lifted into the REPL. */
  config: EndpointConfig;
  /** Update the config (controlled by the parent so the REPL can route queries). */
  onConfigChange: (config: EndpointConfig) => void;
  /** Whether endpoint mode is currently active (queries route to the endpoint). */
  active: boolean;
  /** Toggle endpoint mode on/off. */
  onActiveChange: (active: boolean) => void;
}

/**
 * [OPUS-4.8] sq-2mke — the Connect panel. Renders the endpoint URL + optional bearer
 * token, the live connection-safety warnings (pure `connectionSafetyWarnings`), an
 * "endpoint mode" toggle, and a "Test connection" button that runs a trivial `ASK {}`
 * through the shared client so a misconfig is caught before the user runs a real query.
 */
export function ConnectPanel({
  config,
  onConfigChange,
  active,
  onActiveChange,
}: ConnectPanelProps) {
  const [test, setTest] = React.useState<TestState>({ kind: "idle" });

  // The honest safety findings for the current config — recomputed as the user types.
  // Pure + synchronous (no network), so it is cheap to run on every render.
  const warnings = React.useMemo(
    () => connectionSafetyWarnings(config),
    [config],
  );
  const blocked = hasBlockingWarning(warnings);
  const urlValid = parseEndpointUrl(config.url) !== null;

  // Reset any stale test verdict when the connection details change.
  React.useEffect(() => {
    setTest({ kind: "idle" });
  }, [config.url, config.token]);

  const testConnection = React.useCallback(async () => {
    setTest({ kind: "testing" });
    try {
      // A trivial ASK is the cheapest legal SPARQL read; it exercises the exact request
      // path (POST application/sparql-query + Accept + bearer) a real query would use, so
      // a CORS / auth / reachability problem surfaces here, not mid-query.
      const t0 = performance.now();
      await runEndpointQuery(config, "ASK { ?s ?p ?o }");
      const ms = performance.now() - t0;
      setTest({ kind: "ok", ms });
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setTest({ kind: "fail", message });
    }
  }, [config]);

  return (
    <div className="space-y-3 rounded-lg border bg-muted/30 p-3">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex items-center gap-2 text-xs font-medium text-muted-foreground">
          {active ? (
            <PlugZap className="size-3.5 text-primary" />
          ) : (
            <Plug className="size-3.5" />
          )}
          Endpoint mode
          <span className="font-normal text-muted-foreground/80">
            — run the editor against a running sparq-server (SPARQL 1.1 Protocol)
          </span>
        </div>
        <ModeSwitch active={active} disabled={blocked && !active} onChange={onActiveChange} />
      </div>

      <div className="grid gap-3 sm:grid-cols-[1fr_auto]">
        <div className="space-y-1.5">
          <label htmlFor="endpoint-url" className="text-xs font-medium">
            Endpoint URL
          </label>
          <input
            id="endpoint-url"
            type="url"
            inputMode="url"
            value={config.url}
            placeholder="http://127.0.0.1:3030/sparql"
            spellCheck={false}
            autoComplete="off"
            onChange={(e) => onConfigChange({ ...config, url: e.target.value })}
            className="w-full rounded-lg border bg-muted/40 px-3 py-2 font-mono text-[13px] outline-none focus-visible:ring-3 focus-visible:ring-ring/40"
          />
        </div>
        <div className="space-y-1.5">
          <label htmlFor="endpoint-token" className="text-xs font-medium">
            Bearer token{" "}
            <span className="font-normal text-muted-foreground">(optional)</span>
          </label>
          <input
            id="endpoint-token"
            type="password"
            value={config.token ?? ""}
            placeholder="—"
            spellCheck={false}
            autoComplete="off"
            onChange={(e) => onConfigChange({ ...config, token: e.target.value })}
            className="w-full rounded-lg border bg-muted/40 px-3 py-2 font-mono text-[13px] outline-none focus-visible:ring-3 focus-visible:ring-ring/40 sm:w-56"
          />
        </div>
      </div>

      <div className="flex flex-wrap items-center gap-3">
        <Button
          variant="outline"
          size="sm"
          onClick={testConnection}
          disabled={!urlValid || test.kind === "testing"}
          title="Run a trivial ASK against the endpoint to verify reachability + auth"
        >
          {test.kind === "testing" ? (
            <Loader2 className="size-3.5 animate-spin" />
          ) : (
            <PlugZap className="size-3.5" />
          )}
          Test connection
        </Button>
        <TestResult test={test} />
      </div>

      <SafetyWarnings warnings={warnings} />

      <p className="text-[11px] leading-relaxed text-muted-foreground">
        The token is sent only in the{" "}
        <code className="font-mono">Authorization: Bearer</code> header by the shared
        client and is never logged. sparq-server has{" "}
        <span className="font-medium">no auth by default</span> and binds loopback-only;
        these warnings reflect its real posture (see the sparq-server README) — this
        editor never bypasses a server gate.
      </p>
    </div>
  );
}

// [OPUS-4.8] sq-2mke — the WASM ⇄ endpoint switch. A blocking safety error (invalid URL /
// mixed-content) disables turning endpoint mode ON, but never blocks turning it OFF (so a
// user can always fall back to the in-tab engine).
function ModeSwitch({
  active,
  disabled,
  onChange,
}: {
  active: boolean;
  disabled: boolean;
  onChange: (active: boolean) => void;
}) {
  return (
    <div
      role="tablist"
      aria-label="Execution target"
      className="inline-flex rounded-lg border bg-background p-0.5"
    >
      <SwitchTab
        label="In-tab WASM"
        selected={!active}
        onClick={() => onChange(false)}
        title="Execute queries in this tab on the WebAssembly engine (no network)"
      />
      <SwitchTab
        label="Endpoint"
        selected={active}
        disabled={disabled}
        onClick={() => onChange(true)}
        title={
          disabled
            ? "Fix the endpoint URL / transport warning before connecting"
            : "Route queries to the configured SPARQL 1.1 Protocol endpoint"
        }
      />
    </div>
  );
}

function SwitchTab({
  label,
  selected,
  disabled,
  onClick,
  title,
}: {
  label: string;
  selected: boolean;
  disabled?: boolean;
  onClick: () => void;
  title: string;
}) {
  return (
    <button
      type="button"
      role="tab"
      aria-selected={selected}
      disabled={disabled}
      title={title}
      onClick={onClick}
      className={cn(
        "rounded-md px-2.5 py-1 text-xs font-medium transition-colors outline-none focus-visible:ring-3 focus-visible:ring-ring/40 disabled:cursor-not-allowed disabled:opacity-50",
        selected
          ? "bg-primary text-primary-foreground shadow-sm"
          : "text-muted-foreground hover:text-foreground",
      )}
    >
      {label}
    </button>
  );
}

function TestResult({ test }: { test: TestState }) {
  if (test.kind === "idle") return null;
  if (test.kind === "testing") {
    return (
      <span className="text-xs text-muted-foreground" aria-live="polite">
        Sending a trivial ASK…
      </span>
    );
  }
  if (test.kind === "ok") {
    return (
      <span
        className="inline-flex items-center gap-1.5 text-xs text-[var(--success)]"
        aria-live="polite"
      >
        <CircleCheck className="size-3.5" /> Reachable — ASK answered in{" "}
        {test.ms.toFixed(1)} ms
      </span>
    );
  }
  return (
    <span
      className="inline-flex items-start gap-1.5 text-xs text-destructive"
      aria-live="polite"
    >
      <ShieldAlert className="mt-0.5 size-3.5 shrink-0" />
      <span>{test.message}</span>
    </span>
  );
}

// [OPUS-4.8] sq-2mke — render the classified connection-safety findings. `error` findings
// (invalid URL, mixed content) are hard blocks; `warning` findings (token / query over
// plaintext to a non-loopback host) are advisory; `info` findings (CORS, SERVICE
// allowlist, loopback-token note) are honest reminders. Each carries a stable `code` so
// the rendering keys on it, not on prose.
function SafetyWarnings({ warnings }: { warnings: SafetyWarning[] }) {
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
            w.level === "error" &&
              "border-destructive/30 bg-destructive/5 text-destructive",
            w.level === "warning" &&
              "border-[color-mix(in_oklch,var(--warning)_35%,transparent)] bg-[color-mix(in_oklch,var(--warning)_10%,transparent)] text-[color-mix(in_oklch,var(--warning)_80%,var(--foreground))]",
            w.level === "info" && "border-border bg-muted/40 text-muted-foreground",
          )}
        >
          <SafetyIcon level={w.level} />
          <span>{w.message}</span>
        </li>
      ))}
    </ul>
  );
}

function SafetyIcon({ level }: { level: SafetyWarning["level"] }) {
  if (level === "error") {
    return <ShieldAlert className="mt-0.5 size-3.5 shrink-0" />;
  }
  if (level === "warning") {
    return <ShieldCheck className="mt-0.5 size-3.5 shrink-0" />;
  }
  return <Info className="mt-0.5 size-3.5 shrink-0" />;
}
