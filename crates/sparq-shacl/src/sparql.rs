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

/// One `($name, value)` pre-binding for a validator/constraint query: the value
/// is injected as a single-row `VALUES (?name) { (value) }` table. SHACL §6.3
/// pre-binds `$this`, `$value`, each shape parameter (`$paramName`) and (on a
/// property shape) `$PATH` this way.
pub(crate) type Binding<'a> = (&'a str, &'a Term);

/// A parsed-and-validated `sh:sparql` constraint, ready to run per focus node.
/// Built once when the shapes model is parsed (so a malformed query is rejected
/// up front, not per focus node).
#[derive(Debug, Clone)]
pub(crate) struct PreparedSparql {
    /// The parsed `sh:select` algebra (prefixes already resolved at parse time).
    query: Query,
}

impl PreparedSparql {
    /// Parses `constraint.select` (with its prefixes prepended) into algebra,
    /// returning `None` for a non-SELECT or unparsable query (ill-formed → skip).
    /// `SELECT *` is accepted (every in-scope variable, including `$this`, is a
    /// result variable); only a non-SELECT query form is rejected.
    pub(crate) fn build(constraint: &SparqlConstraint) -> Option<PreparedSparql> {
        let text = format!("{}\n{}", constraint.prefixes, constraint.select);
        let query = SparqlParser::new().parse_query(&text).ok()?;
        // Only SELECT is meaningful for sh:sparql; the solutions drive the result count.
        if !matches!(query, Query::Select { .. }) {
            return None;
        }
        Some(PreparedSparql { query })
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
        let Some(bound) = pre_bind_select(&self.query, &[("this", focus)]) else {
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
        // Message {?var} substitution indexes the EXECUTED result's variables (the
        // injected query appends ?this to the projection, so positions can differ
        // from the source query's projected `self.vars`).
        let result_vars: Vec<String> = result.vars.iter().map(|v| v.as_str().to_string()).collect();

        for row in &result.rows {
            // sh:value (SHACL §5.2.2): the solution's ?value binding when the query
            // projects ?value; otherwise the focus node itself. The focus-node
            // default (when ?value is not projected at all) is what the W3C suite
            // expects — e.g. sparql/node/sparql-003 projects only ?path and its
            // expected sh:value is the focus node. A query that DOES project ?value
            // but leaves it unbound on a row reports no sh:value (None).
            let value = match vpos {
                Some(i) => row.get(i).and_then(|c| c.clone()),
                None => Some(focus.clone()),
            };
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
                (Some(tpl), _) => substitute(tpl, &result_vars, row),
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

/// Builds the single-row `VALUES (?n1 ?n2 …) { (v1 v2 …) }` table that pre-binds
/// each `(name, term)` in `bindings`. Returns `None` if any value is not
/// expressible as a SPARQL ground term (e.g. a blank node — see `term_to_ground`).
fn pre_bind_values(bindings: &[Binding]) -> Option<GraphPattern> {
    let mut variables = Vec::with_capacity(bindings.len());
    let mut row = Vec::with_capacity(bindings.len());
    for (name, term) in bindings {
        variables.push(Variable::new_unchecked(*name));
        row.push(Some(term_to_ground(term)?));
    }
    Some(GraphPattern::Values {
        variables,
        bindings: vec![row],
    })
}

/// Pre-binds each `(name, value)` in `bindings` into a SELECT query by injecting a
/// single-row `VALUES` table joined under the projection, and ensures every bound
/// name is itself projected (so a consumer reading `?this` / `?value` back gets
/// the pre-bound term). Returns `None` for a non-SELECT query or an inexpressible
/// value. SHACL §5.2 (`$this`) and §6.3 (`$this`/`$value`/`$paramName`).
fn pre_bind_select(query: &Query, bindings: &[Binding]) -> Option<Query> {
    let Query::Select {
        dataset,
        pattern,
        base_iri,
    } = query
    else {
        return None;
    };
    let values = pre_bind_values(bindings)?;
    let names: Vec<&str> = bindings.iter().map(|(n, _)| *n).collect();
    Some(Query::Select {
        dataset: dataset.clone(),
        pattern: inject_values(pattern, values, &names),
        base_iri: base_iri.clone(),
    })
}

/// Pre-binds `bindings` into an ASK query's WHERE pattern (no projection to
/// extend) and returns the boolean. A SELECT-validator never reaches here.
/// `None` for a non-ASK query or an inexpressible value.
fn pre_bind_ask(query: &Query, bindings: &[Binding]) -> Option<Query> {
    let Query::Ask {
        dataset,
        pattern,
        base_iri,
    } = query
    else {
        return None;
    };
    let values = pre_bind_values(bindings)?;
    // An ASK's WHERE is wrapped in a (zero-variable) `Project`; descend through it
    // so the VALUES lands inside, not above (a `Project` is otherwise a sub-SELECT
    // boundary `push_values_down` must NOT cross).
    let bound_pattern = match pattern {
        GraphPattern::Project { inner, variables } => GraphPattern::Project {
            inner: Box::new(push_values_down(inner, &values)),
            variables: variables.clone(),
        },
        other => push_values_down(other, &values),
    };
    Some(Query::Ask {
        dataset: dataset.clone(),
        pattern: bound_pattern,
        base_iri: base_iri.clone(),
    })
}

/// Pushes the pre-binding `values` table *down* through single-child algebra
/// wrappers (`Filter` / `Extend` / `OrderBy` / `Group` / `Distinct` / `Reduced` /
/// `Slice` / `Minus`-left / `LeftJoin`-left), joining it at the deepest pattern
/// node. This is load-bearing for FILTER-only patterns like `ASK { FILTER(?value
/// = $param) }`: a VALUES table joined ABOVE the `Filter` leaves `?value`/`$param`
/// UNBOUND inside the filter (the filter scopes only its own inner), so the
/// filter evaluates over empty bindings and the constraint never holds. Joining
/// the VALUES table BELOW the filter puts the pre-bound terms in scope. Robust to
/// however the validator query is laid out (no textual prepend). SHACL §6.3
/// pre-binding semantics (substitute the variable's value everywhere it occurs).
fn push_values_down(pattern: &GraphPattern, values: &GraphPattern) -> GraphPattern {
    match pattern {
        GraphPattern::Filter { expr, inner } => GraphPattern::Filter {
            expr: expr.clone(),
            inner: Box::new(push_values_down(inner, values)),
        },
        GraphPattern::Extend {
            inner,
            variable,
            expression,
        } => GraphPattern::Extend {
            inner: Box::new(push_values_down(inner, values)),
            variable: variable.clone(),
            expression: expression.clone(),
        },
        GraphPattern::OrderBy { inner, expression } => GraphPattern::OrderBy {
            inner: Box::new(push_values_down(inner, values)),
            expression: expression.clone(),
        },
        GraphPattern::Distinct { inner } => GraphPattern::Distinct {
            inner: Box::new(push_values_down(inner, values)),
        },
        GraphPattern::Reduced { inner } => GraphPattern::Reduced {
            inner: Box::new(push_values_down(inner, values)),
        },
        GraphPattern::Slice {
            inner,
            start,
            length,
        } => GraphPattern::Slice {
            inner: Box::new(push_values_down(inner, values)),
            start: *start,
            length: *length,
        },
        GraphPattern::Group {
            inner,
            variables,
            aggregates,
        } => GraphPattern::Group {
            inner: Box::new(push_values_down(inner, values)),
            variables: variables.clone(),
            aggregates: aggregates.clone(),
        },
        // For a left-join / minus, bind on the (required) left side only.
        GraphPattern::LeftJoin {
            left,
            right,
            expression,
        } => GraphPattern::LeftJoin {
            left: Box::new(push_values_down(left, values)),
            right: right.clone(),
            expression: expression.clone(),
        },
        GraphPattern::Minus { left, right } => GraphPattern::Minus {
            left: Box::new(push_values_down(left, values)),
            right: right.clone(),
        },
        // A leaf (BGP / Path / Join / Union / Values / sub-SELECT Project / …):
        // join the VALUES table here so its bindings flow up into every parent.
        other => GraphPattern::Join {
            left: Box::new(values.clone()),
            right: Box::new(other.clone()),
        },
    }
}

/// Joins `values` into a SELECT pattern at the right depth: it descends through
/// the solution-modifier wrappers (`Distinct`/`Reduced`/`Slice`/`OrderBy`) so the
/// VALUES table lands *below* them (a `LIMIT`/`DISTINCT` must apply to the
/// pre-bound results, not the other way around), then joins it under the
/// `Project`'s inner and adds every pre-bound `names` variable to the projection
/// if absent. If there is no `Project` (`SELECT *`), it joins at that point — the
/// pre-bound variables surface regardless.
fn inject_values(pattern: &GraphPattern, values: GraphPattern, names: &[&str]) -> GraphPattern {
    match pattern {
        GraphPattern::Project { inner, variables } => {
            let mut variables = variables.clone();
            for name in names {
                if !variables.iter().any(|v| v.as_str() == *name) {
                    variables.push(Variable::new_unchecked(*name));
                }
            }
            // Push the VALUES down through the WHERE pattern so a pre-bound
            // variable used only inside a FILTER/BIND is still in scope there.
            GraphPattern::Project {
                inner: Box::new(push_values_down(inner, &values)),
                variables,
            }
        }
        GraphPattern::Distinct { inner } => GraphPattern::Distinct {
            inner: Box::new(inject_values(inner, values, names)),
        },
        GraphPattern::Reduced { inner } => GraphPattern::Reduced {
            inner: Box::new(inject_values(inner, values, names)),
        },
        GraphPattern::Slice {
            inner,
            start,
            length,
        } => GraphPattern::Slice {
            inner: Box::new(inject_values(inner, values, names)),
            start: *start,
            length: *length,
        },
        GraphPattern::OrderBy { inner, expression } => GraphPattern::OrderBy {
            inner: Box::new(inject_values(inner, values, names)),
            expression: expression.clone(),
        },
        // No projection wrapper (SELECT *): push down; the bound vars surface regardless.
        other => push_values_down(other, &values),
    }
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

// =============================================================================
// [OPUS-4.8] SHACL §6 — SPARQL-based constraint COMPONENTS (sq-sm2).
//
// A `sh:ConstraintComponent` declares parameters (`sh:parameter` → `sh:path`)
// and a validator (`sh:validator` / `sh:nodeValidator` / `sh:propertyValidator`)
// carrying a `sh:ask` or `sh:select` query. When a shape uses the component's
// parameter predicates, the component activates: the parameter VALUES, `$this`,
// `$value` and (on a property shape) `$PATH` are pre-bound, and the validator
// runs — reusing the same VALUES-injection pre-binding + solution→result mapping
// as the §5.2 `sh:sparql` path above.
// =============================================================================

/// A compiled component validator: either an ASK query (run per value node;
/// ASK=false → one violation) or a SELECT query (run per focus node; each
/// solution → one violation, §5.2 mapping). Built once when the shapes model is
/// parsed (so a malformed query is rejected up front, not per focus node).
#[derive(Debug, Clone)]
pub(crate) enum PreparedValidator {
    Ask(Query),
    Select(Query),
}

impl PreparedValidator {
    /// Parses a validator's query text (prefixes already prepended): an ASK form
    /// is an [`Ask`](Self::Ask), a SELECT form a [`Select`](Self::Select), any
    /// other form (or an unparsable query) is `None` (ill-formed → skipped).
    pub(crate) fn build(text: &str, is_ask: bool) -> Option<PreparedValidator> {
        let query = SparqlParser::new().parse_query(text).ok()?;
        match (&query, is_ask) {
            (Query::Ask { .. }, true) => Some(PreparedValidator::Ask(query)),
            (Query::Select { .. }, false) => Some(PreparedValidator::Select(query)),
            _ => None,
        }
    }
}

/// Substitutes `{?name}` / `{$name}` placeholders in a validator `sh:message`
/// with the given `bindings` (the pre-bound `$this` / `$value` / `$paramName`
/// terms). Used by the ASK-validator path, which has no solution row to draw
/// `{?var}` values from (SHACL §6.3 message templates over pre-bound params).
pub(crate) fn substitute_bindings(template: &str, bindings: &[Binding]) -> String {
    let names: Vec<String> = bindings.iter().map(|(n, _)| (*n).to_string()).collect();
    let row: Vec<Option<Term>> = bindings.iter().map(|(_, t)| Some((*t).clone())).collect();
    substitute(template, &names, &row)
}

/// The shape-side fields a custom-component result inherits (filled by `eval.rs`).
pub(crate) struct ComponentResultFields {
    pub focus: Term,
    pub value: Option<Term>,
    pub path: Option<crate::path::Path>,
    pub default_message: String,
}

/// Runs an ASK validator for one `value` node (SHACL §6.3): pre-binds `$this`,
/// `$value`, the shape's parameter VALUES and (on a property shape) `$PATH`, and
/// returns `true` when the value VIOLATES the constraint (ASK = false). A query
/// that fails to pre-bind (inexpressible value) or errors at runtime is treated
/// as conforming (lenient — never panics validate()).
pub(crate) fn ask_violates(
    query: &Query,
    data: &sparq_core::Graph,
    bindings: &[Binding],
) -> bool {
    let Some(bound) = pre_bind_ask(query, bindings) else {
        return false;
    };
    let prepared = sparq_engine::PreparedQuery::from(bound);
    match sparq_engine::ask_prepared(data, &prepared) {
        Ok(holds) => !holds, // ASK true = conforms; ASK false = violation
        Err(_) => false,
    }
}

/// Runs a SELECT validator for one `focus` node (SHACL §6.3): each returned
/// solution is one violation, mapped exactly as the §5.2 `sh:sparql` path
/// (`?value`/`?path`/`?message` → result fields, with `{?var}` message
/// substitution). `bindings` carries `$this`, the parameter VALUES and (on a
/// property shape) `$PATH`. `message` is the component's `sh:message`, if any.
#[allow(clippy::too_many_arguments)]
pub(crate) fn select_validate(
    query: &Query,
    data: &sparq_core::Graph,
    focus: &Term,
    bindings: &[Binding],
    message: Option<&str>,
    mut make_result: impl FnMut(ComponentResultFields) -> ValidationResult,
    out: &mut Vec<ValidationResult>,
) {
    let Some(bound) = pre_bind_select(query, bindings) else {
        return;
    };
    let prepared = sparq_engine::PreparedQuery::from(bound);
    let Ok(result) = sparq_engine::query_prepared(data, &prepared) else {
        return;
    };
    let pos = |name: &str| result.vars.iter().position(|v| v.as_str() == name);
    let (vpos, ppos, mpos) = (pos("value"), pos("path"), pos("message"));
    let result_vars: Vec<String> = result.vars.iter().map(|v| v.as_str().to_string()).collect();
    for row in &result.rows {
        let value = match vpos {
            Some(i) => row.get(i).and_then(|c| c.clone()),
            None => Some(focus.clone()),
        };
        let path = ppos
            .and_then(|i| row.get(i))
            .and_then(|c| c.clone())
            .and_then(|t| match t {
                Term::NamedNode(n) => Some(crate::path::Path::Predicate(n.as_str().to_string())),
                _ => None,
            });
        let row_message = mpos
            .and_then(|i| row.get(i))
            .and_then(|c| c.clone())
            .map(term_lexical);
        let default_message = match (message, &row_message) {
            (Some(tpl), _) => substitute(tpl, &result_vars, row),
            (None, Some(m)) => m.clone(),
            (None, None) => "Violates SPARQL-based constraint".to_string(),
        };
        out.push(make_result(ComponentResultFields {
            focus: focus.clone(),
            value,
            path,
            default_message,
        }));
    }
}
