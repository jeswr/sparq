//! L4 — fragment-dispatch Direct-Semantics checker + entailment-by-refutation (RESERVED STUB).
//!
//! 🤖 SPARQ agent [FABLE-5]. Populated by bead sq-pbz04.4.4. This module will compose the L3
//! ALCH tableau with the EXISTING RL/EL machinery under explicit completeness guards (the RL
//! divergence guard, the EL skipped-axioms guard, QL pending) and add entailment-by-refutation,
//! returning a definitive verdict only where a branch is complete and `Unknown` otherwise —
//! never a guessed verdict. That bead ALSO owns the `dispatch` feature + re-export lines this
//! layer needs in `Cargo.toml`/`lib.rs` (design record `research/owl2-direct-semantics-scoping.md`
//! §6/§7); it is the sole bead in its wave so those serialized edits do not collide. It is
//! intentionally empty in L1.
