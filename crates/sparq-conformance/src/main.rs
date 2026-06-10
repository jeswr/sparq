//! sparq-conformance: runs the official W3C SPARQL test suites (w3c/rdf-tests,
//! fetched by `scripts/fetch-conformance.sh`) against `sparq_engine` and emits
//! a per-suite pass/fail/skip scoreboard (stdout + conformance-report.md),
//! grouped SPARQL 1.0 / 1.1 / 1.2 / syntax. Evaluation tests exercise the
//! engine; syntax tests exercise the spargebra parser dependency (and thereby
//! what sparq accepts).
//!
//! Informational by default (always exits 0); `--strict` exits 1 on any FAIL
//! so the pass rate can be ratcheted in CI later.

mod compare;
mod manifest;
mod rdf;
mod results;
mod run;

use manifest::{EntryKind, TestEntry};
use run::Status;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// What a section runs from its manifests. The same manifest tree may appear
/// twice with different scopes (e.g. `sparql12/manifest.ttl` mixes evaluation
/// and syntax entries; the 1.1 query/update manifests include their syntax
/// sub-manifests) — scope filtering guarantees each test runs exactly once.
#[derive(Clone, Copy, PartialEq)]
enum Scope {
    /// `mf:QueryEvaluationTest` / `mf:UpdateEvaluationTest` entries.
    Eval,
    /// `mf:{Positive,Negative}[Update]SyntaxTest*` entries.
    Syntax,
}

/// Report section: (label, manifests relative to `<rdf-tests>/sparql/`, scope).
type SectionSpec = (&'static str, &'static [&'static str], Scope);

/// The full target scope, grouped 1.0 / 1.1 / 1.2 / syntax. Protocol, SERVICE
/// evaluation and entailment suites are intentionally absent (see the report
/// footer).
const GROUPS: &[(&str, &[SectionSpec])] = &[
    (
        "SPARQL 1.0",
        &[(
            "query evaluation (sparql10)",
            &["sparql10/manifest-evaluation.ttl"],
            Scope::Eval,
        )],
    ),
    (
        "SPARQL 1.1",
        &[
            (
                "query evaluation (sparql11)",
                &["sparql11/manifest-sparql11-query.ttl"],
                Scope::Eval,
            ),
            (
                "update evaluation (sparql11)",
                &["sparql11/manifest-sparql11-update.ttl"],
                Scope::Eval,
            ),
            (
                "result formats — csv-tsv-res / json-res (sparql11)",
                &["sparql11/manifest-sparql11-results.ttl"],
                Scope::Eval,
            ),
        ],
    ),
    (
        "SPARQL 1.2",
        &[(
            "query + update evaluation (sparql12)",
            &["sparql12/manifest.ttl"],
            Scope::Eval,
        )],
    ),
    (
        "Syntax (spargebra parser: positive = must parse, negative = must be rejected)",
        &[
            (
                "SPARQL 1.0 syntax",
                &["sparql10/manifest-syntax.ttl"],
                Scope::Syntax,
            ),
            (
                "SPARQL 1.1 syntax — query",
                &["sparql11/syntax-query/manifest.ttl"],
                Scope::Syntax,
            ),
            (
                "SPARQL 1.1 syntax — update",
                &[
                    "sparql11/syntax-update-1/manifest.ttl",
                    "sparql11/syntax-update-2/manifest.ttl",
                ],
                Scope::Syntax,
            ),
            (
                "SPARQL 1.1 syntax — federation",
                &["sparql11/syntax-fed/manifest.ttl"],
                Scope::Syntax,
            ),
            (
                "SPARQL 1.2 syntax",
                &["sparql12/manifest.ttl"],
                Scope::Syntax,
            ),
        ],
    ),
];

#[derive(Default)]
struct SuiteStats {
    pass: usize,
    fail: usize,
    skip: usize,
    failures: Vec<(String, String)>, // (test name, reason)
    skips: Vec<(String, String)>,
}

fn main() {
    // Quiet expected engine panics (they are reported as FAILs by the watchdog).
    std::panic::set_hook(Box::new(|_| {}));

    let mut root = PathBuf::from("tests/w3c/rdf-tests");
    let mut report_path = PathBuf::from("conformance-report.md");
    let mut strict = false;
    let mut filter: Option<String> = None;
    let mut verbose = false;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--root" => root = PathBuf::from(args.next().expect("--root needs a path")),
            "--report" => report_path = PathBuf::from(args.next().expect("--report needs a path")),
            "--strict" => strict = true,
            "--filter" => filter = Some(args.next().expect("--filter needs a substring")),
            "--verbose" | "-v" => verbose = true,
            other => {
                eprintln!("unknown argument: {other}");
                eprintln!("usage: sparq-conformance [--root DIR] [--report FILE] [--filter SUBSTR] [--strict] [--verbose]");
                std::process::exit(2);
            }
        }
    }

    if !root.join("sparql").is_dir() {
        eprintln!(
            "test data not found at {} — run scripts/fetch-conformance.sh first",
            root.display()
        );
        std::process::exit(2);
    }
    let suites_root = root
        .join("sparql")
        .canonicalize()
        .expect("canonicalize suites root");

    // Group -> section -> suite -> stats, preserving GROUPS order.
    let mut groups: Vec<(String, Vec<(String, BTreeMap<String, SuiteStats>)>)> = Vec::new();
    let mut other_entries = 0usize;
    let mut total_run = 0usize;

    for (group_label, section_specs) in GROUPS {
        let mut sections: Vec<(String, BTreeMap<String, SuiteStats>)> = Vec::new();
        for (label, tops, scope) in *section_specs {
            let mut entries: Vec<TestEntry> = Vec::new();
            for top in *tops {
                let manifest_path = suites_root.join(top);
                if let Err(e) = manifest::collect(&manifest_path, &suites_root, &mut entries) {
                    eprintln!("failed to load {}: {e}", manifest_path.display());
                }
            }
            let mut suites: BTreeMap<String, SuiteStats> = BTreeMap::new();
            for entry in &entries {
                if let Some(f) = &filter {
                    if !entry.id.contains(f.as_str())
                        && !entry.name.contains(f.as_str())
                        && !entry.suite.contains(f.as_str())
                    {
                        continue;
                    }
                }
                let status = match (&entry.kind, scope) {
                    // Protocol/CSV-format/… tests are out of scope; count them
                    // once (every Other-typed entry occurs in exactly one
                    // Eval-scoped section).
                    (EntryKind::Other(_), Scope::Eval) => {
                        other_entries += 1;
                        continue;
                    }
                    (EntryKind::Other(_), Scope::Syntax) => continue,
                    // Out-of-scope for this section: run by its sibling section
                    // walking the same manifests with the other scope.
                    (EntryKind::QueryEval | EntryKind::UpdateEval, Scope::Syntax) => continue,
                    (
                        EntryKind::PositiveSyntax { .. } | EntryKind::NegativeSyntax { .. },
                        Scope::Eval,
                    ) => continue,
                    _ if entry.withdrawn => Status::Skip("withdrawn test".into()),
                    (EntryKind::QueryEval, Scope::Eval) => run::run_query_test(entry),
                    (EntryKind::UpdateEval, Scope::Eval) => run::run_update_test(entry),
                    (EntryKind::PositiveSyntax { update }, Scope::Syntax) => {
                        run::run_syntax_test(entry, true, *update)
                    }
                    (EntryKind::NegativeSyntax { update }, Scope::Syntax) => {
                        run::run_syntax_test(entry, false, *update)
                    }
                };
                total_run += 1;
                let stats = suites.entry(entry.suite.clone()).or_default();
                match &status {
                    Status::Pass => stats.pass += 1,
                    Status::Fail(reason) => {
                        stats.fail += 1;
                        stats.failures.push((entry.name.clone(), reason.clone()));
                    }
                    Status::Skip(reason) => {
                        stats.skip += 1;
                        stats.skips.push((entry.name.clone(), reason.clone()));
                    }
                }
                if verbose {
                    let tag = match &status {
                        Status::Pass => "PASS".to_string(),
                        Status::Fail(r) => format!("FAIL ({r})"),
                        Status::Skip(r) => format!("SKIP ({r})"),
                    };
                    println!("[{}] {} — {}", entry.suite, entry.name, tag);
                }
            }
            sections.push((label.to_string(), suites));
        }
        groups.push((group_label.to_string(), sections));
    }

    let report = render_report(&groups, &root, other_entries, total_run);
    print!("{report}");
    if let Err(e) = std::fs::write(&report_path, &report) {
        eprintln!("could not write {}: {e}", report_path.display());
    } else {
        eprintln!("\nreport written to {}", report_path.display());
    }

    let total_fail: usize = groups
        .iter()
        .flat_map(|(_, sections)| sections.iter())
        .flat_map(|(_, s)| s.values())
        .map(|s| s.fail)
        .sum();
    if strict && total_fail > 0 {
        std::process::exit(1);
    }
}

fn pct(n: usize, d: usize) -> String {
    if d == 0 {
        "—".into()
    } else {
        format!("{:.1}%", 100.0 * n as f64 / d as f64)
    }
}

fn git_head(dir: &Path) -> String {
    std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".into())
}

fn render_report(
    groups: &[(String, Vec<(String, BTreeMap<String, SuiteStats>)>)],
    root: &Path,
    other_entries: usize,
    total_run: usize,
) -> String {
    use std::fmt::Write;
    let mut md = String::new();
    let _ = writeln!(md, "# sparq W3C SPARQL conformance report\n");
    let _ = writeln!(md, "- rdf-tests commit: `{}`", git_head(root));
    let _ = writeln!(md, "- sparq commit: `{}`", git_head(Path::new(".")));
    let _ = writeln!(
        md,
        "- scope: `mf:QueryEvaluationTest` + `mf:UpdateEvaluationTest` + the four \
         `mf:*SyntaxTest*` kinds, grouped 1.0 / 1.1 / 1.2 / syntax \
         ({total_run} tests run; {other_entries} protocol/format entries out of scope, \
         see the *Not yet run* footer)\n"
    );
    let _ = writeln!(
        md,
        "Pass rate is `pass / (pass + fail)` — skipped tests (unsupported features, \
         clearly reported below) are excluded from the rate but counted in coverage.\n"
    );

    let (mut gp, mut gf, mut gs) = (0usize, 0usize, 0usize);
    for (group_label, sections) in groups {
        let _ = writeln!(md, "## {group_label}\n");
        let (mut grp, mut grf, mut grs) = (0usize, 0usize, 0usize);
        for (label, suites) in sections {
            let _ = writeln!(md, "### {label}\n");
            let _ = writeln!(md, "| suite | pass | fail | skip | pass-rate (of run) |");
            let _ = writeln!(md, "|---|---:|---:|---:|---:|");
            let (mut sp, mut sf, mut ss) = (0usize, 0usize, 0usize);
            for (suite, st) in suites {
                let _ = writeln!(
                    md,
                    "| {suite} | {} | {} | {} | {} |",
                    st.pass,
                    st.fail,
                    st.skip,
                    pct(st.pass, st.pass + st.fail)
                );
                sp += st.pass;
                sf += st.fail;
                ss += st.skip;
            }
            let _ = writeln!(
                md,
                "| **total** | **{sp}** | **{sf}** | **{ss}** | **{}** |\n",
                pct(sp, sp + sf)
            );
            grp += sp;
            grf += sf;
            grs += ss;
        }
        if sections.len() > 1 {
            let _ = writeln!(
                md,
                "**{group_label} subtotal: {grp} pass / {grf} fail / {grs} skip — {}.**\n",
                pct(grp, grp + grf)
            );
        }
        gp += grp;
        gf += grf;
        gs += grs;
    }

    let _ = writeln!(md, "## Overall\n");
    let _ = writeln!(
        md,
        "**{gp} pass / {gf} fail / {gs} skip — pass-rate {} of run, {} of all in-scope tests.**\n",
        pct(gp, gp + gf),
        pct(gp, gp + gf + gs)
    );

    let all_sections = || groups.iter().flat_map(|(_, s)| s.iter());

    // Skip-reason histogram.
    let mut skip_hist: BTreeMap<String, usize> = BTreeMap::new();
    for (_, suites) in all_sections() {
        for st in suites.values() {
            for (_, reason) in &st.skips {
                *skip_hist.entry(reason.clone()).or_default() += 1;
            }
        }
    }
    if !skip_hist.is_empty() {
        let _ = writeln!(md, "## Skip reasons\n");
        let mut hist: Vec<_> = skip_hist.into_iter().collect();
        hist.sort_by(|a, b| b.1.cmp(&a.1));
        let _ = writeln!(md, "| reason | tests |");
        let _ = writeln!(md, "|---|---:|");
        for (reason, n) in hist {
            let _ = writeln!(md, "| {reason} | {n} |");
        }
        let _ = writeln!(md);
    }

    // Failure-reason histogram (coarse buckets) + full list.
    let mut fail_hist: BTreeMap<String, usize> = BTreeMap::new();
    for (_, suites) in all_sections() {
        for st in suites.values() {
            for (_, reason) in &st.failures {
                let bucket = reason
                    .split(':')
                    .next()
                    .unwrap_or(reason)
                    .trim()
                    .to_string();
                *fail_hist.entry(bucket).or_default() += 1;
            }
        }
    }
    if !fail_hist.is_empty() {
        let _ = writeln!(md, "## Failure categories\n");
        let mut hist: Vec<_> = fail_hist.into_iter().collect();
        hist.sort_by(|a, b| b.1.cmp(&a.1));
        let _ = writeln!(md, "| category | tests |");
        let _ = writeln!(md, "|---|---:|");
        for (reason, n) in hist {
            let _ = writeln!(md, "| {reason} | {n} |");
        }
        let _ = writeln!(md);
        let _ = writeln!(md, "<details><summary>All failures</summary>\n");
        for (_, suites) in all_sections() {
            for (suite, st) in suites {
                for (name, reason) in &st.failures {
                    let _ = writeln!(md, "- `{suite}` — **{name}**: {reason}");
                }
            }
        }
        let _ = writeln!(md, "\n</details>\n");
    }

    let _ = writeln!(md, "## Not yet run\n");
    let _ = writeln!(
        md,
        "Remaining W3C SPARQL scope this runner does not cover yet:\n\n\
         - **protocol**: `sparql11/protocol`, `sparql11/graph-store-protocol`, \
           `sparql11/http-rdf-update` (need an HTTP endpoint harness)\n\
         - **service** (federation evaluation): `sparql11/service` (needs live remote \
           endpoints; its syntax subset `sparql11/syntax-fed` *is* run above)\n\
         - **entailment**: `sparql11/entailment` (entailment-regime evaluation; \
           regime-tagged tests elsewhere are skip-reported)\n\
         - `sparql11/service-description`, and the `mf:CSVResultFormatTest` entries of \
           `sparql11/csv-tsv-res` (lossy CSV serialization comparison — the suite's \
           `QueryEvaluationTest`s are run above)"
    );
    md
}
