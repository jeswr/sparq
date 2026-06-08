//! Portable GPU compute spike for the `sparq` triplestore.
//!
//! Goal: sanity-check the cross-platform claim that a single `wgpu` compute
//! kernel runs on Metal (this M1 Mac) AND on Vulkan/CUDA (the Dell XPS Ubuntu +
//! GTX 1650 Ti). The kernel models sparq's actual hot path: a parallel
//! predicate-filter + count over a `u32` column (the object column of a
//! permutation index), i.e. the `FILTER(?o >= lo && ?o < hi)` / range-scan
//! case that BENCHMARKS.md flags as sparq's biggest loss vs QLever.
//!
//! It reports, for several column sizes:
//!   - CPU scalar baseline (same predicate, single thread, the honest "what the
//!     engine does today per scan thread" reference);
//!   - GPU compute-only (kernel dispatch + readback of a tiny counter), the
//!     number you'd get if the column already lived in VRAM;
//!   - GPU end-to-end (host->device upload of the column + dispatch + readback),
//!     the number you pay if you must stream the column across PCIe each query.
//!
//! The gap between the last two is the transfer tax that decides when GPU wins.

use bytemuck::{Pod, Zeroable};
use rayon::prelude::*;
use std::time::Instant;
use wgpu::util::DeviceExt;

const WGSL: &str = r#"
struct Params { n: u32, lo: u32, hi: u32, _pad: u32 };
@group(0) @binding(0) var<storage, read>       col:    array<u32>;
@group(0) @binding(1) var<storage, read_write> counts: array<atomic<u32>>;
@group(0) @binding(2) var<uniform>             params: Params;

// One workgroup accumulates into a shared counter, then one thread flushes it to
// a per-workgroup slot in `counts` (final reduction done on the host over the
// few thousand workgroup slots — trivial). This mirrors a selection-count scan.
var<workgroup> local_count: atomic<u32>;

// `gridw` = number of workgroups along x; the dispatch is tiled into a 2D grid
// because WebGPU caps each dispatch dimension at 65535 workgroups.
@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>,
        @builtin(local_invocation_id)  lid: vec3<u32>,
        @builtin(workgroup_id)         wid: vec3<u32>,
        @builtin(num_workgroups)       ngw: vec3<u32>) {
    if (lid.x == 0u) { atomicStore(&local_count, 0u); }
    workgroupBarrier();
    let flat_wg = wid.y * ngw.x + wid.x;
    let i = flat_wg * 256u + lid.x;
    if (i < params.n) {
        let v = col[i];
        if (v >= params.lo && v < params.hi) {
            atomicAdd(&local_count, 1u);
        }
    }
    workgroupBarrier();
    if (lid.x == 0u) {
        atomicStore(&counts[flat_wg], atomicLoad(&local_count));
    }
}
"#;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Params {
    n: u32,
    lo: u32,
    hi: u32,
    _pad: u32,
}

const WG: u32 = 256;

fn main() {
    pollster::block_on(run());
}

async fn run() {
    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            ..Default::default()
        })
        .await
        .expect("no GPU adapter — wgpu found no Metal/Vulkan/DX/GL backend");

    let info = adapter.get_info();
    println!("== wgpu adapter ==");
    println!("  name    : {}", info.name);
    println!("  backend : {:?}", info.backend);
    println!("  type    : {:?}", info.device_type);
    let limits = adapter.limits();
    println!(
        "  max storage buffer : {} MiB",
        limits.max_storage_buffer_binding_size / (1024 * 1024)
    );

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: limits.clone(),
                memory_hints: wgpu::MemoryHints::Performance,
            },
            None,
        )
        .await
        .expect("request_device failed");

    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("filter"),
        source: wgpu::ShaderSource::Wgsl(WGSL.into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("filter"),
        layout: None,
        module: &module,
        entry_point: "main",
        compilation_options: Default::default(),
        cache: None,
    });

    // Cap test sizes to the adapter's max storage binding (the 1650 Ti / Metal
    // both allow >=128 MiB => >=32M u32, plenty for the spike).
    let max_elems = (limits.max_storage_buffer_binding_size as usize / 4).min(64_000_000);
    let sizes: Vec<usize> = [1_000_000usize, 4_000_000, 16_000_000, 64_000_000]
        .into_iter()
        .filter(|&n| n <= max_elems)
        .collect();

    // ~12.5% selectivity (lo..hi spans 1/8 of the value range) — a moderately
    // selective FILTER, the interesting middle ground.
    let lo = 0u32;
    let hi = u32::MAX / 8;

    println!("\n== filter+count: v in [{lo}, {hi})  (~12.5% selectivity) ==");
    println!(
        "{:>11} | {:>8} | {:>8} | {:>11} | {:>10} | {:>9} | {:>9}",
        "elems", "cpu1 ms", "cpuN ms", "gpu compute", "gpu e2e", "cpu1/gpu", "cpuN/gpu"
    );
    println!("{}", "-".repeat(82));

    for &n in &sizes {
        // Deterministic pseudo-random column of u32 ids.
        let mut col = vec![0u32; n];
        let mut s = 0x9E3779B97F4A7C15u64;
        for x in col.iter_mut() {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            *x = (s >> 11) as u32;
        }

        // ---- CPU scalar baseline (best-of-N) ----
        let cpu_iters = 5;
        let mut cpu_best = f64::MAX;
        let mut cpu_count = 0u64;
        for _ in 0..cpu_iters {
            let t = Instant::now();
            let mut c = 0u64;
            for &v in &col {
                if v >= lo && v < hi {
                    c += 1;
                }
            }
            cpu_best = cpu_best.min(t.elapsed().as_secs_f64() * 1e3);
            cpu_count = c;
        }

        // ---- CPU rayon parallel baseline (all cores; the engine's real CPU path) ----
        let mut par_best = f64::MAX;
        for _ in 0..cpu_iters {
            let t = Instant::now();
            let c: u64 = col
                .par_iter()
                .map(|&v| (v >= lo && v < hi) as u64)
                .sum();
            par_best = par_best.min(t.elapsed().as_secs_f64() * 1e3);
            assert_eq!(c, cpu_count);
        }

        let n_groups = ((n as u32) + WG - 1) / WG;
        // Padded grid: the 2D tiling launches gx*gy >= n_groups workgroups, each
        // of which writes one slot, so the counts buffer must cover the padded
        // total (extra slots stay 0 and contribute nothing to the sum).
        let gx = n_groups.min(65535);
        let gy = (n_groups + 65535 - 1) / 65535;
        let n_slots = gx * gy;
        let params = Params { n: n as u32, lo, hi, _pad: 0 };

        // Persistent device buffers (column lives in VRAM for the compute-only #).
        let col_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("col"),
            contents: bytemuck::cast_slice(&col),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        let counts_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("counts"),
            size: (n_slots as u64) * 4,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let params_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("params"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: (n_slots as u64) * 4,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: col_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: counts_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: params_buf.as_entire_binding() },
            ],
        });

        let dispatch = |label: &str| {
            let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            {
                let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some(label),
                    timestamp_writes: None,
                });
                cp.set_pipeline(&pipeline);
                cp.set_bind_group(0, &bind, &[]);
                // 2D-tile the workgroup grid: each dim is capped at 65535.
                let gx = n_groups.min(65535);
                let gy = (n_groups + 65535 - 1) / 65535;
                cp.dispatch_workgroups(gx, gy, 1);
            }
            enc.copy_buffer_to_buffer(&counts_buf, 0, &readback, 0, (n_slots as u64) * 4);
            enc
        };

        let read_count = || -> u64 {
            let slice = readback.slice(..);
            let (tx, rx) = std::sync::mpsc::channel();
            slice.map_async(wgpu::MapMode::Read, move |r| tx.send(r).unwrap());
            device.poll(wgpu::Maintain::Wait);
            rx.recv().unwrap().unwrap();
            let data = slice.get_mapped_range();
            let groups: &[u32] = bytemuck::cast_slice(&data);
            let total: u64 = groups.iter().map(|&g| g as u64).sum();
            drop(data);
            readback.unmap();
            total
        };

        // Warmup (compile/cache/allocate paths).
        queue.submit(Some(dispatch("warmup").finish()));
        let _ = read_count();

        // ---- GPU compute-only (column already resident) ----
        let gpu_iters = 10;
        let mut gpu_best = f64::MAX;
        let mut gpu_count = 0u64;
        for _ in 0..gpu_iters {
            let t = Instant::now();
            queue.submit(Some(dispatch("compute").finish()));
            gpu_count = read_count();
            gpu_best = gpu_best.min(t.elapsed().as_secs_f64() * 1e3);
        }

        // ---- GPU end-to-end (re-upload the column each time = PCIe tax) ----
        let mut e2e_best = f64::MAX;
        for _ in 0..gpu_iters {
            let t = Instant::now();
            queue.write_buffer(&col_buf, 0, bytemuck::cast_slice(&col));
            queue.submit(Some(dispatch("e2e").finish()));
            let _ = read_count();
            e2e_best = e2e_best.min(t.elapsed().as_secs_f64() * 1e3);
        }

        assert_eq!(cpu_count, gpu_count, "GPU/CPU count mismatch at n={n}");
        println!(
            "{:>11} | {:>8.3} | {:>8.3} | {:>11.3} | {:>10.3} | {:>8.2}x | {:>8.2}x",
            n, cpu_best, par_best, gpu_best, e2e_best, cpu_best / gpu_best, par_best / gpu_best
        );
    }

    println!(
        "\nNote: 'gpu compute' assumes the column is resident in VRAM; 'gpu e2e' \
         re-uploads it each call (PCIe). On this Mac the GPU is integrated \
         (unified memory) so 'xfer' is a memcpy, not a bus transfer — on the \
         XPS's discrete 1650 Ti it crosses PCIe x?? and will be larger."
    );
}
