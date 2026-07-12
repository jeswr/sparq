//! # `trust-graph-bench` — certification-edge closure overhead + strict-additivity envelope
//!
//! [FABLE-5] sq-on7r4. 🤖 SPARQ agent — measurement-only driver for the shipped
//! certification-edge trust-graph closure (`sparq_trust::graph::derive_effective_rules`,
//! the default-OFF `cert-graph` feature, `sq-pfae.15`). The 2026-07 trust-expression
//! record's cost/decidability concern (`sq-pfae.9`) previously had NO runnable harness;
//! this driver gives it one.
//!
//! Two kinds of lane, both FAIL-CLOSED (any assertion mismatch ⇒ non-zero exit):
//!
//! - **Strict-additivity envelope (the load-bearing lane).** The closure documents
//!   "zero (surviving) certifications ⇒ output is `direct_rules` byte-identical". This
//!   driver MEASURES that envelope rather than trusting the in-crate unit test: it
//!   renders input and output rule sets to canonical bytes and asserts byte-EQUALITY for
//!   (a) a zero-certifications input, (b) a `depth_bound = 0` input with edges present,
//!   and (c) an input whose every edge is rejected (forged signature — zero SURVIVORS).
//!   Every timed lane additionally asserts the anchors-prefix is byte-equal (additivity
//!   holds even when edges ARE admitted) and that the admitted count matches the
//!   by-construction expectation, so no timing is ever reported over a wrong answer.
//!
//! - **Closure-overhead timing.** `derive_effective_rules` wall time over a grid of
//!   (anchor count × certification count × scope kind), min-of-K, with a per-edge
//!   amortised cost column. Scope kinds cover the gate's distinct cost paths:
//!   `any` (AnyService — no shape work), `narrow` (a provable shape narrowing — the
//!   structural-containment matcher runs), `broaden` (an additive-target broadening —
//!   rejected at the attenuation gate), `forged` (rejected at the signature gate).
//!
//! ## Honesty
//!
//! Measurement-only, clear-path: fixtures are plaintext in-process structs; NO ZK proof
//! is produced or verified and NO privacy/soundness claim is made (the ZK estate is
//! externally unaudited, gate `sq-qhy4`). Wall-clock output is advisory + NON-CANONICAL
//! on a shared work box (bench/CATALOG.md QUIET-BOX convention); no number printed here
//! is ever committed to markdown. The driver never invokes engine code (`sparq-engine`
//! enters the link only transitively, via `sparq-shacl`'s SHACL-SPARQL path — the same
//! linkage every production `cert-graph` consumer carries); the closure under test is a
//! pure function over in-process fixtures, so measuring it cannot perturb the system
//! under test.

use std::time::Instant;

use oxrdf::{BlankNode, Literal, NamedNode, Term, Triple};
use sparq_trust::{
    certification_message, derive_effective_rules, CertScope, Certification, ShapeRef, TrustRule,
};
use sparq_zk::sig::{public_key_to_hex, SecretKey};

/// Fixed evaluation instant (Unix seconds). Hard-coded so runs are deterministic — the
/// fixtures' validity windows are built around it (never around the actual wall clock).
const NOW: i64 = 1_700_000_000;
/// Anchor freshness bound + certification window half-width: 30 days.
const THIRTY_DAYS: i64 = 30 * 86_400;

fn iri(s: &str) -> NamedNode {
    NamedNode::new(s).expect("fixture IRIs are valid")
}

/// A predicate-scoped SHACL node-shape (the crate's `trust:forPredicate` desugaring):
/// `root sh:targetSubjectsOf p` + one `sh:property [sh:path p; sh:minCount "1"]`.
/// Blank-node labels are derived from `tag` so fixtures are fully deterministic.
fn predicate_shape(p: &str, tag: &str) -> ShapeRef {
    let root = BlankNode::new_unchecked(format!("root{tag}"));
    let prop = BlankNode::new_unchecked(format!("prop{tag}"));
    let sh_target_subjects_of = iri("http://www.w3.org/ns/shacl#targetSubjectsOf");
    let sh_property = iri("http://www.w3.org/ns/shacl#property");
    let sh_path = iri("http://www.w3.org/ns/shacl#path");
    let sh_mincount = iri("http://www.w3.org/ns/shacl#minCount");
    let pred = iri(p);
    ShapeRef {
        root: Term::BlankNode(root.clone()),
        triples: vec![
            Triple::new(root.clone(), sh_target_subjects_of, pred.clone()),
            Triple::new(root, sh_property, prop.clone()),
            Triple::new(prop.clone(), sh_path, pred),
            Triple::new(prop, sh_mincount, Literal::new_simple_literal("1")),
        ],
    }
}

/// A provable NARROWING of `shape`: same selection set, one extra conformance
/// constraint (`sh:datatype xsd:integer` on the property node) — admitted by the
/// attenuation gate via the structural-containment matcher (the expensive path).
fn narrowing_of(shape: &ShapeRef) -> ShapeRef {
    let mut s = shape.clone();
    let prop = s.triples[1].object.clone();
    if let Term::BlankNode(bn) = prop {
        s.triples.push(Triple::new(
            bn,
            iri("http://www.w3.org/ns/shacl#datatype"),
            iri("http://www.w3.org/2001/XMLSchema#integer"),
        ));
    }
    s
}

/// A BROADENING of `shape`: an extra root `sh:targetSubjectsOf` edge widens the
/// selection set — rejected fail-closed at the attenuation gate (`sq-pfae.15` fix).
fn broadening_of(shape: &ShapeRef, extra: &str) -> ShapeRef {
    let mut s = shape.clone();
    if let Term::BlankNode(root) = s.root.clone() {
        s.triples.push(Triple::new(
            root,
            iri("http://www.w3.org/ns/shacl#targetSubjectsOf"),
            iri(extra),
        ));
    }
    s
}

/// Build `count` deterministic anchor rules (distinct source / key / predicate shape /
/// resource scope each), returning the certifier secret keys alongside.
fn build_anchors(count: usize) -> (Vec<SecretKey>, Vec<TrustRule>) {
    let mut sks = Vec::with_capacity(count);
    let mut rules = Vec::with_capacity(count);
    for i in 0..count {
        let sk = SecretKey::from_seed(1 + i as u64);
        let pk = sk.public_key();
        rules.push(TrustRule {
            source: iri(&format!("https://gov.example/fw/{i}")),
            issuer_key: pk,
            shape: predicate_shape(&format!("https://schema.org/p{i}"), &format!("a{i}")),
            scope: iri(&format!("https://pod.example/res/{i}")),
            fresh_within_secs: THIRTY_DAYS,
        });
        sks.push(sk);
    }
    (sks, rules)
}

/// The scope-kind axis of the timing grid — each kind exercises a distinct cost path
/// through the closure's gates.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    /// `CertScope::AnyService` — admitted, no shape-containment work.
    Any,
    /// A provable shape narrowing — admitted through the structural matcher.
    Narrow,
    /// An additive-target broadening — REJECTED at the attenuation gate.
    Broaden,
    /// A malformed signature — REJECTED at the signature gate (zero survivors).
    Forged,
}

impl Kind {
    fn label(self) -> &'static str {
        match self {
            Kind::Any => "any",
            Kind::Narrow => "narrow",
            Kind::Broaden => "broaden",
            Kind::Forged => "forged",
        }
    }
    /// How many of `certs` edges the closure is EXPECTED to admit.
    fn expected_admitted(self, certs: usize) -> usize {
        match self {
            Kind::Any | Kind::Narrow => certs,
            Kind::Broaden | Kind::Forged => 0,
        }
    }
}

/// Build `count` certification edges of one `kind`, round-robined over the anchors.
/// Certified issuers/keys are distinct per edge; windows cover [`NOW`].
fn build_certs(
    kind: Kind,
    count: usize,
    anchors: &[TrustRule],
    sks: &[SecretKey],
) -> Vec<Certification> {
    let mut out = Vec::with_capacity(count);
    for j in 0..count {
        let i = j % anchors.len();
        let scope = match kind {
            Kind::Any | Kind::Forged => CertScope::AnyService,
            Kind::Narrow => CertScope::Shape(narrowing_of(&anchors[i].shape)),
            Kind::Broaden => {
                CertScope::Shape(broadening_of(&anchors[i].shape, "https://schema.org/email"))
            }
        };
        let mut cert = Certification {
            certifier: anchors[i].source.clone(),
            certifier_key: anchors[i].issuer_key,
            certified_issuer: iri(&format!("https://issuer.example/{j}")),
            certified_key: SecretKey::from_seed(10_000 + j as u64).public_key(),
            scope,
            valid_from_unix_secs: NOW - THIRTY_DAYS,
            valid_until_unix_secs: NOW + THIRTY_DAYS,
            signature_hex: String::new(),
        };
        cert.signature_hex = if kind == Kind::Forged {
            // Not hex-decodable ⇒ the signature gate rejects the edge fail-closed.
            "00".to_string()
        } else {
            sks[i].sign_commitment(&certification_message(&cert))
        };
        out.push(cert);
    }
    out
}

/// Canonical byte rendering of a rule set — EVERY `TrustRule` field, including the
/// shape's root term and each defining triple, with unambiguous field/record
/// separators. Two rule slices render byte-equal iff they carry the same rules in the
/// same order, so comparing renders IS the strict-additivity byte-equality check.
fn render_rules(rules: &[TrustRule]) -> Vec<u8> {
    let mut out = Vec::new();
    for r in rules {
        out.extend_from_slice(r.source.as_str().as_bytes());
        out.push(0x1f);
        out.extend_from_slice(public_key_to_hex(&r.issuer_key).as_bytes());
        out.push(0x1f);
        out.extend_from_slice(format!("{:?}", r.shape.root).as_bytes());
        out.push(0x1f);
        for t in &r.shape.triples {
            out.extend_from_slice(format!("{t:?}").as_bytes());
            out.push(0x1d);
        }
        out.push(0x1f);
        out.extend_from_slice(r.scope.as_str().as_bytes());
        out.push(0x1f);
        out.extend_from_slice(r.fresh_within_secs.to_string().as_bytes());
        out.push(0x1e);
    }
    out
}

/// One TSV result row. `min_us`/`per_edge_ns` are empty for the pure envelope lanes.
#[allow(clippy::too_many_arguments)]
fn row(
    lane: &str,
    kind: &str,
    anchors: usize,
    certs: usize,
    derived: usize,
    expected: usize,
    timing: Option<(f64, f64)>,
    ok: bool,
) {
    let (min_us, per_edge_ns) = match timing {
        Some((m, p)) => (format!("{m:.1}"), format!("{p:.0}")),
        None => (String::new(), String::new()),
    };
    println!(
        "{lane}\t{kind}\t{anchors}\t{certs}\t{derived}\t{expected}\t{min_us}\t{per_edge_ns}\t{}",
        if ok { "ok" } else { "FAIL" }
    );
}

fn main() {
    let mut smoke = false;
    let mut sf: usize = 1;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--smoke" => smoke = true,
            "--sf" => {
                sf = args
                    .next()
                    .and_then(|v| v.parse().ok())
                    .expect("--sf takes a positive integer");
            }
            other => {
                eprintln!("unknown argument: {other} (usage: trust-graph-bench [--smoke] [--sf N])");
                std::process::exit(2);
            }
        }
    }

    // Grid tiers: smoke is the per-commit fixed-size tier; --sf scales the biggest lane.
    let (anchor_grid, cert_grid, reps): (Vec<usize>, Vec<usize>, usize) = if smoke {
        (vec![1, 4], vec![4, 16], 3)
    } else {
        (vec![1, 8, 64], vec![16, 128, 512 * sf.max(1)], 5)
    };

    eprintln!(
        "[trust-graph] closure-overhead + strict-additivity envelope ({}; reps={reps}; \
         wall-clock advisory + NON-CANONICAL on a shared box)",
        if smoke { "--smoke".to_string() } else { format!("--sf {sf}") }
    );
    println!("# lane\tkind\tanchors\tcerts\tderived\texpected\tmin_us\tper_edge_ns\tstatus");

    let mut failures = 0usize;

    // ── Envelope lane 1 (LOAD-BEARING): zero certifications ⇒ byte-equal output ──────
    for &a in &anchor_grid {
        let (_sks, anchors) = build_anchors(a);
        let out = derive_effective_rules(&anchors, &[], NOW, 1);
        let ok = render_rules(&out) == render_rules(&anchors);
        failures += usize::from(!ok);
        row("additivity-zero-certs", "-", a, 0, out.len(), a, None, ok);
    }

    // ── Envelope lane 2: depth_bound = 0 with edges PRESENT ⇒ byte-equal output ──────
    {
        let a = *anchor_grid.last().expect("non-empty grid");
        let c = *cert_grid.first().expect("non-empty grid");
        let (sks, anchors) = build_anchors(a);
        let certs = build_certs(Kind::Any, c, &anchors, &sks);
        let out = derive_effective_rules(&anchors, &certs, NOW, 0);
        let ok = render_rules(&out) == render_rules(&anchors);
        failures += usize::from(!ok);
        row("additivity-depth0", "any", a, c, out.len(), a, None, ok);
    }

    // ── Envelope lane 3: edges present but ALL rejected (forged) ⇒ byte-equal ────────
    {
        let a = *anchor_grid.last().expect("non-empty grid");
        let c = *cert_grid.first().expect("non-empty grid");
        let (sks, anchors) = build_anchors(a);
        let certs = build_certs(Kind::Forged, c, &anchors, &sks);
        let out = derive_effective_rules(&anchors, &certs, NOW, 1);
        let ok = render_rules(&out) == render_rules(&anchors);
        failures += usize::from(!ok);
        row("additivity-all-rejected", "forged", a, c, out.len(), a, None, ok);
    }

    // ── Timing lanes: min-of-K closure wall time over the grid, correctness-gated ────
    for &a in &anchor_grid {
        let (sks, anchors) = build_anchors(a);
        let anchors_render = render_rules(&anchors);
        for &c in &cert_grid {
            for kind in [Kind::Any, Kind::Narrow, Kind::Broaden, Kind::Forged] {
                let certs = build_certs(kind, c, &anchors, &sks);
                let expected = kind.expected_admitted(c);

                // Correctness gate FIRST (outside timing): admitted count matches the
                // by-construction expectation AND the anchors-prefix is byte-equal
                // (strict additivity holds even when edges ARE admitted).
                let out = derive_effective_rules(&anchors, &certs, NOW, 1);
                let derived = out.len().saturating_sub(anchors.len());
                let prefix_ok = out.len() >= anchors.len()
                    && render_rules(&out[..anchors.len()]) == anchors_render;
                let ok = derived == expected && prefix_ok;
                failures += usize::from(!ok);

                let mut min_ns = u128::MAX;
                for _ in 0..reps {
                    let t0 = Instant::now();
                    let r = derive_effective_rules(&anchors, &certs, NOW, 1);
                    let dt = t0.elapsed().as_nanos();
                    // Consume the result so the call is never optimised away.
                    assert!(r.len() >= anchors.len());
                    min_ns = min_ns.min(dt);
                }
                let min_us = min_ns as f64 / 1_000.0;
                let per_edge_ns = min_ns as f64 / c.max(1) as f64;
                row(
                    "closure-overhead",
                    kind.label(),
                    a,
                    c,
                    derived,
                    expected,
                    Some((min_us, per_edge_ns)),
                    ok,
                );
            }
        }
    }

    if failures > 0 {
        eprintln!("[trust-graph] FAIL: {failures} lane(s) violated the fail-closed envelope");
        std::process::exit(1);
    }
    eprintln!(
        "[trust-graph] OK: strict-additivity envelope held on every lane (byte-equal \
         output = input for zero surviving certifications; anchors-prefix verbatim throughout)"
    );
}
