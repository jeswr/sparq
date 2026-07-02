# Full W3C JSON-LD 1.1 support — design record (epic sq-oy1f)

**Status:** accepted (proceed-and-document; maintainer may steer post-hoc)
**Author:** Claude Fable 5 (SPARQ agent)
**Date:** 2026-07-02
**Epic:** sq-oy1f · supersedes-and-absorbs the floor-raise beads sq-oy1f.22 (expand/flatten) and sq-t92rs (frame)
**Related design records:** the conformance-ratchet discipline used by the existing `jsonld_suite.rs` lanes

## 1. Problem

sparq has substantial but incomplete JSON-LD 1.1 support, split across two half-pipelines
with different ceilings:

- **toRdf (ingest)** goes through **oxjsonld v0.2** (`rdf-12`), wired into sparq-core behind
  the opt-in `jsonld` feature. 413/467 suite cases pass. The remaining 54 divergences are
  dominated by *missing plumbing around* oxjsonld, not by oxjsonld itself: no
  `LoadDocumentCallback` (remote and `@import` contexts), the `expandContext` /
  `rdfDirection` / `processingMode` options never forwarded, base-IRI dot-segment
  edge cases, and a handful of documents oxjsonld accepts more leniently than the spec.
- **fromRdf / expand / flatten / compact / frame (emit)** are native, dependency-free
  writers in `sparq-engine/src/serialize.rs` (+ `compact.rs`, `frame.rs`) behind
  `serialize-rdf`, using a tiny custom Json AST. They operate **RDF-first**: every output
  form is a projection of an RDF dataset. Floors: fromRdf 51/53, expand 247/385,
  flatten 50/58, compact 186, frame 61/92.
- **Negative tests** are honestly skipped everywhere: every code path is total; the spec's
  error conditions (invalid `@context`, keyword redefinition, `@protected` violation,
  invalid `@embed`, …) are not modelled.
- **html** and **remote-doc** suite categories are not implemented at all.
- The compact/frame **oracle is oxjsonld self-reparse**, which has already hidden one
  silent data-loss bug (`@reverse` to a non-subject object — sq-oy1f.10, fixed) and is a
  standing interoperability risk (a strict processor like pyld can read our output
  differently from oxjsonld).

### 1.1 Why the RDF-first architecture has a hard ceiling

The W3C JSON-LD 1.1 API algorithms (Expansion, Compaction, Flattening, Framing) are
**document-level**: they transform JSON trees, with `@context` state threaded through the
tree. An RDF dataset is a *lossy projection* of an expanded document. Structures that
cannot be recovered from RDF alone — and which account for most of the remaining
expand/compact/frame failures — include:

- **scoped (property/type) contexts and typed contexts** — context switches are a
  document phenomenon; the RDF projection has already erased where they applied;
- **`@nest`** — purely syntactic grouping with no RDF trace;
- **`@index` containers with non-property indexes** — the index value is dropped by toRdf;
- **relative IRIs / `@base` interplay** in expected expanded documents;
- **free-floating nodes and `@value`-only entries** that project to zero triples;
- **framing value patterns** over `@value` alternative arrays — matching happens on the
  expanded document, not on RDF terms;
- **negative tests** — most error conditions are context/document-shape errors invisible
  after projection to RDF.

Patching the RDF-first writers further (the sq-oy1f.22 / sq-t92rs path) buys single-digit
case gains and cannot reach conformance. **Decision: build the document-level pipeline.**

## 2. Goals / non-goals

**Goals**

1. A native, dependency-free implementation of the JSON-LD 1.1 **document pipeline**:
   Context Processing, IRI expansion/compaction, Expansion, Node-Map/Flattening,
   Compaction, Framing, RDF Serialization (`fromRdf`) and Deserialization (`toRdf`).
2. The full **`JsonLdError` code registry** so NegativeEvaluationTests become assertable,
   with per-category negative ratchets.
3. **Document loading** (remote `@context`, `@import`, remote-doc suite category) behind an
   explicit loader abstraction with a hard-off default (no ambient network).
4. **HTML script extraction** (html suite category) behind its own feature.
5. **Processing options** (`base`, `expandContext`, `processingMode`, `rdfDirection`,
   `compactArrays`, `ordered`, framing flags) modelled once and honoured everywhere.
6. Content negotiation across server/CLI/py/wasm including the JSON-LD **profile**
   parameters (`#expanded` / `#compacted` / `#flattened` / `#framed`) and
   `Link rel` context/frame supply.
7. Honest conformance: normative document-level oracles for expand/compact/flatten/frame,
   ratchet floors that only rise, a pyld third-party faithfulness lane, and a
   differential lane between oxjsonld and the native toRdf.

**Non-goals**

- Replacing oxjsonld as the *default* ingest path in this epic (it stays the proven,
  streaming fast path; a switch decision is a *measured* end-of-epic decision record).
- JSON-LD as a SPARQL **results** format (SELECT/ASK) — orthogonal, tracked as sq-oy1f.15.
- CBOR-LD / YAML-LD / JSON-LD-star beyond what `rdf-12` already gives us.
- A full HTML5 DOM parser (the html category needs script-element extraction only).

## 3. Crate architecture

### 3.1 New opt-in crate: `sparq-jsonld`

Per the opt-in feature architecture rule (core stays lean), the pipeline lives in a **new
workspace crate `crates/sparq-jsonld`**, not inside sparq-engine:

- **Zero mandatory dependencies.** The existing custom Json AST moves here (from
  `sparq-engine/src/serialize.rs`) and becomes the crate's public JSON value type. No
  serde_json. The only optional deps arrive with optional features (`http-loader`).
- sparq-engine's `serialize-rdf` JSON-LD writers become **thin adapters** over
  `sparq-jsonld` (public engine API preserved; engine gains an internal `jsonld` feature
  forwarding to the crate). sparq-core's `jsonld` ingest feature is unchanged for
  oxjsonld, and additionally exposes the native pipeline for the strict/conformance path.
- `forbid(unsafe_code)`, MSRV-aligned, README within the readme-template gate.

Module map:

```text
crates/sparq-jsonld/src/
  json.rs        Json AST (moved from engine) + number/JSON-literal canonical form
  error.rs       JsonLdError { code: JsonLdErrorCode, detail } — full spec code registry
  options.rs     JsonLdOptions (base, processingMode, expandContext, rdfDirection,
                 compactArrays, compactToRelative, ordered, produceGeneralizedRdf,
                 frameExpansion, extractAllScripts, frame flags: embed/explicit/
                 omitDefault/requireAll)
  loader.rs      DocumentLoader trait (LoadDocumentCallback) + RemoteDocument
                 { document, documentUrl, contentType, contextUrl, profile }
                 impls: NoopLoader (default: refuses remote fetch —
                 `loading document failed`, or `loading remote context failed`
                 when the failure is a remote `@context`),
                 FsLoader (URL→fixture map), MockLoader (conformance remote-doc, §5)
  http.rs        [feature http-loader] HttpLoader: redirects, content-type + profile,
                 Link rel="…json-ld#context" alternate handling, SSRF allowlist policy
  html.rs        [feature html] script-element extraction (see §6)
  context/
    mod.rs       ActiveContext, TermDefinition (incl. @protected), inverse context
    process.rs   Context Processing Algorithm + Create Term Definition
    iri.rs       IRI Expansion (API §5.2) + IRI Compaction + Term Selection (§7.1–7.2)
  expand.rs      Expansion Algorithm (§5.1) + Value Expansion; frameExpansion mode
  node_map.rs    Node Map Generation (§6.1) + merge
  flatten.rs     Flattening (§6.2)
  compact.rs     Compaction Algorithm (§7) — document-level, over expanded input
  frame.rs       Framing (json-ld-framing §3) — matching, value patterns, @embed
  to_rdf.rs      Deserialize JSON-LD to RDF (§8.2) over expanded docs (native toRdf)
  from_rdf.rs    Serialize RDF as JSON-LD (§8.1): RDF→expanded doc (rdfDirection both
                 modes, @json literals, list reconstruction)
  api.rs         JsonLdProcessor facade: expand/compact/flatten/frame/to_rdf/from_rdf
```

### 3.2 One spec-shaped composition instead of five bespoke writers

All output forms become compositions over the same two hinges (this is exactly the
JSON-LD-API's own factoring):

```text
RDF dataset --from_rdf--> expanded document --compact(ctx)-->  compacted
                          |--node_map+flatten(±ctx)-->         flattened
                          |--frame(frame doc)-->               framed (framed = frame ∘ compact)
                          '--(identity)-->                     expanded

JSON doc --(html extract?)--> parsed --expand(ctx chain, loader, options)--> expanded
         --to_rdf--> RDF dataset          (native path; oxjsonld remains the default)
```

Consequences:

- **compact/frame failures rooted in "the writer never emits shape X"** (scoped contexts,
  `@nest`, `@index`/`@id` maps, array/remote `@context` forms, value patterns) become
  reachable, because compaction/framing now run on expanded documents with a real
  ActiveContext, not on RDF.
- The sq-oy1f.10 *class* of bug (self-reparse-invisible data loss) is structurally
  removed: compaction is a spec algorithm over the expanded doc, not a bespoke inverse
  of the parser.
- The **fromRdf** lane keeps its round-trip oracle and additionally the expanded output
  can be compared document-level where the suite provides normative expected docs.

### 3.3 toRdf strategy: oxjsonld stays, native path completes it

- **Default/fast path:** oxjsonld, unchanged (streaming, proven, 413/467).
- **Conformance/strict path:** native `expand ∘ to_rdf` used (a) by the harness for
  categories/options oxjsonld cannot express (`expandContext`, `rdfDirection`,
  strict error codes), and (b) opt-in via `JsonLdOptions { strict: true }`.
- **Differential lane:** the conformance harness cross-checks oxjsonld vs the native
  toRdf on all positive suite cases both paths can run; divergence is a test failure in
  the lane, resolved either by a native fix or an **upstream oxigraph issue/PR**
  (roll-your-own + upstream doctrine — the known leniency acceptances go upstream).
- End-of-epic **decision record**: measure native vs oxjsonld (perf + conformance) and
  decide default; until then the dependency stays.

## 4. Error model (NegativeEvaluationTests)

`error.rs` carries the full JSON-LD 1.1 error-code registry as a closed enum with the
spec's exact string forms (`"invalid @context"`, `"keyword redefinition"`,
`"protected term redefinition"`, `"invalid @embed value"`, `"loading document failed"`,
`"loading remote context failed"`, `"invalid base direction"`,
`"processing mode conflict"`, `"invalid @nest value"`,
`"invalid reverse property map"`, `"colliding keywords"`, `"invalid @import value"`,
`"context overflow"`, `"invalid script element"`, `"invalid frame"`, …). Rules:

- Every native algorithm is **fallible**: `Result<_, JsonLdError>`. The engine adapters
  translate to the engine's existing error surface without losing the code.
- The harness asserts the **exact `expectErrorCode`** from the manifest — an error of the
  wrong code is a FAIL, not a pass (honesty over score).
- oxjsonld errors are mapped to codes where unambiguous; cases where oxjsonld is lenient
  are covered by the native strict path (§3.3), never by post-hoc guessing.
- Negative floors are ratcheted **per category and separately from positive floors**
  (`TORDF_NEG_FLOOR`, `EXPAND_NEG_FLOOR`, …), starting at their first measured values.

## 5. Document loading (remote-doc, `@import`)

- `DocumentLoader` is a trait (dyn-friendly, sync; wasm gets a caller-supplied callback
  variant). **Default is `NoopLoader`** — any remote fetch is refused: a failed
  top-level document load raises `loading document failed`, and a failed remote
  `@context` dereference raises `loading remote context failed`. No surface
  acquires ambient network by merely enabling `jsonld`.
- `FsLoader` maps URL prefixes to fixture directories; the conformance `MockLoader`
  additionally honours the manifest's `httpStatus` / `redirectTo` / `contentType` /
  HTTP-`Link` options so the remote-doc category runs **hermetically offline**.
- `HttpLoader` (feature `http-loader`, native only) implements the spec's retrieval
  algorithm: redirect following, `application/ld+json` profile handling, `Link
  rel="alternate"` and `rel="http://www.w3.org/ns/json-ld#context"` on non-JSON-LD
  responses, and a **deny-by-default allowlist** (scheme+host) — the server never
  dereferences attacker-supplied context URLs unless the operator configures the
  allowlist (SSRF posture; document in the threat model).
- Context caching: per-processor-call memo (URL→parsed context) with the spec's
  `context overflow` recursion guard; no cross-request cache in the server (cache
  poisoning surface) until a separate hardening pass.

## 6. HTML script extraction (`html` feature)

Minimal, dependency-free scanner — **not** a DOM parser: locate
`<script type="application/ld+json">` elements, honour `extractAllScripts` (concatenate
as an `@graph`-merged array vs first/target only), fragment-id targeting
(`#some-id` → the script with that `id`), `<base href>` for base IRI, and the
`invalid script element` error for malformed/non-JSON content. Character-reference
decoding limited to what script content needs. Gated behind a `html` cargo feature so
the lean builds pay nothing. The html suite category graduates from not-implemented to a
ratcheted lane when this lands.

## 7. Surfaces and content negotiation

| Surface | Feature default | Behaviour |
|---|---|---|
| sparq-server | `jsonld` default ON | `Accept: application/ld+json` honoured today; add the four **profile params** — `profile="http://www.w3.org/ns/json-ld#expanded"` (also `#compacted`, `#flattened`, `#framed`) select the output form. `#compacted`/`#framed` need a context/frame: taken from a request `Link` header with `rel=".../json-ld#context"` / `rel=".../json-ld#frame"` (dereferenced only under the loader policy §5, else 400 with the spec error code). No usable profile → today's default form (stable). Ingest (GSP PUT/POST): profile + Link-context honoured symmetrically; unsatisfiable Accept keeps the existing 406 parity semantics. |
| sparq-cli | `jsonld` default ON | dump/load gain `--jsonld-form expanded\|compacted\|flattened\|framed`, `--jsonld-context <file\|URL>`, `--jsonld-frame <file>`, `--jsonld-base`, `--rdf-direction`; remote fetch only with `--allow-remote-contexts` (HttpLoader). |
| sparq-wasm | OPT-IN, off | unchanged posture (bundle byte-floor). When enabled: same processor; loader = caller-supplied JS callback; no fetch inside the wasm module. |
| sparq-py | carries feature | `expand/compact/flatten/frame` functions + options dict mirroring pyld naming (drop-in familiarity). |

## 8. Conformance harness evolution (`crates/sparq-conformance/tests/jsonld_suite.rs`)

1. **Oracle upgrade (the honesty centre of this design).** expand/flatten/compact/frame
   move from RDF-equivalence / self-reparse to the **normative document-level
   comparison** used by the official suite (deep JSON equality; array order significant
   only where the spec makes it so, i.e. inside `@list`; `ordered:false` semantics
   elsewhere). fromRdf keeps round-trip isomorphism (`Dataset::canonicalize`) *plus*
   document comparison where expected docs exist. toRdf keeps dataset isomorphism.
2. **Floors re-pinned per lane at first measured value under the new oracle** — an oracle
   *strengthening* may lower a number while strictly raising honesty; the re-pin commit
   must state old-oracle vs new-oracle values side by side. After re-pin, floors only
   rise again.
3. **Negative lanes** (§4) and **option-bearing cases**: stop skipping cases with
   `expandContext` / `rdfDirection` / `processingMode` — forward through `JsonLdOptions`.
4. **remote-doc lane** via MockLoader (§5), **html lane** via §6 — both graduate out of
   `NOT_IMPLEMENTED_CATS`.
5. **Differential lane** oxjsonld ↔ native toRdf (§3.3).
6. **pyld faithfulness lane**: CI job (python, pinned pyld) re-expands sparq's
   compacted/framed outputs for the whole corpus and diffs against sparq's own expansion
   — catches self-reparse blind spots by construction. Advisory first, ratcheted once
   stable. Local `cargo test` stays hermetic (lane is CI-only, env-gated).
7. Feature `jsonld-suite` grows forwards to `sparq-jsonld` + its `html` feature; skip
   accounting stays honest (per-reason skip counts printed by the runner).

## 9. Migration & compatibility

- Engine public API (`graph_to_jsonld*`, compaction/framing entry points) is preserved;
  internals delegate to `sparq-jsonld`. The Json AST move is a pure relocation
  re-exported at the old path for one release.
- Each migration bead flips **one lane** to the native pipeline and re-pins that lane's
  floor in the same PR — no big-bang cutover, `main` conformance never regresses.
- Every new public fn gets one direct unit test (coverage-ratchet floor rule).
- Known trap to brief implementers on: feature-gated intra-doc links from always-compiled
  doc-comments (use code spans), and the readme-template hard gate for the new crate.

## 10. Work decomposition (dependency-ordered beads, parent sq-oy1f)

| # | Bead | Depends on | Content |
|---|---|---|---|
| A | sq-oy1f.23 | — | `sparq-jsonld`: Json AST move + `error.rs` registry + `options.rs` + loader trait (Noop/Fs); engine re-export shim |
| B | sq-oy1f.24 | A | Context Processing + Create Term Definition + IRI expand/compact + inverse context |
| C | sq-oy1f.25 | B | Expansion Algorithm (+frameExpansion mode); expand lane → normative oracle, re-pin |
| D | sq-oy1f.26 | C | Node Map Generation + Flattening; flatten lane → normative oracle, re-pin |
| E | sq-oy1f.27 | B, C | Document-level Compaction; engine delegates; compact lane → normative oracle, re-pin |
| F | sq-oy1f.28 | A | RDF→expanded doc per §8.1 (rdfDirection, @json, lists); subsumes engine serialize core |
| G | sq-oy1f.29 | C, D, E | Framing on the native pipeline (value patterns, @explicit/@default, named graphs, @list/@set, bnode @embed); absorbs sq-t92rs |
| H | sq-oy1f.30 | C, F | JsonLdOptions forwarding; native `to_rdf`; differential lane; upstream leniency issues |
| I | sq-oy1f.32 | A, H | HttpLoader (`http-loader`) + SSRF policy + MockLoader; remote-doc + `@import` lanes |
| J | sq-oy1f.31 | C, E | Error raising in context/expand/compact (framing negatives ride G); harness `expectErrorCode` + negative ratchets |
| K | sq-oy1f.33 | A, H | `html` feature scanner + html lane ratchet |
| L | sq-oy1f.34 | D, E, G | server profile conneg + Link context/frame; CLI flags; py API; wasm opt-in wiring |
| M | sq-oy1f.35 | E, G | third-party faithfulness CI lane (advisory→ratchet) |
| N | sq-oy1f.36 | H,I,J,K,L,M | re-pin all floors, docs/SKILL/AGENTS updates, oxjsonld default-vs-native decision record |

Parallelism: {B→C→D, F} start together after A; E/H/J/G fan out mid-epic; I/K/L/M are
independent leaves; N closes. sq-oy1f.22 and sq-t92rs are superseded (noted on the
beads); sq-oy1f.15 (SPARQL-results JSON-LD) and sq-oy1f.21 stay independent.

## 11. Risks / open questions

- **Suite comparison subtleties** (where array order is significant in expected docs)
  must follow the suite's documented comparison, not intuition — C pins this in code
  with fixtures.
- **Floor re-pins that go down under a stronger oracle** need the side-by-side statement
  (§8.2) to stay honest — never re-pin silently.
- **`@embed: @link`** (identity-preserving embedding) remains documented as `@once`
  fallback until a consumer needs real object identity in the output tree.
- **HttpLoader in the server** is deliberately deny-by-default; enabling it is an
  operator decision recorded in config, and belongs in the threat model when I lands.
- oxjsonld major-version drift: the differential lane doubles as our early-warning.
