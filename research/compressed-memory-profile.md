<!-- [FABLE] Authored by Claude Fable 5 (SPARQ architect, Fable-tier decomposition stage).
Epic sq-7d3dj.32.2 (parent sq-7d3dj.32, D7 memory gap). One design record + disjoint
child beads; no implementation in this PR. 2026-07-07. -->

# Block-compressed in-RAM store as a first-class native memory profile

**Status:** decomposition design record (design-only; child beads carry the implementation).
**Date:** 2026-07-07. **Bead:** sq-7d3dj.32.2 (parent sq-7d3dj.32).
**Feeds:** the D7 row of `research/perf-dominance-gap-2026-07.md`; composes with
`research/memory-per-triple-2026-07.md` (the canonical measurement record) and
`research/compressed-perms-verdict.md` (the on-disk analogue's adopted format + latency data).

---

## 0. Problem

The raw six-permutation in-RAM store measures a canonical **84.4 B/triple heap at 50M
triples** (93.4 at 1M) against RDFox's published 34.7–36.9 B/triple best in-memory claims
and 45–85 B/fact product band (`research/memory-per-triple-2026-07.md` §2, envelope sha
`5ce8b4f9`, canonical quiet aarch64 instance, 2026-07-07). The **already-built**
block-compressed in-RAM mode (`Graph::into_compressed`) measures **54.5 B/triple heap at
50M** — inside RDFox's product band — but it is not an operating point anyone can actually
select, compare, or trust:

- **No native surface.** No CLI load path (`query` / `bench`) can select it; only the
  browser wasm binding (`Store.loadCompressed`) and two diagnostic subcommands reach it.
- **No query-cost measurement at scale.** The per-scan block-decode cost of the *in-RAM
  owned-blocks* mode is unmeasured beyond a single-query option in `compare-compress`.
  The 1.5–3.4× lazy / +22%-worst eager numbers in `research/compressed-perms-verdict.md`
  are the *mmap/disk* analogue at ≤10M, not this mode at 50M.
- **No regression gate.** `store_bytes_per_triple` ratchets the raw layout; nothing
  ratchets the compressed layout, so its B/triple could silently regress.
- **Unexplained scale behaviour.** Compressed store B/triple grows 36.75 (1M) → 48.75
  (10M) and then **plateaus** at 48.75 (50M) — un-root-caused.

This record decides what "first-class" means, states the query-time cost model from the
as-built code, frames the scale-growth root cause as testable hypotheses, and cuts the
work into disjoint child beads.

## 1. Verified estate (what already exists on `main`)

Verified against `origin/main` @ `06318fd8b`, not taken from prior docs.

### 1.1 API and surfaces

| Surface | Where | State |
|---|---|---|
| `TripleStore::from_triples_compressed` | `crates/sparq-core/src/store.rs` | implemented + differential-tested (`compressed_scans_match_raw`) |
| `Graph::into_compressed` / `load_str_compressed` | `crates/sparq-core/src/lib.rs` | implemented + round-trip tested; also compacts the dict to a blob |
| `Graph::save_compressed` / `open` auto-detect / `decompress_indexes` | core, `mmap` feature | implemented (`SPQCPRM1` magic; `compressed_save_open_roundtrip`) |
| wasm `Store.loadCompressed` | `crates/sparq-wasm/src/lib.rs` | implemented (`compressed_matches_raw` test) |
| CLI `save [compressed]`, `recompress`, `query-mmap` (auto-detect), `bench-mmap [decompress]`, `compare-compress`, `probe-compress` | `crates/sparq-cli/src/main.rs` | implemented |
| CLI `memstat`, `scripts/bench/bytes-per-triple.sh` | **NOT on main** | exists only in the D7 instrument worktree commit `5ce8b4f9` (locally recoverable); `research/memory-per-triple-2026-07.md` cites it — a reproducibility gap this plan closes (§7, beads 1–2) |
| Engine + substrate consumers | `crates/sparq-engine`, `crates/sparq-substrate` | every path goes through `scan`/`scan_sorted`/`estimate`/`contains` (Cow-based); the `as_slice` `unreachable!` is dead code for compressed perms; the vectorized/LFTJ substrate consumes materialized rows/tries, never `PermData` directly. End-to-end parity gated by `crates/sparq-engine/tests/storage/compressed_build_differential.rs` |

So: correctness of the compressed mode is **implemented-and-verified** (differential tests
at small scale). What is missing is *surface, measurement at scale, gating, and the
scale-growth explanation* — all product/ops work, no core-engine change required.

### 1.2 Canonical memory numbers (2026-07-07 envelope, sha `5ce8b4f9`)

From `research/memory-per-triple-2026-07.md` (canonical, quiet instance; full protocol
there):

| Scale | heap raw | heap compressed | store raw | store compressed | disk raw | disk compressed |
|---|---|---|---|---|---|---|
| 1M (1 085 794) | 93.41 | 42.50 | 84.14 | 36.75 | 80.59 | 38.14 |
| 10M (10 889 330) | 88.56 | 54.73 | 79.26 | 48.75 | 80.32 | 40.23 |
| 50M (54 557 829) | 84.41 | 54.54 | 75.38 | 48.75 | 80.22 | 44.45 |

Honest placement: compressed heap ~54.5 B/triple at scale is **inside RDFox's 45–85
product band, not below its 34.7–36.9 best claim**. The raw-mode levers (sibling beads
.32.1 slack-shrink, .32.3 compact-index, .32.4 dict blob) attack the same gap from the
raw side; this profile is the strongest single already-built lever and composes with .32.4
(it already blob-compacts the dict: 8.3 → 5.8 dict B/triple).

## 2. The design decision: what "first-class" means

Three candidate meanings were on the table:

- **(a) Store-open/load-time mode flag** — a runtime profile selection on the native load
  paths, routing to the existing `into_compressed()`.
- **(b) Per-permutation adaptive choice** — compress some permutations (e.g. the rarely
  scanned ones) automatically.
- **(c) Rebuild-into profile only** — status quo: the API exists, callers who know call it.

**Decision [FABLE]: (a), composed over the existing (c) mechanism.** "First-class" is
defined as the conjunction of four properties, each carried by one child bead:

1. **Selectable** — one environment variable, `SPARQ_STORE_PROFILE`, read in the CLI's
   shared load helper so `query`/`bench`/`reason`/`scaling` inherit it uniformly:
   unset or `raw` → byte-identical current behaviour; `compressed` →
   `Graph::into_compressed()` after load; any other value → hard error (fail-closed, no
   silent typo fall-through). An env var (not a per-subcommand positional) because the
   positional grammars are already crowded, the house precedent exists
   (`SPARQ_BUILD_COMPRESSED`), and the bench harness needs to A/B the *same* command line.
2. **Measured** — per-query WatDiv latency raw-vs-compressed at 1M/10M/50M under the
   canonical quiet-EC2 protocol, envelope-recorded.
3. **Gated** — a deterministic `comp_store_bytes_per_triple` ratchet in ci-bench, exactly
   like the raw floor.
4. **Documented** — `skills/cli/SKILL.md` states the profile, its measured trade, and
   when to choose it.

**Not a cargo feature.** The compressed code paths are compiled unconditionally today
(`PermData::Compressed` is not feature-gated); the choice is per-*workload*, not
per-*build*, so a feature would fragment the test matrix for zero bundle saving. Core
stays lean: no new crate, no new core API — the profile is pure surface over verified
mechanism (`feedback-opt-in-feature-architecture` satisfied by runtime opt-in +
default-path byte-identity).

**(b) per-permutation adaptive is REJECTED for v1**, for three reasons: (i) it destroys
the profile's core contract — a *predictable* memory operating point an operator can size
a box against; (ii) the planner has no per-permutation access-cost model, so adaptive
placement creates latency cliffs the optimizer cannot see; (iii) choosing *which* perms to
compress needs exactly the per-query cost data bead 3 produces. A **hybrid** operating
point (SPO raw + five compressed: ≈ +12 B/triple buys raw-speed `contains()` and
SPO-order scans) is the designed follow-up *if* the measurements show point-lookup/
membership dominance — decide on data, not intuition.

**Honest scope limit:** this profile reduces **steady-state resident footprint only**.
Load-time peak RSS is *not* reduced — `into_compressed` (and `load_str_compressed`)
build the raw perms first and encode from them, so the peak still includes the raw build
(canonical hwm 147 B/triple at 50M raw path). The genuinely peak-bound path remains the
external build (`SPARQ_BUILD_COMPRESSED=1` + `open`, extbuild hwm 73.4 B/triple at 50M).
Docs must not oversell this.

## 3. Query-time cost model (from the as-built encoding)

From `crates/sparq-core/src/compress.rs` on `main`:

- **Encoding:** `BLOCK = 128` rows/block; per block a count varint, the first row
  absolute, then lexicographic deltas where a change in a leading column stores the
  trailing columns **absolute** (LEB128). Directory: one `([Id;3], u32)` per block
  (16 B/block = 0.125 B/triple resident). The in-RAM byte stream and the on-disk
  `SPQCPRM1` stream are identical by design (mmap serves scans with no transcode).
- **Point/prefix probe** (bound leading columns): binary-search the directory, decode 1–2
  blocks (≈ ≤256 rows of varint decode) into an owned `Vec`, trim. Versus raw: a
  zero-allocation borrowed binary-search sub-slice. Cost = a small constant per scan call
  — dominated by one allocation + O(BLOCK) decode.
- **Large range / full scan:** decode ⌈span/128⌉ blocks — O(range) varint decode plus an
  owned materialization the raw mode never pays. This is where the latency delta should
  concentrate; WatDiv's C/F (join-heavy, larger scans) vs L/S (lookup-shaped) split will
  expose it per shape.
- **`estimate()` (planner):** decodes at most the two boundary blocks (O(BLOCK)) — plan
  choice is cost-stable across modes.
- **`contains()` / update paths:** each membership probe (`apply_delta`'s
  `base_contains`, per triple) becomes 1–2 block decodes instead of one binary search —
  the sharpest *relative* regression in the model. The v1 measurement (bead 3) covers the
  read suite; update-workload cost is flagged as a known unmeasured axis, not claimed.
- **No decoded-block cache** — every scan re-decodes. Deliberate for v1: a cache
  reintroduces unbounded working-set variance, defeating the predictable-footprint
  contract. Re-visit only with measurement (rejected-for-now, §8).

Prior evidence to calibrate expectations (NOT a claim about this mode): the disk-analogue
lazy mmap-compressed mode measured 1.5–3.4× (one query 5.9×) at ≤10M
(`research/compressed-perms-verdict.md`); the in-RAM owned mode avoids page faults, so it
should sit at or below those ratios — **to be measured, not asserted** (beads 2–3).

## 4. Root cause frame: compressed B/triple grows 36.75 → 48.75, then plateaus

Per-permutation this is 6.125 → 8.125 B/triple/perm; the dict blob (~5.8–6.0 B/triple)
and directory (0.125) are flat, so growth lives in the block streams. Testable hypotheses:

- **H1 — absolute resets widen with the term space.** When a leading column changes
  (`d0 ≠ 0`), trailing columns are written as *absolute* ids. Distinct terms grow 110K →
  1.05M → 5.13M (17 → 20 → 23 bits); LEB128 quantizes at 7-bit steps (3 B covers ≤21
  bits, 4 B ≤28), so absolutes step 3 → 3 → 4 bytes *for uniformly drawn ids*. High-NDV
  leading permutations (OSP/OPS/SOP) reset on nearly every row.
- **H2 — within-run deltas widen** with id-space density (same 7-bit quantization).
- **H3 — the 10M → 50M plateau discriminates.** A 5× id-space growth (20 → 23 bits) that
  produces *zero* measured B/triple change suggests the byte-step boundaries were not
  crossed for the *typical* id — plausibly because insertion-ordered dictionaries give
  frequent terms small ids. H1/H2/H3 are separable only by per-field byte attribution
  (count / absolutes / d0 / d1 / d2, per permutation, per scale) — that is the spike
  (bead 4), which must *measure* before any encoding change is proposed.

Candidate fixes **iff** attribution warrants: per-block fixed-width bit-packing
(frame-of-reference per column), encoding resets as deltas from the block's first row,
or a block-size change. Any of these changes the shared in-RAM/on-disk byte stream —
i.e. a **versioned format change** (`SPQCPRM1` → `SPQCPRM2`, `open` compat for v1 files)
— so it is decision-gated on the spike's measured projection and is *not* on this epic's
ship path.

## 5. Hard constraints on the decomposition

- **`crates/sparq-core/src/store.rs` is untouchable here** — owned by in-flight sibling
  sq-7d3dj.32.1 (PR #1753), and it carries the #1730 cfg-split where the no-threads
  (`not(feature = "parallel")`) body must stay byte-identical for the feature-OFF wasm
  bundle (`wasm_bundle_bytes` `feature_off_exact` gate). Nothing in this plan needs it:
  `from_triples_compressed` consumes `build_raw_perms` output unchanged.
- **`crates/sparq-core/src/compress.rs` additions must be `#[cfg(test)]`-gated** (bead 4)
  — the file compiles into the wasm bundle; the byte-floor gate must not move.
- **No two child beads share a file** (disjointness contract below); the two files the
  unmerged instrument commit `5ce8b4f9` touches are assigned to exactly one bead each.
- Default path byte-identical; profile strictly opt-in; canonical numbers only from the
  quiet-box protocol; work-box numbers provisional and never baked into docs or tests.

## 6. Decomposition — child beads

All beads are context-independent specs; bead ids are recorded in the epic
(sq-7d3dj.32.2) on creation. File-areas are exclusive per bead.

| # | Surface (crate) | Tier | Files (exclusive) | One-line scope |
|---|---|---|---|---|
| 1 | `sparq-cli` | sonnet | `crates/sparq-cli/src/main.rs`, `crates/sparq-cli/tests/store_profile.rs` (new), `skills/cli/SKILL.md` | `SPARQ_STORE_PROFILE` env in the shared load helper + re-land the `memstat` subcommand from commit `5ce8b4f9` |
| 2 | bench scripts | sonnet | `scripts/bench/compressed-query-delta.sh` (new), `scripts/bench/bytes-per-triple.sh` (re-land), `bench/benchmarks.toml` | raw-vs-compressed WatDiv A/B harness emitting the house JSON envelope; re-land + register the D7 instrument |
| 3 | canonical bench results | sonnet | `bench/canonical-competitor-results/<date>/canonical-compressed-delta-*.json` (new), `research/compressed-memory-profile.md` (§9 results, post-merge), `research/perf-dominance-gap-2026-07.md` (D7 row) | quiet-EC2 canonical run at 1M/10M/50M; publish the B/triple + latency trade table |
| 4 | `sparq-core` (spike) | opus | `crates/sparq-core/src/compress.rs` (`#[cfg(test)]` only) | per-field byte attribution across id-density regimes; H1/H2/H3 verdict; follow-up bead iff a format change is warranted |
| 5 | ci-bench gate | sonnet | `scripts/ci-bench.sh`, `bench/perf-baseline.json` | deterministic `comp_store_bytes_per_triple` ratchet via `compare-compress` (on main today — no dependency on bead 1) |

**Dependencies:** bead 1 → bead 2 → bead 3 (the harness needs the env hook; the canonical
run needs the harness). Beads 4 and 5 are independent and parallel.

**Disjointness audit:** pairwise-empty file intersections across beads 1–5; none touch
`store.rs` (sibling .32.1) or the sibling surfaces of .32.3–.32.6. Bead 3 appends to
*this* record only after it merges (sequenced by the dep chain, no parallel writer).

**Invariants + acceptance tests** live in each bead's description (the four-field
contract: crate / model_tier / invariant / acceptance_test); summary:

1. **Bead 1 invariant:** env unset/`raw` → byte-identical behaviour; `compressed` →
   identical query solutions (result-equivalence); unknown value → hard error.
   *Acceptance:* `cargo test -p sparq-cli` test asserting identical solutions across
   profiles on a fixture, the fail-closed error, and lower reported heap under
   `compressed`.
2. **Bead 2 invariant:** measurement-only (no crate code); per-query counts cross-checked
   against `bench/watdiv/expected-rows.tsv` in BOTH modes — a mismatch fails the run.
   *Acceptance:* SF=1 smoke run emits a valid envelope (sha/date/machine/rows with
   `{query, rows, raw_us, comp_us}`) with all counts matching.
3. **Bead 3 invariant:** canonical numbers only from a dedicated quiet instance
   (orphan-proof self-terminate); honest reporting — if compressed is slower, publish the
   ratio and root cause, never spin. *Acceptance:* envelopes committed with
   `env.quiet_box=true` + count crosschecks green in both modes at all three scales;
   record + D7 row updated.
4. **Bead 4 invariant:** zero behaviour/format/bundle change — the `SPQCPRM1` byte stream
   is untouched and instrumentation is `#[cfg(test)]`.
   *Acceptance:* `cargo test -p sparq-core compress` green + a measured per-field
   attribution table at ≥3 id-density regimes posted to the bead; verdict on H1/H2/H3.
5. **Bead 5 invariant:** deterministic (byte-exact) metric only, `mode=auto`; the gate is
   green on its first commit (floor = the measured CI-scale value).
   *Acceptance:* local ci-bench run emits `comp_store_bytes_per_triple` and
   `scripts/perf-gate.py` passes against the new baseline.

## 7. The instrument re-landing (reproducibility debt)

`research/memory-per-triple-2026-07.md` (merged, #1752) cites
`scripts/bench/bytes-per-triple.sh` and `sparq-cli memstat` at sha `5ce8b4f9` — neither
is on `main` (the instrument lived in a bench worktree; the envelope was recovered from
EC2 console output). Beads 1 and 2 close this: the commit object is locally recoverable,
so the preferred path is cherry-pick + adapt (splitting `memstat` into bead 1's file and
the script into bead 2's), falling back to re-derivation from the record's §1 protocol.
Before starting, each bead checks no newer PR already landed the instrument (the original
bench agent may still hold it).

## 8. Rejected / deferred (with reasons)

- **Per-permutation adaptive compression** — rejected for v1 (§2); revisit with bead-3
  per-query data; the SPO-raw hybrid is the designed first step if membership/point cost
  dominates.
- **Decoded-block LRU cache** — deferred; defeats footprint predictability; measure first.
- **Compressed as the native default** — never without maintainer sign-off + bead-3 data
  (mirrors sibling .32.3's discipline).
- **`sparq-server` / HTTP profile surface** — deferred; the server load path is a hot
  shared surface (≤1 concurrent bead) and the CLI evidence should land first.
- **Cargo-feature gating of the profile** — rejected (§2): runtime workload choice, code
  already unconditionally compiled.
- **Encoding change now** — rejected: spike-first (§4); any change is a versioned format
  migration, not a tweak.

## 9. Results (reserved)

Filled by bead 3 (canonical quiet-EC2 envelope): B/triple + per-query latency trade at
1M/10M/50M, raw vs compressed profile, same binary, same corpus, counts cross-checked.
