# Inference feature-completeness audit (W3C suites)

**Thread:** inference-suites (updated by inference-endgame, 2026-06) · **Harness:**
`crates/sparq-conformance`, binary `sparq-inference-conformance` (fetch once with
`scripts/fetch-inference-suites.sh`, then fully offline; report regenerated as
`inference-conformance-report.md`).

Pinned sources: w3c/rdf-tests `f25dbc092c654d792974848e81bb519d7328f0e8` (shared with the
gating SPARQL harness), w3c/N3 `23ccf3d56b25cb60a68878a04aae0d52493080f0`, OWL 2 WG
test-case export `all.rdf` (Internet Archive snapshot `20160703034201`, sha256-pinned).

## Scoreboard (per-suite Overall lines)

| suite | run | pass | fail | documented divergence | out-of-scope (with reason) | rate of run |
|---|---:|---:|---:|---:|---:|---:|
| RDF Semantics (rdf-mt, 48 active entries) | 48 | **48** | 0 | 0 | 0 | **100%** |
| OWL 2 RL (RL-profile ∧ RDF-based, Approved) | 91 | **78** | 0 | 13 | 36 | **100%** (pass+div) |
| N3 — reasoner manifest (89) | 86 | **83** | 0 | 3 | 3 | **100%** (pass+div) |
| N3 — parser manifest (230) | 214 | **213** | 0 | 1 | 16 | **100%** (pass+div) |
| N3 — extended manifest (978) | 871 | **871** | 0 | 0 | 107 | **100%** |
| N3 — TurtleTests, N3 parser in STRICT Turtle mode (297) | 297 | **297** | 0 | 0 | 0 | **100%** |
| SPARQL 1.1 entailment regimes (70) | 47 | **47** | 0 | 0 | 23 | **100%** |
| **all suites** | **1654** | **1637** | **0** | **17** | **185** | **100.0%** |

**ZERO fails.** The inference CI job now GATES with a ratchet at pass+divergence ≥ 1654
(mirrors the SPARQL ratchet; ci.yml `inference-conformance`). Endgame changes:
sparqldl-10/11/12 fixed (entailment-regime answer restriction + harness eq-ref layer,
see §4); cwm_includes_conclusion engine-side fixed (`ground_triple` formula-value
semantics) and reclassified as a documented divergence — the vendored 2003 reference
is byte-provably from older sources and not deductively closed (see §3 table).

N3 out-of-scope = `rdft:Rejected` entries (upstream explicitly rejected them — e.g. biR's
bare `+` list element, "not allowed in turtle nor n3"), 2 `test:strings`
(log:outputString) + 1 `test:filter` reasoner options. Before the n3-completeness thread
the N3 lines were 52/88, 189/230, 880/978, 210/297 (59–90%).

The gating SPARQL harness is untouched and still at 1225 pass + 4 divergences = 1229/1229.

Harness design notes:
- Entailment checks are blank-node **homomorphisms** from the conclusion into the
  materialized closure over GENERALIZED triples (conclusion bnodes may map onto literals —
  the RDF 1.1 lg/gl rules), with D-entailment literal **value** equality
  (integer/decimal exactly via scaled i128; float/double via f32/f64 rounding; langString
  and XMLLiteral value spaces; ill-formedness detection incl. a strict XML scan).
- The conformance regimes need rules the production materializer deliberately omits as
  store-exploding (axiomatic triples, reflexives). The harness adds the FINITE restriction
  (rdf:_n limited to occurring container-membership properties; declared-vocabulary
  reflexives for the SPARQL regimes) *around* `sparq_reason::materialize` — the core rules
  are always exercised through the production code path.

## 1. RDFS entailment patterns (RDF 1.1 Semantics §9.2.1)

| pattern | production materializer (`rdfs.rs`) | conformance result |
|---|---|---|
| rdfs1 (datatype recognition) | omitted by design | harness layer; rdf-mt passes |
| rdfs2 (domain) / rdfs3 (range) | ✅ (incl. literal objects, generalized) | ✅ |
| rdfs4a/4b (everything is a Resource) | omitted by design (O(terms) bloat) | harness layer; rdf-mt passes |
| rdfs5 / rdfs11 (sp/sc transitivity) | ✅ | ✅ |
| rdfs6 / rdfs8 / rdfs10 (reflexives) | omitted by design | harness layer; rdf-mt + sparql-entailment pass |
| rdfs7 (subPropertyOf use) / rdfs9 (subClassOf use) | ✅ | ✅ |
| rdfs12 (ContainerMembershipProperty ⊑ rdfs:member) | omitted | harness layer (finite rdf:_n restriction) |
| rdfs13 (Datatype ⊑ Literal) | omitted | harness layer |
| RDF/RDFS axiomatic triples | omitted by design | harness layer (finite restriction) |
| rdfD1/rdfD2, D-entailment, D-inconsistency | not materialized | harness (`inference/entail.rs`): value spaces, ill-typed literals, intensional datatype-subclass clash — all 48 rdf-mt entries pass incl. every datatype/float/double case |

**Conclusion:** RDFS materialization is complete for the non-axiomatic fragment; the
axiomatic/reflexive layer exists only in the conformance harness. If a product use-case
ever needs it in the store, `entail::close()` is the reference implementation.

## 2. OWL 2 RL/RDF rules table (Profiles §4.3) — per-rule status in `owl.rs`

**owl-rl-completion thread (2026-06):** the previously-missing rules (cls-oo,
cls-maxqc1/2, scm-hv/svf1/svf2/avf1/avf2, cls-thing/nothing1) are now implemented —
every row below is ✅ or an explicit, argued by-design omission. Rule definitions cite
the OWL 2 Profiles REC §4.3 tables (Table 5 prp-*, Table 6 cls-*, Table 9 scm-*),
<https://www.w3.org/TR/owl2-profiles/#Reasoning_in_OWL_2_RL_and_RDF_Graphs_using_Rules>.
Suite scores are UNCHANGED (rdf-mt 48/48; OWL 2 RL 78 pass + 13 divergences, 100% of
run; entailment 44/47) — as predicted, no Approved RL test exercises these rules, so
none of the 13 divergence-allowlist entries went stale (verified: counts identical,
0 fails). Closure throughput stayed within the 5% budget on the RDFS and OWL-RL bench
workloads (`bench/inference/owl-bench.sh`). (N3 suites: not re-runnable on the macOS
dev host — a pre-existing `n3/mod.rs` panic + runaway-memory test kills the harness
there at the BASE commit too, before any owl.rs change; the N3 code paths are
untouched by this thread and `cargo test -p sparq-reason` (incl. the eye_cases
suite) stays green.)

| rule | status | where / note |
|---|---|---|
| eq-ref | ◑ partial **by design** | reflexive sameAs emitted only for terms touched by equality. Full eq-ref derives `x sameAs x` for EVERY subject/predicate/object in the graph — O(terms) store bloat that no join ever consumes (the union-find already knows reflexivity); the touched-terms restriction is exactly what SPARQL-entailment answers need, and the entailment suite passes on it. |
| eq-sym, eq-trans, eq-rep-s/p/o | ✅ | union-find entity rewriting + full expansion at the end (RDFox approach) |
| eq-diff1 | ✅ | `inconsistencies()` |
| eq-diff2, eq-diff3 (AllDifferent) | ✅ **(this thread)** | `inconsistencies()`, members/distinctMembers lists |
| prp-ap (annotation-property axioms) | ◑ omitted **by design** | premise-free axioms typing the 9 builtin annotation properties (rdfs:label/comment/seeAlso/isDefinedBy, owl:versionInfo/deprecated/priorVersion/backwardCompatibleWith/incompatibleWith) as `owl:AnnotationProperty`. Materializing them injects vocabulary into EVERY closure (even an empty graph's closure becomes non-empty) — the same store-spam class as the RDF/RDFS axiomatic triples, which live in the harness layer by the same argument. No suite test demands them; if a regime ever does, the SPARQL-regime layer (which already adds declared-vocabulary reflexives) is the right home. |
| prp-dom, prp-rng | ✅ | via rdfs2/3 |
| prp-fp, prp-ifp | ✅ | derive sameAs → merge; literal-value clash detected (dt-diff ⊢ eq-diff1, **this thread**) |
| prp-irp, prp-asyp | ✅ **(this thread)** | `inconsistencies()` |
| prp-spo1 | ✅ | rdfs7 |
| prp-spo2 (property chains) | ✅ | |
| prp-eqp1/2, prp-inv1/2, prp-symp, prp-trp | ✅ | |
| prp-pdw, prp-adp | ✅ **(this thread)** | `inconsistencies()` |
| prp-key | ✅ | incl. data properties (literal clash) |
| prp-npa1/2 | ✅ **(this thread)** | `inconsistencies()` |
| cls-thing, cls-nothing1 | ✅ **(owl-rl-completion thread)** | premise-free `owl:Thing/Nothing rdf:type owl:Class`, emitted in `pre_monotone` when the term OCCURS in the data (occurrence guard = the XSD-hierarchy discipline: no injected vocabulary for graphs that never mention Thing/Nothing) |
| cls-nothing2 | ✅ | `inconsistencies()` |
| cls-int1/2, cls-uni | ✅ | |
| cls-com | ✅ | clash |
| cls-svf1/2, cls-avf, cls-hv1/2 | ✅ | |
| cls-maxc1 | ✅ | clash |
| cls-maxc2, cls-maxqc3/4 | ✅ | derive sameAs |
| cls-maxqc1/2 (qualified cardinality 0 clashes) | ✅ **(owl-rl-completion thread)** | `inconsistencies()`: u typed a maxQC-0 restriction [onProperty p; onClass c] with a c-typed p-value (any value when c = owl:Thing); guarded — only the p-edges of qualified-cardinality-0 restrictions are scanned |
| cls-oo (oneOf typing) | ✅ **(owl-rl-completion thread)** | `owl:oneOf` lists decoded in `ClassFeatureIdx`; members typed as instances of the enumeration class — LINEAR in total list length (lists decoded once, not per round); oneOf added to the feature-detection sets so oneOf-only ontologies route to the fixpoint |
| cax-sco, cax-eqc1/2 | ✅ | |
| cax-dw | ✅ | clash |
| cax-adc | ✅ **(this thread)** | clash, members list |
| dt-type1/2, dt-eq, dt-not-type | ◑ harness-level **by design** | literal datatype typing/value equality are VALUE-SPACE judgements, not triple joins: materializing dt-eq means a `sameAs` edge for every co-valued literal pair (quadratic in literals), and dt-type2 types every literal occurrence. The harness D-machinery compares values at entailment-check time instead; rdf-mt passes 48/48 incl. every datatype case. dt-diff's only rule-consequence (clash with derived sameAs) IS in the store path as the literal-sameAs clash. |
| dt-diff | ✅ **(this thread)** | as the literal-sameAs clash |
| scm-cls | ◑ partial **by design** | per declared class: `c ⊑ c`, `c ≡ c`, `c ⊑ owl:Thing`, `owl:Nothing ⊑ c` — reflexive tautologies plus 2·\|classes\| Thing/Nothing edges that no rule premise consumes (every rule joining on `subClassOf` is a no-op on reflexive edges) and no query wants un-asked. Added by the SPARQL-regime layer where the regime explicitly demands them; the entailment suite passes. |
| scm-sco, scm-spo | ✅ | rdfs11/5 |
| scm-eqc1, scm-eqp1 | ✅ | equivalence folded into subsumption both ways |
| scm-eqc2, scm-eqp2 | ✅ **(this thread)** | mutual subsumption ⊢ equivalence (post-pass) |
| scm-op, scm-dp | ◑ harness-level **by design** | per declared property: `p ⊑ p`, `p ≡ p` — the same reflexive-tautology argument as scm-cls (no rule premise is enabled by a reflexive subPropertyOf edge); added by the SPARQL-regime layer only. |
| scm-dom1/2, scm-rng1/2 | ✅ | + **(this thread)** inverseOf domain/range transposition (valid RDF-based entailment beyond the table) |
| scm-hv, scm-svf1/2, scm-avf1/2 | ✅ **(owl-rl-completion thread)** | schema-level restriction subsumption in the per-round class-feature pass (conclusions re-enter the fixpoint and feed rdfs9/11 like the other scm-* rules). INDEXED/GUARDED instead of the naive quadratic restriction×restriction join: grouped by onProperty (svf1/avf1) resp. by filler/value (svf2/avf2/hv), probing only each member's explicit subClassOf/subPropertyOf out-edges — O(restrictions × direct super-edges). NB scm-avf2's conclusion reverses (`c2 ⊑ c1`, contravariant in the property) per Profiles §4.3 Table 9. |
| scm-int, scm-uni | ✅ | schema level |
| XSD datatype hierarchy (numeric tower ⊑) | ✅ **(this thread)** | direct edges for occurring datatype IRIs (WebOnt-I5.8) |

13 documented divergences = expected conclusions PROVABLY outside the RL/RDF rules
(OWL 2 Conformance §2.3 / theorem PR1 scopes completeness to assertion-style conclusions):
TBox-axiom conclusions (chain2trans1), invented class expressions (DisjointClasses-001/003,
ObjectQCR-002, I5.5-005), reified AllDifferent structures, prp-pdw/fp/ifp contrapositives,
datatype-range intersections (I5.8-008/009), and one export-side mis-tag
(ReflexiveObjectProperty is not in the RL grammar). Full rationale list is in the report
and `owl_suite.rs::DOCUMENTED_DIVERGENCES`.

Out-of-scope: 30 non-Approved cases (22 status-absent + 8 Proposed), 3 functional-syntax-only,
2 owl:imports, 1 selection-summary row. Export totals: 124 RL∧RDF-based reasoning cases of
~441 reasoning cases (251 not RL-profiled, 66 direct-semantics-only).

### 2b. sq-350ms re-verification — the OWL-RL row is at the genuine RL ceiling (no sound raise) [OPUS-4.8]

The **owl-rl-completeness-hardening** bead (`sq-350ms`, epic `sq-pbz04`) asked whether
any of the 13 documented OWL-RL divergences can be converted to a TRUE pass by adding
SOUND, IN-PROFILE rule coverage. The answer, after re-checking every divergence's exact
premise→conclusion against the W3C OWL 2 Profiles REC §4.1 (profile grammar) + §4.3
(the RL/RDF rules tables), is **NO — all 13 are PROVABLY outside the RL profile**, so
the OWL-RL row (78 pass + 13 divergence) is at the RL ceiling and the inference ratchet
**HOLDS at 1967** (raising it would require faking a pass, forbidden):

| divergence(s) | needs | why non-RL |
|---|---|---|
| chain2trans1 | `?p a owl:TransitiveProperty` from a self-chain | a TBox-AXIOM conclusion; no RL/RDF rule has an axiom in its head (PR1 completeness is assertion-only) |
| DisjointClasses-001/003, ObjectQCR-002, I5.5-005 | `?x a [owl:complementOf/unionOf …]` | invents an anonymous CLASS EXPRESSION; the RL rules derive no new class expressions |
| DisjointDataProperties-002 | a reified `owl:AllDifferent` structure | the RL rules DETECT inconsistency from AllDifferent (eq-diff2/3) but never CONSTRUCT one |
| DisjointObjectProperties-001/002 | `owl:differentFrom` of the disjoint-property fillers (prp-pdw contrapositive) | **no RL/RDF rule produces `owl:differentFrom` between INDIVIDUALS** — the sole differentFrom-head rule is dt-diff (Table 8, unequal-value LITERAL pairs); otherwise it appears only in rule BODIES (eq-diff*); prp-pdw/adp only emit `false` (verified against Table 5) |
| owl2-rl-rules-fp/ifp-differentFrom | `owl:differentFrom` of subjects (prp-fp/ifp contrapositive) | same — no individual-differentFrom-producing rule; prp-fp/ifp need the SAME subject/object and only derive `sameAs` |
| ReflexiveProperty-001 | a reflexive edge for an individual NOT participating in the property | **`owl:ReflexiveObjectProperty` is EXCLUDED from OWL 2 RL** (Profiles §4.1: "all axioms … apart from disjoint unions of classes and reflexive object property axioms"); there is no `prp-rfx` rule |
| I5.8-008/009 | datatype-range INTERSECTION (`xsd:short ∩ xsd:unsignedInt ⊑ xsd:unsignedShort`) | beyond the RL `dt-*` datatype rules — RL has no datatype-subsumption-via-intersection rule (range propagates only UP `rdfs:subClassOf`, never narrows) |

The contrapositive-`differentFrom` cases are SOUND OWL RDF-Based entailments, but RL
deliberately omits the contrapositives to stay polynomial — adding them would be a
beyond-RL extension, NOT in-profile RL completeness, so per the bead's hard rule they
**remain documented divergences**. (If a complete-classification / beyond-RL surface is
ever wanted, that is the EL/QL classifier territory — `sparq-reason-el`/`-ql` — not more
RL rules.)

**sq-pbz04.1.3 disposition pass (2026-07) [FABLE-5]:** an independent re-audit from the
raw export premises/conclusions (all 13 extracted and checked term-by-term against the
Profiles §4.3 rule HEADS, the §4.1 grammar, and PR1's scope) CONFIRMS the verdict above —
**13/13 PERMANENT, zero in-profile fixes, ratchet unchanged at 1967** — and carries the
disposition into the report-facing rationales: each `owl_suite.rs::DOCUMENTED_DIVERGENCES`
entry now opens `PERMANENT — …` with its specific rule-level grounding, pinned by an
in-crate disposition test (count, uniqueness, tag, and a checkable spec anchor per entry).
Two precision corrections to the earlier wording: (a) `dt-diff` (Table 8) DOES have
`owl:differentFrom` in its head — but only between LITERALS with different data values;
the accurate claim is that no rule derives `differentFrom` between INDIVIDUALS (table row
fixed above); (b) theorem PR1's conclusion scope is ClassAssertion /
ObjectPropertyAssertion / DataPropertyAssertion / SameIndividual — DifferentIndividuals
conclusions sit outside PR1 entirely, independently of the missing contrapositive rules.

What sq-350ms DID land (sound, behaviour-neutral): six in-crate guards in
`owl.rs::tests` — four COMPLETENESS guards pinning the harder MULTI-ROUND assertion-rule
compositions the conformance suite cannot reach a regression in (`cls-svf1` over a
DERIVED filler type; `prp-spo1`⊕`prp-trp` on a transitive super-property; `cls-int1`⊕
`cax-eqc1`; a 2-link `prp-eqp` equivalence chain) and two SOUNDNESS guards proving the
materializer does NOT derive the forbidden `differentFrom`/`sameAs` of the prp-pdw and
prp-fp contrapositive cases. They keep the divergence rationale and the code locked
together; the suite counts and the ratchet are byte-for-byte unchanged.

## 3. N3 — engine and builtin inventory vs EYE/cwm

Reasoner manifest: **83 pass + 2 documented divergences of 86 run** (98.8%); parser
**213+1div/214**, extended **871/871**, TurtleTests **297/297** (the N3 parser in strict
Turtle mode). The n3-completeness thread (this branch) closed the audit's gap list:

- **First-class lists (`Term::List`)**: `( … )` is a term; structural unification in
  forward joins and backward goals; lists constructible in conclusions; `list:append`,
  `list:iterate`, virtual `rdf:first`/`rdf:rest` over list terms; `rdf:nil` ≡ `()`;
  dictionary interning expands list values back to first/rest chains.
- **Quantifiers**: `@forAll`/`@forSome` at document and formula scope (IRI-derived
  deterministic names); premise blanks are existential VARIABLES only outside quoted
  formulae; conclusion blanks instantiate FRESH per (rule, conclusion-binding) firing
  (cwm quant-implies).
- **log:includes scoping**: containment over the subject formula with pattern
  existentials as wildcards, rule variables binding, scope-side quantified terms as
  opaque constants (the cwm quantifiers_limited matrix); `{}` is the literal true (the
  empty formula — includes nothing); `log:supports` closes the scope under its own rules
  first; unbound/non-formula subjects keep the store-scoped NAF idiom.
- **Unification through quoting**: formula-vs-formula multiset unification in join atoms
  (cwm unify1/unify2); `<=` is `log:isImpliedBy` (legacy `log#impliedBy` accepted).
- **Builtin readiness ordering**: premises reorder so builtins run after their input
  producers (cwm "when ready" coroutining).
- **Math**: exact integer/decimal arithmetic incl. decimal^int; real-valued family
  (trig, log, atan2, degrees/radians) always xsd:double with cwm e-notation lexicals and
  REVERSE modes; IEEE INF/NaN propagation (and double division by zero) vs domain errors
  failing the premise; integer-only divisor-sign `math:remainder`; empty sum/product.
- **Builtins added**: list:append/iterate, log:conclusion/parsedAsN3/langlit/supports,
  log:semantics/log:content (opt-in resolver), string:containsRoughly, cwm
  encodeForURI/encodeForFragID, time:dayOfWeek/timeZone/inSeconds (bidirectional) and
  cwm singular hour/minute/second.
- **Policy (document access)**: the engine performs NO I/O; `log:semantics`/`log:content`
  evaluate only when the caller passes a `Resolver` (`reason_n3_terms_with_resolver`).
  The harness supplies a strictly-offline resolver mapping the suite's canonical
  `https://w3c.github.io/N3/tests/` IRIs into the pinned clone, and parses all documents
  against that canonical base (the TurtleTests `.nt` expectations bake those IRIs in).

### Remaining N3 fails/divergences (all annotated in the report)

| entry | class | cause |
|---|---|---|
| cwm_includes_conclusion | documented divergence | engine now derives the `:result :is {…}` formula (`ground_triple` fix: quoted formulae are VALUES, their remaining variables formula-scoped). The vendored ref still cannot match: (1) its quoted `daml:comment` lacks the TAB present in the vendored daml-ex.n3 (`…of\n\tontological…` vs `…of\nontological…`, byte-verified) — generated from an older daml-ex.n3; (2) the ref formula is not closed under its OWN quoted rules (holds `d:father daml:range d:Man`, `daml:range = rdfs:range`, the `{?x ?p1 ?y. ?p1 = ?p2}=>{?x ?p2 ?y}` rule, yet lacks `d:father rdfs:range d:Man`) — we derive 31 statements the 2003 cwm run did not |
| cwm_includes_t11 | documented divergence | the vendored ref reflects a cwm run whose `log:semantics <t10a.n3>` failed and that purged with an unrecorded `--purge`; our resolver derives the schema-checking conclusions |
| cwm_unify_unify1 | documented divergence | vendored ref says `:test a :Successful` (rdf:type) but the action concludes `:test :a ?x` (predicate `<#a>`) — ref generated from an older action |
| numbers.n3 | documented divergence | expected output keeps ONE statement under the generating author's local base (`file:/home/syosi/...#is`) |
| bad_prefix2 | documented divergence | suite-internal conflict: expects undeclared `:` rejected, while the reasoner manifest's own cwm actions (unify1.n3) require cwm's undeclared-`:`-is-`<#>` convention |
| sparqldl-10/11/12 | FIXED (inference-endgame) | entailment-regime answer restriction + harness eq-ref layer — see §4 |

### Parser posture (syntax suites)

The hand-rolled parser now has two dialects: full N3 (quantifiers, @keywords, paths —
also in predicate position, inverted `<-` predicates, iriPropertyLists, zero-predicate
statements, `{}`=true) and STRICT W3C Turtle (`parse_turtle_with_base`: rejects every
N3-only construct, enforces statement termination, declared prefixes, PN_LOCAL/LANGTAG/
escape rules; 297/297 TurtleTests). Both dialects validate IRIREF characters and decode
`\uXXXX`/`\UXXXXXXXX` (escaped forbidden characters are rejected), normalize language
tags to lowercase, and resolve relative IRIs per RFC 3986 (dot segments, query-only
references). NOTE: sparq's product Turtle path is still oxttl (gating SPARQL harness);
the strict mode exists for the N3 subsystem's own surface.

## 4. SPARQL 1.1 entailment regimes

47/47 run pass (RDF + RDFS + OWL-RDF-Based mappings); 23 out-of-scope with reasons
(18 OWL-Direct-only, 4 RIF, 1 D-only). Regime layer added by the runner: rdfD2 predicate
typing, declared-class/property reflexives (rdfs6/10, scm-cls/op/dp), Thing/Nothing edges,
Thing-typing of NamedIndividuals; (endgame) a harness-side **eq-ref** layer over the OwlRl
closure (OWL 2 Profiles §4.3 Table 4 — reflexive `owl:sameAs` for every closure term; the
production materializer omits it as store bloat).

The former 3 fails, fixed by the endgame thread's **answer restriction**
(`run::EntailmentAnswerFilter`, applied to engine solutions before comparison):

- **C1/skolemization filter** (Entailment Regimes §2 condition C1, §3.1): the
  Skolemization function `sk` is defined exactly for the blank nodes of the queried
  graph SG, so a binding to any blank node NOT in SG (e.g. saturation-introduced)
  leaves `sk(P(BGP))` non-ground and is never a solution — §3.1: *"new blank nodes
  introduced in the saturation process are not to be returned in the solutions"*.
  Bindings to bnodes that DO occur in SG are kept (owlds02, "bnodes are not
  existentials with answer", expects one).
- **OWL-Direct name-position filter** (`sparqldl-11/12`): the dual-tagged
  (OWL-Direct + OWL-RDF-Based) tests' vendored expectations are the **Direct-regime**
  answers. Under §7 variables stand *"in place of class names, object property names,
  datatype property names, individual names, or literals"* (extended grammar §7.1.2) —
  an anonymous class expression (bnode) is not a name, so for tests whose regime set
  includes OWL-Direct, bnode bindings for variables in class/property-name positions
  are dropped. (Honesty note: under pure RDF-Based semantics the data's restriction
  bnode WOULD be an answer — §6.4.5 treats bnodes as first-order constants; the filter
  exists because the suite's expected answers were generated under Direct semantics.)
- **`sparqldl-10`** needed no non-distinguished-variable machinery at all: with the
  eq-ref layer in the closure, plain BGP evaluation yields exactly the expected 3
  (duplicate) rows — the query's projected-away variables bind to named individuals
  via `b owl:sameAs b`.

## 5. Prioritized gap map — status after the n3-completeness thread

1. **N3 `Term::List`** — ✅ DONE (first-class list terms, unification, construction,
   append/iterate, virtual first/rest; backward goals over list arguments unify
   structurally).
2. **`@forAll`/`@forSome`** — ✅ DONE (document + formula scope, includes-matrix
   semantics).
3. **Conclusion-bnode instantiation** — ✅ DONE (fresh existentials per
   (rule, conclusion-binding) firing).
4. **log:includes full scoping** — ✅ DONE (formula containment with quantifier
   semantics, `{}` = true, log:supports; store-NAF kept for unbound subjects).
5. **N3-parser Turtle strictness pass** — ✅ DONE (strict dialect; TurtleTests 297/297;
   escape/IRI validation shared with the N3 dialect).
6. **SPARQL-regime answer filtering** (sparqldl-10/11/12) — ✅ DONE
   (inference-endgame thread, 2026-06): entailment-regime answer restriction
   (C1/skolemization + OWL-Direct name positions) and harness eq-ref layer; 47/47 — §4.
7. **OWL leftovers** (cls-oo, cls-maxqc1/2, scm-hv/svf/avf) — ✅ DONE
   (owl-rl-completion thread, 2026-06): the §2 table is now fully ✅ or explicitly
   by-design; every implemented rule has a hand-computed-closure unit test.
8. **Document-access policy** — ✅ DECIDED + IMPLEMENTED: opt-in `Resolver` hook
   (`reason_n3_terms_with_resolver`); engine stays I/O-free by default;
   `log:semantics`/`log:content` work under the harness's offline resolver.
   `log:outputString` (test:strings, 2 entries) remains out of scope.

Remaining N3 work, in suite-pressure order (none has suite pressure left — all suites at
0 fails): `log:collectAllIn`, EYE backward list-state idioms beyond what the suite
exercises. cwm_includes_conclusion is engine-side DONE (the `ground_triple` formula-value
fix; the residual mismatch is upstream-reference rot, documented as a divergence).

## 5b. Performance evidence (inference-endgame, 2026-06)

`bench/inference/eye-comparison.md` records the EYE v11.24.4 head-to-head (same
machine, full parse→closure→serialize pipeline both sides): sparq wins every
workload — DeepTaxonomy (sparq linear-in-closure, EYE ≈N², dt100k vs
extrapolated ≈9 h), grid reachability, and — since the fixpoint-opt thread
(2026-06) — the pure transitive chain by ~2 orders of magnitude too: the O(N³)
chain-transitivity derivation storm noted here as the optimization target is
FIXED by linearizing transitivity through generator edges
(`R(x,y), GEN(y,z) ⊢ R(x,z)`, `TC(GEN)=TC(R)`) in both the OWL prp-trp fixpoint
(owl-bench `owl-transitive` 54 s → 0.25 s) and the N3 chainer (anc500 52 s →
0.18 s engine-internal, closures byte-identical); closure-only callers also
skip proof-step materialization (`StepMode`, grid30 closure −47%). Same doc
carries the owl-bench.sh closure numbers (RDFS 2.1 M-triple closure in
~0.05 s). Wasm artifact: 1,573,895 B (cargo wasm32 release; unchanged through
the fixpoint-opt thread; +8 B over the tracked 1,573,887 B baseline,
attributable to the `ground_triple` fix).

## 5c. Incremental maintenance (incremental-inference thread, 2026-06-12)

All three profiles now have **counting-based incremental closure maintenance** under base
inserts AND deletes — opt-in, zero wasm impact, batch paths untouched (the conformance
scoreboard above is the proof: unchanged at 1637/0/17).

| profile | API | incremental | documented fallback (full re-materialization) |
|---|---|---|---|
| RDFS (T18) | `MaterializedGraph` | all ABox rules (one-step counting against the closed TBox) | TBox mutations (subClassOf/subPropertyOf/domain/range) |
| OWL 2 RL | `MaterializedOwlGraph` | monotone assertional rules (prp-spo1, cax-sco, prp-dom/rng, prp-inv1/2, prp-symp, prp-eqp1/2, cax-eqc1/2) via the px property-orientation closure; prp-trp via an exact transitive layer (per-property effective-edge multisets, closure diffing) | TBox mutations (incl. scm-*: they can only fire on TBox facts); sameAs / Functional / InverseFunctional / chains / restrictions / cardinality / hasKey / oneOf / intersection / union (recursive-equality features → `OwlMode::Fallback`); occurrence-guarded vocab deltas (owl:Thing/Nothing, XSD tower) |
| N3 | `MaterializedN3Graph` | rule sets ANALYSIS proves monotone with input-stratified negation: ground IRI predicates, builtin-parity whitelist (log:uri, log:equalTo/notEqualTo, string:concatenation/scrape/encodeForUri), `?UNSCOPED log:notIncludes` only over underived predicates, recursion via recursive-SCC layers | disqualified rule sets (any other builtin, formula containment, backward rules, conclusion existentials, variable conclusion predicates); guard-predicate deltas; implies-as-data; out-of-whitelist data (sticky) — all re-run the batch engine, always correct |

Correctness: differential property tests (tests/incremental_prop.rs,
incremental_owl_prop.rs, incremental_n3_prop.rs) hold each maintained closure equal to its
from-scratch batch run after EVERY randomized edit batch, including fallback-mode profiles
and guard/TBox rebuild paths. Deletion is exact (counting, no DRed overdelete/rederive) and
costs the same order as insertion.

Numbers (bench/inference/incremental-bench.md; olympics 1.78 M triples + a 1k-doc WAC pod):
1-triple deltas ~1–4 µs vs ~1–1.7 s re-materialization; 10k-triple deltas 16–512×; WAC ACL
edits 11–162 ms vs the 0.84 s engine re-run. The sparq-solid qualification matrix:
common+wac COUNTING, acp-a/b COUNTING, acp-c FALLBACK (`{ ?p ?pred ?r }` variable
conclusion predicate — restructure to ground predicates to qualify).

Merge note (owned by the owl.rs/rdfs.rs thread): `MaterializedOwlGraph::emit_std`
deliberately mirrors a batch quirk — on the monotone `PropExpand` path, a domain/range-only
property absent from the property-orientation map emits NO domain/range typing (the full
fixpoint path does emit it; likely `rdfs::build_prop_expand`'s `all_props` missing
domain/range-only properties). If that batch bug is fixed, drop the mirrored
`None => return` arm in `emit_std` and the px-quirk notes; the differential tests will
flag the divergence automatically.

## 6. Reproduction

```sh
./scripts/fetch-inference-suites.sh         # one-time, pinned
cargo run --release -p sparq-conformance --bin sparq-inference-conformance
# report: inference-conformance-report.md  (--verbose for per-test lines,
# --filter SUBSTR to narrow, --strict to exit 1 on fails)
```

CI: the `inference-conformance` job (informational, non-gating) uploads the report and
prints it to the step summary; ratchet to follow once the N3 score stabilizes.
