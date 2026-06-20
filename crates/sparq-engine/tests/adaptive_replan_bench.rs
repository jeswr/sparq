//! [OPUS-4.8] sq-p6p6 — INDICATIVE micro-benchmark for the local adaptive re-planner.
//!
//! WORK-BOX-INDICATIVE ONLY — this EC2 work box is NON-CANONICAL; the numbers printed here are
//! not canonical evidence and are NOT baked into any doc or test. Run with:
//!
//! ```text
//! cargo test -p sparq-engine --features adaptive-replan-local --release \
//!     --test adaptive_replan_bench -- --ignored --nocapture
//! ```
//!
//! It times the SAME query, executed by the SAME `eval_bgp_binary`, two ways: with the adaptive
//! re-planner ON (the feature default) vs forced OFF (the static baseline), on:
//!
//!   * a MIS-ESTIMATED multi-pattern fixture (a fan-out explosion that makes the static join order
//!     materialise a large intermediate the re-planner avoids) — where the win should appear, and
//!   * a CORRECTLY-ESTIMATED control (a 1:1 chain) — where the re-planner is inert and the two
//!     paths must be within noise (no regression on queries that don't need it).
//!
//! The static baseline is obtained from the SAME feature-on binary via the crate-internal
//! `adaptive` thread-local toggle, exposed for benchmarking through a test-only shim so this is a
//! true apples-to-apples in-process comparison (same code, same data, only the re-plan decision
//! differs).

#![cfg(feature = "adaptive-replan-local")]

use sparq_core::Graph;
use std::time::Instant;

/// A fixture where the re-order genuinely reduces WORK: after a fan-out explosion makes the running
/// `?y` set large, two arms on `?y` remain — `:big` (INFLATES: many objects per y) and `:sel`
/// (PRUNES: only a few y survive). The static estimate ranks `:big` cheaper (few distinct subjects)
/// so it joins `:big` FIRST, materialising a large `?y × :big` intermediate that `:sel` then prunes.
/// The adaptive re-planner observes the explosion and joins the PRUNING `:sel` first, so `:big`
/// processes far fewer rows. The avoided intermediate is the win.
///
/// Shape: `?w :anchor ?x . ?x :fan ?y . ?y :big ?b . ?y :sel ?z`.
fn divergent_graph(scale: usize) -> Graph {
    // The oracle's proven-switching shape, scaled up: a fan-out explosion mis-estimated by the
    // independence model, with two remaining arms whose order the re-planner flips. armA on ?x
    // (estimate-cheap), armB on ?y (estimate-expensive but cheaper once the explosion is observed).
    let mut ttl = String::from("@prefix ex: <http://ex/> .\n");
    let hot = 3;
    let fan = 40 * scale;
    let ay = 20 * scale; // armA objects per hot x
    let by_subj = 10 * scale; // armB distinct subjects
    let by_obj = 60 * scale; // armB objects per subject
    for x in 0..hot {
        ttl.push_str(&format!("ex:w{} ex:anchor ex:hx{} .\n", x, x));
    }
    for x in 0..hot {
        for y in 0..fan {
            ttl.push_str(&format!("ex:hx{} ex:fan ex:y{}_{} .\n", x, x, y));
        }
    }
    for x in 0..(1000 * scale) {
        ttl.push_str(&format!("ex:cx{} ex:fan ex:cy{} .\n", x, x));
    }
    for x in 0..hot {
        for a in 0..ay {
            ttl.push_str(&format!("ex:hx{} ex:armA ex:a{}_{} .\n", x, x, a));
        }
    }
    for y in 0..by_subj {
        for b in 0..by_obj {
            ttl.push_str(&format!("ex:y0_{} ex:armB ex:b{}_{} .\n", y, y, b));
        }
    }
    Graph::load_str(&ttl, "turtle").unwrap()
}

/// A COLLAPSE fixture: a correlated join the independence model thinks is LARGE but is actually
/// tiny (e >> o), so the static plan, expecting a big running set, joins an INFLATING arm before a
/// pruning one; the adaptive plan observes the collapse and reorders. Probes whether the OTHER
/// divergence direction yields a win.
fn collapse_graph(scale: usize) -> Graph {
    let mut ttl = String::from("@prefix ex: <http://ex/> .\n");
    let n = 600 * scale;
    // :u and :v share ?x; the independence estimate of the ?x join is ~ n (LARGE), but only a few x
    // carry BOTH, so the real join COLLAPSES to `overlap` rows.
    for x in 0..n {
        ttl.push_str(&format!("ex:ux{} ex:u ex:y{} .\n", x, x));
    }
    for x in 0..n {
        ttl.push_str(&format!("ex:vx{} ex:v ex:z{} .\n", x, x));
    }
    let overlap = 4 * scale;
    for x in 0..overlap {
        ttl.push_str(&format!("ex:ox{} ex:u ex:oy{} .\n", x, x));
        ttl.push_str(&format!("ex:ox{} ex:v ex:oz{} .\n", x, x));
    }
    // arm on ?y (inflating: many objects) vs arm on ?z (pruning).
    for y in 0..n {
        for k in 0..(4 * scale) {
            ttl.push_str(&format!("ex:y{} ex:armY ex:yy{}_{} .\n", y, y, k));
        }
    }
    for x in 0..overlap {
        for k in 0..(4 * scale) {
            ttl.push_str(&format!("ex:oy{} ex:armY ex:oyy{}_{} .\n", x, x, k));
        }
    }
    ttl.push_str("ex:oz0 ex:armZ ex:zz0 .\n");
    for c in 0..(4000 * scale) {
        ttl.push_str(&format!("ex:czz{} ex:armZ ex:ccz{} .\n", c, c));
    }
    Graph::load_str(&ttl, "turtle").unwrap()
}

/// A correctly-estimated 1:1 chain (the control): the re-planner never triggers.
fn chain_graph(n: usize) -> Graph {
    let mut ttl = String::from("@prefix ex: <http://ex/> .\n");
    for i in 0..n {
        ttl.push_str(&format!("ex:s{} ex:a ex:o{} .\n", i, i));
        ttl.push_str(&format!("ex:o{} ex:b ex:p{} .\n", i, i));
        ttl.push_str(&format!("ex:p{} ex:c ex:z{} .\n", i, i));
    }
    Graph::load_str(&ttl, "turtle").unwrap()
}

fn time_query(g: &Graph, q: &str, adaptive: bool, iters: usize) -> (u128, usize) {
    let run = || {
        let mut rows = 0usize;
        for _ in 0..iters {
            let r = sparq_engine::query(g, q).expect("query");
            rows = r.rows.len();
        }
        rows
    };
    let t0 = Instant::now();
    let rows = if adaptive {
        run()
    } else {
        sparq_engine::adaptive_bench::with_static_plan(run)
    };
    (t0.elapsed().as_micros(), rows)
}

#[test]
#[ignore = "indicative micro-benchmark; run explicitly with --ignored --nocapture (work-box numbers are NON-CANONICAL)"]
fn replan_bench() {
    let scale = 2;
    let iters = 20;

    // --- Mis-estimated case (where the re-planner should help) ---
    let g = divergent_graph(scale);
    let q = "PREFIX ex: <http://ex/> SELECT * WHERE { \
        ?w ex:anchor ?x . ?x ex:fan ?y . ?x ex:armA ?a . ?y ex:armB ?b }";
    // Warm up (dictionary / index caches) and report whether the re-planner switched here.
    let (_, switches) = sparq_engine::adaptive_bench::count_switches(|| sparq_engine::query(&g, q).unwrap());
    let (us_static, rows_s) = time_query(&g, q, false, iters);
    let (us_adaptive, rows_a) = time_query(&g, q, true, iters);
    assert_eq!(rows_s, rows_a, "static and adaptive must return the same row count");
    println!(
        "[INDICATIVE / NON-CANONICAL] mis-estimated fan-out ({} rows, {} iters, {} switches): static={}us adaptive={}us  ratio={:.2}x",
        rows_s,
        iters,
        switches,
        us_static,
        us_adaptive,
        us_static as f64 / us_adaptive.max(1) as f64
    );

    // --- Collapse case (the other divergence direction) ---
    let gco = collapse_graph(scale);
    let qco = "PREFIX ex: <http://ex/> SELECT * WHERE { \
        ?x ex:u ?y . ?x ex:v ?z . ?y ex:armY ?yy . ?z ex:armZ ?zz }";
    let (_, sw_co) = sparq_engine::adaptive_bench::count_switches(|| sparq_engine::query(&gco, qco).unwrap());
    let (cob_s, cob_rs) = time_query(&gco, qco, false, iters);
    let (cob_a, cob_ra) = time_query(&gco, qco, true, iters);
    assert_eq!(cob_rs, cob_ra);
    println!(
        "[INDICATIVE / NON-CANONICAL] collapse ({} rows, {} iters, {} switches): static={}us adaptive={}us  ratio={:.2}x",
        cob_rs, iters, sw_co, cob_s, cob_a, cob_s as f64 / cob_a.max(1) as f64
    );

    // --- Correctly-estimated control (where the re-planner must be inert) ---
    let gc = chain_graph(20_000);
    let qc = "PREFIX ex: <http://ex/> SELECT * WHERE { ?s ex:a ?o . ?o ex:b ?p . ?p ex:c ?z }";
    let _ = sparq_engine::query(&gc, qc).unwrap();
    let (cs, crows_s) = time_query(&gc, qc, false, iters);
    let (ca, crows_a) = time_query(&gc, qc, true, iters);
    assert_eq!(crows_s, crows_a);
    println!(
        "[INDICATIVE / NON-CANONICAL] correctly-estimated chain control ({} rows, {} iters): static={}us adaptive={}us  ratio={:.2}x (must be ~1.0)",
        crows_s, iters, cs, ca, cs as f64 / ca.max(1) as f64
    );
}
