<!-- [OPUS-4.8] JSON-LD pretty/compaction scope record authored by Opus 4.8 (1M context); Fable unavailable — re-review when Fable returns. Bead sq-pxdu (spin-off from sq-ixc3.2). -->
# Scope record — JSON-LD pretty-printing + compaction

> 🤖 SPARQ agent — design-for-review. This is a SCOPING record, not an
> implementation. It decides what (if anything) is worth building for JSON-LD
> output formatting/compaction and decomposes it into atomic beads.

Spin-off from **sq-ixc3.2** (engine-side pretty Turtle/TriG serialiser), which
explicitly left JSON-LD pretty/compaction out of scope. Bead **sq-pxdu**.

## Correction to the brief's premise

The brief states the engine "already has `graph_to_jsonld` with Expanded/
Flattened/Compacted forms (basic prefix `@context`)". That is **correct and
verified** against the code — but two qualifications matter for this decision,
and the brief did not state them:

1. **The JSON-LD output is MINIFIED** (single-line, no indentation). The
   writers in `crates/sparq-engine/src/serialize.rs` only ever push `{`, `,`,
   `:`, `[`, `]` — there is no newline/indent logic in any JSON-LD writer (the
   only `\n` in that file is the string-escape in the literal-escaper, and all
   the indentation machinery is the Turtle/TriG path added by sq-ixc3.2). So
   "pretty JSON-LD" is genuinely absent today.
2. **"Compacted" today means prefix-only `@context`.** `JsonLdForm::Compacted`
   emits an `@context` that maps each *used* prefix to its namespace IRI (via
   `compact_iri` + `write_context`) and abbreviates predicate/type IRIs to
   CURIEs. It does **not** implement the full W3C JSON-LD 1.1 Compaction
   Algorithm (term definitions, `@vocab`, type/language coercion, container
   mappings, `@reverse`, value/node compaction). The doc-comment is honest
   about this ("basic prefix `@context`", "best-effort, lossless").

So the engine already covers the *cheap* end of the design space (prefix
compaction). The open questions are pretty-printing and *full* compaction.

## Current state (verified)

Evidence, all on `origin/main` at the time of writing:

| Surface | What exists | Pretty? | Compaction depth |
|---|---|---|---|
| `sparq-engine::serialize` (`serialize-rdf` feature) | `write_jsonld` / `graph_to_jsonld` / `graph_to_jsonld_with`; `JsonLdForm::{Expanded,Flattened,Compacted}` | No (minified) | Prefix `@context` only |
| `sparq-cli dump … jsonld[-expanded\|-flattened\|-compacted]` | Wired to the three forms; defaults to Expanded | No | Prefix `@context` only |
| `sparq-wasm` `Store` | JSON-LD **ingest** only (`load`/`loadDataset`, behind opt-in `jsonld`); **no** serialize-out method at all | n/a | n/a |
| Site (`site/src/lib/repl-dataset.ts`, `repl-datasets.tsx`) | JSON-LD as an upload/URL **input** format (parsed engine-side via `oxjsonld`); routed through `loadDataset` as a dataset format | n/a | n/a |

Key architectural facts that drive the recommendation:

- **`serialize-rdf` is dependency-free by design.** The native JSON-LD writer
  "emits JSON by hand and pulls in nothing"; `serde_json` is a **dev**
  dependency used only by the round-trip tests (see the `serialize-rdf`
  feature + `serde_json` notes in `crates/sparq-engine/Cargo.toml`). The lean
  core is a stated project principle (opt-in feature architecture).
- **`oxjsonld` is already in the tree** — but only behind the opt-in `jsonld`
  feature in `sparq-core`, and only for **ingest** (`JsonLdParser`, used in
  `crates/sparq-core/src/lib.rs`). It is not linked by the lean default bundle.
- The site already plans dependency-free JSON-LD **syntax highlighting**
  (sq-ixc3.1, in progress) and the engine pretty-Turtle → wasm → site swap is
  already a chain of beads (sq-fe1s → sq-lxej). JSON-LD output should follow the
  same engine-first pattern, not grow a second site-side reshaper.

### Does the existing JSON-LD dep give us compaction "for free"?

No. `oxjsonld` v0.2.5's `JsonLdSerializer` exposes `new`, `with_prefix`,
`with_base_iri`, `for_writer`, `for_tokio_async_writer` — i.e. it offers
**prefix-level** `@context` (exactly what sparq already hand-rolls) and has **no
pretty/indent option and no full-compaction/framing**. So adopting oxjsonld's
serialiser would (a) gain nothing over the current hand-rolled writer, (b) make
`serialize-rdf` depend on `oxjsonld`, breaking the dependency-free property.
Full W3C compaction would need a heavier processor crate (e.g. the `json-ld`
family), which is a much larger dependency and pulls IRI/context machinery the
core does not otherwise want. (oxjsonld API confirmed from its docs.rs page; see
Sources.)

## Design space and options

### (a) Pretty / indented JSON-LD output — tractable, real value

A formatting-only change: emit the same JSON-LD documents the writers produce
today, but indented and with a stable key order (the structural ordering is
**already deterministic** — first-seen subject/predicate order — so this is
purely whitespace + an opt-in flag, mirroring `PrettyOptions` for Turtle). No
new dependency; stays inside `serialize-rdf`. Low effort, clear payoff: the
CLI `dump … jsonld` and (once wired) the site's results view become readable.

### (b) Full JSON-LD 1.1 Compaction — substantial, low marginal value

Implement (or delegate) the W3C Compaction Algorithm: a user-supplied
`@context` with term definitions, `@vocab`, type/language/container coercion,
value + node compaction, `@reverse`. This is a genuinely non-trivial algorithm
(the spec's compaction + IRI-compaction + value-compaction steps are mutually
recursive). Two ways to get it, both costly:

- **Hand-roll in `serialize-rdf`** — keeps the dep-free property but is a large,
  spec-conformance-sensitive body of code to write *and maintain* (it is exactly
  the kind of surface where subtle non-conformance bites interop). High effort.
- **Delegate to a processor crate** — far less code, but adds a heavy dependency
  and breaks the lean-core principle; would have to be its own opt-in feature
  (`jsonld-compact`?), not part of `serialize-rdf`.

Marginal value is low: the prefix `@context` form already gives compact,
readable, lossless output for the common case (CURIE-abbreviated IRIs), which is
what the CLI/site/REPL actually need. Full compaction is a *consumer-shaping*
convenience (matching a caller's specific term vocabulary), which no current
sparq surface requests. **Not recommended now** — capture as a deferred,
opt-in bead with the dep tradeoff flagged, do not build speculatively.

### (c) Framing — out of scope

JSON-LD Framing (selecting/reshaping a subgraph by a frame document) is a
separate, larger algorithm with no current consumer. Explicitly **out of
scope**; noted here only so it is not silently conflated with compaction.

## Recommendation

1. **Do (a): pretty JSON-LD in `serialize-rdf`.** Add an indent option to the
   JSON-LD writers (e.g. a `JsonLdPrettyOptions { indent }` / a `pretty` flag
   on `write_jsonld` / new `*_pretty` entry points), reusing the deterministic
   ordering already present. No new dependency. Mirror the Turtle precedent
   (`PrettyOptions`) and add CLI `jsonld-pretty[-expanded|-flattened|-compacted]`
   out-formats. This is the only JSON-LD-output work that clears the value/effort
   bar today.
2. **Do NOT build (b) full compaction now.** The existing prefix `@context`
   already serves the consumers we have, and either implementation path is
   expensive (hand-roll = large conformance-sensitive code; delegate = heavy
   dep that breaks lean-core). Capture it as a **deferred, opt-in** bead so the
   decision is recorded and revisitable, not a `wontfix` that loses the context.
3. **Don't grow a site-side JSON-LD reshaper.** When the site wants pretty
   JSON-LD output, it should consume the engine writer through the wasm
   serialize surface (the same surface sq-fe1s is adding for Turtle) — fold
   JSON-LD into that surface rather than re-deriving it in TypeScript. Tracked
   as a small follow-up that depends on the wasm serialize surface existing.

This keeps the core lean, follows the engine-first precedent set by sq-ixc3.2,
and avoids speculative algorithm work with no consumer.

## Phased plan (atomic beads)

Ordered; each is a future bead under the GUI epic `sq-ixc3` (siblings of the
Turtle chain sq-ixc3.2 / sq-fe1s / sq-lxej):

1. **`sq-ixc3.3` — Engine pretty JSON-LD writer** (P3, `area:engine`,
   `serialize-rdf`) — add an indent/pretty option to `write_jsonld`/
   `graph_to_jsonld` + `*_pretty` entry points + CLI `jsonld-pretty…`
   out-formats; golden + round-trip tests in both feature states; **no new
   dependency**.
2. **`sq-ixc3.4` — Deferred: full JSON-LD 1.1 compaction, opt-in** (P3,
   `area:engine`) — a `@context`-driven compaction beyond prefix abbreviation.
   MUST be its own opt-in feature (NOT part of dependency-free `serialize-rdf`);
   the bead body records the hand-roll-vs-processor-crate tradeoff so the dep
   cost is decided deliberately. Do not start without a concrete consumer.
3. **`sq-ixc3.5` — Site/wasm: fold JSON-LD into the wasm serialize surface** (P3,
   `area:wasm`) — once the Turtle wasm serialize surface (`sq-fe1s`) exists,
   expose JSON-LD (incl. the new pretty option) through it so the site never
   hand-formats JSON-LD; depends on `sq-fe1s` + `sq-ixc3.3`.

Framing is intentionally NOT a bead (no consumer; out of scope).

## Open questions for the maintainer

- **Is full compaction ever wanted?** It only earns its keep with a concrete
  consumer (e.g. a GUI "export with my vocabulary" feature). If you do not
  foresee one, bead #2 above can be closed `wontfix` instead of kept deferred.
- **Pretty default for the CLI?** Should `sparq-cli dump … jsonld` stay
  minified-by-default (machine output) with `jsonld-pretty` opt-in (the
  proposed plan), matching how `turtle` vs `turtle-pretty` works?

## Sources

- `crates/sparq-engine/src/serialize.rs` (JSON-LD writers, `JsonLdForm`,
  `write_context`, `compact_iri`), `crates/sparq-engine/Cargo.toml`
  (`serialize-rdf` dep-free, `serde_json` dev-only).
- `crates/sparq-cli/src/main.rs` (`dump … jsonld[-…]`), `crates/sparq-wasm/src/lib.rs`
  (ingest-only JSON-LD), `crates/sparq-core/src/lib.rs` (`oxjsonld` ingest).
- `site/src/lib/repl-dataset.ts` (JSON-LD as input format).
- oxjsonld v0.2.5 serialiser API — <https://docs.rs/oxjsonld/latest/oxjsonld/struct.JsonLdSerializer.html>
- Related beads: sq-ixc3.2 (Turtle pretty), sq-fe1s / sq-lxej (Turtle wasm/site),
  sq-ixc3.1 (JSON-LD site highlighting).
