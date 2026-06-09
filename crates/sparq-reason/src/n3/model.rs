//! N3 term model — richer than RDF (adds variables and formulae/graph-terms).

/// An N3 term. Beyond RDF's IRI/literal/blank it has `Var` (universally-quantified `?x`) and
/// `Formula` (a `{ … }` graph term, used as rule premise/conclusion).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Term {
    Iri(String),
    /// lexical value, datatype IRI, optional language tag.
    Lit(String, String, Option<String>),
    Blank(String),
    Var(String),
    Formula(Vec<[Term; 3]>),
}

impl Term {
    pub fn is_ground(&self) -> bool {
        !matches!(self, Term::Var(_)) && !matches!(self, Term::Formula(_))
    }
}

/// A `{ premise } => { conclusion }` rule (log:implies).
#[derive(Clone, Debug)]
pub struct Rule {
    pub premise: Vec<[Term; 3]>,
    pub conclusion: Vec<[Term; 3]>,
}
