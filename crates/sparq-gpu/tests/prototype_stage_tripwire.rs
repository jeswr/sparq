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

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

/// This crate's own manifest, baked in at compile time — no cwd dependence.
const OWN_MANIFEST: &str = include_str!("../Cargo.toml");

/// The package whose reachability the deferral rests on.
const GPU_CRATE: &str = "sparq-gpu";

/// The workspace root manifest — the grandparent of this crate's directory.
fn workspace_root_manifest() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/sparq-gpu sits two directories below the workspace root")
        .join("Cargo.toml")
}

/// Drop `#` comments so prose *mentioning* the crate is never read as a manifest key
/// (this crate's own manifest, for one, discusses `sparq-gpu` at length in comments).
///
/// Heuristic: a `#` inside a TOML string truncates that line early. That can only make
/// the scan quieter on an exotic line, never raise a false alarm.
fn strip_comments(manifest: &str) -> String {
    manifest
        .lines()
        .map(|line| line.split('#').next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Ask **cargo itself** for the workspace's members and their dependency *package
/// identities*, rather than scanning manifest text.
///
/// This is the whole point of the helper: a textual scan cannot honestly claim to cover
/// "every spelling cargo accepts". TOML permits quoted keys, so `"sparq-gpu" = { … }` and
/// `[dependencies.alias]` + `"package" = "sparq-gpu"` name the same dependency while
/// matching no plausible substring rule; and the member set is whatever `[workspace]
/// members` (globs included) says, which need not live under `crates/*`. Cargo resolves
/// all of that for us — quoted and dotted keys, `[dependencies.x]` tables, renames,
/// `[target.'cfg(…)'.…]` sections, dev/build kinds, and members anywhere on disk.
///
/// `--no-deps` reads only the workspace's own manifests: no dependency resolution, no
/// lockfile, no registry access (`--offline` makes that explicit), so this stays fast and
/// hermetic on GPU-less CI. The two dev-dependencies it costs (`serde_json`, `tempfile`)
/// are already in the workspace lockfile and are dev-only, so the trip-wire still does not
/// grow the shipped dependency graph it exists to police.
fn workspace_metadata(manifest: &Path) -> Value {
    let output = Command::new(env!("CARGO"))
        .args([
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
            "--offline",
            "--manifest-path",
        ])
        .arg(manifest)
        .output()
        .expect("cargo metadata must run for the sparq-gpu deferral trip-wire");
    assert!(
        output.status.success(),
        "cargo metadata failed for {}: {}",
        manifest.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("cargo metadata emits format-version 1 JSON")
}

/// Names of the workspace members that declare a dependency on `package` — of any kind
/// (normal, dev, build) and under any `[target.…]` cfg. `dependencies[].name` is the real
/// package identity cargo resolved, so a renamed dependency is reported under the
/// *dependent's* name just like a plain one. A package is never its own dependent.
fn dependents_on(metadata: &Value, package: &str) -> Vec<String> {
    let members: BTreeSet<&str> = metadata["workspace_members"]
        .as_array()
        .expect("cargo metadata lists workspace_members")
        .iter()
        .filter_map(Value::as_str)
        .collect();

    let mut dependents: BTreeSet<String> = BTreeSet::new();
    for pkg in metadata["packages"]
        .as_array()
        .expect("cargo metadata lists packages")
    {
        if !pkg["id"].as_str().is_some_and(|id| members.contains(id)) {
            continue;
        }
        let name = pkg["name"].as_str().expect("every package has a name");
        if name == package {
            continue;
        }
        let depends = pkg["dependencies"]
            .as_array()
            .into_iter()
            .flatten()
            .any(|dep| dep["name"].as_str() == Some(package));
        if depends {
            dependents.insert(name.to_string());
        }
    }
    dependents.into_iter().collect()
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
    let metadata = workspace_metadata(&workspace_root_manifest());
    let dependents = dependents_on(&metadata, GPU_CRATE);

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

/// The trip-wire above is worthless if the detector cannot fire, so pin it directly
/// against a real fixture workspace built on disk and read back through the same
/// `cargo metadata` path.
///
/// Every dependent here uses a spelling the previous *textual* scan missed — quoted
/// dependency keys, a quoted `package` rename key, a dotted quoted dev-dependency, a
/// `[target.…]` build-dependency rename — and one dependent is a member outside
/// `crates/*`, which the previous directory walk never even opened. Neither prose,
/// workspace membership, a same-prefix package, nor the crate itself may raise an alarm.
#[test]
fn dependency_detector_catches_every_spelling() {
    let workspace = tempfile::tempdir().expect("a temp dir for the fixture workspace");
    let root = workspace.path();

    let member = |relative: &str, manifest: &str| {
        let dir = root.join(relative);
        fs::create_dir_all(dir.join("src")).expect("fixture crate directory");
        fs::write(dir.join("src").join("lib.rs"), "").expect("fixture crate root");
        fs::write(dir.join("Cargo.toml"), manifest).expect("fixture manifest");
    };

    // Members deliberately spread beyond `crates/*`, and declared partly by glob.
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nresolver = \"2\"\nmembers = [\"tools/nested-tool\", \"crates/*\", \
         \"vendor/sparq-gpu\", \"vendor/sparq-gpu-helper\"]\n",
    )
    .expect("fixture workspace manifest");

    // The subject itself, and a same-prefix package that must never be confused with it.
    member(
        "vendor/sparq-gpu",
        "[package]\nname = \"sparq-gpu\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    member(
        "vendor/sparq-gpu-helper",
        "[package]\nname = \"sparq-gpu-helper\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );

    // Quoted dependency key, in a member that lives outside `crates/`.
    member(
        "tools/nested-tool",
        r#"[package]
name = "nested-tool"
version = "0.1.0"
edition = "2021"

[dependencies]
"sparq-gpu" = { path = "../../vendor/sparq-gpu" }
"#,
    );
    // Rename whose `package` key is itself quoted.
    member(
        "crates/renamed-quoted",
        r#"[package]
name = "renamed-quoted"
version = "0.1.0"
edition = "2021"

[dependencies.alias]
"package" = "sparq-gpu"
path = "../../vendor/sparq-gpu"
"#,
    );
    // Dotted quoted key, and a dev-dependency rather than a normal one.
    member(
        "crates/devdep",
        r#"[package]
name = "devdep"
version = "0.1.0"
edition = "2021"

[dev-dependencies]
"sparq-gpu".path = "../../vendor/sparq-gpu"
"#,
    );
    // Quoted rename inside a `[target.…]` build-dependency inline table.
    member(
        "crates/targeted",
        r#"[package]
name = "targeted"
version = "0.1.0"
edition = "2021"

[target.'cfg(unix)'.build-dependencies]
gpu = { "package" = "sparq-gpu", path = "../../vendor/sparq-gpu" }
"#,
    );
    // Prose mentioning the crate, plus a dependency on a same-prefix package.
    member(
        "crates/innocent",
        r#"# nothing depends on sparq-gpu
[package]
name = "innocent"
version = "0.1.0"
edition = "2021"

[dependencies]
sparq-gpu-helper = { path = "../../vendor/sparq-gpu-helper" }
"#,
    );

    let metadata = workspace_metadata(&root.join("Cargo.toml"));
    assert_eq!(
        dependents_on(&metadata, GPU_CRATE),
        vec!["devdep", "nested-tool", "renamed-quoted", "targeted"],
        "the detector must see every dependency spelling cargo accepts, from every \
         workspace member, and only those"
    );
}
