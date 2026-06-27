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
//! shapes; see this crate's open beads — `bd list -l area:sparq-shacl`).
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

/// [OPUS-4.8] (sq-rnkdh) A parsed SHACL 1.2 **SPARQL-based node expression**
/// (`sh:select` / `sh:sparqlExpr`, SHACL 1.2 §"SPARQL-based Node Expressions"):
/// a SELECT query that, run with `$this` pre-bound to a focus node, yields the
/// output nodes as the bindings of its FIRST result variable. Used by the
/// SHACL-1.2 SPARQL-valued **targets** (`sh:targetNode [ sh:select … ]`) and the
/// SPARQL-valued **value nodes** of a property shape (`sh:values [ sh:select … ]`
/// / `[ sh:sparqlExpr … ]`). Built once at shapes-model parse so a malformed
/// query is rejected up front (`None`), not per focus node.
///
/// This deliberately reuses the always-on `sh:sparql` pre-binding machinery
/// ([`pre_bind_select`] / `sparq_engine::query_prepared`), so it works in BOTH
/// the default and `shacl-af` feature states (it is SHACL-1.2 core/SPARQL, not a
/// SHACL-AF rule).
#[derive(Debug, Clone)]
pub(crate) struct PreparedSelectExpr {
    /// The parsed SELECT algebra (prefixes already resolved at build time).
    query: Query,
}

impl PreparedSelectExpr {
    /// Builds from a `sh:select` SELECT query (with its `sh:prefixes` prepended).
    /// `None` for a non-SELECT / unparsable query (ill-formed → the expression is
    /// dropped leniently, so its target/value set is empty rather than misfiring).
    pub(crate) fn build_select(prefixes: &str, select: &str) -> Option<PreparedSelectExpr> {
        let text = format!("{}\n{}", prefixes, select);
        let query = SparqlParser::new().parse_query(&text).ok()?;
        if !matches!(query, Query::Select { .. }) {
            return None;
        }
        Some(PreparedSelectExpr { query })
    }

    /// Builds from a `sh:sparqlExpr` SPARQL EXPRESSION string. The SHACL-1.2
    /// derivation wraps the expression in a single-solution projection
    /// `SELECT ((EXPR) AS ?result) WHERE {}`; running it with `$this` pre-bound
    /// evaluates the expression for the focus node and yields its value as the
    /// (single) output node. `None` if the derived query does not parse.
    pub(crate) fn build_expr(prefixes: &str, expr: &str) -> Option<PreparedSelectExpr> {
        let select = format!("SELECT (({}) AS ?result) WHERE {{ }}", expr);
        Self::build_select(prefixes, &select)
    }

    /// Evaluates the expression for `focus` against `data`, returning the output
    /// nodes (the bindings of the FIRST result variable, in solution order,
    /// duplicates preserved — the caller dedups/uses them as value nodes). An
    /// unbound first-variable solution contributes no node. A focus node not
    /// expressible as a VALUES ground term (a blank node) or a runtime query error
    /// yields an empty set (lenient; never panics `validate`).
    pub(crate) fn eval(&self, data: &sparq_core::Graph, focus: &Term) -> Vec<Term> {
        let Some(bound) = pre_bind_select(&self.query, &[("this", focus)]) else {
            return Vec::new();
        };
        Self::run(data, bound)
    }

    /// [OPUS-4.8] (sq-rnkdh) Evaluates the expression as a **target** query — with
    /// no `$this` pre-binding (a target SELECT computes the focus nodes itself) —
    /// returning the bindings of the FIRST result variable as the focus-node set.
    pub(crate) fn eval_target(&self, data: &sparq_core::Graph) -> Vec<Term> {
        Self::run(data, self.query.clone())
    }

    /// Runs a (possibly pre-bound) SELECT and collects the FIRST-result-variable
    /// bindings, skipping unbound rows. A runtime query error yields no nodes
    /// (lenient; never panics `validate`).
    fn run(data: &sparq_core::Graph, query: Query) -> Vec<Term> {
        let prepared = sparq_engine::PreparedQuery::from(query);
        let Ok(result) = sparq_engine::query_prepared(data, &prepared) else {
            return Vec::new();
        };
        // The output nodes are the bindings of the FIRST result variable. For an
        // `eval` (pre-bound) query the injection appends `?this` to the projection
        // only if it was absent, so the author's first projected variable stays at
        // position 0.
        let mut out = Vec::new();
        for row in &result.rows {
            if let Some(Some(t)) = row.first() {
                out.push(t.clone());
            }
        }
        out
    }
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

// =============================================================================
// [OPUS-4.8] 🤖 SPARQ agent — sq-qcnn.1 (epic sq-qcnn test-quality program).
//
// Correctness coverage for the DARK branches of the SHACL-SPARQL pre-binding
// (`sh:sparql` §5.2 and the §6 component validators): the deep-algebra arms of
// `push_values_down` (Group / Slice / Distinct / Reduced / OrderBy / Minus) and
// the fail-closed error paths of `ask_violates` / `select_validate` /
// `PreparedSparql::evaluate` / `pre_bind_ask` / `PreparedValidator::build`.
//
// Two complementary layers:
//   * STRUCTURAL — drive `push_values_down` directly over a hand-shaped
//     `Modifier { Filter { Bgp } }` algebra and assert the load-bearing
//     invariant: the pre-binding VALUES table is JOINED at the deepest leaf
//     (BELOW the modifier and the FILTER), with the modifier wrapper preserved
//     unchanged. This is the real production recursion (not a mock).
//   * SEMANTIC — run a real `sh:select` / `sh:ask` validator whose WHERE wraps
//     the FILTER in each solution-modifier shape against real data, and assert
//     the conforms/violation count derived BY HAND from SHACL §5.2 / §6.3 (the
//     pre-bound `$this` / `$value` / `$param` stays in scope through the
//     modifier, so the constraint evaluates correctly).
//
// Test-only: no behaviour change. (The provably-equivalent `func.rs:318` mutant
// noted on sq-qcnn.1 is in a different module and is deliberately not chased.)
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use oxrdf::{Literal, NamedNode};
    use spargebra::algebra::GraphPattern;
    use sparq_core::Graph;

    /// Parse a SELECT and return its top-level WHERE pattern.
    fn select_pattern(q: &str) -> GraphPattern {
        match SparqlParser::new().parse_query(q).unwrap() {
            Query::Select { pattern, .. } => pattern,
            other => panic!("expected SELECT, got {:?}", other),
        }
    }

    /// A `Filter { Bgp }` leaf to nest under a solution modifier — exactly the
    /// shape an authored `{ ?this :p ?v . FILTER(?v > 0) }` produces.
    fn filter_bgp() -> GraphPattern {
        // `SELECT * { ?this :p ?v . FILTER(?v > 0) }` => Project { Filter { Bgp } };
        // unwrap the Project to get the inner Filter { Bgp }.
        match select_pattern("SELECT * WHERE { ?this <http://x/p> ?v . FILTER(?v > 0) }") {
            GraphPattern::Project { inner, .. } => *inner,
            other => other,
        }
    }

    /// A single-row VALUES table binding `?this` to an IRI (the pre-binding the
    /// production code injects).
    fn values_this() -> GraphPattern {
        let term = Term::NamedNode(NamedNode::new("http://example.org/n").unwrap());
        pre_bind_values(&[("this", &term)]).unwrap()
    }

    /// Assert that `pushed` is `Wrapper -> … -> Filter { Join { Values, Bgp } }`:
    /// the modifier wrapper(s) are preserved above, and the VALUES table is joined
    /// at the deepest leaf (BELOW the FILTER). Walks past the modifier shells the
    /// caller named (`wrappers`), then a Filter, then the Join.
    fn assert_values_at_leaf(pushed: &GraphPattern, descend_filter: bool) {
        // Descend any chain down to the Join that wraps Values + the BGP leaf.
        fn find_values_join(p: &GraphPattern) -> bool {
            match p {
                GraphPattern::Join { left, right } => {
                    matches!(**left, GraphPattern::Values { .. })
                        && matches!(**right, GraphPattern::Bgp { .. })
                        || find_values_join(left)
                        || find_values_join(right)
                }
                GraphPattern::Filter { inner, .. }
                | GraphPattern::Extend { inner, .. }
                | GraphPattern::OrderBy { inner, .. }
                | GraphPattern::Distinct { inner }
                | GraphPattern::Reduced { inner }
                | GraphPattern::Slice { inner, .. }
                | GraphPattern::Group { inner, .. } => find_values_join(inner),
                GraphPattern::Minus { left, .. } | GraphPattern::LeftJoin { left, .. } => {
                    find_values_join(left)
                }
                _ => false,
            }
        }
        assert!(
            find_values_join(pushed),
            "VALUES table not joined at the deepest leaf: {:?}",
            pushed
        );
        if descend_filter {
            // The FILTER must STILL wrap the join (the pre-bound vars are in scope
            // inside it) — i.e. there is a Filter somewhere above the Values join.
            fn has_filter_above_join(p: &GraphPattern) -> bool {
                match p {
                    GraphPattern::Filter { inner, .. } => {
                        find_values_join_inner(inner) || has_filter_above_join(inner)
                    }
                    GraphPattern::Extend { inner, .. }
                    | GraphPattern::OrderBy { inner, .. }
                    | GraphPattern::Distinct { inner }
                    | GraphPattern::Reduced { inner }
                    | GraphPattern::Slice { inner, .. }
                    | GraphPattern::Group { inner, .. } => has_filter_above_join(inner),
                    GraphPattern::Minus { left, .. } | GraphPattern::LeftJoin { left, .. } => {
                        has_filter_above_join(left)
                    }
                    _ => false,
                }
            }
            fn find_values_join_inner(p: &GraphPattern) -> bool {
                match p {
                    GraphPattern::Join { left, .. } => {
                        matches!(**left, GraphPattern::Values { .. })
                    }
                    GraphPattern::Filter { inner, .. } => find_values_join_inner(inner),
                    _ => false,
                }
            }
            assert!(
                has_filter_above_join(pushed),
                "FILTER no longer wraps the pre-binding join: {:?}",
                pushed
            );
        }
    }

    // --- push_values_down: deep-algebra arms (structural invariant) ----------

    #[test]
    fn push_values_down_through_group() {
        let inner = GraphPattern::Group {
            inner: Box::new(filter_bgp()),
            variables: vec![Variable::new_unchecked("this")],
            aggregates: vec![],
        };
        let out = push_values_down(&inner, &values_this());
        // Group preserved on top; VALUES joined below the FILTER at the BGP leaf.
        assert!(matches!(out, GraphPattern::Group { .. }), "{:?}", out);
        assert_values_at_leaf(&out, true);
    }

    #[test]
    fn push_values_down_through_slice() {
        let inner = GraphPattern::Slice {
            inner: Box::new(filter_bgp()),
            start: 1,
            length: Some(5),
        };
        let out = push_values_down(&inner, &values_this());
        // The Slice (LIMIT/OFFSET) MUST stay above the pre-binding join.
        match &out {
            GraphPattern::Slice { start, length, .. } => {
                assert_eq!(*start, 1);
                assert_eq!(*length, Some(5));
            }
            other => panic!("expected Slice, got {:?}", other),
        }
        assert_values_at_leaf(&out, true);
    }

    #[test]
    fn push_values_down_through_distinct() {
        let inner = GraphPattern::Distinct {
            inner: Box::new(filter_bgp()),
        };
        let out = push_values_down(&inner, &values_this());
        assert!(matches!(out, GraphPattern::Distinct { .. }), "{:?}", out);
        assert_values_at_leaf(&out, true);
    }

    #[test]
    fn push_values_down_through_reduced() {
        let inner = GraphPattern::Reduced {
            inner: Box::new(filter_bgp()),
        };
        let out = push_values_down(&inner, &values_this());
        assert!(matches!(out, GraphPattern::Reduced { .. }), "{:?}", out);
        assert_values_at_leaf(&out, true);
    }

    #[test]
    fn push_values_down_through_order_by() {
        let inner = match select_pattern(
            "SELECT * WHERE { ?this <http://x/p> ?v . FILTER(?v > 0) } ORDER BY ?v",
        ) {
            // `Project { OrderBy { Filter { Bgp } } }` => take the OrderBy inner.
            GraphPattern::Project { inner, .. } => *inner,
            other => other,
        };
        assert!(matches!(inner, GraphPattern::OrderBy { .. }));
        let out = push_values_down(&inner, &values_this());
        assert!(matches!(out, GraphPattern::OrderBy { .. }), "{:?}", out);
        assert_values_at_leaf(&out, true);
    }

    #[test]
    fn push_values_down_through_minus_binds_left_only() {
        // `Minus { left, right }` => VALUES joins into the (required) LEFT only;
        // the RIGHT (the excluded set) is left untouched (SHACL §6.3: pre-bind the
        // value everywhere it occurs in the REQUIRED pattern).
        let inner = match select_pattern(
            "SELECT * WHERE { ?this <http://x/p> ?v MINUS { ?this <http://x/q> ?w } }",
        ) {
            GraphPattern::Project { inner, .. } => *inner,
            other => other,
        };
        assert!(matches!(inner, GraphPattern::Minus { .. }));
        let out = push_values_down(&inner, &values_this());
        match &out {
            GraphPattern::Minus { left, right } => {
                // Left now joins the VALUES table; right is unchanged (still a Bgp).
                assert!(
                    matches!(
                        &**left,
                        GraphPattern::Join { left: jl, .. } if matches!(**jl, GraphPattern::Values { .. })
                    ),
                    "MINUS left should join VALUES, got {:?}",
                    left
                );
                assert!(
                    matches!(&**right, GraphPattern::Bgp { .. }),
                    "MINUS right must be untouched, got {:?}",
                    right
                );
            }
            other => panic!("expected Minus, got {:?}", other),
        }
    }

    #[test]
    fn push_values_down_left_join_binds_left_only() {
        // OPTIONAL => LeftJoin; the pre-binding joins the required LEFT only.
        let inner = match select_pattern(
            "SELECT * WHERE { ?this <http://x/p> ?v OPTIONAL { ?this <http://x/q> ?w } }",
        ) {
            GraphPattern::Project { inner, .. } => *inner,
            other => other,
        };
        assert!(matches!(inner, GraphPattern::LeftJoin { .. }));
        let out = push_values_down(&inner, &values_this());
        match &out {
            GraphPattern::LeftJoin { left, .. } => assert!(
                matches!(
                    &**left,
                    GraphPattern::Join { left: jl, .. } if matches!(**jl, GraphPattern::Values { .. })
                ),
                "LeftJoin left should join VALUES, got {:?}",
                left
            ),
            other => panic!("expected LeftJoin, got {:?}", other),
        }
    }

    // --- inject_values: solution-modifier descent (Reduced/Slice/OrderBy) -----

    #[test]
    fn inject_values_descends_distinct_reduced_slice_orderby() {
        let this = Term::NamedNode(NamedNode::new("http://example.org/n").unwrap());
        // Each top-level modifier above the Project is descended by inject_values
        // (the LIMIT/DISTINCT/ORDER BY apply to the PRE-BOUND results), then the
        // bound var is added to the projection.
        for q in [
            "SELECT DISTINCT ?this WHERE { ?this <http://x/p> ?v . FILTER(?v > 0) }",
            "SELECT REDUCED ?this WHERE { ?this <http://x/p> ?v . FILTER(?v > 0) }",
            "SELECT ?this WHERE { ?this <http://x/p> ?v . FILTER(?v > 0) } ORDER BY ?v LIMIT 3",
        ] {
            let query = SparqlParser::new().parse_query(q).unwrap();
            let bound = pre_bind_select(&query, &[("this", &this)]).unwrap();
            // The pre-bound `?this` must surface in the projection (so a consumer
            // reading it back gets the focus node).
            let Query::Select { pattern, .. } = &bound else {
                unreachable!()
            };
            assert!(
                projection_contains(pattern, "this"),
                "?this not projected for `{}`: {:?}",
                q,
                pattern
            );
        }
    }

    /// Does some Project under `p` list a variable named `name`?
    fn projection_contains(p: &GraphPattern, name: &str) -> bool {
        match p {
            GraphPattern::Project { inner, variables } => {
                variables.iter().any(|v| v.as_str() == name) || projection_contains(inner, name)
            }
            GraphPattern::Distinct { inner }
            | GraphPattern::Reduced { inner }
            | GraphPattern::Slice { inner, .. }
            | GraphPattern::OrderBy { inner, .. }
            | GraphPattern::Filter { inner, .. }
            | GraphPattern::Group { inner, .. }
            | GraphPattern::Extend { inner, .. } => projection_contains(inner, name),
            GraphPattern::Join { left, right } | GraphPattern::Minus { left, right } => {
                projection_contains(left, name) || projection_contains(right, name)
            }
            _ => false,
        }
    }

    // --- pre_bind_ask: descend the ASK Project, bind through a modifier --------

    #[test]
    fn pre_bind_ask_descends_into_project_and_modifier() {
        let value = Term::Literal(Literal::new_simple_literal("abcdef"));
        // `ASK { FILTER(STRLEN(STR($value)) <= 3) }` => Project { Filter { Bgp } }.
        let q = SparqlParser::new()
            .parse_query("ASK { FILTER (STRLEN(STR(?value)) <= 3) }")
            .unwrap();
        let bound = pre_bind_ask(&q, &[("value", &value)]).unwrap();
        let Query::Ask { pattern, .. } = &bound else {
            panic!("expected ASK")
        };
        // The outer Project shell is preserved; VALUES joined below the FILTER.
        match pattern {
            GraphPattern::Project { inner, .. } => assert_values_at_leaf(inner, true),
            other => panic!("expected Project shell, got {:?}", other),
        }
    }

    // --- semantic: a real validator whose WHERE uses each modifier ------------

    fn graph(ttl: &str) -> Graph {
        Graph::load_str(ttl, "turtle").unwrap()
    }

    const DATA: &str = r#"
        @prefix ex: <http://example.org/> .
        ex:alice ex:age 30 ; ex:tag "a", "b" .
        ex:bob   ex:age -1 ; ex:tag "c" .
    "#;

    /// SELECT-component validator: run per focus, each solution is a violation.
    /// The WHERE shape (Group/OrderBy/Slice/Minus over the FILTER) must NOT change
    /// the answer — the pre-bound `$this` stays in scope through the modifier.
    fn select_violations(select: &str, focus_iri: &str) -> usize {
        let data = graph(DATA);
        let query = SparqlParser::new().parse_query(select).unwrap();
        let validator = match query {
            Query::Select { .. } => query,
            _ => panic!("expected SELECT"),
        };
        let focus = Term::NamedNode(NamedNode::new(focus_iri).unwrap());
        let bindings: &[Binding] = &[("this", &focus)];
        let mut out = Vec::new();
        select_validate(
            &validator,
            &data,
            &focus,
            bindings,
            None,
            |f: ComponentResultFields| ValidationResult {
                focus_node: f.focus,
                path: f.path,
                value: f.value,
                source_shape: Term::NamedNode(NamedNode::new("http://example.org/S").unwrap()),
                source_component: "http://www.w3.org/ns/shacl#SPARQLConstraintComponent"
                    .to_string(),
                severity: "http://www.w3.org/ns/shacl#Violation".to_string(),
                messages: vec![],
                default_message: f.default_message,
                details: vec![],
            },
            &mut out,
        );
        out.len()
    }

    #[test]
    fn select_validate_group_pre_binds_this() {
        // Per focus, GROUP BY $this with HAVING(COUNT > 1): bob has 1 tag, alice 2.
        // The validator flags a focus with MORE than one ex:tag.
        let q = "SELECT ?this WHERE { ?this <http://example.org/tag> ?t } \
                 GROUP BY ?this HAVING (COUNT(?t) > 1)";
        // alice (2 tags) -> 1 solution -> 1 violation.
        assert_eq!(select_violations(q, "http://example.org/alice"), 1);
        // bob (1 tag) -> 0 solutions -> conforms.
        assert_eq!(select_violations(q, "http://example.org/bob"), 0);
    }

    #[test]
    fn select_validate_order_by_slice_pre_binds_this() {
        // ORDER BY + LIMIT over the pre-binding: alice has age 30 (>0 -> conforms,
        // FILTER keeps only negatives), bob has age -1 -> 1 violation.
        let q = "SELECT ?this ?value WHERE { ?this <http://example.org/age> ?value . \
                 FILTER(?value < 0) } ORDER BY ?value LIMIT 10";
        assert_eq!(select_violations(q, "http://example.org/bob"), 1);
        assert_eq!(select_violations(q, "http://example.org/alice"), 0);
    }

    #[test]
    fn select_validate_minus_pre_binds_this() {
        // MINUS over the pre-binding: flag a focus that has an ex:age but is NOT
        // excluded by the MINUS (here MINUS removes nothing matching). bob age -1
        // present -> the pattern { ?this :age ?value } MINUS {} yields a solution.
        let q = "SELECT ?this ?value WHERE { ?this <http://example.org/age> ?value . \
                 FILTER(?value < 0) MINUS { ?this <http://example.org/never> ?z } }";
        assert_eq!(select_violations(q, "http://example.org/bob"), 1);
        assert_eq!(select_violations(q, "http://example.org/alice"), 0);
    }

    /// ASK-component validator: run per value node; ASK=false (constraint not
    /// satisfied) is a violation.
    fn ask_is_violation(ask: &str, value: &Term) -> bool {
        let data = graph(DATA);
        let query = SparqlParser::new().parse_query(ask).unwrap();
        let this = Term::NamedNode(NamedNode::new("http://example.org/alice").unwrap());
        ask_violates(&query, &data, &[("this", &this), ("value", value)])
    }

    #[test]
    fn ask_violates_filter_pre_binds_value() {
        // ASK { FILTER(STRLEN(STR($value)) <= 3) }: a 6-char value VIOLATES
        // (ASK=false); a 2-char value conforms (ASK=true).
        let q = "ASK { FILTER (STRLEN(STR($value)) <= 3) }";
        let long = Term::Literal(Literal::new_simple_literal("abcdef"));
        let short = Term::Literal(Literal::new_simple_literal("ab"));
        assert!(ask_is_violation(q, &long), "6-char value should violate");
        assert!(!ask_is_violation(q, &short), "2-char value should conform");
    }

    // --- ERROR / fail-closed paths -------------------------------------------

    #[test]
    fn prepared_validator_build_rejects_form_mismatch() {
        // is_ask=true but the query is a SELECT -> None (ill-formed -> skipped).
        assert!(PreparedValidator::build("SELECT * WHERE { ?s ?p ?o }", true).is_none());
        // is_ask=false but the query is an ASK -> None.
        assert!(PreparedValidator::build("ASK { ?s ?p ?o }", false).is_none());
        // A CONSTRUCT (neither ASK nor SELECT) -> None for both flags.
        let construct = "CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }";
        assert!(PreparedValidator::build(construct, true).is_none());
        assert!(PreparedValidator::build(construct, false).is_none());
        // Unparsable -> None.
        assert!(PreparedValidator::build("NOT A QUERY", true).is_none());
        // Well-formed matches build OK.
        assert!(matches!(
            PreparedValidator::build("ASK { ?s ?p ?o }", true),
            Some(PreparedValidator::Ask(_))
        ));
        assert!(matches!(
            PreparedValidator::build("SELECT * WHERE { ?s ?p ?o }", false),
            Some(PreparedValidator::Select(_))
        ));
    }

    #[test]
    fn prepared_sparql_build_rejects_non_select() {
        // A non-SELECT sh:select is ill-formed -> None.
        let ask = SparqlConstraint {
            select: "ASK { ?s ?p ?o }".to_string(),
            prefixes: String::new(),
            message: None,
            severity: None,
            deactivated: false,
            prepared: None,
        };
        assert!(PreparedSparql::build(&ask).is_none());
        // Unparsable -> None.
        let bad = SparqlConstraint {
            select: "SELECT ??? garbage".to_string(),
            prefixes: String::new(),
            message: None,
            severity: None,
            deactivated: false,
            prepared: None,
        };
        assert!(PreparedSparql::build(&bad).is_none());
        // Valid SELECT -> Some.
        let ok = SparqlConstraint {
            select: "SELECT ?this WHERE { ?this ?p ?o }".to_string(),
            prefixes: String::new(),
            message: None,
            severity: None,
            deactivated: false,
            prepared: None,
        };
        assert!(PreparedSparql::build(&ok).is_some());
    }

    #[test]
    fn pre_bind_select_rejects_non_select_query() {
        let this = Term::NamedNode(NamedNode::new("http://example.org/n").unwrap());
        let ask = SparqlParser::new().parse_query("ASK { ?s ?p ?o }").unwrap();
        assert!(pre_bind_select(&ask, &[("this", &this)]).is_none());
    }

    #[test]
    fn pre_bind_ask_rejects_non_ask_query() {
        let this = Term::NamedNode(NamedNode::new("http://example.org/n").unwrap());
        let sel = SparqlParser::new()
            .parse_query("SELECT * WHERE { ?s ?p ?o }")
            .unwrap();
        assert!(pre_bind_ask(&sel, &[("this", &this)]).is_none());
    }

    #[test]
    fn pre_bind_rejects_inexpressible_blank_node() {
        // A blank-node focus cannot be written as a VALUES ground term -> None
        // (the constraint is skipped rather than mis-evaluated, SHACL-leniently).
        let bnode = Term::BlankNode(oxrdf::BlankNode::new("b0").unwrap());
        let sel = SparqlParser::new()
            .parse_query("SELECT ?this WHERE { ?this ?p ?o }")
            .unwrap();
        assert!(pre_bind_select(&sel, &[("this", &bnode)]).is_none());
        let ask = SparqlParser::new().parse_query("ASK { ?s ?p ?o }").unwrap();
        assert!(pre_bind_ask(&ask, &[("this", &bnode)]).is_none());
        // term_to_ground itself returns None for a blank node.
        assert!(term_to_ground(&bnode).is_none());
    }

    #[test]
    fn ask_violates_fail_closed_on_inexpressible_value() {
        // ask_violates pre-binds a blank-node value -> pre_bind_ask None ->
        // returns false (conforms, fail-closed; never panics validate()).
        let data = graph(DATA);
        let bnode = Term::BlankNode(oxrdf::BlankNode::new("b1").unwrap());
        let q = SparqlParser::new()
            .parse_query("ASK { FILTER (STRLEN(STR($value)) <= 3) }")
            .unwrap();
        assert!(!ask_violates(&q, &data, &[("value", &bnode)]));
    }

    #[test]
    fn select_validate_fail_closed_on_inexpressible_focus() {
        // A blank-node focus -> pre_bind_select None -> no results pushed.
        let data = graph(DATA);
        let bnode = Term::BlankNode(oxrdf::BlankNode::new("b2").unwrap());
        let q = SparqlParser::new()
            .parse_query("SELECT ?this WHERE { ?this ?p ?o }")
            .unwrap();
        let mut out = Vec::new();
        select_validate(
            &q,
            &data,
            &bnode,
            &[("this", &bnode)],
            None,
            |_f: ComponentResultFields| unreachable!("no solutions for an inexpressible focus"),
            &mut out,
        );
        assert!(out.is_empty());
    }

    // --- term_lexical / substitute helper edges -------------------------------

    #[test]
    fn term_lexical_covers_each_term_kind() {
        assert_eq!(
            term_lexical(Term::NamedNode(NamedNode::new("http://x/n").unwrap())),
            "http://x/n"
        );
        assert_eq!(
            term_lexical(Term::Literal(Literal::new_simple_literal("hi"))),
            "hi"
        );
        assert_eq!(
            term_lexical(Term::BlankNode(oxrdf::BlankNode::new("b").unwrap())),
            "_:b"
        );
    }

    #[test]
    fn substitute_keeps_unknown_and_unclosed_braces_verbatim() {
        let vars = vec!["this".to_string()];
        let row = vec![Some(Term::NamedNode(NamedNode::new("http://x/n").unwrap()))];
        // Known {?this} substitutes; unknown {?missing} and a non-var {literal}
        // and an unclosed `{` are kept verbatim.
        assert_eq!(
            substitute("a {?this} b {?missing} c {plain} d {", &vars, &row),
            "a http://x/n b {?missing} c {plain} d {"
        );
        // No placeholders at all -> identity.
        assert_eq!(substitute("no braces here", &vars, &row), "no braces here");
    }

    #[test]
    fn substitute_bindings_over_pre_bound_terms() {
        let value = Term::Literal(Literal::new_simple_literal("xyz"));
        let out = substitute_bindings("len of {$value} too big", &[("value", &value)]);
        assert_eq!(out, "len of xyz too big");
    }

    // --- runtime-query-error fail-closed paths --------------------------------
    //
    // A `SERVICE` clause parses fine but the local-only executor refuses it at
    // RUNTIME (`Err("unsupported graph pattern: Service …")`). The three
    // SHACL-SPARQL entry points must treat that as conforming / no-solutions
    // (lenient — a query error NEVER panics validate(); SHACL §5.2 / §6.3).

    #[test]
    fn ask_violates_fail_closed_on_runtime_query_error() {
        let data = graph(DATA);
        let this = Term::NamedNode(NamedNode::new("http://example.org/alice").unwrap());
        let q = SparqlParser::new()
            .parse_query("ASK { SERVICE <http://example.org/remote> { ?s ?p ?o } }")
            .unwrap();
        // ask_prepared returns Err -> ask_violates returns false (conforms).
        assert!(!ask_violates(&q, &data, &[("this", &this)]));
    }

    #[test]
    fn select_validate_fail_closed_on_runtime_query_error() {
        let data = graph(DATA);
        let focus = Term::NamedNode(NamedNode::new("http://example.org/alice").unwrap());
        let q = SparqlParser::new()
            .parse_query(
                "SELECT ?this WHERE { SERVICE <http://example.org/remote> { ?this ?p ?o } }",
            )
            .unwrap();
        let mut out = Vec::new();
        select_validate(
            &q,
            &data,
            &focus,
            &[("this", &focus)],
            None,
            |_f: ComponentResultFields| unreachable!("a runtime query error yields no solutions"),
            &mut out,
        );
        // query_prepared returns Err -> no results pushed.
        assert!(out.is_empty());
    }

    #[test]
    fn prepared_sparql_evaluate_fail_closed_on_runtime_query_error() {
        // The §5.2 `sh:sparql` path: a SERVICE in the sh:select errors at runtime,
        // so PreparedSparql::evaluate pushes no results (conforms).
        let data = graph(DATA);
        let focus = Term::NamedNode(NamedNode::new("http://example.org/alice").unwrap());
        let constraint = SparqlConstraint {
            select: "SELECT ?this WHERE { SERVICE <http://example.org/remote> { ?this ?p ?o } }"
                .to_string(),
            prefixes: String::new(),
            message: None,
            severity: None,
            deactivated: false,
            prepared: None,
        };
        let prepared = PreparedSparql::build(&constraint).expect("a SELECT with SERVICE parses");
        let mut out = Vec::new();
        prepared.evaluate(
            &data,
            &focus,
            &constraint,
            |_f: ResultFields| unreachable!("a runtime query error yields no solutions"),
            &mut out,
        );
        assert!(out.is_empty());
    }

    // --- PreparedSparql::evaluate happy path: ?value / ?path / {?var} mapping --
    //
    // Drives the §5.2 solution -> ValidationResult field mapping (the ?value /
    // ?path / {?var}-message branches) directly through the public evaluate().

    #[test]
    fn prepared_sparql_evaluate_maps_value_path_and_message() {
        // Data: alice ex:knows bob (an IRI value on a predicate path).
        let data =
            graph("@prefix ex: <http://example.org/> . ex:alice ex:knows ex:bob ; ex:age 30 .");
        let focus = Term::NamedNode(NamedNode::new("http://example.org/alice").unwrap());
        // Project ?value and ?path; the constraint message references {?value}.
        let constraint = SparqlConstraint {
            select: "SELECT ?this ?value ?path WHERE { \
                       BIND(<http://example.org/bob> AS ?value) \
                       BIND(<http://example.org/knows> AS ?path) \
                       ?this <http://example.org/knows> ?value }"
                .to_string(),
            prefixes: String::new(),
            message: Some("offender {?value} via {?path}".to_string()),
            severity: None,
            deactivated: false,
            prepared: None,
        };
        let prepared = PreparedSparql::build(&constraint).unwrap();
        let mut out = Vec::new();
        prepared.evaluate(
            &data,
            &focus,
            &constraint,
            |f: ResultFields| ValidationResult {
                focus_node: focus.clone(),
                path: f.path,
                value: f.value,
                source_shape: focus.clone(),
                source_component: "x".to_string(),
                severity: "x".to_string(),
                messages: vec![],
                default_message: f.default_message,
                details: vec![],
            },
            &mut out,
        );
        assert_eq!(out.len(), 1, "alice ex:knows ex:bob -> one solution");
        let r = &out[0];
        assert_eq!(
            r.value,
            Some(Term::NamedNode(
                NamedNode::new("http://example.org/bob").unwrap()
            )),
            "?value maps to the bound bob"
        );
        assert!(
            matches!(&r.path, Some(crate::path::Path::Predicate(p)) if p == "http://example.org/knows"),
            "?path (an IRI) maps to a predicate path, got {:?}",
            r.path
        );
        assert_eq!(
            r.default_message, "offender http://example.org/bob via http://example.org/knows",
            "{{?value}}/{{?path}} substituted from the solution row"
        );
    }

    #[test]
    fn prepared_sparql_evaluate_uses_row_message_when_no_constraint_message() {
        // No constraint sh:message + a projected ?message binding -> the result's
        // default_message is taken from the ?message row value (§5.2.2 precedence:
        // constraint message > ?message binding > generated default).
        let data = graph("@prefix ex: <http://example.org/> . ex:alice ex:age 30 .");
        let focus = Term::NamedNode(NamedNode::new("http://example.org/alice").unwrap());
        let constraint = SparqlConstraint {
            select: "SELECT ?this ?message WHERE { ?this <http://example.org/age> ?v . \
                       BIND(\"row-level reason\" AS ?message) }"
                .to_string(),
            prefixes: String::new(),
            message: None, // no constraint message -> fall through to ?message
            severity: None,
            deactivated: false,
            prepared: None,
        };
        let prepared = PreparedSparql::build(&constraint).unwrap();
        let mut out = Vec::new();
        prepared.evaluate(
            &data,
            &focus,
            &constraint,
            |f: ResultFields| ValidationResult {
                focus_node: focus.clone(),
                path: f.path,
                value: f.value,
                source_shape: focus.clone(),
                source_component: "x".to_string(),
                severity: "x".to_string(),
                messages: vec![],
                default_message: f.default_message,
                details: vec![],
            },
            &mut out,
        );
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].default_message, "row-level reason");
    }

    #[test]
    fn prepared_sparql_evaluate_fail_closed_on_blank_node_focus() {
        // A blank-node focus is inexpressible as VALUES -> evaluate() returns
        // early (the §5.2 unreachable-today guard): no results pushed.
        let data = graph("@prefix ex: <http://example.org/> . ex:alice ex:age 30 .");
        let bnode = Term::BlankNode(oxrdf::BlankNode::new("focus0").unwrap());
        let constraint = SparqlConstraint {
            select: "SELECT ?this WHERE { ?this ?p ?o }".to_string(),
            prefixes: String::new(),
            message: None,
            severity: None,
            deactivated: false,
            prepared: None,
        };
        let prepared = PreparedSparql::build(&constraint).unwrap();
        let mut out = Vec::new();
        prepared.evaluate(
            &data,
            &bnode,
            &constraint,
            |_f: ResultFields| unreachable!("an inexpressible focus yields no solutions"),
            &mut out,
        );
        assert!(out.is_empty());
    }

    #[test]
    fn select_validate_maps_path_when_projected() {
        // A §6 SELECT validator that projects ?path (an IRI) -> the result's
        // sh:resultPath is that predicate path (§5.2.2 mapping in select_validate).
        let data = graph("@prefix ex: <http://example.org/> . ex:alice ex:knows ex:bob .");
        let focus = Term::NamedNode(NamedNode::new("http://example.org/alice").unwrap());
        let query = SparqlParser::new()
            .parse_query(
                "SELECT ?this ?path WHERE { BIND(<http://example.org/knows> AS ?path) \
                 ?this <http://example.org/knows> ?o }",
            )
            .unwrap();
        let mut out = Vec::new();
        select_validate(
            &query,
            &data,
            &focus,
            &[("this", &focus)],
            None,
            |f: ComponentResultFields| ValidationResult {
                focus_node: f.focus,
                path: f.path,
                value: f.value,
                source_shape: focus.clone(),
                source_component: "x".to_string(),
                severity: "x".to_string(),
                messages: vec![],
                default_message: f.default_message,
                details: vec![],
            },
            &mut out,
        );
        assert_eq!(out.len(), 1);
        assert!(
            matches!(&out[0].path, Some(crate::path::Path::Predicate(p)) if p == "http://example.org/knows"),
            "?path (an IRI) -> a predicate path, got {:?}",
            out[0].path
        );
    }

    #[test]
    fn select_validate_non_iri_path_maps_to_none() {
        // A ?path bound to a NON-IRI (a literal) is not a predicate path -> the
        // result carries no sh:resultPath (the `_ => None` arm of the §5.2.2 path
        // mapping). Spec-correct: only an IRI ?path is a predicate path.
        let data = graph("@prefix ex: <http://example.org/> . ex:alice ex:age 30 .");
        let focus = Term::NamedNode(NamedNode::new("http://example.org/alice").unwrap());
        let query = SparqlParser::new()
            .parse_query(
                "SELECT ?this ?path WHERE { BIND(\"not-an-iri\" AS ?path) \
                 ?this <http://example.org/age> ?o }",
            )
            .unwrap();
        let mut out = Vec::new();
        select_validate(
            &query,
            &data,
            &focus,
            &[("this", &focus)],
            None,
            |f: ComponentResultFields| ValidationResult {
                focus_node: f.focus,
                path: f.path,
                value: f.value,
                source_shape: focus.clone(),
                source_component: "x".to_string(),
                severity: "x".to_string(),
                messages: vec![],
                default_message: f.default_message,
                details: vec![],
            },
            &mut out,
        );
        assert_eq!(out.len(), 1);
        assert!(
            out[0].path.is_none(),
            "a literal ?path is not a predicate path"
        );
    }

    // --- [OPUS-4.8] (sq-rnkdh) SHACL 1.2 SPARQL-based node expressions ---------

    const EXPR_DATA: &str = r#"
        @prefix ex: <http://example.org/> .
        ex:alice ex:age 30 ; ex:firstName "Al" ; ex:lastName "Ice" .
        ex:bob   ex:age 15 .
    "#;

    fn iri(s: &str) -> Term {
        Term::NamedNode(NamedNode::new(s).unwrap())
    }

    #[test]
    fn select_expr_build_rejects_non_select() {
        // A non-SELECT (or unparsable) sh:select is ill-formed -> None.
        assert!(PreparedSelectExpr::build_select("", "ASK { ?s ?p ?o }").is_none());
        assert!(PreparedSelectExpr::build_select("", "SELECT ??? nope").is_none());
        assert!(PreparedSelectExpr::build_select("", "SELECT ?x WHERE { ?x ?p ?o }").is_some());
    }

    #[test]
    fn select_expr_eval_pre_binds_this_and_takes_first_var() {
        // sh:values-style: per focus, project the value via $this.
        let data = graph(EXPR_DATA);
        let expr = PreparedSelectExpr::build_select(
            "PREFIX ex: <http://example.org/>",
            "SELECT ?n WHERE { $this ex:firstName ?n }",
        )
        .unwrap();
        let nodes = expr.eval(&data, &iri("http://example.org/alice"));
        assert_eq!(
            nodes,
            vec![Term::Literal(oxrdf::Literal::new_simple_literal("Al"))]
        );
        // A focus with no firstName yields no value nodes.
        assert!(expr.eval(&data, &iri("http://example.org/bob")).is_empty());
    }

    #[test]
    fn select_expr_build_expr_evaluates_expression() {
        // sh:sparqlExpr: SELECT ((EXPR) AS ?result) WHERE {} with $this bound.
        let data = graph(EXPR_DATA);
        let expr = PreparedSelectExpr::build_expr("", "STRLEN(STR($this))").unwrap();
        let nodes = expr.eval(&data, &iri("http://example.org/alice"));
        // STRLEN("http://example.org/alice") = 24.
        assert_eq!(
            nodes,
            vec![Term::Literal(oxrdf::Literal::new_typed_literal(
                "24",
                NamedNode::new("http://www.w3.org/2001/XMLSchema#integer").unwrap(),
            ))]
        );
    }

    #[test]
    fn select_expr_eval_target_runs_unbound_and_collects_first_var() {
        // sh:targetNode [ sh:select ]: no $this, the query SELECTs the focus nodes.
        let data = graph(EXPR_DATA);
        let expr = PreparedSelectExpr::build_select(
            "PREFIX ex: <http://example.org/>",
            "SELECT ?p WHERE { ?p ex:age ?a . FILTER (?a < 18) }",
        )
        .unwrap();
        let nodes = expr.eval_target(&data);
        assert_eq!(nodes, vec![iri("http://example.org/bob")]);
    }
}
