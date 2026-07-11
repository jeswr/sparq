// AUTHORED-BY Claude Opus 4.8
//! DETERMINISTIC in-process micro-benchmark of `iri_chars_serialisable` (the per-child structural
//! guard the container-listing render runs once per member). It times the REFERENCE `.chars()` +
//! Unicode-`is_control` predicate against the optimised ASCII-byte-scan predicate, over a batch of
//! representative store-minted child IRIs (all ASCII — the common case), so the relative cost of the
//! eliminated per-char Unicode property-table lookup is measured in isolation from network/TLS/box
//! noise (the HTTP listing sweep on a loaded box is dominated by syscall+malloc variance and can't
//! resolve a function-level micro-opt). Wall-clock per-op is advisory; the RATIO is the trustworthy
//! figure. Run: `cargo run --release --example iri_guard_microbench [-- <iters>]`.
use std::hint::black_box;
use std::time::Instant;

/// The REFERENCE predicate: the pre-optimisation `.chars()` + `is_control` implementation.
fn reference(iri: &str) -> bool {
    if iri.is_empty() {
        return false;
    }
    !iri.chars().any(|c| {
        c.is_control()
            || c == ' '
            || matches!(c, '<' | '>' | '"' | '{' | '}' | '|' | '^' | '`' | '\\')
    })
}

/// The OPTIMISED predicate: ASCII byte-scan fast path, char fallback only on a non-ASCII byte.
/// (Byte-for-byte identical result to `reference` — pinned by the in-tree equivalence test.)
fn optimised(iri: &str) -> bool {
    if iri.is_empty() {
        return false;
    }
    let bytes = iri.as_bytes();
    let mut all_ascii = true;
    for &b in bytes {
        if b >= 0x80 {
            all_ascii = false;
            break;
        }
        if b < 0x20
            || b == 0x7F
            || matches!(
                b,
                b' ' | b'<' | b'>' | b'"' | b'{' | b'}' | b'|' | b'^' | b'`' | b'\\'
            )
        {
            return false;
        }
    }
    if all_ascii {
        return true;
    }
    !iri.chars().any(|c| {
        c.is_control()
            || c == ' '
            || matches!(c, '<' | '>' | '"' | '{' | '}' | '|' | '^' | '`' | '\\')
    })
}

fn time_it<F: FnMut()>(iters: u64, mut f: F) -> f64 {
    let start = Instant::now();
    for _ in 0..iters {
        f();
    }
    start.elapsed().as_nanos() as f64 / iters as f64
}

fn main() {
    let iters: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(2_000_000);

    // A representative listing membership: store-minted child IRIs (all ASCII, the common case).
    let base = "https://localhost:3000/alice/test/";
    let children: Vec<String> = (0..100).map(|i| format!("{base}item-{i:04}")).collect();

    // Sanity: both predicates agree on every input (the equivalence the optimisation rests on).
    for c in &children {
        assert_eq!(reference(c), optimised(c));
    }

    // Warm up.
    for _ in 0..50_000 {
        for c in &children {
            black_box(optimised(black_box(c)));
        }
    }

    let ref_op = time_it(iters, || {
        for c in &children {
            black_box(reference(black_box(c)));
        }
    });
    let opt_op = time_it(iters, || {
        for c in &children {
            black_box(optimised(black_box(c)));
        }
    });

    let per_child_ref = ref_op / children.len() as f64;
    let per_child_opt = opt_op / children.len() as f64;
    println!(
        "# iri_chars_serialisable micro-bench — iters={iters}, {} children/iter",
        children.len()
    );
    println!("# load={}", load_avg());
    println!();
    println!("REFERENCE (.chars()+is_control)  {per_child_ref:8.3} ns/child   ({ref_op:.1} ns / 100-child render)");
    println!("OPTIMISED (ASCII byte-scan)      {per_child_opt:8.3} ns/child   ({opt_op:.1} ns / 100-child render)");
    println!(
        "DELTA                            {:8.3} ns/child   ({:.1}x faster on the all-ASCII path)",
        per_child_ref - per_child_opt,
        ref_op / opt_op
    );
}

fn load_avg() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|s| s.split_whitespace().next().map(str::to_string))
        .unwrap_or_else(|| "n/a(macos)".to_string())
}
