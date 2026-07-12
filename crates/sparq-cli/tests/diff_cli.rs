//! Subprocess contract tests for the opt-in `diff` command (sq-lsp7k.28).
#![cfg(feature = "diff")]

use std::path::{Path, PathBuf};
use std::process::Command;

fn scratch(tag: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("diff-cli-{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch directory");
    dir
}

fn write(dir: &Path, name: &str, body: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, body).expect("write RDF fixture");
    path
}

fn run(left: &Path, right: &Path, exact: bool) -> (i32, Vec<u8>, Vec<u8>) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_sparq-cli"));
    command.arg("diff").arg(left).arg(right);
    if exact {
        command.arg("--exact");
    }
    let output = command.output().expect("spawn sparq-cli diff");
    (
        output.status.code().unwrap_or(-1),
        output.stdout,
        output.stderr,
    )
}

#[test]
fn changed_sets_emit_exact_sorted_patch_and_exit_one() {
    let dir = scratch("changed");
    let left = write(
        &dir,
        "left.nt",
        "<http://ex/b> <http://ex/p> <http://ex/shared> .\n\
         <http://ex/a> <http://ex/p> <http://ex/old> .\n",
    );
    let right = write(
        &dir,
        "right.ttl",
        "@prefix ex: <http://ex/> .\nex:c ex:p ex:new .\nex:b ex:p ex:shared .\n",
    );

    let (code, stdout, stderr) = run(&left, &right, false);
    assert_eq!(code, 1, "stderr: {}", String::from_utf8_lossy(&stderr));
    assert_eq!(
        stdout,
        b"- <http://ex/a> <http://ex/p> <http://ex/old> .\n\
          + <http://ex/c> <http://ex/p> <http://ex/new> .\n"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn identical_sets_emit_nothing_and_exit_zero() {
    let dir = scratch("identical");
    let left = write(
        &dir,
        "left.nt",
        "_:shared <http://ex/p> <http://ex/o> .\n\
         <http://ex/s> <http://ex/p> <http://ex/o> .\n",
    );
    let right = write(
        &dir,
        "right.nt",
        "<http://ex/s> <http://ex/p> <http://ex/o> .\n\
         _:shared <http://ex/p> <http://ex/o> .\n",
    );

    let (code, stdout, stderr) = run(&left, &right, true);
    assert_eq!(code, 0, "stderr: {}", String::from_utf8_lossy(&stderr));
    assert!(
        stdout.is_empty(),
        "stdout: {}",
        String::from_utf8_lossy(&stdout)
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn subset_emits_only_additions_and_exit_one() {
    let dir = scratch("subset");
    let left = write(
        &dir,
        "left.nt",
        "<http://ex/b> <http://ex/p> <http://ex/shared> .\n",
    );
    let right = write(
        &dir,
        "right.nt",
        "<http://ex/d> <http://ex/p> <http://ex/new> .\n\
         <http://ex/b> <http://ex/p> <http://ex/shared> .\n",
    );

    let (code, stdout, stderr) = run(&left, &right, false);
    assert_eq!(code, 1, "stderr: {}", String::from_utf8_lossy(&stderr));
    assert_eq!(stdout, b"+ <http://ex/d> <http://ex/p> <http://ex/new> .\n");
    let _ = std::fs::remove_dir_all(&dir);
}
