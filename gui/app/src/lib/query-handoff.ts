// [OPUS-5] sq-ixc3.24 — the "open this in the Query tool" bridge.
//
// The Query tool owns its editor text and hydrates it from the active workspace ONCE per
// workspace id, so persisting a query through `setEditorQuery` does NOT move an already-open
// editor. The visual query builder needs exactly that move — its whole contract is that the
// generated SPARQL lands in the REAL editor, unmodified, where it can be read, edited and run
// like any hand-written query (no hidden dialect, no second execution path).
//
// This is the smallest thing that does it: a module-level, one-slot publish/subscribe. It is
// deliberately NOT React context — the publisher (a tool panel) and the subscriber (the Query
// panel) are siblings mounted independently by the tab strip, and a context would have to be
// threaded through the shell to join them.
//
// Semantics:
//   * `publishQuery` notifies every current subscriber. If nobody is listening — the Query tab
//     has never been opened — the query is PARKED instead, so the handoff still lands when that
//     tab first mounts. A handoff that WAS delivered live parks nothing, so it can never be
//     replayed later over the user's own edits.
//   * `takePendingQuery` consumes the parked value exactly once.

export type QueryHandoffListener = (query: string) => void;

const listeners = new Set<QueryHandoffListener>();
let pending: string | null = null;

/** Hand a query to the Query tool. Returns the number of listeners that received it now. */
export function publishQuery(query: string): number {
  const delivered = listeners.size;
  // Park it only when nothing could receive it; a live delivery must not also leave a ghost.
  pending = delivered > 0 ? null : query;
  for (const listener of [...listeners]) listener(query);
  return delivered;
}

/**
 * Subscribe to handoffs. Returns the unsubscribe function (the shape `useEffect` wants).
 * Does NOT replay the pending value — use {@link takePendingQuery} for the mount-time case, so
 * the "apply once" accounting stays in one place.
 */
export function subscribeToQueryHandoff(listener: QueryHandoffListener): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

/** Consume the pending handoff, or null when there is none. Consuming clears it. */
export function takePendingQuery(): string | null {
  const value = pending;
  pending = null;
  return value;
}

/** Test-only reset so a case never inherits another's pending value. */
export function resetQueryHandoff(): void {
  listeners.clear();
  pending = null;
}

// ---------------------------------------------------------------------------
// The restore-vs-handoff gate
// ---------------------------------------------------------------------------
//
// The Query panel has two things that want to own the editor text: the workspace restore (which
// hydrates the saved query once per workspace id, asynchronously) and a handoff. They race only
// at mount: a handoff can open the Query tab, so it can land BEFORE the restore that would then
// clobber it. Modelled here as a pure state machine so that ordering rule is testable without
// mounting React — the panel just keeps one of these in a ref.

export interface RestoreGate {
  /** The workspace id whose saved query is loaded, or null before the first hydration. */
  readonly hydrated: string | null;
  /** Whether the next hydration must be skipped because a handoff won the mount race. */
  readonly skipNextRestore: boolean;
}

export const INITIAL_RESTORE_GATE: RestoreGate = { hydrated: null, skipNextRestore: false };

/**
 * A handoff arrived. It outranks the pending restore ONLY while nothing has been hydrated yet —
 * that is the mount race it exists for. Once a workspace IS loaded, the handoff simply replaces
 * the current text; a LATER workspace switch must still restore that workspace's saved query.
 */
export function onQueryHandoff(gate: RestoreGate): RestoreGate {
  if (gate.hydrated !== null) return gate;
  return { ...gate, skipNextRestore: true };
}

/**
 * The workspace effect wants to load `workspaceId`'s saved query. Returns the next gate and
 * whether the caller should actually restore — false when this workspace is already loaded (never
 * re-clobber in-progress edits) or when a handoff won the mount race for this first hydration.
 */
export function onWorkspaceHydrate(
  gate: RestoreGate,
  workspaceId: string,
): { gate: RestoreGate; restore: boolean } {
  if (gate.hydrated === workspaceId) return { gate, restore: false };
  return {
    gate: { hydrated: workspaceId, skipNextRestore: false },
    restore: !gate.skipNextRestore,
  };
}
