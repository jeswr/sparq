// [OPUS-4.8] sq-atb0 (epic sq-ixc3) — the site-side glue between the live wasm Store and the
// framework-agnostic WORKSPACE model in `@sparq/client`.
//
// The workspace model is pure data + a persistence interface; it knows nothing about the wasm
// engine. This thin module is the ONE place that bridges the two: it serialises the loaded
// dataset to the workspace's N-Quads SNAPSHOT (the save/open cache) and re-hydrates a Store
// from that snapshot on open. It stays in the site (NOT in `@sparq/client`) because it is
// coupled to the site's dataset-format knowledge — exactly like `storeToNQuads` /
// `loadIntoStore` in `@/lib/sparq-wasm`, which it reuses rather than re-implements.

"use client";

import {
  loadSparq,
  loadIntoStore,
  storeToNQuads,
  type WasmStore,
} from "@/lib/sparq-wasm";

/**
 * Serialise the WHOLE dataset of a live wasm {@link WasmStore} (default graph + every named
 * graph) to the N-Quads text a workspace stores as its `dataSnapshot` — the save/open cache.
 * Reuses {@link storeToNQuads}, so it agrees with the "add to current" merge path exactly. An
 * empty store yields `""`.
 */
export function snapshotStore(store: WasmStore): string {
  return storeToNQuads(store);
}

/**
 * Re-hydrate a fresh wasm {@link WasmStore} from a workspace's N-Quads `dataSnapshot`. The
 * snapshot always carries named graphs (it is N-Quads), so it is loaded with the
 * named-graph-preserving `loadDataset` path via {@link loadIntoStore}. An empty / whitespace
 * snapshot yields an empty store. This is the save/open OPEN side: the restored store is
 * byte-for-byte the dataset the workspace was saved with — no re-ingest from the original
 * sources.
 */
export async function restoreStoreFromSnapshot(
  snapshot: string | undefined,
): Promise<WasmStore> {
  const Store = await loadSparq();
  return loadIntoStore(Store, snapshot?.trim() ?? "", "nquads");
}
