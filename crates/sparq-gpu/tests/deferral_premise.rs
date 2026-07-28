//! [OPUS-5] sq-vrye — trip-wire for the deferred `sparq-gpu` threat model.
//!
//! The threat model for this crate is deliberately DEFERRED, and the deferral is
//! justified by unreachability, not by benignity: nothing depends on the crate and
//! it ships in no artifact, so no adversary can invoke a kernel. See
//! `research/gpu-threat-model-deferral.md`.
//!
//! A deferral held only in prose expires silently. These tests hold it in the build:
//! the moment the premise stops being true — the crate gains a dependent, or becomes
//! publishable — they fail and point at the record, so the PR that invalidates the
//! deferral is the PR that has to deal with it.
//!
//! Neither test needs a GPU; unlike `correctness.rs` they never skip.

use std::fs;
use std::path::{Path, PathBuf};

/// Where the exit trigger and the pre-scoped model outline live.
const RECORD: &str = "research/gpu-threat-model-deferral.md";

/// `crates/sparq-gpu` sits two levels below the workspace root.
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crates/<name> is two levels below the workspace root")
        .to_path_buf()
}

/// The manifest lines that actually declare something, with comments stripped.
fn code_lines(manifest: &str) -> impl Iterator<Item = &str> {
    manifest
        .lines()
        .map(|line| line.split('#').next().unwrap_or("").trim())
        .filter(|line| !line.is_empty())
}

/// Exit trigger (b): publication. Removing `publish = false` puts the crate — and
/// the whole `wgpu`/`naga` tree — into a shipped artifact.
#[test]
fn crate_is_still_unpublished() {
    let manifest = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("sparq-gpu Cargo.toml");
    let unpublished = code_lines(&manifest)
        .any(|line| line.starts_with("publish") && line.contains("false"));
    assert!(
        unpublished,
        "sparq-gpu is being published, which ends the sq-vrye threat-model deferral \
         (exit trigger (b)). Write the threat model scoped by {RECORD} §6, or state \
         in the PR why publishing does not create the boundary that record describes."
    );
}

/// Exit trigger (a): integration. A dependent is what turns the kernels from a
/// developer-only measurement rig into code an outside input can reach.
#[test]
fn nothing_in_the_workspace_depends_on_sparq_gpu() {
    let crates_dir = workspace_root().join("crates");
    let entries = fs::read_dir(&crates_dir).expect("crates/ directory");

    let mut dependents: Vec<String> = Vec::new();
    for entry in entries {
        let dir = entry.expect("readable crates/ entry").path();
        let name = match dir.file_name() {
            Some(n) => n.to_string_lossy().into_owned(),
            None => continue,
        };
        if name == "sparq-gpu" {
            continue;
        }
        let Ok(manifest) = fs::read_to_string(dir.join("Cargo.toml")) else {
            continue; // not a crate directory
        };
        if code_lines(&manifest).any(|line| line.contains("sparq-gpu")) {
            dependents.push(name);
        }
    }
    dependents.sort();

    assert!(
        dependents.is_empty(),
        "sparq-gpu now has workspace dependents ({dependents:?}), which ends the sq-vrye \
         threat-model deferral (exit trigger (a)): the kernels become reachable from \
         engine data. Write the threat model scoped by {RECORD} §6 — G-1 (unvalidated \
         group keys) and G-2 (non-terminating probe loop) are the two that turn from \
         caller contracts into attack surface the moment this assertion fires."
    );
}

/// The two tests above are only as good as the document they send you to.
#[test]
fn the_deferral_record_still_exists() {
    let record = workspace_root().join(RECORD);
    assert!(
        record.is_file(),
        "{RECORD} is missing. It carries the exit trigger these tests enforce and the \
         pre-scoped outline for the model itself; deleting it silently un-defers sq-vrye."
    );
}
