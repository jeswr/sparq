# sparq-gpu — the T24d GPU-execution prototype (measured, and parked)

Opt-in [wgpu](https://wgpu.rs) compute kernels for sparq's three hot-path
primitive shapes — FILTER, hash-join probe, GROUP BY COUNT+SUM — over plain
`ValueId`-shaped columns (`&[u32]` / `&[f64]`), built to answer one question
with measurements instead of aspiration:

> **Where (if anywhere) does a GPU beat the CPU for sparq's hot paths once the
> host→device transfer tax is charged honestly?**

The full measured answer, tables, and the recommendation live in
[`research/gpu-verdict.md`](../../research/gpu-verdict.md). One-line version:

> **Parked.** On this M1 (unified memory — the *best possible* transfer
> economics), the GPU loses or merely ties the 8-core CPU on compute-light
> scans even when the column is already device-resident, and only wins on the
> hash-probe shape (~1.5–3× resident, ~1–1.8× including transfer). One winning
> kernel class does not pay for a residency cache, scheduler integration, and
> a second execution backend. Re-open only per the conditions in the verdict.

## What is here

- `src/lib.rs` — the `Gpu` handle (adapter detection, four WGSL compute
  pipelines) and device-resident column/hash-table types.
  - `Gpu::new()` returns `Option`: **no adapter ⇒ `None`**, so CI without a
    GPU skips gracefully (runtime check, not a compile-time assumption).
  - `filter_count_u32` / `filter_count_f64_gt` — FILTER + count. WGSL has no
    f64, so the f64 kernel compares IEEE-754 bit patterns mapped to a
    monotonic u64 key (sign-flip trick): **exact**, NaN-correct, no float math
    on the device.
  - `hash_probe` — probe a resident open-addressing table (linear probing,
    load ≤ 0.5), counting matches and summing payloads (u64 via 32-bit atomic
    carry emulation).
  - `group_aggregate` — COUNT+SUM GROUP BY (keys pre-densified to
    `0..g ≤ 512`) with two-level (workgroup-shared → global) atomics.
- `src/cpu.rs` — scalar CPU reference implementations: the correctness oracles
  and the single-thread baselines.
- `tests/correctness.rs` — GPU vs CPU on random data + IEEE-754 edge cases
  (NaN/±inf/-0.0/subnormals). Skips (passes, with a stderr note) when no
  adapter exists.
- `examples/gpu_bench.rs` — the experiment. Interleaved best-of-N over four
  legs per kernel × {1M, 10M, 100M} elements: `cpu1` (scalar), `cpuN`
  (rayon all-cores), `gpu resident` (column already in device memory),
  `gpu e2e` (re-upload all inputs every call). Checksums asserted equal
  across all legs every round.

  ```sh
  cargo run -p sparq-gpu --release --example gpu_bench
  ```

## Deliberate non-goals

- **No sparq-core dependency.** The kernels take plain slices in exactly the
  shape of sparq-core's permutation-index object columns and dense numeric
  cache, so the measurement needed no engine plumbing — and the engine carries
  zero GPU code.
- **Nothing depends on this crate.** It must never enter the wasm build:
  `cargo tree -p sparq-wasm --target wasm32-unknown-unknown` contains no
  `wgpu` (CI-checkable).
- **Kernels reduce on-device** and read back O(1)/O(groups) bytes — the best
  case for the GPU. Materialising selected rows would add a device→host
  stream and can only make the GPU look worse, so the measured verdict is an
  *upper bound* on GPU benefit.

## Limits worth knowing

- WGSL has no `f64`: the bit-order trick makes comparisons exact, but any
  future *arithmetic* (SUM over xsd:double) would need emulation or downcast.
- `group_aggregate` caps at 512 groups (workgroup shared-memory tile); higher
  cardinalities need a different (global-atomic or sort-based) kernel.
- Hash tables are built on the host; only the probe is offloaded.
- `max_storage_buffer_binding_size` caps resident column size (4 GiB − 1 on
  this M1; commonly 128 MiB–2 GiB elsewhere — check `Gpu::max_storage_bytes`).
