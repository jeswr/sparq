//! N3 surface-syntax **serialization** — the crate's single writer for [`Term`]s, ground
//! statements, and (`sq-xqchl.2`) whole RULES.
//!
//! The term/statement writers moved here from `incremental` (which used them as the
//! fallback / differential-oracle path) so the rule writer below shares ONE definition of
//! how a term is rendered — a second copy is how a serializer and its parser drift apart.
//! Lossless for parser-shaped terms: blank labels round-trip verbatim, language tags are
//! already lowercase, plain strings re-acquire `xsd:string`.
//!
//! # Echoing rules back out (EYE `--pass-all` / `--pass-all-ground`)
//!
//! The forward chainer CONSUMES rules — [`crate::reason_n3`] returns only entailed ground
//! facts — so a caller that wants EYE's "deductive closure PLUS the rules" output needs the
//! rules rendered back into N3. [`write_rule`] does that from the PARSED rule (not the
//! source text), which is why two details differ from the input document verbatim:
//!
//! * **Premise blank nodes.** The parser rewrites a rule-scoped premise blank `_:x` to a
//!   fresh rule variable `?__bn<rule>_x` (N3 semantics: a premise existential matches like a
//!   variable). [`write_rule`] maps that name back to `_:x`, so the emitted rule re-parses
//!   to the same rule rather than leaking an engine-internal variable name.
//! * **Prefixes / layout.** Everything is written in full `<…>` IRI form, one statement per
//!   line — no `@prefix` declarations are reconstructed. The document is semantically the
//!   same N3, not byte-identical to the input.

use super::model::{Rule, Term};

const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";

/// The SWAP **variable** namespace. A universal rule variable `?x` written as
/// `<http://www.w3.org/2000/10/swap/var#x>` is EYE's convention for a rule carried in a
/// document that must contain no syntactic variables — see [`RuleVars::VarIris`].
pub const VAR_NS: &str = "http://www.w3.org/2000/10/swap/var#";

/// The engine-internal prefix the N3 parser gives a rewritten premise blank node
/// (`_:x` in rule `i` becomes `?__bn{i}_x`) — reversed by [`write_rule`].
const PREMISE_BLANK_VAR: &str = "__bn";

fn quote_into(v: &str, out: &mut String) {
    for c in v.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
}

/// Write one N3 term in its surface syntax: IRIs `<…>`, literals `"lex"` (+ `@lang` /
/// `^^<dt>`, `xsd:string` left implicit), blanks `_:l`, variables `?v`, lists `( … )`,
/// formulae `{ … }`, RDF-star quoted triples `<< s p o >>`.
pub fn write_term(t: &Term, out: &mut String) {
    match t {
        Term::Iri(i) => {
            out.push('<');
            out.push_str(i);
            out.push('>');
        }
        Term::Lit(v, _, Some(lang)) => {
            out.push('"');
            quote_into(v, out);
            out.push('"');
            out.push('@');
            out.push_str(lang);
        }
        Term::Lit(v, dt, None) => {
            out.push('"');
            quote_into(v, out);
            out.push('"');
            if dt != XSD_STRING {
                out.push_str("^^<");
                out.push_str(dt);
                out.push('>');
            }
        }
        Term::Blank(l) => {
            out.push_str("_:");
            out.push_str(l);
        }
        Term::Var(v) => {
            out.push('?');
            out.push_str(v);
        }
        Term::List(ms) => {
            out.push('(');
            for m in ms {
                out.push(' ');
                write_term(m, out);
            }
            out.push_str(" )");
        }
        Term::Formula(ts) => {
            out.push('{');
            for t in ts {
                out.push(' ');
                write_term(&t[0], out);
                out.push(' ');
                write_term(&t[1], out);
                out.push(' ');
                write_term(&t[2], out);
                out.push_str(" .");
            }
            out.push_str(" }");
        }
        // RDF-star quoted-triple term — round-trips through the N3 parser's
        // `<< s p o >>` form (GH #2012). [FABLE-5]
        Term::Triple(tr) => {
            out.push_str("<< ");
            write_term(&tr[0], out);
            out.push(' ');
            write_term(&tr[1], out);
            out.push(' ');
            write_term(&tr[2], out);
            out.push_str(" >>");
        }
    }
}

/// Write one ground statement as `s p o .` plus a newline.
pub fn write_statement(f: &[Term; 3], out: &mut String) {
    write_term(&f[0], out);
    out.push(' ');
    write_term(&f[1], out);
    out.push(' ');
    write_term(&f[2], out);
    out.push_str(" .\n");
}

/// Serialize ground facts back to N3 (the fallback / differential-oracle path).
pub fn serialize_facts<'a>(facts: impl Iterator<Item = &'a [Term; 3]>) -> String {
    let mut out = String::new();
    for f in facts {
        write_statement(f, &mut out);
    }
    out
}

/// How a rule's universal variables are written when the rule is echoed into an output
/// document ([`write_rule`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuleVars {
    /// N3 variables, `?x` — EYE `--pass-all`. The emitted rule re-parses as the SAME rule,
    /// so the document can be fed straight back into the reasoner.
    N3,
    /// SWAP `var:` IRIs, `<http://www.w3.org/2000/10/swap/var#x>` ([`VAR_NS`]) — EYE
    /// `--pass-all-ground`. No syntactic variable survives, so the whole document is
    /// ordinary RDF-shaped N3 for a downstream consumer that cannot handle `?x`. It is
    /// NOT re-reasonable: re-parsed, those IRIs are constants, and the rule only fires on
    /// literally-matching `var:` data. Blank nodes are left as blank nodes.
    VarIris,
}

/// Which arrow a rule is written with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuleKind {
    /// A forward rule, `{ premise } => { conclusion } .` (`log:implies`).
    Forward,
    /// A goal-directed backward rule, `{ conclusion } <= { premise } .` (`log:isImpliedBy`).
    Backward,
}

/// Write one rule as an N3 statement (`{ … } => { … } .` / `{ … } <= { … } .`) plus a
/// newline, rendering its variables per `vars`.
///
/// The premise-blank rewrite the parser applies is undone first — see the module docs — so
/// a `RuleVars::N3` round trip yields an equivalent rule.
pub fn write_rule(r: &Rule, kind: RuleKind, vars: RuleVars, out: &mut String) {
    let (left, arrow, right) = match kind {
        RuleKind::Forward => (&r.premise, " => ", &r.conclusion),
        RuleKind::Backward => (&r.conclusion, " <= ", &r.premise),
    };
    write_formula(left, vars, out);
    out.push_str(arrow);
    write_formula(right, vars, out);
    out.push_str(" .\n");
}

/// `{ s p o . … }` over rule-side statements, with each term put through [`rule_term`].
fn write_formula(stmts: &[[Term; 3]], vars: RuleVars, out: &mut String) {
    out.push('{');
    for row in stmts {
        for t in row {
            out.push(' ');
            write_term(&rule_term(t, vars), out);
        }
        out.push_str(" .");
    }
    out.push_str(" }");
}

/// A rule-side term prepared for output: the parser's `?__bn<i>_x` premise-blank variables
/// become `_:x` again, and — under [`RuleVars::VarIris`] — every remaining variable becomes
/// its `var:` IRI.
///
/// Recursion mirrors the parser's own rule rewrite (`rewrite_term`, `into_formulae = false`):
/// it descends through lists and quoted triples, which are transparent rule structure, but
/// NOT into a quoted `{ … }` formula, whose variables belong to that formula.
fn rule_term(t: &Term, vars: RuleVars) -> Term {
    match t {
        Term::List(ms) => Term::List(ms.iter().map(|m| rule_term(m, vars)).collect()),
        Term::Triple(tr) => Term::Triple(Box::new([
            rule_term(&tr[0], vars),
            rule_term(&tr[1], vars),
            rule_term(&tr[2], vars),
        ])),
        Term::Var(v) => match premise_blank_label(v) {
            Some(label) => Term::Blank(label.to_string()),
            None if vars == RuleVars::VarIris => Term::Iri(format!("{VAR_NS}{v}")),
            None => t.clone(),
        },
        _ => t.clone(),
    }
}

/// The original blank-node label behind a rewritten premise blank `__bn<digits>_<label>`,
/// or `None` for an ordinary variable.
fn premise_blank_label(v: &str) -> Option<&str> {
    let rest = v.strip_prefix(PREMISE_BLANK_VAR)?;
    let (digits, label) = rest.split_once('_')?;
    (!digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())).then_some(label)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rendered(t: &Term) -> String {
        let mut s = String::new();
        write_term(t, &mut s);
        s
    }

    #[test]
    fn writes_each_term_kind() {
        assert_eq!(rendered(&Term::Iri("http://ex/a".into())), "<http://ex/a>");
        assert_eq!(rendered(&Term::Blank("b0".into())), "_:b0");
        assert_eq!(rendered(&Term::Var("x".into())), "?x");
        assert_eq!(rendered(&Term::Lit("hi".into(), XSD_STRING.into(), None)), "\"hi\"");
        assert_eq!(
            rendered(&Term::Lit("1".into(), "http://ex/d".into(), None)),
            "\"1\"^^<http://ex/d>"
        );
        let tagged = Term::Lit("hi".into(), XSD_STRING.into(), Some("en".into()));
        assert_eq!(rendered(&tagged), "\"hi\"@en");
        assert_eq!(rendered(&Term::List(vec![Term::Var("x".into())])), "( ?x )");
        assert_eq!(rendered(&Term::Formula(vec![])), "{ }");
    }

    #[test]
    fn quotes_escapes() {
        let t = Term::Lit("a\"b\\c\nd".into(), XSD_STRING.into(), None);
        assert_eq!(rendered(&t), "\"a\\\"b\\\\c\\nd\"");
    }

    #[test]
    fn recognises_only_well_formed_premise_blank_names() {
        assert_eq!(premise_blank_label("__bn0_x"), Some("x"));
        assert_eq!(premise_blank_label("__bn12_a_b"), Some("a_b"));
        assert_eq!(premise_blank_label("__bnx_y"), None); // no digits
        assert_eq!(premise_blank_label("__bn0"), None); // no separator
        assert_eq!(premise_blank_label("x"), None);
    }

    #[test]
    fn var_iris_grounding_skips_blanks_and_nested_formulae() {
        let inner = Term::Formula(vec![[
            Term::Var("y".into()),
            Term::Iri("http://ex/p".into()),
            Term::Var("y".into()),
        ]]);
        // A nested formula's variables are formula-scoped: left alone.
        assert_eq!(rendered(&rule_term(&inner, RuleVars::VarIris)), rendered(&inner));
        // A rewritten premise blank goes back to `_:x` in BOTH styles.
        for style in [RuleVars::N3, RuleVars::VarIris] {
            let t = rule_term(&Term::Var("__bn3_x".into()), style);
            assert_eq!(rendered(&t), "_:x");
        }
        assert_eq!(
            rendered(&rule_term(&Term::Var("z".into()), RuleVars::VarIris)),
            "<http://www.w3.org/2000/10/swap/var#z>"
        );
    }
}
