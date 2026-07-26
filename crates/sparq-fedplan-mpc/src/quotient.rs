//! **Phase 6 (provenance half)** — the per-source **quotient summary**, the new plan-time input
//! the FedUP-style (WWW'24) result-aware combination prune evaluates provenance over (design
//! record §3 / §8 Phase 6; bead sq-xkrt, epics sq-pwr / sq-0jsc).
//!
//! The first Phase-6 slice (sq-pwr.3, [`crate::combination`]) could only implement the
//! *unsatisfiable-conjunct collapse* rule, and **declined** any value-overlap prune, because
//! [`sparq_fedplan::SourceDescriptor`] exposes no way to reason about which *values* a source
//! actually carries — its authority test is a single-IRI predicate with no enumerator. FedUP's
//! actual lever is a different input: a **quotient summary** of the source's graph, over which
//! the query is evaluated at plan time to obtain the **provenance** (which source-combinations
//! can jointly produce a binding). This module adds that input as an explicit, *source-declared*
//! record — orthogonal to [`sparq_fedplan::SourceDescriptor`] (content) and
//! [`crate::SourcePrivacyDescriptor`] (privacy), exactly as the design record's §4.2 does for
//! privacy.
//!
//! # The quotient (the load-bearing definition)
//!
//! The quotient is the **authority quotient** `q`, the single quotient at this tier:
//!
//! * an IRI with an extractable `scheme://authority` prefix ↦ [`QuotientTerm::Authority`] of that
//!   prefix (`http://ex.org/a/b` ↦ `http://ex.org`);
//! * an IRI with no extractable authority (`urn:…`, a relative IRI) ↦
//!   [`QuotientTerm::OpaqueIri`] of the IRI verbatim — the finest class, and still a *function*;
//! * **any** literal ↦ [`QuotientTerm::Literal`], a single class. **A literal's value is never
//!   recorded in a summary** (see [`SummaryTerm::Literal`], which carries no payload) — the
//!   salary in the four-flatmates query cannot leak through a summary.
//!
//! Predicates are kept **concrete** (`q` is the identity on the predicate position), because
//! federated selection is predicate-driven and quotienting predicates to their authority would
//! make the summary too coarse to prune anything. That position-dependent quotient is why the
//! consuming pass must **decline** when a variable occurs in a predicate position in one pattern
//! and a subject/object position in another (its quotient image would be ill-defined) — see
//! [`crate::combination::SummaryDeclineReason`].
//!
//! # Why a prune over this summary is recall-safe (the soundness argument)
//!
//! Let `Σ_S` be a source `S`'s summary and `G_S` its concrete graph. Declaring a summary
//! [`complete`](SourceQuotientSummaryBuilder::complete) asserts the **over-approximation
//! property**:
//!
//! > for every concrete triple `(s, p, o) ∈ G_S`, the quotient triple `(q(s), p, q(o)) ∈ Σ_S`.
//!
//! Now suppose a source-combination `a` (one source per pattern) yields a concrete BGP solution
//! `μ`, i.e. every pattern `i` is matched by some `t_i ∈ G_{a(i)}`. Then `q(t_i) ∈ Σ_{a(i)}`, and
//! `q ∘ μ` is a well-defined assignment (because `q` is a *function*, and — given the
//! predicate-variable guard above — each variable is quotiented in one domain only) that matches
//! every quotiented pattern against `Σ_{a(i)}`. So a concrete answer implies a **summary**
//! answer; contrapositively, a combination with **no** summary answer has **no** concrete answer
//! and can be pruned without losing a result. Because the summary over-approximates, the converse
//! does **not** hold — a surviving combination is *not* promised to produce an answer, only not
//! provably barred from one.
//!
//! Everything else is recall-safe by *default*: a source that publishes **no** summary, or one
//! that does not declare it [`complete`](SourceQuotientSummaryBuilder::complete), constrains
//! nothing and is **never** pruned (the consuming pass treats it as matching anything). Declaring
//! a summary complete is therefore the source's own opt-in to being prunable, and a summary that
//! covered only part of the graph while claiming completeness would break the invariant above —
//! hence the deliberately blunt wording on [`SourceQuotientSummaryBuilder::complete`].
//!
//! # Honesty / threat model (no claim beyond a plan-time public summary)
//!
//! A quotient summary is **published metadata**, the same kind of plan-time public input as a
//! VoID descriptor — and publishing one **is a disclosure**: it reveals which *authorities* the
//! source's subjects/objects come from, per predicate. It does **not** reveal literal values
//! (collapsed to one class), individual IRIs (collapsed to their authority), or cardinalities
//! (the summary is a *set*). A source that considers even the authority-level shape sensitive
//! simply publishes nothing and is never pruned. This module performs **NO** MPC, runs **NO**
//! secret-sharing, and makes **NO** soundness/privacy/security claim; the MPC estate
//! (`sparq-mpc`) is research-grade, **honest-majority semi-honest only**, and is **NOT**
//! externally audited (sq-qhy4 / sq-9hrn pending). See the crate `README.md`.
//!
//! [OPUS-4.8] sq-xkrt.

use std::collections::BTreeSet;

use sparq_fedplan::SourceId;

/// One class of the **authority quotient** `q` — the image of a concrete RDF term.
///
/// Ordered/hashable so summaries and the derived bindings are deterministic sets. See the module
/// docs for the definition of `q` and why it is position-dependent (predicates stay concrete).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum QuotientTerm {
    /// The `scheme://authority` prefix of an IRI (e.g. `http://ex.org` for
    /// `http://ex.org/a/b`). The usual class for a global-IRI join key.
    Authority(String),
    /// An IRI with **no** extractable authority (`urn:isbn:…`, a relative IRI), quotiented to
    /// itself. Finer than [`QuotientTerm::Authority`], but still a deterministic function of the
    /// term — which is all the soundness argument needs.
    OpaqueIri(String),
    /// **Any** literal. Literals collapse to a single class: a summary never records a literal's
    /// value, so no literal can leak through one.
    Literal,
}

/// The quotient of a concrete IRI under `q` — [`QuotientTerm::Authority`] when a
/// `scheme://authority` prefix is extractable, else [`QuotientTerm::OpaqueIri`] of the IRI
/// verbatim.
///
/// The authority is the text up to (but not including) the first `/`, `?` or `#` after `://`,
/// matching the notion `sparq-fedplan`'s own HiBISCuS authority prune uses, so the two agree on
/// what "same authority" means. An empty authority (`http:///path`) is not extractable and falls
/// back to the opaque class.
pub fn quotient_iri(iri: &str) -> QuotientTerm {
    let Some(scheme_end) = iri.find("://") else {
        return QuotientTerm::OpaqueIri(iri.to_string());
    };
    let after = scheme_end + 3;
    let rest = &iri[after..];
    let host_len = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    if host_len == 0 {
        return QuotientTerm::OpaqueIri(iri.to_string());
    }
    QuotientTerm::Authority(iri[..after + host_len].to_string())
}

/// A term as *declared* to a [`SourceQuotientSummaryBuilder`]: an IRI (recorded only as its
/// quotient) or "a literal" (recorded as the single [`QuotientTerm::Literal`] class).
///
/// [`SummaryTerm::Literal`] deliberately carries **no payload**: a summary cannot record a
/// literal's value even by mistake.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SummaryTerm {
    /// A concrete IRI. Only [`quotient_iri`] of it enters the summary.
    Iri(String),
    /// A literal — any literal. Its value is **not** recorded.
    Literal,
}

impl SummaryTerm {
    /// Convenience constructor for an IRI term.
    pub fn iri(iri: impl Into<String>) -> SummaryTerm {
        SummaryTerm::Iri(iri.into())
    }

    /// This term's class under the quotient `q`.
    pub fn quotient(&self) -> QuotientTerm {
        match self {
            SummaryTerm::Iri(i) => quotient_iri(i),
            SummaryTerm::Literal => QuotientTerm::Literal,
        }
    }
}

/// One triple of a [`SourceQuotientSummary`]: subject and object quotiented under `q`, predicate
/// kept **concrete** (see the module docs).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct QuotientTriple {
    /// The subject's quotient class.
    pub subject: QuotientTerm,
    /// The concrete predicate IRI (predicates are not quotiented).
    pub predicate: String,
    /// The object's quotient class.
    pub object: QuotientTerm,
}

/// A source's **quotient summary** — the plan-time, source-declared over-approximation of its
/// graph the result-aware combination prune evaluates provenance over
/// ([`crate::prune_source_combinations_with_summaries`]).
///
/// Built with [`SourceQuotientSummary::builder`], which quotients every declared term itself, so
/// a caller cannot supply a term quotiented under some *other* function and silently break the
/// soundness argument. Default posture is **incomplete** ⇒ the summary constrains nothing and the
/// source is never pruned; [`SourceQuotientSummaryBuilder::complete`] is the source's explicit
/// opt-in.
///
/// Carries **no** secret material (literal values are never recorded) and runs **no**
/// privacy-bearing logic. Makes **no** soundness/privacy claim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceQuotientSummary {
    /// Which source this summary is for (the same stable id `sparq-fedplan` keys on).
    id: SourceId,
    /// Whether the source declares this summary a **complete over-approximation** of its graph.
    /// Only a complete summary may prune (see the module docs); `false` is the default.
    complete: bool,
    /// The quotient triples, deduplicated and ordered (the seam is deterministic throughout).
    triples: BTreeSet<QuotientTriple>,
}

impl SourceQuotientSummary {
    /// Starts a builder for the source named `id`. The summary starts **empty and incomplete** —
    /// i.e. it prunes nothing until the source both declares triples and calls
    /// [`SourceQuotientSummaryBuilder::complete`].
    pub fn builder(id: SourceId) -> SourceQuotientSummaryBuilder {
        SourceQuotientSummaryBuilder {
            id,
            complete: false,
            triples: BTreeSet::new(),
        }
    }

    /// The source this summary describes.
    pub fn id(&self) -> &SourceId {
        &self.id
    }

    /// Whether the source declared this summary a complete over-approximation of its graph. Only
    /// a complete summary is allowed to prune a source-combination.
    pub fn is_complete(&self) -> bool {
        self.complete
    }

    /// The quotient triples, ascending (deterministic).
    pub fn triples(&self) -> &BTreeSet<QuotientTriple> {
        &self.triples
    }

    /// How many distinct quotient triples the summary declares.
    pub fn len(&self) -> usize {
        self.triples.len()
    }

    /// Whether the summary declares no quotient triple at all. NOTE: an **empty** summary that is
    /// also [`complete`](SourceQuotientSummary::is_complete) is a declaration that the source's
    /// graph is empty — it prunes every combination it appears in. An empty *incomplete* summary
    /// (the default) prunes nothing.
    pub fn is_empty(&self) -> bool {
        self.triples.is_empty()
    }
}

/// Builder for a [`SourceQuotientSummary`]. It quotients each declared term itself — the only
/// construction path, so every summary in the system is expressed under the *same* quotient `q`.
#[derive(Debug, Clone)]
pub struct SourceQuotientSummaryBuilder {
    id: SourceId,
    complete: bool,
    triples: BTreeSet<QuotientTriple>,
}

impl SourceQuotientSummaryBuilder {
    /// Records that the source's graph contains a triple with these terms. Only the *quotient* of
    /// `subject` / `object` is stored (and, for a literal, only the fact that it is one); the
    /// predicate is stored concretely.
    ///
    /// Declaring the same quotient triple twice is a no-op — the summary is a set.
    pub fn triple(
        mut self,
        subject: SummaryTerm,
        predicate: impl Into<String>,
        object: SummaryTerm,
    ) -> Self {
        self.triples.insert(QuotientTriple {
            subject: subject.quotient(),
            predicate: predicate.into(),
            object: object.quotient(),
        });
        self
    }

    /// Declares this summary a **complete over-approximation of the source's ENTIRE graph** —
    /// every triple the source can answer with, private ones included, has had its quotient
    /// declared here. This is what makes the source prunable.
    ///
    /// Do **not** call this for a summary built over only part of the graph (e.g. only the
    /// public subgraph, or only the triples matching one predicate): the combination prune's
    /// recall-safety rests on the over-approximation property stated in the module docs, and an
    /// under-approximating "complete" summary would let it drop a combination that really does
    /// produce answers. Omitting the call is always safe — an incomplete summary simply never
    /// prunes.
    pub fn complete(mut self) -> Self {
        self.complete = true;
        self
    }

    /// Finalises the summary.
    pub fn build(self) -> SourceQuotientSummary {
        SourceQuotientSummary {
            id: self.id,
            complete: self.complete,
            triples: self.triples,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authority_quotient_collapses_a_namespace() {
        // Two IRIs under the same authority land in the SAME class; a different authority does
        // not. This collapse is the whole reason a summary is small.
        assert_eq!(
            quotient_iri("http://ex.org/a/b"),
            QuotientTerm::Authority("http://ex.org".to_string())
        );
        assert_eq!(
            quotient_iri("http://ex.org/other#frag"),
            QuotientTerm::Authority("http://ex.org".to_string())
        );
        assert_ne!(quotient_iri("http://ex.org/a"), quotient_iri("http://b.org/a"));
        // `?` and `#` delimit the authority just like `/` (matching fedplan's authority notion).
        assert_eq!(
            quotient_iri("http://h.example?q=1"),
            QuotientTerm::Authority("http://h.example".to_string())
        );
        // No path at all ⇒ the whole IRI is the authority.
        assert_eq!(
            quotient_iri("https://example.org"),
            QuotientTerm::Authority("https://example.org".to_string())
        );
    }

    #[test]
    fn unextractable_authority_falls_back_to_the_opaque_class() {
        // A `urn:` IRI and an empty-authority IRI have no extractable authority: they quotient to
        // themselves. Still a function (the only thing soundness needs), just a finer class.
        assert_eq!(
            quotient_iri("urn:isbn:123"),
            QuotientTerm::OpaqueIri("urn:isbn:123".to_string())
        );
        assert_eq!(
            quotient_iri("http:///path"),
            QuotientTerm::OpaqueIri("http:///path".to_string())
        );
        // Two distinct urns stay distinct.
        assert_ne!(quotient_iri("urn:a"), quotient_iri("urn:b"));
    }

    #[test]
    fn literal_values_never_enter_a_summary() {
        // The load-bearing privacy property of the summary input: a literal contributes only the
        // single `Literal` class, so no value (a salary, a name) can be read back out of it.
        assert_eq!(SummaryTerm::Literal.quotient(), QuotientTerm::Literal);
        let s = SourceQuotientSummary::builder(SourceId::new("http://a/"))
            .triple(
                SummaryTerm::iri("http://ex.org/alice"),
                "http://ex/owes",
                SummaryTerm::Literal,
            )
            .build();
        let t = s.triples().iter().next().expect("one triple");
        assert_eq!(t.object, QuotientTerm::Literal);
        // Nothing in the rendered summary mentions a value — there is none to mention.
        assert!(!format!("{:?}", s).contains("100000"));
    }

    #[test]
    fn summary_is_a_deduplicating_set_and_defaults_incomplete() {
        // Same quotient triple declared twice (two concrete IRIs under one authority) ⇒ one entry.
        let s = SourceQuotientSummary::builder(SourceId::new("http://a/"))
            .triple(
                SummaryTerm::iri("http://ex.org/alice"),
                "http://ex/name",
                SummaryTerm::Literal,
            )
            .triple(
                SummaryTerm::iri("http://ex.org/bob"),
                "http://ex/name",
                SummaryTerm::Literal,
            )
            .build();
        assert_eq!(s.len(), 1, "both subjects collapse to one authority class");
        assert!(!s.is_empty());
        // Default posture is INCOMPLETE — a summary prunes nothing until the source opts in.
        assert!(!s.is_complete());
        assert_eq!(s.id(), &SourceId::new("http://a/"));

        let c = SourceQuotientSummary::builder(SourceId::new("http://a/"))
            .complete()
            .build();
        assert!(c.is_complete());
        assert!(c.is_empty(), "a complete-but-empty summary declares an empty graph");
    }

    #[test]
    fn builder_quotients_terms_itself_so_summaries_share_one_quotient() {
        // The builder is the ONLY construction path and applies `q` itself: a caller cannot inject
        // a term quotiented under some other function. Declaring the concrete IRI yields exactly
        // `quotient_iri` of it.
        let s = SourceQuotientSummary::builder(SourceId::new("http://a/"))
            .triple(
                SummaryTerm::iri("http://ex.org/alice"),
                "http://ex/knows",
                SummaryTerm::iri("urn:x:bob"),
            )
            .build();
        let t = s.triples().iter().next().expect("one triple");
        assert_eq!(t.subject, quotient_iri("http://ex.org/alice"));
        assert_eq!(t.object, quotient_iri("urn:x:bob"));
        assert_eq!(t.predicate, "http://ex/knows", "predicates stay concrete");
    }
}
