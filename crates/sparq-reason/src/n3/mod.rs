//! Notation3 (N3) rule reasoning — a forward-chaining engine toward EYE-reasoner parity.
//!
//! N3 adds rules (`{ premise } => { conclusion }`), variables (`?x`), formulae (`{ … }`) and
//! **builtins** (`math:`, `string:`, `log:`, …) on top of Turtle. EYE is a mature reasoner
//! with hundreds of builtins; this is the foundation: a parser (`parser`), a term model
//! (`model`), and a semi-naive forward chainer that applies rules to a fixpoint with variable
//! binding and a growing set of builtins. Coverage expands builtin-by-builtin, validated
//! against EYE's own test cases (see the `eye_cases` tests).
//!
//! Builtins implemented (roadmap T12 — growing toward EYE parity):
//!   * `math:` comparisons (greaterThan/lessThan/notGreaterThan/notLessThan/equalTo/notEqualTo)
//!     and functional (sum/difference/product/quotient/max/min/exponentiation/negation/
//!     absoluteValue/rounded/floor/ceiling/memberCount) over `( … )` list arguments, plus the
//!     trig/hyperbolic/log family (sin/cos/tan/asin/acos/atan/sinh/cosh/tanh/asinh/acosh/
//!     atanh, degrees/radians, `(x base) math:logarithm` = log_base(x), and `(x y) math:atan2`
//!     which — exactly like eye.pl — computes `atan(x/y)`, not the quadrant-aware C atan2);
//!   * `string:` concatenation/length/contains/containsIgnoringCase/startsWith/endsWith/
//!     greaterThan/lessThan/matches/notMatches/replace/lowerCase/upperCase,
//!     `string:encodeForUri` (RFC 3986 percent-encoding, the XPath `fn:encode-for-uri`
//!     unreserved set — see [`encode_for_uri`]), `string:scrape`
//!     (first regex capture group), and a `string:format` SUBSET (`%s` `%d` `%f` `%%` only —
//!     C-printf width/precision flags fail the premise rather than mis-format);
//!   * `list:` length/first/last and the member/in generators;
//!   * `time:` year/month/day (EYE's set), plus hours/minutes/seconds (SWAP/cwm);
//!   * `log:` equalTo/notEqualTo, includes/notIncludes (store-scoped when the subject is
//!     `{ }`/unbound — scoped negation as failure — or true formula containment when the
//!     subject is a ground `{ … }` formula; see [`scoped_negation`]), `log:conjunction`
//!     (merge a list of formulae; duplicate triples deduped, original order kept),
//!     `log:uri` (IRI ↔ xsd:string, both directions), and `log:dtlit`
//!     (`("lex" xsd:dt) log:dtlit "lex"^^xsd:dt`, both directions).
//!
//! Backward rules (`<=`, log:impliedBy) are GOAL-DIRECTED, matching EYE: they never fire
//! forward and their conclusions are never materialized into the closure on their own.
//! When a forward-rule premise atom (e.g. an EYE-style query rule `{goal} => {goal}`)
//! finds no supporting fact — or in addition to its fact matches — the engine resolves it
//! against backward-rule conclusions SLD-style: unify the goal with a (variable-renamed)
//! conclusion, then prove that rule's premise (joins, builtins, and further backward rules,
//! depth-bounded by [`BW_DEPTH`]). Verified against eyereasoner/eye `reasoning/backward`:
//! the old "reverse `<=` into a forward rule" treatment derives NOTHING there, because the
//! premise (`?X math:greaterThan ?Y`) is a pure builtin that is only evaluable once the
//! GOAL binds the variables. Known gap (documented, not hacked): goals whose arguments are
//! `( … )` lists or quoted formulae are not structurally unified with backward conclusions
//! (rule-local list structure uses per-rule blank nodes), so recursive list-state idioms
//! (EYE's fibonacci/collatz/peasant) are out of scope for now.
//!
//! Deliberately NOT implemented (checked against EYE/SWAP):
//!   * `math:greaterThanOrEqual`/`lessThanOrEqual` — not in the SWAP math vocabulary nor in
//!     EYE's builtin table; the canonical names are notLessThan/notGreaterThan (implemented).
//!   * `list:append`/`list:rest` — producing a NEW list requires a first-class list value in
//!     [`Term`]; lists currently exist only as rule-local rdf:first/rest structure resolved
//!     up front ([`extract_lists`]), and `intern` has no dictionary representation for a
//!     list value, so a produced list could neither be bound to a variable nor emitted into
//!     the ground closure. Needs a `Term::List` variant first — deferred rather than hacked.
//!   * `time:localTime`/`time:gmTime` — wall-clock reads; a deterministic closure cannot
//!     depend on when it is computed. (`time:inSeconds` is not an EYE builtin at all —
//!     eye-builtins.n3 lists only year/month/day/localTime under `time:`.)
//!   * `log:semantics`/`log:content` — dereference and parse remote/local documents; the
//!     reasoner is a pure function of its input text, so document fetching stays out.
//!   * `log:rawType` — EYE distinguishes labeled vs unlabeled blank nodes by their origin,
//!     which our parser does not track; reporting an approximate type would silently
//!     disagree with EYE, so it is deferred until bnode origin is recorded.
//!   * `string:format` directives beyond `%s`/`%d`/`%f`/`%%` (width, precision, `%e`/`%g`,
//!     length modifiers) — unimplemented directives FAIL the premise (no silent mangling).
//!   * Reverse (object-bound) modes of the unary math builtins (EYE's `?X math:sin 0.5`
//!     solving X = asin 0.5) — only the forward subject→object direction is evaluated
//!     (EXCEPT `math:negation`, which is bidirectional like EYE);
//!     premises are evaluated left-to-right with no coroutining (EYE `when`-delays).
//!
//! Next increments: `Term::List` (first-class list values: list:append, backward goals over
//! list arguments), non-ground formula scopes for `log:includes` (quantification),
//! `log:collectAllIn` (scoped aggregation).

mod model;
pub mod parser;

pub use model::{Rule, Term};
use rustc_hash::{FxHashMap, FxHashSet};
use sparq_core::dict::{Dict, Id};
use std::collections::HashMap;

/// Facts + access indexes, so a rule-body join atom is an O(1)/O(matches) lookup instead of a
/// full scan. Indexed by (predicate, subject)→objects, (predicate, object)→subjects, and
/// predicate→facts — the patterns that arise when a rule atom's predicate (and often one
/// argument) is already bound. Maintained incrementally as the closure grows. Without this,
/// even semi-naive evaluation degrades to O(N²) on recursive rule chains (DeepTaxonomy).
#[derive(Default)]
struct FactIndex {
    all: FxHashSet<[Term; 3]>,
    ps: FxHashMap<(Term, Term), Vec<Term>>, // (pred, subj) -> objects
    po: FxHashMap<(Term, Term), Vec<Term>>, // (pred, obj) -> subjects
    p: FxHashMap<Term, Vec<[Term; 3]>>,     // pred -> facts (predicate-only-bound)
}

impl FactIndex {
    fn from_iter(facts: impl IntoIterator<Item = [Term; 3]>) -> FactIndex {
        let mut ix = FactIndex::default();
        for f in facts {
            ix.insert(f);
        }
        ix
    }
    fn contains(&self, t: &[Term; 3]) -> bool {
        self.all.contains(t)
    }
    fn len(&self) -> usize {
        self.all.len()
    }
    fn insert(&mut self, t: [Term; 3]) -> bool {
        if !self.all.insert(t.clone()) {
            return false;
        }
        let [s, p, o] = &t;
        self.ps.entry((p.clone(), s.clone())).or_default().push(o.clone());
        self.po.entry((p.clone(), o.clone())).or_default().push(s.clone());
        self.p.entry(p.clone()).or_default().push(t.clone());
        true
    }
    /// Candidate facts matching a (partially-ground) pattern, via the most selective index.
    fn candidates(&self, s: &Term, p: &Term, o: &Term) -> Vec<[Term; 3]> {
        let (sg, pg, og) = (s.is_ground(), p.is_ground(), o.is_ground());
        if pg && sg {
            self.ps
                .get(&(p.clone(), s.clone()))
                .map(|os| os.iter().map(|ob| [s.clone(), p.clone(), ob.clone()]).collect())
                .unwrap_or_default()
        } else if pg && og {
            self.po
                .get(&(p.clone(), o.clone()))
                .map(|ss| ss.iter().map(|sb| [sb.clone(), p.clone(), o.clone()]).collect())
                .unwrap_or_default()
        } else if pg {
            self.p.get(p).cloned().unwrap_or_default()
        } else {
            self.all.iter().cloned().collect() // predicate unbound — rare; fall back to scan
        }
    }
}

const MATH: &str = "http://www.w3.org/2000/10/swap/math#";
const LOG: &str = "http://www.w3.org/2000/10/swap/log#";
const STRING: &str = "http://www.w3.org/2000/10/swap/string#";
const LIST: &str = "http://www.w3.org/2000/10/swap/list#";
const TIME: &str = "http://www.w3.org/2000/10/swap/time#";

/// Depth bound for goal-directed (`<=`) resolution: a backward proof may chain through at
/// most this many backward-rule applications. Bounds runaway recursion (e.g. a rule whose
/// premise re-poses its own goal) — within the bound, proofs are exhaustive.
const BW_DEPTH: usize = 64;

/// Goal-directed context: the document's backward (`<=`) rules plus a counter for renaming
/// rule variables apart (standardizing apart, so nested applications of the same rule do not
/// capture each other's bindings).
struct BwCtx<'a> {
    rules: &'a [Rule],
    rename: std::cell::Cell<usize>,
}

impl<'a> BwCtx<'a> {
    fn new(rules: &'a [Rule]) -> BwCtx<'a> {
        BwCtx { rules, rename: std::cell::Cell::new(0) }
    }
}

/// One derivation step: a `conclusion` triple was produced by rule `rule` (its index in the
/// document's rule order) from the ground `premises` (the supporting facts under the binding;
/// premise atoms proven by backward rules are not themselves facts and are not listed).
pub struct ProofStep {
    pub conclusion: [Id; 3],
    pub rule: usize,
    pub premises: Vec<[Id; 3]>,
}

/// Parse N3 `src`, run the rule closure, and return the entailed GROUND triples interned into
/// `dict`. The rules/formulae/variables are consumed by reasoning; only ground facts remain.
pub fn reason_n3(dict: &mut Dict, src: &str) -> Result<Vec<[Id; 3]>, String> {
    Ok(reason_n3_proof(dict, src)?.0)
}

/// As [`reason_n3`], but also return the derivation (a [`ProofStep`] for each NEWLY-derived
/// triple, in derivation order) — the EYE `--proof` analogue.
pub fn reason_n3_proof(dict: &mut Dict, src: &str) -> Result<(Vec<[Id; 3]>, Vec<ProofStep>), String> {
    let parsed = parser::parse(src)?;
    let (facts, steps) = run_closure(parsed);
    intern_closure(dict, &facts, &steps)
}

/// Term-level closure result — what [`reason_n3_terms`] returns. Unlike
/// [`reason_n3`], nothing is interned: facts may contain `{ … }` formula terms
/// (which have no dictionary representation) and survive here untouched.
pub struct N3Closure {
    /// The full ground closure: original facts plus every derivation (a set).
    pub facts: Vec<[Term; 3]>,
    /// Only the newly-derived triples, in derivation order.
    pub derived: Vec<[Term; 3]>,
    /// Forward (`=>`) / backward (`<=`) rule counts of the parsed document.
    pub n_rules: usize,
    pub n_backward_rules: usize,
}

/// As [`reason_n3`], but returns the closure at the TERM level (no dictionary)
/// and resolves relative IRIs against `base` when given — the entry point used
/// by the W3C N3 conformance harness (cwm/EYE-style: `--think` then compare).
pub fn reason_n3_terms(src: &str, base: Option<&str>) -> Result<N3Closure, String> {
    let parsed = match base {
        Some(b) => parser::parse_with_base(src, b)?,
        None => parser::parse(src)?,
    };
    let (n_rules, n_backward_rules) = (parsed.rules.len(), parsed.backward_rules.len());
    let (facts, steps) = run_closure(parsed);
    Ok(N3Closure {
        facts: facts.all.into_iter().collect(),
        derived: steps.into_iter().map(|(g, _, _)| g).collect(),
        n_rules,
        n_backward_rules,
    })
}

/// The semi-naive forward-chaining fixpoint shared by the id-level and
/// term-level entry points. Returns the final fact set plus the derivation
/// steps `(conclusion, rule index, supporting premises)` in derivation order.
fn run_closure(parsed: parser::Parsed) -> (FactIndex, Vec<([Term; 3], usize, Vec<[Term; 3]>)>) {
    let parser::Parsed { facts: facts0, rules, backward_rules } = parsed;
    let mut facts = FactIndex::from_iter(facts0);
    let bw = BwCtx::new(&backward_rules);
    // Derivation steps at the term level (interned to ids once at the end).
    let mut steps: Vec<([Term; 3], usize, Vec<[Term; 3]>)> = Vec::new();

    // Which predicates can a backward rule conclude? A forward rule with a premise atom on
    // such a predicate cannot use the semi-naive delta restriction (a backward proof is not
    // a fact in any delta), so it re-evaluates fully each round — see `needs_full` below.
    let bw_concl_preds: FxHashSet<&str> = backward_rules
        .iter()
        .flat_map(|r| r.conclusion.iter())
        .filter_map(|c| match &c[1] {
            Term::Iri(i) => Some(i.as_str()),
            _ => None,
        })
        .collect();
    let bw_any_var_pred = backward_rules
        .iter()
        .flat_map(|r| r.conclusion.iter())
        .any(|c| matches!(&c[1], Term::Var(_)));

    // SEMI-NAIVE fixpoint: each round, a positive rule only fires on bindings that involve at
    // least one NEWLY-derived fact (the `delta`) — run once per join-atom position, with that
    // atom restricted to `delta` and the rest to all facts. This avoids re-deriving the whole
    // closure every round (the naive blow-up on recursive rule chains). Rules with scoped
    // negation are non-monotonic, and rules whose join atoms may be proven by BACKWARD rules
    // have support outside the fact deltas — both re-evaluate against ALL facts each round
    // (correct; the fixpoint still terminates because conclusions are deduped). Pure-builtin
    // rules (no join atom) fire only in round 0.
    let rule_meta: Vec<(Vec<usize>, bool)> = rules
        .iter()
        .map(|r| {
            let joins: Vec<usize> =
                r.premise.iter().enumerate().filter(|(_, p)| is_join_atom(p)).map(|(i, _)| i).collect();
            let has_neg = r.premise.iter().any(|p| scoped_negation(&p[1]).is_some());
            let needs_bw = joins.iter().any(|&k| match &r.premise[k][1] {
                Term::Iri(i) => bw_any_var_pred || bw_concl_preds.contains(i.as_str()),
                _ => !backward_rules.is_empty(),
            });
            (joins, has_neg || needs_bw)
        })
        .collect();

    let mut delta: FxHashSet<[Term; 3]> = facts.all.clone(); // round 0: every fact is "new"
    let mut first_round = true;
    loop {
        let mut produced: Vec<([Term; 3], usize, Vec<[Term; 3]>)> = Vec::new();
        for (ri, rule) in rules.iter().enumerate() {
            let (joins, needs_full) = &rule_meta[ri];
            let bindings: Vec<Binding> = if *needs_full || joins.is_empty() {
                // non-monotonic / backward-supported / constant rule: full evaluation
                // (negation + backward) every round, or round-0 only (constant).
                if *needs_full || first_round {
                    match_premise(&rule.premise, &facts, &bw)
                } else {
                    Vec::new()
                }
            } else {
                // Semi-naive: union over delta-at-each-join-position (dedup via facts.insert).
                let mut bs = Vec::new();
                for &k in joins {
                    bs.extend(match_premise_seeded(
                        &rule.premise,
                        &facts,
                        &Binding::new(),
                        Some((&delta, k)),
                        &bw,
                        BW_DEPTH,
                    ));
                }
                bs
            };
            for b in bindings {
                for c in &rule.conclusion {
                    if let Some(g) = ground_triple(c, &b) {
                        if !facts.contains(&g) {
                            // The supporting facts: premise patterns instantiated under b that
                            // are actual facts (excludes builtins / list structure).
                            let prem: Vec<[Term; 3]> = rule
                                .premise
                                .iter()
                                .filter_map(|p| ground_triple(p, &b))
                                .filter(|t| facts.contains(t))
                                .collect();
                            produced.push((g, ri, prem));
                        }
                    }
                }
            }
        }
        let mut new_delta: FxHashSet<[Term; 3]> = FxHashSet::default();
        for (g, ri, prem) in produced {
            if facts.insert(g.clone()) {
                new_delta.insert(g.clone());
                steps.push((g, ri, prem));
            }
        }
        first_round = false;
        if new_delta.is_empty() {
            break;
        }
        delta = new_delta;
    }
    (facts, steps)
}

/// Intern a term-level closure + derivation into the dictionary ([`reason_n3`] /
/// [`reason_n3_proof`] output form). Errors on formula-valued facts, which have
/// no dictionary representation (use [`reason_n3_terms`] for those documents).
fn intern_closure(
    dict: &mut Dict,
    facts: &FactIndex,
    steps: &[([Term; 3], usize, Vec<[Term; 3]>)],
) -> Result<(Vec<[Id; 3]>, Vec<ProofStep>), String> {
    // First-class list values have no dictionary representation — expand them
    // into rdf:first/rest blank-node chains (one chain per list VALUE, shared
    // across the facts that mention it).
    let mut exp = ListExpander::default();
    let fact_rows: Vec<[Term; 3]> = facts.all.iter().cloned().collect();
    let (fact_rows, mut extra) = exp.expand_rows(&fact_rows);
    // Intern the ground closure into the dictionary.
    let mut out = Vec::with_capacity(fact_rows.len() + extra.len());
    let mut rows = fact_rows;
    rows.append(&mut extra);
    for t in &rows {
        out.push([intern(dict, &t[0])?, intern(dict, &t[1])?, intern(dict, &t[2])?]);
    }
    // Intern the proof steps (list terms expanded to their chain heads; the
    // chain structure itself is already in the closure rows above).
    let mut proof = Vec::with_capacity(steps.len());
    for (g, ri, prem) in steps {
        let mut it = |t: &[Term; 3], d: &mut Dict, e: &mut ListExpander| -> Result<[Id; 3], String> {
            let r = e.expand_row(t);
            Ok([intern(d, &r[0])?, intern(d, &r[1])?, intern(d, &r[2])?])
        };
        let conclusion = it(g, dict, &mut exp)?;
        let premises =
            prem.iter().map(|p| it(p, dict, &mut exp)).collect::<Result<Vec<_>, _>>()?;
        proof.push(ProofStep { conclusion, rule: *ri, premises });
    }
    Ok((out, proof))
}

/// Expands first-class `Term::List` values into rdf:first/rest blank-node
/// chains for consumers that need pure RDF triples (the dictionary-interning
/// entry points). One chain per distinct list value; `()` becomes `rdf:nil`.
#[derive(Default)]
struct ListExpander {
    heads: FxHashMap<Term, Term>,
    structure: Vec<[Term; 3]>,
    counter: usize,
}

impl ListExpander {
    fn expand_rows(&mut self, rows: &[[Term; 3]]) -> (Vec<[Term; 3]>, Vec<[Term; 3]>) {
        let out: Vec<[Term; 3]> = rows.iter().map(|r| self.expand_row(r)).collect();
        (out, std::mem::take(&mut self.structure))
    }
    fn expand_row(&mut self, row: &[Term; 3]) -> [Term; 3] {
        [self.expand(&row[0]), self.expand(&row[1]), self.expand(&row[2])]
    }
    fn expand(&mut self, t: &Term) -> Term {
        let Term::List(ms) = t else { return t.clone() };
        if ms.is_empty() {
            return Term::Iri(parser::RDF_NIL.into());
        }
        if let Some(head) = self.heads.get(t) {
            return head.clone();
        }
        let members: Vec<Term> = ms.iter().map(|m| self.expand(m)).collect();
        let mut tail = Term::Iri(parser::RDF_NIL.into());
        for m in members.into_iter().rev() {
            self.counter += 1;
            let node = Term::Blank(format!("_l{}", self.counter));
            self.structure.push([node.clone(), Term::Iri(parser::RDF_FIRST.into()), m]);
            self.structure.push([node.clone(), Term::Iri(parser::RDF_REST.into()), tail]);
            tail = node;
        }
        self.heads.insert(t.clone(), tail.clone());
        tail
    }
}

type Binding = HashMap<String, Term>;

/// All variable bindings under which every premise pattern holds (joining against `facts`,
/// evaluating builtins as filters/computations). N3 collections `( … )` in the premise are
/// rule-local list STRUCTURE (rdf:first/rest over fresh bnodes), not data to match — they are
/// extracted up front and consumed by the functional builtins (e.g. `math:sum`).
fn match_premise(premise: &[[Term; 3]], facts: &FactIndex, bw: &BwCtx) -> Vec<Binding> {
    match_premise_seeded(premise, facts, &Binding::new(), None, bw, BW_DEPTH)
}

/// Match `premise` starting from an existing partial binding `seed`. For SEMI-NAIVE
/// evaluation, `delta_at = Some((delta, k))` restricts the join atom at premise index `k` to
/// match only the `delta` set (the newly-derived facts) rather than the full `facts` — so the
/// driver, by running once per join-atom index, considers only bindings that involve ≥1 new
/// fact. `None` = naive (every join atom matches all facts); also used for the negation
/// sub-formula recursion.
fn match_premise_seeded(
    premise: &[[Term; 3]],
    facts: &FactIndex,
    seed: &Binding,
    delta_at: Option<(&FxHashSet<[Term; 3]>, usize)>,
    bw: &BwCtx,
    depth: usize,
) -> Vec<Binding> {
    // Semi-naive: seed from the DELTA atom first (delta is small → most selective), then
    // evaluate the rest against the index. Doing the delta atom first makes it prune early
    // instead of letting a non-delta atom do a full predicate-index scan.
    if let Some((delta, k)) = delta_at {
        let mut seeds = Vec::new();
        for fact in delta {
            if let Some(nb) = unify(&premise[k], fact, seed) {
                seeds.push(nb);
            }
        }
        if seeds.is_empty() {
            return Vec::new();
        }
        let rest: Vec<[Term; 3]> =
            premise.iter().enumerate().filter(|&(i, _)| i != k).map(|(_, p)| p.clone()).collect();
        let mut out = Vec::new();
        for s in &seeds {
            out.extend(match_premise_seeded(&rest, facts, s, None, bw, depth));
        }
        return out;
    }
    let mut bindings: Vec<Binding> = vec![seed.clone()];
    for pat in premise {
        // log:includes / log:notIncludes — does the SCOPE include the object formula?
        //   * subject `{ }` (empty formula) or unbound/non-formula: the scope is the current
        //     store (the engine's scoped-negation-as-failure idiom, matching prior behaviour);
        //   * subject a ground non-empty `{ … }` formula: true formula containment — the
        //     pattern is matched against THAT formula's triples only;
        //   * subject a NON-ground formula (free variables inside `{ … }`): needs
        //     quantification machinery — future work; the premise fails (matches nothing).
        // `log:includes` may BIND free variables of the object pattern (one binding per
        // match, like EYE); `log:notIncludes` holds iff no match exists.
        if let Some(is_not) = scoped_negation(&pat[1]) {
            let inner: &[[Term; 3]] = match &pat[2] {
                Term::Formula(t) => t,
                _ => &[],
            };
            let mut next = Vec::new();
            for b in bindings {
                let matches: Option<Vec<Binding>> = match apply(&pat[0], &b) {
                    Term::Formula(ts) if !ts.is_empty() => {
                        if ts.iter().all(|t| t.iter().all(Term::is_ground)) {
                            // Formula containment is SYNTACTIC: backward rules describe the
                            // store, not arbitrary quoted graphs — no goal-direction here.
                            let scope = FactIndex::from_iter(ts);
                            let no_bw = BwCtx::new(&[]);
                            Some(match_premise_seeded(inner, &scope, &b, None, &no_bw, 0))
                        } else {
                            None // non-ground scope formula: unsupported (future work)
                        }
                    }
                    _ => Some(match_premise_seeded(inner, facts, &b, None, bw, depth)),
                };
                let Some(matches) = matches else { continue };
                if is_not {
                    if matches.is_empty() {
                        next.push(b);
                    }
                } else {
                    next.extend(matches);
                }
            }
            bindings = next;
            if bindings.is_empty() {
                break;
            }
            continue;
        }
        if let Some(gen) = list_generator(&pat[1]) {
            // list:member / list:in / list:iterate — one binding per member.
            let (list_pos, var_pos) = match gen {
                ListGen::Member | ListGen::Iterate => (&pat[0], &pat[2]),
                ListGen::In => (&pat[2], &pat[0]),
            };
            let mut next = Vec::new();
            for b in &bindings {
                let head = apply(list_pos, b);
                // A first-class `( … )` list value, hand-written rdf:first/rest
                // rule structure, or a data list reached through a bound
                // variable (walked from the fact store).
                let members: Option<Vec<Term>> = match &head {
                    Term::List(ms) => Some(ms.clone()),
                    _ => fact_list(&head, facts),
                };
                if let Some(members) = members {
                    for (ix, m) in members.iter().enumerate() {
                        let mv = apply(m, b);
                        let target = match gen {
                            ListGen::Member | ListGen::In => mv,
                            // (?index ?value) pairs, 0-based (EYE list:iterate).
                            ListGen::Iterate => Term::List(vec![
                                Term::Lit(ix.to_string(), parser::XSD_INTEGER.into(), None),
                                mv,
                            ]),
                        };
                        let mut nb = b.clone();
                        if unify_term(var_pos, &target, &mut nb) {
                            next.push(nb);
                        }
                    }
                }
            }
            bindings = next;
        } else if let Some(f) = functional_builtin(&pat[1]) {
            bindings = bindings
                .into_iter()
                .filter_map(|b| eval_functional(f, &pat[0], &pat[2], facts, b))
                .collect();
        } else if let Some(op) = binder_builtin(&pat[1]) {
            bindings = bindings.into_iter().filter_map(|b| eval_binder(op, &pat[0], &pat[2], b)).collect();
        } else if let Some(op) = builtin(&pat[1]) {
            bindings.retain(|b| eval_builtin(op, &pat[0], &pat[2], b));
        } else {
            // Join atom: selective FactIndex lookup (no full scan) for each current binding,
            // PLUS goal-directed resolution against the backward (`<=`) rules.
            let mut next = Vec::new();
            for b in &bindings {
                let (s_a, p_a, o_a) = (apply(&pat[0], b), apply(&pat[1], b), apply(&pat[2], b));
                // Virtual rdf:first / rdf:rest over a first-class list value —
                // cwm/EYE expose list structure to matching even though the
                // list is a term, not triples (`?L rdf:first ?X` computes).
                if let (Term::List(ms), Term::Iri(pi)) = (&s_a, &p_a) {
                    if pi == parser::RDF_FIRST || pi == parser::RDF_REST {
                        if !ms.is_empty() {
                            let val = if pi == parser::RDF_FIRST {
                                ms[0].clone()
                            } else {
                                Term::List(ms[1..].to_vec())
                            };
                            let mut nb = b.clone();
                            if unify_term(&pat[2], &val, &mut nb) {
                                next.push(nb);
                            }
                        }
                        continue; // a list term is never the subject of stored first/rest triples
                    }
                }
                let cands = facts.candidates(&s_a, &p_a, &o_a);
                for fact in &cands {
                    if let Some(nb) = unify(pat, fact, b) {
                        next.push(nb);
                    }
                }
                if !bw.rules.is_empty() && depth > 0 {
                    next.extend(backward_prove(pat, b, facts, bw, depth - 1));
                }
            }
            bindings = next;
        }
        if bindings.is_empty() {
            break;
        }
    }
    bindings
}

/// Whether a premise pattern is a JOIN atom (matched against facts), as opposed to a builtin,
/// list generator/structure, or scoped-negation atom.
fn is_join_atom(pat: &[Term; 3]) -> bool {
    // A literal-list subject under rdf:first/rest is the VIRTUAL list-access
    // computation, not a store join.
    let virtual_list = matches!(&pat[0], Term::List(_))
        && matches!(&pat[1], Term::Iri(i) if i == parser::RDF_FIRST || i == parser::RDF_REST);
    builtin(&pat[1]).is_none()
        && functional_builtin(&pat[1]).is_none()
        && binder_builtin(&pat[1]).is_none()
        && list_generator(&pat[1]).is_none()
        && scoped_negation(&pat[1]).is_none()
        && !virtual_list
}

/// Goal-directed (`<=`) resolution of one premise atom: for each backward rule whose
/// conclusion unifies with the goal (under the current binding `b`), prove that rule's
/// premise — recursively allowing joins, builtins, and further backward rules — and project
/// each proof back onto the goal's variables. SLD resolution with standardized-apart rule
/// variables and a depth bound (`depth` counts REMAINING backward applications).
fn backward_prove(pat: &[Term; 3], b: &Binding, facts: &FactIndex, bw: &BwCtx, depth: usize) -> Vec<Binding> {
    let goal = [apply(&pat[0], b), apply(&pat[1], b), apply(&pat[2], b)];
    let mut out = Vec::new();
    for rule in bw.rules {
        for concl in &rule.conclusion {
            // Cheap gate: ground predicates must agree before we rename/unify anything.
            if let (Term::Iri(gp), Term::Iri(cp)) = (&goal[1], &concl[1]) {
                if gp != cp {
                    continue;
                }
            }
            // Standardize apart: fresh names for this rule application's variables.
            let n = bw.rename.get();
            bw.rename.set(n + 1);
            let rc = [rename_vars(&concl[0], n), rename_vars(&concl[1], n), rename_vars(&concl[2], n)];
            let mut subst = b.clone();
            if !(unify_walked(&goal[0], &rc[0], &mut subst)
                && unify_walked(&goal[1], &rc[1], &mut subst)
                && unify_walked(&goal[2], &rc[2], &mut subst))
            {
                continue;
            }
            let prem: Vec<[Term; 3]> = rule
                .premise
                .iter()
                .map(|t| [rename_vars(&t[0], n), rename_vars(&t[1], n), rename_vars(&t[2], n)])
                .collect();
            for sol in match_premise_seeded(&prem, facts, &subst, None, bw, depth) {
                // Project the proof onto the goal's variables (walk chains like
                // outer-var → renamed-var → ground value).
                let mut nb = b.clone();
                let mut ok = true;
                for g in pat {
                    if let Term::Var(_) = g {
                        // Deep-resolve: walk the chain, then substitute inside
                        // any list structure the value carries.
                        let val = apply(&walk(g, &sol), &sol);
                        if (val.is_ground() || matches!(val, Term::Formula(_)))
                            && !unify_term(g, &val, &mut nb)
                        {
                            ok = false;
                            break;
                        }
                    }
                }
                if ok {
                    out.push(nb);
                }
            }
        }
    }
    out
}

/// `t` with every variable renamed into the standardize-apart space of rule application `n`
/// (recursing into quoted formulae).
fn rename_vars(t: &Term, n: usize) -> Term {
    match t {
        Term::Var(v) => Term::Var(format!("__bw{n}_{v}")),
        Term::Formula(ts) => Term::Formula(
            ts.iter().map(|tr| [rename_vars(&tr[0], n), rename_vars(&tr[1], n), rename_vars(&tr[2], n)]).collect(),
        ),
        Term::List(ms) => Term::List(ms.iter().map(|m| rename_vars(m, n)).collect()),
        _ => t.clone(),
    }
}

/// Resolve `t` through any chain of variable bindings in `s` (bounded against cycles).
fn walk(t: &Term, s: &Binding) -> Term {
    let mut cur = t.clone();
    for _ in 0..s.len() + 1 {
        match &cur {
            Term::Var(v) => match s.get(v) {
                Some(next) => cur = next.clone(),
                None => break,
            },
            _ => break,
        }
    }
    cur
}

/// Unification where BOTH sides may contain variables (goal ↔ backward-rule conclusion),
/// chain-resolving each side through `s` first. Unbound-vs-unbound binds left → right, so a
/// free goal variable points AT the rule variable and is grounded by the premise proof.
fn unify_walked(a: &Term, c: &Term, s: &mut Binding) -> bool {
    let aw = walk(a, s);
    let cw = walk(c, s);
    if aw == cw {
        return true;
    }
    match (&aw, &cw) {
        (Term::Var(v), _) => {
            s.insert(v.clone(), cw.clone());
            true
        }
        (_, Term::Var(v)) => {
            s.insert(v.clone(), aw.clone());
            true
        }
        // Structural list unification, member by member (either side may hold
        // variables — backward goals over list arguments).
        (Term::List(xs), Term::List(ys)) => {
            xs.len() == ys.len() && xs.iter().zip(ys).all(|(x, y)| unify_walked(x, y, s))
        }
        _ => false,
    }
}

/// Walk an rdf:first/rest list that lives in the FACT store (a collection
/// asserted in the data, reached through a bound variable) — the complement of
/// [`extract_lists`], which resolves rule-local structure. `rdf:nil` is the
/// empty list.
fn fact_list(head: &Term, facts: &FactIndex) -> Option<Vec<Term>> {
    let first = Term::Iri(parser::RDF_FIRST.into());
    let rest = Term::Iri(parser::RDF_REST.into());
    let nil = Term::Iri(parser::RDF_NIL.into());
    let empty = Term::List(Vec::new());
    let mut out = Vec::new();
    let mut cur = head.clone();
    let mut guard = 0;
    loop {
        if cur == nil || cur == empty {
            return Some(out);
        }
        if guard > 100_000 {
            return None;
        }
        guard += 1;
        let f = facts.ps.get(&(first.clone(), cur.clone()))?.first()?.clone();
        out.push(f);
        cur = facts.ps.get(&(rest.clone(), cur.clone()))?.first()?.clone();
    }
}

/// Try to unify pattern triple `pat` with ground fact `f`, extending binding `b`.
fn unify(pat: &[Term; 3], f: &[Term; 3], b: &Binding) -> Option<Binding> {
    let mut nb = b.clone();
    for i in 0..3 {
        if !unify_term(&pat[i], &f[i], &mut nb) {
            return None;
        }
    }
    Some(nb)
}

fn unify_term(pat: &Term, val: &Term, b: &mut Binding) -> bool {
    match (pat, val) {
        (Term::Var(v), _) => match b.get(v) {
            Some(existing) => existing == val,
            None => {
                b.insert(v.clone(), val.clone());
                true
            }
        },
        // First-class lists unify STRUCTURALLY: same length, members pairwise
        // (so `(?x)` matches `(17)` binding ?x=17 — cwm/EYE list unification).
        (Term::List(ps), Term::List(vs)) => {
            ps.len() == vs.len() && ps.iter().zip(vs).all(|(p, v)| unify_term(p, v, b))
        }
        (other, _) => other == val,
    }
}

/// Substitute bound variables in `t` (recursing into list members); returns the
/// term (possibly still containing free vars).
fn apply(t: &Term, b: &Binding) -> Term {
    match t {
        Term::Var(v) => b.get(v).cloned().unwrap_or_else(|| t.clone()),
        Term::List(ms) => Term::List(ms.iter().map(|m| apply(m, b)).collect()),
        _ => t.clone(),
    }
}

/// Instantiate a conclusion triple under binding `b`; `None` if any term stays non-ground.
fn ground_triple(t: &[Term; 3], b: &Binding) -> Option<[Term; 3]> {
    let g = [apply(&t[0], b), apply(&t[1], b), apply(&t[2], b)];
    if g.iter().all(|x| x.is_ground()) {
        Some(g)
    } else {
        None
    }
}

#[derive(Clone, Copy)]
enum Builtin {
    // numeric (math:)
    Gt,
    Lt,
    NotGt,
    NotLt,
    MathEq,
    MathNe,
    // term (log:)
    LogEq,
    LogNe,
    // string (string:)
    StrContains,
    StrStarts,
    StrEnds,
    StrGt,
    StrLt,
    StrMatches,        // string:matches (regex)
    StrNotMatches,     // string:notMatches (regex, negated; invalid regex ⇒ premise fails)
    StrContainsIgnCase, // string:containsIgnoringCase
    StrNotGt,           // string:notGreaterThan
    StrNotLt,           // string:notLessThan
    StrEqIgnCase,       // string:equalIgnoringCase
    StrNeIgnCase,       // string:notEqualIgnoringCase
}

fn builtin(p: &Term) -> Option<Builtin> {
    let Term::Iri(i) = p else { return None };
    if let Some(f) = i.strip_prefix(MATH) {
        return Some(match f {
            "greaterThan" => Builtin::Gt,
            "lessThan" => Builtin::Lt,
            "notGreaterThan" => Builtin::NotGt,
            "notLessThan" => Builtin::NotLt,
            "equalTo" => Builtin::MathEq,
            "notEqualTo" => Builtin::MathNe,
            _ => return None,
        });
    }
    if let Some(f) = i.strip_prefix(LOG) {
        return Some(match f {
            "equalTo" => Builtin::LogEq,
            "notEqualTo" => Builtin::LogNe,
            _ => return None,
        });
    }
    if let Some(f) = i.strip_prefix(STRING) {
        return Some(match f {
            "contains" => Builtin::StrContains,
            "containsIgnoringCase" => Builtin::StrContainsIgnCase,
            "equalIgnoringCase" => Builtin::StrEqIgnCase,
            "notEqualIgnoringCase" => Builtin::StrNeIgnCase,
            "startsWith" => Builtin::StrStarts,
            "endsWith" => Builtin::StrEnds,
            "greaterThan" => Builtin::StrGt,
            "lessThan" => Builtin::StrLt,
            "notGreaterThan" => Builtin::StrNotGt,
            "notLessThan" => Builtin::StrNotLt,
            "matches" => Builtin::StrMatches,
            "notMatches" => Builtin::StrNotMatches,
            _ => return None,
        });
    }
    None
}

fn eval_builtin(op: Builtin, s: &Term, o: &Term, b: &Binding) -> bool {
    let (s, o) = (apply(s, b), apply(o, b));
    match op {
        Builtin::LogEq => s == o,
        Builtin::LogNe => s != o,
        Builtin::StrContains
        | Builtin::StrStarts
        | Builtin::StrEnds
        | Builtin::StrGt
        | Builtin::StrLt
        | Builtin::StrNotGt
        | Builtin::StrNotLt
        | Builtin::StrMatches
        | Builtin::StrNotMatches
        | Builtin::StrContainsIgnCase
        | Builtin::StrEqIgnCase
        | Builtin::StrNeIgnCase => {
            let (Some(x), Some(y)) = (lex(&s), lex(&o)) else { return false };
            match op {
                Builtin::StrContains => x.contains(y),
                Builtin::StrStarts => x.starts_with(y),
                Builtin::StrEnds => x.ends_with(y),
                Builtin::StrGt => x > y,
                Builtin::StrLt => x < y,
                Builtin::StrNotGt => x <= y,
                Builtin::StrNotLt => x >= y,
                Builtin::StrMatches => regex::Regex::new(y).map(|re| re.is_match(x)).unwrap_or(false),
                Builtin::StrNotMatches => regex::Regex::new(y).map(|re| !re.is_match(x)).unwrap_or(false),
                Builtin::StrContainsIgnCase => x.to_lowercase().contains(&y.to_lowercase()),
                Builtin::StrEqIgnCase => x.to_lowercase() == y.to_lowercase(),
                Builtin::StrNeIgnCase => x.to_lowercase() != y.to_lowercase(),
                _ => unreachable!(),
            }
        }
        _ => {
            let (Some(x), Some(y)) = (num(&s), num(&o)) else { return false };
            match op {
                Builtin::Gt => x > y,
                Builtin::Lt => x < y,
                Builtin::NotGt => x <= y,
                Builtin::NotLt => x >= y,
                Builtin::MathEq => x == y,
                Builtin::MathNe => x != y,
                _ => unreachable!(),
            }
        }
    }
}

/// Bidirectional binary builtins over a SINGLE subject term (not a `( … )` list): either
/// side may be the unknown, and evaluating binds it.
#[derive(Clone, Copy)]
enum Bidi {
    LogUri, // log:uri — IRI ↔ its text as an xsd:string (either direction)
}

fn binder_builtin(p: &Term) -> Option<Bidi> {
    let Term::Iri(i) = p else { return None };
    match i.strip_prefix(LOG) {
        Some("uri") => Some(Bidi::LogUri),
        _ => None,
    }
}

/// Evaluate a bidirectional builtin: compute the bound side, unify with the other (binding
/// a variable or filtering on equality). `None` ⇒ the premise fails for this binding.
fn eval_binder(op: Bidi, s: &Term, o: &Term, b: Binding) -> Option<Binding> {
    let (sv, ov) = (apply(s, &b), apply(o, &b));
    let mut nb = b;
    match op {
        Bidi::LogUri => match (&sv, &ov) {
            // forward: IRI subject → its text as a string literal
            (Term::Iri(i), _) => {
                let lit = Term::Lit(i.clone(), "http://www.w3.org/2001/XMLSchema#string".into(), None);
                if unify_term(o, &lit, &mut nb) {
                    Some(nb)
                } else {
                    None
                }
            }
            // reverse: string object → the IRI it names
            (Term::Var(_), Term::Lit(v, _, _)) => {
                if unify_term(s, &Term::Iri(v.clone()), &mut nb) {
                    Some(nb)
                } else {
                    None
                }
            }
            _ => None,
        },
    }
}

#[derive(Clone, Copy)]
enum ListGen {
    Member,  // ?list list:member ?x
    In,      // ?x list:in ?list
    Iterate, // ?list list:iterate (?index ?value) — 0-based, one binding per member
}

fn list_generator(p: &Term) -> Option<ListGen> {
    let Term::Iri(i) = p else { return None };
    match i.strip_prefix(LIST) {
        Some("member") => Some(ListGen::Member),
        Some("in") => Some(ListGen::In),
        Some("iterate") => Some(ListGen::Iterate),
        _ => None,
    }
}

/// `log:includes` (→ `Some(false)`) / `log:notIncludes` (→ `Some(true)`, negation as failure).
fn scoped_negation(p: &Term) -> Option<bool> {
    let Term::Iri(i) = p else { return None };
    match i.strip_prefix(LOG) {
        Some("includes") => Some(false),
        Some("notIncludes") => Some(true),
        _ => None,
    }
}

/// The lexical string of a literal term (for `string:` builtins).
fn lex(t: &Term) -> Option<&str> {
    match t {
        Term::Lit(v, _, _) => Some(v.as_str()),
        _ => None,
    }
}

/// RFC 3986 percent-encoding as `string:encodeForUri` computes it: every UTF-8 byte
/// outside the *unreserved* set (`ALPHA / DIGIT / "-" / "." / "_" / "~"`, RFC 3986
/// §2.3) becomes `%XX` with uppercase hex (§2.1's canonical form). This is XPath
/// `fn:encode-for-uri` / SPARQL `ENCODE_FOR_URI`, the strictest of the encoding
/// flavours — reserved delimiters (`/ ? # & = :` …) and spaces are all encoded, so
/// the result is safe to splice into ANY URI component and the encoding is
/// injective (no input can smuggle a delimiter past it).
///
/// Public (not just the builtin's engine room) so callers that must mint the same
/// IRIs OUTSIDE a reasoner run — e.g. `sparq-solid`'s session-side pair-principal
/// minting — share one definition instead of re-implementing it.
///
/// # Examples
///
/// ```
/// use sparq_reason::n3::encode_for_uri;
/// assert_eq!(encode_for_uri("AZaz09-._~"), "AZaz09-._~"); // unreserved: untouched
/// assert_eq!(encode_for_uri("a&b=c"), "a%26b%3Dc");       // delimiters: encoded
/// assert_eq!(encode_for_uri("café"), "caf%C3%A9");        // UTF-8 bytes, uppercase hex
/// ```
pub fn encode_for_uri(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// The numeric value of a literal term (for `math:` builtins).
fn num(t: &Term) -> Option<f64> {
    match t {
        Term::Lit(v, _, _) => match v.as_str() {
            "INF" | "+INF" => Some(f64::INFINITY),
            "-INF" => Some(f64::NEG_INFINITY),
            "NaN" => Some(f64::NAN),
            _ => v.parse::<f64>().ok(),
        },
        _ => None,
    }
}

/// EXACT numeric tower for the arithmetic `math:` builtins, matching EYE's
/// Prolog rational arithmetic on the cases the suites exercise: integers and
/// decimals compute exactly (scaled i128), doubles (an `e` exponent or
/// INF/NaN) in f64. Strings coerce by lexical shape — `("2.7" "2")
/// math:difference 0.7` must hold EXACTLY (f64 gives 0.700…0002).
#[derive(Clone, Copy, Debug)]
enum NumVal {
    Int(i128),
    /// mantissa, scale: value = mantissa / 10^scale.
    Dec(i128, u32),
    F64(f64),
}

fn numval(t: &Term) -> Option<NumVal> {
    let Term::Lit(v, _, _) = t else { return None };
    let v = v.trim();
    match v {
        "INF" | "+INF" => return Some(NumVal::F64(f64::INFINITY)),
        "-INF" => return Some(NumVal::F64(f64::NEG_INFINITY)),
        "NaN" => return Some(NumVal::F64(f64::NAN)),
        _ => {}
    }
    if v.contains(['e', 'E']) {
        return v.parse::<f64>().ok().map(NumVal::F64);
    }
    if let Some((int, frac)) = v.split_once('.') {
        let digits = format!("{int}{frac}");
        if let Ok(m) = digits.parse::<i128>() {
            return Some(NumVal::Dec(m, frac.len() as u32));
        }
        return v.parse::<f64>().ok().map(NumVal::F64);
    }
    v.parse::<i128>()
        .ok()
        .map(NumVal::Int)
        .or_else(|| v.parse::<f64>().ok().map(NumVal::F64))
}

impl NumVal {
    fn to_f64(self) -> f64 {
        match self {
            NumVal::Int(i) => i as f64,
            NumVal::Dec(m, s) => m as f64 / 10f64.powi(s as i32),
            NumVal::F64(f) => f,
        }
    }
    /// Both values as scale-aligned (mantissa, scale), when exact.
    fn aligned(a: NumVal, b: NumVal) -> Option<(i128, i128, u32)> {
        let part = |v: NumVal| match v {
            NumVal::Int(i) => Some((i, 0u32)),
            NumVal::Dec(m, s) => Some((m, s)),
            NumVal::F64(_) => None,
        };
        let ((ma, sa), (mb, sb)) = (part(a)?, part(b)?);
        let s = sa.max(sb);
        let up = |m: i128, from: u32| -> Option<i128> {
            m.checked_mul(10i128.checked_pow(s - from)?)
        };
        Some((up(ma, sa)?, up(mb, sb)?, s))
    }
    fn eq(a: NumVal, b: NumVal) -> bool {
        match NumVal::aligned(a, b) {
            Some((x, y, _)) => x == y,
            None => a.to_f64() == b.to_f64(),
        }
    }
}

/// Drop trailing zero scale: Dec(700, 2) → Dec(7, 1); Dec(30, 1) → Dec(3, 0)
/// stays a Dec (decimal-typed inputs keep the decimal type, lexical `x.0`).
fn dec_norm(m: i128, s: u32) -> (i128, u32) {
    let (mut m, mut s) = (m, s);
    while s > 0 && m % 10 == 0 {
        m /= 10;
        s -= 1;
    }
    (m, s)
}

/// Render a NumVal as an N3 literal, EYE-style: Int → xsd:integer; Dec →
/// xsd:decimal with at least one fraction digit; F64 → integer when whole
/// (matching the old behavior), else decimal lexical.
fn numval_term(v: NumVal) -> Term {
    match v {
        NumVal::Int(i) => {
            Term::Lit(i.to_string(), "http://www.w3.org/2001/XMLSchema#integer".into(), None)
        }
        NumVal::Dec(m, s) => {
            let (m, s) = dec_norm(m, s);
            let neg = m < 0;
            let digits = m.unsigned_abs().to_string();
            let s = s as usize;
            let (int, frac) = if digits.len() > s {
                (digits[..digits.len() - s].to_string(), digits[digits.len() - s..].to_string())
            } else {
                ("0".to_string(), format!("{:0>width$}", digits, width = s))
            };
            let frac = if frac.is_empty() { "0".to_string() } else { frac };
            Term::Lit(
                format!("{}{int}.{frac}", if neg { "-" } else { "" }),
                "http://www.w3.org/2001/XMLSchema#decimal".into(),
                None,
            )
        }
        NumVal::F64(f) => number_term(f),
    }
}

/// Functional `math:`/`string:`/`list:`/`time:` builtins: the subject is a `( … )` list (or a
/// single value for the unary ops), and the object is computed.
#[derive(Clone, Copy)]
enum Func {
    // list-arg
    Sum,
    Difference,
    Product,
    Quotient,
    Remainder,       // math:remainder (a mod b)
    IntegerQuotient, // math:integerQuotient (floor(a/b))
    Max,
    Min,
    Exponentiation,
    Logarithm,   // (x base) math:logarithm log_base(x) — EYE: log(U)/log(V)
    Atan2,       // (x y) math:atan2 — EYE's eye.pl computes atan(x/y), NOT C atan2; we match
    MemberCount, // math:memberCount — list length, or distinct triple count of a formula
    Concat,      // string:concatenation
    Format,      // string:format — ( fmt args… ); %s/%d/%f/%% subset, else premise fails
    Scrape,      // string:scrape — ( str regex ); the FIRST capture group of the first match
    Length,      // list:length
    StrLength,   // string:length (Unicode scalar count)
    Replace,     // string:replace (regex): ( str pattern replacement ) string:replace ?out
    First,       // list:first
    Last,        // list:last
    Append,      // list:append — ( list… ) list:append ?out (first-class list result)
    Conjunction, // log:conjunction — merge a list of formulae into one formula
    Dtlit,       // log:dtlit — ( "lex" xsd:dt ) ↔ "lex"^^xsd:dt (both directions)
    // single-value-arg (string case mapping, Unicode-aware)
    LowerCase,    // string:lowerCase
    UpperCase,    // string:upperCase
    EncodeForUri, // string:encodeForUri — RFC 3986 percent-encoding (see [`encode_for_uri`])
    // single-value-arg (unary math)
    Negation,
    AbsoluteValue,
    Rounded,
    Floor,
    Ceiling,
    // single-value-arg trig/hyperbolic (forward direction only; see module doc)
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Sinh,
    Cosh,
    Tanh,
    Asinh,
    Acosh,
    Atanh,
    Degrees, // radians → degrees (x·180/π), matching eye.pl
    Radians, // degrees → radians (x·π/180)
    // single-value-arg (time: components of an xsd:dateTime)
    Year,
    Month,
    Day,
    Hours,
    Minutes,
    Seconds,
}

fn functional_builtin(p: &Term) -> Option<Func> {
    let Term::Iri(i) = p else { return None };
    if let Some(f) = i.strip_prefix(MATH) {
        return Some(match f {
            "sum" => Func::Sum,
            "difference" => Func::Difference,
            "product" => Func::Product,
            "quotient" => Func::Quotient,
            "remainder" => Func::Remainder,
            "integerQuotient" => Func::IntegerQuotient,
            "max" => Func::Max,
            "min" => Func::Min,
            "exponentiation" => Func::Exponentiation,
            "logarithm" => Func::Logarithm,
            "atan2" => Func::Atan2,
            "memberCount" => Func::MemberCount,
            "negation" => Func::Negation,
            "absoluteValue" => Func::AbsoluteValue,
            "rounded" => Func::Rounded,
            "floor" => Func::Floor,
            "ceiling" => Func::Ceiling,
            "sin" => Func::Sin,
            "cos" => Func::Cos,
            "tan" => Func::Tan,
            "asin" => Func::Asin,
            "acos" => Func::Acos,
            "atan" => Func::Atan,
            "sinh" => Func::Sinh,
            "cosh" => Func::Cosh,
            "tanh" => Func::Tanh,
            "asinh" => Func::Asinh,
            "acosh" => Func::Acosh,
            "atanh" => Func::Atanh,
            "degrees" => Func::Degrees,
            "radians" => Func::Radians,
            _ => return None,
        });
    }
    if let Some(f) = i.strip_prefix(TIME) {
        return Some(match f {
            "year" => Func::Year,
            "month" => Func::Month,
            "day" => Func::Day,
            "hours" => Func::Hours,
            "minutes" => Func::Minutes,
            "seconds" => Func::Seconds,
            _ => return None,
        });
    }
    if let Some(f) = i.strip_prefix(LOG) {
        return match f {
            "conjunction" => Some(Func::Conjunction),
            "dtlit" => Some(Func::Dtlit),
            _ => None,
        };
    }
    match (i.strip_prefix(STRING), i.strip_prefix(LIST)) {
        (Some("concatenation"), _) => Some(Func::Concat),
        (Some("format"), _) => Some(Func::Format),
        (Some("scrape"), _) => Some(Func::Scrape),
        (Some("length"), _) => Some(Func::StrLength),
        (Some("replace"), _) => Some(Func::Replace),
        (Some("lowerCase"), _) => Some(Func::LowerCase),
        (Some("upperCase"), _) => Some(Func::UpperCase),
        (Some("encodeForUri"), _) => Some(Func::EncodeForUri),
        (_, Some("length")) => Some(Func::Length),
        (_, Some("first")) => Some(Func::First),
        (_, Some("last")) => Some(Func::Last),
        (_, Some("append")) => Some(Func::Append),
        _ => None,
    }
}

/// Evaluate a functional builtin `(members) op object`: resolve the list members under `b`,
/// compute, then either bind the object variable to the result or filter if it is ground.
fn eval_functional(
    f: Func,
    subj: &Term,
    obj: &Term,
    facts: &FactIndex,
    b: Binding,
) -> Option<Binding> {
    const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";
    // log:dtlit needs the UNAPPLIED member terms: its reverse mode binds them by
    // decomposing a ground object literal into ( "lexical" datatype-IRI ).
    if let Func::Dtlit = f {
        let Term::List(members) = subj else { return None };
        if members.len() != 2 {
            return None;
        }
        let (m0, m1) = (apply(&members[0], &b), apply(&members[1], &b));
        let mut nb = b;
        return match (&m0, &m1) {
            // forward: ( "lex" xsd:dt ) → "lex"^^xsd:dt
            (Term::Lit(v, _, _), Term::Iri(dt)) => {
                let lit = Term::Lit(v.clone(), dt.clone(), None);
                unify_term(obj, &lit, &mut nb).then_some(nb)
            }
            // reverse: decompose a ground object literal into its parts
            _ => {
                let Term::Lit(v, dt, _) = apply(obj, &nb) else { return None };
                (unify_term(&members[0], &Term::Lit(v.clone(), XSD_STRING.into(), None), &mut nb)
                    && unify_term(&members[1], &Term::Iri(dt.clone()), &mut nb))
                .then_some(nb)
            }
        };
    }
    // math:negation is bidirectional in EYE: `?x math:negation 3` solves
    // ?x = -3 (the one reverse mode the suites rely on).
    if matches!(f, Func::Negation) && !subj.is_ground() && !matches!(subj, Term::List(_)) {
        let s_applied = apply(subj, &b);
        if !s_applied.is_ground() {
            let o_applied = apply(obj, &b);
            if let Some(v) = numval(&o_applied) {
                let negated = match v {
                    NumVal::Int(i) => NumVal::Int(i.checked_neg()?),
                    NumVal::Dec(m, sc) => NumVal::Dec(m.checked_neg()?, sc),
                    NumVal::F64(x) => NumVal::F64(-x),
                };
                let mut nb = b;
                return unify_term(subj, &numval_term(negated), &mut nb).then_some(nb);
            }
            return None;
        }
    }
    // Arguments: the list members (rule-local structure, a data list walked
    // from the fact store, or `rdf:nil` = the empty list), else a singleton
    // for the unary (math:/string:/time:) ops.
    let subj_applied = apply(subj, &b);
    let resolved_list: Option<Vec<Term>> = match &subj_applied {
        // First-class list value (already substituted by `apply`).
        Term::List(ms) => Some(ms.clone()),
        // A data list written as rdf:first/rest triples, via a bound variable.
        _ => fact_list(&subj_applied, facts).map(|ms| ms.iter().map(|m| apply(m, &b)).collect()),
    };
    let was_list = resolved_list.is_some();
    // The list:-namespace ops are only defined ON lists.
    if matches!(f, Func::Length | Func::First | Func::Last | Func::Append) && !was_list {
        return None;
    }
    let args: Vec<Term> = match resolved_list {
        Some(members) => members,
        None => vec![subj_applied.clone()],
    };
    if args.is_empty()
        && !matches!(f, Func::Conjunction | Func::MemberCount | Func::Length | Func::Append)
    {
        return None;
    }
    let result: Term = match f {
        Func::Conjunction => {
            // Merge a list of formulae (EYE conjoin): duplicates deduped, order preserved.
            // The boolean literal `true` counts as the empty formula (EYE's conjoin(true)).
            let mut merged: Vec<[Term; 3]> = Vec::new();
            for a in &args {
                match a {
                    Term::Formula(ts) => {
                        for t in ts {
                            if !merged.contains(t) {
                                merged.push(t.clone());
                            }
                        }
                    }
                    Term::Lit(v, _, _) if v == "true" => {}
                    _ => return None,
                }
            }
            Term::Formula(merged)
        }
        Func::MemberCount => match &args[..] {
            // a `( … )` list: its length; a quoted formula: its DISTINCT triple count
            [Term::Formula(ts)] => {
                let distinct: FxHashSet<&[Term; 3]> = ts.iter().collect();
                number_term(distinct.len() as f64)
            }
            _ if was_list => number_term(args.len() as f64),
            _ => return None,
        },
        Func::Format => {
            // ( fmt args… ) string:format ?out — honest %s/%d/%f/%% subset; any other
            // directive (or an argument-count mismatch, which EYE throws on) fails.
            let fmt = lex(&args[0])?.to_string();
            let mut out = String::new();
            let mut argi = 1;
            let mut chars = fmt.chars();
            while let Some(c) = chars.next() {
                if c != '%' {
                    out.push(c);
                    continue;
                }
                match chars.next()? {
                    '%' => out.push('%'),
                    's' => {
                        out.push_str(lex(args.get(argi)?)?);
                        argi += 1;
                    }
                    'd' => {
                        let n = num(args.get(argi)?)?;
                        out.push_str(&(n as i64).to_string());
                        argi += 1;
                    }
                    'f' => {
                        let n = num(args.get(argi)?)?;
                        out.push_str(&format!("{n:.6}")); // C printf default precision
                        argi += 1;
                    }
                    _ => return None, // unsupported directive: fail, don't mangle
                }
            }
            if argi != args.len() {
                return None;
            }
            Term::Lit(out, XSD_STRING.into(), None)
        }
        Func::Scrape => {
            // ( str regex ) string:scrape ?out — first capture group of the first match.
            if args.len() != 2 {
                return None;
            }
            let re = regex::Regex::new(lex(&args[1])?).ok()?;
            let cap = re.captures(lex(&args[0])?)?.get(1)?.as_str().to_string();
            Term::Lit(cap, XSD_STRING.into(), None)
        }
        Func::Concat => {
            let mut s = String::new();
            for a in &args {
                match a {
                    Term::Lit(v, _, _) => s.push_str(v),
                    _ => return None,
                }
            }
            Term::Lit(s, "http://www.w3.org/2001/XMLSchema#string".into(), None)
        }
        Func::Length => number_term(args.len() as f64),
        Func::StrLength => number_term(lex(&args[0])?.chars().count() as f64),
        Func::LowerCase => {
            Term::Lit(lex(&args[0])?.to_lowercase(), "http://www.w3.org/2001/XMLSchema#string".into(), None)
        }
        Func::UpperCase => {
            Term::Lit(lex(&args[0])?.to_uppercase(), "http://www.w3.org/2001/XMLSchema#string".into(), None)
        }
        Func::EncodeForUri => {
            Term::Lit(encode_for_uri(lex(&args[0])?), "http://www.w3.org/2001/XMLSchema#string".into(), None)
        }
        Func::First => args.first()?.clone(),
        Func::Last => args.last()?.clone(),
        Func::Append => {
            // ( l1 l2 … ) list:append ?out — concatenate; every member must
            // itself be a list (first-class, or data first/rest structure).
            let mut merged: Vec<Term> = Vec::new();
            for a in &args {
                match a {
                    Term::List(ms) => merged.extend(ms.iter().cloned()),
                    other => merged.extend(fact_list(other, facts)?),
                }
            }
            Term::List(merged)
        }
        Func::Replace => {
            // ( str pattern replacement ) string:replace ?out — regex replace-all.
            if args.len() != 3 {
                return None;
            }
            let re = regex::Regex::new(lex(&args[1])?).ok()?;
            let out = re.replace_all(lex(&args[0])?, lex(&args[2])?).into_owned();
            Term::Lit(out, "http://www.w3.org/2001/XMLSchema#string".into(), None)
        }
        Func::Year | Func::Month | Func::Day | Func::Hours | Func::Minutes | Func::Seconds => {
            number_term(datetime_part(lex(&args[0])?, f)? as f64)
        }
        _ => {
            // Unary numeric/trig/time builtins take a DIRECT value, never a
            // `( … )` list — `(1) math:rounded ?x` must fail (cwm/EYE agree;
            // the suites assert it via :FAILURE rules).
            let unary = matches!(
                f,
                Func::Negation
                    | Func::AbsoluteValue
                    | Func::Rounded
                    | Func::Floor
                    | Func::Ceiling
                    | Func::Sin
                    | Func::Cos
                    | Func::Tan
                    | Func::Asin
                    | Func::Acos
                    | Func::Atan
                    | Func::Sinh
                    | Func::Cosh
                    | Func::Tanh
                    | Func::Asinh
                    | Func::Acosh
                    | Func::Atanh
                    | Func::Degrees
                    | Func::Radians
            );
            if unary && was_list {
                return None;
            }
            if let Some(exact) = eval_exact(f, &args) {
                exact
            } else {
                let nums: Vec<f64> = args.iter().map(num).collect::<Option<_>>()?;
                let two = |n: &[f64]| if n.len() == 2 { Some((n[0], n[1])) } else { None };
                let v = match f {
                    Func::Sum => nums.iter().sum(),
                    Func::Product => nums.iter().product(),
                    Func::Max => nums.iter().copied().fold(f64::NEG_INFINITY, f64::max),
                    Func::Min => nums.iter().copied().fold(f64::INFINITY, f64::min),
                    Func::Difference => {
                        let (a, b) = two(&nums)?;
                        a - b
                    }
                    Func::Quotient => {
                        let (a, b) = two(&nums)?;
                        if b == 0.0 {
                            return None;
                        }
                        a / b
                    }
                    Func::Exponentiation => {
                        let (a, b) = two(&nums)?;
                        a.powf(b)
                    }
                    Func::Logarithm => {
                        // (x base) → log_base(x); EYE computes log(U)/log(V).
                        let (x, base) = two(&nums)?;
                        if x <= 0.0 || base <= 0.0 || base == 1.0 {
                            return None;
                        }
                        x.ln() / base.ln()
                    }
                    Func::Atan2 => {
                        // EYE's eye.pl: `W is atan(U/V)` — deliberately matched (see module doc).
                        let (x, y) = two(&nums)?;
                        if y == 0.0 {
                            return None;
                        }
                        (x / y).atan()
                    }
                    Func::Remainder => {
                        let (a, b) = two(&nums)?;
                        if b == 0.0 {
                            return None;
                        }
                        a % b
                    }
                    Func::IntegerQuotient => {
                        let (a, b) = two(&nums)?;
                        if b == 0.0 {
                            return None;
                        }
                        (a / b).floor()
                    }
                    Func::Negation => -nums[0],
                    Func::AbsoluteValue => nums[0].abs(),
                    Func::Rounded => (nums[0] + 0.5).floor(), // round-half-UP (suite refs)
                    Func::Floor => nums[0].floor(),
                    Func::Ceiling => nums[0].ceil(),
                    Func::Sin => nums[0].sin(),
                    Func::Cos => nums[0].cos(),
                    Func::Tan => nums[0].tan(),
                    Func::Asin => nums[0].asin(),
                    Func::Acos => nums[0].acos(),
                    Func::Atan => nums[0].atan(),
                    Func::Sinh => nums[0].sinh(),
                    Func::Cosh => nums[0].cosh(),
                    Func::Tanh => nums[0].tanh(),
                    Func::Asinh => nums[0].asinh(),
                    Func::Acosh => nums[0].acosh(),
                    Func::Atanh => nums[0].atanh(),
                    Func::Degrees => nums[0] * 180.0 / std::f64::consts::PI,
                    Func::Radians => nums[0] * std::f64::consts::PI / 180.0,
                    _ => unreachable!(),
                };
                if v.is_nan() {
                    return None; // domain error (asin 2, acosh 0.5, …): the premise fails
                }
                number_term(v)
            }
        }
    };
    // Bind the object variable; if it is already GROUND, compare NUMERICALLY
    // when both sides are numbers (so `15.5` matches a computed `15.5e0` and
    // exact-decimal results match their reference lexical forms), else
    // structurally.
    let mut nb = b;
    let obj_applied = apply(obj, &nb);
    if obj_applied.is_ground() {
        if let (Some(x), Some(y)) = (numval(&obj_applied), numval(&result)) {
            return NumVal::eq(x, y).then_some(nb);
        }
    }
    if unify_term(obj, &result, &mut nb) {
        Some(nb)
    } else {
        None
    }
}

/// Exact evaluation of the arithmetic builtins when every argument is an
/// integer or decimal (scaled-i128): returns `None` to fall back to f64
/// (doubles involved, overflow, or a non-exact quotient). The RESULT TYPE
/// follows EYE: all-integer in → integer out; any decimal in → decimal out
/// (with at least one fraction digit in the lexical form).
fn eval_exact(f: Func, args: &[Term]) -> Option<Term> {
    let vals: Vec<NumVal> = args.iter().map(numval).collect::<Option<_>>()?;
    if vals.iter().any(|v| matches!(v, NumVal::F64(_))) {
        return None;
    }
    let any_dec = vals.iter().any(|v| matches!(v, NumVal::Dec(_, _)));
    let pair = || -> Option<(i128, i128, u32)> {
        if vals.len() == 2 {
            NumVal::aligned(vals[0], vals[1])
        } else {
            None
        }
    };
    let renorm = |m: i128, s: u32| -> NumVal {
        if any_dec {
            NumVal::Dec(m, s)
        } else {
            NumVal::Int(m) // s is 0 when no decimals were involved
        }
    };
    // Unary ops: value = m / 10^s; integer-valued results keep the input's
    // numeric type (decimal in → `x.0` out, matching the cwm references).
    let unary_int = |round: fn(i128, i128) -> i128| -> Option<NumVal> {
        let (m, s) = match vals[0] {
            NumVal::Int(i) => (i, 0u32),
            NumVal::Dec(m, s) => (m, s),
            NumVal::F64(_) => return None,
        };
        let pow = 10i128.checked_pow(s)?;
        let v = round(m, pow);
        Some(if s == 0 { NumVal::Int(v) } else { NumVal::Dec(v, 0) })
    };
    let out = match f {
        Func::Sum => {
            let mut acc = NumVal::Int(0);
            for &v in &vals {
                let (a, b, s) = NumVal::aligned(acc, v)?;
                acc = NumVal::Dec(a.checked_add(b)?, s);
            }
            let NumVal::Dec(m, s) = acc else { return None };
            renorm(m, s)
        }
        Func::Product => {
            let mut acc = NumVal::Int(1);
            for &v in &vals {
                let (ma, sa) = match acc {
                    NumVal::Int(i) => (i, 0),
                    NumVal::Dec(m, s) => (m, s),
                    NumVal::F64(_) => return None,
                };
                let (mb, sb) = match v {
                    NumVal::Int(i) => (i, 0),
                    NumVal::Dec(m, s) => (m, s),
                    NumVal::F64(_) => return None,
                };
                acc = NumVal::Dec(ma.checked_mul(mb)?, sa.checked_add(sb)?);
            }
            let NumVal::Dec(m, s) = acc else { return None };
            renorm(m, s)
        }
        Func::Difference => {
            let (a, b, s) = pair()?;
            renorm(a.checked_sub(b)?, s)
        }
        Func::Max | Func::Min => {
            let mut best = vals[0];
            for &v in &vals[1..] {
                let (a, b, _) = NumVal::aligned(best, v)?;
                let take = if matches!(f, Func::Max) { b > a } else { b < a };
                if take {
                    best = v;
                }
            }
            best
        }
        Func::Quotient => {
            // Long-divide to an exact decimal if one exists within i128 range.
            let (mut a, b, _) = pair()?;
            if b == 0 {
                return None;
            }
            let mut scale = 0u32;
            while a % b != 0 && scale < 34 {
                a = a.checked_mul(10)?;
                scale += 1;
            }
            if a % b != 0 {
                return None; // not exact — f64 fallback
            }
            if any_dec || scale > 0 {
                NumVal::Dec(a / b, scale)
            } else {
                NumVal::Int(a / b)
            }
        }
        Func::Remainder => {
            // Integer remainder only (Prolog rem); anything else → f64.
            match (vals[0], vals[1]) {
                (NumVal::Int(a), NumVal::Int(b)) if b != 0 => NumVal::Int(a % b),
                _ => return None,
            }
        }
        Func::IntegerQuotient => {
            let (a, b, _) = pair()?;
            if b == 0 {
                return None;
            }
            NumVal::Int(a.div_euclid(b))
        }
        Func::Negation => match vals[0] {
            NumVal::Int(i) => NumVal::Int(i.checked_neg()?),
            NumVal::Dec(m, s) => NumVal::Dec(m.checked_neg()?, s),
            NumVal::F64(_) => return None,
        },
        Func::AbsoluteValue => match vals[0] {
            NumVal::Int(i) => NumVal::Int(i.checked_abs()?),
            NumVal::Dec(m, s) => NumVal::Dec(m.checked_abs()?, s),
            NumVal::F64(_) => return None,
        },
        // round-half-UP: floor(x + 1/2) — what the suite references encode
        // (-2.5 → -2, 0.5 → 1, 2.5 → 3). rounded keeps the decimal TYPE
        // (`-3.0`), while floor/ceiling return integers — both per the cwm
        // reference outputs.
        Func::Rounded => unary_int(|m, pow| (m + pow / 2).div_euclid(pow))?,
        Func::Floor => match unary_int(|m, pow| m.div_euclid(pow))? {
            NumVal::Dec(m, _) => NumVal::Int(m),
            v => v,
        },
        Func::Ceiling => match unary_int(|m, pow| -((-m).div_euclid(pow)))? {
            NumVal::Dec(m, _) => NumVal::Int(m),
            v => v,
        },
        _ => return None, // trig/log/exponentiation: f64 path
    };
    Some(numval_term(out))
}

/// Extract a component of an `xsd:dateTime`/`xsd:date` lexical form for `time:` builtins.
/// Lexical: `[-]YYYY-MM-DD[Thh:mm:ss[.sss]][Z|±hh:mm]`.
fn datetime_part(s: &str, f: Func) -> Option<i64> {
    let (date, time) = s.split_once('T').unwrap_or((s, ""));
    let neg = date.starts_with('-');
    let mut dparts = date.trim_start_matches('-').split('-');
    match f {
        Func::Year => dparts.next()?.parse::<i64>().ok().map(|y| if neg { -y } else { y }),
        Func::Month => dparts.nth(1)?.parse().ok(),
        Func::Day => dparts.nth(2)?.parse().ok(),
        Func::Hours | Func::Minutes | Func::Seconds => {
            // strip any timezone (Z, +hh:mm, -hh:mm) — the time itself has no +/-/Z.
            let t = time.split(['+', '-', 'Z']).next().unwrap_or(time);
            let idx = match f {
                Func::Hours => 0,
                Func::Minutes => 1,
                Func::Seconds => 2,
                _ => unreachable!(),
            };
            let part = t.split(':').nth(idx)?;
            part.split('.').next().unwrap_or(part).parse().ok()
        }
        _ => None,
    }
}

/// Render an `f64` result as an N3 numeric literal (integer when whole, else decimal).
fn number_term(v: f64) -> Term {
    if v.fract() == 0.0 && v.abs() < 9.007e15 {
        Term::Lit((v as i64).to_string(), "http://www.w3.org/2001/XMLSchema#integer".into(), None)
    } else {
        Term::Lit(format!("{v}"), "http://www.w3.org/2001/XMLSchema#decimal".into(), None)
    }
}

/// Intern an N3 ground term into the dictionary.
fn intern(dict: &mut Dict, t: &Term) -> Result<Id, String> {
    Ok(match t {
        Term::Iri(i) => dict.intern_iri(i),
        Term::Lit(v, dt, lang) => dict.intern_lit(v, dt, lang.as_deref()),
        Term::Blank(b) => dict.intern_blank(b),
        Term::Var(_) | Term::Formula(_) => return Err("non-ground term in closure".into()),
        Term::List(_) => return Err("unexpanded list term in closure".into()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn closure(src: &str) -> (Dict, FxHashSet<[Id; 3]>) {
        let mut dict = Dict::new();
        let triples = reason_n3(&mut dict, src).unwrap();
        let set = triples.into_iter().collect();
        (dict, set)
    }
    fn id(dict: &Dict, iri: &str) -> Id {
        use oxrdf::{NamedNode, Term as OT};
        dict.lookup(&OT::NamedNode(NamedNode::new_unchecked(iri.to_string())))
    }
    fn has(dict: &Dict, set: &FxHashSet<[Id; 3]>, s: &str, p: &str, o: &str) -> bool {
        let (a, b, c) = (id(dict, s), id(dict, p), id(dict, o));
        a != 0 && b != 0 && c != 0 && set.contains(&[a, b, c])
    }

    #[test]
    fn proof_records_derivations() {
        // Chained rules: each derived triple should have a proof step naming its premise.
        let src = r#"
            @prefix : <http://ex/> .
            :Socrates a :Man .
            { ?x a :Man } => { ?x a :Mortal } .
            { ?x a :Mortal } => { ?x a :Being } .
        "#;
        let mut dict = Dict::new();
        let (_triples, proof) = reason_n3_proof(&mut dict, src).unwrap();
        // Two derivations: Socrates a Mortal (from Socrates a Man), Socrates a Being (from Mortal).
        assert_eq!(proof.len(), 2, "two derivation steps");
        let mortal = dict.intern_iri("http://ex/Mortal");
        let man = dict.intern_iri("http://ex/Man");
        let socrates = dict.intern_iri("http://ex/Socrates");
        let ty = dict.intern_iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type");
        let step = proof.iter().find(|s| s.conclusion == [socrates, ty, mortal]).expect("Mortal step");
        assert_eq!(step.premises, vec![[socrates, ty, man]], "Mortal derived from (Socrates a Man)");
    }

    #[test]
    fn simple_rule_socrates() {
        // The canonical N3 rule: every Man is Mortal.
        let src = r#"
            @prefix : <http://ex/> .
            :Socrates a :Man .
            { ?x a :Man } => { ?x a :Mortal } .
        "#;
        let (d, s) = closure(src);
        assert!(has(&d, &s, "http://ex/Socrates", "http://www.w3.org/1999/02/22-rdf-syntax-ns#type", "http://ex/Mortal"));
    }

    #[test]
    fn backward_rule_arrow() {
        // `{ conclusion } <= { premise }` is GOAL-DIRECTED (EYE semantics): it never fires
        // forward on its own; a forward rule whose premise needs the conclusion (here an
        // EYE-style query rule `{goal} => {goal}`) triggers the backward proof.
        let src = r#"
            @prefix : <http://ex/> .
            :Socrates a :Man .
            { ?x a :Mortal } <= { ?x a :Man } .
            { ?x a :Mortal } => { ?x a :Mortal } .
        "#;
        let (d, s) = closure(src);
        let ty = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
        assert!(has(&d, &s, "http://ex/Socrates", ty, "http://ex/Mortal"), "query rule drives backward proof");

        // Without a forward rule posing the goal, the backward rule must NOT materialize
        // anything — EYE outputs no `:Mortal` triple for this document either.
        let src_no_query = r#"
            @prefix : <http://ex/> .
            :Socrates a :Man .
            { ?x a :Mortal } <= { ?x a :Man } .
        "#;
        let (d2, s2) = closure(src_no_query);
        assert!(!has(&d2, &s2, "http://ex/Socrates", ty, "http://ex/Mortal"), "no goal, no backward firing");
    }

    #[test]
    fn backward_rule_builtin_premise() {
        // The eyereasoner/eye reasoning/backward case, inlined: the backward premise is a
        // PURE BUILTIN over the goal's variables — only evaluable once the query rule's
        // goal binds ?X/?Y (a forward reversal of `<=` derives nothing here).
        let src = r#"
            @prefix math: <http://www.w3.org/2000/10/swap/math#>.
            @prefix : <http://example.org/#>.
            { ?X :moreInterestingThan ?Y. } <= { ?X math:greaterThan ?Y. }.
            { 5 :moreInterestingThan 3. } => { 5 :moreInterestingThan 3. }.
        "#;
        let (mut d, s) = closure(src);
        let int = "http://www.w3.org/2001/XMLSchema#integer";
        let (five, three) = (d.intern_lit("5", int, None), d.intern_lit("3", int, None));
        assert!(
            s.contains(&[five, id(&d, "http://example.org/#moreInterestingThan"), three]),
            "5 :moreInterestingThan 3 proven goal-directed"
        );
    }

    #[test]
    fn backward_rule_chained_and_base_case() {
        // Backward rules chaining through other backward rules, plus a `<= true` base case.
        let src = r#"
            @prefix : <http://ex/> .
            :rex a :Dog .
            { ?x a :Animal } <= { ?x a :Dog } .
            { ?x :needs :food } <= { ?x a :Animal } .
            { :water a :Necessity } <= true .
            { ?x :needs :food } => { ?x :gets :food } .
            { :water a :Necessity } => { :water :is :necessary } .
        "#;
        let (d, s) = closure(src);
        assert!(has(&d, &s, "http://ex/rex", "http://ex/gets", "http://ex/food"), "two-step backward chain");
        assert!(has(&d, &s, "http://ex/water", "http://ex/is", "http://ex/necessary"), "<= true base case");
        // The intermediate backward conclusions are NOT materialized.
        let ty = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
        assert!(!has(&d, &s, "http://ex/rex", ty, "http://ex/Animal"), "backward conclusions stay virtual");
    }

    #[test]
    fn backward_rule_recursion_bounded() {
        // A self-referential backward rule must not hang: the depth bound cuts the cycle
        // and the engine still terminates (deriving nothing for the unprovable goal).
        let src = r#"
            @prefix : <http://ex/> .
            :seed :p :o .
            { ?x :loops ?y } <= { ?y :loops ?x } .
            { ?a :loops ?b } => { ?a :looped ?b } .
        "#;
        let (d, s) = closure(src);
        assert!(has(&d, &s, "http://ex/seed", "http://ex/p", "http://ex/o"), "facts survive");
        assert!(
            !s.iter().any(|[_, p, _]| *p == id(&d, "http://ex/looped")),
            "cyclic backward goal proves nothing"
        );
    }

    #[test]
    fn transitive_via_rule() {
        // Define transitivity with an N3 rule and close a chain.
        let src = r#"
            @prefix : <http://ex/> .
            :a :before :b . :b :before :c . :c :before :d .
            { ?x :before ?y . ?y :before ?z } => { ?x :before ?z } .
        "#;
        let (d, s) = closure(src);
        assert!(has(&d, &s, "http://ex/a", "http://ex/before", "http://ex/c"));
        assert!(has(&d, &s, "http://ex/a", "http://ex/before", "http://ex/d"));
        assert!(has(&d, &s, "http://ex/b", "http://ex/before", "http://ex/d"));
    }

    #[test]
    fn functional_math_sum_and_product() {
        // (?a ?b) math:sum ?s computes ?s; chained with product.
        let src = r#"
            @prefix : <http://ex/> .
            @prefix math: <http://www.w3.org/2000/10/swap/math#> .
            :rect :width 4 ; :height 5 .
            { ?r :width ?w . ?r :height ?h . (?w ?h) math:product ?area } => { ?r :area ?area } .
            { ?r :width ?w . ?r :height ?h . (?w ?h) math:sum ?half } => { ?r :perimeterHalf ?half } .
        "#;
        let (mut d, s) = closure(src);
        let int = "http://www.w3.org/2001/XMLSchema#integer";
        let area_id = d.intern_lit("20", int, None); // 4*5
        let half_id = d.intern_lit("9", int, None); // 4+5
        assert!(s.contains(&[id(&d, "http://ex/rect"), id(&d, "http://ex/area"), area_id]), "math:product 4*5=20");
        assert!(s.contains(&[id(&d, "http://ex/rect"), id(&d, "http://ex/perimeterHalf"), half_id]), "math:sum 4+5=9");
    }

    #[test]
    fn functional_string_length() {
        // ?w string:length ?n  — Unicode scalar count (T12).
        let src = r#"
            @prefix : <http://ex/> .
            @prefix string: <http://www.w3.org/2000/10/swap/string#> .
            :a :word "hello" .
            { ?x :word ?w . ?w string:length ?n } => { ?x :wordLen ?n } .
        "#;
        let (mut d, s) = closure(src);
        let five = d.intern_lit("5", "http://www.w3.org/2001/XMLSchema#integer", None);
        assert!(s.contains(&[id(&d, "http://ex/a"), id(&d, "http://ex/wordLen"), five]), "string:length(hello)=5");
    }

    #[test]
    fn string_matches_regex() {
        let src = r#"
            @prefix : <http://ex/> .
            @prefix string: <http://www.w3.org/2000/10/swap/string#> .
            :a :code "AB-123" . :b :code "xyz" .
            { ?x :code ?c . ?c string:matches "^[A-Z]+-[0-9]+$" } => { ?x a :Valid } .
        "#;
        let (d, s) = closure(src);
        assert!(has(&d, &s, "http://ex/a", "http://www.w3.org/1999/02/22-rdf-syntax-ns#type", "http://ex/Valid"));
        assert!(!has(&d, &s, "http://ex/b", "http://www.w3.org/1999/02/22-rdf-syntax-ns#type", "http://ex/Valid"));
    }

    #[test]
    fn string_replace_regex() {
        let src = r#"
            @prefix : <http://ex/> .
            @prefix string: <http://www.w3.org/2000/10/swap/string#> .
            :a :raw "a1b2c3" .
            { ?x :raw ?r . ( ?r "[0-9]" "_" ) string:replace ?out } => { ?x :clean ?out } .
        "#;
        let (mut d, s) = closure(src);
        let expected = d.intern_lit("a_b_c_", "http://www.w3.org/2001/XMLSchema#string", None);
        assert!(s.contains(&[id(&d, "http://ex/a"), id(&d, "http://ex/clean"), expected]), "replace [0-9]->_ = a_b_c_");
    }

    #[test]
    fn functional_math_remainder_intquotient() {
        let src = r#"
            @prefix : <http://ex/> .
            @prefix math: <http://www.w3.org/2000/10/swap/math#> .
            :x :v 17 .
            { ?x :v ?v . (?v 5) math:remainder ?r . (?v 5) math:integerQuotient ?q } => { ?x :rem ?r . ?x :quot ?q } .
        "#;
        let (mut d, s) = closure(src);
        let int = "http://www.w3.org/2001/XMLSchema#integer";
        assert!(s.contains(&[id(&d, "http://ex/x"), id(&d, "http://ex/rem"), d.intern_lit("2", int, None)]), "17 mod 5 = 2");
        assert!(s.contains(&[id(&d, "http://ex/x"), id(&d, "http://ex/quot"), d.intern_lit("3", int, None)]), "17 div 5 = 3");
    }

    #[test]
    fn functional_list_first_last() {
        // ( … ) list:first / list:last over a rule-local collection (T12).
        let src = r#"
            @prefix : <http://ex/> .
            @prefix list: <http://www.w3.org/2000/10/swap/list#> .
            { ( :a :b :c ) list:first ?f . ( :a :b :c ) list:last ?z } => { :s :first ?f . :s :last ?z } .
        "#;
        let (d, s) = closure(src);
        assert!(has(&d, &s, "http://ex/s", "http://ex/first", "http://ex/a"));
        assert!(has(&d, &s, "http://ex/s", "http://ex/last", "http://ex/c"));
    }

    #[test]
    fn path_syntax_forward() {
        // `?x!:mother :knows ?f` — the path ?x!:mother is the mother node; desugars to
        // (?x :mother _m)(_m :knows ?f).
        let src = r#"
            @prefix : <http://ex/> .
            :alice :mother :mary .
            :mary :knows :bob .
            { ?x!:mother :knows ?f } => { ?x :motherKnows ?f } .
        "#;
        let (d, s) = closure(src);
        assert!(has(&d, &s, "http://ex/alice", "http://ex/motherKnows", "http://ex/bob"), "forward path !");
    }

    #[test]
    fn path_syntax_backward() {
        // `?x^:mother` — the subject whose :mother is ?x (i.e. ?x's children).
        let src = r#"
            @prefix : <http://ex/> .
            :alice :mother :mary .
            { ?child :mother ?m . ?m^:mother :name ?cn } => { ?m :hasChildNamed ?cn } .
            :alice :name "Alice" .
        "#;
        // ?m^:mother is a child of ?m; simpler: verify ^ desugars to a backward triple.
        let (d, s) = closure(src);
        // ?m=mary: mary^:mother = alice (alice :mother mary); alice :name "Alice"
        // ⊢ mary :hasChildNamed "Alice".
        assert!(
            s.iter().any(|[a, p, _]| *a == id(&d, "http://ex/mary") && *p == id(&d, "http://ex/hasChildNamed")),
            "backward path ^ derived mary :hasChildNamed"
        );
    }

    #[test]
    fn scoped_negation_not_includes() {
        // log:notIncludes — negation as failure: a Person with no recorded email is :NoEmail.
        let src = r#"
            @prefix : <http://ex/> .
            @prefix log: <http://www.w3.org/2000/10/swap/log#> .
            :alice a :Person .
            :bob a :Person .
            :bob :hasEmail "bob@x" .
            { ?x a :Person . { } log:notIncludes { ?x :hasEmail ?e } } => { ?x a :NoEmail } .
        "#;
        let (d, s) = closure(src);
        let ty = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
        assert!(has(&d, &s, "http://ex/alice", ty, "http://ex/NoEmail"), "alice has no email → NoEmail");
        assert!(!has(&d, &s, "http://ex/bob", ty, "http://ex/NoEmail"), "bob has email → excluded");
    }

    #[test]
    fn string_contains_filter() {
        let src = r#"
            @prefix : <http://ex/> .
            @prefix string: <http://www.w3.org/2000/10/swap/string#> .
            :a :label "hello world" .
            :b :label "goodbye" .
            { ?x :label ?l . ?l string:contains "world" } => { ?x a :Matched } .
        "#;
        let (d, s) = closure(src);
        let ty = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
        assert!(has(&d, &s, "http://ex/a", ty, "http://ex/Matched"), "string:contains match");
        assert!(!has(&d, &s, "http://ex/b", ty, "http://ex/Matched"), "non-match excluded");
    }

    #[test]
    fn list_member_generator() {
        let src = r#"
            @prefix : <http://ex/> .
            @prefix list: <http://www.w3.org/2000/10/swap/list#> .
            :s :p :o .
            { ( :a :b :c ) list:member ?x } => { ?x a :Listed } .
        "#;
        let (d, s) = closure(src);
        let ty = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
        for m in ["a", "b", "c"] {
            assert!(has(&d, &s, &format!("http://ex/{m}"), ty, "http://ex/Listed"), "list:member {m}");
        }
    }

    #[test]
    fn time_components_and_unary_math() {
        let src = r#"
            @prefix : <http://ex/> .
            @prefix time: <http://www.w3.org/2000/10/swap/time#> .
            @prefix math: <http://www.w3.org/2000/10/swap/math#> .
            :e :when "2024-03-15T10:30:45"^^<http://www.w3.org/2001/XMLSchema#dateTime> .
            :e :temp -5 .
            { ?x :when ?d . ?d time:year ?y . ?d time:month ?mo . ?d time:day ?dd } => { ?x :y ?y ; :mo ?mo ; :dd ?dd } .
            { ?x :temp ?t . ?t math:absoluteValue ?a } => { ?x :absTemp ?a } .
        "#;
        let (mut d, s) = closure(src);
        let int = "http://www.w3.org/2001/XMLSchema#integer";
        let lit = |d: &mut Dict, n: &str| d.intern_lit(n, int, None);
        let (y, mo, dd, a) = (lit(&mut d, "2024"), lit(&mut d, "3"), lit(&mut d, "15"), lit(&mut d, "5"));
        let e = id(&d, "http://ex/e");
        assert!(s.contains(&[e, id(&d, "http://ex/y"), y]), "time:year = 2024");
        assert!(s.contains(&[e, id(&d, "http://ex/mo"), mo]), "time:month = 3");
        assert!(s.contains(&[e, id(&d, "http://ex/dd"), dd]), "time:day = 15");
        assert!(s.contains(&[e, id(&d, "http://ex/absTemp"), a]), "math:absoluteValue(-5) = 5");
    }

    #[test]
    fn math_max_and_list_length() {
        let src = r#"
            @prefix : <http://ex/> .
            @prefix math: <http://www.w3.org/2000/10/swap/math#> .
            @prefix list: <http://www.w3.org/2000/10/swap/list#> .
            :d :seed "x" .
            { ( 3 7 2 ) math:max ?m . ( 3 7 2 ) list:length ?n } => { :d :maxVal ?m ; :count ?n } .
        "#;
        let (mut d, s) = closure(src);
        let int = "http://www.w3.org/2001/XMLSchema#integer";
        let seven = d.intern_lit("7", int, None);
        let three = d.intern_lit("3", int, None);
        assert!(s.contains(&[id(&d, "http://ex/d"), id(&d, "http://ex/maxVal"), seven]), "math:max = 7");
        assert!(s.contains(&[id(&d, "http://ex/d"), id(&d, "http://ex/count"), three]), "list:length = 3");
    }

    #[test]
    fn list_member_with_bound_variables() {
        // list:member resolves members THROUGH the current binding: ( ?v :extra ) with ?v
        // joined from data generates one binding per (substituted) member.
        let src = r#"
            @prefix : <http://ex/> .
            @prefix list: <http://www.w3.org/2000/10/swap/list#> .
            :s :p :o1 . :s :p :o2 .
            { :s :p ?v . ( ?v :extra ) list:member ?m } => { ?m a :Seen } .
        "#;
        let (d, s) = closure(src);
        let ty = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
        for m in ["o1", "o2", "extra"] {
            assert!(has(&d, &s, &format!("http://ex/{m}"), ty, "http://ex/Seen"), "list:member {m}");
        }
    }

    #[test]
    fn list_in_generator() {
        // ?x list:in ( … ) — the inverse direction of list:member.
        let src = r#"
            @prefix : <http://ex/> .
            @prefix list: <http://www.w3.org/2000/10/swap/list#> .
            :seed :p :o .
            { ?x list:in ( :a :b ) } => { ?x a :InList } .
        "#;
        let (d, s) = closure(src);
        let ty = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
        assert!(has(&d, &s, "http://ex/a", ty, "http://ex/InList"), "list:in a");
        assert!(has(&d, &s, "http://ex/b", ty, "http://ex/InList"), "list:in b");
        assert!(!has(&d, &s, "http://ex/o", ty, "http://ex/InList"), "non-member excluded");
    }

    #[test]
    fn string_lower_upper_case() {
        // string:lowerCase / string:upperCase — functional, Unicode-aware case mapping.
        let src = r#"
            @prefix : <http://ex/> .
            @prefix string: <http://www.w3.org/2000/10/swap/string#> .
            :a :word "HeLLo Wörld" .
            { ?x :word ?w . ?w string:lowerCase ?l . ?w string:upperCase ?u } => { ?x :lower ?l ; :upper ?u } .
        "#;
        let (mut d, s) = closure(src);
        let xs = "http://www.w3.org/2001/XMLSchema#string";
        let lo = d.intern_lit("hello wörld", xs, None);
        let up = d.intern_lit("HELLO WÖRLD", xs, None);
        assert!(s.contains(&[id(&d, "http://ex/a"), id(&d, "http://ex/lower"), lo]), "string:lowerCase");
        assert!(s.contains(&[id(&d, "http://ex/a"), id(&d, "http://ex/upper"), up]), "string:upperCase");
    }

    #[test]
    fn string_encode_for_uri() {
        // string:encodeForUri — RFC 3986 percent-encoding (fn:encode-for-uri set):
        // unreserved untouched, delimiters/space encoded, non-ASCII as UTF-8 bytes
        // with uppercase hex. Object-ground use acts as an equality filter.
        let src = r#"
            @prefix : <http://ex/> .
            @prefix string: <http://www.w3.org/2000/10/swap/string#> .
            :a :raw "AZaz09-._~" .
            :b :raw "https://alice.ex/card#me&client=x" .
            :c :raw "café déjà" .
            { ?x :raw ?r . ?r string:encodeForUri ?e } => { ?x :enc ?e } .
            { ?x :raw ?r . ?r string:encodeForUri "AZaz09-._~" } => { ?x a :Unchanged } .
        "#;
        let (mut d, s) = closure(src);
        let xs = "http://www.w3.org/2001/XMLSchema#string";
        let ty = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
        let ea = d.intern_lit("AZaz09-._~", xs, None);
        let eb = d.intern_lit("https%3A%2F%2Falice.ex%2Fcard%23me%26client%3Dx", xs, None);
        let ec = d.intern_lit("caf%C3%A9%20d%C3%A9j%C3%A0", xs, None);
        assert!(s.contains(&[id(&d, "http://ex/a"), id(&d, "http://ex/enc"), ea]), "unreserved untouched");
        assert!(s.contains(&[id(&d, "http://ex/b"), id(&d, "http://ex/enc"), eb]), "delimiters encoded");
        assert!(s.contains(&[id(&d, "http://ex/c"), id(&d, "http://ex/enc"), ec]), "UTF-8 bytes, uppercase hex");
        assert!(has(&d, &s, "http://ex/a", ty, "http://ex/Unchanged"), "ground object: filter passes");
        assert!(!has(&d, &s, "http://ex/b", ty, "http://ex/Unchanged"), "ground object: filter rejects");
        // the helper directly (shared with sparq-solid's session-side pair minting)
        assert_eq!(super::encode_for_uri(" %"), "%20%25", "space and percent themselves");
        assert_eq!(super::encode_for_uri("नमस्ते"), "%E0%A4%A8%E0%A4%AE%E0%A4%B8%E0%A5%8D%E0%A4%A4%E0%A5%87");
    }

    #[test]
    fn string_not_matches_regex() {
        // string:notMatches — negated regex filter.
        let src = r#"
            @prefix : <http://ex/> .
            @prefix string: <http://www.w3.org/2000/10/swap/string#> .
            :a :code "abc" . :b :code "123" .
            { ?x :code ?c . ?c string:notMatches "^[0-9]+$" } => { ?x a :NonNumeric } .
        "#;
        let (d, s) = closure(src);
        let ty = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
        assert!(has(&d, &s, "http://ex/a", ty, "http://ex/NonNumeric"), "abc does not match digits");
        assert!(!has(&d, &s, "http://ex/b", ty, "http://ex/NonNumeric"), "123 matches → excluded");
    }

    #[test]
    fn log_includes_ground_formula() {
        // Ground-formula containment: { f } log:includes { pattern } matches the pattern
        // against the SUBJECT formula's triples (not the store) and binds its variables.
        let src = r#"
            @prefix : <http://ex/> .
            @prefix log: <http://www.w3.org/2000/10/swap/log#> .
            :x :q :y .
            { { :a :p :b . :c :p :d } log:includes { ?s :p :b } } => { ?s a :FoundInFormula } .
            { { :a :p :b } log:includes { :x :q :y } } => { :s a :StoreLeak } .
        "#;
        let (d, s) = closure(src);
        let ty = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
        assert!(has(&d, &s, "http://ex/a", ty, "http://ex/FoundInFormula"), "?s bound inside the formula");
        assert!(!has(&d, &s, "http://ex/c", ty, "http://ex/FoundInFormula"), ":c :p :d does not match ?s :p :b");
        // :x :q :y IS in the store but NOT in the subject formula — must not leak through.
        assert!(!has(&d, &s, "http://ex/s", ty, "http://ex/StoreLeak"), "formula scope must not see the store");
    }

    #[test]
    fn log_not_includes_ground_formula() {
        // notIncludes over a ground subject formula: containment failure, scoped to it.
        let src = r#"
            @prefix : <http://ex/> .
            @prefix log: <http://www.w3.org/2000/10/swap/log#> .
            :seed :p :o .
            { { :a :p :b } log:notIncludes { :a :p :c } } => { :s :notInc :yes } .
            { { :a :p :b } log:notIncludes { :a :p :b } } => { :s :bad :yes } .
        "#;
        let (d, s) = closure(src);
        assert!(has(&d, &s, "http://ex/s", "http://ex/notInc", "http://ex/yes"), "absent triple → notIncludes holds");
        assert!(!has(&d, &s, "http://ex/s", "http://ex/bad", "http://ex/yes"), "present triple → notIncludes fails");
    }

    #[test]
    fn trig_exact_values() {
        // The EYE trig family at exact points: sin 0 = 0, cos 0 = 1, sin(π/2) = 1, tan 0 = 0,
        // acos 1 = 0, sinh 0 = 0, cosh 0 = 1, tanh 0 = 0 (all exact in IEEE f64).
        let src = r#"
            @prefix : <http://ex/> .
            @prefix math: <http://www.w3.org/2000/10/swap/math#> .
            :t :seed 0 .
            { ?x :seed ?z .
              ?z math:sin ?s . ?z math:cos ?c . ?z math:tan ?t .
              ?z math:sinh ?sh . ?z math:cosh ?ch . ?z math:tanh ?th .
              1 math:acos ?ac . 1.5707963267948966 math:sin ?shalf }
            => { ?x :sin ?s ; :cos ?c ; :tan ?t ; :sinh ?sh ; :cosh ?ch ; :tanh ?th ;
                    :acos1 ?ac ; :sinHalfPi ?shalf } .
        "#;
        let (mut d, s) = closure(src);
        let int = "http://www.w3.org/2001/XMLSchema#integer";
        let zero = d.intern_lit("0", int, None);
        let one = d.intern_lit("1", int, None);
        let t = id(&d, "http://ex/t");
        for (p, v) in [
            ("sin", zero), ("cos", one), ("tan", zero), ("sinh", zero), ("cosh", one),
            ("tanh", zero), ("acos1", zero), ("sinHalfPi", one),
        ] {
            assert!(s.contains(&[t, id(&d, &format!("http://ex/{p}")), v]), "math:{p} exact value");
        }
    }

    #[test]
    fn trig_domain_error_fails_premise() {
        // asin 2 is NaN — the premise must FAIL (no binding), not emit a NaN literal.
        let src = r#"
            @prefix : <http://ex/> .
            @prefix math: <http://www.w3.org/2000/10/swap/math#> .
            :seed :p :o .
            { 2 math:asin ?x } => { :s :bad ?x } .
        "#;
        let (d, s) = closure(src);
        assert!(!s.iter().any(|[a, p, _]| *a == id(&d, "http://ex/s") && *p == id(&d, "http://ex/bad")));
    }

    #[test]
    fn degrees_radians_roundtrip() {
        // π rad math:degrees 180; 180° math:radians π (both exact in f64, same ops as eye.pl).
        let src = r#"
            @prefix : <http://ex/> .
            @prefix math: <http://www.w3.org/2000/10/swap/math#> .
            :a :rad 3.141592653589793 . :a :deg 180 .
            { ?x :rad ?r . ?r math:degrees ?d } => { ?x :inDegrees ?d } .
            { ?x :deg ?g . ?g math:radians ?r2 } => { ?x :inRadians ?r2 } .
        "#;
        let (mut d, s) = closure(src);
        let deg = d.intern_lit("180", "http://www.w3.org/2001/XMLSchema#integer", None);
        let rad = d.intern_lit("3.141592653589793", "http://www.w3.org/2001/XMLSchema#decimal", None);
        let a = id(&d, "http://ex/a");
        assert!(s.contains(&[a, id(&d, "http://ex/inDegrees"), deg]), "π rad = 180°");
        assert!(s.contains(&[a, id(&d, "http://ex/inRadians"), rad]), "180° = π rad");
    }

    #[test]
    fn logarithm_and_atan2() {
        // (x base) math:logarithm log_base(x); (x y) math:atan2 = atan(x/y) (EYE's impl).
        let src = r#"
            @prefix : <http://ex/> .
            @prefix math: <http://www.w3.org/2000/10/swap/math#> .
            :seed :p :o .
            { (8 2) math:logarithm ?l . (1024 2) math:logarithm ?k . (0 1) math:atan2 ?a }
            => { :s :log8 ?l ; :log1024 ?k ; :atan ?a } .
            { (5 1) math:logarithm ?bad } => { :s :badLog ?bad } .
        "#;
        let (mut d, s) = closure(src);
        let int = "http://www.w3.org/2001/XMLSchema#integer";
        let st = id(&d, "http://ex/s");
        let three = d.intern_lit("3", int, None);
        let ten = d.intern_lit("10", int, None);
        let zero = d.intern_lit("0", int, None);
        assert!(s.contains(&[st, id(&d, "http://ex/log8"), three]), "log_2 8 = 3");
        assert!(s.contains(&[st, id(&d, "http://ex/log1024"), ten]), "log_2 1024 = 10");
        assert!(s.contains(&[st, id(&d, "http://ex/atan"), zero]), "atan2(0,1) = 0");
        assert!(!s.iter().any(|[a, p, _]| *a == st && *p == id(&d, "http://ex/badLog")), "base 1 fails");
    }

    #[test]
    fn string_format_subset() {
        // %s / %d / %% work; an unsupported directive fails the premise (no mangling).
        let src = r#"
            @prefix : <http://ex/> .
            @prefix string: <http://www.w3.org/2000/10/swap/string#> .
            :seed :p :o .
            { ("%s scored %d%%" "alice" 95) string:format ?out } => { :s :msg ?out } .
            { ("%x" 255) string:format ?bad } => { :s :hex ?bad } .
        "#;
        let (mut d, s) = closure(src);
        let msg = d.intern_lit("alice scored 95%", "http://www.w3.org/2001/XMLSchema#string", None);
        let st = id(&d, "http://ex/s");
        assert!(s.contains(&[st, id(&d, "http://ex/msg"), msg]), "%s/%d/%% formatting");
        assert!(!s.iter().any(|[a, p, _]| *a == st && *p == id(&d, "http://ex/hex")), "%x unsupported → fails");
    }

    #[test]
    fn string_scrape_capture_group() {
        // EYE biP.n3 strs1: the FIRST capture group of the first match, as xsd:string.
        let src = r#"
            @prefix : <http://ex/> .
            @prefix string: <http://www.w3.org/2000/10/swap/string#> .
            :seed :p :o .
            { ("http://example.org/1995/manifesto" "http://([^/]+)/([^/]+)") string:scrape ?h }
            => { :s :host ?h } .
        "#;
        let (mut d, s) = closure(src);
        let host = d.intern_lit("example.org", "http://www.w3.org/2001/XMLSchema#string", None);
        assert!(s.contains(&[id(&d, "http://ex/s"), id(&d, "http://ex/host"), host]), "capture group 1");
    }

    #[test]
    fn log_conjunction_merges_formulae() {
        // EYE biP.n3 logc2/logc3 semantics: merge a list of formulae (empties contribute
        // nothing); the result can be a log:includes scope in the same premise.
        let src = r#"
            @prefix : <http://ex/> .
            @prefix log: <http://www.w3.org/2000/10/swap/log#> .
            :seed :p :o .
            { ({} {:u :v :w} {:x :y :z. :j :k :l}) log:conjunction {:u :v :w. :x :y :z. :j :k :l} }
            => { :s :merged :exact } .
            { ( {:a :p :b} {:c :q :d} ) log:conjunction ?F . ?F log:includes { ?s :q :d } }
            => { ?s a :FoundViaConjunction } .
        "#;
        let (d, s) = closure(src);
        assert!(has(&d, &s, "http://ex/s", "http://ex/merged", "http://ex/exact"), "exact merge (logc2)");
        let ty = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
        assert!(has(&d, &s, "http://ex/c", ty, "http://ex/FoundViaConjunction"), "merged formula as scope");
    }

    #[test]
    fn log_uri_both_directions() {
        // forward: IRI → its text as xsd:string; reverse: string → the IRI it names.
        let src = r#"
            @prefix : <http://ex/> .
            @prefix log: <http://www.w3.org/2000/10/swap/log#> .
            :alice :knows :bob .
            { :alice log:uri ?u } => { :r :uriStr ?u } .
            { ?who log:uri "http://ex/bob" } => { ?who a :Named } .
        "#;
        let (mut d, s) = closure(src);
        let u = d.intern_lit("http://ex/alice", "http://www.w3.org/2001/XMLSchema#string", None);
        assert!(s.contains(&[id(&d, "http://ex/r"), id(&d, "http://ex/uriStr"), u]), "IRI → string");
        let ty = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
        assert!(has(&d, &s, "http://ex/bob", ty, "http://ex/Named"), "string → IRI");
    }

    #[test]
    fn log_dtlit_both_directions() {
        // forward: ( "lex" xsd:dt ) → "lex"^^xsd:dt; reverse: decompose a ground literal.
        let src = r#"
            @prefix : <http://ex/> .
            @prefix log: <http://www.w3.org/2000/10/swap/log#> .
            @prefix time: <http://www.w3.org/2000/10/swap/time#> .
            @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
            :e :when "2024-03-15T10:30:45"^^xsd:dateTime .
            { ("2024-01-01" xsd:date) log:dtlit ?lit . ?lit time:year ?y } => { :d :y ?y } .
            { ?x :when ?w . (?lex ?dt) log:dtlit ?w } => { ?x :lexOf ?lex ; :dtOf ?dt } .
        "#;
        let (mut d, s) = closure(src);
        let y = d.intern_lit("2024", "http://www.w3.org/2001/XMLSchema#integer", None);
        assert!(s.contains(&[id(&d, "http://ex/d"), id(&d, "http://ex/y"), y]), "forward dtlit feeds time:year");
        let lex = d.intern_lit("2024-03-15T10:30:45", "http://www.w3.org/2001/XMLSchema#string", None);
        let e = id(&d, "http://ex/e");
        assert!(s.contains(&[e, id(&d, "http://ex/lexOf"), lex]), "reverse: lexical part");
        assert!(
            s.contains(&[e, id(&d, "http://ex/dtOf"), id(&d, "http://www.w3.org/2001/XMLSchema#dateTime")]),
            "reverse: datatype part"
        );
    }

    #[test]
    fn math_member_count() {
        // EYE biP.n3 mathm1/mathm2: list length, or DISTINCT triple count of a formula.
        let src = r#"
            @prefix : <http://ex/> .
            @prefix math: <http://www.w3.org/2000/10/swap/math#> .
            :seed :p :o .
            { (:u :v :u) math:memberCount ?n } => { :s :listCount ?n } .
            { {:s :p :o1. :s :p :o2. :s :p :o1} math:memberCount ?m } => { :s :graphCount ?m } .
        "#;
        let (mut d, s) = closure(src);
        let int = "http://www.w3.org/2001/XMLSchema#integer";
        let st = id(&d, "http://ex/s");
        assert!(s.contains(&[st, id(&d, "http://ex/listCount"), d.intern_lit("3", int, None)]), "list len 3");
        assert!(s.contains(&[st, id(&d, "http://ex/graphCount"), d.intern_lit("2", int, None)]), "2 distinct");
    }

    #[test]
    fn string_contains_ignoring_case() {
        // EYE biP.n3 strci1: "Tim" string:containsIgnoringCase "IM".
        let src = r#"
            @prefix : <http://ex/> .
            @prefix string: <http://www.w3.org/2000/10/swap/string#> .
            :a :name "Tim" . :b :name "Bob" .
            { ?x :name ?n . ?n string:containsIgnoringCase "IM" } => { ?x a :Match } .
        "#;
        let (d, s) = closure(src);
        let ty = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
        assert!(has(&d, &s, "http://ex/a", ty, "http://ex/Match"), "case-insensitive hit");
        assert!(!has(&d, &s, "http://ex/b", ty, "http://ex/Match"), "non-match excluded");
    }

    #[test]
    fn first_class_list_unification() {
        // `( ?x )` unifies structurally with a data list `( 17 )` (cwm unify2).
        let src = r#"
            @prefix : <http://ex/> .
            ( 17 ) a :TestCase .
            { ( ?x ) a :TestCase } => { ?x a :RESULT } .
        "#;
        let (mut d, s) = closure(src);
        let i17 = d.intern_lit("17", "http://www.w3.org/2001/XMLSchema#integer", None);
        let ty = id(&d, "http://www.w3.org/1999/02/22-rdf-syntax-ns#type");
        assert!(s.contains(&[i17, ty, id(&d, "http://ex/RESULT")]), "list unification binds ?x=17");
    }

    #[test]
    fn list_append_builtin() {
        let src = r#"
            @prefix : <http://ex/> .
            @prefix list: <http://www.w3.org/2000/10/swap/list#> .
            :seed :p :o .
            { ((1 2) (3)) list:append (1 2 3) } => { :s a :AppendOk } .
            { (() (1)) list:append (1) } => { :s a :EmptyOk } .
            { ((:a) (:b)) list:append ?out . ?out list:member ?m } => { ?m a :Member } .
        "#;
        let (d, s) = closure(src);
        let ty = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
        assert!(has(&d, &s, "http://ex/s", ty, "http://ex/AppendOk"), "append filter");
        assert!(has(&d, &s, "http://ex/s", ty, "http://ex/EmptyOk"), "empty list append");
        assert!(has(&d, &s, "http://ex/a", ty, "http://ex/Member"), "constructed list is iterable");
        assert!(has(&d, &s, "http://ex/b", ty, "http://ex/Member"), "constructed list is iterable");
    }

    #[test]
    fn list_iterate_and_virtual_first_rest() {
        let src = r#"
            @prefix : <http://ex/> .
            @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
            @prefix list: <http://www.w3.org/2000/10/swap/list#> .
            ((:q)) a :Thing .
            { (:a :b) list:iterate (1 ?v) } => { ?v a :Second } .
            { ?X a :Thing . ?X rdf:rest ?Y } => { ?Y a :Thing } .
            { ?X a :Thing; rdf:first (?B) } => { ?B a :GreatThing } .
        "#;
        let (d, s) = closure(src);
        let ty = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
        assert!(has(&d, &s, "http://ex/b", ty, "http://ex/Second"), "list:iterate index/value");
        assert!(has(&d, &s, "http://ex/q", ty, "http://ex/GreatThing"), "virtual rdf:first over list");
    }

    #[test]
    fn math_builtin_filter() {
        // math:greaterThan as a premise filter: adults are people over 17.
        let src = r#"
            @prefix : <http://ex/> .
            @prefix math: <http://www.w3.org/2000/10/swap/math#> .
            :alice :age 30 . :bob :age 12 .
            { ?p :age ?a . ?a math:greaterThan 17 } => { ?p a :Adult } .
        "#;
        let (d, s) = closure(src);
        assert!(has(&d, &s, "http://ex/alice", "http://www.w3.org/1999/02/22-rdf-syntax-ns#type", "http://ex/Adult"), "alice is adult");
        assert!(!has(&d, &s, "http://ex/bob", "http://www.w3.org/1999/02/22-rdf-syntax-ns#type", "http://ex/Adult"), "bob is NOT adult");
    }
}
