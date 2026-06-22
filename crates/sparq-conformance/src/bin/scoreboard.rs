//! [OPUS-4.8] sq-ncvq.16 — `sparq-conformance-scoreboard`: emits the CONSOLIDATED
//! conformance scoreboard (stdout + `conformance-scoreboard.md`).
//!
//! ONE artifact listing every conformance suite the project ratchets — W3C
//! SPARQL, inference, W3C SHACL core, W3C SHACL-SPARQL, and OGC GeoSPARQL — each
//! with its ratchet floor and the CI job that enforces it. The per-suite detail
//! reports are still produced by their own runners (the SPARQL/inference binaries
//! here, the crate-local `cargo test` runners in sparq-shacl / sparq-geo); this
//! binary is the index that pulls them into a single view, closing the
//! drift-scanner's `conformance-split` finding (§5.E).
//!
//! Hermetic + fast: it renders the static central registry
//! ([`sparq_conformance::scoreboard::SUITES`]); it does NOT re-run the suites or
//! need any fetched test data. Always exits 0.
#![forbid(unsafe_code)] // [OPUS-4.8] sq-emay: crate has zero `unsafe`

use sparq_conformance::scoreboard;
use std::path::PathBuf;

fn main() {
    let mut report_path = PathBuf::from("conformance-scoreboard.md");
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--report" => report_path = PathBuf::from(args.next().expect("--report needs a path")),
            other => {
                eprintln!("unknown argument: {other}");
                eprintln!("usage: sparq-conformance-scoreboard [--report FILE]");
                std::process::exit(2);
            }
        }
    }

    let md = scoreboard::render_scoreboard();
    print!("{md}");
    if let Err(e) = std::fs::write(&report_path, &md) {
        eprintln!("could not write {}: {e}", report_path.display());
    } else {
        eprintln!("\nscoreboard written to {}", report_path.display());
    }
}
