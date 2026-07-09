<!-- [FABLE] Remaining-work fleet decomposition for epic sq-oy1f, authored by Claude Fable 5 (Fable-tier architect stage). Delta record over research/jsonld-1.1-design.md — it refines the execution plan; it does not change the accepted architecture. -->
# JSON-LD 1.1 — remaining-work decomposition for the Fable-tier fleet (epic sq-oy1f)

> 🤖 SPARQ agent — decomposition record (architect stage). No code ships from this PR.

**Status:** accepted (proceed-and-document; maintainer may steer post-hoc — steering issue linked from the PR)
**Author:** Claude Fable 5 (SPARQ agent)
**Date:** 2026-07-07
**Epic:** sq-oy1f (user-prioritised P1, gh-757)
**Prior records:** `research/jsonld-1.1-design.md` (the accepted document-level-pipeline architecture, 2026-07-02), `research/jsonld-support-roadmap.md` (gap analysis), `research/jsonld-pretty-compaction-scope.md`

## 1. What this record is (and is not)

The epic already has an accepted architecture and a lettered [A]–[N] bead plan
(`research/jsonld-1.1-design.md` §10). This record does **not** re-architect. It is the
**execution re-cut of the remaining ~48% of the epic** so a cheap fleet
(sonnet/opus impl agents) can drain it in parallel waves with **zero shared-file merge
conflict**, plus the honest premise corrections found by re-verifying the codebase on
2026-07-07. One epic → one decomposition record; the child beads carry no PRs of their
own — each becomes exactly one impl PR at implementation time.

## 2. Verified build state (2026-07-07, origin/main)

Surveyed directly (crate sources, conformance harness, CI, surfaces) — not taken from
bead text:

| Piece | State |
|---|---|
| `crates/sparq-jsonld` scaffold ([A] sq-oy1f.23) | **Done.** Zero-dep crate; `json.rs` AST, full `JsonLdErrorCode` registry, `JsonLdOptions`, `DocumentLoader` (Noop/Fs). All future module files (`compact.rs`, `flatten.rs`, `node_map.rs`, `frame.rs`, `to_rdf.rs`, `from_rdf.rs`, `api.rs`) exist as **stubs already declared in `lib.rs`** — a later bead can fill its own file without touching `lib.rs`. |
| Context Processing / Create Term Definition / IRI expansion ([B] sq-oy1f.24) | **Done** (`context/process.rs`, `context/iri.rs`). |
| Inverse context / IRI compaction / term selection (sq-90mu3) | **Done** (`context/inverse.rs`). |
| Document-level Expansion ([C] sq-oy1f.25) | **Done** (`expand.rs`, incl. frameExpansion mode); the conformance expand lane already runs the **native document-level oracle** (sq-kk1mq, `EXPAND_FLOOR = 240/385`). |
| Compaction / Node-map / Flattening / Framing / to_rdf / from_rdf (document-level) | **Absent** — stub modules only. The shipping compact/frame writers are the RDF-first ones in `sparq-engine-serialize` with the documented ceiling. |
| Conformance lanes | One gated mega-file `crates/sparq-conformance/tests/jsonld_suite.rs` (~1.9k lines): toRdf 413/467 (oxjsonld, dataset isomorphism), fromRdf 51/53 (self-reparse round-trip), compact 186/246 (self-reparse), frame 61/92 (normative answer-equivalence over RDF), expand 240/385 (native document oracle), flatten 50/58 (RDF oracle). `html` + `remote-doc` are honest `NOT_IMPLEMENTED_CATS`. Negative tests are skipped everywhere. |
| Floor duplication | Each floor lives in **three places**: the test-file `const`, `scoreboard::SUITES.ratchet_floor` (textual sync guard covers this pair), and a **hard-coded shell floor in `ci.yml`** that the guard does *not* cover — this drift red-gated PR #1463 (bead sq-oy1f.40). |
| Surfaces | Server: `application/ld+json` conneg on CONSTRUCT/DESCRIBE + GSP read/write, always emits Flattened, **no profile params**, SD under-advertises JSON-LD (`SD_RESULT_FORMATS`/`SD_INPUT_FORMATS` omit it). CLI: full in/out token matrix incl. `jsonld-compact --context`; **no framing flag**. wasm: `serialize` + `serializeCompact`, opt-in, byte-floor guarded; **no framed output**. py: ingest default-on (sq-oy1f.20), **zero serialize-out**. |
| Engine facade split (#1542) | JSON-LD writers now live in `crates/sparq-engine-serialize` (re-exported verbatim through `sparq_engine::serialize`). The prior design record's `sparq-engine/src/serialize.rs` paths are stale; beads below use the real paths. |

## 3. Premise corrections (honesty first)

1. **sq-oy1f.4 ("DECISION: default-on for native CLI + server") is already implemented.**
   `jsonld` is in `default = [...]` of both `sparq-cli` and `sparq-server` (and `sparq-py`),
   each annotated `MAINTAINER-DIRECTED DEFAULT-ON EXCEPTION (sq-oy1f.4)`. The bead was
   never closed and shows as a P1 "blocked on design-review" — stale. **Action: close
   with evidence.** The remaining default-on question (oxjsonld-vs-native ingest default)
   is a *measured end-of-epic* decision and stays in [N].
2. **Three post-oracle bug beads (sq-oy1f.37/.38/.39) and two floor beads
   (sq-oy1f.22, sq-t92rs) overlap the lettered plan.** .37/.39 are both expand-lane
   correctness in `sparq-jsonld/src/expand.rs` (same file); .38 is expand-lane harness
   wiring (FsLoader for suite-local contexts); .22's expand half is the same chase and
   its flatten half is [D]'s oracle re-pin; sq-t92rs is explicitly absorbed by [G].
   Left as five beads they collide or duplicate. **Action: consolidate (§5).**
3. **[L] (sq-oy1f.34) spans four crates** (server, CLI, py, wasm) — it violates the
   ≤1-crate-per-bead conflict partition and would serialise the whole surface wave.
   **Action: split into four per-crate beads (§5).**
4. **[E]/[F] as written also rewire `sparq-engine-serialize`** ("engine delegates",
   "subsumes the core of serialize.rs") — putting the highest-risk shared file
   (behaviour-preservation of the public writer API) inside two otherwise-parallel
   algorithm beads. **Action: extract one dedicated engine-cutover bead (§5).**
5. The site `/try` playground was removed (#1516; `/app` is the single workbench) —
   surface beads must not reference it.

## 4. The disjointness problem and the enabler decision

**Hard invariant for the fleet: no two beads in the same wave touch the same file.**
Today every lane-flipping bead (D, E, F, G, H, J, I, K, the expand fixes) must edit the
same three files: `tests/jsonld_suite.rs` (all six lane runners + floors in one file),
`src/scoreboard.rs` (`SUITES.ratchet_floor`), and `ci.yml` (duplicated shell floors).
As cut, the epic's tail is forced serial.

**Options considered**

- *(a) Serialise everything with dep edges.* Sound but slow: ~9 beads single-file. Rejected.
- *(b) One-floors-file single-sourcing, allow same-file parallel edits in separated
  blocks.* Relies on git merge luck; violates the hard invariant. Rejected.
- *(c) Harness modularisation (chosen):* one enabler refactor (R0, re-scoped
  sq-oy1f.40) makes every subsequent lane bead's conformance footprint **per-lane
  files only**:
  - Split `tests/jsonld_suite.rs` into one thin root (mod decls only) +
    `tests/jsonld_suite/{common.rs, to_rdf.rs, from_rdf.rs, expand.rs, compact.rs,
    flatten.rs, frame.rs}` — still **one** test binary (submodules of the root), so the
    CI invocation and compile cost are unchanged.
  - Move the six floor constants into lib-side per-lane modules
    `src/floors/{to_rdf,from_rdf,expand,compact,flatten,frame}.rs`;
    `scoreboard::SUITES` **imports** them (compile-time single source — the textual
    sync guard for these six rows becomes structurally unnecessary and is retired for
    them; `all_crate_test_suites_are_guarded` is taught the lib-sourced pattern).
  - `ci.yml`'s belt-and-braces shell floors stop being hard-coded: the "Enforce
    ratchet" step **greps each floor from `src/floors/<lane>.rs`** at job runtime. The
    presence check (`^TOTAL <lane>` lines) stays. This *structurally* closes
    sq-oy1f.40's drift class instead of adding another textual guard.

  After R0, a floor re-pin touches exactly two per-lane files (`tests/jsonld_suite/<lane>.rs`
  behaviourally, `src/floors/<lane>.rs` for the value) and nothing shared. Registry
  *additions* (new lanes: negatives, remote-doc, html) still touch `scoreboard.rs` +
  `ci.yml` + `common.rs` — those three beads (J, I, K) are therefore placed in
  **different waves / an explicit ordering chain**, never parallel with each other.

**Within `crates/sparq-jsonld`** the crate-level "≤1 bead per crate" heuristic is
deliberately refined to file-level: the [A] scaffold pre-created every algorithm module
as a stub already wired into `lib.rs`, so D/E/F/G/H/J each own disjoint `src/` and
`tests/` files and none touches `lib.rs`. (Flat re-exports + the `api.rs` facade land
once, in [N].)

## 5. Bead consolidations and splits

| Action | Beads | Rationale |
|---|---|---|
| **Re-scope** sq-oy1f.40 → **R0 harness modularisation** | .40 | Same files as its original scope (`scoreboard_floors.rs`, `ci.yml`); the refactor *removes* the duplication the guard would have watched. |
| **Merge** sq-oy1f.38 + .39 (+ expand half of .22) → sq-oy1f.37 | .37 absorbs; close .38/.39 | All are expand-lane correctness/wiring in the same two files (`src/expand.rs`, expand lane). One bead = one PR = no collision. |
| **Close** sq-oy1f.22 | superseded | Expand half → .37; flatten half → [D]'s oracle re-pin (already noted in [D]). |
| **Close** sq-t92rs | superseded | [G] explicitly absorbs it (both beads already say so). |
| **Close** sq-oy1f.4 | already implemented | §3.1 evidence; nothing left to decide in it. |
| **Extract** engine-cutover → new bead | from [E]/[F] scope | The `sparq-engine-serialize` rewiring (public API preserved, internals delegate to the native pipeline) is its own single-crate, opus-tier bead behind D+E+F+G. [E]/[F] become pure `sparq-jsonld` + own-lane beads. |
| **Split** sq-oy1f.34 [L] → L1 server (keeps the id) + L2 CLI + L3 py + L4 wasm | 3 new beads | Four crates, four disjoint beads; L3 additionally closes the discovered py serialize-out gap, L1 additionally fixes the SD under-advertisement. |

Out of scope of this plan, unchanged and independently tracked: sq-oy1f.15 (SPARQL
*results* JSON-LD — non-goal per the design record, no W3C REC), sq-oy1f.21
(named-graph entailment in sparq-reason — not a JSON-LD bead; it merely lives under the
epic).

## 6. Wave plan (each wave internally file-disjoint)

Tiers are the cheapest sound model: `sonnet` for mechanical/refactor/wiring work with
crisp acceptance, `opus` for spec-algorithm fidelity and the behaviour-preservation
cutover. Nothing here is haiku-rote (every bead carries a both-feature-state gate and a
ratchet honesty rule). No ZK/MPC surface is involved.

| Wave | Bead | Crate(s) | Tier | Files (exclusive within the wave) |
|---|---|---|---|---|
| **W0** | sq-oy1f.40 **R0 harness modularisation** | sparq-conformance (+ci.yml) | sonnet | `tests/jsonld_suite.rs` → root + `tests/jsonld_suite/*`, `src/floors/*` (new), `src/scoreboard.rs`, `tests/scoreboard_floors.rs`, `.github/workflows/ci.yml` (jsonld job) |
| **W1** | sq-oy1f.26 **[D] node map + flattening** | sparq-jsonld + own lane | opus | `src/node_map.rs`, `src/flatten.rs`, `tests/flatten.rs` (new), `tests/jsonld_suite/flatten.rs`, `src/floors/flatten.rs` |
| **W1** | sq-oy1f.27 **[E] document-level compaction** | sparq-jsonld + own lane | opus | `src/compact.rs`, `tests/compact.rs` (new), `tests/jsonld_suite/compact.rs`, `src/floors/compact.rs` |
| **W1** | sq-oy1f.28 **[F] from_rdf** | sparq-jsonld + own lane | opus | `src/from_rdf.rs`, `tests/from_rdf.rs` (new), `tests/jsonld_suite/from_rdf.rs`, `src/floors/from_rdf.rs` |
| **W1** | sq-oy1f.37 **expand-lane correctness** (absorbs .38/.39) | sparq-jsonld + own lane | opus | `src/expand.rs`, `tests/expand.rs`, `tests/jsonld_suite/expand.rs`, `src/floors/expand.rs` |
| **W2** | sq-oy1f.29 **[G] framing** | sparq-jsonld + own lane | opus | `src/frame.rs`, `tests/frame.rs` (new), `tests/jsonld_suite/frame.rs`, `src/floors/frame.rs` |
| **W2** | sq-oy1f.30 **[H] native to_rdf + options + differential** | sparq-jsonld + own lane | opus | `src/to_rdf.rs`, `tests/to_rdf.rs` (new), `tests/jsonld_suite/to_rdf.rs`, `src/floors/to_rdf.rs` (differential asserted in-lane; **no** new `SUITES` entry, **no** `ci.yml` edit) |
| **W2** | sq-oy1f.31 **[J] negative lanes** | sparq-jsonld + conformance registry | sonnet | error-raising in `src/context/process.rs`/`src/expand.rs`/`src/compact.rs` (+ their crate tests), `tests/jsonld_suite/negative.rs` (new), `src/floors/negatives.rs` (new), `src/scoreboard.rs`, `ci.yml` (sole W2 registry-toucher) |
| **W3** | sq-oy1f.32 **[I] remote documents + SSRF policy** | sparq-jsonld + conformance registry | opus | `src/http.rs` (new), `src/loader.rs` (MockLoader), `sparq-jsonld/Cargo.toml` (`http-loader`), `tests/jsonld_suite/remote_doc.rs` (new), `src/floors/remote_doc.rs` (new), `tests/jsonld_suite/common.rs` (drop `remote-doc` from NOT_IMPLEMENTED), `src/scoreboard.rs`, `ci.yml`, `research/threat-model.md` |
| **W3** | sq-oy1f.41 **[CUT] engine-serialize cutover** | sparq-engine-serialize | opus | `crates/sparq-engine-serialize/src/*` only (public API byte-preserved; internals delegate: compacted = `from_rdf ∘ compact`, framed = `from_rdf ∘ frame`, expanded/flattened = native pipeline) |
| **W3** | sq-oy1f.35 **[M] pyld faithfulness lane** | CI + scripts | sonnet | `.github/workflows/jsonld-pyld.yml` (new), `scripts/pyld-faithfulness/*` (new) — advisory first |
| **W4** | sq-oy1f.33 **[K] HTML script extraction** | sparq-jsonld + conformance registry | sonnet | `src/html.rs`, `sparq-jsonld/Cargo.toml` (`html`), `tests/jsonld_suite/html.rs` (new), `src/floors/html.rs` (new), `tests/jsonld_suite/common.rs`, `src/scoreboard.rs`, `ci.yml` (ordering edge after [I] — shared registry files, marked NON-parallel) |
| **W4** | sq-oy1f.34 **[L1] server profile conneg** | sparq-server | sonnet | `src/negotiate.rs`, `src/graph.rs`, `src/http.rs`, `src/descriptors.rs` (profile params, Link context/frame under loader policy, SD advertisement fix) |
| **W4** | sq-oy1f.42 **[L2] CLI framing + options** | sparq-cli | sonnet | `src/main.rs` (+ CLI SKILL.md): `jsonld-framed[-pretty]` out-formats + `--frame <file>`, `--jsonld-base`, `--rdf-direction`; remote contexts only behind `--allow-remote-contexts` |
| **W4** | sq-oy1f.43 **[L3] py serialize-out** | sparq-py | sonnet | `src/lib.rs` (+ py README/SKILL): `serialize(format=...)` + `expand/compact/flatten/frame` with a pyld-style options dict — closes the py write-out gap |
| **W4** | sq-oy1f.44 **[L4] wasm framed + native forms** | sparq-wasm | sonnet | `src/serialize.rs`: `serializeFramed`, native-pipeline forms via the cutover; loader stays caller-supplied JS callback; lean bundle byte-floor unchanged |
| **W5** | sq-oy1f.36 **[N] consolidation** | cross-cutting (serial, last) | opus | all floors re-pinned (side-by-side statements), `api.rs` facade + flat re-exports + `publish` flip, AGENTS/SKILL/README sync, **measured** oxjsonld-vs-native default decision record, upstream oxigraph leniency issues filed |

Dependency edges (added where not already present): `.40 → {.26, .27, .28, .37}`;
`{.26, .27} → .29` and `.27 → .31` and `.28 → .30` (existing); `.37 → .31`
(shared `src/expand.rs`); `.30 → .32` (existing); `.31 → .32 → .33` (registry-file
ordering chain, NON-parallel by construction); `{.26, .27, .28, .29} → .41`;
`.41 → {.34, .42, .43, .44}`; `{.27, .29} → .35` (existing); everything open → `.36`.

Critical path: `.40 → .27 → .29 → .41 → .34 → .36` (6 serial steps; width 3–5
elsewhere).

## 7. Per-bead invariants and acceptance (the fleet contract)

Every bead carries these in `bd` (`--acceptance` + description); summarised here:

| Bead | Load-bearing invariant | Acceptance test |
|---|---|---|
| .40 R0 | **Pure refactor**: floor *values* byte-identical, all six lanes green at unchanged floors; ci.yml floors sourced from `src/floors/` (drift class removed) | `cargo test -p sparq-conformance --features jsonld-suite --test jsonld_suite` + `--test scoreboard_floors`; mutation spot-check: bump one floor const → suite must go red |
| .37 expand | Ratchet honesty: `EXPAND_FLOOR` only rises; no spec-error raised on positive cases | expand lane ≥ new floor; the previously-failing suite ids named in the bead flip to pass |
| .26 [D] | Flatten lane moves to the normative document oracle; re-pin side-by-side (oracle strengthening may lower the number, honesty stated in the PR) | `cargo test -p sparq-jsonld --test flatten` + flatten lane ≥ re-pinned floor |
| .27 [E] | Compaction is the spec algorithm over expanded docs (no self-reparse oracle); COMPACT_FLOOR re-pin side-by-side | `cargo test -p sparq-jsonld --test compact` + compact lane ≥ re-pinned floor |
| .28 [F] | fromRdf keeps round-trip isomorphism AND adds document-level comparison; floor re-pin side-by-side | `cargo test -p sparq-jsonld --test from_rdf` + fromRdf lane ≥ re-pinned floor |
| .29 [G] | Frame lane on native pipeline; targets the 28 known divergences; FRAME_FLOOR re-pin side-by-side | `cargo test -p sparq-jsonld --test frame` + frame lane ≥ re-pinned floor |
| .30 [H] | oxjsonld stays the default ingest; divergence between oxjsonld and native toRdf is a FAIL (fixed natively or filed upstream, never suppressed) | `cargo test -p sparq-jsonld --test to_rdf` + toRdf lane ≥ 413 (rising) + differential assertions green |
| .31 [J] | Wrong error code = FAIL, not pass; negative floors pinned at first measured value, rise-only | negative lane(s) ≥ pinned floors; mutation spot-check: mis-map one code → lane red |
| .32 [I] | **Fail-closed**: no ambient network — NoopLoader default everywhere; HttpLoader only behind `http-loader` + explicit allowlist; remote-doc lane hermetic (MockLoader) | both feature states build/test green; remote-doc lane ≥ pinned floor; a no-allowlist HttpLoader fetch must error |
| .41 [CUT] | Engine-serialize public API and behaviour preserved (all existing inline pyld-verified tests + fromRdf/compact/frame lanes green, floors not lowered) | `cargo test -p sparq-engine-serialize --features serialize-rdf` + full jsonld suite at standing floors |
| .35 [M] | Advisory-only until stable (cannot red-gate main); local `cargo test` stays hermetic | workflow runs on a corpus sample; documented promote-to-ratchet criterion |
| .33 [K] | `html` feature off = zero cost; scanner is not a DOM parser (scope pinned) | both feature states green; html lane ≥ pinned floor |
| .34 [L1] | No-profile requests keep today's Flattened output byte-stable; unsatisfiable Accept keeps 406 parity; Link-context deref only under loader policy (else 400 + spec error code) | `cargo test -p sparq-server --features jsonld` (+ default) incl. new profile-param integration tests |
| .42 [L2] | Both feature states green; remote contexts require the explicit flag | `cargo test -p sparq-cli` incl. new dump/load round-trip tests for framed form |
| .43 [L3] | Ingest surface unchanged; new fns mirror pyld naming; one direct unit test per new public fn (coverage floor) | `cargo test -p sparq-py` + maturin-built smoke test in CI lane |
| .44 [L4] | Lean default bundle byte-floor **unchanged** (perf-gate `wasm_bundle_bytes`) | wasm-pack test + the existing bundle-size gate in both feature states |
| .36 [N] | Floors only rise from their post-pipeline re-pins; the default-ingest decision is **measured**, never asserted | full suite green; scoreboard + docs consistency checks |

## 8. Honesty rules restated for the fleet

- **Floor re-pins under a stronger oracle may go down** — every re-pin PR states
  old-oracle vs new-oracle side by side (the sq-kk1mq precedent: 247 → 240). Never
  re-pin silently; after re-pin, rise-only.
- **Conformance counts are deterministic suite metrics** (pinned suite commits) — they
  are the only numbers beads may cite. No wall-clock performance numbers anywhere
  (work-box timings are non-canonical); the [N] ingest-default decision runs on the
  canonical bench harness.
- **The differential lane never papers over a divergence** — each one ends as a native
  fix or an upstream oxigraph issue, listed in the PR.
- **[I] is a security-sensitive surface** (SSRF): fail-closed invariant, opus tier, and
  its PR routes through the escalated adversarial-review path rather than fleet
  auto-arm.
- Known CI traps to brief every impl agent on: feature-gated intra-doc links from
  always-compiled doc-comments (use code spans), the readme-template hard gate, the
  coverage ratchet's need for one direct unit test per new public fn.

## 9. Decisions taken under proceed-and-document

1. **R0 harness modularisation + floor single-sourcing** (§4) — replaces sq-oy1f.40's
   original "extend the textual guard to ci.yml" with a structural fix (values sourced
   from one place; guard retired where the compiler now enforces sync).
2. **Bead consolidation** (§5): closes sq-oy1f.4 (already implemented), .22, .38, .39,
   sq-t92rs; splits [L]; extracts the engine cutover.
3. Both are recorded on the beads and in a short steering GitHub issue so the
   maintainer can redirect post-hoc.
