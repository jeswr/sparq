# dict-spill worktree status

Task: external/spillable term dictionary for `Graph::build_external` (bounded peak RSS
under a configurable memory budget). Design: research/external-dictionary.md.

## Done
- Audit of extsort.rs, dict.rs (ShardedDict, save_mmap format, write_record),
  build_external_opts + sharded ingest pipeline, commit 137fb39, memtier ADDENDUM.
- Design doc committed: research/external-dictionary.md (seq-tagged spill + external
  dedup/rank, byte-identical to the sharded path).

## In flight
- Implementation: crates/sparq-core/src/dictspill.rs (new, feature `dict-spill`),
  lib.rs spill ingest variant + build_external wiring, dict.rs pub(crate) exposure +
  canonical (hash,id) sort in save_mmap.

## Next (exact)
1. Implement dictspill.rs per design phases 1–5.
2. Tests: byte-identity vs build_external_opts(sharded=true) with tiny budget;
   engine differential test file crates/sparq-engine/tests/dict_spill_differential.rs.
3. Fuzz: SPARQ_FUZZ_DICTSPILL=1 mode in crates/sparq-bench/src/fuzz.rs (NT conversion
   + spill build + open), run 20k.
4. Bench: synthetic high-cardinality dataset 10M/100M, /usr/bin/time -l, budget on/off.
5. Gate: cargo test --workspace --exclude sparq-py --release --no-fail-fast 2>&1 | grep -aE "^test result"
6. wasm byte check: cargo build -p sparq-wasm --target wasm32-unknown-unknown --release;
   stat -f%z target/wasm32-unknown-unknown/release/sparq_wasm.wasm (baseline 1,643,095).

## Rules
- Commit on dict-spill only; NEVER push/merge.
