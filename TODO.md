# TODO

## sparq-core: cheap graph snapshot API (would simplify the server's update path)

The server's SPARQL Update path (T17 wiring, `crates/sparq-server/src/http.rs::Writer`)
achieves microsecond steady-state updates **without core changes** by double-buffering:
two physical `Graph`s alternate between *published* and *spare*, because `Graph` is not
(and should not be naively) `Clone` — a deep copy of the dictionary arena and the six
permutation indexes is O(n).

A core **cheap snapshot** API would remove the two costs the double buffer pays:

* `Graph::snapshot(&self) -> GraphSnapshot` (or `Graph: cheap Clone`): an O(1)
  copy sharing the immutable base — `TripleStore::perms` and the compacted
  dictionary storage behind `Arc`s — with the mutable parts (the delta `Overlay`,
  the append-only dictionary tail, the numeric-cache tail) either cloned (they are
  small, O(pending updates)) or made copy-on-write.
* The writer would then keep ONE master `Graph`, `update_in_place` it (µs), and
  publish `Arc::new(master.snapshot())` per commit.

Wins over the current server design:
* removes the ~2x graph residency (the second buffer);
* removes the one-time O(graph) rebuild the **first** update pays to materialise
  the second buffer (and again after a failed update discards a buffer);
* removes the spare-reclaim wait / rebuild fallback when a reader holds a snapshot
  past the query budget.

Requirements for the API: snapshots must be (1) O(overlay), never O(triples);
(2) immutable and `Send + Sync`; (3) unaffected by later `update_in_place`/`compact`
calls on the master (compaction must not invalidate live snapshots — e.g. the new
base goes into fresh `Arc`s). The server-side wiring keeps its shape: publish slot
`RwLock<Arc<Graph>>` + single-writer mutex; only `Writer`'s buffer juggling collapses.
