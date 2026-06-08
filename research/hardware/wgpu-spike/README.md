# wgpu-spike — portable GPU compute sanity check for sparq

A tiny **standalone** crate (its own `[workspace]`; NOT a member of the sparq
workspace, so it never perturbs the engine build) that benchmarks sparq's hottest
path — a parallel predicate-filter + count over a `u32` column (the object column
of a permutation index, the `FILTER(?o in [lo,hi))` / range-scan case) — across:

- single-thread scalar CPU,
- all-core rayon CPU,
- GPU compute-only (column resident in VRAM),
- GPU end-to-end (re-upload the column each call = the transfer tax).

It uses **wgpu** so the *same WGSL kernel* runs on Metal (the M1 dev Mac),
Vulkan/CUDA (the Dell XPS NVIDIA GPU), DX12, and WebGPU — proving the "write the
GPU path once, run everywhere including the browser" strategy.

## Run

```sh
cargo run --release
```

It prints the adapter (backend + device) and a table of times + cpu/gpu ratios for
1M / 4M / 16M / 64M element columns. Counts are asserted equal CPU vs GPU.

## What it found (see ../gpu-and-cloud.md §0 for the full numbers)

On M1/Metal: GPU compute-only wins only above ~16–64 M elements (~2–3× vs CPU);
GPU end-to-end (with upload) **loses everywhere** (4–12× slower) because the
transfer tax dominates — even on M1's unified memory. Conclusion: a GPU path for
sparq must keep data **resident** in VRAM; per-query host→device streaming is a
non-starter.

## Run it on the XPS (NVIDIA / Vulkan)

See `../remote-access-setup.md §5.3`. Expect the adapter line to read
`backend: Vulkan / NVIDIA GeForce GTX 1650 Ti` — the same kernel, different
silicon — and compare the crossover numbers to the M1/Metal ones.
