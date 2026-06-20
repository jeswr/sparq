"use client";

// [OPUS-4.8] sq-ixc3.9 — the thin TOP BAR (h-10, research/gui-design.md §A.2):
// LOCAL⇄ENDPOINT target switch · store size · ⌘K hint · theme toggle · engine status LED.
//
// The target switch + Cmd-K are STUBS in this foundation shell (the endpoint/connect tool is
// sq-ixc3.11/the Server tab; the command palette is sq-ixc3.10). They render and are honestly
// labelled "soon" so the bar is complete chrome — they don't fabricate behaviour.

import * as React from "react";
import { useTheme } from "next-themes";
import { Moon, Sun, Command, ExternalLink } from "lucide-react";

import { Button } from "@/components/ui/button";
import { useEngine } from "@/lib/engine-context";

/** The honesty-website link target (the marketing site, opened in a NEW tab / system browser). */
const WEBSITE_URL = "https://jeswr.github.io/sparq/";

function StatusLed() {
  const { status } = useEngine();
  const color =
    status.kind === "ready"
      ? "bg-[var(--success)]"
      : status.kind === "error"
        ? "bg-destructive"
        : "bg-[var(--warning)] animate-pulse";
  const label =
    status.kind === "ready"
      ? "Engine ready"
      : status.kind === "error"
        ? "Engine error"
        : "Warming…";
  return (
    <span className="flex items-center gap-1.5 text-xs text-muted-foreground" title={label}>
      <span className={`size-2 rounded-full ${color}`} aria-hidden />
      {/* The literal "Engine ready" copy is the deterministic hook the tauri-driver e2e waits on. */}
      <span>{label}</span>
    </span>
  );
}

function ThemeToggle() {
  const { resolvedTheme, setTheme } = useTheme();
  const [mounted, setMounted] = React.useState(false);
  React.useEffect(() => setMounted(true), []);
  if (!mounted) {
    return <Button variant="ghost" size="icon-sm" aria-label="Toggle theme" />;
  }
  const isDark = resolvedTheme === "dark";
  return (
    <Button
      variant="ghost"
      size="icon-sm"
      aria-label="Toggle theme"
      onClick={() => setTheme(isDark ? "light" : "dark")}
    >
      {isDark ? <Sun /> : <Moon />}
    </Button>
  );
}

export function TopBar() {
  const { storeSize } = useEngine();
  return (
    <header className="flex h-10 shrink-0 items-center gap-3 border-b bg-card px-3">
      <span className="font-mono text-sm font-semibold tracking-tight">sparq</span>
      <span className="text-[10px] uppercase tracking-wider text-muted-foreground">
        workbench
      </span>

      {/* LOCAL⇄ENDPOINT target switch — stub (the Server/endpoint tool is a later phase). */}
      <div className="ml-2 flex items-center rounded-md border bg-background text-xs">
        <button
          className="rounded-l-md bg-primary px-2 py-1 font-medium text-primary-foreground"
          title="Run queries against the in-tab engine (this build)"
        >
          LOCAL
        </button>
        <button
          className="cursor-not-allowed rounded-r-md px-2 py-1 text-muted-foreground"
          title="Endpoint mode (connect to a running sparq-server) — coming in a later phase"
          disabled
        >
          ENDPOINT
        </button>
      </div>

      <span className="tabular text-xs text-muted-foreground">
        {storeSize.toLocaleString()} quads
      </span>

      <div className="ml-auto flex items-center gap-1.5">
        {/* ⌘K hint — the command palette is sq-ixc3.10. */}
        <span
          className="hidden items-center gap-1 rounded-md border px-1.5 py-0.5 text-[11px] text-muted-foreground sm:flex"
          title="Command palette — coming in a later phase"
        >
          <Command className="size-3" />K
        </span>
        <StatusLed />
        <ThemeToggle />
        <Button
          variant="ghost"
          size="sm"
          asChild
          title="Open the sparq website (the marketing/docs site) in your browser"
        >
          <a href={WEBSITE_URL} target="_blank" rel="noreferrer noopener">
            Help <ExternalLink className="size-3" />
          </a>
        </Button>
      </div>
    </header>
  );
}
