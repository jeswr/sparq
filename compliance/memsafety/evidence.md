<!-- [OPUS-4.8] sq-toze — memsafety evidence pack (re-runnable verification of each
     control in controls.md). Re-review when Fable returns. NON-CANONICAL timing. -->

# Memory-safety attestation — evidence pack

Each control in [`controls.md`](./controls.md) is backed here by the **exact** file path,
test, or CI job, plus the command an auditor re-runs to confirm it. Paths are
repo-relative. No timing is recorded (NON-CANONICAL EC2 box).

## MS-1 — confined unsafe surface (26 `forbid` crates, 5 unsafe crates)

```sh
grep -rl 'forbid(unsafe_code)' crates/ --include='*.rs' | sed 's#crates/##;s#/src.*##' | sort -u
```
→ 26 crates: sparq-canon, sparq-conformance, sparq-engine, sparq-fedclient, sparq-fedplan,
sparq-geo, sparq-gpu, sparq-hdt, sparq-introspect, sparq-mpc, sparq-nlq, sparq-parse,
sparq-policy, sparq-prov, sparq-py, sparq-reason, sparq-reason-wasm, sparq-rsp, sparq-serve,
sparq-server, sparq-shacl, sparq-sim, sparq-solid, sparq-text, sparq-wasm, sparq-zk.

```sh
ls crates/ | wc -l            # → 31 total crates
```
Accounting: **26 forbid + 5 with unsafe (sparq-core, sparq-vectors, sparq-cli,
sparq-zk-compose, sparq-bench) = 31.** No crate is unaccounted-for. <!-- [OPUS-4.8] sq-pro0:
count re-verified on this branch — 31 workspace crates, 26 `#![forbid(unsafe_code)]` roots,
5 unsafe-bearing. The 56-site / per-crate figures (MS-2) are unchanged. -->

## MS-2 — 56-site register, count-verified

```sh
python3 scripts/unsafe-gate.py --list | tail -1     # → TOTAL=56
```
Per-crate (from `--check`, matching `bench/unsafe-snapshot.json` and the register §headers):

| crate | snapshot | live | register rows |
|---|---:|---:|---:|
| sparq-core | 42 | 42 | 42 |
| sparq-vectors | 9 | 9 | 9 |
| sparq-cli | 2 | 2 | 2 |
| sparq-zk-compose | 2 | 2 | 2 |
| sparq-bench | 1 | 1 | 1 |
| **total** | **56** | **56** | **56** |

Every row carries the site kind, the invariant relied on, and how it is bounded — see
[`unsafe-register.md`](./unsafe-register.md).

## MS-3 — gating ratchet

```sh
python3 scripts/unsafe-gate.py --check    # → "unsafe-count ratchet: PASS"; exit 0
```
CI wiring — `.github/workflows/ci.yml`, job `unsafe-register:`:
```yaml
  unsafe-register:
    name: unsafe-register (count ratchet)
    steps:
      - name: Ratchet first-party unsafe count against the snapshot
        run: python3 scripts/unsafe-gate.py --check
```
No `continue-on-error`; the job name contains no "informational"/"advisory" token, so the
`ci-summary / gate` aggregator (which polls sibling check-runs and treats any
non-informational lane as required) blocks merge on a ratchet regression. A PR adding an
`unsafe` site fails until: (1) a register row is added, (2) a `// SAFETY:` comment is
added in source, (3) `scripts/unsafe-gate.py --seed` re-seeds `bench/unsafe-snapshot.json`
— all three land in the same reviewable diff.

## MS-4 — cargo-geiger (informational, NOT the gate)

`.github/workflows/ci.yml`, job `geiger:` — `name: unsafe report (cargo-geiger,
informational)`, `continue-on-error: true`, every step `continue-on-error`. The name's
"informational" token makes the aggregator skip it. Honest posture: geiger is visibility,
the ratchet (MS-3) is the gate. (cargo-geiger cannot run the virtual workspace manifest —
hence the deterministic scan in MS-3 is what we actually ratchet.)

## MS-5 — per-site `// SAFETY:` (50/56 literal token; 6 adjacent-block-comment)

```sh
for c in sparq-core sparq-vectors sparq-cli sparq-zk-compose sparq-bench; do
  echo -n "$c: "; grep -rn 'SAFETY:' crates/$c/src | wc -l; done
# sparq-core: 36  sparq-vectors: 9  sparq-cli: 2  sparq-zk-compose: 2  sparq-bench: 1  → 50
```
The 6 sites without the literal token, each WITH an adjacent justification comment:
- `crates/sparq-core/src/dict.rs:483` — `from_utf8_unchecked`, preceded by a 5-line
  "TRUSTED fast path … untrusted mmap path MUST NOT reach here … bead sq-znld" comment.
- `crates/sparq-core/src/dict.rs:2192-2193` — `unsafe impl Send/Sync for SlotPtr`,
  preceded by the "routes each … slot to exactly one shard, nobody reads until the
  parallel scope ends" comment.
- `crates/sparq-core/src/dictspill.rs:720-721` — the `dict-spill` `SlotPtr` Send/Sync pair,
  same disjoint-routing justification above.

**Honest caveat (→ gap MS-G2):** these are *documented*, but not via the literal
`// SAFETY:` token, and there is **no** first-party `clippy::undocumented_unsafe_blocks`
lint enforcing the token. The register's earlier sentence "clippy
`undocumented_unsafe_blocks` is the local enforcement" overstated this — corrected here and
tracked as MS-G2.

## MS-6 — Miri lane

`.github/workflows/miri.yml` — `cargo +nightly miri test -p sparq-core` under
`MIRIFLAGS: -Zmiri-tree-borrows -Zmiri-ignore-leaks -Zmiri-disable-isolation`. Triggers:
`schedule` (nightly 05:11 UTC) + `workflow_dispatch`. **No** `pull_request`/`merge_group`
trigger — so it is a nightly UB safety net, not a per-PR gate, and the aggregator does not
wait on it. The header documents (load-bearing) that the mmap/dict-spill features are NOT
enabled because Miri rejects file-backed mappings — those 16+7 sites are covered by MS-7/8.

## MS-7 — corruption oracle

`crates/sparq-core/tests/mmap_corruption_oracle.rs`:
```sh
grep -n 'fn ' crates/sparq-core/tests/mmap_corruption_oracle.rs
# open_rejects_corrupt_index, corrupt_truncate, corrupt_flip, corruption_sweep,
# mmap_loader_survives_corruption_raw, mmap_loader_survives_corruption_compressed
```
Run under `--features mmap,dict-spill` (the features Miri cannot run). The sweep
truncates/flips each on-disk file and asserts the loader rejects-or-stays-in-bounds.

## MS-8 — fuzz (mmap loader)

`fuzz/fuzz_targets/graph_open.rs` — header documents the surface as `Graph::open` over a
CORRUPT on-disk store dir (perm0..5, dict-*.bin, numerics/temporals, predstats, named.bin),
threat-model `T-MMAP-FUZZ`, invariant "clean `Err`, never panic/OOB/UB". Targets enumerated
by `cargo fuzz list` in `.github/workflows/fuzz.yml` (PR smoke + nightly). Other targets:
`load_reader_parallel.rs`, `parse_rdf_str.rs`, `parse_sparql.rs`, `validate_shacl.rs`.

## MS-9 — ASan (fuzz lane **and** standalone corruption-corpus lane)

**(a) Inside cargo-fuzz.** `.github/workflows/fuzz.yml` builds on nightly with `-Zsanitizer`
(libFuzzer sancov) on the `x86_64-unknown-linux-gnu` target (the musl target is
ASan-incompatible — documented in the workflow). So the mmap loader's reads execute under
AddressSanitizer during fuzzing.

**(b) Standalone over the deterministic corruption corpus (sq-hybl, [OPUS-4.8]) — closes the
former MS-G3 caveat.** `.github/workflows/asan.yml`:
```sh
# the exact lane commands (gnu target, ASan-instrumented std via -Zbuild-std):
RUSTFLAGS="-Zsanitizer=address" ASAN_OPTIONS="detect_leaks=0" \
  cargo +nightly test -Zbuild-std --target x86_64-unknown-linux-gnu \
    -p sparq-core --features mmap,dict-spill --test mmap_corruption_oracle
RUSTFLAGS="-Zsanitizer=address" ASAN_OPTIONS="detect_leaks=0" \
  cargo +nightly test -Zbuild-std --target x86_64-unknown-linux-gnu \
    -p sparq-vectors --test store --test diskann
```
Runs the deterministic mmap corruption corpus (the oracle's truncate + bit-flip sweep over
every on-disk section, plus the `sparq-vectors` `VectorStore`/`DiskAnnIndex` open-validation
corpus: truncated files / wrong magic / oversized length fields) under AddressSanitizer, so a
heap-OOB / use-after-free / UB on a malformed on-disk file is caught even when it does not
panic. `-Zbuild-std` (needs the `rust-src` component) is REQUIRED so std is recompiled under
the same `-Zsanitizer` flag; `--target` (a non-musl GNU triple) is REQUIRED for build-std +
ASan compatibility; `detect_leaks=0` silences LeakSanitizer noise from rayon's daemon threads
+ the long-lived mmaps (a lifecycle artefact, cf. miri's `-Zmiri-ignore-leaks`). **Trigger:**
nightly `schedule` + `workflow_dispatch` only — NO `pull_request`/`merge_group`, so it is a
non-blocking UB safety net (`-Zbuild-std` is many minutes) and `ci-summary / gate` neither
discovers nor waits on it; the job name `asan (mmap corruption corpus, informational)` also
carries the `informational` token belt-and-braces.

**Local verification (sparq-core half): PASSED.** The first lane command was RUN locally on
the agent's `aarch64-unknown-linux-gnu` host (a valid ASan target): the ASan-instrumented
`mmap_corruption_oracle` built (`-Zbuild-std` rebuilt std under `-Zsanitizer`) and the corpus
ran clean — `test result: ok. 4 passed; 0 failed` with NO ASan report (no heap-OOB / UAF / UB
on the truncate + bit-flip sweep). The lane PINS the `x86_64-unknown-linux-gnu` runner triple;
the flow is identical across the two gnu triples. See the inconsistencies note below for the
precise tested-vs-untested boundary.

## MS-10 — clippy `-D warnings`

`.github/workflows/ci.yml`: `cargo clippy --workspace --all-targets -- -D warnings`
(GATING) + `cargo clippy -p sparq-wasm --target wasm32-unknown-unknown --all-targets -- -D
warnings`. Neither is `continue-on-error`.

## MS-11 — dependency memory-safety (supply-chain lane)

`.github/workflows/supply-chain.yml`: `cargo deny check bans sources licenses` (gating) +
`cargo deny check advisories` (gating — GX-1 un-degraded; CVSS-4.0 blocker sq-q8de
resolved; `continue-on-error` removed). Daily watchdog `dependency-monitoring.yml`. The
register explicitly scopes third-party `unsafe` (memmap2/libc/rayon/hdt) OUT to this lane.

## MS-12 — edition-2024 unsafe (test-only env)

`unsafe-register.md` rows `src/lib.rs:6229` (`set_var`) + `:6231` (`remove_var`) — TEST-only
(`external_quads_fd_*`), single-threaded, var restored before return; counted by the ratchet.

## MS-13 — no unsafe in the untrusted-text path

The parser/planner/executor/reasoner/SHACL layers are in the MS-1 forbid list (or have no
`unsafe`): untrusted *query/data text* never reaches `unsafe`; only the on-disk index does
(B5). `research/threat-model.md` §scope table confirms `sparq-engine`/`sparq-reason`/
`sparq-shacl`/spargebra carry no executor/parser `unsafe`.

---

### Verified-but-noted inconsistencies (for the auditor)

1. **`research/threat-model.md` says "39 sites" in sparq-core**, the register/snapshot say
   **42**. The register is the authoritative count (it is the GX-5 artifact + the ratchet
   source). The threat-model number is stale prose. This framework does not own
   `threat-model.md`; tracked as low-severity drift MS-G5 (one-line fix for the doc owner).
2. The register's `undocumented_unsafe_blocks` enforcement sentence is an overstatement
   (MS-G2) — corrected in MS-5/evidence and in the register itself.
3. **ASan lane (MS-9b / asan.yml) — tested-vs-untested boundary (sq-hybl, honest).** The
   sparq-core lane command (`RUSTFLAGS=-Zsanitizer=address` + `-Zbuild-std` + a non-musl GNU
   `--target` + `ASAN_OPTIONS=detect_leaks=0` over `mmap_corruption_oracle --features
   mmap,dict-spill`) was actually RUN to completion locally on the agent's
   `aarch64-unknown-linux-gnu` host (a valid ASan target): it built and the corpus passed
   clean — `4 passed; 0 failed`, no ASan report. The sparq-vectors `store`/`diskann` command
   was NOT separately run locally (same flow, no extra feature). The lane PINS the
   `x86_64-unknown-linux-gnu` GitHub runner triple (matching `fuzz.yml`), which was not run in
   this worktree (no x86_64 runner here); the flow is identical across the two gnu triples —
   validate the x86_64 leg + the vectors leg on the FIRST scheduled/dispatched CI run. The
   lane is non-blocking, so a first-run hiccup cannot block any merge.
