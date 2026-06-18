//! Entity & relation linking — mapping a question's mentions to concrete IRIs in the
//! store, then expanding each linked entity with its structurally-similar siblings.
//!
//! This is the **index-grounded linking** step of the design
//! (`research/genai-nl-to-sparql.md` §2.6 "Entity & relation linking", §8.3
//! "Index-grounded entity/relation linking with cardinality priors"): the prompt's
//! schema summary tells the model what *classes and predicates* exist, but not which
//! concrete IRIs the question's proper nouns ("Tarantino", "France") resolve to. Without
//! that, value-bound and entity-bound queries can only be guessed. Linking closes the
//! **entity gap** (§2, gap 2) cheaply, with no model and no network:
//!
//! 1. **Mention generation** — the question is split into word n-grams (1..=3 words);
//!    very short tokens and a small stop-word set are dropped from unigrams.
//! 2. **Lexical entity linking** — a once-built `lowercased label → entity` index over
//!    the common label predicates (`rdfs:label`, `skos:prefLabel`, `schema:name`,
//!    `foaf:name`, `dc:title`, `dcterms:title`) resolves a mention to candidate
//!    entities; exact label equality outranks containment, and rarer (higher-IDF) label
//!    predicates outrank `rdfs:label`.
//! 3. **Structural expansion (sparq-sim)** — each linked entity is expanded with its
//!    top structurally-similar siblings via [`sparq_sim::Sim::most_similar`] (the wiring
//!    `sq-uw40` calls for), so the model sees a few worked examples of the entity's
//!    *shape* — the predicates it participates in and a sibling it resembles.
//! 4. **Relation linking** — mentions are matched against predicate IRI local-names
//!    (camelCase split), ranked by triple count (the cardinality prior of §8.3).
//!
//! The result is rendered as a compact prompt section ([`Linking::to_prompt_section`])
//! appended to the schema summary. Everything is read-only over the public `sparq-core`
//! API; the index is built once per [`crate::Nlq`] and reused across `ask` calls.

use std::collections::BTreeMap;

use oxrdf::{NamedNode, Term};
use sparq_core::Graph;
use sparq_sim::Sim;

/// Predicates whose literal objects are treated as entity labels for linking, paired
/// with a small ranking bonus (rarer/more-specific naming predicates outrank the
/// ubiquitous `rdfs:label`). Order is fixed and dataset-agnostic.
const LABEL_PREDICATES: &[(&str, f64)] = &[
    ("http://www.w3.org/2004/02/skos/core#prefLabel", 1.5),
    ("http://schema.org/name", 1.4),
    ("http://xmlns.com/foaf/0.1/name", 1.4),
    ("http://purl.org/dc/terms/title", 1.3),
    ("http://purl.org/dc/elements/1.1/title", 1.3),
    ("http://www.w3.org/2000/01/rdf-schema#label", 1.0),
];

/// A tiny, language-neutral stop list: dropped from *unigram* mentions only (multi-word
/// mentions keep them, so "King of France" still matches). Kept deliberately small.
const STOP_WORDS: &[&str] = &[
    "the", "a", "an", "of", "in", "on", "for", "to", "and", "or", "is", "are", "was", "were", "be",
    "by", "with", "as", "at", "from", "that", "this", "these", "those", "how", "many", "what",
    "which", "who", "where", "when", "list", "show", "all", "each", "there", "do", "does", "did",
    "has", "have", "had",
];

/// One entity linked from the question, with the structurally-similar siblings
/// `sparq-sim` surfaced for it.
#[derive(Debug, Clone, PartialEq)]
pub struct LinkedEntity {
    /// The question span that matched (lower-cased).
    pub mention: String,
    /// The resolved entity IRI.
    pub iri: NamedNode,
    /// A human-readable label for the entity (the matched label literal).
    pub label: String,
    /// `true` if the mention equalled the whole label, `false` if it was a substring.
    pub exact: bool,
    /// Structurally-similar sibling IRIs (best first) from
    /// [`sparq_sim::Sim::most_similar`] — the worked examples handed to the model.
    pub similar: Vec<NamedNode>,
}

/// One relation (predicate) linked from the question by local-name match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkedRelation {
    pub mention: String,
    pub iri: NamedNode,
    /// Triple count behind the predicate (the cardinality prior — §8.3).
    pub triples: u64,
}

/// The linking result for one question.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Linking {
    pub entities: Vec<LinkedEntity>,
    pub relations: Vec<LinkedRelation>,
}

impl Linking {
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty() && self.relations.is_empty()
    }

    /// Renders the linked entities/relations as a prompt section, or `None` when nothing
    /// linked (so the prompt — and therefore any recorded fixture — is unchanged when
    /// linking adds no signal). The rendering is deterministic: entities then relations,
    /// each already in rank order.
    pub fn to_prompt_section(&self) -> Option<String> {
        if self.is_empty() {
            return None;
        }
        let mut s = String::from(
            "# Linked from the question\n\
             These IRIs from the question's proper nouns are present in THIS dataset — \
             prefer them over guessed IRIs:\n",
        );
        for e in &self.entities {
            let kind = if e.exact { "exact" } else { "contains" };
            s.push_str(&format!(
                "- entity \"{}\" ({kind}) -> <{}>  (label: {})\n",
                e.mention,
                e.iri.as_str(),
                e.label
            ));
            if !e.similar.is_empty() {
                let sims: Vec<String> = e
                    .similar
                    .iter()
                    .map(|n| format!("<{}>", n.as_str()))
                    .collect();
                s.push_str(&format!("    similar entities: {}\n", sims.join(", ")));
            }
        }
        for r in &self.relations {
            s.push_str(&format!(
                "- relation \"{}\" -> <{}>  ({} triples)\n",
                r.mention,
                r.iri.as_str(),
                r.triples
            ));
        }
        Some(s)
    }
}

/// A reusable, index-grounded linker built once over a graph: it snapshots a
/// `lowercased-label -> (entity, label, score)` map from the label predicates and the
/// predicate local-names, then [`link`](EntityLinker::link)s any number of questions
/// against them. Construction cost is one scan per present label predicate; `link` is
/// then string work plus one [`sparq_sim::Sim::most_similar`] call per linked entity.
pub struct EntityLinker<'g> {
    sim: Sim<'g>,
    /// lowercased label -> the best (entity, original-label, score) for it.
    labels: BTreeMap<String, (NamedNode, String, f64)>,
    /// lowercased predicate local-name -> (predicate IRI, triple count).
    predicates: BTreeMap<String, (NamedNode, u64)>,
    /// How many similar siblings to attach per linked entity.
    expand_k: usize,
    /// Max entities / relations to return (keeps the prompt section bounded).
    max_links: usize,
}

impl<'g> EntityLinker<'g> {
    /// Builds a linker over `graph`. `expand_k` is the number of structurally-similar
    /// siblings attached to each linked entity (0 disables the sparq-sim expansion);
    /// `max_links` bounds the entities and relations rendered into the prompt.
    pub fn build(graph: &'g Graph, expand_k: usize, max_links: usize) -> Self {
        let labels = build_label_index(graph);
        let predicates = build_predicate_index(graph);
        EntityLinker {
            sim: Sim::new(graph),
            labels,
            predicates,
            expand_k,
            max_links,
        }
    }

    /// Links `question` to entities and relations in the graph.
    pub fn link(&self, question: &str) -> Linking {
        let mentions = mentions(question);

        // ---- Entity linking: best candidate per mention, exact beats substring. ----
        // Keyed by IRI so the same entity matched by several mentions appears once; the
        // value carries a private rank `(exact, label-predicate score)` used only to
        // order the output (kept off the public struct).
        let mut by_iri: BTreeMap<String, (LinkedEntity, (bool, f64))> = BTreeMap::new();
        for m in &mentions {
            if let Some((iri, label, exact, score)) = self.best_entity_for(m) {
                let key = iri.as_str().to_string();
                let rank = (exact, score);
                let better = by_iri
                    .get(&key)
                    .map(|(_, prev)| rank > *prev)
                    .unwrap_or(true);
                if better {
                    by_iri.insert(
                        key,
                        (
                            LinkedEntity {
                                mention: m.clone(),
                                iri,
                                label,
                                exact,
                                similar: Vec::new(),
                            },
                            rank,
                        ),
                    );
                }
            }
        }
        let mut ranked: Vec<(LinkedEntity, (bool, f64))> = by_iri.into_values().collect();
        // Rank: exact first, then by label-predicate score, then IRI for determinism.
        ranked.sort_by(|(ea, ra), (eb, rb)| {
            rb.0.cmp(&ra.0)
                .then(rb.1.partial_cmp(&ra.1).unwrap_or(std::cmp::Ordering::Equal))
                .then(ea.iri.as_str().cmp(eb.iri.as_str()))
        });
        let mut entities: Vec<LinkedEntity> = ranked.into_iter().map(|(e, _)| e).collect();
        entities.truncate(self.max_links);

        // ---- Structural expansion via sparq-sim (the sq-uw40 wiring). ----
        if self.expand_k > 0 {
            for e in &mut entities {
                let term = Term::NamedNode(e.iri.clone());
                e.similar = self
                    .sim
                    .most_similar(&term, self.expand_k)
                    .into_iter()
                    .filter_map(|(t, _score)| match t {
                        Term::NamedNode(n) => Some(n),
                        _ => None,
                    })
                    .collect();
            }
        }

        // ---- Relation linking against predicate local-names. ----
        let mut relations: Vec<LinkedRelation> = Vec::new();
        let mut seen_pred: BTreeMap<String, ()> = BTreeMap::new();
        for m in &mentions {
            if let Some((iri, triples)) = self.predicates.get(m) {
                if seen_pred.insert(iri.as_str().to_string(), ()).is_none() {
                    relations.push(LinkedRelation {
                        mention: m.clone(),
                        iri: iri.clone(),
                        triples: *triples,
                    });
                }
            }
        }
        // Most-used predicates first (cardinality prior).
        relations.sort_by(|a, b| {
            b.triples
                .cmp(&a.triples)
                .then(a.iri.as_str().cmp(b.iri.as_str()))
        });
        relations.truncate(self.max_links);

        Linking {
            entities,
            relations,
        }
    }

    /// The best entity for one mention: exact label match wins over substring; among
    /// ties, the higher label-predicate score; returns `(iri, label, exact, score)`.
    fn best_entity_for(&self, mention: &str) -> Option<(NamedNode, String, bool, f64)> {
        // Exact match on the full label.
        if let Some((iri, label, score)) = self.labels.get(mention) {
            return Some((iri.clone(), label.clone(), true, *score));
        }
        // Substring: the mention is contained in a label, or a label in the mention.
        // Only for mentions long enough to be discriminating (avoid "ed" matching).
        if mention.len() < 4 {
            return None;
        }
        let mut best: Option<(NamedNode, String, f64)> = None;
        for (lbl, (iri, orig, score)) in &self.labels {
            if lbl.contains(mention) || mention.contains(lbl.as_str()) {
                let s = *score;
                if best.as_ref().map(|b| s > b.2).unwrap_or(true) {
                    best = Some((iri.clone(), orig.clone(), s));
                }
            }
        }
        best.map(|(iri, label, score)| (iri, label, false, score))
    }
}

/// Splits a question into candidate mentions: word unigrams (minus stop words and very
/// short tokens), bigrams and trigrams, all lower-cased. A word is a maximal run of
/// alphanumerics; punctuation is a boundary.
fn mentions(question: &str) -> Vec<String> {
    let words: Vec<String> = question
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(|w| w.to_lowercase())
        .collect();
    let mut out: Vec<String> = Vec::new();
    // Unigrams (filtered).
    for w in &words {
        if w.len() >= 3 && !STOP_WORDS.contains(&w.as_str()) {
            out.push(w.clone());
        }
    }
    // Bigrams and trigrams (unfiltered — multi-word spans are inherently specific).
    for win in 2..=3 {
        if words.len() >= win {
            for chunk in words.windows(win) {
                out.push(chunk.join(" "));
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

/// One scan per present label predicate, building the lowercased-label index. When two
/// entities share a label, the higher-scoring predicate (else the lexicographically
/// smaller IRI) wins, so the index is deterministic.
fn build_label_index(graph: &Graph) -> BTreeMap<String, (NamedNode, String, f64)> {
    let mut idx: BTreeMap<String, (NamedNode, String, f64)> = BTreeMap::new();
    for &(pred_iri, score) in LABEL_PREDICATES {
        let Ok(pred) = NamedNode::new(pred_iri) else {
            continue;
        };
        let Some(pid) = graph.id_of(&Term::NamedNode(pred.clone())) else {
            continue;
        };
        // Subject-bound is unavailable on the public scan; iterate predicate-bound via
        // the store's pattern scan.
        let scan = graph.store.scan(&[None, Some(pid), None]);
        for row in scan.rows.iter() {
            let [s, _, o] = scan.to_spo(row);
            let (Term::NamedNode(subj), Term::Literal(lit)) =
                (graph.dict.term(s), graph.dict.term(o))
            else {
                continue;
            };
            let label = lit.value().to_string();
            let key = label.to_lowercase();
            if key.is_empty() {
                continue;
            }
            let candidate = (subj, label, score);
            match idx.get(&key) {
                Some((existing_iri, _, existing_score))
                    if (*existing_score, existing_iri.as_str())
                        >= (score, candidate.0.as_str()) => {}
                _ => {
                    idx.insert(key, candidate);
                }
            }
        }
    }
    idx
}

/// Builds the predicate-local-name index: every predicate's local name (and a
/// space-split camelCase form) mapped to the predicate IRI and its triple count.
fn build_predicate_index(graph: &Graph) -> BTreeMap<String, (NamedNode, u64)> {
    let mut idx: BTreeMap<String, (NamedNode, u64)> = BTreeMap::new();
    // Distinct predicates are the distinct values of column 1; iterate predicate-sorted.
    let mut last: Option<sparq_core::dict::Id> = None;
    for [_, p, _] in graph.iter_ids_sorted(1) {
        if last == Some(p) {
            continue;
        }
        last = Some(p);
        let Term::NamedNode(pred) = graph.dict.term(p) else {
            continue;
        };
        let triples = graph.store.pred_stat(p).map_or(0, |s| s.count) as u64;
        for name in local_name_forms(pred.as_str()) {
            // Prefer the higher-triple predicate when two share a local name.
            match idx.get(&name) {
                Some((_, t)) if *t >= triples => {}
                _ => {
                    idx.insert(name, (pred.clone(), triples));
                }
            }
        }
    }
    idx
}

/// The matchable lower-cased forms of a predicate IRI: its local name, and — when the
/// local name is camelCase — the words joined by a space (so "directedBy" also matches
/// the mention "directed by").
fn local_name_forms(iri: &str) -> Vec<String> {
    let local = iri
        .rsplit(['#', '/'])
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or(iri);
    let mut forms = vec![local.to_lowercase()];
    let split = split_camel(local);
    if split.to_lowercase() != local.to_lowercase() {
        forms.push(split.to_lowercase());
    }
    forms.sort();
    forms.dedup();
    forms.into_iter().filter(|s| s.len() >= 3).collect()
}

/// Inserts a space before each interior uppercase letter ("directedBy" -> "directed By",
/// "hasName" -> "has Name"); also treats '_' and '-' as word breaks.
fn split_camel(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    let mut prev_lower = false;
    for c in s.chars() {
        if c == '_' || c == '-' {
            if !out.ends_with(' ') {
                out.push(' ');
            }
            prev_lower = false;
            continue;
        }
        if c.is_uppercase() && prev_lower {
            out.push(' ');
        }
        out.push(c);
        prev_lower = c.is_lowercase() || c.is_numeric();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph() -> Graph {
        let ttl = r#"
            @prefix ex: <http://example.org/> .
            @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
            @prefix foaf: <http://xmlns.com/foaf/0.1/> .
            ex:tarantino a ex:Director ; rdfs:label "Quentin Tarantino" ;
                ex:directed ex:pulpFiction , ex:killBill .
            ex:nolan a ex:Director ; rdfs:label "Christopher Nolan" ;
                ex:directed ex:inception .
            ex:pulpFiction a ex:Film ; rdfs:label "Pulp Fiction" ; ex:director ex:tarantino .
            ex:killBill a ex:Film ; rdfs:label "Kill Bill" ; ex:director ex:tarantino .
            ex:inception a ex:Film ; rdfs:label "Inception" ; ex:director ex:nolan .
            ex:france a ex:Country ; foaf:name "France" .
        "#;
        Graph::load_str(ttl, "turtle").expect("graph parses")
    }

    #[test]
    fn mentions_includes_ngrams_and_drops_stopwords() {
        let m = mentions("Which films did Tarantino direct?");
        assert!(m.contains(&"tarantino".to_string()));
        assert!(m.contains(&"films".to_string()));
        assert!(!m.contains(&"did".to_string())); // stop word, dropped as unigram
        assert!(m.contains(&"which films".to_string())); // bigram keeps everything
    }

    #[test]
    fn links_exact_entity_label() {
        let g = graph();
        let linker = EntityLinker::build(&g, 0, 8);
        let l = linker.link("What did Quentin Tarantino direct?");
        let names: Vec<&str> = l.entities.iter().map(|e| e.iri.as_str()).collect();
        assert!(
            names.contains(&"http://example.org/tarantino"),
            "expected tarantino linked, got {names:?}"
        );
        let e = l
            .entities
            .iter()
            .find(|e| e.iri.as_str() == "http://example.org/tarantino")
            .unwrap();
        assert!(e.exact, "full-label match should be exact");
        assert_eq!(e.label, "Quentin Tarantino");
    }

    #[test]
    fn links_via_non_rdfs_label_predicate() {
        let g = graph();
        let linker = EntityLinker::build(&g, 0, 8);
        let l = linker.link("How big is France?");
        assert!(l
            .entities
            .iter()
            .any(|e| e.iri.as_str() == "http://example.org/france"));
    }

    #[test]
    fn structural_expansion_surfaces_siblings() {
        let g = graph();
        let linker = EntityLinker::build(&g, 3, 8);
        let l = linker.link("Tell me about Quentin Tarantino.");
        let e = l
            .entities
            .iter()
            .find(|e| e.iri.as_str() == "http://example.org/tarantino")
            .expect("tarantino linked");
        // Nolan is the other Director with the same (directed -> Film) shape; sparq-sim
        // should surface him as a structural sibling.
        assert!(
            e.similar
                .iter()
                .any(|n| n.as_str() == "http://example.org/nolan"),
            "expected nolan as a structural sibling, got {:?}",
            e.similar.iter().map(|n| n.as_str()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn relation_linking_matches_local_name_and_camelcase() {
        let g = graph();
        let linker = EntityLinker::build(&g, 0, 8);
        let l = linker.link("Who is the director of Inception?");
        assert!(
            l.relations
                .iter()
                .any(|r| r.iri.as_str() == "http://example.org/director"),
            "expected ex:director linked, got {:?}",
            l.relations
                .iter()
                .map(|r| r.iri.as_str())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn camelcase_splitting() {
        assert_eq!(split_camel("directedBy"), "directed By");
        assert_eq!(split_camel("has_name"), "has name");
        assert_eq!(split_camel("birth-place"), "birth place");
        assert_eq!(split_camel("director"), "director");
    }

    #[test]
    fn empty_linking_renders_no_section() {
        let g = graph();
        let linker = EntityLinker::build(&g, 0, 8);
        let l = linker.link("zzz qqq xyzzy");
        assert!(l.is_empty());
        assert!(l.to_prompt_section().is_none());
    }

    #[test]
    fn prompt_section_is_deterministic_and_contains_iris() {
        let g = graph();
        let linker = EntityLinker::build(&g, 2, 8);
        let l = linker.link("What did Quentin Tarantino direct?");
        let s = l.to_prompt_section().expect("non-empty linking renders");
        assert!(s.contains("http://example.org/tarantino"));
        // Stable across repeated builds (deterministic indexes + sorts).
        let l2 = EntityLinker::build(&g, 2, 8).link("What did Quentin Tarantino direct?");
        assert_eq!(s, l2.to_prompt_section().unwrap());
    }
}
