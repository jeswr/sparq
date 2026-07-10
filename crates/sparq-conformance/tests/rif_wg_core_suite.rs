//! [FABLE-5] sq-pbz04.5.5 (epic sq-pbz04.5) — the **W3C RIF Working Group test-suite
//! conformance arm**, Core dialect, driven end-to-end through the REAL path:
//! RIF/XML `import` → `validate` → `closure` → conclusion oracle. Wired as a
//! crate-local `cargo test` + a pinned pass-count FLOOR that may only RISE
//! (registered in the central `scoreboard::SUITES` and guarded by
//! `tests/scoreboard_floors.rs`), mirroring the sibling `d-entail` lane in this crate.
//!
//! ## What this is (and what it is NOT)
//!
//! DISTINCT from the self-asserting `rif_core_suite.rs` **expressivity ratchet** (a
//! sparq-EXTENSION-labelled battery of hand-written axes over the `rif::Document`
//! model): THIS lane runs the ACTUAL **W3C RIF WG test cases** (the `Core_v1.22`
//! archive from `www.w3.org/2005/rules/test/repository/`), so it is tallied as a
//! STANDARDS-suite lane (`family = "W3C RIF"`) with an HONEST denominator — the
//! printed per-category **skip taxonomy IS the denominator's honesty**.
//!
//! We still do NOT claim the SPARQL **RIF entailment regime** (`sparql11/entailment`
//! `rif01..rif06` — a strictly larger integration through the SPARQL protocol with
//! externally-referenced rule documents); that stays a tracked, named out-of-scope
//! item exactly as `rif_core_suite.rs` frames it.
//!
//! ## Categories + the ONE subtle oracle rule (NET vacuity)
//!
//! The W3C manifest tags each test by TYPE (the archive encodes it by directory):
//! `PositiveEntailmentTest`, `NegativeEntailmentTest`, `PositiveSyntaxTest`,
//! `NegativeSyntaxTest`, `ImportRejectionTest`. The oracle:
//!
//! * **PositiveEntailmentTest** — PASS iff the premise imports+validates+closes AND
//!   the conclusion is SATISFIED by the closure. An un-importable premise is a
//!   named SKIP, never a fail (the importer's scope gap is not a wrong answer).
//! * **NegativeEntailmentTest** — the load-bearing rule: PASS **only** when the
//!   premise imported, validated AND closed successfully AND the (non-)conclusion is
//!   **not** satisfied. A premise that failed to import (unsupported construct) is a
//!   **SKIP, NEVER a pass** — otherwise every unsupported feature would launder into
//!   a vacuous "not entailed". This is the NET vacuity rule; it is enforced here and
//!   directly TESTED (`net_vacuity_unimportable_premise_is_skip_not_pass`).
//! * **PositiveSyntaxTest** — the input SHOULD import; PASS iff it imports. An input
//!   using a construct outside the importer's supported subset is a SKIP (the
//!   importer under-accepts on that construct — a completeness gap, not a wrong
//!   answer), never a fail.
//! * **NegativeSyntaxTest / ImportRejectionTest** — the input SHOULD be rejected;
//!   PASS iff the importer rejects it FOR A REASON WITHIN THE SUPPORTED SUBSET (a
//!   genuine RIF-Core violation the importer models — e.g. an unsafe/non-range-
//!   restricted rule, an Import directive). A rejection merely because the construct
//!   is unsupported is a SKIP, not a pass — the same vacuity guard as NET, applied to
//!   the syntax polarity: an unsupported-construct rejection must not launder into a
//!   spurious "correctly rejected".
//!
//! ### Named skip taxonomy (printed per run — incompleteness stays legible)
//!
//! `skip:non-core`, `skip:imports`, `skip:unsupported-builtin`,
//! `skip:condition-shape`. Every import failure maps to exactly one, derived from the
//! `ImportError` variant, so the denominator's honesty is visible each run.
//!
//! ## Conclusion matching (value-aware)
//!
//! Ground conclusions and existential-free conjunctions first. A conclusion is a bare
//! `<Frame>`/`<Atom>`/`<Member>`/`<Subclass>` (or a conjunction) that lowers to ground
//! triples; PASS iff every such triple is in the closure. NUMERIC object literals are
//! compared VALUE-AWARE via `sparq_substrate::numeric::as_numeric` (so an entailed
//! `"2.0"^^xsd:decimal` is not a false negative against a derived `"2"^^xsd:decimal`);
//! other typed literals compare lexically+datatype. A conclusion outside this ground,
//! existential-free shape is `skip:condition-shape` — never a guessed pass.
//!
//! ## The HONEST current floor
//!
//! `RIF_WG_CORE_FLOOR` is the ACTUAL MEASURED pass count of the arm against the pinned
//! `Core_v1.22` archive, not an aspirational target. It is a MEASURED **0** at the
//! current importer, for a SOUND reason in each polarity — no test currently PASSES
//! non-vacuously:
//!
//! * **Positive/Negative Entailment** — every real premise exceeds the importer's
//!   single-slot-frame subset (multi-slot frames `obj[p1->v1 p2->v2]`, positional
//!   predicate `Atom(op args…)`, `Import` directives), so it does not import →
//!   `skip:condition-shape` / `skip:imports`. Under the NET vacuity rule an
//!   un-importable premise is a SKIP, never a (vacuous) pass, so NE tests do not
//!   inflate the count.
//! * **Positive Syntax** — the inputs use positional atoms the importer under-accepts,
//!   so "should import" cannot be demonstrated → `skip:condition-shape`.
//! * **Negative Syntax / Import Rejection** — the inputs ARE rejected, but only
//!   VACUOUSLY: the safeness NegativeSyntaxTests are rejected because of an unsupported
//!   positional atom (not the range-restriction violation they target), and every
//!   ImportRejectionTest is rejected by the importer's BLANKET `<Import>` fail-close (it
//!   never dereferences the imports closure, so it rejects a VALID import identically —
//!   it cannot genuinely demonstrate detecting the SPECIFIC import invalidity). Both are
//!   named SKIPs, never passes — the import/syntax-polarity analogue of the NET rule.
//!
//! So all 46 Core+Approved tests land in the named skip buckets. That is the honest
//! denominator: the arm is fully wired, fail-closed, and RISE-READY — the floor ratchets
//! above 0 the moment the importer's Core coverage grows (tracked follow-up beads:
//! multi-slot frame import, positional-atom import; and a genuine imports-closure
//! consistency check would graduate the ImportRejection cases). A floor of 0 with a
//! legible skip taxonomy is an HONEST standards-suite result, not a hidden failure — and
//! it is far more honest than an inflated count laundered from vacuous rejections.
//!
//! ## Feature gating (both states) + fixtures
//!
//! The whole lane is behind this crate's opt-in `rif-wg-core` feature (forwards to
//! `sparq-reason/rif-xml`, i.e. the RIF/XML importer + `rif-core` model, plus
//! `sparq-substrate/numeric` for the value-aware compare). With the feature OFF this
//! file compiles to a single self-SKIP `#[test]` — no RIF/XML importer links, the
//! lean-core posture, so the default + `--workspace` builds are byte-for-byte
//! unchanged. The `Core_v1.22` archive is FETCHED (not vendored — no redistribution
//! license; see `scripts/fetch-inference-suites.sh`) into the gitignored
//! `tests/w3c/rif-core/`; when absent the runner SKIPS so a fresh offline checkout
//! stays green. `RIF_WG_CORE_FLOOR` is read TEXTUALLY by `tests/scoreboard_floors.rs`,
//! so the central scoreboard's mirrored value can never silently drift.

// [FABLE-5] When the lane feature is OFF the runner is a single self-SKIP test so the
// default + `--workspace` builds neither link the RIF/XML importer nor go red on a
// fresh checkout. (cfg gate, not a runtime branch — zero RIF code compiles in the
// default state, the lean-core invariant.)
#[cfg(not(feature = "rif-wg-core"))]
#[test]
fn rif_wg_core_suite_skipped_without_feature() {
    eprintln!(
        "SKIP: the W3C RIF WG Core conformance lane is OFF — build with \
         `--features rif-wg-core` (and run scripts/fetch-inference-suites.sh) to run it."
    );
}

#[cfg(feature = "rif-wg-core")]
mod gated {
    use sparq_core::dict::{Dict, Id};
    use sparq_reason::rif::{Atom, Document, Term};
    use sparq_reason::rif_xml::{self, ImportError};
    use std::path::PathBuf;

    /// The W3C RIF WG Core pass FLOOR (the RATCHET). The ACTUAL MEASURED pass count at
    /// the pinned `Core_v1.22` archive — NOT an aspirational target. MIRRORED in the
    /// central scoreboard (`scoreboard::SUITES`) and read TEXTUALLY by the guard test
    /// `tests/scoreboard_floors.rs`. It may only RISE — never lower it (raise as the
    /// RIF/XML importer's Core coverage grows and real W3C tests graduate from SKIP to
    /// PASS). At the current importer it is a MEASURED 0: every real W3C RIF Core file
    /// exceeds the importer's single-slot-frame subset (multi-slot frames, positional
    /// predicate atoms, Import directives, bare-conclusion roots), so all Core+Approved
    /// tests land in the named skip buckets — the honest denominator the printed skip
    /// taxonomy makes legible. A STANDARDS-suite result (family "W3C RIF"), NOT a
    /// self-asserting expressivity ratchet. [FABLE-5] sq-pbz04.5.5
    pub const RIF_WG_CORE_FLOOR: usize = 0;

    /// The pinned archive version (mirrors `scripts/fetch-inference-suites.sh`).
    const CORE_VERSION: &str = "Core_v1.22";

    // ----------------------------------------------------------------- fixtures

    fn rif_core_root() -> PathBuf {
        // CARGO_MANIFEST_DIR is crates/sparq-conformance; the archive is extracted at
        // the workspace-root `tests/w3c/rif-core/Core_v1.22/Approved/…` by
        // scripts/fetch-inference-suites.sh.
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/w3c/rif-core")
            .join(CORE_VERSION)
            .join("Approved")
    }

    // ---------------------------------------------------------------- oracle types

    /// The W3C RIF test-case types (the archive encodes each by directory name).
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum TestKind {
        PositiveEntailment,
        NegativeEntailment,
        PositiveSyntax,
        NegativeSyntax,
        ImportRejection,
    }

    impl TestKind {
        fn dir(self) -> &'static str {
            match self {
                TestKind::PositiveEntailment => "PositiveEntailmentTest",
                TestKind::NegativeEntailment => "NegativeEntailmentTest",
                TestKind::PositiveSyntax => "PositiveSyntaxTest",
                TestKind::NegativeSyntax => "NegativeSyntaxTest",
                TestKind::ImportRejection => "ImportRejectionTest",
            }
        }
        const ALL: &'static [TestKind] = &[
            TestKind::PositiveEntailment,
            TestKind::NegativeEntailment,
            TestKind::PositiveSyntax,
            TestKind::NegativeSyntax,
            TestKind::ImportRejection,
        ];
    }

    /// The per-test verdict. `Skip` carries the NAMED taxonomy bucket; a `Fail` is a
    /// genuine wrong answer within the supported subset (which reds the lane).
    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Outcome {
        Pass,
        /// A named skip — the importer under-accepts / under-models this construct, or
        /// the conclusion is out of the ground existential-free shape. NEVER a pass.
        Skip(&'static str),
        /// A genuine wrong answer on the supported subset (reds the lane).
        Fail(String),
    }

    /// Map an `ImportError` to the named skip taxonomy bucket (the denominator's
    /// honesty). Every un-importable premise/input lands in exactly one bucket, printed
    /// per run.
    fn skip_bucket(e: &ImportError) -> &'static str {
        match e {
            ImportError::ImportDirective { .. } => "skip:imports",
            ImportError::NonCoreElement { .. } => "skip:non-core",
            ImportError::UnknownExternal { .. } => "skip:unsupported-builtin",
            // A named-arg uniterm, an unrecognized element (bare positional `<Atom>`, an
            // out-of-subset construct), a malformed shape (multi-slot frame), or a
            // validation failure that is NOT a modelled Core violation are all "the
            // condition/rule shape is outside what the importer models" — the
            // condition-shape bucket.
            ImportError::NamedArgUniterm { .. }
            | ImportError::UnrecognizedElement { .. }
            | ImportError::MalformedXml(_)
            | ImportError::ValidationFailed(_) => "skip:condition-shape",
        }
    }

    // ------------------------------------------------------- entity pre-resolution

    /// Resolve the DTD **internal-subset** `<!ENTITY name "value">` declarations that
    /// every W3C RIF/XML test file uses (`&rif;`, `&xs;`, `&rdf;`), then strip the
    /// `<!DOCTYPE …>` block. This is a SOUND, self-contained pre-pass — the entities
    /// are declared IN the document's own internal subset (no external DTD, no network),
    /// so substituting them is a faithful part of "reading the W3C RIF/XML file". The
    /// `rif_xml::import` entity resolver only handles the five predefined XML entities
    /// (it fail-closes on any other general entity), so without this pre-pass every
    /// real W3C file would be rejected at the entity step before its structure is ever
    /// seen — masking the true (structural) skip taxonomy. std-only string work.
    fn resolve_internal_entities(src: &str) -> String {
        let mut entities: Vec<(String, String)> = Vec::new();
        for chunk in src.split("<!ENTITY").skip(1) {
            let decl_end = chunk.find('>').unwrap_or(chunk.len());
            let decl = chunk[..decl_end].trim();
            let mut it = decl.splitn(2, char::is_whitespace);
            let name = it.next().unwrap_or("").trim().to_string();
            let value = it
                .next()
                .unwrap_or("")
                .trim()
                .trim_matches(|c| c == '"' || c == '\'')
                .to_string();
            if !name.is_empty() {
                entities.push((name, value));
            }
        }
        let mut out = src.to_string();
        // Strip the whole DOCTYPE internal-subset block (or a bare `<!DOCTYPE …>`).
        if let Some(start) = out.find("<!DOCTYPE") {
            if let Some(rel) = out[start..].find("]>") {
                out.replace_range(start..start + rel + 2, "");
            } else if let Some(rel) = out[start..].find('>') {
                out.replace_range(start..start + rel + 1, "");
            }
        }
        for (name, value) in &entities {
            out = out.replace(&format!("&{};", name), value);
        }
        out
    }

    /// Read a `.rif` file and pre-resolve its internal-subset entities. `None` if the
    /// file is absent (some tests have no such document, e.g. a NegativeSyntaxTest has
    /// only an `-input.rif`).
    fn read_rif(dir: &std::path::Path, name: &str) -> Option<String> {
        let raw = std::fs::read_to_string(dir.join(name)).ok()?;
        Some(resolve_internal_entities(&raw))
    }

    /// Wrap a bare RIF condition/atom element (a W3C conclusion / non-conclusion file is
    /// a bare `<Frame>`/`<Atom>`/`<Member>`/`<Subclass>` root, NOT a `<Document>`) in a
    /// synthetic `<Document>` so `rif_xml::import` — whose entry point requires a
    /// `<Document>` root — can parse it as a single ground FACT. This is a faithful,
    /// SOUND lowering: a bare conclusion condition is a universally-closed ground fact,
    /// which is exactly a one-atom, empty-body rule. The wrap only supplies the standard
    /// `Document/payload/Group/sentence` scaffold RIF/XML requires; it adds no rule
    /// content. A bare positional `<Atom>` still fails to import (the importer's
    /// positional-atom gap) → an honest `skip:condition-shape`, never a guessed pass.
    ///
    /// The bare element already carries the RIF default namespace (`xmlns=…rif#`) which
    /// the wrapping `<Document>` re-declares identically, so the child inherits it (and
    /// any redundant re-declaration on the child is inert). Only the OUTER XML preamble
    /// (`<?xml …?>`, already entity-resolved and DOCTYPE-stripped) is dropped; the root
    /// element and everything under it are preserved verbatim.
    fn wrap_conclusion(src: &str) -> String {
        // `read_rif` has already DOCTYPE-stripped; strip a leading `<?xml …?>` prolog too
        // (only when it IS at the very start — a `?>` mid-document must not be cut). The
        // remaining root element (a bare `<Frame>`/`<Atom>`/`<Member>`/`<Subclass>`) is
        // preserved verbatim and re-parented under the standard scaffold as a ground fact.
        let body = if src.trim_start().starts_with("<?xml") {
            match src.find("?>") {
                Some(pos) => &src[pos + 2..],
                None => src,
            }
        } else {
            src
        };
        format!(
            "<Document xmlns=\"http://www.w3.org/2007/rif#\">\
             <payload><Group><sentence>{}</sentence></Group></payload></Document>",
            body.trim()
        )
    }

    /// The `<id>-<suffix>.rif` file name for a test whose directory is `id`.
    fn rif_name(id: &str, suffix: &str) -> String {
        format!("{}-{}.rif", id, suffix)
    }

    // -------------------------------------------------------- conclusion oracle

    /// Lower a ground (existential-free) conclusion `Document` to its object triples,
    /// or `None` if it is outside the ground, existential-free shape (→
    /// `skip:condition-shape`). A conclusion imports as a `rif::Document` whose rules
    /// are all facts (empty body) with ground positive atoms.
    fn conclusion_ground_triples(doc: &Document) -> Option<Vec<Atom>> {
        let mut atoms = Vec::new();
        for rule in &doc.rules {
            // A conclusion is a conjunction of facts: every rule must be a ground fact
            // (no body, no variables, a positive Frame/Member/Subclass atom).
            if !rule.body.is_empty() {
                return None;
            }
            for head in &rule.head {
                match head {
                    Atom::Frame { .. } | Atom::Member { .. } | Atom::Subclass { .. } => {
                        // Every term must be an IRI or a typed literal — exactly what the
                        // triple encoder (`term_to_id`) and the closure satisfier can
                        // decide. A `Var` (non-ground / existential) OR a `List` term is
                        // OUT OF SHAPE → skip. This is LOAD-BEARING against a NET
                        // false-pass: `term_to_id` returns `None` for a `List`, so a
                        // ground-but-list-bearing non-conclusion would be
                        // `closure_satisfies == false` by UN-ENCODABILITY (not genuine
                        // non-entailment) → a vacuous NET "not entailed" Pass. Requiring
                        // encodable terms here makes the satisfier's verdict a genuine
                        // entailment decision, never an artefact of what the oracle
                        // cannot represent. (sq-pbz04.5.5 review Finding 1.)
                        if !atom_is_encodable_ground(head) {
                            return None;
                        }
                        atoms.push(head.clone());
                    }
                    // Equal / Builtin in a conclusion is out of the ground-triple shape.
                    Atom::Equal { .. } | Atom::Builtin { .. } => return None,
                }
            }
        }
        if atoms.is_empty() {
            None
        } else {
            Some(atoms)
        }
    }

    /// Is `t` an IRI or a typed literal — i.e. a term `term_to_id` can encode to a dict
    /// `Id` and `closure_satisfies` can decide? A `Var` or a `List` is NOT encodable.
    fn term_is_encodable(t: &Term) -> bool {
        matches!(t, Term::Iri(_) | Term::Lit { .. })
    }

    /// Every term of a positive conclusion atom is an encodable ground term (no `Var`, no
    /// `List`). This is stronger than "has no variable" — it also excludes `List` terms
    /// the satisfier cannot represent (see `conclusion_ground_triples`).
    fn atom_is_encodable_ground(a: &Atom) -> bool {
        match a {
            Atom::Frame { obj, pred, val } => {
                term_is_encodable(obj) && term_is_encodable(pred) && term_is_encodable(val)
            }
            Atom::Member { obj, class } => term_is_encodable(obj) && term_is_encodable(class),
            Atom::Subclass { sub, sup } => term_is_encodable(sub) && term_is_encodable(sup),
            // Equal / Builtin are rejected upstream; conservatively not encodable here.
            Atom::Equal { .. } | Atom::Builtin { .. } => false,
        }
    }

    const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
    const RDFS_SUBCLASS_OF: &str = "http://www.w3.org/2000/01/rdf-schema#subClassOf";

    /// The three (s, p, o) `Term`s a ground positive conclusion atom lowers to.
    fn atom_to_triple_terms(a: &Atom) -> Option<[Term; 3]> {
        match a {
            Atom::Frame { obj, pred, val } => Some([obj.clone(), pred.clone(), val.clone()]),
            Atom::Member { obj, class } => Some([
                obj.clone(),
                Term::Iri(RDF_TYPE.to_string()),
                class.clone(),
            ]),
            Atom::Subclass { sub, sup } => Some([
                sub.clone(),
                Term::Iri(RDFS_SUBCLASS_OF.to_string()),
                sup.clone(),
            ]),
            _ => None,
        }
    }

    /// Encode a RIF `Term` (constant only — vars are rejected upstream) to a dict `Id`.
    fn term_to_id(dict: &mut Dict, t: &Term) -> Option<Id> {
        use oxrdf::{Literal, NamedNode, Term as OxTerm};
        match t {
            Term::Iri(iri) => Some(dict.intern(&OxTerm::NamedNode(NamedNode::new_unchecked(
                iri.clone(),
            )))),
            Term::Lit { lex, datatype } => {
                let lit =
                    Literal::new_typed_literal(lex.clone(), NamedNode::new_unchecked(datatype.clone()));
                Some(dict.intern(&OxTerm::Literal(lit)))
            }
            // Lists / vars are outside the ground-triple conclusion shape.
            Term::Var(_) | Term::List(_) => None,
        }
    }

    /// Is the object literal a numeric that must be compared VALUE-AWARE?
    fn numeric_value(t: &Term) -> Option<sparq_substrate::numeric::Num> {
        use oxrdf::{Literal, NamedNode};
        if let Term::Lit { lex, datatype } = t {
            let lit =
                Literal::new_typed_literal(lex.clone(), NamedNode::new_unchecked(datatype.clone()));
            sparq_substrate::numeric::as_numeric(&lit)
        } else {
            None
        }
    }

    /// Does the closure SATISFY a ground conclusion triple (s, p, o)? The subject and
    /// predicate compare by exact dict identity; the OBJECT compares VALUE-AWARE when it
    /// is numeric (so `"2.0"^^xsd:decimal` ≡ a derived `"2"^^xsd:decimal`), else by
    /// exact dict identity (lexical + datatype).
    fn closure_satisfies(dict: &mut Dict, closure: &[[Id; 3]], triple: &[Term; 3]) -> bool {
        let sid = match term_to_id(dict, &triple[0]) {
            Some(i) => i,
            None => return false,
        };
        let pid = match term_to_id(dict, &triple[1]) {
            Some(i) => i,
            None => return false,
        };
        let want_num = numeric_value(&triple[2]);
        let oid = term_to_id(dict, &triple[2]);
        for row in closure {
            if row[0] != sid || row[1] != pid {
                continue;
            }
            // Exact object identity (covers non-numeric typed literals + IRIs).
            if let Some(oid) = oid {
                if row[2] == oid {
                    return true;
                }
            }
            // Value-aware numeric fallback: the closure's object, if numeric, must equal
            // the conclusion object's numeric value (no lexical false negative). RESTRICTED
            // to the EXACT-RATIONAL tiers (`xsd:integer` / `xsd:decimal`, incl. the integer
            // sub-types) via `Num::to_dec()`: there, value-equality across datatypes is
            // unambiguous (`"2"^^integer` ≡ `"2.0"^^decimal`). A float/double object is NOT
            // value-matched cross-type — `xsd:float`/`xsd:double` have their own distinct
            // primitive value spaces, so cross-primitive rational identification could
            // over-match a conclusion that is not genuinely entailed; those require the
            // exact-identity match above. (sq-pbz04.5.5 review NIT 4.)
            if let Some(want) = want_num {
                if let (Some(want_dec), oxrdf::Term::Literal(l)) = (want.to_dec(), dict.term(row[2]))
                {
                    if let Some(got_dec) = sparq_substrate::numeric::as_numeric(&l).and_then(|n| n.to_dec()) {
                        if got_dec.cmp(want_dec) == Some(std::cmp::Ordering::Equal) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Does the closure entail EVERY ground triple of a ground existential-free
    /// conclusion? (`None` conclusion shape is decided by the caller, not here.)
    fn closure_entails_conclusion(dict: &mut Dict, closure: &[[Id; 3]], atoms: &[Atom]) -> bool {
        atoms.iter().all(|a| {
            atom_to_triple_terms(a)
                .map(|t| closure_satisfies(dict, closure, &t))
                .unwrap_or(false)
        })
    }

    // ----------------------------------------------------------- per-test drivers

    /// Import a premise/input document, returning the validated `Document` OR the named
    /// skip bucket for its `ImportError`.
    fn import_or_skip(src: &str) -> Result<Document, &'static str> {
        rif_xml::import(src.as_bytes()).map_err(|e| skip_bucket(&e))
    }

    /// Drive a (Positive|Negative)EntailmentTest. The NET vacuity rule lives here: an
    /// un-importable NET premise is a SKIP, never a (vacuous) pass.
    fn run_entailment(kind: TestKind, dir: &std::path::Path, id: &str) -> Outcome {
        let premise_src = match read_rif(dir, &rif_name(id, "premise")) {
            Some(s) => s,
            None => return Outcome::Skip("skip:condition-shape"),
        };
        // import → validate (import() validates internally) → closure.
        let doc = match import_or_skip(&premise_src) {
            Ok(d) => d,
            // NET VACUITY: an un-importable premise NEVER passes — in EITHER polarity.
            Err(bucket) => return Outcome::Skip(bucket),
        };
        let mut dict = Dict::default();
        let closure = match doc.closure(&mut dict) {
            Ok(c) => c,
            // A closure error on a validated premise is a genuine wrong-path signal, but
            // for an out-of-subset construct that slipped validation it is still a
            // shape skip, never a pass. Treat as a named skip (fail-closed).
            Err(_) => return Outcome::Skip("skip:condition-shape"),
        };

        let concl_suffix = match kind {
            TestKind::PositiveEntailment => "conclusion",
            TestKind::NegativeEntailment => "nonconclusion",
            _ => unreachable!("run_entailment only for entailment kinds"),
        };
        let concl_src = match read_rif(dir, &rif_name(id, concl_suffix)) {
            Some(s) => s,
            None => return Outcome::Skip("skip:condition-shape"),
        };
        // A W3C conclusion file is a BARE `<Frame>`/`<Atom>`/… root (not a `<Document>`);
        // wrap it in the standard scaffold so the importer parses it as a ground fact.
        // If it is STILL un-importable (a positional `<Atom>` root, a multi-slot frame)
        // the conclusion is outside the ground-triple shape we can decide →
        // condition-shape skip (never a guessed pass).
        let concl_doc = match import_or_skip(&wrap_conclusion(&concl_src)) {
            Ok(d) => d,
            Err(_) => return Outcome::Skip("skip:condition-shape"),
        };
        let atoms = match conclusion_ground_triples(&concl_doc) {
            Some(a) => a,
            None => return Outcome::Skip("skip:condition-shape"),
        };
        let entailed = closure_entails_conclusion(&mut dict, &closure, &atoms);
        match kind {
            TestKind::PositiveEntailment => {
                if entailed {
                    Outcome::Pass
                } else {
                    Outcome::Fail(format!("{}: conclusion NOT entailed by closure", id))
                }
            }
            TestKind::NegativeEntailment => {
                // PASS iff the premise imported+validated+closed (it did — we are here)
                // AND the non-conclusion is NOT satisfied.
                if entailed {
                    Outcome::Fail(format!("{}: non-conclusion WAS entailed by closure", id))
                } else {
                    Outcome::Pass
                }
            }
            _ => unreachable!(),
        }
    }

    /// Drive a PositiveSyntaxTest: the input SHOULD import. PASS iff it imports; an
    /// out-of-subset construct is a SKIP (importer completeness gap, not a wrong
    /// answer).
    fn run_positive_syntax(dir: &std::path::Path, id: &str) -> Outcome {
        let src = match read_rif(dir, &rif_name(id, "input")) {
            Some(s) => s,
            None => return Outcome::Skip("skip:condition-shape"),
        };
        match import_or_skip(&src) {
            Ok(_) => Outcome::Pass,
            Err(bucket) => Outcome::Skip(bucket),
        }
    }

    /// Drive a NegativeSyntaxTest / ImportRejectionTest: the input SHOULD be rejected.
    /// PASS iff the importer rejects it because it GENUINELY DETECTED the specific Core
    /// violation the test targets — NOT merely because the input happens to use a
    /// construct the importer does not support (or an import it blanket-refuses). This
    /// is the syntax/import-polarity analogue of the NET vacuity rule.
    ///
    /// The genuinely-modelled, non-vacuous rejections are a `ValidationFailed` carrying a
    /// RIF-Core SAFETY / syntactic violation the checker DETECTS: `UnboundHeadVar`,
    /// `UnboundBuiltinInput`, `BadBuiltinArity`, `BuiltinInHead`, `EqualInConclusion`
    /// (`is_genuine_core_violation`). A `NegativeSyntaxTest` (e.g. `Core_NonSafeness`)
    /// targets exactly a range-restriction violation, so detecting it is a genuine pass.
    /// Every OTHER rejection is VACUOUS and is a named SKIP, never a pass:
    ///
    /// * `UnrecognizedElement` / `UnknownExternal` / `NamedArgUniterm` — rejected only
    ///   because the construct (a positional `<Atom>`, an unsupported builtin, …) is
    ///   outside the importer's supported subset; it would reject a VALID such document
    ///   too, so the rejection does not demonstrate detecting the targeted violation.
    /// * `ImportDirective` (an `ImportRejectionTest`) — the importer fail-closes on ANY
    ///   `<Import>` WITHOUT dereferencing or checking the imports closure, so it rejects
    ///   a VALID import identically. It therefore cannot genuinely demonstrate detecting
    ///   the SPECIFIC import invalidity the test targets (a multi-context constant, an
    ///   invalid DL/RDF combination, …). Counting it would launder the blanket
    ///   import-refusal into a spurious "correctly rejected" — the exact vacuity the NET
    ///   rule forbids. → `skip:imports`.
    /// * `ValidationFailed(DistinctGroundEqual | Nonmonotonic)` — these are NOT
    ///   violation-detections but UNSUPPORTED-CAPABILITY fail-closes: `DistinctGroundEqual`
    ///   rejects a *VALID* RIF-Core document (a ground body `Equal` like
    ///   `"1"^^integer = "1.0"^^decimal` is legal Core) pending the value-space comparator
    ///   (sq-v5evr/#1646); `Nonmonotonic` is a defensive internal-invariant guard. Neither
    ///   demonstrates detecting the targeted violation — treating them as a pass would
    ///   launder an unsupported-capability rejection, exactly the vacuity this guard
    ///   forbids (sq-pbz04.5.5 review Finding 2). → the named condition-shape skip.
    fn run_negative_syntax(kind: TestKind, dir: &std::path::Path, id: &str) -> Outcome {
        let src = match read_rif(dir, &rif_name(id, "input")) {
            Some(s) => s,
            None => return Outcome::Skip("skip:condition-shape"),
        };
        match rif_xml::import(src.as_bytes()) {
            // The importer ACCEPTED an input that SHOULD be rejected → genuine wrong
            // answer on the supported subset (reds the lane).
            Ok(_) => Outcome::Fail(format!("{}: importer ACCEPTED a should-reject input", id)),
            Err(e) => {
                // A genuine (non-vacuous) rejection = a ValidationFailed carrying a Core
                // violation the checker DETECTS (safety / syntactic) — NOT an
                // unsupported-capability fail-close and NOT an unsupported-construct
                // rejection. Applies to BOTH reject kinds: an ImportRejectionTest whose
                // payload ALSO violates safety would be a genuine detection; a blanket
                // ImportDirective refusal is not (see doc).
                let genuinely_modelled = match &e {
                    ImportError::ValidationFailed(re) => is_genuine_core_violation(re),
                    _ => false,
                };
                let _ = kind; // both reject kinds share the same non-vacuous criterion
                if genuinely_modelled {
                    Outcome::Pass
                } else {
                    // Rejected only because the construct is unsupported / imports are
                    // blanket-refused / a capability is unsupported — VACUOUS. Bucket by
                    // the actual error so the taxonomy stays honest.
                    Outcome::Skip(skip_bucket(&e))
                }
            }
        }
    }

    /// Is `re` a RIF-Core violation the checker GENUINELY DETECTS (a safety /
    /// range-restriction / syntactic-restriction violation) — as opposed to an
    /// UNSUPPORTED-CAPABILITY fail-close (`DistinctGroundEqual`, `Nonmonotonic`) that
    /// rejects a document the checker simply cannot yet handle? Only the former is a
    /// non-vacuous "correctly rejected" for a NegativeSyntax / ImportRejection test.
    /// EXPLICIT (not a catch-all) so a NEW `RifError` variant cannot silently be treated
    /// as a genuine detection. (sq-pbz04.5.5 review Finding 2.)
    fn is_genuine_core_violation(re: &sparq_reason::rif::RifError) -> bool {
        use sparq_reason::rif::RifError::*;
        match re {
            // Genuine detections of a modelled Core violation.
            UnboundHeadVar { .. }
            | UnboundBuiltinInput { .. }
            | BadBuiltinArity { .. }
            | BuiltinInHead { .. }
            | EqualInConclusion => true,
            // Unsupported-capability fail-closes — NOT a detection of the targeted
            // violation (they reject VALID Core / are defensive guards). Vacuous → skip.
            DistinctGroundEqual { .. } | Nonmonotonic { .. } => false,
        }
    }

    /// Enumerate every `Approved/<TestType>/<id>/` directory for a kind.
    fn tests_of_kind(kind: TestKind) -> Vec<(String, PathBuf)> {
        let base = rif_core_root().join(kind.dir());
        let mut out = Vec::new();
        if let Ok(rd) = std::fs::read_dir(&base) {
            for entry in rd.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    if let Some(id) = p.file_name().and_then(|s| s.to_str()) {
                        out.push((id.to_string(), p.clone()));
                    }
                }
            }
        }
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    fn run_one(kind: TestKind, dir: &std::path::Path, id: &str) -> Outcome {
        match kind {
            TestKind::PositiveEntailment | TestKind::NegativeEntailment => {
                run_entailment(kind, dir, id)
            }
            TestKind::PositiveSyntax => run_positive_syntax(dir, id),
            TestKind::NegativeSyntax | TestKind::ImportRejection => {
                run_negative_syntax(kind, dir, id)
            }
        }
    }

    // ----------------------------------------------------------------- the lane

    #[test]
    fn rif_wg_core_ratchet() {
        let root = rif_core_root();
        if !root.exists() {
            eprintln!(
                "SKIP: W3C RIF WG Core archive not present under {} — run \
                 scripts/fetch-inference-suites.sh",
                root.display()
            );
            return;
        }

        let mut pass = 0usize;
        let mut fails: Vec<String> = Vec::new();
        // Named skip taxonomy tally (the denominator's honesty).
        let mut skips: std::collections::BTreeMap<&'static str, usize> =
            std::collections::BTreeMap::new();
        let mut total = 0usize;

        for &kind in TestKind::ALL {
            for (id, dir) in tests_of_kind(kind) {
                total += 1;
                match run_one(kind, &dir, &id) {
                    Outcome::Pass => pass += 1,
                    Outcome::Skip(bucket) => *skips.entry(bucket).or_default() += 1,
                    Outcome::Fail(why) => fails.push(why),
                }
            }
        }

        if total == 0 {
            eprintln!(
                "SKIP: W3C RIF WG Core archive present at {} but empty — re-run \
                 scripts/fetch-inference-suites.sh",
                root.display()
            );
            return;
        }

        // [FABLE-5] The CI ratchet greps this exact `TOTAL rif-wg-core` line (pass count
        // in field $3), mirroring the D-entailment lane's `TOTAL <cat>` shape. Positional
        // `println!` args (not inline `{pass}`) per the CodeQL `rust/unused-variable`
        // false-positive guard in the shared agent contract.
        println!("\nW3C RIF WG Core conformance (pinned {} archive)", CORE_VERSION);
        let skip_total: usize = skips.values().sum();
        println!(
            "TOTAL rif-wg-core {} {} {} (floor {})",
            pass,
            fails.len(),
            skip_total,
            RIF_WG_CORE_FLOOR
        );
        println!("skip taxonomy (the honest denominator, {} of {} tests):", skip_total, total);
        for (bucket, n) in &skips {
            println!("  {} {}", n, bucket);
        }
        if !fails.is_empty() {
            println!("RIF WG Core FAILS (genuine wrong answers on the supported subset):");
            for f in &fails {
                println!("  - {}", f);
            }
        }

        assert!(
            fails.is_empty(),
            "RIF WG Core lane has genuine wrong answers on the supported subset: {:?}",
            fails
        );
        // [FABLE-5] `pass >= RIF_WG_CORE_FLOOR` is the RATCHET check (the floor may only
        // RISE). At the current MEASURED floor of 0 the comparison is trivially true (0
        // is the `usize` minimum — clippy's `absurd_extreme_comparisons`), but the check
        // is SEMANTICALLY the ratchet and MUST stay a `>=` so raising the floor above 0
        // (as the importer's Core coverage grows and real tests graduate SKIP→PASS)
        // immediately enforces the new floor without a code edit here. The `#[allow]` is
        // scoped to this one statement; the moment the floor is raised the lint no longer
        // fires and the check bites for real.
        #[allow(clippy::absurd_extreme_comparisons)]
        {
            assert!(
                pass >= RIF_WG_CORE_FLOOR,
                "RIF WG Core pass count {} regressed below the ratchet floor {}",
                pass,
                RIF_WG_CORE_FLOOR
            );
        }
    }

    // ------------------------------------------------------------- oracle unit tests
    //
    // These pin the load-bearing oracle rules WITHOUT the fetched fixtures (so they run
    // offline, feature-ON), driving the SAME real functions the lane uses.

    /// NET VACUITY (the one subtle oracle rule): an un-importable premise yields a SKIP,
    /// NEVER a pass — otherwise an unsupported construct laundered into a vacuous
    /// "not entailed". This drives the REAL `run_entailment` over a synthetic test
    /// directory whose premise is a genuinely-unsupported RIF/XML construct (an `Import`
    /// directive) and asserts the outcome is a Skip bucket, not Pass.
    #[test]
    fn net_vacuity_unimportable_premise_is_skip_not_pass() {
        let tmp = std::env::temp_dir().join(format!(
            "rif_wg_net_vacuity_{}_{}",
            std::process::id(),
            "neg"
        ));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        // A premise the importer genuinely CANNOT import: an Import directive
        // (fail-closed → ImportError::ImportDirective → skip:imports). This is exactly
        // the vacuity trap — with no NET guard the "closure" would be empty and the
        // non-conclusion trivially not-entailed → a FALSE pass.
        let premise = r#"<Document xmlns="http://www.w3.org/2007/rif#">
  <directive><Import><location>http://example.org/ext</location></Import></directive>
  <payload><Group></Group></payload>
</Document>"#;
        // A well-formed ground non-conclusion (would be "not entailed" against an empty
        // closure — the vacuous pass we must PREVENT).
        let nonconcl = r#"<Frame xmlns="http://www.w3.org/2007/rif#">
  <object><Const type="http://www.w3.org/2007/rif#iri">http://example.org/a</Const></object>
  <slot><Const type="http://www.w3.org/2007/rif#iri">http://example.org/p</Const><Const type="http://www.w3.org/2007/rif#iri">http://example.org/b</Const></slot>
</Frame>"#;
        std::fs::write(tmp.join(rif_name("neg", "premise")), premise).unwrap();
        std::fs::write(tmp.join(rif_name("neg", "nonconclusion")), nonconcl).unwrap();

        let outcome = run_entailment(TestKind::NegativeEntailment, &tmp, "neg");
        let _ = std::fs::remove_dir_all(&tmp);

        // MUST be a skip (never a pass) — the NET vacuity rule.
        assert!(
            matches!(outcome, Outcome::Skip(_)),
            "NET with an un-importable premise MUST be a SKIP (vacuity rule), got {:?}",
            outcome
        );
        assert_ne!(
            outcome,
            Outcome::Pass,
            "an un-importable NET premise must NEVER be a (vacuous) pass"
        );
    }

    /// Mutation check for the NET vacuity guard: a NET whose premise DOES import but
    /// whose non-conclusion IS entailed must FAIL (a real regression), and one whose
    /// non-conclusion is NOT entailed must PASS — so the guard is non-vacuous (it
    /// distinguishes a genuine not-entailed from a genuine entailed).
    #[test]
    fn net_genuine_entailed_nonconclusion_fails() {
        let tmp = std::env::temp_dir().join(format!("rif_wg_net_genuine_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        // An IMPORTABLE premise: a single-slot ground frame fact (in the supported
        // subset), plus a rule deriving a second frame.
        let premise = r#"<Document xmlns="http://www.w3.org/2007/rif#">
  <payload><Group>
    <sentence><Frame>
      <object><Const type="http://www.w3.org/2007/rif#iri">http://example.org/a</Const></object>
      <slot><Const type="http://www.w3.org/2007/rif#iri">http://example.org/p</Const><Const type="http://www.w3.org/2007/rif#iri">http://example.org/b</Const></slot>
    </Frame></sentence>
  </Group></payload>
</Document>"#;
        // A non-conclusion that IS in the premise → entailed → a NET here must FAIL.
        let nonconcl = r#"<Frame xmlns="http://www.w3.org/2007/rif#">
  <object><Const type="http://www.w3.org/2007/rif#iri">http://example.org/a</Const></object>
  <slot><Const type="http://www.w3.org/2007/rif#iri">http://example.org/p</Const><Const type="http://www.w3.org/2007/rif#iri">http://example.org/b</Const></slot>
</Frame>"#;
        std::fs::write(tmp.join(rif_name("g", "premise")), premise).unwrap();
        std::fs::write(tmp.join(rif_name("g", "nonconclusion")), nonconcl).unwrap();
        let entailed_outcome = run_entailment(TestKind::NegativeEntailment, &tmp, "g");
        // Same premise, a non-conclusion NOT entailed → PASS.
        let nonconcl_absent = r#"<Frame xmlns="http://www.w3.org/2007/rif#">
  <object><Const type="http://www.w3.org/2007/rif#iri">http://example.org/a</Const></object>
  <slot><Const type="http://www.w3.org/2007/rif#iri">http://example.org/q</Const><Const type="http://www.w3.org/2007/rif#iri">http://example.org/z</Const></slot>
</Frame>"#;
        std::fs::write(tmp.join(rif_name("g", "nonconclusion")), nonconcl_absent).unwrap();
        let absent_outcome = run_entailment(TestKind::NegativeEntailment, &tmp, "g");
        let _ = std::fs::remove_dir_all(&tmp);

        // The guard is NON-VACUOUS: it distinguishes entailed (Fail) from not-entailed (Pass).
        assert!(
            matches!(entailed_outcome, Outcome::Fail(_)),
            "a NET whose non-conclusion IS entailed must FAIL, got {:?}",
            entailed_outcome
        );
        assert_eq!(
            absent_outcome,
            Outcome::Pass,
            "a NET whose non-conclusion is NOT entailed (premise imported+closed) must PASS"
        );
    }

    /// A PositiveEntailmentTest whose premise imports and whose conclusion is entailed
    /// PASSES — the positive end-to-end path over the REAL import→closure→oracle,
    /// including the VALUE-AWARE numeric object comparison (`"2"^^integer` in the closure
    /// satisfies a `"2.0"^^decimal` conclusion object).
    #[test]
    fn positive_entailment_value_aware_numeric_conclusion_passes() {
        let tmp = std::env::temp_dir().join(format!("rif_wg_pe_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        // Premise: a ground frame fact whose object is "2"^^xsd:integer.
        let premise = r#"<Document xmlns="http://www.w3.org/2007/rif#">
  <payload><Group>
    <sentence><Frame>
      <object><Const type="http://www.w3.org/2007/rif#iri">http://example.org/a</Const></object>
      <slot><Const type="http://www.w3.org/2007/rif#iri">http://example.org/n</Const><Const type="http://www.w3.org/2001/XMLSchema#integer">2</Const></slot>
    </Frame></sentence>
  </Group></payload>
</Document>"#;
        // Conclusion: a BARE `<Frame>` root (the real W3C conclusion-file shape — NOT a
        // `<Document>`), the SAME frame but object "2.0"^^xsd:decimal — a DIFFERENT
        // lexical form + datatype, value-equal. This exercises `wrap_conclusion` (bare
        // root → single ground fact). A lexical comparator would false-negative; the
        // value-aware numeric compare must PASS.
        let conclusion = r#"<Frame xmlns="http://www.w3.org/2007/rif#">
  <object><Const type="http://www.w3.org/2007/rif#iri">http://example.org/a</Const></object>
  <slot><Const type="http://www.w3.org/2007/rif#iri">http://example.org/n</Const><Const type="http://www.w3.org/2001/XMLSchema#decimal">2.0</Const></slot>
</Frame>"#;
        std::fs::write(tmp.join(rif_name("pe", "premise")), premise).unwrap();
        std::fs::write(tmp.join(rif_name("pe", "conclusion")), conclusion).unwrap();
        let outcome = run_entailment(TestKind::PositiveEntailment, &tmp, "pe");
        let _ = std::fs::remove_dir_all(&tmp);
        assert_eq!(
            outcome,
            Outcome::Pass,
            "a value-equal numeric conclusion (\"2.0\"^^decimal vs derived \"2\"^^integer) must PASS"
        );
    }

    /// The named skip taxonomy maps each `ImportError` variant to its documented bucket
    /// (the printed denominator's honesty). Pins the mapping so a new variant cannot
    /// silently fall into the wrong bucket.
    #[test]
    fn skip_taxonomy_buckets_are_named() {
        use sparq_reason::rif::RifError;
        assert_eq!(
            skip_bucket(&ImportError::ImportDirective { location: String::new() }),
            "skip:imports"
        );
        assert_eq!(
            skip_bucket(&ImportError::NonCoreElement {
                element: "x".into(),
                reason: "y".into()
            }),
            "skip:non-core"
        );
        assert_eq!(
            skip_bucket(&ImportError::UnknownExternal { iri: "z".into() }),
            "skip:unsupported-builtin"
        );
        assert_eq!(
            skip_bucket(&ImportError::UnrecognizedElement { tag: "Atom".into() }),
            "skip:condition-shape"
        );
        assert_eq!(
            skip_bucket(&ImportError::MalformedXml("dup slot".into())),
            "skip:condition-shape"
        );
        // A validation failure (a modelled Core violation) buckets as condition-shape
        // when it reaches the skip path (the reject-kind drivers treat it as a modelled
        // pass instead — see run_negative_syntax).
        assert_eq!(
            skip_bucket(&ImportError::ValidationFailed(RifError::UnboundHeadVar {
                var: "x".into()
            })),
            "skip:condition-shape"
        );
    }

    /// The DTD internal-subset entity pre-resolution is LOAD-BEARING: a real W3C RIF/XML
    /// file (with the `<?xml?>` prolog, the `<!DOCTYPE … [ <!ENTITY rif …> … ]>` internal
    /// subset, and `&rif;`/`&xs;` entity references) is un-importable RAW — the importer
    /// fail-closes on the unrecognised `rif` general entity — and importable ONLY after
    /// `resolve_internal_entities`. This drives a full real-file-shaped premise +
    /// conclusion (single-slot frame, in the importer's supported subset) end-to-end and
    /// asserts (a) the raw premise is un-importable, and (b) after resolution the whole
    /// PositiveEntailment PASSES. So the pre-pass is a genuine mechanism, not a no-op.
    #[test]
    fn entity_pre_resolution_is_load_bearing_end_to_end() {
        // Exactly the real-file shape: prolog + DOCTYPE internal subset + &entity; refs.
        let raw_premise = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE Document [
  <!ENTITY rif  "http://www.w3.org/2007/rif#">
  <!ENTITY xs   "http://www.w3.org/2001/XMLSchema#">
]>
<Document xmlns="&rif;">
  <payload><Group>
    <sentence><Frame>
      <object><Const type="&rif;iri">http://example.org/a</Const></object>
      <slot><Const type="&rif;iri">http://example.org/p</Const><Const type="&xs;integer">7</Const></slot>
    </Frame></sentence>
  </Group></payload>
</Document>"#;
        // RAW (un-resolved) must be un-importable — the importer fail-closes on `&rif;`.
        assert!(
            rif_xml::import(raw_premise.as_bytes()).is_err(),
            "a RAW real-file premise with DTD entities must be un-importable (proves the \
             pre-pass is load-bearing)"
        );
        // RESOLVED must import.
        let resolved = resolve_internal_entities(raw_premise);
        assert!(
            rif_xml::import(resolved.as_bytes()).is_ok(),
            "after resolve_internal_entities the same premise must import"
        );

        // End-to-end through the lane driver over a temp test dir.
        let tmp = std::env::temp_dir().join(format!("rif_wg_ent_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let raw_conclusion = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE Document [ <!ENTITY rif "http://www.w3.org/2007/rif#"> <!ENTITY xs "http://www.w3.org/2001/XMLSchema#"> ]>
<Frame xmlns="&rif;">
  <object><Const type="&rif;iri">http://example.org/a</Const></object>
  <slot><Const type="&rif;iri">http://example.org/p</Const><Const type="&xs;integer">7</Const></slot>
</Frame>"#;
        std::fs::write(tmp.join(rif_name("e", "premise")), raw_premise).unwrap();
        std::fs::write(tmp.join(rif_name("e", "conclusion")), raw_conclusion).unwrap();
        let outcome = run_entailment(TestKind::PositiveEntailment, &tmp, "e");
        let _ = std::fs::remove_dir_all(&tmp);
        assert_eq!(
            outcome,
            Outcome::Pass,
            "a real-file-shaped PositiveEntailment (single-slot frame) must PASS end-to-end"
        );
    }

    /// [sq-pbz04.5.5 review Finding 1] A ground NET non-conclusion whose object is a
    /// `List` term is UN-ENCODABLE by the satisfier (`term_to_id` cannot map a `List`),
    /// so counting it "not entailed" would be a FALSE (vacuous) pass. The conclusion
    /// shape filter must reject it → the NET is a SKIP, never a Pass. Drives the REAL
    /// `run_entailment`: an importable ground-fact premise + a bare-frame non-conclusion
    /// whose slot value is a `<List>` (a real Core term the importer parses).
    #[test]
    fn net_list_object_nonconclusion_is_skip_not_vacuous_pass() {
        let tmp = std::env::temp_dir().join(format!("rif_wg_list_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        // Importable premise (a single-slot ground frame fact).
        let premise = r#"<Document xmlns="http://www.w3.org/2007/rif#">
  <payload><Group>
    <sentence><Frame>
      <object><Const type="http://www.w3.org/2007/rif#iri">http://example.org/a</Const></object>
      <slot><Const type="http://www.w3.org/2007/rif#iri">http://example.org/p</Const><Const type="http://www.w3.org/2007/rif#iri">http://example.org/b</Const></slot>
    </Frame></sentence>
  </Group></payload>
</Document>"#;
        // A bare-frame non-conclusion whose slot VALUE is a List(1 2) — ground, but the
        // satisfier cannot encode a List, so it must be SKIP:condition-shape, never Pass.
        let nonconcl = r#"<Frame xmlns="http://www.w3.org/2007/rif#">
  <object><Const type="http://www.w3.org/2007/rif#iri">http://example.org/a</Const></object>
  <slot><Const type="http://www.w3.org/2007/rif#iri">http://example.org/q</Const>
    <List><items>
      <Const type="http://www.w3.org/2001/XMLSchema#integer">1</Const>
      <Const type="http://www.w3.org/2001/XMLSchema#integer">2</Const>
    </items></List>
  </slot>
</Frame>"#;
        // Confirm the non-conclusion IS importable (a real ground List-bearing frame) —
        // so the SKIP below is from the ENCODABILITY filter, not an import failure. This
        // is what makes the regression test non-vacuous: it exercises the exact path a
        // graduated NET would take.
        assert!(
            rif_xml::import(wrap_conclusion(&resolve_internal_entities(nonconcl)).as_bytes())
                .is_ok(),
            "the List-bearing non-conclusion must IMPORT (so the skip is from the \
             encodability filter, not an import error)"
        );
        std::fs::write(tmp.join(rif_name("l", "premise")), premise).unwrap();
        std::fs::write(tmp.join(rif_name("l", "nonconclusion")), nonconcl).unwrap();
        let outcome = run_entailment(TestKind::NegativeEntailment, &tmp, "l");
        let _ = std::fs::remove_dir_all(&tmp);
        assert!(
            matches!(outcome, Outcome::Skip(_)),
            "a ground-but-UN-ENCODABLE (List-object) NET non-conclusion MUST be a SKIP \
             (never a vacuous 'not entailed' pass), got {:?}",
            outcome
        );
        assert_ne!(outcome, Outcome::Pass, "un-encodable NET must NEVER pass vacuously");
    }

    /// [sq-pbz04.5.5 review Finding 2] `is_genuine_core_violation` accepts only the
    /// checker's DETECTED Core violations and rejects the UNSUPPORTED-CAPABILITY
    /// fail-closes (`DistinctGroundEqual`, `Nonmonotonic`), so a reject-kind test rejected
    /// only because a capability is unsupported does NOT launder into a vacuous pass.
    #[test]
    fn genuine_core_violation_excludes_unsupported_capability_failclose() {
        use sparq_reason::rif::RifError;
        // Genuine detections → true.
        assert!(is_genuine_core_violation(&RifError::UnboundHeadVar { var: "x".into() }));
        assert!(is_genuine_core_violation(&RifError::UnboundBuiltinInput { var: "y".into() }));
        assert!(is_genuine_core_violation(&RifError::EqualInConclusion));
        // Unsupported-capability fail-closes → false (must NOT count as a genuine reject).
        assert!(!is_genuine_core_violation(&RifError::DistinctGroundEqual {
            left: "\"1\"^^xsd:integer".into(),
            right: "\"1.0\"^^xsd:decimal".into(),
        }));
        assert!(!is_genuine_core_violation(&RifError::Nonmonotonic {
            what: "Naf".into()
        }));
    }
}
