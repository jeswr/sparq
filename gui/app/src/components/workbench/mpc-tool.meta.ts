// [OPUS-5] sq-ixc3.17 — the MPC tool's honesty-metadata override, split out of the panel file
// so the tool-panel registry can read it EAGERLY (the rail/tab/palette honesty read path needs
// it at first paint) while the panel component stays behind a lazy dynamic import().

import type { ToolOverride } from "@/data/tools";

/**
 * Honesty-metadata override — flips this tool from an honest stub to a working panel.
 * The tier STAYS `live-sim`: the panel runs a faithful in-tab JS illustration of the protocol
 * shape over contributions read from the live store — it is NOT the native `sparq-mpc` crate
 * and NOT live MPC. Never edit data/tools.ts.
 */
export const MPC_TOOL_OVERRIDE: ToolOverride = {
  built: true,
  group: "working",
  blurb:
    "Secure threshold over a column of the live store: reveal only Σ contributions ≥ threshold. In-tab JS illustration; NOT the native protocol, NOT live MPC.",
};
