//! sq-g6b6 — RDF-1.2 N-Quads **canonical-token tracking suite** for the
//! NON-STANDARD `rdf12-triple-terms` profile (follow-up from sq-hslb).
//!
//! The profile single-sources its canonical token rules — the `<<( … )>>`
//! triple-term form, the `"…"@lang--dir` directional-language form, and the
//! literal escaping — from oxrdf 0.3's `Display` (and re-parses them with
//! oxttl 0.2). W3C **rdf12-n-quads** was NOT yet verified final when this
//! suite was written, so the token rules must be RE-VERIFIED against the
//! final Recommendation, and against any canonical-escaping change upstream.
//!
//! This suite makes that re-verification impossible to miss:
//!
//! 1. [`serializer_versions_pinned`] pins the **resolved** oxrdf/oxttl
//!    serializer versions from the workspace `Cargo.lock`. Any bump fails the
//!    test with instructions: re-check the byte-exact token expectations below
//!    against the upstream changelog + the (then-final) W3C rdf12-n-quads
//!    grammar, then update the pin.
//! 2. The remaining tests assert **byte-exact** canonical lines for the token
//!    edge cases, so even a same-version behavioural drift (or a patched
//!    vendored copy) is caught, and a future serializer bump that changes the
//!    canonical bytes goes red here rather than silently changing every
//!    canonical hash downstream.
//!
//! If a failure here is a *deliberate* upstream alignment with the final spec,
//! updating the expectations is a **canonical-output break** for this profile
//! (hashes over canonical documents change) — call that out in the PR.
#![cfg(feature = "rdf12-triple-terms")]

use oxrdf::{Literal, NamedNode, NamedOrBlankNode, Term, Triple};
use std::path::Path;

/// The oxrdf line whose `Display` the profile serializes canonical tokens with.
const PINNED_OXRDF: &str = "0.3.3";
/// The oxttl line the profile re-parses its own canonical lines with.
const PINNED_OXTTL: &str = "0.2.3";

fn iri(s: &str) -> NamedNode {
    NamedNode::new(s).unwrap()
}

fn tt(s: NamedOrBlankNode, p: NamedNode, o: Term) -> Term {
    Term::Triple(Box::new(Triple::new(s, p, o)))
}

/// One ground triple whose object is `object`, canonicalized; returns the
/// single canonical line (ground input ⇒ serialization is pure `Display`).
fn canon_line(object: Term) -> String {
    let t = Triple::new(
        NamedOrBlankNode::NamedNode(iri("http://ex/a")),
        iri("http://ex/says"),
        object,
    );
    let c = sparq_canon::canonicalize_triples_rdf12(&[t]).unwrap();
    assert_eq!(c.lines.len(), 1, "one input triple, one canonical line");
    c.lines[0].clone()
}

/// Every resolved version of `package` in the workspace `Cargo.lock`
/// (the crate also pins an oxrdf 0.2 / oxttl 0.1 pair for the rdf-canon
/// bridge, so a package name can resolve to several versions).
fn locked_versions(package: &str) -> Vec<String> {
    let lock = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.lock");
    let lock = std::fs::read_to_string(&lock)
        .unwrap_or_else(|e| panic!("read workspace Cargo.lock at {:?}: {e}", lock));
    let mut versions = Vec::new();
    let mut last_name: Option<String> = None;
    for line in lock.lines() {
        let line = line.trim();
        if let Some(n) = line.strip_prefix("name = \"") {
            last_name = Some(n.trim_end_matches('"').to_string());
        } else if let Some(v) = line.strip_prefix("version = \"") {
            if last_name.as_deref() == Some(package) {
                versions.push(v.trim_end_matches('"').to_string());
            }
            last_name = None;
        }
    }
    versions.sort();
    versions
}

/// The tracking pin (sq-g6b6): the resolved oxrdf-0.3.x / oxttl-0.2.x
/// serializer versions must match the versions the byte-exact token
/// expectations in this file were verified against.
///
/// **If this fails after a dependency bump:** (1) diff the upstream
/// oxrdf/oxttl changelog for canonical-token / escaping changes and re-check
/// it against the W3C rdf12-n-quads grammar (final REC, once published);
/// (2) confirm every byte-exact test in this file still passes (if one
/// changed, that is a canonical-output break — say so in the PR); (3) update
/// `PINNED_OXRDF` / `PINNED_OXTTL`.
#[test]
fn serializer_versions_pinned() {
    let oxrdf03: Vec<String> = locked_versions("oxrdf")
        .into_iter()
        .filter(|v| v.starts_with("0.3."))
        .collect();
    assert_eq!(
        oxrdf03,
        vec![PINNED_OXRDF.to_string()],
        "resolved oxrdf 0.3.x changed: re-verify the canonical-token edge cases \
         in this file against upstream + W3C rdf12-n-quads, then update the pin \
         (sq-g6b6; see module docs)"
    );
    let oxttl02: Vec<String> = locked_versions("oxttl")
        .into_iter()
        .filter(|v| v.starts_with("0.2."))
        .collect();
    assert_eq!(
        oxttl02,
        vec![PINNED_OXTTL.to_string()],
        "resolved oxttl 0.2.x changed: re-verify the canonical-token edge cases \
         in this file against upstream + W3C rdf12-n-quads, then update the pin \
         (sq-g6b6; see module docs)"
    );
}

/// Triple-term token form: `<<(` SP … SP `)>>`, inner terms space-separated.
#[test]
fn triple_term_token_byte_exact() {
    let line = canon_line(tt(
        NamedOrBlankNode::NamedNode(iri("http://ex/s")),
        iri("http://ex/p"),
        Term::Literal(Literal::new_simple_literal("v")),
    ));
    assert_eq!(
        line,
        r#"<http://ex/a> <http://ex/says> <<( <http://ex/s> <http://ex/p> "v" )>> ."#
    );
}

/// Nested triple-term token form: the inner `<<( … )>>` sits in the object
/// slot of the outer token with the same single-space separation.
#[test]
fn nested_triple_term_token_byte_exact() {
    let inner = tt(
        NamedOrBlankNode::NamedNode(iri("http://ex/s2")),
        iri("http://ex/p2"),
        Term::Literal(Literal::new_simple_literal("w")),
    );
    let line = canon_line(tt(
        NamedOrBlankNode::NamedNode(iri("http://ex/s")),
        iri("http://ex/p"),
        inner,
    ));
    assert_eq!(
        line,
        r#"<http://ex/a> <http://ex/says> <<( <http://ex/s> <http://ex/p> <<( <http://ex/s2> <http://ex/p2> "w" )>> )>> ."#
    );
}

/// Directional-language token form: `@lang--dir`, direction lowercase
/// `ltr`/`rtl`, no explicit datatype IRI — both nested in a triple term and
/// at the top level.
#[test]
fn directional_language_token_byte_exact() {
    let rtl = Literal::new_directional_language_tagged_literal(
        "שלום",
        "he",
        oxrdf::BaseDirection::Rtl,
    )
    .unwrap();
    let line = canon_line(tt(
        NamedOrBlankNode::NamedNode(iri("http://ex/s")),
        iri("http://ex/p"),
        Term::Literal(rtl),
    ));
    assert_eq!(
        line,
        "<http://ex/a> <http://ex/says> <<( <http://ex/s> <http://ex/p> \"שלום\"@he--rtl )>> ."
    );

    let ltr =
        Literal::new_directional_language_tagged_literal("hello", "en", oxrdf::BaseDirection::Ltr)
            .unwrap();
    let line = canon_line(Term::Literal(ltr));
    assert_eq!(
        line,
        r#"<http://ex/a> <http://ex/says> "hello"@en--ltr ."#
    );
}

/// Canonical string escaping inside a triple-term literal: ECHAR forms for
/// `"` `\` LF CR TAB (plus `\b`/`\f`), `\uXXXX` for other C0 controls and
/// DEL. This is the surface most likely to move if the final rdf12-n-quads
/// canonical form tightens escaping — keep it byte-exact.
#[test]
fn literal_escaping_in_triple_term_byte_exact() {
    let value = "q:\" b:\\ nl:\n cr:\r tab:\t bs:\u{08} ff:\u{0C} c0:\u{01} del:\u{7F}";
    let line = canon_line(tt(
        NamedOrBlankNode::NamedNode(iri("http://ex/s")),
        iri("http://ex/p"),
        Term::Literal(Literal::new_simple_literal(value)),
    ));
    assert_eq!(
        line,
        "<http://ex/a> <http://ex/says> <<( <http://ex/s> <http://ex/p> \
         \"q:\\\" b:\\\\ nl:\\n cr:\\r tab:\\t bs:\\b ff:\\f c0:\\u0001 del:\\u007F\" )>> ."
    );
}

/// Parser/serializer agreement on the canonical tokens: every canonical line
/// above must re-parse through oxttl's N-Quads parser (the parser the profile
/// itself uses) and `Display` back to the identical bytes. A one-sided
/// upstream change (parser xor serializer) breaks here even if the serialized
/// form alone still looks plausible.
#[test]
fn canonical_token_lines_round_trip_through_oxttl() {
    let lines = [
        r#"<http://ex/a> <http://ex/says> <<( <http://ex/s> <http://ex/p> "v" )>> ."#,
        r#"<http://ex/a> <http://ex/says> <<( <http://ex/s> <http://ex/p> <<( <http://ex/s2> <http://ex/p2> "w" )>> )>> ."#,
        "<http://ex/a> <http://ex/says> <<( <http://ex/s> <http://ex/p> \"שלום\"@he--rtl )>> .",
        r#"<http://ex/a> <http://ex/says> "hello"@en--ltr ."#,
        "<http://ex/a> <http://ex/says> <<( <http://ex/s> <http://ex/p> \
         \"q:\\\" b:\\\\ nl:\\n cr:\\r tab:\\t bs:\\b ff:\\f c0:\\u0001 del:\\u007F\" )>> .",
    ];
    for line in lines {
        let quads: Vec<oxrdf::Quad> = oxttl::NQuadsParser::new()
            .for_slice(line.as_bytes())
            .collect::<Result<_, _>>()
            .unwrap_or_else(|e| panic!("canonical line must re-parse: {line:?}: {e}"));
        assert_eq!(quads.len(), 1, "exactly one quad per line: {line:?}");
        let q = &quads[0];
        let reserialized = format!("{} {} {} .", q.subject, q.predicate, q.object);
        assert_eq!(
            reserialized, line,
            "parse ∘ print must be the identity on canonical token lines"
        );
    }
}
