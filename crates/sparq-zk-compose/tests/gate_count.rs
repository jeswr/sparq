// [OPUS-4.8] written while Fable 5 unavailable — re-review when Fable returns.
//! sq-c5f: gate-count regression gate.
//!
//! Records the ACIR/UltraHonk gate count (`bb gates -s ultra_honk` →
//! `circuit_size`, the noir-optimisation ground truth) of every compiled
//! `zk/compose/` circuit-family member in a CHECKED-IN snapshot
//! (`tests/gate_count_snapshot.json`), and a test that recompiles each member,
//! reads its real `circuit_size`, and FAILS if it exceeds
//! `baseline * (1 + tolerance/100)`. This prevents silent circuit BLOAT (a
//! refactor that quietly doubles a relation's constraints).
//!
//! Tolerance (3%, in the snapshot) absorbs the small cross-platform/build
//! variance in bb's gate counts — measured ~0.5–0.75% on the scan members
//! between the darwin-arm64 baseline and this box's linux-x86_64 build, and 0%
//! on the filter members — while still tripping on real growth (a relation that
//! grows typically moves tens of percent, far outside 3%).
//!
//! Toolchain-gated: needs `nargo` + `bb`. Skips cleanly when absent (like the
//! e2e full-prove tests), so default CI without the toolchain stays green. It is
//! NOT `#[ignore]`d — gate counts are cheap (compile + `bb gates`, no proving),
//! so the regression gate runs in the normal suite when the toolchain is
//! present.
//!
//! Re-baseline after an INTENTIONAL circuit change: run
//! `bench/zk-compose/scripts/gate_counts.sh` and copy the new `circuit_size`
//! values into `tests/gate_count_snapshot.json` (and the bench JSON).

use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The checked-in baseline snapshot (compiled into the test binary).
const SNAPSHOT_JSON: &str = include_str!("gate_count_snapshot.json");

#[derive(Deserialize)]
struct Snapshot {
    tolerance_pct: f64,
    members: std::collections::BTreeMap<String, u64>,
}

/// `bb gates` output shape: `{ "functions": [ { "circuit_size": N, ... } ] }`.
#[derive(Deserialize)]
struct BbGates {
    functions: Vec<BbFunction>,
}
#[derive(Deserialize)]
struct BbFunction {
    circuit_size: u64,
}

fn toolchain_available() -> bool {
    fn on_path(tool: &str) -> bool {
        Command::new(tool)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    on_path("nargo") && on_path("bb")
}

/// `zk/compose/` workspace root, relative to this crate (workspace layout:
/// `crates/sparq-zk-compose` → `../../zk/compose`). Mirrors
/// `CircuitProver::from_crate_root`.
fn compose_dir() -> PathBuf {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    here.parent()
        .and_then(Path::parent)
        .map(|root| root.join("zk").join("compose"))
        .expect("workspace layout")
}

/// Compile the whole workspace once (idempotent), producing `target/<pkg>.json`
/// ACIR for every member.
fn compile_workspace(dir: &Path) {
    let out = Command::new("nargo")
        .arg("compile")
        .arg("--workspace")
        .current_dir(dir)
        .output()
        .expect("nargo runs");
    assert!(
        out.status.success(),
        "nargo compile --workspace failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// `bb gates -s ultra_honk` → the member's `circuit_size`.
fn gate_count(dir: &Path, member: &str) -> u64 {
    let acir = dir.join("target").join(format!("{member}.json"));
    let out = Command::new("bb")
        .arg("gates")
        .arg("-s")
        .arg("ultra_honk")
        .arg("-b")
        .arg(&acir)
        .output()
        .expect("bb runs");
    assert!(
        out.status.success(),
        "bb gates {member} failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let parsed: BbGates =
        serde_json::from_slice(&out.stdout).expect("bb gates emits the expected JSON shape");
    parsed
        .functions
        .first()
        .expect("at least one function in the circuit")
        .circuit_size
}

/// sq-c5f — the regression gate. For every member in the checked-in snapshot,
/// assert the live gate count has not regressed beyond the tolerance.
#[test]
fn gate_count_regression() {
    if !toolchain_available() {
        eprintln!("nargo/bb absent; skipping gate-count regression gate");
        return;
    }
    let snapshot: Snapshot =
        serde_json::from_str(SNAPSHOT_JSON).expect("gate_count_snapshot.json parses");
    let dir = compose_dir();
    compile_workspace(&dir);

    let mut regressions = Vec::new();
    for (member, &baseline) in &snapshot.members {
        let live = gate_count(&dir, member);
        let ceiling = (baseline as f64 * (1.0 + snapshot.tolerance_pct / 100.0)).floor() as u64;
        let delta_pct = (live as f64 - baseline as f64) / baseline as f64 * 100.0;
        eprintln!(
            "{member}: baseline={baseline} live={live} ({delta_pct:+.2}%) ceiling={ceiling}"
        );
        if live > ceiling {
            regressions.push(format!(
                "{member}: live={live} exceeds ceiling={ceiling} (baseline={baseline}, +{delta_pct:.2}% > {:.1}% tolerance)",
                snapshot.tolerance_pct
            ));
        }
    }

    assert!(
        regressions.is_empty(),
        "circuit gate-count REGRESSION(s) detected (silent bloat — re-baseline only if INTENTIONAL):\n{}",
        regressions.join("\n")
    );
}

/// Belt-and-braces: the snapshot must cover every compiled member directory under
/// `zk/compose/` (so a new family member can't be added without a baseline). This
/// runs WITHOUT the toolchain — it only reads directory names + the snapshot.
#[test]
fn snapshot_covers_every_member() {
    let snapshot: Snapshot =
        serde_json::from_str(SNAPSHOT_JSON).expect("gate_count_snapshot.json parses");
    let dir = compose_dir();
    if !dir.exists() {
        eprintln!("zk/compose absent; skipping coverage check");
        return;
    }
    // Member packages are the subdirs with a Nargo.toml that are NOT the shared
    // `compose_core` library (which is not a proving member).
    let mut missing = Vec::new();
    for entry in std::fs::read_dir(&dir).expect("read zk/compose") {
        let entry = entry.expect("dir entry");
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if name == "compose_core" || name == "target" {
            continue;
        }
        if !entry.path().join("Nargo.toml").exists() {
            continue;
        }
        if !snapshot.members.contains_key(&name) {
            missing.push(name);
        }
    }
    assert!(
        missing.is_empty(),
        "compiled member(s) have NO gate-count baseline in tests/gate_count_snapshot.json: {missing:?} \
         — add a baseline (run bench/zk-compose/scripts/gate_counts.sh)"
    );
}
