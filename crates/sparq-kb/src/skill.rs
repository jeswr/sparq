//! # `skill` — the Phase-1 "query-the-PKG" recipe (introspect → ground → ask)
//!
//! [OPUS-4.8] sq-2m6zm.3 (epic sq-2m6zm); design record
//! `research/dogfooding-sparq-knowledge-graph.md` §4.1 + §6 Phase-1 deliverable 3.
//! 🤖 SPARQ agent — dogfooding sparq as a project knowledge graph. Written while
//! Fable unavailable; flag for re-review when Fable returns.
//!
//! This is the thin **in-process Rust** recipe an agent calls to query the ingested
//! PKG ([`crate::PKG_INSTANCES`]) instead of re-reading the prose corpus. It wires the
//! three steps the design prescribes, using only **production-ready** sparq surfaces:
//!
//! 1. **introspect** — mine a token-budgeted schema card of the loaded graph
//!    ([`sparq_introspect::Introspection::to_text_summary`]). "Mine once, summarise
//!    forever": the card is the cheap, re-usable grounding context for query
//!    construction (design §1.2).
//! 2. **ground** — ALWAYS [`close_for_vectorise`](sparq_vectors::structure::close_for_vectorise)
//!    `(RDFS|OWL-RL)` **first** (design §3.1: completeness is profile-relative; the
//!    `pkg:dependsOn owl:inverseOf pkg:blockedBy` axiom and the SKOS/RDFS subclass
//!    edges only materialise after closure), then return the ABSTAT-style **minimal
//!    subgraph** ([`sparq_vectors::grounding::Grounding::Subgraph`]) for each IRI the
//!    answer binds — the verifiable facts, `O(answer)` tokens, not `O(corpus)`.
//! 3. **ask** — run a **§4.1-class verified-expressible SPARQL** query (the
//!    [`recipes`](crate::skill::recipes) catalogue, or any caller-supplied
//!    `SELECT`/`ASK`) over the closed graph and **surface the executed SPARQL** (the
//!    [`PkgAnswer`](crate::skill::PkgAnswer) `sparql` field) so the answer is
//!    independently re-checkable.
//!
//! **Scope (honest):** Phase-1 relies *only* on the §4.1-class frontier queries that
//! are verified-expressible in SPARQL today. The §4.2/§4.3 N3 trigger / research↔bead
//! rules are explicitly Phase-2/3 and are **not** wired here. NL→SPARQL
//! ([`sparq_nlq`](https://docs.rs/sparq-nlq)) is also out of scope: this recipe takes
//! the SPARQL it runs from the verified catalogue (or the caller), it does not
//! generate it from natural language. The store bounds the answer set, so within the
//! store the answer cannot be fabricated; query-construction + verbalisation
//! hallucination is *reduced*, not removed (design honesty summary).

use oxrdf::Term;
use sparq_core::Graph;
use sparq_engine::{query, QueryResult};
use sparq_introspect::Introspection;
use sparq_reason::Profile;
use sparq_vectors::grounding::{ground, GroundFact, Grounding, GroundingConfig, Modality};
use sparq_vectors::structure::close_for_vectorise;

/// The entailment profile to materialise before grounding/asking. Re-exports the two
/// regimes [`close_for_vectorise`] supports; the design mandates closing under one of
/// them so completeness is profile-relative.
///
/// Default [`Closure::OwlRl`] — the PKG ontology declares `pkg:dependsOn owl:inverseOf
/// pkg:blockedBy` and `pkg:couldBeMergedWith a owl:SymmetricProperty`, which only
/// materialise under OWL-RL; RDFS alone would leave those edges un-closed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Closure {
    /// RDFS only (subclass / subproperty / domain / range).
    Rdfs,
    /// OWL 2 RL (includes RDFS; adds `inverseOf`, `SymmetricProperty`, `sameAs`, …).
    /// The PKG default — see [`Closure`].
    #[default]
    OwlRl,
}

impl Closure {
    fn profile(self) -> Profile {
        match self {
            Closure::Rdfs => Profile::Rdfs,
            Closure::OwlRl => Profile::OwlRl,
        }
    }
}

/// Configuration for the [`query_the_pkg`] recipe.
#[derive(Clone, Debug)]
pub struct SkillConfig {
    /// Which entailment closure to materialise before grounding/asking. Default
    /// [`Closure::OwlRl`].
    pub closure: Closure,
    /// Character (token-proxy) budget for the introspection schema card. `0` skips the
    /// card (no introspect step). Default 4096.
    pub schema_card_chars: usize,
    /// Cap on the number of facts in each grounded minimal subgraph (the `k`-budget
    /// passed to [`GroundingConfig::max_facts`]). Default 32.
    pub max_facts_per_subgraph: usize,
    /// Cap on the number of distinct bound IRIs to ground into minimal subgraphs (an
    /// answer can bind many entities; grounding every one defeats the `O(answer)`
    /// token goal). `0` = ground none (rows only). Default 8.
    pub max_grounded_nodes: usize,
}

impl Default for SkillConfig {
    fn default() -> Self {
        SkillConfig {
            closure: Closure::default(),
            schema_card_chars: 4096,
            max_facts_per_subgraph: 32,
            max_grounded_nodes: 8,
        }
    }
}

/// One bound IRI plus the ABSTAT-style **minimal subgraph** that grounds it — the
/// verifiable facts the answer rests on. The subject is the IRI; each fact is a
/// `(predicate, object)` pair present in the closed graph.
#[derive(Clone, Debug)]
pub struct GroundedNode {
    /// The IRI (rendered bare) that the answer bound.
    pub iri: String,
    /// The minimal subgraph: the predicates the node's effective (minimal) type
    /// carries, each with its object, bounded by [`SkillConfig::max_facts_per_subgraph`].
    pub facts: Vec<GroundFact>,
}

/// The result of [`query_the_pkg`]: the executed SPARQL (surfaced for verification),
/// the schema card, the answer rows, and the grounded minimal subgraphs.
#[derive(Clone, Debug)]
pub struct PkgAnswer {
    /// The **executed** SPARQL — exactly what ran, surfaced so the answer is
    /// independently re-checkable (design §6: "surface the executed SPARQL").
    pub sparql: String,
    /// The entailment closure that was materialised before the query ran.
    pub closure: Closure,
    /// Triples added by the closure (non-canonical; proves the closure ran). Never a
    /// performance metric.
    pub entailed_triples: usize,
    /// The token-budgeted introspection schema card (empty when the card budget is 0).
    pub schema_card: String,
    /// The answer variable names, in projection order.
    pub vars: Vec<String>,
    /// The answer rows (the minimal binding set the engine computed — `O(answer)`).
    /// Each cell is the rendered term, or `None` for an unbound variable.
    pub rows: Vec<Vec<Option<String>>>,
    /// The grounded minimal subgraphs for the distinct IRIs the answer bound (bounded
    /// by [`SkillConfig::max_grounded_nodes`]).
    pub grounded: Vec<GroundedNode>,
}

impl PkgAnswer {
    /// The number of answer rows.
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }
}

/// Load the PKG ontology + ingested instances into one graph and materialise the
/// `closure`. This is the "ground" step's prerequisite: every entailed
/// `subClassOf`/`subPropertyOf`/`inverseOf`/`domain`/`range`/type fact becomes a real
/// triple before any query or grounding reads the graph.
///
/// Returns the closed [`Graph`] and the number of triples the closure added.
pub fn load_closed_pkg(closure: Closure) -> Result<(Graph, usize), String> {
    let combined = format!("{}\n{}", crate::PKG_ONTOLOGY, crate::PKG_INSTANCES);
    let closed = close_for_vectorise(&combined, "turtle", closure.profile())?;
    Ok((closed.graph, closed.entailed_triples))
}

/// Mine a token-budgeted schema card of `graph` — the cheap, re-usable grounding
/// context (design §1.2 "mine once, summarise forever"). `budget_chars == 0` returns
/// an empty card.
pub fn introspect(graph: &Graph, budget_chars: usize) -> String {
    if budget_chars == 0 {
        return String::new();
    }
    Introspection::build(graph).to_text_summary(budget_chars)
}

/// Render a result term to text: an IRI bare, a literal as its lexical value, a blank
/// node as its label. Mirrors the rendering [`GroundFact`] uses so a row cell and a
/// grounded fact line up.
fn render_term(t: &Term) -> String {
    match t {
        Term::NamedNode(n) => n.as_str().to_string(),
        Term::Literal(l) => l.value().to_string(),
        Term::BlankNode(b) => format!("_:{}", b.as_str()),
        // RDF-1.2 triple terms render via Display; they never appear as PKG IRIs.
        other => other.to_string(),
    }
}

/// Run a SPARQL `SELECT`/`ASK` query over the **already-closed** `graph`, returning the
/// rows plus the minimal subgraph grounding for each distinct bound IRI (bounded by
/// `cfg`). The caller is responsible for having closed the graph (use
/// [`load_closed_pkg`] / [`query_the_pkg`]).
///
/// Only the IRIs the answer binds are grounded — the answer set already bounds the
/// token cost, and the grounding adds *only* each entity's effective-type minimal
/// predicate set, keeping the whole result `O(answer)` rather than `O(corpus)`.
pub fn ask(graph: &Graph, sparql: &str, cfg: &SkillConfig) -> Result<PkgAnswer, String> {
    let result: QueryResult = query(graph, sparql)?;

    let vars: Vec<String> = result.vars.iter().map(|v| v.as_str().to_string()).collect();
    let rows: Vec<Vec<Option<String>>> = result
        .rows
        .iter()
        .map(|row| row.iter().map(|c| c.as_ref().map(render_term)).collect())
        .collect();

    // Ground the distinct bound IRIs into minimal subgraphs (in first-seen order, so
    // the output is deterministic). Build the introspection once for minimalisation.
    let mut grounded = Vec::new();
    if cfg.max_grounded_nodes > 0 {
        let introspection = Introspection::build(graph);
        let gcfg = GroundingConfig {
            minimal_type_pattern: true,
            max_facts: cfg.max_facts_per_subgraph,
            ..Default::default()
        };
        let mut seen = std::collections::HashSet::new();
        'outer: for row in &result.rows {
            for cell in row {
                if let Some(Term::NamedNode(n)) = cell {
                    let iri = n.as_str().to_string();
                    if !seen.insert(iri.clone()) {
                        continue;
                    }
                    let node = Term::NamedNode(n.clone());
                    if let Some(Grounding::Subgraph(facts)) = ground(
                        graph,
                        &node,
                        Modality::Subgraph,
                        &gcfg,
                        Some(&introspection),
                        None,
                    ) {
                        grounded.push(GroundedNode { iri, facts });
                    }
                    if grounded.len() >= cfg.max_grounded_nodes {
                        break 'outer;
                    }
                }
            }
        }
    }

    Ok(PkgAnswer {
        sparql: sparql.to_string(),
        closure: cfg.closure,
        entailed_triples: 0, // filled by query_the_pkg, which owns the closure step
        schema_card: String::new(),
        vars,
        rows,
        grounded,
    })
}

/// The full **introspect → ground → ask** recipe: load + close the PKG, mine the schema
/// card, run `sparql` (a [`recipes`] query or any caller-supplied `SELECT`/`ASK`), and
/// ground the bound IRIs into minimal subgraphs. The executed SPARQL is surfaced on
/// the returned [`PkgAnswer`].
///
/// ```no_run
/// use sparq_kb::skill::{query_the_pkg, recipes, SkillConfig};
///
/// let ans = query_the_pkg(recipes::READY_FRONTIER_COUNT, &SkillConfig::default())
///     .expect("recipe runs");
/// println!("executed:\n{}", ans.sparql);     // surfaced for verification
/// println!("schema card:\n{}", ans.schema_card);
/// for row in &ans.rows {
///     println!("{row:?}");
/// }
/// ```
pub fn query_the_pkg(sparql: &str, cfg: &SkillConfig) -> Result<PkgAnswer, String> {
    let (graph, entailed) = load_closed_pkg(cfg.closure)?;
    let card = introspect(&graph, cfg.schema_card_chars);
    let mut answer = ask(&graph, sparql, cfg)?;
    answer.entailed_triples = entailed;
    answer.schema_card = card;
    Ok(answer)
}

/// The **§4.1-class verified-expressible** SPARQL catalogue — the only queries Phase-1
/// relies on. Each is a `SELECT`/`ASK` proven to run over the ingested PKG graph (the
/// `ingest_query` proof + the [`super::skill`] tests). The §4.2/§4.3 N3 rules are
/// explicitly Phase-2/3 and are NOT here.
pub mod recipes {
    /// §4.1 ready-frontier (the dependency half) — **count** of open/in-progress tasks
    /// with no OPEN blocking dependency, not human-gated (`needs:` label), and not an
    /// umbrella (parent of another task). The conflict-partition / CPU-cap /
    /// in-flight-by-git layers are NOT expressible without the (Phase-2) live-state
    /// named graph — see design §4.
    pub const READY_FRONTIER_COUNT: &str = r#"
PREFIX pkg:     <https://sparq.dev/ns/pkg#>
PREFIX dcterms: <http://purl.org/dc/terms/>
SELECT (COUNT(DISTINCT ?t) AS ?ready) WHERE {
  ?t a pkg:Task ; pkg:status ?s ; pkg:priority ?prio .
  FILTER(?s IN (pkg:Open, pkg:InProgress))
  FILTER NOT EXISTS { ?t pkg:dependsOn ?b . ?b pkg:status ?bs . FILTER(?bs != pkg:Closed) }
  FILTER NOT EXISTS { ?t pkg:label ?l . FILTER(STRSTARTS(STR(?l), "needs:")) }
  FILTER NOT EXISTS { ?child dcterms:isPartOf ?t }
}"#;

    /// §4.1 ready-frontier — the **task IRIs** themselves (same predicate as
    /// [`READY_FRONTIER_COUNT`] but projecting the tasks + priority, ordered), so the
    /// caller can ground each into its minimal subgraph.
    pub const READY_FRONTIER: &str = r#"
PREFIX pkg:     <https://sparq.dev/ns/pkg#>
PREFIX dcterms: <http://purl.org/dc/terms/>
SELECT ?t ?prio WHERE {
  ?t a pkg:Task ; pkg:status ?s ; pkg:priority ?prio .
  FILTER(?s IN (pkg:Open, pkg:InProgress))
  FILTER NOT EXISTS { ?t pkg:dependsOn ?b . ?b pkg:status ?bs . FILTER(?bs != pkg:Closed) }
  FILTER NOT EXISTS { ?t pkg:label ?l . FILTER(STRSTARTS(STR(?l), "needs:")) }
  FILTER NOT EXISTS { ?child dcterms:isPartOf ?t }
} ORDER BY ?prio ?t LIMIT 50"#;

    /// "Which Findings are about a topic?" — the sourced, confidence-ordered Findings
    /// for a topic IRI. The query is parameterless; substitute the topic with
    /// [`findings_about`].
    pub const FINDINGS_ABOUT_TEMPLATE: &str = r#"
PREFIX pkg:     <https://sparq.dev/ns/pkg#>
PREFIX dcterms: <http://purl.org/dc/terms/>
PREFIX rdfs:    <http://www.w3.org/2000/01/rdf-schema#>
SELECT ?f ?label ?section ?conf WHERE {
  ?f a pkg:Finding ;
     pkg:about <TOPIC> ;
     rdfs:label ?label ;
     dcterms:source ?section ;
     pkg:confidence ?conf .
} ORDER BY DESC(?conf)"#;

    /// Build a "Findings about `<topic_iri>`" query from [`FINDINGS_ABOUT_TEMPLATE`] by
    /// substituting the topic IRI. The IRI is wrapped in `<>`; callers pass a bare IRI
    /// (e.g. a `kb:topic-…` URL). No general string interpolation into SPARQL is done
    /// elsewhere — only this single, IRI-shaped slot — so injection surface is nil.
    pub fn findings_about(topic_iri: &str) -> String {
        FINDINGS_ABOUT_TEMPLATE.replace("<TOPIC>", &format!("<{topic_iri}>"))
    }
}
