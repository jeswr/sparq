//! Conservative write-footprint analysis for commutativity batching
//! (research/concurrent-serving.md §6.5; litreview A §A.2(c)).
//!
//! Under RDF **set semantics**, `INSERT DATA` / `DELETE DATA` operations commute
//! whenever they cannot touch the same triple with opposite polarity: inserting
//! two (even identical) triple sets is order-insensitive (set union), deleting
//! two is order-insensitive (set difference), and an insert and a delete commute
//! when their triple sets are disjoint. Proving triple-level disjointness per
//! pair is exactly the per-update work the design rejects, so the detection here
//! is **graph-level and conservative**, per the wave plan's rule
//! ("same-named-graph or unanalyzable update = barrier"), refined one sound
//! notch: two data updates conflict iff one *inserts into* a graph the other
//! *deletes from* (opposite polarity on a shared graph — the only graph-level
//! situation where the same triple could be both inserted and deleted, making
//! order observable). Same-graph same-polarity updates commute outright, which
//! keeps the bulk-ingest-into-one-pod stream batchable. Anything that is not a
//! pure data update — `DELETE/INSERT … WHERE`, `DROP`, `CLEAR`, `LOAD`,
//! `CREATE`, or text this parser rejects — has an unanalyzable (or
//! order-sensitive) footprint and is a **barrier**: it commutes with nothing.
//!
//! The footprint is derived from the update **text** (never trusted from the
//! caller — the caller-supplied `touched` pods drive *epoch bumps*, a cache
//! contract; commutativity is a correctness license and must come from the
//! update itself). Parsing costs ~2 µs for prod-solid-server-sized updates
//! (§1.2) on the writer thread, and is only paid when the writer is configured
//! with [`CommitGranularity::CommuteGroup`](crate::CommitGranularity).

use std::sync::Arc;

use rustc_hash::FxHashSet;
use spargebra::term::GraphName;
use spargebra::{GraphUpdateOperation, SparqlParser};

/// A write-target graph slot: the default graph or a named graph (by IRI).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum TargetGraph {
    /// The dataset's default graph.
    Default,
    /// A named graph, by IRI (in the Solid profile: the pod resource graph).
    Named(Arc<str>),
}

impl From<&GraphName> for TargetGraph {
    fn from(g: &GraphName) -> Self {
        match g {
            GraphName::DefaultGraph => TargetGraph::Default,
            GraphName::NamedNode(n) => TargetGraph::Named(Arc::from(n.as_str())),
        }
    }
}

/// The statically derived write footprint of one update — the conflict tag of
/// §6.5, computed instead of declared.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Footprint {
    /// A pure data update (every operation `INSERT DATA` or `DELETE DATA`):
    /// graph-level insert/delete write sets, statically known from the text.
    Data {
        /// Graphs some `INSERT DATA` operation writes into.
        inserts: FxHashSet<TargetGraph>,
        /// Graphs some `DELETE DATA` operation deletes from.
        deletes: FxHashSet<TargetGraph>,
    },
    /// Everything else (`WHERE`-driven updates, graph management, `LOAD`,
    /// unparseable text): unanalyzable or order-sensitive — commutes with
    /// nothing, sequencing barrier.
    Barrier,
}

impl Footprint {
    /// Derives the footprint of a SPARQL Update string. Text the parser rejects
    /// is conservatively a [`Footprint::Barrier`] — classification never
    /// *rejects* an update (the engine's own parser stays the authority; it will
    /// fail the update at apply time with its own error).
    pub fn of_sparql(update: &str) -> Footprint {
        let parsed = match SparqlParser::new().parse_update(update) {
            Ok(u) => u,
            Err(_) => return Footprint::Barrier,
        };
        let mut inserts = FxHashSet::default();
        let mut deletes = FxHashSet::default();
        for op in &parsed.operations {
            match op {
                GraphUpdateOperation::InsertData { data } => {
                    inserts.extend(data.iter().map(|q| TargetGraph::from(&q.graph_name)));
                }
                GraphUpdateOperation::DeleteData { data } => {
                    deletes.extend(data.iter().map(|q| TargetGraph::from(&q.graph_name)));
                }
                _ => return Footprint::Barrier,
            }
        }
        Footprint::Data { inserts, deletes }
    }

    /// Whether two updates provably commute under the conservative graph-level
    /// rule (module docs): both are pure data updates, and neither inserts into
    /// a graph the other deletes from. Symmetric. A [`Footprint::Barrier`]
    /// commutes with nothing — not even another barrier.
    pub fn commutes_with(&self, other: &Footprint) -> bool {
        match (self, other) {
            (
                Footprint::Data { inserts: i1, deletes: d1 },
                Footprint::Data { inserts: i2, deletes: d2 },
            ) => i1.is_disjoint(d2) && d1.is_disjoint(i2),
            _ => false,
        }
    }

    /// True for [`Footprint::Barrier`].
    pub fn is_barrier(&self) -> bool {
        matches!(self, Footprint::Barrier)
    }
}
