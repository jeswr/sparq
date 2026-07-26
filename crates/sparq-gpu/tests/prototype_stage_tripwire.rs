//! Trip-wire for the DEFERRED sparq-gpu threat model (bead sq-vrye, issue #3387).
//!
//! `sparq-gpu` is deliberately listed **out of scope** in `research/threat-model.md`:
//! it is a `publish = false`, depended-on-by-nothing measurement prototype (roadmap
//! T24d; verdict **PARK** in `research/gpu-verdict.md`), so a STRIDE model of it today
//! would model a surface no user of sparq can reach. That deferral is only honest
//! while both of the facts that justify it hold:
//!
//! 1. the crate is **not published** (`publish = false` in its own manifest), and
//! 2. **nothing in the workspace depends on it** — it is not a reachable execution
//!    backend, just a re-test rig.
//!
//! Either one ceasing to be true *is* "exiting the prototype stage", the trigger
//! sq-vrye names. These tests fail at exactly that moment, so the deferral expires
//! loudly rather than rotting: whoever publishes the crate or wires the kernels into
//! the engine has to write the threat model in the same change.
//!
//! This is a repository-invariant test, not a kernel test: it needs no GPU adapter and
//! runs (and must stay green) on GPU-less CI alongside `correctness.rs`.

use std::fs;
use std::path::{Path, PathBuf};

/// This crate's own manifest, baked in at compile time — no cwd dependence.
const OWN_MANIFEST: &str = include_str!("../Cargo.toml");

/// The `crates/` directory — the parent of this crate's directory.
fn crates_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/sparq-gpu has a parent directory")
        .to_path_buf()
}

/// Drop `#` comments so prose *mentioning* the crate is never read as a dependency
/// (this crate's own manifest, for one, discusses `sparq-gpu` at length in comments).
///
/// Heuristic: a `#` inside a TOML string truncates that line early. That can only make
/// the scan quieter on an exotic line (e.g. a `git = "…#branch"` dependency), never
/// raise a false alarm, and no manifest in this workspace declares a dependency that
/// way.
fn strip_comments(manifest: &str) -> String {
    manifest
        .lines()
        .map(|line| line.split('#').next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Does this manifest declare a dependency **on** `sparq-gpu`, in any of the spellings
/// cargo accepts? Note the workspace-root `members = [… "crates/sparq-gpu" …]` listing
/// is membership, not a dependency, and must NOT match.
fn declares_sparq_gpu_dependency(manifest: &str) -> bool {
    strip_comments(manifest).lines().map(str::trim).any(|line| {
        // `sparq-gpu = { … }`, `sparq-gpu.workspace = true`, `sparq-gpu.version = …`
        line.strip_prefix("sparq-gpu")
            .is_some_and(|rest| rest.starts_with([' ', '=', '.']))
            // `[dependencies.sparq-gpu]`, `[target.'cfg(…)'.dev-dependencies.sparq-gpu]`
            || (line.starts_with('[') && line.ends_with(".sparq-gpu]"))
    })
}

/// Fact 1: the crate is still unpublished.
#[test]
fn sparq_gpu_is_still_unpublished() {
    let unpublished = strip_comments(OWN_MANIFEST)
        .lines()
        .map(str::trim)
        .any(|line| line.starts_with("publish") && line.contains("false"));

    assert!(
        unpublished,
        "crates/sparq-gpu dropped `publish = false`, so it is leaving the \
         measurement-prototype stage — the exact trigger bead sq-vrye / issue #3387 \
         defers the threat model on. Before this lands: write the model (move the \
         `sparq-gpu` row out of the deferred out-of-scope table in \
         research/threat-model.md into a modelled boundary) and delete this trip-wire."
    );
}

/// Fact 2: nothing in the workspace depends on it — kernels are unreachable from any
/// shipped surface, so there is no attacker-reachable path to model yet.
#[test]
fn nothing_in_the_workspace_depends_on_sparq_gpu() {
    let crates = crates_dir();
    let mut dependents: Vec<String> = Vec::new();

    // The workspace root manifest, whose `[workspace.dependencies]` would be the first
    // step of wiring the crate in.
    let root = crates.join("..").join("Cargo.toml");
    if let Ok(text) = fs::read_to_string(&root) {
        if declares_sparq_gpu_dependency(&text) {
            dependents.push("<workspace root>".to_string());
        }
    }

    for entry in fs::read_dir(&crates).expect("crates/ is readable") {
        let dir = entry.expect("readable directory entry").path();
        let name = match dir.file_name().and_then(|n| n.to_str()) {
            Some("sparq-gpu") | None => continue,
            Some(name) => name.to_string(),
        };
        if let Ok(text) = fs::read_to_string(dir.join("Cargo.toml")) {
            if declares_sparq_gpu_dependency(&text) {
                dependents.push(name);
            }
        }
    }

    assert!(
        dependents.is_empty(),
        "{} now depend(s) on sparq-gpu, so the GPU kernels have become reachable from \
         another crate — the exact trigger bead sq-vrye / issue #3387 defers the threat \
         model on. Before this lands: model the new boundary (untrusted column data → \
         WGSL kernel → device, plus adapter/driver trust and the readback path) in \
         research/threat-model.md and delete this trip-wire.",
        dependents.join(", ")
    );
}

/// The trip-wires above are worthless if the detector cannot fire, so pin it directly:
/// every dependency spelling must be caught, and neither prose nor workspace membership
/// may raise a false alarm.
#[test]
fn dependency_detector_catches_every_spelling() {
    for manifest in [
        "[dependencies]\nsparq-gpu = { path = \"../sparq-gpu\" }\n",
        "[dependencies]\nsparq-gpu.workspace = true\n",
        "[dev-dependencies]\nsparq-gpu = \"0.1\"\n",
        "[dependencies.sparq-gpu]\npath = \"../sparq-gpu\"\n",
        "[target.'cfg(not(target_arch = \"wasm32\"))'.dependencies.sparq-gpu]\nversion = \"0.1\"\n",
    ] {
        assert!(
            declares_sparq_gpu_dependency(manifest),
            "missed a real dependency: {:?}",
            manifest
        );
    }

    for manifest in [
        // Workspace membership is not a dependency.
        "[workspace]\nmembers = [\"crates/sparq-core\", \"crates/sparq-gpu\"]\n",
        // Prose in a comment is not a dependency.
        "# nothing depends on sparq-gpu\n[dependencies]\nwgpu = \"30\"\n",
        "[dependencies]\nsparq-gpu-helper = \"0.1\"\n",
        "[package]\nname = \"sparq-gpu\"\n",
    ] {
        assert!(
            !declares_sparq_gpu_dependency(manifest),
            "false alarm on: {:?}",
            manifest
        );
    }
}
