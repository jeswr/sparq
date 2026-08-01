// [OPUS-5] sq-ixc3.17 — the ZK tool's honesty-metadata override, split out of the panel file
// so the tool-panel registry can read it EAGERLY (the rail/tab/palette honesty read path needs
// it at first paint) while the panel itself stays behind a lazy dynamic import() and out of the
// first-load bundle — which matters more here than for any other tool, since the panel's chunk
// pulls the ~MB bb.js + noir_js WASM glue.

import type { ToolOverride } from "@/data/tools";

/**
 * Honesty-metadata override — flips this tool from an honest stub to a working panel.
 * The tier STAYS `live-bbjs`: the proving is real 3rd-party WASM (bb.js UltraHonk) running
 * in the tab, and it is NOT externally audited. Never edit data/tools.ts.
 */
export const ZK_TOOL_OVERRIDE: ToolOverride = {
  built: true,
  group: "working",
  blurb:
    "Prove a FILTER over an integer literal in the live store without disclosing it — in-tab UltraHonk. Research-track; NOT externally audited.",
};
