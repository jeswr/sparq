//! RESEARCH SPIKE — tiny HTTP/1.1 load generator for the sparq-server baseline
//! measurements in research/concurrent-serving.md.
//!
//! Pure std (threads + blocking sockets, keep-alive). Two arrival models:
//!
//! * **closed-loop** (`--rate 0`): N connections firing back-to-back — measures the
//!   server's saturation throughput; latencies are only meaningful relative to the
//!   concurrency level (classic closed-loop caveat).
//! * **open-loop** (`--rate R`): Poisson arrivals at R req/s split across the
//!   connections; latency is measured from the request's *scheduled* arrival time,
//!   not the actual socket write — i.e. coordinated-omission-safe: if the server
//!   stalls, the queueing delay the stall causes is charged to every delayed
//!   request.
//!
//! `--slow QUERY --slow-every MS` injects an expensive query on a dedicated extra
//! connection every MS milliseconds (the head-of-line demonstration).
//!
//! Output: one summary line per run — req/s, status counts, p50/p90/p99/p999/max
//! in microseconds. Not a benchmark framework; just enough to calibrate the
//! design document honestly.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

fn main() {
    let mut addr = "127.0.0.1:3030".to_string();
    let mut conns = 8usize;
    let mut duration = 10u64;
    let mut rate = 0u64; // 0 = closed loop
    let mut query = "SELECT * WHERE { ?s ?p ?o } LIMIT 1".to_string();
    let mut slow: Option<String> = None;
    let mut slow_every_ms = 1000u64;
    let mut label = "run".to_string();

    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        let mut next = |flag: &str| args.next().unwrap_or_else(|| panic!("{flag} needs a value"));
        match a.as_str() {
            "--addr" => addr = next("--addr"),
            "--conns" => conns = next("--conns").parse().unwrap(),
            "--duration" => duration = next("--duration").parse().unwrap(),
            "--rate" => rate = next("--rate").parse().unwrap(),
            "--query" => query = next("--query"),
            "--query-file" => query = std::fs::read_to_string(next("--query-file")).unwrap(),
            "--slow" => slow = Some(next("--slow")),
            "--slow-file" => slow = Some(std::fs::read_to_string(next("--slow-file")).unwrap()),
            "--slow-every" => slow_every_ms = next("--slow-every").parse().unwrap(),
            "--label" => label = next("--label"),
            other => panic!("unknown arg {other}"),
        }
    }

    let req = request_bytes(&addr, &query);
    let stop = Arc::new(AtomicBool::new(false));
    let errors = Arc::new(AtomicU64::new(0));

    // Optional head-of-line injector: a dedicated connection firing the slow query.
    let slow_handle = slow.map(|sq| {
        let req = request_bytes(&addr, &sq);
        let addr = addr.clone();
        let stop = stop.clone();
        std::thread::spawn(move || {
            let mut lat = Vec::new();
            let mut conn = TcpStream::connect(&addr).expect("slow conn");
            conn.set_nodelay(true).unwrap();
            while !stop.load(Ordering::Relaxed) {
                let t = Instant::now();
                if do_request(&mut conn, &req).is_none() {
                    conn = TcpStream::connect(&addr).expect("slow reconnect");
                    continue;
                }
                lat.push(t.elapsed().as_micros() as u64);
                std::thread::sleep(Duration::from_millis(slow_every_ms));
            }
            lat
        })
    });

    let start = Instant::now();
    let deadline = start + Duration::from_secs(duration);
    let per_conn_rate = if rate > 0 { rate as f64 / conns as f64 } else { 0.0 };

    let workers: Vec<_> = (0..conns)
        .map(|i| {
            let addr = addr.clone();
            let req = req.clone();
            let stop = stop.clone();
            let errors = errors.clone();
            std::thread::spawn(move || {
                let mut lat: Vec<u64> = Vec::with_capacity(1 << 20);
                let mut ok = 0u64;
                let mut statuses = [0u64; 6]; // 1xx..5xx + other
                let mut conn = TcpStream::connect(&addr).expect("connect");
                conn.set_nodelay(true).unwrap();
                let mut rng = 0x9e3779b97f4a7c15u64 ^ (i as u64).wrapping_mul(0xa076_1d64_78bd_642f);
                // Open loop: each request has a scheduled time; latency is measured
                // from schedule, so server stalls are charged (no coordinated omission).
                let mut next_at = Instant::now();
                loop {
                    if stop.load(Ordering::Relaxed) || Instant::now() >= deadline {
                        break;
                    }
                    if per_conn_rate > 0.0 {
                        // Exponential inter-arrival (Poisson process).
                        rng ^= rng << 13;
                        rng ^= rng >> 7;
                        rng ^= rng << 17;
                        let u = (rng >> 11) as f64 / (1u64 << 53) as f64;
                        let gap = -u.max(1e-12).ln() / per_conn_rate;
                        next_at += Duration::from_secs_f64(gap);
                        let now = Instant::now();
                        if next_at > now {
                            std::thread::sleep(next_at - now);
                        }
                    } else {
                        next_at = Instant::now();
                    }
                    match do_request(&mut conn, &req) {
                        Some(status) => {
                            let bucket = ((status / 100).clamp(1, 5) as usize) - 1;
                            statuses[bucket] += 1;
                            if status == 200 {
                                ok += 1;
                            }
                            lat.push(next_at.elapsed().as_micros() as u64);
                        }
                        None => {
                            errors.fetch_add(1, Ordering::Relaxed);
                            match TcpStream::connect(&addr) {
                                Ok(c) => {
                                    conn = c;
                                    conn.set_nodelay(true).unwrap();
                                }
                                Err(_) => break,
                            }
                        }
                    }
                }
                (lat, ok, statuses)
            })
        })
        .collect();

    let mut all = Vec::new();
    let mut ok = 0u64;
    let mut statuses = [0u64; 6];
    for w in workers {
        let (l, o, s) = w.join().unwrap();
        all.extend(l);
        ok += o;
        for (a, b) in statuses.iter_mut().zip(s) {
            *a += b;
        }
    }
    let elapsed = start.elapsed().as_secs_f64();
    stop.store(true, Ordering::Relaxed);
    let slow_lat = slow_handle.map(|h| h.join().unwrap());

    all.sort_unstable();
    let pct = |p: f64| -> u64 {
        if all.is_empty() {
            return 0;
        }
        all[((all.len() as f64 * p) as usize).min(all.len() - 1)]
    };
    println!(
        "[{label}] mode={} conns={conns} dur={elapsed:.1}s total={} ok={ok} statuses(1-5xx)={statuses:?} io-errs={} thr={:.0} req/s  p50={}us p90={}us p99={}us p999={}us max={}us",
        if rate > 0 { format!("open@{rate}rps") } else { "closed".into() },
        all.len(),
        errors.load(Ordering::Relaxed),
        all.len() as f64 / elapsed,
        pct(0.50),
        pct(0.90),
        pct(0.99),
        pct(0.999),
        all.last().copied().unwrap_or(0),
    );
    if let Some(sl) = slow_lat {
        let n = sl.len();
        let mean = if n > 0 { sl.iter().sum::<u64>() / n as u64 } else { 0 };
        println!("[{label}] slow-query injections={n} mean={mean}us");
    }
}

/// Builds a keep-alive GET /sparql?query=... request.
fn request_bytes(addr: &str, query: &str) -> Vec<u8> {
    let mut enc = String::new();
    for b in query.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => enc.push(b as char),
            b' ' => enc.push('+'),
            _ => enc.push_str(&format!("%{b:02X}")),
        }
    }
    format!(
        "GET /sparql?query={enc} HTTP/1.1\r\nHost: {addr}\r\nAccept: application/sparql-results+json\r\nConnection: keep-alive\r\n\r\n"
    )
    .into_bytes()
}

/// Writes one request and reads exactly one response (Content-Length framing —
/// sparq-server always sets it). Returns the status code, or None on socket error.
fn do_request(conn: &mut TcpStream, req: &[u8]) -> Option<u16> {
    conn.write_all(req).ok()?;
    let mut buf = Vec::with_capacity(4096);
    let mut tmp = [0u8; 16384];
    // Read until end of headers.
    let header_end = loop {
        if let Some(pos) = find_subslice(&buf, b"\r\n\r\n") {
            break pos + 4;
        }
        let n = conn.read(&mut tmp).ok()?;
        if n == 0 {
            return None;
        }
        buf.extend_from_slice(&tmp[..n]);
    };
    let headers = std::str::from_utf8(&buf[..header_end]).ok()?;
    let status: u16 = headers.split_whitespace().nth(1)?.parse().ok()?;
    let clen: usize = headers
        .lines()
        .find_map(|l| {
            let (k, v) = l.split_once(':')?;
            k.eq_ignore_ascii_case("content-length").then(|| v.trim().parse().ok())?
        })
        .unwrap_or(0);
    let mut have = buf.len() - header_end;
    while have < clen {
        let n = conn.read(&mut tmp).ok()?;
        if n == 0 {
            return None;
        }
        have += n;
    }
    Some(status)
}

fn find_subslice(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}
