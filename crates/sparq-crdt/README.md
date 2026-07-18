<!-- [FABLE-5] sq-tag1q.7.4: internal-stub README for a publish=false crate. internal-stub -->
# sparq-crdt

**Executable formal model + convergence verification harness** for the SPARQL-CRDT
proposal (`site/specs/sparql-crdt.typ`, bead `sq-tag1q.4`): the exact dotted-set join
equations (`CRDT-JOIN-1`), causal clock/cloud contexts (`CRDT-CTX-1`), evaluate-at-origin
SPARQL Update compilation (`CRDT-MUT-*` / `CRDT-UPD-*`), a bounded exhaustive
multi-replica model checker, and generated schedules that permute, duplicate, batch,
snapshot, compact, and replay deltas (bead `sq-tag1q.7.4`).

**Internal — not published** (`publish = false`). This crate is *not* the production
SPARQL-CRDT implementation (the rest of epic `sq-tag1q.7` supplies that); it is the
executable specification the production crate must be differentially tested against.

**Evidence, not proof.** The bounded model check is exhaustive only within its stated
bounds, and property tests over generated schedules are sampled evidence. No formal
convergence proof is claimed here: the semilattice argument sketched in the proposal
remains an open proof obligation until a mechanized or peer-reviewed proof artifact
exists and has been independently reviewed.
