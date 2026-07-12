//! [FABLE-5] sq-ohnj1 — the eye-js `--query` compat translator.
//!
//! eye-js's `n3reasoner(data, query)` runs EYE with `--query queryfile`: the query file is an
//! N3 rule document `{ premise } => { conclusion }`, and EYE outputs every INSTANTIATED
//! conclusion `Cθ` for each binding `θ` that satisfies the premise over the deductive closure
//! of `data` (a SELECT/CONSTRUCT over the closure — NOT a monotone "new facts" delta, since a
//! query conclusion already present in the closure is still emitted). sparq has no `--query`
//! primitive, so the compat layer reproduces it as a SPARQL `CONSTRUCT { conclusion } WHERE
//! { premise }` evaluated over the materialised closure (`sparq_engine::construct_ntriples`).
//!
//! This module is the pure `N3-query-rule -> CONSTRUCT-string` translator. It is deliberately
//! FAIL-CLOSED: a query rule whose premise uses an N3 builtin (`math:`/`string:`/`list:`/
//! `log:`/`time:`), or any term this compat path cannot yet faithfully express as a plain
//! SPARQL BGP (a quoted `{ … }` formula, a first-class `( … )` list, or a quoted
//! `<< s p o >>` triple term), is REJECTED with a clear error rather than
//! silently mistranslated into a triple-pattern match (which would return WRONG answers). The
//! deferred surface is tracked as a follow-up bead and documented in the package README.

use sparq_reason::n3::parser;
use sparq_reason::n3::Term;

/// Swap builtin namespaces whose predicates are EVALUATED by EYE (not matched as data). A BGP
/// cannot evaluate them, so a query premise using one is rejected (fail-closed) rather than
/// mistranslated into a triple pattern that would silently return wrong answers.
const BUILTIN_NAMESPACES: &[&str] = &[
    "http://www.w3.org/2000/10/swap/math#",
    "http://www.w3.org/2000/10/swap/string#",
    "http://www.w3.org/2000/10/swap/list#",
    "http://www.w3.org/2000/10/swap/log#",
    "http://www.w3.org/2000/10/swap/time#",
];

/// Translate an eye-js N3 query document into one SPARQL `CONSTRUCT` per forward rule.
///
/// Returns one CONSTRUCT string per `{ premise } => { conclusion }` rule in the document (a
/// query document usually has exactly one). Errors — fail-closed — when the document has no
/// forward rule, or a rule uses a builtin / formula / list the compat filter does not support.
pub(crate) fn n3_query_to_constructs(query: &str) -> Result<Vec<String>, String> {
    let parsed = parser::parse(query).map_err(|e| format!("compat query: parse error: {}", e))?;
    if parsed.rules.is_empty() {
        return Err(
            "compat query filter: the query document contains no `{ … } => { … }` forward rule. \
             Only forward-rule (SELECT/CONSTRUCT-style) queries are supported (backward rules and \
             fact-only documents are out of scope for v1 — see the package README)."
                .to_string(),
        );
    }
    let mut out = Vec::with_capacity(parsed.rules.len());
    for rule in &parsed.rules {
        let where_bgp = render_bgp(&rule.premise)?;
        let template = render_bgp(&rule.conclusion)?;
        // Positional format args (not inline `{template}`) so CodeQL's rust/unused-variable
        // false positive does not fire on this crate.
        out.push(format!("CONSTRUCT {{ {} }} WHERE {{ {} }}", template, where_bgp));
    }
    Ok(out)
}

/// Render a list of N3 triple rows as a SPARQL basic graph pattern (`s p o . …`).
fn render_bgp(rows: &[[Term; 3]]) -> Result<String, String> {
    let mut out = String::new();
    for row in rows {
        // A builtin PREDICATE is evaluated by EYE, not matched as data — reject fail-closed.
        if let Term::Iri(p) = &row[1] {
            if BUILTIN_NAMESPACES.iter().any(|ns| p.starts_with(ns)) {
                return Err(format!(
                    "compat query filter: the query premise uses the N3 builtin <{}>, which the \
                     v1 SPARQL-CONSTRUCT compat path cannot evaluate (a BGP would silently match \
                     it as data). Builtin-bearing query rules are not yet supported — see the \
                     package README / follow-up bead.",
                    p
                ));
            }
        }
        let s = render_term(&row[0])?;
        let p = render_term(&row[1])?;
        let o = render_term(&row[2])?;
        out.push_str(&s);
        out.push(' ');
        out.push_str(&p);
        out.push(' ');
        out.push_str(&o);
        out.push_str(" . ");
    }
    Ok(out)
}

/// Render one N3 term as a SPARQL term. `Blank` maps to a fresh-but-consistent variable so a
/// blank shared between premise and conclusion carries its binding through the CONSTRUCT (a
/// SPARQL template blank would NOT — it mints a fresh node per solution). `Formula`/`List`/
/// `Triple` are rejected: no verified faithful BGP rendering exists on this path yet.
fn render_term(t: &Term) -> Result<String, String> {
    match t {
        Term::Iri(iri) => Ok(format!("<{}>", escape_iri(iri))),
        Term::Var(v) => Ok(format!("?{}", sanitize_var(v))),
        // A blank in a query rule behaves like an existential/variable; carry it as a variable
        // so premise->conclusion binding is preserved.
        Term::Blank(l) => Ok(format!("?__b_{}", sanitize_var(l))),
        Term::Lit(lex, dt, lang) => {
            if let Some(lang) = lang {
                Ok(format!("\"{}\"@{}", escape_literal(lex), lang))
            } else if dt.is_empty() || dt == "http://www.w3.org/2001/XMLSchema#string" {
                Ok(format!("\"{}\"", escape_literal(lex)))
            } else {
                Ok(format!("\"{}\"^^<{}>", escape_literal(lex), escape_iri(dt)))
            }
        }
        Term::Formula(_) => Err(
            "compat query filter: a quoted `{ … }` formula term in the query is not supported by \
             the v1 SPARQL-CONSTRUCT compat path (see the package README / follow-up bead)."
                .to_string(),
        ),
        Term::List(_) => Err(
            "compat query filter: a first-class `( … )` list term in the query is not supported \
             by the v1 SPARQL-CONSTRUCT compat path (see the package README / follow-up bead)."
                .to_string(),
        ),
        // FAIL-CLOSED like Formula/List: a faithful rendering would require the engine's
        // SPARQL quoted-triple-pattern surface to be verified end-to-end through this
        // CONSTRUCT compat path first — reject rather than risk a silent mistranslation.
        // [FABLE-5]
        Term::Triple(_) => Err(
            "compat query filter: a quoted `<< s p o >>` triple term in the query is not \
             supported by the v1 SPARQL-CONSTRUCT compat path (see the package README / \
             follow-up bead)."
                .to_string(),
        ),
    }
}

/// Escape a char sequence forbidden in a SPARQL `IRIREF` (`<>"{}|^`\` and control chars) as
/// `\uXXXX`, so a rendered IRI is always a well-formed IRIREF.
fn escape_iri(iri: &str) -> String {
    let mut out = String::with_capacity(iri.len());
    for c in iri.chars() {
        match c {
            '<' | '>' | '"' | '{' | '}' | '|' | '^' | '`' | '\\' => {
                out.push_str(&format!("\\u{:04X}", c as u32))
            }
            c if (c as u32) <= 0x20 => out.push_str(&format!("\\u{:04X}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// Escape a lexical value for a SPARQL `STRING_LITERAL_QUOTE` (the two mandatory chars plus the
/// C0 controls a quoted literal forbids unescaped).
fn escape_literal(lex: &str) -> String {
    let mut out = String::with_capacity(lex.len());
    for c in lex.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}

/// A SPARQL variable name (`VARNAME`) admits `[A-Za-z0-9_]` plus a few unicode ranges; keep the
/// ASCII-safe subset and map anything else to `_` so an N3 var/blank label is always a legal
/// SPARQL variable. Deterministic (per-char), so distinct labels stay distinct in practice.
fn sanitize_var(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for c in name.chars() {
        if c.is_ascii_alphanumeric() || c == '_' {
            out.push(c);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        out.push('_');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The socrates query `{:Socrates a ?WHAT} => {:Socrates a ?WHAT}` translates to a
    /// CONSTRUCT that projects every `:Socrates a ?WHAT` from the closure.
    #[test]
    fn socrates_query_translates_to_construct() {
        let q = r#"@prefix : <http://example.org/socrates#>.
            {:Socrates a ?WHAT} => {:Socrates a ?WHAT}."#;
        let cs = n3_query_to_constructs(q).expect("socrates query translates");
        assert_eq!(cs.len(), 1, "one forward rule -> one CONSTRUCT: {:?}", cs);
        let c = &cs[0];
        assert!(c.starts_with("CONSTRUCT {"), "shape: {}", c);
        assert!(c.contains("?WHAT"), "variable preserved: {}", c);
        assert!(
            c.contains("<http://example.org/socrates#Socrates>"),
            "subject IRI resolved + rendered: {}",
            c
        );
        assert!(
            c.contains("<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>"),
            "`a` expands to rdf:type: {}",
            c
        );
    }

    /// A document with no forward rule is rejected (fail-closed).
    #[test]
    fn no_rule_is_rejected() {
        let q = "@prefix : <http://ex/>. :a :b :c .";
        let err = n3_query_to_constructs(q).unwrap_err();
        assert!(err.contains("no `{ … } => { … }` forward rule"), "{}", err);
    }

    /// A builtin predicate in the premise is rejected (fail-closed) rather than mistranslated.
    #[test]
    fn builtin_predicate_is_rejected() {
        let q = r#"@prefix math: <http://www.w3.org/2000/10/swap/math#>.
            @prefix : <http://ex/>.
            { ?x :age ?a. ?a math:greaterThan 18 } => { ?x a :Adult }."#;
        let err = n3_query_to_constructs(q).unwrap_err();
        assert!(err.contains("N3 builtin"), "rejects builtin: {}", err);
        assert!(err.contains("math#greaterThan"), "names the builtin: {}", err);
    }

    /// Literal rendering: datatype + language + plain string forms are well-formed.
    #[test]
    fn literal_rendering() {
        assert_eq!(
            render_term(&Term::Lit("hi".into(), String::new(), None)).unwrap(),
            "\"hi\""
        );
        assert_eq!(
            render_term(&Term::Lit(
                "hi".into(),
                "http://www.w3.org/2001/XMLSchema#string".into(),
                None
            ))
            .unwrap(),
            "\"hi\"",
            "xsd:string is rendered plain"
        );
        assert_eq!(
            render_term(&Term::Lit(
                "42".into(),
                "http://www.w3.org/2001/XMLSchema#integer".into(),
                None
            ))
            .unwrap(),
            "\"42\"^^<http://www.w3.org/2001/XMLSchema#integer>"
        );
        assert_eq!(
            render_term(&Term::Lit("bonjour".into(), String::new(), Some("fr".into()))).unwrap(),
            "\"bonjour\"@fr"
        );
        // Escaping: a quote in the lexical value is escaped.
        assert_eq!(
            render_term(&Term::Lit("a\"b".into(), String::new(), None)).unwrap(),
            "\"a\\\"b\""
        );
    }

    /// A blank in the query maps to a consistent variable (shared premise/conclusion binding),
    /// and a formula/list/quoted-triple term is rejected fail-closed.
    #[test]
    fn blank_maps_to_var_and_formula_list_rejected() {
        assert_eq!(
            render_term(&Term::Blank("x".into())).unwrap(),
            "?__b_x",
            "blank -> variable so its binding threads through the CONSTRUCT"
        );
        assert!(render_term(&Term::Formula(vec![])).is_err());
        assert!(render_term(&Term::List(vec![])).is_err());
        // RDF 1.2 quoted-triple term (GH #2012): fail-closed on this path. [FABLE-5]
        assert!(render_term(&Term::Triple(Box::new([
            Term::Iri("http://ex/s".into()),
            Term::Iri("http://ex/p".into()),
            Term::Iri("http://ex/o".into()),
        ])))
        .is_err());
    }

    /// IRI escaping neutralises IRIREF-forbidden characters.
    #[test]
    fn iri_escaping() {
        assert_eq!(escape_iri("http://ex/a"), "http://ex/a");
        assert_eq!(escape_iri("http://ex/a>b"), "http://ex/a\\u003Eb");
        assert_eq!(escape_iri("http://ex/a b"), "http://ex/a\\u0020b");
    }

    /// Variable sanitisation keeps the ASCII-safe subset and never yields an empty name.
    #[test]
    fn var_sanitisation() {
        assert_eq!(sanitize_var("WHAT"), "WHAT");
        assert_eq!(sanitize_var("a-b.c"), "a_b_c");
        assert_eq!(sanitize_var(""), "_");
    }
}
