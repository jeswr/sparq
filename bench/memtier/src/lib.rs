//! Shared helpers for the memory-tiering spikes: RSS sampling, per-file page-cache
//! residency via mincore(2), and the query-battery runner.

use std::time::Instant;

/// Current resident set size of this process in KiB (via `ps`, macOS + Linux).
pub fn rss_kb() -> u64 {
    let out = std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()])
        .output()
        .expect("ps");
    String::from_utf8_lossy(&out.stdout).trim().parse().unwrap_or(0)
}

/// Page-cache residency of a file: (resident_pages, total_pages, page_size).
///
/// Maps the file read-only WITHOUT touching it and asks mincore(2). The pages are
/// shared with any other mapping of the same file in this process (the store's own
/// mmap), so this reflects what the battery actually paged in. Technique validated
/// by the differential between battery-touched and untouched permutation files.
pub fn file_residency(path: &std::path::Path) -> std::io::Result<(usize, usize)> {
    use std::os::unix::io::AsRawFd;
    let f = std::fs::File::open(path)?;
    let len = f.metadata()?.len() as usize;
    if len == 0 {
        return Ok((0, 0));
    }
    let page = unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as usize;
    let n_pages = (len + page - 1) / page;
    unsafe {
        let addr = libc::mmap(std::ptr::null_mut(), len, libc::PROT_READ, libc::MAP_SHARED, f.as_raw_fd(), 0);
        if addr == libc::MAP_FAILED {
            return Err(std::io::Error::last_os_error());
        }
        let mut vec = vec![0u8; n_pages];
        #[cfg(target_os = "macos")]
        let rc = libc::mincore(addr, len, vec.as_mut_ptr() as *mut libc::c_char);
        #[cfg(not(target_os = "macos"))]
        let rc = libc::mincore(addr, len, vec.as_mut_ptr());
        let resident = if rc == 0 { vec.iter().filter(|&&b| b & 1 != 0).count() } else { usize::MAX };
        libc::munmap(addr, len);
        if rc != 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok((resident, n_pages))
    }
}

/// Residency histogram over `buckets` equal byte ranges of the file — shows WHERE in a
/// permutation the battery's working set lives (the per-predicate-range tiering signal).
pub fn residency_histogram(path: &std::path::Path, buckets: usize) -> std::io::Result<Vec<f64>> {
    use std::os::unix::io::AsRawFd;
    let f = std::fs::File::open(path)?;
    let len = f.metadata()?.len() as usize;
    if len == 0 {
        return Ok(vec![0.0; buckets]);
    }
    let page = unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as usize;
    let n_pages = (len + page - 1) / page;
    unsafe {
        let addr = libc::mmap(std::ptr::null_mut(), len, libc::PROT_READ, libc::MAP_SHARED, f.as_raw_fd(), 0);
        if addr == libc::MAP_FAILED {
            return Err(std::io::Error::last_os_error());
        }
        let mut vec = vec![0u8; n_pages];
        #[cfg(target_os = "macos")]
        let rc = libc::mincore(addr, len, vec.as_mut_ptr() as *mut libc::c_char);
        #[cfg(not(target_os = "macos"))]
        let rc = libc::mincore(addr, len, vec.as_mut_ptr());
        libc::munmap(addr, len);
        if rc != 0 {
            return Err(std::io::Error::last_os_error());
        }
        let per = (n_pages + buckets - 1) / buckets.max(1);
        Ok(vec
            .chunks(per.max(1))
            .map(|c| c.iter().filter(|&&b| b & 1 != 0).count() as f64 / c.len() as f64)
            .collect())
    }
}

/// One query's battery result.
pub struct QueryResultLine {
    pub name: String,
    pub rows: Option<usize>,
    pub min_us: f64,
    pub median_us: f64,
    pub max_us: f64,
    pub err: Option<String>,
}

/// Runs every `*.rq` in `dir` (sorted) `iters` times in count mode; per query reports
/// min / median / max latency in microseconds (min is the contention-robust statistic
/// on a loaded machine; median+max expose the variance honestly).
pub fn run_suite(g: &sparq_core::Graph, dir: &str, iters: usize) -> Vec<QueryResultLine> {
    run_suite_mode(g, dir, iters, "count")
}

/// Like [`run_suite`] but with an execution mode: `count` (engine count fast paths) or
/// `json` (materialize + serialize the full SPARQL-JSON result — the realistic serving
/// profile, which exercises the dictionary on output).
pub fn run_suite_mode(g: &sparq_core::Graph, dir: &str, iters: usize, mode: &str) -> Vec<QueryResultLine> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("error reading {dir}: {e}"))
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "rq").unwrap_or(false))
        .collect();
    entries.sort();

    let mut out = Vec::new();
    for path in entries {
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let sparql = std::fs::read_to_string(&path).unwrap();
        let mut times = Vec::with_capacity(iters);
        let mut rows = None;
        let mut err = None;
        for _ in 0..iters {
            let t = Instant::now();
            let res = match mode {
                "json" => sparq_engine::query_json(g, &sparql).map(|s| {
                    let n = s.len();
                    std::hint::black_box(s);
                    n
                }),
                _ => sparq_engine::count(g, &sparql),
            };
            match res {
                Ok(n) => {
                    rows = Some(n);
                    times.push(t.elapsed().as_secs_f64() * 1e6);
                }
                Err(e) => {
                    err = Some(e);
                    break;
                }
            }
        }
        times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let (min_us, median_us, max_us) = if times.is_empty() {
            (f64::NAN, f64::NAN, f64::NAN)
        } else {
            (times[0], times[times.len() / 2], times[times.len() - 1])
        };
        out.push(QueryResultLine { name, rows, min_us, median_us, max_us, err });
    }
    out
}
