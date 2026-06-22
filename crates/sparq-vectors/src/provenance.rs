//! Provenance **weight extractor** `w(t)` — read a per-triple **provenance-quality weight**
//! `w(t) ∈ (0, 1]` out of a graph's PROV-O / DQV annotations into the layout the structure-aware
//! KGE trainer consumes (provenance-driven GenAI-KB **Phase 4**, design
//! `research/provenance-driven-genai-kb.md` §USE-1 / §5 Phase 4, epic `sq-2489d`).
//!
//! [OPUS-4.8] sq-2489d.4. This **mirrors [`crate::shacl_priors`]** (the P2 SHACL/OWL prior
//! extractor): a small, **read-only** reader keyed by graph term ids that consumes only
//! `sparq-core`'s public id-scan API. It does **not** redefine the provenance model — it
//! consumes the same `pkg:confidence` / `pkg:assurance` / `prov:wasDerivedFrom` terms the
//! Phase-1 join (`sparq-nlq`'s `provenance` module) reads and the Phase-3 DQV terms `pkg.ttl`
//! declares. Crucially it does this **without pulling the engine** (unlike the SPARQL-backed
//! Phase-1 join): the `structure` feature stays lean — this reader is a direct id-level scan,
//! exactly like [`crate::shacl_priors`]'s `functional_properties` axiom reader.
//!
//! # What `w(t)` combines (design §USE-1)
//!
//! For an object-property triple `(h, r, t)` the weight is derived from the **head** entity `h`'s
//! provenance (the subject the fact is asserted about — `h`'s Finding/resource carries the
//! `prov:wasDerivedFrom` lineage and the `pkg:confidence` / `pkg:assurance` quality axes):
//!
//! | provenance fact | contribution |
//! |---|---|
//! | `pkg:confidence "c"` on `h` (epistemic weight, `0..1`) | the **confidence factor** `c` |
//! | `pkg:confidence "c"` on `h`'s `prov:wasDerivedFrom` source (source reliability) | folded in as the source-reliability factor (min over sources) |
//! | `pkg:assurance secx:Proven` | high **assurance multiplier** ([`WeightConfig::proven`]) |
//! | `pkg:assurance secx:Claimed` | mid multiplier ([`WeightConfig::claimed`]) |
//! | `pkg:assurance secx:Conjectured` | low multiplier ([`WeightConfig::conjectured`]) |
//! | *no provenance at all* | `w(t) = 1.0` (fail-open: a plain graph is **unchanged**) |
//!
//! `w(t) = clamp( assurance_mult · confidence_factor · source_reliability , floor , 1.0 )`, with a
//! configurable `floor > 0` so a weight is never zero (a zero would silently delete a positive from
//! the loss — the honest behaviour is "down-weight, never drop").
//!
//! # The on/off ablation (load-bearing — design §5 Phase 4)
//!
//! Provenance-weighting ships **behind an on/off ablation** and is adopted **only on measured
//! lift**. [`WeightMode`] is that switch: [`WeightMode::Uniform`] returns `1.0` for **every** triple
//! (the ablation **off** baseline — identical to today's unweighted trainer);
//! [`WeightMode::Provenance`] applies the `w(t)` above. A harness runs the *same* trainer in both
//! modes and compares filtered Hits@k / MRR (see `crate::eval::run_weight_ablation` under the `kge`
//! feature). **No accuracy claim is made here** — the literal/confidence-aware KGE line reports
//! *inconsistent* gains, so any lift is unproven and dataset-dependent; the bead's acceptance bar is
//! "adopt only if the measured lift clears a pre-registered bar, and ABANDON otherwise".
//!
//! # Honesty
//!
//! Every input is **read from the graph, never invented**. A head with no provenance simply has
//! `w(t) = 1.0` (fail-open). The weight only ever *scales a training-loss contribution*; it changes
//! no exact answer and is never surfaced as a confidence number to a user.

use rustc_hash::FxHashMap;
use sparq_core::dict::{Id, TermParts, NO_ID};
use sparq_core::Graph;

/// `pkg:confidence` — the numeric confidence (`0..1`) on a Finding (epistemic weight) or a Source
/// (reliability). Mirrors `sparq-nlq`'s `provenance::PKG_CONFIDENCE`.
pub const PKG_CONFIDENCE: &str = "https://sparq.dev/ns/pkg#confidence";
/// `pkg:assurance` — the epistemic-basis axis (`secx:Proven` / `Claimed` / `Conjectured`).
pub const PKG_ASSURANCE: &str = "https://sparq.dev/ns/pkg#assurance";
/// `prov:wasDerivedFrom` — the PROV-O lineage edge to a fact's source (its sub-property
/// `pkg:discoveredFrom` is read transparently because the closure materialises it as the
/// super-property, but the super-property IRI is what is scanned here).
pub const PROV_WAS_DERIVED_FROM: &str = "http://www.w3.org/ns/prov#wasDerivedFrom";
/// `pkg:discoveredFrom` — the PKG-native lineage sub-property of `prov:wasDerivedFrom`. Scanned in
/// addition to the super-property so the source-reliability factor resolves on an un-closed graph
/// (where the sub→super entailment has not been materialised).
pub const PKG_DISCOVERED_FROM: &str = "https://sparq.dev/ns/pkg#discoveredFrom";

/// The three `secx:` epistemic-basis IRIs (`pkg:assurance` objects). The `secx:` namespace is the
/// sec-prop extension the PKG reuses for the assurance axis; the design records both the
/// `sparq.dev/ns/secx#` form (used by the Phase-1 fixtures) and the canonical `w3id.org` form.
const SECX_PROVEN: [&str; 2] =
    ["https://sparq.dev/ns/secx#Proven", "https://w3id.org/zkp-sparql/sec-prop#Proven"];
const SECX_CLAIMED: [&str; 2] =
    ["https://sparq.dev/ns/secx#Claimed", "https://w3id.org/zkp-sparql/sec-prop#Claimed"];
const SECX_CONJECTURED: [&str; 2] =
    ["https://sparq.dev/ns/secx#Conjectured", "https://w3id.org/zkp-sparql/sec-prop#Conjectured"];

/// The **ablation switch** for provenance-weighting (design §5 Phase 4). The trainer runs
/// identically under both modes except for the per-positive weight, so a harness can measure
/// provenance-weighted vs uniform on the *same* graph, seed, and negatives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WeightMode {
    /// Every triple weighs `1.0` — the ablation **off** baseline, byte-identical to the unweighted
    /// trainer (no provenance is read at all).
    Uniform,
    /// Apply the provenance-quality weight `w(t)` of [`ProvenanceWeights`] — the ablation **on** arm.
    Provenance,
}

/// Tunable factors for the `w(t)` combination. The defaults follow design §USE-1
/// (`secx:Proven` → high, `Claimed` → mid, `Conjectured` → low) and are exposed so the ablation
/// harness can sweep them; they are **not** canonical and carry no accuracy claim.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WeightConfig {
    /// Assurance multiplier for `pkg:assurance secx:Proven`. Default `1.0`.
    pub proven: f32,
    /// Assurance multiplier for `pkg:assurance secx:Claimed`. Default `0.7`.
    pub claimed: f32,
    /// Assurance multiplier for `pkg:assurance secx:Conjectured`. Default `0.4`.
    pub conjectured: f32,
    /// Assurance multiplier when `pkg:assurance` is **absent** (unknown basis). Default `1.0`
    /// (fail-open — a missing axis never *penalises*; only an explicit lower-assurance axis does).
    pub default_assurance: f32,
    /// The lower clamp on `w(t)` (`> 0`), so a positive is never *dropped* from the loss (a zero
    /// weight would silently delete it). Default `0.05`.
    pub floor: f32,
}

impl Default for WeightConfig {
    fn default() -> WeightConfig {
        WeightConfig { proven: 1.0, claimed: 0.7, conjectured: 0.4, default_assurance: 1.0, floor: 0.05 }
    }
}

/// Per-entity provenance facts mined from a graph: the `pkg:confidence` of each subject, its
/// `pkg:assurance` basis, and the source ids it was `prov:wasDerivedFrom`. Keyed by graph term id.
/// This is the input to [`ProvenanceWeights::weight_of`].
///
/// Built by a single id-level scan of the graph (no engine, no SPARQL) — the same shape as
/// [`crate::shacl_priors`]'s `functional_properties` axiom reader.
#[derive(Clone, Debug, Default)]
pub struct ProvenanceWeights {
    /// `pkg:confidence` decimal of each subject id (`0..1`), when asserted and parseable.
    confidence: FxHashMap<Id, f32>,
    /// `pkg:assurance` basis of each subject id, when asserted.
    assurance: FxHashMap<Id, Assurance>,
    /// The source ids each subject id was derived from (`prov:wasDerivedFrom` / `pkg:discoveredFrom`).
    derived_from: FxHashMap<Id, Vec<Id>>,
    config: WeightConfig,
}

/// The epistemic basis read off a `pkg:assurance` edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Assurance {
    Proven,
    Claimed,
    Conjectured,
}

impl ProvenanceWeights {
    /// Mine provenance facts from `graph` with the default [`WeightConfig`]. The graph should
    /// already be **closed** (call [`materialise_closure`](crate::structure::materialise_closure)
    /// first) so an entailed `prov:wasDerivedFrom` (from a `pkg:discoveredFrom` sub-property) is a
    /// real triple — but the reader scans the sub-property directly too, so it also works on an
    /// un-closed graph.
    pub fn mine(graph: &Graph) -> ProvenanceWeights {
        Self::mine_with(graph, WeightConfig::default())
    }

    /// Mine provenance facts from `graph` with an explicit [`WeightConfig`].
    pub fn mine_with(graph: &Graph, config: WeightConfig) -> ProvenanceWeights {
        let conf_pred = graph.id_of(&iri(PKG_CONFIDENCE));
        let assurance_pred = graph.id_of(&iri(PKG_ASSURANCE));
        let derived_pred = graph.id_of(&iri(PROV_WAS_DERIVED_FROM));
        let discovered_pred = graph.id_of(&iri(PKG_DISCOVERED_FROM));

        // Resolve the assurance-object ids up front (None when the graph never mentions them).
        let proven = first_present(graph, &SECX_PROVEN);
        let claimed = first_present(graph, &SECX_CLAIMED);
        let conjectured = first_present(graph, &SECX_CONJECTURED);

        let mut confidence: FxHashMap<Id, f32> = FxHashMap::default();
        let mut assurance: FxHashMap<Id, Assurance> = FxHashMap::default();
        let mut derived_from: FxHashMap<Id, Vec<Id>> = FxHashMap::default();

        for [s, p, o] in graph.iter_ids() {
            if Some(p) == conf_pred {
                // A subject may (malformedly) carry two confidences; the first parseable wins and
                // later ones are ignored — deterministic, never a panic.
                if let std::collections::hash_map::Entry::Vacant(e) = confidence.entry(s) {
                    if let Some(c) = literal_f32(graph, o) {
                        e.insert(c.clamp(0.0, 1.0));
                    }
                }
            } else if Some(p) == assurance_pred {
                let basis = if Some(o) == proven {
                    Some(Assurance::Proven)
                } else if Some(o) == claimed {
                    Some(Assurance::Claimed)
                } else if Some(o) == conjectured {
                    Some(Assurance::Conjectured)
                } else {
                    None
                };
                if let Some(b) = basis {
                    assurance.entry(s).or_insert(b);
                }
            } else if Some(p) == derived_pred || Some(p) == discovered_pred {
                derived_from.entry(s).or_default().push(o);
            }
        }

        ProvenanceWeights { confidence, assurance, derived_from, config }
    }

    /// The configured factors (so a harness can read what it swept).
    pub fn config(&self) -> &WeightConfig {
        &self.config
    }

    /// The number of subjects with at least one mined provenance fact (confidence / assurance /
    /// lineage). `0` ⇒ the graph carries no provenance and every `w(t)` is `1.0`.
    pub fn annotated_subjects(&self) -> usize {
        let mut seen: rustc_hash::FxHashSet<Id> = rustc_hash::FxHashSet::default();
        seen.extend(self.confidence.keys().copied());
        seen.extend(self.assurance.keys().copied());
        seen.extend(self.derived_from.keys().copied());
        seen.len()
    }

    /// Is the graph entirely free of provenance (so weighting is a no-op `1.0` everywhere)?
    pub fn is_empty(&self) -> bool {
        self.confidence.is_empty() && self.assurance.is_empty() && self.derived_from.is_empty()
    }

    /// The provenance weight `w(t) ∈ [floor, 1.0]` for the object-property triple `(h, r, t)` under
    /// `mode`. Derived from the **head** `h`'s provenance (the subject the fact is about).
    ///
    /// Under [`WeightMode::Uniform`] this is **always** `1.0` (the ablation-off baseline — provenance
    /// is not even consulted). Under [`WeightMode::Provenance`] it is the design-§USE-1 combination;
    /// a head with no provenance still returns `1.0` (fail-open).
    pub fn weight_of(&self, triple: [Id; 3], mode: WeightMode) -> f32 {
        if mode == WeightMode::Uniform {
            return 1.0;
        }
        let [h, _r, _t] = triple;
        self.weight_for_subject(h)
    }

    /// The provenance weight contributed by a single subject `h` (the head of a positive). Public so
    /// a future confidence-biased walker / pooler can reuse the same scalar.
    pub fn weight_for_subject(&self, h: Id) -> f32 {
        let cfg = &self.config;

        // Assurance multiplier (default when the axis is absent — fail-open, never a penalty).
        let assurance_mult = match self.assurance.get(&h) {
            Some(Assurance::Proven) => cfg.proven,
            Some(Assurance::Claimed) => cfg.claimed,
            Some(Assurance::Conjectured) => cfg.conjectured,
            None => cfg.default_assurance,
        };

        // The head's own confidence (epistemic weight), default 1.0.
        let head_conf = self.confidence.get(&h).copied().unwrap_or(1.0);

        // Source reliability: the MIN confidence over the head's sources (a fact is only as reliable
        // as its least reliable source). A source with no asserted confidence contributes 1.0
        // (fail-open). No sources ⇒ 1.0.
        let source_reliability = match self.derived_from.get(&h) {
            None => 1.0,
            Some(sources) => sources
                .iter()
                .map(|s| self.confidence.get(s).copied().unwrap_or(1.0))
                .fold(1.0f32, f32::min),
        };

        let w = assurance_mult * head_conf * source_reliability;
        // Clamp to [floor, 1.0]: never zero (would drop the positive), never above 1.0.
        w.clamp(cfg.floor.max(f32::MIN_POSITIVE), 1.0)
    }
}

// ---- helpers ----------------------------------------------------------------------------------

/// A named-node term for an IRI string.
fn iri(s: &str) -> oxrdf::Term {
    oxrdf::Term::NamedNode(oxrdf::NamedNode::new_unchecked(s))
}

/// Resolve an IRI string to its interned id, or `None` if the graph never mentions it.
fn id_of_iri(graph: &Graph, s: &str) -> Option<Id> {
    graph.id_of(&iri(s)).filter(|&i| i != NO_ID)
}

/// The first of `candidates` that the graph interns (so a graph using either the `sparq.dev` or the
/// `w3id.org` secx form resolves). `None` when none is present.
fn first_present(graph: &Graph, candidates: &[&str]) -> Option<Id> {
    candidates.iter().find_map(|s| id_of_iri(graph, s))
}

/// Parse the lexical form of a literal object id as an `f32` (`0..1` confidence). `None` for a
/// non-literal object or an unparseable lexical form (fail-open — a malformed confidence is
/// treated as "no confidence asserted", never a panic).
fn literal_f32(graph: &Graph, o: Id) -> Option<f32> {
    match graph.dict.term_parts(o) {
        TermParts::Lit { value, .. } => value.trim().parse::<f32>().ok().filter(|c| c.is_finite()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structure::close_for_vectorise;
    use sparq_reason::Profile;

    // A PKG-shaped fixture mirroring the real pkg.ttl predicate set so the reader exercises the
    // REAL id-scan path, not a mock. Three facts: high-assurance Proven (alice), mid-assurance
    // Claimed with a so-so source (bob), and a bare fact with NO provenance (carol). [OPUS-4.8]
    const PKG: &str = r#"
@prefix pkg:  <https://sparq.dev/ns/pkg#> .
@prefix prov: <http://www.w3.org/ns/prov#> .
@prefix secx: <https://sparq.dev/ns/secx#> .
@prefix ex:   <http://ex/> .

ex:source-reliable  pkg:confidence "1.0" .
ex:source-soso      pkg:confidence "0.5" .

ex:alice
  pkg:confidence "0.9" ;
  pkg:assurance  secx:Proven ;
  prov:wasDerivedFrom ex:source-reliable ;
  ex:knows ex:bob .

ex:bob
  pkg:confidence "0.8" ;
  pkg:assurance  secx:Claimed ;
  prov:wasDerivedFrom ex:source-soso ;
  ex:knows ex:carol .

ex:carol ex:knows ex:alice .
"#;

    fn graph() -> Graph {
        // Close so any pkg:discoveredFrom → prov:wasDerivedFrom entailment is materialised; the
        // fixture uses the super-property directly, so the closure simply confirms the read path.
        close_for_vectorise(PKG, "turtle", Profile::Rdfs).unwrap().graph
    }

    fn id(g: &Graph, s: &str) -> Id {
        g.id_of(&iri(s)).unwrap()
    }

    #[test]
    fn uniform_mode_is_always_one() {
        let g = graph();
        let pw = ProvenanceWeights::mine(&g);
        let alice = id(&g, "http://ex/alice");
        let knows = id(&g, "http://ex/knows");
        let bob = id(&g, "http://ex/bob");
        // Even a fully-annotated head weighs exactly 1.0 under the ablation-off baseline.
        assert_eq!(pw.weight_of([alice, knows, bob], WeightMode::Uniform), 1.0);
    }

    #[test]
    fn provenance_mode_reads_confidence_and_assurance() {
        let g = graph();
        let pw = ProvenanceWeights::mine(&g);
        let cfg = WeightConfig::default();
        let alice = id(&g, "http://ex/alice");
        let bob = id(&g, "http://ex/bob");
        let knows = id(&g, "http://ex/knows");
        let carol = id(&g, "http://ex/carol");

        // alice: Proven(1.0) · conf(0.9) · source-reliable(1.0) = 0.9.
        let wa = pw.weight_of([alice, knows, bob], WeightMode::Provenance);
        assert!((wa - cfg.proven * 0.9 * 1.0).abs() < 1e-6, "alice w = {wa}");

        // bob: Claimed(0.7) · conf(0.8) · source-soso(0.5) = 0.7·0.8·0.5 = 0.28.
        let wb = pw.weight_of([bob, knows, carol], WeightMode::Provenance);
        assert!((wb - cfg.claimed * 0.8 * 0.5).abs() < 1e-6, "bob w = {wb}");

        // bob's lower-assurance + so-so source down-weights it below alice (the load-bearing
        // ordering: high-assurance facts rank higher).
        assert!(wb < wa, "lower-assurance/source fact must weigh less ({wb} < {wa})");
    }

    #[test]
    fn no_provenance_is_fail_open_one() {
        let g = graph();
        let pw = ProvenanceWeights::mine(&g);
        let knows = id(&g, "http://ex/knows");
        let carol = id(&g, "http://ex/carol");
        let alice = id(&g, "http://ex/alice");
        // carol has NO confidence/assurance/source → w = 1.0 (a plain fact is unchanged).
        assert_eq!(pw.weight_of([carol, knows, alice], WeightMode::Provenance), 1.0);
    }

    #[test]
    fn plain_graph_has_no_provenance() {
        // A graph with zero PROV-O/DQV annotations: the reader is empty and every weight is 1.0,
        // so the trainer is byte-identical to the unweighted path.
        let plain = "@prefix ex: <http://ex/> .\nex:a ex:knows ex:b .\nex:b ex:knows ex:a .";
        let g = Graph::load_str(plain, "turtle").unwrap();
        let pw = ProvenanceWeights::mine(&g);
        assert!(pw.is_empty(), "plain graph carries no provenance");
        assert_eq!(pw.annotated_subjects(), 0);
        let a = id(&g, "http://ex/a");
        let b = id(&g, "http://ex/b");
        let knows = id(&g, "http://ex/knows");
        assert_eq!(pw.weight_of([a, knows, b], WeightMode::Provenance), 1.0);
    }

    #[test]
    fn weight_never_zero_floor_clamped() {
        // A degenerate config (conjectured = 0) must still floor-clamp, never emit a 0 weight that
        // would silently delete the positive from the loss.
        let ttl = r#"
@prefix pkg:  <https://sparq.dev/ns/pkg#> .
@prefix secx: <https://sparq.dev/ns/secx#> .
@prefix ex:   <http://ex/> .
ex:x pkg:assurance secx:Conjectured ; pkg:confidence "0.0" ; ex:rel ex:y .
ex:y ex:rel ex:x .
"#;
        let g = Graph::load_str(ttl, "turtle").unwrap();
        let cfg = WeightConfig { conjectured: 0.0, ..WeightConfig::default() };
        let pw = ProvenanceWeights::mine_with(&g, cfg);
        let x = id(&g, "http://ex/x");
        let y = id(&g, "http://ex/y");
        let rel = id(&g, "http://ex/rel");
        let w = pw.weight_of([x, rel, y], WeightMode::Provenance);
        assert!(w >= cfg.floor.max(f32::MIN_POSITIVE), "w = {w} must be floor-clamped > 0");
        assert!(w > 0.0, "w must never be zero (would drop the positive)");
    }

    #[test]
    fn malformed_confidence_is_ignored_not_panic() {
        // A non-numeric confidence is treated as "no confidence" (fail-open), never a panic.
        let ttl = r#"
@prefix pkg: <https://sparq.dev/ns/pkg#> .
@prefix ex:  <http://ex/> .
ex:a pkg:confidence "not-a-number" ; ex:rel ex:b .
ex:b ex:rel ex:a .
"#;
        let g = Graph::load_str(ttl, "turtle").unwrap();
        let pw = ProvenanceWeights::mine(&g);
        let a = id(&g, "http://ex/a");
        let b = id(&g, "http://ex/b");
        let rel = id(&g, "http://ex/rel");
        // No parseable confidence + no assurance → w = 1.0.
        assert_eq!(pw.weight_of([a, rel, b], WeightMode::Provenance), 1.0);
    }
}
