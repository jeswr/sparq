//! Differential fuzzer: generates random small graphs (with **mixed datatypes** on
//! a numeric predicate — the case that stresses inline-integer range-pruning) and
//! random queries across every plan shape, then checks that
//!   sparq `query().len()`  ==  Oxigraph solution count            (full differential)
//!   sparq `query()` rows    ==  Oxigraph's solution MULTISET      (`check_bindings`)
//!   sparq `count()`         ==  sparq `query().len()`             (count-path differential)
//!
//! The MULTISET check is what makes a same-cardinality WRONG ANSWER visible: a count
//! differential cannot see a property path landing on the wrong endpoints, a `COUNT` /
//! `MIN` / `MAX` returning the wrong number, or a `BIND` / `VALUES` / sub-select binding
//! the wrong term. Every projected binding is compared, duplicates included, order
//! independently; the deliberate exclusions (arbitrary row CHOICE under `LIMIT` /
//! `OFFSET`, and the harness-forced numeric-lexical normalisation) are documented on
//! `check_bindings` / `term_key` and counted in the summary line.
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

use crate::oracle::{self, Oracle, OracleResult, OxigraphOracle, SubprocessOracle};

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
                // `update-*` is the UPDATE differential's namespace in this shared
                // registry: update_fuzz.rs owns those detectors and pins them with its
                // own committed-allowlist test. Not unknown — just not this
                // comparator's, so no warning and (correctly) no effect here.
                Some(other) if other.starts_with("update-") => {}
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
    // sq-avd75: the `OPTIONAL … FILTER(!bound(?v))` negation idiom — the TRIGGER
    // shape of rewrite (b) of the `algebra-rewrite` pass (#1735), which this crate
    // builds ON (see Cargo.toml). Without it the pass's anti-join half had no
    // standing oracle at all.
    "antijoin",
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
/// * **No `ORDER BY` outside the `order` category** — `check_ordered` compares the
///   first projected column against Oxigraph's `?a`.
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
        // ANTI-JOIN (sq-avd75): the `OPTIONAL { B } FILTER(!bound(?v))` negation
        // idiom. This is the TRIGGER shape of rewrite (b) of the `algebra-rewrite`
        // pass (#1735) — `Filter(!bound(?v), LeftJoin(A, B))` → `Minus(A, B)` — which
        // sparq-bench compiles ON, so a TRIGGER shape below is answered by sparq
        // through the REWRITTEN algebra while Oxigraph answers it through the plain
        // OPTIONAL+FILTER. That makes the pass's result-equivalence claim an oracle
        // check on random data rather than only a fixed-fixture assertion
        // (`sparq-engine/tests/rewrite_pass.rs`, which runs on the feature-matrix leg).
        //
        // The `minus` category is deliberately NOT this shape: it generates an
        // EXPLICIT `MINUS`, i.e. the rewrite's OUTPUT operator, and so can never
        // exercise the recogniser.
        //
        // Shapes 0..=2 MEET the rewrite's firing conditions (`?v` certain in B, absent
        // from A, A and B sharing a certain variable, and — since `rewrite_filter`
        // matches `LeftJoin { expression: None }` — an UNFILTERED `OPTIONAL`). Shapes 3
        // and 4 are the two DECLINE shapes: the pass must return the algebra verbatim
        // and the un-rewritten `LeftJoin` path stays oracle-checked. Keeping both sides
        // in ONE category is deliberate — a recogniser that got LOOSER would show up
        // here as a wrong answer, which is the failure the conservatism guards against.
        // `antijoin_category_is_the_rewrite_b_trigger` pins which shape is which.
        "antijoin" => match rng.below(5) {
            // Plain anti-join on the shared subject: named subjects with no age.
            0 => "?s ex:name ?n OPTIONAL { ?s ex:age ?a } FILTER(!bound(?a))".to_string(),
            // Same, over the edge column: named subjects with no outgoing `ex:p`.
            1 => "?s ex:name ?n OPTIONAL { ?s ex:p ?o } FILTER(!bound(?o))".to_string(),
            // Anti-join whose A side is itself a join: `ex:p` targets with no name.
            2 => "?s ex:p ?o OPTIONAL { ?o ex:name ?n } FILTER(!bound(?n))".to_string(),
            // DECLINE (theta): the OPTIONAL carries its own FILTER, which the parser
            // lifts into `LeftJoin { expression: Some(…) }`. `Minus(A, B)` is NOT
            // equivalent to that (the negation is "no age ABOVE the threshold", so the
            // condition has to move into B), and the pass matches `expression: None`
            // only — so it declines. Answering it correctly through the un-rewritten
            // path is exactly what the oracle checks here.
            3 => format!(
                "?s ex:name ?n OPTIONAL {{ ?s ex:age ?a . FILTER(?a > {}) }} FILTER(!bound(?a))",
                rng.below(120)
            ),
            // DECLINE (no shared variable): `ex:absent` is never a subject in
            // `gen_graph` (which emits only `ex:n0..ex:n15`), so B is empty AND shares
            // no variable with A — every A row survives, and a pass that fired here
            // would be WRONG, not merely unhelpful.
            _ => "?s ex:age ?a OPTIONAL { ex:absent ex:name ?n } FILTER(!bound(?n))".to_string(),
        },
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
        oxigraph::sparql::QueryResults::Boolean(_) => Ok(1),
        oxigraph::sparql::QueryResults::Graph(g) => Ok(g.count()),
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

/// Oxigraph's ordered sequence of a single projected variable, as term strings.
// clippy: differential oracle pins oxigraph's legacy Store::query semantics
#[allow(deprecated)]
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
        return Err(format!(
            "ORDER sequence differs\n  sparq={sparq:?}\n  oxi  ={oxi:?}"
        ));
    }
    Ok(())
}

/// One engine's COMPLETE solution multiset, canonicalised for an order-INSENSITIVE,
/// duplicate-PRESERVING comparison: each row is its bound `(variable name, N-Triples
/// term)` pairs sorted by variable, and the rows themselves are sorted. Sorting the
/// pairs makes projection ORDER irrelevant and represents an unbound variable by its
/// absence; sorting the rows (rather than collecting into a set) keeps duplicate rows
/// significant, which is what SPARQL's bag semantics require.
type Solutions = Vec<Vec<(String, String)>>;

/// XSD normalises a numeric literal's LEXICAL form, and Oxigraph's store additionally
/// normalises every type DERIVED from `xsd:integer` (`xsd:int`, `xsd:long`, the
/// unsigned/bounded types, …) down to `xsd:integer` when it loads the graph. sparq
/// keeps the term exactly as written, so the two engines report a different — but
/// equally faithful — datatype for the SAME stored triple. Collapsing the derived
/// integer types onto their `xsd:integer` base makes the two comparable; the primitive
/// numeric types stay DISTINCT, so an `integer`-vs-`decimal`-vs-`double` disagreement
/// on a computed value is still a mismatch.
fn canonical_numeric_datatype(dt: &str) -> &str {
    match dt {
        "http://www.w3.org/2001/XMLSchema#decimal"
        | "http://www.w3.org/2001/XMLSchema#double"
        | "http://www.w3.org/2001/XMLSchema#float" => dt,
        // everything else in the numeric family derives from xsd:integer
        _ => "http://www.w3.org/2001/XMLSchema#integer",
    }
}

/// The comparison key of one bound term: its N-Triples form — so lexical form, datatype,
/// language tag and IRI all matter — EXCEPT that a NUMERIC literal is keyed by its exact
/// decimal VALUE (`decimal_expansion`, arbitrary precision, never f64) under its
/// canonical numeric datatype.
///
/// That single exception is forced by the HARNESS, not a softening of the oracle:
/// Oxigraph's store re-canonicalises numeric literals on LOAD, so a graph containing
/// `"-0"^^xsd:integer` / `"116"^^xsd:int` comes back as `"0"^^xsd:integer` /
/// `"116"^^xsd:integer` from Oxigraph and unchanged from sparq — the same RDF value,
/// differing only in a lexical form / derived datatype Oxigraph no longer has. The same
/// applies to a COMPUTED numeric (`SUM` over integers yields `"113"` vs `"113.0"`):
/// XPath defines the arithmetic VALUE, and SPARQL does not pin which lexical form an
/// implementation writes for it. Everything the spec DOES pin stays compared EXACTLY —
/// the value is compared digit-for-digit (the generator's >2^53 integers and 18-digit
/// decimals, which a shared f64 would wrongly equate, still separate here), and a wrong
/// number, primitive datatype, IRI, language tag or string is still a mismatch.
fn term_key(t: &oxigraph::model::Term) -> String {
    if let oxigraph::model::Term::Literal(l) = t {
        if oxi_family(t) == Some("num") {
            if let Some((neg, int, frac)) = decimal_expansion(l.value()) {
                let sign = if neg { "-" } else { "" };
                let point = if frac.is_empty() { "" } else { "." };
                let dt = canonical_numeric_datatype(l.datatype().as_str());
                return format!("\"{sign}{int}{point}{frac}\"^^<{dt}>");
            }
        }
    }
    t.to_string()
}

fn canon_solutions(mut rows: Solutions) -> Solutions {
    for r in rows.iter_mut() {
        r.sort();
    }
    rows.sort();
    rows
}

/// Oxigraph's full solution multiset. `None` when the query did not produce solutions
/// (ASK / CONSTRUCT — the generator emits neither) or a solution failed to decode.
// clippy: differential oracle pins oxigraph's legacy Store::query semantics
#[allow(deprecated)]
fn oxi_solutions(store: &Store, q: &str) -> Option<Solutions> {
    match store.query(q).ok()? {
        oxigraph::sparql::QueryResults::Solutions(s) => {
            let mut out: Solutions = Vec::new();
            for sol in s {
                let sol = sol.ok()?;
                out.push(
                    sol.iter()
                        .map(|(v, t)| (v.as_str().to_string(), term_key(t)))
                        .collect(),
                );
            }
            Some(canon_solutions(out))
        }
        _ => None,
    }
}

/// sparq's full solution multiset, in the same canonical form as `oxi_solutions`.
/// Both engines materialise the SAME `oxrdf::Term` type, so one `term_key` decides
/// both sides identically.
fn sparq_solutions(g: &sparq_core::Graph, q: &str) -> Option<Solutions> {
    let r = sparq_engine::query(g, q).ok()?;
    let rows = r
        .rows
        .iter()
        .map(|row| {
            r.vars
                .iter()
                .zip(row.iter())
                .filter_map(|(v, t)| t.as_ref().map(|t| (v.as_str().to_string(), term_key(t))))
                .collect()
        })
        .collect();
    Some(canon_solutions(rows))
}

/// A blank node in either engine's ANSWER makes the two multisets incomparable BY LABEL:
/// SPARQL leaves bnode labels implementation-defined, so equal answers can carry
/// different ones. The generator emits no blank nodes (see `gen_query`), so this is a
/// guard, not a routine skip — but it is a REAL coverage gap whenever it fires, and
/// `BindingsCheck::TriageBnode` keeps it visible (see that variant). [OPUS-5] sq-qcnn.7
fn has_bnode(s: &Solutions) -> bool {
    s.iter().flatten().any(|(_, t)| t.starts_with("_:"))
}

/// The outcome of a full-binding comparison attempt. Distinguishing the two NON-compared
/// outcomes is the point: a bnode answer is a coverage GAP (an answer nobody checked),
/// whereas a `LIMIT`-without-total-order answer is a case that is not differential-testable
/// at all. Folding them into one number would report the gap as if it were the latter.
/// [OPUS-5] sq-qcnn.7
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BindingsCheck {
    /// The full solution multiset was compared.
    Compared,
    /// Deliberately NOT comparable: `LIMIT`/`OFFSET` without a total order (the row CHOICE
    /// is arbitrary, so two conformant engines may legitimately differ), or a result that is
    /// not a solution sequence.
    SkippedRowChoice,
    /// A blank node in either answer. Comparing these needs cross-oracle blank-node
    /// ISOMORPHISM (canonical labelling), which `sparq-difftest`'s `iso` module now provides
    /// but which reaches this harness only with the difftest wiring (`sq-qcnn.5`). Until
    /// then this is routed to an explicitly COUNTED triage bucket — never a silent skip, and
    /// never folded into `SkippedRowChoice`, so a run that stopped comparing bnode answers
    /// shows up as a number rather than as green.
    TriageBnode,
}

/// The first row present in `a` but not in `b` (as a multiset), for the repro message.
fn first_extra_row(a: &Solutions, b: &Solutions) -> Option<String> {
    let mut rest = b.clone();
    for row in a {
        match rest.iter().position(|r| r == row) {
            Some(i) => {
                rest.remove(i);
            }
            None => return Some(format!("{row:?}")),
        }
    }
    None
}

/// FULL-BINDING differential: sparq's complete solution multiset must equal Oxigraph's
/// — every projected binding, duplicates included — not merely its CARDINALITY. The
/// count differential alone cannot see a same-cardinality WRONG ANSWER (a path landing
/// on the wrong endpoints, a `COUNT`/`MIN`/`MAX` off by one, a `BIND`/`VALUES`/
/// sub-select binding the wrong term), which is exactly what the `path`, `aggregate`,
/// `subquery`, `values` and `bind` categories generate.
///
/// `Ok(BindingsCheck::SkippedRowChoice)` = deliberately NOT compared (surfaced as
/// `bindings_skipped` in the summary, so a shard that compared nothing is visible rather
/// than silently green):
///   * `LIMIT` / `OFFSET` without a total order — SPARQL leaves the row CHOICE
///     arbitrary there, so two conformant engines may return different (equally valid)
///     rows. The `limit` category is exactly that shape; the `order` category's
///     `ORDER BY … LIMIT` is covered by `check_ordered`, which compares the projected
///     sequence element-for-element (strictly stronger than this multiset check).
///   * a non-solutions result.
///
/// `Ok(BindingsCheck::TriageBnode)` = a blank node in either answer (see `has_bnode`):
/// its own COUNTED triage bucket, not a silent skip and not a row-choice skip.
fn check_bindings(g: &sparq_core::Graph, store: &Store, q: &str) -> Result<BindingsCheck, String> {
    if q.contains("LIMIT") || q.contains("OFFSET") {
        return Ok(BindingsCheck::SkippedRowChoice);
    }
    let (Some(sparq), Some(oxi)) = (sparq_solutions(g, q), oxi_solutions(store, q)) else {
        return Ok(BindingsCheck::SkippedRowChoice);
    };
    if has_bnode(&sparq) || has_bnode(&oxi) {
        return Ok(BindingsCheck::TriageBnode);
    }
    compare_solutions(&sparq, &oxi).map(|()| BindingsCheck::Compared)
}

/// Multiset equality of two canonicalised solution sets, with a repro-sized diff.
/// Comparing the SORTED row vectors (not sets) makes this sensitive to DUPLICATES —
/// SPARQL's bag semantics mean `{a, a, b}` and `{a, b, b}` are different answers.
fn compare_solutions(sparq: &Solutions, oxi: &Solutions) -> Result<(), String> {
    if sparq == oxi {
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

/// The outcome of one oracle-vs-oracle comparison. `Skipped` and `Broken` are COUNTED buckets,
/// printed with the others: an oracle pair that silently stopped comparing anything must show up
/// as a number, and an oracle that stopped WORKING must show up as a different number.
#[derive(Debug, Clone, PartialEq, Eq)]
enum OracleCross {
    /// Both oracles produced the same result under the neutral value-canonical comparison.
    Agree,
    /// The oracles produced different results, with a repro-sized description.
    Disagree(String),
    /// Not comparable: one oracle DECLINED the query ([`oracle::OracleError::Unsupported`]), the
    /// shape is not differential-testable (`LIMIT`/`OFFSET` without a total order), the two
    /// returned different result FORMS, or an answer carries blank nodes.
    Skipped,
    /// An oracle was not operational: a spawn failure, timeout, crash, garbage stdout
    /// ([`oracle::OracleError::Backend`]) or a rejection of the input graph
    /// ([`oracle::OracleError::Data`]). A HARNESS fault, deliberately not the same bucket as
    /// `Skipped` — "the oracle declined" and "the oracle broke" must not be the same number, or a
    /// dead second oracle compares zero cases and the run still reads green.
    Broken(String),
}

/// Consult one oracle for [`cross_check_oracles`], mapping a failure onto the cross-check buckets.
///
/// Only a DECLINE is a skip. A `Data` rejection is itself a finding (both oracles are handed the
/// *same* serialisation, so one refusing it is a fault of that oracle) and a `Backend` failure
/// means the oracle is not operational at all; collapsing either into `Skipped` is exactly how a
/// misconfigured or dead oracle would compare nothing while the run stayed successful.
fn eval_for_cross(o: &dyn Oracle, data: &str, query: &str) -> Result<OracleResult, OracleCross> {
    match o.eval(data, query) {
        Ok(r) => Ok(r),
        Err(oracle::OracleError::Unsupported(_)) => Err(OracleCross::Skipped),
        Err(e @ (oracle::OracleError::Data(_) | oracle::OracleError::Backend(_))) => {
            Err(OracleCross::Broken(format!("{}: {}", o.name(), e)))
        }
    }
}

/// Compare two oracles against each other over the **neutral** result form (`sparq-difftest`),
/// never through either engine's own value code.
///
/// Honest bounds on what a disagreement here means — this is the whole reason it is non-fatal:
/// Jena and rdflib are *implementations*, not the specification, so "the oracles disagree" is
/// evidence of a spec ambiguity or of one engine's non-conformance and is **not attributable to
/// sparq**. The N-way triage that turns such a case into a reviewed, checked-in allowlist entry
/// with a written reason (design record §5.2) is a separate bead; until it exists this function's
/// job is to make the divergence VISIBLE, not to adjudicate it.
///
/// Both sides go through the trait's `eval(data, query)`, so the in-process oracle re-parses the
/// graph here rather than reusing the store the caller already loaded. That is a deliberate trade:
/// it keeps this function a pure two-oracle comparison with no Oxigraph-specific coupling, and the
/// cost is a second parse of a small generated graph on an opt-in path whose other side is a
/// process spawn — which dominates it.
fn cross_check_oracles(a: &dyn Oracle, b: &dyn Oracle, data: &str, query: &str) -> OracleCross {
    // Same exclusion as `check_bindings`: under `LIMIT`/`OFFSET` without a total order the row
    // CHOICE is arbitrary, so two conformant engines may legitimately return different rows.
    if query.contains("LIMIT") || query.contains("OFFSET") {
        return OracleCross::Skipped;
    }
    // An oracle that could not answer has not answered wrongly — but only a DECLINE is a skip;
    // a break lands in `Broken` (see `eval_for_cross`). Evaluated in sequence, not as a pair, so
    // a decline by the cheap in-process oracle (passed as `a`) skips the case without paying for
    // the expensive one's process spawn.
    let ra = match eval_for_cross(a, data, query) {
        Ok(r) => r,
        Err(cross) => return cross,
    };
    let rb = match eval_for_cross(b, data, query) {
        Ok(r) => r,
        Err(cross) => return cross,
    };
    match (&ra, &rb) {
        (OracleResult::Solutions(x), OracleResult::Solutions(y)) => {
            // Blank-node labels are engine-local, so labelled comparison across engines is
            // meaningless. Comparing these needs RDFC-1.0 canonical labelling
            // (`sparq_difftest::iso`), which reaches this harness with the difftest wiring node —
            // until then this is a counted skip, matching `BindingsCheck::TriageBnode`'s posture.
            if sparq_difftest::solutions_have_blank_nodes(x)
                || sparq_difftest::solutions_have_blank_nodes(y)
            {
                return OracleCross::Skipped;
            }
            if sparq_difftest::multiset_equal(x, y) {
                OracleCross::Agree
            } else {
                OracleCross::Disagree(format!(
                    "solution MULTISET differs between oracles: {} has {} rows, {} has {} rows",
                    a.name(),
                    x.len(),
                    b.name(),
                    y.len()
                ))
            }
        }
        (OracleResult::Boolean(x), OracleResult::Boolean(y)) => {
            if x == y {
                OracleCross::Agree
            } else {
                OracleCross::Disagree(format!(
                    "ASK differs between oracles: {}={x}, {}={y}",
                    a.name(),
                    b.name()
                ))
            }
        }
        (OracleResult::Graph(x), OracleResult::Graph(y)) => {
            // `CONSTRUCT`/`DESCRIBE` graphs are compared up to blank-node ISOMORPHISM (RDFC-1.0),
            // never by label. An `Err` is the canonicaliser refusing (e.g. a poison graph hitting
            // the hash-n-degree call limit) — a counted skip, not a divergence.
            match sparq_difftest::graph_isomorphic(x, y) {
                Ok(true) => OracleCross::Agree,
                Ok(false) => OracleCross::Disagree(format!(
                    "CONSTRUCT graph differs between oracles (up to bnode isomorphism): \
                     {} has {} triples, {} has {} triples",
                    a.name(),
                    x.len(),
                    b.name(),
                    y.len()
                )),
                Err(_) => OracleCross::Skipped,
            }
        }
        // Different result FORMS — one oracle answered a different question than the other, which
        // is a harness/protocol fault rather than a semantic divergence this node can judge.
        _ => OracleCross::Skipped,
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
    // [SONNET-4.6] sq-qcnn.8 — the oracles. Oxigraph is always present (in-process); a SECOND,
    // independent oracle is opt-in: absent `SPARQ_FUZZ_ORACLE2_CMD` this run makes exactly the
    // comparisons it always made and spawns nothing — no JVM, no Python, no extra process.
    let oxi_oracle = OxigraphOracle::new();
    let oracle2 = SubprocessOracle::from_env();
    match &oracle2 {
        Some(o) => println!("second oracle: {}", o.describe()),
        None => println!(
            "second oracle: disabled (set {} to enable — see crates/sparq-bench/oracles/README.md)",
            oracle::ENV_CMD
        ),
    }

    let mut checked = 0u64;
    let mut skipped_unsupported = 0u64;
    let mut adjudicated_cross_family = 0u64;
    let mut adjudicated_bnode_iri = 0u64;
    let mut full_mismatch = 0u64;
    let mut count_mismatch = 0u64;
    let mut bindings_checked = 0u64;
    let mut bindings_skipped = 0u64;
    // [OPUS-5] sq-qcnn.7: bnode answers get their OWN counter — a coverage gap must be a
    // visible number, not an entry in the row-choice skip bucket.
    let mut bindings_triage_bnode = 0u64;
    let mut first_repro: Option<String> = None;
    // [SONNET-4.6] sq-qcnn.8 — second-oracle buckets. All four are printed; none is silent, and
    // `o2_broken` (the oracle itself failed) is deliberately NOT folded into `o2_skipped`.
    let mut o2_agree = 0u64;
    let mut o2_disagree = 0u64;
    let mut o2_skipped = 0u64;
    let mut o2_broken = 0u64;
    let mut first_o2_divergence: Option<String> = None;
    let mut first_o2_fault: Option<String> = None;

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
        // Loading goes through the `Oracle` impl (sq-qcnn.8) rather than open-coding
        // `Store::new()` + `load_from_reader` here, so Oxigraph's oracle behaviour lives in ONE
        // place and a second engine can stand beside it. The same two steps; a load failure is
        // still a reported mismatch, and a `Store::new()` failure — previously an `unwrap()`
        // panic — now takes that same path rather than aborting the shard.
        let store = match oxi_oracle.load(&ttl) {
            Ok(s) => s,
            Err(e) => {
                // Both engines parse the same Turtle; a divergence here is itself a bug.
                report_repro(&mut first_repro, seed, &q, &ttl, &format!("{e}"));
                full_mismatch += 1;
                continue;
            }
        };

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
                Ok(BindingsCheck::Compared) => bindings_checked += 1,
                Ok(BindingsCheck::SkippedRowChoice) => bindings_skipped += 1,
                Ok(BindingsCheck::TriageBnode) => bindings_triage_bnode += 1,
                Err(detail) => {
                    full_mismatch += 1;
                    report_repro(&mut first_repro, seed, &q, &ttl, &detail);
                }
            }
        }

        // SECOND-ORACLE cross-check (sq-qcnn.8) — opt-in, and deliberately ORACLE-vs-ORACLE.
        // sparq has already been compared against Oxigraph above, so the information this adds
        // is precisely the one the single-oracle harness cannot produce: a case where sparq and
        // Oxigraph AGREE and an engine of unrelated lineage does not. That is the shape a bug the
        // two Rust engines SHARE would take. A DISAGREEMENT is NOT a sparq verdict and is never
        // fatal here — see `cross_check_oracles`. A BROKEN oracle is a different matter: that is
        // a harness fault and it fails the run at the end.
        if let Some(o2) = &oracle2 {
            match cross_check_oracles(&oxi_oracle, o2, &ttl, &q) {
                OracleCross::Agree => o2_agree += 1,
                OracleCross::Skipped => o2_skipped += 1,
                // The oracle broke rather than declined: counted apart, and fatal at the end of
                // the run (below) — a non-operational oracle compared nothing and must not pass.
                OracleCross::Broken(detail) => {
                    o2_broken += 1;
                    if first_o2_fault.is_none() {
                        first_o2_fault = Some(format!("seed={seed}\nquery:\n{q}\n{detail}"));
                    }
                }
                OracleCross::Disagree(detail) => {
                    o2_disagree += 1;
                    if first_o2_divergence.is_none() {
                        first_o2_divergence = Some(format!(
                            "seed={seed}\nquery:\n{q}\ngraph:\n{ttl}\n{detail}"
                        ));
                    }
                }
            }
        }

        // Order-sensitive differential (ORDER BY queries only): the sequence itself
        // must match, not just the cardinality.
        if q.contains("ORDER BY") {
            if let Err(detail) = check_ordered(&g, &store, &q) {
                full_mismatch += 1;
                report_repro(&mut first_repro, seed, &q, &ttl, &detail);
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
         bindings_checked={bindings_checked} bindings_skipped(row-choice)={bindings_skipped} \
         bindings_triage(bnode)={bindings_triage_bnode} \
         adjudicated(cross-family-eq)={adjudicated_cross_family} adjudicated(bnode-iri)={adjudicated_bnode_iri} \
         full_mismatch={full_mismatch} count_mismatch={count_mismatch}",
        seed_start + count
    );
    // A SEPARATE line, printed only when a second oracle ran: `scripts/ci-file-differential-
    // failure.py` scrapes the LAST line starting with `fuzz[`, so the summary above must stay
    // byte-compatible for consumers that predate this bucket.
    if let Some(o2) = &oracle2 {
        println!(
            "oracle2[{}] agree={o2_agree} disagree={o2_disagree} skipped={o2_skipped} broken={o2_broken}",
            o2.name()
        );
        if let Some(d) = &first_o2_divergence {
            // Reported, never suppressed — and NOT a failure. Two oracles disagreeing means a
            // spec ambiguity or one engine's non-conformance; which one is a human, reviewed
            // call. Recording those decisions in a checked-in reviewed allowlist (and only then
            // gating on the bucket) is design record §5.2 and its own follow-on bead.
            println!(
                "\nFIRST ORACLE-vs-ORACLE DIVERGENCE (triage — NOT a sparq verdict, NOT a failure):\n{d}"
            );
        }
        if let Some(f) = &first_o2_fault {
            // Unlike a divergence, this IS a failure: a spawn error, timeout, crash, garbage
            // stdout or a rejected input graph means the configured oracle was not operational,
            // so the cases it was consulted on compared NOTHING. Printed here and exited on
            // below, after any sparq repro has also been printed.
            println!(
                "\nERROR: second oracle {} was NOT OPERATIONAL — {} of {} consulted cases failed \
                 with a backend/data fault (a harness failure, not a skip):\n{}",
                o2.name(),
                o2_broken,
                o2_agree + o2_disagree + o2_skipped + o2_broken,
                f
            );
        }
    }
    if let Some(r) = first_repro {
        println!("\nFIRST FAILING CASE:\n{r}");
        std::process::exit(1);
    }
    // A broken second oracle fails the run (reported just above). Deliberately after the repro
    // print so a sparq divergence is never hidden by a harness fault.
    if o2_broken > 0 {
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
                // `update-*` classes belong to update_fuzz.rs, whose own
                // committed-allowlist test pins them the same way.
                id.starts_with("update-")
                    || ["cross-family-eq-type-error", "bnode-iri-inequality"].contains(&id),
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
                        "category {cat}: check_ordered compares the first projected \
                         column against Oxigraph's `?a`\n{q}"
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

    /// sq-avd75: the `antijoin` category exists for ONE reason — to put rewrite (b)
    /// of the `algebra-rewrite` pass (#1735) under the nightly Oxigraph oracle. It
    /// only does that if the shapes it emits are shapes the recogniser ACTUALLY
    /// fires on: a grammar that merely *looks* like the `OPTIONAL … FILTER(!bound)`
    /// idiom but that the (deliberately conservative) pass declines would fuzz the
    /// un-rewritten path forever while reporting a green `antijoin` shard.
    ///
    /// So this asserts the split directly, on the PLAN: the three trigger shapes must
    /// reach an anti-join (`Minus` in `EXPLAIN`), and the two shapes the pass is
    /// documented to DECLINE — a theta `OPTIONAL` (its FILTER is lifted into
    /// `LeftJoin { expression: Some(…) }`, where `Minus(A, B)` is not equivalent) and
    /// an `A`/`B` pair sharing no variable — must NOT. Firing on either would be
    /// unsound, not merely unhelpful, so both directions are pinned.
    ///
    /// MUTATION GUARD: weaken a trigger shape so a firing condition fails (drop the
    /// shared `?s`, bind the `!bound` variable in A, put a FILTER in its OPTIONAL) and
    /// the `fired` half goes red; drop either decline shape and the `declined` half
    /// loses its case — assert the counts separately so neither can silently vanish.
    #[test]
    fn antijoin_category_is_the_rewrite_b_trigger() {
        let (mut fired, mut declined_theta, mut declined_disjoint) = (0u64, 0u64, 0u64);
        for seed in 0..200u64 {
            let (ttl, q) = case(seed, "antijoin");
            assert!(
                q.contains("OPTIONAL {") && q.contains("FILTER(!bound(?"),
                "seed {seed}: not the OPTIONAL+!bound idiom\n{q}"
            );
            let g = sparq_core::Graph::load_str(&ttl, "turtle").unwrap();
            let ex = sparq_engine::explain(&g, &q)
                .unwrap_or_else(|e| panic!("seed {seed}: EXPLAIN failed: {e}\n{q}"));
            // The two DECLINE shapes, by their markers: a SECOND `FILTER` (the one
            // inside the OPTIONAL — the theta shape) and the `ex:absent` subject (the
            // no-shared-variable shape).
            let theta = q.matches("FILTER").count() == 2;
            let disjoint = q.contains("ex:absent");
            if theta || disjoint {
                assert!(
                    !ex.contains("Minus"),
                    "seed {seed}: the pass must DECLINE this shape — `Minus(A, B)` is \
                     not equivalent to it\n{q}\n{ex}"
                );
                if theta {
                    declined_theta += 1;
                } else {
                    declined_disjoint += 1;
                }
            } else {
                assert!(
                    ex.contains("Minus"),
                    "seed {seed}: the trigger shape did NOT reach an anti-join, so this \
                     shard fuzzes the un-rewritten path and rewrite (b) stays \
                     oracle-less\n{q}\n{ex}"
                );
                fired += 1;
            }
        }
        assert!(fired > 0, "no seed exercised the REWRITTEN anti-join path");
        assert!(declined_theta > 0, "no seed exercised the theta DECLINE path");
        assert!(
            declined_disjoint > 0,
            "no seed exercised the no-shared-variable DECLINE path"
        );
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
        let wrong = vec![vec![(
            "c".to_string(),
            "\"3\"^^<http://www.w3.org/2001/XMLSchema#integer>".to_string(),
        )]];
        assert_ne!(wrong, sparq, "the mutation must change the VALUE");
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
        let o = wrong[0]
            .iter_mut()
            .find(|(v, _)| v == "o")
            .expect("?o is projected");
        assert_ne!(o.1, "<http://ex/n0>", "the mutation must change the VALUE");
        o.1 = "<http://ex/n0>".to_string();
        let wrong = canon_solutions(wrong);
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
        let row = |v: &str| vec![("x".to_string(), v.to_string())];
        let (n0, n1) = ("<http://ex/n0>", "<http://ex/n1>");
        let a = canon_solutions(vec![row(n0), row(n0), row(n1)]);
        let b = canon_solutions(vec![row(n0), row(n1), row(n1)]);
        assert_eq!(a.len(), b.len());
        assert!(compare_solutions(&a, &b).is_err());
    }

    /// The harness-forced numeric key (see `term_key`) must absorb ONLY the lexical /
    /// derived-datatype normalisation Oxigraph's store performs on load — never a
    /// difference in VALUE, and never across primitive numeric types. The >2^53
    /// integers and 18-digit decimals the generator emits specifically to defeat an
    /// f64 oracle must still separate.
    #[test]
    fn numeric_key_absorbs_lexical_form_but_not_value() {
        let key = |lex: &str, dt: &str| {
            term_key(&Term::Literal(Literal::new_typed_literal(lex, xsd(dt))))
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
        // a non-numeric literal keeps its exact N-Triples form
        assert_eq!(
            term_key(&Term::Literal(Literal::new_simple_literal("5"))),
            "\"5\""
        );
    }

    /// The value-level oracle must actually RUN on the new categories rather than
    /// skip them — `check_bindings` returning `Ok(false)` for everything would be a
    /// silently green differential. `LIMIT`/`OFFSET` is the one deliberate skip (the
    /// row CHOICE is arbitrary without a total order; `check_ordered` covers `order`).
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
                    Ok(BindingsCheck::Compared) => compared += 1,
                    Ok(BindingsCheck::SkippedRowChoice) => {}
                    Ok(BindingsCheck::TriageBnode) => panic!(
                        "category {cat} seed {seed}: the generator emitted a blank-node answer \
                         — gen_query is documented as bnode-free\n{q}"
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

    /// [OPUS-5] sq-qcnn.7: a blank-node answer must reach its OWN counted triage bucket,
    /// NOT the row-choice skip bucket. Those two numbers mean different things — a
    /// row-choice skip is a case that is not differential-testable at all, a bnode skip is
    /// an answer nobody compared — and merging them would report the coverage gap as if it
    /// were the documented non-testable case.
    ///
    /// This is the mutation guard for that split: fold `TriageBnode` back into
    /// `SkippedRowChoice` (or drop the `has_bnode` arm entirely) and this test goes red.
    #[test]
    fn a_blank_node_answer_is_counted_triage_not_a_row_choice_skip() {
        // The generator is bnode-free, so this shape is written by hand: `_:b` is a real
        // blank node in BOTH engines' answers, with engine-local labels.
        let ttl = "@prefix ex: <http://ex/> .\n_:b ex:age 5 .\nex:n0 ex:age 6 .\n";
        let g = sparq_core::Graph::load_str(ttl, "turtle").unwrap();
        let store = oxi_store(ttl);
        let bnode_q = "PREFIX ex: <http://ex/>\nSELECT ?s WHERE { ?s ex:age ?a }";
        // Precondition: both engines really do bind a blank node here, so the assertion
        // below is about the routing and not about an accidentally-empty answer.
        assert!(has_bnode(&sparq_solutions(&g, bnode_q).unwrap()));
        assert!(has_bnode(&oxi_solutions(&store, bnode_q).unwrap()));
        assert_eq!(
            check_bindings(&g, &store, bnode_q).unwrap(),
            BindingsCheck::TriageBnode
        );

        // The same graph WITHOUT the blank node in the projection is compared normally —
        // so the bucket is reached by the bnode, not by anything else about this case.
        let ground_q = "PREFIX ex: <http://ex/>\nSELECT ?a WHERE { ex:n0 ex:age ?a }";
        assert_eq!(
            check_bindings(&g, &store, ground_q).unwrap(),
            BindingsCheck::Compared
        );
        // ...and `LIMIT` still lands in the DISTINCT row-choice bucket.
        let limit_q = "PREFIX ex: <http://ex/>\nSELECT ?a WHERE { ?s ex:age ?a } LIMIT 1";
        assert_eq!(
            check_bindings(&g, &store, limit_q).unwrap(),
            BindingsCheck::SkippedRowChoice
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

    // ── SECOND-ORACLE cross-check (sq-qcnn.8) ────────────────────────────────────────

    const O2_GRAPH: &str = "@prefix ex: <http://ex/> .\nex:a ex:p 1 .\nex:b ex:p 2 .\n";

    /// An oracle that returns a canned answer — the only way to stage a controlled
    /// oracle-vs-oracle DISAGREEMENT without a second engine installed on the box.
    struct StubOracle(&'static str, Result<OracleResult, crate::oracle::OracleError>);

    impl Oracle for StubOracle {
        fn name(&self) -> &str {
            self.0
        }
        fn eval(&self, _: &str, _: &str) -> Result<OracleResult, crate::oracle::OracleError> {
            self.1.clone()
        }
    }

    #[test]
    fn oracles_that_answer_alike_agree() {
        let a = OxigraphOracle::new();
        let b = OxigraphOracle::new();
        assert_eq!(
            cross_check_oracles(&a, &b, O2_GRAPH, "SELECT ?s WHERE { ?s <http://ex/p> ?o }"),
            OracleCross::Agree
        );
        assert_eq!(
            cross_check_oracles(&a, &b, O2_GRAPH, "ASK { <http://ex/a> <http://ex/p> 1 }"),
            OracleCross::Agree
        );
        assert_eq!(
            cross_check_oracles(
                &a,
                &b,
                O2_GRAPH,
                "CONSTRUCT { ?s <http://ex/r> ?o } WHERE { ?s <http://ex/p> ?o }"
            ),
            OracleCross::Agree
        );
    }

    /// The load-bearing case: a second oracle returning a DIFFERENT answer must be flagged.
    /// Oxigraph answers this query with two rows; the stub answers with one.
    #[test]
    fn a_second_oracle_with_a_different_answer_is_flagged() {
        let oxi = OxigraphOracle::new();
        let one_row = {
            let mut row = std::collections::BTreeMap::new();
            row.insert(
                "s".to_string(),
                sparq_difftest::Term::Iri("http://ex/a".into()),
            );
            StubOracle("stub", Ok(OracleResult::Solutions(vec![row])))
        };
        match cross_check_oracles(
            &oxi,
            &one_row,
            O2_GRAPH,
            "SELECT ?s WHERE { ?s <http://ex/p> ?o }",
        ) {
            OracleCross::Disagree(d) => {
                assert!(d.contains("oxigraph has 2 rows"), "detail was {d:?}");
                assert!(d.contains("stub has 1 rows"), "detail was {d:?}");
            }
            other => panic!("a differing second oracle must DISAGREE, got {other:?}"),
        }

        // A differing ASK is likewise a divergence, not a skip.
        let says_false = StubOracle("stub", Ok(OracleResult::Boolean(false)));
        assert!(matches!(
            cross_check_oracles(
                &oxi,
                &says_false,
                O2_GRAPH,
                "ASK { <http://ex/a> <http://ex/p> 1 }"
            ),
            OracleCross::Disagree(_)
        ));

        // A differing CONSTRUCT graph is compared up to bnode isomorphism, and still differs.
        let empty_graph = StubOracle("stub", Ok(OracleResult::Graph(vec![])));
        assert!(matches!(
            cross_check_oracles(
                &oxi,
                &empty_graph,
                O2_GRAPH,
                "CONSTRUCT { ?s <http://ex/r> ?o } WHERE { ?s <http://ex/p> ?o }"
            ),
            OracleCross::Disagree(_)
        ));
    }

    /// Every NON-comparison route lands in the counted `Skipped` bucket rather than being
    /// mistaken for agreement — an oracle pair that compared nothing must not read as green.
    #[test]
    fn non_comparable_cases_skip_rather_than_agree() {
        let oxi = OxigraphOracle::new();

        // (1) Arbitrary row choice under LIMIT — checked BEFORE either oracle is consulted,
        // so a stub that would otherwise disagree still skips.
        let disagrees = StubOracle("stub", Ok(OracleResult::Solutions(vec![])));
        assert_eq!(
            cross_check_oracles(
                &oxi,
                &disagrees,
                O2_GRAPH,
                "SELECT ?s WHERE { ?s <http://ex/p> ?o } LIMIT 1"
            ),
            OracleCross::Skipped
        );

        // (2) The second oracle declined the query.
        let declines = StubOracle(
            "stub",
            Err(crate::oracle::OracleError::Unsupported("nope".into())),
        );
        assert_eq!(
            cross_check_oracles(
                &oxi,
                &declines,
                O2_GRAPH,
                "SELECT ?s WHERE { ?s <http://ex/p> ?o }"
            ),
            OracleCross::Skipped
        );

        // (3) The two oracles returned different result FORMS.
        let wrong_form = StubOracle("stub", Ok(OracleResult::Boolean(true)));
        assert_eq!(
            cross_check_oracles(
                &oxi,
                &wrong_form,
                O2_GRAPH,
                "SELECT ?s WHERE { ?s <http://ex/p> ?o }"
            ),
            OracleCross::Skipped
        );

        // (4) A blank node in an answer — labels are engine-local, so a labelled comparison
        // across engines is meaningless until RDFC-1.0 canonical labelling is wired in.
        let bnode = {
            let mut row = std::collections::BTreeMap::new();
            row.insert("s".to_string(), sparq_difftest::Term::Blank("b0".into()));
            StubOracle("stub", Ok(OracleResult::Solutions(vec![row])))
        };
        assert_eq!(
            cross_check_oracles(
                &oxi,
                &bnode,
                O2_GRAPH,
                "SELECT ?s WHERE { ?s <http://ex/p> ?o }"
            ),
            OracleCross::Skipped
        );
    }

    /// The load-bearing separation: an oracle that BROKE (spawn failure, timeout, crash, garbage
    /// stdout) or rejected the DATA must never land in the skip bucket — that is precisely how a
    /// dead second oracle would compare zero cases while the run still reported success.
    #[test]
    fn a_broken_oracle_is_never_a_skip() {
        let oxi = OxigraphOracle::new();
        let query = "SELECT ?s WHERE { ?s <http://ex/p> ?o }";

        for err in [
            crate::oracle::OracleError::Backend("spawning java: No such file".into()),
            crate::oracle::OracleError::Data("turtle parse error at line 2".into()),
        ] {
            let broken = StubOracle("stub", Err(err.clone()));
            match cross_check_oracles(&oxi, &broken, O2_GRAPH, query) {
                OracleCross::Broken(detail) => {
                    assert!(detail.contains("stub"), "detail was {detail:?}");
                    assert!(
                        detail.contains(&err.to_string()),
                        "detail {detail:?} must carry the underlying fault {err}"
                    );
                }
                other => panic!("a broken oracle must not be {other:?} (err was {err})"),
            }
        }

        // The FIRST oracle breaking is the same fault, and is not masked by the second answering.
        let broken_first = StubOracle("stub", Err(crate::oracle::OracleError::Backend("x".into())));
        assert!(matches!(
            cross_check_oracles(&broken_first, &oxi, O2_GRAPH, query),
            OracleCross::Broken(_)
        ));

        // ...but a DECLINE still is a skip: only `Unsupported` may reach that bucket.
        let declines = StubOracle(
            "stub",
            Err(crate::oracle::OracleError::Unsupported("nope".into())),
        );
        assert_eq!(
            cross_check_oracles(&oxi, &declines, O2_GRAPH, query),
            OracleCross::Skipped
        );
    }
}
