//! In-circuit RDFS entailment derivation-step proof (`entail_rdfs_k{K}_n{N}`
//! family, sq-g91d; the PRIVACY upgrade for the `Rdfs`/`Owl` regimes over the
//! host-side disclosed-base re-check `crate::verifier::bind_entailment`).
//!
//! ## Why this relation exists (the sq-314 privacy gap it closes)
//! `bind_entailment` (host, sq-314) makes the manifest's `entailment_regime`
//! ENFORCED, but it grounds each derivation step's antecedents ONLY against the
//! DISCLOSED scan base: an antecedent is accepted iff it byte-equals a disclosed
//! scan row (or an earlier derived triple). So an entailed presentation must
//! DISCLOSE every antecedent triple, defeating privacy for inference-bearing
//! presentations (the section-5 inference goal). This relation removes that
//! disclosure: it proves a derived triple follows by ONE RDFS rule application from
//! antecedents that are members of the COMMITTED graph -- WITHOUT disclosing the
//! antecedents -- bound to the SAME per-graph commitments the scan sub-proofs
//! carry.
//!
//! ## Statement (single RDFS rule step; EXISTENCE / disclosed-base scope)
//! For the disclosed rule `rule` over the union of `K` committed graphs
//! `commitments` (the same flat Poseidon2 commitments `scan_check` recomputes),
//! with the public schema-vocabulary encodings `rdf_type` / `rdfs_subclassof` /
//! `rdfs_subpropertyof` (which the verifier pins to the real IRIs' term
//! encodings) and the PUBLIC `derived` triple:
//!
//! > There exist antecedent triples `(a_0, a_1)`, each a member of some
//! > committed graph in the attested source set, such that `(a_0, a_1)` is a
//! > well-formed instance of `rule` whose consequent is exactly `derived`.
//!
//! The antecedents are PRIVATE -- the verifier learns only the derived triple
//! and that it is RDFS-entailed by the committed (undisclosed) base. This is the
//! in-circuit analogue of `DerivationStep::is_well_formed` +
//! `bind_entailment`'s grounding, with the grounding moved from
//! disclosed-row-equality to in-circuit commitment membership.
//!
//! ## SCOPE -- what is and is NOT proved (load-bearing honesty)
//! - SINGLE STEP: exactly one rule application. Multi-hop RDFS closure (chaining
//!   a derived triple as a later step's antecedent) requires the intermediate to
//!   be committed too, which it need not be; the full fixpoint-SATURATION /
//!   completeness-under-entailment proof is a SEPARATE, UNBUILT deliverable and
//!   is NOT claimed here.
//! - DISCLOSED-BASE MEMBERSHIP: an antecedent is grounded by membership in a
//!   committed graph (the asserted base is a subset of the closure). An
//!   antecedent that is itself only entailed (in the closure but not the
//!   asserted base) is NOT provable by this relation -- fail-closed, exactly
//!   like the host `bind_entailment`.
//! - v1 RULES: the two binary RDFS rules `rdfs9` (subClassOf-into-type) and
//!   `rdfs7` (subPropertyOf), the same pair `EntailmentRule` ships. Both have
//!   exactly two antecedents, so `antecedents` is fixed arity 2.
//!
//! ## NOT a soundness claim
//! This module BUILDS a circuit; it does not make the composition verifier
//! sound. The verifier stack is internally re-audited but NOT externally audited
//! (`sq-qhy4` pending); nothing here is a proven cryptographic guarantee. The
//! compiled `entail_rdfs_k{K}_n{N}` bin member (with its gate-count baseline) and
//! the verifier-side manifest/dispatch/public-input wiring (the sibling of the
//! `path_reach` sq-3kd2g.6 wiring) are follow-up beads.

use crate::hashes::{commit_fold, h3};

/// RDFS rule selector -- the PUBLIC `rule` input value. Mirrors
/// `crate::manifest`/`crate::derivation::EntailmentRule` ordering.
pub global RULE_RDFS9_SUBCLASS_TYPE: u32 = 0; // (x type c1),(c1 subClassOf c2) |- (x type c2)
pub global RULE_RDFS7_SUBPROPERTY: u32 = 1; //  (p1 subPropertyOf p2),(x p1 y) |- (x p2 y)

/// The generic single-step RDFS entailment relation; each
/// `entail_rdfs_k{K}_n{N}` bin package is one monomorphisation. See the module
/// docs for the statement and scope.
///
/// Public inputs (declaration order -- the verifier reconstructs the
/// public-input vector in this order): `commitments`, `rule`, `rdf_type`,
/// `rdfs_subclassof`, `rdfs_subpropertyof`, `derived`, `attribution`. (`main`
/// prepends `challenge`, unconstrained in the relation, exactly as every other
/// member.) Private witnesses: `antecedents`, `counts`, `enc`.
///
/// `commitments` MUST be strictly increasing on the field representative
/// (`commitments[0] < commitments[1] < ...`, plan S2.5 step 1b) -- the same
/// distinct-graph guard `scan_check` enforces, forcing the `K` graphs pairwise
/// distinct.
pub fn entail_step_check<let K: u32, let N: u32>(
    commitments: [Field; K],
    rule: u32,
    rdf_type: Field,
    rdfs_subclassof: Field,
    rdfs_subpropertyof: Field,
    derived: [Field; 3],
    attribution: [bool; K],
    antecedents: [[Field; 3]; 2],
    counts: [u32; K],
    enc: [[[Field; 3]; N]; K],
) {
    // 1. Commitment recompute (binds every active slot's encodings to the PUBLIC
    //    per-graph commitments -- identical to `scan_check` step 1). Without this
    //    the witnessed `enc` would be free and antecedent membership vacuous.
    for g in 0..K {
        assert(counts[g] <= N, "graph size exceeds circuit bucket");
        let mut leaves: [Field; N] = [0; N];
        for i in 0..N {
            leaves[i] = h3(enc[g][i][0], enc[g][i][1], enc[g][i][2]);
        }
        assert_eq(commit_fold(leaves, counts[g]), commitments[g], "graph commitment mismatch");
    }

    // 1b. Distinct-graph strict ordering (same guard as `scan_check` step 1b):
    //     pairwise-distinct committed graphs. Empty for K=1.
    for g in 1..K {
        assert(
            commitments[g - 1].lt(commitments[g]),
            "per-graph commitments not strictly increasing (duplicate-inclusion / COUNT forgery)",
        );
    }

    // 2. Rule well-formedness (the in-circuit analogue of
    //    `DerivationStep::is_well_formed`). The disclosed `rule` selects WHICH
    //    variable-sharing/consequent shape the antecedents + `derived` must
    //    satisfy; the schema predicates (`rdf_type` etc.) are PUBLIC so the
    //    verifier pins them to the true IRI encodings. Positions are [s, p, o].
    let a0 = antecedents[0];
    let a1 = antecedents[1];

    let is9 = rule == RULE_RDFS9_SUBCLASS_TYPE;
    let is7 = rule == RULE_RDFS7_SUBPROPERTY;
    assert(is9 | is7, "unknown entailment rule");

    // rdfs9: a0=(x, type, c1), a1=(c1, subClassOf, c2) |- derived=(x, type, c2).
    let rdfs9_ok = (a0[1] == rdf_type)
        & (a1[1] == rdfs_subclassof)
        & (a0[2] == a1[0]) //          shared class c1
        & (derived[0] == a0[0]) //     x
        & (derived[1] == rdf_type)
        & (derived[2] == a1[2]); //    c2

    // rdfs7: a0=(p1, subPropertyOf, p2), a1=(x, p1, y) |- derived=(x, p2, y).
    let rdfs7_ok = (a0[1] == rdfs_subpropertyof)
        & (a0[0] == a1[1]) //          shared property p1
        & (derived[0] == a1[0]) //     x
        & (derived[1] == a0[2]) //     p2
        & (derived[2] == a1[2]); //    y

    // Exactly the disclosed rule's shape must hold (is9/is7 are mutually
    // exclusive, so at most one disjunct is live).
    assert(
        (is9 & rdfs9_ok) | (is7 & rdfs7_ok),
        "derivation step is not a valid instance of its rule",
    );

    // 3. Antecedent membership (the PRIVACY step): every antecedent must occur in
    //    some active slot of some committed graph -- the same union row-membership
    //    relation `scan_check` uses, but the antecedent is a PRIVATE witness, so
    //    the proof grounds it WITHOUT disclosing it. `contrib[g]` accumulates the
    //    per-graph source attribution alongside the sweep. Both antecedents are
    //    ACTIVE (a rule always has both), so membership is unconditional.
    let mut contrib: [bool; K] = [false; K];
    for ai in 0..2 {
        let ant = antecedents[ai];
        let mut present = false;
        for g in 0..K {
            let mut present_g = false;
            for i in 0..N {
                let slot_active = i < counts[g];
                let eq =
                    (enc[g][i][0] == ant[0]) & (enc[g][i][1] == ant[1]) & (enc[g][i][2] == ant[2]);
                present_g |= slot_active & eq;
            }
            contrib[g] |= present_g;
            present |= present_g;
        }
        assert(present, "antecedent not a member of any committed graph");
    }

    // 4. Source attribution: `attribution[g]` (PUBLIC) must equal "graph `g`
    //    contributes an antecedent to the witnessed derivation", in BOTH
    //    directions (a prover can neither hide nor invent a contributing graph) --
    //    the same exact-set discipline as `scan_check` step 4. It is
    //    DERIVATION-RELATIVE: it attests the source graphs of the antecedents the
    //    prover chose, exactly as `path_reach`'s attribution is chain-relative.
    for g in 0..K {
        assert(
            attribution[g] == contrib[g],
            "attribution does not match the proved source graphs",
        );
    }
}
