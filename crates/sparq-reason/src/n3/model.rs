//! N3 term model — richer than RDF (adds variables and formulae/graph-terms).

/// An N3 term. Beyond RDF's IRI/literal/blank it has `Var` (universally-quantified `?x`),
/// `Formula` (a `{ … }` graph term, used as rule premise/conclusion), and `List`
/// (a first-class `( … )` collection value — N3 lists are terms, not rdf:first/rest
/// triple structure; `rdf:nil` is the empty list).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Term {
    Iri(String),
    /// lexical value, datatype IRI, optional language tag.
    Lit(String, String, Option<String>),
    Blank(String),
    Var(String),
    Formula(Vec<[Term; 3]>),
    List(Vec<Term>),
}

impl Term {
    pub fn is_ground(&self) -> bool {
        match self {
            Term::Var(_) => false,
            // A quoted formula is a VALUE — ground when its contents are
            // (so rules may conclude formula-valued facts: log:conjunction,
            // log:conclusion, log:parsedAsN3 results).
            Term::Formula(ts) => ts.iter().all(|r| r.iter().all(Term::is_ground)),
            Term::List(ms) => ms.iter().all(Term::is_ground),
            _ => true,
        }
    }
}

/// A `{ premise } => { conclusion }` rule (log:implies).
#[derive(Clone, Debug)]
pub struct Rule {
    pub premise: Vec<[Term; 3]>,
    pub conclusion: Vec<[Term; 3]>,
}
