# Upstream w3c/rdf-canon: RDF-1.2 triple-term canonicalization drafts (4 items)

**Bead:** sq-63g0 (from sq-hslb / PR #933) · **Status:** drafts prepared, NOT yet filed upstream —
awaiting @jeswr review per the upstream-contribution protocol; when filed, file as **DRAFT** only ·
**Author:** SPARQ agent 🤖 [FABLE-5] · **Date:** 2026-07-17

## Context

While implementing the opt-in, NON-STANDARD `rdf12-triple-terms` canonicalization profile
(`crates/sparq-canon/src/rdf12.rs`, sq-hslb / PR #933), four spec-level ambiguities / gaps surfaced
that belong upstream, not in sparq. None of them is an error report against the current W3C
[RDFC-1.0](https://www.w3.org/TR/rdf-canon/) spec or its test suite — RDFC-1.0 is defined for the
RDF-1.1 data model and sparq's standard path passes the vendored suite byte-for-byte (see item 4's
grounding). They are all about what happens when the canon spec meets RDF-1.2 **triple terms**
(`<<( s p o )>>` as an object), which is currently **undefined** upstream
([w3c/rdf-star-wg#114](https://github.com/w3c/rdf-star-wg/issues/114)).

Items 1–2 and 4 target **w3c/rdf-canon**; item 3 may belong in **w3c/rdf-n-quads** (or the RDF-star
WG) — flagged per-item. Items 2 and 4 could be folded into item 1's issue at the maintainer's
discretion; they are drafted separately so each question stands alone.

Honesty note: sparq's profile is explicitly experimental and non-standard. The drafts below present
its design choices as *one possible answer* offered as a data point, never as the presumed outcome.

## Item 1 — w3c/rdf-canon issue: nested blank nodes in triple terms have no defined treatment

**Type:** DRAFT issue · **Repo:** w3c/rdf-canon · **Cross-refs:** w3c/rdf-star-wg#114

### Draft issue body

> **Title: RDF-1.2 triple terms: blank nodes nested inside a triple term have no defined
> treatment in RDFC-1.0**
>
> RDFC-1.0 is defined over the RDF-1.1 data model, where a blank node can only be a *component* of
> a quad in the subject, object, or graph-name position (§4.4 step 2 builds the blank-node-to-quads
> map from exactly those positions; §4.6/§4.8 hash over them). RDF-1.2 adds triple terms
> (`<<( s p o )>>` as an object), which can contain blank nodes at arbitrary nesting depth — and
> the spec currently has no rule for them: a bnode nested inside a triple term is not a
> "component" of any quad under the current definition, so it is never enrolled, hashed, or
> relabelled, and a naive implementation either mislabels it or must reject the input.
>
> This is the canonicalization face of w3c/rdf-star-wg#114. Two natural options:
>
> 1. **Fail closed** on any triple term containing a blank node (or on triple terms entirely) —
>    what conforming RDFC-1.0 implementations effectively do today, and a defensible v1 position.
> 2. **Extend the component notion**: a blank node occurring anywhere inside a quad's triple-term
>    object (at any depth, in the triple term's subject or object position) is a component of the
>    *containing quad*; the blank-node-to-quads map (§4.4 step 2), Hash First Degree Quads (§4.6,
>    including the `a`/`z` special relabelling), Hash Related Blank Node (§4.7), and the
>    Hash N-Degree Quads gossip (§4.8) all recurse through triple-term subject/object positions;
>    final serialization relabels nested bnodes with their issued `c14nN` labels in place.
>
> As a data point: sparq (a Rust RDF store) ships option 2 as an explicitly non-standard, opt-in
> experimental profile ("rdf12-triple-terms"). Implementation experience there: the extension is
> structurally small (the recursion reuses the §4.6–§4.8 machinery unchanged), it is byte-identical
> to standard RDFC-1.0 on any triple-term-free input, and the resulting canonical form is stable
> under blank-node relabelling and quad reordering in its test suite — but at least one further
> spec decision is forced immediately (the §4.7 position marker for nested bnodes; filed
> separately), which suggests the WG defining this once is better than implementations diverging.
>
> Is the WG's intent to define triple-term canonicalization in a future RDFC revision, and if so,
> is the "nested bnodes are components of the containing quad + recursive HNDQ" direction the
> intended shape?

### Notes for @jeswr

- sparq reference points: `crates/sparq-canon/src/rdf12.rs` module docs;
  `collect_bnodes_term` (the component-extension), `hash_n_degree_quads` (the recursive gossip).
- The draft deliberately does not claim the sparq design is *correct* in the RDFC-1.0
  uniqueness/soundness sense — no external review of the extension exists.

## Item 2 — w3c/rdf-canon issue: §4.7 position marker for triple-term-internal blank nodes

**Type:** DRAFT issue (or a section of item 1, maintainer's call) · **Repo:** w3c/rdf-canon

### Draft issue body

> **Title: Hash Related Blank Node position marker is unspecified for blank nodes nested inside
> triple terms**
>
> Assuming triple-term canonicalization gets defined (see the companion issue on nested blank
> nodes as quad components), §4.7 Hash Related Blank Node forces an immediate sub-decision. For
> RDF-1.1 positions the hashed input is `position + predicate + identifier` with position markers
> `s`/`o`/`g`. For a blank node nested *inside* a triple-term object there is no defined marker.
>
> The simplest extension — and what sparq's experimental profile does — is to treat any nested
> bnode as position `o` of the *containing quad*, hashing `"o" + containing-quad-predicate +
> identifier`. That conflates, for §4.7 purposes, a top-level object bnode with a triple-term-
> internal bnode related via the same predicate, e.g. `_:a <p> _:b .` vs
> `_:a <p> <<( <s> <q> _:b )>> .` both hash `_:b` as `o<p>…`. Since §4.7 hashes only feed the
> gossip's grouping/ordering (§4.8 explores and disambiguates via full first-degree/n-degree
> hashes), a coarser marker should affect work factor and label assignment, not stability — but
> the spec should say so, or say otherwise.
>
> Question for the WG: should the marker for nested bnodes carry a structural sub-discriminator —
> e.g. the *inner* predicate (the one syntactically adjacent to the bnode), a nesting depth, or a
> path of positions — rather than the bare containing-quad `o` + predicate? A finer marker
> discriminates earlier (fewer §4.8 permutations on adversarial inputs) at the cost of a more
> complex definition; either choice changes issued labels, so it must be pinned before any two
> implementations can agree byte-for-byte.

### Notes for @jeswr

- The sparq choice is `crates/sparq-canon/src/rdf12.rs` `hash_related_blank_node`:
  `Position::Object → format!("o{}", quad.predicate)` where `quad` is the containing quad — i.e.
  bare-`o`, no sub-discriminator. If upstream lands a finer marker, sparq's profile must migrate
  (a breaking change to issued labels under the profile; the standard path is unaffected).

## Item 3 — canonical N-Quads 1.2 token form: which normative reference will the canon spec cite?

**Type:** DRAFT issue · **Repo:** w3c/rdf-canon (possibly better routed to w3c/rdf-n-quads —
maintainer's call; the two are cross-linked in the draft)

### Draft issue body

> **Title: Canonical N-Quads token rules for RDF-1.2 forms (`<<( … )>>`, `@lang--dir`) — what
> will a triple-term-aware canonicalization cite?**
>
> RDFC-1.0's output is defined in terms of Canonical N-Quads
> ([n-quads §canonical-quads](https://www.w3.org/TR/n-quads/#canonical-quads)), which is RDF-1.1
> and token-exact by design — canonical hashes are computed over these bytes. RDF-1.2 N-Quads
> introduces new surface forms: the triple-term token `<<( s p o )>>` and the directional language
> tag `"…"@lang--dir`. As of this writing the RDF-1.2 N-Quads canonical-form section is not yet a
> stable normative reference, so an implementation experimenting with triple-term
> canonicalization has no pinned answer to byte-level questions such as: exact whitespace inside
> `<<( … )>>` (one space after `<<(` and before `)>>`?); case of the base direction token
> (`ltr`/`rtl`); language-tag case normalization interaction with `--dir`; and whether canonical
> escaping rules inside nested triple terms are exactly those of the containing line.
>
> For its experimental profile, sparq currently single-sources these tokens from the
> oxrdf/oxttl 0.3 serializer (Oxigraph's RDF-1.2 implementation) rather than hand-rolling them —
> i.e. it inherits whatever that serializer emits. That is a pragmatic pin, not a normative one.
>
> Question: once RDF-1.2 N-Quads is final, will Canonical N-Quads (as cited by a future
> triple-term-aware canon spec) normatively fix these token rules, and is the current RDF-1.2
> N-Quads draft's canonical section already stable enough for implementations to target? If there
> is a better venue for the token-level questions (rdf-n-quads issue tracker), happy to move this
> there.

### Notes for @jeswr

- sparq reference points: `rdf12.rs` `serialize_quad_line` / `canonical_line_no_newline` (both are
  thin over oxrdf-0.3 `Display`); module docs "The serialization re-uses oxrdf-0.3's canonical
  `Display` … so the token rules are single-sourced in oxttl/oxrdf rather than hand-rolled here."
- If oxrdf's emission ever diverges from the final RDF-1.2 canonical form, sparq's profile output
  changes across an oxrdf upgrade — worth a pinned-vector regression test when the spec lands
  (candidate follow-up bead once upstream answers).

## Item 4 — w3c/rdf-canon test suite: no RDF-1.2 triple-term vectors (gap, not an error)

**Type:** DRAFT issue · **Repo:** w3c/rdf-canon · **Blocked on:** items 1–2 (semantics first)

### Draft issue body

> **Title: Test suite: proposal to add RDF-1.2 triple-term eval/map vectors once triple-term
> semantics are agreed**
>
> Not an error report — the suite is correct and complete for RDFC-1.0's RDF-1.1 scope, and it
> passes byte-for-byte in our implementation. This is a forward-looking gap: the suite contains
> zero vectors exercising RDF-1.2 triple terms (`<<( … )>>`), so any implementation experimenting
> with triple-term canonicalization ahead of the spec has no official conformance anchor, and two
> such implementations have no way to detect divergence.
>
> Once the WG settles triple-term semantics (see the nested-blank-node and position-marker
> issues), proposal: add eval (`testNNN-in.nq` → `testNNN-rdfc10.nq`) and issued-identifier map
> (`testNNN-rdfc10map.json`) vectors covering at least — a ground triple term (no nested bnodes);
> a bnode in the triple term's subject and in its object; nesting depth ≥ 2; the same bnode label
> occurring both top-level and triple-term-internally; two quads distinguishable only through
> their triple-term internals (forcing the gossip through the recursion); a
> `"…"@lang--dir` literal inside a triple term (pinning the canonical token form); and a negative
> vector for whatever the spec rejects. Happy to contribute candidate vectors under the W3C test
> suite licenses once semantics are agreed.

### Grounding (verified in-repo, 2026-07-17)

- Vendored snapshot: `crates/sparq-canon/tests/rdf-canon-testdata/` at upstream commit
  `15619df2fda7a4ca88308733789b6774517f9638` (2026-02-24), 150 files under `rdfc10/`.
- `grep -rl '<<(' tests/rdf-canon-testdata/` → **0 files**: no triple-term vector exists.
- Byte-for-byte pass: `tests/rdf_canon_suite.rs` (manifest-driven, standard path) and
  `tests/rdf12_triple_term_canon.rs` invariant 1 (the v2 profile agrees byte-identically with the
  standard path on every suite eval vector).

## Next steps (after @jeswr review — nothing filed yet)

1. @jeswr reviews the four draft bodies above (wording, venue for item 3, whether items 2/4 fold
   into item 1).
2. On approval: file each as a **DRAFT** issue on w3c/rdf-canon (item 3 possibly on
   w3c/rdf-n-quads), cross-referencing w3c/rdf-star-wg#114, and @jeswr-tag before marking ready
   for upstream maintainers. Do NOT file non-draft.
3. Record the upstream issue URLs back in this file and in the `rdf12` module docs, and open
   follow-up beads for whatever upstream decides (position-marker migration, pinned token-form
   regression vectors, contributed suite vectors).

@jeswr — please review the four draft bodies above **before** anything is opened on w3c/rdf-canon
(upstream-contribution protocol; sq-63g0).
