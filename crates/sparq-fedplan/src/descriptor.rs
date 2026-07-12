//! The source **descriptor** model — what the planner knows about each remote source
//! *before* contacting it, and how to build it from the served statistics.
//!
//! A `sparq-server` already serves, at `/.well-known/void`, a W3C VoID description of
//! its dataset (triple/entity/class/property counts + per-class and per-predicate
//! partitions) **plus** the mined **characteristic sets** under the `scs:` vocab
//! (`http://sparq.dev/ns/cs#`). A [`SourceDescriptor`] is the in-planner image of that
//! document for one source: its capability set (predicates, classes, IRI authorities)
//! and its skew-aware per-predicate / per-characteristic-set statistics.
//!
//! Build one with the typed [`SourceDescriptor::builder`] API, or parse it straight
//! from the served N-Triples with [`SourceDescriptor::from_void_nt`].
//!
//! [OPUS-4.8] sq-a35t.

use rustc_hash::{FxHashMap, FxHashSet};

/// The VoID vocabulary namespace.
pub const VOID_NS: &str = "http://rdfs.org/ns/void#";
/// The served characteristic-set vocabulary namespace (`scs:`).
pub const CS_NS: &str = "http://sparq.dev/ns/cs#";

/// A stable identifier for a federated source (conventionally its endpoint URL). Used
/// as the deterministic tie-break key throughout the planner.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SourceId(pub String);

impl SourceId {
    pub fn new(id: impl Into<String>) -> SourceId {
        SourceId(id.into())
    }
}

/// One `void:propertyPartition`: a predicate this source holds, with its triple count
/// and (when known) the per-subject average multiplicity — the skew signal CostFed-style
/// cardinality uses instead of a uniform guess.
#[derive(Debug, Clone)]
pub struct PredPartition {
    pub predicate: String,
    /// `void:triples` for this predicate (total triples with this predicate).
    pub triples: u64,
    /// `void:distinctSubjects` for this predicate (distinct subjects), 0 if unknown.
    pub distinct_subjects: u64,
    /// Distinct objects for this predicate, 0 if unknown. (VoID's per-predicate
    /// `void:distinctObjects` when served; otherwise left 0 and treated as unknown.)
    pub distinct_objects: u64,
}

impl PredPartition {
    /// Average per-subject multiplicity (`triples / distinctSubjects`); `1.0` when the
    /// distinct-subject count is unknown (the recall-safe non-skewed default).
    pub fn avg_multiplicity(&self) -> f64 {
        if self.distinct_subjects == 0 {
            1.0
        } else {
            self.triples as f64 / self.distinct_subjects as f64
        }
    }
}

/// One `void:classPartition`: a class this source holds, with its instance count.
#[derive(Debug, Clone)]
pub struct ClassPartition {
    pub class: String,
    /// `void:entities` for this class (instances).
    pub entities: u64,
}

/// One served **characteristic set** (`scs:CharacteristicSet`): a distinct per-subject
/// predicate set, its subject count (`scs:subjects`) and per-predicate average
/// multiplicity (`scs:avgMultiplicity`, aligned with `predicates`). This is the
/// correlation signal the marginals lose — the planner uses it for star-join
/// cardinality.
#[derive(Debug, Clone)]
pub struct CharSet {
    /// Predicate IRIs of this set, sorted ascending (so superset tests are merge walks).
    pub predicates: Vec<String>,
    /// `scs:subjects` — subjects whose exact predicate set this is.
    pub subjects: u64,
    /// `scs:avgMultiplicity`, aligned with `predicates` — avg triples per subject for
    /// each predicate in the set.
    pub avg_multiplicity: Vec<f64>,
}

/// A federated source's full descriptor: capability set + skew-aware statistics, as
/// mined from the served VoID + `scs:` document. The capability sets (`authorities`,
/// predicate keys, class keys) drive **recall-safe** HiBISCuS pruning; the counts and
/// multiplicities drive CostFed/CS cardinality.
#[derive(Debug, Clone)]
pub struct SourceDescriptor {
    /// Stable source id (endpoint URL).
    pub id: SourceId,
    /// `void:triples` — total triples this source holds (0 if unknown).
    pub total_triples: u64,
    /// Per-predicate partitions, keyed by predicate IRI.
    predicates: FxHashMap<String, PredPartition>,
    /// Per-class partitions, keyed by class IRI.
    classes: FxHashMap<String, ClassPartition>,
    /// Served characteristic sets (for star-join cardinality).
    char_sets: Vec<CharSet>,
    /// The set of IRI **authorities** this source is known to mint terms under
    /// (scheme-and-host, e.g. `http://dbpedia.org`). Derived from every IRI seen in a
    /// partition (predicates, classes). A pattern's bound-IRI position whose authority
    /// is absent here *cannot* match this source — the HiBISCuS prune. EMPTY means
    /// "unknown authorities" ⇒ never prune on authority (recall-safe).
    authorities: FxHashSet<String>,
    /// Whether the authority set is *complete* — i.e. the descriptor enumerated every
    /// IRI authority the source can produce. Only a complete authority set may prune a
    /// pattern whose bound IRI's authority is absent. A descriptor parsed from VoID
    /// partitions sees only predicate/class authorities, NOT subject/object instance
    /// authorities, so it is **incomplete** and authority-pruning of subject/object
    /// positions is disabled (recall-safe). Predicate-position pruning still applies
    /// because the predicate partition set IS the complete predicate capability set.
    authorities_complete: bool,
}

/// Builder for a [`SourceDescriptor`] (the typed construction path; the parse path
/// [`SourceDescriptor::from_void_nt`] funnels through it too).
pub struct SourceDescriptorBuilder {
    id: SourceId,
    total_triples: u64,
    predicates: FxHashMap<String, PredPartition>,
    classes: FxHashMap<String, ClassPartition>,
    char_sets: Vec<CharSet>,
    authorities_complete: bool,
}

impl SourceDescriptor {
    /// Starts a typed builder for the source named `id`.
    pub fn builder(id: SourceId) -> SourceDescriptorBuilder {
        SourceDescriptorBuilder {
            id,
            total_triples: 0,
            predicates: FxHashMap::default(),
            classes: FxHashMap::default(),
            char_sets: Vec::new(),
            authorities_complete: false,
        }
    }

    /// This source's id.
    pub fn id(&self) -> &SourceId {
        &self.id
    }

    /// The predicate partition for `predicate`, if this source holds it.
    pub fn predicate(&self, predicate: &str) -> Option<&PredPartition> {
        self.predicates.get(predicate)
    }

    /// Whether this source holds *any* triple with `predicate` (its predicate
    /// capability set membership).
    pub fn has_predicate(&self, predicate: &str) -> bool {
        self.predicates.contains_key(predicate)
    }

    /// The class partition for `class`, if this source holds instances of it.
    pub fn class(&self, class: &str) -> Option<&ClassPartition> {
        self.classes.get(class)
    }

    /// Whether this source holds any instance of `class`.
    pub fn has_class(&self, class: &str) -> bool {
        self.classes.contains_key(class)
    }

    /// Whether the descriptor declares ANY `void:classPartition`. A descriptor with no
    /// class section means "class membership unknown" (so a bound-class prune must NOT
    /// fire — recall-safe); a descriptor WITH class partitions enumerates every class
    /// the source holds instances of, so an absent class then proves no instances.
    pub fn declares_any_class(&self) -> bool {
        !self.classes.is_empty()
    }

    /// The served characteristic sets.
    pub fn char_sets(&self) -> &[CharSet] {
        &self.char_sets
    }

    /// Whether the descriptor's IRI-authority capability set is complete (every
    /// authority the source can produce is enumerated). When false, authority pruning
    /// of subject/object positions is disabled (recall-safe).
    pub fn authorities_complete(&self) -> bool {
        self.authorities_complete
    }

    /// Whether `iri`'s authority is in this source's known authority set. Returns
    /// `true` (cannot rule out) when the authority set is incomplete or the IRI has no
    /// extractable authority — the recall-safe default.
    pub fn may_hold_authority(&self, iri: &str) -> bool {
        if !self.authorities_complete {
            return true;
        }
        match authority_of(iri) {
            Some(a) => self.authorities.contains(&a),
            None => true, // unparseable authority ⇒ cannot rule out.
        }
    }

    /// `Σ_{C ⊇ q} subjects(C)`: distinct subjects on this source carrying EVERY
    /// predicate of `q` (the star's subject-variable ndv estimate). `q` is sorted
    /// ascending by the caller.
    pub fn star_subjects(&self, q: &[String]) -> u64 {
        self.char_sets
            .iter()
            .filter(|c| is_subset(q, &c.predicates))
            .map(|c| c.subjects)
            .sum()
    }

    /// Neumann & Moerkotte's star estimate over this source's characteristic sets:
    /// `Σ_{C ⊇ q} subjects(C) · Π_{p∈q} avg_mult(C, p)` — the expected result rows of
    /// the star `?s p ?o_p` for every `p ∈ q`. `q` is sorted ascending. Returns 0.0
    /// when no served set covers `q` (the caller falls back to the marginal estimate).
    pub fn star_cardinality(&self, q: &[String]) -> f64 {
        let mut total = 0.0f64;
        for c in &self.char_sets {
            if !is_subset(q, &c.predicates) {
                continue;
            }
            let mut rows = c.subjects as f64;
            for p in q {
                let i = c
                    .predicates
                    .binary_search(p)
                    .expect("subset member present");
                rows *= c.avg_multiplicity[i];
            }
            total += rows;
        }
        total
    }
}

impl SourceDescriptorBuilder {
    /// Sets `void:triples` (total triples).
    pub fn total_triples(mut self, n: u64) -> Self {
        self.total_triples = n;
        self
    }

    /// Adds a `void:propertyPartition`.
    pub fn predicate(mut self, p: PredPartition) -> Self {
        self.predicates.insert(p.predicate.clone(), p);
        self
    }

    /// Adds a `void:classPartition`.
    pub fn class(mut self, c: ClassPartition) -> Self {
        self.classes.insert(c.class.clone(), c);
        self
    }

    /// Adds a served `scs:CharacteristicSet`. Predicates are normalised to sorted
    /// order (keeping `avg_multiplicity` aligned). Sets with `subjects == 0` or a
    /// length mismatch are dropped (mirrors the engine's `CsTable::new`).
    pub fn char_set(mut self, mut cs: CharSet) -> Self {
        if cs.subjects == 0 || cs.predicates.len() != cs.avg_multiplicity.len() {
            return self;
        }
        if !cs.predicates.windows(2).all(|w| w[0] < w[1]) {
            let mut paired: Vec<(String, f64)> =
                cs.predicates.into_iter().zip(cs.avg_multiplicity).collect();
            paired.sort_by(|a, b| a.0.cmp(&b.0));
            cs = CharSet {
                predicates: paired.iter().map(|(p, _)| p.clone()).collect(),
                avg_multiplicity: paired.iter().map(|(_, m)| *m).collect(),
                subjects: cs.subjects,
            };
        }
        self.char_sets.push(cs);
        self
    }

    /// Declares the IRI-authority capability set **complete**: authority pruning of
    /// subject/object positions becomes active for this source. Use only when the
    /// descriptor truly enumerates every authority the source can mint (e.g. a
    /// HiBISCuS-style capability set / `void:uriSpace` declaration). A descriptor built
    /// only from VoID predicate/class partitions must NOT call this (recall-safe).
    pub fn authorities_complete(mut self) -> Self {
        self.authorities_complete = true;
        self
    }

    /// Finalises the descriptor, deriving the authority set from every predicate/class
    /// IRI (and any explicitly-declared instance authority — none in this slice).
    pub fn build(self) -> SourceDescriptor {
        let mut authorities = FxHashSet::default();
        for p in self.predicates.keys() {
            if let Some(a) = authority_of(p) {
                authorities.insert(a);
            }
        }
        for c in self.classes.keys() {
            if let Some(a) = authority_of(c) {
                authorities.insert(a);
            }
        }
        SourceDescriptor {
            id: self.id,
            total_triples: self.total_triples,
            predicates: self.predicates,
            classes: self.classes,
            char_sets: self.char_sets,
            authorities,
            authorities_complete: self.authorities_complete,
        }
    }
}

impl SourceDescriptor {
    /// Parses a served descriptor from its N-Triples form — the document a
    /// `sparq-server` serves at `/.well-known/void` (VoID partitions + `scs:`
    /// characteristic sets). Blank-node-keyed partitions (`void:propertyPartition`,
    /// `void:classPartition`, `scs:` sets) are reassembled by blank-node label.
    ///
    /// The parse path produces an **incomplete** authority set (VoID partitions reveal
    /// only predicate/class authorities, never subject/object instance authorities), so
    /// subject/object authority-pruning stays disabled — recall-safe by construction.
    /// To enable it, build via [`SourceDescriptor::builder`] and call
    /// [`SourceDescriptorBuilder::authorities_complete`] with a vetted capability set.
    pub fn from_void_nt(id: SourceId, nt: &str) -> Result<SourceDescriptor, String> {
        use oxrdf::{NamedOrBlankNode, Term as OxTerm};
        let v = |local: &str| format!("{VOID_NS}{local}");
        let cs = |local: &str| format!("{CS_NS}{local}");

        // Collect blank-node partition rows: bnode label -> (predicate -> objects).
        let mut bn: FxHashMap<String, FxHashMap<String, Vec<OxTerm>>> = FxHashMap::default();
        let mut total_triples = 0u64;
        // scs ordered-list members: bnode -> Vec<(index, value)> for predicate/mult lists.
        for r in oxttl::NTriplesParser::new().for_slice(nt.as_bytes()) {
            let t = r.map_err(|e| format!("malformed descriptor N-Triples: {e}"))?;
            let pred = t.predicate.as_str().to_string();
            // Top-level dataset void:triples (subject is an IRI, not a partition bnode).
            if pred == v("triples") {
                if let NamedOrBlankNode::NamedNode(_) = &t.subject {
                    if let Some(n) = as_u64(&t.object) {
                        total_triples = total_triples.max(n);
                    }
                    continue;
                }
            }
            if let NamedOrBlankNode::BlankNode(b) = &t.subject {
                bn.entry(b.as_str().to_string())
                    .or_default()
                    .entry(pred)
                    .or_default()
                    .push(t.object.clone());
            }
        }

        let mut builder = SourceDescriptor::builder(id).total_triples(total_triples);

        for props in bn.values() {
            // --- void:propertyPartition: keyed by void:property.
            if let Some(p) = props.get(&v("property")).and_then(|o| first_iri(o)) {
                builder = builder.predicate(PredPartition {
                    predicate: p,
                    triples: props
                        .get(&v("triples"))
                        .and_then(|o| first_u64(o))
                        .unwrap_or(0),
                    distinct_subjects: props
                        .get(&v("distinctSubjects"))
                        .and_then(|o| first_u64(o))
                        .unwrap_or(0),
                    distinct_objects: props
                        .get(&v("distinctObjects"))
                        .and_then(|o| first_u64(o))
                        .unwrap_or(0),
                });
                continue;
            }
            // --- void:classPartition: keyed by void:class.
            if let Some(c) = props.get(&v("class")).and_then(|o| first_iri(o)) {
                builder = builder.class(ClassPartition {
                    class: c,
                    entities: props
                        .get(&v("entities"))
                        .and_then(|o| first_u64(o))
                        .unwrap_or(0),
                });
                continue;
            }
            // --- scs:CharacteristicSet: scs:subjects + per-predicate scs:predicateStat
            //     blank nodes (each carrying void:property + scs:avgMultiplicity).
            let is_cs = props
                .get(RDF_TYPE)
                .map(|o| o.iter().any(|t| iri_eq(t, &cs("CharacteristicSet"))))
                .unwrap_or(false);
            if is_cs {
                let subjects = props
                    .get(&cs("subjects"))
                    .and_then(|o| first_u64(o))
                    .unwrap_or(0);
                let mut preds: Vec<(String, f64)> = Vec::new();
                if let Some(stats) = props.get(&cs("predicateStat")) {
                    for st in stats {
                        if let OxTerm::BlankNode(sb) = st {
                            if let Some(stat) = bn.get(sb.as_str()) {
                                if let Some(p) = stat.get(&v("property")).and_then(|o| first_iri(o))
                                {
                                    let m = stat
                                        .get(&cs("avgMultiplicity"))
                                        .and_then(|o| first_f64(o))
                                        .unwrap_or(1.0);
                                    preds.push((p, m));
                                }
                            }
                        }
                    }
                }
                preds.sort_by(|a, b| a.0.cmp(&b.0));
                builder = builder.char_set(CharSet {
                    predicates: preds.iter().map(|(p, _)| p.clone()).collect(),
                    avg_multiplicity: preds.iter().map(|(_, m)| *m).collect(),
                    subjects,
                });
            }
        }

        Ok(builder.build())
    }
}

const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

// ---- small RDF-term helpers ----------------------------------------------------

fn iri_eq(t: &oxrdf::Term, iri: &str) -> bool {
    matches!(t, oxrdf::Term::NamedNode(n) if n.as_str() == iri)
}

fn first_iri(os: &[oxrdf::Term]) -> Option<String> {
    os.iter().find_map(|t| match t {
        oxrdf::Term::NamedNode(n) => Some(n.as_str().to_string()),
        _ => None,
    })
}

fn as_u64(t: &oxrdf::Term) -> Option<u64> {
    match t {
        oxrdf::Term::Literal(l) => l.value().parse::<u64>().ok(),
        _ => None,
    }
}

fn first_u64(os: &[oxrdf::Term]) -> Option<u64> {
    os.iter().find_map(as_u64)
}

fn first_f64(os: &[oxrdf::Term]) -> Option<f64> {
    os.iter().find_map(|t| match t {
        oxrdf::Term::Literal(l) => l.value().parse::<f64>().ok(),
        _ => None,
    })
}

/// The IRI **authority** (scheme + `://` + host[:port]) of an absolute IRI, e.g.
/// `http://dbpedia.org/resource/Berlin` ⇒ `http://dbpedia.org`. `None` for IRIs with no
/// `://` (relative, `urn:`, etc.) — those are never used to prune (recall-safe).
fn authority_of(iri: &str) -> Option<String> {
    let scheme_end = iri.find("://")?;
    let after = scheme_end + 3;
    let rest = &iri[after..];
    // Authority ends at the first `/`, `?`, `#`, or end of string.
    let host_len = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    if host_len == 0 {
        return None;
    }
    Some(iri[..after + host_len].to_string())
}

/// `a ⊆ b` over two ascending-sorted string slices (merge walk).
fn is_subset(a: &[String], b: &[String]) -> bool {
    let mut j = 0;
    'outer: for x in a {
        while j < b.len() {
            match b[j].as_str().cmp(x.as_str()) {
                std::cmp::Ordering::Less => j += 1,
                std::cmp::Ordering::Equal => {
                    j += 1;
                    continue 'outer;
                }
                std::cmp::Ordering::Greater => return false,
            }
        }
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    // [OPUS-4.8] sq-qik4: `SourceDescriptorBuilder` is the return type of the PUBLIC
    // `SourceDescriptor::builder` and the link target of `from_void_nt`'s doc, so it must be a
    // nameable crate-level public item. Referencing it through `crate::` (NOT `super::`) makes a
    // future drop of the re-export a compile error here — not just a residual `cargo doc` warning.
    #[test]
    fn builder_type_is_publicly_nameable() {
        let b: crate::SourceDescriptorBuilder =
            SourceDescriptor::builder(SourceId::new("http://s/"));
        // The builder method named in `from_void_nt`'s doc opts the descriptor into complete
        // authorities; the built descriptor must then report `authorities_complete() == true`.
        let d = b.authorities_complete().build();
        assert!(d.authorities_complete());
    }

    #[test]
    fn authority_extraction() {
        assert_eq!(
            authority_of("http://dbpedia.org/resource/Berlin"),
            Some("http://dbpedia.org".to_string())
        );
        assert_eq!(
            authority_of("https://www.wikidata.org/entity/Q64"),
            Some("https://www.wikidata.org".to_string())
        );
        assert_eq!(
            authority_of("http://example.org:8080/x#y"),
            Some("http://example.org:8080".to_string())
        );
        // No authority ⇒ None (never prunes).
        assert_eq!(authority_of("urn:isbn:123"), None);
        assert_eq!(authority_of("relativeThing"), None);
    }

    #[test]
    fn avg_multiplicity_defaults_to_one_when_unknown() {
        let p = PredPartition {
            predicate: "http://ex/p".into(),
            triples: 100,
            distinct_subjects: 0,
            distinct_objects: 0,
        };
        assert_eq!(p.avg_multiplicity(), 1.0);
        let p2 = PredPartition {
            distinct_subjects: 50,
            ..p
        };
        assert_eq!(p2.avg_multiplicity(), 2.0);
    }

    #[test]
    fn star_cardinality_matches_cs_model() {
        // 100 subjects {a,b} (a avg 1x, b avg 2x), 50 subjects {a,c} (a 1x, c 1x).
        let d = SourceDescriptor::builder(SourceId::new("s"))
            .char_set(CharSet {
                predicates: vec!["http://ex/a".into(), "http://ex/b".into()],
                subjects: 100,
                avg_multiplicity: vec![1.0, 2.0],
            })
            .char_set(CharSet {
                predicates: vec!["http://ex/a".into(), "http://ex/c".into()],
                subjects: 50,
                avg_multiplicity: vec![1.0, 1.0],
            })
            .build();
        let a = "http://ex/a".to_string();
        let b = "http://ex/b".to_string();
        let c = "http://ex/c".to_string();
        assert_eq!(d.star_subjects(std::slice::from_ref(&a)), 150);
        assert_eq!(d.star_subjects(&[a.clone(), b.clone()]), 100);
        assert_eq!(d.star_subjects(&[b.clone(), c.clone()]), 0);
        assert_eq!(d.star_cardinality(std::slice::from_ref(&a)), 150.0);
        assert_eq!(d.star_cardinality(&[a.clone(), b.clone()]), 200.0); // 100 * 1 * 2
        assert_eq!(d.star_cardinality(&[b, c]), 0.0);
    }

    #[test]
    fn parse_void_nt_predicate_and_class_partitions() {
        // Minimal served VoID document with one property + one class partition.
        let nt = r#"<http://s/> <http://rdfs.org/ns/void#triples> "300"^^<http://www.w3.org/2001/XMLSchema#integer> .
<http://s/> <http://rdfs.org/ns/void#propertyPartition> _:p1 .
_:p1 <http://rdfs.org/ns/void#property> <http://xmlns.com/foaf/0.1/knows> .
_:p1 <http://rdfs.org/ns/void#triples> "200"^^<http://www.w3.org/2001/XMLSchema#integer> .
_:p1 <http://rdfs.org/ns/void#distinctSubjects> "100"^^<http://www.w3.org/2001/XMLSchema#integer> .
<http://s/> <http://rdfs.org/ns/void#classPartition> _:c1 .
_:c1 <http://rdfs.org/ns/void#class> <http://xmlns.com/foaf/0.1/Person> .
_:c1 <http://rdfs.org/ns/void#entities> "100"^^<http://www.w3.org/2001/XMLSchema#integer> .
"#;
        let d = SourceDescriptor::from_void_nt(SourceId::new("http://s/"), nt).unwrap();
        assert_eq!(d.total_triples, 300);
        let p = d.predicate("http://xmlns.com/foaf/0.1/knows").unwrap();
        assert_eq!(p.triples, 200);
        assert_eq!(p.distinct_subjects, 100);
        assert_eq!(p.avg_multiplicity(), 2.0);
        assert!(d.has_class("http://xmlns.com/foaf/0.1/Person"));
        assert_eq!(
            d.class("http://xmlns.com/foaf/0.1/Person")
                .unwrap()
                .entities,
            100
        );
        // Parsed descriptors are authority-INCOMPLETE (recall-safe).
        assert!(!d.authorities_complete());
    }

    #[test]
    fn parse_void_nt_characteristic_set() {
        let nt = r#"<http://s/> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://sparq.dev/ns/cs#CharacteristicSet> .
_:cs1 <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://sparq.dev/ns/cs#CharacteristicSet> .
_:cs1 <http://sparq.dev/ns/cs#subjects> "100"^^<http://www.w3.org/2001/XMLSchema#integer> .
_:cs1 <http://sparq.dev/ns/cs#predicateStat> _:st1 .
_:cs1 <http://sparq.dev/ns/cs#predicateStat> _:st2 .
_:st1 <http://rdfs.org/ns/void#property> <http://ex/a> .
_:st1 <http://sparq.dev/ns/cs#avgMultiplicity> "1.0"^^<http://www.w3.org/2001/XMLSchema#double> .
_:st2 <http://rdfs.org/ns/void#property> <http://ex/b> .
_:st2 <http://sparq.dev/ns/cs#avgMultiplicity> "2.0"^^<http://www.w3.org/2001/XMLSchema#double> .
"#;
        let d = SourceDescriptor::from_void_nt(SourceId::new("http://s/"), nt).unwrap();
        assert_eq!(d.char_sets().len(), 1);
        let cs = &d.char_sets()[0];
        assert_eq!(cs.subjects, 100);
        assert_eq!(
            cs.predicates,
            vec!["http://ex/a".to_string(), "http://ex/b".to_string()]
        );
        assert_eq!(cs.avg_multiplicity, vec![1.0, 2.0]);
        assert_eq!(
            d.star_cardinality(&["http://ex/a".into(), "http://ex/b".into()]),
            200.0
        );
    }

    // ============================================================================
    // [OPUS-4.8] sq-bif.3 — correctness suite: previously-uncovered descriptor branches
    // (the parse error path, the char-set validation/normalisation guards, the top-level
    // void:triples max-merge, authority_of edge cases, may_hold_authority on an
    // unparseable IRI under a complete set, and the is_subset / star-estimate non-cover
    // paths). Drives the REAL parser/builder/estimator, not a re-implementation.
    // ============================================================================

    // ---- A malformed N-Triples descriptor is a clean Err (not a panic / partial parse).
    #[test]
    fn parse_void_nt_malformed_is_error() {
        let bad = "<http://s/> this is not n-triples";
        let r = SourceDescriptor::from_void_nt(SourceId::new("http://s/"), bad);
        assert!(r.is_err(), "malformed N-Triples must return Err");
        assert!(
            r.unwrap_err().contains("malformed descriptor N-Triples"),
            "the error message names the malformed-descriptor cause"
        );
    }

    // ---- An EMPTY descriptor document parses to a valid, empty descriptor (no partitions,
    //      0 total triples, authority-incomplete).
    #[test]
    fn parse_void_nt_empty_document() {
        let d = SourceDescriptor::from_void_nt(SourceId::new("http://s/"), "").unwrap();
        assert_eq!(d.total_triples, 0);
        assert!(d.char_sets().is_empty());
        assert!(!d.declares_any_class());
        assert!(!d.authorities_complete());
    }

    // ---- Top-level dataset `void:triples` takes the MAX over several values on an IRI
    //      subject (mirrors the parser's `total_triples.max(n)`), and a `void:triples` on a
    //      BLANK subject is a partition count, not the dataset total.
    #[test]
    fn parse_void_nt_total_triples_takes_max() {
        let nt = r#"<http://s/> <http://rdfs.org/ns/void#triples> "100"^^<http://www.w3.org/2001/XMLSchema#integer> .
<http://s/> <http://rdfs.org/ns/void#triples> "500"^^<http://www.w3.org/2001/XMLSchema#integer> .
<http://s/> <http://rdfs.org/ns/void#triples> "300"^^<http://www.w3.org/2001/XMLSchema#integer> .
"#;
        let d = SourceDescriptor::from_void_nt(SourceId::new("http://s/"), nt).unwrap();
        assert_eq!(d.total_triples, 500, "dataset total is the max of the served values");
    }

    // ---- The builder DROPS a characteristic set with `subjects == 0` (degenerate, mirrors
    //      the engine's CsTable::new) and one whose `predicates`/`avg_multiplicity` lengths
    //      disagree (malformed) — but keeps a valid one.
    #[test]
    fn builder_drops_degenerate_char_sets() {
        let d = SourceDescriptor::builder(SourceId::new("S"))
            // subjects == 0 ⇒ dropped.
            .char_set(CharSet {
                predicates: vec!["http://ex/a".into()],
                subjects: 0,
                avg_multiplicity: vec![1.0],
            })
            // length mismatch (2 predicates, 1 multiplicity) ⇒ dropped.
            .char_set(CharSet {
                predicates: vec!["http://ex/a".into(), "http://ex/b".into()],
                subjects: 10,
                avg_multiplicity: vec![1.0],
            })
            // valid ⇒ kept.
            .char_set(CharSet {
                predicates: vec!["http://ex/c".into()],
                subjects: 7,
                avg_multiplicity: vec![3.0],
            })
            .build();
        assert_eq!(d.char_sets().len(), 1, "only the valid char set survives");
        assert_eq!(d.char_sets()[0].subjects, 7);
        assert_eq!(d.char_sets()[0].predicates, vec!["http://ex/c".to_string()]);
    }

    // ---- The builder NORMALISES an out-of-order char set's predicates to ascending order,
    //      carrying the aligned `avg_multiplicity` with them (so the binary-search superset
    //      walk in `star_cardinality` stays correct).
    #[test]
    fn builder_sorts_char_set_predicates_keeping_multiplicity_aligned() {
        // Supplied descending: (b, 2.0), (a, 1.0) ⇒ must become (a,1.0),(b,2.0).
        let d = SourceDescriptor::builder(SourceId::new("S"))
            .char_set(CharSet {
                predicates: vec!["http://ex/b".into(), "http://ex/a".into()],
                subjects: 100,
                avg_multiplicity: vec![2.0, 1.0],
            })
            .build();
        let cs = &d.char_sets()[0];
        assert_eq!(
            cs.predicates,
            vec!["http://ex/a".to_string(), "http://ex/b".to_string()],
            "predicates normalised to ascending order"
        );
        assert_eq!(
            cs.avg_multiplicity,
            vec![1.0, 2.0],
            "avg_multiplicity follows its predicate through the sort"
        );
        // The aligned sort keeps the star estimate correct: 100 subjects * a(1) * b(2) = 200.
        assert_eq!(
            d.star_cardinality(&["http://ex/a".into(), "http://ex/b".into()]),
            200.0
        );
    }

    // ---- `authority_of` edge cases beyond the existing test: an IRI with an empty host
    //      (`http:///path` ⇒ host_len 0 ⇒ None), an authority-only IRI (no path), and a
    //      query/fragment immediately after the host.
    #[test]
    fn authority_of_edge_cases() {
        // Empty host between `://` and the first `/` ⇒ None (never prunes).
        assert_eq!(authority_of("http:///path"), None);
        // Authority with no path at all ⇒ the whole thing is the authority.
        assert_eq!(
            authority_of("https://example.org"),
            Some("https://example.org".to_string())
        );
        // Query / fragment delimit the authority just like `/`.
        assert_eq!(
            authority_of("http://h.example?q=1"),
            Some("http://h.example".to_string())
        );
        assert_eq!(
            authority_of("http://h.example#frag"),
            Some("http://h.example".to_string())
        );
    }

    // ---- A COMPLETE-authority source keeps a pattern whose bound IRI has NO extractable
    //      authority (a `urn:`/relative IRI): `may_hold_authority` returns true (cannot rule
    //      out) even under a complete set — the recall-safe unparseable-authority default.
    #[test]
    fn complete_authority_keeps_unparseable_iri() {
        let d = SourceDescriptor::builder(SourceId::new("S"))
            .predicate(PredPartition {
                predicate: "http://ex/p".into(),
                triples: 10,
                distinct_subjects: 10,
                distinct_objects: 10,
            })
            .authorities_complete()
            .build();
        // A urn: subject has no `://` authority ⇒ cannot be ruled out ⇒ kept.
        assert!(
            d.may_hold_authority("urn:isbn:123"),
            "an unparseable-authority IRI is never pruned, even under a complete set"
        );
        // A known foreign http authority IS ruled out (the complete set is active).
        assert!(!d.may_hold_authority("http://elsewhere.example/x"));
        // The source's own predicate authority is kept.
        assert!(d.may_hold_authority("http://ex/something"));
    }

    // ---- `may_hold_authority` always returns true on an INCOMPLETE authority set, even for
    //      a clearly-foreign IRI (the default VoID-parsed posture).
    #[test]
    fn incomplete_authority_set_never_rules_out() {
        let d = SourceDescriptor::builder(SourceId::new("S"))
            .predicate(PredPartition {
                predicate: "http://ex/p".into(),
                triples: 10,
                distinct_subjects: 10,
                distinct_objects: 10,
            })
            .build(); // NOT authorities_complete.
        assert!(d.may_hold_authority("http://anything.example/at-all"));
    }

    // ---- `star_subjects` / `star_cardinality` over a query the served sets do NOT all
    //      cover: only the sets that are a SUPERSET of the query contribute; a query no set
    //      covers yields 0. Exercises the `is_subset` merge-walk Greater/exhausted branches.
    #[test]
    fn star_estimate_only_counts_covering_sets() {
        let d = SourceDescriptor::builder(SourceId::new("S"))
            // set {a,b}: 100 subjects, a 1×, b 2×.
            .char_set(CharSet {
                predicates: vec!["http://ex/a".into(), "http://ex/b".into()],
                subjects: 100,
                avg_multiplicity: vec![1.0, 2.0],
            })
            // set {a}: 50 subjects, a 1×.
            .char_set(CharSet {
                predicates: vec!["http://ex/a".into()],
                subjects: 50,
                avg_multiplicity: vec![1.0],
            })
            .build();
        let a = "http://ex/a".to_string();
        let b = "http://ex/b".to_string();
        let z = "http://ex/z".to_string();
        // {a} ⊆ both sets ⇒ 100 + 50 subjects.
        assert_eq!(d.star_subjects(std::slice::from_ref(&a)), 150);
        // {a,b} ⊆ only the first set ⇒ 100 subjects, cardinality 100*1*2 = 200.
        assert_eq!(d.star_subjects(&[a.clone(), b.clone()]), 100);
        assert_eq!(d.star_cardinality(&[a.clone(), b]), 200.0);
        // {a,z} ⊆ NO set (z absent ⇒ the merge walk hits Greater/exhausted) ⇒ 0.
        assert_eq!(d.star_subjects(&[a.clone(), z.clone()]), 0);
        assert_eq!(d.star_cardinality(&[a, z]), 0.0);
        // An EMPTY query is a subset of every set ⇒ subjects summed, cardinality = subjects.
        assert_eq!(d.star_subjects(&[]), 150);
        assert_eq!(d.star_cardinality(&[]), 150.0);
    }

    // ---- A char set with NO served sets reports 0 for any non-empty query (no fabrication).
    #[test]
    fn star_estimate_with_no_char_sets_is_zero() {
        let d = SourceDescriptor::builder(SourceId::new("S"))
            .predicate(PredPartition {
                predicate: "http://ex/a".into(),
                triples: 100,
                distinct_subjects: 100,
                distinct_objects: 100,
            })
            .build();
        assert_eq!(d.star_subjects(&["http://ex/a".into()]), 0);
        assert_eq!(d.star_cardinality(&["http://ex/a".into()]), 0.0);
    }

    // ---- Accessors line up: `predicate`/`has_predicate`/`class`/`has_class`/`id` read back
    //      what the builder put in, and return None/false for absent keys.
    #[test]
    fn descriptor_accessors_round_trip() {
        let d = SourceDescriptor::builder(SourceId::new("http://s/"))
            .total_triples(42)
            .predicate(PredPartition {
                predicate: "http://ex/p".into(),
                triples: 10,
                distinct_subjects: 5,
                distinct_objects: 4,
            })
            .class(ClassPartition {
                class: "http://ex/C".into(),
                entities: 3,
            })
            .build();
        assert_eq!(d.id(), &SourceId::new("http://s/"));
        assert_eq!(d.total_triples, 42);
        assert!(d.has_predicate("http://ex/p"));
        assert!(!d.has_predicate("http://ex/absent"));
        assert_eq!(d.predicate("http://ex/p").unwrap().triples, 10);
        assert!(d.predicate("http://ex/absent").is_none());
        assert!(d.has_class("http://ex/C"));
        assert!(!d.has_class("http://ex/Absent"));
        assert_eq!(d.class("http://ex/C").unwrap().entities, 3);
        assert!(d.class("http://ex/Absent").is_none());
        assert!(d.declares_any_class());
    }
}
