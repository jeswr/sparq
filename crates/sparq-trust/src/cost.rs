//! # `cost.rs` — the P8 cost / decidability spike (`sq-pfae.9`)
//!
//! The design record `research/solid-trust-graph-authz-design.md` closes §7.1 C′ with
//! *"**No formal complexity bound is proven here**"* and defers to P8 two obligations:
//!
//! 1. **Bound admission-rule evaluation cost.**
//! 2. **Confirm every seeding direction is one-side-bound** — the war story being a
//!    *two-unbound-atom* rule (an atom whose subject **and** object are both unbound,
//!    so the join enumerates a whole predicate extent) that blew up seeding.
//!
//! This module discharges both **mechanically and deterministically**. Nothing here
//! samples a clock, an allocator, or the host: work-box / EC2 wall-clock timings are
//! **non-canonical** in this repository and are deliberately *not measured at all*. The
//! metrics below are operation **counts** and a **static** analysis of rule text, both
//! of which are reproducible byte-for-byte on any machine — so they are the only kind of
//! metric that may be gated in CI.
//!
//! ## 1. The admission-gate cost bound (proved by construction, checked by test)
//!
//! [`crate::admit::admit`] is a **single pass**, not a fixpoint: an admitted fact is
//! never fed back into the rule set, and no rule can derive another rule. Its whole cost
//! is therefore the literal loop nest — `for rule in rules { for triple in graph { … } }`
//! — with every per-step check short-circuiting. Writing `R = rules.len()` and
//! `T = graph.len()`, the exact worst case is
//!
//! ```text
//!   commit(G)                   ≤ 1
//!   parse(signature)            ≤ 1
//!   scope_covers                ≤ R
//!   verify(issuer signature)    ≤ R
//!   freshness                   ≤ R
//!   revocation guard            ≤ R
//!   reserved-predicate guard    ≤ R·T
//!   SHACL statement-type check  ≤ R·T
//!   holder binding              ≤ R·T
//!   ────────────────────────────────────
//!   total gate operations       ≤ 2 + 4R + 3R·T          i.e.  Θ(R·T)
//! ```
//!
//! [`admission_cost_bound`] is that closed form; [`admit_measured`] runs the **real,
//! unmodified gate** (the meter is threaded through the shipped code path — there is no
//! `cfg` fork) and returns the counts it actually incurred. The bound is **tight**: a
//! credential whose every triple passes every rule attains it exactly, which is what the
//! acceptance test pins from both sides (measured never exceeds the bound; the saturated
//! fixture *equals* it), so the bound can be neither inflated nor deflated unnoticed.
//!
//! Each SHACL statement-type check is itself one run of the shipped, **terminating**
//! `sparq-shacl` validator over `G` — a side condition, never recursive rule expansion —
//! so termination of the gate needs no further argument. This is a bound on **gate
//! operations**, not on elementary machine steps, and it is emphatically **not** a
//! statement about latency.
//!
//! ## 2. Seeding-direction analysis (the one-side-bound obligation)
//!
//! The rules that reach the N3 evaluator on the admission path are the
//! controller-authored `.acr` ABAC rules [`crate::wire::derive_grants`] runs. An atom is
//! **one-side-bound** when its *subject or object* is a constant or a variable already
//! bound by an earlier-joined atom — i.e. the join is driven from a bound side rather
//! than by scanning a whole extent. [`analyse_seeding`] parses a Notation3 document and,
//! per rule, greedily picks a join order (lowest atom index first, so the answer is
//! deterministic) in which every atom is one-side-bound; any atom no order can bind that
//! way is reported as [`UnseededAtom`], classified as
//! [`UnseededKind::PredicateAnchored`] (the two-unbound-atom shape — bounded only by the
//! predicate's extent) or [`UnseededKind::Unanchored`] (`?s ?p ?o` — bounded by nothing).
//! It also reports **range-restriction** failures: a head variable the body never binds
//! ([`RuleSeeding::unsafe_head_vars`]), the unsafe `{ ?x p ?x }`-style formula the
//! `crate::admissibility` module docs already warn about.
//!
//! [`require_one_side_bound`] is the **fail-closed guard** a caller should run over any
//! `.acr` rule text before handing it to `derive_grants`: it errors on an unseedable
//! atom, on an unsafe head, **and** on anything it cannot parse. The parser is
//! deliberately conservative — blank-node property lists, collections, nested formulae
//! and reverse implication (`<=`) are all [`SeedingError`]s rather than silent passes.
//!
//! ### The honest finding this spike records
//!
//! Not every ruleset in this crate is one-side-bound, and P8 is where that is stated
//! rather than assumed. The `.acr` ABAC rule of design §3.1
//! (`{ ?x schema:age ?y . ?y math:greaterThan 18 }`) **is**, and so is the
//! `crate::admissibility` discharge rule (its `odrl:gteq` constant object seeds it). The
//! `crate::admissibility` **transitive-closure** rule
//! (`{ ?a secx:strongerThan ?b . ?b secx:strongerThan ?c }`) is **not** — it is a
//! genuine two-unbound-atom seed. It is safe for a different reason, which this module
//! does not launder into a bound it has not proven: its extent is a **closed, bundled,
//! constant** fact base (`crate::admissibility::LEVEL_ORDERS`), not an external graph, so
//! its closure is bounded by that constant rather than by anything an attacker supplies.
//!
//! ## Scope — what is NOT bounded here
//!
//! `admit_static`, `admit_with_status`, the certification-edge closure (`graph`), and the
//! trust-expression evaluator (`expression`) are out of scope for this module; only
//! [`crate::admit::admit`], the gate P8 names, is metered. No claim is made about
//! `sparq-reason`'s own evaluation cost — [`analyse_seeding`] analyses *rule shape*, not
//! the reasoner. And none of this is a security, soundness, or privacy property: it is a
//! termination/cost argument over a research prototype whose ZK estate remains externally
//! unaudited (`sq-qhy4`).
//!
//! ## Opt-in by construction
//!
//! Behind the **default-OFF `cost-bound`** cargo feature. It is pure `std` arithmetic and
//! string analysis — **no new dependency** — and the lean default build gains nothing but
//! the meter increments the gate already carries.
//!
//! [SONNET-4.6] sq-pfae.9 (epic sq-pfae, issue #3281). 🤖 SPARQ agent — trust-graph
//! authorisation PoC.

use crate::admit::{admit_metered, AdmissionMeter, AdmittedFact, PresentedCredential, Session};
use crate::policy::TrustRule;
use oxrdf::NamedNode;
use std::collections::BTreeSet;

// ── 1. The admission-gate cost bound ──────────────────────────────────────────

/// The input size of one admission-gate evaluation: the number of trust rules and the
/// number of triples in the presented credential graph. These are the **only** two
/// quantities the gate's cost depends on (module docs).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdmissionShape {
    /// `R` — the number of trust rules the gate is evaluated against.
    pub rules: usize,
    /// `T` — the number of triples in the presented credential graph `G`.
    pub graph_triples: usize,
}

/// A deterministic count of the operations one admission-gate evaluation performed (or,
/// from [`admission_cost_bound`], the worst case it *could* perform).
///
/// Every field is an operation count. There is deliberately **no timing, duration,
/// allocation or host field**: work-box / EC2 wall-clock numbers are non-canonical in
/// this repository, so only counts like these may be gated.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AdmissionCost {
    /// RDFC-1.0 canonicalise + commit of the credential graph `G`.
    pub graph_commitments: usize,
    /// Issuer-signature hex parses.
    pub signature_parses: usize,
    /// `trust:scope` containment checks.
    pub scope_checks: usize,
    /// Schnorr verifications of the issuer signature over `C(G)`.
    pub signature_verifications: usize,
    /// Per-request freshness comparisons.
    pub freshness_checks: usize,
    /// Input-stratified revocation-guard reads (`sq-tu4e`).
    pub revocation_checks: usize,
    /// Reserved-predicate guard evaluations.
    pub reserved_predicate_checks: usize,
    /// `sparq-shacl` statement-type validations.
    pub shape_validations: usize,
    /// Holder-binding comparisons.
    pub holder_checks: usize,
}

impl AdmissionCost {
    /// The total number of gate operations (the sum of every field), saturating.
    #[must_use]
    pub fn total(&self) -> usize {
        self.graph_commitments
            .saturating_add(self.signature_parses)
            .saturating_add(self.scope_checks)
            .saturating_add(self.signature_verifications)
            .saturating_add(self.freshness_checks)
            .saturating_add(self.revocation_checks)
            .saturating_add(self.reserved_predicate_checks)
            .saturating_add(self.shape_validations)
            .saturating_add(self.holder_checks)
    }

    /// Whether `self` dominates `other` **componentwise** (`self[i] >= other[i]` for
    /// every field) — the relation a bound must have to a measurement. Componentwise,
    /// not merely on [`total`](Self::total): a bound that were slack in one field and
    /// tight in another would otherwise pass while hiding a real overrun.
    #[must_use]
    pub fn dominates(&self, other: &Self) -> bool {
        self.graph_commitments >= other.graph_commitments
            && self.signature_parses >= other.signature_parses
            && self.scope_checks >= other.scope_checks
            && self.signature_verifications >= other.signature_verifications
            && self.freshness_checks >= other.freshness_checks
            && self.revocation_checks >= other.revocation_checks
            && self.reserved_predicate_checks >= other.reserved_predicate_checks
            && self.shape_validations >= other.shape_validations
            && self.holder_checks >= other.holder_checks
    }
}

impl From<AdmissionMeter> for AdmissionCost {
    fn from(m: AdmissionMeter) -> Self {
        // Exhaustive destructuring on purpose: adding a counter to the gate's meter
        // without surfacing it here is then a COMPILE error, not a silently missing
        // dimension in the bound.
        let AdmissionMeter {
            graph_commitments,
            signature_parses,
            scope_checks,
            signature_verifications,
            freshness_checks,
            revocation_checks,
            reserved_predicate_checks,
            shape_validations,
            holder_checks,
        } = m;
        Self {
            graph_commitments,
            signature_parses,
            scope_checks,
            signature_verifications,
            freshness_checks,
            revocation_checks,
            reserved_predicate_checks,
            shape_validations,
            holder_checks,
        }
    }
}

/// The **closed-form worst case** for an admission-gate evaluation of the given shape —
/// the `2 + 4R + 3R·T` bound of the module docs, per dimension.
///
/// Every arithmetic step saturates, so an absurd `shape` yields `usize::MAX` rather than
/// overflowing (a bound that saturates still dominates any real measurement, because a
/// real measurement cannot exceed `usize::MAX` either).
///
/// The bound is **tight**: a credential whose every triple passes every rule attains it
/// exactly.
#[must_use]
pub fn admission_cost_bound(shape: AdmissionShape) -> AdmissionCost {
    let AdmissionShape {
        rules,
        graph_triples,
    } = shape;
    let pairs = rules.saturating_mul(graph_triples);
    AdmissionCost {
        graph_commitments: 1,
        signature_parses: 1,
        scope_checks: rules,
        signature_verifications: rules,
        freshness_checks: rules,
        revocation_checks: rules,
        reserved_predicate_checks: pairs,
        shape_validations: pairs,
        holder_checks: pairs,
    }
}

/// Run the **unmodified** admission gate and return its result alongside the
/// deterministic [`AdmissionCost`] it incurred.
///
/// This is [`crate::admit::admit`] with the meter it already threads read out — the same
/// function, the same order of checks, the same fail-closed behaviour. It exists so a
/// test can compare a real evaluation against [`admission_cost_bound`]; it is **not** a
/// second admission path and must never be given different semantics.
#[must_use]
pub fn admit_measured(
    cred: &PresentedCredential,
    rules: &[TrustRule],
    session: &Session,
    target: &NamedNode,
) -> (Vec<AdmittedFact>, AdmissionCost) {
    let mut meter = AdmissionMeter::default();
    let admitted = admit_metered(cred, rules, session, target, &mut meter);
    (admitted, AdmissionCost::from(meter))
}

// ── 2. Seeding-direction analysis ─────────────────────────────────────────────

/// Why a body atom could not be reached by any one-side-bound join order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnseededKind {
    /// Subject and object are both unbound variables but the predicate is a constant —
    /// the **two-unbound-atom** shape. Seeding it enumerates that predicate's whole
    /// extent, which is the blow-up the design record's war story records.
    PredicateAnchored,
    /// Nothing in the atom is bound (`?s ?p ?o`) — bounded by no extent at all.
    Unanchored,
}

/// A body atom of a rule that no one-side-bound join order can reach.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnseededAtom {
    /// The atom's index within the rule body, counting `;`/`,` expansions in order.
    pub atom_index: usize,
    /// The atom rendered as `subject predicate object` (its parsed terms, space-joined).
    pub atom: String,
    /// Why it is unseedable.
    pub kind: UnseededKind,
}

/// The seeding analysis of one `{ body } => { head }` rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleSeeding {
    /// The rule's index within the analysed document (0-based, textual order).
    pub rule_index: usize,
    /// The number of body atoms (after `;`/`,` expansion).
    pub body_atoms: usize,
    /// The chosen join order as body-atom indices: the greedy one-side-bound order,
    /// with any unseedable atoms appended in index order. Deterministic.
    pub seeding_order: Vec<usize>,
    /// Body atoms no one-side-bound order can reach. Empty iff the rule is fully
    /// one-side-bound.
    pub unseeded: Vec<UnseededAtom>,
    /// Head variables the body never binds — a range-restriction (safety) failure, sorted.
    pub unsafe_head_vars: Vec<String>,
}

impl RuleSeeding {
    /// Whether every body atom is one-side-bound **and** the head is range-restricted.
    #[must_use]
    pub fn is_one_side_bound(&self) -> bool {
        self.unseeded.is_empty() && self.unsafe_head_vars.is_empty()
    }
}

/// The seeding analysis of a whole Notation3 document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeedingReport {
    /// One entry per `=>` rule found, in textual order. Ground facts and `@prefix`
    /// declarations are not rules and are skipped.
    pub rules: Vec<RuleSeeding>,
}

impl SeedingReport {
    /// Whether **every** rule in the document is one-side-bound and range-restricted.
    /// A document with no rules at all is vacuously `true` — callers that require a
    /// rule to be present must check [`rules`](Self::rules) themselves.
    #[must_use]
    pub fn all_one_side_bound(&self) -> bool {
        self.rules.iter().all(RuleSeeding::is_one_side_bound)
    }

    /// The rules that are not one-side-bound (or not range-restricted), in order.
    #[must_use]
    pub fn violations(&self) -> Vec<&RuleSeeding> {
        self.rules
            .iter()
            .filter(|r| !r.is_one_side_bound())
            .collect()
    }
}

/// Why a Notation3 document could not be analysed. Every variant is a **refusal to
/// decide**, never a pass: the analysis is conservative by construction, so a caller
/// using [`require_one_side_bound`] fails closed on anything unsupported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeedingError {
    /// A construct the conservative parser does not model: a blank-node property list
    /// (`[ … ]`), a collection (`( … )`), a nested formula inside a rule, reverse
    /// implication (`<=`), or a stray token. Carries a human-readable description.
    Unsupported(String),
    /// A rule body or head whose statements do not resolve to exactly three terms each.
    MalformedAtom(String),
    /// An unterminated IRI, string literal, or formula.
    Unterminated(String),
}

impl std::fmt::Display for SeedingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported(m) => write!(f, "unsupported N3 construct: {}", m),
            Self::MalformedAtom(m) => write!(f, "malformed atom: {}", m),
            Self::Unterminated(m) => write!(f, "unterminated {}", m),
        }
    }
}

impl std::error::Error for SeedingError {}

/// Why [`require_one_side_bound`] denied a document. Both variants are denials: a
/// document that cannot be analysed is refused exactly as one with a real violation is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeedingDenial {
    /// The document parsed, but some rule is not one-side-bound / not range-restricted.
    /// Inspect [`SeedingReport::violations`] for which.
    NotOneSideBound(SeedingReport),
    /// The document could not be analysed at all (fail closed).
    Unanalysable(SeedingError),
}

impl std::fmt::Display for SeedingDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotOneSideBound(r) => write!(
                f,
                "{} of {} rule(s) are not one-side-bound",
                r.violations().len(),
                r.rules.len()
            ),
            Self::Unanalysable(e) => write!(f, "document not analysable: {}", e),
        }
    }
}

impl std::error::Error for SeedingDenial {}

/// Analyse every `{ body } => { head }` rule in `n3` for one-side-bound seeding and head
/// range-restriction (module docs §2).
///
/// # Errors
/// [`SeedingError`] if the document uses a construct the conservative parser does not
/// model, or does not parse — never a silent pass.
pub fn analyse_seeding(n3: &str) -> Result<SeedingReport, SeedingError> {
    let toks = tokenise(n3)?;
    let mut rules = Vec::new();
    let mut i = 0usize;
    while i < toks.len() {
        match &toks[i] {
            Tok::Open => {
                let (body_toks, next) = formula_body(&toks, i)?;
                // A `{ … }` must be a rule antecedent: `=> { … } .`
                if !matches!(toks.get(next), Some(Tok::Arrow)) {
                    return Err(SeedingError::Unsupported(
                        "a formula that is not a `{ body } => { head }` rule".to_owned(),
                    ));
                }
                let head_start = next + 1;
                if !matches!(toks.get(head_start), Some(Tok::Open)) {
                    return Err(SeedingError::Unsupported(
                        "`=>` not followed by a `{ head }` formula".to_owned(),
                    ));
                }
                let (head_toks, after_head) = formula_body(&toks, head_start)?;
                let body = atoms(&body_toks)?;
                let head = atoms(&head_toks)?;
                rules.push(analyse_rule(rules.len(), &body, &head));
                // An optional statement terminator after the rule.
                i = if matches!(toks.get(after_head), Some(Tok::Dot)) {
                    after_head + 1
                } else {
                    after_head
                };
            }
            // Top-level ground facts / `@prefix` declarations: not rules, skipped.
            Tok::Arrow => {
                return Err(SeedingError::Unsupported(
                    "`=>` outside a `{ body } => { head }` rule".to_owned(),
                ))
            }
            Tok::Close => {
                return Err(SeedingError::Unterminated("formula (stray `}`)".to_owned()))
            }
            _ => i += 1,
        }
    }
    Ok(SeedingReport { rules })
}

/// The **fail-closed guard**: `Ok(())` iff every rule in `n3` is one-side-bound and
/// range-restricted. Run this over controller-authored `.acr` rule text before handing
/// it to [`crate::wire::derive_grants`].
///
/// # Errors
/// [`SeedingDenial::NotOneSideBound`] when some rule violates the property, or
/// [`SeedingDenial::Unanalysable`] when the document cannot be analysed. Both are
/// denials — a document the analysis cannot decide never passes.
pub fn require_one_side_bound(n3: &str) -> Result<(), SeedingDenial> {
    match analyse_seeding(n3) {
        Err(e) => Err(SeedingDenial::Unanalysable(e)),
        Ok(report) if report.all_one_side_bound() => Ok(()),
        Ok(report) => Err(SeedingDenial::NotOneSideBound(report)),
    }
}

/// The greedy, deterministic one-side-bound order for a single rule.
fn analyse_rule(rule_index: usize, body: &[[String; 3]], head: &[[String; 3]]) -> RuleSeeding {
    let mut bound: BTreeSet<&str> = BTreeSet::new();
    let mut taken = vec![false; body.len()];
    let mut seeding_order = Vec::with_capacity(body.len());

    // Repeatedly take the LOWEST-index remaining atom whose subject or object is a
    // constant or an already-bound variable. Lowest-index keeps the answer independent
    // of iteration order, so the report is reproducible.
    while let Some(pick) = (0..body.len()).find(|&k| {
        !taken[k] && (term_bound(&body[k][0], &bound) || term_bound(&body[k][2], &bound))
    }) {
        taken[pick] = true;
        seeding_order.push(pick);
        for t in &body[pick] {
            if is_var(t) {
                bound.insert(t.as_str());
            }
        }
    }

    // Whatever is left cannot be one-side-bound under ANY order (the greedy pick is
    // complete for this property: an atom becomes seedable only when some variable it
    // shares gets bound, and every binding a remaining atom could contribute is already
    // in `bound` or comes from another remaining atom that is equally stuck).
    let mut unseeded = Vec::new();
    for (k, atom) in body.iter().enumerate() {
        if taken[k] {
            continue;
        }
        let kind = if term_bound(&atom[1], &bound) {
            UnseededKind::PredicateAnchored
        } else {
            UnseededKind::Unanchored
        };
        unseeded.push(UnseededAtom {
            atom_index: k,
            atom: atom.join(" "),
            kind,
        });
        seeding_order.push(k);
    }
    // The evaluator still binds these atoms' variables, just expensively — so the head
    // range-restriction check sees the whole body, not only the seedable prefix.
    for (k, atom) in body.iter().enumerate() {
        if taken[k] {
            continue;
        }
        for t in atom {
            if is_var(t) {
                bound.insert(t.as_str());
            }
        }
    }

    let unsafe_head_vars: Vec<String> = head
        .iter()
        .flatten()
        .filter(|t| is_var(t))
        .filter(|t| !bound.contains(t.as_str()))
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    RuleSeeding {
        rule_index,
        body_atoms: body.len(),
        seeding_order,
        unseeded,
        unsafe_head_vars,
    }
}

fn is_var(t: &str) -> bool {
    t.starts_with('?')
}

/// A term is "bound" at a join step iff it is a constant, or a variable already bound.
fn term_bound(t: &str, bound: &BTreeSet<&str>) -> bool {
    !is_var(t) || bound.contains(t)
}

// ── The conservative Notation3 tokeniser ──────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
enum Tok {
    Open,
    Close,
    Arrow,
    Dot,
    Semi,
    Comma,
    Term(String),
}

/// Tokenise a Notation3 document. Anything the analysis does not model is an error, so
/// the caller cannot mistake "not understood" for "no violation".
fn tokenise(src: &str) -> Result<Vec<Tok>, SeedingError> {
    let b: Vec<char> = src.chars().collect();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < b.len() {
        let c = b[i];
        if c.is_whitespace() {
            i += 1;
        } else if c == '#' {
            // A comment: `#` inside an IRI or a string is consumed by those arms below,
            // so reaching here always means a real comment.
            while i < b.len() && b[i] != '\n' {
                i += 1;
            }
        } else if c == '<' {
            if b.get(i + 1) == Some(&'=') {
                return Err(SeedingError::Unsupported(
                    "reverse implication `<=`".to_owned(),
                ));
            }
            let end = (i + 1..b.len()).find(|&k| b[k] == '>').ok_or_else(|| {
                SeedingError::Unterminated("IRI (no closing `>`)".to_owned())
            })?;
            out.push(Tok::Term(b[i..=end].iter().collect()));
            i = end + 1;
        } else if c == '"' {
            let mut k = i + 1;
            while k < b.len() && b[k] != '"' {
                k += if b[k] == '\\' { 2 } else { 1 };
            }
            if k >= b.len() {
                return Err(SeedingError::Unterminated("string literal".to_owned()));
            }
            let mut end = k + 1;
            // An optional datatype (`^^…`) or language tag (`@…`) belongs to the literal.
            if b.get(end) == Some(&'^') && b.get(end + 1) == Some(&'^') {
                end += 2;
                if b.get(end) == Some(&'<') {
                    let close = (end + 1..b.len()).find(|&k| b[k] == '>').ok_or_else(|| {
                        SeedingError::Unterminated("datatype IRI (no closing `>`)".to_owned())
                    })?;
                    end = close + 1;
                } else {
                    while end < b.len() && !is_delim(b[end]) {
                        end += 1;
                    }
                }
            } else if b.get(end) == Some(&'@') {
                end += 1;
                while end < b.len() && !is_delim(b[end]) {
                    end += 1;
                }
            }
            out.push(Tok::Term(b[i..end].iter().collect()));
            i = end;
        } else if c == '{' {
            out.push(Tok::Open);
            i += 1;
        } else if c == '}' {
            out.push(Tok::Close);
            i += 1;
        } else if c == '.' {
            out.push(Tok::Dot);
            i += 1;
        } else if c == ';' {
            out.push(Tok::Semi);
            i += 1;
        } else if c == ',' {
            out.push(Tok::Comma);
            i += 1;
        } else if c == '=' {
            if b.get(i + 1) == Some(&'>') {
                out.push(Tok::Arrow);
                i += 2;
            } else {
                return Err(SeedingError::Unsupported("bare `=`".to_owned()));
            }
        } else if c == '[' || c == ']' {
            return Err(SeedingError::Unsupported(
                "blank-node property list `[ … ]`".to_owned(),
            ));
        } else if c == '(' || c == ')' {
            return Err(SeedingError::Unsupported("collection `( … )`".to_owned()));
        } else {
            let start = i;
            while i < b.len() && !is_delim(b[i]) {
                i += 1;
            }
            if i == start {
                return Err(SeedingError::Unsupported(format!("token `{}`", c)));
            }
            out.push(Tok::Term(b[start..i].iter().collect()));
        }
    }
    Ok(out)
}

fn is_delim(c: char) -> bool {
    c.is_whitespace()
        || matches!(
            c,
            '{' | '}' | '.' | ';' | ',' | '[' | ']' | '(' | ')' | '"' | '=' | '#' | '<' | '>'
        )
}

/// Given `toks[open] == Tok::Open`, return the tokens strictly inside the formula and the
/// index just past its `}`. A nested formula is unsupported (never silently accepted).
fn formula_body(toks: &[Tok], open: usize) -> Result<(Vec<Tok>, usize), SeedingError> {
    let mut inner = Vec::new();
    let mut i = open + 1;
    while i < toks.len() {
        match &toks[i] {
            Tok::Close => return Ok((inner, i + 1)),
            Tok::Open => {
                return Err(SeedingError::Unsupported(
                    "nested formula inside a rule".to_owned(),
                ))
            }
            t => inner.push(t.clone()),
        }
        i += 1;
    }
    Err(SeedingError::Unterminated(
        "formula (no closing `}`)".to_owned(),
    ))
}

/// Expand a formula's token stream into `subject predicate object` atoms, honouring the
/// `;` predicate-object and `,` object shorthands.
fn atoms(toks: &[Tok]) -> Result<Vec<[String; 3]>, SeedingError> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < toks.len() {
        if matches!(toks[i], Tok::Dot) {
            i += 1;
            continue;
        }
        let subject = take_term(toks, &mut i)?;
        loop {
            let predicate = take_term(toks, &mut i)?;
            loop {
                let object = take_term(toks, &mut i)?;
                out.push([subject.clone(), predicate.clone(), object]);
                if matches!(toks.get(i), Some(Tok::Comma)) {
                    i += 1;
                    continue;
                }
                break;
            }
            if matches!(toks.get(i), Some(Tok::Semi)) {
                i += 1;
                // A trailing `;` before `.` or end of formula closes the statement.
                if matches!(toks.get(i), None | Some(Tok::Dot)) {
                    break;
                }
                continue;
            }
            break;
        }
        match toks.get(i) {
            None => break,
            Some(Tok::Dot) => i += 1,
            Some(t) => {
                return Err(SeedingError::MalformedAtom(format!(
                    "expected `.` after a statement, found {:?}",
                    t
                )))
            }
        }
    }
    Ok(out)
}

fn take_term(toks: &[Tok], i: &mut usize) -> Result<String, SeedingError> {
    match toks.get(*i) {
        Some(Tok::Term(t)) => {
            *i += 1;
            Ok(t.clone())
        }
        other => Err(SeedingError::MalformedAtom(format!(
            "expected a term, found {:?}",
            other
        ))),
    }
}
