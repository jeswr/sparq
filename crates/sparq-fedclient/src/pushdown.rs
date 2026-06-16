//! Capability-aware pushdown (design §4.3).
//!
//! Phase 0 PLACEHOLDER — no logic yet. This module will, for each leaf and each FedX
//! exclusive group, build the MAXIMAL evaluable sub-algebra for that source's
//! `Capability` and push it: projections (only join + output variables), the FILTER
//! conjuncts the source's `pushable_filters` cover (pushing a conjunct ONLY when all its
//! variables are bound in the group — the common-variable check Comunica is documented to
//! omit), and ORDER/LIMIT when the capability allows. Bind-join across groups reuses the
//! engine's `render_values_block` + `bind_block_size` (VALUES for endpoints, `maxMpR`
//! blocks for brTPF). Anything a source cannot evaluate is kept LOCAL.
//!
//! HONEST EDGE (design §7): push only the sub-algebra whose remote semantics are
//! provably identical to local evaluation (numeric coercion / collation / language-tag /
//! datatype / NaN / timezone); when in doubt, keep it local. Pushdown only ever NARROWS
//! a source's result, so it is correctness-preserving (Phase 4).
//
// [OPUS-4.8] sq-s1uy (epic sq-dnko): Phase-0 skeleton module — populated in Phase 4.
