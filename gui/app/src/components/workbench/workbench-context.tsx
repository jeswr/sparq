"use client";

// [OPUS-4.8] sq-ixc3.10 — a thin context exposing the shell's tab actions to the tools, so a tool
// panel's Cmd-K commands (e.g. the Query tool's "Run query") can FOCUS their own tab when invoked
// from anywhere in the workbench. The shell (workbench.tsx) owns the tab state; this just hands the
// `openTool` capability down without prop-drilling through ToolPanel.

import * as React from "react";

/**
 * [OPUS-5] sq-ixc3.24 — one hand-off of SPARQL text to the Query tool. `seq` increments on every
 * send so the Query editor re-loads the text even when the SAME query is sent twice; without it a
 * repeat send would be a silent no-op the user reads as "the button did nothing".
 */
export interface QueryHandoff {
  sparql: string;
  seq: number;
}

export interface WorkbenchActions {
  /** Open a tool as a tab (or focus it if already open). */
  openTool: (toolId: string) => void;
  /**
   * [OPUS-5] sq-ixc3.24 — load `sparql` into the Query tool's editor and focus that tab. The text
   * crosses VERBATIM (the visual builder emits standard SPARQL and nothing else) and REPLACES the
   * editor's current content, which the Query tool then persists per workspace as usual.
   */
  sendToQueryEditor: (sparql: string) => void;
  /** The latest hand-off, for the Query tool to consume. `null` until something is sent. */
  queryHandoff: QueryHandoff | null;
}

const WorkbenchContext = React.createContext<WorkbenchActions | null>(null);

export function WorkbenchProvider({
  actions,
  children,
}: {
  actions: WorkbenchActions;
  children: React.ReactNode;
}) {
  return <WorkbenchContext.Provider value={actions}>{children}</WorkbenchContext.Provider>;
}

/** The shell tab actions. Returns null when used outside a provider (e.g. an isolated test). */
export function useWorkbench(): WorkbenchActions | null {
  return React.useContext(WorkbenchContext);
}
