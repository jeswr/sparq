//! SHACL-SPARQL (`sh:sparql`, W3C SHACL §5.2): SPARQL-based constraints.
//!
//! A `sh:sparql` constraint carries an `sh:select` query that is run against the
//! data graph once per focus node; **each returned solution is a violation**. The
//! focus node is supplied to the query by *pre-binding* the variable `$this`
//! (SHACL §5.2.1) — done here at the algebra level by injecting a `VALUES (?this)
//! { (<focus>) }` table into the parsed query's WHERE clause, which is robust to
//! however the author laid the query text out (a textual prepend is not).
//!
//! `sh:prefixes` (via `sh:declare` / `sh:prefix` / `sh:namespace`) supply the
//! query's prefix declarations; they are pre-pended to the `sh:select` text
//! before parsing. Per the spec a `sh:sparql` constraint MUST carry a `sh:select`
//! whose result variables include neither a reserved nor an undeclared-prefix
//! token — anything that fails to parse is treated as an ill-formed constraint
//! and skipped (consistent with this crate's lenient handling of ill-formed
//! shapes; see TODO.md).
//!
//! Solution → validation-result mapping (SHACL §5.2.2):
//!   * `sh:focusNode`   ← the focus node ($this);
//!   * `sh:value`       ← the `?value` binding, if the solution binds it;
//!   * `sh:resultPath`  ← the `?path` binding when it is an IRI (a predicate path);
//!   * `sh:resultMessage` ← the constraint's `sh:message` (with `{?var}`
//!     substitution from the solution), else a `?message` binding, else a default.
//!
//! `$shapesGraph` / `$currentShape` pre-binding: scoped honestly. `$currentShape`
//! is bound to the source shape's node (available and cheap). `$shapesGraph` is
//! NOT bound — this validator evaluates the `sh:select` against the DATA graph
//! only (it has no named-graph handle to the shapes graph at query time), so a
//! query that dereferences `$shapesGraph` will simply find no solutions for it.
//! See the crate TODO for the deferred full SPARQL-based-constraint-component
//! machinery.

use crate::model::SparqlConstraint;
use crate::report::ValidationResult;
use oxrdf::{Term, Variable};
use spargebra::algebra::GraphPattern;
use spargebra::term::GroundTerm;
use spargebra::{Query, SparqlParser};

/// A parsed-and-validated `sh:sparql` constraint, ready to run per focus node.
/// Built once when the shapes model is parsed (so a malformed query is rejected
/// up front, not per focus node).
#[derive(Debug, Clone)]
pub(crate) struct PreparedSparql {
    /// The parsed `sh:select` algebra (prefixes already resolved at parse time).
    query: Query,
    /// All projected variable names, for `{?var}` message substitution and for
    /// validating up front that the query has an explicit projection.
    vars: Vec<String>,
}

impl PreparedSparql {
    /// Parses `constraint.select` (with its prefixes prepended) into algebra,
    /// returning `None` for a non-SELECT or unparsable query (ill-formed → skip).
    pub(crate) fn build(constraint: &SparqlConstraint) -> Option<PreparedSparql> {
        let text = format!("{}\n{}", constraint.prefixes, constraint.select);
        let query = SparqlParser::new().parse_query(&text).ok()?;
        // Only SELECT is meaningful for sh:sparql; the projection drives the result count.
        let Query::Select { pattern, .. } = &query else {
            return None;
        };
        let vars: Vec<String> = projected_vars(pattern)?
            .iter()
            .map(|v| v.as_str().to_string())
            .collect();
        Some(PreparedSparql { query, vars })
    }

    /// Runs the constraint for one `focus` node against `data`, pushing one
    /// [`ValidationResult`] per solution. `make_result` lets the caller stamp the
    /// shape-owned fields (source shape/component, severity, shape messages).
    pub(crate) fn evaluate(
        &self,
        data: &sparq_core::Graph,
        focus: &Term,
        constraint: &SparqlConstraint,
        mut make_result: impl FnMut(ResultFields) -> ValidationResult,
        out: &mut Vec<ValidationResult>,
    ) {
        let Some(bound) = pre_bind(&self.query, focus) else {
            return; // focus node not expressible as a VALUES ground term (unreachable today)
        };
        let prepared = sparq_engine::PreparedQuery::from(bound);
        let Ok(result) = sparq_engine::query_prepared(data, &prepared) else {
            return; // a runtime query error → no solutions (lenient; never panics validate())
        };
        // Map QueryResult var positions to our cached projection indices: the
        // injected query keeps the same projected variables (we add ?this to the
        // projection if it was absent), so look variables up by name to be safe.
        let pos = |name: &str| result.vars.iter().position(|v| v.as_str() == name);
        let (vpos, ppos, mpos) = (pos("value"), pos("path"), pos("message"));

        for row in &result.rows {
            let value = vpos.and_then(|i| row.get(i)).and_then(|c| c.clone());
            let path = ppos
                .and_then(|i| row.get(i))
                .and_then(|c| c.clone())
                .and_then(|t| match t {
                    Term::NamedNode(n) => Some(crate::path::Path::Predicate(n.as_str().to_string())),
                    _ => None,
                });
            // Message precedence: the constraint's sh:message (with {?var}
            // substitution), else a bound ?message, else a generated default.
            let row_message = mpos
                .and_then(|i| row.get(i))
                .and_then(|c| c.clone())
                .map(term_lexical);
            let default_message = match (&constraint.message, &row_message) {
                (Some(tpl), _) => substitute(tpl, &self.vars, row),
                (None, Some(m)) => m.clone(),
                (None, None) => "Violates SPARQL-based constraint".to_string(),
            };
            out.push(make_result(ResultFields {
                value,
                path,
                default_message,
            }));
        }
    }
}

/// The per-solution fields the SPARQL evaluation contributes; the caller fills in
/// the shape-owned fields (focus node, source shape/component, severity, messages).
pub(crate) struct ResultFields {
    pub value: Option<Term>,
    pub path: Option<crate::path::Path>,
    pub default_message: String,
}

/// The projected variables of a SELECT pattern (the outermost `Project`), or the
/// variables mentioned when there is no explicit projection (`SELECT *`).
fn projected_vars(pattern: &GraphPattern) -> Option<&Vec<Variable>> {
    match pattern {
        GraphPattern::Project { variables, .. } => Some(variables),
        _ => None,
    }
}

/// Pre-binds `$this` to `focus` by injecting a single-row `VALUES (?this) { (focus) }`
/// table joined under the projection, and ensures `?this` is itself projected (so
/// a consumer reading `?this` back gets the focus node). Returns `None` if `focus`
/// is not expressible as a SPARQL ground term (blank nodes are allowed).
fn pre_bind(query: &Query, focus: &Term) -> Option<Query> {
    let Query::Select {
        dataset,
        pattern,
        base_iri,
    } = query
    else {
        return None;
    };
    let ground = term_to_ground(focus)?;
    let this = Variable::new_unchecked("this");
    let values = GraphPattern::Values {
        variables: vec![this.clone()],
        bindings: vec![vec![Some(ground)]],
    };
    // Wrap the *inner* of the projection with a Join against the VALUES table so
    // the binding is visible to the whole WHERE clause; keep the projection (and
    // add ?this to it if the author did not already project it).
    let new_pattern = match pattern {
        GraphPattern::Project { inner, variables } => {
            let mut variables = variables.clone();
            if !variables.iter().any(|v| v.as_str() == "this") {
                variables.push(this);
            }
            GraphPattern::Project {
                inner: Box::new(GraphPattern::Join {
                    left: Box::new(values),
                    right: inner.clone(),
                }),
                variables,
            }
        }
        // No explicit projection (SELECT *): join at the top and project nothing
        // extra — `?this` is in scope and surfaces in the result vars regardless.
        other => GraphPattern::Join {
            left: Box::new(values),
            right: Box::new(other.clone()),
        },
    };
    Some(Query::Select {
        dataset: dataset.clone(),
        pattern: new_pattern,
        base_iri: base_iri.clone(),
    })
}

/// An RDF term as a SPARQL `GroundTerm` for a VALUES row. Variables/triple-terms
/// that cannot appear in VALUES yield `None` (focus nodes are IRIs/blank nodes in
/// practice, both of which are expressible).
fn term_to_ground(t: &Term) -> Option<GroundTerm> {
    match t {
        Term::NamedNode(n) => Some(GroundTerm::NamedNode(n.clone())),
        Term::Literal(l) => Some(GroundTerm::Literal(l.clone())),
        // A blank node cannot be written in a VALUES table (no syntax); SHACL focus
        // nodes are rarely blank, and when they are this constraint is skipped
        // rather than mis-evaluated. (oxrdf BlankNode has no GroundTerm form.)
        Term::BlankNode(_) => None,
        #[allow(unreachable_patterns)]
        _ => None,
    }
}

/// The lexical string a bound term contributes to a `?message` / `{?var}` slot.
fn term_lexical(t: Term) -> String {
    match t {
        Term::NamedNode(n) => n.into_string(),
        Term::Literal(l) => l.value().to_string(),
        Term::BlankNode(b) => format!("_:{}", b.as_str()),
        #[allow(unreachable_patterns)]
        other => other.to_string(),
    }
}

/// Substitutes `{?var}` / `{$var}` placeholders in a SHACL message template with
/// the solution's bindings (SHACL §5.2.2 message templates). Unbound or unknown
/// variables are left as the literal placeholder text.
fn substitute(template: &str, vars: &[String], row: &[Option<Term>]) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        if let Some(close) = after.find('}') {
            let token = after[..close].trim();
            let name = token.strip_prefix(['?', '$']);
            let replaced = name.and_then(|n| {
                vars.iter()
                    .position(|v| v == n)
                    .and_then(|i| row.get(i))
                    .and_then(|c| c.clone())
                    .map(term_lexical)
            });
            match replaced {
                Some(s) => out.push_str(&s),
                None => {
                    // Not a known {?var}: keep it verbatim.
                    out.push('{');
                    out.push_str(&after[..close]);
                    out.push('}');
                }
            }
            rest = &after[close + 1..];
        } else {
            out.push('{');
            rest = after;
        }
    }
    out.push_str(rest);
    out
}
