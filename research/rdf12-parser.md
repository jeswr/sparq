# RDF 1.2 — Implementer's Reference for a Spec-Compliant Rust Parser

Scope: the RDF 1.2 abstract data model and its concrete syntaxes (N-Triples, N-Quads, Turtle, TriG), targeted at building an RDF 1.2 parser in Rust for "sparq" (currently oxrdf/oxttl + a custom N-Triples byte parser). Every grammar production is quoted from W3C primary sources; every claim is cited. **All status info is as of the June 2026 snapshot of the specs.**

> **One-paragraph orientation.** RDF 1.2 is the official W3C successor to the Community-Group "RDF-star". It replaces RDF-star's single ambiguous `<< s p o >>` quoted-triple construct with **two distinct things**: (1) a **triple term** `<<( s p o )>>` — an unasserted RDF term that *is* a triple, allowed **only in object position**; and (2) **reification sugar** `<< s p o >>` / `~reifier` / `{| ... |}` which desugars to `reifier rdf:reifies <<( s p o )>>` plus the asserted base triple. It also adds **base direction** on language strings (`"text"@lang--dir`, datatype `rdf:dirLangString`). This is the single biggest reason you cannot treat RDF 1.2 as "RDF-star with cosmetic changes": the term model and the syntax both changed.

---

## 0. Specification inventory, URLs, and maturity (Section 4 of the brief — read this first)

The RDF 1.2 suite is **split in maturity**. This matters because you build the *data model / semantics* against stable CR docs, but the *concrete-syntax grammars* are still Working Drafts and can shift.

| Document | URL | Status (June 2026) |
|---|---|---|
| **RDF 1.2 Concepts and Abstract Data Model** | https://www.w3.org/TR/rdf12-concepts/ | **Candidate Recommendation Snapshot, 07 April 2026** |
| **RDF 1.2 Semantics** | https://www.w3.org/TR/rdf12-semantics/ | **Candidate Recommendation Snapshot, 07 April 2026** |
| **RDF 1.2 Schema** (vocabulary) | https://www.w3.org/TR/rdf12-schema/ | **Working Draft, 28 March 2026** |
| **RDF 1.2 N-Triples** | https://www.w3.org/TR/rdf12-n-triples/ | **Working Draft, 15 May 2026** |
| **RDF 1.2 N-Quads** | https://www.w3.org/TR/rdf12-n-quads/ | **Working Draft, 01 June 2026** |
| **RDF 1.2 Turtle** | https://www.w3.org/TR/rdf12-turtle/ | **Working Draft, 28 May 2026** |
| **RDF 1.2 TriG** | https://www.w3.org/TR/rdf12-trig/ | **Working Draft, 01 June 2026** |

Editor's drafts (often ahead of TR): `https://w3c.github.io/rdf-concepts/spec/`, `/rdf-turtle/spec/`, `/rdf-n-triples/spec/`, `/rdf-trig/spec/`, `/rdf-schema/spec/`.

W3C's CR call for implementations (Concepts + Semantics): "The RDF & SPARQL Working Group invites implementations of the following two Candidate Recommendation Snapshots." (https://www.w3.org/news/2026/w3c-invites-implementations-of-rdf-1-2-concepts-and-abstract-data-model-and-rdf-1-2-semantics/) The Concepts CR "is not expected to advance to Recommendation any earlier than 05 May 2026" (https://www.w3.org/TR/rdf12-concepts/).

**Stability guidance for the implementer:**
- **Stable to build against now:** the *abstract model* (triple terms, `rdf:reifies` reification, base direction) and *semantics* — these are at CR.
- **In flux (track the editor's drafts):** the **concrete-syntax grammars** (N-Triples, N-Quads, Turtle, TriG) are all still **Working Drafts**. The token productions for `<<( )>>`, `<<`/`>>`, `~`, `{| |}`, and `LANG_DIR` are settled enough that Oxigraph, Jena 5.4, and RDF4J already implement them, but production *numbering* and minor edge cases (e.g. exact whitespace handling around `<<(`) can still change. Pin to a dated WD and re-diff before declaring conformance.
- **Conformance levels** (Concepts): "Full conformance supports graphs and datasets with triples that contain triple terms. Basic conformance only supports … basic RDF terms." (https://www.w3.org/TR/rdf12-concepts/) A parser that rejects triple terms is still a *Basic*-conformant RDF 1.2 processor.

---

## 1. Data model changes vs RDF 1.1

### 1.1 The four kinds of RDF term (was three)

RDF 1.2 Concepts: RDF terms are "**IRIs, literals, blank nodes, and triple terms**", and "IRIs, literals and blank nodes are said to be **basic RDF terms**." (https://www.w3.org/TR/rdf12-concepts/) The triple term is the new fourth kind.

### 1.2 Triple terms — the abstract syntax

A triple term is an RDF triple used **as a term**:

> "An RDF triple used as the object of another triple is called a **triple term**. In a given RDF graph, a triple can appear as a triple term, an asserted triple, or both." (https://www.w3.org/TR/rdf12-concepts/)

Asserted-vs-term distinction:

> "An RDF triple that is an element of an RDF graph is also said to be **asserted** in that RDF graph." and "By using **non-asserted triple terms** … one can make statements about unasserted statements; for example, if one is unsure whether a given proposition actually holds." (https://www.w3.org/TR/rdf12-concepts/)

**Critical structural constraint (parser-load-bearing):** a triple term is *only* an object. The abstract syntax for the triple a term wraps is "If s is an IRI or a blank node, p is an IRI, and o is an RDF triple, then (s, p, o) is an RDF triple" — so **the triple-term's own subject must be an IRI or blank node** (not a triple term), while its **object may itself be a triple term** (nesting). This is confirmed by the concrete grammar (§2) and by the Jena implementors: "Triple terms are only permitted in the object position, and **unlike the work of the RDF-star CG, triple terms are not valid in the subject position**." (https://jena.apache.org/documentation/rdf-star/, via apache/jena issue #2805)

### 1.3 How this differs from RDF-1.1-era RDF-star "quoted triples"

| Aspect | RDF-star (CG, ~2021) | RDF 1.2 (WG, current) |
|---|---|---|
| Syntax for a triple-as-term | `<< s p o >>` | `<<( s p o )>>` |
| Asserted? | A quoted triple was **not** asserted, but `<< s p o >>` in subject **and** object positions both existed | Triple term is **never** asserted; allowed in **object only** |
| Subject position | Quoted triple allowed as subject | **Forbidden** as subject |
| Reification mechanism | Quoted triple *is* the identifier (term identity) | Separate **reifier** node linked via `rdf:reifies` to the triple term |
| Annotation `{| |}` | annotated the quoted triple directly | desugars to `reifier rdf:reifies <<( … )>>` + annotation triples on the reifier |

Oxigraph's own summary: "RDF 1.2 includes triple terms using a slightly different syntax as RDF-star … raw triple terms are now written `<<( ts tp to )>>` … and not `<< s p o >>`." (https://github.com/oxigraph/oxigraph/releases/tag/v0.5.0)

### 1.4 The reification vocabulary (`rdf:reifies`, reifier, triple annotation)

> "A **reifying triple** is a triple where the predicate is `rdf:reifies` and the object is a triple term. The subject of that triple is called a **reifier**, and it can be the subject or object of other triples." (https://www.w3.org/TR/rdf12-concepts/)

> "The subset of triples including the reifier as subject … is called a **triple annotation**" (when the triple term corresponds to an asserted triple). (https://www.w3.org/TR/rdf12-concepts/)

Vocabulary terms and IRIs (from RDF 1.2 Schema, https://www.w3.org/TR/rdf12-schema/):
- **`rdf:reifies`** = `http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies`, `rdf:type rdf:Property`, **`rdfs:domain rdfs:Resource`**, **`rdfs:range rdfs:Proposition`**, "associates a resource with a proposition". 
  - ⚠️ **Note the discrepancy to resolve in code:** the Schema doc gives the *range as `rdfs:Proposition`* (the semantic class of propositions, `http://www.w3.org/2000/01/rdf-schema#Proposition`). Triple terms denote propositions, so syntactically the object of `rdf:reifies` is always a triple term, but the declared `rdfs:range` is the semantic class `rdfs:Proposition`, **not** a syntactic `rdf:TripleTerm` class. Some secondary sources loosely say "range `rdf:TripleTerm`" — do not rely on that; the normative Schema says `rdfs:Proposition`.
- **`rdf:TripleTerm`** — as of these drafts, **not defined as a vocabulary class** in the Schema or namespace document. "Triple term" is an abstract-syntax concept, not a vocabulary IRI. Do not emit/expect an `rdf:TripleTerm` class.
- **`rdf:langString`** = `…#langString`, `rdf:type rdfs:Datatype`, "The datatype of language-tagged string values".
- **`rdf:dirLangString`** = `…#dirLangString`, `rdf:type rdfs:Datatype`, "directional language-tagged string values".
- **Classic reification** (`rdf:Statement`, `rdf:subject`, `rdf:predicate`, `rdf:object`) is **retained, not deprecated** — it appears in RDF 1.2 Schema §"Old-style Reification" (non-normative), with all terms keeping their domains/ranges and **no deprecation notice**. (https://www.w3.org/TR/rdf12-schema/)

### 1.5 Directional language-tagged strings (base direction)

> A literal has "a base direction that MUST be one of the following: `ltr`, indicating … left-to-right; `rtl`, indicating … right-to-left." (https://www.w3.org/TR/rdf12-concepts/)
> "A literal is a **directional language-tagged string** if both the language tag and the base direction are present." Such literals get datatype IRI `rdf:dirLangString`; literals with a language tag but no direction keep `rdf:langString`. (https://www.w3.org/TR/rdf12-concepts/)

So the literal-datatype rule a parser must implement:
- string + `@lang` (no direction) → `rdf:langString`
- string + `@lang--dir` → `rdf:dirLangString` (and you must store the direction)
- string + `^^<dt>` → `dt`
- plain string → `xsd:string`

### 1.6 Semantics (for correctness, not parsing)

RDF 1.2 Semantics (CR, https://www.w3.org/TR/rdf12-semantics/) interprets a ground triple term via an injective triple-extension mapping: "if E is a ground triple term, then `I(E) = IT(I(E.s), I(E.p), I(E.o))`" where "IT [is] an injective mapping from IR × IP × IR into IR." Reification is handled through RDFS with `rdfs:Proposition` and an entailment rule relating `<<( aaa bbb ccc )>>` appearing in a triple to its proposition. A parser doesn't need this, but it confirms the term-identity model behind `rdf:reifies`.

---

## 2. The exact grammars (this is what you implement)

### 2.1 N-Triples 1.2 — full grammar delta

Source: https://www.w3.org/TR/rdf12-n-triples/ (WD 15 May 2026) and editor's draft BNF https://w3c.github.io/rdf-n-triples/spec/ntriples.bnf. Productions quoted verbatim:

```
[1]  ntriplesDoc      ::= statement? (EOL statement)* EOL?
[2]  statement        ::= directive | triple
[3]  directive        ::= versionDirective
[4]  versionDirective ::= 'VERSION' versionSpecifier
[5]  versionSpecifier ::= STRING_LITERAL_QUOTE
[6]  triple           ::= subject predicate object '.'
[7]  subject          ::= IRIREF | BLANK_NODE_LABEL
[8]  predicate        ::= IRIREF
[9]  object           ::= IRIREF | BLANK_NODE_LABEL | literal | tripleTerm
[10] literal          ::= STRING_LITERAL_QUOTE ('^^' IRIREF | LANG_DIR)?
[11] tripleTerm       ::= '<<(' subject predicate object ')>>'
```
Terminals (unchanged from 1.1 except `LANG_DIR`):
```
[13] IRIREF               ::= '<' ([^#x00-#x20<>"{}|^`\] | UCHAR)* '>'
[14] BLANK_NODE_LABEL     ::= '_:' (PN_CHARS_U | [0-9]) ((PN_CHARS | '.')* PN_CHARS)?
[15] LANG_DIR             ::= '@' [a-zA-Z]+ ('-' [a-zA-Z0-9]+)* ('--' [a-zA-Z]+)?
[16] STRING_LITERAL_QUOTE ::= '"' ([^#x22#x5C#x0A#x0D] | ECHAR | UCHAR)* '"'
[17] UCHAR                ::= ('\u' HEX HEX HEX HEX) | ('\U' HEX HEX HEX HEX HEX HEX HEX HEX)
[18] ECHAR                ::= '\' [tbnrf\"']
[23] EOL                  ::= [#x0D#x0A]+
```

**Delta vs N-Triples 1.1:**
1. `object` gains a fourth alternative `tripleTerm` (`[9]`).
2. New `tripleTerm ::= '<<(' subject predicate object ')>>'` (`[11]`). Because it reuses `[7] subject` (= `IRIREF | BLANK_NODE_LABEL`) and `[9] object`, **the triple-term subject cannot be a triple term, but the triple-term object recursively can** — i.e. nesting is right-branching only.
3. The old `langtag` token is replaced by `LANG_DIR` (`[15]`), adding the optional `('--' [a-zA-Z]+)?` direction suffix.
4. New optional `VERSION "1.2"` directive (`[2]`–`[5]`) at the top of a document.

Spec prose: "A triple term is represented as a `tripleTerm` with `subject`, `predicate`, and `object` preceded by `<<(` and followed by `)>>`." "Note that triple terms may be nested." "A triple term may be the object of an RDF triple." (https://w3c.github.io/rdf-n-triples/spec/)

### 2.2 N-Quads 1.2 — delta

Source: https://www.w3.org/TR/rdf12-n-quads/ (WD 01 June 2026):
```
[1]  nquadsDoc  ::= statement? (EOL statement)* EOL?
[2]  statement  ::= directive | quad
[6]  quad       ::= subject predicate object graphLabel? '.'
[7]  subject    ::= IRIREF | BLANK_NODE_LABEL
[8]  predicate  ::= IRIREF
[9]  object     ::= IRIREF | BLANK_NODE_LABEL | literal | tripleTerm
[10] graphLabel ::= IRIREF | BLANK_NODE_LABEL
[11] literal    ::= STRING_LITERAL_QUOTE (('^^' IRIREF) | LANG_DIR)?
[12] tripleTerm ::= '<<(' subject predicate object ')>>'
[16] LANG_DIR   ::= '@' [a-zA-Z]+ ('-' [a-zA-Z0-9]+)* ('--' [a-zA-Z]+)?
```
N-Quads = N-Triples + an **optional 4th `graphLabel`** before the `.` (`[6]`). The triple-term and `LANG_DIR` deltas are identical to N-Triples. **The graph label itself cannot be a triple term** (only `IRIREF | BLANK_NODE_LABEL`), and triple terms still appear in object position only.

### 2.3 Turtle 1.2 — full grammar (new productions in bold context)

Source: https://www.w3.org/TR/rdf12-turtle/ (WD 28 May 2026), §6.5. Verbatim:
```
[1]  turtleDoc            ::= statement*
[2]  statement            ::= directive | ( triples '.' )
[3]  directive            ::= prefixID | base | version | sparqlPrefix | sparqlBase | sparqlVersion
[4]  prefixID             ::= '@prefix' PNAME_NS IRIREF '.'
[5]  base                 ::= '@base' IRIREF '.'
[6]  version              ::= '@version' VersionSpecifier '.'
[7]  sparqlPrefix         ::= "PREFIX" PNAME_NS IRIREF
[8]  sparqlBase           ::= "BASE" IRIREF
[9]  sparqlVersion        ::= "VERSION" VersionSpecifier
[10] VersionSpecifier     ::= STRING_LITERAL_QUOTE | STRING_LITERAL_SINGLE_QUOTE
[11] triples              ::= ( subject predicateObjectList )
                            | ( blankNodePropertyList predicateObjectList? )
                            | ( reifiedTriple predicateObjectList? )
[12] predicateObjectList  ::= verb objectList ( ';' ( verb objectList )? )*
[13] objectList           ::= object annotation ( ',' object annotation )*
[14] verb                 ::= predicate | 'a'
[15] subject              ::= iri | BlankNode | collection
[16] predicate            ::= iri
[17] object               ::= iri | BlankNode | collection | blankNodePropertyList | literal | tripleTerm | reifiedTriple
[18] literal              ::= RDFLiteral | NumericLiteral | BooleanLiteral
[19] blankNodePropertyList::= '[' predicateObjectList ']'
[20] collection           ::= '(' object* ')'
[21] NumericLiteral       ::= INTEGER | DECIMAL | DOUBLE
[22] RDFLiteral           ::= String ( LANG_DIR | ( '^^' iri ) )?
[23] BooleanLiteral       ::= 'true' | 'false'
[24] String               ::= STRING_LITERAL_QUOTE | STRING_LITERAL_SINGLE_QUOTE
                            | STRING_LITERAL_LONG_SINGLE_QUOTE | STRING_LITERAL_LONG_QUOTE
[25] iri                  ::= IRIREF | PrefixedName
[26] PrefixedName         ::= PNAME_LN | PNAME_NS
[27] BlankNode            ::= BLANK_NODE_LABEL | ANON
[28] reifier              ::= '~' ( iri | BlankNode )?
[29] reifiedTriple        ::= '<<' rtSubject verb rtObject reifier? '>>'
[30] rtSubject            ::= iri | BlankNode | reifiedTriple
[31] rtObject             ::= iri | BlankNode | literal | tripleTerm | reifiedTriple
[32] tripleTerm           ::= '<<(' ttSubject verb ttObject ')>>'
[33] ttSubject            ::= iri | BlankNode
[34] ttObject             ::= iri | BlankNode | literal | tripleTerm
[35] annotation           ::= ( reifier | annotationBlock )*
[36] annotationBlock      ::= '{|' predicateObjectList '|}'
[42] LANG_DIR             ::= '@' [a-zA-Z]+ ( '-' [a-zA-Z0-9]+ )* ( '--' [a-zA-Z]+ )?
```
Selected terminals (note the new `<<(`/`)>>`/`{|`/`|}`/`~` tokens; rest unchanged from Turtle 1.1):
```
[38] IRIREF                ::= '<' ( [^#x00-#x20<>"{}|^`\] | UCHAR )* '>'
[41] BLANK_NODE_LABEL      ::= '_:' ( PN_CHARS_U | [0-9] ) ( ( PN_CHARS | '.' )* PN_CHARS )?
[43] INTEGER               ::= [+-]? [0-9]+
[44] DECIMAL               ::= [+-]? ( [0-9]* '.' [0-9]+ )
[45] DOUBLE                ::= [+-]? ( ( [0-9]+ ( '.' [0-9]* )? ) | ( '.' [0-9]+ ) ) EXPONENT
[51] UCHAR                 ::= ( '\u' HEX HEX HEX HEX ) | ( '\U' HEX HEX HEX HEX HEX HEX HEX HEX )
[52] ECHAR                 ::= '\' [tbnrf\"']
[54] ANON                  ::= '[' WS* ']'
```

**Delta vs Turtle 1.1:**
- `triples` (`[11]`) gains a `reifiedTriple predicateObjectList?` alternative — a reified triple can stand at subject position of a statement.
- `objectList` (`[13]`) now is `object annotation (...)` — **every object can carry a trailing `annotation`**.
- `object` (`[17]`) gains `tripleTerm` and `reifiedTriple`.
- `RDFLiteral` (`[22]`) uses `LANG_DIR` instead of `LANGTAG`.
- New productions: `reifier [28]`, `reifiedTriple [29]`, `rtSubject [30]`, `rtObject [31]`, `tripleTerm [32]`, `ttSubject [33]`, `ttObject [34]`, `annotation [35]`, `annotationBlock [36]`, plus `version`/`sparqlVersion`/`VersionSpecifier` directives.

**Two distinct `<<` lexemes** — your tokenizer must distinguish:
- `<<(` … `)>>` = **triple term** (`tripleTerm`, `[32]`) — produces an unasserted triple-term term.
- `<<` … `>>` = **reified triple** (`reifiedTriple`, `[29]`) — sugar; asserts the inner triple AND links a reifier.

Note the asymmetry: inside a `reifiedTriple` (`<< … >>`) the subject may itself be a `reifiedTriple` (`[30] rtSubject`), but inside a `tripleTerm` (`<<( … )>>`) the subject may **only** be `iri | BlankNode` (`[33] ttSubject`). Object recurses into `tripleTerm` in both.

### 2.4 TriG 1.2 — delta

Source: https://www.w3.org/TR/rdf12-trig/ (WD 01 June 2026):
```
[1] trigDoc        ::= (directive | block)*
[2] block          ::= triplesOrGraph | wrappedGraph | triples2 | ("GRAPH" labelOrSubject wrappedGraph)
[3] triplesOrGraph ::= (labelOrSubject (wrappedGraph | (predicateObjectList '.')))
                     | (reifiedTriple predicateObjectList? '.')
[4] triples2       ::= (blankNodePropertyList predicateObjectList? '.') | (collection predicateObjectList '.')
[5] wrappedGraph   ::= '{' triplesBlock? '}'
[6] triplesBlock   ::= triples ('.' triplesBlock?)?
[7] labelOrSubject ::= iri | BlankNode
```
"RDF 1.2 TriG shares the reifying triples and annotation syntax extensions with RDF 1.2 Turtle" — i.e. the same `reifiedTriple [34]`, `tripleTerm [37]`, `annotation [40]`, `annotationBlock [41]` productions (renumbered). (https://www.w3.org/TR/rdf12-trig/) So: implement the Turtle term/annotation machinery once and reuse it for TriG, adding only the graph-block wrapping.

### 2.5 Reification / annotation desugaring — the exact triples produced

These desugaring rules are the heart of the Turtle/TriG parser. All examples quoted from https://www.w3.org/TR/rdf12-turtle/.

**(a) Bare reified triple as object** — `reifiedTriple` in `rtObject`/object position **asserts the inner triple** and yields a reifier:
```
<< :employee38 :jobTitle "Assistant Designer" ~ _:id >> :accordingTo :employee22 .
```
expands to:
```
_:id rdf:reifies <<( :employee38 :jobTitle "Assistant Designer" )>> .
_:id :accordingTo :employee22 .
```
(And the inner triple `:employee38 :jobTitle "Assistant Designer"` is asserted separately when the reified triple appears in a *statement/assertion* context — see the spec's worked examples; the `reifiedTriple`'s role is to mint the reifier and the `rdf:reifies` link.)

**Reifier defaulting:** "If no reifiers are present, or a reifier is not immediately followed by an iri or BlankNode, **a fresh RDF blank node is allocated**, as with `<< :s :p :o >>` or `<< :s :p :o ~ >>`." (https://www.w3.org/TR/rdf12-turtle/)

**(b) Annotation with explicit reifier + block:**
```
:alice :name "Alice" ~ :t {| :statedBy :bob ; :recorded "2021-07-07"^^xsd:date |} .
```
expands to:
```
:alice :name "Alice" .
:t rdf:reifies <<( :alice :name "Alice" )>> .
:t :statedBy :bob .
:t :recorded "2021-07-07"^^xsd:date .
```

**(c) Annotation block, implicit reifier (fresh blank node):**
```
:alice :name "Alice" {| :statedBy :bob |} .
```
expands to:
```
:alice :name "Alice" .
_:b0 rdf:reifies <<( :alice :name "Alice" )>> .
_:b0 :statedBy :bob .
```
"If such blocks are not immediately preceded by explicit reifiers, each block is associated with a **fresh RDF blank node** allocated as its reifier." (https://www.w3.org/TR/rdf12-turtle/)

**(d) Multiple reifiers on one triple** — each mints its own reifying triple, all pointing at the **same** triple term:
```
:alice :name "Alice" ~ :stmt1 ~ :stmt2 .
```
→
```
:alice :name "Alice" .
:stmt1 rdf:reifies <<( :alice :name "Alice" )>> .
:stmt2 rdf:reifies <<( :alice :name "Alice" )>> .
```
Multiple annotation blocks likewise each get their own fresh blank node, all reifying the same triple term.

**Implementation algorithm for the annotation in `objectList`:** when you parse `object annotation`, record the current `(curSubject, curPredicate, curObject)`. For each `reifier` or `annotationBlock` in the `annotation`: construct the triple term `<<( curSubject curPredicate curObject )>>`; determine the reifier (explicit iri/blank node, or fresh blank node); emit `reifier rdf:reifies <<( … )>>`; then for an `annotationBlock`, parse its `predicateObjectList` with **the reifier as the subject**. The base triple `curSubject curPredicate curObject` is itself asserted. (Synthesized from §"reification"/§7.3.3 of https://www.w3.org/TR/rdf12-turtle/.)

---

## 3. Lexical / parser pitfalls to get right

1. **Two `<<` tokens.** `<<(` opens a triple term; `<<` (not followed by `(`) opens a reified triple. Tokenize `<<(` and `)>>` as atomic; do not let a generic `<<` rule swallow the `(`. Closing tokens `)>>` vs `>>` likewise differ.
2. **Triple terms are object-only.** Reject `<<( … )>>` in subject (Turtle `subject [15]` has no `tripleTerm`; N-Triples `subject [7]` is `IRIREF|BLANK_NODE_LABEL`). Reject a triple term as a graph label (N-Quads `graphLabel [10]`) and as a predicate (predicate is always `iri`/`IRIREF`).
3. **Asymmetric nesting.**
   - Triple-term subject: `iri | BlankNode` only (`ttSubject [33]` / N-Triples `subject`). No nested triple term in subject.
   - Triple-term object: may recurse into `tripleTerm` (`ttObject [34]` / N-Triples `object`). Nesting is unbounded and right-branching — guard recursion depth to avoid stack overflow on adversarial input.
   - `reifiedTriple` subject **may** nest another `reifiedTriple` (`rtSubject [30]`), unlike triple terms.
4. **Reified triples interact with `predicateObjectList` and blank-node property lists.** In Turtle, `<< s p o >>` may be a statement subject (`triples [11]`) and may appear as an `object` (`[17]`). A `blankNodePropertyList` `[ … ]` and a `collection` `( … )` may appear as objects but **not** inside a `tripleTerm` (its object is `iri | BlankNode | literal | tripleTerm` only — no collections / property lists). So `<<( :s :p ( 1 2 ) )>>` is **invalid**; `:s :p ( 1 2 )` with an annotation is fine.
5. **`LANG_DIR` validation.** Lexically `'@' [a-zA-Z]+ ('-' [a-zA-Z0-9]+)* ('--' [a-zA-Z]+)?`. The language subtag part follows BCP 47; the `--` introduces the **direction**, which must be exactly `ltr` or `rtl` (the grammar accepts any `[a-zA-Z]+`, so **you must add a semantic check** rejecting anything other than `ltr`/`rtl`). Examples: `"foo"@en` (langString), `"foo"@en-US--ltr` (dirLangString, ltr), `"שלום"@he--rtl`. Direction without a language is not expressible (the `@` always introduces a language tag first).
6. **Datatype assignment** (unchanged shape, new branch): `@lang` → `rdf:langString`; `@lang--dir` → `rdf:dirLangString` + store direction; `^^<dt>` → `dt`; bare → `xsd:string`.
7. **Whitespace inside `<<( )>>` and `{| |}`.** Treat `<<(`, `)>>`, `{|`, `|}`, `~` as distinct multi-char tokens but allow normal Turtle whitespace/comments between the inner terms. `~` may be followed by whitespace then an optional `iri|BlankNode`; a `~` with nothing parseable after it means "fresh blank node".
8. **IRI resolution, numeric/boolean literals, escaping, UCHAR/ECHAR** — **unchanged from RDF 1.1**. `INTEGER`/`DECIMAL`/`DOUBLE`/`BooleanLiteral`, `\u`/`\U` escapes, string-quote forms, `@base`/`PREFIX` relative-IRI resolution all carry over verbatim. The only terminal-level change is `LANGTAG → LANG_DIR`. New optional `VERSION "1.2"` / `@version` directive should be accepted (and may be ignored) but is not required.
9. **`a` keyword** still expands to `rdf:type` in `verb`; it is **not** allowed inside a `tripleTerm`/`reifiedTriple` predicate slot in N-Triples/N-Quads (those use raw `IRIREF`), but **is** allowed in Turtle/TriG `verb` inside `<<( ttSubject verb ttObject )>>` (`[32]` uses `verb`).

---

## 4. Existing implementations to learn from

### 4.1 Oxigraph (oxrdf / oxttl / oxrdfio) — the path of least resistance for sparq

- RDF 1.2 ships in **Oxigraph v0.5.0** (released ~13 Sept) behind feature flags **`rdf-12`** (data model/parsing) and **`sparql-12`**: "Support for current W3C working drafts RDF 1.2 and SPARQL 1.2, hidden behind the `rdf-12` or `sparql-12` features." These **replace the old `rdf-star` feature**. Disabled by default in the `oxigraph` crate; enabled by default in Python/JS bindings and CLI. (https://github.com/oxigraph/oxigraph/releases/tag/v0.5.0)
- **oxrdf API for triple terms:** the `Term` enum is `NamedNode | BlankNode | Literal | Triple(Box<Triple>)`, where the `Triple` variant is "Available on crate feature `rdf-12` only" and "the union of IRIs, blank nodes, literals and triples (if the `rdf-12` feature is enabled)." (https://docs.rs/oxrdf/latest/oxrdf/enum.Term.html) So a **triple term is modeled as `Term::Triple(Box<Triple>)`**. Consistent with the spec's object-only rule, `Subject` does **not** gain a triple variant under `rdf-12` (this is the key change from the old RDF-star model, where subjects could be triples).
- **Base direction:** oxrdf exposes a `BaseDirection` enum (gated on `rdf-12`) described as "A directional language-tagged string base-direction," paired with `Literal` to represent `rdf:dirLangString`. (https://docs.rs/oxrdf/latest/oxrdf/)
- **oxttl** "provides N-Triple, N-Quad, Turtle, TriG and N3 parsing and serialization" and under `rdf-12` parses the `<<( )>>` / `<<` `>>` / `~` / `{| |}` / `LANG_DIR` syntax.
- **Migration caveat:** oxigraph issue #1286 ("Migration from RDF-star to RDF 1.2", opened 24 May 2025) tracks how stored RDF-star triple terms (which allowed subject triples) map onto the RDF 1.2 model; the migration of *on-disk* data is non-trivial, but **for a fresh parser this is not your problem** — you just consume the RDF 1.2 grammar.

**Bottom line for sparq:** you can almost certainly get RDF 1.2 **"for free" by upgrading to oxttl/oxrdf 0.5.x and enabling the `rdf-12` feature**, rather than hand-rolling. Your existing custom N-Triples byte parser only needs to learn (a) `<<( … )>>` in object position with right-branching nesting, and (b) the `LANG_DIR` `--dir` suffix → `rdf:dirLangString` — both are small, localized changes. If you keep the hand-rolled fast path for plain N-Triples, gate the triple-term/dir-lang branches behind a feature so the hot path stays branch-light.

### 4.2 Apache Jena

- **Jena 5.4.0** ships an experimental **RDF 1.2 preview**: "Turtle, Trig, N-Triples, N-Quads and SPARQL parsing have been updated for triple terms and initial text direction" (RDF/XML excluded). New Model-API class **`StatementTerm`** for triple terms; **`org.apache.jena.system.RDFStar`** provides RDF-star ⇄ reification translation helpers for migration. "Triple terms are only permitted in the object position … not valid in the subject position." (https://jena.apache.org/documentation/rdf-star/; apache/jena issue #2805; PR "Changes to the Model API for RDF 1.2 triple terms and triple reifiers")

### 4.3 Eclipse RDF4J

- "RDF4J's model API now uses RDF 1.2 triple-term terminology, with legacy RDF-star parser/writer formats replaced by corresponding triple-term APIs and formats, and triple terms are supported in object position." SPARQL 1.2 conformance suite is enabled for MemoryStore; NativeStore/LMDBStore "still have known gaps for triple-term and base-direction support." (https://github.com/eclipse-rdf4j/rdf4j/discussions/4963; https://rdf4j.org/documentation/programming/rdfstar/)

### 4.4 N3.js / RDF.js

- N3.js targets RDF 1.2: "A reifier creates a reference to an occurrence of the abstract triple term, e.g. `:r rdf:reifies <<( :s :p :o )>>`," and supports the Turtle annotation shortcut `:s :p :o {| :a :b |} .`. It implements reifiers and the `rdf:reifies` desugaring in JS. (https://github.com/rdfjs/N3.js — README; https://rdf.js.org/N3.js/docs/N3Parser.html) Useful as a reference for the desugaring algorithm and edge-case test vectors. (You are in this very repo, so its `lib/N3Parser.js` reification/annotation handling is a directly-readable reference implementation.)

### 4.5 serd

- No authoritative RDF 1.2 support information surfaced; serd historically tracked RDF-star quoted triples. Do not assume serd reflects the current `<<( )>>` model — verify against its current source before using it as a reference.

---

## Appendix: minimal conformance reference for the sparq parser

The implementation work is tracked in beads (`bd list -l area:sparq-core`); this
is the spec digest the beads reference, kept here as design reference (not a task
list). RDF 1.2 surface-syntax requirements for the parser:

- Tokenize `<<(`, `)>>`, `<<`, `>>`, `~`, `{|`, `|}` as distinct lexemes; `LANG_DIR` with optional `--dir`.
- N-Triples/N-Quads: accept `tripleTerm` in object only; right-branching nesting; depth guard.
- N-Quads: optional 4th `graphLabel` (IRI/blank only).
- Turtle/TriG: `reifiedTriple` (`<< … >>`) with optional `~reifier`, desugaring to base triple + `reifier rdf:reifies <<( … )>>`; fresh blank-node reifier when omitted.
- Turtle/TriG: `{| … |}` annotation blocks on each object; multiple reifiers/blocks each mint a reifier over the **same** triple term.
- Reject triple terms in subject/predicate/graph-label; reject collections/blankNodePropertyLists inside `tripleTerm`.
- `@lang--dir` → `rdf:dirLangString` + stored direction; validate `dir ∈ {ltr, rtl}`; `@lang` → `rdf:langString`.
- Accept (may ignore) `VERSION "1.2"` / `@version` directives.
- Everything else (IRI resolution, numerics, booleans, escapes, UCHAR/ECHAR, prefixes/base) unchanged from RDF 1.1.

### Primary sources
- RDF 1.2 Concepts (CR): https://www.w3.org/TR/rdf12-concepts/
- RDF 1.2 Semantics (CR): https://www.w3.org/TR/rdf12-semantics/
- RDF 1.2 Schema (WD): https://www.w3.org/TR/rdf12-schema/
- RDF 1.2 N-Triples (WD): https://www.w3.org/TR/rdf12-n-triples/ · BNF: https://w3c.github.io/rdf-n-triples/spec/ntriples.bnf
- RDF 1.2 N-Quads (WD): https://www.w3.org/TR/rdf12-n-quads/
- RDF 1.2 Turtle (WD): https://www.w3.org/TR/rdf12-turtle/
- RDF 1.2 TriG (WD): https://www.w3.org/TR/rdf12-trig/
- W3C CR call for implementations: https://www.w3.org/news/2026/w3c-invites-implementations-of-rdf-1-2-concepts-and-abstract-data-model-and-rdf-1-2-semantics/
- Oxigraph v0.5.0 release: https://github.com/oxigraph/oxigraph/releases/tag/v0.5.0 · oxrdf docs: https://docs.rs/oxrdf/latest/oxrdf/ · migration issue #1286: https://github.com/oxigraph/oxigraph/issues/1286
- Jena RDF-star/1.2: https://jena.apache.org/documentation/rdf-star/ · issue #2805: https://github.com/apache/jena/issues/2805
- RDF4J: https://github.com/eclipse-rdf4j/rdf4j/discussions/4963 · https://rdf4j.org/documentation/programming/rdfstar/
- N3.js: https://github.com/rdfjs/N3.js

---

## Report summary

I produced the full reference document above (returned inline, not as a file, per instructions). Key findings the implementer must act on:

1. **Two `<<` constructs, not one.** RDF 1.2 splits RDF-star's `<< s p o >>` into a **triple term `<<( s p o )>>`** (an unasserted term, **object-position only**) and **reification sugar `<< s p o >>` / `~reifier` / `{| … |}`** that desugars to `reifier rdf:reifies <<( s p o )>>` plus the asserted base triple. Verbatim grammar productions for N-Triples, N-Quads, Turtle, and TriG are quoted with production numbers.

2. **Object-only + asymmetric nesting.** Triple-term subject is `iri|BlankNode` only; triple-term object may recurse into a triple term. `reifiedTriple` subjects *may* nest other reified triples; triple terms may **not** nest in subject, and may not contain collections/blank-node property lists.

3. **Base direction.** `LANG_DIR ::= '@' [a-zA-Z]+ ('-' [a-zA-Z0-9]+)* ('--' [a-zA-Z]+)?`; `@lang--dir` → datatype `rdf:dirLangString` with `dir ∈ {ltr,rtl}` (grammar permits any letters, so a semantic check is required); `@lang` → `rdf:langString`.

4. **Maturity is split:** Concepts + Semantics are at **Candidate Recommendation (07 Apr 2026)** — stable to build the model against; **all four concrete syntaxes are still Working Drafts (May–Jun 2026)** — settled enough to implement but pin to a dated draft. Classic reification vocabulary is retained (non-normative), not deprecated; `rdf:reifies` has `rdfs:domain rdfs:Resource` / `rdfs:range rdfs:Proposition` (not a syntactic `rdf:TripleTerm` class — that class is not defined).

5. **sparq can likely get RDF 1.2 for free** by upgrading to **oxttl/oxrdf 0.5.x with the `rdf-12` feature** (which replaced `rdf-star`); triple terms are `Term::Triple(Box<Triple>)`, base direction via a `BaseDirection` enum. The only changes to your hand-rolled N-Triples byte parser are the object-position `<<( … )>>` branch (with depth-guarded right-branching recursion) and the `LANG_DIR` `--dir` suffix. Jena 5.4 (`StatementTerm`), RDF4J, and the local N3.js parser are good reference implementations of the desugaring algorithm.

Most load-bearing local file for cross-reference: this repo's `lib/N3Parser.js` already implements the RDF 1.2 reifier/annotation desugaring in JS.