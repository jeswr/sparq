//! RESEARCH SPIKE — page-cache reality check for mmap'd permutations.
//!
//! Questions answered against one raw perm file:
//!  1. Is an UNTOUCHED mapping resident? (demand paging — "drop cold" for free)
//!  2. After touching every page, what is resident?
//!  3. Does madvise(MADV_DONTNEED) actually evict, and what does re-touch cost
//!     (the re-introduction latency for an explicitly dropped mmap'd perm)?
//!
//! usage: madvise <perm-file>

use std::os::unix::io::AsRawFd;
use std::time::Instant;

unsafe fn resident_pages(addr: *mut libc::c_void, len: usize, page: usize) -> usize {
    let n_pages = (len + page - 1) / page;
    let mut vec = vec![0u8; n_pages];
    #[cfg(target_os = "macos")]
    let rc = libc::mincore(addr, len, vec.as_mut_ptr() as *mut libc::c_char);
    #[cfg(not(target_os = "macos"))]
    let rc = libc::mincore(addr, len, vec.as_mut_ptr());
    assert_eq!(rc, 0, "mincore failed");
    vec.iter().filter(|&&b| b & 1 != 0).count()
}

fn main() {
    let path = std::env::args().nth(1).expect("usage: madvise <perm-file>");
    let f = std::fs::File::open(&path).unwrap();
    let len = f.metadata().unwrap().len() as usize;
    let page = unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as usize;
    let n_pages = (len + page - 1) / page;
    println!("file\t{path}\t{len} bytes\t{n_pages} pages of {page}");

    unsafe {
        let addr = libc::mmap(std::ptr::null_mut(), len, libc::PROT_READ, libc::MAP_SHARED, f.as_raw_fd(), 0);
        assert_ne!(addr, libc::MAP_FAILED);

        // 1. untouched
        let r0 = resident_pages(addr, len, page);
        println!("resident_untouched\t{r0}\t({:.1}%)", 100.0 * r0 as f64 / n_pages as f64);

        // 2. touch every page (sum one byte per page), timed = first-touch page-in cost
        let t = Instant::now();
        let mut acc = 0u64;
        let p = addr as *const u8;
        let mut off = 0;
        while off < len {
            acc += *p.add(off) as u64;
            off += page;
        }
        std::hint::black_box(acc);
        let touch1 = t.elapsed().as_secs_f64();
        let r1 = resident_pages(addr, len, page);
        println!("first_touch_all_s\t{touch1:.3}\tresident\t{r1}\t({:.1}%)", 100.0 * r1 as f64 / n_pages as f64);

        // warm re-touch for comparison
        let t = Instant::now();
        let mut acc = 0u64;
        let mut off = 0;
        while off < len {
            acc += *p.add(off) as u64;
            off += page;
        }
        std::hint::black_box(acc);
        println!("warm_touch_all_s\t{:.3}", t.elapsed().as_secs_f64());

        // 3. MADV_DONTNEED, then re-touch (the re-introduction cost after an explicit drop)
        let rc = libc::madvise(addr, len, libc::MADV_DONTNEED);
        println!("madvise_dontneed_rc\t{rc}");
        let r2 = resident_pages(addr, len, page);
        println!("resident_after_dontneed\t{r2}\t({:.1}%)", 100.0 * r2 as f64 / n_pages as f64);

        let t = Instant::now();
        let mut acc = 0u64;
        let mut off = 0;
        while off < len {
            acc += *p.add(off) as u64;
            off += page;
        }
        std::hint::black_box(acc);
        let retouch = t.elapsed().as_secs_f64();
        let r3 = resident_pages(addr, len, page);
        println!("retouch_after_dontneed_s\t{retouch:.3}\tresident\t{r3}\t({:.1}%)", 100.0 * r3 as f64 / n_pages as f64);

        libc::munmap(addr, len);
    }
}
