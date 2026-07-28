//! The neutral term model and the two comparison regimes (strict RDF equality; value-canonical
//! keying). Engine-independent. [OPUS-4.8] sq-qcnn.4

use crate::numeric::{
    canonical_numeric_string, order_numeric_string, parse_numeric, promoted_tie, NumericValue,
};
use crate::temporal::{parse_datetime, parse_duration};

/// The XSD namespace prefix.
pub const XSD: &str = "http://www.w3.org/2001/XMLSchema#";
/// `xsd:string` — the datatype a simple (plain) literal folds to under RDF 1.1.
pub const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";
/// `xsd:boolean`.
pub const XSD_BOOLEAN: &str = "http://www.w3.org/2001/XMLSchema#boolean";
/// `rdf:langString` — the datatype of a language-tagged literal.
pub const RDF_LANGSTRING: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString";

/// A neutral RDF term, parsed from an oracle's wire form and independent of any engine's term type.
///
/// A projected-but-unbound variable is modelled by the *absence* of the variable from a
/// [`crate::json::Solution`] (distinct from bound-to-empty-string), not by a variant here.
#[derive(Debug, Clone)]
pub enum Term {
    /// An IRI.
    Iri(String),
    /// A blank node, carrying its **engine-local** label. Cross-oracle blank-node isomorphism is a
    /// separate DAG node (`sq-qcnn.7`); here a blank node compares by label, which is only meaningful
    /// *within* one engine's result.
    Blank(String),
    /// A literal: lexical form, datatype IRI, and an optional language tag. A simple literal is
    /// represented with `datatype = xsd:string` (or an empty datatype, treated as `xsd:string`).
    Literal {
        lexical: String,
        datatype: String,
        lang: Option<String>,
    },
    /// An RDF 1.2 triple term (subject, predicate, object).
    Triple(Box<[Term; 3]>),
}

/// The effective datatype of a literal: `rdf:langString` if language-tagged, else `xsd:string` for an
/// empty datatype (the simple-literal ≡ `xsd:string` fold), else the datatype as given.
pub(crate) fn effective_datatype<'a>(datatype: &'a str, lang: &Option<String>) -> &'a str {
    if lang.is_some() {
        RDF_LANGSTRING
    } else if datatype.is_empty() {
        XSD_STRING
    } else {
        datatype
    }
}

/// **Strict RDF term equality** (the data-sourced regime): two terms are equal iff they are the same
/// RDF term, with exactly the two RDF-1.1 equalities folded in — simple-literal ≡ `xsd:string`, and
/// case-insensitive language tags. There is **no** numeric/temporal canonicalisation, so
/// `"01"^^xsd:integer` stays distinct from `"1"^^xsd:integer` (as RDF requires for a data-sourced
/// binding). Blank nodes compare by their engine-local label (see [`Term::Blank`]).
pub fn term_equal_rdf(a: &Term, b: &Term) -> bool {
    match (a, b) {
        (Term::Iri(x), Term::Iri(y)) => x == y,
        (Term::Blank(x), Term::Blank(y)) => x == y,
        (Term::Triple(x), Term::Triple(y)) => {
            term_equal_rdf(&x[0], &y[0])
                && term_equal_rdf(&x[1], &y[1])
                && term_equal_rdf(&x[2], &y[2])
        }
        (
            Term::Literal {
                lexical: lx,
                datatype: dx,
                lang: gx,
            },
            Term::Literal {
                lexical: ly,
                datatype: dy,
                lang: gy,
            },
        ) => {
            let edx = effective_datatype(dx, gx);
            let edy = effective_datatype(dy, gy);
            if edx != edy {
                return false;
            }
            if edx == RDF_LANGSTRING {
                lx == ly
                    && gx
                        .as_deref()
                        .unwrap_or_default()
                        .eq_ignore_ascii_case(gy.as_deref().unwrap_or_default())
            } else {
                lx == ly
            }
        }
        _ => false,
    }
}

/// **Value-canonical multiset key** (the computed/decision regime): a stable string keyed by exact
/// value within each datatype, so cross-engine canonical-*lexical* variance collapses (`05` vs `5`,
/// `1.50` vs `1.5`, `6` vs `6.0E0`, `true` vs `1`, same-instant dateTimes) while a genuinely different
/// value, term kind, or datatype does not. Language tags are lowercased; simple literals fold to
/// `xsd:string`.
///
/// The `\u{1f}` unit separator cannot appear in a datatype IRI, and a literal's lexical is the *last*
/// field of its key, so a leaf (IRI/blank/literal) key is unambiguous. A **triple** key, however,
/// nests three leaf keys — and a literal lexical *can* contain any character, including `\u{1f}` — so
/// plain delimiter-joining would be ambiguous: two distinct triple terms could concatenate to one key
/// (a false "equal"). Each nested component is therefore **length-prefixed** (`field`) so decoding
/// boundaries never depend on the separator and the composite key stays injective.
pub fn canonical_key(term: &Term) -> String {
    match term {
        Term::Iri(x) => format!("I\u{1f}{}", x),
        Term::Blank(x) => format!("B\u{1f}{}", x),
        Term::Triple(t) => format!(
            "T\u{1f}{}{}{}",
            field(&canonical_key(&t[0])),
            field(&canonical_key(&t[1])),
            field(&canonical_key(&t[2]))
        ),
        Term::Literal {
            lexical,
            datatype,
            lang,
        } => literal_key(lexical, datatype, lang),
    }
}

/// **`ORDER BY` sort-key-equivalence key**: like [`canonical_key`], but keyed by SPARQL *ordering*
/// equality rather than by RDF term identity. Two terms sharing this key are **tied** under
/// `ORDER BY` (neither is `<` the other), so their relative order is unspecified and either engine
/// may emit them in either order.
///
/// The implication runs one way only: this partition *refines* the tie relation, so a tied pair may
/// still key apart (see the unmodelled cases below). Refining cannot *hide* an ordering bug — it
/// only splits a maximal run further — but it does **manufacture a false failure**, by rejecting a
/// permutation the spec permits. A false failure is a defect in a differential oracle, not a safe
/// direction, so the split cases are not left to the key: `order_key_splits_tie` names the pairs it
/// splits, and [`crate::multiset::order_by_compare`] consults that before reporting a divergence on
/// such a column, returning its fail-closed verdict instead.
///
/// The one place it differs from [`canonical_key`] is **numeric promotion**: SPARQL compares numeric
/// operands after promotion, so `1`^^`xsd:integer`, `1.0`^^`xsd:decimal` and `1.0E0`^^`xsd:double`
/// are one tie class. [`canonical_key`] keeps the datatype in the key — right for a result *bag*,
/// where those are three different RDF terms — and would split that class into separate maximal
/// runs, rejecting two conforming sequences that differ only by a permitted permutation. Every
/// numeric literal therefore folds here to one datatype-free decimal key
/// (`crate::numeric::order_numeric_string`, whose lossy-promotion residual is documented there).
///
/// Two other regimes are deliberately left to the distinct [`canonical_key`]s, for two *different*
/// reasons. §15.1 **specifies** the order between term kinds (unbound < blank node < IRI < literal),
/// so those pairs are strictly ordered and never tied — splitting them is simply right. Pairs the
/// operator mapping does not order — two different language tags, an unmodelled datatype, a triple
/// term — are ordered by an *implementation-defined* total order this engine-independent crate
/// cannot know; distinct keys are again right (a total order ties nothing), but two engines may pick
/// *different* total orders, which this comparator would report as a cross-run reordering. That
/// residual is a caller precondition — sort on a column whose values this crate can order — not
/// something a key can repair. Within one modelled datatype nothing more is needed:
/// [`canonical_key`] already collapses value-equal lexical variance (`05`/`5`, `true`/`1`,
/// same-instant dateTimes), which is exactly the same-datatype tie class.
pub fn order_equiv_key(term: &Term) -> String {
    numeric_value(term)
        .map(|n| numeric_order_key(&n))
        .unwrap_or_else(|| canonical_key(term))
}

/// Whether `a` and `b` are **tied** under SPARQL `ORDER BY` yet key **apart** under
/// [`order_equiv_key`] — the pairs for which run partitioning by that key is not a sound oracle,
/// because it would split a run the spec allows either engine to permute.
///
/// Only numeric operands can land here, and only via `f64` promotion: a `double`/`float` compared
/// against an exact value ties whenever the two differ below `f64` precision, and `NaN` ties with
/// every numeric (see [`crate::numeric::promoted_tie`]). The tie relation is then not transitive, so
/// no key can express it — [`crate::multiset::order_by_compare`] uses this predicate to fail closed
/// instead of returning a verdict it cannot justify.
pub(crate) fn order_key_splits_tie(a: &Term, b: &Term) -> bool {
    let (Some(x), Some(y)) = (numeric_value(a), numeric_value(b)) else {
        return false;
    };
    numeric_order_key(&x) != numeric_order_key(&y) && promoted_tie(&x, &y)
}

/// The numeric value of a literal term; `None` for anything else (a language-tagged literal is
/// never numeric, and no other term kind carries a numeric value space).
fn numeric_value(term: &Term) -> Option<NumericValue> {
    let Term::Literal {
        lexical,
        datatype,
        lang,
    } = term
    else {
        return None;
    };
    if lang.is_some() {
        return None;
    }
    parse_numeric(lexical, effective_datatype(datatype, lang))
}

/// The datatype-free ordering key of a numeric value.
fn numeric_order_key(n: &NumericValue) -> String {
    // `N` tags no `canonical_key` variant, so a promoted numeric key can never collide with the key
    // of a non-numeric term.
    format!("N\u{1f}{}", order_numeric_string(n))
}

/// Length-prefix a nested component key (`"<byte-len>\u{1f}<key>"`) so composite (triple) keys stay
/// injective even when a component itself contains the `\u{1f}` separator — a literal lexical may
/// contain any character, and a triple key nests other keys. Decoding reads the digit run up to the
/// first separator (a byte length), then exactly that many bytes, so the boundary never depends on the
/// separator appearing inside the component.
fn field(key: &str) -> String {
    format!("{}\u{1f}{}", key.len(), key)
}

/// Value-canonical key for a literal.
fn literal_key(lexical: &str, datatype: &str, lang: &Option<String>) -> String {
    if let Some(tag) = lang {
        // Language-tagged: keyed by lowercased tag + lexical (distinct from any typed literal).
        return format!("L@\u{1f}{}\u{1f}{}", tag.to_ascii_lowercase(), lexical);
    }
    let dt = effective_datatype(datatype, lang);
    format!("L\u{1f}{}\u{1f}{}", dt, canonical_lexical(lexical, dt))
}

/// The value-canonical **lexical form** of a non-language-tagged literal under its effective
/// datatype: the exact numeric expansion for a numeric datatype, the UTC-normalised instant /
/// canonical duration for a temporal one, `true`/`false` for `xsd:boolean`, and the lexical
/// unchanged for strings and any datatype this crate does not model (no canonicalisation is
/// invented for a datatype whose value space is unknown here).
///
/// Shared by [`canonical_key`] and by the blank-node isomorphism encoder
/// ([`crate::iso`]), so a bnode-bearing result and a ground one collapse cross-engine
/// canonical-lexical variance by exactly the same rule.
pub(crate) fn canonical_lexical(lexical: &str, dt: &str) -> String {
    if let Some(n) = parse_numeric(lexical, dt) {
        return canonical_numeric_string(&n);
    }
    if let Some(v) = parse_datetime(lexical, dt) {
        return v.canonical_key();
    }
    if let Some(d) = parse_duration(lexical, dt) {
        return d.canonical_key();
    }
    if dt == XSD_BOOLEAN {
        match lexical.trim() {
            "true" | "1" => return "true".to_string(),
            "false" | "0" => return "false".to_string(),
            _ => {}
        }
    }
    // Strings, IRIs-as-literal, and any unknown datatype: exact lexical (no canonicalisation).
    lexical.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lit(lexical: &str, datatype: &str) -> Term {
        Term::Literal {
            lexical: lexical.to_string(),
            datatype: datatype.to_string(),
            lang: None,
        }
    }
    fn langlit(lexical: &str, lang: &str) -> Term {
        Term::Literal {
            lexical: lexical.to_string(),
            datatype: RDF_LANGSTRING.to_string(),
            lang: Some(lang.to_string()),
        }
    }
    const XSD_INT: &str = "http://www.w3.org/2001/XMLSchema#integer";
    const XSD_DT: &str = "http://www.w3.org/2001/XMLSchema#dateTime";

    #[test]
    fn term_equal_rdf_simple_string_and_lang() {
        // simple literal ≡ xsd:string.
        assert!(term_equal_rdf(&lit("abc", ""), &lit("abc", XSD_STRING)));
        // language tags compare case-insensitively.
        assert!(term_equal_rdf(&langlit("chat", "en"), &langlit("chat", "EN")));
        // lang literal is NOT equal to a plain string of the same lexical.
        assert!(!term_equal_rdf(&langlit("chat", "en"), &lit("chat", XSD_STRING)));
        // strict: no numeric canonicalisation for data-sourced terms.
        assert!(!term_equal_rdf(&lit("01", XSD_INT), &lit("1", XSD_INT)));
        // IRIs, blanks, triples, and cross-kind.
        assert!(term_equal_rdf(
            &Term::Iri("http://a".into()),
            &Term::Iri("http://a".into())
        ));
        assert!(term_equal_rdf(&Term::Blank("b0".into()), &Term::Blank("b0".into())));
        let t1 = Term::Triple(Box::new([
            Term::Iri("http://s".into()),
            Term::Iri("http://p".into()),
            lit("1", XSD_INT),
        ]));
        let t2 = Term::Triple(Box::new([
            Term::Iri("http://s".into()),
            Term::Iri("http://p".into()),
            lit("1", XSD_INT),
        ]));
        assert!(term_equal_rdf(&t1, &t2));
        assert!(!term_equal_rdf(&Term::Iri("http://a".into()), &Term::Blank("a".into())));
    }

    #[test]
    fn canonical_key_collapses_value_variants() {
        // numeric lexical variance collapses under the value regime (unlike strict equality).
        assert_eq!(canonical_key(&lit("01", XSD_INT)), canonical_key(&lit("1", XSD_INT)));
        // simple literal ≡ xsd:string, keyed identically.
        assert_eq!(canonical_key(&lit("abc", "")), canonical_key(&lit("abc", XSD_STRING)));
        // language tag lowercased in the key.
        assert_eq!(canonical_key(&langlit("chat", "EN")), canonical_key(&langlit("chat", "en")));
        // boolean 1 ≡ true.
        assert_eq!(canonical_key(&lit("1", XSD_BOOLEAN)), canonical_key(&lit("true", XSD_BOOLEAN)));
        // same instant, different lexical -> same key.
        assert_eq!(
            canonical_key(&lit("2020-01-01T13:00:00Z", XSD_DT)),
            canonical_key(&lit("2020-01-01T14:00:00+01:00", XSD_DT))
        );
        // different value / different datatype / different kind -> different key.
        assert_ne!(canonical_key(&lit("1", XSD_INT)), canonical_key(&lit("2", XSD_INT)));
        assert_ne!(canonical_key(&lit("1", XSD_INT)), canonical_key(&lit("1", XSD_STRING)));
        assert_ne!(canonical_key(&Term::Iri("x".into())), canonical_key(&Term::Blank("x".into())));
        // unknown datatype keeps exact lexical.
        assert_ne!(
            canonical_key(&lit("01", "http://example.org/weird")),
            canonical_key(&lit("1", "http://example.org/weird"))
        );
    }

    #[test]
    fn canonical_lexical_collapses_value_variants_only_where_the_value_space_is_known() {
        // numeric / boolean / temporal: value-canonical.
        assert_eq!(canonical_lexical("01", XSD_INT), canonical_lexical("1", XSD_INT));
        assert_eq!(canonical_lexical("1", XSD_BOOLEAN), "true");
        assert_eq!(canonical_lexical("0", XSD_BOOLEAN), "false");
        assert_eq!(
            canonical_lexical("2020-01-01T13:00:00Z", XSD_DT),
            canonical_lexical("2020-01-01T14:00:00+01:00", XSD_DT)
        );
        // strings and any datatype this crate does not model: lexical UNCHANGED (no invented
        // canonicalisation for an unknown value space).
        assert_eq!(canonical_lexical("abc", XSD_STRING), "abc");
        assert_eq!(canonical_lexical("01", "http://example.org/weird"), "01");
        assert_eq!(canonical_lexical("maybe", XSD_BOOLEAN), "maybe");
        // a genuinely different value still separates.
        assert_ne!(canonical_lexical("1", XSD_INT), canonical_lexical("2", XSD_INT));
    }

    #[test]
    fn order_equiv_key_folds_numeric_promotion_but_canonical_key_does_not() {
        const XSD_DEC: &str = "http://www.w3.org/2001/XMLSchema#decimal";
        const XSD_DBL: &str = "http://www.w3.org/2001/XMLSchema#double";
        // The defect this key exists for: SPARQL orders numerics after promotion, so these three
        // are ONE tie class whose relative order is unspecified.
        assert_eq!(
            order_equiv_key(&lit("1", XSD_INT)),
            order_equiv_key(&lit("1.0", XSD_DEC))
        );
        assert_eq!(
            order_equiv_key(&lit("1", XSD_INT)),
            order_equiv_key(&lit("1.0E0", XSD_DBL))
        );
        // ... while the BAG key must still keep them apart (they are different RDF terms).
        assert_ne!(
            canonical_key(&lit("1", XSD_INT)),
            canonical_key(&lit("1.0", XSD_DEC))
        );
        // A genuinely different numeric value stays a different tie class, across datatypes too.
        assert_ne!(
            order_equiv_key(&lit("1", XSD_INT)),
            order_equiv_key(&lit("1.5", XSD_DEC))
        );
        assert_ne!(
            order_equiv_key(&lit("1", XSD_INT)),
            order_equiv_key(&lit("2.0", XSD_DEC))
        );
        // Exponent-scaled values fold to the same key regardless of which class they came from.
        assert_eq!(
            order_equiv_key(&lit("100", XSD_INT)),
            order_equiv_key(&lit("1.0E2", XSD_DBL))
        );
        // Non-numerics fall through to `canonical_key` unchanged: a numeric never collides with a
        // string/IRI/blank/lang literal, and `<` on those pairs ERRORS (implementation-defined
        // order), so they must stay distinct keys.
        assert_eq!(order_equiv_key(&lit("1", XSD_STRING)), canonical_key(&lit("1", XSD_STRING)));
        assert_ne!(order_equiv_key(&lit("1", XSD_INT)), order_equiv_key(&lit("1", XSD_STRING)));
        assert_ne!(order_equiv_key(&lit("1", XSD_INT)), order_equiv_key(&Term::Iri("1".into())));
        assert_ne!(order_equiv_key(&lit("1", XSD_INT)), order_equiv_key(&langlit("1", "en")));
        // Specials keep their own tokens (no decimal form; `<` errors on NaN).
        assert_ne!(order_equiv_key(&lit("NaN", XSD_DBL)), order_equiv_key(&lit("INF", XSD_DBL)));
        assert_eq!(
            order_equiv_key(&lit("NaN", XSD_DBL)),
            order_equiv_key(&lit("NaN", XSD_DBL))
        );
        // An unmodelled datatype is not numeric: exact lexical, as `canonical_key` gives.
        assert_ne!(
            order_equiv_key(&lit("1", "http://example.org/weird")),
            order_equiv_key(&lit("1.0", "http://example.org/weird"))
        );
        // An exponent-form double still folds onto the integer of that value — i.e. the decimal
        // re-parse handles exponent notation rather than silently falling back to the raw lexical.
        let ten_to_300 = format!("1{}", "0".repeat(300));
        assert_eq!(
            order_equiv_key(&lit(&ten_to_300, XSD_INT)),
            order_equiv_key(&lit("1E300", XSD_DBL))
        );
        // Promotion to f64 is lossy, so an integer beyond f64 precision keys apart from the double
        // it promotes to even though the two are TIED under `ORDER BY`. Promotion-equality is not
        // transitive, so no key can fix that — the pair must instead be DETECTED, so the comparator
        // can fail closed rather than reject a legal permutation of the tie.
        let i70p1 = lit("1180591620717411303425", XSD_INT); // 2^70 + 1
        let d70 = lit("1180591620717411303424", XSD_DBL); // 2^70, exact as an f64
        assert_ne!(order_equiv_key(&i70p1), order_equiv_key(&d70));
        assert!(order_key_splits_tie(&i70p1, &d70));
        assert!(order_key_splits_tie(&d70, &i70p1), "the predicate is symmetric");
        // `NaN` ties with every numeric (`<` is false both ways) while keying on its own token.
        assert!(order_key_splits_tie(&lit("NaN", XSD_DBL), &lit("1", XSD_INT)));
    }

    /// The detector must be NARROW: firing on a pair that is strictly ordered, or on one the key
    /// already ties, would turn every ordered comparison into a skip.
    #[test]
    fn order_key_splits_tie_fires_only_where_the_key_cannot_model_the_tie() {
        const XSD_DEC: &str = "http://www.w3.org/2001/XMLSchema#decimal";
        const XSD_DBL: &str = "http://www.w3.org/2001/XMLSchema#double";
        // Strictly ordered numerics, within and across classes.
        assert!(!order_key_splits_tie(&lit("1", XSD_INT), &lit("2", XSD_INT)));
        assert!(!order_key_splits_tie(&lit("1", XSD_INT), &lit("2.0", XSD_DBL)));
        assert!(!order_key_splits_tie(&lit("1", XSD_INT), &lit("1.5", XSD_DEC)));
        // Tied AND keyed together — nothing to fail closed on.
        assert!(!order_key_splits_tie(&lit("1", XSD_INT), &lit("1.0E0", XSD_DBL)));
        // Two exact operands never promote, so consecutive huge integers stay strictly ordered.
        assert!(!order_key_splits_tie(
            &lit("1180591620717411303425", XSD_INT),
            &lit("1180591620717411303426", XSD_INT)
        ));
        // Non-numerics are ordered by §15.1 / an implementation-defined total order: not this
        // predicate's business.
        assert!(!order_key_splits_tie(&lit("1", XSD_INT), &lit("1", XSD_STRING)));
        assert!(!order_key_splits_tie(&lit("1", XSD_INT), &Term::Iri("x".into())));
        assert!(!order_key_splits_tie(&langlit("1", "en"), &langlit("1", "fr")));
    }

    #[test]
    fn triple_key_is_injective_across_component_boundaries() {
        // A literal lexical may legitimately contain the `\u{1f}` unit separator. Without
        // length-prefixing the nested keys, two distinct triple terms could concatenate to the same
        // key (a false "equal" that would mask a divergence). These two triples differ only in where
        // the separator-bearing content sits, and must therefore key differently.
        let sep = "\u{1f}";
        let t1 = Term::Triple(Box::new([
            lit(&format!("a{sep}I{sep}b"), XSD_STRING),
            Term::Iri("c".into()),
            Term::Iri("d".into()),
        ]));
        let t2 = Term::Triple(Box::new([
            lit("a", XSD_STRING),
            Term::Iri("b".into()),
            Term::Iri(format!("c{sep}I{sep}d")),
        ]));
        assert_ne!(canonical_key(&t1), canonical_key(&t2));
        // Equal triples still key equal (regression guard on the length-prefix scheme).
        let t3 = Term::Triple(Box::new([
            Term::Iri("s".into()),
            Term::Iri("p".into()),
            lit("1", XSD_INT),
        ]));
        let t4 = Term::Triple(Box::new([
            Term::Iri("s".into()),
            Term::Iri("p".into()),
            lit("01", XSD_INT),
        ]));
        assert_eq!(canonical_key(&t3), canonical_key(&t4));
    }
}
