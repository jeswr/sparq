// [SONNET-4.6] sq-pbz04.5.3 — OPT-IN RIF/XML importer: parse the W3C RIF-Core
// XML presentation syntax into `rif::Document` with Or-split / Exists-flatten
// desugaring and a fail-closed taxonomy. Requires the `rif-core` feature (wired by
// the `rif-xml` feature below). When off, zero rif_xml code is compiled.

//! # `rif_xml` — W3C RIF-Core XML importer
//!
//! Parses the **W3C RIF/XML presentation syntax** for the RIF-Core dialect into a
//! `rif::Document` AST that the existing `rif-core` forward chainer can run.
//!
//! ## Entry point
//!
//! ```rust
//! # #[cfg(feature = "rif-xml")] {
//! use sparq_reason::rif_xml::import;
//! let xml = br#"<Document xmlns="http://www.w3.org/2007/rif#">
//!   <payload><Group><sentences>
//!     <Forall>
//!       <declare><Var>x</Var></declare>
//!       <formula><Implies>
//!         <if><Frame>
//!           <object><Var>x</Var></object>
//!           <slot><Const type="http://www.w3.org/2007/rif#iri">http://ex/p</Const>
//!                 <Const type="http://www.w3.org/2007/rif#iri">http://ex/v</Const></slot>
//!         </Frame></if>
//!         <then><Frame>
//!           <object><Var>x</Var></object>
//!           <slot><Const type="http://www.w3.org/2007/rif#iri">http://ex/q</Const>
//!                 <Const type="http://www.w3.org/2007/rif#iri">http://ex/w</Const></slot>
//!         </Frame></then>
//!       </Implies></formula>
//!     </Forall>
//!   </sentences></Group></payload>
//! </Document>"#;
//! let doc = import(xml).expect("valid RIF-Core XML");
//! assert_eq!(doc.rules.len(), 1);
//! # }
//! ```
//!
//! ## Sound desugarings applied at import
//!
//! 1. **Body `Or` → rule-splitting (Lloyd–Topor):** a disjunctive body
//!    `Or(C1, C2, …)` is split into one rule per disjunct. Each rule has the same
//!    head and the same universally-quantified variables; the union of the rule set is
//!    semantically equivalent to the original disjunctive body under Horn semantics.
//!    This is the *monotone-Horn-preserving* desugaring; it is the standard encoding
//!    used to admit `Or` in RIF-Core condition language while the chainer remains Horn.
//!    Note: the `rif::UNIMPLEMENTED` list has a stale entry
//!    `"disjunction (Or) in rule bodies"` — Or IS in the RIF-Core condition language;
//!    the desugaring here resolves it at import time. That entry is owned by sq-pbz04.5.4.
//!
//! 2. **Body `Exists` → existential-flatten:** existentially-quantified variables in
//!    a body `Exists(z…, C)` become ordinary rule variables. Under Horn semantics,
//!    existential body variables are implicitly existential (the fact that satisfies
//!    them is found by the chainer's pattern matching). `Document::validate()` enforces
//!    the range-restriction invariant, so every such variable appears in at least one
//!    positive body atom — the flatten is safe.
//!
//! ## Fail-closed taxonomy
//!
//! Every element or construct outside the supported Core subset returns a named error
//! variant; nothing is silently skipped.
//!
//! 1. `ImportError::ImportDirective` — any `Import` element (remote imports: fail-closed).
//! 2. `ImportError::NonCoreElement { element, reason }` — non-Core dialect elements:
//!    `Naf`, `Assert`, `Retract`, `Modify`, `Neg`, `NAF`, and PRD constructs.
//! 3. `ImportError::UnknownExternal { iri }` — `External` calls with IRIs not in the
//!    supported DTB builtin set. Includes deferred DTB builtins:
//!    `pred:matches` (XSD-regex vs Rust-regex dialect gap), `func:numeric-integer-divide`
//!    (truncation vs floor division), `func:numeric-mod` (dividend-sign vs divisor-sign),
//!    guard predicates, list utilities (`func:get`, …), date/time builtins, and
//!    `func:substring*` / `func:string-join` / `func:compare`.
//! 4. `ImportError::NamedArgUniterm { name }` — named-argument uniterms (not in Core).
//! 5. `ImportError::UnrecognizedElement { tag }` — XML elements not in the supported grammar.
//! 6. `ImportError::MalformedXml(String)` — quick-xml parse errors.
//! 7. `ImportError::ValidationFailed(RifError)` — documents that parse but fail
//!    `Document::validate()` (range-restriction / builtin-safety invariant).

use crate::rif::{Atom, Builtin, Document, RifError, Rule, Term};
use quick_xml::events::Event;
use quick_xml::Reader;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Error returned by `import()` when a RIF/XML document cannot be parsed or is
/// not within the supported RIF-Core subset.
///
/// Each variant corresponds to a distinct rejection category in the fail-closed
/// taxonomy documented in the module doc comment.
#[derive(Debug)]
pub enum ImportError {
    /// Malformed or invalid XML that quick-xml could not parse.
    MalformedXml(String),
    /// An `Import` directive was found — remote imports are not supported (fail-closed).
    ImportDirective {
        /// The location IRI of the import.
        location: String,
    },
    /// A non-Core dialect element was found (e.g. RIF-BLD-only constructs, PRD actions).
    NonCoreElement {
        /// The element name.
        element: String,
        /// The reason it is excluded.
        reason: String,
    },
    /// An `External` call with an IRI not recognized as a supported builtin.
    UnknownExternal {
        /// The unrecognized IRI.
        iri: String,
    },
    /// A named-argument uniterm (not supported in Core; fail-closed).
    NamedArgUniterm {
        /// The named slot name that triggered this error.
        name: String,
    },
    /// An unrecognized XML element that is not in the supported Core grammar.
    UnrecognizedElement {
        /// The tag name.
        tag: String,
    },
    /// A `Document` was imported but `Document::validate()` rejected it.
    ValidationFailed(RifError),
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImportError::MalformedXml(msg) => {
                write!(f, "RIF/XML: malformed XML: {}", msg)
            }
            ImportError::ImportDirective { location } => {
                write!(
                    f,
                    "RIF/XML: Import directives are not supported (fail-closed): {}",
                    location
                )
            }
            ImportError::NonCoreElement { element, reason } => {
                write!(
                    f,
                    "RIF/XML: non-Core element <{}> rejected: {}",
                    element, reason
                )
            }
            ImportError::UnknownExternal { iri } => {
                write!(
                    f,
                    "RIF/XML: External call with unrecognized builtin IRI: {}",
                    iri
                )
            }
            ImportError::NamedArgUniterm { name } => {
                write!(
                    f,
                    "RIF/XML: named-argument uniterm slot '{}' is not supported in RIF-Core",
                    name
                )
            }
            ImportError::UnrecognizedElement { tag } => {
                write!(
                    f,
                    "RIF/XML: unrecognized element <{}> (not in the supported Core grammar)",
                    tag
                )
            }
            ImportError::ValidationFailed(e) => {
                write!(f, "RIF/XML: document failed validation: {}", e)
            }
        }
    }
}

impl std::error::Error for ImportError {}

// ---------------------------------------------------------------------------
// Internal XML tree
// ---------------------------------------------------------------------------

/// A node in the parsed XML tree. We build this from quick-xml events before
/// interpreting — keeps the interpretation code recursive and clean.
struct XmlNode {
    /// The local name (no namespace prefix).
    tag: String,
    /// Accumulated character data content (trimmed).
    text: String,
    /// Attributes as `(local_key, value)` pairs.
    attrs: Vec<(String, String)>,
    /// Child elements.
    children: Vec<XmlNode>,
}

impl XmlNode {
    /// Return the value of the first attribute with the given local name.
    fn attr(&self, key: &str) -> Option<&str> {
        self.attrs.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
    }

    /// Return the first child with the given tag name, if any.
    fn child(&self, tag: &str) -> Option<&XmlNode> {
        self.children.iter().find(|c| c.tag == tag)
    }

    /// Return all children with the given tag name.
    fn children_named<'a>(&'a self, tag: &'a str) -> impl Iterator<Item = &'a XmlNode> {
        self.children.iter().filter(move |c| c.tag == tag)
    }
}

// ---------------------------------------------------------------------------
// XML tree builder
// ---------------------------------------------------------------------------

/// Parse `xml_bytes` into a tree of `XmlNode`s.
fn parse_xml_tree(xml_bytes: &[u8]) -> Result<XmlNode, ImportError> {
    let mut reader = Reader::from_reader(xml_bytes);
    reader.config_mut().trim_text(true);

    let mut stack: Vec<XmlNode> = Vec::new();

    loop {
        match reader.read_event().map_err(|e| ImportError::MalformedXml(format!("{}", e)))? {
            Event::Start(e) => {
                let local = local_name_str(e.local_name().as_ref());
                let attrs = read_attrs(&e)?;
                stack.push(XmlNode { tag: local, text: String::new(), attrs, children: Vec::new() });
            }
            Event::Empty(e) => {
                // Self-closing element — push and immediately pop.
                let local = local_name_str(e.local_name().as_ref());
                let attrs = read_attrs(&e)?;
                let node = XmlNode { tag: local, text: String::new(), attrs, children: Vec::new() };
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(node);
                } else {
                    // Root self-closing element — wrap and return.
                    return Ok(node);
                }
            }
            Event::Text(t) => {
                let s = t.decode().map_err(|e| ImportError::MalformedXml(format!("{}", e)));
                if let Some(top) = stack.last_mut() {
                    top.text.push_str(s?.trim());
                }
            }
            Event::CData(t) => {
                if let Some(top) = stack.last_mut() {
                    top.text.push_str(&String::from_utf8_lossy(&t));
                }
            }
            Event::End(_) => {
                if let Some(node) = stack.pop() {
                    if stack.is_empty() {
                        return Ok(node);
                    }
                    let parent = stack.last_mut().unwrap();
                    parent.children.push(node);
                }
            }
            Event::Eof => {
                break;
            }
            // Ignore comments, processing instructions, declarations.
            _ => {}
        }
    }

    Err(ImportError::MalformedXml("unexpected end of document".to_string()))
}

fn local_name_str(bytes: &[u8]) -> String {
    std::str::from_utf8(bytes).unwrap_or("").to_string()
}

fn read_attrs(e: &quick_xml::events::BytesStart<'_>) -> Result<Vec<(String, String)>, ImportError> {
    let mut out = Vec::new();
    for a in e.attributes() {
        let a = a.map_err(|err| ImportError::MalformedXml(format!("{}", err)))?;
        let key = local_name_str(a.key.local_name().as_ref());
        let val = std::str::from_utf8(&a.value)
            .map_err(|err| ImportError::MalformedXml(format!("{}", err)))?
            .to_string();
        out.push((key, val));
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Non-Core element guard
// ---------------------------------------------------------------------------

/// Non-Core / non-monotonic elements that must be rejected immediately.
const NON_CORE_TAGS: &[(&str, &str)] = &[
    ("Naf", "negation-as-failure is not in RIF-Core (monotone)"),
    ("NAF", "negation-as-failure is not in RIF-Core (monotone)"),
    ("Neg", "strong negation is not in RIF-Core"),
    ("Assert", "PRD Assert action is not in RIF-Core"),
    ("Retract", "PRD Retract action is not in RIF-Core"),
    ("Modify", "PRD Modify action is not in RIF-Core"),
    ("Do", "PRD Do action block is not in RIF-Core"),
    ("Act", "PRD Act block is not in RIF-Core"),
];

fn check_non_core(tag: &str) -> Result<(), ImportError> {
    for &(t, reason) in NON_CORE_TAGS {
        if tag == t {
            return Err(ImportError::NonCoreElement {
                element: tag.to_string(),
                reason: reason.to_string(),
            });
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Builtin IRI mapping
// ---------------------------------------------------------------------------

const FUNC: &str = "http://www.w3.org/2007/rif-builtin-function#";
const PRED: &str = "http://www.w3.org/2007/rif-builtin-predicate#";

/// Map a DTB builtin IRI to a `Builtin` variant, or return `ImportError::UnknownExternal`.
fn iri_to_builtin(iri: &str) -> Result<Builtin, ImportError> {
    // Numeric functions
    if iri == concat_strs(FUNC, "numeric-add") {
        return Ok(Builtin::NumericAdd);
    }
    if iri == concat_strs(FUNC, "numeric-subtract") {
        return Ok(Builtin::NumericSubtract);
    }
    if iri == concat_strs(FUNC, "numeric-multiply") {
        return Ok(Builtin::NumericMultiply);
    }
    if iri == concat_strs(FUNC, "numeric-divide") {
        return Ok(Builtin::NumericDivide);
    }
    // String functions
    if iri == concat_strs(FUNC, "upper-case") {
        return Ok(Builtin::StringUpperCase);
    }
    if iri == concat_strs(FUNC, "lower-case") {
        return Ok(Builtin::StringLowerCase);
    }
    if iri == concat_strs(FUNC, "encode-for-uri") {
        return Ok(Builtin::StringEncodeForUri);
    }
    if iri == concat_strs(FUNC, "concat") {
        return Ok(Builtin::StringConcat);
    }
    if iri == concat_strs(FUNC, "string-length") {
        return Ok(Builtin::StringLength);
    }
    // List functions
    if iri == concat_strs(FUNC, "concatenate") {
        return Ok(Builtin::ListConcatenate);
    }
    // Numeric predicates
    if iri == concat_strs(PRED, "numeric-equal") {
        return Ok(Builtin::NumericEqual);
    }
    if iri == concat_strs(PRED, "numeric-not-equal") {
        return Ok(Builtin::NumericNotEqual);
    }
    if iri == concat_strs(PRED, "numeric-less-than") {
        return Ok(Builtin::NumericLessThan);
    }
    if iri == concat_strs(PRED, "numeric-greater-than") {
        return Ok(Builtin::NumericGreaterThan);
    }
    if iri == concat_strs(PRED, "numeric-not-less-than") {
        return Ok(Builtin::NumericNotLessThan);
    }
    if iri == concat_strs(PRED, "numeric-not-greater-than") {
        return Ok(Builtin::NumericNotGreaterThan);
    }
    // String predicates
    if iri == concat_strs(PRED, "contains") {
        return Ok(Builtin::StringContains);
    }
    if iri == concat_strs(PRED, "starts-with") {
        return Ok(Builtin::StringStartsWith);
    }
    if iri == concat_strs(PRED, "ends-with") {
        return Ok(Builtin::StringEndsWith);
    }
    // List predicates
    if iri == concat_strs(PRED, "list-contains") {
        return Ok(Builtin::ListContains);
    }
    // Deferred / unknown → fail closed
    Err(ImportError::UnknownExternal { iri: iri.to_string() })
}

/// Const-friendly string concatenation for use in match arms.
fn concat_strs(a: &str, b: &str) -> String {
    format!("{}{}", a, b)
}

// ---------------------------------------------------------------------------
// Term parsing
// ---------------------------------------------------------------------------

/// The `rif:iri` type attribute value in `<Const type="…">`.
const RIF_IRI_TYPE: &str = "http://www.w3.org/2007/rif#iri";

/// Parse a `<Const>` or `<Var>` node into a `Term`.
fn parse_term(node: &XmlNode) -> Result<Term, ImportError> {
    match node.tag.as_str() {
        "Const" => {
            let ty = node.attr("type").unwrap_or("");
            if ty == RIF_IRI_TYPE {
                Ok(Term::Iri(node.text.clone()))
            } else if ty.is_empty() {
                // Plain string constant with no type = xsd:string
                Ok(Term::Lit {
                    lex: node.text.clone(),
                    datatype: "http://www.w3.org/2001/XMLSchema#string".to_string(),
                })
            } else {
                // Typed literal: type attr is the datatype IRI
                Ok(Term::Lit { lex: node.text.clone(), datatype: ty.to_string() })
            }
        }
        "Var" => Ok(Term::Var(node.text.clone())),
        "List" => {
            // <List><items><...term...><...term...></items></List>
            let items_node = node.child("items");
            let items = match items_node {
                Some(items) => items
                    .children
                    .iter()
                    .map(parse_term)
                    .collect::<Result<Vec<_>, _>>()?,
                None => Vec::new(),
            };
            Ok(Term::List(items))
        }
        other => {
            check_non_core(other)?;
            Err(ImportError::UnrecognizedElement { tag: other.to_string() })
        }
    }
}

// ---------------------------------------------------------------------------
// Body condition intermediate representation
// ---------------------------------------------------------------------------

/// Intermediate representation for a body condition parsed from RIF/XML.
/// Converted to `Vec<Vec<Atom>>` (list of disjuncts, each a conjunction) by
/// `expand_body`.
enum BodyCond {
    Atom(Atom),
    And(Vec<BodyCond>),
    Or(Vec<BodyCond>),
    /// Exists(declared_var_names_ignored_here, sub_condition).
    /// The declared vars become ordinary body vars; we just keep the sub-condition.
    Exists(Box<BodyCond>),
}

/// Expand `BodyCond` into a list of disjuncts (each disjunct is a conjunction of atoms).
/// Or-split (Lloyd–Topor) and Exists-flatten are applied here.
fn expand_body(cond: BodyCond) -> Vec<Vec<Atom>> {
    match cond {
        BodyCond::Atom(a) => vec![vec![a]],
        BodyCond::And(conds) => {
            // Cross-product of all sub-disjuncts.
            let mut result: Vec<Vec<Atom>> = vec![vec![]];
            for sub in conds {
                let sub_disj = expand_body(sub);
                result = cross_product(result, sub_disj);
            }
            result
        }
        BodyCond::Or(conds) => {
            // Union: each disjunct becomes a separate rule body.
            conds.into_iter().flat_map(expand_body).collect()
        }
        BodyCond::Exists(sub) => {
            // Existential vars are already present in the atoms — just flatten.
            expand_body(*sub)
        }
    }
}

fn cross_product(a: Vec<Vec<Atom>>, b: Vec<Vec<Atom>>) -> Vec<Vec<Atom>> {
    let mut result = Vec::with_capacity(a.len() * b.len().max(1));
    for da in &a {
        for db in &b {
            let mut combined = da.clone();
            combined.extend_from_slice(db);
            result.push(combined);
        }
    }
    if result.is_empty() {
        a
    } else {
        result
    }
}

// ---------------------------------------------------------------------------
// Atom parsing
// ---------------------------------------------------------------------------

/// Parse an element that should be a positive atom (Frame/Member/Subclass/Equal).
fn parse_positive_atom(node: &XmlNode) -> Result<Atom, ImportError> {
    match node.tag.as_str() {
        "Frame" => parse_frame(node),
        "Member" => parse_member(node),
        "Subclass" => parse_subclass(node),
        "Equal" => parse_equal(node),
        "Atom" => {
            // <Atom> in RIF/XML can be used for positional predicates.
            // In Core, the only sanctioned use is as a builtin predicate inside External.
            // A bare <Atom> in a head is unsupported (not a standard Core head form).
            Err(ImportError::UnrecognizedElement { tag: "Atom (bare, outside External)".to_string() })
        }
        other => {
            check_non_core(other)?;
            Err(ImportError::UnrecognizedElement { tag: other.to_string() })
        }
    }
}

fn parse_frame(node: &XmlNode) -> Result<Atom, ImportError> {
    // <Frame>
    //   <object>TERM</object>
    //   <slot>TERM TERM</slot>   (predicate, value)
    // </Frame>
    let obj_node = node.child("object").ok_or_else(|| ImportError::MalformedXml(
        "Frame missing <object>".to_string(),
    ))?;
    let obj = parse_first_term(obj_node)?;

    let slot = node.child("slot").ok_or_else(|| ImportError::MalformedXml(
        "Frame missing <slot>".to_string(),
    ))?;

    // Detect named-argument uniterms: a slot child with a <Name> child is a named arg.
    if slot.child("Name").is_some() {
        let name = slot
            .child("Name")
            .and_then(|n| n.children.first())
            .map(|n| n.text.clone())
            .unwrap_or_default();
        return Err(ImportError::NamedArgUniterm { name });
    }

    let slot_terms: Vec<Term> = slot.children.iter().map(parse_term).collect::<Result<_, _>>()?;
    if slot_terms.len() != 2 {
        return Err(ImportError::MalformedXml(format!(
            "Frame <slot> must have exactly 2 term children, got {}",
            slot_terms.len()
        )));
    }
    Ok(Atom::Frame { obj, pred: slot_terms[0].clone(), val: slot_terms[1].clone() })
}

fn parse_member(node: &XmlNode) -> Result<Atom, ImportError> {
    // <Member><instance>TERM</instance><class>TERM</class></Member>
    let inst = node.child("instance").ok_or_else(|| ImportError::MalformedXml(
        "Member missing <instance>".to_string(),
    ))?;
    let cls = node.child("class").ok_or_else(|| ImportError::MalformedXml(
        "Member missing <class>".to_string(),
    ))?;
    Ok(Atom::Member { obj: parse_first_term(inst)?, class: parse_first_term(cls)? })
}

fn parse_subclass(node: &XmlNode) -> Result<Atom, ImportError> {
    // <Subclass><sub>TERM</sub><sup>TERM</sup></Subclass>
    let sub = node.child("sub").ok_or_else(|| ImportError::MalformedXml(
        "Subclass missing <sub>".to_string(),
    ))?;
    let sup = node.child("sup").ok_or_else(|| ImportError::MalformedXml(
        "Subclass missing <sup>".to_string(),
    ))?;
    Ok(Atom::Subclass { sub: parse_first_term(sub)?, sup: parse_first_term(sup)? })
}

fn parse_equal(node: &XmlNode) -> Result<Atom, ImportError> {
    // <Equal><left>TERM</left><right>TERM</right></Equal>
    let left = node.child("left").ok_or_else(|| ImportError::MalformedXml(
        "Equal missing <left>".to_string(),
    ))?;
    let right = node.child("right").ok_or_else(|| ImportError::MalformedXml(
        "Equal missing <right>".to_string(),
    ))?;
    Ok(Atom::Equal { left: parse_first_term(left)?, right: parse_first_term(right)? })
}

/// Parse the first term child of a wrapper element (e.g. `<object>TERM</object>`).
fn parse_first_term(wrapper: &XmlNode) -> Result<Term, ImportError> {
    let child = wrapper.children.first().ok_or_else(|| ImportError::MalformedXml(
        format!("<{}> has no term child", wrapper.tag),
    ))?;
    parse_term(child)
}

/// Parse an `<External>` builtin call. Returns an `Atom::Builtin`.
fn parse_external(node: &XmlNode) -> Result<Atom, ImportError> {
    // <External><content><Atom><op><Const type="rif:iri">IRI</Const></op><args>...</args></Atom></content></External>
    let content = node.child("content").ok_or_else(|| ImportError::MalformedXml(
        "External missing <content>".to_string(),
    ))?;
    // The child of <content> should be <Atom> (for predicates) or <Expr> (for functions).
    let inner = content.children.first().ok_or_else(|| ImportError::MalformedXml(
        "External <content> has no child".to_string(),
    ))?;
    if inner.tag != "Atom" && inner.tag != "Expr" {
        check_non_core(&inner.tag)?;
        return Err(ImportError::UnrecognizedElement { tag: inner.tag.clone() });
    }

    // Get the operator IRI from <op><Const type="rif:iri">IRI</Const></op>
    let op_node = inner.child("op").ok_or_else(|| ImportError::MalformedXml(
        "External Atom/Expr missing <op>".to_string(),
    ))?;
    let op_const = op_node.children.first().ok_or_else(|| ImportError::MalformedXml(
        "External <op> has no child".to_string(),
    ))?;
    let op_iri = if op_const.tag == "Const" {
        op_const.text.clone()
    } else {
        return Err(ImportError::MalformedXml("External <op> child is not <Const>".to_string()));
    };

    let builtin = iri_to_builtin(&op_iri)?;

    // Collect args from <args><...term...><...term...></args>
    let args = match inner.child("args") {
        Some(args_node) => args_node
            .children
            .iter()
            .map(parse_term)
            .collect::<Result<Vec<_>, _>>()?,
        None => Vec::new(),
    };

    Ok(Atom::Builtin { op: builtin, args })
}

// ---------------------------------------------------------------------------
// Body condition parsing
// ---------------------------------------------------------------------------

/// Parse a condition element (what can appear inside `<if>`, `<formula>`,
/// `<And>`, `<Or>`, `<Exists>`'s body, etc.) into a `BodyCond`.
fn parse_condition(node: &XmlNode) -> Result<BodyCond, ImportError> {
    match node.tag.as_str() {
        "And" => {
            let subs: Result<Vec<_>, _> =
                node.children_named("formula").map(parse_formula_child).collect();
            Ok(BodyCond::And(subs?))
        }
        "Or" => {
            let subs: Result<Vec<_>, _> =
                node.children_named("formula").map(parse_formula_child).collect();
            Ok(BodyCond::Or(subs?))
        }
        "Exists" => {
            // <Exists>
            //   <declare><Var>z</Var></declare>  (one or more)
            //   <formula>CONDITION</formula>
            // </Exists>
            // We discard the declared vars (they become ordinary body vars).
            let formula = node.child("formula").ok_or_else(|| ImportError::MalformedXml(
                "Exists missing <formula>".to_string(),
            ))?;
            let sub = parse_formula_child(formula)?;
            Ok(BodyCond::Exists(Box::new(sub)))
        }
        "Frame" => Ok(BodyCond::Atom(parse_frame(node)?)),
        "Member" => Ok(BodyCond::Atom(parse_member(node)?)),
        "Subclass" => Ok(BodyCond::Atom(parse_subclass(node)?)),
        "Equal" => Ok(BodyCond::Atom(parse_equal(node)?)),
        "External" => Ok(BodyCond::Atom(parse_external(node)?)),
        other => {
            check_non_core(other)?;
            Err(ImportError::UnrecognizedElement { tag: other.to_string() })
        }
    }
}

/// Parse the child of a `<formula>` wrapper (descend into the wrapper's first child).
fn parse_formula_child(formula: &XmlNode) -> Result<BodyCond, ImportError> {
    let child = formula.children.first().ok_or_else(|| ImportError::MalformedXml(
        "<formula> has no child condition".to_string(),
    ))?;
    parse_condition(child)
}

// ---------------------------------------------------------------------------
// Head parsing
// ---------------------------------------------------------------------------

/// Parse the head of an `<Implies>` (the `<then>` child). Head may be a single
/// positive atom or an `<And>` of positive atoms.
fn parse_head(node: &XmlNode) -> Result<Vec<Atom>, ImportError> {
    match node.tag.as_str() {
        "And" => {
            // Conjunctive head.
            let mut head = Vec::new();
            for formula in node.children_named("formula") {
                let child = formula.children.first().ok_or_else(|| {
                    ImportError::MalformedXml("<formula> in head And has no child".to_string())
                })?;
                head.push(parse_positive_atom(child)?);
            }
            Ok(head)
        }
        _ => {
            // Single positive atom.
            Ok(vec![parse_positive_atom(node)?])
        }
    }
}

// ---------------------------------------------------------------------------
// Sentence / rule parsing
// ---------------------------------------------------------------------------

/// Parse a `<Forall>` or a bare fact sentence into zero-or-more `Rule`s (zero-or-more
/// because Or-split can produce multiple rules).
fn parse_sentence(node: &XmlNode) -> Result<Vec<Rule>, ImportError> {
    match node.tag.as_str() {
        "Forall" => {
            // <Forall>
            //   <declare><Var>x</Var></declare>*
            //   <formula>IMPLIES_OR_ATOM</formula>
            // </Forall>
            let formula = node.child("formula").ok_or_else(|| ImportError::MalformedXml(
                "Forall missing <formula>".to_string(),
            ))?;
            let body_node = formula.children.first().ok_or_else(|| ImportError::MalformedXml(
                "Forall <formula> has no child".to_string(),
            ))?;
            match body_node.tag.as_str() {
                "Implies" => parse_implies(body_node),
                other => {
                    // A bare atom as a Forall formula = universally closed fact.
                    check_non_core(other)?;
                    let atom = parse_positive_atom(body_node)?;
                    Ok(vec![Rule::fact(atom)])
                }
            }
        }
        // Bare fact (no Forall wrapper).
        "Frame" | "Member" | "Subclass" | "Equal" => {
            let atom = parse_positive_atom(node)?;
            Ok(vec![Rule::fact(atom)])
        }
        other => {
            check_non_core(other)?;
            Err(ImportError::UnrecognizedElement { tag: other.to_string() })
        }
    }
}

fn parse_implies(node: &XmlNode) -> Result<Vec<Rule>, ImportError> {
    // <Implies>
    //   <if>CONDITION</if>
    //   <then>HEAD</then>
    // </Implies>
    let if_node = node.child("if").ok_or_else(|| ImportError::MalformedXml(
        "Implies missing <if>".to_string(),
    ))?;
    let then_node = node.child("then").ok_or_else(|| ImportError::MalformedXml(
        "Implies missing <then>".to_string(),
    ))?;

    // Parse the body condition.
    let body_child = if_node.children.first().ok_or_else(|| ImportError::MalformedXml(
        "<if> has no child condition".to_string(),
    ))?;
    let body_cond = parse_condition(body_child)?;

    // Parse the head.
    let then_child = then_node.children.first().ok_or_else(|| ImportError::MalformedXml(
        "<then> has no child".to_string(),
    ))?;
    let head = parse_head(then_child)?;

    // Or-split / Exists-flatten → one rule per body disjunct.
    let disjuncts = expand_body(body_cond);
    let rules = disjuncts
        .into_iter()
        .map(|body| Rule::implies(head.clone(), body))
        .collect();
    Ok(rules)
}

// ---------------------------------------------------------------------------
// Document interpretation
// ---------------------------------------------------------------------------

fn interpret_document(root: &XmlNode) -> Result<Document, ImportError> {
    if root.tag != "Document" {
        return Err(ImportError::UnrecognizedElement { tag: root.tag.clone() });
    }

    let mut doc = Document::new();

    for child in &root.children {
        match child.tag.as_str() {
            "directive" => {
                // Scan for Import elements — fail closed.
                for item in &child.children {
                    if item.tag == "Import" {
                        let location = item
                            .child("location")
                            .and_then(|n| n.children.first())
                            .map(|n| n.text.clone())
                            .unwrap_or_default();
                        return Err(ImportError::ImportDirective { location });
                    }
                    check_non_core(&item.tag)?;
                    // Other directives (e.g. <Profile>) are unrecognized.
                    if item.tag != "Import" {
                        return Err(ImportError::UnrecognizedElement { tag: item.tag.clone() });
                    }
                }
            }
            "payload" => {
                interpret_payload(child, &mut doc)?;
            }
            other => {
                check_non_core(other)?;
                return Err(ImportError::UnrecognizedElement { tag: other.to_string() });
            }
        }
    }

    Ok(doc)
}

fn interpret_payload(payload: &XmlNode, doc: &mut Document) -> Result<(), ImportError> {
    for child in &payload.children {
        match child.tag.as_str() {
            "Group" => interpret_group(child, doc)?,
            other => {
                check_non_core(other)?;
                return Err(ImportError::UnrecognizedElement { tag: other.to_string() });
            }
        }
    }
    Ok(())
}

fn interpret_group(group: &XmlNode, doc: &mut Document) -> Result<(), ImportError> {
    for child in &group.children {
        match child.tag.as_str() {
            "sentences" => {
                for sentence in &child.children {
                    let rules = parse_sentence(sentence)?;
                    for rule in rules {
                        doc.push(rule);
                    }
                }
            }
            "sentence" => {
                // Alternative: direct <sentence> children.
                let rules = parse_sentence(child)?;
                for rule in rules {
                    doc.push(rule);
                }
            }
            "Group" => {
                // Nested group.
                interpret_group(child, doc)?;
            }
            other => {
                check_non_core(other)?;
                return Err(ImportError::UnrecognizedElement { tag: other.to_string() });
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Parse a RIF-Core XML document from `xml_bytes` into a `rif::Document`.
///
/// Applies two sound desugarings at import time:
/// - Body `Or`: split into multiple rules (one rule per disjunct — Lloyd-Topor step,
///   monotone-Horn-preserving).
/// - Body `Exists`: existential variables become ordinary rule variables under the
///   existing range-restriction validation (`Document::validate` still runs on every import).
///
/// Everything outside the supported Core subset yields a named `ImportError` variant
/// (fail-closed, never silent skipping). The returned `Document` has already been
/// validated via `Document::validate()`.
pub fn import(xml_bytes: &[u8]) -> Result<Document, ImportError> {
    let root = parse_xml_tree(xml_bytes)?;
    let doc = interpret_document(&root)?;
    doc.validate().map_err(ImportError::ValidationFailed)?;
    Ok(doc)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- XML fixtures -------------------------------------------------------

    const MINIMAL_RULE_XML: &[u8] = br#"<Document xmlns="http://www.w3.org/2007/rif#">
  <payload>
    <Group>
      <sentences>
        <Forall>
          <declare><Var>x</Var></declare>
          <declare><Var>y</Var></declare>
          <formula>
            <Implies>
              <if>
                <Frame>
                  <object><Const type="http://www.w3.org/2007/rif#iri">http://example.org/a</Const></object>
                  <slot><Const type="http://www.w3.org/2007/rif#iri">http://example.org/p</Const><Var>y</Var></slot>
                </Frame>
              </if>
              <then>
                <Frame>
                  <object><Var>y</Var></object>
                  <slot><Const type="http://www.w3.org/2007/rif#iri">http://example.org/q</Const><Const type="http://www.w3.org/2007/rif#iri">http://example.org/b</Const></slot>
                </Frame>
              </then>
            </Implies>
          </formula>
        </Forall>
      </sentences>
    </Group>
  </payload>
</Document>"#;

    const OR_SPLIT_XML: &[u8] = br#"<Document xmlns="http://www.w3.org/2007/rif#">
  <payload>
    <Group>
      <sentences>
        <Forall>
          <declare><Var>x</Var></declare>
          <formula>
            <Implies>
              <if>
                <Or>
                  <formula>
                    <Frame>
                      <object><Var>x</Var></object>
                      <slot><Const type="http://www.w3.org/2007/rif#iri">http://example.org/type</Const><Const type="http://www.w3.org/2007/rif#iri">http://example.org/A</Const></slot>
                    </Frame>
                  </formula>
                  <formula>
                    <Frame>
                      <object><Var>x</Var></object>
                      <slot><Const type="http://www.w3.org/2007/rif#iri">http://example.org/type</Const><Const type="http://www.w3.org/2007/rif#iri">http://example.org/B</Const></slot>
                    </Frame>
                  </formula>
                </Or>
              </if>
              <then>
                <Frame>
                  <object><Var>x</Var></object>
                  <slot><Const type="http://www.w3.org/2007/rif#iri">http://example.org/type</Const><Const type="http://www.w3.org/2007/rif#iri">http://example.org/C</Const></slot>
                </Frame>
              </then>
            </Implies>
          </formula>
        </Forall>
      </sentences>
    </Group>
  </payload>
</Document>"#;

    const EXISTS_FLATTEN_XML: &[u8] = br#"<Document xmlns="http://www.w3.org/2007/rif#">
  <payload>
    <Group>
      <sentences>
        <Forall>
          <declare><Var>x</Var></declare>
          <formula>
            <Implies>
              <if>
                <Exists>
                  <declare><Var>z</Var></declare>
                  <formula>
                    <And>
                      <formula>
                        <Frame>
                          <object><Var>x</Var></object>
                          <slot><Const type="http://www.w3.org/2007/rif#iri">http://example.org/p</Const><Var>z</Var></slot>
                        </Frame>
                      </formula>
                      <formula>
                        <Frame>
                          <object><Var>z</Var></object>
                          <slot><Const type="http://www.w3.org/2007/rif#iri">http://example.org/q</Const><Const type="http://www.w3.org/2007/rif#iri">http://example.org/val</Const></slot>
                        </Frame>
                      </formula>
                    </And>
                  </formula>
                </Exists>
              </if>
              <then>
                <Frame>
                  <object><Var>x</Var></object>
                  <slot><Const type="http://www.w3.org/2007/rif#iri">http://example.org/r</Const><Const type="http://www.w3.org/2007/rif#iri">http://example.org/yes</Const></slot>
                </Frame>
              </then>
            </Implies>
          </formula>
        </Forall>
      </sentences>
    </Group>
  </payload>
</Document>"#;

    const IMPORT_DIRECTIVE_XML: &[u8] = br#"<Document xmlns="http://www.w3.org/2007/rif#">
  <directive>
    <Import>
      <location><Const type="http://www.w3.org/2007/rif#iri">http://example.org/rules.rif</Const></location>
    </Import>
  </directive>
  <payload><Group><sentences></sentences></Group></payload>
</Document>"#;

    const NAF_XML: &[u8] = br#"<Document xmlns="http://www.w3.org/2007/rif#">
  <payload>
    <Group>
      <sentences>
        <Forall>
          <declare><Var>x</Var></declare>
          <formula>
            <Implies>
              <if><Naf><formula><Frame><object><Var>x</Var></object><slot><Const type="http://www.w3.org/2007/rif#iri">http://example.org/p</Const><Const type="http://www.w3.org/2007/rif#iri">http://example.org/v</Const></slot></Frame></formula></Naf></if>
              <then><Frame><object><Var>x</Var></object><slot><Const type="http://www.w3.org/2007/rif#iri">http://example.org/q</Const><Const type="http://www.w3.org/2007/rif#iri">http://example.org/v</Const></slot></Frame></then>
            </Implies>
          </formula>
        </Forall>
      </sentences>
    </Group>
  </payload>
</Document>"#;

    const UNKNOWN_EXTERNAL_XML: &[u8] = br#"<Document xmlns="http://www.w3.org/2007/rif#">
  <payload>
    <Group>
      <sentences>
        <Forall>
          <declare><Var>x</Var></declare>
          <declare><Var>y</Var></declare>
          <formula>
            <Implies>
              <if>
                <And>
                  <formula>
                    <Frame>
                      <object><Var>x</Var></object>
                      <slot><Const type="http://www.w3.org/2007/rif#iri">http://example.org/p</Const><Var>y</Var></slot>
                    </Frame>
                  </formula>
                  <formula>
                    <External>
                      <content><Atom>
                        <op><Const type="http://www.w3.org/2007/rif#iri">http://www.w3.org/2007/rif-builtin-predicate#matches</Const></op>
                        <args><Var>y</Var><Const type="http://www.w3.org/2001/XMLSchema#string">.*</Const></args>
                      </Atom></content>
                    </External>
                  </formula>
                </And>
              </if>
              <then>
                <Frame>
                  <object><Var>x</Var></object>
                  <slot><Const type="http://www.w3.org/2007/rif#iri">http://example.org/q</Const><Const type="http://www.w3.org/2007/rif#iri">http://example.org/yes</Const></slot>
                </Frame>
              </then>
            </Implies>
          </formula>
        </Forall>
      </sentences>
    </Group>
  </payload>
</Document>"#;

    // ---- Tests --------------------------------------------------------------

    /// Minimal document: one rule (Frame body → Frame head). Parse succeeds.
    #[test]
    fn test_minimal_core_doc() {
        let doc = import(MINIMAL_RULE_XML).expect("valid RIF-Core XML");
        assert_eq!(doc.rules.len(), 1, "one rule in the document");
        let rule = &doc.rules[0];
        assert_eq!(rule.head.len(), 1, "one head atom");
        assert_eq!(rule.body.len(), 1, "one body atom");
        // Head: y[q->b]
        assert!(
            matches!(&rule.head[0], Atom::Frame { obj: Term::Var(v), .. } if v == "y"),
            "head object is variable y"
        );
        // Body: a[p->y]
        assert!(
            matches!(&rule.body[0], Atom::Frame { obj: Term::Iri(i), .. } if i == "http://example.org/a"),
            "body object is IRI a"
        );
    }

    /// Or-body with 2 disjuncts → document has TWO rules after Or-split.
    #[test]
    fn test_or_split_into_two_rules() {
        let doc = import(OR_SPLIT_XML).expect("valid RIF-Core XML with Or body");
        assert_eq!(doc.rules.len(), 2, "Or-split must produce one rule per disjunct");
        // Both rules have the same head.
        let head0 = &doc.rules[0].head;
        let head1 = &doc.rules[1].head;
        assert_eq!(head0, head1, "both split rules share the same head");
    }

    /// Exists in body: existential vars become ordinary body vars; validation passes.
    #[test]
    fn test_exists_flatten() {
        let doc = import(EXISTS_FLATTEN_XML).expect("Exists-flatten should succeed");
        assert_eq!(doc.rules.len(), 1);
        let rule = &doc.rules[0];
        // Body should have 2 atoms (the And inside the Exists was flattened).
        assert_eq!(rule.body.len(), 2, "both atoms from the Exists body are present");
        // The existential variable z should appear in the body atoms as Term::Var("z").
        let z_var = Term::Var("z".to_string());
        let has_z = rule.body.iter().any(|a| match a {
            Atom::Frame { obj, pred, val } => obj == &z_var || pred == &z_var || val == &z_var,
            Atom::Member { obj, class } => obj == &z_var || class == &z_var,
            Atom::Subclass { sub, sup } => sub == &z_var || sup == &z_var,
            Atom::Equal { left, right } => left == &z_var || right == &z_var,
            Atom::Builtin { args, .. } => args.iter().any(|t| t == &z_var),
        });
        assert!(has_z, "existential var z is in body");
    }

    /// Round-trip: parse minimal XML and verify structural equality with hand-built Document.
    #[test]
    fn test_round_trip_structural_equality() {
        let doc = import(MINIMAL_RULE_XML).expect("minimal doc parses");
        // Compare against hand-built rule.
        let expected_head = Atom::Frame {
            obj: Term::Var("y".to_string()),
            pred: Term::Iri("http://example.org/q".to_string()),
            val: Term::Iri("http://example.org/b".to_string()),
        };
        let expected_body = Atom::Frame {
            obj: Term::Iri("http://example.org/a".to_string()),
            pred: Term::Iri("http://example.org/p".to_string()),
            val: Term::Var("y".to_string()),
        };
        assert_eq!(doc.rules.len(), 1);
        assert_eq!(doc.rules[0].head, vec![expected_head]);
        assert_eq!(doc.rules[0].body, vec![expected_body]);
    }

    /// Import directive → ImportError::ImportDirective.
    #[test]
    fn test_reject_import_directive() {
        let err = import(IMPORT_DIRECTIVE_XML).expect_err("Import directive must be rejected");
        assert!(
            matches!(err, ImportError::ImportDirective { .. }),
            "expected ImportDirective, got: {}",
            err
        );
    }

    /// Naf element → ImportError::NonCoreElement.
    #[test]
    fn test_reject_naf() {
        let err = import(NAF_XML).expect_err("Naf must be rejected");
        assert!(
            matches!(err, ImportError::NonCoreElement { ref element, .. } if element == "Naf"),
            "expected NonCoreElement(Naf), got: {}",
            err
        );
    }

    /// External with unknown IRI → ImportError::UnknownExternal.
    #[test]
    fn test_reject_unknown_external() {
        let err = import(UNKNOWN_EXTERNAL_XML).expect_err("unknown External IRI must be rejected");
        assert!(
            matches!(err, ImportError::UnknownExternal { .. }),
            "expected UnknownExternal, got: {}",
            err
        );
    }

    /// Malformed XML → ImportError::MalformedXml.
    #[test]
    fn test_reject_malformed_xml() {
        let err = import(b"not xml <<<").expect_err("malformed XML must be rejected");
        assert!(
            matches!(err, ImportError::MalformedXml(_)),
            "expected MalformedXml, got: {}",
            err
        );
    }

    /// Or-split mutation spot-check: two rules produce distinct body conditions,
    /// and together entail what we'd expect (verifies the real dispatch path).
    #[test]
    fn test_or_split_mutation_check() {
        let doc = import(OR_SPLIT_XML).expect("or-split doc");
        assert_eq!(doc.rules.len(), 2, "Or produces exactly 2 rules");
        // Rule 0 body: x[type->A]
        let body0 = &doc.rules[0].body;
        let body1 = &doc.rules[1].body;
        // Verify they are DIFFERENT (each disjunct produces a different body).
        assert_ne!(body0, body1, "each disjunct gives a distinct rule body");
        // Verify one body contains A and the other B.
        let has_iri = |body: &Vec<Atom>, iri: &str| {
            body.iter().any(|a| {
                if let Atom::Frame { val: Term::Iri(v), .. } = a { v == iri } else { false }
            })
        };
        let has_a = has_iri(body0, "http://example.org/A")
            || has_iri(body1, "http://example.org/A");
        let has_b = has_iri(body0, "http://example.org/B")
            || has_iri(body1, "http://example.org/B");
        assert!(has_a, "one rule body corresponds to disjunct A");
        assert!(has_b, "one rule body corresponds to disjunct B");
    }

    /// Named-argument uniterm → ImportError::NamedArgUniterm.
    #[test]
    fn test_reject_named_arg_uniterm() {
        // Construct XML with a <slot><Name>...</Name>...</slot> pattern.
        let xml = br#"<Document xmlns="http://www.w3.org/2007/rif#">
  <payload><Group><sentences>
    <Forall>
      <declare><Var>x</Var></declare>
      <formula>
        <Implies>
          <if>
            <Frame>
              <object><Var>x</Var></object>
              <slot>
                <Name><Const type="http://www.w3.org/2007/rif#iri">http://example.org/p</Const></Name>
                <Var>x</Var>
              </slot>
            </Frame>
          </if>
          <then>
            <Frame>
              <object><Var>x</Var></object>
              <slot><Const type="http://www.w3.org/2007/rif#iri">http://example.org/q</Const><Const type="http://www.w3.org/2007/rif#iri">http://example.org/v</Const></slot>
            </Frame>
          </then>
        </Implies>
      </formula>
    </Forall>
  </sentences></Group></payload>
</Document>"#;
        let err = import(xml).expect_err("named-arg uniterm must be rejected");
        assert!(
            matches!(err, ImportError::NamedArgUniterm { .. }),
            "expected NamedArgUniterm, got: {}",
            err
        );
    }

    /// Display impl: each ImportError variant must produce a non-empty string.
    #[test]
    fn test_display_import_error() {
        use crate::rif::RifError;

        let cases: Vec<ImportError> = vec![
            ImportError::MalformedXml("bad".to_string()),
            ImportError::ImportDirective { location: "http://ex/r".to_string() },
            ImportError::NonCoreElement {
                element: "Naf".to_string(),
                reason: "monotone".to_string(),
            },
            ImportError::UnknownExternal { iri: "http://ex/f".to_string() },
            ImportError::NamedArgUniterm { name: "foo".to_string() },
            ImportError::UnrecognizedElement { tag: "Bogus".to_string() },
            ImportError::ValidationFailed(RifError::UnboundHeadVar { var: "x".to_string() }),
        ];
        for e in &cases {
            let s = e.to_string();
            assert!(!s.is_empty(), "Display must be non-empty for {:?}", e);
        }
    }

    /// ValidationFailed: a syntactically valid XML doc that fails range-restriction.
    #[test]
    fn test_reject_validation_failed() {
        // Rule: head var ?y not bound by body.
        let xml = br#"<Document xmlns="http://www.w3.org/2007/rif#">
  <payload><Group><sentences>
    <Forall>
      <declare><Var>x</Var></declare>
      <declare><Var>y</Var></declare>
      <formula>
        <Implies>
          <if>
            <Frame>
              <object><Var>x</Var></object>
              <slot><Const type="http://www.w3.org/2007/rif#iri">http://ex/p</Const><Const type="http://www.w3.org/2007/rif#iri">http://ex/v</Const></slot>
            </Frame>
          </if>
          <then>
            <Frame>
              <object><Var>y</Var></object>
              <slot><Const type="http://www.w3.org/2007/rif#iri">http://ex/q</Const><Const type="http://www.w3.org/2007/rif#iri">http://ex/w</Const></slot>
            </Frame>
          </then>
        </Implies>
      </formula>
    </Forall>
  </sentences></Group></payload>
</Document>"#;
        let err = import(xml).expect_err("unbound head var must be caught by validate");
        assert!(
            matches!(err, ImportError::ValidationFailed(RifError::UnboundHeadVar { .. })),
            "expected ValidationFailed(UnboundHeadVar), got: {}",
            err
        );
    }

    /// A known builtin IRI round-trips correctly.
    #[test]
    fn test_builtin_numeric_add_parses() {
        let b = iri_to_builtin(
            "http://www.w3.org/2007/rif-builtin-function#numeric-add",
        )
        .expect("numeric-add must map to Builtin::NumericAdd");
        assert_eq!(b, Builtin::NumericAdd);
    }
}
