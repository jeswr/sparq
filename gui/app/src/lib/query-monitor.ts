// [FABLE-5] sq-ixc3.19 — the live query monitor: a tiny observable registry of the
// queries THIS workbench session has in flight, each with a working Kill.
//
// HONEST SCOPE (stated in the panel UI too): this tracks queries ISSUED FROM THIS
// WORKBENCH — the in-tab wasm runs (cooperatively cancelled between streamed batches;
// the same mechanism as the Query tool's Stop) and endpoint-mode requests (killed by
// aborting the underlying `fetch`). It is NOT a server-side monitor: sparq-server has no
// running-queries registry / kill endpoint today (queries there are bounded by the
// cooperative `QueryBudget` timeout + row cap), so a GLOBAL all-clients monitor is
// tracked as follow-up work rather than faked here.
//
// Plain module state + subscriber set (not React context) so it is unit-testable with
// the node runner and consumable from React via `useSyncExternalStore`.

/** Where a tracked query is executing. */
export type QueryTarget = "local" | "native" | "endpoint";

/** One in-flight query. */
export interface RunningQuery {
  /** Monotonic per-session id. */
  id: number;
  /** Short human label (the tool + mode that started it). */
  label: string;
  /** The SPARQL text (the panel truncates for display; never logged elsewhere). */
  sparql: string;
  target: QueryTarget;
  /** performance.now() at start — for a live elapsed readout (measured, not a benchmark). */
  startedAt: number;
  /** Abort handle: local runs check it between streamed batches; endpoint runs abort fetch. */
  controller: AbortController;
}

let nextId = 1;
let running: RunningQuery[] = [];
const listeners = new Set<() => void>();

function emit(): void {
  for (const l of listeners) l();
}

/** Subscribe to registry changes (returns the unsubscribe). `useSyncExternalStore`-shaped. */
export function subscribeRunning(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

/** The current in-flight list (a stable snapshot reference until the next change). */
export function getRunning(): RunningQuery[] {
  return running;
}

/**
 * Track a query for its lifetime: registers it, and returns its abort signal + the
 * `finish` the issuer MUST call (in a `finally`) to deregister. The signal must actually
 * be wired into the run (RunOptions.signal / the fetch init) — the registry can only
 * kill what listens.
 */
export function trackQuery(
  label: string,
  sparql: string,
  target: QueryTarget,
): { id: number; signal: AbortSignal; finish: () => void } {
  const entry: RunningQuery = {
    id: nextId++,
    label,
    sparql,
    target,
    startedAt: typeof performance !== "undefined" ? performance.now() : 0,
    controller: new AbortController(),
  };
  running = [...running, entry];
  emit();
  return {
    id: entry.id,
    signal: entry.controller.signal,
    finish: () => {
      running = running.filter((r) => r.id !== entry.id);
      emit();
    },
  };
}

/**
 * Kill one tracked query: aborts its controller (the wasm run cancels at the next batch
 * boundary; an endpoint fetch rejects with AbortError). The entry stays listed until its
 * issuer's `finish` runs — the list reflects what is genuinely still executing, not an
 * optimistic removal.
 */
export function killQuery(id: number): boolean {
  const entry = running.find((r) => r.id === id);
  if (!entry) return false;
  entry.controller.abort();
  return true;
}

/** TEST-ONLY: reset module state between node-test cases. */
export function resetQueryMonitorForTests(): void {
  running = [];
  nextId = 1;
  listeners.clear();
}
