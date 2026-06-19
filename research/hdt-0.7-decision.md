<!-- [OPUS-4.8] sq-4wo.2 — HDT 0.4-vs-0.7 decision analysis for the maintainer (#758 / #703 / sq-wzm4).
     Design-for-review: data + options + a recommendation matrix. The accept/reject/roll-our-own
     decision is the MAINTAINER'S. Authored on Opus 4.8 (Fable unavailable). -->

# HDT crate 0.4 vs 0.7 — upgrade decision analysis

Status: **design-for-review (data for the maintainer's decision)**. Re-opens the
Dependabot #703 / `sq-wzm4` question — closed on a lean-core concern — with measured
data, per #758.

The maintainer's four questions (#758), each answered below with sourced evidence:

1. Is there a **performance difference** between 0.4 and 0.7?
2. Would we gain from **rolling our own** decoder (and could we **contribute upstream**)?
3. Do 0.7's **new deps** (`clap`/`env_logger` + others) provide **features useful to sparq**?
4. A clear **recommendation matrix** for accept-0.7 / stay-0.4 / roll-our-own / contribute-upstream.

> Honesty notes for this doc:
>
> - Any throughput figure here was measured on the **EC2 work box** (`aarch64`, stable
>   `rustc 1.96.0`) and is **NON-CANONICAL** — directional only, never to be baked into a
>   canonical doc/test. It is kept in a clearly-marked analysis table per the no-perf gate.
> - Upstream changelog facts are sourced to the `hdt` repo / crates.io / docs.rs (see
>   §"Sources"). Where a claim is the upstream maintainer's own benchmark, it is labelled
>   as such, not as an independently reproduced result.
> - **Premise corrections to the brief / prior record appear in §0** — verify before trusting
>   the older "0.6 needs nightly on aarch64" / "clap is bin-only" framings.

---

## 0. Premise corrections (verify these first)

Two pieces of the standing record turned out to be **stale or imprecise** when checked
against the actual crates:

- **CORRECTION A — "hdt ≥0.6 won't build on stable aarch64 (qwt `prefetch` needs nightly)"
  is no longer true.** `crates/sparq-hdt/Cargo.toml` and bead `sq-2l1` pin to 0.4 because
  `hdt 0.6 → qwt 0.3.4`'s default `prefetch` feature required nightly on `aarch64`.
  **Measured here:** `hdt 0.7.1 → qwt 0.3.5` (with `prefetch` still in qwt's default set)
  **compiles cleanly in both dev and release on stable `rustc 1.96.0`, `aarch64`.** The
  upstream commit log corroborates this — 0.7.x includes *"chore: Bump QWT to fix build with
  stable toolchain. Fixes #121."* So the *compile* blocker recorded in `sq-2l1` is **resolved**;
  the remaining objection is purely **dependency weight**, not buildability.

- **CORRECTION B — the clap entry point is `qwt`, not hdt's own `cli` feature.** hdt 0.7.1's
  *own* `Cargo.toml` makes `clap` optional (behind its `cli` feature, for the `hdt` binary) —
  so reading hdt's manifest alone suggests `default-features = false` avoids clap. **It does
  not.** clap arrives **transitively and unconditionally through `qwt 0.3.5`**, which hdt
  requires for rank/select on the read path. In `qwt 0.3.5`, `clap` is
  `kind=normal, optional=false` (likewise `bincode`, `serde`, `rand`, `mem_dbg`). So **any
  hdt ≥0.6 consumer pulls clap regardless of features.** This refines #758's wording
  ("hdt 0.7 pulls clap + env_logger onto the read path"): correct in *effect*, but the
  mechanism for clap is **qwt**, which has implications for the upstream-fix path (§3, §4).

---

## 1. Performance: 0.4 vs 0.7

### 1a. What actually changed on the decode path

The only runtime-relevant change between 0.4 and 0.7 is the **`sucds` → `qwt` rank/select
refactor**, which landed in **0.6.0** (upstream PR #102, *"Refactor from sucds to QWT and
support 32 bit targets"*). 0.7.0/0.7.1 are a Sophia `0.9→0.10` adaptation, a qwt API change
(`Bitmap::new` now takes a slice), a mutable-header API addition, the stable-toolchain qwt
bump, and a packaging fix — **no decode-path perf work is attributed to 0.7.x.**

The upstream maintainer's **own** benchmark for the sucds→qwt switch (PR #102, gungraun)
reports **load: instructions −14.07%, estimated cycles −8.97%**, with query operations
changing `<1%`. Treat as the maintainer's reported figure on their harness, not independently
reproduced.

### 1b. Does that delta reach *sparq's* read path?

This is the load-bearing distinction. sparq does **not** load HDT through the full
`hdt::Hdt::read`. Since the H1–H6 work (`crates/sparq-hdt/src/decode.rs`), the **default**
path is sparq's own **direct decoder** (`graph_from_reader`), which:

- **reuses** upstream's byte-level **section readers** — `FourSectDict::read`,
  `DictSectPFC`, `Sequence::read`, `Bitmap::read`, `ControlInfo::read`, `Header::read`; but
- **skips** `TriplesBitmap::new` entirely — the wavelet matrix, the per-object
  `Vec<Vec<u32>>`, the `sort_by_cached_key`, and the OP-index (`Rank9Sel` / qwt).

The `sucds→qwt` change is **inside the index-build code sparq's direct path deliberately
skips.** So the upstream ~9–14%-on-load improvement applies to the path sparq uses **only as
the differential-oracle** (`graph_from_hdt`, used in tests/bench A/B), **not** to the
production `graph_from_reader`. The bytes sparq's direct decoder actually touches
(PFC sections, Log64 sequences, CRCs) are essentially unchanged across 0.4→0.7.

### 1c. Measured (NON-CANONICAL, work-box `aarch64`, stable `rustc 1.96.0`)

A/B microbench of the **upstream** `Hdt::read` (the path the qwt change affects) on an
identical 300k-triple synthetic `.hdt` (1.47 MB), median of 9 after 2 warm-ups, repeated:

| Version | path measured | median `Hdt::read` | spread | note |
|---|---|---|---|---|
| hdt 0.4.0 | upstream full (sucds) | ~0.0555 s | min 0.0553 / max 0.0557 | NON-CANONICAL |
| hdt 0.7.1 | upstream full (qwt) | ~0.0490 s | min 0.0467 / max 0.0503 | NON-CANONICAL |

Directional read: **0.7.1's `Hdt::read` is ~10–12% faster here**, consistent with the
maintainer's reported sucds→qwt gain. **But this is the path sparq's production loader
bypasses.** On sparq's actual default `graph_from_reader`, the expected delta is
**near-zero** (same section readers, no index build either way). This was not separately
microbenched because the decoder code is identical at the byte level; that is the honest
expectation, not a measured claim.

**Bottom line (Q1):** there *is* a real ~9–14%-on-load upstream improvement (0.6's
sucds→qwt), reproduced directionally here as ~10–12% — but it lands on the **index-build
path sparq does not run**. For sparq's production HDT read path the perf difference between
0.4 and 0.7 is expected to be **negligible**. 0.7 brings **no new decode-path speedup** of
its own. There is no perf *case* for the upgrade.

---

## 2. Roll-our-own — what sparq would gain, and the upstream angle

### 2a. sparq has already rolled ~most of it

`crates/sparq-hdt` is **not** a thin wrapper anymore. It already owns:

- the **triples-section scan** (`decode.rs`, levers H1–H2) — reads `bitmap_y/z` +
  `sequence_y/z` directly and walks SPO, skipping `TriplesBitmap::new`;
- the **PFC dictionary decode** (H3–H4, H6) — block-sequential decode of the four PFC
  sections, interning from borrowed `&str`, the four sections decoded concurrently on rayon;
- the **in-memory encoder** (`encode.rs`, sq-ashy) — builds FourSectDict PFC + BitmapTriples
  directly from sparq's dict+triples, no N-Triples round-trip.

What it still **borrows** from the `hdt` crate is the **low-level byte readers/writers** of
the container sections (`FourSectDict::read`, `DictSectPFC::compress`, `Sequence::read`,
`Bitmap::read`, `TriplesBitmap::from_triples`, the `*::write` methods, `ControlInfo`,
`Header`, the `containers::rdf` term types). These are stable, well-tested, on-disk-quirk-
exact (the 3-vbyte PFC preamble, the deliberate vbyte off-by-one, CRC8/CRC32 layout). The
`sophia` feature is only pulled on the **opt-in `write`** path (`hdt/sophia`); the read path
is already sophia-free.

### 2b. What a full roll-our-own (drop the `hdt` dep entirely) would buy

| Gain | Real? | Notes |
|---|---|---|
| Remove `clap` + `env_logger` + `qwt` + `rayon`(upstream's) from the dep graph | **Yes** | The whole point of #758's lean-core concern. ~67 packages removed (see §3). |
| Perf control on the read path | **Marginal** | sparq already skips the costly index build; the remaining cost is PFC + Log64 + CRC, which sparq would re-implement at parity, not faster. No measured headroom over what sparq already drives. |
| No nightly/MSRV coupling to qwt | **Yes (but moot)** | qwt 0.3.5 already builds on stable (Correction A). |
| Tailored entirely to sparq-core types | Partial | encode/decode already intern straight into sparq's `Dict`; the borrow is only the section codecs. |

The **cost** of a full roll-our-own is re-implementing and *fuzz-/differential-testing* the
PFC + Bitmap + Sequence + CRC byte codecs against hdt-cpp/hdt-java output — exactly the
"on-disk quirks" the plan (`research/parsing-optimization-plan.md`, H1 risk note) flags as
**medium-high risk**. sparq currently gets those for free, byte-exact, from a maintained crate
that is **differentially tested in sparq's own suite** (`tests/decode_paths.rs`,
`tests/roundtrip.rs`, `tests/write_roundtrip.rs`) against `Hdt::read`/`Hdt::write`.

**Net:** a full roll-our-own removes the dep bloat but buys **no measured perf** and assumes
**real correctness risk** for the section codecs. It is justified *only* if the lean-core /
supply-chain objection to qwl+clap+env_logger is decisive on its own terms (it may well be —
see §3/§4).

### 2c. Contribute upstream instead (already drafted)

`crates/sparq-hdt/UPSTREAM.md` already contains **two ready-to-file** contributions:

- **`Hdt::from_triples` / sophia-free section builders** (sq-ashy) — the builders
  (`DictSectPFC::compress`, `TriplesBitmap::from_triples`, `*::write`) are already `pub`;
  the only friction is that the documented entry point (`read_nt`) is `sophia`-gated. A
  lighter, sophia-free builder feature would let sparq's `write` path drop `hdt/sophia`.
- **`Hdt::triples_streaming(reader)`** (sq-fkj) — a decode-only entry point that yields SPO
  ids without building the wavelet/OP-index. **sparq's `decode.rs` is a reference
  implementation already.** If upstream adopts it, sparq could **delete its vendored decoder
  and call upstream** — collapsing the roll-our-own surface back to the maintained crate.

Neither of these, however, removes the **clap/env_logger/qwt weight** — those ride in on
qwt (clap) and hdt's own non-optional deps (env_logger, rayon). A *separate* upstream ask
would be needed to make `env_logger` optional in hdt, and a **`qwt` upstream change** to
gate clap behind a feature (qwt's clap is non-optional today). The clap removal is therefore
**not within hdt's gift alone** — it needs a qwt change or hdt dropping qwt.

---

## 3. The new dependencies — features vs bloat

### 3a. Measured dependency-graph delta (`cargo tree -e normal`, `default-features = false`)

| | hdt 0.4.0 | hdt 0.7.1 |
|---|---|---|
| normal-dep packages (incl. transitive) | **21** | **88** |
| rank/select lib | `sucds 0.8.3` (+ anyhow, num-traits) | `qwt 0.3.5` (+ clap, bincode, serde, rand, mem_dbg, minimum_redundancy, …) |
| logging | `log` facade only | `log` **+ `env_logger`** (anstream/anstyle/env_filter) |
| parallelism | none | `rayon` (+ crossbeam) |
| CLI arg parsing | none | `clap 4.6.1` (clap_builder/clap_derive/strsim) — **via qwt** |

That is a **~4× growth** in the normal-dependency graph, almost entirely from `qwt`'s own
transitive tree plus `env_logger`/`rayon`.

### 3b. What each new dep actually provides — and to whom

- **`qwt`** — the new rank/select backbone for the triples bitmap. Useful to **upstream's
  query path**; **not exercised by sparq's direct decoder** (sparq skips the qwt-backed index
  build). So for sparq it is **paid for but unused at runtime** on the default read path. It
  *is* what makes the read path compile, so it cannot simply be turned off.
- **`clap`** — argument parsing for qwt's own bin/bench scaffolding. **Pure bloat for sparq**
  (no sparq code parses HDT CLI args via clap). Unavoidable because qwt's clap dep is
  non-optional (Correction B).
- **`env_logger`** — a logging *implementation* (vs the `log` facade). A **non-optional hdt
  library dependency** in 0.7.1. sparq does not want a library dep to choose a logging
  backend — that is the application's call. **Bloat / policy smell for a library consumer.**
- **`rayon`** (upstream's) — used in hdt's **`nt.rs`** N-Triples→HDT build path, **not** the
  decode/read path. sparq already brings its own rayon for the PFC decode; upstream's is
  redundant for sparq's use.
- **No new *format* features.** Confirmed against upstream `src/`: **no QUADS / named-graph /
  HDTQ / HDT-v2 / compressed-HDT-variant support** is added in 0.6/0.7 — the crate stays
  **triples-only**. There is **no bitmap-format improvement** sparq would consume (the qwt
  change is an in-memory rank/select swap, not an on-disk format change; sparq's on-disk
  reader is unaffected). The Sophia bump (0.9→0.10) only matters behind the opt-in `sophia`
  feature.

**Bottom line (Q3):** 0.7's added deps deliver **nothing sparq needs**. clap/env_logger are
CLI/logging weight; rayon-upstream and qwt serve paths sparq bypasses; and **no new HDT
format capability** (quads, compressed variants, bitmap format) arrives that sparq would use.
This *confirms* the original #703 close rationale, and strengthens it: the weight is larger
(88 vs 21 packages) and **clap is unavoidable**, not feature-gateable on the hdt side.

---

## 4. Recommendation matrix

| Option | Pros | Cons | Verdict |
|---|---|---|---|
| **Accept 0.7.1 as-is** | Builds on stable aarch64 now (Correction A); internal API sparq uses is compatible (verified — `FourSectDict::read`, `DictSectPFC::compress(&BTreeSet<&str>,_)`, `Sequence::read`, `Bitmap::read`, `ControlInfo`, `Header`, `IdKind`, `containers::rdf` all resolve on 0.7.1); upstream load path ~9–14% faster (irrelevant to sparq's direct path). | Pulls **88 vs 21** packages incl. **clap (unavoidable, via qwt) + env_logger (non-optional) + rayon**; **no perf gain on sparq's actual path**; **no useful new feature**; widens the supply-chain/audit surface for `sparq-hdt` (which is opt-in but still shipped). | **Not recommended on current evidence** — pure cost, no benefit. |
| **Stay on 0.4.0 (current pin)** | Lean (21 pkgs); zero clap/env_logger; sparq's direct decoder + encoder already deliver the perf and the in-memory write; fully tested. | 0.4 is older; if upstream drops 0.4 maintenance, security fixes would require a bump or a vendored copy. | **Recommended for now** — status quo is honestly the best cost/benefit. Track upstream. |
| **Roll-our-own (drop `hdt` dep entirely)** | Removes **all** of qwt/clap/env_logger/rayon; full control. | Re-implement + fuzz/diff-test the PFC + Bitmap + Sequence + CRC **byte codecs** (medium-high risk per the plan); **no measured perf upside** over today; ongoing maintenance of on-disk-format edge cases sparq currently gets free. | **Only if** the lean-core/supply-chain objection to qwt+clap+env_logger is judged decisive **and** 0.4 maintenance lapses. Defer until then. |
| **Contribute upstream** | sparq's `decode.rs`/`encode.rs` are reference impls for the two **already-drafted** UPSTREAM.md asks (sq-ashy sophia-free builders; sq-fkj decode-only entry point); lets sparq *delete* its vendored decoder if adopted; good-citizen. | Does **not** remove clap/env_logger/qwt weight (clap needs a *qwt* change; env_logger needs a separate hdt change); upstream-acceptance + release latency outside sparq's control. | **Recommended in parallel** with "stay on 0.4" — file the two drafted PRs; additionally open issues asking hdt to make `env_logger` optional and qwt to feature-gate `clap`. Low-risk, upside-only. |

### Recommended posture (for the maintainer to confirm)

1. **Keep the `hdt = "0.4"` pin** (reject Dependabot #703 / `sq-wzm4` again) — but update the
   *reason* in `Cargo.toml`/`sq-2l1`: the stale "0.6 needs nightly on aarch64" rationale is
   **resolved**; the live reason is **dependency weight + zero benefit to sparq's direct
   path**, and specifically **qwt's non-optional clap**.
2. **File the two UPSTREAM.md contributions** (sq-ashy, sq-fkj) — they stand on their own merit
   and could later collapse sparq's vendored decoder back onto a maintained upstream entry point.
3. **Open two upstream issues** asking for (a) `env_logger` to be optional in hdt, and
   (b) `qwt` to feature-gate `clap`. If both land, **re-evaluate accepting a future hdt** —
   that is the only path on which the upgrade becomes lean enough to be worthwhile.
4. **Defer roll-our-own** unless 0.4 maintenance lapses or the supply-chain objection becomes
   decisive; sparq already owns the perf-critical decode/encode logic, so the residual
   roll-our-own surface (section byte codecs) is *risk without measured reward* today.

---

## Phased plan (future beads for the orchestrator)

1. **`sq-4wo.2a` — Update the pin rationale.** Correct `crates/sparq-hdt/Cargo.toml` + bead
   `sq-2l1`: strike the "0.6/qwt needs nightly aarch64" reason (resolved on 0.7.1/qwt 0.3.5,
   stable rustc 1.96), replace with "pinned on dependency-weight + no benefit to the direct
   decoder; qwt's clap is non-optional." Doc-only. *(Depends on maintainer confirming
   "stay on 0.4".)*
2. **`sq-4wo.2b` — File upstream PR: sophia-free section builders** (UPSTREAM.md sq-ashy).
   Reference impl = `encode.rs`. Lets `sparq-hdt`'s `write` feature drop `hdt/sophia`.
3. **`sq-4wo.2c` — File upstream PR/issue: decode-only `triples_streaming`** (UPSTREAM.md
   sq-fkj). Reference impl = `decode.rs`. If adopted, a follow-up bead deletes sparq's
   vendored decoder in favour of the upstream entry point.
4. **`sq-4wo.2d` — Open upstream issues: make `env_logger` optional (hdt) + feature-gate
   `clap` (qwt).** These are the *only* changes that make a future hdt lean enough to accept.
5. **`sq-4wo.2e` — Re-evaluate the bump when (2d) lands** (or when crates.io marks 0.4
   yanked/unmaintained). Gate: a fresh `cargo tree -e normal` on the then-current hdt shows
   clap+env_logger gone; re-run the canonical-runner A/B against the direct decoder.
6. **`sq-4wo.2f` (contingent) — Roll-our-own section codecs**, *only if* (5) cannot reach a
   lean graph and 0.4 maintenance lapses. Scope = PFC + Bitmap + Sequence + CRC byte
   readers/writers with the existing differential oracle as the correctness gate.

---

## Open questions for the maintainer

- **Decision:** confirm "stay on 0.4 + contribute upstream", or do you want a different
  posture (e.g. accept 0.7 anyway for freshness, or commit to roll-our-own now)?
- **Supply-chain weight tolerance:** is the **88-vs-21-package, clap-unavoidable** footprint
  the decisive factor, independent of perf? If yes, that shifts the long-run answer toward
  roll-our-own once 0.4 is unmaintained.
- **Upstream appetite:** are you happy to be the one to open the qwt-clap-feature-gate issue,
  or should sparq carry a vendored qwt-free reader regardless of upstream's response?

---

## Sources

- sparq code: `crates/sparq-hdt/{Cargo.toml,UPSTREAM.md,src/decode.rs,src/encode.rs,src/write.rs,src/lib.rs}`;
  `research/parsing-optimization-plan.md` (HDT levers H1–H6); beads `sq-2l1`, `sq-th5i`,
  `sq-4wo.1`, `sq-ashy`, `sq-fkj`.
- Dependency facts: `cargo tree -e normal` on probe crates depending on `hdt = "=0.4.0"` vs
  `hdt = "0.7"` (both `default-features = false`), resolved against the live crates.io index;
  `qwt 0.3.5` dependency kinds from the crates.io API (`clap`/`bincode`/`serde`/`rand`/`mem_dbg`
  = `kind=normal, optional=false`).
- Build feasibility: `cargo build` and `cargo build --release` of `hdt 0.7.1`
  (`default-features = false`) on `aarch64`, stable `rustc 1.96.0` — both succeed.
- API compatibility: a probe importing/calling the exact internal API surface
  `sparq-hdt` uses (`FourSectDict::read`, `DictSectPFC::compress(&BTreeSet<&str>,_)`,
  `Sequence::read`, `ControlInfo`, `ControlType::Triples`, `Header::read`, `IdKind`,
  `containers::rdf::*`) compiles on 0.7.1.
- Upstream changelog/perf: `hdt` GitHub releases/commits (0.6.0 PR #102 sucds→qwt; the
  0.6.0→0.7.1 commit compare; the qwt stable-toolchain fix) and crates.io/docs.rs manifests —
  gathered via web research. The sucds→qwt **load: instructions −14.07%, cycles −8.97%** figure
  is the **upstream maintainer's own gungraun benchmark** (PR #102), not independently
  reproduced; 0.7.x has **no** attributed decode-path perf change.
- Microbench (**NON-CANONICAL, work-box `aarch64`, stable `rustc 1.96.0`**): median-of-9
  `Hdt::read` on a 300k-triple / 1.47 MB synthetic `.hdt`, 0.4.0 ~55 ms vs 0.7.1 ~49 ms.
  Directional only; measures the upstream full path sparq's direct decoder bypasses.
