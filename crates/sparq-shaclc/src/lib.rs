//! [FABLE-5] (epic sq-tonhr, child sq-tonhr.6) SHACL Compact Syntax
//! (1.2 + extended) parser for sparq — **rdf-shuttle-generated**.
//!
//! Two checked-in parsers generated from ONE Shuttle grammar
//! (`grammar/shaclc12ext.shuttle`, vendored; provenance in [`provenance`]):
//!
//! * **strict** ([`Profile::Strict`], `src/raw/shaclc12.rs`): the W3C CG
//!   SHACL Compact Syntax surface plus the RDF 1.2 layer (triple terms in
//!   value positions, directional language tags, `TripleTerm` nodeKind,
//!   `reifierShape` / `reificationRequired`). The four shaclc-js extensions
//!   are absent from its parse tables, so strict mode provably rejects them
//!   — fixing shaclc-js's strict-mode enforcement leak by construction.
//! * **extended** ([`Profile::Extended`], `src/raw/shaclc12ext.rs`): strict
//!   plus the shaclc-js extensions (turtle-style node-shape annotation
//!   lists, `% … %` property-shape escapes, trailing arbitrary Turtle
//!   statements, the `a` keyword).
//!
//! The crate-root API converts the generated term model into
//! [`oxrdf::Triple`] shapes triples — the same model `sparq-shacl` consumes
//! — so an SCS document feeds SHACL validation exactly like a Turtle shapes
//! graph would. The raw generated modules (streaming callback + chunked
//! push parsing on the zero-dependency generated term model) stay public in
//! [`raw`].
//!
//! COEXISTENCE (sq-tonhr.6): this crate does not replace `sparq-shacl`'s
//! hand-rolled `scs` feature; the two are differential-tested against each
//! other on every fixture both accept (`tests/differential_scs.rs`).
//!
//! Parse-only today: the print direction (serializer with the
//! residual-based "not compact-expressible" verdict) lands once rdf-shuttle
//! gen-rs derives the residual-consumption printer its gen-js backend
//! already has.

#![forbid(unsafe_code)]

pub mod provenance;
pub mod raw;

use oxrdf::Triple;

/// The default base IRI when an SCS document declares no `BASE` directive,
/// matching the shaclc-js / W3C-corpus convention (and `sparq-shacl`'s
/// `scs::DEFAULT_BASE`).
pub const DEFAULT_BASE: &str = "urn:x-base:default";

/// Which generated parser to use. See the crate docs for the exact split.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Profile {
    /// SHACL Compact Syntax 1.2, W3C CG surface + RDF 1.2 layer only.
    Strict,
    /// [`Profile::Strict`] plus the four shaclc-js extensions.
    Extended,
}

/// A SHACL Compact Syntax parse error (1-based source position, plus the
/// generated parser's stable machine-readable code, e.g.
/// `"UNDECLARED_PREFIX"` or `"UNEXPECTED_TOKEN"`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaclcError {
    /// 1-based line of the offending token.
    pub line: usize,
    /// 1-based column (byte-counted) of the offending token.
    pub column: usize,
    /// Stable machine-readable error code from the generated parser.
    pub code: Option<&'static str>,
    /// Human-readable message.
    pub message: String,
}

impl std::fmt::Display for ShaclcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SHACL-CS parse error (line {}:{}): {}", self.line, self.column, self.message)
    }
}

impl std::error::Error for ShaclcError {}

/// What a parse leaves behind besides the triples.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    /// Prefixes in declaration order — the five predeclared ones
    /// (`rdf` / `rdfs` / `sh` / `xsd` / `owl`) first, then `PREFIX`
    /// declarations.
    pub prefixes: Vec<(String, String)>,
    /// The final base IRI (the `BASE` directive if declared, else the
    /// `base` argument).
    pub base: String,
}

/// Bridges one generated module's term model to `oxrdf` (each artifact has
/// its own nominal types, so the bridge is stamped per module).
macro_rules! oxrdf_bridge {
    ($m:path, $modname:ident) => {
        mod $modname {
            use super::{Outcome, ShaclcError};
            use oxrdf::{BlankNode, Literal, NamedNode, NamedOrBlankNode, Term, Triple};
            use $m as m;

            fn internal(what: &str) -> ShaclcError {
                ShaclcError {
                    line: 0,
                    column: 0,
                    code: Some("INTERNAL"),
                    message: format!("generated parser emitted {what} in an impossible position"),
                }
            }

            fn term(t: &m::Term) -> Result<Term, ShaclcError> {
                Ok(match t {
                    // IRIs are grammar-validated (IRIREF charset) and RFC-3986
                    // resolved by the generated parser; blank-node labels are
                    // parser-allocated (`b0`, `b1`, ...).
                    m::Term::NamedNode(v) => Term::NamedNode(NamedNode::new_unchecked(v.as_ref())),
                    m::Term::BlankNode(v) => Term::BlankNode(BlankNode::new_unchecked(v.as_ref())),
                    m::Term::Literal(l) => {
                        let lit = if l.language.is_empty() {
                            let m::Term::NamedNode(dt) = &l.datatype else {
                                return Err(internal("a non-IRI literal datatype"));
                            };
                            Literal::new_typed_literal(
                                l.value.as_ref(),
                                NamedNode::new_unchecked(dt.as_ref()),
                            )
                        } else if let Some(dir) = &l.direction {
                            let dir = match dir.as_ref() {
                                "ltr" => oxrdf::BaseDirection::Ltr,
                                "rtl" => oxrdf::BaseDirection::Rtl,
                                other => return Err(internal(&format!("base direction '{other}'"))),
                            };
                            Literal::new_directional_language_tagged_literal_unchecked(
                                l.value.as_ref(),
                                l.language.as_ref(),
                                dir,
                            )
                        } else {
                            Literal::new_language_tagged_literal_unchecked(
                                l.value.as_ref(),
                                l.language.as_ref(),
                            )
                        };
                        Term::Literal(lit)
                    }
                    m::Term::Triple(q) => Term::Triple(Box::new(triple(q)?)),
                })
            }

            fn triple(t: &m::Triple) -> Result<Triple, ShaclcError> {
                let subject = match term(&t.subject)? {
                    Term::NamedNode(n) => NamedOrBlankNode::NamedNode(n),
                    Term::BlankNode(b) => NamedOrBlankNode::BlankNode(b),
                    _ => return Err(internal("a non-resource subject")),
                };
                let Term::NamedNode(predicate) = term(&t.predicate)? else {
                    return Err(internal("a non-IRI predicate"));
                };
                Ok(Triple { subject, predicate, object: term(&t.object)? })
            }

            pub(super) fn parse(text: &str, base: &str) -> Result<(Vec<Triple>, Outcome), ShaclcError> {
                let mut raw_triples: Vec<m::Triple> = Vec::new();
                let outcome = m::parse(text, Some(base), |t| raw_triples.push(t)).map_err(|e| {
                    ShaclcError { line: e.line, column: e.column, code: e.code, message: e.message }
                })?;
                let mut triples = Vec::with_capacity(raw_triples.len());
                for t in &raw_triples {
                    triples.push(triple(t)?);
                }
                Ok((triples, Outcome { prefixes: outcome.prefixes, base: outcome.base }))
            }
        }
    };
}

oxrdf_bridge!(crate::raw::shaclc12, bridge_strict);
oxrdf_bridge!(crate::raw::shaclc12ext, bridge_ext);

/// Parses a SHACL Compact Syntax document into `oxrdf` shapes triples with
/// the chosen [`Profile`]. Relative IRIs resolve against `base` (pass
/// [`DEFAULT_BASE`] to mirror the no-`BASE` convention); a document-level
/// `BASE` directive overrides it for subsequent IRIs.
pub fn parse(text: &str, base: &str, profile: Profile) -> Result<(Vec<Triple>, Outcome), ShaclcError> {
    match profile {
        Profile::Strict => bridge_strict::parse(text, base),
        Profile::Extended => bridge_ext::parse(text, base),
    }
}

/// [`parse`] with [`Profile::Strict`].
pub fn parse_strict(text: &str, base: &str) -> Result<(Vec<Triple>, Outcome), ShaclcError> {
    bridge_strict::parse(text, base)
}

/// [`parse`] with [`Profile::Extended`].
pub fn parse_extended(text: &str, base: &str) -> Result<(Vec<Triple>, Outcome), ShaclcError> {
    bridge_ext::parse(text, base)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxrdf::Term;

    const DOC: &str =
        "BASE <http://example.org/b>\nPREFIX ex: <http://example.org/test#>\nshape ex:S -> ex:C {\n}\n";

    #[test]
    fn parse_strict_emits_shape_triples() {
        let (triples, outcome) = parse_strict(DOC, DEFAULT_BASE).expect("parse");
        assert!(!triples.is_empty());
        assert_eq!(outcome.base, "http://example.org/b");
        let types: Vec<String> = triples
            .iter()
            .filter(|t| t.predicate.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type")
            .map(|t| t.object.to_string())
            .collect();
        assert!(
            types.iter().any(|o| o.contains("http://www.w3.org/ns/shacl#NodeShape")),
            "expected a sh:NodeShape typing, got {types:?}"
        );
    }

    #[test]
    fn parse_dispatches_on_profile() {
        let s = parse(DOC, DEFAULT_BASE, Profile::Strict).expect("strict");
        let e = parse(DOC, DEFAULT_BASE, Profile::Extended).expect("extended");
        assert_eq!(s.0, e.0);
        assert_eq!(s.1, e.1);
    }

    #[test]
    fn predeclared_prefixes_come_first_in_declaration_order() {
        let (_, outcome) = parse_strict("shape <http://e/S> {\n}\n", DEFAULT_BASE).expect("parse");
        let names: Vec<&str> = outcome.prefixes.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(names, ["rdf", "rdfs", "sh", "xsd", "owl"]);
        assert_eq!(outcome.base, DEFAULT_BASE);
    }

    #[test]
    fn declared_prefix_appends_after_predeclared() {
        let doc = "PREFIX ex: <http://example.org/x#>\nshape ex:S {\n}\n";
        let (_, outcome) = parse_strict(doc, DEFAULT_BASE).expect("parse");
        assert_eq!(
            outcome.prefixes.last().map(|(k, v)| (k.as_str(), v.as_str())),
            Some(("ex", "http://example.org/x#"))
        );
    }

    #[test]
    fn strict_rejects_the_annotation_list_extension_with_position() {
        // The turtle-style node-shape annotation list is a shaclc-js
        // EXTENSION: extended accepts, strict rejects — the enforcement-leak
        // fix, observable at the API level.
        let doc = "PREFIX ex: <http://example.org/test#>\nshape ex:S ;\n  ex:myProperty 1\n{\n}\n";
        let err = parse_strict(doc, DEFAULT_BASE).expect_err("strict must reject");
        assert!(err.line >= 2, "error should point at the extension, got {err}");
        assert!(err.code.is_some(), "stable code expected, got {err:?}");
        assert!(parse_extended(doc, DEFAULT_BASE).is_ok(), "extended must accept");
    }

    #[test]
    fn undeclared_prefix_is_a_stable_code() {
        let err = parse_strict("shape nope:S {\n}\n", DEFAULT_BASE).expect_err("must reject");
        assert_eq!(err.code, Some("UNDECLARED_PREFIX"));
        let shown = err.to_string();
        assert!(shown.contains("line 1"), "display carries the position: {shown}");
    }

    #[test]
    fn dir_lang_literal_converts_to_directional_oxrdf_literal() {
        let doc =
            "PREFIX ex: <http://example.org/test#>\nshape ex:S {\n  ex:greeting hasValue=\"x\"@en--ltr .\n}\n";
        let (triples, _) = parse_strict(doc, DEFAULT_BASE).expect("parse");
        let lit = triples
            .iter()
            .find_map(|t| match &t.object {
                Term::Literal(l) if l.value() == "x" => Some(l.clone()),
                _ => None,
            })
            .expect("dir-lang literal present");
        assert_eq!(lit.language(), Some("en"));
        assert_eq!(lit.direction(), Some(oxrdf::BaseDirection::Ltr));
    }

    #[test]
    fn triple_term_value_converts_to_oxrdf_triple_term() {
        let doc =
            "PREFIX ex: <http://example.org/test#>\nshape ex:S {\n  ex:statement hasValue=<<( ex:s ex:p ex:o )>> .\n}\n";
        let (triples, _) = parse_strict(doc, DEFAULT_BASE).expect("parse");
        assert!(
            triples.iter().any(|t| matches!(&t.object, Term::Triple(_))),
            "expected an embedded triple term object"
        );
    }

    #[test]
    fn profile_and_error_types_behave_like_values() {
        assert_eq!(Profile::Strict, Profile::Strict);
        assert_ne!(Profile::Strict, Profile::Extended);
        let e = ShaclcError { line: 2, column: 5, code: None, message: "m".into() };
        assert_eq!(e.clone(), e);
        assert!(format!("{e:?}").contains("ShaclcError"));
    }
}
