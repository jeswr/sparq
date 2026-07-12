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
//! category ∈ { all (default) | bgp | filter | optional | union | minus | limit |
//!              distinct | order } — lets a workflow shard the space across agents.

use oxigraph::store::Store;

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

/// A random query in the chosen category. Always valid SPARQL 1.1 (both engines
/// parse it); restricted to the surface sparq supports (no property paths).
fn gen_query(rng: &mut Rng, category: &str) -> String {
    let pfx = "PREFIX ex: <http://ex/>\nPREFIX xsd: <http://www.w3.org/2001/XMLSchema#>\n";
    // Pick an effective category (when "all", choose one at random).
    let cats = [
        "bgp", "filter", "equality", "optional", "union", "minus", "limit", "distinct", "order",
    ];
    let cat = if category == "all" {
        cats[rng.below(cats.len() as u64) as usize]
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

pub fn run(seed_start: u64, count: u64, category: &str) {
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
         adjudicated(cross-family-eq)={adjudicated_cross_family} adjudicated(bnode-iri)={adjudicated_bnode_iri} \
         full_mismatch={full_mismatch} count_mismatch={count_mismatch}",
        seed_start + count
    );
    if let Some(r) = first_repro {
        println!("\nFIRST FAILING CASE:\n{r}");
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
