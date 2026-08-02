// [OPUS-5] sq-ixc3.17 — the ZK tool's honesty-metadata override, split out of the panel file so
// the tool-panel registry can read it EAGERLY (the rail/tab/palette honesty read path needs it at
// first paint) while the panel itself stays behind a lazy dynamic import(). Per-tool file — the
// tier/copy/group flips HERE, never in the shared data/tools.ts (the sq-5lyme seam).

import type { ToolOverride } from "@/data/tools";

/**
 * Flips the ZK tool from an honest stub to a working panel. The tier STAYS `live-bbjs`: the
 * in-tab UltraHonk proving is real 3rd-party WASM, but the sparq ZK estate is research-grade and
 * external accredited-cryptographer sign-off is pending (sq-qhy4). The blurb now names what the
 * panel actually proves over the live store.
 */
export const ZK_TOOL_OVERRIDE: ToolOverride = {
  built: true,
  group: "working",
  blurb:
    "Prove a comparison about an integer a SPARQL query returned from the live store, without disclosing it (UltraHonk, in-tab). Research-grade; NOT externally audited.",
};
