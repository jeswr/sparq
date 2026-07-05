# Research-KB program — live literature ingestion, tiered public dump, and the ingest-iteration loop

<!-- 🤖 SPARQ agent (Claude Fable 5) — front-decomposition design record for epic sq-tzars. [FABLE-5] -->

**Status:** decomposition record — automated-dump plan ABORTED per #1552 (sq-yfh5d, 2026-07-05).
See "Dump location" below.
**Epic:** `sq-tzars` · **Maintainer directive:** 2026-07-05 · **GitHub:** #1110, #1111.
**Builds on (does not duplicate):** `research/provenance-driven-genai-kb.md` (the master
GenAI-KB record; "master record" below) and `research/dogfooding-sparq-knowledge-graph.md`
(the PKG provenance/ontology brief). Read those for the substrate design; read this for
what happens next and in what order.

## Dump location

The first KB dump was saved to **`sparq-org/research-kb`** (private repo) on 2026-07-05.
Path layout inside that repo:

```text
dumps/
  2026-07-05/
    pkg-hand-authored.ttl.gz   — hand-authored PKG tier
    pkg-ontology.ttl.gz        — PKG ontology export
    manifest.json              — per-tier statement counts, SHACL conformance, tool versions
    dump-provenance.ttl        — prov:Activity with generatedAtTime + prov:used
```

Browse at: https://github.com/sparq-org/research-kb/tree/main/dumps

**Automated-push plan status (aborted, #1552 / sq-yfh5d):** The maintainer confirmed
(2026-07-05) that because the dump is already saved in this repo, the recurring
automated-push plan is aborted. The `kb-dump.yml` workflow has been updated to:
- trigger only on `workflow_dispatch` (push-to-main trigger removed);
- skip gracefully when `KB_DUMP_TOKEN` is absent, printing a notice with the manual-refresh
  path, rather than hard-failing.

To refresh the dump in the future: run `.github/workflows/kb-dump.yml` manually via
the GitHub Actions UI, supplying a PAT with `Contents:write` on `sparq-org/research-kb`.

---

## 0. What this record is

The maintainer asked (2026-07-05, paraphrased): *properly start building a database of the
latest research on LLMs + knowledge representation, neurosymbolic AI, and whatever else
makes sparq better on performance, correctness, and usefulness of features — particularly
the genAI features' usefulness for self-improvement; a CORE API key is now provisioned
(Semantic Scholar is not available); save a dump of the database content in a git repo in
the sparq-org organisation; and iterate on the workflow for ingesting data into the store.*

This is the ONE design record for that program. It states the verified starting point, the
design decisions (with trade-offs), and the decomposition into nine disjoint child beads
under `sq-tzars`. One research PR (this file); N implementation PRs later, one per bead.

## 1. Corrected premise — this is NOT greenfield (verified 2026-07-05)

"Really properly start building" could read as "start from scratch". The tree says
otherwise, and the right move is to **extend the merged estate, not restart it**:

- **Implemented and verified** (all paths spot-checked against `origin/main` today):
  - `crates/sparq-kb` — reuse-first PKG ontology + SHACL shapes + `vocab.rs` byte-pin;
    `pkg-query` + the NL tool envelope (ADOPTED per the measured pkg-dogfood verdict).
  - Literature scaffolding behind the default-OFF `literature` feature (master record
    Phase 5, merged): `connector.rs` (`parse_openalex_batch` → DOI-keyed `SourceStub`),
    `extract.rs` (`Extractor` trait; `RecordedExtractor` is the only shipped impl),
    `ground.rs` (justification must be an entailed span; cited DOIs must resolve in-batch;
    quarantine-never-drop), `pipeline.rs` (`emit_turtle` stamps `prov:wasDerivedFrom` /
    `prov:wasGeneratedBy` / `prov:wasAttributedTo`), machine-tier SHACL caps
    (`pkg:confidence` ≤ 0.7, assurance ≠ `secx:Proven`).
  - `crates/sparq-kb/ingest/ingest_pkg.py` — deterministic bead/skill projectors through
    the same SHACL gate, quarantine sidecar. **Manual**, not CI-wired.
- **Designed only** (master record, unimplemented): named-graph tier separation (§4.7),
  any live connector, any live extractor, per-merge CI ingest, any dump/export.
- **Gaps verified directly** (not taken from any brief on faith):
  - `SourceStub` has **no license field** (`connector.rs`) — nothing captures licensing,
    which a public dump needs for redistribution tiering.
  - `prov:generatedAtTime` appears **nowhere** in the emitter or the shapes
    (grep-verified) — findings carry derivation but no retrieval timestamp.
  - `sparq-org` contains only `sparq`, `noir_XPath`, `noir_IEEE754` — the dump repo is
    net-new.
  - The committed literature fixtures are **fabricated** OpenAlex-shaped data (honest and
    fine for scaffolding tests; the live program should also record real responses).
- **Access reality:** CORE API v3 key provisioned (local env file only — never committed,
  logged, or echoed; this record deliberately does not restate its path). OpenAlex key +
  polite-pool mailto exist. Semantic Scholar is unavailable — the SPECTER2 near-dup layer
  stays designed-only, and any future S2 data carries a CC BY-NC / no-redistribution
  licensing constraint (already noted in the master record; do not block on it).

The old Phase-6 bead `sq-t5f3l` (gated live pilot, `[needs-access]`) is **superseded** by
this decomposition (closed with a note): its blocker is partially cleared by the CORE key
and its scope is now `sq-tzars.1` + `sq-tzars.6` + `sq-tzars.9`. Independent open items
`sq-2489d.6` (token A/B) and `sq-2489d.7` (DQV projection) are untouched by this program.

## 2. Decisions

Each is a `[FABLE-5]` judgment call made under the proceed-and-document rule; the
maintainer can steer post-hoc via the tracking issue.

1. **CORE v3 is the first live connector; OpenAlex live can follow.** CORE is what we have
   a key for and serves full-text-leaning search; the `SourceStub` boundary is already
   source-agnostic, so a CORE adapter (`parse_core_batch`) sits beside
   `parse_openalex_batch` at near-zero architectural cost. Trade-off: CORE's coverage of
   CS venues is weaker than S2's — accepted; S2 is unavailable.
2. **License capture is mandatory and fail-closed.** `SourceStub` gains
   `license: Option<String>`; `None` (unknown) is treated as non-redistributable
   everywhere downstream. Trade-off: over-restrictive for some genuinely-open records with
   missing metadata — accepted; a public dump must never leak by default.
3. **`prov:generatedAtTime` closes the retrieval-timestamp spec gap.** Neither the
   recovered provenance spec nor the emitter stamps *when* a finding was retrieved /
   generated; weighting-by-freshness and dump provenance both want it. Decision: stamp it
   on every machine finding and its retrieval `prov:Activity`, SHACL-require it on the
   machine tier. This is an addition to the recovered spec, in its spirit — flagged for
   maintainer review on the tracking issue rather than waiting on it.
4. **Live extraction = a Haiku sub-agent via the Claude Code CLI, behind the existing
   `Extractor` trait.** Per #1139 no Anthropic API key is needed on this box, which
   removes the last credential blocker for a live pilot. Record/replay stays the ONLY CI
   path (CI makes zero live model calls — a master-record hard rule this program keeps).
   Machine-tier caps are additionally enforced defensively at the extractor boundary
   (clamp ≤ 0.7, downgrade `Proven`), so the SHACL gate is the second line, not the only
   line.
5. **Tier separation v1 = per-tier Turtle artifacts + tier graph IRIs in `vocab.rs`, not
   in-store named graphs.** `sparq-kb` today loads one in-memory `Graph` per query; there
   is no persistent store to hold named graphs. Per-tier artifacts satisfy the two real
   requirements — the master record's §4.7 "queryable-apart" tiers and the dump's
   license partition — without inventing store machinery this program doesn't need.
   Honest scope statement: this is the *dump-sufficient slice* of §4.7, and in-store
   named-graph separation remains future work (it is not claimed).
6. **Tiers:** `hand-authored` (ingest_pkg.py projector output), `machine` (literature
   pipeline output), `license-restricted` (machine findings from unknown / absent /
   non-redistributable-licensed sources; public projection is metadata-only — DOI, title,
   year, license status, and **no abstract-derived text**).
7. **Dump repo = `sparq-org/research-kb`, created PRIVATE.** Layout:
   `dumps/YYYY-MM-DD/` holding per-tier `.ttl.gz` artifacts + `manifest.json` (per-tier
   statement counts, SHACL conformance, tool versions) + `dump-provenance.ttl` (the dump
   as a `prov:Activity` with `prov:generatedAtTime` + `prov:used`), plus a README stating
   tier semantics and license posture. Cadence: manual `workflow_dispatch` first; weekly
   only after the first audited pilot. **Flipping public is a maintainer decision**, gated
   on license-tier enforcement being mechanically verified (zero restricted statements in
   public artifacts, asserted at export time).
8. **Topic scope** (the seed registry transcribes this; every seed names the sparq
   attribute it serves):
   - *LLM × knowledge representation* and *neurosymbolic AI* (maintainer-named);
   - *query optimization / database systems* (serves performance);
   - *engine correctness + logic-bug testing* — TLP / metamorphic-testing family, feeding
     the existing papers/testing line of work (serves correctness);
   - *agentic memory + self-improvement* (serves genAI-feature usefulness — the KB's own
     reason to exist);
   - *RDF / SPARQL systems*;
   - *ZK-adjacent* literature only where relevant, and clearly scoped: ingested papers are
     knowledge **about the field**, and nothing ingested changes sparq's own verifier
     status — external accredited-cryptographer sign-off for the v1 verifier is still
     pending (`sq-qhy4`), and the MPC layer remains semi-honest-only.
9. **The ingestion-iteration loop is enforced in code, not in prose** (the maintainer's
   explicit "iterate on the workflow" ask): PREREG (the audit bar is written to the run
   sidecar *before* extraction; default proposal, maintainer-steerable: audited precision
   ≥ 0.8 on a ≥ 20-finding uniform sample — a pre-registered threshold choice, not a
   measured claim) → hard-capped **dry-run** (no KB mutation) → sample audit → grounding
   rate + conformance + quarantine counts recorded **verbatim, append-only** → an
   `adopt-topic | iterate | abandon` verdict → change `extract`/`ground` → repeat. Each
   iteration's sidecar is committed so the loop's history is auditable. No performance or
   quality number from a work-box run is canonical.
10. **What this program does NOT do:** no S2/SPECTER2 (unavailable); no live model or
    network calls in CI; no calibrated-confidence claim (master record §6.4 stays open —
    carried, not answered); no restart of the merged Phases 1–5.

## 3. The decomposition (nine disjoint child beads under `sq-tzars`)

Tier = the cheapest model that can do the fragment soundly. Every bead body carries the
full spec (what/why/where, `INVARIANT:`, acceptance); the table is the audit view.

| Bead | Fragment | Surface | Tier | Load-bearing invariant |
| --- | --- | --- | --- | --- |
| `sq-tzars.1` | CORE v3 live connector, `literature-live` feature, `SourceStub.license` | `sparq-kb` (connector) | sonnet | feature off-by-default; zero network in CI; key never logged; license captured, unknown ⇒ fail-closed |
| `sq-tzars.2` | `prov:generatedAtTime` on findings + activities, SHACL-required | `sparq-kb` (emitter/shapes) | haiku | timestamp-less machine finding is quarantined, never accepted |
| `sq-tzars.3` | Seed-query registry mapped to sparq attributes | `sparq-kb/ingest` (data) | haiku | every seed names a registered topic + attribute; data-only |
| `sq-tzars.4` | Per-merge PKG ingest CI (`ingest_pkg.py`) | `.github/workflows` | haiku | post-merge only; fail-closed with quarantine artifact; SHA-pinned |
| `sq-tzars.5` | §6.1/§6.2/§6.4 option tables + calibration evidence (spike) | none (comments) | sonnet | no invented answers; maintainer decides |
| `sq-tzars.6` | Live Haiku batch `Extractor` (Claude Code sub-agent path) | `sparq-kb` (extract) | sonnet | zero live model calls in CI; caps enforced at the boundary; errors surface |
| `sq-tzars.7` | Tier partition (hand-authored / machine / license-restricted) | `sparq-kb` (vocab/pipeline/ingest) | sonnet | exactly one tier per statement; unknown license ⇒ restricted; metadata-only public projection |
| `sq-tzars.8` | `sparq-org/research-kb` (PRIVATE) + tier-aware export + manifest | `scripts` + external repo | sonnet | private until maintainer flips; zero restricted statements or secrets in public artifacts |
| `sq-tzars.9` | Hard-capped dry-run pilot loop, prereg bar, honest metrics | `sparq-kb` (bin) | opus | no KB mutation before the pre-registered bar passes; caps fail-stop; metrics verbatim |

Acceptance tests (mechanical, per bead — the `verify` stage runs these):
`sq-tzars.1/.6/.9` → `cargo test -p sparq-kb --features literature,literature-live` (plus
default-build unchanged for `.1`); `sq-tzars.2/.7` →
`cargo test -p sparq-kb --features literature,validate` with an explicit fail-closed
negative case; `sq-tzars.3` → `python3 crates/sparq-kb/ingest/validate_seeds.py`;
`sq-tzars.4/.8` → `actionlint` + a dispatch/dry run; `sq-tzars.8` additionally the export
script's leak-check self-test (including an injected-leak negative); `sq-tzars.5` →
recorded maintainer decision.

### Disjointness audit (files per bead)

- `sq-tzars.1`: `crates/sparq-kb/Cargo.toml`, `src/literature.rs`,
  `src/literature/connector.rs`, `src/literature/connector_core.rs` (new),
  `src/literature/extract.rs` (one test literal for the new field),
  `fixtures/literature/core-batch.json` (new), `tests/core_connector.rs` (new).
- `sq-tzars.2`: `src/literature/pipeline.rs`, `shapes/literature.shapes.ttl`,
  `src/vocab.rs`, `tests/literature_pipeline.rs`.
- `sq-tzars.3`: `ingest/literature-seeds.toml` (new), `ingest/validate_seeds.py` (new).
- `sq-tzars.4`: `.github/workflows/pkg-ingest.yml` (new) — only.
- `sq-tzars.5`: no repo files (bead/issue comments; deliberately NOT a research PR, per
  the one-research-PR-per-epic rule).
- `sq-tzars.6`: `src/literature/extract_live.rs` (new), `src/literature.rs`,
  `Cargo.toml` (only if a dependency is genuinely needed).
- `sq-tzars.7`: `src/vocab.rs`, `src/literature/pipeline.rs`, `ingest/ingest_pkg.py`,
  `tests/literature_pipeline.rs`.
- `sq-tzars.8`: `scripts/export-kb-dump.py` (new), `.github/workflows/kb-dump.yml` (new),
  plus the external `sparq-org/research-kb` repo (created in that bead, not here).
- `sq-tzars.9`: `src/bin/literature_pilot.rs` (new), `Cargo.toml` (bin section).

No two **parallel** beads share a file. The shared files ride the dependency chains,
which exist for real ordering anyway and are marked non-parallel in the bead bodies:
`Cargo.toml` + `literature.rs` on `.1 → .6 → .9`; `pipeline.rs` + `vocab.rs` +
`tests/literature_pipeline.rs` on `.2 → .7`. The two new workflow files (`.4`, `.8`) are
distinct. `ingest/ingest_pkg.py` belongs to `.7` alone (`.4` invokes it unmodified;
`.3` adds only new files beside it).

### Dependency graph and dispatch waves

```text
wave 1 (parallel):  .1 (sonnet)   .2 (haiku)   .3 (haiku)   .4 (haiku)   .5 (sonnet, maintainer-decision)
                      |              |
wave 2:             .6 (sonnet)   .7 (sonnet)
                      |              |    \
wave 3:               +------+------+     .8 (sonnet)
                             |
                    .9 (opus, maintainer-arm)
```

Edges: `.1 → .6` (feature scaffolding + shared files), `.2 → .7` (timestamps precede
tiering + shared files), `.7 → .8` (export partitions by tier),
`{.1, .3, .6, .7} → .9` (the pilot runs the whole pipe and must land tiered + timestamped
data only). If the scheduler additionally serialises same-crate beads, dispatch wave 1 as
`.1` + `.3` + `.4` (three disjoint surfaces) and slot `.2` immediately after `.1` merges.

### Arm discipline

Fleet auto-arm is fine for `.1`–`.4` and `.6`–`.8` on green acceptance. **`sq-tzars.9` is
maintainer-armed** (first live data entering the KB; the run ends in a verdict, not a
commitment). **`sq-tzars.5` ends in a maintainer decision, not a merge.** The public flip
of `sparq-org/research-kb` is likewise the maintainer's call, not any agent's.

## 4. Items carried to the maintainer (not answered here)

1. Master record **§6.1** (DQV Note-status posture — arguably settled by the merged P3
   adoption, but the decision deserves a recorded yes/no), **§6.2** (research-verdict enum
   before bulk ingestion), **§6.4** (confidence-calibration source; until one exists,
   hedging reflects *asserted* assurance and no "calibrated" claim is made) — all packaged
   with evidence by `sq-tzars.5`.
2. The `prov:generatedAtTime` addition (Decision 3) — review post-hoc.
3. When to flip `sparq-org/research-kb` public, once the leak check is demonstrably
   enforced.
4. Note per #1111: several prior KB verdicts were measured under Opus; where a verdict
   gates a *decision* in this program (e.g. topic adopt/abandon in `.9`), re-running under
   the current strongest model is in scope for that bead's verdict write-up.

## 5. Verified source files (absolute, as inspected 2026-07-05)

- `/home/ubuntu/sparq/crates/sparq-kb/Cargo.toml` (feature ladder: `validate` / `query` /
  `close` / `literature`, all default-OFF)
- `/home/ubuntu/sparq/crates/sparq-kb/src/literature/connector.rs` (`SourceStub` — no
  license field today; `parse_openalex_batch`; `normalise_doi`)
- `/home/ubuntu/sparq/crates/sparq-kb/src/literature/extract.rs` (`Extractor` trait;
  `RecordedExtractor` only)
- `/home/ubuntu/sparq/crates/sparq-kb/src/literature/ground.rs` (entailed-span +
  in-batch citation grounding; quarantine-never-drop)
- `/home/ubuntu/sparq/crates/sparq-kb/src/literature/pipeline.rs` (`emit_turtle` stamps
  derivation/attribution provenance; no `generatedAtTime`)
- `/home/ubuntu/sparq/crates/sparq-kb/shapes/literature.shapes.ttl` +
  `shapes/pkg.shapes.ttl` (machine caps; no timestamp requirement today)
- `/home/ubuntu/sparq/crates/sparq-kb/ingest/ingest_pkg.py` (manual projectors + SHACL
  gate + quarantine sidecar)
- `/home/ubuntu/sparq/research/provenance-driven-genai-kb.md` (§4.7 tier requirement;
  §5 phase roadmap; §6 open questions)
- `/home/ubuntu/sparq/research/dogfooding-sparq-knowledge-graph.md` (provenance +
  explored-status source model)
