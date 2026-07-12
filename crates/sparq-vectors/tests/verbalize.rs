//! `verbalize` / `embed_entities` integration on a small fixture graph: template
//! shape, language preference, predicate priority, type naming, prefixes, value caps,
//! char budget, and the embed-side guarantees (coverage, determinism, dim mismatch).

use oxrdf::{NamedNode, Term};
use sparq_core::Graph;
use sparq_vectors::{
    embed_entities, verbalize, Embedder, EntityTextConfig, HashEmbedder, ObjectKind, PropertyGroup,
    VectorStore,
};

const TTL: &str = r#"
@prefix rdfs:   <http://www.w3.org/2000/01/rdf-schema#> .
@prefix skos:   <http://www.w3.org/2004/02/skos/core#> .
@prefix schema: <http://schema.org/> .
@prefix ex:     <http://example.org/> .

# The full template: label + type (labeled in two languages) + description.
ex:bolt rdfs:label "Usain Bolt"@en, "Usain Bolt (fr)"@fr ;
        a ex:Athlete ;
        schema:description "Jamaican sprinter, eight-time Olympic champion."@en ;
        ex:occupation "sprinter", "philanthropist" .
ex:Athlete rdfs:label "athlète"@fr, "athlete"@en .

# Label priority: rdfs:label must beat skos:prefLabel.
ex:both rdfs:label "From rdfs label" ; skos:prefLabel "From skos" .

# Language fallback: only a French label — must still verbalize.
ex:paris rdfs:label "Paris (ville)"@fr .

# Unlabeled type: rendered by IRI local name.
ex:dog rdfs:label "Rex" ; a ex:GoodBoy .

# Type only, no literal text anywhere: must NOT verbalize (a bare "a athlete"
# passage matches every athlete and nothing else).
ex:silent a ex:Athlete .

# Plain literal ("" in the chain) preferred over an unlisted language.
ex:mixed rdfs:label "plain label", "deutsches Label"@de .
"#;

fn term(iri: &str) -> Term {
    Term::NamedNode(NamedNode::new(iri).unwrap())
}

fn ex(local: &str) -> Term {
    term(&format!("http://example.org/{local}"))
}

#[test]
fn default_template_label_type_description() {
    let g = Graph::load_str(TTL, "turtle").unwrap();
    let cfg = EntityTextConfig::default();
    assert_eq!(
        verbalize(&g, &ex("bolt"), &cfg).unwrap(),
        "Usain Bolt. a athlete. Jamaican sprinter, eight-time Olympic champion.",
        "label by language chain, type by its own en label, description appended"
    );
}

#[test]
fn language_preference_and_fallback() {
    let g = Graph::load_str(TTL, "turtle").unwrap();

    // French-first chain flips both the entity label and the type's label.
    let fr = EntityTextConfig {
        languages: vec!["fr".into(), "en".into(), String::new()],
        ..Default::default()
    };
    assert_eq!(
        verbalize(&g, &ex("bolt"), &fr).unwrap(),
        "Usain Bolt (fr). a athlète. Jamaican sprinter, eight-time Olympic champion.",
        "fr label and fr type word win; the en-only description still falls back"
    );

    // A graph labeled only in an unlisted language still verbalizes (last-resort rank).
    let en = EntityTextConfig::default();
    assert_eq!(verbalize(&g, &ex("paris"), &en).unwrap(), "Paris (ville)");

    // "" in the chain means plain literals beat unlisted languages.
    assert_eq!(verbalize(&g, &ex("mixed"), &en).unwrap(), "plain label");
}

#[test]
fn predicate_priority_within_a_group() {
    let g = Graph::load_str(TTL, "turtle").unwrap();
    let cfg = EntityTextConfig::default();
    assert_eq!(verbalize(&g, &ex("both"), &cfg).unwrap(), "From rdfs label");
}

#[test]
fn unlabeled_type_renders_as_local_name() {
    let g = Graph::load_str(TTL, "turtle").unwrap();
    let cfg = EntityTextConfig::default();
    assert_eq!(verbalize(&g, &ex("dog"), &cfg).unwrap(), "Rex. a GoodBoy");
}

#[test]
fn type_only_entities_are_skipped() {
    let g = Graph::load_str(TTL, "turtle").unwrap();
    let cfg = EntityTextConfig::default();
    assert_eq!(verbalize(&g, &ex("silent"), &cfg), None);
    assert_eq!(
        verbalize(&g, &ex("nowhere"), &cfg),
        None,
        "unknown term is None, not panic"
    );
}

#[test]
fn extra_prefixed_literal_group_with_value_cap() {
    let g = Graph::load_str(TTL, "turtle").unwrap();
    let mut cfg = EntityTextConfig::default();
    cfg.groups.push(
        PropertyGroup::literal(vec![
            NamedNode::new("http://example.org/occupation").unwrap()
        ])
        .with_prefix("occupation: ")
        .with_max_values(2),
    );
    let text = verbalize(&g, &ex("bolt"), &cfg).unwrap();
    assert!(
        text.ends_with("occupation: philanthropist, sprinter")
            || text.ends_with("occupation: sprinter, philanthropist"),
        "both occupation values joined under the prefix (order = scan order): {text}"
    );

    // max_values = 1 keeps only the first value.
    cfg.groups.last_mut().unwrap().max_values = 1;
    let one = verbalize(&g, &ex("bolt"), &cfg).unwrap();
    let occ = one.rsplit(". ").next().unwrap();
    assert!(occ == "occupation: sprinter" || occ == "occupation: philanthropist");
    assert!(!occ.contains(','), "value cap respected: {one}");
}

#[test]
fn char_budget_truncates_but_keeps_the_label() {
    let g = Graph::load_str(TTL, "turtle").unwrap();

    // Whole-piece fit: label + type fit, description does not start.
    let mut cfg = EntityTextConfig {
        max_chars: "Usain Bolt. a athlete".chars().count(),
        ..Default::default()
    };
    assert_eq!(
        verbalize(&g, &ex("bolt"), &cfg).unwrap(),
        "Usain Bolt. a athlete"
    );

    // Mid-piece overflow: the overflowing piece is truncated at a char boundary.
    cfg.max_chars = 8;
    assert_eq!(verbalize(&g, &ex("bolt"), &cfg).unwrap(), "Usain Bo");
}

#[test]
fn entity_label_groups_only_enrich() {
    // A config whose ONLY group is the type (EntityLabel) must verbalize nothing:
    // Some requires at least one Literal-group contribution.
    let g = Graph::load_str(TTL, "turtle").unwrap();
    let cfg = EntityTextConfig {
        groups: vec![PropertyGroup::entity_label(vec![NamedNode::new(
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
        )
        .unwrap()])
        .with_prefix("a ")],
        ..Default::default()
    };
    assert_eq!(cfg.groups[0].kind, ObjectKind::EntityLabel);
    assert_eq!(verbalize(&g, &ex("bolt"), &cfg), None);
}

#[test]
fn embed_entities_covers_verbalizable_entities_and_is_deterministic() {
    let g = Graph::load_str(TTL, "turtle").unwrap();
    let cfg = EntityTextConfig::default();
    let embedder = HashEmbedder::new(32);
    let path = std::env::temp_dir().join(format!(
        "sparq-vectors-verbalize-{}.spqv",
        std::process::id()
    ));

    let mut store = VectorStore::create(&path, 32).unwrap();
    let n = embed_entities(&g, &mut store, &embedder, &cfg).unwrap();
    // bolt, Athlete (has its own rdfs:label), both, paris, dog, mixed — not silent.
    assert_eq!(n, 6);
    store.finalize().unwrap();

    let id = |t: &Term| g.id_of(t).unwrap();
    assert!(
        store.get(id(&ex("silent"))).is_none(),
        "type-only entity not embedded"
    );

    // Each stored vector is exactly the embedding of the entity's verbalization —
    // verbalize() is the inspectable contract for what got embedded.
    for who in ["bolt", "both", "paris", "dog", "mixed"] {
        let t = ex(who);
        let text = verbalize(&g, &t, &cfg).unwrap();
        let expect = &embedder.embed(&[&text]).unwrap()[0];
        assert_eq!(
            store.get(id(&t)).unwrap(),
            &expect[..],
            "vector for {who} = embed(verbalize)"
        );
    }

    // Determinism: a second run (batch=1 to exercise chunking) produces identical bytes.
    let path2 = std::env::temp_dir().join(format!(
        "sparq-vectors-verbalize2-{}.spqv",
        std::process::id()
    ));
    let mut store2 = VectorStore::create(&path2, 32).unwrap();
    let mut cfg2 = cfg.clone();
    cfg2.batch = 1;
    assert_eq!(
        embed_entities(&g, &mut store2, &embedder, &cfg2).unwrap(),
        6
    );
    store2.finalize().unwrap();
    for who in ["bolt", "both", "paris", "dog", "mixed"] {
        let t = ex(who);
        assert_eq!(store.get(id(&t)), store2.get(id(&t)));
    }

    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(&path2);
}

#[test]
fn embed_entities_rejects_dim_mismatch() {
    let g = Graph::load_str(TTL, "turtle").unwrap();
    let path = std::env::temp_dir().join(format!(
        "sparq-vectors-verbalize3-{}.spqv",
        std::process::id()
    ));
    let mut store = VectorStore::create(&path, 8).unwrap();
    let err = embed_entities(
        &g,
        &mut store,
        &HashEmbedder::new(16),
        &EntityTextConfig::default(),
    )
    .unwrap_err();
    assert!(err.contains("dim"), "{err}");
    let _ = std::fs::remove_file(&path);
}
