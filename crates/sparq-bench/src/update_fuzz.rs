//! SPARQL **UPDATE** differential fuzzer vs Oxigraph (bead sq-3dyje.4). [FABLE-5]
//!
//! The query-side differential fuzzer (`fuzz.rs`) has zero UPDATE coverage, and
//! `sparq-engine/src/update.rs` is otherwise guarded by a single mechanism family
//! (unit tests + the fixed W3C conformance corpus). This module closes that gap with
//! a seeded generator of **ground-term update sequences** over the deterministic
//! UPDATE subset, applied step-by-step to THREE independent implementations:
//!
//!   1. sparq's rebuild path        (`sparq_engine::update` — decode / apply / rebuild),
//!   2. sparq's delta-overlay path  (`sparq_engine::update_in_place`),
//!   3. in-process Oxigraph 0.5     (`Store::update`).
//!
//! After EVERY step it asserts, per the AGENTS.md oracle-strength rule (term
//! structure and full answer sets, never row counts):
//!
//!   * **canonical dataset equality** — each store rendered to sorted N-Quads lines
//!     (all terms are ground, so sorted N-Quads IS the canonical form: no blank-node
//!     isomorphism to adjudicate) — rebuild == in-place == Oxigraph;
//!   * **probe SELECT equality** — the full sorted binding set of
//!     `SELECT ?s ?p ?o { ?s ?p ?o }` and `SELECT ?g ?s ?p ?o { GRAPH ?g { ?s ?p ?o } }`
//!     through each engine's own query path.
//!
//! A per-step compare means any divergence is localized to the exact offending
//! operation, and the final-store comparison is the last step's compare.
//!
//! GENERATED SUBSET (deterministic by construction — no `NOW()`/`RAND()`/`UUID()`/
//! `BNODE()`, and **no blank nodes in v1**, which avoids graph-isomorphism
//! adjudication): `INSERT DATA` / `DELETE DATA` (with `GRAPH` blocks),
//! `DELETE/INSERT … WHERE` (ground and variable templates, `WITH`, `USING`, variable
//! graph names, `DELETE WHERE`), `CLEAR` / `DROP` / `CREATE` / `COPY` / `MOVE` / `ADD`
//! (always `SILENT`, so absent-graph error behaviour — which the spec leaves
//! implementation-defined — cannot masquerade as a semantic divergence; the state
//! compare is the whole oracle), plus occasional two-operation compound requests
//! (`op ; op`) for the within-one-update sequencing path. `LOAD` is out of scope in
//! v1 (its source-fetch surface differs per engine). Graph EXISTENCE (an empty named
//! graph created by `CREATE`) is invisible to a quad-level compare and out of scope.
//!
//! KNOWN SHARED-ORACLE BLIND SPOT (honest boundary): both engines parse updates with
//! `spargebra`, so a parser-level desugaring bug (e.g. in COPY/MOVE/ADD expansion)
//! would affect both sides identically and cannot be caught here — only
//! evaluation-layer divergence is observable.
//!
//! The ADJUDICATED-divergence mechanism mirrors `fuzz.rs`: the comparator consults
//! `bench/differential-divergences.json` for classes whose id starts with `update-`.
//! There are none today, so the run is STRICT (every divergence fails); a future
//! adjudicated class additionally needs a detector added here before it can absorb
//! anything (a listed class without a detector stays strict, with a loud warning) —
//! never a silent skip.
//!
//! Usage: `sparq-bench update-fuzz --seed-start N --seed-count M`
//! Every case is reproducible from its seed; on failure the log carries
//! `MISMATCH seed=N` lines plus a `FIRST FAILING CASE:` block (the same
//! machine-parseable contract as `fuzz.rs`, consumed by
//! `scripts/ci-file-differential-failure.py`).

use oxigraph::store::Store;
use sparq_core::Graph;

// ── deterministic RNG ────────────────────────────────────────────────────────────

/// Deterministic SplitMix64 — no clock/entropy, so every case is reproducible from
/// its seed. (Same generator as `fuzz.rs`; duplicated because the bead keeps this
/// file's edits disjoint from the query fuzzer's.)
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

// ── generator ────────────────────────────────────────────────────────────────────
//
// Small term pools so random DELETE DATA / WHERE conditions collide with previously
// inserted data at high probability (a delete that never matches exercises nothing).

fn gen_subject(rng: &mut Rng) -> String {
    format!("<http://ex/s{}>", rng.below(6))
}

fn gen_predicate(rng: &mut Rng) -> String {
    format!("<http://ex/p{}>", rng.below(4))
}

/// Ground objects across the term kinds the dictionary treats differently: IRIs,
/// plain strings, canonical `xsd:integer` (the inline-id path), language-tagged
/// strings. Non-canonical numeric lexical forms (`"05"^^xsd:integer`) are a known
/// future extension, deliberately not generated in v1.
fn gen_object(rng: &mut Rng) -> String {
    match rng.below(6) {
        0 | 1 => format!("<http://ex/o{}>", rng.below(6)),
        2 | 3 => format!("\"lit{}\"", rng.below(5)),
        4 => format!("{}", rng.below(20)),
        _ => format!("\"tag{}\"@en", rng.below(3)),
    }
}

/// One of the 3 named graphs. A small pool keeps CLEAR/DROP/COPY/MOVE/ADD landing on
/// graphs that actually hold data.
fn gen_graph_iri(rng: &mut Rng) -> String {
    format!("<http://ex/g{}>", rng.below(3))
}

/// Generator state threaded through one sequence: the ground quads emitted by
/// earlier `INSERT DATA` operations, as `(graph IRI or None-for-default, "s p o .")`.
/// `DELETE DATA` (and the ground `WHERE` condition) sample mostly from this pool so
/// deletions/conditions actually HIT live data — a purely random quad almost never
/// matches the store, which was measured to let a neutered DELETE DATA path slip
/// through a 200-seed window undetected. Still a pure function of the seed.
#[derive(Default)]
struct GenState {
    inserted: Vec<(Option<String>, String)>,
}

/// A random fresh ground quad: `(graph slot, triple text)`.
fn gen_quad(rng: &mut Rng) -> (Option<String>, String) {
    let t = format!("{} {} {} .", gen_subject(rng), gen_predicate(rng), gen_object(rng));
    let slot = if rng.chance(2, 5) { Some(gen_graph_iri(rng)) } else { None };
    (slot, t)
}

/// Renders a quad into a data block: bare triple or `GRAPH`-wrapped.
fn push_quad(block: &mut String, slot: &Option<String>, t: &str) {
    match slot {
        Some(g) => block.push_str(&format!(" GRAPH {} {{ {} }}", g, t)),
        None => block.push_str(&format!(" {}", t)),
    }
}

/// The `INSERT DATA` block: 1..=5 fresh ground quads, recorded in the pool.
fn gen_insert_block(rng: &mut Rng, st: &mut GenState) -> String {
    let n = 1 + rng.below(5);
    let mut s = String::new();
    for _ in 0..n {
        let (slot, t) = gen_quad(rng);
        push_quad(&mut s, &slot, &t);
        st.inserted.push((slot, t));
    }
    s
}

/// The `DELETE DATA` block: 1..=5 quads, each drawn from the inserted pool with
/// probability 7/10 (a hit unless an intervening op already removed it) and fresh
/// random otherwise (the delete-of-absent-quad no-op path).
fn gen_delete_block(rng: &mut Rng, st: &GenState) -> String {
    let n = 1 + rng.below(5);
    let mut s = String::new();
    for _ in 0..n {
        let (slot, t) = if !st.inserted.is_empty() && rng.chance(7, 10) {
            st.inserted[rng.below(st.inserted.len() as u64) as usize].clone()
        } else {
            gen_quad(rng)
        };
        push_quad(&mut s, &slot, &t);
    }
    s
}

/// A `DELETE/INSERT … WHERE` operation. Templates are ground or variable; the WHERE
/// is a single BGP (optionally GRAPH-wrapped / re-scoped) — instantiation against
/// ground data yields only ground triples, so every shape is deterministic. The
/// query-side differential owns complex WHERE evaluation; the shapes here exist to
/// drive the update-side template instantiation, `WITH`/`USING` re-scoping, and
/// variable-graph-name paths.
fn gen_where_op(rng: &mut Rng, st: &GenState) -> String {
    match rng.below(9) {
        // Conditional ground insert: fires iff the (ground) condition triple exists.
        // The condition samples the inserted pool half the time so both the
        // condition-holds and condition-fails branches are exercised.
        0 => {
            let cond = if !st.inserted.is_empty() && rng.chance(1, 2) {
                let (slot, t) = &st.inserted[rng.below(st.inserted.len() as u64) as usize];
                match slot {
                    Some(g) => format!("GRAPH {} {{ {} }}", g, t),
                    None => t.clone(),
                }
            } else {
                format!("{} {} {} .", gen_subject(rng), gen_predicate(rng), gen_object(rng))
            };
            format!(
                "INSERT {{ {} {} {} }} WHERE {{ {} }}",
                gen_subject(rng),
                gen_predicate(rng),
                gen_object(rng),
                cond
            )
        }
        // Remove every `p` edge from the default graph.
        1 => {
            let p = gen_predicate(rng);
            format!("DELETE {{ ?s {} ?o }} WHERE {{ ?s {} ?o }}", p, p)
        }
        // Predicate rename in the default graph (delete + insert see the same pre-state).
        2 => {
            let p = gen_predicate(rng);
            let p2 = gen_predicate(rng);
            format!(
                "DELETE {{ ?s {} ?o }} INSERT {{ ?s {} ?o }} WHERE {{ ?s {} ?o }}",
                p, p2, p
            )
        }
        // Copy the default graph into a named graph (template GRAPH block).
        3 => format!(
            "INSERT {{ GRAPH {} {{ ?s ?p ?o }} }} WHERE {{ ?s ?p ?o }}",
            gen_graph_iri(rng)
        ),
        // Cross-graph delete with a VARIABLE graph name in the template.
        4 => {
            let p = gen_predicate(rng);
            format!(
                "DELETE {{ GRAPH ?g {{ ?s {} ?o }} }} WHERE {{ GRAPH ?g {{ ?s {} ?o }} }}",
                p, p
            )
        }
        // WITH-scoped predicate rename inside one named graph.
        5 => {
            let g = gen_graph_iri(rng);
            let p = gen_predicate(rng);
            let p2 = gen_predicate(rng);
            format!(
                "WITH {} DELETE {{ ?s {} ?o }} INSERT {{ ?s {} ?o }} WHERE {{ ?s {} ?o }}",
                g, p, p2, p
            )
        }
        // USING re-scoping: WHERE evaluates against the named graph, the deletion
        // template applies to the default graph.
        6 => format!(
            "DELETE {{ ?s ?p ?o }} USING {} WHERE {{ ?s ?p ?o }}",
            gen_graph_iri(rng)
        ),
        // Pull a named graph's triples into the default graph.
        7 => format!(
            "INSERT {{ ?s ?p ?o }} WHERE {{ GRAPH {} {{ ?s ?p ?o }} }}",
            gen_graph_iri(rng)
        ),
        // The DELETE WHERE shorthand (its own grammar production).
        _ => {
            let p = gen_predicate(rng);
            format!("DELETE WHERE {{ ?s {} ?o }}", p)
        }
    }
}

/// A graph-management operation. Always `SILENT`: the spec leaves the error-vs-no-op
/// choice for absent/existing targets implementation-defined (a store MAY succeed),
/// so a non-SILENT error-status difference between engines would not be a semantic
/// divergence — SILENT removes that ambiguity and keeps "every op must succeed on
/// every engine, and the states must match" as the whole oracle.
fn gen_structural(rng: &mut Rng) -> String {
    // DEFAULT or a named graph, for COPY/MOVE/ADD endpoints.
    fn graph_or_default(rng: &mut Rng) -> String {
        if rng.chance(1, 4) {
            "DEFAULT".to_string()
        } else {
            format!("GRAPH {}", gen_graph_iri(rng))
        }
    }
    match rng.below(8) {
        0 | 1 => {
            let target = match rng.below(5) {
                0 => "DEFAULT".to_string(),
                1 => "NAMED".to_string(),
                2 => "ALL".to_string(),
                _ => format!("GRAPH {}", gen_graph_iri(rng)),
            };
            format!("CLEAR SILENT {}", target)
        }
        2 | 3 => {
            let target = match rng.below(5) {
                0 => "DEFAULT".to_string(),
                1 => "NAMED".to_string(),
                2 => "ALL".to_string(),
                _ => format!("GRAPH {}", gen_graph_iri(rng)),
            };
            format!("DROP SILENT {}", target)
        }
        4 => format!("CREATE SILENT GRAPH {}", gen_graph_iri(rng)),
        // COPY/MOVE/ADD, including the same-source-and-destination corner (a spec
        // no-op) — both engines desugar via spargebra, so only evaluation-layer
        // handling can diverge here.
        _ => {
            let verb = ["COPY", "MOVE", "ADD"][rng.below(3) as usize];
            format!(
                "{} SILENT {} TO {}",
                verb,
                graph_or_default(rng),
                graph_or_default(rng)
            )
        }
    }
}

/// One update-request string. Mostly a single operation; 1-in-10 a two-operation
/// compound (`op ; op`) to exercise within-one-request sequencing.
fn gen_op(rng: &mut Rng, st: &mut GenState) -> String {
    fn single(rng: &mut Rng, st: &mut GenState) -> String {
        match rng.below(100) {
            0..=34 => format!("INSERT DATA {{{} }}", gen_insert_block(rng, st)),
            35..=49 => format!("DELETE DATA {{{} }}", gen_delete_block(rng, st)),
            50..=74 => gen_where_op(rng, st),
            _ => gen_structural(rng),
        }
    }
    let first = single(rng, st);
    if rng.chance(1, 10) {
        format!("{} ;\n{}", first, single(rng, st))
    } else {
        first
    }
}

/// The whole update sequence for one seed: 4..=10 update requests, applied (and
/// differentially checked) one at a time.
fn gen_sequence(rng: &mut Rng) -> Vec<String> {
    let n = 4 + rng.below(7) as usize;
    let mut st = GenState::default();
    (0..n).map(|_| gen_op(rng, &mut st)).collect()
}

// ── canonical dataset snapshots ──────────────────────────────────────────────────

/// A sparq `Graph` (default graph + named graphs) as SORTED N-Quads lines. All
/// generated terms are ground, so the sorted line set is a canonical form of the
/// dataset — byte-comparable across engines (both sides render `oxrdf` terms).
/// Duplicate lines are NOT collapsed: a store yielding the same quad twice would be
/// an invariant break worth failing on.
fn sparq_nquads(g: &Graph) -> Vec<String> {
    fn triples_of(g: &Graph, suffix: &str, out: &mut Vec<String>) {
        let scan = g.store.scan(&[None, None, None]);
        for r in scan.rows.iter() {
            let t = scan.to_spo(r);
            out.push(format!(
                "{} {} {}{} .",
                g.dict.term(t[0]),
                g.dict.term(t[1]),
                g.dict.term(t[2]),
                suffix
            ));
        }
    }
    let mut out = Vec::new();
    triples_of(g, "", &mut out);
    for (name, sub) in &g.named {
        triples_of(sub, &format!(" {}", name), &mut out);
    }
    out.sort();
    out
}

/// The Oxigraph store as SORTED N-Quads lines, rendered identically.
fn oxi_nquads(store: &Store) -> Result<Vec<String>, String> {
    use oxigraph::model::GraphName;
    let mut out = Vec::new();
    for q in store.iter() {
        let q = q.map_err(|e| format!("oxigraph iter error: {}", e))?;
        let line = match &q.graph_name {
            GraphName::DefaultGraph => {
                format!("{} {} {} .", q.subject, q.predicate, q.object)
            }
            g => format!("{} {} {} {} .", q.subject, q.predicate, q.object, g),
        };
        out.push(line);
    }
    out.sort();
    Ok(out)
}

/// The lines on exactly one side — the human-readable core of a divergence report.
fn one_sided(label_a: &str, a: &[String], label_b: &str, b: &[String]) -> String {
    let bset: std::collections::BTreeSet<&String> = b.iter().collect();
    let aset: std::collections::BTreeSet<&String> = a.iter().collect();
    let mut s = String::new();
    s.push_str(&format!("only in {}:\n", label_a));
    for l in a.iter().filter(|l| !bset.contains(l)) {
        s.push_str(&format!("  {}\n", l));
    }
    s.push_str(&format!("only in {}:\n", label_b));
    for l in b.iter().filter(|l| !aset.contains(l)) {
        s.push_str(&format!("  {}\n", l));
    }
    s
}

// ── probe SELECTs (full binding-set compare, per the oracle-strength rule) ────────

const PROBES: &[(&str, &[&str])] = &[
    ("SELECT ?s ?p ?o WHERE { ?s ?p ?o }", &["s", "p", "o"]),
    (
        "SELECT ?g ?s ?p ?o WHERE { GRAPH ?g { ?s ?p ?o } }",
        &["g", "s", "p", "o"],
    ),
];

/// The full sorted binding set of a probe through sparq's query path. Row cells are
/// rendered terms in projection order.
fn sparq_probe(g: &Graph, q: &str) -> Result<Vec<String>, String> {
    let r = sparq_engine::query(g, q).map_err(|e| format!("sparq probe error: {}", e))?;
    let mut rows: Vec<String> = r
        .rows
        .iter()
        .map(|row| {
            row.iter()
                .map(|t| {
                    t.as_ref()
                        .map(|t| t.to_string())
                        .unwrap_or_else(|| "UNDEF".to_string())
                })
                .collect::<Vec<_>>()
                .join(" | ")
        })
        .collect();
    rows.sort();
    Ok(rows)
}

/// The full sorted binding set of a probe through Oxigraph's query path.
// clippy: the differential oracle pins oxigraph's legacy Store::query semantics
#[allow(deprecated)]
fn oxi_probe(store: &Store, q: &str, vars: &[&str]) -> Result<Vec<String>, String> {
    match store.query(q).map_err(|e| e.to_string())? {
        oxigraph::sparql::QueryResults::Solutions(s) => {
            let mut rows = Vec::new();
            for sol in s {
                let sol = sol.map_err(|e| e.to_string())?;
                rows.push(
                    vars.iter()
                        .map(|v| {
                            sol.get(*v)
                                .map(|t| t.to_string())
                                .unwrap_or_else(|| "UNDEF".to_string())
                        })
                        .collect::<Vec<_>>()
                        .join(" | "),
                );
            }
            rows.sort();
            Ok(rows)
        }
        _ => Err("probe did not return solutions".to_string()),
    }
}

// ── the per-seed differential ────────────────────────────────────────────────────

/// Applies one seed's update sequence to all three implementations, checking the
/// canonical dataset + both probes after every step. Returns the number of update
/// requests applied, or the divergence detail (step-localized).
///
/// `inject_divergence_at`: test-only comparator non-vacuity knob — after applying
/// step `i` to all three engines, a marker quad is inserted into the OXIGRAPH store
/// only, so the comparator MUST report a divergence at that step (see
/// `tests::injected_divergence_is_caught`). `None` in production.
fn check_seed(seed: u64, inject_divergence_at: Option<usize>) -> Result<u64, String> {
    let mut rng = Rng::new(seed);
    let ops = gen_sequence(&mut rng);
    let mut g_rebuild = Graph::new();
    let mut g_inplace = Graph::new();
    let store = Store::new().map_err(|e| format!("oxigraph store init: {}", e))?;

    let fail = |step: usize, op: &str, detail: String| -> String {
        format!(
            "step={} of {}\nop: {}\n{}\nrepro: cargo run -p sparq-bench --release -- \
             update-fuzz --seed-start {} --seed-count 1\n--- full sequence ---\n{}",
            step,
            ops.len(),
            op,
            detail,
            seed,
            ops.iter()
                .enumerate()
                .map(|(i, o)| format!("[{}] {}", i, o))
                .collect::<Vec<_>>()
                .join("\n")
        )
    };

    for (i, op) in ops.iter().enumerate() {
        // Apply to all three implementations. Every generated op is inside the
        // supported deterministic subset, so an error from ANY engine is itself a
        // divergence (strict — there is no unsupported-skip in this harness).
        g_rebuild = sparq_engine::update(&g_rebuild, op)
            .map_err(|e| fail(i, op, format!("sparq update (rebuild path) error: {}", e)))?;
        sparq_engine::update_in_place(&mut g_inplace, op)
            .map_err(|e| fail(i, op, format!("sparq update_in_place error: {}", e)))?;
        store
            .update(op.as_str())
            .map_err(|e| fail(i, op, format!("oxigraph update error: {}", e)))?;

        if inject_divergence_at == Some(i) {
            use oxigraph::model::{GraphName, NamedNode, Quad};
            let n = |s: &str| NamedNode::new(s).expect("valid IRI");
            let marker = Quad::new(
                n("http://ex/injected"),
                n("http://ex/injected"),
                n("http://ex/injected"),
                GraphName::DefaultGraph,
            );
            store
                .insert(&marker)
                .map_err(|e| format!("marker insert failed: {}", e))?;
        }

        // (a) Canonical dataset equality — ground terms, so sorted N-Quads is
        // canonical. Localizes a divergence to this exact step.
        let nq_rebuild = sparq_nquads(&g_rebuild);
        let nq_inplace = sparq_nquads(&g_inplace);
        let nq_oxi = oxi_nquads(&store).map_err(|e| fail(i, op, e))?;
        if nq_rebuild != nq_oxi {
            return Err(fail(
                i,
                op,
                format!(
                    "canonical dataset differs (sparq rebuild vs oxigraph)\n{}",
                    one_sided("sparq(rebuild)", &nq_rebuild, "oxigraph", &nq_oxi)
                ),
            ));
        }
        if nq_inplace != nq_oxi {
            return Err(fail(
                i,
                op,
                format!(
                    "canonical dataset differs (sparq in-place vs oxigraph)\n{}",
                    one_sided("sparq(in-place)", &nq_inplace, "oxigraph", &nq_oxi)
                ),
            ));
        }

        // (b) Probe SELECTs — the query-path view of the updated store, full sorted
        // binding sets (never counts). Checked for BOTH sparq graphs: the in-place
        // one reads through the live delta overlay.
        for (probe, vars) in PROBES {
            let oxi = oxi_probe(&store, probe, vars)
                .map_err(|e| fail(i, op, format!("oxigraph probe error: {}", e)))?;
            for (label, g) in [("rebuild", &g_rebuild), ("in-place", &g_inplace)] {
                let sparq = sparq_probe(g, probe).map_err(|e| fail(i, op, e))?;
                if sparq != oxi {
                    return Err(fail(
                        i,
                        op,
                        format!(
                            "probe {:?} binding set differs (sparq {} vs oxigraph)\n{}",
                            probe,
                            label,
                            one_sided("sparq", &sparq, "oxigraph", &oxi)
                        ),
                    ));
                }
            }
        }
    }
    Ok(ops.len() as u64)
}

// ── adjudicated-divergence allowlist (update classes) ─────────────────────────────

/// The state line for the adjudicated-divergence mechanism. The registry is the same
/// `bench/differential-divergences.json` the query fuzzer consumes; UPDATE classes
/// use an `update-` id prefix. None exist today, so v1 runs STRICT — and a listed
/// `update-*` class without a detector in this file ALSO stays strict (loud warning),
/// mirroring the fail-toward-flagging posture of `fuzz.rs`.
fn allowlist_state() -> String {
    let path = std::env::var("SPARQ_FUZZ_DIVERGENCES").unwrap_or_else(|_| {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../bench/differential-divergences.json"
        )
        .to_string()
    });
    let strict = |why: &str| format!("({}): {} — STRICT (every divergence fails)", path, why);
    let s = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => return strict(&format!("unreadable ({})", e)),
    };
    let v: serde_json::Value = match serde_json::from_str(&s) {
        Ok(v) => v,
        Err(e) => return strict(&format!("invalid JSON ({})", e)),
    };
    let update_ids: Vec<String> = v["classes"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[])
        .iter()
        .filter_map(|c| c["id"].as_str())
        .filter(|id| id.starts_with("update-"))
        .map(str::to_string)
        .collect();
    if update_ids.is_empty() {
        return strict("no adjudicated `update-*` classes");
    }
    for id in &update_ids {
        eprintln!(
            "warning: divergence allowlist lists update class {:?} but update-fuzz has \
             no detector for it — that class stays STRICT",
            id
        );
    }
    strict(&format!(
        "listed update classes {:?} have NO detectors here",
        update_ids
    ))
}

// ── entry point ──────────────────────────────────────────────────────────────────

pub fn run(seed_start: u64, count: u64) {
    println!("update-fuzz divergence allowlist {}", allowlist_state());
    let mut checked = 0u64;
    let mut ops_applied = 0u64;
    let mut mismatch = 0u64;
    let mut first_repro: Option<String> = None;

    for seed in seed_start..seed_start + count {
        match check_seed(seed, None) {
            Ok(n) => {
                checked += 1;
                ops_applied += n;
            }
            Err(detail) => {
                mismatch += 1;
                // One machine-greppable line per failing seed (the contract
                // scripts/ci-file-differential-failure.py parses).
                eprintln!("MISMATCH seed={}", seed);
                if first_repro.is_none() {
                    first_repro = Some(format!("seed={}\n{}", seed, detail));
                }
            }
        }
    }

    println!(
        "update-fuzz seeds {}..{} : checked={} update_requests={} mismatch={}",
        seed_start,
        seed_start + count,
        checked,
        ops_applied,
        mismatch
    );
    if let Some(r) = first_repro {
        println!("\nFIRST FAILING CASE:\n{}", r);
        std::process::exit(1);
    }
}

// ── tests (the per-PR fixed-window smoke + comparator non-vacuity) ────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// The per-PR BLOCKING smoke: a fixed seed window through the full three-way
    /// differential (both sparq update paths vs Oxigraph, canonical dataset + probe
    /// binding sets per step). The randomized soak (advancing window) is the nightly
    /// `differential-update.yml` lane, not this test — keep this window small enough
    /// to stay fast in debug builds.
    #[test]
    fn fixed_window_smoke() {
        for seed in 0..40 {
            if let Err(detail) = check_seed(seed, None) {
                panic!("seed={} diverged:\n{}", seed, detail);
            }
        }
    }

    /// Comparator NON-VACUITY: a quad present in only one store MUST be reported as
    /// a divergence at exactly the injected step. If this test ever passes with the
    /// comparison logic hollowed out, the whole harness is asserting nothing.
    #[test]
    fn injected_divergence_is_caught() {
        let err = check_seed(0, Some(0)).expect_err("an injected extra quad must diverge");
        assert!(
            err.contains("canonical dataset differs"),
            "divergence must be reported by the canonical-dataset compare, got:\n{}",
            err
        );
        assert!(
            err.contains("step=0"),
            "divergence must be localized to the injected step, got:\n{}",
            err
        );
        assert!(
            err.contains("http://ex/injected"),
            "the report must show the offending quad, got:\n{}",
            err
        );
    }

    /// The generator is a pure function of the seed (the seed IS the corpus — the
    /// repro contract depends on this).
    #[test]
    fn generator_is_deterministic() {
        let a = gen_sequence(&mut Rng::new(12345));
        let b = gen_sequence(&mut Rng::new(12345));
        assert_eq!(a, b);
        assert!(a.len() >= 4 && a.len() <= 10, "sequence length in 4..=10");
    }

    /// The two snapshot renderers agree on a concrete mixed default/named-graph
    /// dataset built through BOTH engines' own update paths — the byte-level
    /// foundation the canonical compare rests on (IRIs, plain + typed + tagged
    /// literals, named-graph rendering).
    #[test]
    fn snapshot_renderers_agree_on_known_dataset() {
        let op = "INSERT DATA { <http://ex/s0> <http://ex/p0> \"lit0\" . \
                  <http://ex/s1> <http://ex/p1> 7 . \
                  GRAPH <http://ex/g0> { <http://ex/s2> <http://ex/p2> \"tag0\"@en . } }";
        let g = sparq_engine::update(&Graph::new(), op).expect("sparq update");
        let store = Store::new().expect("oxigraph store");
        store.update(op).expect("oxigraph update");
        let a = sparq_nquads(&g);
        let b = oxi_nquads(&store).expect("oxigraph iter");
        assert_eq!(a, b);
        assert_eq!(a.len(), 3);
        assert!(
            a.iter().any(|l| l.ends_with("<http://ex/g0> .")),
            "named-graph quad must carry its graph IRI: {:?}",
            a
        );
        assert!(
            a.iter()
                .any(|l| l.contains("\"7\"^^<http://www.w3.org/2001/XMLSchema#integer>")),
            "bare integer must render as a typed literal on both sides: {:?}",
            a
        );
    }
}
