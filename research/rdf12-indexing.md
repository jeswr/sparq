# How RDF Engines Store and Index Triple Terms (RDF-star / RDF 1.2) — A Design Reference for `sparq`

This report is a primary-source reference for designing the on-disk + in-memory index for triple terms in `sparq` (Rust; dictionary-encoded `u32` term ids; six sorted permutation indexes of `[u32;3]` — SPO/SOP/PSO/POS/OSP/OPS; mmap out-of-core mode). Every non-obvious claim is cited.

---

## 0. Critical framing change: RDF 1.2 ≠ old RDF-star (read this first)

The single most consequential fact for your index design is that **RDF 1.2 narrowed where a triple term may appear**. In the original RDF-star / SPARQL-star model (Hartig & Thompson, "Reification Done Right", 2014), an embedded triple `<<s p o>>` could occur in **both subject and object** position. RDF 1.2 changed the syntax to `<<( s p o )>>` and **restricts triple terms to the object position only**:

- RDF 1.2 N-Triples grammar: `subject ::= IRIREF | BLANK_NODE_LABEL`, but `object ::= IRIREF | BLANK_NODE_LABEL | literal | tripleTerm`. A triple term is written `<<( subject predicate object )>>` and "may be nested." ([w3.org/TR/rdf12-n-triples](https://www.w3.org/TR/rdf12-n-triples/))
- RDF 1.2 Concepts: "a triple term is an RDF triple used as the object of another triple… triple terms can only appear in the object position." Triple terms are full RDF terms ("IRIs, literals, blank nodes, and triple terms are collectively known as RDF terms"), and a triple term "need not be asserted." ([w3.org/TR/rdf12-concepts](https://www.w3.org/TR/rdf12-concepts/))
- The intended pattern is the **reifier**: a *reifying triple* has predicate `rdf:reifies` and a triple term as object; the subject (the "reifier") is an ordinary IRI/bnode that you then make further statements about. "Triple terms always denote abstract, logical propositions, while reifiers may denote a variety of things… it is expected that the reifiers (rather than the triple terms) will be used in further statements." ([w3.org/TR/rdf12-concepts](https://www.w3.org/TR/rdf12-concepts/), [W3C WG issue #152](https://github.com/w3c/rdf-star-wg/issues/152))
- Oxigraph's own changelog confirms the model break explicitly: "RDF 1.2 does not support triple terms in the subject position anymore." ([Oxigraph CHANGELOG, 0.5.0-beta.1, 2025-06-20](https://github.com/oxigraph/oxigraph/blob/main/CHANGELOG.md))

**Why this matters for `sparq`:** under RDF 1.2 a triple term is never a *subject*. It only ever appears as an *object*. So the question "can a triple term flow through all six permutation indexes as an ordinary `u32`?" has a simpler answer than under old RDF-star: a triple-term id is just another object id. The hard part is only the *back-mapping* (ttid → components) needed to evaluate `<<( ?s ?p ?o )>>` patterns. (Your design should still keep the option open for old-style subject-position embedding if you want SPARQL-star/property-graph compatibility, but the standards-compliant target is object-only.)

---

## 1. The core representation problem

A triple term is a *term that is itself a triple*. You must give it something that flows through the term machinery (so `:r rdf:reifies <<( s p o )>>` is storable/queryable like any quad), **and** keep its `(s,p,o)` components indexable so `<<( ?s ?p ?o )>> ... ` / `:r rdf:reifies <<( ?s ?p ?o )>>` can match by component binding. Two strategies dominate in real systems.

### (a) Term-as-id — encode the triple into the dictionary / inline it as a value
The triple term becomes a first-class encoded term. There are two sub-variants seen in the wild:

- **Inline recursive encoding (no separate id).** The triple term is stored *inline* in the index entry as a recursively-encoded `(s,p,o)` blob. Oxigraph does exactly this: its `EncodedTerm` has a `Triple(Arc<EncodedTriple>)` variant where `EncodedTriple { subject, predicate, object: EncodedTerm }` recurses, and "RDF-star triples are stored inline of these indexes in the subject > predicate > object order." Only the *string components* of the nested triple are pushed to the `id2str` table — the triple itself gets no separate string id. ([Oxigraph Architecture wiki](https://github.com/oxigraph/oxigraph/wiki/Architecture); [numeric_encoder.rs](https://github.com/oxigraph/oxigraph/blob/main/lib/oxigraph/src/storage/numeric_encoder.rs)) Blazegraph's "Reification Done Right" does the same thing for statement-identifiers (SIDs): it uses "variable length and recursively embedded encodings of the Subject and Object of a statement" inlined directly into the statement indices. ([Blazegraph RDR wiki](https://github.com/blazegraph/database/wiki/Reification_Done_Right))
- **Hash/opaque-IRI id (a real dictionary id).** The triple term is mapped to one scalar id via a deterministic function of its components, and a side table recovers the components. RDF4J/GraphDB serialize an embedded triple as a special IRI `urn:rdf4j:triple:xxx`, where `xxx` is the Base64url encoding of the N-Triples of the embedded triple — i.e. the triple is folded into a single dictionary key. ([rdf4j.org/documentation/programming/rdfstar](https://rdf4j.org/documentation/programming/rdfstar/); [GraphDB 11.3 RDF-star docs](https://graphdb.ontotext.com/documentation/11.3/rdf-sparql-star.html)) Stardog's `identifier()` "generates an IRI based on input RDF terms (subject, predicate, object…) by hashing the input terms." ([Stardog Edge Properties](https://docs.stardog.com/query-stardog/edge-properties))

**Tradeoffs of (a):**
- *Inline recursive*: zero extra id-space, perfect for *outbound* lookups (given a ttid/inline-blob, you already have the components, so `rdf:reifies <<( ?s ?p ?o )>>` projection is free). Cost: index keys become *variable-length* (a triple-term object expands a 4-byte slot into an embedded triple), which breaks fixed-width `[u32;3]` assumptions and complicates merge-joins; and matching `<<( ?s ?p ?o )>>` by *component binding* still needs a way to find which entries carry a triple term with given components. Blazegraph notes this is "very efficient if you have a small number of statements (<5) about each RDF statement" and degrades as the fan-out grows. ([Blazegraph RDR wiki](https://github.com/blazegraph/database/wiki/Reification_Done_Right))
- *Hash/opaque id*: keeps fixed-width ids (great for your `[u32;3]` design) but you **must** keep a ttid → (s,p,o) side table to decode, and a component → ttid index to match `<<( ?s ?p ?o )>>` patterns; hashing also risks collisions (Oxigraph uses 128-bit SipHash for its string hashes to keep collision probability negligible — a 32-bit id would be far too small for content-hashing). ([Oxigraph Architecture wiki](https://github.com/oxigraph/oxigraph/wiki/Architecture))

### (b) Reification-style / separate triple-term table
Keep triple terms in a *dedicated* table keyed by `(s_id, p_id, o_id) → ttid` with its own indexes, separate from the main quad indexes. This is the classic reification layout (one statement → its own row + indexes) and is what GraphDB's Wikidata numbers compare against: native embedded-triple storage shrank the repo to 22,465 MB vs 36,768 MB for standard reification — a ~39% reduction — showing the dedicated-reification-table approach is the *baseline* the inline approaches beat on size. ([GraphDB 11.3 RDF-star docs](https://graphdb.ontotext.com/documentation/11.3/rdf-sparql-star.html))

**Tradeoffs of (b):** clean separation (the main `[u32;3]` indexes never change shape); component-pattern matching is a normal index probe on the side table; ids are dense `u32`. Cost: an extra table + its own permutation indexes, and every triple-term reference does an extra join through the table. This is essentially strategy (a)-hash with the side table promoted to a first-class, independently-indexed structure — and for a fixed-width engine like yours it is the cleaner of the two.

---

## 2. How real engines do it

| Engine | Strategy | Storage of the triple term | Notes / numbers |
|---|---|---|---|
| **Oxigraph** (Rust; RocksDB + in-mem) | (a) inline recursive | `EncodedTerm::Triple(Arc<EncodedTriple>)`; stored *inline* in the quad indexes in s>p>o order; nested triples recurse; only leaf strings go to `id2str`. 11 RocksDB tables: `id2str`, a named-graph list, and 9 quad-order tables (spo, pos, osp, spog, posg, ospg, gspo, gpos, gosp). | RDF-star dropped in favor of RDF 1.2 ("triple terms"); subject-position triple terms removed; DBs auto-migrate on first open in 0.5. ([Architecture](https://github.com/oxigraph/oxigraph/wiki/Architecture); [numeric_encoder.rs](https://github.com/oxigraph/oxigraph/blob/main/lib/oxigraph/src/storage/numeric_encoder.rs); [CHANGELOG](https://github.com/oxigraph/oxigraph/blob/main/CHANGELOG.md)) |
| **Apache Jena TDB2** | (a) term-as-id, *no on-disk change* | A node table maps every RDF term to an 8-byte NodeId; B+tree indexes over triple/quad permutations of those NodeIds. "Current support for RDF-star has **not made any changes to the on-disk data structures except adding the new RDF term type**." Quoted triples become a new node kind in the node table. | Quoted triples treated as a `Resource` carrying a `Statement`. Known slowness: a nested SPARQL-star pattern `<<<<?s ?p ?o>> ?a ?b>> ?x ?y` took ~639s on 9.4M triples on TDB2 4.3.2 (seconds on GraphDB), because matching unbound triple-term components forces broad scans. ([Jena RDF-star docs (search snapshot)](https://jena.apache.org/documentation/rdf-star/); [TDB architecture](https://jena.apache.org/documentation/tdb/architecture.html); [survey arXiv:2102.13027](https://arxiv.org/pdf/2102.13027); [Jena issue #1744](https://github.com/apache/jena/issues/1744)) |
| **RDF4J** (native / LMDB / memory) | (a) hash/opaque IRI | Core model adds a new `Triple` Resource type. Non-RDF-star serializations encode an embedded triple as `urn:rdf4j:triple:<Base64url(N-Triples)>`. The native `TripleStore` stores statements as four integer ids (s,p,o,context) into a `ValueStore`. | RDF-star quoted triples supported in the **memory** store; **native and LMDB stores still have gaps** for triple-term / base-direction support. ([rdf4j.org rdfstar](https://rdf4j.org/documentation/programming/rdfstar/); [native TripleStore.java](https://github.com/eclipse-rdf4j/rdf4j-storage/blob/master/nativerdf/src/main/java/org/eclipse/rdf4j/sail/nativerdf/TripleStore.java); [RDF4J discussion #4963](https://github.com/eclipse-rdf4j/rdf4j/discussions/4963)) |
| **GraphDB** (Ontotext) | (a) new RDF type, ref-only | Embedded triple is a new RDF type stored as a *single* triple; **does not assert** the referenced statement. Same `urn:rdf4j:triple:` Base64 encoding for non-star formats. Component access via `rdf:subject/predicate/object`, type test `rdf:isTriple`, construction `rdf:Statement`. | ~39% smaller than standard reification on Wikidata (22,465 vs 36,768 MB). Performance was "seconds" where Jena took 639s. ([GraphDB 11.3 RDF-star](https://graphdb.ontotext.com/documentation/11.3/rdf-sparql-star.html)) |
| **Stardog** | (a) hash id | Edge properties (7.1+) built on RDF-star with **storage-layer + query-engine changes**; `identifier()` makes an IRI by hashing s,p,o(,g). | Designed to avoid reification's triple/pattern blow-up. ([Stardog Edge Properties](https://docs.stardog.com/query-stardog/edge-properties)) |
| **Blazegraph** (the original RDF-star, "RDR") | (a) inline recursive SID | A "statement identifier" per statement, **inlined recursively** into the SPO/POS/OSP statement indices with variable-length encoding; statements bind to `<< >>` variables directly with no materialized reified rows. | "Very efficient… <5 statements about each RDF statement"; degrades vs plain reification as fan-out grows. ([Blazegraph RDR wiki](https://github.com/blazegraph/database/wiki/Reification_Done_Right); [wiki.blazegraph.com RDR](https://wiki.blazegraph.com/wiki/index.php/Reification_Done_Right)) |
| **AnzoGraph** | (a), in-memory LPG | RDF-star edge-property syntax `<< s p o >> prop val`; all-in-memory with disk backup not used for queries. Hard cap of **255 property values per edge**. ([AnzoGraph LPG docs](https://docs.cambridgesemantics.com/anzograph/v3.1/userdoc/lpgs.htm)) |
| **Virtuoso** | none (historically) | No RDF-star/SPARQL-star support as of the community discussion. ([OpenLink community thread](https://community.openlinksw.com/t/virtuoso-support-for-rdf-star-and-sparql-star/2280)) |
| **QLever** | not documented for RDF-star | Achieved full SPARQL 1.1 compliance (June 2025); indexing encodes common identifiers directly into numeric ids to skip string lookups, scaling past a trillion triples — but no documented RDF-star/triple-term support found. ([QLever GitHub](https://github.com/ad-freiburg/qlever); [Wikipedia](https://en.wikipedia.org/wiki/QLever)) |

**Papers worth citing in your design doc:**
- Hartig & Thompson, *Foundations of RDF★ and SPARQL★ / "Reification Done Right"* (2014) — the origin of the embedded-triple model and SPARQL-star semantics. ([arXiv:1406.3399](https://arxiv.org/pdf/1406.3399); [Semantic Scholar](https://www.semanticscholar.org/paper/Foundations-of-RDF%E2%8B%86-and-SPARQL%E2%8B%86-(An-Alternative-to-Hartig/36e70ee51cb7b7ec12faac934ae6b6a4d9da15a8))
- Weiss, Karras, Bernstein, *Hexastore: Sextuple Indexing for Semantic Web Data Management* (VLDB 2008) — the canonical justification for the 6 permutations you already maintain: "six indexes for all triple permutations (spo, sop, pso, pos, osp, ops)," dictionary-encoded, so every triple pattern hits a covering index and first joins become merge-joins, at ~5× index space. ([cs.au.dk/~karras/hexastore.pdf](https://www.cs.au.dk/~karras/hexastore.pdf))
- *A Survey of RDF Stores & SPARQL Engines* (arXiv:2102.13027) — background on dictionary encoding + permutation indexing across RDF-3X, Virtuoso, Jena, etc. ([arXiv:2102.13027](https://arxiv.org/pdf/2102.13027))
- W3C RDF 1.2 Concepts / N-Triples / Primer — the normative triple-term & reifier model. ([Concepts](https://www.w3.org/TR/rdf12-concepts/); [N-Triples](https://www.w3.org/TR/rdf12-n-triples/))

---

## 3. Indexing for triple-term PATTERN MATCHING

The query you must serve is, e.g., `:r rdf:reifies <<( ?s ?p ?o )>> .` (RDF 1.2) or the SPARQL-star `<<( ?s ?p ?o )>> :saidBy :alice` — both reduce to: *find triple-term ids whose `(s,p,o)` match a (possibly partial) binding, then join them into the outer pattern.*

Two index directions are needed:

1. **ttid → (s,p,o)** (decode / project) — needed whenever a bound triple term must be expanded to bind `?s ?p ?o`, or when projecting/serializing. Inline-recursive engines (Oxigraph, Blazegraph) get this for free because the components live in the index entry; hash-id engines (RDF4J/GraphDB/Stardog) need an explicit side table. ([Oxigraph numeric_encoder.rs](https://github.com/oxigraph/oxigraph/blob/main/lib/oxigraph/src/storage/numeric_encoder.rs); [Blazegraph RDR](https://github.com/blazegraph/database/wiki/Reification_Done_Right))

2. **(s,p,o) → ttid** (component lookup) — needed when the triple term's components are *bound* (fully or partially) and you must find the matching triple-term id(s). With *partial* bindings (e.g. `<<( :alice ?p ?o )>>`) you need the same permutation logic over `(s,p,o,ttid)` that you already use over `(s,p,o)` — which is exactly why Jena, lacking targeted triple-term component indexes, scans broadly and is slow (639s nested-pattern case). ([Jena issue #1744](https://github.com/apache/jena/issues/1744))

**Nesting:** a triple term may contain triple terms ("triple terms may be nested" — [N-Triples spec](https://www.w3.org/TR/rdf12-n-triples/)). Both inline-recursive (Oxigraph's `Arc<EncodedTriple>` recurses) and id-based (the inner triple term first gets its own ttid, which is then a component id of the outer one) handle this; the id-based approach handles deep nesting with *fixed-width* entries, which is the better fit for `sparq`. Note Jena's pathological case was precisely a *doubly nested* pattern.

**Does the existing 6-permutation `[u32;3]` design extend?** Yes — and cleanly, *because RDF 1.2 puts triple terms only in object position*:
- A reifying quad `:r rdf:reifies <<(s p o)>>` is just an ordinary triple `(r_id, reifies_id, ttid)` where `ttid` is the object id. It flows through all six existing permutation indexes unchanged — no new index needed for the *outer* statement.
- The *only* genuinely new structures are the two directional maps for the triple term itself: **ttid → (s,p,o)** and **(s,p,o) → ttid**. The forward map is a single sorted array indexed by (ttid − base) → `[u32;3]`. The reverse map, to support partial-component patterns, is best served by reusing your permutation machinery on a 4-tuple `(s,p,o,ttid)` — but in practice you rarely need all six orders here (see §4).

So: triple-term ids become ordinary `u32`s flowing through the existing indexes (outer statements need *nothing* new), plus essentially **one** new bidirectional triple-term table. This is materially simpler than the old RDF-star case (where subject-position embedding would have forced triple-term ids through the S-leading permutations as well).

---

## 4. Concrete recommendation for `sparq`

Adopt **strategy (a)-hash-id specialized as a dedicated, dense-id triple-term table** — the cleanest match for a fixed-width, dict-encoded, mmap engine. Concretely:

**4.1 Allocate triple-term ids from a tagged `u32` range.**
Reserve the high bit (or a high range) of the `u32` id space as the *triple-term tag*, exactly as you already tag the inline-integer range. A bound id with the tag set means "this is a triple term; look it up in the triple-term table," analogous to how Oxigraph/QLever fold meaning into the id bits to skip lookups ([QLever indexing](https://github.com/ad-freiburg/qlever); Oxigraph inline encodings, [Architecture](https://github.com/oxigraph/oxigraph/wiki/Architecture)). This keeps every index entry a fixed `[u32;3]` — no variable-length keys, preserving your merge-join and mmap layout. Budget: a 1-bit tag leaves 2^31 triple terms; if that's tight, use a 2-bit term-kind tag (IRI / literal-ish / triple-term / inline) and 30-bit payload.

**4.2 Add exactly one new on-disk structure: the triple-term table, with two faces.**
- **Forward (ttid → [s,p,o])**: a flat, mmap-friendly array of `[u32;3]` indexed by `ttid − tt_base`. O(1) decode; trivially out-of-core. This is your "node table for triple terms," mirroring Jena's node-table-only change ([Jena RDF-star](https://jena.apache.org/documentation/rdf-star/)).
- **Reverse ((s,p,o) → ttid)**: to *intern* triple terms (dedupe on build) and to answer fully-bound `<<( s p o )>>`, keep one sorted index of `[s,p,o,ttid]` ordered SPO. Dedup at build time = content interning, so identical triple terms share one ttid (matching Oxigraph/Blazegraph identity-by-components, [numeric_encoder.rs](https://github.com/oxigraph/oxigraph/blob/main/lib/oxigraph/src/storage/numeric_encoder.rs)).

**4.3 Cover partial-component patterns with at most two more orders — not six.**
Pure `<<( ?s ?p ?o )>>` with *all three* unbound is answered by scanning the forward array (or, more usefully, by joining through the outer `rdf:reifies` pattern, which restricts you to only the ttids that are actually referenced). For *partially*-bound triple-term patterns you only need orders led by the bound positions. In practice the common bindings are subject-led and object-led, so **SPO + OPS over `(s,p,o,ttid)` covers the realistic cases**; add POS if predicate-led triple-term lookup matters for your workloads. You do **not** need the full sextuple here because triple terms are a small minority of terms and are reached primarily by joining from their referencing statement, not by free 3-variable enumeration. (Contrast Jena, whose lack of any such targeted index is the documented cause of the 639s blow-up — [issue #1744](https://github.com/apache/jena/issues/1744).)

**4.4 Query/join flow for `:r rdf:reifies <<( ?s ?p ?o )>>`:**
1. Probe the existing PSO/POS index for `(?, reifies_id, ?)` (or bind `:r`) → yields `(r_id, ttid)` pairs; `ttid` is tagged.
2. For each `ttid`, O(1) forward-array lookup → `[s,p,o]`, binding `?s ?p ?o`.
3. If `?s/?p/?o` are partially pre-bound, instead drive from the reverse `(s,p,o,ttid)` index (SPO/POS/OPS) to get candidate ttids, then probe the outer pattern. Standard merge/hash join — the triple-term id behaves as an ordinary object id throughout, so your existing join operators are unchanged.

**4.5 Nesting:** intern inner triple terms first (bottom-up) so an inner ttid is a normal component id inside the outer triple term's `[s,p,o]`. Deep nesting stays fixed-width and reuses the same two maps recursively. This sidesteps the variable-length-key problem that inline-recursive engines (Oxigraph, Blazegraph) accept in exchange for free decode.

**4.6 Impact on build/query paths (kept minimal):**
- *Build*: add a triple-term interning pass (hash map `[s,p,o] → ttid`, allocate tagged ids, append to forward array, populate reverse index). Then encoding proceeds exactly as today — referencing statements are ordinary triples with a tagged object id. No change to the six main permutation builders.
- *Query*: one new operator — "expand ttid" (forward lookup) and "match triple-term pattern" (reverse lookup). Everything else (the six indexes, merge-joins, mmap iteration) is untouched because the triple-term id is just a `u32`.
- *mmap/out-of-core*: the forward array and the reverse SPO(/OPS/POS) index are plain sorted `[u32;k]` slices — identical in spirit to your existing permutation files, so they memory-map and stream the same way.

**Net:** because RDF 1.2 confines triple terms to object position ([RDF 1.2 Concepts](https://www.w3.org/TR/rdf12-concepts/); [Oxigraph CHANGELOG](https://github.com/oxigraph/oxigraph/blob/main/CHANGELOG.md)), `sparq`'s dict-encoded `[u32;3]` + 6-permutation + mmap architecture extends with **one tagged id range** and **one bidirectional triple-term table** (forward `ttid→[s,p,o]` array + a reverse `[s,p,o,ttid]` index in 1–3 orders). No change to the existing six indexes, no variable-length keys, and triple-term ids flow through your current joins as ordinary `u32`s — capturing Oxigraph/Jena/Blazegraph's "minimal node-table change" insight while avoiding Jena's missing-component-index performance trap.

---

## Sources

- Oxigraph Architecture wiki — term encoding, inline triple storage, RocksDB tables: https://github.com/oxigraph/oxigraph/wiki/Architecture
- Oxigraph `numeric_encoder.rs` — `EncodedTerm::Triple(Arc<EncodedTriple>)`, recursion, id2str: https://github.com/oxigraph/oxigraph/blob/main/lib/oxigraph/src/storage/numeric_encoder.rs
- Oxigraph CHANGELOG — RDF-star → RDF 1.2, subject-position drop, migration: https://github.com/oxigraph/oxigraph/blob/main/CHANGELOG.md
- Apache Jena RDF-star support: https://jena.apache.org/documentation/rdf-star/
- Apache Jena TDB architecture (node table, B+tree permutations): https://jena.apache.org/documentation/tdb/architecture.html
- Jena issue #1744 — slow SPARQL-star nested pattern (639s): https://github.com/apache/jena/issues/1744
- RDF4J RDF-star programming docs — `Triple` type, `urn:rdf4j:triple:` encoding: https://rdf4j.org/documentation/programming/rdfstar/
- RDF4J native `TripleStore.java` — four-integer-id statements: https://github.com/eclipse-rdf4j/rdf4j-storage/blob/master/nativerdf/src/main/java/org/eclipse/rdf4j/sail/nativerdf/TripleStore.java
- RDF4J discussion #4963 — native/LMDB store RDF-star gaps: https://github.com/eclipse-rdf4j/rdf4j/discussions/4963
- GraphDB 11.3 RDF-star docs — embedded triple as new type, size numbers, functions: https://graphdb.ontotext.com/documentation/11.3/rdf-sparql-star.html
- Stardog Edge Properties — storage-layer changes, `identifier()` hashing: https://docs.stardog.com/query-stardog/edge-properties
- Blazegraph "Reification Done Right" wiki — SIDs, recursive inline encoding: https://github.com/blazegraph/database/wiki/Reification_Done_Right and https://wiki.blazegraph.com/wiki/index.php/Reification_Done_Right
- AnzoGraph LPG / RDF-star docs — edge-property syntax, 255-value cap: https://docs.cambridgesemantics.com/anzograph/v3.1/userdoc/lpgs.htm
- Virtuoso RDF-star community thread: https://community.openlinksw.com/t/virtuoso-support-for-rdf-star-and-sparql-star/2280
- QLever GitHub + Wikipedia — id encoding, scale, SPARQL 1.1 compliance: https://github.com/ad-freiburg/qlever and https://en.wikipedia.org/wiki/QLever
- Hartig & Thompson, *Foundations of RDF★ and SPARQL★* (2014): https://arxiv.org/pdf/1406.3399 and https://www.semanticscholar.org/paper/Foundations-of-RDF%E2%8B%86-and-SPARQL%E2%8B%86-(An-Alternative-to-Hartig/36e70ee51cb7b7ec12faac934ae6b6a4d9da15a8
- Weiss et al., *Hexastore* (VLDB 2008) — six permutations, dictionary encoding: https://www.cs.au.dk/~karras/hexastore.pdf
- *A Survey of RDF Stores & SPARQL Engines* (arXiv:2102.13027): https://arxiv.org/pdf/2102.13027
- W3C RDF 1.2 Concepts — triple-term & reifier model, object-only constraint: https://www.w3.org/TR/rdf12-concepts/
- W3C RDF 1.2 N-Triples — `<<( s p o )>>` grammar, object-only, nesting: https://www.w3.org/TR/rdf12-n-triples/
- W3C RDF-star WG issue #152 — reification vs triple terms vs `rdf:reifies`: https://github.com/w3c/rdf-star-wg/issues/152