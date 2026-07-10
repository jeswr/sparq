# JSON-LD 1.1 full-support gap-scan — beyond expand (2026-07)

<!-- 🤖 SPARQ agent [FABLE-5] — FRONT-architect design record for epic sq-oy1f. -->

**Status:** decomposition verdict (design-only; no implementation in this PR)
**Epic:** sq-oy1f — full W3C JSON-LD 1.1 support (parse + serialise across all surfaces, user-prioritised)
**Prior records:** `research/jsonld-1.1-remaining-work-decomposition.md` (2026-07-07 fleet plan, PR #1764),
`research/jsonld-1.1-design.md`, `research/jsonld-support-roadmap.md`
**New child beads created by this record:** sq-oy1f.46 (fuzz), sq-oy1f.47 (EARL)

## 1. Verdict up front (premise correction)

The commissioning brief asked for a decomposition of the "not-yet-beaded" remainder of the
epic: **compaction, framing, content-negotiation, and the conformance lanes for each**. That
premise is largely **stale**: all four of those pillars were already decomposed into disjoint
fleet-contract beads by the 2026-07-07 plan and remain open with exclusive file lists, tiers,
invariants, and acceptance tests (§4). Re-cutting them would create duplicate beads and
merge-collision risk with the in-flight wave.

What a fresh survey of the actual code (2026-07-10, `origin/main` after #1857) DOES find
unbeaded is narrow and real:

1. **No fuzz coverage for JSON-LD at all** — while `application/ld+json` ingest is
   default-on for attacker-reachable surfaces (server GSP `PUT`/`POST` bodies) → **sq-oy1f.46**.
2. **No EARL report generation** — sparq runs six ratcheted W3C lanes but produces no
   machine-readable conformance evidence, so it cannot appear in the W3C json-ld-api /
   json-ld-framing implementation reports → **sq-oy1f.47**.

Everything else is either implemented-and-verified (§3), already beaded (§4), or an explicit
non-goal (§7). Two beads is the honest decomposition; padding it further would manufacture
work the wave plan already owns.

## 2. What "full JSON-LD 1.1" requires (spec map)

Normative sources: **JSON-LD 1.1** (W3C REC), **JSON-LD 1.1 Processing Algorithms and API**
(W3C REC, "API"), **JSON-LD 1.1 Framing** (W3C REC). Section numbers below follow the usage
already pinned in the epic's beads.

| Capability | Spec anchor | Owner |
| --- | --- | --- |
| Context processing + term definitions | API §4.1–4.2 | DONE (sq-oy1f.24) |
| Inverse context + IRI compaction / term selection | API §4.3, §7.1–7.2 | DONE (sq-90mu3) |
| Expansion (document-level, incl. frameExpansion) | API §5 | DONE (sq-oy1f.25/.37/.45) |
| Node map generation + flattening | API §6.1–6.2 | DONE (sq-oy1f.26, #1811); lane close-out in flight |
| **Compaction (document-level)** | API §7 | BEADED — sq-oy1f.27 |
| Serialize RDF as JSON-LD (fromRdf) | API §8.1 | BEADED — sq-oy1f.28 |
| Deserialize JSON-LD to RDF (native strict toRdf) | API §8.2 | BEADED — sq-oy1f.30 (oxjsonld stays default ingest) |
| **Framing** (`@embed`/`@explicit`/`@omitDefault`/`@requireAll`, value patterns, named graphs) | Framing REC (keywords + framing algorithm) | BEADED — sq-oy1f.29 |
| Error codes on negative cases | API error registry | BEADED — sq-oy1f.31 (registry itself DONE, sq-oy1f.23) |
| Remote documents / `@import` (loader, SSRF posture) | API remote document retrieval | BEADED — sq-oy1f.32 |
| HTML script extraction | API HTML content algorithms | BEADED — sq-oy1f.33 |
| **Content-negotiation** (`application/ld+json` + profile params `#expanded/#compacted/#flattened/#framed`, `Link` context/frame) | JSON-LD 1.1 REC, IANA considerations | base DONE (sq-oy1f.1); profiles BEADED — sq-oy1f.34 |
| **Conformance** (ratcheted W3C lanes per algorithm) | w3c/json-ld-api + w3c/json-ld-framing suites | 6 lanes LIVE (§3); negatives/remote-doc/html lanes beaded (.31/.32/.33) |
| Public conformance evidence (EARL) | EARL 1.0 Schema; suite `reports/` convention | **GAP → sq-oy1f.47** |
| Adversarial-input robustness (fuzz) | n/a (house threat-model discipline) | **GAP → sq-oy1f.46** |

## 3. Implemented and verified on `origin/main` (2026-07-10)

Verified against the code, not the epic text.

- **Parse (toRdf)**: oxjsonld ingest, default-on for native CLI + server (decision sq-oy1f.4);
  wasm stays opt-in behind the lean-bundle byte floor.
- **Native pipeline crate** `crates/sparq-jsonld` (zero mandatory deps, `publish = false`):
  `json.rs` AST, full closed `JsonLdErrorCode` registry, `JsonLdOptions`, `DocumentLoader`
  (Noop deny-by-default / Fs), context processing + inverse context (~3.0 kLOC), `expand()`
  (~1.4 kLOC), `node_map.rs` + `flatten()`. `compact.rs` / `frame.rs` / `from_rdf.rs` /
  `to_rdf.rs` are **declared stubs** awaiting their beads — no `todo!()` anywhere.
- **Legacy RDF-first writers** in sparq-engine-serialize still power compacted/framed output
  today; the native pipeline replaces their internals via the cutover bead (sq-oy1f.41) with
  the public API preserved.
- **Conformance lanes** (harness modularised, lib-side single-source floors, ci grep — sq-oy1f.40).
  Floors on `origin/main`; rise-only; denominators are per the pinned suite manifests
  (`scripts/fetch-jsonld-tests.sh` / `fetch-jsonld-framing-tests.sh`, see `src/floors/*.rs`):

  | Lane | Floor | Oracle today |
  | --- | --- | --- |
  | toRdf | 413 (of 467) | oxjsonld → canonical-dataset isomorphism vs normative N-Quads |
  | expand | 276 | native `expand()` vs normative expected document (`json_ld_equal`) |
  | flatten | 53 (of 58) | native `flatten()` vs normative expected document |
  | compact | 186 (of 246) | RDF round-trip equivalence (upgrades to normative document oracle in sq-oy1f.27) |
  | frame | 61 (of 92) | RDF-equivalence vs normative expected doc (native re-pin in sq-oy1f.29) |
  | fromRdf | 51 (of 53) | writer round-trip isomorphism (document comparison added in sq-oy1f.28) |

- **Surfaces already live**: server accept+emit `application/ld+json` on CONSTRUCT/DESCRIBE +
  GSP read AND GSP `PUT`/`POST` bodies (sq-oy1f.1, tests in
  `crates/sparq-server/tests/jsonld_content_negotiation.rs`); CLI full in/out token matrix
  incl. `jsonld-compact --context`; py ingest default-on (sq-oy1f.20); wasm
  expanded/flattened/prefix-compacted + `serializeCompact`; GUI import/export lists JSON-LD
  (`gui/app/src/lib/rdf-format.ts`).

## 4. Already beaded — do not re-cut

The 2026-07-07 wave plan stands unchanged (W0 done → W1 .27/.28 [+.26 close-out in flight] →
W2 .29/.30/.31 → W3 .32/.41/.35 → W4 .33/.34/.42/.43/.44 → W5 .36). The brief's three
"missing" pillars map onto it as:

- **Compaction** → sq-oy1f.27 (algorithm + compact lane to the normative oracle) feeding
  sq-oy1f.41 (engine cutover) and the surface beads.
- **Framing** → sq-oy1f.29 (native framing incl. the 28 known divergences; absorbs sq-t92rs)
  + surface slices sq-oy1f.42 (CLI `--frame`), .43 (py), .44 (wasm `serializeFramed`).
- **Content-negotiation** → sq-oy1f.34 (Accept profile params + `Link` context/frame under
  loader policy + SD advertisement fix); sq-oy1f.15 (P3) keeps the non-REC SELECT/ASK
  `ld+json` question separate.
- **Conformance** → per-lane re-pins ride the algorithm beads; negatives sq-oy1f.31;
  remote-doc sq-oy1f.32; html sq-oy1f.33; third-party faithfulness sq-oy1f.35; final floor
  re-pin + docs + the measured oxjsonld-default decision sq-oy1f.36.

## 5. New bead A — sq-oy1f.46: JSON-LD fuzz target (fail-closed on arbitrary bytes)

**Why it is load-bearing now.** `fuzz/fuzz_targets/` covers N-Triples/N-Quads/Turtle/TriG
(`parse_rdf_str`), SPARQL, `graph_open`, parallel load, and SHACL — but **no JSON-LD**, even
though GSP `PUT`/`POST` accepts `application/ld+json` bodies default-on (attacker-controlled
bytes reach oxjsonld today) and the native pipeline adds recursion-rich surface (deep
`@context` chains, scoped-context re-expansion, node-map blowup) as the wave lands.

**Shape.** `fuzz/fuzz_targets/jsonld_pipeline.rs`: bytes → sparq-jsonld JSON parse →
`expand()` (NoopLoader, spec-default options) → `flatten_expanded()`; the same bytes through
the engine's oxjsonld ingest entrypoint (one or two `[[bin]]` targets, implementer's choice).
Committed minimal handcrafted seeds in `fuzz/seeds/jsonld_pipeline/` (no vendored W3C files).
No workflow edit: the CI fuzz lane auto-discovers targets via `cargo fuzz list` (sq-o4pi
precedent), which also keeps this bead clear of sq-c9q4r (fuzz.yml hardening). Crashes
discovered by fuzzing are filed as separate bug beads, not fixed in-bead.

**Invariant:** fail-closed — structured errors, never a panic / stack overflow / abort;
recursion + size guards bounded.
**Acceptance:** `cargo +nightly fuzz build` green; the fuzz.yml PR replay leg replays the
seed corpus exactly-once with zero crashes.
**Tier:** sonnet. **Files (exclusive):** `fuzz/Cargo.toml`, `fuzz/fuzz_targets/jsonld_pipeline.rs`,
`fuzz/seeds/jsonld_pipeline/`.

## 6. New bead B — sq-oy1f.47: EARL report emitter (post-[N], dep-sequenced)

**Why.** The W3C implementation reports for json-ld-api and json-ld-framing are built from
submitted EARL; sparq generates none (the only EARL in-tree is the vendored ruby reference
report). sq-hmd7l.22 already plans to compare against *peers'* EARL — sparq should have its
own, generated from the same runs that enforce the floors, as the public, machine-readable
conformance claim.

**Shape.** Env-gated EARL 1.0 (Turtle) emitter: one `earl:Assertion` per manifest test
(`earl:passed` / `earl:failed` / `earl:untested` for honest-skips and NOT_IMPLEMENTED
categories) + `doap:Project` metadata, emitted from the per-lane runners' per-case outcomes
(single source — never a re-run under a different oracle). A self-check test asserts the
per-lane `earl:passed` counts equal the scoreboard's lane pass counts. CI uploads the `.ttl`
as an advisory artifact — not a gate.

**Sequencing (the one real dep edge).** The per-lane runner files, `common.rs`, and `ci.yml`
are owned by the in-flight W1–W5 fleet contracts. Rather than silently emitting a colliding
bead, sq-oy1f.47 is dep-sequenced after sq-oy1f.36 ([N] close-out) and marked NON-parallel;
it takes exclusive ownership of its file list only then.

**Honesty rule baked into the bead:** the report claims only measured outcomes — failures are
emitted as `earl:failed`, never omitted; no blanket "conformant" statement while any lane
floor is below its lane total.

**Invariant:** emitted EARL mirrors the measured lane outcomes exactly (self-check enforced);
floor rise-only discipline untouched.
**Acceptance:** `cargo test -p sparq-conformance --features jsonld-suite --test jsonld_suite`
green incl. the earl self-check; the emitted `.ttl` parses with sparq's own Turtle parser.
**Tier:** sonnet. **Files (exclusive, post-.36):** `crates/sparq-conformance/src/earl.rs`,
`tests/jsonld_suite/earl.rs`, minimal per-case plumbing in `tests/jsonld_suite/common.rs`,
ci.yml artifact-upload lines.

## 7. Surveyed and explicitly NOT beaded

- **sparq-solid Accept headers** — corrected premise: `crates/sparq-solid` is the
  authorization layer (WAC/ACP/ODRL/trust); it fetches no RDF documents and has no
  content-negotiation surface. Nothing to wire.
- **GUI** — already imports/exports JSON-LD via the engine writer; it inherits the native
  pipeline through the sq-oy1f.41 cutover for free. Framed export in the GUI needs a frame
  document UX and has no consumer — non-goal until one exists.
- **Benchmarking** — owned by sq-hmd7l.15 (jsonld-bench suite registered in
  `bench/benchmarks.toml`, competitors jsonld.js + titanium-json-ld pre-registered). Not
  duplicated here; the epic's perf angle rides sq-oy1f.36's measured oxjsonld-vs-native
  decision.
- **A generic document-API processor surface in wasm** (jsonld.js-style
  `expand(doc)`/`compact(doc, ctx)` on raw documents, detached from a Store) — scope growth
  beyond "parse + serialise across all surfaces"; revisit only with a named consumer.
- **JSON-LD streaming profile** (W3C Note, not a REC) and **CBOR-LD** (separate WG, not a
  REC) — out of the epic's "full W3C JSON-LD 1.1" definition.
- **SELECT/ASK solution sets as `ld+json`** — no W3C REC exists; stays the separate P3
  decision bead sq-oy1f.15, unchanged.
- **sq-p91s8** (expand_value `{}` node-ref edge, P3) — its own text defers it until "the
  normative document-level expand oracle lands"; that oracle HAS landed (sq-kk1mq/.25/.45),
  so it is now actionable as a normal expand-lane ratchet fix. It remains an existing bead —
  noted here so the next expand-lane pass picks it up rather than re-discovering it.

## 8. Phased plan (delta only)

Existing waves W1–W5 are unchanged and authoritative (`research/jsonld-1.1-remaining-work-decomposition.md` §7).
Delta from this record:

- **sq-oy1f.46 (fuzz)** — no dependencies, touches only `fuzz/`; launchable immediately, in
  parallel with any wave.
- **sq-oy1f.47 (EARL)** — W6: strictly after sq-oy1f.36 (dep edge in place), serial with
  nothing else expected to be in flight on sparq-conformance at that point.

Disjointness: the two new beads share no files with each other or with any open bead's
exclusive file list; sq-oy1f.47's only overlap risk (harness/common/ci.yml) is resolved by
the dep edge, not by hope.
