//! [FABLE-5] sq-3dyje.3 — Property-based tests for sparq-engine evaluation:
//! engine ≡ independent naive reference on random graphs × random queries,
//! join-order permutation invariance, and ORDER BY laws over the real `Value` path.
//!
//! Design record: research/testing-strategy-assessment-2026-07.md §7.1 (sparq-engine row);
//! comparison model: research/differential-testing-value-level.md §2–§3.
//!
//! # The three property families
//!
//! 1. **`eval ≡ naive reference` (`family1_*`).** A random small graph and a random
//!    query (BGP / OPTIONAL / UNION / FILTER / BIND / DISTINCT / subset projection /
//!    ORDER BY / LIMIT) are generated; the engine's solution multiset must equal an
//!    in-test naive nested-loop evaluator's under MULTISET semantics (§2.1 of the
//!    comparison-model record). ORDER BY output is additionally checked for
//!    sortedness (see family 3); ORDER BY + LIMIT without a total order is checked
//!    as sub-multiset + cardinality (§2.2: which key-tied rows survive a slice is
//!    implementation-defined).
//! 2. **Join-order invariance (`family2_*`).** Permuting the triple patterns of a
//!    BGP never changes the result multiset — the planner (greedy GOO, or DPccp
//!    under `dp-planner`) reorders for performance only.
//! 3. **ORDER BY laws (`family3_*` + the sortedness check inside family 1).**
//!    Over tie-free integer sort keys the engine's ASC sequence equals the oracle's
//!    exact sort, DESC is its exact reverse, and LIMIT k is the exact k-prefix.
//!    Over arbitrary mixed-kind keys the engine sequence must respect every pair
//!    the SPARQL ordering defines: unbound < blank < IRI < literal, IRIs by
//!    codepoint, numerics by value, xsd:string by codepoint, booleans false<true.
//!    Pairs the spec leaves open (cross-literal-class, langString, bnode labels)
//!    are deliberately unconstrained.
//!
//! # Oracle independence
//!
//! The reference evaluator shares NO code with the engine: it works on a test-side
//! term model (`T`) and a test-side query AST, evaluates by substitution-based
//! nested loops over a plain `Vec<[T; 3]>`, and compares by rendered N-Triples
//! strings. The engine is driven end-to-end through `sparq_engine::query` on the
//! SPARQL text rendered from the same AST — the engine's parser, planner, and
//! executor are all inside the tested boundary; the oracle touches none of them
//! (the §2.3 "independence trap" of the comparison-model record).
//!
//! # Plan paths
//!
//! With default features the engine path is the greedy GOO planner + scalar exec.
//! Compiled with `--features dp-planner,vectorized`, DPccp plans by DEFAULT
//! (tests/dp_default.rs) and the vectorized kernels engage — every property then
//! ALSO cross-checks `without_dp_planner` (greedy) against the same oracle, so one
//! run validates both plan paths case-by-case.
//!
//! # Pinned engine semantics the oracle encodes (probe-verified in this tree)
//!
//! Each rule below was pinned by running the real engine before writing the oracle:
//!
//! * literals keep their lexical form end-to-end (`"1.50"^^xsd:decimal` stays "1.50");
//!   blank-node labels round-trip through load+query.
//! * `=` on literals: numeric×numeric / string×string / boolean×boolean compare by
//!   value; langString = anything-not-langString → `false`; any other cross-class
//!   literal pair (string×numeric, boolean×numeric, boolean×string) → type error.
//!   `=` across term kinds (IRI / bnode / literal) → `false`. Unbound operand → error.
//! * `<` `>` against an integer constant: numeric operand compares by value; any
//!   non-numeric operand (IRI, bnode, string, boolean, langString) → type error.
//! * three-valued logic is exactly SPARQL's: `err||true=true`, `err||false=err`,
//!   `err&&false=false`, `!err=err`; a FILTER error drops the row.
//! * `BIND` arithmetic: int+int → canonical integer; decimal+int keeps the operand's
//!   scale ("1.50"+1 → "2.50"); a type-error BIND leaves the variable UNBOUND and
//!   keeps the row. Computed booleans render as `"true|false"^^xsd:boolean`.
//! * ORDER BY: unbound < blank < IRI < literal; IRIs / xsd:strings by codepoint;
//!   numerics by value across int/decimal/double; booleans false < true.
//!
//! # Documented generator narrowings (each carries a reason + a follow-up bead)
//!
//! * `isIRI/isBlank/isLiteral/isNumeric` are only generated over vars bound in the
//!   REQUIRED part of the query: the engine returns `false` for `isIRI(?unbound)`
//!   where SPARQL 17.2 mandates a type error (probe-verified divergence — beaded,
//!   see sq-qeltv; the oracle is NOT fudged to the engine here, the case is
//!   excluded from generation instead).
//! * `STR(?v)` (inside the STRLEN BIND form) masks blank nodes out of the generated
//!   graph: the engine extends STR to bnodes (an engine-internal label) where SPARQL
//!   makes it a type error — same bead (sq-qeltv).
//! * `?v + k` masks xsd:double literals out of the generated graph: a computed
//!   double's rendered lexical would require replicating the engine's float
//!   formatter inside the oracle (an independence-trap violation). Widening bead
//!   sq-ilweo.
//! * xsd:double NaN is not generated: `<` is not a total order under NaN, so the
//!   ORDER BY law has no spec-defined expectation (substrate-level NaN ordering is
//!   covered by sparq-substrate/tests/proptest_order_numeric.rs). ±INF IS generated.
//!
//! # Non-vacuity (mutation-verified, see PR)
//!
//! Verified by temporarily injecting engine bugs into `src/exec.rs` and observing
//! the properties go red with shrunk counterexamples (then reverting; the committed
//! src tree is byte-identical to main):
//! * FILTER-drop (`apply_filter_scalar`: `keep` overridden to all-`true`) → family 1
//!   red, shrunk to a 1-pattern query with `FILTER(!isIRI(?v0))`.
//! * join-dup (`bind_join`: `sjoin::bind_combine` emitted twice) → families 1 AND 2
//!   red (every joined row duplicated vs the oracle multiset).
//! * DISTINCT no-op (`distinct_bindings` emptied) → family 1 red on a
//!   `SELECT DISTINCT` query whose oracle multiset deduped.
//!
//! Additionally `oracle_known_answer_*` pin the oracle itself against hand-computed
//! rows (a degenerate oracle cannot silently agree with the engine), and
//! `generator_diversity_floor` asserts the generated corpus actually contains
//! nonempty results, duplicate rows, unbound cells, and every query shape.

use proptest::prelude::*;
use sparq_core::Graph;
use sparq_engine::query;

const XSD: &str = "http://www.w3.org/2001/XMLSchema#";

// ═══════════════════════════════════════════════════════════════════════════════
// Test-side term model (independent of the engine's Value / dict code)
// ═══════════════════════════════════════════════════════════════════════════════

/// A test-side RDF term. Carries exactly the information needed to (a) emit the
/// N-Triples data / SPARQL query text and (b) evaluate the naive reference.
#[derive(Clone, Debug, PartialEq)]
enum T {
    Iri(String),
    BNode(String),
    /// simple literal ≡ xsd:string (RDF 1.1); rendered without a datatype suffix,
    /// which is also how the engine's `Term::to_string()` renders xsd:string.
    Str(String),
    /// language-tagged string: (lexical, lowercase tag).
    Lang(String, String),
    Int(i64),
    /// xsd:decimal as (mantissa, scale), scale ≥ 1; lexical is the canonical
    /// scaled rendering (`render_dec`).
    Dec(i128, u32),
    /// xsd:double as (lexical form, value). Data-sourced only (never computed by a
    /// generated BIND — see the generator-narrowing notes in the header).
    Dbl(String, f64),
    Bool(bool),
}

fn render_dec(m: i128, scale: u32) -> String {
    let neg = m < 0;
    let abs = m.unsigned_abs();
    let pow = 10u128.pow(scale);
    let int_part = abs / pow;
    let frac = abs % pow;
    let mut s = String::new();
    if neg {
        s.push('-');
    }
    s.push_str(&int_part.to_string());
    s.push('.');
    let frac_str = frac.to_string();
    for _ in frac_str.len()..(scale as usize) {
        s.push('0');
    }
    s.push_str(&frac_str);
    s
}

impl T {
    /// N-Triples / SPARQL rendering. Matches the engine's `Term::to_string()` for
    /// every term this file can generate or compute (probe-verified).
    fn render(&self) -> String {
        match self {
            T::Iri(i) => format!("<{}>", i),
            T::BNode(b) => format!("_:{}", b),
            T::Str(s) => format!("\"{}\"", s),
            T::Lang(s, tag) => format!("\"{}\"@{}", s, tag),
            T::Int(n) => format!("\"{}\"^^<{}integer>", n, XSD),
            T::Dec(m, sc) => format!("\"{}\"^^<{}decimal>", render_dec(*m, *sc), XSD),
            T::Dbl(lex, _) => format!("\"{}\"^^<{}double>", lex, XSD),
            T::Bool(b) => format!("\"{}\"^^<{}boolean>", b, XSD),
        }
    }

    /// The SPARQL `STR()` of this term: IRI string or literal lexical form.
    /// Blank nodes are excluded by generation (see header).
    fn str_value(&self) -> Option<String> {
        match self {
            T::Iri(i) => Some(i.clone()),
            T::BNode(_) => None,
            T::Str(s) | T::Lang(s, _) => Some(s.clone()),
            T::Int(n) => Some(n.to_string()),
            T::Dec(m, sc) => Some(render_dec(*m, *sc)),
            T::Dbl(lex, _) => Some(lex.clone()),
            T::Bool(b) => Some(b.to_string()),
        }
    }
}

/// Exact numeric value of a literal, if it is numeric.
#[derive(Clone, Debug)]
enum Num {
    I(i64),
    D(i128, u32),
    F(f64),
}

fn num_of(t: &T) -> Option<Num> {
    match t {
        T::Int(n) => Some(Num::I(*n)),
        T::Dec(m, s) => Some(Num::D(*m, *s)),
        T::Dbl(_, v) => Some(Num::F(*v)),
        _ => None,
    }
}

/// Exact value comparison on the numeric tower. int/decimal compare exactly in
/// i128; anything involving a double converts to f64 — exact for every value this
/// file generates (|int| ≤ 4000, decimal mantissa/10^scale is a correctly-rounded
/// quotient of small integers, doubles come from a fixed finite pool; NaN is never
/// generated).
fn num_cmp(a: &Num, b: &Num) -> std::cmp::Ordering {
    use Num::*;
    match (a, b) {
        (I(x), I(y)) => x.cmp(y),
        (D(mx, sx), D(my, sy)) => {
            let scale = (*sx).max(*sy);
            let ax = mx * 10i128.pow(scale - sx);
            let ay = my * 10i128.pow(scale - sy);
            ax.cmp(&ay)
        }
        (I(x), D(my, sy)) => num_cmp(&D(*x as i128, 0), &D(*my, *sy)),
        (D(mx, sx), I(y)) => num_cmp(&D(*mx, *sx), &D(*y as i128, 0)),
        _ => {
            let fx = num_f64(a);
            let fy = num_f64(b);
            fx.partial_cmp(&fy).expect("NaN is never generated")
        }
    }
}

fn num_f64(n: &Num) -> f64 {
    match n {
        Num::I(x) => *x as f64,
        Num::D(m, s) => (*m as f64) / 10f64.powi(*s as i32),
        Num::F(v) => *v,
    }
}

/// SPARQL `=` under the engine's probe-pinned model (see header table).
/// `Err(())` is a type error.
fn eq_spec(a: &T, b: &T) -> Result<bool, ()> {
    use T::*;
    match (a, b) {
        (Iri(x), Iri(y)) => Ok(x == y),
        (BNode(x), BNode(y)) => Ok(x == y),
        // different term kinds (or kind vs literal): definitely not the same term.
        (Iri(_) | BNode(_), _) | (_, Iri(_) | BNode(_)) => Ok(false),
        // literals from here on.
        (Lang(l1, t1), Lang(l2, t2)) => Ok(l1 == l2 && t1 == t2),
        (Lang(..), _) | (_, Lang(..)) => Ok(false),
        (Str(x), Str(y)) => Ok(x == y),
        (Bool(x), Bool(y)) => Ok(x == y),
        _ => match (num_of(a), num_of(b)) {
            (Some(x), Some(y)) => Ok(num_cmp(&x, &y) == std::cmp::Ordering::Equal),
            // cross-class among {numeric, string, boolean}: type error.
            _ => Err(()),
        },
    }
}

/// SPARQL `<` / `>` of a term against an integer constant: numeric-only, anything
/// else is a type error (probe-pinned).
fn cmp_int_spec(t: &T, k: i64) -> Result<std::cmp::Ordering, ()> {
    num_of(t).map(|n| num_cmp(&n, &Num::I(k))).ok_or(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// Test-side query AST + SPARQL rendering
// ═══════════════════════════════════════════════════════════════════════════════

/// Variable ids: 0..=3 subject/object vars, 3..=4 predicate vars (3 is shared so
/// one variable can join across positions), 9 the BIND target.
type V = u8;
const BIND_VAR: V = 9;

fn vname(v: V) -> String {
    format!("?v{}", v)
}

#[derive(Clone, Debug, PartialEq)]
enum Pos {
    Var(V),
    Const(T),
}

#[derive(Clone, Debug, PartialEq)]
struct Tp {
    s: Pos,
    p: Pos,
    o: Pos,
}

impl Tp {
    fn render(&self) -> String {
        let r = |p: &Pos| match p {
            Pos::Var(v) => vname(*v),
            Pos::Const(t) => t.render(),
        };
        format!("{} {} {}", r(&self.s), r(&self.p), r(&self.o))
    }
    fn vars(&self, out: &mut Vec<V>) {
        for p in [&self.s, &self.p, &self.o] {
            if let Pos::Var(v) = p {
                if !out.contains(v) {
                    out.push(*v);
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
enum Body {
    Bgp(Vec<Tp>),
    /// base BGP OPTIONAL { right BGP }
    Opt(Vec<Tp>, Vec<Tp>),
    /// { left BGP } UNION { right BGP }
    Union(Vec<Tp>, Vec<Tp>),
}

impl Body {
    fn render(&self) -> String {
        let bgp = |tps: &[Tp]| tps.iter().map(Tp::render).collect::<Vec<_>>().join(" . ");
        match self {
            Body::Bgp(tps) => bgp(tps),
            Body::Opt(l, r) => format!("{} OPTIONAL {{ {} }}", bgp(l), bgp(r)),
            Body::Union(a, b) => format!("{{ {} }} UNION {{ {} }}", bgp(a), bgp(b)),
        }
    }
    /// All variables, sorted.
    fn vars(&self) -> Vec<V> {
        let mut out = Vec::new();
        let mut add = |tps: &[Tp]| {
            for tp in tps {
                tp.vars(&mut out);
            }
        };
        match self {
            Body::Bgp(tps) => add(tps),
            Body::Opt(l, r) | Body::Union(l, r) => {
                add(l);
                add(r);
            }
        }
        out.sort_unstable();
        out
    }
    /// Variables bound in EVERY solution (the required part): all vars of a BGP,
    /// the base vars of an OPTIONAL, the intersection of UNION branch vars. Used
    /// to keep `is*()` off possibly-unbound vars (see the header narrowing note).
    fn certain_vars(&self) -> Vec<V> {
        match self {
            Body::Bgp(tps) => {
                let mut out = Vec::new();
                for tp in tps {
                    tp.vars(&mut out);
                }
                out.sort_unstable();
                out
            }
            Body::Opt(l, _) => Body::Bgp(l.clone()).certain_vars(),
            Body::Union(a, b) => {
                let va = Body::Bgp(a.clone()).certain_vars();
                let vb = Body::Bgp(b.clone()).certain_vars();
                va.into_iter().filter(|v| vb.contains(v)).collect()
            }
        }
    }
}

#[derive(Clone, Debug)]
enum Bind {
    /// BIND((?v + k) AS ?v9)
    PlusK(V, i64),
    /// BIND((?v < k) AS ?v9)
    LtK(V, i64),
    /// BIND(STRLEN(STR(?v)) AS ?v9)
    StrLenOf(V),
}

impl Bind {
    fn render(&self) -> String {
        match self {
            Bind::PlusK(v, k) => format!("BIND(({} + {}) AS {})", vname(*v), k, vname(BIND_VAR)),
            Bind::LtK(v, k) => format!("BIND(({} < {}) AS {})", vname(*v), k, vname(BIND_VAR)),
            Bind::StrLenOf(v) => format!("BIND(STRLEN(STR({})) AS {})", vname(*v), vname(BIND_VAR)),
        }
    }
}

#[derive(Clone, Debug)]
enum Expr {
    EqVV(V, V),
    EqVC(V, T), // T is an IRI or a literal (bnodes are not legal in expressions)
    LtVK(V, i64),
    GtVK(V, i64),
    Bound(V),
    IsIri(V),
    IsBlank(V),
    IsLit(V),
    IsNum(V),
    Not(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
}

impl Expr {
    fn render(&self) -> String {
        match self {
            Expr::EqVV(a, b) => format!("({} = {})", vname(*a), vname(*b)),
            Expr::EqVC(v, c) => format!("({} = {})", vname(*v), c.render()),
            Expr::LtVK(v, k) => format!("({} < {})", vname(*v), k),
            Expr::GtVK(v, k) => format!("({} > {})", vname(*v), k),
            Expr::Bound(v) => format!("BOUND({})", vname(*v)),
            Expr::IsIri(v) => format!("isIRI({})", vname(*v)),
            Expr::IsBlank(v) => format!("isBlank({})", vname(*v)),
            Expr::IsLit(v) => format!("isLiteral({})", vname(*v)),
            Expr::IsNum(v) => format!("isNumeric({})", vname(*v)),
            Expr::Not(e) => format!("(!{})", e.render()),
            Expr::And(a, b) => format!("({} && {})", a.render(), b.render()),
            Expr::Or(a, b) => format!("({} || {})", a.render(), b.render()),
        }
    }
}

#[derive(Clone, Debug)]
struct Q {
    body: Body,
    bind: Option<Bind>,
    filter: Option<Expr>,
    distinct: bool,
    /// Projected variables, sorted, nonempty, ⊆ in-scope vars.
    project: Vec<V>,
    /// (variable ∈ project, descending?)
    order: Option<(V, bool)>,
    limit: Option<usize>,
}

impl Q {
    fn render(&self) -> String {
        let mut s = String::from("SELECT ");
        if self.distinct {
            s.push_str("DISTINCT ");
        }
        for v in &self.project {
            s.push_str(&vname(*v));
            s.push(' ');
        }
        s.push_str("WHERE { ");
        s.push_str(&self.body.render());
        if let Some(b) = &self.bind {
            s.push(' ');
            s.push_str(&b.render());
        }
        if let Some(f) = &self.filter {
            s.push_str(" FILTER");
            s.push_str(&f.render());
        }
        s.push_str(" }");
        if let Some((v, desc)) = &self.order {
            if *desc {
                s.push_str(&format!(" ORDER BY DESC({})", vname(*v)));
            } else {
                s.push_str(&format!(" ORDER BY {}", vname(*v)));
            }
        }
        if let Some(k) = self.limit {
            s.push_str(&format!(" LIMIT {}", k));
        }
        s
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// The naive reference evaluator (nested loops; shares nothing with the engine)
// ═══════════════════════════════════════════════════════════════════════════════

/// A solution mapping over the small var universe.
type Row = [Option<T>; 10];

fn empty_row() -> Row {
    Default::default()
}

fn match_pos(p: &Pos, t: &T, row: &Row) -> Option<Option<(V, T)>> {
    match p {
        Pos::Const(c) => (c == t).then_some(None),
        Pos::Var(v) => match &row[*v as usize] {
            Some(bound) => (bound == t).then_some(None),
            None => Some(Some((*v, t.clone()))),
        },
    }
}

fn match_tp(tp: &Tp, triple: &[T; 3], row: &Row) -> Option<Row> {
    let mut ext = row.clone();
    for (pos, term) in [(&tp.s, &triple[0]), (&tp.p, &triple[1]), (&tp.o, &triple[2])] {
        match match_pos(pos, term, &ext) {
            None => return None,
            Some(None) => {}
            Some(Some((v, t))) => ext[v as usize] = Some(t),
        }
    }
    Some(ext)
}

fn eval_bgp(data: &[[T; 3]], tps: &[Tp]) -> Vec<Row> {
    let mut sols = vec![empty_row()];
    for tp in tps {
        let mut next = Vec::new();
        for sol in &sols {
            for triple in data {
                if let Some(ext) = match_tp(tp, triple, sol) {
                    next.push(ext);
                }
            }
        }
        sols = next;
    }
    sols
}

fn compatible_merge(l: &Row, r: &Row) -> Option<Row> {
    let mut out = l.clone();
    for i in 0..out.len() {
        match (&out[i], &r[i]) {
            (Some(a), Some(b)) if a != b => return None,
            (None, Some(b)) => out[i] = Some(b.clone()),
            _ => {}
        }
    }
    Some(out)
}

fn eval_body(data: &[[T; 3]], body: &Body) -> Vec<Row> {
    match body {
        Body::Bgp(tps) => eval_bgp(data, tps),
        Body::Opt(l, r) => {
            let left = eval_bgp(data, l);
            let right = eval_bgp(data, r);
            let mut out = Vec::new();
            for lr in &left {
                let mut matched = false;
                for rr in &right {
                    if let Some(m) = compatible_merge(lr, rr) {
                        out.push(m);
                        matched = true;
                    }
                }
                if !matched {
                    out.push(lr.clone());
                }
            }
            out
        }
        Body::Union(a, b) => {
            let mut out = eval_bgp(data, a);
            out.extend(eval_bgp(data, b));
            out
        }
    }
}

/// BIND semantics: a type error leaves the variable unbound and KEEPS the row.
fn eval_bind(b: &Bind, row: &Row) -> Option<T> {
    match b {
        Bind::PlusK(v, k) => match row[*v as usize].as_ref()? {
            T::Int(n) => Some(T::Int(n + k)),
            T::Dec(m, s) => Some(T::Dec(m + (*k as i128) * 10i128.pow(*s), *s)),
            // The generator masks doubles out of the graph when a PlusK BIND is
            // present (header note): reaching this arm means the mask is broken.
            T::Dbl(..) => panic!("generator mask violated: PlusK over a double"),
            _ => None,
        },
        Bind::LtK(v, k) => {
            let t = row[*v as usize].as_ref()?;
            cmp_int_spec(t, *k).ok().map(|o| T::Bool(o == std::cmp::Ordering::Less))
        }
        Bind::StrLenOf(v) => {
            let t = row[*v as usize].as_ref()?;
            t.str_value().map(|s| T::Int(s.chars().count() as i64))
        }
    }
}

/// Three-valued (Kleene) FILTER expression evaluation; `Err(())` is a type error.
fn eval_expr(e: &Expr, row: &Row) -> Result<bool, ()> {
    let get = |v: &V| row[*v as usize].as_ref().ok_or(());
    match e {
        Expr::EqVV(a, b) => eq_spec(get(a)?, get(b)?),
        Expr::EqVC(v, c) => eq_spec(get(v)?, c),
        Expr::LtVK(v, k) => Ok(cmp_int_spec(get(v)?, *k)? == std::cmp::Ordering::Less),
        Expr::GtVK(v, k) => Ok(cmp_int_spec(get(v)?, *k)? == std::cmp::Ordering::Greater),
        Expr::Bound(v) => Ok(row[*v as usize].is_some()),
        // is* over an unbound var is a type error per SPARQL 17.2; the generator
        // never produces that case (see header narrowing note + bead).
        Expr::IsIri(v) => Ok(matches!(get(v)?, T::Iri(_))),
        Expr::IsBlank(v) => Ok(matches!(get(v)?, T::BNode(_))),
        Expr::IsLit(v) => Ok(!matches!(get(v)?, T::Iri(_) | T::BNode(_))),
        Expr::IsNum(v) => Ok(num_of(get(v)?).is_some()),
        Expr::Not(inner) => eval_expr(inner, row).map(|b| !b),
        Expr::And(a, b) => match (eval_expr(a, row), eval_expr(b, row)) {
            (Ok(false), _) | (_, Ok(false)) => Ok(false),
            (Ok(true), Ok(true)) => Ok(true),
            _ => Err(()),
        },
        Expr::Or(a, b) => match (eval_expr(a, row), eval_expr(b, row)) {
            (Ok(true), _) | (_, Ok(true)) => Ok(true),
            (Ok(false), Ok(false)) => Ok(false),
            _ => Err(()),
        },
    }
}

/// The full naive evaluation: body → BIND → FILTER → project → DISTINCT.
/// Returns the (unordered) solution multiset as rendered rows; ordering laws are
/// checked against the engine output separately.
fn oracle_eval(data: &[[T; 3]], q: &Q) -> Vec<Vec<Option<String>>> {
    let mut rows = eval_body(data, &q.body);
    if let Some(b) = &q.bind {
        for row in &mut rows {
            row[BIND_VAR as usize] = eval_bind(b, row);
        }
    }
    if let Some(f) = &q.filter {
        rows.retain(|row| eval_expr(f, row) == Ok(true));
    }
    let mut projected: Vec<Vec<Option<String>>> = rows
        .iter()
        .map(|row| {
            q.project
                .iter()
                .map(|v| row[*v as usize].as_ref().map(T::render))
                .collect()
        })
        .collect();
    if q.distinct {
        let mut seen = std::collections::HashSet::new();
        projected.retain(|r| seen.insert(row_key(r)));
    }
    projected
}

// ═══════════════════════════════════════════════════════════════════════════════
// Data → N-Triples, engine execution, and the comparison model
// ═══════════════════════════════════════════════════════════════════════════════

fn to_ntriples(data: &[[T; 3]]) -> String {
    let mut s = String::new();
    for t in data {
        s.push_str(&format!("{} {} {} .\n", t[0].render(), t[1].render(), t[2].render()));
    }
    s
}

/// An RDF graph is a SET of triples: the oracle deduplicates exactly like the store.
fn dedup_data(mut data: Vec<[T; 3]>) -> Vec<[T; 3]> {
    let mut seen = std::collections::HashSet::new();
    data.retain(|t| seen.insert(format!("{} {} {}", t[0].render(), t[1].render(), t[2].render())));
    data
}

const UNBOUND_KEY: &str = "\u{1}unbound\u{1}";

fn row_key(row: &[Option<String>]) -> String {
    row.iter()
        .map(|c| c.as_deref().unwrap_or(UNBOUND_KEY))
        .collect::<Vec<_>>()
        .join("\u{1f}")
}

fn multiset(rows: &[Vec<Option<String>>]) -> Vec<String> {
    let mut keys: Vec<String> = rows.iter().map(|r| row_key(r)).collect();
    keys.sort_unstable();
    keys
}

/// Run the engine end-to-end (parse → plan → execute) and render the rows.
fn engine_rows(g: &Graph, qtext: &str) -> Vec<Vec<Option<String>>> {
    let r = query(g, qtext).unwrap_or_else(|e| panic!("engine rejected generated query {:?}: {}", qtext, e));
    r.rows
        .iter()
        .map(|row| row.iter().map(|c| c.as_ref().map(|t| t.to_string())).collect())
        .collect()
}

/// Every plan path compiled into this binary, as (label, result-rows) pairs.
/// Default build: greedy GOO. With `dp-planner`: DPccp is the default path and the
/// greedy path is cross-checked via `without_dp_planner`. With `vectorized` the
/// vectorized kernels are engaged inside whichever planner runs.
fn engine_paths(g: &Graph, qtext: &str) -> Vec<(&'static str, Vec<Vec<Option<String>>>)> {
    #[allow(unused_mut)] // mut is only needed when dp-planner pushes the second path
    let mut out = vec![("default", engine_rows(g, qtext))];
    #[cfg(feature = "dp-planner")]
    out.push(("greedy(no-dp)", sparq_engine::without_dp_planner(|| engine_rows(g, qtext))));
    out
}

/// Parse a rendered engine cell back into the test-side model — only the term
/// shapes this file can generate or compute ever appear.
fn parse_rendered(s: &str) -> T {
    if let Some(inner) = s.strip_prefix('<').and_then(|r| r.strip_suffix('>')) {
        return T::Iri(inner.to_string());
    }
    if let Some(label) = s.strip_prefix("_:") {
        return T::BNode(label.to_string());
    }
    assert!(s.starts_with('"'), "unexpected rendered term: {}", s);
    let close = s.rfind('"').expect("closing quote");
    let lex = &s[1..close];
    let suffix = &s[close + 1..];
    if suffix.is_empty() {
        return T::Str(lex.to_string());
    }
    if let Some(tag) = suffix.strip_prefix('@') {
        return T::Lang(lex.to_string(), tag.to_string());
    }
    let dt = suffix
        .strip_prefix("^^<")
        .and_then(|r| r.strip_suffix('>'))
        .unwrap_or_else(|| panic!("unexpected literal suffix in {}", s));
    match dt.strip_prefix(XSD) {
        Some("integer") => T::Int(lex.parse().expect("integer lexical")),
        Some("boolean") => T::Bool(lex == "true"),
        Some("decimal") => {
            let (int_part, frac) = lex.split_once('.').expect("decimal point");
            let neg = int_part.starts_with('-');
            let digits: i128 = format!("{}{}", int_part.trim_start_matches('-'), frac).parse().expect("decimal digits");
            T::Dec(if neg { -digits } else { digits }, frac.len() as u32)
        }
        Some("double") => {
            let v = match lex {
                "INF" => f64::INFINITY,
                "-INF" => f64::NEG_INFINITY,
                other => other.parse().expect("double lexical"),
            };
            T::Dbl(lex.to_string(), v)
        }
        _ => panic!("unexpected datatype in rendered term: {}", s),
    }
}

/// The partial SPARQL ordering the engine's total ORDER BY order must refine:
/// coarse ranks (unbound < blank < IRI < literal) plus within-class orders where
/// the spec/probes define them. Returns per-class monotonicity violations.
fn check_sorted(keys: &[Option<T>], desc: bool, ctx: &str) -> Result<(), String> {
    let oriented = |o: std::cmp::Ordering| if desc { o.reverse() } else { o };
    // (a) coarse rank sequence must be monotone.
    let rank = |k: &Option<T>| match k {
        None => 0u8,
        Some(T::BNode(_)) => 1,
        Some(T::Iri(_)) => 2,
        Some(_) => 3,
    };
    let mut prev_rank: Option<u8> = None;
    for k in keys {
        let r = rank(k);
        if let Some(p) = prev_rank {
            if oriented(p.cmp(&r)) == std::cmp::Ordering::Greater {
                return Err(format!("rank order violated ({} then {}) in {}", p, r, ctx));
            }
        }
        prev_rank = Some(r);
    }
    // (b) within-class subsequences must be monotone where the order is defined.
    let check_sub = |label: &str, extract: &dyn Fn(&Option<T>) -> Option<Vec<u8>>| -> Result<(), String> {
        let mut prev: Option<Vec<u8>> = None;
        for k in keys {
            if let Some(key) = extract(k) {
                if let Some(p) = &prev {
                    if oriented(p.cmp(&key)) == std::cmp::Ordering::Greater {
                        return Err(format!("{} subsequence order violated in {}", label, ctx));
                    }
                }
                prev = Some(key);
            }
        }
        Ok(())
    };
    check_sub("iri", &|k| match k {
        Some(T::Iri(i)) => Some(i.as_bytes().to_vec()),
        _ => None,
    })?;
    check_sub("xsd:string", &|k| match k {
        Some(T::Str(s)) => Some(s.as_bytes().to_vec()),
        _ => None,
    })?;
    check_sub("boolean", &|k| match k {
        Some(T::Bool(b)) => Some(vec![u8::from(*b)]),
        _ => None,
    })?;
    // numerics: compare by exact value — encode as a monotone-comparable check via
    // pairwise walk (values are not byte-encodable, so walk directly).
    let mut prev_num: Option<Num> = None;
    for k in keys {
        if let Some(n) = k.as_ref().and_then(num_of) {
            if let Some(p) = &prev_num {
                if oriented(num_cmp(p, &n)) == std::cmp::Ordering::Greater {
                    return Err(format!("numeric subsequence order violated in {}", ctx));
                }
            }
            prev_num = Some(n);
        }
    }
    Ok(())
}

fn is_sub_multiset(sub: &[String], full: &[String]) -> bool {
    // both sorted
    let mut it = full.iter();
    'outer: for s in sub {
        for f in it.by_ref() {
            match f.cmp(s) {
                std::cmp::Ordering::Less => {}
                std::cmp::Ordering::Equal => continue 'outer,
                std::cmp::Ordering::Greater => return false,
            }
        }
        return false;
    }
    true
}

/// The full family-1 check for one (data, query) case against one engine path.
fn check_case(data: &[[T; 3]], q: &Q, label: &str, rows: &[Vec<Option<String>>]) -> Result<(), TestCaseError> {
    let oracle = oracle_eval(data, q);
    let en_ms = multiset(rows);
    let or_ms = multiset(&oracle);
    let ctx = || {
        format!(
            "path={} query={:?}\ndata:\n{}engine rows: {:?}\noracle rows: {:?}",
            label,
            q.render(),
            to_ntriples(data),
            rows,
            oracle
        )
    };
    match q.limit {
        None => prop_assert_eq!(&en_ms, &or_ms, "multiset mismatch: {}", ctx()),
        Some(k) => {
            prop_assert_eq!(rows.len(), k.min(or_ms.len()), "LIMIT cardinality mismatch: {}", ctx());
            prop_assert!(is_sub_multiset(&en_ms, &or_ms), "LIMIT rows not a sub-multiset: {}", ctx());
        }
    }
    if let Some((v, desc)) = &q.order {
        let col = q.project.iter().position(|p| p == v).expect("order var projected");
        let keys: Vec<Option<T>> = rows.iter().map(|r| r[col].as_deref().map(parse_rendered)).collect();
        if let Err(e) = check_sorted(&keys, *desc, &ctx()) {
            return Err(TestCaseError::fail(e));
        }
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// Generators
// ═══════════════════════════════════════════════════════════════════════════════

const SUBJ_IRIS: [&str; 6] = [
    "http://ex/s0",
    "http://ex/s1",
    "http://ex/s2",
    "http://ex/s3",
    "http://ex/s4",
    "http://ex/s5",
];
const PRED_IRIS: [&str; 4] = ["http://ex/p0", "http://ex/p1", "http://ex/p2", "http://ex/p3"];
const BNODE_LABELS: [&str; 3] = ["b0", "b1", "b2"];
const STR_POOL: [&str; 6] = ["", "a", "b", "ab", "hi", "z9"];
const LANG_TAGS: [&str; 2] = ["en", "fr"];
/// Fixed double pool: (lexical, value). Finite values + ±INF; no NaN (header note).
const DBL_POOL: [(&str, f64); 7] = [
    ("0.0E0", 0.0),
    ("-0.0E0", -0.0),
    ("1.5E0", 1.5),
    ("-2.5E0", -2.5),
    ("1.0E2", 100.0),
    ("3.25E0", 3.25),
    ("INF", f64::INFINITY),
];
const NEG_INF: (&str, f64) = ("-INF", f64::NEG_INFINITY);

/// Which term families a generated graph may contain — derived from the query so
/// the oracle never has to model an engine surface it cannot predict (header notes).
#[derive(Clone, Copy, Debug, Default)]
struct DataMask {
    no_doubles: bool,
    no_bnodes: bool,
}

fn mask_of(q: &Q) -> DataMask {
    DataMask {
        no_doubles: matches!(q.bind, Some(Bind::PlusK(..))),
        no_bnodes: matches!(q.bind, Some(Bind::StrLenOf(..))),
    }
}

fn arb_literal(mask: DataMask) -> BoxedStrategy<T> {
    let mut opts: Vec<BoxedStrategy<T>> = vec![
        (-30i64..=30).prop_map(T::Int).boxed(),
        prop_oneof![Just(0i64), Just(1), Just(-1), Just(1000)].prop_map(T::Int).boxed(),
        ((-3000i128..=3000), (1u32..=2)).prop_map(|(m, s)| T::Dec(m, s)).boxed(),
        proptest::sample::select(&STR_POOL[..]).prop_map(|s| T::Str(s.to_string())).boxed(),
        (proptest::sample::select(&STR_POOL[..]), proptest::sample::select(&LANG_TAGS[..]))
            .prop_map(|(s, t)| T::Lang(s.to_string(), t.to_string()))
            .boxed(),
        any::<bool>().prop_map(T::Bool).boxed(),
    ];
    if !mask.no_doubles {
        opts.push(
            proptest::sample::select(&DBL_POOL[..])
                .prop_map(|(l, v)| T::Dbl(l.to_string(), v))
                .boxed(),
        );
        opts.push(Just(T::Dbl(NEG_INF.0.to_string(), NEG_INF.1)).boxed());
    }
    proptest::strategy::Union::new(opts).boxed()
}

fn arb_subject(mask: DataMask) -> BoxedStrategy<T> {
    if mask.no_bnodes {
        proptest::sample::select(&SUBJ_IRIS[..]).prop_map(|i| T::Iri(i.to_string())).boxed()
    } else {
        prop_oneof![
            4 => proptest::sample::select(&SUBJ_IRIS[..]).prop_map(|i| T::Iri(i.to_string())),
            1 => proptest::sample::select(&BNODE_LABELS[..]).prop_map(|b| T::BNode(b.to_string())),
        ]
        .boxed()
    }
}

fn arb_triple(mask: DataMask) -> impl Strategy<Value = [T; 3]> {
    (
        arb_subject(mask),
        proptest::sample::select(&PRED_IRIS[..]).prop_map(|p| T::Iri(p.to_string())),
        prop_oneof![2 => arb_subject(mask), 3 => arb_literal(mask)],
    )
        .prop_map(|(s, p, o)| [s, p, o])
}

fn arb_data(mask: DataMask, max: usize) -> impl Strategy<Value = Vec<[T; 3]>> {
    proptest::collection::vec(arb_triple(mask), 0..=max).prop_map(dedup_data)
}

fn arb_pos_s() -> impl Strategy<Value = Pos> {
    prop_oneof![
        3 => (0u8..=3).prop_map(Pos::Var),
        2 => proptest::sample::select(&SUBJ_IRIS[..]).prop_map(|i| Pos::Const(T::Iri(i.to_string()))),
    ]
}

fn arb_pos_p() -> impl Strategy<Value = Pos> {
    prop_oneof![
        1 => (3u8..=4).prop_map(Pos::Var),
        4 => proptest::sample::select(&PRED_IRIS[..]).prop_map(|p| Pos::Const(T::Iri(p.to_string()))),
    ]
}

fn arb_pos_o() -> impl Strategy<Value = Pos> {
    prop_oneof![
        4 => (0u8..=3).prop_map(Pos::Var),
        1 => proptest::sample::select(&SUBJ_IRIS[..]).prop_map(|i| Pos::Const(T::Iri(i.to_string()))),
        2 => arb_literal(DataMask::default()).prop_map(Pos::Const),
    ]
}

fn arb_tp() -> impl Strategy<Value = Tp> {
    (arb_pos_s(), arb_pos_p(), arb_pos_o()).prop_map(|(s, p, o)| Tp { s, p, o })
}

fn arb_bgp(min: usize, max: usize) -> impl Strategy<Value = Vec<Tp>> {
    proptest::collection::vec(arb_tp(), min..=max)
}

/// Force at least one variable so the SELECT projection is never empty.
fn ensure_var(mut body: Body) -> Body {
    if body.vars().is_empty() {
        let fix = |tps: &mut Vec<Tp>| tps[0].s = Pos::Var(0);
        match &mut body {
            Body::Bgp(tps) | Body::Opt(tps, _) | Body::Union(tps, _) => fix(tps),
        }
    }
    body
}

fn arb_body() -> impl Strategy<Value = Body> {
    prop_oneof![
        3 => arb_bgp(1, 3).prop_map(Body::Bgp),
        1 => (arb_bgp(1, 2), arb_bgp(1, 2)).prop_map(|(l, r)| Body::Opt(l, r)),
        1 => (arb_bgp(1, 2), arb_bgp(1, 2)).prop_map(|(a, b)| Body::Union(a, b)),
    ]
    .prop_map(ensure_var)
}

/// Expression constants: IRIs or literals (blank nodes are not legal in expressions).
fn arb_expr_const() -> impl Strategy<Value = T> {
    prop_oneof![
        1 => proptest::sample::select(&SUBJ_IRIS[..]).prop_map(|i| T::Iri(i.to_string())),
        3 => arb_literal(DataMask::default()),
    ]
}

/// Leaf/inner expression seeds; resolved against the query's variable lists in
/// `build_expr` so generation composes without nested `prop_flat_map` towers.
#[derive(Clone, Debug)]
enum ExprSeed {
    EqVV(u8, u8),
    EqVC(u8, T),
    LtVK(u8, i64),
    GtVK(u8, i64),
    Bound(u8),
    Is(u8, u8), // (which of the four is* forms, var seed) — needs a certain var
    Not(Box<ExprSeed>),
    And(Box<ExprSeed>, Box<ExprSeed>),
    Or(Box<ExprSeed>, Box<ExprSeed>),
}

fn arb_expr_seed() -> impl Strategy<Value = ExprSeed> {
    let leaf = prop_oneof![
        (any::<u8>(), any::<u8>()).prop_map(|(a, b)| ExprSeed::EqVV(a, b)),
        (any::<u8>(), arb_expr_const()).prop_map(|(v, c)| ExprSeed::EqVC(v, c)),
        (any::<u8>(), -5i64..=15).prop_map(|(v, k)| ExprSeed::LtVK(v, k)),
        (any::<u8>(), -5i64..=15).prop_map(|(v, k)| ExprSeed::GtVK(v, k)),
        any::<u8>().prop_map(ExprSeed::Bound),
        (0u8..4, any::<u8>()).prop_map(|(w, v)| ExprSeed::Is(w, v)),
    ];
    leaf.prop_recursive(2, 8, 2, |inner| {
        prop_oneof![
            inner.clone().prop_map(|e| ExprSeed::Not(Box::new(e))),
            (inner.clone(), inner.clone()).prop_map(|(a, b)| ExprSeed::And(Box::new(a), Box::new(b))),
            (inner.clone(), inner).prop_map(|(a, b)| ExprSeed::Or(Box::new(a), Box::new(b))),
        ]
    })
}

/// Resolve seed var indices modulo the available variable lists. `filter_vars` is
/// every var the FILTER may mention; `certain` restricts the `is*` forms (header
/// narrowing note: the engine's `is*(unbound)` diverges from SPARQL 17.2).
fn build_expr(seed: &ExprSeed, filter_vars: &[V], certain: &[V]) -> Expr {
    let pick = |s: u8, pool: &[V]| pool[s as usize % pool.len()];
    match seed {
        ExprSeed::EqVV(a, b) => Expr::EqVV(pick(*a, filter_vars), pick(*b, filter_vars)),
        ExprSeed::EqVC(v, c) => Expr::EqVC(pick(*v, filter_vars), c.clone()),
        ExprSeed::LtVK(v, k) => Expr::LtVK(pick(*v, filter_vars), *k),
        ExprSeed::GtVK(v, k) => Expr::GtVK(pick(*v, filter_vars), *k),
        ExprSeed::Bound(v) => Expr::Bound(pick(*v, filter_vars)),
        ExprSeed::Is(which, v) => {
            if certain.is_empty() {
                // no certainly-bound var to apply is* to — degrade to BOUND.
                return Expr::Bound(pick(*v, filter_vars));
            }
            let var = pick(*v, certain);
            match which % 4 {
                0 => Expr::IsIri(var),
                1 => Expr::IsBlank(var),
                2 => Expr::IsLit(var),
                _ => Expr::IsNum(var),
            }
        }
        ExprSeed::Not(e) => Expr::Not(Box::new(build_expr(e, filter_vars, certain))),
        ExprSeed::And(a, b) => Expr::And(
            Box::new(build_expr(a, filter_vars, certain)),
            Box::new(build_expr(b, filter_vars, certain)),
        ),
        ExprSeed::Or(a, b) => Expr::Or(
            Box::new(build_expr(a, filter_vars, certain)),
            Box::new(build_expr(b, filter_vars, certain)),
        ),
    }
}

/// Independent raw seeds assembled into a valid query by `build_query` — this keeps
/// the strategy flat (no nested `prop_flat_map` towers) while every choice still
/// shrinks toward simpler queries.
#[derive(Clone, Debug)]
struct QSeed {
    body: Body,
    bind: Option<(u8, u8, i64)>, // (form, var seed, k)
    filter: Option<ExprSeed>,
    distinct: bool,
    proj_mask: u16,
    order: Option<(u8, bool)>, // (projected-var seed, desc)
    limit: Option<usize>,
}

fn arb_qseed() -> impl Strategy<Value = QSeed> {
    (
        arb_body(),
        proptest::option::weighted(0.35, (0u8..3, any::<u8>(), 0i64..=3)),
        proptest::option::weighted(0.45, arb_expr_seed()),
        proptest::bool::weighted(0.25),
        any::<u16>(),
        proptest::option::weighted(0.35, (any::<u8>(), any::<bool>())),
        proptest::option::weighted(0.2, 0usize..=8),
    )
        .prop_map(|(body, bind, filter, distinct, proj_mask, order, limit)| QSeed {
            body,
            bind,
            filter,
            distinct,
            proj_mask,
            order,
            limit,
        })
}

fn build_query(seed: QSeed) -> Q {
    let body_vars = seed.body.vars();
    let certain = seed.body.certain_vars();
    let bind = seed.bind.map(|(form, vs, k)| {
        let v = body_vars[vs as usize % body_vars.len()];
        match form % 3 {
            0 => Bind::PlusK(v, k),
            1 => Bind::LtK(v, k),
            _ => Bind::StrLenOf(v),
        }
    });
    let mut scope = body_vars.clone();
    if bind.is_some() {
        scope.push(BIND_VAR);
    }
    // FILTER may mention any in-scope var (including the BIND target: BIND
    // precedes FILTER in the rendered group, so it is visible there).
    let filter = seed.filter.as_ref().map(|f| build_expr(f, &scope, &certain));
    // Projection: the mask-selected subset of in-scope vars; never empty.
    let mut project: Vec<V> = scope
        .iter()
        .enumerate()
        .filter(|(i, _)| seed.proj_mask & (1 << (i % 16)) != 0)
        .map(|(_, v)| *v)
        .collect();
    if project.is_empty() {
        project = scope.clone();
    }
    let order = seed.order.map(|(vs, desc)| (project[vs as usize % project.len()], desc));
    Q {
        body: seed.body,
        bind,
        filter,
        distinct: seed.distinct,
        project,
        order,
        limit: seed.limit,
    }
}

/// Pool of subject-position terms for pattern instantiation, mask-respecting.
fn subject_pool(mask: DataMask) -> Vec<T> {
    let mut out: Vec<T> = SUBJ_IRIS.iter().map(|i| T::Iri(i.to_string())).collect();
    if !mask.no_bnodes {
        out.extend(BNODE_LABELS.iter().map(|b| T::BNode(b.to_string())));
    }
    out
}

/// Pool of object-position terms for pattern instantiation, mask-respecting.
fn object_pool(mask: DataMask) -> Vec<T> {
    let mut out = subject_pool(mask);
    out.extend([-2i64, 0, 1, 3, 17].map(T::Int));
    out.extend([T::Dec(150, 2), T::Dec(-25, 2), T::Dec(40, 1)]);
    out.extend(STR_POOL.iter().map(|s| T::Str(s.to_string())));
    out.push(T::Lang("a".to_string(), "en".to_string()));
    out.extend([T::Bool(true), T::Bool(false)]);
    if !mask.no_doubles {
        out.extend(DBL_POOL.iter().map(|(l, v)| T::Dbl(l.to_string(), *v)));
    }
    out
}

/// Instantiate every pattern of `body` under ONE shared variable assignment decoded
/// from `seed` — the injected triples then satisfy each BGP simultaneously, so
/// generated cases produce nonempty (and join-exercising) results at a useful rate
/// instead of comparing empty-vs-empty (the `generator_diversity_floor` guard).
/// Variables are classified by the strictest position they occupy: predicate
/// position needs a predicate IRI, subject position an IRI/bnode, object-only
/// positions may take any term.
fn instantiate(body: &Body, seed: &[u8], mask: DataMask) -> Vec<[T; 3]> {
    let vars = body.vars();
    let all_tps: Vec<&Tp> = match body {
        Body::Bgp(tps) => tps.iter().collect(),
        Body::Opt(l, r) | Body::Union(l, r) => l.iter().chain(r.iter()).collect(),
    };
    let position_class = |v: &V| {
        let mut class = 2u8; // 0 = predicate, 1 = subject, 2 = object-only
        for tp in &all_tps {
            if tp.p == Pos::Var(*v) {
                class = 0;
            } else if tp.s == Pos::Var(*v) && class > 1 {
                class = 1;
            }
        }
        class
    };
    let subj = subject_pool(mask);
    let obj = object_pool(mask);
    let assignment: Vec<(V, T)> = vars
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let s = *seed.get(i).unwrap_or(&0) as usize;
            let t = match position_class(v) {
                0 => T::Iri(PRED_IRIS[s % PRED_IRIS.len()].to_string()),
                1 => subj[s % subj.len()].clone(),
                _ => obj[s % obj.len()].clone(),
            };
            (*v, t)
        })
        .collect();
    let resolve = |p: &Pos| match p {
        Pos::Const(t) => t.clone(),
        Pos::Var(v) => assignment.iter().find(|(av, _)| av == v).expect("assigned var").1.clone(),
    };
    all_tps.iter().map(|tp| [resolve(&tp.s), resolve(&tp.p), resolve(&tp.o)]).collect()
}

/// A full family-1 case: the query is generated first so the graph generator can
/// apply the query-derived masks (header narrowing notes); the graph is a blend of
/// independent random triples and 0..=2 shared-assignment instantiations of the
/// query's own patterns (see `instantiate`).
fn arb_case() -> impl Strategy<Value = (Vec<[T; 3]>, Q)> {
    arb_qseed().prop_map(build_query).prop_flat_map(|q| {
        let mask = mask_of(&q);
        let nvars = q.body.vars().len();
        (
            arb_data(mask, 14),
            proptest::collection::vec(proptest::collection::vec(any::<u8>(), nvars), 0..=2),
            Just(q),
        )
            .prop_map(move |(mut data, seeds, q)| {
                for seed in &seeds {
                    data.extend(instantiate(&q.body, seed, mask));
                }
                (dedup_data(data), q)
            })
    })
}

// ═══════════════════════════════════════════════════════════════════════════════
// Family 1: engine ≡ naive reference (every compiled plan path)
// ═══════════════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 96,
        ..ProptestConfig::default()
    })]

    /// Core oracle property: for a random graph and a random query, the engine's
    /// solution multiset equals the naive reference's, on every compiled plan path;
    /// ORDER BY output additionally respects the SPARQL ordering; LIMIT is checked
    /// as sub-multiset + cardinality (implementation-defined slice, §2.2).
    #[test]
    fn family1_engine_matches_naive_reference((data, q) in arb_case()) {
        let g = Graph::load_str(&to_ntriples(&data), "ntriples").expect("generated N-Triples parse");
        let qtext = q.render();
        for (label, rows) in engine_paths(&g, &qtext) {
            check_case(&data, &q, label, &rows)?;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Family 2: join-order permutation invariance
// ═══════════════════════════════════════════════════════════════════════════════

fn arb_perm_case() -> impl Strategy<Value = (Vec<[T; 3]>, Vec<Tp>, Vec<Tp>, Option<ExprSeed>)> {
    (arb_bgp(2, 4).prop_map(|b| match ensure_var(Body::Bgp(b)) { Body::Bgp(t) => t, _ => unreachable!() }))
        .prop_flat_map(|tps| {
            let nvars = Body::Bgp(tps.clone()).vars().len();
            (
                arb_data(DataMask::default(), 14),
                proptest::collection::vec(proptest::collection::vec(any::<u8>(), nvars), 0..=2),
                Just(tps.clone()),
                Just(tps).prop_shuffle(),
                proptest::option::weighted(0.4, arb_expr_seed()),
            )
                .prop_map(|(mut data, seeds, tps, shuffled, fseed)| {
                    for seed in &seeds {
                        data.extend(instantiate(&Body::Bgp(tps.clone()), seed, DataMask::default()));
                    }
                    (dedup_data(data), tps, shuffled, fseed)
                })
        })
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 64,
        ..ProptestConfig::default()
    })]

    /// Permuting the triple patterns of a BGP (with an optional FILTER on top) must
    /// not change the solution multiset — the planner may reorder joins for cost
    /// only. Checked engine-vs-engine on every compiled plan path.
    #[test]
    fn family2_join_order_invariance((data, tps, shuffled, fseed) in arb_perm_case()) {
        let build = |patterns: Vec<Tp>| {
            let body = Body::Bgp(patterns);
            let vars = body.vars();
            let certain = body.certain_vars();
            let filter = fseed.as_ref().map(|f| build_expr(f, &vars, &certain));
            Q { body, bind: None, filter, distinct: false, project: vars, order: None, limit: None }
        };
        let q1 = build(tps);
        let q2 = build(shuffled);
        let g = Graph::load_str(&to_ntriples(&data), "ntriples").expect("generated N-Triples parse");
        for (label, rows1) in engine_paths(&g, &q1.render()) {
            let rows2 = engine_rows(&g, &q2.render());
            prop_assert_eq!(
                multiset(&rows1), multiset(&rows2),
                "join-order variance on path {}:\n{}\nvs\n{}\ndata:\n{}",
                label, q1.render(), q2.render(), to_ntriples(&data)
            );
        }
        // and both agree with the oracle (ties family 2 to the independent spec).
        for (label, rows1) in engine_paths(&g, &q1.render()) {
            check_case(&data, &q1, label, &rows1)?;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Family 3: ORDER BY laws on tie-free integer keys (exact sequences)
// ═══════════════════════════════════════════════════════════════════════════════

fn arb_orderby_case() -> impl Strategy<Value = (Vec<(usize, i64)>, usize)> {
    (
        proptest::collection::btree_set(-50i64..=50, 1..=12),
        0usize..=14,
    )
        .prop_map(|(vals, k)| {
            let pairs: Vec<(usize, i64)> = vals.into_iter().enumerate().collect();
            (pairs, k)
        })
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 64,
        ..ProptestConfig::default()
    })]

    /// With pairwise-distinct integer sort keys the order is total, so exact
    /// sequence laws hold: ASC equals the oracle's sort, DESC is its exact
    /// reverse, and `LIMIT k` is the exact k-prefix of the full ordered result.
    #[test]
    fn family3_orderby_exact_sequence_laws((pairs, k) in arb_orderby_case()) {
        // one triple per distinct integer value; subjects may repeat.
        let data: Vec<[T; 3]> = pairs
            .iter()
            .map(|(i, v)| {
                [T::Iri(SUBJ_IRIS[i % SUBJ_IRIS.len()].to_string()), T::Iri(PRED_IRIS[0].to_string()), T::Int(*v)]
            })
            .collect();
        let g = Graph::load_str(&to_ntriples(&data), "ntriples").expect("generated N-Triples parse");
        let render_row = |(i, v): &(usize, i64)| {
            vec![
                Some(format!("<{}>", SUBJ_IRIS[i % SUBJ_IRIS.len()])),
                Some(T::Int(*v).render()),
            ]
        };
        let mut expected_asc: Vec<Vec<Option<String>>> = pairs.iter().map(render_row).collect();
        expected_asc.sort_by_key(|row| {
            // sort by the integer value parsed back from the rendered cell — pairs
            // are already value-sorted (BTreeSet), this keeps the oracle honest.
            match parse_rendered(row[1].as_deref().unwrap()) {
                T::Int(n) => n,
                other => panic!("non-integer key {:?}", other),
            }
        });
        let base = format!("SELECT ?s ?o WHERE {{ ?s <{}> ?o }}", PRED_IRIS[0]);
        for (label, asc) in engine_paths(&g, &format!("{} ORDER BY ?o", base)) {
            prop_assert_eq!(&asc, &expected_asc, "ASC exact sequence, path {}", label);
        }
        let mut expected_desc = expected_asc.clone();
        expected_desc.reverse();
        for (label, desc) in engine_paths(&g, &format!("{} ORDER BY DESC(?o)", base)) {
            prop_assert_eq!(&desc, &expected_desc, "DESC is exact reverse of ASC, path {}", label);
        }
        let prefix: Vec<_> = expected_asc.iter().take(k).cloned().collect();
        for (label, lim) in engine_paths(&g, &format!("{} ORDER BY ?o LIMIT {}", base, k)) {
            prop_assert_eq!(&lim, &prefix, "ORDER BY + LIMIT is the exact prefix, path {}", label);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Oracle known-answer pins (a degenerate oracle cannot silently agree)
// ═══════════════════════════════════════════════════════════════════════════════

/// OPTIONAL + shared-var join, hand-computed. Data:
///   s0 p0 s1 ; s1 p0 s2 ; s0 p1 "a"
/// Query: SELECT ?v0 ?v1 ?v2 WHERE { ?v0 <p0> ?v1 OPTIONAL { ?v1 <p0> ?v2 } }
/// Expected rows: (s0, s1, s2) and (s1, s2, unbound).
#[test]
fn oracle_known_answer_optional() {
    let s = |i: usize| T::Iri(SUBJ_IRIS[i].to_string());
    let p = |i: usize| T::Iri(PRED_IRIS[i].to_string());
    let data = vec![
        [s(0), p(0), s(1)],
        [s(1), p(0), s(2)],
        [s(0), p(1), T::Str("a".to_string())],
    ];
    let q = Q {
        body: Body::Opt(
            vec![Tp { s: Pos::Var(0), p: Pos::Const(p(0)), o: Pos::Var(1) }],
            vec![Tp { s: Pos::Var(1), p: Pos::Const(p(0)), o: Pos::Var(2) }],
        ),
        bind: None,
        filter: None,
        distinct: false,
        project: vec![0, 1, 2],
        order: None,
        limit: None,
    };
    let expected = vec![
        vec![Some(s(0).render()), Some(s(1).render()), Some(s(2).render())],
        vec![Some(s(1).render()), Some(s(2).render()), None],
    ];
    assert_eq!(multiset(&oracle_eval(&data, &q)), multiset(&expected), "oracle vs hand-computed");
    let g = Graph::load_str(&to_ntriples(&data), "ntriples").unwrap();
    assert_eq!(multiset(&engine_rows(&g, &q.render())), multiset(&expected), "engine vs hand-computed");
}

/// UNION + DISTINCT + subset projection + FILTER, hand-computed. Data:
///   s0 p0 1 ; s1 p0 2 ; s0 p1 1
/// Query: SELECT DISTINCT ?v0 WHERE { { ?v0 <p0> ?v1 } UNION { ?v0 <p1> ?v1 }
///                                    FILTER((?v1 < 2)) }
/// Branch rows: (s0,1) (s1,2) | (s0,1); filter keeps ?v1=1 rows; projection to ?v0
/// gives s0, s0; DISTINCT gives one s0.
#[test]
fn oracle_known_answer_union_distinct_filter() {
    let s = |i: usize| T::Iri(SUBJ_IRIS[i].to_string());
    let p = |i: usize| T::Iri(PRED_IRIS[i].to_string());
    let data = vec![
        [s(0), p(0), T::Int(1)],
        [s(1), p(0), T::Int(2)],
        [s(0), p(1), T::Int(1)],
    ];
    let q = Q {
        body: Body::Union(
            vec![Tp { s: Pos::Var(0), p: Pos::Const(p(0)), o: Pos::Var(1) }],
            vec![Tp { s: Pos::Var(0), p: Pos::Const(p(1)), o: Pos::Var(1) }],
        ),
        bind: None,
        filter: Some(Expr::LtVK(1, 2)),
        distinct: true,
        project: vec![0],
        order: None,
        limit: None,
    };
    let expected = vec![vec![Some(s(0).render())]];
    assert_eq!(multiset(&oracle_eval(&data, &q)), multiset(&expected), "oracle vs hand-computed");
    let g = Graph::load_str(&to_ntriples(&data), "ntriples").unwrap();
    assert_eq!(multiset(&engine_rows(&g, &q.render())), multiset(&expected), "engine vs hand-computed");
    // without DISTINCT the duplicate survives — pins multiset (not set) semantics.
    let q_bag = Q { distinct: false, ..q };
    let expected_bag = vec![vec![Some(s(0).render())], vec![Some(s(0).render())]];
    assert_eq!(multiset(&oracle_eval(&data, &q_bag)), multiset(&expected_bag), "bag semantics");
    assert_eq!(multiset(&engine_rows(&g, &q_bag.render())), multiset(&expected_bag), "engine bag semantics");
}

/// BIND forms, hand-computed: PlusK over int/decimal/error, LtK boolean/error,
/// STRLEN(STR()) over IRI and literals.
#[test]
fn oracle_known_answer_bind_forms() {
    let s = |i: usize| T::Iri(SUBJ_IRIS[i].to_string());
    let p = |i: usize| T::Iri(PRED_IRIS[i].to_string());
    let data = vec![
        [s(0), p(0), T::Int(41)],
        [s(1), p(0), T::Dec(150, 2)],
        [s(2), p(0), T::Str("hi".to_string())],
    ];
    let g = Graph::load_str(&to_ntriples(&data), "ntriples").unwrap();
    let mk = |bind: Bind| Q {
        body: Body::Bgp(vec![Tp { s: Pos::Var(0), p: Pos::Const(p(0)), o: Pos::Var(1) }]),
        bind: Some(bind),
        filter: None,
        distinct: false,
        project: vec![0, 1, BIND_VAR],
        order: None,
        limit: None,
    };
    for q in [mk(Bind::PlusK(1, 1)), mk(Bind::LtK(1, 42)), mk(Bind::StrLenOf(1))] {
        let oracle = oracle_eval(&data, &q);
        let engine = engine_rows(&g, &q.render());
        assert_eq!(multiset(&engine), multiset(&oracle), "BIND {:?}", q.bind);
    }
    // Spot-pin exact computed values so the oracle's arithmetic is itself checked:
    let q = mk(Bind::PlusK(1, 1));
    let oracle = oracle_eval(&data, &q);
    let cells: std::collections::HashSet<String> =
        oracle.iter().map(|r| r[2].clone().unwrap_or_else(|| "UNBOUND".to_string())).collect();
    assert!(cells.contains(&format!("\"42\"^^<{}integer>", XSD)), "int + 1");
    assert!(cells.contains(&format!("\"2.50\"^^<{}decimal>", XSD)), "decimal scale preserved");
    assert!(cells.contains("UNBOUND"), "type-error BIND leaves the var unbound");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Generator diversity floor (anti-vacuity: the corpus is not trivially empty)
// ═══════════════════════════════════════════════════════════════════════════════

/// Samples the family-1 case strategy with a FIXED seed (deterministic) and
/// asserts the corpus contains the situations the properties are supposed to
/// exercise. If a generator change starves the corpus, this fails loudly rather
/// than letting the properties pass vacuously.
#[test]
fn generator_diversity_floor() {
    use proptest::strategy::ValueTree;
    use proptest::test_runner::{Config, RngAlgorithm, TestRng, TestRunner};
    let mut runner = TestRunner::new_with_rng(
        Config::default(),
        TestRng::from_seed(RngAlgorithm::ChaCha, &[7u8; 32]),
    );
    let strategy = arb_case();
    let n = 300;
    let (mut nonempty, mut dups, mut unbound_cells, mut multi_row) = (0, 0, 0, 0);
    let (mut bgp, mut opt, mut union, mut filt, mut bind, mut distinct, mut order, mut limit) =
        (0, 0, 0, 0, 0, 0, 0, 0);
    for _ in 0..n {
        let (data, q) = strategy.new_tree(&mut runner).expect("gen").current();
        match q.body {
            Body::Bgp(_) => bgp += 1,
            Body::Opt(..) => opt += 1,
            Body::Union(..) => union += 1,
        }
        filt += usize::from(q.filter.is_some());
        bind += usize::from(q.bind.is_some());
        distinct += usize::from(q.distinct);
        order += usize::from(q.order.is_some());
        limit += usize::from(q.limit.is_some());
        let rows = oracle_eval(&data, &q);
        nonempty += usize::from(!rows.is_empty());
        multi_row += usize::from(rows.len() > 1);
        let ms = multiset(&rows);
        dups += usize::from(ms.windows(2).any(|w| w[0] == w[1]));
        unbound_cells += usize::from(rows.iter().any(|r| r.iter().any(Option::is_none)));
    }
    eprintln!(
        "diversity: nonempty={}/{} multi_row={} dups={} unbound={} shapes: bgp={} opt={} union={} filter={} bind={} distinct={} order={} limit={}",
        nonempty, n, multi_row, dups, unbound_cells, bgp, opt, union, filt, bind, distinct, order, limit
    );
    // Conservative floors — deterministic because the seed is fixed.
    assert!(nonempty * 100 >= n * 25, "nonempty results: {}/{}", nonempty, n);
    assert!(multi_row * 100 >= n * 15, "multi-row results: {}/{}", multi_row, n);
    assert!(dups >= 5, "duplicate-containing multisets: {}", dups);
    assert!(unbound_cells >= 5, "results with unbound cells: {}", unbound_cells);
    for (label, count) in [
        ("bgp", bgp), ("optional", opt), ("union", union), ("filter", filt),
        ("bind", bind), ("distinct", distinct), ("order", order), ("limit", limit),
    ] {
        assert!(count >= 10, "query shape {} generated only {} times in {}", label, count, n);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Unit pins for the comparison plumbing itself
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn parse_rendered_roundtrips_every_generated_shape() {
    let terms = [
        T::Iri("http://ex/s0".to_string()),
        T::BNode("b1".to_string()),
        T::Str("hi".to_string()),
        T::Str(String::new()),
        T::Lang("a".to_string(), "en".to_string()),
        T::Int(-42),
        T::Dec(-25, 2),
        T::Dec(4000, 1),
        T::Dbl("1.5E0".to_string(), 1.5),
        T::Dbl("-INF".to_string(), f64::NEG_INFINITY),
        T::Bool(true),
    ];
    for t in terms {
        assert_eq!(parse_rendered(&t.render()), t, "round-trip of {:?}", t);
    }
}

#[test]
fn check_sorted_detects_violations() {
    // an inverted numeric pair must be flagged...
    let bad = vec![Some(T::Int(2)), Some(T::Int(1))];
    assert!(check_sorted(&bad, false, "unit").is_err());
    // ...but is fine descending.
    assert!(check_sorted(&bad, true, "unit").is_ok());
    // unbound must come first ascending,
    let bad2 = vec![Some(T::Int(1)), None];
    assert!(check_sorted(&bad2, false, "unit").is_err());
    // cross-class literal pairs are unconstrained in either direction,
    let free = vec![Some(T::Str("z".to_string())), Some(T::Int(1))];
    assert!(check_sorted(&free, false, "unit").is_ok());
    assert!(check_sorted(&free, true, "unit").is_ok());
    // and value-equal cross-type numerics are a tie, not a violation.
    let tie = vec![Some(T::Dec(10, 1)), Some(T::Int(1))];
    assert!(check_sorted(&tie, false, "unit").is_ok());
}

#[test]
fn render_dec_matches_engine_scale_rules() {
    assert_eq!(render_dec(250, 2), "2.50");
    assert_eq!(render_dec(-25, 2), "-0.25");
    assert_eq!(render_dec(40, 1), "4.0");
    assert_eq!(render_dec(0, 2), "0.00");
    assert_eq!(render_dec(-3000, 2), "-30.00");
}
