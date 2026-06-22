<!-- [OPUS-4.8] JSON-LD support gap-analysis + prioritised roadmap authored by Opus 4.8 (1M context); Fable unavailable — re-review when Fable returns. Epic sq-oy1f (user-prioritised, gh-757). -->
# JSON-LD 1.1 support — gap analysis + prioritised roadmap

> 🤖 SPARQ agent — design-for-review. This is a RESEARCH/DESIGN record (survey +
> plan), not implementation. Epic **sq-oy1f**; the maintainer (@jeswr) reviews
> the three load-bearing decisions (compaction approach, framing scope,
> opt-in-vs-default) and the phased bead plan. No code ships from this PR.

User-prioritised (jeswr, 2026-06-19): *"Please add support for JSON-LD (parsing
and serialisation) to your roadmap, and prioritise getting support in place."*

## TL;DR (the three load-bearing calls + the fastest path)

1. **Full 1.1 Compaction (sq-ixc3.4):** **hand-roll** the algorithm inside the
   dependency-free `serialize-rdf` feature. Rationale below — the alternative
   (delegate to the `json-ld` crate family) pulls a heavy async/context dep tree
   that would break lean-core and would have to be its own opt-in feature for
   little net code saving over a focused, prefix-context-only compaction; sparq
   already owns the writer that the algorithm extends.
2. **Framing:** **deferred from v1** (own future bead under the epic, not
   greenlit yet). It is a separate, larger algorithm with no current consumer;
   pulling it into v1 widens the conformance surface for no user-visible payoff.
3. **opt-in vs default-on:** **keep JSON-LD opt-in-buildable, but flip the
   shipped surfaces' defaults so JSON-LD is on wherever it is cheap.** Concretely:
   keep the `jsonld` (parse) and `serialize-rdf` (write) cargo features as
   opt-in toggles (lean default `cargo build -p sparq-wasm` stays under the
   `wasm_bundle_bytes` baseline), BUT turn JSON-LD ON in the native CLI/server
   binaries (no bundle-size constraint there) and keep it ON in the site bundle.
   This needs jeswr's sign-off because it changes the native server/CLI default
   feature set.

**Fastest path to "support in place":** **server content-negotiation** —
teach the SPARQL/GSP endpoints to (a) accept an `application/ld+json` request
body and (b) emit `application/ld+json` on `Accept` (bead **sq-oy1f.1** below).
The engine writer and the core parser already exist; this is wiring + a feature
flag on `sparq-server`, and it is the single biggest "JSON-LD is supported"
visible win because it lights up every HTTP client at once.

---

## 0. Correction to the brief's premise (honesty first)

The brief and the epic body are **substantially accurate**, but two premises
needed correcting against the actual `origin/main` code:

1. **The brief says "#900/sq-ixc3.5 added Store.serialize JSON-LD output …
   confirm the SITE consumes the engine writer."** The wasm `Store.serialize`
   JSON-LD output binding **does** exist (in `crates/sparq-wasm/src/serialize.rs`,
   behind `serialize-rdf`), and #923 added caller-supplied prefixes. **But the
   deployed site does NOT yet consume it for output.** The site's
   `js/package.json` `build:wasm` script builds `--features shacl,jsonld` —
   which gives JSON-LD **ingest** but does **not** enable `serialize-rdf`, and
   no site code calls `Store.serialize`. So today the **site has JSON-LD parse
   only, not serialise-out**; the serialise binding exists in the engine/wasm
   but is unwired in the site. (Verified: `site/src/lib/data-formats.ts`,
   `site/src/lib/repl-dataset.ts` treat `jsonld` as an *input* format only; no
   `.serialize(` call under `site/src`.)
2. **The brief implies JSON-LD output depends on the `jsonld` feature.** It does
   **not**. The `jsonld` feature gates only the `oxjsonld` *parser* (ingest).
   The JSON-LD *writer* lives under `serialize-rdf` and is a hand-rolled native
   emitter that links **zero** new dependencies (`serde_json` is dev-only). These
   are two independent opt-in features and the roadmap must treat them so.

Everything else in the epic's current-state inventory checks out.

## 1. What already ships (verified against `origin/main`)

### 1a. Parse (ingest) — `oxjsonld` behind the opt-in `jsonld` feature (sq-dvyi)

- `crates/sparq-core/Cargo.toml`: `jsonld = ["dep:oxjsonld"]`, with
  `oxjsonld = { version = "0.2", features = ["rdf-12"], optional = true }`.
  Locked at **oxjsonld 0.2.5**. Its dependency tree is **light** —
  `json-event-parser`, `oxiri`, `oxrdf`, `ryu-js` (all the same oxigraph family
  already in-tree). It is *not* a heavyweight processor.
- `crates/sparq-core/src/lib.rs`: `JsonLdParser::new()` is wired into the
  `Graph::load_str` / `load_str_with_base` / `load_dataset` / parallel-load paths
  behind `#[cfg(feature = "jsonld")]`; `is_jsonld_format` accepts `"jsonld"`,
  `"json-ld"`, `"application/ld+json"`. In a build *without* the feature, a
  `"jsonld"` format string errors (it is not silently mis-parsed as Turtle).
- `crates/sparq-wasm/src/lib.rs`: `Store::load`/`loadDataset` accept the JSON-LD
  format strings under the wasm `jsonld` feature (forwards to
  `sparq-core/jsonld`). The site's `build:wasm` (`--features shacl,jsonld`) turns
  it on, so the browser REPL upload/URL loader handles JSON-LD today.
- `crates/sparq-cli/src/main.rs`: `dump <file> <in-format> <out-format>` accepts
  `jsonld` as an **in-format** (parse) when the CLI is built with the relevant
  feature; the CLI links the parser via `sparq-core`.

**What oxjsonld 0.2.5 actually covers (its real capabilities + limits).** *(See
§4 / Sources for the documented confirmation; treat any item marked
"unverified" as an open check, not a claim.)* oxjsonld implements the **JSON-LD
1.1 → RDF (toRdf) direction**: context processing, expansion, and conversion to
RDF triples/quads. From its source (oxigraph `lib/oxjsonld`, modules `context`,
`expansion`, `to_rdf`, `from_rdf`, `profile`) it is **more complete than the
epic body assumed** — it implements the full Context Processing + Expansion
algorithms and supports **both JSON-LD 1.0 and 1.1** (`JsonLdProcessingMode`
defaults to 1.1):

- **Implemented (in):** local/inline `@context` resolution; `@import`; scoped
  contexts and **type-scoped** contexts (`@propagate`); `@nest`; `@json`
  (`@type: @json`); `@included`; `@reverse`/`@graph`/`@list`/`@set`; explicit
  1.0-vs-1.1 gating. A **streaming** profile
  (`with_profile(JsonLdProfile::Streaming)`) and a `lenient()` fast mode exist.
- **The two operational limits to design around:**
  1. **Remote `@context` fetching is opt-in, NOT built-in.** With no
     `LoadDocumentCallback` wired, loading an `http(s)` `@context` URL errors
     (*"No LoadDocumentCallback has been set to load remote contexts"*). So a
     sparq surface that must resolve remote contexts has to supply an HTTP
     loader (native: trivial; wasm: a fetch shim, or document the limit).
     There is also a `MAX_CONTEXT_RECURSION = 8` guard.
  2. **The serializer is prefix-`@context` only** — it is a *streaming*
     RDF→JSON-LD writer (full IRI predicates + a prefix/`@base` `@context`),
     explicitly *not* the full RDF-as-JSON-LD algorithm, with **no Compaction,
     Flattening, or Framing**. (sparq does not use oxjsonld's serializer — it
     hand-rolls its own writer under `serialize-rdf`, §1b.)

(All confirmed from the oxjsonld v0.2.5 source; see §4 + Sources.)

### 1b. Serialise (write) — native writer under `serialize-rdf` (no new deps)

- `crates/sparq-engine/src/serialize.rs`: `write_jsonld` / `graph_to_jsonld` /
  `graph_to_jsonld_with` (+ `_pretty` variants, sq-ixc3.3) emit JSON-LD 1.1 in
  three forms via `JsonLdForm::{Expanded, Flattened, Compacted}`. It is a
  **hand-rolled emitter** that "emits JSON by hand and pulls in nothing"
  (`serde_json` is a *dev* dependency for round-trip tests only), so
  `serialize-rdf` stays dependency-free — a stated lean-core property.
- **`Compacted` today = prefix-`@context` only.** `write_context` emits one
  `prefix → namespace` member per *used* prefix and abbreviates predicate/type
  IRIs to CURIEs; `@type` shorthand for `rdf:type`; `@language`/`@type` on value
  objects where needed. It does **NOT** implement the W3C 1.1 Compaction
  Algorithm: no term definitions, no `@vocab`, no type/language/container
  coercion, no `@reverse`, no value/node compaction against a caller-supplied
  `@context`. This is the **deferred sq-ixc3.4**, now un-deferred (jeswr is the
  consumer). The doc-comments are honest about this ("basic prefix `@context`").
- **No framing** anywhere. `Flattened` is the JSON-LD 1.1 *flattening shape*
  (every node gets an `@id`, dataset-as-`@graph`), but Framing
  (frame-document-driven subgraph reshaping) is entirely absent.
- **Round-trip property:** the writer is the inverse of the toRdf algorithm —
  feeding its output back through a JSON-LD→RDF processor reconstructs the same
  triples (tested against `serde_json` in dev).

### 1c. Surface coverage (which surfaces expose JSON-LD today)

| Surface | JSON-LD IN (parse) | JSON-LD OUT (serialise) |
|---|---|---|
| `sparq-core` `Graph::load_*` | yes — opt-in `jsonld` (oxjsonld) | — |
| `sparq-engine::serialize` | — | yes — opt-in `serialize-rdf` (expanded/flattened/prefix-compacted + pretty) |
| `sparq-wasm` `Store` | yes — opt-in `jsonld` (`load`/`loadDataset`) | yes — opt-in `serialize-rdf` (`Store.serialize`, #900/#923) |
| `sparq-cli` `dump` | yes — `jsonld` in-format | yes — `serialize-rdf` out-format (`jsonld[-…]`, `jsonld-pretty[-…]`) |
| Site REPL (`build:wasm = shacl,jsonld`) | yes — upload/URL parse | **NO — not wired** (bundle omits `serialize-rdf`; no `Store.serialize` call) |
| **`sparq-server`** SPARQL/GSP/TPF | **NO — absent** | **NO — absent** |

**The server is the big hole.** Verified in `crates/sparq-server`:

- **Read/output negotiation** (`negotiate.rs::negotiate_graph`, for
  CONSTRUCT/DESCRIBE + GSP reads) supports `application/n-triples`,
  `text/turtle`, `application/rdf+xml` — **no `application/ld+json`** branch.
- **Write/ingest** (`http.rs::rdf_format_for`, the GSP/upload body matrix)
  supports turtle / n-triples / n-quads / trig / rdf+xml — **no
  `application/ld+json`**.
- `crates/sparq-server/Cargo.toml` does **not** enable `sparq-core/jsonld` or
  `serialize-rdf`, so even the dependencies for server-side JSON-LD are not
  linked. Lighting up the server is a feature-flag + two match-arm change plus
  negotiation/content-type plumbing — the highest-leverage gap (see §5).

## 2. The W3C JSON-LD 1.1 algorithm surface, mapped to sparq

The JSON-LD 1.1 API (<https://www.w3.org/TR/json-ld11-api/>) defines six
algorithms; Framing is a separate Rec (<https://www.w3.org/TR/json-ld11-framing/>).
Mapped to sparq's current state:

| Algorithm | What it does | sparq state |
|---|---|---|
| **Expansion** | Resolve `@context`, make every term/IRI explicit, drop the context | shipped (in) via oxjsonld (toRdf subsumes expansion); shipped (out) as `JsonLdForm::Expanded` |
| **toRdf** (RDF Serialization) | JSON-LD document → RDF dataset | shipped (in) via oxjsonld |
| **fromRdf** (RDF Deserialization) | RDF dataset → expanded JSON-LD | shipped (out) — the native `Expanded` writer is exactly a fromRdf emitter |
| **Flattening** | One node object per subject, all under `@graph` | shipped (out) as `JsonLdForm::Flattened` |
| **Compaction** | Apply a caller `@context` (term defs / `@vocab` / coercion / `@reverse` / value+node compaction) | PARTIAL (out) — prefix-`@context` only; full algorithm = **sq-ixc3.4 (missing)** |
| **Framing** | Reshape/select a subgraph by a frame document | MISSING — no consumer; deferred |

**Net:** the **RDF↔JSON-LD bridge is functionally complete** (toRdf in via
oxjsonld, fromRdf/expanded/flattened out natively). The two genuine algorithm
gaps are **full Compaction** and **Framing**. Everything else is surfacing +
content-negotiation + conformance wiring.

## 3. The three load-bearing decisions (with honest trade-offs)

### 3a. Full 1.1 Compaction (sq-ixc3.4) — recommend HAND-ROLL

Two paths, both real:

- **(A) Hand-roll inside `serialize-rdf`.** Extends the writer sparq already
  owns. Keeps the dependency-free property (the lean-core principle). Cost: the
  Compaction + IRI-Compaction + Value-Compaction steps are mutually recursive
  and spec-conformance-sensitive — a non-trivial body of code to write *and
  maintain*, and exactly where subtle non-conformance bites interop.
- **(B) Delegate to the `json-ld` crate family** (timothee-haudebourg /
  spruceid `json-ld`, `json-ld-compaction`, …). Far less compaction code to
  *write*. Cost: a **heavy, async** dependency tree — `json-ld` v0.21.4 pulls
  the whole haudebourg RDF ecosystem (`iref`, `rdf-types`, `locspan`,
  `contextual`, `langtag`, `json-syntax`, **`futures`** — the API is async,
  `.await`-based), MSRV 1.83 — that breaks lean-core and would have to be its
  **own** opt-in feature (never folded into the dependency-free
  `serialize-rdf`). It duplicates context machinery the core does not otherwise
  want, and a second full JSON-LD stack alongside oxjsonld is a maintenance +
  binary-size liability. **Two further facts make (B) less attractive than the
  bead assumed:** (i) the `json-ld` crate has **no framing and (apparently) no
  fromRdf** — so delegating buys *only* compaction+flattening, not the rest of
  the gap; and (ii) its code activity appears to have **stalled (last
  substantive release Dec 2024)** — the 2026 crates.io timestamp looks like a
  metadata bump, so adopting it risks an unmaintained heavy dep. (No other
  independent full-conformance JSON-LD processor exists in Rust; `sophia_jsonld`
  and `ssi-json-ld` both wrap this same `json-ld` crate.)

**Recommendation: (A) hand-roll, scoped tightly.** The decisive factor is that
sparq's compaction need is *bounded*: a caller-supplied `@context` of term
definitions + `@vocab` + datatype/language coercion + container mappings, applied
to a graph the engine already emits. The full recursive value/node-compaction is
implementable as a focused pass over the existing node model without importing a
general-purpose processor. Path (B)'s code saving is real but is **paid for in
permanent dependency weight** that the project has repeatedly chosen against
(opt-in feature architecture). If conformance proves too costly to hand-roll to
the bar set by the test suite (§3d), (B) is the documented fallback — but it must
be gated behind its own feature (proposed name `jsonld-compact`), **never** inside
`serialize-rdf`. **sq-ixc3.4's approach is set to (A) by this record; it is NOT
duplicated.**

### 3b. Framing — recommend DEFER from v1

Framing is a separate Rec and a larger algorithm (frame matching, `@embed`,
`@explicit`, `@requireAll`, `@default`, `@omitDefault`) with **no current sparq
consumer**. Including it in v1 widens the conformance surface and the
hand-roll/delegate cost for zero user-visible payoff against the user's actual
ask (parse + serialise). Capture it as a **deferred** future bead under the epic
(blocked-on-design-review: only build with a concrete consumer, e.g. a GUI
"shape my export" feature). Note: a `manifest-frame.jsonld` already sits in the
vendored `tests/w3c/rdf-tests/` tree — that is the *rdf-tests* frame manifest,
not the JSON-LD framing suite, and does not imply framing support.

### 3c. opt-in vs default-on — recommend KEEP opt-in-buildable, FLIP shipped defaults

This is the key product decision, so state the constraint precisely. The lean
constraint is **not** "JSON-LD must be absent" — it is "the **lean default
`cargo build -p sparq-wasm` (no features)** must stay under the published
`wasm_bundle_bytes` baseline" (gated in `scripts/perf-gate.py` at a 2% auto
threshold; the baseline rises only on a deliberate RAISE). That metric tracks
*one specific build*: the lean browser bundle.

Therefore the recommendation splits by surface:

- **wasm lean default:** **stay opt-in.** Keep `jsonld` + `serialize-rdf` OFF in
  the bare `cargo build -p sparq-wasm` so the tracked `wasm_bundle_bytes`
  baseline is unchanged. (The site bundle already opts in to `jsonld`.)
- **native CLI + server binaries:** **turn JSON-LD ON by default.** There is no
  bundle-size gate on native binaries, oxjsonld is light, and JSON-LD's
  ubiquity (it is the dominant linked-data interchange format on the web) means
  a server that cannot speak `application/ld+json` is surprising. Concretely:
  enable `jsonld` + `serialize-rdf` in `sparq-cli` and `sparq-server` default
  features. **This needs jeswr's sign-off** because it changes the native
  default feature set (and adds a small amount to native binary size).
- **site bundle:** **add `serialize-rdf`** to `build:wasm` so the REPL can emit
  JSON-LD (closing the §0 gap), keeping `jsonld` on for ingest. The lean-bundle
  metric is unaffected (it tracks the no-feature build, not the site build).

Net product position: *JSON-LD is opt-in only where byte size is gated (the lean
wasm bundle); it is on everywhere a human or HTTP client actually meets sparq.*

### 3d. Conformance — recommend a ratcheted W3C JSON-LD test-suite gate

Mirror the existing ratchets in `crates/sparq-conformance` (the central
`scoreboard::SUITES` registry + per-suite floor consts that may only RISE; the
SPARQL/inference binaries and the crate-local SHACL/geo/Solid `cargo test`
runners). Wire the **official W3C JSON-LD 1.1 test suite** (the `w3c/json-ld-api`
repo `tests/` — expand/compact/flatten/toRdf/fromRdf/html/remote-doc — plus
`w3c/json-ld-framing` for frame). It is **manifest-driven exactly like the
vendored `rdf-tests` suite** (`manifest.jsonld` → per-category manifests; entries
typed `jld:PositiveEvaluationTest` / `jld:NegativeEvaluationTest` /
`jld:PositiveSyntaxTest`, with `input`/`expect`/`expectErrorCode` + an `option`
block for `processingMode`/`specVersion`/`base`/`useJCS`/`requires`). Approximate
category sizes (from the live manifests): expand ~385, toRdf ~467, compact ~246,
flatten ~58, fromRdf ~53, html ~50, remote-doc ~18; frame ~92. **For oxjsonld's
real surface, only `expand` and `toRdf` are meaningfully gateable today** (its
serializer is prefix-only); `fromRdf`/`expand`-out/`flatten`-out are gated against
the native *writer*. Wire as manifest-driven runners with monotonic pass-count
floors, one category at a time:

- **toRdf** — run JSON-LD docs through `oxjsonld` and compare RDF (the
  highest-value gate; it validates the *parse* path users hit today).
- **fromRdf / expand / flatten** — run RDF through the native writer; compare
  against expected expanded/flattened JSON-LD (validates the *writer*).
- **compact** — added when sq-ixc3.4 lands (gates the new algorithm).
- **frame** — only if/when framing is built (deferred).

Add the JSON-LD suites as new `Suite` rows in `scoreboard::SUITES` and a
crate-local floor const, so the consolidated scoreboard + the floor-sync guard
(`scoreboard_floors.rs`) cover them. **Honest note:** sparq will *not* be 100%
conformant on day one (negative/error tests, remote-context tests, and any
oxjsonld limit in §1a will fail) — the ratchet records the real pass count and
only forbids regressions, exactly as the SPARQL/SHACL ratchets do. Do **not**
claim full conformance; claim a rising floor.

## 4. External survey (oxjsonld limits, the `json-ld` crate family, test suite)

Grounded in primary sources (crate source, docs.rs, crates.io, W3C repos);
items I could not confirm are marked **unverified**, not asserted.

### 4a. oxjsonld v0.2.5 (the in-tree parser)

Source: oxigraph `lib/oxjsonld` (modules `context`, `expansion`, `to_rdf`,
`from_rdf`, `profile`); docs.rs/oxjsonld. Deliberately lean deps
(`json-event-parser`, `oxiri`, `oxrdf`, `ryu-js`, `thiserror`; optional `tokio`
for `async-tokio`) — **no HTTP client pulled in.** Parser supports full 1.1
Context Processing + Expansion (scoped + type-scoped contexts via `@propagate`,
`@import`, `@nest`, `@json`, `@included`, `@reverse`/`@graph`/`@list`/`@set`),
both 1.0 and 1.1 modes, a Streaming profile and a `lenient()` mode. **Limits:**
remote `@context` fetching requires a user-supplied `LoadDocumentCallback` (errors
otherwise; `MAX_CONTEXT_RECURSION = 8`); the serializer is a streaming
prefix-`@context`/`@base` writer with **no Compaction/Flattening/Framing** (sparq
does not use it — it hand-rolls its own writer). See §1a/§1b.

### 4b. The `json-ld` crate family (the only full-conformance Rust processor)

Source: `timothee-haudebourg/json-ld` v0.21.4; docs.rs/json-ld. Implements
**Expansion, Compaction, Flattening, toRdf** via `JsonLdProcessor`; **no framing
crate** in the workspace and **fromRdf not found (unverified, likely absent)**.
**Heavy + async:** pulls `iref`/`rdf-types`/`locspan`/`contextual`/`langtag`/
`json-syntax`/`futures` (async `.await` API; `reqwest`/`serde` optional), MSRV
1.83. It is the de-facto conformant processor (the W3C report's "Sophia (Rust)"
entry wraps it; `ssi-json-ld` also wraps it). **Maintenance: code activity appears
stalled — last substantive release 0.21.2 (Dec 2024); the 2026 crates.io
timestamp is likely a metadata bump (unverified as actively maintained).** This
is the §3a delegate candidate; the framing/fromRdf gaps + async heaviness +
staleness are why §3a recommends hand-roll.

### 4c. W3C JSON-LD 1.1 algorithms + test suite

API rec (<https://www.w3.org/TR/json-ld11-api/>): Expansion, Compaction,
Flattening, toRdf (Deserialize JSON-LD to RDF), fromRdf (Serialize RDF as
JSON-LD). Framing is a separate rec (<https://www.w3.org/TR/json-ld11-framing/>),
driven by `@embed`/`@explicit`/`@requireAll`/`@default`/`@omitDefault`. Test
suites: `w3c/json-ld-api` (`tests/`, manifest-driven, positive/negative entries,
~1,300 tests across expand/compact/flatten/toRdf/fromRdf/html/remote-doc) and
`w3c/json-ld-framing` (~92 frame tests). The W3C Processor Conformance report
shows even reference processors are **not at 100%** (compaction ~95–99.6%) — so a
**ratcheted pass-count floor** (the sparq pattern) is the right gate, not a
"fully conformant" claim. See §3d.

**Source URLs:** oxjsonld
<https://github.com/oxigraph/oxigraph/tree/main/lib/oxjsonld>,
<https://docs.rs/oxjsonld>; json-ld
<https://github.com/timothee-haudebourg/json-ld>, <https://docs.rs/json-ld>; W3C
<https://www.w3.org/TR/json-ld11-api/>, <https://www.w3.org/TR/json-ld11-framing/>,
<https://github.com/w3c/json-ld-api/tree/main/tests>,
<https://w3c.github.io/json-ld-api/reports/>,
<https://github.com/w3c/json-ld-framing>.

## 5. Prioritised implementation sequence (child beads under epic sq-oy1f)

Ordered by value — what gets "support in place" fastest first. Each is a future
bead; `--deps sq-oy1f` makes it a child of the epic. Beads needing jeswr's
design sign-off are marked **[design-review]**; greenlit ones **[ready]**.
sq-ixc3.4 already exists and is referenced (not duplicated).

1. **sq-oy1f.1 — Server JSON-LD content-negotiation (in + out)** **[ready]** —
   *the fastest path to "support in place."* Add `application/ld+json` to
   `negotiate.rs::GraphFormat`/`negotiate_graph` (CONSTRUCT/DESCRIBE + GSP read
   output) and to `http.rs::rdf_format_for` (GSP/upload write body); enable
   `jsonld` + `serialize-rdf` on `sparq-server`. Native-only, no bundle impact.
   Lights up every HTTP client at once. *(area:sparq-server)*
2. **sq-oy1f.2 — Wire JSON-LD conformance: toRdf + fromRdf/expand/flatten
   ratchet** **[ready]** — vendor the W3C JSON-LD 1.1 suite under `tests/w3c/`,
   add manifest-driven runners + monotonic floors, register in
   `scoreboard::SUITES` + the floor-sync guard. Establishes the honest baseline
   and protects every later change. *(area:sparq-conformance)*
3. **sq-oy1f.3 — Site REPL JSON-LD serialise-out** **[ready]** — add
   `serialize-rdf` to `js/package.json` `build:wasm`; expose JSON-LD as an
   *output* format in the REPL (consuming `Store.serialize`, not a TS reshaper).
   Closes the §0 site gap; lean-bundle metric unaffected. *(area:wasm / site)*
4. **sq-oy1f.4 — Promote JSON-LD to default-on in native CLI + server**
   **[design-review]** — flip `jsonld` + `serialize-rdf` into the default
   features of `sparq-cli` and `sparq-server`; keep the lean wasm bundle opt-in.
   Gated on jeswr's §3c sign-off (changes native default feature set). *(area:cli,
   area:sparq-server)*
5. **sq-ixc3.4 (EXISTING — un-deferred) — Full W3C 1.1 Compaction, hand-rolled
   under `serialize-rdf`** **[design-review]** — implement term defs / `@vocab` /
   type-language-container coercion / `@reverse` / value+node compaction against
   a caller `@context`, inside the dependency-free writer. Approach **set to
   hand-roll** by §3a; fallback to an own-feature `jsonld-compact` crate-delegate
   only if conformance cost forces it. Add the `compact` conformance category
   (extends bead 2). Gated on jeswr confirming hand-roll vs delegate. *(area:engine)*
6. **sq-oy1f.5 — Expose full Compaction through CLI + wasm + server** **[ready,
   blocked-on sq-ixc3.4]** — once sq-ixc3.4 lands, surface caller-`@context`
   compaction in `dump`, `Store.serialize`, and the server's JSON-LD output
   negotiation (e.g. honour a context profile/param). *(area:cli, area:wasm,
   area:sparq-server)*
7. **sq-oy1f.6 — JSON-LD Framing (DEFERRED)** **[design-review]** — frame-document
   subgraph reshaping + the `frame` conformance category. Build only with a
   concrete consumer (per §3b). Captured so the decision is recorded, not lost.
   *(area:engine)*

**Dependency shape:** 1, 2, 3 are independent and parallelisable (the greenlit
front). 4 depends on jeswr's §3c decision. sq-ixc3.4 depends on jeswr's §3a
confirmation; 5 depends on sq-ixc3.4. 6 is deferred. **Single fastest item:
sq-oy1f.1** (server content-negotiation).

## 6. Open questions for the maintainer

- **§3c default-on:** OK to flip `jsonld` + `serialize-rdf` ON in the native CLI
  + server defaults (lean wasm stays opt-in)? This is the product call.
- **§3a compaction:** confirm **hand-roll** for sq-ixc3.4 (vs delegate to the
  `json-ld` crate family behind a new `jsonld-compact` feature)? The trade-off is
  dependency weight vs spec-conformance maintenance burden.
- **§3b framing:** confirm framing stays **deferred** (own bead, build only with
  a consumer)? Or is there a near-term consumer (e.g. a GUI export-shaping
  feature) that pulls it into v1?
- **Compaction `@context` source on the server:** when a client requests
  `application/ld+json`, where does the caller `@context` come from — a default
  prefix context, an `?context=` URL param, or always expanded? (affects
  sq-oy1f.5's surface.)

## Sources

- **Code (origin/main):** `crates/sparq-core/{Cargo.toml,src/lib.rs}`
  (`jsonld` feature, oxjsonld ingest, `is_jsonld_format`);
  `crates/sparq-engine/{Cargo.toml,src/serialize.rs}` (`serialize-rdf`
  dependency-free writer, `JsonLdForm`, `write_context`, `graph_to_jsonld*`);
  `crates/sparq-wasm/{Cargo.toml,src/serialize.rs,src/lib.rs}` (`Store.serialize`
  #900/#923, `jsonld`/`serialize-rdf`/`scs` features);
  `crates/sparq-cli/src/main.rs` (`dump … jsonld[-…]`);
  `crates/sparq-server/src/{negotiate.rs,http.rs}` (`negotiate_graph`,
  `rdf_format_for` — no JSON-LD), `crates/sparq-server/Cargo.toml` (no
  `jsonld`/`serialize-rdf`); `crates/sparq-conformance/src/{lib.rs,scoreboard.rs}`
  + `tests/scoreboard_floors.rs` (the ratchet pattern); `scripts/perf-gate.py`
  (`wasm_bundle_bytes` gating); `js/package.json` (`build:wasm` features);
  `site/src/lib/{data-formats.ts,repl-dataset.ts}` (JSON-LD as input only).
- **Prior record:** `research/jsonld-pretty-compaction-scope.md` (sq-pxdu — the
  pretty/compaction scoping that produced sq-ixc3.3/.4/.5).
- **W3C:** JSON-LD 1.1 <https://www.w3.org/TR/json-ld11/>; JSON-LD 1.1 API
  <https://www.w3.org/TR/json-ld11-api/>; JSON-LD 1.1 Framing
  <https://www.w3.org/TR/json-ld11-framing/>.
- **External survey citations:** see §4.
- **Related beads:** sq-oy1f (epic), sq-ixc3.4 (compaction, un-deferred),
  sq-ixc3.5/#900 + sq-l5kr/#923 (wasm serialise-out), sq-dvyi/#519 (JSON-LD
  ingest), sq-ixc3.3 (pretty JSON-LD writer).
