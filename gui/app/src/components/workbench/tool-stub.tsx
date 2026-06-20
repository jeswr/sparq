"use client";

// [OPUS-4.8] sq-ixc3.9 — an HONEST tool stub. The foundation shell registers every TOOL but
// only the Query tool is wired; the rest open a panel that states plainly what the tool WILL do
// over the live store, its honesty tier, and which bead lands it. It NEVER fabricates a result
// or dresses a walkthrough/soon capability up as live.

import { Badge } from "@/components/ui/badge";
import { toolById, TIER_META } from "@/data/tools";

/** Which bead lands each tool (so the stub is self-documenting, not a dead placeholder). */
const TOOL_BEAD: Record<string, string> = {
  "graph-view": "sq-lyp8",
  shacl: "sq-ixc3.11",
  inference: "sq-ixc3.11",
  "full-text": "sq-ixc3.11",
  vector: "sq-zeai",
  geosparql: "sq-zeai",
  streaming: "sq-ixc3.11",
  zk: "sq-ixc3.11",
  mpc: "sq-ixc3.11",
  server: "sq-2mke",
};

/** Tier → the not-yet-audited / not-live caveat the privacy + honesty gates require. */
const TIER_CAVEAT: Record<string, string> = {
  "live-bbjs":
    "In-tab UltraHonk proving is real, but the ZK estate is research-grade — the v1 verifier is internally re-audited only, external accredited-cryptographer sign-off is pending — so it is NOT an audited cryptographic guarantee.",
  "live-sim":
    "This will be a faithful in-tab JS illustration of the protocol shape, NOT the native sparq-mpc crate and NOT live MPC; sparq-mpc itself is honest-majority semi-honest only.",
  walkthrough:
    "This capability is a native-only crate not yet compiled to wasm, so it will start as a captured-I/O walkthrough until a portability spike or endpoint mode lands.",
};

export function ToolStub({ toolId }: { toolId: string }) {
  const tool = toolById(toolId);
  if (!tool) {
    return (
      <div className="flex h-full items-center justify-center p-8 text-sm text-muted-foreground">
        Unknown tool: {toolId}
      </div>
    );
  }
  const Icon = tool.icon;
  const tier = TIER_META[tool.tier];
  const bead = TOOL_BEAD[toolId];
  const caveat = TIER_CAVEAT[tool.tier];
  return (
    <div className="flex h-full items-center justify-center overflow-auto p-8">
      <div className="max-w-md space-y-3 text-center">
        <Icon className="mx-auto size-10 text-muted-foreground" />
        <div className="flex items-center justify-center gap-2">
          <h2 className="text-lg font-semibold">{tool.label}</h2>
          <span className="flex items-center gap-1.5 text-xs text-muted-foreground">
            <span className={`size-2 rounded-full ${tier.dot}`} aria-hidden />
            {tier.label}
          </span>
        </div>
        <p className="text-sm text-muted-foreground">{tool.blurb}</p>
        {caveat ? (
          <p className="rounded-md border border-[var(--warning)]/30 bg-[var(--warning)]/5 p-2 text-xs text-muted-foreground">
            {caveat}
          </p>
        ) : null}
        <div className="flex items-center justify-center gap-2 pt-1">
          <Badge variant="muted">Not yet built</Badge>
          {bead ? <Badge variant="outline">{bead}</Badge> : null}
        </div>
        <p className="text-xs text-muted-foreground">
          This tool is registered in the workbench shell; the working panel is a later phase.
          The Query tool runs live over the in-tab engine today.
        </p>
      </div>
    </div>
  );
}
