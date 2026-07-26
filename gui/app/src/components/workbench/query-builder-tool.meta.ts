// [SONNET-4.6] sq-ixc3.24 (#2700) — the Query builder's honesty-metadata override, split out of
// the panel file so the tool-panel registry can read it EAGERLY (the rail/tab/palette honesty
// read path needs it at first paint) while the panel itself stays behind a lazy dynamic import()
// and out of the first-load bundle (the sq-5lyme seam).

import type { ToolOverride } from "@/data/tools";

/**
 * The builder ships WORKING: it draws a pattern over the live in-tab store, drives its predicate
 * pickers from real introspection (characteristic sets + SHACL shapes present in the store), and
 * emits standard SPARQL 1.1 into the Query tool's editor. It runs nothing itself and invents no
 * suggestions — with an empty store and no shapes the pickers say so.
 */
export const QUERY_BUILDER_TOOL_OVERRIDE: ToolOverride = {
  tier: "live",
  built: true,
  group: "working",
};
