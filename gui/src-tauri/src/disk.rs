// [OPUS-4.8] sq-cno90 (#820 follow-up) — a precise native disk-usage probe for the GUI status
// bar's store-footprint figure.
//
// WHY. The status bar's store-footprint estimate is the UTF-8 byte length of the persisted
// whole-dataset N-Quads snapshot (the engine context's snapshot, sq-ixc3.13). That is an
// HONEST measured value, but it is an ESTIMATE of the on-disk size — it does not count the
// workspace index JSON, the per-workspace metadata, or any encoding overhead the on-disk
// `<app-local-data>/workspaces/*.json` files carry. This module adds the OS-reported number:
// a recursive `stat()` walk of the resolved `$APPLOCALDATA/workspaces` tree that sums the real
// byte size every file reports, so the desktop shell can show what the OS actually stores.
//
// HONESTY. This is the REAL on-disk byte total of the workspaces tree, or nothing — never a
// fabricated figure. The status bar PREFERS this OS-reported value in the desktop shell and
// falls back to the snapshot-bytes estimate on the web target (where no native FS exists),
// always making clear which of the two is shown. The walk counts regular-file byte lengths
// (`metadata.len()`); it does not chase symlinks out of the tree and does not try to model the
// filesystem's block/allocation overhead (an apparent-size sum, the same notion `du --bytes`
// reports), so it is exact for the JSON workspace files this app writes.
//
// No `unsafe`, no performance claim — this is a plain `std::fs` walk.

use std::path::Path;

use serde::Serialize;
use tauri::Manager;

/// The result of the disk-usage probe: the resolved workspaces directory and its real on-disk
/// byte total. Serialised to the webview so the status bar can show the OS-reported footprint
/// (and label it as such). `exists` is false when the workspaces dir has not been created yet
/// (a fresh install before the first persisted workspace) — then `bytes` is 0, which is the
/// honest answer, not a fabricated one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DiskUsage {
    /// The resolved absolute path of the `$APPLOCALDATA/workspaces` tree the figure covers.
    pub path: String,
    /// The summed real byte size of every regular file under `path` (apparent size, `du --bytes`).
    pub bytes: u64,
    /// Whether the workspaces directory exists yet (false on a fresh install → `bytes` is 0).
    pub exists: bool,
}

/// Recursively sum the real byte size (`metadata.len()`) of every regular file under `root`.
///
/// This is the load-bearing byte-summing walk the [`disk_usage`] command wraps. It is a pure
/// function of the filesystem so it is unit-tested directly (no Tauri runtime needed):
///
///   * a missing `root` sums to `0` (a fresh install before any workspace is persisted);
///   * directory entries recurse; regular files contribute `metadata.len()`;
///   * symlinks are NOT followed (we read the link's own metadata via the directory entry, whose
///     `len()` is the link itself), so the walk cannot escape the tree or loop on a cycle;
///   * an unreadable entry is skipped rather than aborting the whole sum — a partial real total
///     is still honest, and the probe must never panic the command.
///
/// The total is an APPARENT-SIZE sum (the same notion `du --bytes` / `du --apparent-size`
/// reports), not an allocated-blocks figure; for the small JSON workspace files this app writes
/// the two agree in practice, and apparent size is the portable, filesystem-independent number.
pub fn dir_size_bytes(root: &Path) -> u64 {
    // `symlink_metadata` reads the entry itself (does not traverse a symlink target).
    let Ok(meta) = std::fs::symlink_metadata(root) else {
        // Missing path (fresh install) or an unreadable root — honestly contributes nothing.
        return 0;
    };

    if meta.is_file() {
        return meta.len();
    }

    if !meta.is_dir() {
        // A symlink or other special entry at this position: do not follow it (would let the
        // walk escape the tree / loop), and do not count the link node's own size.
        return 0;
    }

    let Ok(entries) = std::fs::read_dir(root) else {
        // Directory we cannot enumerate — skip it rather than abort the whole walk.
        return 0;
    };

    let mut total: u64 = 0;
    for entry in entries.flatten() {
        // `saturating_add` so a pathological tree can never overflow-panic the probe.
        total = total.saturating_add(dir_size_bytes(&entry.path()));
    }
    total
}

/// The native disk-usage probe command: resolve `$APPLOCALDATA/workspaces` and return its REAL
/// on-disk byte total (the recursive [`dir_size_bytes`] sum), the resolved path, and whether the
/// dir exists yet.
///
/// This is the OS-reported counterpart to the webview's snapshot-bytes estimate — the same
/// `<app-local-data>/workspaces` tree the granted `fs` capability scopes (capabilities/
/// default.json) and the `@sparq/client` `TauriWorkspaceStore` persists into. The status bar
/// PREFERS this value in the desktop shell and falls back to the snapshot estimate on the web
/// target (where this command does not exist). It is read-only and panic-free: a missing dir is
/// `bytes: 0, exists: false` (an honest fresh-install answer, not a fabricated figure), and a
/// path-resolution failure is the only `Err` (stringified for the webview's `Result<_, String>`).
#[tauri::command]
pub fn disk_usage(app: tauri::AppHandle) -> Result<DiskUsage, String> {
    // `app_local_data_dir()` is the SAME base dir the `fs` capability + workspace store key off
    // (BaseDirectory::AppLocalData); the probe scopes itself to its `workspaces/` subtree only.
    let base = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("cannot resolve app local data dir: {}", e))?;
    let workspaces = base.join("workspaces");
    let exists = workspaces.is_dir();
    let bytes = dir_size_bytes(&workspaces);
    Ok(DiskUsage {
        path: workspaces.to_string_lossy().into_owned(),
        bytes,
        exists,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// A unique scratch dir under the system temp dir for one test, removed on drop.
    struct Scratch {
        dir: std::path::PathBuf,
    }

    impl Scratch {
        fn new(tag: &str) -> Self {
            let dir = std::env::temp_dir().join(format!(
                "sparq-disk-{}-{}-{:?}",
                tag,
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos())
                    .unwrap_or(0)
            ));
            std::fs::create_dir_all(&dir).expect("mk scratch dir");
            Self { dir }
        }

        fn write(&self, rel: &str, bytes: &[u8]) {
            let path = self.dir.join(rel);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).expect("mk parent");
            }
            let mut f = std::fs::File::create(&path).expect("create file");
            f.write_all(bytes).expect("write file");
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    #[test]
    fn missing_path_sums_to_zero() {
        // A path that does not exist (fresh install, no workspaces yet) is an honest 0, not a panic.
        let missing = std::env::temp_dir().join("sparq-disk-definitely-absent-xyz-123");
        assert_eq!(dir_size_bytes(&missing), 0);
    }

    #[test]
    fn a_single_file_reports_its_real_byte_length() {
        let s = Scratch::new("single");
        s.write("only.json", b"0123456789"); // exactly 10 bytes
        assert_eq!(dir_size_bytes(&s.dir), 10);
    }

    #[test]
    fn sums_every_file_across_nested_directories() {
        // The load-bearing invariant: the walk is RECURSIVE and sums each file's real size.
        let s = Scratch::new("nested");
        s.write("a.json", &[0u8; 100]); // 100
        s.write("b.json", &[0u8; 23]); //  23
        s.write("nested/c.json", &[0u8; 7]); //   7
        s.write("nested/deep/d.json", &[0u8; 1000]); // 1000
        // An empty directory contributes nothing.
        std::fs::create_dir_all(s.dir.join("empty-dir")).expect("mk empty dir");
        assert_eq!(dir_size_bytes(&s.dir), 100 + 23 + 7 + 1000);
    }

    #[test]
    fn an_empty_directory_sums_to_zero() {
        let s = Scratch::new("empty");
        assert_eq!(dir_size_bytes(&s.dir), 0);
    }

    #[cfg(unix)]
    #[test]
    fn a_symlink_is_not_followed_so_the_walk_cannot_escape_the_tree() {
        // Guard the least-privilege intent: a symlink pointing OUT of the workspaces tree must
        // not pull the linked file's bytes into the total (which would also enable a traversal
        // escape). We point a link at a large file outside the scratch dir and assert it is not
        // counted.
        use std::os::unix::fs::symlink;
        let s = Scratch::new("symlink");
        s.write("real.json", &[0u8; 50]); // the only thing that should count: 50 bytes

        let outside = std::env::temp_dir().join(format!(
            "sparq-disk-outside-{}-{:?}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::write(&outside, [0u8; 9999]).expect("write outside file");

        let link = s.dir.join("escape.json");
        symlink(&outside, &link).expect("create symlink");

        // Only `real.json` (50 bytes) counts; the symlink target's 9999 bytes are excluded.
        assert_eq!(dir_size_bytes(&s.dir), 50);

        let _ = std::fs::remove_file(&outside);
    }
}
