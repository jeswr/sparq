"use client";

// [FABLE-5] sq-ixc3.14 — the per-workspace FEDERATION egress-allowlist control + the
// workspace⇄engine sync + the honest run-location badge.
//
// Federated SERVICE queries execute on the DESKTOP shell's native engine (the in-tab WASM
// engine cannot dial remote SPARQL endpoints — CORS), under the engine's STRICT fail-closed
// egress policy: ONLY the endpoints allowlisted here are reachable, everything else is refused
// pre-HTTP (the same `--service-allow` policy the network-exposed sparq-server enforces). The
// allowlist is PERSISTED on the workspace (survives a restart) and pushed to the engine in
// lockstep by {@link FederationBridge}, exactly like the inference-mode bridge.
//
// HONESTY: on the hosted web build the control still edits the persisted setting, but says
// plainly that SERVICE execution is native-only there (the tools.ts taxonomy's framing) — the
// browser build never pretends it can federate.

import * as React from "react";
import { Popover as PopoverPrimitive } from "radix-ui";
import { Globe, Plus, X } from "lucide-react";

import { cn } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";
import { useEngine } from "@/lib/engine-context";
import { useWorkspace } from "@/lib/workspace-context";
import { queryUsesService } from "@/lib/federation";

/**
 * Keep the ENGINE's federation egress allowlist in lockstep with the active workspace's
 * PERSISTED setting. Renders null; mounted once in the workbench shell (like the inference
 * bridge) so a restored workspace's allowlist applies before its first SERVICE run.
 */
export function FederationBridge() {
  const { serviceAllowlist } = useWorkspace();
  const { setServiceAllowlist } = useEngine();
  React.useEffect(() => {
    setServiceAllowlist(serviceAllowlist);
  }, [serviceAllowlist, setServiceAllowlist]);
  return null;
}

/**
 * The honest "where does this query run" badge for the Query action row. A plain query runs
 * in the in-tab WASM engine; a SERVICE-bearing query runs on the desktop's NATIVE engine
 * (federated, allowlist-gated) — or, on the web build, is labelled native-only per the
 * tools.ts tier taxonomy instead of pretending.
 */
export function RunLocationBadge({ query }: { query: string }) {
  const { nativeFederationAvailable } = useEngine();
  const federated = queryUsesService(query);
  if (!federated) {
    return (
      <Badge
        variant="outline"
        className="h-5 gap-1 text-[10px]"
        title="Where this query runs"
        data-run-location="local-wasm"
      >
        LOCAL · in-tab WASM
      </Badge>
    );
  }
  if (nativeFederationAvailable) {
    return (
      <Badge
        variant="outline"
        className="h-5 gap-1 text-[10px] text-primary"
        title="SERVICE detected: this query runs on the desktop's native engine, dialing only endpoints on the workspace's federation allowlist (fail-closed)"
        data-run-location="native-federated"
      >
        NATIVE · federated SERVICE
      </Badge>
    );
  }
  return (
    <Badge
      variant="outline"
      className="h-5 gap-1 text-[10px] text-muted-foreground"
      title="SERVICE detected: federated queries are native-only — the browser build cannot dial remote SPARQL endpoints (CORS). Run this in the sparq desktop app."
      data-run-location="service-native-only"
    >
      SERVICE · native-only
    </Badge>
  );
}

/**
 * The compact per-workspace federation allowlist editor for the query toolbar. Entries are
 * `host`, `host:port`, or `*.suffix` (the same grammar as sparq-server's `--service-allow`).
 * FAIL-CLOSED: an empty list refuses every SERVICE endpoint.
 */
export function FederationControl({ className }: { className?: string }) {
  const { serviceAllowlist, setServiceAllowlist } = useWorkspace();
  const { nativeFederationAvailable } = useEngine();
  const [draft, setDraft] = React.useState("");

  const add = React.useCallback(() => {
    const entry = draft.trim();
    if (!entry) return;
    setDraft("");
    if (serviceAllowlist.includes(entry)) return;
    void setServiceAllowlist([...serviceAllowlist, entry]);
  }, [draft, serviceAllowlist, setServiceAllowlist]);

  const remove = React.useCallback(
    (entry: string) => {
      void setServiceAllowlist(serviceAllowlist.filter((e) => e !== entry));
    },
    [serviceAllowlist, setServiceAllowlist],
  );

  return (
    <PopoverPrimitive.Root>
      <PopoverPrimitive.Trigger asChild>
        <button
          type="button"
          data-federation-control
          title="Federation: the SPARQL endpoints SERVICE queries may dial (fail-closed allowlist)"
          className={cn(
            "flex items-center gap-1.5 rounded-md border bg-background px-1.5 py-0.5",
            "text-[11px] text-muted-foreground hover:bg-accent/40",
            className,
          )}
        >
          <Globe className="size-3.5" aria-hidden />
          <span className="font-medium">Federation</span>
          <span data-federation-count className={serviceAllowlist.length > 0 ? "text-primary" : ""}>
            {serviceAllowlist.length}
          </span>
        </button>
      </PopoverPrimitive.Trigger>
      <PopoverPrimitive.Portal>
        <PopoverPrimitive.Content
          data-federation-popover
          align="start"
          sideOffset={4}
          className={cn(
            "z-50 w-72 rounded-lg border bg-popover p-2 text-popover-foreground shadow-md",
            "data-[state=open]:animate-in data-[state=closed]:animate-out",
            "data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0",
          )}
        >
          <p className="px-1 pb-1 text-xs font-medium">Federation egress allowlist</p>
          <p className="px-1 pb-2 text-[11px] leading-snug text-muted-foreground">
            SERVICE queries run on the native engine and may dial ONLY these endpoints
            (host, host:port, or *.suffix). Fail-closed: an empty list refuses every
            SERVICE endpoint.
          </p>
          {!nativeFederationAvailable && (
            <p
              className="mx-1 mb-2 rounded border border-dashed px-2 py-1 text-[11px] text-muted-foreground"
              data-federation-web-note
            >
              Native-only: this browser build cannot execute SERVICE queries (CORS). The
              setting persists with the workspace and applies in the desktop app.
            </p>
          )}
          <ul className="mb-2 space-y-1" aria-label="Allowlisted endpoints">
            {serviceAllowlist.length === 0 && (
              <li className="px-1 text-[11px] italic text-muted-foreground" data-federation-empty>
                No endpoints allowlisted — all SERVICE calls refused.
              </li>
            )}
            {serviceAllowlist.map((entry) => (
              <li
                key={entry}
                data-federation-entry={entry}
                className="flex items-center justify-between gap-2 rounded bg-accent/30 px-2 py-0.5"
              >
                <code className="truncate text-[11px]">{entry}</code>
                <button
                  type="button"
                  aria-label={`Remove ${entry} from the allowlist`}
                  data-federation-remove={entry}
                  onClick={() => remove(entry)}
                  className="text-muted-foreground hover:text-destructive"
                >
                  <X className="size-3" aria-hidden />
                </button>
              </li>
            ))}
          </ul>
          <div className="flex items-center gap-1">
            <input
              value={draft}
              data-federation-input
              onChange={(e) => setDraft(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  e.preventDefault();
                  add();
                }
              }}
              placeholder="query.example.org:8890"
              aria-label="Endpoint host to allowlist"
              className="h-6 flex-1 rounded border bg-background px-1.5 text-[11px] outline-none focus:border-primary"
            />
            <button
              type="button"
              data-federation-add
              onClick={add}
              aria-label="Add endpoint to the allowlist"
              className="flex h-6 items-center gap-0.5 rounded border px-1.5 text-[11px] text-muted-foreground hover:bg-accent/40"
            >
              <Plus className="size-3" aria-hidden /> Allow
            </button>
          </div>
        </PopoverPrimitive.Content>
      </PopoverPrimitive.Portal>
    </PopoverPrimitive.Root>
  );
}
