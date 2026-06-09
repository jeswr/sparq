# SPARQL 1.2 — Implementer's Reference (primary-source)

Compiled for the **sparq** Rust engine. All claims cited to W3C SPARQL 1.2 / RDF 1.2 documents. Grammar quoted verbatim from the 5 June 2026 Query WD. Note: as of June 2026 every SPARQL 1.2 document is a **Working Draft** (the RDF 1.2 core docs are CR Snapshots) — none are Recommendations yet, so treat specifics as stable-in-direction but version-pin against the dated drafts below.

---

## 0. Document set, maturity, and exact dates

Source: [RDF & SPARQL WG publications](https://www.w3.org/groups/wg/rdf-star/publications/)

| Document | Status | Date |
|---|---|---|
| **RDF 1.2 Concepts and Abstract Data Model** | **CR Snapshot** | 7 Apr 2026 |
| **RDF 1.2 Semantics** | **CR Snapshot** | 7 Apr 2026 |
| SPARQL 1.2 Query Language | Working Draft | **5 Jun 2026** |
| SPARQL 1.2 Update | Working Draft | 23 Apr 2026 |
| SPARQL 1.2 Protocol | Working Draft | 26 Apr 2026 |
| SPARQL 1.2 Federated Query | Working Draft | 23 Apr 2026 |
| SPARQL 1.2 Service Description | Working Draft | 23 Apr 2026 |
| SPARQL 1.2 Entailment Regimes | Working Draft | 9 Apr 2026 |
| SPARQL 1.2 Results JSON Format | Working Draft | 28 Mar 2026 |
| SPARQL 1.2 Results CSV/TSV Formats | Working Draft | 28 Mar 2026 |
| SPARQL 1.2 Results XML Format | Working Draft | 27 Dec 2024 |
| SPARQL 1.2 Graph Store Protocol | Working Draft | 19 Dec 2024 |
| RDF 1.2 Turtle / TriG / N-Triples / N-Quads / XML | Working Draft | May–Jun 2026 |
| RDF 1.2 Schema | Working Draft | 28 Mar 2026 |

**What's stable vs in flux:** the data model (triple terms, `rdf:reifies`, directional language strings) is locked by the RDF 1.2 Concepts/Semantics CR Snapshots — W3C [invited implementations of RDF 1.2 Concepts and Semantics](https://www.w3.org/news/2026/w3c-invites-implementations-of-rdf-1-2-concepts-and-abstract-data-model-and-rdf-1-2-semantics/) (7 Apr 2026). The SPARQL surface (grammar productions, function names, results encodings) is in WD and could still shift, but the WD has reached a self-consistent state and matches the rdf-tests suite. To advance to PR each feature needs ≥2 independent implementations. **Build against the dated drafts above; don't assume "SPARQL 1.2" is frozen.**

URLs: [Query](https://www.w3.org/TR/sparql12-query/) · [Update](https://www.w3.org/TR/sparql12-update/) · [Concepts](https://www.w3.org/TR/rdf12-concepts/) · [Turtle](https://www.w3.org/TR/rdf12-turtle/) · [Results JSON](https://www.w3.org/TR/sparql12-results-json/) · [Results CSV/TSV](https://www.w3.org/TR/sparql12-results-csv-tsv/) · [Federated Query](https://www.w3.org/TR/sparql12-federated-query/)

---

## 1. SPARQL 1.2 vs 1.1 — what's new

The Query spec's [Appendix A — Changes](https://www.w3.org/TR/sparql12-query/#a-changes-between-sparql-1-1-query-language-and-sparql-1-2-query-language) lists the **normative** changes verbatim:

> - Update grammar for triple terms, reifiers, reified triples, annotation syntax, and triple term functions in 19.7 Grammar
> - Add functions related to triple terms to 17.4.6 Functions on Triple Terms: **TRIPLE, isTRIPLE, SUBJECT, PREDICATE, OBJECT**
> - Update grammar for literal base direction syntax
> - Update grammar for VERSION declaration and a new section to describe its usage
> - Add functions related to language tag and base direction: **LANGDIR, hasLANG, hasLANGDIR, and STRLANGDIR**
> - Define parser input as being an RDF string. Exclude Unicode surrogates from Unicode escape sequences
> - Remove concepts of plain and simple literals, in favor of explicit mentions of xsd:string
> - Migrate XML Schema references to 1.1 … Update references to XPath from 2.0 to 3.1
> - Define EBV as a functional form
> - Forbid duplicated variables in VALUES
> - Add in-between term type ORDER BY support for triple terms in 15.1 ORDER BY
> - Fixes the previously informal definition of EXISTS by adding a formal definition in 17.4.1.4 … extending the eval function with a solution mapping μ_ctx as third argument
> - Rename function RDFterm-equal as 17.4.2.2 **sameValue** and expand the definition to cover literal arguments of differing datatypes where the values are known to be equal or to be not equal
> - Expand the restriction on the use of * projection on queries that have implicit grouping
> - Escape sequence processing has been changed to be processed during parsing, not before. This aligns SPARQL with escape sequences in Turtle.

### 1.1 The core feature: triple terms (RDF-star → RDF 1.2)

**Terminology shift from RDF-star — critical for an implementer:** RDF 1.2 dropped the "quoted triple" concept. The data-model object is now a **triple term**, and it may appear **only in the object position** of a triple — *not* the subject. ([Concepts §3.6](https://www.w3.org/TR/rdf12-concepts/#x3-6-triple-terms)):

> An RDF triple used as the object of another triple is called a **triple term**. … Since triple terms are triples, equality of triple terms is the same as triple equality.

The two syntaxes are deliberately distinct (this trips up implementers migrating from RDF-star):

- **`<<( s p o )>>`** — a **triple term** (the actual term, object-position only). This is the new "TripleTerm".
- **`<< s p o >>`** and **`<< s p o ~ id >>`** — a **reified triple** (syntactic sugar producing `id rdf:reifies <<( s p o )>>`). This is the old RDF-star `<< >>` repurposed.

([Turtle §2.10–2.11](https://www.w3.org/TR/rdf12-turtle/#x2-11-reifying-triples)):

> A reifying triple is a triple where the predicate is **rdf:reifies** and the object is a triple term. The subject of that triple is called a **reifier**. … A reifiedTriple is syntactic sugar representing a reifying triple … `<< :s :p :o ~ :IRIREF >>` … If no reifiers are present, or a reifier is not immediately followed by an iri or BlankNode, a fresh RDF blank node is allocated.

Expansion (Turtle Examples 23–25):
```
<< :employee38 :jobTitle "Assistant Designer" >> :accordingTo :employee22 .
# ≡
_:id rdf:reifies <<( :employee38 :jobTitle "Assistant Designer" )>> .
_:id :accordingTo :employee22 .
```

**Triple terms are transparent** ([Concepts §1.5](https://www.w3.org/TR/rdf12-concepts/#x1-5-triple-terms-and-reification)): a term inside a triple term denotes the same resource as in an asserted triple. A triple term is **not asserted** by virtue of appearing inside a reifying triple — `:Bob` claiming a proposition does not make it a graph member.

### 1.2 New built-in functions — CURRENT names and exact signatures

From [§17.4.6](https://www.w3.org/TR/sparql12-query/#x17-4-6-functions-on-triple-terms). **The current names are `TRIPLE`/`isTRIPLE`/`SUBJECT`/`PREDICATE`/`OBJECT`** — there is no `hasTRIPLE` or `TRIPLE_TERM` function (those were never adopted):

```
triple-term TRIPLE(RDF-term subj, RDF-term pred, RDF-term obj)
RDF-term    SUBJECT(triple-term tt)
RDF-term    PREDICATE(triple-term tt)
RDF-term    OBJECT(triple-term tt)
xsd:boolean isTRIPLE(RDF-term term)
```

- `TRIPLE(s,p,o)`: if `(s,p,o)` is a valid RDF triple (subj IRI/bnode; pred IRI; obj IRI/triple-term/bnode/literal) returns a triple term; **otherwise raises an error**. Shorthand `<<( subj pred obj )>>` is equivalent but **only allows a variable or a directly-written RDF term in each slot (no arbitrary expressions), and subject/predicate are limited to IRI or variable**. The function form accepts arbitrary expressions.
- `SUBJECT`/`PREDICATE`/`OBJECT`: error if argument is not a triple term.
- `isTRIPLE(term)`: true iff term is a triple term, else false (no error).

**Directional language-string functions** ([§17.4.2.9–.11, .17](https://www.w3.org/TR/sparql12-query/#x17-4-2-9-langdir)):
```
xsd:string LANGDIR(literal ltrl)                                  // base direction "ltr"/"rtl", or "" if none; error on non-literal
xsd:boolean hasLANG(RDF-term term)                                // literal has a language tag (datatype rdf:langString or rdf:dirLangString)
xsd:boolean hasLANGDIR(RDF-term term)                             // literal has a base direction (datatype rdf:dirLangString)
literal STRLANGDIR(xsd:string lexical, xsd:string langTag, xsd:string baseDir)  // builds rdf:dirLangString literal
```
- `STRLANGDIR`: `langTag` MUST NOT be empty; `baseDir` MUST be exactly `"ltr"` or `"rtl"` (case-sensitive — `"LTR"` → error). Result e.g. `STRLANGDIR("abc","en","ltr")` → `"abc"@en--ltr`.
- `LANG` (§17.4.2.8) still returns just the language subtag; the `LANGDIR` function is the new accessor for direction.
- The signature shown for `hasLANG`/`hasLANGDIR` reads `xsd:string` in the WD but the body and example table define a boolean result (likely a spec typo — treat as `xsd:boolean`).

**sameValue** ([§17.4.2.2](https://www.w3.org/TR/sparql12-query/#x17-4-2-2-samevalue)) — renames `RDFterm-equal`. It cannot be called directly; it defines `=` for term pairs not covered by the operator-mapping table. Key rules for an implementer: equal terms → TRUE; IRI/bnode → FALSE; exactly one triple term → FALSE; both triple terms → pairwise `sameValue` (TRUE if all components TRUE, error if any component errors, else FALSE); `NaN`/`NaN` of xsd:double/float → TRUE; ill-typed literal → error; otherwise determine by value or error.

### 1.3 VERSION declaration

New [§4.3](https://www.w3.org/TR/sparql12-query/#x4-3-version-announcement). A `VERSION "x"` directive in the Prologue announces required syntax/semantics. Version labels ([§4.3.1](https://www.w3.org/TR/sparql12-query/#x4-3-1-version-labels)):

| Label | Syntax |
|---|---|
| `"1.2"` | full SPARQL 1.2 (triple terms + triple patterns with a triple pattern in subject/object) |
| `"1.2-basic"` | SPARQL 1.2 syntax **without triple terms and without triple patterns that have a triple pattern in subject/object position** |
| `"1.1"` | SPARQL 1.1 syntax (use in a VERSION directive is *discouraged* — would break 1.1 parsers) |

Conformance nests: `"1.1"` ⊆ `"1.2-basic"` ⊆ `"1.2"`. Processors MAY treat unrecognized labels as error or warning.

### 1.4 Other deltas (federation, results, properties)

- **Property paths**: unchanged in 1.2 (the grammar productions [94]–[102] are identical to 1.1).
- **Aggregation**: no new aggregates; only editorial fixes to the Sum/Group/set-function definitions, plus the `multiplicity` function replacing `card[Ω](μ)` ([§18.4](https://www.w3.org/TR/sparql12-query/#x18-4-basic-graph-patterns)), and tightened `SELECT *` restriction under implicit grouping.
- **Federated Query / SERVICE**: **no new SERVICE features.** [Federated Query Changes](https://www.w3.org/TR/sparql12-federated-query/) are editorial only ("RDF data model not data format", errata-fq-1 fix to the SERVICE+VALUES example, 1.1→1.2 reference bumps). Triple terms simply pass through `SERVICE`.
- **Results formats**: now carry triple terms and base direction (see §4 below).
- **Update**: triple-term/reifier syntax in `INSERT DATA`/`DELETE DATA`/templates; revised `LOAD` for documents containing a dataset ([Update Changes](https://www.w3.org/TR/sparql12-update/#a-changes-between-sparql-1-1-update-and-sparql-1-2-update)).

---

## 2. The grammar (verbatim, [§19.7](https://www.w3.org/TR/sparql12-query/#x19-7-grammar))

EBNF per XML 1.1 §6. Entry points: `QueryUnit`, `UpdateUnit`. LL(1) with uppercase terminals; longest-match tokenizing; keywords case-insensitive except `a`.

**The new/changed productions for triple terms, reifiers, annotations, version, and directional literals** (delimiters preserved exactly):

```
[4]   Prologue          ::= ( BaseDecl | PrefixDecl | VersionDecl )*
[7]   VersionDecl       ::= 'VERSION' VersionSpecifier
[8]   VersionSpecifier  ::= STRING_LITERAL1 | STRING_LITERAL2

[57]  TriplesBlock           ::= TriplesSameSubjectPath ( '.' TriplesBlock? )?
[58]  ReifiedTripleBlock     ::= ReifiedTriple PropertyList
[59]  ReifiedTripleBlockPath ::= ReifiedTriple PropertyListPath

[69]  DataBlockValue    ::= iri | RDFLiteral | NumericLiteral | BooleanLiteral | 'UNDEF' | TripleTermData
[70]  Reifier           ::= '~' VarOrReifierId?
[71]  VarOrReifierId    ::= Var | iri | BlankNode

[81]  TriplesSameSubject     ::= VarOrTerm PropertyListNotEmpty | TriplesNode PropertyList | ReifiedTripleBlock
[86]  Object            ::= GraphNode Annotation
[87]  TriplesSameSubjectPath ::= VarOrTerm PropertyListPathNotEmpty | TriplesNodePath PropertyListPath | ReifiedTripleBlockPath
[93]  ObjectPath        ::= GraphNodePath AnnotationPath

[109] AnnotationPath      ::= ( Reifier | AnnotationBlockPath )*
[110] AnnotationBlockPath ::= '{|' PropertyListPathNotEmpty '|}'
[111] Annotation          ::= ( Reifier | AnnotationBlock )*
[112] AnnotationBlock     ::= '{|' PropertyListNotEmpty '|}'

[113] GraphNode         ::= VarOrTerm | TriplesNode | ReifiedTriple
[114] GraphNodePath     ::= VarOrTerm | TriplesNodePath | ReifiedTriple
[115] VarOrTerm         ::= Var | iri | RDFLiteral | NumericLiteral | BooleanLiteral | BlankNode | NIL | TripleTerm

[116] ReifiedTriple        ::= '<<' ReifiedTripleSubject Verb ReifiedTripleObject Reifier? '>>'
[117] ReifiedTripleSubject ::= Var | iri | RDFLiteral | NumericLiteral | BooleanLiteral | BlankNode | ReifiedTriple | TripleTerm
[118] ReifiedTripleObject  ::= Var | iri | RDFLiteral | NumericLiteral | BooleanLiteral | BlankNode | ReifiedTriple | TripleTerm

[119] TripleTerm        ::= '<<(' TripleTermSubject Verb TripleTermObject ')>>'
[120] TripleTermSubject ::= Var | iri | RDFLiteral | NumericLiteral | BooleanLiteral | BlankNode | TripleTerm
[121] TripleTermObject  ::= Var | iri | RDFLiteral | NumericLiteral | BooleanLiteral | BlankNode | TripleTerm

[122] TripleTermData        ::= '<<(' TripleTermDataSubject ( iri | 'a' ) TripleTermDataObject ')>>'
[123] TripleTermDataSubject ::= iri
[124] TripleTermDataObject  ::= iri | RDFLiteral | NumericLiteral | BooleanLiteral | TripleTermData

[136] PrimaryExpression     ::= BrackettedExpression | BuiltInCall | iriOrFunction | RDFLiteral
                              | NumericLiteral | BooleanLiteral | Var | ExprTripleTerm
[137] ExprTripleTerm        ::= '<<(' ExprTripleTermSubject Verb ExprTripleTermObject ')>>'
[138] ExprTripleTermSubject ::= iri | Var
[139] ExprTripleTermObject  ::= iri | RDFLiteral | NumericLiteral | BooleanLiteral | Var | ExprTripleTerm

[149] RDFLiteral        ::= String ( LANG_DIR | '^^' iri )?
[165] LANG_DIR          ::= '@' [a-zA-Z]+ ('-' [a-zA-Z0-9]+)* ('--' [a-zA-Z]+)?
```

`BuiltInCall` [141] adds (verbatim): `… | 'LANGDIR' '(' Expression ')' | … | 'STRLANGDIR' '(' Expression ',' Expression ',' Expression ')' | … | 'hasLANG' '(' Expression ')' | 'hasLANGDIR' '(' Expression ')' | … | 'isTRIPLE' '(' Expression ')' | 'TRIPLE' '(' Expression ',' Expression ',' Expression ')' | 'SUBJECT' '(' Expression ')' | 'PREDICATE' '(' Expression ')' | 'OBJECT' '(' Expression ')'`.

**Where each form is allowed (the load-bearing constraints):**
- A **TripleTerm `<<( )>>`** is reachable via `VarOrTerm` [115] → it may sit in **subject or object** of a triple pattern *syntactically*, but matching against a graph only succeeds in object position (see §18.1.3 below). In data/expressions it's restricted: `TripleTermData` [122] (used in `VALUES`/quad-data) forbids variables and forbids non-IRI subjects; `ExprTripleTerm` [137] (in expressions/`BIND`) allows only IRI/var in subject and IRI/var in predicate.
- A **ReifiedTriple `<< >>`** is reachable via `GraphNode`/`GraphNodePath` [113][114] and `ReifiedTripleBlock` [58][59] → it can be a subject (it heads a property list) or an object. It carries an optional `Reifier` (`~ id`).
- **Annotation blocks `{| |}`** [110][112] attach to an `Object`/`ObjectPath` via `Annotation`/`AnnotationPath` [86][93][109][111] — a *sequence* of reifiers/blocks.
- Nesting: `ReifiedTripleSubject/Object` and `TripleTermSubject/Object` recurse into `ReifiedTriple`/`TripleTerm`, so arbitrary nesting is allowed syntactically.

**Grammar note (reifier/annotation restriction), quoted from §19.7:**
> A reifier or annotation syntax is only permitted after a triple when the property position is a simple path (an IRI, the keyword `a`, or a variable), and not for other path expressions.

Plain-text grammar file: linked at the end of §19.7 of the spec.

---

## 3. Algebra + evaluation semantics

### Triple-pattern definition ([§18.1.3](https://www.w3.org/TR/sparql12-query/#x18-1-3-triple-patterns)), verbatim:

> A triple pattern is a 3-tuple … If s is an RDF term, a variable, **or a triple pattern**; p is an IRI or a variable; and o is an RDF term, a variable, **or a triple pattern**; then (s,p,o) is a triple pattern. Triple patterns do not permit cycles…
>
> **Note** A triple pattern that has another triple pattern in its **subject** position will fail to match on any RDF graph because an RDF triple cannot have a triple term in its subject position.

So `<<( ... )>>` and `<< ... >>` in subject position are legal to *write* but never match (this is what `"1.2-basic"` forbids). Object-position triple terms match against triple-term objects in the data.

### Translation to algebra
- [§18.3.2.1 Expand Syntax Forms](https://www.w3.org/TR/sparql12-query/#x18-3-2-1-expand-syntax-forms): *"Expand abbreviations for IRIs and triple patterns given in Section 4."* — the reifier/annotation sugar (`<< >>`, `{| |}`) is expanded here into ordinary triple patterns using `rdf:reifies` (mechanics defined in [Turtle §2.11](https://www.w3.org/TR/rdf12-turtle/#x2-11-reifying-triples); the SPARQL spec defers the expansion algorithm to that). The `<<( )>>` triple term remains as a term node in the resulting triple pattern.
- [§18.3.2.5](https://www.w3.org/TR/sparql12-query/#x18-3-2-5-translate-basic-graph-patterns): adjacent triple patterns → `BGP(triples)`. Triple terms are just ordinary RDF terms inside those triples — **no new algebra operator is introduced**; matching reduces to subgraph matching.

### BGP matching ([§18.4.1](https://www.w3.org/TR/sparql12-query/#x18-4-1-sparql-basic-graph-pattern-matching)), verbatim:

> **Pattern Instance Mapping** P = combination of an RDF instance mapping σ (blank nodes→terms) and solution mapping μ (variables→terms): P(x) = μ(σ(x)).
> **Basic Graph Pattern Matching**: μ is a solution for BGP from G when there is a pattern instance mapping P such that **P(BGP) is a subgraph of G** and μ is the restriction of P to the query variables in BGP.
> `multiplicity(μ|Ω)` = number of distinct σ such that P=μ(σ) is a pattern instance mapping and P(BGP) is a subgraph of G.

**Implementer consequence:** matching a triple term is structural/recursive equality of triples (Concepts §3.6: triple-term equality = triple equality). A variable inside `<<( ?s ?p ?o )>>` binds to the corresponding component of a matched triple-term object. So a pattern like `?x :q <<( ?s ?p ?o )>>` joins `?s/?p/?o` against the components of any triple-term object — implementable as a recursive term-match in your BGP evaluator / WCOJ index (index triple terms as first-class terms; recurse for nested ones).

### `TRIPLE()`/`SUBJECT()` etc. in BIND & CONSTRUCT
- In `BIND`/`CONSTRUCT`, `TRIPLE(s,p,o)` (or `<<( )>>`) **constructs** a triple-term value; it errors if `(s,p,o)` isn't a well-formed triple (e.g. literal subject). `SUBJECT/PREDICATE/OBJECT` deconstruct.
- **CONSTRUCT dedup** (added note in §16.2): triples produced by CONSTRUCT are deduplicated (the result is a set/graph). Worth honoring when emitting reifying triples.

### Directional language strings, comparison/filter semantics
A `rdf:dirLangString` literal has lexical form + language tag + base direction (`ltr`/`rtl`) ([Concepts §3.4.3](https://www.w3.org/TR/rdf12-concepts/#x3-4-3-initial-text-direction)). Two such literals are `sameTerm` only if all three components match. `=`/`sameValue` over them follows the literal rules in §17.4.2.2. `ORDER BY` (§15.1) defines an in-between ordering slot for triple terms relative to other term types.

---

## 4. Result formats with triple terms / direction

**JSON** ([§3.2.2](https://www.w3.org/TR/sparql12-results-json/#x3-2-2-encoding-rdf-terms)):
```json
{"type":"triple","value":{"subject":S,"predicate":P,"object":O}}   // S/P/O recursively encoded
{"type":"literal","value":"S","xml:lang":"L","its:dir":"B"}        // directional literal: new its:dir key
```
Existing types: `uri`, `literal` (+ optional `xml:lang`/`datatype`), `bnode`.

**TSV** ([§4.2](https://www.w3.org/TR/sparql12-results-csv-tsv/)): triple terms written as `<<( subject predicate object )>>` using SPARQL/Turtle term syntax, components space-separated, recursive.
**CSV** ([§3.2](https://www.w3.org/TR/sparql12-results-csv-tsv/)): same `<<( … )>>` wrapper but components use `STR()`-style values; literal objects wrapped in `""`.
**XML**: `<triple>` element with nested `<subject>/<predicate>/<object>`; direction via an `its:dir`-aligned attribute (Results XML WD, the oldest at 27 Dec 2024).

---

## 5. Implementations to learn from

### spargebra / Oxigraph (Rust) — the direct upgrade path for sparq
- **spargebra v0.4.6** (2026-03-14) exposes a **`sparql-12` cargo feature** that parses SPARQL 1.2 incl. triple terms; also `standard-unicode-escaping`. ([docs.rs/spargebra](https://docs.rs/crate/spargebra/latest)). Its `spargebra::term` module already has `Triple`/`GroundTriple` structs and `Term`/`GroundTerm`/`TermPattern`/`GroundTermPattern` enums whose union covers triples — i.e. **triple-term AST nodes are present, gated behind the feature**. ([spargebra term module](https://docs.rs/spargebra/latest/spargebra/term/index.html)). **sparq can get SPARQL 1.2 algebra parsing largely for free by upgrading spargebra and enabling `sparql-12`** — the algebra stays "SPARQL 1.1 algebra objects" with triple terms as ordinary terms, so your BGP/FILTER/OPTIONAL/UNION/path/WCOJ machinery extends rather than gets replaced.
- **Oxigraph**: `rdf-star` feature **removed**; replaced by `rdf-12`/`sparql-12` in **0.5.0-beta.1 (2025-06-20)**, current **0.5.7 (2026-04-19)**. RDF 1.2 syntax differs from RDF-star and **drops triple terms in subject position**. ([oxigraph CHANGELOG](https://github.com/oxigraph/oxigraph/blob/main/CHANGELOG.md), [oxigraph SPARQL wiki](https://github.com/oxigraph/oxigraph/wiki/SPARQL)). Features off by default in the crate, on by default in CLI/Python/JS. This is the cleanest reference impl of the exact 1.2 matching semantics in Rust.

### Other engines (mostly still RDF-star era; migrating)
- **Apache Jena/ARQ**: mature RDF-star/SPARQL-star — quoted triples in Turtle/TriG/N-Triples/N-Quads parsers, TDB1/TDB2 + in-memory, JSON/XML result extensions, on by default in Fuseki; back-translation relies on one reification per quoted triple. ([Jena RDF-star](https://jena.apache.org/documentation/rdf-star/)). Watch for its 1.2 `rdf:reifies` migration.
- **Eclipse RDF4J**: experimental RDF-star/SPARQL-star; triple terms in object position aligned with RDF 1.2; not complete for every store. ([RDF4J RDF-star](https://rdf4j.org/documentation/programming/rdfstar/)).
- **Stardog / GraphDB**: RDF-star + SPARQL-star with triple-term JSON results (GraphDB docs: [rdf-sparql-star](https://graphdb.ontotext.com/documentation/10.8/rdf-sparql-star.html)).
- **QLever**: full SPARQL 1.1 (incl. Update + Graph Store Protocol) as of June 2025; 1.2/triple-term status not confirmed in current sources. ([QLever](https://github.com/ad-freiburg/qlever)).

### Conformance test suite — `w3c/rdf-tests`, `sparql/sparql12/`
The suite **exists in `main`** (the published HTML index still says "coming soon"). Run from the repo, not the rendered page. Directory layout (GitHub API on `w3c/rdf-tests`):
```
sparql/sparql12/
  manifest.ttl
  eval-triple-terms/        basic-*.{rq,srj}, construct-*.{rq,ttl}, data-*.ttl
  syntax-triple-terms-positive/   *.rq (incl. annotation-anonreifier-*, annotation blocks)
  syntax-triple-terms-negative/
  lang-basedir/             directional language string tests
  expression/  grouping/  codepoint-escapes/  version/  syntax/  rdf11/
```
Representative eval test (`eval-triple-terms/basic-2.rq` + `data-0-tripleterms.ttl`):
```sparql
PREFIX : <http://example/>
SELECT * { <<:a :b :c>> ?p ?o }       # reified-triple pattern in subject → never matches a graph
```
```turtle
:a :q <<(:a :b :c)>> .                                  # triple term as object
:f :g <<( :s :p <<(:x2 :y3 123 )>> )>> .                # nested triple term
```
Manifests are Turtle (`manifest.ttl`) with positive/negative syntax tests, `QueryEvaluationTest` entries pairing `.rq` + data + expected results (`.srj` JSON / `.ttl` for CONSTRUCT). Fetch raw via `raw.githubusercontent.com/w3c/rdf-tests/main/sparql/sparql12/...`. The suite is community-maintained; contribute via PR ([rdf-tests](https://github.com/w3c/rdf-tests), [test index](https://w3c.github.io/rdf-tests/)). RDF 1.2 conformance = relevant RDF 1.1 tests + RDF 1.2 tests.

---

## Implementer's punch-list for sparq

1. **Upgrade spargebra to ≥0.4.6, enable `sparql-12`** → parser yields triple-term AST nodes inside the existing 1.1 algebra. Lowest-effort path to a 1.2 front end.
2. **Term model**: add a triple-term variant to your RDF term type (recursive). Index it as a first-class term; triple-term equality = recursive triple equality.
3. **BGP/WCOJ**: extend matching to recurse into `<<( )>>` object terms; bind variables in components. No new algebra operator needed. Reject (empty-result) subject-position triple terms/patterns.
4. **Syntax expansion**: expand `<< s p o ~ id >>` and `{| p o |}` into `rdf:reifies` triples per Turtle §2.11 *before* algebra translation (allocate fresh bnodes where no reifier given; each reifier in an annotation block emits its own reifying triple).
5. **Functions**: implement `TRIPLE/SUBJECT/PREDICATE/OBJECT/isTRIPLE` and `LANGDIR/hasLANG/hasLANGDIR/STRLANGDIR`; rename `RDFterm-equal`→`sameValue` with the expanded triple-term/literal rules.
6. **Literals**: add base direction to language-tagged literals (`rdf:dirLangString`), parse `@lang--dir` via `LANG_DIR` [165].
7. **Prologue**: parse/validate `VERSION`; gate triple terms on `"1.2"` vs `"1.2-basic"`.
8. **Results**: emit `{"type":"triple",...}` + `its:dir` in JSON; `<<( … )>>` in CSV/TSV.
9. **Test**: clone `w3c/rdf-tests`, run `sparql/sparql12/manifest.ttl` (syntax pos/neg, eval-triple-terms, lang-basedir, version) alongside the sparql11 suite.

**Caveat:** everything in the SPARQL 1.2 *surface* is Working Draft (5 Jun 2026 for Query); the *data model* is CR. Pin to the dated drafts and re-diff before claiming conformance.