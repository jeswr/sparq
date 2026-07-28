//! Differential fuzzer: generates random small graphs (with **mixed datatypes** on
//! a numeric predicate — the case that stresses inline-integer range-pruning) and
//! random queries across every plan shape, then checks that
//!   sparq `query().len()`  ==  Oxigraph solution count            (full differential)
//!   sparq `query()` rows    ==  Oxigraph's solution MULTISET      (`check_bindings`)
//!   sparq `query()` order   ==  Oxigraph's `ORDER BY` sequence    (`check_ordered`)
//!   sparq `ask()`           ==  Oxigraph's ASK boolean            (`check_ask`)
//!   sparq `construct_or_describe()` == Oxigraph's graph, up to bnode iso (`check_graph`)
//!   sparq `count()`         ==  sparq `query().len()`             (count-path differential)
//!
//! The MULTISET check is what makes a same-cardinality WRONG ANSWER visible: a count
//! differential cannot see a property path landing on the wrong endpoints, a `COUNT` /
//! `MIN` / `MAX` returning the wrong number, or a `BIND` / `VALUES` / sub-select binding
//! the wrong term. Every projected binding is compared, duplicates included, order
//! independently; the deliberate exclusions (arbitrary row CHOICE under `LIMIT` /
//! `OFFSET`, and the harness-forced derived-integer datatype normalisation) are documented
//! on `check_bindings` / `harness_datatype` and counted in the summary line.
//!
//! [OPUS-5] sq-qcnn.5: every value-level equality here is decided by **`sparq-difftest`**,
//! the engine-independent normalisation library (exact `num-bigint`/`bigdecimal` numerics,
//! XSD temporal rules, third-party RDFC-1.0 blank-node labelling), NOT by any sparq crate —
//! see the section header above `to_difftest` for why that independence is load-bearing.
//! `ASK` and `CONSTRUCT`/`DESCRIBE` are compared by their own oracles (the boolean, and the
//! triple set up to blank-node isomorphism) rather than by a cardinality, which cannot see
//! either. The generator emits only `SELECT` today — extending it to the graph/boolean forms
//! is `sq-qcnn.6`, so those two oracles are currently exercised by this module's unit tests.
//!
//! Oxigraph is an independent, mature SPARQL implementation, so an agreement over
//! thousands of random cases is strong evidence the optimizations (tagged ValueIds,
//! range-pruning, count-only joins, lazy count, filter pushdown, sort-merge OPTIONAL,
//! LIMIT early-termination) preserve SPARQL semantics. A mismatch prints a full
//! deterministic repro (seed + query + the graph).
//!
//! The ADJUDICATED cross-engine divergence classes are exempt from blind Oxigraph
//! agreement — and they live machine-readably in `bench/differential-divergences.json`
//! (bead sq-0iqzw), which this comparator CONSUMES (see `DivergenceAllowlist`):
//!   * `cross-family-eq-type-error` (sq-eibog): `=` / `!=` between literals of
//!     DIFFERENT comparison families (numeric vs string, xsd:integer vs `"s2"`, …).
//!     SPARQL 1.1 §17.4.1.7 makes such a comparison a TYPE ERROR (the row is filtered
//!     out); Oxigraph leniently resolves it to a boolean and KEEPS the row. sparq
//!     follows the spec (it passes all 15 W3C `sparql10/expr-equals` tests). For that
//!     shape the oracle re-derives the SPEC-CORRECT count itself (see
//!     `spec_filter_count`) and flags only a residual, genuine divergence — not
//!     Oxigraph's leniency.
//!   * `bnode-iri-inequality` (sq-ai2wa): `=` / `!=` pairing a blank node with an
//!     IRI — identity vs type-error reading (inert for this generator; see the JSON).
//!
//! A known-class mismatch is SKIPPED-WITH-COUNT (surfaced in the summary line); any
//! mismatch outside the listed classes FAILS. A missing/empty allowlist runs STRICT.
//!
//! Usage: `sparq-bench fuzz <seed_start> <count> [category]`
//! category ∈ `CATEGORIES` (below) or `all` (default) — lets a workflow shard the
//! space across agents (`.github/workflows/differential.yml` runs one shard per
//! category).

use oxigraph::store::Store;
// [OPUS-5] sq-qcnn.5 — the ENGINE-INDEPENDENT comparators (see the section header above
// `to_difftest`): every value-level equality decision in this harness is taken by a crate
// that depends on no sparq crate.
use sparq_difftest::{
    canonical_key, graph_isomorphic, multiset_equal, order_by_equal, solutions_have_blank_nodes,
    solutions_isomorphic, Solution as DSolution, Term as DTerm,
};

// ── ADJUDICATED-DIVERGENCE ALLOWLIST (bead sq-0iqzw) ─────────────────────────────
//
// The adjudicated cross-engine divergence classes live machine-readably in
// `bench/differential-divergences.json` (id + adjudication bead + spec clause). The
// comparator CONSUMES that file: a mismatch inside a listed class is SKIPPED-WITH-
// COUNT (surfaced in the summary line), anything else FAILS. Removing an entry from
// the file re-enables the strict differential for that class, so the JSON — not this
// code — is the source of truth for WHAT is adjudicated (the code only carries the
// per-class detectors). A missing/unreadable/malformed file runs STRICT (toward
// flagging, never toward skipping). [FABLE-5]

struct DivergenceAllowlist {
    /// sq-eibog: cross-family literal `=`/`!=` type errors (Oxigraph is lenient;
    /// handled by the spec-re-derivation sub-oracle, NOT a blind skip).
    cross_family_eq_type_error: bool,
    /// sq-ai2wa: blank-node-vs-IRI `=`/`!=` (identity vs type-error reading).
    bnode_iri_inequality: bool,
    /// Where the allowlist was loaded from (for the summary line).
    source: String,
}

impl DivergenceAllowlist {
    fn strict(source: String) -> Self {
        DivergenceAllowlist {
            cross_family_eq_type_error: false,
            bnode_iri_inequality: false,
            source,
        }
    }

    /// Load from `SPARQ_FUZZ_DIVERGENCES` (a CI/agent override), else the committed
    /// repo default resolved relative to this crate's manifest (works from any cwd).
    fn load() -> Self {
        let path = std::env::var("SPARQ_FUZZ_DIVERGENCES").unwrap_or_else(|_| {
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../bench/differential-divergences.json"
            )
            .to_string()
        });
        match std::fs::read_to_string(&path) {
            Ok(s) => Self::from_json(&s, &path),
            Err(e) => {
                eprintln!(
                    "warning: divergence allowlist {path} unreadable ({e}) — running \
                     STRICT (every mismatch fails)"
                );
                Self::strict(path)
            }
        }
    }

    /// Parse the allowlist JSON. An unknown class id is IGNORED with a loud warning
    /// (fail-STRICT: the comparator has no detector for it, so that class keeps
    /// failing rather than being silently "absorbed" by nothing); malformed JSON is
    /// also strict.
    fn from_json(s: &str, source: &str) -> Self {
        let mut out = Self::strict(source.to_string());
        let v: serde_json::Value = match serde_json::from_str(s) {
            Ok(v) => v,
            Err(e) => {
                eprintln!(
                    "warning: divergence allowlist {source} is not valid JSON ({e}) — \
                     running STRICT"
                );
                return out;
            }
        };
        for class in v["classes"].as_array().map(Vec::as_slice).unwrap_or(&[]) {
            match class["id"].as_str() {
                Some("cross-family-eq-type-error") => out.cross_family_eq_type_error = true,
                Some("bnode-iri-inequality") => out.bnode_iri_inequality = true,
                Some(other) => eprintln!(
                    "warning: divergence allowlist {source} lists unknown class id \
                     {other:?} — no detector for it; that class stays STRICT"
                ),
                None => eprintln!(
                    "warning: divergence allowlist {source} has a class without a \
                     string `id` — ignored (strict)"
                ),
            }
        }
        out
    }
}

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
            // High-precision decimals (>15 sig. digits) where two distinct values can
            // share an f64 — the decimal analogue of the >2^53 integer case.
            let hp_dec = [
                "0.123456789012345678",
                "0.123456789012345679",
                "1.000000000000000001",
                "0.299999999999999999",
            ];
            let v = match rng.below(9) {
                0 => format!("{}", rng.below(120)), // canonical integer (inline)
                1 => format!("\"{}\"^^xsd:int", rng.below(120)), // xsd:int (not inline)
                2 => format!("\"{}.5\"^^xsd:decimal", rng.below(120)), // decimal
                3 => format!("\"-{}\"^^xsd:integer", rng.below(60)), // negative integer (not inline)
                4 => format!("\"0{}\"^^xsd:integer", 1 + rng.below(9)), // non-canonical (leading zero)
                5 => format!("\"{}.0\"^^xsd:double", rng.below(120)),   // double
                6 => format!("\"{}\"^^xsd:integer", 9007199254740990u64 + rng.below(6)), // near 2^53 — f64-inexact
                7 => format!("\"{}\"^^xsd:decimal", hp_dec[rng.below(4) as usize]), // high-precision decimal
                _ => format!("\"s{}\"", rng.below(5)), // plain string (non-numeric)
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
    // Signed threshold straddling the data range AND zero — negative thresholds are
    // non-sargable (parsed as UnaryMinus), forcing the residual compare path that a
    // non-numeric operand must turn into a type error, not a string comparison.
    // 1-in-6: a threshold near 2^53, where an f64 numeric model loses integer precision.
    if rng.chance(1, 6) {
        let t = 9007199254740990u64 + rng.below(6);
        return format!("FILTER({var} {op} {t})");
    }
    let t = rng.below(160) as i64 - 40; // -40..119
    format!("FILTER({var} {op} {t})")
}

/// Every query category the generator can emit. `all` picks one uniformly at
/// random per seed; `.github/workflows/differential.yml` runs ONE NIGHTLY SHARD PER
/// ENTRY (running only `all` hides category-dense bugs — see that workflow's header),
/// so adding a category here means adding a shard there too. `categories_have_a_shard`
/// (below) pins the two lists together.
const CATEGORIES: &[&str] = &[
    "bgp", "filter", "equality", "optional", "union", "minus", "limit", "distinct", "order",
    // sq-j80vk: the SPARQL 1.1 surfaces that previously had ZERO standing
    // wrong-answer differential vs the Oxigraph oracle.
    "path", "aggregate", "subquery", "exists", "values", "bind", "graph",
];

/// A random query in the chosen category. **Every category must keep these
/// invariants** (sq-j80vk) or the comparator's oracles mis-fire:
///
/// * **Always-valid SPARQL 1.1**, restricted to the surface sparq supports —
///   Oxigraph must parse it too. (A surface sparq declines is skipped as
///   `unsupported`, which is fair; a WRONG ANSWER is not.)
/// * **A deterministic solution multiset.** Never `LIMIT`/`OFFSET` without a total
///   order over the projected rows, and never a `LIMIT` inside a sub-select that is
///   then joined: SPARQL leaves the row CHOICE arbitrary there, so two conformant
///   engines may legitimately return different (equally valid) rows. (`check_bindings`
///   consequently skips the VALUE comparison for any query carrying `LIMIT`/`OFFSET`,
///   so a new shape that needs one loses its value-level oracle.)
/// * **No `=` / `!=` outside the `equality` / `filter` categories.** Those exact
///   texts are what `parse_eq_filter` recognises as the adjudicated sq-eibog shape;
///   a look-alike elsewhere would be re-derived by a sub-oracle that does not model
///   the surrounding query.
/// * **A PLAIN-VARIABLE sort key**, and — under `LIMIT`/`OFFSET` — one that covers every
///   projected variable. `check_ordered` fails CLOSED otherwise (skip-with-count, see
///   `order_by_vars` / `sort_key_is_total`), so a category that sorted by an expression, or
///   truncated a sequence whose ties are visible in the projection, would silently lose its
///   order-level oracle rather than gain a wrong one. The `order` category satisfies both by
///   projecting exactly its sort variable.
/// * **No blank nodes.** Keeping them out of the generator is exactly what keeps the
///   sq-ai2wa (bnode-vs-IRI) allowlist class INERT; introducing one makes that class
///   live (its detector is already wired) and needs its own adjudication first.
fn gen_query(rng: &mut Rng, category: &str) -> String {
    let pfx = "PREFIX ex: <http://ex/>\nPREFIX xsd: <http://www.w3.org/2001/XMLSchema#>\n";
    // Pick an effective category (when "all", choose one at random).
    let cat = if category == "all" {
        CATEGORIES[rng.below(CATEGORIES.len() as u64) as usize]
    } else {
        category
    };

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
            let pat = if var == "?a" {
                "?s ex:age ?a"
            } else {
                "?s ex:val ?v"
            };
            let extra = if rng.chance(1, 2) {
                " . ?s ex:name ?n"
            } else {
                ""
            };
            format!("{pat}{extra} {}", gen_filter(rng, var))
        }
        "optional" => match rng.below(3) {
            0 => "?s ex:name ?n OPTIONAL { ?s ex:age ?a }".to_string(),
            1 => "?s ex:age ?a OPTIONAL { ?s ex:p ?o }".to_string(),
            _ => "?s ex:name ?n OPTIONAL { ?s ex:age ?a . FILTER(?a > 50) }".to_string(),
        },
        "equality" => {
            // `=` / `!=` against a constant of a possibly-different type, over the
            // MIXED column — exercises RDFterm-equal (known-different vs type error).
            let op = if rng.chance(1, 2) { "=" } else { "!=" };
            let rhs = match rng.below(8) {
                0 => format!("{}", rng.below(120)),              // integer
                1 => format!("\"s{}\"", rng.below(5)),           // plain string
                2 => "ex:n0".to_string(),                        // IRI
                3 => "\"true\"^^xsd:boolean".to_string(),        // boolean
                4 => format!("\"{}\"^^xsd:int", rng.below(120)), // xsd:int
                5 => format!("\"{}\"^^xsd:integer", 9007199254740990u64 + rng.below(6)), // near 2^53
                6 => format!(
                    "\"{}\"^^xsd:decimal",
                    [
                        "0.123456789012345678",
                        "0.123456789012345679",
                        "0.299999999999999999"
                    ][rng.below(3) as usize]
                ), // high-precision decimal
                _ => format!("\"{}.5\"^^xsd:decimal", rng.below(120)),                   // decimal
            };
            let var = if rng.chance(1, 2) {
                ("?v", "ex:val")
            } else {
                ("?a", "ex:age")
            };
            return format!(
                "{pfx}SELECT ?s WHERE {{ ?s {} {} FILTER({} {op} {rhs}) }}",
                var.1, var.0, var.0
            );
        }
        "union" => "{ ?s ex:age ?a } UNION { ?s ex:name ?a }".to_string(),
        "minus" => "?s ex:name ?n MINUS { ?s ex:age ?a }".to_string(),
        // ── sq-j80vk categories ──────────────────────────────────────────────────
        // PROPERTY PATHS (SPARQL 1.1 §9). `ex:p` is the edge predicate, so the graph
        // is a small random digraph with chains and cycles — `+`/`*` closure, inverse,
        // sequence and the negated property set all have non-trivial answers on it.
        //
        // ZERO-LENGTH forms (`*`, `?`) are emitted ONLY with their start end already
        // bound to a term that OCCURS IN THE GRAPH (`?s ex:p ?x . ?x ex:p* ?o`). The
        // zero-length match of a ground endpoint that does NOT occur in the data is a
        // REAL, currently-unadjudicated cross-engine divergence (sparq yields the
        // zero-length solution — which is what the W3C property-path tests expect,
        // cf. sparq-conformance/FINDINGS.md F11 — while Oxigraph 0.5 returns nothing);
        // absorbing it here would mean widening the allowlist without an adjudication,
        // which sq-j80vk explicitly forbids. It is filed as its own follow-up instead.
        "path" => match rng.below(9) {
            0 => "?s ex:p/ex:p ?o".to_string(),      // sequence
            1 => "?s ^ex:p ?o".to_string(),          // inverse
            2 => "?s ex:p+ ?o".to_string(),          // transitive closure
            3 => "?s (ex:p|ex:name) ?o".to_string(), // alternative
            4 => "?s ex:p/ex:age ?a".to_string(),    // sequence into a value column
            5 => "?s ex:p+/ex:name ?n".to_string(),  // closure then a value column
            6 => "?s !(ex:p|ex:age) ?o".to_string(), // negated property set
            // zero-or-more / zero-or-one from a start that is certainly a graph node.
            7 => "?s ex:p ?x . ?x ex:p* ?o".to_string(),
            _ => "?s ex:p ?x . ?x ex:p? ?o".to_string(),
        },
        // AGGREGATES / GROUP BY / HAVING (§11). The aggregate's VALUE is compared
        // directly — `check_bindings` matches the full solution multiset, so a bare
        // `COUNT(*)`'s single row is checked for the right NUMBER, not merely for being
        // one row. `HAVING` additionally makes the value visible to the CARDINALITY
        // differential (the number of surviving groups is a function of the aggregated
        // values), which keeps the two oracles independent. Numeric aggregates stay on
        // `ex:age` (an all-`xsd:integer` column) — SUM/AVG over the deliberately MIXED
        // `ex:val` column is a type error whose propagation is not what this category is
        // pinning.
        //
        // `AVG` is exercised in `HAVING` but never PROJECTED: XPath leaves the precision
        // of decimal division implementation-defined (`op:numeric-divide`, ≥18 digits),
        // and sparq rounds where Oxigraph truncates, so a projected `AVG` would differ
        // in its last digit between two CONFORMANT engines. Against an integer HAVING
        // threshold that last digit can never flip the comparison, so the value stays
        // oracle-checked without manufacturing a false mismatch.
        "aggregate" => {
            let k = rng.below(4);
            let t = rng.below(200);
            return match rng.below(7) {
                0 => format!("{pfx}SELECT (COUNT(*) AS ?c) WHERE {{ ?s ex:age ?a }}"),
                1 => format!("{pfx}SELECT ?s (COUNT(?o) AS ?c) WHERE {{ ?s ex:p ?o }} GROUP BY ?s"),
                2 => format!(
                    "{pfx}SELECT ?s (COUNT(?o) AS ?c) WHERE {{ ?s ex:p ?o }} GROUP BY ?s \
                     HAVING(COUNT(?o) > {k})"
                ),
                3 => format!(
                    "{pfx}SELECT ?s (SUM(?a) AS ?t) WHERE {{ ?s ex:age ?a }} GROUP BY ?s \
                     HAVING(SUM(?a) > {t})"
                ),
                4 => format!(
                    "{pfx}SELECT ?n (COUNT(DISTINCT ?s) AS ?c) WHERE {{ ?s ex:name ?n }} \
                     GROUP BY ?n HAVING(COUNT(DISTINCT ?s) > {k})"
                ),
                5 => format!(
                    "{pfx}SELECT (MIN(?a) AS ?mn) (MAX(?a) AS ?mx) WHERE {{ ?s ex:age ?a }} \
                     HAVING(MAX(?a) > {t})"
                ),
                _ => format!(
                    "{pfx}SELECT ?s (SUM(?a) AS ?sm) WHERE {{ ?s ex:p ?o . ?o ex:age ?a }} \
                     GROUP BY ?s HAVING(AVG(?a) > {t})"
                ),
            };
        }
        // SUB-SELECTS (§12). No `LIMIT` inside a joined sub-select (see the
        // determinism invariant above). The aggregating shapes push the aggregate's
        // VALUE into the outer row count via an outer `FILTER`, the same trick
        // `HAVING` plays for the `aggregate` category.
        "subquery" => {
            let k = rng.below(4);
            return match rng.below(5) {
                0 => format!(
                    "{pfx}SELECT * WHERE {{ ?s ex:name ?n {{ SELECT ?s WHERE {{ ?s ex:age ?a }} }} }}"
                ),
                1 => format!(
                    "{pfx}SELECT * WHERE {{ {{ SELECT DISTINCT ?s WHERE {{ ?s ex:p ?o }} }} \
                     ?s ex:age ?a }}"
                ),
                2 => format!(
                    "{pfx}SELECT * WHERE {{ {{ SELECT ?s (COUNT(?o) AS ?c) WHERE {{ ?s ex:p ?o }} \
                     GROUP BY ?s }} FILTER(?c > {k}) }}"
                ),
                3 => format!(
                    "{pfx}SELECT * WHERE {{ ?s ex:age ?a \
                     OPTIONAL {{ {{ SELECT ?s ?n WHERE {{ ?s ex:name ?n }} }} }} }}"
                ),
                _ => format!(
                    "{pfx}SELECT * WHERE {{ ?s ex:name ?n \
                     {{ SELECT ?s WHERE {{ ?s ex:age ?a . FILTER(?a > {}) }} }} }}",
                    rng.below(120)
                ),
            };
        }
        // EXISTS / NOT EXISTS (§8.1.1, §17.4.1.4) — correlated, so the inner pattern
        // is evaluated with the outer row substituted in.
        "exists" => match rng.below(6) {
            0 => "?s ex:age ?a FILTER EXISTS { ?s ex:name ?n }".to_string(),
            1 => "?s ex:age ?a FILTER NOT EXISTS { ?s ex:p ?o }".to_string(),
            2 => "?s ex:name ?n FILTER NOT EXISTS { ?s ex:age ?a . FILTER(?a > 50) }".to_string(),
            3 => "?s ex:p ?o FILTER EXISTS { ?o ex:age ?a }".to_string(),
            4 => format!(
                "?s ex:age ?a FILTER(EXISTS {{ ?s ex:name ?n }} && ?a < {})",
                rng.below(120)
            ),
            _ => "?s ex:name ?n FILTER NOT EXISTS { ?s ex:p ?o . ?o ex:age ?a }".to_string(),
        },
        // VALUES / inline data (§10.2.1), incl. the multi-column and UNDEF forms.
        "values" => {
            let (a, b) = (rng.below(120), rng.below(120));
            match rng.below(4) {
                0 => "?s ex:age ?a VALUES ?s { ex:n0 ex:n1 ex:n2 }".to_string(),
                1 => format!("VALUES ?a {{ {a} {b} }} ?s ex:age ?a"),
                2 => format!("?s ex:age ?a VALUES (?s ?a) {{ (ex:n0 {a}) (ex:n1 UNDEF) }}"),
                _ => "?s ex:name ?n VALUES ?n { \"nm0\" \"nm1\"@en }".to_string(),
            }
        }
        // BIND (§10.2.2). A bare BIND cannot change the row count, so every shape
        // FILTERs on the bound variable — that makes the computed VALUE observable in
        // the count differential. `COALESCE` over an OPTIONAL covers the UNBOUND path.
        "bind" => {
            let t = rng.below(160) as i64 - 40;
            match rng.below(5) {
                0 => format!("?s ex:age ?a BIND(?a + 1 AS ?b) FILTER(?b > {t})"),
                1 => format!("?s ex:age ?a BIND(?a * 2 AS ?b) FILTER(?b < {t})"),
                2 => format!(
                    "?s ex:name ?n OPTIONAL {{ ?s ex:age ?a }} BIND(COALESCE(?a, 0) AS ?b) \
                     FILTER(?b > {t})"
                ),
                3 => format!("?s ex:age ?a BIND(IF(?a > {t}, 1, 0) AS ?b) FILTER(?b > 0)"),
                _ => format!(
                    "?s ex:age ?a . ?s ex:name ?n BIND(STRLEN(STR(?n)) AS ?l) FILTER(?l > {})",
                    rng.below(5)
                ),
            }
        }
        // GRAPH (§13.3). The harness loads ONE Turtle document, so the dataset has a
        // default graph and NO named graphs — every `GRAPH` block is therefore empty,
        // and what these shapes pin is that sparq agrees: a `GRAPH` block must NOT
        // leak the default graph's triples, and an empty named-graph side must still
        // COMPOSE correctly (UNION keeps the left rows; OPTIONAL keeps them with `?g`
        // / `?n` unbound). Named-graph DATA is out of scope here — the store-mode
        // shards (dict-spill) re-serialise through triples only, so quads in the
        // generated document would diverge for harness reasons, not engine ones.
        "graph" => match rng.below(4) {
            0 => "GRAPH ?g { ?s ex:age ?a }".to_string(),
            1 => "{ ?s ex:age ?a } UNION { GRAPH ?g { ?s ex:age ?a } }".to_string(),
            2 => "?s ex:age ?a OPTIONAL { GRAPH ?g { ?s ex:name ?n } }".to_string(),
            _ => "GRAPH ex:g1 { ?s ex:p ?o }".to_string(),
        },
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

// clippy: differential oracle pins oxigraph's legacy Store::query semantics
#[allow(deprecated)]
fn oxi_count(store: &Store, q: &str) -> Result<usize, String> {
    match store.query(q).map_err(|e| e.to_string())? {
        oxigraph::sparql::QueryResults::Solutions(s) => Ok(s.count()),
        // [OPUS-5] sq-qcnn.5: an ASK / CONSTRUCT / DESCRIBE result is NOT count-comparable
        // (see `QueryForm`) — one boolean result whichever way it answered, and a graph of
        // the right SIZE can still be the wrong graph. `run` routes those forms to
        // `check_ask` / `check_graph` before reaching here, so arriving here means the form
        // classifier missed a case: that must become a SKIP, never a count comparison.
        _ => Err("not a solution sequence (ASK / CONSTRUCT are compared by their own \
                  oracle, never by a count)"
            .to_string()),
    }
}

// ── DOCUMENTED-DIVERGENCE ORACLE: cross-family `=` / `!=` type errors (sq-eibog) ──
//
// Oxigraph and sparq DISAGREE, by design, on `=` / `!=` between two literals whose
// datatypes are in different comparison families (e.g. `"71"^^xsd:integer != "s2"`,
// or `?age != "s4"` where `?age` is `xsd:integer`). This is NOT a sparq bug: it is a
// KNOWN leniency in Oxigraph's reference behaviour.
//
// SPARQL 1.1 §17.3 (Operator Mapping) + §17.4.1.7 (RDFterm-equal): when no
// type-specific `=` operator applies (both numeric / both xsd:string / both boolean /
// both dateTime), `A = B` falls back to `RDFterm-equal`, which **"produces a type
// error if the arguments are both literal but are not the same RDF term"**. `A != B`
// is `fn:not(RDFterm-equal(A, B))`, so it errors too, and a FILTER whose expression
// errors on a row DROPS that row (§17.4, effective boolean value of an error is
// undefined → the row is eliminated). sparq implements this (it PASSES all 15 W3C
// `sparql10/expr-equals` evaluation tests, incl. `eq-3`/`eq-4`/`eq2-1`, whose
// authoritative `!=` result `result-eq2-2.ttl` EXCLUDES every numeric-vs-string pair).
// Oxigraph instead resolves cross-family `=` to FALSE (so `!=` to TRUE) and KEEPS the
// row — a lenient, non-conformant reading. Blindly trusting Oxigraph's count here
// would flag spec-correct sparq output as a mismatch.
//
// The oracle stays sound: when the two counts differ, we RE-DERIVE the spec-correct
// count independently — take the ORIGINAL query with only its `FILTER(...)` clause
// removed (so the exact pre-filter solution multiset, incl. any join and duplicates,
// is preserved), read each solution's bound term, and re-apply the spec's `=`/`!=`
// rule term-by-term here (a self-contained reference, NOT sparq's evaluator). We flag
// a mismatch ONLY when sparq disagrees with THAT spec-correct count. A genuine
// over/under-exclusion bug therefore still trips the fuzzer; only the Oxigraph-leniency
// delta is absorbed.

/// The single-variable `=` / `!=` FILTER shape the generator emits over the mixed
/// column: the filtered variable, the operator, and the constant right-hand side.
struct EqFilter {
    var: String,   // "v" or "a" (the graph variable under test)
    negated: bool, // true for `!=`, false for `=`
    rhs: String,   // the literal/IRI constant, verbatim from the query
}

/// Parse the generated `filter` / `equality` query IFF it is a single-variable
/// `=` / `!=` over the mixed column against a CONSTANT — the only shape on which
/// Oxigraph's cross-family leniency can diverge. Returns `None` for any other query
/// (those keep the strict Oxigraph-count differential).
fn parse_eq_filter(q: &str) -> Option<EqFilter> {
    let (op, negated) = if q.contains("!=") {
        ("!=", true)
    } else if q.contains(" = ") {
        ("=", false)
    } else {
        return None;
    };
    // Only the equality/filter shapes bind the tested var to ex:val / ex:age.
    let var = if q.contains("ex:val ?v") {
        "v"
    } else if q.contains("ex:age ?a") {
        "a"
    } else {
        return None;
    };
    // Extract `FILTER(?var OP RHS)` — the generator always emits exactly this text.
    let needle = format!("FILTER(?{var} {op} ");
    let start = q.find(&needle)? + needle.len();
    let rest = &q[start..];
    let end = rest.find(')')?;
    let rhs = rest[..end].trim().to_string();
    Some(EqFilter {
        var: var.to_string(),
        negated,
        rhs,
    })
}

/// The bound `?var` term of EVERY pre-FILTER solution of the ORIGINAL query — i.e. the
/// query with its `FILTER(...)` clause removed — so the spec-correct FILTER can be
/// re-applied per solution independently of both engines. Removing only the FILTER (not
/// rebuilding a fresh pattern) preserves the exact solution MULTISET, including any join
/// to a second pattern (`?s ex:name ?n`) and duplicate rows, so the re-derived count is
/// exact for every generated equality shape — single-pattern or 2-pattern-join.
// clippy: differential oracle pins oxigraph's legacy Store::query semantics
#[allow(deprecated)]
fn all_var_terms(store: &Store, q: &str, f: &EqFilter) -> Option<Vec<oxigraph::model::Term>> {
    // Delete the `FILTER(...)` clause from the original query text (the generator emits
    // exactly one, always well-formed and balanced), keeping all patterns intact.
    let fstart = q.find("FILTER(")?;
    let after = &q[fstart + "FILTER(".len()..];
    let close = after.find(')')?; // the generated FILTER body has no nested parens
    let stripped = format!("{}{}", &q[..fstart], &after[close + 1..]);
    // Widen the projection to `*` so the filtered variable is exposed even when the
    // original query only projects `?s` (`SELECT ?s WHERE …`). This preserves the
    // solution MULTISET/cardinality of the WHERE clause exactly (SELECT projection
    // does not deduplicate without DISTINCT), so the per-row count stays exact.
    let star_projected = stripped.replacen("SELECT ?s WHERE", "SELECT * WHERE", 1);
    match store.query(&star_projected).ok()? {
        oxigraph::sparql::QueryResults::Solutions(s) => Some(
            s.filter_map(|sol| sol.ok())
                .filter_map(|sol| sol.get(f.var.as_str()).cloned())
                .collect(),
        ),
        _ => None,
    }
}

/// The comparison FAMILY of an Oxigraph literal term for spec-correct `=`: two
/// literals decide (TRUE/FALSE) only within the same family; anything else that is
/// not the same term is a TYPE ERROR. `None` = not a literal (IRI / bnode / triple).
fn oxi_family(t: &oxigraph::model::Term) -> Option<&'static str> {
    use oxigraph::model::Term;
    let lit = match t {
        Term::Literal(l) => l,
        _ => return None, // IRI / bnode — handled by identity in `spec_eq`
    };
    let dt = lit.datatype().as_str().to_string();
    if lit.language().is_some() {
        return Some("lang");
    }
    Some(match dt.as_str() {
        "http://www.w3.org/2001/XMLSchema#integer"
        | "http://www.w3.org/2001/XMLSchema#decimal"
        | "http://www.w3.org/2001/XMLSchema#double"
        | "http://www.w3.org/2001/XMLSchema#float"
        | "http://www.w3.org/2001/XMLSchema#int"
        | "http://www.w3.org/2001/XMLSchema#long"
        | "http://www.w3.org/2001/XMLSchema#short"
        | "http://www.w3.org/2001/XMLSchema#byte"
        | "http://www.w3.org/2001/XMLSchema#nonNegativeInteger"
        | "http://www.w3.org/2001/XMLSchema#nonPositiveInteger"
        | "http://www.w3.org/2001/XMLSchema#negativeInteger"
        | "http://www.w3.org/2001/XMLSchema#positiveInteger"
        | "http://www.w3.org/2001/XMLSchema#unsignedInt"
        | "http://www.w3.org/2001/XMLSchema#unsignedLong"
        | "http://www.w3.org/2001/XMLSchema#unsignedShort"
        | "http://www.w3.org/2001/XMLSchema#unsignedByte" => "num",
        "http://www.w3.org/2001/XMLSchema#string" => "str",
        "http://www.w3.org/2001/XMLSchema#boolean" => "bool",
        "http://www.w3.org/2001/XMLSchema#dateTime"
        | "http://www.w3.org/2001/XMLSchema#dateTimeStamp" => "dateTime",
        "http://www.w3.org/2001/XMLSchema#date" => "date",
        d if d.starts_with("http://www.w3.org/2001/XMLSchema#") => "otherXsd",
        _ => "unknown",
    })
}

/// SPARQL 1.1 `A = B` as three values: `Some(true)`, `Some(false)`, or `None` (type
/// error). Self-contained reference for the generated constant-vs-term comparison —
/// deliberately independent of sparq's evaluator so it is a real oracle. Only the
/// families the generator can produce (num / str / bool / dateTime / date / otherXsd
/// / lang / IRI) need be decided; every genuinely cross-family literal pair is a
/// type error per §17.4.1.7.
fn spec_eq(a: &oxigraph::model::Term, b: &oxigraph::model::Term) -> Option<bool> {
    // sameTerm decides even for unknown datatypes.
    if a == b {
        return Some(true);
    }
    let (fa, fb) = (oxi_family(a), oxi_family(b));
    // At least one non-literal (IRI / bnode) and not the same term → known different.
    if fa.is_none() || fb.is_none() {
        return Some(false);
    }
    let (fa, fb) = (fa.unwrap(), fb.unwrap());
    if fa != fb {
        return None; // cross-family literals, not same term → TYPE ERROR
    }
    use oxigraph::model::Term::Literal;
    let (la, lb) = match (a, b) {
        (Literal(x), Literal(y)) => (x, y),
        _ => unreachable!("both are literals (families matched)"),
    };
    match fa {
        // EXACT numeric value equality — NOT via f64. The generator deliberately emits
        // integers/decimals beyond 2^53 (e.g. 9007199254740992..994) that collapse to a
        // single f64; sparq compares them exactly and an f64 oracle would wrongly report
        // them equal and flag sparq. `numeric_eq_exact` compares the decimal expansions.
        "num" => numeric_eq_exact(la.value(), lb.value()),
        "str" => Some(la.value() == lb.value()),
        "bool" => {
            let norm = |v: &str| matches!(v, "true" | "1");
            let ok = |v: &str| matches!(v, "true" | "false" | "0" | "1");
            if ok(la.value()) && ok(lb.value()) {
                Some(norm(la.value()) == norm(lb.value()))
            } else {
                None
            }
        }
        // The remaining families (dateTime / date / otherXsd / lang / unknown) never
        // occur in the mixed FILTER columns (`ex:val` / `ex:age`) or as a generated
        // equality constant, so no same-family, non-identical pair of them ever reaches
        // this oracle. A value-equal-and-identical pair was already decided by the
        // `a == b` early return; anything else here is left UNDECIDED (`None`), which is
        // the conservative choice — it can only ever DROP a row this oracle never sees.
        _ => None,
    }
}

/// EXACT numeric value equality of two XSD numeric lexicals (integer / decimal /
/// double), compared as arbitrary-precision decimal expansions — never through f64.
/// This matches SPARQL `op:numeric-equal` after XPath type promotion (int/decimal/double
/// share a value space) WITHOUT the f64-collapse that would wrongly equate distinct
/// integers/decimals above 2^53 (the very case the fuzzer stresses). Returns `None` on
/// an ill-formed lexical (a SPARQL type error).
fn numeric_eq_exact(x: &str, y: &str) -> Option<bool> {
    Some(decimal_expansion(x)? == decimal_expansion(y)?)
}

/// Normalize an XSD numeric lexical to a canonical `(negative, integer_digits,
/// fraction_digits)` with no leading/trailing insignificant zeros, so two expansions
/// are equal IFF the values are equal. Handles a leading sign, a fraction point, and a
/// decimal exponent (`e`/`E`, for xsd:double/float). Signed zero normalizes to `+0`.
fn decimal_expansion(lex: &str) -> Option<(bool, String, String)> {
    let lex = lex.trim();
    // Split off a decimal exponent (xsd:double / float).
    let (mantissa, exp) = match lex.split_once(['e', 'E']) {
        Some((m, e)) => (m, e.parse::<i64>().ok()?),
        None => (lex, 0),
    };
    let (neg, digits) = match mantissa.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, mantissa.strip_prefix('+').unwrap_or(mantissa)),
    };
    let (int_part, frac_part) = match digits.split_once('.') {
        Some((i, f)) => (i, f),
        None => (digits, ""),
    };
    if int_part.is_empty() && frac_part.is_empty() {
        return None;
    }
    if !int_part.chars().all(|c| c.is_ascii_digit())
        || !frac_part.chars().all(|c| c.is_ascii_digit())
    {
        return None;
    }
    // Build the full digit string and the position of the decimal point (from the left).
    let mut all: String = int_part.chars().chain(frac_part.chars()).collect();
    // point index within `all` counting from the start of int_part, then shift by exp.
    let point = int_part.len() as i64 + exp;
    // Reconstruct integer/fraction around the (possibly shifted) point.
    let (new_int, new_frac) = if point <= 0 {
        let zeros = "0".repeat((-point) as usize);
        (String::from("0"), format!("{zeros}{all}"))
    } else if (point as usize) >= all.len() {
        all.push_str(&"0".repeat(point as usize - all.len()));
        (all, String::new())
    } else {
        let f = all.split_off(point as usize);
        (all, f)
    };
    // Strip insignificant zeros.
    let int_norm = new_int.trim_start_matches('0');
    let int_norm = if int_norm.is_empty() { "0" } else { int_norm };
    let frac_norm = new_frac.trim_end_matches('0');
    let is_zero = int_norm == "0" && frac_norm.is_empty();
    Some((neg && !is_zero, int_norm.to_string(), frac_norm.to_string()))
}

/// Parse the generated RHS constant (verbatim query text) into an Oxigraph `Term`.
/// Handles the exact spellings `gen_filter` / the `equality` category emit: a bare
/// integer, a `"…"`-quoted string, a `"…"^^xsd:TYPE` typed literal, and an `ex:` IRI.
fn parse_rhs_term(rhs: &str) -> Option<oxigraph::model::Term> {
    use oxigraph::model::{Literal, NamedNode, Term};
    let xsd = |t: &str| NamedNode::new(format!("http://www.w3.org/2001/XMLSchema#{t}")).unwrap();
    if let Some(rest) = rhs.strip_prefix('"') {
        // "lex" or "lex"^^xsd:TYPE
        let close = rest.find('"')?;
        let lex = &rest[..close];
        let after = &rest[close + 1..];
        if let Some(dt) = after.strip_prefix("^^xsd:") {
            return Some(Term::Literal(Literal::new_typed_literal(lex, xsd(dt))));
        }
        return Some(Term::Literal(Literal::new_simple_literal(lex)));
    }
    if let Some(local) = rhs.strip_prefix("ex:") {
        return Some(Term::NamedNode(
            NamedNode::new(format!("http://ex/{local}")).unwrap(),
        ));
    }
    // bare integer / boolean keyword
    if rhs.parse::<i64>().is_ok() {
        return Some(Term::Literal(Literal::new_typed_literal(
            rhs,
            xsd("integer"),
        )));
    }
    None
}

/// The SPEC-CORRECT solution count for a single-variable `=` / `!=` FILTER over the
/// mixed column — the spec's own answer, computed independently of both engines.
/// `None` when the shape is not a recognised constant equality filter (caller keeps
/// the strict Oxigraph differential in that case).
fn spec_filter_count(store: &Store, q: &str) -> Option<usize> {
    let f = parse_eq_filter(q)?;
    let rhs = parse_rhs_term(&f.rhs)?;
    let terms = all_var_terms(store, q, &f)?;
    let kept = terms
        .iter()
        .filter(|t| match spec_eq(t, &rhs) {
            // `=` keeps eq==true; `!=` keeps eq==false; a type error drops the row.
            Some(eq) => eq != f.negated,
            None => false,
        })
        .count();
    Some(kept)
}

/// sq-ai2wa detector — deliberately NARROW (never widen beyond the adjudication in
/// bench/differential-divergences.json): the mismatching case must be the recognised
/// single-variable constant `=`/`!=` FILTER shape AND pair a BLANK NODE on one side
/// with an IRI on the other. The current generator emits no blank nodes and Oxigraph
/// shares sparq's identity reading of non-literal `=`/`!=`, so this cannot fire
/// today; it exists so generator/oracle growth (or an Oxigraph behaviour change)
/// surfaces as an ADJUDICATED skip-with-count instead of a false positive. [FABLE-5]
fn is_bnode_iri_inequality(store: &Store, q: &str) -> bool {
    use oxigraph::model::Term;
    let Some(f) = parse_eq_filter(q) else {
        return false;
    };
    let Some(rhs) = parse_rhs_term(&f.rhs) else {
        return false;
    };
    let Some(terms) = all_var_terms(store, q, &f) else {
        return false;
    };
    terms.iter().any(|t| {
        matches!(
            (t, &rhs),
            (Term::BlankNode(_), Term::NamedNode(_)) | (Term::NamedNode(_), Term::BlankNode(_))
        )
    })
}

// ── THE CANONICAL CROSS-ORACLE COMPARATORS (sq-qcnn.5) ───────────────────────────
//
// The value-level comparison is delegated WHOLESALE to `sparq-difftest` — the
// engine-independent normalisation library built for exactly this job (sq-qcnn.4,
// `research/differential-testing-value-level.md`). That crate depends on NO sparq crate:
// its numeric tower is `num-bigint` / `bigdecimal`, its XSD rules are implemented from the
// spec, and its blank-node labelling is third-party RDFC-1.0 (`rdf-canon`), NOT
// `sparq-canon`. Keying both sides through sparq's OWN value code would make a bug there
// apply identically to both sides and CANCEL — the differential would go blind exactly
// where it must see. Keying them through `sparq-difftest` does not.
//
// `oxrdf` is used here purely as a CARRIER: both engines already materialise the same
// `oxrdf::Term`, and `to_difftest` copies it field-for-field into the neutral model. No
// equality decision is taken on the oxrdf side — which terms count as equal is settled
// entirely by `sparq_difftest::canonical_key` / `sparq_difftest::iso`.

/// One engine's solution sequence in the neutral model, **in engine order**. Order is kept
/// (not sorted away as it was under the local comparator) because the `ORDER BY` oracle
/// needs it; the multiset oracle is order-insensitive by construction.
type Solutions = Vec<DSolution>;

/// XSD normalises a numeric literal's LEXICAL form, and Oxigraph's store additionally
/// normalises every type DERIVED from `xsd:integer` (`xsd:int`, `xsd:long`, the
/// unsigned/bounded types, …) down to `xsd:integer` when it loads the graph. sparq keeps the
/// term exactly as written, so the two engines report a different — but equally faithful —
/// datatype for the SAME stored triple.
///
/// That is a HARNESS artefact, not a softening of the oracle, so it is applied on the way
/// INTO the neutral model rather than hidden inside the comparator: the types derived from
/// `xsd:integer` collapse onto their base, and nothing else moves. The primitive numeric
/// types stay DISTINCT (`integer` vs `decimal` vs `double`), and the VALUE is still compared
/// exactly and at arbitrary precision inside `sparq-difftest` — so the >2^53 integers and
/// 18-digit decimals the generator emits to defeat an f64 oracle still separate.
fn harness_datatype(dt: &str) -> &str {
    const XSD: &str = "http://www.w3.org/2001/XMLSchema#";
    const INTEGER: &str = "http://www.w3.org/2001/XMLSchema#integer";
    match dt.strip_prefix(XSD) {
        Some(
            "long" | "int" | "short" | "byte" | "nonNegativeInteger" | "positiveInteger"
            | "nonPositiveInteger" | "negativeInteger" | "unsignedLong" | "unsignedInt"
            | "unsignedShort" | "unsignedByte",
        ) => INTEGER,
        _ => dt,
    }
}

/// Copy one engine's `oxrdf` term into `sparq-difftest`'s neutral term model — a carrier
/// conversion, not an equality decision (see the section header).
fn to_difftest(t: &oxigraph::model::Term) -> DTerm {
    use oxigraph::model::Term as OTerm;
    match t {
        OTerm::NamedNode(n) => DTerm::Iri(n.as_str().to_string()),
        OTerm::BlankNode(b) => DTerm::Blank(b.as_str().to_string()),
        OTerm::Literal(l) => DTerm::Literal {
            lexical: l.value().to_string(),
            datatype: harness_datatype(l.datatype().as_str()).to_string(),
            lang: l.language().map(str::to_string),
        },
        OTerm::Triple(t) => DTerm::Triple(Box::new(triple_to_difftest(t))),
    }
}

/// Copy an `oxrdf` triple (a `CONSTRUCT`/`DESCRIBE` result triple, or an RDF-1.2 triple
/// term) into the neutral model.
fn triple_to_difftest(t: &oxigraph::model::Triple) -> [DTerm; 3] {
    [
        to_difftest(&oxigraph::model::Term::from(t.subject.clone())),
        DTerm::Iri(t.predicate.as_str().to_string()),
        to_difftest(&t.object),
    ]
}

/// Oxigraph's full solution sequence, in engine order. `None` when the query did not produce
/// a solution sequence (an `ASK`/`CONSTRUCT` — compared by its own oracle) or a solution
/// failed to decode.
// clippy: differential oracle pins oxigraph's legacy Store::query semantics
#[allow(deprecated)]
fn oxi_solutions(store: &Store, q: &str) -> Option<Solutions> {
    match store.query(q).ok()? {
        oxigraph::sparql::QueryResults::Solutions(s) => {
            let mut out = Solutions::new();
            for sol in s {
                let sol = sol.ok()?;
                out.push(
                    sol.iter()
                        .map(|(v, t)| (v.as_str().to_string(), to_difftest(t)))
                        .collect(),
                );
            }
            Some(out)
        }
        _ => None,
    }
}

/// sparq's full solution sequence, in the same neutral form as `oxi_solutions`. Both engines
/// materialise the SAME `oxrdf::Term`, so one conversion feeds both sides identically.
fn sparq_solutions(g: &sparq_core::Graph, q: &str) -> Option<Solutions> {
    let r = sparq_engine::query(g, q).ok()?;
    Some(
        r.rows
            .iter()
            .map(|row| {
                r.vars
                    .iter()
                    .zip(row.iter())
                    .filter_map(|(v, t)| {
                        t.as_ref().map(|t| (v.as_str().to_string(), to_difftest(t)))
                    })
                    .collect()
            })
            .collect(),
    )
}

/// The outcome of one cross-oracle comparison attempt. The non-failing outcomes mean
/// DIFFERENT things and are counted separately: an answer compared by value, an answer that
/// needed blank-node isomorphism, an answer that is not differential-testable at all, and an
/// answer nobody could compare. Folding them into one number would report a coverage GAP as
/// if it were a documented non-testable case. [OPUS-5] sq-qcnn.5
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CheckOutcome {
    /// The full answer was compared by VALUE (`multiset_equal` / `order_by_equal`).
    Compared,
    /// Compared up to a consistent bijection of the blank nodes (`solutions_isomorphic`,
    /// RDFC-1.0). Blank-node labels are engine-local, so a bnode-bearing answer is only
    /// comparable up to a bijection — before this wiring it was routed to a counted TRIAGE
    /// bucket and compared by nobody.
    ComparedIsomorphic,
    /// Deliberately NOT comparable: `LIMIT`/`OFFSET` without a total order (the row CHOICE is
    /// arbitrary, so two conformant engines may legitimately differ), a sort key this harness
    /// does not model, or a result that is not a solution sequence.
    SkippedRowChoice,
    /// An `ORDER BY` answer carrying a blank node. `sparq-difftest` has no order-PRESERVING
    /// isomorphism comparator (`canonical_solutions` canonicalises the table as a whole, which
    /// loses the sequence), so the ordered check cannot run on it. Counted on its own so the
    /// gap stays a number rather than reading as green.
    SkippedBnodeOrder,
}

/// Per-oracle tallies of the [`CheckOutcome`]s, printed verbatim in the summary line.
#[derive(Default)]
struct CheckCounts {
    compared: u64,
    compared_iso: u64,
    skipped_row_choice: u64,
    skipped_bnode_order: u64,
}

impl CheckCounts {
    fn record(&mut self, o: CheckOutcome) {
        match o {
            CheckOutcome::Compared => self.compared += 1,
            CheckOutcome::ComparedIsomorphic => self.compared_iso += 1,
            CheckOutcome::SkippedRowChoice => self.skipped_row_choice += 1,
            CheckOutcome::SkippedBnodeOrder => self.skipped_bnode_order += 1,
        }
    }
}

impl std::fmt::Display for CheckCounts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "compared={} iso={} skip(row-choice)={} skip(bnode-order)={}",
            self.compared, self.compared_iso, self.skipped_row_choice, self.skipped_bnode_order
        )
    }
}

/// The value-canonical key of one solution: its `(variable, canonical term key)` pairs, held
/// structurally. This is the SAME keying `sparq_difftest::multiset_equal` compares by, so the
/// repro diff below can never disagree with the verdict above it.
fn row_key(s: &DSolution) -> Vec<(String, String)> {
    s.iter()
        .map(|(v, t)| (v.clone(), canonical_key(t)))
        .collect()
}

/// The first row present in `a` but not in `b` (as a multiset), for the repro message.
fn first_extra_row(a: &Solutions, b: &Solutions) -> Option<String> {
    let mut rest: Vec<Vec<(String, String)>> = b.iter().map(row_key).collect();
    for row in a.iter().map(row_key) {
        match rest.iter().position(|r| *r == row) {
            Some(i) => {
                rest.remove(i);
            }
            None => return Some(format!("{row:?}")),
        }
    }
    None
}

/// FULL-BINDING differential: sparq's complete solution multiset must equal Oxigraph's —
/// every projected binding, duplicates included — not merely its CARDINALITY. The count
/// differential alone cannot see a same-cardinality WRONG ANSWER (a path landing on the wrong
/// endpoints, a `COUNT`/`MIN`/`MAX` off by one, a `BIND`/`VALUES`/sub-select binding the
/// wrong term), which is exactly what the `path`, `aggregate`, `subquery`, `values` and
/// `bind` categories generate.
///
/// `Ok(CheckOutcome::SkippedRowChoice)`:
///   * `LIMIT` / `OFFSET` — SPARQL leaves the row CHOICE arbitrary without a total order, so
///     two conformant engines may return different (equally valid) rows. The `limit` category
///     is exactly that shape; the `order` category's `ORDER BY … LIMIT` is covered by
///     `check_ordered`, which additionally pins the sequence.
///   * a non-solutions result.
///
/// `Ok(CheckOutcome::ComparedIsomorphic)` = a blank node in either answer: compared up to a
/// bijection of the blank nodes instead of by label.
fn check_bindings(g: &sparq_core::Graph, store: &Store, q: &str) -> Result<CheckOutcome, String> {
    if q.contains("LIMIT") || q.contains("OFFSET") {
        return Ok(CheckOutcome::SkippedRowChoice);
    }
    let (Some(sparq), Some(oxi)) = (sparq_solutions(g, q), oxi_solutions(store, q)) else {
        return Ok(CheckOutcome::SkippedRowChoice);
    };
    if solutions_have_blank_nodes(&sparq) || solutions_have_blank_nodes(&oxi) {
        return match solutions_isomorphic(&sparq, &oxi) {
            Ok(true) => Ok(CheckOutcome::ComparedIsomorphic),
            Ok(false) => Err(format!(
                "solution multiset differs UP TO BLANK-NODE ISOMORPHISM \
                 (sparq {} rows / oxi {} rows)",
                sparq.len(),
                oxi.len()
            )),
            // A canonical labelling that cannot be produced is invalid/unsupported oracle
            // output, not agreement — it must be reported, never absorbed.
            Err(e) => Err(format!("blank-node canonicalisation failed: {e}")),
        };
    }
    compare_solutions(&sparq, &oxi).map(|()| CheckOutcome::Compared)
}

/// Multiset equality of two solution sequences, with a repro-sized diff. Bag semantics:
/// `{a, a, b}` and `{a, b, b}` are DIFFERENT answers, so this is sensitive to duplicates.
fn compare_solutions(sparq: &Solutions, oxi: &Solutions) -> Result<(), String> {
    if multiset_equal(sparq, oxi) {
        return Ok(());
    }
    let only_sparq = first_extra_row(sparq, oxi).unwrap_or_else(|| "-".to_string());
    let only_oxi = first_extra_row(oxi, sparq).unwrap_or_else(|| "-".to_string());
    Err(format!(
        "solution MULTISET differs (sparq {} rows / oxi {} rows)\n  \
         only in sparq: {only_sparq}\n  only in oxi  : {only_oxi}",
        sparq.len(),
        oxi.len()
    ))
}

/// The variables an `ORDER BY` clause sorts on, in clause order — `None` when the clause
/// holds anything other than a bare `?v` / `ASC(?v)` / `DESC(?v)`.
///
/// Fail-CLOSED on an expression sort key (`ORDER BY (?a + 1)`): partitioning the sequence by
/// `?a` when the engine sorted by `f(?a)` would SPLIT a genuine tie run into ordered
/// sub-runs and so demand an agreement SPARQL does not require — a manufactured mismatch.
/// The caller treats `None` as a skip-with-count.
fn order_by_vars(q: &str) -> Option<Vec<String>> {
    let mut clause = q.split("ORDER BY").nth(1)?;
    // The ORDER BY clause runs to the end of the query or to the next solution modifier.
    for kw in ["LIMIT", "OFFSET", "VALUES"] {
        if let Some(i) = clause.find(kw) {
            clause = &clause[..i];
        }
    }
    let flat = clause.replace(['(', ')', ','], " ");
    let mut vars = Vec::new();
    for tok in flat.split_whitespace() {
        if let Some(name) = tok.strip_prefix('?') {
            if name.is_empty() {
                return None;
            }
            vars.push(name.to_string());
        } else if !tok.eq_ignore_ascii_case("ASC") && !tok.eq_ignore_ascii_case("DESC") {
            return None;
        }
    }
    if vars.is_empty() {
        None
    } else {
        Some(vars)
    }
}

/// Is the sort key TOTAL over what the projection can distinguish — i.e. does every variable
/// bound anywhere in the answer appear in the sort key? When it does, rows sharing a sort key
/// are the SAME solution, so a `LIMIT` truncation that cuts a tie run cannot make two
/// conformant engines differ.
fn sort_key_is_total(sols: &Solutions, sort_vars: &[String]) -> bool {
    sols.iter()
        .flat_map(|s| s.keys())
        .all(|v| sort_vars.contains(v))
}

/// The whole sequence as canonical row keys, for the repro message.
fn seq_keys(s: &Solutions) -> Vec<Vec<(String, String)>> {
    s.iter().map(row_key).collect()
}

/// ORDER-SENSITIVE differential for `ORDER BY` queries (the multiset check cannot see a
/// reordering): the two engines' sequences must agree up to permutation WITHIN each maximal
/// run of rows equal on the sort key (`sparq_difftest::order_by_equal`), comparing EVERY
/// projected variable — not just the first column against Oxigraph's `?a`, which is all the
/// pre-`sq-qcnn.5` check did.
///
/// SPARQL `ORDER BY` is a PARTIAL order: rows tied on the sort key may appear in any relative
/// order, so an element-for-element sequence comparison would be wrong in general. The
/// equivalence-class comparison is exactly as strong as the spec where the spec pins the
/// order, and no stronger where it does not.
///
/// `Ok(CheckOutcome::SkippedRowChoice)`:
///   * the sort key is not a list of plain variables (see `order_by_vars`);
///   * `LIMIT`/`OFFSET` truncates the sequence and the sort key is NOT total over the
///     projected variables — truncation then cuts a tie run and WHICH tied rows survive is
///     arbitrary, so this stays cardinality-only. With a total sort key each run is a single
///     row, the truncation is deterministic, and the check runs: that is the "add a total
///     tiebreaker" escape hatch, and it is why the `order` category (whose only projected
///     variable IS its sort key) is still compared under its `LIMIT`.
///   * one side produced no solution sequence.
///
/// `Ok(CheckOutcome::SkippedBnodeOrder)`: a blank node in either answer (see [`CheckOutcome`]).
fn check_ordered(g: &sparq_core::Graph, store: &Store, q: &str) -> Result<CheckOutcome, String> {
    let Some(sort_vars) = order_by_vars(q) else {
        return Ok(CheckOutcome::SkippedRowChoice);
    };
    let (Some(sparq), Some(oxi)) = (sparq_solutions(g, q), oxi_solutions(store, q)) else {
        return Ok(CheckOutcome::SkippedRowChoice);
    };
    if (q.contains("LIMIT") || q.contains("OFFSET"))
        && !(sort_key_is_total(&sparq, &sort_vars) && sort_key_is_total(&oxi, &sort_vars))
    {
        return Ok(CheckOutcome::SkippedRowChoice);
    }
    if solutions_have_blank_nodes(&sparq) || solutions_have_blank_nodes(&oxi) {
        return Ok(CheckOutcome::SkippedBnodeOrder);
    }
    compare_ordered(&sparq, &oxi, &sort_vars).map(|()| CheckOutcome::Compared)
}

/// Sort-key-equivalence-class equality of two ordered solution sequences, with a repro-sized
/// diff. Split out of `check_ordered` so the comparison itself is testable against
/// hand-authored sequences (the engines always agree on the generated shapes, so a live test
/// cannot show what a REORDERED answer does).
fn compare_ordered(sparq: &Solutions, oxi: &Solutions, sort_vars: &[String]) -> Result<(), String> {
    let refs: Vec<&str> = sort_vars.iter().map(String::as_str).collect();
    if order_by_equal(sparq, oxi, &refs) {
        return Ok(());
    }
    Err(format!(
        "ORDER BY sequence differs (up to sort-key equivalence classes over {:?})\n  sparq={:?}\n  oxi  ={:?}",
        sort_vars,
        seq_keys(sparq),
        seq_keys(oxi)
    ))
}

/// Which RESULT FORM a query asks for. The three forms need three DIFFERENT oracles — a
/// solution multiset, a BOOLEAN, and a triple SET — and comparing the last two by CARDINALITY
/// is simply wrong: an `ASK` is one result whether it answered `true` or `false`, and a
/// `CONSTRUCT` graph of the right SIZE built from the wrong triples counts the same as the
/// right one. [OPUS-5] sq-qcnn.5
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QueryForm {
    Select,
    Ask,
    /// `CONSTRUCT` / `DESCRIBE` — a graph result.
    Graph,
}

/// The result form of a query, read from the first keyword after the prologue. This is a
/// HARNESS-side classifier over the surface the generator emits (no comments, no `#`-quoted
/// keywords) — both engines still do the real parsing, and a form this misreads becomes a
/// SKIP (`oxi_count` refuses a non-solutions result), never a wrong comparison.
fn query_form(q: &str) -> QueryForm {
    let mut it = q.split_whitespace();
    while let Some(tok) = it.next() {
        let upper = tok.to_ascii_uppercase();
        match upper.as_str() {
            "PREFIX" => {
                it.next();
                it.next();
            }
            "BASE" => {
                it.next();
            }
            _ if upper.starts_with("ASK") => return QueryForm::Ask,
            _ if upper.starts_with("CONSTRUCT") || upper.starts_with("DESCRIBE") => {
                return QueryForm::Graph
            }
            _ => return QueryForm::Select,
        }
    }
    QueryForm::Select
}

/// `ASK` differential: the BOOLEAN itself. A cardinality oracle cannot see this at all —
/// Oxigraph reports one boolean result either way, so `true` and `false` have the same
/// "count". `Ok(None)` = an engine declined the query (a fair skip, not a wrong answer).
// clippy: differential oracle pins oxigraph's legacy Store::query semantics
#[allow(deprecated)]
fn check_ask(g: &sparq_core::Graph, store: &Store, q: &str) -> Result<Option<()>, String> {
    let Ok(sparq) = sparq_engine::ask(g, q) else {
        return Ok(None);
    };
    let Ok(oxigraph::sparql::QueryResults::Boolean(oxi)) = store.query(q) else {
        return Ok(None);
    };
    if sparq != oxi {
        return Err(format!("ASK boolean differs: sparq={sparq} oxigraph={oxi}"));
    }
    Ok(Some(()))
}

/// `CONSTRUCT` / `DESCRIBE` differential: the resulting GRAPH, compared as a triple SET up to
/// a bijection of its blank nodes (`sparq_difftest::graph_isomorphic`, RDFC-1.0). Comparing
/// triple COUNTS would pass a graph of the right size built from the wrong triples; comparing
/// blank-node LABELS would fail two correct answers that merely named their fresh template
/// blank nodes differently (SPARQL 1.1 §16.2 mints a fresh blank node per solution).
// clippy: differential oracle pins oxigraph's legacy Store::query semantics
#[allow(deprecated)]
fn check_graph(g: &sparq_core::Graph, store: &Store, q: &str) -> Result<Option<()>, String> {
    let Ok(sparq) = sparq_engine::construct_or_describe(g, q) else {
        return Ok(None);
    };
    let Ok(oxigraph::sparql::QueryResults::Graph(it)) = store.query(q) else {
        return Ok(None);
    };
    let mut oxi = Vec::new();
    for t in it {
        match t {
            Ok(t) => oxi.push(t),
            Err(_) => return Ok(None),
        }
    }
    let sparq_t: Vec<[DTerm; 3]> = sparq.iter().map(triple_to_difftest).collect();
    let oxi_t: Vec<[DTerm; 3]> = oxi.iter().map(triple_to_difftest).collect();
    match graph_isomorphic(&sparq_t, &oxi_t) {
        Ok(true) => Ok(Some(())),
        Ok(false) => Err(format!(
            "CONSTRUCT/DESCRIBE graph differs up to blank-node isomorphism \
             (sparq {} triples / oxi {} triples)",
            sparq_t.len(),
            oxi_t.len()
        )),
        Err(e) => Err(format!("graph canonicalisation failed: {e}")),
    }
}

pub fn run(seed_start: u64, count: u64, category: &str) {
    // An unknown category would otherwise fall through `gen_query`'s catch-all arm
    // and quietly fuzz a single BGP shape — i.e. a typo in the CI shard matrix would
    // report a green shard that never exercised its surface. Fail loudly instead.
    if category != "all" && !CATEGORIES.contains(&category) {
        eprintln!(
            "error: unknown fuzz category {category:?} — known: all, {}",
            CATEGORIES.join(", ")
        );
        std::process::exit(2);
    }
    // The adjudicated-divergence allowlist (bench/differential-divergences.json) —
    // loaded ONCE; the active state is printed so every CI log shows exactly which
    // classes were skip-with-count vs strict for this run.
    let allow = DivergenceAllowlist::load();
    println!(
        "divergence allowlist ({}): cross-family-eq-type-error={} bnode-iri-inequality={}",
        allow.source,
        if allow.cross_family_eq_type_error {
            "adjudicated (sq-eibog)"
        } else {
            "STRICT"
        },
        if allow.bnode_iri_inequality {
            "adjudicated (sq-ai2wa)"
        } else {
            "STRICT"
        },
    );
    let mut checked = 0u64;
    let mut skipped_unsupported = 0u64;
    let mut adjudicated_cross_family = 0u64;
    let mut adjudicated_bnode_iri = 0u64;
    let mut full_mismatch = 0u64;
    let mut count_mismatch = 0u64;
    // [OPUS-5] sq-qcnn.5: each oracle keeps its OWN split tallies (compared / compared up to
    // blank-node isomorphism / skipped) — a coverage gap must be a visible number, never
    // folded into the documented not-testable bucket.
    let mut bindings = CheckCounts::default();
    let mut ordered = CheckCounts::default();
    let mut ask_checked = 0u64;
    let mut graph_checked = 0u64;
    let mut first_repro: Option<String> = None;

    for seed in seed_start..seed_start + count {
        let mut rng = Rng::new(seed);
        let ttl = gen_graph(&mut rng);
        let q = gen_query(&mut rng, category);

        let g = match sparq_core::Graph::load_str(&ttl, "turtle") {
            Ok(g) => g,
            Err(e) => {
                report_repro(
                    &mut first_repro,
                    seed,
                    &q,
                    &ttl,
                    &format!("sparq load error: {e}"),
                );
                full_mismatch += 1;
                continue;
            }
        };
        // With SPARQ_FUZZ_COMPRESS=1, validate the BLOCK-COMPRESSED store end-to-end:
        // identical results to Oxigraph prove the compressed scan path is correct.
        let g = if std::env::var("SPARQ_FUZZ_COMPRESS").is_ok() {
            g.into_compressed()
        } else {
            g
        };
        // With SPARQ_FUZZ_MMAP=1, save the graph and reopen it MEMORY-MAPPED (perms +
        // numeric cache + the mmap dictionary) — validates the out-of-core read paths
        // (mmap'd term lookup + materialisation) against Oxigraph. One reused temp dir,
        // overwritten each seed (the prior `g2` has dropped, releasing its mmaps).
        let g = if std::env::var("SPARQ_FUZZ_MMAP").is_ok() {
            let dir = std::env::temp_dir().join(format!("sparq_fuzz_mmap_{}", std::process::id()));
            g.save(&dir).expect("save");
            sparq_core::Graph::open(&dir).expect("open")
        } else {
            g
        };
        // With SPARQ_FUZZ_DICTSPILL=1, REBUILD the store through the spilled-dictionary
        // external build (`dict-spill`) with a TINY budget — constant dedup-cache epoch
        // clears and many-run external sorts on every case — and reopen it memory-mapped.
        // Agreement with Oxigraph over thousands of cases validates the spilled id
        // assignment + streamed dictionary end-to-end. The parsed graph is re-serialized
        // to N-Triples (the only format the spill path accepts; the generated Turtle is
        // triple-term-free, so terms round-trip exactly).
        let g = if std::env::var("SPARQ_FUZZ_DICTSPILL").is_ok() {
            let scan = g.store.scan(&[None, None, None]);
            let mut nt = String::new();
            for r in scan.rows.iter() {
                let spo = scan.to_spo(r);
                nt.push_str(&format!(
                    "{} {} {} .\n",
                    g.dict.term(spo[0]),
                    g.dict.term(spo[1]),
                    g.dict.term(spo[2])
                ));
            }
            let dir = std::env::temp_dir().join(format!("sparq_fuzz_dsp_{}", std::process::id()));
            std::fs::remove_dir_all(&dir).ok();
            let cfg = sparq_core::dictspill::SpillConfig {
                mem_budget: 1,
                disk_floor: 0,
            };
            sparq_core::Graph::build_external_spill(nt.as_bytes(), "ntriples", &dir, 64, &cfg)
                .expect("dict-spill build");
            sparq_core::Graph::open(&dir).expect("open")
        } else {
            g
        };
        let store = Store::new().unwrap();
        if let Err(e) = store.load_from_reader(oxigraph::io::RdfFormat::Turtle, ttl.as_bytes()) {
            // Both engines parse the same Turtle; a divergence here is itself a bug.
            report_repro(
                &mut first_repro,
                seed,
                &q,
                &ttl,
                &format!("oxigraph load error: {e}"),
            );
            full_mismatch += 1;
            continue;
        }

        // ASK / CONSTRUCT / DESCRIBE are answered by their OWN oracle — the boolean and the
        // triple set — never by a count (see `QueryForm`). The generator emits only SELECT
        // today (the graph/boolean generator shapes are sq-qcnn.6), so these arms are
        // exercised by this module's unit tests until that lands.
        let form = query_form(&q);
        if form != QueryForm::Select {
            let outcome = match form {
                QueryForm::Ask => check_ask(&g, &store, &q),
                _ => check_graph(&g, &store, &q),
            };
            match outcome {
                Ok(Some(())) => {
                    checked += 1;
                    if form == QueryForm::Ask {
                        ask_checked += 1;
                    } else {
                        graph_checked += 1;
                    }
                }
                // An engine declined the surface — fair to skip (not a wrong answer).
                Ok(None) => skipped_unsupported += 1,
                Err(detail) => {
                    checked += 1;
                    full_mismatch += 1;
                    report_repro(&mut first_repro, seed, &q, &ttl, &detail);
                }
            }
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
            // Before flagging, consult the ADJUDICATED divergence classes loaded from
            // bench/differential-divergences.json (a class absent from that file
            // keeps the strict differential — the file is the source of truth).
            //
            // sq-eibog (cross-family `=`/`!=` type error) is NOT a blind skip: if
            // this is a constant equality filter over the mixed column, re-derive the
            // SPEC-CORRECT count and flag only when sparq disagrees with THAT (a
            // genuine bug), not with Oxigraph's lenient count. Any non-equality shape
            // has `spec_filter_count == None` and keeps the strict differential.
            let spec = if allow.cross_family_eq_type_error {
                spec_filter_count(&store, &q)
            } else {
                None
            };
            match spec {
                Some(spec) if sparq_full == spec => {
                    // sparq matches the spec; Oxigraph is lenient here — the
                    // adjudicated sq-eibog class. Skip WITH COUNT (summary line).
                    adjudicated_cross_family += 1;
                }
                Some(spec) => {
                    full_mismatch += 1;
                    report_repro(
                        &mut first_repro,
                        seed,
                        &q,
                        &ttl,
                        &format!(
                            "sparq query().len()={sparq_full} != SPEC-correct={spec} \
                             (oxigraph={oxi}, lenient cross-family =/!=)"
                        ),
                    );
                }
                None if allow.bnode_iri_inequality && is_bnode_iri_inequality(&store, &q) => {
                    // sq-ai2wa: bnode-vs-IRI `=`/`!=` — adjudicated identity-vs-type-
                    // error ambiguity. Skip WITH COUNT (inert for this generator; see
                    // the JSON entry).
                    adjudicated_bnode_iri += 1;
                }
                None => {
                    full_mismatch += 1;
                    report_repro(
                        &mut first_repro,
                        seed,
                        &q,
                        &ttl,
                        &format!("sparq query().len()={sparq_full} != oxigraph={oxi}"),
                    );
                }
            }
        }

        // FULL-BINDING differential — the answer VALUES, not just how many there are.
        // Run only when the two cardinalities already agree: a cardinality difference
        // has just been adjudicated above (the sq-eibog / sq-ai2wa classes differ in
        // COUNT, so re-reporting them here would double-count one divergence), and a
        // genuine count mismatch has already been reported.
        if sparq_full == oxi {
            match check_bindings(&g, &store, &q) {
                Ok(outcome) => bindings.record(outcome),
                Err(detail) => {
                    full_mismatch += 1;
                    report_repro(&mut first_repro, seed, &q, &ttl, &detail);
                }
            }
        }

        // Order-sensitive differential (ORDER BY queries only): the sequence itself
        // must match — up to permutation within each sort-key equivalence class — not
        // just the cardinality.
        if q.contains("ORDER BY") {
            match check_ordered(&g, &store, &q) {
                Ok(outcome) => ordered.record(outcome),
                Err(detail) => {
                    full_mismatch += 1;
                    report_repro(&mut first_repro, seed, &q, &ttl, &detail);
                }
            }
        }

        // JSON-path differential: serialising directly from ids (query_json, incl. the
        // streaming single-pattern / 2-pattern-join paths) must yield the SAME multiset
        // of bindings as building the QueryResult then serialising. Order-independent.
        if let Ok(qj) = sparq_engine::query_json(&g, &q) {
            let via_result =
                sparq_engine::json::to_sparql_json(&sparq_engine::query(&g, &q).unwrap());
            if bindings_multiset(&qj) != bindings_multiset(&via_result) {
                full_mismatch += 1;
                report_repro(
                    &mut first_repro,
                    seed,
                    &q,
                    &ttl,
                    "query_json bindings multiset != to_sparql_json(query())",
                );
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
         bindings[{bindings}] ordered[{ordered}] ask_checked={ask_checked} graph_checked={graph_checked} \
         adjudicated(cross-family-eq)={adjudicated_cross_family} adjudicated(bnode-iri)={adjudicated_bnode_iri} \
         full_mismatch={full_mismatch} count_mismatch={count_mismatch}",
        seed_start + count
    );
    if let Some(r) = first_repro {
        println!("\nFIRST FAILING CASE:\n{r}");
        std::process::exit(1);
    }
    // NON-VACUITY GUARD (sq-j80vk): a shard that skipped every case as unsupported
    // compared NOTHING while still reporting success — which is exactly how a
    // generator regression would hide (each new category is only worth a shard while
    // both engines actually answer it). A run that asked for seeds and checked none
    // is a harness failure, not a pass.
    if count > 0 && checked == 0 {
        println!(
            "ERROR: fuzz[{category}] checked 0 of {count} seeds — the differential was \
             VACUOUS (every generated query was rejected by an engine)."
        );
        std::process::exit(1);
    }
}

/// Parses SPARQL-JSON results into a canonical, order-independent multiset of bindings
/// (each binding's keys sorted; the rows sorted) so two serialisations can be compared
/// regardless of row / key order.
fn bindings_multiset(json: &str) -> Vec<String> {
    let v: serde_json::Value = serde_json::from_str(json).expect("valid SPARQL JSON");
    let mut rows: Vec<String> = v["results"]["bindings"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|b| {
                    let mut kv: Vec<(String, String)> = b
                        .as_object()
                        .unwrap()
                        .iter()
                        .map(|(k, val)| (k.clone(), val.to_string()))
                        .collect();
                    kv.sort();
                    format!("{kv:?}")
                })
                .collect()
        })
        .unwrap_or_default();
    rows.sort();
    rows
}

fn report_repro(slot: &mut Option<String>, seed: u64, q: &str, ttl: &str, msg: &str) {
    // One line per failing seed (machine-greppable), so differential runs across harness
    // MODES (baseline / mmap / compress / dict-spill) can compare failing-seed SETS, not
    // just counts — a mode is clean iff its set equals the baseline's.
    eprintln!("MISMATCH seed={seed}");
    if slot.is_none() {
        *slot = Some(format!(
            "seed={seed}\n{msg}\n--- query ---\n{q}\n--- graph ---\n{ttl}"
        ));
    }
}

#[cfg(test)]
mod tests {
    //! Regression tests for the differential-oracle fix (sq-eibog): the fuzzer's
    //! `=`/`!=`-over-mixed-literals class must be adjudicated against the SPARQL 1.1
    //! spec, NOT against Oxigraph's lenient cross-family reading. Anchored on the
    //! reproducing fuzz cases AND the W3C `sparql10/expr-equals` data.
    use super::*;
    use oxigraph::model::{Literal, NamedNode, Term};

    fn xsd(t: &str) -> NamedNode {
        NamedNode::new(format!("http://www.w3.org/2001/XMLSchema#{t}")).unwrap()
    }
    fn num(t: &str, v: &str) -> Term {
        Term::Literal(Literal::new_typed_literal(v, xsd(t)))
    }
    fn s(v: &str) -> Term {
        Term::Literal(Literal::new_simple_literal(v))
    }
    fn iri(local: &str) -> Term {
        Term::NamedNode(NamedNode::new(format!("http://ex/{local}")).unwrap())
    }

    /// SPARQL 1.1 §17.4.1.7: cross-family literal `=` (numeric vs string, etc.) is a
    /// TYPE ERROR — `spec_eq` returns `None`, so the row is dropped from BOTH `=` and
    /// `!=`. This is the crux of the fuzz divergence.
    #[test]
    fn cross_family_equality_is_a_type_error() {
        // integer vs plain string → error (the seed=15 / seed=182 class)
        assert_eq!(spec_eq(&num("integer", "71"), &s("s2")), None);
        assert_eq!(spec_eq(&num("int", "84"), &s("s2")), None);
        assert_eq!(spec_eq(&num("double", "15.0"), &s("s2")), None);
        // string vs integer constant → error (the seed=1320 `?v != 47` class)
        assert_eq!(spec_eq(&s("s1"), &num("integer", "47")), None);
        // integer vs the near-2^53 integer used by the generator — SAME family, decided
        assert_eq!(spec_eq(&num("integer", "1"), &num("int", "1")), Some(true));
        assert_eq!(
            spec_eq(&num("integer", "9007199254740992"), &num("integer", "7")),
            Some(false)
        );
    }

    /// EXACT numeric equality (the >2^53 case the fuzzer stresses): distinct integers
    /// that share an f64 must NOT be reported equal. Cross-type int/decimal/double value
    /// equality must hold. Ill-formed lexicals are type errors.
    #[test]
    fn exact_numeric_equality_beyond_f64() {
        // These four are DISTINCT integers that all collapse to the same f64.
        assert_eq!(
            numeric_eq_exact("9007199254740992", "9007199254740993"),
            Some(false)
        );
        assert_eq!(
            numeric_eq_exact("9007199254740992", "9007199254740994"),
            Some(false)
        );
        assert_eq!(
            numeric_eq_exact("9007199254740992", "9007199254740992"),
            Some(true)
        );
        // High-precision decimals that share an f64.
        assert_eq!(
            numeric_eq_exact("0.123456789012345678", "0.123456789012345679"),
            Some(false)
        );
        // Cross-type value equality (integer vs decimal vs double), incl. leading zeros,
        // signs, and exponents.
        assert_eq!(numeric_eq_exact("1", "1.0"), Some(true));
        assert_eq!(numeric_eq_exact("01", "1"), Some(true));
        assert_eq!(numeric_eq_exact("1", "1.0e0"), Some(true));
        assert_eq!(numeric_eq_exact("100", "1.0e2"), Some(true));
        assert_eq!(numeric_eq_exact("0.5", "5.0e-1"), Some(true));
        assert_eq!(numeric_eq_exact("-0", "0"), Some(true));
        assert_eq!(numeric_eq_exact("-23", "23"), Some(false));
        assert_eq!(numeric_eq_exact("15.0", "15"), Some(true));
        // Ill-formed → error.
        assert_eq!(numeric_eq_exact("abc", "1"), None);
        assert_eq!(numeric_eq_exact("1", ""), None);
        // The same case, end to end through spec_eq (the seed=11746 class).
        assert_eq!(
            spec_eq(
                &num("integer", "9007199254740993"),
                &num("integer", "9007199254740992")
            ),
            Some(false)
        );
    }

    /// Same-family and identity cases still DECIDE (true/false), so genuine
    /// over/under-exclusion in those would still trip the fuzzer.
    #[test]
    fn same_family_and_identity_decide() {
        assert_eq!(spec_eq(&s("s3"), &s("s2")), Some(false)); // strings compare
        assert_eq!(spec_eq(&s("s2"), &s("s2")), Some(true));
        assert_eq!(spec_eq(&iri("n0"), &s("s2")), Some(false)); // IRI vs literal: identity
        assert_eq!(spec_eq(&iri("n0"), &iri("n0")), Some(true));
        // unknown-datatype literal, same term → true (sameTerm rescues); different → error
        let my = |v: &str| {
            Term::Literal(Literal::new_typed_literal(
                v,
                NamedNode::new("http://ex/myType").unwrap(),
            ))
        };
        assert_eq!(spec_eq(&my("zzz"), &my("zzz")), Some(true));
        assert_eq!(spec_eq(&my("zzz"), &my("aaa")), None);
        assert_eq!(spec_eq(&my("zzz"), &s("zzz")), None); // cross-family (unknown vs str)
    }

    /// The W3C `sparql10/expr-equals` `!=` case (`result-eq2-2.ttl`) over `data-eq.ttl`.
    /// The bead-relevant, SPEC-UNAMBIGUOUS property — on which §17.4.1.7, the W3C
    /// reference result, and sparq's evaluator all AGREE — is that **every
    /// numeric-vs-non-numeric literal pair is a TYPE ERROR** (absent from the `!=`
    /// result): no numeric literal appears anywhere in `result-eq2-2.ttl`'s 12 rows.
    ///
    /// (The DAWG `!=` reference additionally makes the plain-string-vs-UNKNOWN-datatype
    /// corner — `"zzz" != "zzz"^^:myType` — a decided TRUE, giving 12. sparq's engine
    /// treats that specific corner as an open-world TYPE ERROR instead — a defensible
    /// reading that the *approved* manifest never exercises, since `:eq-2-2` there points
    /// at the `=` query. That corner is irrelevant to this fuzzer: the generator emits no
    /// unknown-datatype literals. This oracle deliberately mirrors sparq's engine so it
    /// never wrongly flags; we assert the agreed numeric-exclusion property below.)
    #[test]
    fn w3c_expr_equals_numeric_pairs_are_type_errors() {
        let numerics = [
            num("integer", "1"),
            num("integer", "01"),
            num("double", "1.0e0"),
            num("double", "1.0"),
            num("double", "1"),
        ];
        let non_numeric_literals = [
            Term::Literal(Literal::new_typed_literal(
                "zzz",
                NamedNode::new("http://example.org/things#myType").unwrap(),
            )),
            s("zzz"),
            s("1"),
        ];
        // Numeric vs any DIFFERENT-value numeric decides; numeric vs any non-numeric
        // literal is a TYPE ERROR (absent from BOTH `=` and `!=`), matching the W3C
        // reference which contains no numeric literal in its `!=` result.
        for a in &numerics {
            for b in &non_numeric_literals {
                assert_eq!(spec_eq(a, b), None, "{a} vs {b} must be a type error");
                assert_eq!(
                    spec_eq(b, a),
                    None,
                    "{b} vs {a} must be a type error (symmetric)"
                );
            }
        }
        // Numeric value-equality still DECIDES (all these are value 1).
        assert_eq!(
            spec_eq(&num("integer", "1"), &num("double", "1.0e0")),
            Some(true)
        );
    }

    /// End-to-end: the exact reproducing case (seed=15). sparq must equal the
    /// SPEC-correct count (2 — the two plain strings), NOT Oxigraph's lenient 8. The
    /// oracle's `spec_filter_count` must AGREE with sparq (so it is NOT flagged).
    #[test]
    fn seed15_numeric_vs_string_ne_matches_spec_not_oxigraph() {
        let ttl = r#"@prefix ex: <http://ex/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
ex:n0 ex:val "84"^^xsd:int .
ex:n1 ex:val "15.0"^^xsd:double .
ex:n3 ex:val 71 .
ex:n4 ex:val "s3" .
ex:n5 ex:val "9007199254740992"^^xsd:integer .
ex:n6 ex:val "s1" .
ex:n11 ex:val "56"^^xsd:int .
ex:n13 ex:val 7 .
"#;
        let q = "PREFIX ex: <http://ex/>\nPREFIX xsd: <http://www.w3.org/2001/XMLSchema#>\nSELECT ?s WHERE { ?s ex:val ?v FILTER(?v != \"s2\") }";
        let g = sparq_core::Graph::load_str(ttl, "turtle").unwrap();
        let sparq_n = sparq_engine::query(&g, q).unwrap().len();
        let store = Store::new().unwrap();
        store
            .load_from_reader(oxigraph::io::RdfFormat::Turtle, ttl.as_bytes())
            .unwrap();
        let oxi = oxi_count(&store, q).unwrap();
        let spec = spec_filter_count(&store, q).expect("recognised equality filter");
        assert_eq!(spec, 2, "only the two plain strings survive != \"s2\"");
        assert_eq!(sparq_n, 2, "sparq is spec-correct");
        assert_eq!(oxi, 8, "oxigraph is lenient (keeps the 6 numeric rows)");
        assert_eq!(sparq_n, spec, "oracle: sparq matches spec → NOT flagged");
        assert_ne!(
            sparq_n, oxi,
            "would be a false-positive under the naive oracle"
        );
    }

    /// All-`xsd:integer` column vs a string constant (`?age != "s4"`, seed=182 class):
    /// EVERY row errors → spec count 0. sparq must return 0, not Oxigraph's full count.
    #[test]
    fn all_integer_column_vs_string_constant_is_empty() {
        let ttl =
            "@prefix ex: <http://ex/> .\nex:n0 ex:age 14 .\nex:n1 ex:age 42 .\nex:n2 ex:age 39 .\n";
        let q = "PREFIX ex: <http://ex/>\nSELECT ?s WHERE { ?s ex:age ?a FILTER(?a != \"s4\") }";
        let g = sparq_core::Graph::load_str(ttl, "turtle").unwrap();
        let sparq_n = sparq_engine::query(&g, q).unwrap().len();
        let store = Store::new().unwrap();
        store
            .load_from_reader(oxigraph::io::RdfFormat::Turtle, ttl.as_bytes())
            .unwrap();
        let spec = spec_filter_count(&store, q).unwrap();
        assert_eq!(spec, 0);
        assert_eq!(sparq_n, 0);
        assert_eq!(oxi_count(&store, q).unwrap(), 3, "oxigraph keeps all 3");
    }

    /// Integer CONSTANT vs the mixed column (`?v != 47`, seed=1320 class): numeric rows
    /// compare numerically (decided), string/unknown rows error (dropped).
    #[test]
    fn integer_constant_vs_mixed_column() {
        let ttl = r#"@prefix ex: <http://ex/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
ex:n0 ex:val 47 .
ex:n1 ex:val 7 .
ex:n2 ex:val "47"^^xsd:int .
ex:n3 ex:val "s1" .
ex:n4 ex:val "s2" .
"#;
        let q = "PREFIX ex: <http://ex/>\nPREFIX xsd: <http://www.w3.org/2001/XMLSchema#>\nSELECT ?s WHERE { ?s ex:val ?v FILTER(?v != 47) }";
        let g = sparq_core::Graph::load_str(ttl, "turtle").unwrap();
        let sparq_n = sparq_engine::query(&g, q).unwrap().len();
        let store = Store::new().unwrap();
        store
            .load_from_reader(oxigraph::io::RdfFormat::Turtle, ttl.as_bytes())
            .unwrap();
        let spec = spec_filter_count(&store, q).unwrap();
        // 47 == 47 (int) and 47 == "47"^^xsd:int → those two are FALSE for !=; 7 → TRUE;
        // "s1"/"s2" → type error (dropped). So exactly 1 row survives (the `7`).
        assert_eq!(spec, 1);
        assert_eq!(sparq_n, 1);
    }

    /// GUARD: the oracle must NOT be a blanket "trust sparq". A genuine over-exclusion
    /// on a SAME-FAMILY comparison (where the spec DOES decide) is still caught: if
    /// sparq under-counted here, `spec_filter_count` (which errs only cross-family)
    /// would disagree and the case would be flagged.
    #[test]
    fn same_family_string_filter_stays_strict() {
        let ttl = "@prefix ex: <http://ex/> .\nex:n0 ex:val \"s1\" .\nex:n1 ex:val \"s2\" .\nex:n2 ex:val \"s3\" .\n";
        let q = "PREFIX ex: <http://ex/>\nSELECT ?s WHERE { ?s ex:val ?v FILTER(?v != \"s2\") }";
        let store = Store::new().unwrap();
        store
            .load_from_reader(oxigraph::io::RdfFormat::Turtle, ttl.as_bytes())
            .unwrap();
        // Pure-string column: spec and oxigraph AGREE (2), so no leniency to absorb.
        assert_eq!(spec_filter_count(&store, q).unwrap(), 2);
        assert_eq!(oxi_count(&store, q).unwrap(), 2);
    }

    /// The COMMITTED allowlist (bench/differential-divergences.json) must parse and
    /// enable exactly the adjudicated classes — pins the JSON ids ⟷ this comparator's
    /// detectors so neither can drift silently (sq-0iqzw). [FABLE-5]
    #[test]
    fn committed_allowlist_enables_exactly_the_adjudicated_classes() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../bench/differential-divergences.json"
        );
        let s = std::fs::read_to_string(path).expect("committed allowlist readable");
        let a = DivergenceAllowlist::from_json(&s, path);
        assert!(a.cross_family_eq_type_error, "sq-eibog class must be listed");
        assert!(a.bnode_iri_inequality, "sq-ai2wa class must be listed");
        // …and the file lists NOTHING this comparator lacks a detector for (an
        // undetectable entry would be a claimed-but-unenforced allowlisting).
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        for c in v["classes"].as_array().expect("classes array") {
            let id = c["id"].as_str().expect("string id");
            assert!(
                ["cross-family-eq-type-error", "bnode-iri-inequality"].contains(&id),
                "class {id:?} in the committed file has no detector in fuzz.rs"
            );
            assert!(
                c["bead"].as_str().is_some_and(|b| b.starts_with("sq-")),
                "class {id:?} must cite its adjudication bead"
            );
        }
    }

    /// An empty / malformed / unknown-id allowlist means STRICT — the FILE, not this
    /// binary, is the source of truth for what is adjudicated; anything the file
    /// does not (recognisably) list fails the differential again.
    #[test]
    fn empty_or_malformed_allowlist_is_strict() {
        let a = DivergenceAllowlist::from_json(r#"{"version":1,"classes":[]}"#, "test");
        assert!(!a.cross_family_eq_type_error);
        assert!(!a.bnode_iri_inequality);
        let a = DivergenceAllowlist::from_json("not json", "test");
        assert!(!a.cross_family_eq_type_error && !a.bnode_iri_inequality);
        // An unknown id is ignored (no detector => that class stays strict), while a
        // recognised sibling entry still applies.
        let a = DivergenceAllowlist::from_json(
            r#"{"classes":[{"id":"some-future-class"},{"id":"bnode-iri-inequality"}]}"#,
            "test",
        );
        assert!(!a.cross_family_eq_type_error);
        assert!(a.bnode_iri_inequality);
    }

    /// sq-ai2wa detector: fires ONLY for a bnode-vs-IRI `=`/`!=` pairing (the narrow
    /// adjudicated class), never for literal comparisons (those are sq-eibog's spec
    /// sub-oracle or a genuine bug) and never for non-equality shapes.
    #[test]
    fn bnode_iri_detector_is_narrow() {
        let ttl = "@prefix ex: <http://ex/> .\nex:n0 ex:val _:b0 .\nex:n1 ex:val \"s1\" .\n";
        let store = Store::new().unwrap();
        store
            .load_from_reader(oxigraph::io::RdfFormat::Turtle, ttl.as_bytes())
            .unwrap();
        let q_iri =
            "PREFIX ex: <http://ex/>\nSELECT ?s WHERE { ?s ex:val ?v FILTER(?v != ex:n0) }";
        assert!(
            is_bnode_iri_inequality(&store, q_iri),
            "bnode-vs-IRI != must be detected"
        );
        let q_str =
            "PREFIX ex: <http://ex/>\nSELECT ?s WHERE { ?s ex:val ?v FILTER(?v != \"s2\") }";
        assert!(
            !is_bnode_iri_inequality(&store, q_str),
            "a literal RHS is NOT the sq-ai2wa class"
        );
        let q_range = "PREFIX ex: <http://ex/>\nSELECT ?s WHERE { ?s ex:age ?a FILTER(?a > 3) }";
        assert!(!is_bnode_iri_inequality(&store, q_range));
    }

    // ── sq-j80vk: invariants of the EXPANDED generator surface ──────────────────

    /// One `(graph, query)` case, built exactly as `run` builds it for a seed.
    fn case(seed: u64, category: &str) -> (String, String) {
        let mut rng = Rng::new(seed);
        let ttl = gen_graph(&mut rng);
        let q = gen_query(&mut rng, category);
        (ttl, q)
    }

    fn oxi_store(ttl: &str) -> Store {
        let store = Store::new().unwrap();
        store
            .load_from_reader(oxigraph::io::RdfFormat::Turtle, ttl.as_bytes())
            .unwrap();
        store
    }

    /// EVERY category must emit SPARQL 1.1 that the INDEPENDENT oracle (Oxigraph)
    /// parses AND evaluates — a query only sparq understands is not a differential at
    /// all. Pinning that sparq answers it too is what keeps a category from silently
    /// degrading into an all-`skipped(unsupported)` shard that reports green while
    /// comparing nothing.
    #[test]
    fn every_category_is_evaluable_by_both_engines() {
        for cat in CATEGORIES {
            for seed in 0..60u64 {
                let (ttl, q) = case(seed, cat);
                oxi_count(&oxi_store(&ttl), &q).unwrap_or_else(|e| {
                    panic!("category {cat} seed {seed}: oxigraph rejected\n{q}\n{e}")
                });
                let g = sparq_core::Graph::load_str(&ttl, "turtle").unwrap();
                sparq_engine::query(&g, &q).unwrap_or_else(|e| {
                    panic!("category {cat} seed {seed}: sparq rejected\n{q}\n{e}")
                });
            }
        }
    }

    /// NON-VACUITY: a category whose queries always return ZERO rows compares 0 to 0
    /// forever. Every category must bind rows on a healthy share of seeds. (`graph`
    /// is the deliberate floor — the harness's dataset has no named graphs, so its
    /// two bare-`GRAPH` shapes are empty BY DESIGN and only the UNION / OPTIONAL
    /// compositions bind rows.)
    #[test]
    fn every_category_binds_rows_on_a_healthy_share_of_seeds() {
        const SEEDS: u64 = 120;
        for cat in CATEGORIES {
            let mut non_empty = 0u64;
            for seed in 0..SEEDS {
                let (ttl, q) = case(seed, cat);
                let g = sparq_core::Graph::load_str(&ttl, "turtle").unwrap();
                if sparq_engine::query(&g, &q).map(|r| r.len()).unwrap_or(0) > 0 {
                    non_empty += 1;
                }
            }
            assert!(
                non_empty * 4 >= SEEDS,
                "category {cat}: only {non_empty}/{SEEDS} seeds bind ANY row — that \
                 shard's differential is near-vacuous"
            );
        }
    }

    /// The oracle-safety invariants documented on `gen_query`. They are what keeps
    /// each adjudicated-divergence sub-oracle (`parse_eq_filter` / `spec_filter_count`
    /// for sq-eibog, `check_ordered`, the sq-ai2wa bnode detector) applicable ONLY to
    /// the shapes it actually models — a category that broke one would manufacture
    /// FALSE mismatches rather than find real ones.
    #[test]
    fn generator_invariants_hold_for_every_category() {
        for cat in CATEGORIES {
            for seed in 0..300u64 {
                let (_, q) = case(seed, cat);
                if !matches!(*cat, "equality" | "filter") {
                    assert!(
                        !q.contains("!=") && !q.contains(" = "),
                        "category {cat}: an `=`/`!=` text outside equality/filter is \
                         re-derived by the sq-eibog sub-oracle, which does not model \
                         this query\n{q}"
                    );
                }
                if *cat != "order" {
                    assert!(
                        !q.contains("ORDER BY"),
                        "category {cat}: an ORDER BY needs a plain-variable sort key (and, \
                         under LIMIT, one covering every projected variable) or it loses \
                         its order-level oracle to a skip\n{q}"
                    );
                } else {
                    // The `order` category must keep BOTH conditions `check_ordered` needs,
                    // or its shard silently degrades to cardinality-only under its `LIMIT`.
                    assert!(
                        order_by_vars(&q).is_some(),
                        "category {cat}: the sort key must be plain variables\n{q}"
                    );
                }
                assert!(
                    !q.contains("_:"),
                    "category {cat}: a blank node makes the sq-ai2wa allowlist class \
                     LIVE — that needs its own adjudication first\n{q}"
                );
                if *cat == "subquery" {
                    assert!(
                        !q.contains("LIMIT"),
                        "category {cat}: a LIMIT inside a joined sub-select makes the \
                         row CHOICE arbitrary, so two conformant engines may return \
                         different (equally valid) counts\n{q}"
                    );
                }
            }
        }
    }

    /// The nightly shard matrix (`.github/workflows/differential.yml`) must carry ONE
    /// SHARD PER CATEGORY: running only `all` hides category-dense bugs (that
    /// workflow's header records the sq-lr2ii case), so a category without its own
    /// shard gets a fraction of the standing exploration it is supposed to get.
    #[test]
    fn every_category_has_a_nightly_shard() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../.github/workflows/differential.yml"
        );
        let wf = std::fs::read_to_string(path).expect("differential.yml readable");
        for cat in CATEGORIES {
            assert!(
                wf.contains(&format!("category: {cat},")),
                "no nightly shard for category {cat:?} in {path}"
            );
        }
    }

    // ── the FULL-BINDING differential (`check_bindings`) ────────────────────

    /// A graph with chains (`ex:p`), a value column and names — enough for the
    /// path / aggregate / subquery / values / bind shapes to bind real rows.
    const BINDING_TTL: &str = r#"@prefix ex: <http://ex/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
ex:n0 ex:p ex:n1 . ex:n0 ex:age 10 . ex:n0 ex:name "nm0" .
ex:n1 ex:p ex:n2 . ex:n1 ex:age 20 . ex:n1 ex:name "nm1" .
ex:n2 ex:p ex:n3 . ex:n2 ex:age 30 . ex:n2 ex:name "nm2" .
ex:n3 ex:age 40 . ex:n3 ex:name "nm3" .
"#;

    /// A neutral-model solution literal, for the hand-authored mutation fixtures.
    fn dsol(pairs: &[(&str, DTerm)]) -> DSolution {
        pairs
            .iter()
            .map(|(v, t)| (v.to_string(), t.clone()))
            .collect()
    }

    fn dint(n: &str) -> DTerm {
        DTerm::Literal {
            lexical: n.to_string(),
            datatype: "http://www.w3.org/2001/XMLSchema#integer".to_string(),
            lang: None,
        }
    }

    fn diri(iri: &str) -> DTerm {
        DTerm::Iri(iri.to_string())
    }

    fn binding_case(q: &str) -> (Solutions, Solutions) {
        let g = sparq_core::Graph::load_str(BINDING_TTL, "turtle").unwrap();
        let store = oxi_store(BINDING_TTL);
        (
            sparq_solutions(&g, q).expect("sparq solutions"),
            oxi_solutions(&store, q).expect("oxigraph solutions"),
        )
    }

    /// MUTATION PROOF that the oracle is not vacuous: a bare `COUNT(*)` returns ONE
    /// row whatever the count is, so the cardinality differential cannot see a wrong
    /// count at all. Perturbing only the VALUE — same row, same variable, same
    /// cardinality — must make the comparator fail. Were `check_bindings` still
    /// count-only, this assertion would not hold.
    #[test]
    fn same_cardinality_wrong_aggregate_value_is_caught() {
        let q = "PREFIX ex: <http://ex/>\nSELECT (COUNT(*) AS ?c) WHERE { ?s ex:age ?a }";
        let (sparq, oxi) = binding_case(q);
        assert_eq!(sparq.len(), 1, "a bare aggregate is always exactly one row");
        compare_solutions(&sparq, &oxi).expect("both engines agree on the real answer");

        // sparq "returns" 3 where the fixture's answer is 4 — one row either way.
        let wrong = vec![dsol(&[("c", dint("3"))])];
        assert_ne!(
            row_key(&wrong[0]),
            row_key(&sparq[0]),
            "the mutation must change the VALUE"
        );
        assert_eq!(wrong.len(), sparq.len(), "the mutation preserves cardinality");
        assert!(
            compare_solutions(&wrong, &oxi).is_err(),
            "a wrong aggregate VALUE at the right cardinality must fail the oracle"
        );
    }

    /// The same mutation proof for a BINDING rather than an aggregate: a property path
    /// that lands on the wrong endpoint keeps the row count and changes only `?o`.
    #[test]
    fn same_cardinality_wrong_path_endpoint_is_caught() {
        let q = "PREFIX ex: <http://ex/>\nSELECT * WHERE { ?s ex:p/ex:p ?o }";
        let (sparq, oxi) = binding_case(q);
        assert!(!sparq.is_empty(), "the fixture must bind rows");
        compare_solutions(&sparq, &oxi).expect("both engines agree on the real answer");

        // Redirect one endpoint to a node that IS in the graph (so only the VALUE is
        // wrong, not the shape of the answer).
        let mut wrong = sparq.clone();
        let o = wrong[0].get_mut("o").expect("?o is projected");
        assert_ne!(
            canonical_key(o),
            canonical_key(&diri("http://ex/n0")),
            "the mutation must change the VALUE"
        );
        *o = diri("http://ex/n0");
        assert_eq!(wrong.len(), sparq.len(), "the mutation preserves cardinality");
        assert!(
            compare_solutions(&wrong, &oxi).is_err(),
            "a wrong path ENDPOINT at the right cardinality must fail the oracle"
        );
    }

    /// SPARQL is bag semantics: same cardinality, same DISTINCT rows, different
    /// multiplicities is still a wrong answer, so the comparator must not degrade
    /// into set comparison.
    #[test]
    fn duplicate_multiplicity_is_significant() {
        let row = |v: &str| dsol(&[("x", diri(v))]);
        let (n0, n1) = ("http://ex/n0", "http://ex/n1");
        let a = vec![row(n0), row(n0), row(n1)];
        let b = vec![row(n0), row(n1), row(n1)];
        assert_eq!(a.len(), b.len());
        assert!(compare_solutions(&a, &b).is_err());
    }

    /// The HARNESS-forced datatype normalisation (`harness_datatype`) must absorb ONLY the
    /// derived-integer collapse Oxigraph's store performs on load — never a difference in
    /// VALUE, and never across primitive numeric types.
    #[test]
    fn harness_datatype_collapses_only_the_derived_integer_types() {
        let xsd_iri = |t: &str| format!("http://www.w3.org/2001/XMLSchema#{}", t);
        let integer = xsd_iri("integer");
        for derived in ["int", "long", "short", "byte", "unsignedInt", "positiveInteger"] {
            assert_eq!(
                harness_datatype(&xsd_iri(derived)),
                integer,
                "{} derives from xsd:integer and must collapse onto it",
                derived
            );
        }
        // The primitive numeric types and everything non-numeric are left ALONE.
        for kept in ["integer", "decimal", "double", "float", "string", "boolean", "dateTime"] {
            assert_eq!(harness_datatype(&xsd_iri(kept)), xsd_iri(kept));
        }
        assert_eq!(harness_datatype("http://example.org/weird"), "http://example.org/weird");
    }

    /// The value-canonical key the comparator decides by (`sparq-difftest`, fed through
    /// `to_difftest`) must absorb the lexical / derived-datatype normalisation Oxigraph's
    /// store performs on load — never a difference in VALUE, and never across primitive
    /// numeric types. The >2^53 integers and 18-digit decimals the generator emits
    /// specifically to defeat an f64 oracle must still separate.
    #[test]
    fn numeric_key_absorbs_lexical_form_but_not_value() {
        let key = |lex: &str, dt: &str| {
            canonical_key(&to_difftest(&Term::Literal(Literal::new_typed_literal(
                lex,
                xsd(dt),
            ))))
        };
        // lexical / derived-type normalisation → SAME key
        assert_eq!(key("-0", "integer"), key("0", "integer"));
        assert_eq!(key("116", "int"), key("116", "integer"));
        assert_eq!(key("07", "integer"), key("7", "integer"));
        assert_eq!(key("113.0", "decimal"), key("113", "decimal"));
        // value differences → DIFFERENT key, including beyond f64 resolution
        assert_ne!(key("9007199254740992", "integer"), key("9007199254740993", "integer"));
        assert_ne!(
            key("0.123456789012345678", "decimal"),
            key("0.123456789012345679", "decimal")
        );
        assert_ne!(key("5", "integer"), key("6", "integer"));
        // primitive numeric types stay distinct
        assert_ne!(key("5", "integer"), key("5", "decimal"));
        assert_ne!(key("5", "decimal"), key("5", "double"));
        // a simple literal is xsd:string under RDF 1.1, and is NOT the integer 5
        assert_eq!(
            canonical_key(&to_difftest(&Term::Literal(Literal::new_simple_literal("5")))),
            key("5", "string")
        );
        assert_ne!(
            canonical_key(&to_difftest(&Term::Literal(Literal::new_simple_literal("5")))),
            key("5", "integer")
        );
        // IRIs and blank nodes carry through as their own kinds.
        assert_eq!(
            canonical_key(&to_difftest(&iri("n0"))),
            canonical_key(&diri("http://ex/n0"))
        );
    }

    /// The value-level oracle must actually RUN on the new categories rather than
    /// skip them — `check_bindings` skipping everything would be a silently green
    /// differential. `LIMIT`/`OFFSET` is the one deliberate skip (the row CHOICE is
    /// arbitrary without a total order; `check_ordered` covers `order`).
    #[test]
    fn check_bindings_compares_the_value_carrying_categories() {
        let g = sparq_core::Graph::load_str(BINDING_TTL, "turtle").unwrap();
        let store = oxi_store(BINDING_TTL);
        for cat in CATEGORIES {
            let mut compared = 0;
            for seed in 0..60u64 {
                let mut rng = Rng::new(seed);
                let _ = gen_graph(&mut rng); // keep the generator's draw sequence
                let q = gen_query(&mut rng, cat);
                match check_bindings(&g, &store, &q) {
                    Ok(CheckOutcome::Compared) => compared += 1,
                    Ok(CheckOutcome::SkippedRowChoice) => {}
                    Ok(other) => panic!(
                        "category {cat} seed {seed}: unexpected {other:?} — gen_query is \
                         documented as bnode-free and emits only SELECT\n{q}"
                    ),
                    Err(e) => panic!("category {cat} seed {seed}: {e}\n{q}"),
                }
            }
            if matches!(*cat, "limit" | "order") {
                assert_eq!(compared, 0, "category {cat} is the documented row-choice skip");
            } else {
                assert!(
                    compared > 0,
                    "category {cat}: check_bindings compared NOTHING — the value-level \
                     differential is vacuous for that shard"
                );
            }
        }
    }

    /// [OPUS-5] sq-qcnn.5: a blank-node answer is now COMPARED — up to a consistent
    /// bijection of the blank nodes (RDFC-1.0, `sparq-difftest`) — instead of being routed to
    /// a counted triage bucket nobody checked. Labels are engine-local, so the two engines
    /// name the same blank node differently and a by-LABEL comparison would fail two correct
    /// answers.
    ///
    /// Mutation guard: give the oracle a store holding DIFFERENT data (a genuine engine
    /// disagreement) and the isomorphism check must go RED. Drop the `solutions_isomorphic`
    /// arm and the first assertion goes red; weaken it to "bnode ⇒ agree" and the second does.
    #[test]
    fn a_blank_node_answer_is_compared_up_to_isomorphism() {
        // The generator is bnode-free, so this shape is written by hand: `_:b` is a real
        // blank node in BOTH engines' answers, with engine-local labels.
        let ttl = "@prefix ex: <http://ex/> .\n_:b ex:age 5 .\nex:n0 ex:age 6 .\n";
        let g = sparq_core::Graph::load_str(ttl, "turtle").unwrap();
        let store = oxi_store(ttl);
        let bnode_q = "PREFIX ex: <http://ex/>\nSELECT ?s WHERE { ?s ex:age ?a }";
        // Precondition: both engines really do bind a blank node here, so the assertion
        // below is about the routing and not about an accidentally-empty answer.
        assert!(solutions_have_blank_nodes(&sparq_solutions(&g, bnode_q).unwrap()));
        assert!(solutions_have_blank_nodes(&oxi_solutions(&store, bnode_q).unwrap()));
        assert_eq!(
            check_bindings(&g, &store, bnode_q).unwrap(),
            CheckOutcome::ComparedIsomorphic
        );

        // RED ON A WRONG ANSWER: an oracle holding a THIRD blank node answers a different
        // (non-isomorphic) table for the same query, and the comparator must say so.
        let divergent = oxi_store("@prefix ex: <http://ex/> .\n_:b ex:age 5 .\n_:c ex:age 7 .\nex:n0 ex:age 6 .\n");
        assert!(
            check_bindings(&g, &divergent, bnode_q).is_err(),
            "a bnode answer that is NOT isomorphic must fail the oracle"
        );

        // The same graph WITHOUT the blank node in the projection is compared by value —
        // so the isomorphism path is reached by the bnode, not by anything else here.
        let ground_q = "PREFIX ex: <http://ex/>\nSELECT ?a WHERE { ex:n0 ex:age ?a }";
        assert_eq!(
            check_bindings(&g, &store, ground_q).unwrap(),
            CheckOutcome::Compared
        );
        // ...and `LIMIT` still lands in the DISTINCT row-choice bucket.
        let limit_q = "PREFIX ex: <http://ex/>\nSELECT ?a WHERE { ?s ex:age ?a } LIMIT 1";
        assert_eq!(
            check_bindings(&g, &store, limit_q).unwrap(),
            CheckOutcome::SkippedRowChoice
        );
    }

    // ── the ORDER-BY EQUIVALENCE-CLASS differential (`check_ordered`) ────────────

    /// `ORDER BY` is a PARTIAL order, so the comparator must permit permutation WITHIN a
    /// sort-key tie run and reject reordering ACROSS runs — over EVERY projected variable.
    ///
    /// The last assertion is the mutation proof of the sq-qcnn.5 generalisation: the
    /// pre-wiring check compared only the FIRST projected column against Oxigraph's `?a`, so
    /// a sequence whose first column is identical and whose SECOND column is wrong passed it.
    /// It must now fail.
    #[test]
    fn ordered_compare_permits_ties_and_catches_reordering_in_any_column() {
        let sort = vec!["k".to_string()];
        let row = |k: &str, v: &str| dsol(&[("k", dint(k)), ("v", diri(v))]);
        let a = vec![row("1", "a"), row("1", "b"), row("2", "c")];
        // Tied rows (?k = 1) permuted: legal, both engines are conformant.
        let tied = vec![row("1", "b"), row("1", "a"), row("2", "c")];
        compare_ordered(&a, &tied, &sort).expect("a tie run may permute");
        // Reordering ACROSS runs is a real ORDER BY violation.
        let reordered = vec![row("2", "c"), row("1", "a"), row("1", "b")];
        assert!(compare_ordered(&a, &reordered, &sort).is_err());
        // The generalisation: same first column, WRONG second column.
        let wrong_second = vec![row("1", "a"), row("1", "b"), row("2", "zzz")];
        assert_eq!(
            wrong_second.iter().map(|s| canonical_key(&s["k"])).collect::<Vec<_>>(),
            a.iter().map(|s| canonical_key(&s["k"])).collect::<Vec<_>>(),
            "the mutation must leave the sort column identical"
        );
        assert!(
            compare_ordered(&a, &wrong_second, &sort).is_err(),
            "a wrong NON-sort column must fail — comparing only the first column would miss it"
        );
    }

    /// `order_by_vars` reads a plain-variable sort key and FAILS CLOSED on anything else:
    /// partitioning by `?a` when the engine sorted by `f(?a)` would manufacture mismatches.
    #[test]
    fn order_by_vars_reads_plain_variables_and_fails_closed_on_expressions() {
        let vars = |q: &str| order_by_vars(q);
        assert_eq!(vars("SELECT ?a WHERE { } ORDER BY ?a"), Some(vec!["a".into()]));
        assert_eq!(
            vars("SELECT ?a WHERE { } ORDER BY DESC(?a) LIMIT 3"),
            Some(vec!["a".into()])
        );
        assert_eq!(
            vars("SELECT * WHERE { } ORDER BY ?a ASC(?b) DESC(?c) OFFSET 1"),
            Some(vec!["a".into(), "b".into(), "c".into()])
        );
        // fail-closed: an expression sort key, and no ORDER BY at all.
        assert_eq!(vars("SELECT ?a WHERE { } ORDER BY (?a + 1)"), None);
        assert_eq!(vars("SELECT ?a WHERE { } ORDER BY STRLEN(?n)"), None);
        assert_eq!(vars("SELECT ?a WHERE { }"), None);
    }

    /// A `LIMIT`ed sequence is comparable only when the sort key is TOTAL over what the
    /// projection can distinguish; otherwise truncation cuts a tie run and WHICH tied rows
    /// survive is arbitrary, so the case stays cardinality-only.
    #[test]
    fn a_truncated_non_total_sort_key_stays_cardinality_only() {
        let sols = vec![dsol(&[("k", dint("1")), ("v", diri("http://ex/a"))])];
        assert!(sort_key_is_total(&sols, &["k".to_string(), "v".to_string()]));
        assert!(!sort_key_is_total(&sols, &["k".to_string()]));
        // Empty answers cannot be distinguished by anything, so any sort key is total.
        assert!(sort_key_is_total(&Solutions::new(), &["k".to_string()]));

        // Live: the `order` category projects ONLY its sort variable, so its `LIMIT` IS
        // compared — that is the "total tiebreaker" escape hatch, and losing it would make
        // the whole `order` shard cardinality-only.
        let g = sparq_core::Graph::load_str(BINDING_TTL, "turtle").unwrap();
        let store = oxi_store(BINDING_TTL);
        let total = "PREFIX ex: <http://ex/>\nSELECT ?a WHERE { ?s ex:age ?a } ORDER BY ?a LIMIT 3";
        assert_eq!(
            check_ordered(&g, &store, total).unwrap(),
            CheckOutcome::Compared
        );
        // The same query projecting a variable the sort key does NOT cover is skipped.
        let partial =
            "PREFIX ex: <http://ex/>\nSELECT ?a ?s WHERE { ?s ex:age ?a } ORDER BY ?a LIMIT 3";
        assert_eq!(
            check_ordered(&g, &store, partial).unwrap(),
            CheckOutcome::SkippedRowChoice
        );
    }

    // ── the ASK / CONSTRUCT oracles (`check_ask` / `check_graph`) ────────────────

    /// The result FORM decides which oracle runs; misreading it is what would send an ASK or
    /// a CONSTRUCT back to the cardinality comparator that cannot see either.
    #[test]
    fn query_form_classifies_the_three_result_forms() {
        assert_eq!(query_form("SELECT * WHERE { ?s ?p ?o }"), QueryForm::Select);
        assert_eq!(
            query_form("PREFIX ex: <http://ex/>\nASK { ?s ex:age ?a }"),
            QueryForm::Ask
        );
        assert_eq!(
            query_form("PREFIX ex: <http://ex/> PREFIX xsd: <http://x/> ASK { ?s ex:age ?a }"),
            QueryForm::Ask,
            "a single-line prologue must not hide the form"
        );
        assert_eq!(
            query_form("BASE <http://ex/>\nCONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }"),
            QueryForm::Graph
        );
        assert_eq!(query_form("DESCRIBE <http://ex/n0>"), QueryForm::Graph);
        // `ASK{` with no space still classifies.
        assert_eq!(query_form("ASK{ ?s ?p ?o }"), QueryForm::Ask);
        // And the count oracle REFUSES a non-solutions result rather than counting it.
        let store = oxi_store(BINDING_TTL);
        assert!(oxi_count(&store, "PREFIX ex: <http://ex/>\nASK { ?s ex:age ?a }").is_err());
        assert!(oxi_count(
            &store,
            "PREFIX ex: <http://ex/>\nCONSTRUCT { ?s ex:a ?a } WHERE { ?s ex:age ?a }"
        )
        .is_err());
    }

    /// `ASK` is compared as a BOOLEAN. A cardinality oracle cannot see this at all: Oxigraph
    /// reports one boolean result whichever way it answered, so `true` and `false` have the
    /// same "count" — which is why `oxi_count` now refuses the form outright.
    ///
    /// Mutation guard: an oracle holding data that makes the ASK answer the OTHER way must
    /// turn the check red. Delete the boolean comparison and this test goes green (verified).
    #[test]
    fn ask_is_compared_as_a_boolean_not_a_count() {
        let g = sparq_core::Graph::load_str(BINDING_TTL, "turtle").unwrap();
        let store = oxi_store(BINDING_TTL);
        let yes = "PREFIX ex: <http://ex/>\nASK { ?s ex:age ?a }";
        let no = "PREFIX ex: <http://ex/>\nASK { ?s ex:missing ?a }";
        // Both engines agree, both ways round.
        assert!(sparq_engine::ask(&g, yes).unwrap());
        assert!(!sparq_engine::ask(&g, no).unwrap());
        assert_eq!(check_ask(&g, &store, yes).unwrap(), Some(()));
        assert_eq!(check_ask(&g, &store, no).unwrap(), Some(()));

        // RED ON A WRONG ANSWER: an oracle whose data makes `no` answer TRUE.
        let divergent = oxi_store("@prefix ex: <http://ex/> .\nex:n0 ex:missing 1 .\n");
        assert!(
            check_ask(&g, &divergent, no).is_err(),
            "a disagreeing ASK boolean must fail the oracle"
        );
    }

    /// `CONSTRUCT` is compared as a triple SET up to a bijection of its blank nodes. Two
    /// correct engines mint DIFFERENT labels for a template blank node (SPARQL 1.1 §16.2
    /// mints a fresh one per solution), so a by-label comparison would fail them both; a
    /// COUNT comparison would instead pass a graph of the right size built from the wrong
    /// triples.
    ///
    /// Mutation guard: an oracle holding data that yields the same NUMBER of triples with
    /// different subjects must turn the check red.
    #[test]
    // clippy: differential oracle pins oxigraph's legacy Store::query semantics
    #[allow(deprecated)]
    fn construct_is_compared_as_a_triple_set_up_to_bnode_isomorphism() {
        let g = sparq_core::Graph::load_str(BINDING_TTL, "turtle").unwrap();
        let store = oxi_store(BINDING_TTL);
        let ground = "PREFIX ex: <http://ex/>\nCONSTRUCT { ?s ex:a ?a } WHERE { ?s ex:age ?a }";
        assert_eq!(check_graph(&g, &store, ground).unwrap(), Some(()));

        // A template blank node: fresh per solution, and labelled independently by each
        // engine — only isomorphism can see that these two answers agree.
        let bnodes =
            "PREFIX ex: <http://ex/>\nCONSTRUCT { _:x ex:a ?a } WHERE { ?s ex:age ?a }";
        let sparq_triples = sparq_engine::construct_or_describe(&g, bnodes).unwrap();
        assert!(
            !sparq_triples.is_empty(),
            "the fixture must construct blank-node-bearing triples"
        );
        assert_eq!(check_graph(&g, &store, bnodes).unwrap(), Some(()));

        // RED ON A WRONG ANSWER, at the SAME triple count: four ages either way, but on
        // different subjects.
        let divergent = oxi_store(
            "@prefix ex: <http://ex/> .\nex:z0 ex:age 10 . ex:z1 ex:age 20 . \
             ex:z2 ex:age 30 . ex:z3 ex:age 40 .\n",
        );
        let sparq_n = sparq_engine::construct_or_describe(&g, ground).unwrap().len();
        let oxi_n = match divergent.query(ground).unwrap() {
            oxigraph::sparql::QueryResults::Graph(it) => it.count(),
            _ => panic!("CONSTRUCT must produce a graph"),
        };
        assert_eq!(sparq_n, oxi_n, "the mutation must preserve the triple COUNT");
        assert!(
            check_graph(&g, &divergent, ground).is_err(),
            "a same-size WRONG graph must fail the oracle"
        );
    }

    /// The split tallies are what keeps a coverage gap a visible NUMBER: each outcome must
    /// land in its own bucket and be printed.
    #[test]
    fn check_counts_keep_the_outcomes_apart() {
        let mut c = CheckCounts::default();
        c.record(CheckOutcome::Compared);
        c.record(CheckOutcome::Compared);
        c.record(CheckOutcome::ComparedIsomorphic);
        c.record(CheckOutcome::SkippedRowChoice);
        c.record(CheckOutcome::SkippedBnodeOrder);
        assert_eq!(
            c.to_string(),
            "compared=2 iso=1 skip(row-choice)=1 skip(bnode-order)=1"
        );
    }

    /// Non-equality shapes are not recognised → the strict Oxigraph differential is
    /// kept (`spec_filter_count` returns `None`), so ordering / join / range-pruning
    /// bugs are unaffected by this fix.
    #[test]
    fn non_equality_shapes_keep_strict_differential() {
        let store = Store::new().unwrap();
        store
            .load_from_reader(
                oxigraph::io::RdfFormat::Turtle,
                b"@prefix ex: <http://ex/> .\nex:n0 ex:age 5 .\n".as_slice(),
            )
            .unwrap();
        assert!(spec_filter_count(
            &store,
            "PREFIX ex: <http://ex/>\nSELECT * WHERE { ?s ex:age ?a FILTER(?a > 3) }"
        )
        .is_none());
        assert!(spec_filter_count(
            &store,
            "PREFIX ex: <http://ex/>\nSELECT * WHERE { ?s ex:age ?a }"
        )
        .is_none());
    }
}
