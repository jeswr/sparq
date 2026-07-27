# Shuttle gen-rs backend gaps: status verification + the remaining quad-emission design (sq-uyney)

> 🤖 SPARQ agent [OPUS-5], 2026-07-27. Bead **sq-uyney** (issue #2698), child of the
> Shuttle epic **sq-tonhr** (design record:
> [`shacl-compact-and-shuttle-parsers-2026-07.md`](./shacl-compact-and-shuttle-parsers-2026-07.md)).
> This record supersedes caveat 4 of
> [`shuttle-gen-rs-go-no-go-2026-07.md`](./shuttle-gen-rs-go-no-go-2026-07.md), which listed all
> three gaps as open and attributed all three to the Rust backend. **Two are closed upstream;
> the third is open in BOTH backends, not just gen-rs.**

## Why this record exists

The go/no-go record named three `gen-rs` gaps that gate the *next* Shuttle grammars (N-Quads,
TriG) rather than the go/no-go verdict itself: `emit @graph`, a FIRST-set bitmask limited to
≤ 128 token kinds, and a fixed `@maxdepth` constant. sq-uyney is the bead to close them. The
work lives in the **upstream** repository `jeswr/rdf-shuttle` (`packages/gen-rs`, `packages/gen-js`,
and the shared front end) — sparq only vendors the *generated* artifacts — so this bead's
sparq-side deliverable is the verification, the correction, and the design; the code change
itself belongs in a draft PR on that repository (see *What is deliberately not done*).

## Verification pin and method

Upstream `jeswr/rdf-shuttle` was cloned fresh on 2026-07-27 and read at

```text
89bbe6d2ee38b4ec3379f39f0c8bd25894f8efac  "meta + both backends: grammar-driven @maxdepth(N)
                                           production annotation [FABLE-5]"  (2026-07-12)
```

All claims below were re-derived from that tree, not from the earlier record. The upstream
node-level suite for the Rust backend was run and is green:

```sh
git clone https://github.com/jeswr/rdf-shuttle && cd rdf-shuttle/packages/gen-rs
node --test test/gen.test.js      # 7 pass, 0 fail (includes the two sq-uyney tests below)
```

## Status of the three gaps

| gap | status at 89bbe6d | evidence |
|---|---|---|
| `>128` token kinds → FIRST-mask array fallback | **closed** | `packages/gen-rs/src/parser-gen.js` `testFor()`; test `gen.test.js` "…fall back to a `[u64; W]` FIRST-mask array with the right bits" |
| grammar-driven `@maxdepth` | **closed** | front end `packages/gen-js/src/meta.js` (prodAnnot parse); `packages/gen-rs/src/parser-gen.js` + `packages/gen-js/src/parser-gen.js` guard emission; test "`@maxdepth(N)` drives the generated depth guard in both backends" |
| `emit @graph` (N-Quads/TriG) | **open — in BOTH backends** | identical `throw` in `packages/gen-rs/src/clausec.js` and `packages/gen-js/src/clausec.js`; reproduced below |

### Closed: the `[u64; W]` FIRST-mask fallback

`ParserGen.testFor()` now selects the mask representation from the token-kind count: `u64` at
≤ 64 kinds, `u128` at ≤ 128, and beyond that a `const FS_n: [u64; W]` word array with the
membership test compiled to

```rust
(FS_n[(self.tk >> 6) as usize] >> (self.tk & 63)) & 1 != 0
```

The upstream test drives a synthetic 140-keyword grammar (`test/synthetic-fixture.mjs`),
recovers the set bits out of the emitted array literal, and asserts they are exactly the 140
keyword kinds — so it fails if the word/bit split is wrong, not merely if the array is absent.
It also re-asserts the artifact contract on the fallback path (std-only, no `unsafe`).

### Closed: grammar-driven `@maxdepth(N)`

The front end parses a `@maxdepth(N)` **production annotation** (positive integer; the other
`prodAnnot`s named by the grammar of the grammar language stay a loud error rather than a silent
no-op). Both backends emit the annotated cap in place of their built-in default, which remains
`8192` for un-annotated productions. The test pins three things: the generated Rust guard text
at the annotated cap, the preserved default in the checked-in `turtle12` artifact, and a
*behavioural* boundary through the JS backend (nesting at the cap parses, one deeper raises
`MAXDEPTH`) — the depth guard is therefore witnessed, not just pattern-matched.

### Open: `emit @graph`

Both backends still refuse the graph slot with the same message. Reproduced today against a
minimal grammar that declares `emits quads ;` and uses `emit s p o @ g`:

```text
gen-rs THREW: emit @graph not supported by the triples-emitting backend yet
gen-js THREW: emit @graph not supported by the triples-emitting backend yet
```

Three corrections to the earlier framing follow from that run:

1. **It is not a gen-rs gap.** The Rust and JS backends carry byte-identical throws in the same
   clause compiler position. Whoever closes it must close it in both, or the cross-backend
   conformance identity that the go/no-go verdict rests on stops being checkable for quad
   grammars.
2. **The front end is already ready.** The grammar surface parses both halves: the `@g` slot on
   an emit clause becomes the `g` field of the emit AST node, and the module header parses
   `emits <shape>` with an optional ` bag`. Nothing new is needed to *express* quad emission.
3. **The declared emission shape is inert.** Neither backend ever reads `headers.emits` — a grep
   for header reads in both `src/` trees returns only `start`, `profile`, `target` and
   `spec-ref`. A grammar may declare `emits quads` today and still get a triple-shaped artifact
   (which is precisely why the throw calls itself "the triples-emitting backend"). Closing the
   gap therefore means *wiring the header*, not adding a flag.

## Non-regression for sparq (measured, not assumed)

sparq's only Shuttle consumer is `sparq-shaclc`, which vendors two generated artifacts and pins
the generating commit in [`crates/sparq-shaclc/src/provenance.rs`](../crates/sparq-shaclc/src/provenance.rs)
(currently `b7801c5`, i.e. behind upstream HEAD). Regenerating both artifacts from HEAD with the
recorded CLI invocations reproduces the checked-in files **byte-identically**, and the vendored
grammar's SHA-256 is unchanged:

```sh
node src/cli.js ../../grammars/shaclc12ext.shuttle -o /tmp/shaclc12.head.rs    --profile rdf12
node src/cli.js ../../grammars/shaclc12ext.shuttle -o /tmp/shaclc12ext.head.rs --profile rdf12,ext
diff /tmp/shaclc12.head.rs    crates/sparq-shaclc/src/raw/shaclc12.rs      # identical
diff /tmp/shaclc12ext.head.rs crates/sparq-shaclc/src/raw/shaclc12ext.rs   # identical
```

Both landed changes are output-neutral for this grammar — as expected, since SHACL-CS stays far
below 128 token kinds and annotates no production with `@maxdepth`. So the pin bump is a chore
with a zero-line artifact diff, not a correctness event, and `tests/provenance_drift.rs` stays
green either way.

## Design for the remaining gap: quad-shaped emission

### It is already specified upstream

This is not a new design. `spec/SHUTTLE.md` and `docs/rfc/0001-shuttle-v0.2-design.md` in the
same repository already fix the semantics: a module declares `shtl:emits` as one of
`TripleSet | QuadSet | TripleBag | QuadBag`; a template is `⟨s, p, o[, g][, when]⟩` with `g`
an **inherited** attribute defaulting to the default graph; positional typing constrains the
slot (`shtl:g ⊑ GraphT`); and the lattice rung for N-Quads is described as *"one production
delta (optional graph label); ZERO semantic delta — the template was quad-shaped all along;
module-closed constant folding keeps the Turtle artifact free of the graph slot"*. The v0.1
surface the backends actually compile is behind that spec on exactly one point: the emission
shape is hard-coded rather than derived from the header.

### What the v0.1-surface implementation costs

The value world is the saving grace: every runtime value in the generated Rust is the single
closed `Term` enum, and grammar-signature types such as `subjT`/`graphT` are erased to it. So
the `@g` expression compiles through the existing term-argument path with **no new value
machinery** — what changes is the shape of the emitted *item* and of the sink.

Touch points in `packages/gen-rs`:

| file | what is triple-shaped today |
|---|---|
| `src/clausec.js` | the `emit` case: the `cl.g` throw, plus the two `emit_q(s, p, o)` call sites (guarded `when` form and plain form) |
| `src/generate.js` | `struct Machine<'i, F: FnMut(Triple)>`, `stmt_buf: Vec<Triple>`, `fn emit_q(&mut self, s, p, o)`, `pub fn parse<F: FnMut(Triple)>`, `pub fn parse_to_triples`, `pub struct PushParser<F: FnMut(Triple)>` |
| `src/serializer-gen.js` | `Writer::triple`, `write_triples` |

(`packages/gen-js` mirrors all three, and its writer additionally rejects a non-default graph at
runtime with "named graphs are not expressible".) One naming trap for the implementer: gen-rs's
`semType === 'graph'` marks an **effect-only production with no synthesized value** — it has
nothing to do with the RDF graph slot.

### Proposed shape

1. **Derive the artifact shape from the header.** Read `headers.emits` in both backends. A
   triple-shaped grammar must produce a **byte-identical** artifact to today's — that invariant
   is directly checkable against `packages/gen-rs/generated/turtle12.rs` and against sparq's two
   vendored SHACL-CS artifacts, and it is the cheapest possible regression gate for this change.
2. **Quad-shaped grammars emit `Quad`.** `pub struct Quad { subject, predicate, object,
   graph_name: Option<Term> }`, where `None` is the default graph. Prefer the `Option` over a
   `Term::DefaultGraph` variant: it keeps `Term` closed, so neither the serializer nor the
   residual printer grows a match arm that can only ever be unreachable in term position.
   The sink becomes `F: FnMut(Quad)`, with `stmt_buf: Vec<Quad>`, `emit_q(s, p, o, g)`,
   `parse_to_quads`, and `PushParser<F: FnMut(Quad)>`. Statement-granular push rollback is
   unaffected — it buffers whatever the item type is.
3. **Compile the slot, do not special-case it.** In the clause compiler the throw becomes
   `Some(<term argument>)` for `@g` and `None` otherwise, in both the guarded and plain emit
   paths.
4. **Diagnose at the header, not at the backend.** `@g` in a grammar that declares `emits
   triples` should be a front-end error naming the header — a better message than today's
   backend throw, and it keeps the two backends from drifting on which grammars they accept.
5. **The writer is where the real work is.** Emitting the N-Quads graph column (a fourth term
   before the terminating `.`) is a small print-rule delta. TriG is not: a grouped writer must
   partition the residual by graph and re-enter the Turtle block printer per group, which is a
   genuine design step and should be scoped as its own bead rather than smuggled into this one.
6. **Threading `g` in a v0.1-surface grammar.** v0.1 has no inherited attributes, but productions
   take parameters (precedent: `targetClass(n: subjT)` in the vendored `shaclc12ext.shuttle`), so
   a TriG grammar can thread the label explicitly as `triplesBlock(g: term)`. The v0.2 module
   design replaces that with the inherited `g` attribute plus constant folding; the v0.1 shape
   should be written so it maps onto that without a semantic change.
7. **Tests, mirroring how the two closed gaps were tested.** Add an `emits quads` synthetic
   fixture beside the existing one and assert the artifact contract on the quad path (`pub struct
   Quad`, `pub fn parse_to_quads`, `None` graph for `@g`-free emits, std-only, no `unsafe`), plus
   the byte-identity of the triple artifacts. Cross-backend identity (`test/conformance.sh`) needs
   a quad oracle pair, so the honest first grammar is minimal N-Quads against the W3C N-Quads
   suite — TriG after, once the grouped writer exists.

### Consequence for the sparq bead map

Only the *quad* legs are blocked. In the epic's phasing, sq-tonhr.8 (NT/NQ) and sq-tonhr.9
(Turtle 1.2/TriG) each mix a triple-shaped syntax with a quad-shaped one: the **N-Triples and
Turtle 1.2 legs are not blocked by this gap at all** and can proceed against the backend as it
stands today, while the N-Quads and TriG legs wait on the upstream change. That split is worth
respecting when those beads are cut, rather than parking both whole beads behind one upstream PR.

## What is deliberately not done here

No upstream patch was written. The remaining change belongs in `jeswr/rdf-shuttle`, and this lane
can neither push to that repository nor open a pull request on it; a patch that exists only as
text inside a sparq markdown file would be unverifiable by any gate on either side. Per
`AGENTS.md` § *Upstream contributions*, the change lands as a **draft** PR on that repository,
self-identified as agent-authored, with @jeswr as the review gate and a "not yet ready for
maintainer review" note — captured as a follow-up rather than guessed at here.

## Reproduction

```sh
git clone https://github.com/jeswr/rdf-shuttle && cd rdf-shuttle
git rev-parse HEAD                                   # expect 89bbe6d…
cd packages/gen-rs && node --test test/gen.test.js   # the two sq-uyney tests
# the emit @graph gap, both backends: generate any grammar containing `emit s p o @ g`
# the sparq non-regression check: regenerate the two SHACL-CS artifacts per
# scripts/regen-shuttle-parsers.sh and diff against crates/sparq-shaclc/src/raw/
```
