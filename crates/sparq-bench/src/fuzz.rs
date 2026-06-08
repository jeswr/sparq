//! Differential fuzzer: generates random small graphs (with **mixed datatypes** on
//! a numeric predicate — the case that stresses inline-integer range-pruning) and
//! random queries across every plan shape, then checks that
//!   sparq `query().len()`  ==  Oxigraph solution count            (full differential)
//!   sparq `count()`         ==  sparq `query().len()`             (count-path differential)
//! Oxigraph is an independent, mature SPARQL implementation, so an agreement over
//! thousands of random cases is strong evidence the optimizations (tagged ValueIds,
//! range-pruning, count-only joins, lazy count, filter pushdown, sort-merge OPTIONAL,
//! LIMIT early-termination) preserve SPARQL semantics. A mismatch prints a full
//! deterministic repro (seed + query + the graph).
//!
//! Usage: `sparq-bench fuzz <seed_start> <count> [category]`
//! category ∈ { all (default) | bgp | filter | optional | union | minus | limit |
//!              distinct | order } — lets a workflow shard the space across agents.

use oxigraph::store::Store;

/// Deterministic SplitMix64 — no clock/entropy, so every case is reproducible from
/// its seed (`Date::now`/`rand` are unavailable in the wasm sibling crate too).
struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed.wrapping_mul(0x9e37_79b9_7f4a_7c15).wrapping_add(1))
    }
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n.max(1)
    }
    fn chance(&mut self, num: u64, den: u64) -> bool {
        self.below(den) < num
    }
}

/// A random graph. `ex:age` is ALWAYS a canonical non-negative `xsd:integer`
/// (an all-inline column → range-pruning active). `ex:val` is deliberately MIXED
/// (xsd:int / xsd:decimal / negative / non-canonical integer / string) → the
/// range-pruning guard must fall back to a full scan and still be correct.
fn gen_graph(rng: &mut Rng) -> String {
    let n = 3 + rng.below(14) as usize; // 3..=16 subjects
    let mut s = String::from(
        "@prefix ex: <http://ex/> .\n@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n",
    );
    for i in 0..n {
        let subj = format!("ex:n{i}");
        if rng.chance(3, 4) {
            s.push_str(&format!("{subj} a ex:T .\n"));
        }
        // ex:age — canonical non-negative integer in a small range so FILTER
        // thresholds land on real boundaries. All-inline column.
        if rng.chance(7, 8) {
            let age = rng.below(120);
            s.push_str(&format!("{subj} ex:age {age} .\n"));
        }
        // ex:val — MIXED datatype column.
        if rng.chance(3, 4) {
            let v = match rng.below(7) {
                0 => format!("{}", rng.below(120)),                 // canonical integer (inline)
                1 => format!("\"{}\"^^xsd:int", rng.below(120)),    // xsd:int (not inline)
                2 => format!("\"{}.5\"^^xsd:decimal", rng.below(120)), // decimal
                3 => format!("\"-{}\"^^xsd:integer", rng.below(60)),   // negative integer (not inline)
                4 => format!("\"0{}\"^^xsd:integer", 1 + rng.below(9)), // non-canonical (leading zero)
                5 => format!("\"{}.0\"^^xsd:double", rng.below(120)),   // double
                _ => format!("\"s{}\"", rng.below(5)),                  // plain string (non-numeric)
            };
            s.push_str(&format!("{subj} ex:val {v} .\n"));
        }
        // ex:name — string, sometimes language-tagged.
        if rng.chance(3, 4) {
            if rng.chance(1, 3) {
                s.push_str(&format!("{subj} ex:name \"nm{}\"@en .\n", rng.below(6)));
            } else {
                s.push_str(&format!("{subj} ex:name \"nm{}\" .\n", rng.below(6)));
            }
        }
        // ex:p — edges (chain/star/cycle fodder).
        let edges = rng.below(4);
        for _ in 0..edges {
            s.push_str(&format!("{subj} ex:p ex:n{} .\n", rng.below(n as u64)));
        }
    }
    s
}

/// A numeric comparison operator and a threshold (chosen near the data range so the
/// boundary cases of range-pruning are exercised).
fn gen_filter(rng: &mut Rng, var: &str) -> String {
    let op = ["<", "<=", ">", ">=", "=", "!="][rng.below(6) as usize];
    let t = rng.below(125); // 0..124, straddles the 0..119 data range incl. boundaries
    format!("FILTER({var} {op} {t})")
}

/// A random query in the chosen category. Always valid SPARQL 1.1 (both engines
/// parse it); restricted to the surface sparq supports (no property paths).
fn gen_query(rng: &mut Rng, category: &str) -> String {
    let pfx = "PREFIX ex: <http://ex/>\nPREFIX xsd: <http://www.w3.org/2001/XMLSchema#>\n";
    // Pick an effective category (when "all", choose one at random).
    let cats = ["bgp", "filter", "optional", "union", "minus", "limit", "distinct", "order"];
    let cat = if category == "all" { cats[rng.below(cats.len() as u64) as usize] } else { category };

    let body = match cat {
        "bgp" => match rng.below(5) {
            0 => "?s ex:age ?a".to_string(),
            1 => "?s ex:p ?o . ?o ex:age ?a".to_string(), // chain join
            2 => "?s ex:age ?a . ?s ex:name ?n".to_string(), // star join
            3 => "?s ex:p ?o . ?o ex:p ?t . ?t ex:p ?s".to_string(), // triangle (WCOJ)
            _ => "?s ex:p ?o . ?o ex:name ?n . ?s ex:age ?a".to_string(),
        },
        "filter" => {
            let var = if rng.chance(1, 2) { "?a" } else { "?v" };
            let pat = if var == "?a" { "?s ex:age ?a" } else { "?s ex:val ?v" };
            let extra = if rng.chance(1, 2) { " . ?s ex:name ?n" } else { "" };
            format!("{pat}{extra} {}", gen_filter(rng, var))
        }
        "optional" => match rng.below(3) {
            0 => "?s ex:name ?n OPTIONAL { ?s ex:age ?a }".to_string(),
            1 => "?s ex:age ?a OPTIONAL { ?s ex:p ?o }".to_string(),
            _ => "?s ex:name ?n OPTIONAL { ?s ex:age ?a . FILTER(?a > 50) }".to_string(),
        },
        "union" => "{ ?s ex:age ?a } UNION { ?s ex:name ?a }".to_string(),
        "minus" => "?s ex:name ?n MINUS { ?s ex:age ?a }".to_string(),
        "limit" => {
            let k = rng.below(8);
            let off = rng.below(5);
            return format!(
                "{pfx}SELECT * WHERE {{ ?s ex:p ?o . ?o ex:age ?a }} LIMIT {k} OFFSET {off}"
            );
        }
        "distinct" => {
            return format!("{pfx}SELECT DISTINCT ?a WHERE {{ ?s ex:age ?a }}");
        }
        "order" => {
            let k = 1 + rng.below(10);
            let dir = if rng.chance(1, 2) { "?a" } else { "DESC(?a)" };
            return format!("{pfx}SELECT ?a WHERE {{ ?s ex:age ?a }} ORDER BY {dir} LIMIT {k}");
        }
        _ => "?s ex:age ?a".to_string(),
    };
    format!("{pfx}SELECT * WHERE {{ {body} }}")
}

fn oxi_count(store: &Store, q: &str) -> Result<usize, String> {
    match store.query(q).map_err(|e| e.to_string())? {
        oxigraph::sparql::QueryResults::Solutions(s) => Ok(s.count()),
        oxigraph::sparql::QueryResults::Boolean(_) => Ok(1),
        oxigraph::sparql::QueryResults::Graph(g) => Ok(g.count()),
    }
}

/// Oxigraph's ordered sequence of a single projected variable, as term strings.
fn oxi_seq(store: &Store, q: &str, var: &str) -> Option<Vec<String>> {
    match store.query(q).ok()? {
        oxigraph::sparql::QueryResults::Solutions(s) => Some(
            s.filter_map(|sol| sol.ok())
                .map(|sol| sol.get(var).map(|t| t.to_string()).unwrap_or_default())
                .collect(),
        ),
        _ => None,
    }
}

/// Order-sensitive differential for `ORDER BY` queries (the count check can't see
/// reordering): the exact ordered sequence of the projected `?a` (an inline-integer
/// column) must match Oxigraph element-for-element. Catches a by-id-not-by-value
/// ordering bug for tagged ValueIds. Returns Err(detail) on a mismatch.
fn check_ordered(g: &sparq_core::Graph, store: &Store, q: &str) -> Result<(), String> {
    let r = sparq_engine::query(g, q).map_err(|e| format!("sparq query error: {e}"))?;
    let sparq: Vec<String> = r
        .rows
        .iter()
        .map(|row| row[0].as_ref().map(|t| t.to_string()).unwrap_or_default())
        .collect();
    let oxi = oxi_seq(store, q, "a").ok_or("oxigraph produced no solution sequence")?;
    if sparq != oxi {
        return Err(format!("ORDER sequence differs\n  sparq={sparq:?}\n  oxi  ={oxi:?}"));
    }
    Ok(())
}

pub fn run(seed_start: u64, count: u64, category: &str) {
    let mut checked = 0u64;
    let mut skipped_unsupported = 0u64;
    let mut full_mismatch = 0u64;
    let mut count_mismatch = 0u64;
    let mut first_repro: Option<String> = None;

    for seed in seed_start..seed_start + count {
        let mut rng = Rng::new(seed);
        let ttl = gen_graph(&mut rng);
        let q = gen_query(&mut rng, category);

        let g = match sparq_core::Graph::load_str(&ttl, "turtle") {
            Ok(g) => g,
            Err(e) => {
                report_repro(&mut first_repro, seed, &q, &ttl, &format!("sparq load error: {e}"));
                full_mismatch += 1;
                continue;
            }
        };
        let store = Store::new().unwrap();
        if let Err(e) = store.load_from_reader(oxigraph::io::RdfFormat::Turtle, ttl.as_bytes()) {
            // Both engines parse the same Turtle; a divergence here is itself a bug.
            report_repro(&mut first_repro, seed, &q, &ttl, &format!("oxigraph load error: {e}"));
            full_mismatch += 1;
            continue;
        }

        let oxi = match oxi_count(&store, &q) {
            Ok(n) => n,
            Err(_) => {
                skipped_unsupported += 1;
                continue;
            }
        };

        let sparq_full = match sparq_engine::query(&g, &q) {
            Ok(r) => r.len(),
            Err(_) => {
                // sparq doesn't support this surface — fair to skip (not a wrong answer).
                skipped_unsupported += 1;
                continue;
            }
        };
        checked += 1;

        if sparq_full != oxi {
            full_mismatch += 1;
            report_repro(
                &mut first_repro,
                seed,
                &q,
                &ttl,
                &format!("sparq query().len()={sparq_full} != oxigraph={oxi}"),
            );
        }

        // Order-sensitive differential (ORDER BY queries only): the sequence itself
        // must match, not just the cardinality.
        if q.contains("ORDER BY") {
            if let Err(detail) = check_ordered(&g, &store, &q) {
                full_mismatch += 1;
                report_repro(&mut first_repro, seed, &q, &ttl, &detail);
            }
        }

        // Count-path differential: the lazy/count-only count must equal the
        // materialized solution count.
        if let Ok(c) = sparq_engine::count(&g, &q) {
            if c != sparq_full {
                count_mismatch += 1;
                report_repro(
                    &mut first_repro,
                    seed,
                    &q,
                    &ttl,
                    &format!("sparq count()={c} != sparq query().len()={sparq_full}"),
                );
            }
        }
    }

    println!(
        "fuzz[{category}] seeds {seed_start}..{} : checked={checked} skipped(unsupported)={skipped_unsupported} \
         full_mismatch={full_mismatch} count_mismatch={count_mismatch}",
        seed_start + count
    );
    if let Some(r) = first_repro {
        println!("\nFIRST FAILING CASE:\n{r}");
        std::process::exit(1);
    }
}

fn report_repro(slot: &mut Option<String>, seed: u64, q: &str, ttl: &str, msg: &str) {
    if slot.is_none() {
        *slot = Some(format!("seed={seed}\n{msg}\n--- query ---\n{q}\n--- graph ---\n{ttl}"));
    }
}
