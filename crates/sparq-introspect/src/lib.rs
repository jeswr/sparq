//! sparq-introspect: **ontology/schema introspection** over a [`sparq_core::Graph`] —
//! the effective schema a knowledge graph *actually uses*, mined from the store's
//! sorted permutation indexes (GenAI phase 2, see `research/genai-design.md` §2 and
//! `research/genai-ontology-introspection.md`).
//!
//! Everything is a **sorted scan, never a join-engine call**:
//!
//! - **Characteristic sets** (Neumann & Moerkotte, ICDE 2011): one SPO scan groups
//!   subjects by their exact predicate set — subjects are contiguous, predicates within
//!   a subject run are sorted, so the per-subject predicate list falls out of run
//!   boundaries. Each distinct set carries its subject count and per-predicate triple
//!   counts (the paper's structure: `count(C)` + per-predicate multiplicity).
//! - **Schema summary**: class extents from the `rdf:type` block (one POS range),
//!   per-class predicate usage with coverage ratios, per-predicate global stats
//!   (triples, distinct subjects/objects, literal-vs-IRI object split, datatype
//!   distribution, sample values), and **observed** domain/range histograms — the
//!   most-common subject/object classes per predicate — alongside any **declared**
//!   `rdfs:domain`/`rdfs:range` triples present.
//! - **Vocabulary detection**: namespaces in use (IRIs split at the last `#`/`/`, the
//!   same split the dictionary itself stores) with distinct-term counts, recognised
//!   against a bundled table of well-known vocabularies.
//!
//! Outputs: the [`Introspection`] struct tree, [`Introspection::to_json`] (the machine
//! surface for LLM grounding), and [`Introspection::to_text_summary`] (a compact,
//! most-important-first digest under a character budget — the prompt-ready "schema
//! card" deck the design doc prescribes).
//!
//! The crate is opt-in and read-only over the public `sparq-core` API; the default
//! engine build does not include it and carries no introspection code.

use oxrdf::{NamedNode, Term};
use rustc_hash::FxHashMap;
use serde::Serialize;
use sparq_core::dict::{self, Id, TermParts, INLINE_BASE};
use sparq_core::Graph;

const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const RDFS_DOMAIN: &str = "http://www.w3.org/2000/01/rdf-schema#domain";
const RDFS_RANGE: &str = "http://www.w3.org/2000/01/rdf-schema#range";
const XSD_INTEGER: &str = "http://www.w3.org/2001/XMLSchema#integer";

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
            max_sample_chars: 60,
        }
    }
}

/// An IRI with a count — the unit of every histogram in this crate.
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct Counted {
    pub iri: String,
    pub count: u64,
}

/// One distinct **characteristic set** (Neumann & Moerkotte): the exact set of
/// predicates emitted by some group of subjects.
#[derive(Clone, Debug, Serialize)]
pub struct CharacteristicSet {
    /// The predicate IRIs, sorted by dictionary id (deterministic).
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
#[derive(Clone, Debug, Serialize)]
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
#[derive(Clone, Debug, Serialize)]
pub struct ClassPredicate {
    pub predicate: String,
    /// Instances of the class with at least one triple via this predicate.
    pub subjects: u64,
    /// Total triples via this predicate whose subject is an instance of the class.
    pub triples: u64,
    /// `subjects / instances` of the class, in `[0, 1]`.
    pub coverage: f64,
}

/// A class (an `rdf:type` object) and how its instances are described.
#[derive(Clone, Debug, Serialize)]
pub struct ClassProfile {
    pub class: String,
    pub instances: u64,
    /// Predicates appearing on instances of this class, by descending subject count.
    pub predicates: Vec<ClassPredicate>,
}

/// Triple counts by object kind for one predicate (the literal-vs-IRI split).
#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct ObjectKinds {
    pub iri: u64,
    pub literal: u64,
    pub blank: u64,
    pub triple_term: u64,
}

/// Global statistics for one predicate.
#[derive(Clone, Debug, Serialize)]
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
    /// Sample object values from the sorted index (deterministic; literals quoted,
    /// IRIs bare). The first sample is the predicate's *minimum* object value.
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
#[derive(Clone, Debug, Serialize)]
pub struct VocabularyUse {
    pub namespace: String,
    pub prefix: Option<String>,
    pub title: Option<String>,
    /// Distinct dictionary terms (IRIs) under this namespace.
    pub terms: u64,
}

/// The detected vocabularies: top namespaces plus exact tail aggregates.
#[derive(Clone, Debug, Serialize)]
pub struct Vocabularies {
    /// Total number of distinct namespaces in the dictionary.
    pub distinct: u64,
    /// The top namespaces by term count (bounded by [`BuildOptions::max_namespaces`]).
    pub namespaces: Vec<VocabularyUse>,
    /// Namespaces beyond the cap, and the distinct terms they cover.
    pub elided_namespaces: u64,
    pub elided_terms: u64,
}

/// The full introspection result. Build with [`Introspection::build`]; export with
/// [`to_json`](Introspection::to_json) / [`to_text_summary`](Introspection::to_text_summary).
#[derive(Clone, Debug, Serialize)]
pub struct Introspection {
    pub triples: u64,
    /// Distinct subjects.
    pub subjects: u64,
    /// Classes by descending instance count.
    pub classes: Vec<ClassProfile>,
    /// Predicates by descending triple count.
    pub predicates: Vec<PredicateProfile>,
    pub characteristic_sets: CharacteristicSets,
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

/// Per-characteristic-set accumulator.
struct CsAcc {
    subjects: u64,
    /// Aligned with the key's predicates: Σ triples.
    triples: Vec<u64>,
    /// class id -> subjects in this set typed with the class.
    classes: FxHashMap<Id, u64>,
}

impl Introspection {
    /// Builds the full introspection with default [`BuildOptions`]. Cost: one full SPO
    /// scan (characteristic sets + per-class usage + observed domains), one pass over
    /// the POS blocks of every predicate (object kinds, datatypes, samples, observed
    /// ranges), one pass over the dictionary (vocabularies) — all sorted scans over
    /// indexes the store already keeps.
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
        let mut class_preds: FxHashMap<Id, FxHashMap<Id, (u64, u64)>> = FxHashMap::default();
        let mut subjects: u64 = 0;

        let scan = graph.store.scan(&[None, None, None]);
        let rows = scan.rows.as_ref();
        // The full scan is served by a subject-leading permutation; map rows to
        // canonical (s, p, o) via the scan's own permutation for safety.
        let (mut ps, mut ms): (Vec<Id>, Vec<u64>) = (Vec::new(), Vec::new());
        let mut i = 0;
        while i < rows.len() {
            let [s, ..] = scan.to_spo(&rows[i]);
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
            subjects += 1;
            let types = subj_types.get(&s);
            for (idx, &p) in ps.iter().enumerate() {
                let pa = preds.entry(p).or_default();
                pa.triples += ms[idx];
                pa.distinct_subjects += 1;
                if let Some(ts) = types {
                    for &c in ts {
                        *pa.domains.entry(c).or_default() += 1;
                        let cp = class_preds.entry(c).or_default().entry(p).or_default();
                        cp.0 += 1;
                        cp.1 += ms[idx];
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
                if dict::is_inline(o) {
                    acc.kinds.literal += n;
                    *acc.datatypes.entry(XSD_INTEGER.to_string()).or_default() += n;
                    if acc.samples.len() < opts.samples_per_predicate {
                        acc.samples.push(format!("\"{}\"", o - INLINE_BASE));
                    }
                } else {
                    match graph.dict.term_parts(o) {
                        TermParts::Iri { prefix, suffix } => {
                            acc.kinds.iri += n;
                            if let Some(ts) = subj_types.get(&o) {
                                for &c in ts {
                                    *acc.ranges.entry(c).or_default() += 1;
                                }
                            }
                            if acc.samples.len() < opts.samples_per_predicate {
                                acc.samples.push(truncate_chars(
                                    &format!("{prefix}{suffix}"),
                                    opts.max_sample_chars,
                                ));
                            }
                        }
                        TermParts::Lit {
                            value,
                            datatype,
                            lang,
                        } => {
                            acc.kinds.literal += n;
                            *acc.datatypes.entry(datatype.to_string()).or_default() += n;
                            if acc.samples.len() < opts.samples_per_predicate {
                                let rendered = match lang {
                                    Some(l) => format!("\"{value}\"@{l}"),
                                    None => format!("\"{value}\""),
                                };
                                acc.samples
                                    .push(truncate_chars(&rendered, opts.max_sample_chars));
                            }
                        }
                        TermParts::Blank(b) => {
                            acc.kinds.blank += n;
                            if acc.samples.len() < opts.samples_per_predicate {
                                acc.samples
                                    .push(truncate_chars(&format!("_:{b}"), opts.max_sample_chars));
                            }
                        }
                        TermParts::Triple(_) => {
                            acc.kinds.triple_term += n;
                            if acc.samples.len() < opts.samples_per_predicate {
                                acc.samples.push(truncate_chars(
                                    &graph.dict.term(o).to_string(),
                                    opts.max_sample_chars,
                                ));
                            }
                        }
                    }
                }
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
                            .map(|(&p, &(subj, tri))| ClassPredicate {
                                predicate: iri_str(p),
                                subjects: subj,
                                triples: tri,
                                coverage: subj as f64 / instances.max(1) as f64,
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
        let mut cs_vec: Vec<(Box<[Id]>, CsAcc)> = cs.into_iter().collect();
        cs_vec.sort_by(|a, b| b.1.subjects.cmp(&a.1.subjects).then(a.0.cmp(&b.0)));
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
            .map(|(key, acc)| CharacteristicSet {
                predicates: key.iter().map(|&p| iri_str(p)).collect(),
                subjects: acc.subjects,
                predicate_triples: acc.triples,
                classes: top_counted(&acc.classes, opts.max_classes_per_histogram),
            })
            .collect();

        Introspection {
            triples: graph.len() as u64,
            subjects,
            classes,
            predicates,
            characteristic_sets: CharacteristicSets {
                distinct: distinct_cs,
                sets,
                elided_sets,
                elided_subjects,
            },
            vocabularies,
        }
    }

    /// Serialises the full introspection to pretty-printed JSON — the machine surface
    /// for LLM grounding (and for any other tool).
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("introspection serialises to JSON")
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
                    let hint = self.predicate_hint(&cp.predicate, &mut prefixes);
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
    /// observed range class, else the dominant literal datatype, plus one sample —
    /// from the predicate's GLOBAL profile (per-class counts, global hints).
    fn predicate_hint(&self, predicate: &str, prefixes: &mut PrefixAssigner) -> String {
        let Some(p) = self.predicates.iter().find(|p| p.predicate == predicate) else {
            return String::new();
        };
        let mut hint = String::new();
        if let Some(r) = p.inferred_ranges.first() {
            hint.push_str(&format!(" → {}", prefixes.compact(&r.iri)));
        } else if let Some(dt) = p.datatypes.first() {
            hint.push_str(&format!(" → {}", prefixes.compact(&dt.iri)));
        }
        if let Some(s) = p.samples.first() {
            hint.push_str(&format!(", e.g. {}", prefixes.compact(s)));
        }
        hint
    }
}

/// Whether a dictionary id names an IRI (inline ids are integers, never IRIs).
fn is_iri(graph: &Graph, id: Id) -> bool {
    !dict::is_inline(id) && matches!(graph.dict.term_parts(id), TermParts::Iri { .. })
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
}
