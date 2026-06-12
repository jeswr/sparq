//! RESEARCH SPIKE — build a MIXED-FORMAT index dir: selected permutations re-encoded
//! block-compressed, the rest symlinked raw. `Graph::open` auto-detects the format per
//! perm file, so this exercises the per-permutation hot/cold tiering that the existing
//! storage layer already supports — with zero crate changes.
//!
//! usage: mix <src-raw-dir> <dst-dir> <comma-list of perm indices to compress, e.g. 1,2,5>

use sparq_core::compress::CompressedPerm;
use sparq_core::dict::Id;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (src, dst, list) = match (args.get(1), args.get(2), args.get(3)) {
        (Some(a), Some(b), Some(c)) => (
            std::fs::canonicalize(a).expect("src dir"),
            std::path::PathBuf::from(b),
            c.clone(),
        ),
        _ => {
            eprintln!("usage: mix <src-raw-dir> <dst-dir> <perm-indices-to-compress e.g. 1,2,5>");
            std::process::exit(2);
        }
    };
    let compress: Vec<usize> = if list == "none" {
        Vec::new()
    } else {
        list.split(',').map(|s| s.parse().expect("perm index")).collect()
    };
    std::fs::create_dir_all(&dst).unwrap();

    for entry in std::fs::read_dir(&src).unwrap().filter_map(Result::ok) {
        let name = entry.file_name().to_string_lossy().to_string();
        let to = dst.join(&name);
        if to.exists() {
            std::fs::remove_file(&to).unwrap();
        }
        let perm_idx = name
            .strip_prefix("perm")
            .and_then(|s| s.strip_suffix(".bin"))
            .and_then(|s| s.parse::<usize>().ok());
        match perm_idx {
            Some(i) if compress.contains(&i) => {
                let bytes = std::fs::read(entry.path()).unwrap();
                assert!(
                    bytes.len() < 8 || bytes[..8] != sparq_core::compress::FILE_MAGIC,
                    "{name} in src is already compressed; mix wants a raw src dir"
                );
                assert_eq!(bytes.len() % 12, 0, "{name}: not a whole number of [u32;3] rows");
                let mut rows: Vec<[Id; 3]> = Vec::with_capacity(bytes.len() / 12);
                for c in bytes.chunks_exact(12) {
                    rows.push([
                        u32::from_le_bytes(c[0..4].try_into().unwrap()),
                        u32::from_le_bytes(c[4..8].try_into().unwrap()),
                        u32::from_le_bytes(c[8..12].try_into().unwrap()),
                    ]);
                }
                let mut w = std::io::BufWriter::new(std::fs::File::create(&to).unwrap());
                CompressedPerm::encode(&rows).write_to(&mut w).unwrap();
                std::io::Write::flush(&mut w).unwrap();
                let new_len = std::fs::metadata(&to).unwrap().len();
                eprintln!("compressed {name}: {} -> {} bytes ({:.2}x)", bytes.len(), new_len, bytes.len() as f64 / new_len as f64);
            }
            _ => {
                // Everything else (raw perms, dict, numerics, predstats): symlink, so a
                // mixed dir costs only the re-encoded files' disk.
                std::os::unix::fs::symlink(entry.path(), &to).unwrap();
            }
        }
    }
    eprintln!("mixed dir written to {}", dst.display());
}
