<!-- [OPUS-4.8] sq-toze — memsafety framework control table. Authored while Fable
     unavailable — re-review when Fable returns. Engineer↔auditor loop (epic sq-toze). -->

# Memory-safety attestation — control table

**Framework:** Memory-safety attestation (sparq's headline safety claim — a Rust
RDF/SPARQL data-engine consumed as a dependency in high-security settings).
**Scope:** the first-party `crates/` tree only. Threat-model boundary **B5**
(`research/threat-model.md`): *hostile on-disk index → mmap loader → `unsafe` pointer
reinterpret* is the load-bearing surface.
**Companion docs:** [`evidence.md`](./evidence.md) (per-claim file/CI verification),
[`gap-register.md`](./gap-register.md) (open gaps + beads),
[`unsafe-register.md`](./unsafe-register.md) (the 56-site per-site justification register),
[`audit-log.md`](./audit-log.md) (the engineer↔auditor rounds + final verdict).

## Status legend

- **PASS** — a technical control in the codebase/CI with verified, re-runnable evidence
  (file path + line / test name / CI job). The auditor re-ran or re-read the cited
  artifact.
- **AUDIT-READY** — control + documentation in place, but a *certificate* needs an
  accredited external body / formal verification we cannot substitute for.
- **OPEN-gap** — not met (or only partially); recorded in `gap-register.md` with a `bd`
  bead. **Not** papered over.

<!-- [OPUS-4.8] reconciled with post-remediation re-audit (sq-gbp4); see ZK-verdict cross-ref sweep -->
> Honesty note. This framework does **not** touch the ZK/MPC estate's soundness — that is
> `cryptoreview`. The `sparq-zk-compose` advisory-lock `unsafe` (2 sites) is in scope here
> only as *memory*-safety (a `libc::flock` FFI), **not** as a cryptographic guarantee. The
> ZK verifier's published posture (`SECURITY.md`: remediated but NOT externally audited — no
> production guarantee until the external sign-off `sq-qhy4` completes) is untouched by any
> claim below.

## Controls

| # | Control | Status | Evidence (file / test / CI job) |
|---|---|---|---|
| MS-1 | **Unsafe surface is *confined*** — only 5 first-party crates contain `unsafe`; the other 20 are `#![forbid(unsafe_code)]`, so a new `unsafe` anywhere else fails to compile. | **PASS** | `#![forbid(unsafe_code)]` in 20 crates (verified: `grep -rl forbid(unsafe_code) crates/` → 20 crates; 25 crates total). Enumerated in `evidence.md` §MS-1. |
| MS-2 | **Every first-party `unsafe` site is enumerated + justified** — the 56-site register records each site's kind, the invariant it relies on, and how it is bounded/tested. | **PASS** | `compliance/memsafety/unsafe-register.md` (56 rows). Count cross-checked: `scripts/unsafe-gate.py --list` → `TOTAL=56`, matching `bench/unsafe-snapshot.json` (`total: 56`) and the register's per-crate split (sparq-core 42, sparq-vectors 9, sparq-cli 2, sparq-zk-compose 2, sparq-bench 1). |
| MS-3 | **Unsafe-count RATCHET gates every PR** — a PR cannot add an `unsafe` site without updating the register + re-seeding the snapshot; the count gate is merge-blocking. | **PASS** | `scripts/unsafe-gate.py --check` (re-run: PASS, live 56 == snapshot 56). CI job `unsafe-register (count ratchet)` in `.github/workflows/ci.yml` (`unsafe-register:` job, no `continue-on-error`, name has no "informational"/"advisory" token ⇒ the `ci-summary / gate` aggregator treats it as REQUIRED). |
| MS-4 | **cargo-geiger visibility** — third-party + first-party `unsafe` is surfaced in the CI summary. | **PASS** (informational by design) | `geiger` job `unsafe report (cargo-geiger, informational)` in `ci.yml` — `continue-on-error: true`, name contains "informational" ⇒ NON-gating. The *gating* control is MS-3 (the deterministic ratchet); geiger is visibility only. Honestly labelled: cargo-geiger is **not** the gate. |
| MS-5 | **Per-site `// SAFETY:` justification in source** — the safety argument lives next to the code. | **PASS-with-caveat** | 50 `// SAFETY:` comments across the 5 unsafe crates; the remaining 6 sites (the `from_utf8_unchecked` TRUSTED fast path `dict.rs:483`, and the two `unsafe impl Send`/`Sync for SlotPtr` pairs in `dict.rs:2192-93` + `dictspill.rs:720-21`) carry a justification in an adjacent block comment rather than the literal `// SAFETY:` token. **Caveat:** there is NO first-party clippy `undocumented_unsafe_blocks` lint enforcing the literal token (see OPEN-gap MS-G2) — enforcement of the *token* is by the register + review, not by a lint. |
| MS-6 | **Miri UB lane** over the pure-Rust unsafe reachable without mmap — aliasing / provenance / data-race detection on the parallel scatter writes, POD↔bytes reinterprets, `from_utf8_unchecked` over in-memory buffers, `MaybeUninit`+`set_len`. | **PASS** (nightly safety-net) | `.github/workflows/miri.yml` — `cargo miri test -p sparq-core` under `-Zmiri-tree-borrows`, nightly + `workflow_dispatch`. **Honestly scoped:** Miri *structurally cannot* run the 16 mmap-backed sites (file-backed mappings) — that is MS-7, not Miri. Off the per-PR critical path by design (nightly), so it is a standing safety net, not a merge gate. |
| MS-7 | **Deterministic corruption oracle** for the B5 mmap sites Miri cannot reach — a hostile/corrupt on-disk index must reject or stay in-bounds, never UB. | **PASS** | `crates/sparq-core/tests/mmap_corruption_oracle.rs` (`corruption_sweep`, `mmap_loader_survives_corruption_raw/_compressed`, `open_rejects_corrupt_index`), run under `--features mmap,dict-spill`. Verified the file exists + names the B5 surface. |
| MS-8 | **Coverage-guided fuzzing of the mmap loader** — libFuzzer drives hostile bytes into every on-disk byte-parser (counts/lengths/magic/offset tables/manifest). | **PASS** | `fuzz/fuzz_targets/graph_open.rs` (`sparq_core::Graph::open` over a corrupt store dir; threat-model `T-MMAP-FUZZ`). Plus `load_reader_parallel.rs`, `parse_rdf_str.rs`, `parse_sparql.rs`, `validate_shacl.rs`. CI: `.github/workflows/fuzz.yml` (PR smoke + nightly; `cargo fuzz list` auto-enumerates targets). |
| MS-9 | **AddressSanitizer over the hostile-input path** — the fuzz lane builds with `-Zsanitizer=address` (cargo-fuzz default) on the gnu (dynamic-libc) target, so the mmap loader's reads run under ASan during fuzzing. | **PASS-with-caveat** | `fuzz.yml` uses nightly + `-Zsanitizer` (libFuzzer sancov); the musl target is rejected by ASan, the gnu target is sanitizer-compatible. **Caveat:** there is no *standalone* ASan unit-test lane over the corpus/oracle outside fuzzing (MS-G3, deferred) — ASan coverage is only as deep as the fuzz corpus reaches. |
| MS-10 | **`clippy -D warnings` gate** — the lint-clean workspace bars classes of memory/aliasing footguns clippy detects. | **PASS** | `ci.yml` `cargo clippy --workspace --all-targets -- -D warnings` (GATING) + a `wasm32` clippy gate. Verified the `-D warnings` invocation is present and not `continue-on-error`. |
| MS-11 | **Dependency memory-safety** — third-party `unsafe` (memmap2, libc, rayon, hdt, …) is governed by the supply-chain lane, not silently trusted. | **PASS** (defers per-dep proof to supply-chain) | `cargo deny check advisories bans sources licenses` — **all gating** (`.github/workflows/supply-chain.yml`; GX-1 advisories un-degraded, `continue-on-error` removed). Daily `dependency-monitoring.yml` watchdog. The *register* explicitly scopes third-party `unsafe` OUT to this lane (no laundering of dep unsafe into a first-party claim). |
| MS-12 | **Edition-2024 `unsafe` hygiene** — the new edition-2024 `unsafe` obligations (`set_var`/`remove_var` now `unsafe`) are enumerated, not hidden — the 2 test-only env sites are in the register with their single-threaded-test invariant. | **PASS** | `unsafe-register.md` rows `src/lib.rs:6229`, `:6231` (TEST-only, restored before return). Counted by the ratchet like any other site. |
| MS-13 | **No `unsafe` in the parser / planner / executor / reasoner / SHACL layers** — the RDF/SPARQL ingest + query path that touches untrusted *query/data* text is 100% safe Rust (the untrusted-bytes-to-mmap boundary is the only unsafe surface). | **PASS** | `sparq-engine`, `sparq-parse`, `sparq-reason`, `sparq-shacl`, vendored `spargebra` are all `#![forbid(unsafe_code)]` / no `unsafe` (per MS-1 list + `research/threat-model.md` §scope table). Untrusted *text* never reaches `unsafe`; only the on-disk index (B5) does. |

## External / out-of-agent-scope items (label, do not claim)

- **Formal verification of the unsafe core** (e.g. Kani / Prusti / a separation-logic
  proof of the mmap validators) is **not** done — the assurance is Miri + oracle + fuzz +
  the per-site argument, which is strong but is *testing/justification*, not *proof*. An
  external formal-methods review would be the certificate-grade step. Tracked: gap MS-G4.
- **An accredited third-party memory-safety audit** of the B5 surface is external by
  definition; the register + coverage matrix is the evidence pack such an auditor would
  consume.

## Coverage matrix (which mechanism bounds which class of site)

| Site class | # sites | Miri (MS-6) | Oracle (MS-7) | Fuzz+ASan (MS-8/9) | Per-site arg (MS-2/5) |
|---|---|:--:|:--:|:--:|:--:|
| Pure-Rust in-memory (scatter writes, POD↔bytes, `from_utf8_unchecked` over our buffers, `set_len`) | ~26 | ✅ runs | n/a | partial (reached via parse path) | ✅ |
| mmap-backed B5 (every `Mmap::map`/`MmapMut`, slice-reinterpret off a live map, `madvise`) | 16 | ❌ structurally cannot | ✅ | ✅ (`graph_open`) | ✅ |
| dict-spill scatter + FFI (`sysconf`/`statvfs`, hash-routed `*mut u64`) | 7 | ❌ (`dict-spill`⇒`mmap`) | ✅ | ✅ | ✅ |
| Prefetch hints / asm (`_mm_prefetch`, `prfm`, ptr `add` for hint) | ~4 | ✅ (hint-only, cannot fault) | n/a | n/a | ✅ |
| Bench/CLI/zk-compose FFI (`getrusage`, `flock`, dump-perm mmap) | ~3 | n/a (non-core) | n/a | n/a | ✅ |

The matrix is the honest answer to "is every unsafe site covered by *something*": yes —
every site has at least the per-site argument (MS-2), and every B5 mmap site that Miri
cannot reach is covered by the oracle **and** the fuzz+ASan lane.
