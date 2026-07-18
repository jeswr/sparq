//! Evaluate-at-origin SPARQL Update compilation and envelope/delta derivation
//! (`CRDT-MUT-1..3`, `CRDT-UPD-1`, `CRDT-UPD-INSERT-DATA-1`,
//! `CRDT-UPD-DELETE-DATA-1`, `CRDT-UPD-WHERE-1..3`, `CRDT-UPD-BATCH-1`).
//!
//! The replication log is never the update text: an origin evaluates each
//! operation once against one immutable local snapshot, compiles it to
//! concrete quad-and-dot effects, and publishes one atomic envelope. Receivers
//! join the derived delta; they never re-evaluate a `WHERE` pattern
//! (`CRDT-UPD-WHERE-3`).

use std::collections::{BTreeMap, BTreeSet};

use crate::context::{CausalContext, Counter, Dot, ReplicaId};
use crate::state::{Delta, Quad, State};

/// A model SPARQL Update operation, restricted to the version-1 profile
/// (`CRDT-SCOPE-1`). Ground quads stand in for parsed triples after graph
/// resolution and skolemisation, which the algebraic model abstracts away.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Op {
    /// `INSERT DATA`: ground quads to add (`CRDT-UPD-INSERT-DATA-1`).
    InsertData(Vec<Quad>),
    /// `DELETE DATA`: ground quads whose *observed* dots are removed
    /// (`CRDT-UPD-DELETE-DATA-1`).
    DeleteData(Vec<Quad>),
    /// `DELETE/INSERT ... WHERE` after origin-side evaluation: the concrete
    /// instantiated delete set `D` and insert set `I` (`CRDT-UPD-WHERE-1..2`).
    DeleteInsertWhere {
        /// Concrete quads to remove (observed dots collected at the origin).
        delete: Vec<Quad>,
        /// Concrete quads to insert (one fresh dot per distinct quad).
        insert: Vec<Quad>,
    },
    /// A snapshot-dependent pattern delete, `DELETE WHERE { ?s p ?o }`: the
    /// delete set is every quad with this predicate *visible in the origin
    /// snapshot*. Keeping the pattern un-instantiated in the model makes the
    /// origin-snapshot dependence of `CRDT-UPD-WHERE-1` observable to the
    /// model checker: different interleavings compile different delete sets.
    DeleteWherePredicate(u8),
}

/// The concrete, atomic wire unit compiled from one origin operation
/// (the model projection of the `sec-interchange` JSON envelope).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Envelope {
    /// Origin replica identifier.
    pub origin: ReplicaId,
    /// Origin journal sequence (`CRDT-UPD-RETRY-1` identity component).
    pub sequence: u64,
    /// The complete data causal context of the immutable origin snapshot.
    /// Audit metadata only: it is *not* joined as a removal context
    /// (`CRDT-UPD-1` note), and `to_delta` ignores it.
    pub basis: CausalContext,
    /// Dotted adds: one fresh dot per distinct inserted quad (`CRDT-MUT-1`).
    pub adds: Vec<(Quad, Dot)>,
    /// Observed removes: per quad, exactly the dots visible in the origin
    /// snapshot (`CRDT-MUT-2`). Quads with no observed dots are omitted.
    pub removes: Vec<(Quad, BTreeSet<Dot>)>,
}

impl Envelope {
    /// Derive the joinable data delta, exactly as the interchange section
    /// specifies:
    ///
    /// ```text
    /// delta.store   = each adds[i].quad -> { adds[i].dot }
    /// delta.context = union(all adds[i].dot, all removes[i].dots)
    /// ```
    #[must_use]
    pub fn to_delta(&self) -> Delta {
        let mut store: BTreeMap<Quad, BTreeSet<Dot>> = BTreeMap::new();
        let mut dots: BTreeSet<Dot> = BTreeSet::new();
        for &(q, d) in &self.adds {
            store.entry(q).or_default().insert(d);
            dots.insert(d);
        }
        for (_, removed) in &self.removes {
            dots.extend(removed.iter().copied());
        }
        Delta::from_parts(store, CausalContext::from_dots(dots))
    }
}

/// One model replica: a durable dot-counter lineage (`CRDT-DOT-1`), a journal
/// sequence, and replica state. It acts as both origin-evaluator and replica
/// conformance role.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Replica {
    id: ReplicaId,
    next_counter: Counter,
    next_sequence: u64,
    state: State,
}

impl Replica {
    /// A fresh replica with an empty state and counter lineage starting at 1.
    #[must_use]
    pub fn new(id: ReplicaId) -> Self {
        Self {
            id,
            next_counter: 1,
            next_sequence: 1,
            state: State::bottom(),
        }
    }

    /// This replica's identifier.
    #[must_use]
    pub fn id(&self) -> ReplicaId {
        self.id
    }

    /// The current replica state.
    #[must_use]
    pub fn state(&self) -> &State {
        &self.state
    }

    fn mint_dot(&mut self) -> Dot {
        let d = Dot::new(self.id, self.next_counter);
        self.next_counter += 1;
        d
    }

    /// Execute one operation as one origin transaction (`CRDT-UPD-1`): read
    /// one immutable snapshot, compile concrete effects, mint dots, build the
    /// envelope, and atomically join its delta into local state.
    ///
    /// Per `CRDT-MUT-1`, insertion mints a fresh dot per distinct quad even if
    /// the quad is already visible; per `CRDT-MUT-2` a removal carries exactly
    /// the snapshot-visible dots (an absent quad contributes none); per
    /// `CRDT-MUT-3` a quad both deleted and inserted gets its old dots removed
    /// and a fresh live dot that never enters the removal context.
    pub fn execute(&mut self, op: &Op) -> Envelope {
        let snapshot = self.state.clone();
        let (delete_set, insert_set): (BTreeSet<Quad>, BTreeSet<Quad>) = match op {
            Op::InsertData(quads) => (BTreeSet::new(), quads.iter().copied().collect()),
            Op::DeleteData(quads) => (quads.iter().copied().collect(), BTreeSet::new()),
            Op::DeleteInsertWhere { delete, insert } => (
                delete.iter().copied().collect(),
                insert.iter().copied().collect(),
            ),
            Op::DeleteWherePredicate(p) => (
                snapshot.visible().into_iter().filter(|q| q.p == *p).collect(),
                BTreeSet::new(),
            ),
        };
        let removes: Vec<(Quad, BTreeSet<Dot>)> = delete_set
            .into_iter()
            .filter_map(|q| snapshot.store().get(&q).map(|dots| (q, dots.clone())))
            .collect();
        let adds: Vec<(Quad, Dot)> = insert_set.into_iter().map(|q| (q, self.mint_dot())).collect();
        let envelope = Envelope {
            origin: self.id,
            sequence: self.next_sequence,
            basis: snapshot.context().clone(),
            adds,
            removes,
        };
        self.next_sequence += 1;
        self.state = self.state.join(&envelope.to_delta());
        envelope
    }

    /// Execute a multi-operation request with SPARQL's sequential semantics:
    /// each operation is evaluated against the local result of the preceding
    /// ones (`CRDT-UPD-BATCH-1`). Returns one envelope per operation.
    pub fn execute_request(&mut self, ops: &[Op]) -> Vec<Envelope> {
        ops.iter().map(|op| self.execute(op)).collect()
    }

    /// Join the data deltas of several envelopes into one batch delta.
    /// `CRDT-UPD-BATCH-1` permits shipping this joined fragment only if it
    /// yields the same state as ordered evaluation — the batching property the
    /// generated-schedule tests assert.
    #[must_use]
    pub fn batched_delta(envelopes: &[Envelope]) -> Delta {
        envelopes
            .iter()
            .fold(Delta::bottom(), |acc, e| acc.join(&e.to_delta()))
    }

    /// Admit and join one received delta (replica role). Before the join, the
    /// receiver validation of `CRDT-WIRE-4` / `CRDT-STATE-1` is modelled:
    /// the delta must be well formed, and a delta dot already known under a
    /// *different* quad is rejected.
    ///
    /// The reuse scan consults only the live store because that is the whole
    /// enforceable surface: normative state (`CRDT-STATE-1`) is `(store,
    /// context)`, so once a quad's dots are removed the dot-to-quad
    /// association survives nowhere a receiver could consult. That loses no
    /// safety — a removed dot stays in the causal context, and `CRDT-JOIN-1`
    /// drops any store dot the receiving side's context already contains, so
    /// a delta reusing a removed dot under another quad joins as a no-op and
    /// can never make the forged quad visible (see
    /// `apply_absorbs_reused_removed_dot_without_reviving_it`). The context
    /// tombstone survives snapshots and compaction (`CRDT-JOURNAL-2`,
    /// `CRDT-CTX-1`), so the same protection holds after catch-up. Full
    /// no-reuse is the *origin's* durable-counter obligation (`CRDT-DOT-1`).
    ///
    /// # Errors
    ///
    /// Returns a description of the violated invariant; local state is then
    /// unchanged (fail closed).
    pub fn apply(&mut self, delta: &Delta) -> Result<(), String> {
        delta.check_invariants()?;
        for (q, dots) in delta.store() {
            for d in dots {
                for (local_q, local_dots) in self.state.store() {
                    if local_q != q && local_dots.contains(d) {
                        return Err(format!(
                            "CRDT-WIRE-4: dot {:?} already known under {:?}, delta asserts {:?}",
                            d, local_q, q
                        ));
                    }
                }
            }
        }
        let next = self.state.join(delta);
        next.check_invariants()?;
        self.state = next;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::GraphKey;

    fn q(s: u8, p: u8) -> Quad {
        Quad::new(s, p, 0, GraphKey::Default)
    }

    #[test]
    fn insert_data_dedupes_and_mints_one_fresh_dot_per_distinct_quad() {
        let mut r = Replica::new(1);
        let env = r.execute(&Op::InsertData(vec![q(1, 1), q(1, 1), q(2, 1)]));
        assert_eq!(env.adds.len(), 2); // deduplicated before minting (CRDT-MUT-1)
        assert!(env.removes.is_empty());
        assert_eq!(env.origin, 1);
        assert_eq!(env.sequence, 1);
        assert!(env.basis.is_empty());
        assert!(r.state().is_visible(&q(1, 1)));
        // Re-inserting a visible quad still mints a fresh (invisible) dot.
        let env2 = r.execute(&Op::InsertData(vec![q(1, 1)]));
        assert_eq!(env2.adds.len(), 1);
        assert_eq!(env2.sequence, 2);
        assert_ne!(env2.adds[0].1, env.adds[0].1);
    }

    #[test]
    fn delete_data_removes_only_observed_dots_and_skips_absent_quads() {
        let mut r = Replica::new(1);
        r.execute(&Op::InsertData(vec![q(1, 1)]));
        let env = r.execute(&Op::DeleteData(vec![q(1, 1), q(9, 9)]));
        assert!(env.adds.is_empty());
        assert_eq!(env.removes.len(), 1); // absent quad contributes no dots
        assert!(!r.state().is_visible(&q(1, 1)));
        r.state().check_invariants().unwrap();
    }

    #[test]
    fn delete_insert_where_overlap_leaves_quad_visible_with_fresh_dot() {
        // CRDT-MUT-3: same quad deleted and inserted in one operation.
        let mut r = Replica::new(1);
        let first = r.execute(&Op::InsertData(vec![q(1, 1)]));
        let env = r.execute(&Op::DeleteInsertWhere {
            delete: vec![q(1, 1)],
            insert: vec![q(1, 1)],
        });
        let old_dot = first.adds[0].1;
        let new_dot = env.adds[0].1;
        assert_ne!(old_dot, new_dot);
        assert!(env.removes[0].1.contains(&old_dot));
        assert!(!env.removes[0].1.contains(&new_dot)); // fresh dot not removed
        assert!(r.state().is_visible(&q(1, 1)));
        assert_eq!(
            r.state().store().get(&q(1, 1)),
            Some(&std::collections::BTreeSet::from([new_dot]))
        );
    }

    #[test]
    fn delete_where_predicate_compiles_against_the_origin_snapshot_only() {
        let mut a = Replica::new(1);
        let mut b = Replica::new(2);
        a.execute(&Op::InsertData(vec![q(1, 5)]));
        // B concurrently inserts another quad matching the same predicate.
        let b_env = b.execute(&Op::InsertData(vec![q(2, 5)]));
        // A's pattern delete observes only its own snapshot (CRDT-UPD-WHERE-1).
        let del = a.execute(&Op::DeleteWherePredicate(5));
        assert_eq!(del.removes.len(), 1);
        a.apply(&b_env.to_delta()).unwrap();
        // B's concurrent, unobserved add survives (CRDT-UPD-WHERE-3 example).
        assert!(a.state().is_visible(&q(2, 5)));
        assert!(!a.state().is_visible(&q(1, 5)));
    }

    #[test]
    fn execute_request_is_sequential_and_batched_delta_matches_it() {
        let mut r = Replica::new(1);
        let before = r.state().clone();
        let envs = r.execute_request(&[
            Op::InsertData(vec![q(1, 1)]),
            Op::DeleteData(vec![q(1, 1)]), // observes the dot added just above
            Op::InsertData(vec![q(2, 2)]),
        ]);
        assert_eq!(envs.len(), 3);
        assert!(!envs[1].removes.is_empty()); // later op saw the earlier effect
        let batched = Replica::batched_delta(&envs);
        assert_eq!(before.join(&batched), *r.state()); // CRDT-UPD-BATCH-1
        assert_eq!(r.state().visible(), std::collections::BTreeSet::from([q(2, 2)]));
    }

    #[test]
    fn to_delta_derivation_ignores_basis_and_covers_removed_dots() {
        let mut r = Replica::new(1);
        r.execute(&Op::InsertData(vec![q(1, 1)]));
        let env = r.execute(&Op::DeleteData(vec![q(1, 1)]));
        let delta = env.to_delta();
        assert!(delta.store().is_empty());
        assert!(delta.context().contains(Dot::new(1, 1)));
        // basis (snapshot context) is audit-only: not part of the delta context
        // beyond the dots explicitly carried as adds/removes.
        assert_eq!(delta.context().dots().len(), 1);
    }

    #[test]
    fn apply_rejects_dot_reuse_under_a_different_quad() {
        let mut r = Replica::new(1);
        let env = r.execute(&Op::InsertData(vec![q(1, 1)]));
        let mut peer = Replica::new(2);
        peer.apply(&env.to_delta()).unwrap();
        // Forge a delta reusing the same dot under another quad.
        let forged = Delta::from_parts(
            std::collections::BTreeMap::from([(
                q(7, 7),
                std::collections::BTreeSet::from([env.adds[0].1]),
            )]),
            CausalContext::from_dots([env.adds[0].1]),
        );
        let before = peer.state().clone();
        assert!(peer.apply(&forged).is_err());
        assert_eq!(*peer.state(), before); // fail closed: state unchanged
    }

    #[test]
    fn apply_absorbs_reused_removed_dot_without_reviving_it() {
        // Historical counterpart of the live-reuse rejection above: after an
        // observed removal the dot-to-quad association is gone from the store,
        // so the CRDT-WIRE-4 scan cannot reject the reuse — but the dot stays
        // in the causal context, which CRDT-JOIN-1 treats as a tombstone, so
        // the forged add joins as a no-op and never becomes visible.
        let mut r = Replica::new(1);
        let env = r.execute(&Op::InsertData(vec![q(1, 1)]));
        let removal = r.execute(&Op::DeleteData(vec![q(1, 1)]));
        let mut peer = Replica::new(2);
        peer.apply(&env.to_delta()).unwrap();
        peer.apply(&removal.to_delta()).unwrap();
        assert!(!peer.state().is_visible(&q(1, 1)));
        // Forge a delta reusing the removed dot under another quad.
        let forged = Delta::from_parts(
            std::collections::BTreeMap::from([(
                q(7, 7),
                std::collections::BTreeSet::from([env.adds[0].1]),
            )]),
            CausalContext::from_dots([env.adds[0].1]),
        );
        let before = peer.state().clone();
        peer.apply(&forged).unwrap();
        assert!(!peer.state().is_visible(&q(7, 7))); // tombstoned, not revived
        assert_eq!(*peer.state(), before); // no-op join: state unchanged
        peer.state().check_invariants().unwrap();
    }

    #[test]
    fn duplicate_apply_is_idempotent() {
        let mut a = Replica::new(1);
        let env = a.execute(&Op::InsertData(vec![q(1, 1)]));
        let mut b = Replica::new(2);
        b.apply(&env.to_delta()).unwrap();
        let once = b.state().clone();
        b.apply(&env.to_delta()).unwrap();
        assert_eq!(*b.state(), once);
    }

    #[test]
    fn replica_accessors_expose_id_and_state() {
        let r = Replica::new(9);
        assert_eq!(r.id(), 9);
        assert_eq!(*r.state(), State::bottom());
    }
}
