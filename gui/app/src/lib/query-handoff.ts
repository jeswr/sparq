// [SONNET-4.6] sq-ixc3.24 (#2700) — the one-way SPARQL handoff from a tool that GENERATES a
// query (the visual query builder) into the Query tool's editor.
//
// Why a latch and not a context value: the Query tab may not be MOUNTED when the handoff is
// requested (a user can close it and work in the builder alone), so a plain event would be lost
// between `request()` and the tab opening. The pending query is therefore held until a listener
// takes it — the caller does `requestQueryHandoff(sparql)` then `openTool("query")`, and the
// Query panel picks it up whether it was already mounted or mounts a moment later.
//
// It carries TEXT ONLY: the receiving editor stays a plain SPARQL editor the user can edit and
// run, with no hidden state travelling alongside the query.

type QueryHandoffListener = (sparql: string) => void;

const listeners = new Set<QueryHandoffListener>();
let pending: string | null = null;

/**
 * Hand a SPARQL query to the Query tool's editor. Delivered immediately if the editor is
 * mounted, otherwise latched until it mounts (a newer request replaces an undelivered one).
 */
export function requestQueryHandoff(sparql: string): void {
  if (listeners.size === 0) {
    pending = sparql;
    return;
  }
  pending = null;
  for (const listener of [...listeners]) listener(sparql);
}

/**
 * Subscribe the Query editor. Any latched query is delivered on subscribe (and cleared, so it
 * is applied exactly once). Returns the unsubscribe function.
 */
export function subscribeQueryHandoff(listener: QueryHandoffListener): () => void {
  listeners.add(listener);
  if (pending !== null) {
    const sparql = pending;
    pending = null;
    listener(sparql);
  }
  return () => {
    listeners.delete(listener);
  };
}

/** Test-only: drop any latched query and every listener. */
export function resetQueryHandoff(): void {
  listeners.clear();
  pending = null;
}
