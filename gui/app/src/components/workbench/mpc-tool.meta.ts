// [OPUS-5] sq-ixc3.17 — the MPC tool's honesty-metadata override, split out of the panel file so
// the tool-panel registry can read it EAGERLY (the rail/tab/palette honesty read path needs it at
// first paint) while the panel itself stays behind a lazy dynamic import(). Per-tool file — the
// tier/copy/group flips HERE, never in the shared data/tools.ts (the sq-5lyme seam).

import type { ToolOverride } from "@/data/tools";

/**
 * Flips the MPC tool from an honest stub to a working panel. The tier STAYS `live-sim`: what runs
 * is an in-tab JS illustration of the additive-sharing protocol shape over the user's live store,
 * NOT the native `sparq-mpc` crate and NOT live MPC. The blurb now names the live-store binding,
 * which is the only thing that changed.
 */
export const MPC_TOOL_OVERRIDE: ToolOverride = {
  built: true,
  group: "working",
  blurb:
    "Secret-share values a SPARQL query returns from the live store and disclose only the ≥ threshold bit. In-tab JS illustration; NOT the native protocol, NOT live MPC.",
};
