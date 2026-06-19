// [OPUS-4.8] sq-jxl0: single-source the crate overview from README.md so crates.io
// (package.readme) and the docs.rs front page render identical content. The README's
// rust fences are API-map sketches marked `ignore` (they reference an external graph).
#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)] // [OPUS-4.8] sq-emay: crate has zero `unsafe`

use oxrdf::vocab::xsd;
use oxrdf::{Literal, NamedNode, Term};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use sparq_core::dict::{self, Id, TermParts, INLINE_BASE};
use sparq_core::Graph;

const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const RDFS_DOMAIN: &str = "http://www.w3.org/2000/01/rdf-schema#domain";
const RDFS_RANGE: &str = "http://www.w3.org/2000/01/rdf-schema#range";
const XSD_INTEGER: &str = "http://www.w3.org/2001/XMLSchema#integer";
const VOID_NS: &str = "http://rdfs.org/ns/void#";
/// [OPUS-4.8] sq-mr32 (federation A3/Z2): the **sparq characteristic-set** extension
/// vocabulary. VoID has no native term for per-entity-type predicate co-occurrence
/// statistics (the Neumann & Moerkotte characteristic sets sparq mines), so the served
/// descriptor expresses them under this documented sparq namespace, alongside (not
/// replacing) the standard VoID terms. A remote federation source-selector that does
/// not understand it simply ignores these triples; one that does gets star/multi-join
/// cardinality estimates far sharper than bare `void:propertyPartition` counts.
///
/// Terms (all under `<http://sparq.dev/ns/cs#>`):
/// - `scs:CharacteristicSet` — the rdf:type of each characteristic-set node.
/// - `scs:characteristicSet` — links the `void:Dataset` to one characteristic-set node.
/// - `scs:distinctCharacteristicSets` — total distinct sets in the dataset (on the dataset).
/// - `scs:subjects` — `count(C)`: subjects whose *exact* predicate set this is.
/// - `scs:predicateStat` — links a set to one per-predicate statistic node.
/// - `scs:avgMultiplicity` — `predicate_triples / subjects` for that predicate in the set.
///
/// Each per-predicate statistic node reuses `void:property` (the predicate IRI) and
/// `void:triples` (Σ triples that predicate emits across the set's subjects), so a
/// VoID-aware-but-cs-unaware client still reads a meaningful property partition.
const CS_NS: &str = "http://sparq.dev/ns/cs#";

/// [OPUS-4.8] sq-3n4: the conventional file extension for a persisted introspection
/// sidecar — the mined effective schema as JSON, written next to the source dataset so
/// later processes can produce summaries / VoID without re-mining the graph. See
/// [`Introspection::save`]/[`Introspection::load`] and [`sidecar_path_for`].
pub const SIDECAR_EXTENSION: &str = "introspect";

/// [OPUS-4.8] sq-3n4: the conventional sidecar path for a dataset — the dataset path
/// with [`SIDECAR_EXTENSION`] **appended** (not replacing the dataset's own extension),
/// so `data/olympics.nt` ⇒ `data/olympics.nt.introspect`. Appending (rather than
/// swapping the extension) keeps the sidecar unambiguous when two datasets differ only
/// by extension (`g.nt` vs `g.ttl`) and mirrors how companion files like `*.nt.gz` read.
pub fn sidecar_path_for(dataset: impl AsRef<std::path::Path>) -> std::path::PathBuf {
    let p = dataset.as_ref();
    let mut name = p.file_name().unwrap_or(p.as_os_str()).to_os_string();
    name.push(".");
    name.push(SIDECAR_EXTENSION);
    p.with_file_name(name)
}

/// Well-known vocabularies, recognised by namespace: `(prefix, namespace, title)`.
/// Bundled (offline, WASM-safe) — no network lookup.
const WELL_KNOWN: &[(&str, &str, &str)] = &[
    ("rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#", "RDF"),
    (
        "rdfs",
        "http://www.w3.org/2000/01/rdf-schema#",
        "RDF Schema",
    ),
    ("owl", "http://www.w3.org/2002/07/owl#", "OWL"),
    (
        "xsd",
        "http://www.w3.org/2001/XMLSchema#",
        "XML Schema datatypes",
    ),
    (
        "foaf",
        "http://xmlns.com/foaf/0.1/",
        "FOAF (people & agents)",
    ),
    ("skos", "http://www.w3.org/2004/02/skos/core#", "SKOS"),
    ("schema", "http://schema.org/", "Schema.org"),
    ("schema", "https://schema.org/", "Schema.org"),
    ("dcterms", "http://purl.org/dc/terms/", "Dublin Core terms"),
    (
        "dc",
        "http://purl.org/dc/elements/1.1/",
        "Dublin Core elements",
    ),
    ("void", "http://rdfs.org/ns/void#", "VoID dataset metadata"),
    (
        "geo",
        "http://www.w3.org/2003/01/geo/wgs84_pos#",
        "WGS84 geo positioning",
    ),
    (
        "geosparql",
        "http://www.opengis.net/ont/geosparql#",
        "OGC GeoSPARQL",
    ),
    ("prov", "http://www.w3.org/ns/prov#", "PROV-O provenance"),
    ("sh", "http://www.w3.org/ns/shacl#", "SHACL"),
    ("dbo", "http://dbpedia.org/ontology/", "DBpedia ontology"),
    ("dbr", "http://dbpedia.org/resource/", "DBpedia resources"),
    ("dbp", "http://dbpedia.org/property/", "DBpedia properties"),
    ("wd", "http://www.wikidata.org/entity/", "Wikidata entities"),
    (
        "wdt",
        "http://www.wikidata.org/prop/direct/",
        "Wikidata direct properties",
    ),
    ("p", "http://www.wikidata.org/prop/", "Wikidata properties"),
    ("gn", "http://www.geonames.org/ontology#", "GeoNames"),
    ("vcard", "http://www.w3.org/2006/vcard/ns#", "vCard"),
    ("time", "http://www.w3.org/2006/time#", "OWL-Time"),
    ("qb", "http://purl.org/linked-data/cube#", "RDF Data Cube"),
    ("sioc", "http://rdfs.org/sioc/ns#", "SIOC"),
    (
        "org",
        "http://www.w3.org/ns/org#",
        "W3C organization ontology",
    ),
    ("dcat", "http://www.w3.org/ns/dcat#", "DCAT data catalogs"),
    ("ldp", "http://www.w3.org/ns/ldp#", "Linked Data Platform"),
];

// ---- Output structs (the Rust surface; serde-serializable for `to_json`) ------

/// Tuning knobs for [`Introspection::build_with`]. The defaults are sized for LLM
/// grounding: small per-entry histograms, a bounded characteristic-set table.
#[derive(Clone, Debug)]
pub struct BuildOptions {
    /// Sample object values kept per predicate (taken from the sorted POS block, so
    /// they are deterministic and start at the predicate's minimum value).
    pub samples_per_predicate: usize,
    /// Cap on each class histogram: inferred domains/ranges per predicate and
    /// `rdf:type` annotations per characteristic set.
    pub max_classes_per_histogram: usize,
    /// Characteristic sets retained (by descending subject count); the long tail is
    /// aggregated into [`CharacteristicSets::elided_sets`]/`elided_subjects`.
    pub max_char_sets: usize,
    /// Namespaces retained (by descending term count); the long tail — datasets like
    /// olympics mint thousands of per-instance namespaces — is aggregated into
    /// [`Vocabularies::elided_namespaces`]/`elided_terms`.
    pub max_namespaces: usize,
    /// Cross-class `(C, p, D)` join hints retained (by descending triple count); the
    /// tail is aggregated into [`JoinHints::elided_hints`]/`elided_triples`.
    pub max_join_hints: usize,
    /// Sample values are truncated to this many characters.
    pub max_sample_chars: usize,
}

impl Default for BuildOptions {
    fn default() -> Self {
        BuildOptions {
            samples_per_predicate: 3,
            max_classes_per_histogram: 8,
            max_char_sets: 1000,
            max_namespaces: 200,
            max_join_hints: 1000,
            max_sample_chars: 60,
        }
    }
}

/// An IRI with a count — the unit of every histogram in this crate.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Counted {
    pub iri: String,
    pub count: u64,
}

/// One distinct **characteristic set** (Neumann & Moerkotte): the exact set of
/// predicates emitted by some group of subjects.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CharacteristicSet {
    /// The predicate IRIs, sorted lexicographically (deterministic across store
    /// builds — dictionary-id order varies with the build path).
    pub predicates: Vec<String>,
    /// How many subjects have *exactly* this predicate set (`count(C)` in the paper).
    pub subjects: u64,
    /// Aligned with `predicates`: total triples those subjects emit per predicate.
    /// Average multiplicity (the paper's per-predicate occurrence statistic) is
    /// `predicate_triples[i] / subjects`.
    pub predicate_triples: Vec<u64>,
    /// `rdf:type` histogram of the subjects in this set (top entries) — the declared
    /// classes behind this emergent entity type, when any exist.
    pub classes: Vec<Counted>,
}

/// The characteristic-set table: the retained top sets plus exact tail aggregates.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CharacteristicSets {
    /// Total number of *distinct* characteristic sets in the graph.
    pub distinct: u64,
    /// The top sets by subject count (bounded by [`BuildOptions::max_char_sets`]).
    pub sets: Vec<CharacteristicSet>,
    /// Distinct sets beyond the cap, and the subjects they cover.
    pub elided_sets: u64,
    pub elided_subjects: u64,
}

/// Usage of one predicate on instances of one class.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClassPredicate {
    pub predicate: String,
    /// Instances of the class with at least one triple via this predicate.
    pub subjects: u64,
    /// Total triples via this predicate whose subject is an instance of the class.
    pub triples: u64,
    /// `subjects / instances` of the class, in `[0, 1]`.
    pub coverage: f64,
    /// [OPUS-4.8] sq-3n4: sample object values **scoped to this class** (literals
    /// quoted, IRIs bare), bounded by [`BuildOptions::samples_per_predicate`]. Unlike
    /// [`PredicateProfile::samples`] — which are global across every subject of the
    /// predicate and so can show values that belong only to a *different*, larger class
    /// (the "looks odd on minority classes" problem) — these are drawn only from triples
    /// whose subject is an instance of *this* class. Selection is the lexicographically
    /// smallest rendered distinct values among that class's objects, so it is
    /// deterministic across store builds (dictionary-id order varies with the build
    /// path).
    pub samples: Vec<String>,
}

/// A class (an `rdf:type` object) and how its instances are described.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClassProfile {
    pub class: String,
    pub instances: u64,
    /// Predicates appearing on instances of this class, by descending subject count.
    pub predicates: Vec<ClassPredicate>,
}

/// Triple counts by object kind for one predicate (the literal-vs-IRI split).
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct ObjectKinds {
    pub iri: u64,
    pub literal: u64,
    pub blank: u64,
    pub triple_term: u64,
}

/// Global statistics for one predicate.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PredicateProfile {
    pub predicate: String,
    pub triples: u64,
    pub distinct_subjects: u64,
    pub distinct_objects: u64,
    /// Triple counts by object kind.
    pub objects: ObjectKinds,
    /// `objects.literal / triples` — the literal-vs-IRI object ratio, in `[0, 1]`.
    pub literal_fraction: f64,
    /// Datatype distribution of literal objects (triples per datatype IRI;
    /// language-tagged strings appear as `rdf:langString`, inline integers as
    /// `xsd:integer`).
    pub datatypes: Vec<Counted>,
    /// Sample object values (literals quoted, IRIs bare): the lexicographically
    /// smallest rendered distinct values, so the selection is deterministic across
    /// store builds (index order varies with dictionary-id assignment).
    pub samples: Vec<String>,
    /// **Observed domain**: classes of this predicate's subjects, by descending
    /// distinct-subject count (most-common first) — inferred from usage.
    pub inferred_domains: Vec<Counted>,
    /// **Observed range**: classes of this predicate's IRI objects, by descending
    /// distinct-object count — inferred from usage.
    pub inferred_ranges: Vec<Counted>,
    /// Declared `rdfs:domain` / `rdfs:range` objects present in the graph (often
    /// absent or wrong — which is why the observed histograms exist).
    pub declared_domains: Vec<String>,
    pub declared_ranges: Vec<String>,
}

/// A namespace in use, with its distinct-term count and (when recognised) the
/// well-known prefix and title from the bundled vocabulary table.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VocabularyUse {
    pub namespace: String,
    pub prefix: Option<String>,
    pub title: Option<String>,
    /// Distinct dictionary terms (IRIs) under this namespace.
    pub terms: u64,
}

/// The detected vocabularies: top namespaces plus exact tail aggregates.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vocabularies {
    /// Total number of distinct namespaces in the dictionary.
    pub distinct: u64,
    /// The top namespaces by term count (bounded by [`BuildOptions::max_namespaces`]).
    pub namespaces: Vec<VocabularyUse>,
    /// Namespaces beyond the cap, and the distinct terms they cover.
    pub elided_namespaces: u64,
    pub elided_terms: u64,
}

/// A cross-class join hint: `(subject_class) --predicate--> (object_class)` with the
/// triple count behind that edge — the C–p→D table the introspect TODO records as a
/// follow-up. Mined from the same SPO scan that builds the characteristic sets: for
/// each triple whose subject is typed `C` and whose (IRI) object is typed `D`, the
/// `(C, p, D)` cell is incremented. It quantifies which classes actually join through
/// which predicate, the join-cardinality signal a planner or NL→SPARQL prompt wants
/// beyond the per-predicate global observed range.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct JoinHint {
    pub subject_class: String,
    pub predicate: String,
    pub object_class: String,
    /// Triples whose subject is an instance of `subject_class`, whose predicate is
    /// `predicate`, and whose (IRI) object is an instance of `object_class`. A triple
    /// contributes to every `(C, p, D)` cell its subject's and object's types span
    /// (multi-typed subjects/objects count under each declared type).
    pub triples: u64,
}

/// The cross-class join-hint table: the retained top edges plus exact tail aggregates.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JoinHints {
    /// Total number of *distinct* `(C, p, D)` edges observed.
    pub distinct: u64,
    /// The top edges by triple count (bounded by [`BuildOptions::max_join_hints`]).
    pub hints: Vec<JoinHint>,
    /// Edges beyond the cap, and the triples they cover.
    pub elided_hints: u64,
    pub elided_triples: u64,
}

/// The full introspection result. Build with [`Introspection::build`]; export with
/// [`to_json`](Introspection::to_json) / [`to_text_summary`](Introspection::to_text_summary).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Introspection {
    pub triples: u64,
    /// Distinct subjects.
    pub subjects: u64,
    /// Distinct subjects that carry at least one `rdf:type` (the typed entities — the
    /// `void:entities` count). `<= subjects`.
    pub entities: u64,
    /// Classes by descending instance count.
    pub classes: Vec<ClassProfile>,
    /// Predicates by descending triple count.
    pub predicates: Vec<PredicateProfile>,
    pub characteristic_sets: CharacteristicSets,
    /// Cross-class `(C, p, D)` join hints, by descending triple count.
    pub join_hints: JoinHints,
    /// Namespaces in use, by descending term count.
    pub vocabularies: Vocabularies,
}

// ---- Build -------------------------------------------------------------------

/// Per-predicate accumulator filled by the SPO scan.
#[derive(Default)]
struct PredAcc {
    triples: u64,
    distinct_subjects: u64,
    /// class id -> distinct subjects of this predicate typed with the class.
    domains: FxHashMap<Id, u64>,
}

/// [OPUS-4.8] sq-3n4: per (class, predicate) accumulator — usage counts plus the
/// class-scoped sample labels. `samples` holds the lex-smallest distinct rendered
/// object values seen on triples of this predicate whose subject is an instance of the
/// class (bounded by [`BuildOptions::samples_per_predicate`]).
#[derive(Default)]
struct ClassPredAcc {
    /// Instances of the class with at least one triple via this predicate.
    subjects: u64,
    /// Total triples via this predicate whose subject is an instance of the class.
    triples: u64,
    /// Class-scoped sample object values, lex-smallest distinct first.
    samples: Vec<String>,
}

/// Per-characteristic-set accumulator.
/// Keep the `cap` lexicographically smallest sample strings, ascending — the
/// selection (not just the order) is then independent of dictionary-id assignment.
fn keep_min_sample(samples: &mut Vec<String>, cap: usize, s: String) {
    if samples.len() < cap {
        samples.push(s);
        samples.sort_unstable();
    } else if let Some(last) = samples.last_mut() {
        if s < *last {
            *last = s;
            samples.sort_unstable();
        }
    }
}

/// [OPUS-4.8] sq-3n4: as [`keep_min_sample`], but skips a value already retained — the
/// per-class sample path feeds *every* triple's object (not the POS-collapsed distinct
/// objects the per-predicate path sees), so the same value can arrive repeatedly; this
/// keeps the retained set a set of **distinct** rendered values, matching the
/// per-predicate samples' distinct-object semantics.
fn keep_min_sample_distinct(samples: &mut Vec<String>, cap: usize, s: String) {
    // A repeated value that is already retained, or one not smaller than the current
    // max once the cap is full, contributes nothing — cheap guards before the insert.
    if samples.contains(&s) {
        return;
    }
    keep_min_sample(samples, cap, s);
}

struct CsAcc {
    subjects: u64,
    /// Aligned with the key's predicates: Σ triples.
    triples: Vec<u64>,
    /// class id -> subjects in this set typed with the class.
    classes: FxHashMap<Id, u64>,
}

/// One characteristic set in **dictionary-id space** — the pre-resolution form the
/// engine's `cs-planner` feature consumes (no IRI strings; ids are the queried
/// graph's own dictionary ids). See [`characteristic_set_ids`].
#[derive(Debug, Clone)]
pub struct CsIdSet {
    /// The predicate ids, ascending (the SPO scan emits each subject's predicates
    /// sorted, so the set key is naturally ordered).
    pub predicates: Box<[Id]>,
    /// Subjects whose exact predicate set this is (`count(C)`).
    pub subjects: u64,
    /// Aligned with `predicates`: total triples those subjects emit per predicate
    /// (`avg_mult(C, p) = predicate_triples[i] / subjects`).
    pub predicate_triples: Box<[u64]>,
}

/// The EXACT characteristic-set table of the graph's default graph in
/// **dictionary-id space** — the planner-facing accessor recorded in this crate's
/// TODO ("keep the pre-resolution `FxHashMap<Box<[Id]>, CsAcc>` form behind a
/// `cs-planner`-facing accessor"). One full SPO scan, no string resolution, no
/// caps (unlike [`Introspection::build`]'s LLM-facing table, which resolves IRIs
/// and elides the tail): cardinality estimation needs every set, exactly.
///
/// Feed it to `sparq_engine::cs::CsTable` (the engine's opt-in `cs-planner`
/// feature) by mapping each entry to the engine's `CsSet`; the ids are only
/// meaningful against THIS graph's dictionary, so rebuild the table whenever the
/// graph is rebuilt. Determinism: sets are returned by descending subject count,
/// ties by predicate-id key.
pub fn characteristic_set_ids(graph: &Graph) -> Vec<CsIdSet> {
    let mut cs: FxHashMap<Box<[Id]>, (u64, Vec<u64>)> = FxHashMap::default();
    let scan = graph.store.scan(&[None, None, None]);
    let rows = scan.rows.as_ref();
    let (mut ps, mut ms): (Vec<Id>, Vec<u64>) = (Vec::new(), Vec::new());
    let mut i = 0;
    while i < rows.len() {
        let [s, ..] = scan.to_spo(&rows[i]);
        ps.clear();
        ms.clear();
        // One subject run: predicates arrive sorted; count each predicate's triples.
        let mut j = i;
        while j < rows.len() {
            let [s2, p, _] = scan.to_spo(&rows[j]);
            if s2 != s {
                break;
            }
            let mut k = j;
            while k < rows.len() {
                let [s3, p3, _] = scan.to_spo(&rows[k]);
                if s3 != s || p3 != p {
                    break;
                }
                k += 1;
            }
            ps.push(p);
            ms.push((k - j) as u64);
            j = k;
        }
        match cs.get_mut(ps.as_slice()) {
            Some((subjects, triples)) => {
                *subjects += 1;
                for (idx, &m) in ms.iter().enumerate() {
                    triples[idx] += m;
                }
            }
            None => {
                cs.insert(ps.clone().into_boxed_slice(), (1, ms.clone()));
            }
        }
        i = j;
    }
    drop(scan);
    let mut sets: Vec<CsIdSet> = cs
        .into_iter()
        .map(|(predicates, (subjects, triples))| CsIdSet {
            predicates,
            subjects,
            predicate_triples: triples.into_boxed_slice(),
        })
        .collect();
    sets.sort_unstable_by(|a, b| b.subjects.cmp(&a.subjects).then_with(|| a.predicates.cmp(&b.predicates)));
    sets
}

impl Introspection {
    /// Builds the full introspection with default [`BuildOptions`]. Cost: one full SPO
    /// scan (characteristic sets + per-class usage + observed domains + cross-class
    /// join hints), one pass over the POS blocks of every predicate (object kinds,
    /// datatypes, samples, observed ranges), one pass over the dictionary
    /// (vocabularies) — all sorted scans over indexes the store already keeps.
    pub fn build(graph: &Graph) -> Introspection {
        Self::build_with(graph, &BuildOptions::default())
    }

    pub fn build_with(graph: &Graph, opts: &BuildOptions) -> Introspection {
        let id_of = |iri: &str| graph.id_of(&Term::NamedNode(NamedNode::new_unchecked(iri)));
        let type_id = id_of(RDF_TYPE);

        // ---- 1. Type map: subject -> class ids, class -> instance count. One range
        // scan of the rdf:type block. Only IRI objects count as classes.
        let mut subj_types: FxHashMap<Id, Vec<Id>> = FxHashMap::default();
        let mut class_instances: FxHashMap<Id, u64> = FxHashMap::default();
        if let Some(tid) = type_id {
            let scan = graph.store.scan(&[None, Some(tid), None]);
            for row in scan.rows.iter() {
                let [s, _, c] = scan.to_spo(row);
                if is_iri(graph, c) {
                    subj_types.entry(s).or_default().push(c);
                    *class_instances.entry(c).or_default() += 1;
                }
            }
        }

        // ---- 2. One full SPO scan: subjects are contiguous and predicates sorted
        // within each subject run, so the per-subject predicate set (with per-predicate
        // multiplicities) falls out of run boundaries — the characteristic-set build,
        // per-predicate subject stats, observed domains, and per-class usage all in
        // one pass.
        let mut cs: FxHashMap<Box<[Id]>, CsAcc> = FxHashMap::default();
        let mut preds: FxHashMap<Id, PredAcc> = FxHashMap::default();
        // [OPUS-4.8] sq-3n4: per (class, predicate): (subjects, triples, class-scoped
        // sample labels). The samples are the lex-smallest distinct rendered objects
        // among triples whose subject is an instance of the class — so a minority class
        // shows ITS OWN representative values, not the predicate's global minimum (which
        // can belong entirely to a different, larger class).
        let mut class_preds: FxHashMap<Id, FxHashMap<Id, ClassPredAcc>> = FxHashMap::default();
        // Cross-class join hints: (subject_class, predicate, object_class) -> triples.
        // Filled in the same scan — when the subject is typed AND the object is a typed
        // IRI, the cell for every (C, p, D) the subject's/object's types span is bumped.
        let mut join: FxHashMap<(Id, Id, Id), u64> = FxHashMap::default();
        let mut subjects: u64 = 0;
        let mut entities: u64 = 0;

        let scan = graph.store.scan(&[None, None, None]);
        let rows = scan.rows.as_ref();
        // The full scan is served by a subject-leading permutation; map rows to
        // canonical (s, p, o) via the scan's own permutation for safety.
        let (mut ps, mut ms): (Vec<Id>, Vec<u64>) = (Vec::new(), Vec::new());
        let mut i = 0;
        while i < rows.len() {
            let [s, ..] = scan.to_spo(&rows[i]);
            let types = subj_types.get(&s);
            ps.clear();
            ms.clear();
            let mut j = i;
            while j < rows.len() {
                let [s2, p, _] = scan.to_spo(&rows[j]);
                if s2 != s {
                    break;
                }
                let mut k = j;
                while k < rows.len() {
                    let [s3, p3, o3] = scan.to_spo(&rows[k]);
                    if s3 != s || p3 != p {
                        break;
                    }
                    // Cross-class join hint: this triple's (typed subject) --p--> (typed
                    // IRI object). One object-type lookup per triple; only typed
                    // subjects with typed object reach the inner product.
                    if let Some(ts) = types {
                        if let Some(os) = subj_types.get(&o3) {
                            for &c in ts {
                                for &d in os {
                                    *join.entry((c, p, d)).or_default() += 1;
                                }
                            }
                        }
                        // [OPUS-4.8] sq-3n4: class-scoped sample labels. Render this
                        // typed subject's object once and offer it to each of the
                        // subject's classes' (class, predicate) sample sets — so a
                        // minority class keeps its OWN representative values instead of
                        // the predicate's global minimum. Rendered identically to the
                        // global per-predicate samples; the distinct-keeping helper
                        // matches their distinct-object semantics.
                        let rendered = render_object_sample(graph, o3, opts.max_sample_chars);
                        for &c in ts {
                            let cp = class_preds.entry(c).or_default().entry(p).or_default();
                            keep_min_sample_distinct(
                                &mut cp.samples,
                                opts.samples_per_predicate,
                                rendered.clone(),
                            );
                        }
                    }
                    k += 1;
                }
                ps.push(p);
                ms.push((k - j) as u64);
                j = k;
            }
            subjects += 1;
            if types.is_some() {
                entities += 1;
            }
            for (idx, &p) in ps.iter().enumerate() {
                let pa = preds.entry(p).or_default();
                pa.triples += ms[idx];
                pa.distinct_subjects += 1;
                if let Some(ts) = types {
                    for &c in ts {
                        *pa.domains.entry(c).or_default() += 1;
                        let cp = class_preds.entry(c).or_default().entry(p).or_default();
                        cp.subjects += 1;
                        cp.triples += ms[idx];
                    }
                }
            }
            if !cs.contains_key(ps.as_slice()) {
                cs.insert(
                    ps.clone().into_boxed_slice(),
                    CsAcc {
                        subjects: 0,
                        triples: vec![0; ps.len()],
                        classes: FxHashMap::default(),
                    },
                );
            }
            let acc = cs.get_mut(ps.as_slice()).expect("just inserted");
            acc.subjects += 1;
            for (idx, &m) in ms.iter().enumerate() {
                acc.triples[idx] += m;
            }
            if let Some(ts) = types {
                for &c in ts {
                    *acc.classes.entry(c).or_default() += 1;
                }
            }
            i = j;
        }
        drop(scan);

        // ---- 3. Per-predicate object profile: one object-sorted (POS) range scan per
        // predicate — equal objects are adjacent, so distinct objects, kind counts,
        // datatype distribution, samples, and the observed range (type lookup once per
        // distinct object) all come from run boundaries. Σ over predicates = one full
        // POS pass.
        struct ObjAcc {
            distinct: u64,
            kinds: ObjectKinds,
            datatypes: FxHashMap<String, u64>,
            ranges: FxHashMap<Id, u64>,
            samples: Vec<String>,
        }
        let mut objs: FxHashMap<Id, ObjAcc> = FxHashMap::default();
        for &p in preds.keys() {
            let scan = graph.store.scan_sorted(&[None, Some(p), None], 2);
            let rows = scan.rows.as_ref();
            let mut acc = ObjAcc {
                distinct: 0,
                kinds: ObjectKinds::default(),
                datatypes: FxHashMap::default(),
                ranges: FxHashMap::default(),
                samples: Vec::new(),
            };
            let mut i = 0;
            while i < rows.len() {
                let [_, _, o] = scan.to_spo(&rows[i]);
                let mut k = i;
                while k < rows.len() && scan.to_spo(&rows[k])[2] == o {
                    k += 1;
                }
                let n = (k - i) as u64;
                acc.distinct += 1;
                // Kind / datatype / observed-range accounting (needs `n` + the parts).
                if dict::is_inline(o) {
                    acc.kinds.literal += n;
                    *acc.datatypes.entry(XSD_INTEGER.to_string()).or_default() += n;
                } else {
                    match graph.dict.term_parts(o) {
                        TermParts::Iri { .. } => {
                            acc.kinds.iri += n;
                            if let Some(ts) = subj_types.get(&o) {
                                for &c in ts {
                                    *acc.ranges.entry(c).or_default() += 1;
                                }
                            }
                        }
                        TermParts::Lit { datatype, .. } => {
                            acc.kinds.literal += n;
                            *acc.datatypes.entry(datatype.to_string()).or_default() += n;
                        }
                        TermParts::Blank(_) => acc.kinds.blank += n,
                        TermParts::Triple(_) => acc.kinds.triple_term += n,
                    }
                }
                // Sample label — rendered by the shared helper so the global samples and
                // the per-class samples ([OPUS-4.8] sq-3n4) are byte-identical. Distinct
                // objects are already run-collapsed here, so `keep_min_sample` suffices.
                keep_min_sample(
                    &mut acc.samples,
                    opts.samples_per_predicate,
                    render_object_sample(graph, o, opts.max_sample_chars),
                );
                i = k;
            }
            objs.insert(p, acc);
        }

        // ---- 4. Declared rdfs:domain / rdfs:range (when present): one range scan each.
        let mut declared_domains: FxHashMap<Id, Vec<Id>> = FxHashMap::default();
        let mut declared_ranges: FxHashMap<Id, Vec<Id>> = FxHashMap::default();
        for (iri, map) in [
            (RDFS_DOMAIN, &mut declared_domains),
            (RDFS_RANGE, &mut declared_ranges),
        ] {
            if let Some(did) = id_of(iri) {
                let scan = graph.store.scan(&[None, Some(did), None]);
                for row in scan.rows.iter() {
                    let [s, _, c] = scan.to_spo(row);
                    if is_iri(graph, c) {
                        map.entry(s).or_default().push(c);
                    }
                }
            }
        }

        // ---- 5. Vocabulary detection: one pass over the dictionary. The dictionary
        // already stores every IRI split at the last `#`/`/` (its prefix table), so the
        // namespace of each distinct term is a borrowed lookup.
        let mut ns_terms: FxHashMap<&str, u64> = FxHashMap::default();
        for id in 1..=graph.dict.len() as Id {
            if let TermParts::Iri { prefix, .. } = graph.dict.term_parts(id) {
                *ns_terms.entry(prefix).or_default() += 1;
            }
        }
        let distinct_ns = ns_terms.len() as u64;
        let mut namespaces: Vec<VocabularyUse> = ns_terms
            .into_iter()
            .map(|(ns, terms)| {
                let known = WELL_KNOWN.iter().find(|(_, n, _)| *n == ns);
                VocabularyUse {
                    namespace: ns.to_string(),
                    prefix: known.map(|(p, _, _)| p.to_string()),
                    title: known.map(|(_, _, t)| t.to_string()),
                    terms,
                }
            })
            .collect();
        namespaces.sort_by(|a, b| b.terms.cmp(&a.terms).then(a.namespace.cmp(&b.namespace)));
        let mut elided_namespaces = 0u64;
        let mut elided_terms = 0u64;
        if namespaces.len() > opts.max_namespaces {
            for v in &namespaces[opts.max_namespaces..] {
                elided_namespaces += 1;
                elided_terms += v.terms;
            }
            namespaces.truncate(opts.max_namespaces);
        }
        let vocabularies = Vocabularies {
            distinct: distinct_ns,
            namespaces,
            elided_namespaces,
            elided_terms,
        };

        // ---- 6. Resolve ids -> strings and assemble, sorted most-frequent-first.
        let iri_str = |id: Id| -> String {
            match graph.dict.term_parts(id) {
                TermParts::Iri { prefix, suffix } => format!("{prefix}{suffix}"),
                _ => graph.dict.term(id).to_string(), // non-IRI (defensive)
            }
        };
        let top_counted = |map: &FxHashMap<Id, u64>, cap: usize| -> Vec<Counted> {
            let mut v: Vec<(Id, u64)> = map.iter().map(|(&k, &n)| (k, n)).collect();
            v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
            v.truncate(cap);
            v.into_iter()
                .map(|(k, n)| Counted {
                    iri: iri_str(k),
                    count: n,
                })
                .collect()
        };

        let mut classes: Vec<ClassProfile> = class_instances
            .iter()
            .map(|(&c, &instances)| {
                let mut predicates: Vec<ClassPredicate> = class_preds
                    .get(&c)
                    .map(|m| {
                        m.iter()
                            .map(|(&p, cp)| ClassPredicate {
                                predicate: iri_str(p),
                                subjects: cp.subjects,
                                triples: cp.triples,
                                coverage: cp.subjects as f64 / instances.max(1) as f64,
                                samples: cp.samples.clone(),
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                predicates.sort_by(|a, b| {
                    b.subjects
                        .cmp(&a.subjects)
                        .then(a.predicate.cmp(&b.predicate))
                });
                ClassProfile {
                    class: iri_str(c),
                    instances,
                    predicates,
                }
            })
            .collect();
        classes.sort_by(|a, b| b.instances.cmp(&a.instances).then(a.class.cmp(&b.class)));

        let mut predicates: Vec<PredicateProfile> = preds
            .iter()
            .map(|(&p, pa)| {
                let oa = objs
                    .get(&p)
                    .expect("object profile built for every predicate");
                let mut datatypes: Vec<Counted> = oa
                    .datatypes
                    .iter()
                    .map(|(dt, &n)| Counted {
                        iri: dt.clone(),
                        count: n,
                    })
                    .collect();
                datatypes.sort_by(|a, b| b.count.cmp(&a.count).then(a.iri.cmp(&b.iri)));
                PredicateProfile {
                    predicate: iri_str(p),
                    triples: pa.triples,
                    distinct_subjects: pa.distinct_subjects,
                    distinct_objects: oa.distinct,
                    objects: oa.kinds,
                    literal_fraction: oa.kinds.literal as f64 / pa.triples.max(1) as f64,
                    datatypes,
                    samples: oa.samples.clone(),
                    inferred_domains: top_counted(&pa.domains, opts.max_classes_per_histogram),
                    inferred_ranges: top_counted(&oa.ranges, opts.max_classes_per_histogram),
                    declared_domains: declared_domains
                        .get(&p)
                        .map(|v| v.iter().map(|&c| iri_str(c)).collect())
                        .unwrap_or_default(),
                    declared_ranges: declared_ranges
                        .get(&p)
                        .map(|v| v.iter().map(|&c| iri_str(c)).collect())
                        .unwrap_or_default(),
                }
            })
            .collect();
        predicates.sort_by(|a, b| {
            b.triples
                .cmp(&a.triples)
                .then(a.predicate.cmp(&b.predicate))
        });

        let distinct_cs = cs.len() as u64;
        // Resolve each set's predicates up front and sort them lexicographically
        // (with the per-predicate triple counts kept aligned), so both the in-set
        // order and the between-set tie-break are independent of dictionary-id
        // assignment, which varies with the store build path.
        let mut cs_vec: Vec<(Vec<(String, u64)>, CsAcc)> = cs
            .into_iter()
            .map(|(key, acc)| {
                let mut preds: Vec<(String, u64)> = key
                    .iter()
                    .zip(&acc.triples)
                    .map(|(&p, &t)| (iri_str(p), t))
                    .collect();
                preds.sort_unstable();
                (preds, acc)
            })
            .collect();
        cs_vec.sort_by(|a, b| b.1.subjects.cmp(&a.1.subjects).then_with(|| a.0.cmp(&b.0)));
        let mut elided_sets = 0u64;
        let mut elided_subjects = 0u64;
        if cs_vec.len() > opts.max_char_sets {
            for (_, acc) in &cs_vec[opts.max_char_sets..] {
                elided_sets += 1;
                elided_subjects += acc.subjects;
            }
            cs_vec.truncate(opts.max_char_sets);
        }
        let sets: Vec<CharacteristicSet> = cs_vec
            .into_iter()
            .map(|(preds, acc)| CharacteristicSet {
                predicates: preds.iter().map(|(s, _)| s.clone()).collect(),
                subjects: acc.subjects,
                predicate_triples: preds.iter().map(|&(_, t)| t).collect(),
                classes: top_counted(&acc.classes, opts.max_classes_per_histogram),
            })
            .collect();

        // Cross-class join hints: resolve ids, sort by descending triple count
        // (tie-break on the resolved triple key for determinism across store builds),
        // cap, and aggregate the tail.
        let distinct_join = join.len() as u64;
        let mut join_vec: Vec<JoinHint> = join
            .into_iter()
            .map(|((c, p, d), triples)| JoinHint {
                subject_class: iri_str(c),
                predicate: iri_str(p),
                object_class: iri_str(d),
                triples,
            })
            .collect();
        join_vec.sort_by(|a, b| {
            b.triples.cmp(&a.triples).then_with(|| {
                (&a.subject_class, &a.predicate, &a.object_class).cmp(&(
                    &b.subject_class,
                    &b.predicate,
                    &b.object_class,
                ))
            })
        });
        let mut elided_hints = 0u64;
        let mut elided_join_triples = 0u64;
        if join_vec.len() > opts.max_join_hints {
            for h in &join_vec[opts.max_join_hints..] {
                elided_hints += 1;
                elided_join_triples += h.triples;
            }
            join_vec.truncate(opts.max_join_hints);
        }

        Introspection {
            triples: graph.len() as u64,
            subjects,
            entities,
            classes,
            predicates,
            characteristic_sets: CharacteristicSets {
                distinct: distinct_cs,
                sets,
                elided_sets,
                elided_subjects,
            },
            join_hints: JoinHints {
                distinct: distinct_join,
                hints: join_vec,
                elided_hints,
                elided_triples: elided_join_triples,
            },
            vocabularies,
        }
    }

    /// Serialises the full introspection to pretty-printed JSON — the machine surface
    /// for LLM grounding (and for any other tool).
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("introspection serialises to JSON")
    }

    /// [OPUS-4.8] sq-3n4: parses a persisted introspection back from the JSON
    /// [`to_json`](Introspection::to_json) produced — the in-memory inverse of
    /// [`save`](Introspection::save)/[`load`](Introspection::load) for callers that hold
    /// the bytes themselves (e.g. a WASM tab that cached the sidecar in IndexedDB). Once
    /// rehydrated, every O(output) export — [`to_text_summary`](Introspection::to_text_summary),
    /// [`schema_summary_for`](Introspection::schema_summary_for),
    /// [`to_void`](Introspection::to_void), … — runs off the struct with **no graph
    /// rescan**, the whole point of the sidecar.
    pub fn from_json(json: &str) -> serde_json::Result<Introspection> {
        serde_json::from_str(json)
    }

    /// [OPUS-4.8] sq-3n4: writes the introspection to a persisted **`*.introspect`
    /// sidecar** — the mined effective schema as JSON, alongside the source graph — so a
    /// later process can produce summaries / VoID / retrieval-mode cards **without
    /// rescanning the graph** ([`build`](Introspection::build) is `O(|G| + |dict|)`;
    /// reloading the sidecar is `O(output)`). The conventional extension is
    /// [`SIDECAR_EXTENSION`] (`.introspect`); see [`sidecar_path_for`] to derive it from
    /// a dataset path. The format is exactly [`to_json`](Introspection::to_json)'s, so a
    /// sidecar is also a plain JSON document any other tool can read.
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        std::fs::write(path, self.to_json())
    }

    /// [OPUS-4.8] sq-3n4: loads a persisted [`save`](Introspection::save) sidecar. A
    /// malformed sidecar surfaces as [`std::io::ErrorKind::InvalidData`] (so the one
    /// `io::Result` covers both the read and the parse). Round-trips
    /// [`save`](Introspection::save) exactly.
    pub fn load(path: impl AsRef<std::path::Path>) -> std::io::Result<Introspection> {
        let json = std::fs::read_to_string(path)?;
        Self::from_json(&json).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Emits a [W3C VoID](https://www.w3.org/TR/void/) description of the dataset as
    /// **N-Triples** (a syntactic subset of Turtle, so the output parses as either —
    /// no serializer dependency, oxrdf renders every term RFC-correctly).
    ///
    /// `dataset_iri` names the `void:Dataset` resource (e.g. the dataset's URL).
    /// The top-level dataset carries:
    /// - `void:triples` — total triples (exact);
    /// - `void:entities` — distinct subjects carrying an `rdf:type` (exact);
    /// - `void:distinctSubjects` — distinct subjects (exact);
    /// - `void:classes` — distinct classes (`rdf:type` objects) (exact);
    /// - `void:properties` — distinct predicates (exact).
    ///
    /// It also emits one `void:classPartition` per class (with `void:class` and
    /// `void:entities` = instance count) and one `void:propertyPartition` per predicate
    /// (with `void:property`, `void:triples`, and `void:distinctSubjects` for that
    /// predicate). Partitions are blank nodes.
    ///
    /// NOT emitted (honest scope): `void:distinctObjects` — the crate tracks distinct
    /// objects only *per predicate*, never a global de-duplicated count, so a faithful
    /// global figure is unavailable without an extra union pass; the per-predicate
    /// partitions carry `void:distinctSubjects` but not a per-predicate
    /// `void:distinctObjects` either (the per-predicate `distinct_objects` mixes IRIs
    /// and literals, which VoID's `void:distinctObjects` does not distinguish — left
    /// out rather than emitted misleadingly). `void:vocabulary`/`void:uriSpace`
    /// linkset partitions are also out of scope here.
    pub fn to_void(&self, dataset_iri: &str) -> String {
        use std::fmt::Write as _;
        let v = |local: &str| format!("{VOID_NS}{local}");
        let ds = NamedNode::new_unchecked(dataset_iri);
        let mut out = String::new();
        // Helpers: each writes one N-Triples line (subject is either the dataset IRI or
        // a blank node label `_:bN`). oxrdf NamedNode/Literal Display is N-Triples-safe.
        let iri = |s: &str| NamedNode::new_unchecked(s).to_string();
        let int = |n: u64| Literal::new_typed_literal(n.to_string(), xsd::INTEGER).to_string();

        // ---- Top-level dataset.
        let _ = writeln!(out, "{ds} <{RDF_TYPE}> <{}> .", v("Dataset"));
        let _ = writeln!(out, "{ds} <{}> {} .", v("triples"), int(self.triples));
        let _ = writeln!(out, "{ds} <{}> {} .", v("entities"), int(self.entities));
        let _ = writeln!(
            out,
            "{ds} <{}> {} .",
            v("distinctSubjects"),
            int(self.subjects)
        );
        let _ = writeln!(
            out,
            "{ds} <{}> {} .",
            v("classes"),
            int(self.classes.len() as u64)
        );
        let _ = writeln!(
            out,
            "{ds} <{}> {} .",
            v("properties"),
            int(self.predicates.len() as u64)
        );

        // ---- Class partitions (one blank node each).
        let mut bnode = 0u64;
        for c in &self.classes {
            let b = format!("_:c{bnode}");
            bnode += 1;
            let _ = writeln!(out, "{ds} <{}> {b} .", v("classPartition"));
            let _ = writeln!(out, "{b} <{}> {} .", v("class"), iri(&c.class));
            let _ = writeln!(out, "{b} <{}> {} .", v("entities"), int(c.instances));
        }

        // ---- Property partitions (one blank node each).
        for p in &self.predicates {
            let b = format!("_:p{bnode}");
            bnode += 1;
            let _ = writeln!(out, "{ds} <{}> {b} .", v("propertyPartition"));
            let _ = writeln!(out, "{b} <{}> {} .", v("property"), iri(&p.predicate));
            let _ = writeln!(out, "{b} <{}> {} .", v("triples"), int(p.triples));
            let _ = writeln!(
                out,
                "{b} <{}> {} .",
                v("distinctSubjects"),
                int(p.distinct_subjects)
            );
        }
        out
    }

    /// [OPUS-4.8] sq-mr32 (federation A3/Z2): the VoID description (`to_void`) **plus**
    /// the characteristic-set source statistics, as one N-Triples document.
    ///
    /// This is the served federation-descriptor surface: it is a strict superset of
    /// `to_void` — every standard VoID triple is emitted unchanged — followed by the
    /// characteristic-set extension (see `CS_NS`). sparq already mines these sets
    /// (Neumann & Moerkotte's per-entity-type predicate co-occurrence + per-predicate
    /// multiplicity); exposing them lets a remote, CostFed/Odyssey-class source-selector
    /// estimate star- and multi-join cardinalities against this node far more accurately
    /// than the bare `void:propertyPartition` counts allow.
    ///
    /// Shape, per retained characteristic set (`self.characteristic_sets.sets`, already
    /// bounded by [`BuildOptions::max_char_sets`] and ordered by descending subject
    /// count — so the served document is bounded and deterministic):
    ///
    /// ```text
    /// <dataset> scs:characteristicSet _:csN .
    /// <dataset> scs:distinctCharacteristicSets "<distinct>"^^xsd:integer .
    /// _:csN a scs:CharacteristicSet ;
    ///        scs:subjects "<count(C)>"^^xsd:integer ;
    ///        scs:predicateStat _:csN_M .
    /// _:csN_M void:property <predicate> ;
    ///          void:triples "<Σ triples>"^^xsd:integer ;
    ///          scs:avgMultiplicity "<triples/subjects>"^^xsd:decimal .
    /// ```
    ///
    /// The elided long tail is summarised on the dataset (`scs:distinctCharacteristicSets`
    /// is the EXACT distinct-set count, not just the retained count, so a consumer knows
    /// whether the served sets are complete). Reuses `void:property`/`void:triples` on the
    /// per-predicate nodes so a VoID-aware client still reads them as property partitions.
    pub fn to_void_with_cs(&self, dataset_iri: &str) -> String {
        use std::fmt::Write as _;
        let mut out = self.to_void(dataset_iri);
        let ds = NamedNode::new_unchecked(dataset_iri);
        let cs = |local: &str| format!("{CS_NS}{local}");
        let v = |local: &str| format!("{VOID_NS}{local}");
        let iri = |s: &str| NamedNode::new_unchecked(s).to_string();
        let int = |n: u64| Literal::new_typed_literal(n.to_string(), xsd::INTEGER).to_string();
        let dec = |val: &str| Literal::new_typed_literal(val.to_string(), xsd::DECIMAL).to_string();

        // Dataset-level: the EXACT distinct-set count (independent of how many were
        // retained for the served partitions), so a consumer knows the tail exists.
        let _ = writeln!(
            out,
            "{ds} <{}> {} .",
            cs("distinctCharacteristicSets"),
            int(self.characteristic_sets.distinct)
        );

        // One node per retained characteristic set. Blank-node labels are local to this
        // document and distinct from the VoID partitions' `_:cN`/`_:pN` (here `_:csN`,
        // `_:csN_M`), so the two blank-node spaces never collide.
        for (si, set) in self.characteristic_sets.sets.iter().enumerate() {
            let cnode = format!("_:cs{si}");
            let _ = writeln!(out, "{ds} <{}> {cnode} .", cs("characteristicSet"));
            let _ = writeln!(out, "{cnode} <{RDF_TYPE}> <{}> .", cs("CharacteristicSet"));
            let _ = writeln!(out, "{cnode} <{}> {} .", cs("subjects"), int(set.subjects));
            for (pi, (pred, &triples)) in set
                .predicates
                .iter()
                .zip(&set.predicate_triples)
                .enumerate()
            {
                let pnode = format!("_:cs{si}_{pi}");
                let _ = writeln!(out, "{cnode} <{}> {pnode} .", cs("predicateStat"));
                let _ = writeln!(out, "{pnode} <{}> {} .", v("property"), iri(pred));
                let _ = writeln!(out, "{pnode} <{}> {} .", v("triples"), int(triples));
                // avg multiplicity = triples / subjects, rendered as a fixed-precision
                // xsd:decimal (subjects >= 1 for any retained set).
                let mult = triples as f64 / set.subjects.max(1) as f64;
                let _ = writeln!(
                    out,
                    "{pnode} <{}> {} .",
                    cs("avgMultiplicity"),
                    dec(&format!("{mult:.4}"))
                );
            }
        }
        out
    }

    /// Renders a compact, prompt-ready text digest under `budget_chars` characters,
    /// most important information first: dataset totals, then a prefix glossary of
    /// exactly the namespaces the summary uses, then classes (with per-class predicate
    /// usage, coverage, range hints and samples), then the characteristic-set
    /// patterns, then global predicate stats. Output is truncated greedily at line
    /// granularity; a final `…` line marks elision.
    ///
    /// Prefixes are assigned lazily, in order of first use: well-known vocabularies
    /// get their conventional prefix (`foaf:`, `xsd:`, …), everything else `ns1`,
    /// `ns2`, … — so the glossary stays small and every compacted name in the body is
    /// resolvable from it.
    pub fn to_text_summary(&self, budget_chars: usize) -> String {
        let mut prefixes = PrefixAssigner::new();
        // ---- Body lines (everything below the glossary), built in full first so the
        // glossary can list exactly the namespaces in use.
        let mut body: Vec<String> = Vec::new();

        // Classes with per-class usage — the heart of the card deck.
        if !self.classes.is_empty() {
            body.push("## Classes (by instance count)".to_string());
            for c in &self.classes {
                body.push(format!(
                    "### {} — {} instances",
                    prefixes.compact(&c.class),
                    c.instances
                ));
                // rdf:type rows are implicit in the section structure — skip them.
                for cp in c
                    .predicates
                    .iter()
                    .filter(|cp| cp.predicate != RDF_TYPE)
                    .take(8)
                {
                    let pct = (cp.coverage * 100.0).round() as u64;
                    let mult = cp.triples as f64 / cp.subjects.max(1) as f64;
                    let mult = if mult > 1.05 {
                        format!(", avg {mult:.1}/subj")
                    } else {
                        String::new()
                    };
                    let hint = self.predicate_hint(
                        &cp.predicate,
                        cp.samples.first().map(|s| s.as_str()),
                        &mut prefixes,
                    );
                    body.push(format!(
                        "- {} — {}/{} subjects ({pct}%{mult}){hint}",
                        prefixes.compact(&cp.predicate),
                        cp.subjects,
                        c.instances
                    ));
                }
            }
        }

        // Characteristic sets: the emergent entity types.
        let cs = &self.characteristic_sets;
        if !cs.sets.is_empty() {
            body.push("## Entity patterns (characteristic predicate sets)".to_string());
            for set in cs.sets.iter().take(12) {
                let preds: Vec<String> =
                    set.predicates.iter().map(|p| prefixes.compact(p)).collect();
                let label = set
                    .classes
                    .first()
                    .map(|c| format!(" — mostly {}", prefixes.compact(&c.iri)))
                    .unwrap_or_default();
                body.push(format!(
                    "- {{{}}} × {} subjects{label}",
                    preds.join(", "),
                    set.subjects
                ));
            }
            let elided = cs.distinct.saturating_sub(cs.sets.len().min(12) as u64);
            if elided > 0 {
                body.push(format!("… and {elided} rarer patterns"));
            }
        }

        // Global predicate stats (selectivity signal).
        if !self.predicates.is_empty() {
            body.push("## Predicates (global)".to_string());
            for p in &self.predicates {
                let lit_pct = (p.literal_fraction * 100.0).round() as u64;
                let kinds = if lit_pct == 0 {
                    "IRIs".to_string()
                } else if lit_pct == 100 {
                    "literals".to_string()
                } else {
                    format!("{lit_pct}% literals")
                };
                let sample = p
                    .samples
                    .first()
                    .map(|s| format!(", e.g. {}", prefixes.compact(s)))
                    .unwrap_or_default();
                body.push(format!(
                    "- {} — {} triples, {} subj, {} obj ({kinds}{sample})",
                    prefixes.compact(&p.predicate),
                    p.triples,
                    p.distinct_subjects,
                    p.distinct_objects
                ));
            }
        }

        // ---- Assemble under the budget: header, glossary (used namespaces, in
        // first-use order), body.
        let mut w = BudgetWriter::new(budget_chars);
        w.line(&format!(
            "# Schema summary — {} triples, {} subjects, {} classes, {} predicates, {} entity patterns",
            self.triples,
            self.subjects,
            self.classes.len(),
            self.predicates.len(),
            self.characteristic_sets.distinct
        ));
        let vocab_of = |ns: &str| {
            self.vocabularies
                .namespaces
                .iter()
                .find(|v| v.namespace == ns)
        };
        if !prefixes.assigned.is_empty() && !w.full() {
            w.line("## Prefixes");
            for (ns, pfx) in &prefixes.assigned {
                let v = vocab_of(ns);
                let title = v
                    .and_then(|v| v.title.as_deref())
                    .or_else(|| {
                        WELL_KNOWN
                            .iter()
                            .find(|(_, n, _)| n == ns)
                            .map(|(_, _, t)| *t)
                    })
                    .map(|t| format!(" — {t}"))
                    .unwrap_or_default();
                let terms = v
                    .map(|v| format!(" ({} terms)", v.terms))
                    .unwrap_or_default();
                if !w.line(&format!("{pfx}: {ns}{title}{terms}")) {
                    break;
                }
            }
        }
        for l in &body {
            if !w.line(l) {
                break;
            }
        }
        w.finish()
    }

    /// A short range hint for a predicate line in the class section: the dominant
    /// observed range class, else the dominant literal datatype (from the predicate's
    /// GLOBAL profile), plus one sample. [OPUS-4.8] sq-3n4: when `class_sample` is
    /// supplied (the class-scoped sample for this predicate) it is preferred for the
    /// `e.g.` example — so a minority class shows ITS OWN representative value rather
    /// than the predicate's global minimum, which may belong only to a larger class; the
    /// global sample is the fallback when the class has none.
    fn predicate_hint(
        &self,
        predicate: &str,
        class_sample: Option<&str>,
        prefixes: &mut PrefixAssigner,
    ) -> String {
        let Some(p) = self.predicates.iter().find(|p| p.predicate == predicate) else {
            return String::new();
        };
        let mut hint = String::new();
        if let Some(r) = p.inferred_ranges.first() {
            hint.push_str(&format!(" → {}", prefixes.compact(&r.iri)));
        } else if let Some(dt) = p.datatypes.first() {
            hint.push_str(&format!(" → {}", prefixes.compact(&dt.iri)));
        }
        if let Some(s) = class_sample.or_else(|| p.samples.first().map(|s| s.as_str())) {
            hint.push_str(&format!(", e.g. {}", prefixes.compact(s)));
        }
        hint
    }

    /// **Retrieval-mode summary**: a prompt-ready digest scoped to a set of seed IRIs,
    /// for KGs whose full schema is too large to fit a prompt (the 10k-property-KG
    /// path). Each seed is matched against the mined schema — a seed naming a **class**
    /// pulls that class's profile (per-predicate usage) and the cross-class join edges
    /// it participates in (as subject or object); a seed naming a **predicate** pulls
    /// that predicate's global profile. Only the matched slice is rendered, under
    /// `budget_chars`, most-relevant-first, with the same prefix glossary discipline as
    /// `to_text_summary`.
    ///
    /// This is struct-level scoping (it filters the already-mined profiles by IRI);
    /// it does not re-scan the graph, so it cannot expand to neighbours the build did
    /// not already profile (e.g. it will not chase the *instances* of a seed entity —
    /// the crate retains class/predicate profiles, not per-subject adjacency). Seeds
    /// that match nothing are reported in a trailing note.
    pub fn schema_summary_for(&self, seeds: &[&str], budget_chars: usize) -> String {
        let mut prefixes = PrefixAssigner::new();
        let mut body: Vec<String> = Vec::new();
        let mut matched = 0usize;

        // ---- Seed classes: profile + join edges touching the class.
        for &seed in seeds {
            if let Some(c) = self.classes.iter().find(|c| c.class == seed) {
                matched += 1;
                body.push(format!(
                    "### {} — {} instances",
                    prefixes.compact(&c.class),
                    c.instances
                ));
                for cp in c
                    .predicates
                    .iter()
                    .filter(|cp| cp.predicate != RDF_TYPE)
                    .take(12)
                {
                    let pct = (cp.coverage * 100.0).round() as u64;
                    let hint = self.predicate_hint(
                        &cp.predicate,
                        cp.samples.first().map(|s| s.as_str()),
                        &mut prefixes,
                    );
                    body.push(format!(
                        "- {} — {}/{} subjects ({pct}%){hint}",
                        prefixes.compact(&cp.predicate),
                        cp.subjects,
                        c.instances
                    ));
                }
                // Cross-class join edges where this class is the subject or object.
                let edges: Vec<&JoinHint> = self
                    .join_hints
                    .hints
                    .iter()
                    .filter(|h| h.subject_class == seed || h.object_class == seed)
                    .take(8)
                    .collect();
                for h in edges {
                    body.push(format!(
                        "  join: {} --{}--> {} ({} triples)",
                        prefixes.compact(&h.subject_class),
                        prefixes.compact(&h.predicate),
                        prefixes.compact(&h.object_class),
                        h.triples
                    ));
                }
            }
        }

        // ---- Seed predicates: global profile.
        for &seed in seeds {
            if let Some(p) = self.predicates.iter().find(|p| p.predicate == seed) {
                matched += 1;
                let lit_pct = (p.literal_fraction * 100.0).round() as u64;
                let kinds = if lit_pct == 0 {
                    "IRIs".to_string()
                } else if lit_pct == 100 {
                    "literals".to_string()
                } else {
                    format!("{lit_pct}% literals")
                };
                let sample = p
                    .samples
                    .first()
                    .map(|s| format!(", e.g. {}", prefixes.compact(s)))
                    .unwrap_or_default();
                body.push(format!(
                    "### {} — {} triples, {} subj, {} obj ({kinds}{sample})",
                    prefixes.compact(&p.predicate),
                    p.triples,
                    p.distinct_subjects,
                    p.distinct_objects
                ));
                if let Some(d) = p.inferred_domains.first() {
                    body.push(format!("  domain: {}", prefixes.compact(&d.iri)));
                }
                if let Some(r) = p.inferred_ranges.first() {
                    body.push(format!("  range: {}", prefixes.compact(&r.iri)));
                }
            }
        }

        let unmatched: Vec<&&str> = seeds
            .iter()
            .filter(|&&s| {
                !self.classes.iter().any(|c| c.class == s)
                    && !self.predicates.iter().any(|p| p.predicate == s)
            })
            .collect();

        // ---- Assemble: header, glossary, body, unmatched note.
        let mut w = BudgetWriter::new(budget_chars);
        w.line(&format!(
            "# Schema for {matched}/{} seeds",
            seeds.len()
        ));
        if !prefixes.assigned.is_empty() && !w.full() {
            w.line("## Prefixes");
            for (ns, pfx) in &prefixes.assigned {
                let title = self
                    .vocabularies
                    .namespaces
                    .iter()
                    .find(|v| v.namespace == *ns)
                    .and_then(|v| v.title.as_deref())
                    .map(|t| format!(" — {t}"))
                    .unwrap_or_default();
                if !w.line(&format!("{pfx}: {ns}{title}")) {
                    break;
                }
            }
        }
        for l in &body {
            if !w.line(l) {
                break;
            }
        }
        if !unmatched.is_empty() {
            let note = format!("(no schema for {} seed(s))", unmatched.len());
            w.line(&note);
        }
        w.finish()
    }
}

/// Whether a dictionary id names an IRI (inline ids are integers, never IRIs).
fn is_iri(graph: &Graph, id: Id) -> bool {
    !dict::is_inline(id) && matches!(graph.dict.term_parts(id), TermParts::Iri { .. })
}

/// [OPUS-4.8] sq-3n4: renders an object id to the sample-string form used by every
/// histogram in this crate — literals quoted (`"v"`, `"v"@en`), IRIs bare, blanks
/// `_:b`, triple terms via the dict's Display — truncated to `max_chars`. Factored out
/// of the per-predicate POS scan so the per-class sample labels render objects
/// byte-for-byte identically (same selection key, same truncation).
fn render_object_sample(graph: &Graph, o: Id, max_chars: usize) -> String {
    if dict::is_inline(o) {
        return format!("\"{}\"", o - INLINE_BASE);
    }
    match graph.dict.term_parts(o) {
        TermParts::Iri { prefix, suffix } => {
            truncate_chars(&format!("{prefix}{suffix}"), max_chars)
        }
        TermParts::Lit {
            value,
            lang,
            datatype: _,
        } => {
            let rendered = match lang {
                Some(l) => format!("\"{value}\"@{l}"),
                None => format!("\"{value}\""),
            };
            truncate_chars(&rendered, max_chars)
        }
        TermParts::Blank(b) => truncate_chars(&format!("_:{b}"), max_chars),
        TermParts::Triple(_) => truncate_chars(&graph.dict.term(o).to_string(), max_chars),
    }
}

/// Truncates to `max` characters on a char boundary, appending `…` when cut.
fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let cut: String = s.chars().take(max.saturating_sub(1)).collect();
    format!("{cut}…")
}

/// Lazy prefix assignment for the text summary: namespaces (IRIs split at the last
/// `#`/`/`, the dictionary's own rule) get their well-known prefix when recognised,
/// else `ns1`, `ns2`, … in order of first use — so the glossary lists exactly the
/// namespaces the summary's body references.
struct PrefixAssigner {
    /// `(namespace, prefix)` in first-use order (small — linear search is fine).
    assigned: Vec<(String, String)>,
    generated: usize,
}

impl PrefixAssigner {
    fn new() -> Self {
        PrefixAssigner {
            assigned: Vec::new(),
            generated: 0,
        }
    }

    /// Compacts an IRI to `prefix:local`; quoted samples and unsplittable strings
    /// pass through unchanged.
    fn compact(&mut self, s: &str) -> String {
        if s.starts_with('"') {
            return s.to_string();
        }
        let cut = match s.rfind(['#', '/']) {
            Some(i) => i + 1,
            None => return s.to_string(),
        };
        let (ns, local) = s.split_at(cut);
        if local.is_empty() {
            return s.to_string();
        }
        if let Some((_, pfx)) = self.assigned.iter().find(|(n, _)| n == ns) {
            return format!("{pfx}:{local}");
        }
        let pfx = match WELL_KNOWN.iter().find(|(_, n, _)| *n == ns) {
            Some((p, _, _)) => p.to_string(),
            None => {
                self.generated += 1;
                format!("ns{}", self.generated)
            }
        };
        self.assigned.push((ns.to_string(), pfx.clone()));
        format!("{pfx}:{local}")
    }
}

/// Appends lines while they fit a character budget; `finish` adds a `…` marker when
/// anything was elided. The final string never exceeds the budget.
struct BudgetWriter {
    out: String,
    used: usize, // chars
    limit: usize,
    truncated: bool,
}

impl BudgetWriter {
    fn new(limit: usize) -> Self {
        BudgetWriter {
            out: String::new(),
            used: 0,
            limit,
            truncated: false,
        }
    }

    /// Appends `s` + newline if it fits (reserving 2 chars for the elision marker);
    /// returns whether it was appended.
    fn line(&mut self, s: &str) -> bool {
        let cost = s.chars().count() + 1;
        if self.used + cost + 2 > self.limit {
            self.truncated = true;
            return false;
        }
        self.out.push_str(s);
        self.out.push('\n');
        self.used += cost;
        true
    }

    fn full(&self) -> bool {
        self.truncated
    }

    fn finish(mut self) -> String {
        if self.truncated && self.used + 2 <= self.limit {
            self.out.push_str("…\n");
        }
        self.out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph(ttl: &str) -> Graph {
        let prefix = "@prefix : <http://ex.org/> . \
                      @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> . \
                      @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> . \
                      @prefix foaf: <http://xmlns.com/foaf/0.1/> . \
                      @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n";
        Graph::load_str(&format!("{prefix}{ttl}"), "turtle").unwrap()
    }

    fn ex(s: &str) -> String {
        format!("http://ex.org/{s}")
    }

    #[test]
    fn characteristic_sets_exact() {
        // a: {p, q} (p twice); b: {p, q}; c: {p}.  Two distinct sets.
        let g = graph(":a :p :x , :y ; :q :z . :b :p :x ; :q :z . :c :p :x .");
        let ix = Introspection::build(&g);
        assert_eq!(ix.triples, 6);
        assert_eq!(ix.subjects, 3);
        let cs = &ix.characteristic_sets;
        assert_eq!(cs.distinct, 2);
        assert_eq!(cs.elided_sets, 0);
        // Sorted by subject count: {p,q} ×2 first, {p} ×1 second.
        assert_eq!(cs.sets[0].subjects, 2);
        assert_eq!(cs.sets[0].predicates, vec![ex("p"), ex("q")]);
        // a emits p twice, b once -> 3 p-triples; q once each -> 2.
        assert_eq!(cs.sets[0].predicate_triples, vec![3, 2]);
        assert_eq!(cs.sets[1].subjects, 1);
        assert_eq!(cs.sets[1].predicates, vec![ex("p")]);
        assert_eq!(cs.sets[1].predicate_triples, vec![1]);
        // Counts cover every subject exactly once.
        let total: u64 = cs.sets.iter().map(|s| s.subjects).sum();
        assert_eq!(total + cs.elided_subjects, ix.subjects);
    }

    #[test]
    fn characteristic_set_cap_aggregates_tail() {
        // Four subjects with four distinct predicate sets; cap at 2.
        let g = graph(":a :p1 :x . :b :p2 :x . :c :p3 :x . :d :p1 :x ; :p2 :x .");
        let opts = BuildOptions {
            max_char_sets: 2,
            ..BuildOptions::default()
        };
        let ix = Introspection::build_with(&g, &opts);
        let cs = &ix.characteristic_sets;
        assert_eq!(cs.distinct, 4);
        assert_eq!(cs.sets.len(), 2);
        assert_eq!(cs.elided_sets, 2);
        assert_eq!(cs.elided_subjects, 2);
    }

    #[test]
    fn class_profiles_instances_and_coverage() {
        let g = graph(
            ":alice rdf:type foaf:Person ; foaf:name \"Alice\" ; foaf:age 30 .
             :bob   rdf:type foaf:Person ; foaf:name \"Bob\" .
             :acme  rdf:type :Company ; :employs :alice .",
        );
        let ix = Introspection::build(&g);
        assert_eq!(ix.classes.len(), 2);
        let person = &ix.classes[0]; // 2 instances > 1
        assert_eq!(person.class, "http://xmlns.com/foaf/0.1/Person");
        assert_eq!(person.instances, 2);
        let name = person
            .predicates
            .iter()
            .find(|p| p.predicate.ends_with("name"))
            .unwrap();
        assert_eq!(name.subjects, 2);
        assert_eq!(name.triples, 2);
        assert_eq!(name.coverage, 1.0);
        let age = person
            .predicates
            .iter()
            .find(|p| p.predicate.ends_with("age"))
            .unwrap();
        assert_eq!(age.subjects, 1);
        assert_eq!(age.coverage, 0.5);
        let company = &ix.classes[1];
        assert_eq!(company.class, ex("Company"));
        assert_eq!(company.instances, 1);
    }

    #[test]
    fn predicate_profiles_kinds_datatypes_and_samples() {
        let g = graph(
            ":a :p \"v\" . :a :p \"w\"@en . :a :p 7 . :a :p :iriObj . :b :p :iriObj .
             :a :q \"1999-01-01\"^^xsd:date .",
        );
        let ix = Introspection::build(&g);
        let p = ix
            .predicates
            .iter()
            .find(|p| p.predicate == ex("p"))
            .unwrap();
        assert_eq!(p.triples, 5);
        assert_eq!(p.distinct_subjects, 2);
        assert_eq!(p.distinct_objects, 4); // "v", "w"@en, 7, :iriObj
        assert_eq!(p.objects.literal, 3);
        assert_eq!(p.objects.iri, 2);
        assert_eq!(p.literal_fraction, 3.0 / 5.0);
        // Datatypes: xsd:string, rdf:langString, xsd:integer (inline) — 1 each.
        assert_eq!(p.datatypes.len(), 3);
        assert!(p.datatypes.iter().all(|d| d.count == 1));
        assert!(p.datatypes.iter().any(|d| d.iri.ends_with("langString")));
        assert!(p.datatypes.iter().any(|d| d.iri.ends_with("#string")));
        assert!(p.datatypes.iter().any(|d| d.iri.ends_with("#integer")));
        assert!(!p.samples.is_empty() && p.samples.len() <= 3);
        let q = ix
            .predicates
            .iter()
            .find(|p| p.predicate == ex("q"))
            .unwrap();
        assert_eq!(q.datatypes[0].iri, "http://www.w3.org/2001/XMLSchema#date");
        assert_eq!(q.samples, vec!["\"1999-01-01\"".to_string()]);
    }

    #[test]
    fn domain_range_inference_and_declared() {
        let g = graph(
            ":alice rdf:type foaf:Person . :bob rdf:type foaf:Person . :acme rdf:type :Company .
             :alice :worksAt :acme . :bob :worksAt :acme .
             :worksAt rdfs:domain foaf:Person ; rdfs:range :Org .",
        );
        let ix = Introspection::build(&g);
        let w = ix
            .predicates
            .iter()
            .find(|p| p.predicate == ex("worksAt"))
            .unwrap();
        // Observed domain: 2 distinct Person subjects.
        assert_eq!(
            w.inferred_domains[0],
            Counted {
                iri: "http://xmlns.com/foaf/0.1/Person".into(),
                count: 2
            }
        );
        // Observed range: 1 distinct Company object — usage disagrees with the
        // declared :Org, and both are reported.
        assert_eq!(
            w.inferred_ranges[0],
            Counted {
                iri: ex("Company"),
                count: 1
            }
        );
        assert_eq!(
            w.declared_domains,
            vec!["http://xmlns.com/foaf/0.1/Person".to_string()]
        );
        assert_eq!(w.declared_ranges, vec![ex("Org")]);
        // A predicate on untyped subjects has no inferred domain.
        let g2 = graph(":x :p :y .");
        let ix2 = Introspection::build(&g2);
        let p = &ix2.predicates[0];
        assert!(p.inferred_domains.is_empty() && p.inferred_ranges.is_empty());
        assert!(p.declared_domains.is_empty() && p.declared_ranges.is_empty());
    }

    #[test]
    fn vocabulary_detection_counts_and_well_known() {
        let g = graph(":alice rdf:type foaf:Person ; foaf:name \"Alice\" ; foaf:knows :bob .");
        let ix = Introspection::build(&g);
        let foaf = ix
            .vocabularies
            .namespaces
            .iter()
            .find(|v| v.namespace == "http://xmlns.com/foaf/0.1/")
            .unwrap();
        // Distinct foaf terms: Person, name, knows.
        assert_eq!(foaf.terms, 3);
        assert_eq!(foaf.prefix.as_deref(), Some("foaf"));
        assert_eq!(foaf.title.as_deref(), Some("FOAF (people & agents)"));
        let exns = ix
            .vocabularies
            .namespaces
            .iter()
            .find(|v| v.namespace == "http://ex.org/")
            .unwrap();
        assert_eq!(exns.terms, 2); // :alice, :bob
        assert!(exns.prefix.is_none() && exns.title.is_none());
        let rdf = ix
            .vocabularies
            .namespaces
            .iter()
            .find(|v| v.prefix.as_deref() == Some("rdf"))
            .unwrap();
        assert_eq!(rdf.terms, 1); // rdf:type
    }

    #[test]
    fn to_json_is_valid_and_complete() {
        let g = graph(":alice rdf:type foaf:Person ; foaf:name \"Alice\" .");
        let ix = Introspection::build(&g);
        let v: serde_json::Value = serde_json::from_str(&ix.to_json()).unwrap();
        assert_eq!(v["triples"], 2);
        assert_eq!(v["subjects"], 1);
        assert_eq!(v["classes"][0]["class"], "http://xmlns.com/foaf/0.1/Person");
        assert_eq!(v["classes"][0]["instances"], 1);
        assert_eq!(v["characteristic_sets"]["distinct"], 1);
        assert_eq!(v["characteristic_sets"]["sets"][0]["subjects"], 1);
        assert!(v["predicates"].as_array().unwrap().len() == 2);
        assert!(v["vocabularies"]["namespaces"]
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x["prefix"] == "foaf"));
    }

    #[test]
    fn text_summary_mentions_schema_and_respects_budget() {
        let g = graph(
            ":alice rdf:type foaf:Person ; foaf:name \"Alice\" ; :worksAt :acme .
             :bob rdf:type foaf:Person ; foaf:name \"Bob\" .
             :acme rdf:type :Company .",
        );
        let ix = Introspection::build(&g);
        let s = ix.to_text_summary(4000);
        assert!(s.chars().count() <= 4000);
        assert!(
            s.contains("foaf:Person"),
            "summary must name the dominant class:\n{s}"
        );
        assert!(
            s.contains("Company"),
            "summary must name the minor class:\n{s}"
        );
        assert!(
            s.contains("foaf:name"),
            "summary must name predicate usage:\n{s}"
        );
        assert!(s.contains("2 instances"), "summary must carry counts:\n{s}");
        assert!(
            s.contains("foaf: http://xmlns.com/foaf/0.1/"),
            "prefix glossary:\n{s}"
        );
        // Tight budgets truncate at line granularity, mark elision, and never overflow.
        for budget in [40, 120, 300, 800] {
            let t = ix.to_text_summary(budget);
            assert!(
                t.chars().count() <= budget,
                "budget {budget} overflowed: {} chars",
                t.chars().count()
            );
        }
        let tiny = ix.to_text_summary(150);
        assert!(
            tiny.ends_with("…\n"),
            "truncated summary must carry the elision marker:\n{tiny}"
        );
        // The header (most important line) survives any sane budget.
        assert!(ix.to_text_summary(120).starts_with("# Schema summary"));
    }

    #[test]
    fn empty_and_typeless_graphs() {
        let empty = Graph::load_str("", "turtle").unwrap();
        let ix = Introspection::build(&empty);
        assert_eq!(ix.triples, 0);
        assert_eq!(ix.subjects, 0);
        assert!(ix.classes.is_empty() && ix.predicates.is_empty());
        assert_eq!(ix.characteristic_sets.distinct, 0);
        assert!(serde_json::from_str::<serde_json::Value>(&ix.to_json()).is_ok());
        assert!(ix.to_text_summary(500).starts_with("# Schema summary"));

        // No rdf:type at all: classes empty, characteristic sets still built.
        let untyped = graph(":a :p :x ; :q :y . :b :p :z .");
        let ix = Introspection::build(&untyped);
        assert!(ix.classes.is_empty());
        assert_eq!(ix.characteristic_sets.distinct, 2);
        assert!(ix
            .characteristic_sets
            .sets
            .iter()
            .all(|s| s.classes.is_empty()));
    }

    /// The dict-id accessor (`characteristic_set_ids`) must agree with the
    /// string-resolved table `Introspection::build` produces — same sets, same
    /// subject counts, same per-predicate triple totals — while staying in id space.
    #[test]
    fn characteristic_set_ids_matches_resolved_table() {
        let g = graph(
            ":a a :T ; :p :x ; :p :y ; :q 1 .
             :b a :T ; :p :z ; :q 2 .
             :c :p :w .
             :d :q 3 ; :r :u .",
        );
        let ids = characteristic_set_ids(&g);
        // Exact: 4 subjects, 4 distinct sets ({type,p,q} x2 has 2 subjects? no:
        // :a {type,p(2),q}, :b {type,p,q} -> same SET, multiplicity differs).
        let ix = Introspection::build(&g);
        assert_eq!(ids.len() as u64, ix.characteristic_sets.distinct);
        assert_eq!(ids.iter().map(|s| s.subjects).sum::<u64>(), ix.subjects);
        // Resolve each id set to IRIs and compare against the built table.
        let resolved: Vec<(Vec<String>, u64, Vec<u64>)> = ids
            .iter()
            .map(|s| {
                let mut preds: Vec<(String, u64)> = s
                    .predicates
                    .iter()
                    .zip(s.predicate_triples.iter())
                    .map(|(&p, &t)| (g.dict.term(p).to_string().trim_matches(['<', '>']).to_string(), t))
                    .collect();
                preds.sort();
                (
                    preds.iter().map(|(p, _)| p.clone()).collect(),
                    s.subjects,
                    preds.iter().map(|&(_, t)| t).collect(),
                )
            })
            .collect();
        for set in &ix.characteristic_sets.sets {
            let found = resolved
                .iter()
                .find(|(preds, ..)| *preds == set.predicates)
                .unwrap_or_else(|| panic!("set {:?} missing from id table", set.predicates));
            assert_eq!(found.1, set.subjects, "subject count for {:?}", set.predicates);
            assert_eq!(found.2, set.predicate_triples, "triples for {:?}", set.predicates);
        }
        // Id-space invariants: ascending predicate ids, deterministic order.
        assert!(ids.iter().all(|s| s.predicates.windows(2).all(|w| w[0] < w[1])));
        assert!(ids.windows(2).all(|w| w[0].subjects >= w[1].subjects));
        // The two-subject set {rdf:type, :p, :q} leads.
        assert_eq!(ids[0].subjects, 2);
        assert_eq!(ids[0].predicate_triples.iter().sum::<u64>(), 2 + 3 + 2, "type x2, p x3, q x2");
    }

    /// Cross-class join hints: `(C, p, D)` edge triple counts, exact on a fixture, and
    /// reflected in the JSON surface.
    #[test]
    fn join_hints_cross_class_edges() {
        // 2 Persons each work at the one Company; one Person knows the other Person.
        let g = graph(
            ":alice rdf:type foaf:Person ; :worksAt :acme ; foaf:knows :bob .
             :bob   rdf:type foaf:Person ; :worksAt :acme .
             :acme  rdf:type :Company .",
        );
        let ix = Introspection::build(&g);
        let person = "http://xmlns.com/foaf/0.1/Person";
        let knows = "http://xmlns.com/foaf/0.1/knows";
        let find = |c: &str, p: &str, d: &str| {
            ix.join_hints
                .hints
                .iter()
                .find(|h| h.subject_class == c && h.predicate == p && h.object_class == d)
        };
        // Person --worksAt--> Company: 2 triples (alice, bob).
        let wa = find(person, &ex("worksAt"), &ex("Company")).expect("Person worksAt Company");
        assert_eq!(wa.triples, 2);
        // Person --knows--> Person: 1 triple (alice knows bob; both typed Person).
        let kn = find(person, knows, person).expect("Person knows Person");
        assert_eq!(kn.triples, 1);
        // Exactly these two distinct edges (untyped objects/subjects contribute none).
        assert_eq!(ix.join_hints.distinct, 2);
        assert_eq!(ix.join_hints.elided_hints, 0);
        // Sorted by descending triple count: worksAt (2) before knows (1).
        assert_eq!(ix.join_hints.hints[0].triples, 2);
        assert_eq!(ix.join_hints.hints[1].triples, 1);
        // JSON surface carries the edges.
        let v: serde_json::Value = serde_json::from_str(&ix.to_json()).unwrap();
        assert_eq!(v["join_hints"]["distinct"], 2);
        assert_eq!(v["join_hints"]["hints"][0]["triples"], 2);
        assert_eq!(v["entities"], 3); // alice, bob, acme are all typed.
    }

    #[test]
    fn join_hints_cap_aggregates_tail() {
        // Three distinct (C,p,D) edges; cap at 1.
        let g = graph(
            ":a rdf:type :A ; :p1 :x ; :p2 :y ; :p3 :z .
             :x rdf:type :X . :y rdf:type :Y . :z rdf:type :Z .",
        );
        let opts = BuildOptions {
            max_join_hints: 1,
            ..BuildOptions::default()
        };
        let ix = Introspection::build_with(&g, &opts);
        assert_eq!(ix.join_hints.distinct, 3);
        assert_eq!(ix.join_hints.hints.len(), 1);
        assert_eq!(ix.join_hints.elided_hints, 2);
        assert_eq!(ix.join_hints.elided_triples, 2);
    }

    /// VoID export: exact counts on the top-level dataset and partitions; the output
    /// parses back as N-Triples.
    #[test]
    fn void_export_counts_and_parses() {
        let g = graph(
            ":alice rdf:type foaf:Person ; foaf:name \"Alice\" ; :worksAt :acme .
             :bob   rdf:type foaf:Person ; foaf:name \"Bob\" .
             :acme  rdf:type :Company .",
        );
        let ix = Introspection::build(&g);
        let nt = ix.to_void("http://ex.org/dataset");

        // Re-parse: the VoID document is valid N-Triples (hence valid Turtle).
        let re: std::collections::HashMap<(String, String), String> =
            oxttl::NTriplesParser::new()
                .for_slice(nt.as_bytes())
                .map(|t| {
                    let t = t.expect("valid N-Triples");
                    (
                        (t.subject.to_string(), t.predicate.to_string()),
                        t.object.to_string(),
                    )
                })
                .collect();

        let ds = "<http://ex.org/dataset>".to_string();
        let v = |l: &str| format!("<{VOID_NS}{l}>");
        let lit = |n: u64| {
            oxrdf::Literal::new_typed_literal(n.to_string(), oxrdf::vocab::xsd::INTEGER).to_string()
        };
        // Top-level dataset counts (exact). 6 triples: alice(type,name,worksAt),
        // bob(type,name), acme(type).
        assert_eq!(re.get(&(ds.clone(), v("triples"))), Some(&lit(6)));
        assert_eq!(re.get(&(ds.clone(), v("entities"))), Some(&lit(3))); // alice, bob, acme
        assert_eq!(re.get(&(ds.clone(), v("distinctSubjects"))), Some(&lit(3)));
        assert_eq!(re.get(&(ds.clone(), v("classes"))), Some(&lit(2))); // Person, Company
        // 4 predicates: rdf:type, foaf:name, :worksAt (+ the partition properties are
        // distinct predicates of the *original* graph, not the VoID doc).
        assert_eq!(re.get(&(ds.clone(), v("properties"))), Some(&lit(3)));
        // A class partition for Person carries void:class + void:entities=2.
        let nt2 = ix.to_void("http://ex.org/dataset");
        assert!(
            nt2.contains(&v("classPartition")),
            "must emit class partitions"
        );
        assert!(
            nt2.contains(&v("propertyPartition")),
            "must emit property partitions"
        );
        // The Person class partition's entity count (2) appears.
        assert!(nt2.contains("http://xmlns.com/foaf/0.1/Person"));
    }

    /// [OPUS-4.8] sq-mr32 (federation A3/Z2): the VoID+CS export is a strict superset of
    /// `to_void` (every standard VoID triple unchanged) plus the characteristic-set
    /// statistics under the `scs:` extension, and the whole document re-parses as valid
    /// RDF.
    #[test]
    fn void_with_cs_superset_and_carries_char_sets() {
        // alice/bob share the set {type, name}; carol has {type, name, worksAt}.
        let g = graph(
            ":alice rdf:type foaf:Person ; foaf:name \"Alice\" .
             :bob   rdf:type foaf:Person ; foaf:name \"Bob\" .
             :carol rdf:type foaf:Person ; foaf:name \"Carol\" ; :worksAt :acme .",
        );
        let ix = Introspection::build(&g);
        let void = ix.to_void("http://ex.org/dataset");
        let withcs = ix.to_void_with_cs("http://ex.org/dataset");

        // Strict superset: every base VoID line is present verbatim in the CS variant.
        for line in void.lines() {
            assert!(
                withcs.contains(line),
                "VoID+CS must contain every base VoID line; missing: {line}"
            );
        }
        assert!(
            withcs.len() > void.len(),
            "VoID+CS must add the characteristic-set triples"
        );

        // Re-parse the whole document — valid N-Triples (hence valid Turtle).
        let triples: Vec<oxrdf::Triple> = oxttl::NTriplesParser::new()
            .for_slice(withcs.as_bytes())
            .map(|t| t.expect("VoID+CS is valid N-Triples"))
            .collect();
        let cs = |l: &str| format!("{CS_NS}{l}");

        // Dataset carries the EXACT distinct-set count (here 2: {type,name}, {type,name,worksAt}).
        assert_eq!(ix.characteristic_sets.distinct, 2);
        let distinct_lit = oxrdf::Literal::new_typed_literal(
            ix.characteristic_sets.distinct.to_string(),
            oxrdf::vocab::xsd::INTEGER,
        )
        .to_string();
        assert!(
            withcs.contains(&format!(
                "<http://ex.org/dataset> <{}> {distinct_lit} .",
                cs("distinctCharacteristicSets")
            )),
            "must carry exact distinct-set count: {withcs}"
        );

        // One typed CharacteristicSet node per retained set, each with scs:subjects.
        let typed_cs = triples
            .iter()
            .filter(|t| {
                t.predicate.as_str() == RDF_TYPE
                    && t.object.to_string() == format!("<{}>", cs("CharacteristicSet"))
            })
            .count();
        assert_eq!(typed_cs, ix.characteristic_sets.sets.len());
        assert!(typed_cs >= 2);

        // Per-predicate stat nodes reuse void:property + void:triples and add
        // scs:avgMultiplicity, and name a real predicate from the graph.
        assert!(triples
            .iter()
            .any(|t| t.predicate.as_str() == cs("avgMultiplicity")));
        assert!(withcs.contains("http://xmlns.com/foaf/0.1/name"));
        // avg multiplicity for a single-valued predicate ({type,name} ×2 subjects, name
        // once each) renders as the fixed-precision decimal 1.0000.
        assert!(
            withcs.contains("1.0000"),
            "avg multiplicity should render as a fixed-precision decimal: {withcs}"
        );
    }

    /// Retrieval-mode (seed-scoped) summary: only the seeds' slice is rendered, under
    /// budget, and unmatched seeds are noted.
    #[test]
    fn schema_summary_for_seeds() {
        let g = graph(
            ":alice rdf:type foaf:Person ; foaf:name \"Alice\" ; :worksAt :acme .
             :bob   rdf:type foaf:Person ; foaf:name \"Bob\" .
             :acme  rdf:type :Company .",
        );
        let ix = Introspection::build(&g);
        let person = "http://xmlns.com/foaf/0.1/Person";
        let s = ix.schema_summary_for(&[person, "http://ex.org/nonexistent"], 4000);
        assert!(s.chars().count() <= 4000);
        // The seed class is profiled.
        assert!(s.contains("foaf:Person"), "seed class profile:\n{s}");
        assert!(s.contains("2 instances"), "instance count:\n{s}");
        assert!(s.contains("foaf:name"), "per-class predicate:\n{s}");
        // The join edge from Person is surfaced.
        assert!(s.contains("join:"), "join edge:\n{s}");
        // 1 matched seed, 1 unmatched note.
        assert!(s.contains("1/2 seeds"), "header:\n{s}");
        assert!(s.contains("no schema for 1"), "unmatched note:\n{s}");
        // A predicate seed pulls the predicate's global profile.
        let sp = ix.schema_summary_for(&[&ex("worksAt")], 4000);
        assert!(sp.contains("worksAt"), "predicate seed profile:\n{sp}");
        assert!(sp.contains("range:"), "predicate range hint:\n{sp}");
        // Budget is respected even when tight.
        for budget in [30, 120, 500] {
            let t = ix.schema_summary_for(&[person], budget);
            assert!(t.chars().count() <= budget, "budget {budget} overflow");
        }
    }

    /// [OPUS-4.8] sq-3n4: per-class sample labels isolate a minority class. The shared
    /// predicate `:p` is used by a large `:Big` class (with the lexicographically
    /// smallest object values) and a one-instance `:Minor` class (a large value). The
    /// global per-predicate samples are the small `:Big` values — so the minority class
    /// must NOT borrow them; its own `samples` field carries its own value, and the text
    /// summary renders it on the minority class's line.
    #[test]
    fn per_class_samples_isolate_minority_class() {
        let g = graph(
            ":b1 a :Big ; :p \"aaa\" .
             :b2 a :Big ; :p \"aab\" .
             :b3 a :Big ; :p \"aac\" .
             :m1 a :Minor ; :p \"zzz\" .",
        );
        let ix = Introspection::build(&g);

        // The GLOBAL predicate samples are the lex-smallest across every subject — the
        // :Big values; "zzz" is NOT among them.
        let gp = ix
            .predicates
            .iter()
            .find(|p| p.predicate == ex("p"))
            .unwrap();
        assert_eq!(gp.samples, vec!["\"aaa\"", "\"aab\"", "\"aac\""]);
        assert!(!gp.samples.iter().any(|s| s == "\"zzz\""));

        let class = |name: &str| ix.classes.iter().find(|c| c.class == ex(name)).unwrap();
        let class_pred = |name: &str| {
            class(name)
                .predicates
                .iter()
                .find(|cp| cp.predicate == ex("p"))
                .unwrap()
        };

        // The minority class's sample is ITS OWN value, not the global :Big minimum.
        let minor = class_pred("Minor");
        assert_eq!(minor.samples, vec!["\"zzz\""]);
        // The big class's per-class samples are its own values (capped at 3).
        let big = class_pred("Big");
        assert_eq!(big.samples, vec!["\"aaa\"", "\"aab\"", "\"aac\""]);

        // The text summary renders the minority class's own example on its line — the
        // exact "looks odd on minority classes" defect the bead names.
        let s = ix.to_text_summary(4000);
        let minor_line = s
            .lines()
            .find(|l| l.contains(":p ") && l.contains("1/1"))
            .unwrap_or_else(|| panic!("minority :p line missing:\n{s}"));
        assert!(
            minor_line.contains("zzz"),
            "minority class line must show its own sample, not the global min:\n{minor_line}"
        );
        assert!(
            !minor_line.contains("aaa"),
            "minority class line must not borrow the dominant class's sample:\n{minor_line}"
        );

        // The per-class samples survive a JSON round-trip on the public surface.
        let v: serde_json::Value = serde_json::from_str(&ix.to_json()).unwrap();
        let classes = v["classes"].as_array().unwrap();
        let minor_json = classes.iter().find(|c| c["class"] == ex("Minor")).unwrap();
        let p_json = minor_json["predicates"]
            .as_array()
            .unwrap()
            .iter()
            .find(|cp| cp["predicate"] == ex("p"))
            .unwrap();
        assert_eq!(p_json["samples"][0], "\"zzz\"");
    }

    /// [OPUS-4.8] sq-3n4: per-class samples are bounded by `samples_per_predicate` and
    /// hold DISTINCT values (the per-class path feeds every triple, not POS-collapsed
    /// distinct objects, so repeats must be de-duplicated).
    #[test]
    fn per_class_samples_distinct_and_bounded() {
        // :Thing instances repeat the value "dup" and add three distinct others.
        let g = graph(
            ":a a :Thing ; :p \"dup\" , \"dup\" , \"m\" .
             :b a :Thing ; :p \"dup\" , \"n\" .
             :c a :Thing ; :p \"o\" .",
        );
        let opts = BuildOptions {
            samples_per_predicate: 2,
            ..BuildOptions::default()
        };
        let ix = Introspection::build_with(&g, &opts);
        let thing = ix.classes.iter().find(|c| c.class == ex("Thing")).unwrap();
        let cp = thing
            .predicates
            .iter()
            .find(|cp| cp.predicate == ex("p"))
            .unwrap();
        // Cap honoured, lex-smallest distinct: "dup", "m" (NOT a duplicated "dup").
        assert_eq!(cp.samples.len(), 2);
        assert_eq!(cp.samples, vec!["\"dup\"", "\"m\""]);
        let mut sorted = cp.samples.clone();
        sorted.dedup();
        assert_eq!(sorted.len(), cp.samples.len(), "samples must be distinct");
    }

    /// [OPUS-4.8] sq-3n4: the persisted `*.introspect` sidecar round-trips byte-exactly
    /// through `save`/`load`, and a loaded sidecar produces the same O(output) summaries
    /// as the freshly-built one WITHOUT re-touching the graph.
    #[test]
    fn sidecar_save_load_roundtrip() {
        let g = graph(
            ":alice rdf:type foaf:Person ; foaf:name \"Alice\" ; foaf:age 30 ; :worksAt :acme .
             :bob   rdf:type foaf:Person ; foaf:name \"Bob\" .
             :acme  rdf:type :Company ; :name \"Acme\" .",
        );
        let ix = Introspection::build(&g);

        // Unique temp path (no extra dev-deps): pid + a nanosecond stamp.
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "sparq-introspect-sidecar-{}-{stamp}.introspect",
            std::process::id()
        ));

        ix.save(&path).expect("save sidecar");
        let loaded = Introspection::load(&path).expect("load sidecar");
        let _ = std::fs::remove_file(&path);

        // Byte-exact JSON round-trip (struct equality via its canonical serialisation).
        assert_eq!(ix.to_json(), loaded.to_json());
        // Headline numbers and a deep field survive.
        assert_eq!(loaded.triples, ix.triples);
        assert_eq!(loaded.entities, ix.entities);
        assert_eq!(loaded.classes.len(), ix.classes.len());
        assert_eq!(
            loaded.characteristic_sets.distinct,
            ix.characteristic_sets.distinct
        );
        // Per-class samples (sq-3n4's other half) survive into the sidecar too.
        let loaded_person = loaded
            .classes
            .iter()
            .find(|c| c.class == "http://xmlns.com/foaf/0.1/Person")
            .unwrap();
        assert!(loaded_person
            .predicates
            .iter()
            .any(|cp| !cp.samples.is_empty()));

        // The whole point: every O(output) export runs off the loaded struct, no rescan.
        assert_eq!(
            loaded.to_text_summary(4000),
            ix.to_text_summary(4000),
            "loaded sidecar must reproduce the summary exactly"
        );
        assert_eq!(
            loaded.to_void("http://ex.org/dataset"),
            ix.to_void("http://ex.org/dataset")
        );
    }

    /// [OPUS-4.8] sq-3n4: `from_json` is the in-memory inverse of `to_json`, and a
    /// missing / malformed sidecar surfaces as a clean `io::Error`.
    #[test]
    fn sidecar_from_json_and_error_paths() {
        let g = graph(":a rdf:type :T ; :p \"v\" .");
        let ix = Introspection::build(&g);
        let back = Introspection::from_json(&ix.to_json()).expect("parse own JSON");
        assert_eq!(back.to_json(), ix.to_json());

        // A nonexistent path is a NotFound io::Error (read failure).
        let err = Introspection::load("/no/such/sparq-introspect-sidecar.introspect").unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);

        // Malformed JSON surfaces as InvalidData (parse failure mapped into io).
        let bad = std::env::temp_dir().join(format!(
            "sparq-introspect-bad-{}.introspect",
            std::process::id()
        ));
        std::fs::write(&bad, b"{ not valid introspect json ]").unwrap();
        let err = Introspection::load(&bad).unwrap_err();
        let _ = std::fs::remove_file(&bad);
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }

    /// [OPUS-4.8] sq-3n4: the conventional sidecar path appends `.introspect` to the
    /// dataset name (keeping the dataset's own extension), so two datasets differing only
    /// by extension get distinct sidecars.
    #[test]
    fn sidecar_path_convention() {
        use std::path::Path;
        assert_eq!(
            sidecar_path_for("data/olympics.nt"),
            Path::new("data/olympics.nt.introspect")
        );
        assert_eq!(
            sidecar_path_for("data/olympics.ttl"),
            Path::new("data/olympics.ttl.introspect")
        );
        // Distinct datasets that share a stem but differ by extension never collide.
        assert_ne!(sidecar_path_for("g.nt"), sidecar_path_for("g.ttl"));
        assert_eq!(SIDECAR_EXTENSION, "introspect");
    }
}
