//! Constraint-component evaluation: walks each targeted shape's focus nodes and
//! emits [`ValidationResult`]s via direct graph scans (no SPARQL round-trip).

use crate::model::{sh, Component, ComponentDef, Shape, ShapesModel, Target};
use crate::path::Path;
use crate::report::ValidationResult;
use crate::sparql::{ComponentResultFields, PreparedValidator};
use crate::view::GraphView;
use oxrdf::{Literal, Term};
use rustc_hash::FxHashMap;
use sparq_core::Graph;
use std::cmp::Ordering;

const RDF_LANG_STRING: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString";
const XSD: &str = "http://www.w3.org/2001/XMLSchema#";

/// Runs every targeted shape over its focus nodes. Results are deliberately NOT
/// deduplicated: the SHACL test suite expects one result per constraint-component
/// OCCURRENCE (e.g. two sh:class violations on one value) and per traversal route
/// (a nested shape reached through two parents reports twice). Duplicate focus
/// nodes from overlapping targets ARE collapsed (in `focus_nodes`).
pub(crate) fn validate_graph(data: &Graph, shapes: &ShapesModel) -> Vec<ValidationResult> {
    let mut v = Validator {
        data: GraphView::new(data),
        shapes,
        stack: Vec::new(),
        memo: vec![FxHashMap::default(); shapes.shapes.len()],
        min_reentry: usize::MAX,
        regexes: FxHashMap::default(),
    };
    let mut out = Vec::new();
    for &sid in &shapes.targeted {
        for focus in v.focus_nodes(&shapes.shapes[sid]) {
            v.validate_shape(sid, &focus, &mut out);
        }
    }
    out
}

/// [OPUS-4.8] (sq-d1dw, `shacl-af`) True iff `node` conforms to the shape `sid`
/// over `data` — i.e. validating `node` against that shape (as a standalone
/// focus node, ignoring the shape's own targets) produces no results. The
/// SHACL-AF rules module uses this to evaluate `sh:condition` gating: a rule
/// fires for a focus node only when it conforms to every condition shape. Reuses
/// the full constraint validator (Core + sh:sparql + nested shapes), so a
/// condition may be any shape. Gated to the feature so it adds nothing when off.
#[cfg(feature = "shacl-af")]
pub(crate) fn conforms_node(data: &Graph, shapes: &ShapesModel, sid: usize, node: &Term) -> bool {
    let mut v = Validator {
        data: GraphView::new(data),
        shapes,
        stack: Vec::new(),
        memo: vec![FxHashMap::default(); shapes.shapes.len()],
        min_reentry: usize::MAX,
        regexes: FxHashMap::default(),
    };
    v.conforms(node, sid)
}

struct Validator<'a> {
    data: GraphView<'a>,
    shapes: &'a ShapesModel,
    /// (focus, shape) pairs currently being validated — the recursion guard for
    /// cyclic sh:node / sh:property references (re-entry counts as conforming).
    stack: Vec<(Term, usize)>,
    /// `(focus, shape) → bool` conformance memo, indexed by shape id. Only
    /// CONTEXT-FREE results are stored — see [`conforms`](Validator::conforms)
    /// for the cycle rule that keeps this sound under the recursion guard.
    memo: Vec<FxHashMap<Term, bool>>,
    /// The lowest stack depth the recursion guard re-entered since the current
    /// `conforms` frame was pushed (`usize::MAX` = none). A frame may be
    /// memoised only if no re-entry pointed BELOW it.
    min_reentry: usize,
    regexes: FxHashMap<String, Option<regex::Regex>>,
}

impl<'a> Validator<'a> {
    fn focus_nodes(&self, shape: &Shape) -> Vec<Term> {
        let mut out = Vec::new();
        for t in &shape.targets {
            match t {
                Target::Node(n) => out.push(n.clone()),
                Target::Class(c) | Target::ImplicitClass(c) => {
                    out.extend(self.data.instances_of(c))
                }
                Target::SubjectsOf(p) => out.extend(self.data.subjects_of(p)),
                Target::ObjectsOf(p) => out.extend(self.data.objects_of(p)),
            }
        }
        crate::view::dedup(out)
    }

    /// True iff `node` produces no results against `shape` (conformance
    /// checking), memoised per `(focus, shape)` pair — shared shapes reached
    /// through several routes (sh:and members, a sh:node target re-visited per
    /// value, …) validate once.
    ///
    /// # Memoisation under the recursion guard
    ///
    /// Re-entry of an in-progress `(node, shape)` pair counts as conforming
    /// (recursion is undefined in SHACL), which makes a frame's result depend
    /// on the *stack it ran under* whenever a re-entry escaped below it. The
    /// rule that keeps the memo sound: track the LOWEST stack depth any guard
    /// hit pointed at (`min_reentry`) and store a frame's result only when no
    /// re-entry pointed below that frame. A frame that closes its own cycle
    /// (re-entry at exactly its depth) is still memoisable — re-running it
    /// from an empty stack reproduces the same self-referential assumption.
    fn conforms(&mut self, node: &Term, sid: usize) -> bool {
        if let Some(i) = self.stack.iter().position(|(n, s)| n == node && *s == sid) {
            // Recursive re-entry: treat as conforming, and record how deep the
            // cycle reaches so the frames above it skip the memo.
            self.min_reentry = self.min_reentry.min(i);
            return true;
        }
        if let Some(&cached) = self.memo[sid].get(node) {
            return cached;
        }
        let depth = self.stack.len();
        let saved = std::mem::replace(&mut self.min_reentry, usize::MAX);
        self.stack.push((node.clone(), sid));
        let mut tmp = Vec::new();
        self.validate_shape(sid, node, &mut tmp);
        self.stack.pop();
        let ok = tmp.is_empty();
        if self.min_reentry >= depth {
            // No re-entry escaped below this frame: the result is context-free.
            self.memo[sid].insert(node.clone(), ok);
        }
        // Propagate cycle reach to the enclosing frame.
        self.min_reentry = saved.min(self.min_reentry);
        ok
    }

    fn validate_shape(&mut self, sid: usize, focus: &Term, out: &mut Vec<ValidationResult>) {
        let shape = &self.shapes.shapes[sid];
        if shape.deactivated {
            return;
        }
        let values: Vec<Term> = match &shape.path {
            Some(path) => path.values(&self.data, focus),
            None => vec![focus.clone()],
        };
        let components = shape.components.clone();
        for comp in &components {
            self.eval_component(sid, focus, &values, comp, out);
        }
    }

    /// A result owned by shape `sid` for `focus`.
    fn result(
        &self,
        sid: usize,
        focus: &Term,
        value: Option<Term>,
        component: &str,
        message: String,
    ) -> ValidationResult {
        let shape = &self.shapes.shapes[sid];
        ValidationResult {
            focus_node: focus.clone(),
            path: shape.path.clone(),
            value,
            source_shape: shape.node.clone(),
            source_component: sh(component),
            severity: shape.severity.clone(),
            messages: shape.messages.clone(),
            default_message: message,
        }
    }

    #[allow(clippy::too_many_lines)]
    fn eval_component(
        &mut self,
        sid: usize,
        focus: &Term,
        values: &[Term],
        comp: &Component,
        out: &mut Vec<ValidationResult>,
    ) {
        match comp {
            Component::Class(cls) => {
                for v in values {
                    if !self.data.is_instance_of(v, cls) {
                        out.push(self.result(
                            sid,
                            focus,
                            Some(v.clone()),
                            "ClassConstraintComponent",
                            format!("Value does not have class {cls}"),
                        ));
                    }
                }
            }
            Component::Datatype(dt) => {
                for v in values {
                    let ok = match v {
                        Term::Literal(l) => l.datatype().as_str() == dt && well_formed(l),
                        _ => false,
                    };
                    if !ok {
                        out.push(self.result(
                            sid,
                            focus,
                            Some(v.clone()),
                            "DatatypeConstraintComponent",
                            format!("Value does not have datatype <{dt}>"),
                        ));
                    }
                }
            }
            Component::NodeKind(kind) => {
                for v in values {
                    let local = kind.strip_prefix(crate::model::SH).unwrap_or(kind);
                    let ok = match v {
                        Term::NamedNode(_) => {
                            matches!(local, "IRI" | "BlankNodeOrIRI" | "IRIOrLiteral")
                        }
                        Term::BlankNode(_) => {
                            matches!(local, "BlankNode" | "BlankNodeOrIRI" | "BlankNodeOrLiteral")
                        }
                        Term::Literal(_) => {
                            matches!(local, "Literal" | "BlankNodeOrLiteral" | "IRIOrLiteral")
                        }
                        #[allow(unreachable_patterns)]
                        _ => false,
                    };
                    if !ok {
                        out.push(self.result(
                            sid,
                            focus,
                            Some(v.clone()),
                            "NodeKindConstraintComponent",
                            format!("Value does not have node kind <{kind}>"),
                        ));
                    }
                }
            }
            Component::MinCount(n) => {
                if (values.len() as u64) < *n {
                    out.push(self.result(
                        sid,
                        focus,
                        None,
                        "MinCountConstraintComponent",
                        format!("Fewer than {n} values"),
                    ));
                }
            }
            Component::MaxCount(n) => {
                if (values.len() as u64) > *n {
                    out.push(self.result(
                        sid,
                        focus,
                        None,
                        "MaxCountConstraintComponent",
                        format!("More than {n} values"),
                    ));
                }
            }
            Component::MinExclusive(b) => self.range(
                sid,
                focus,
                values,
                b,
                &[Ordering::Greater],
                "MinExclusiveConstraintComponent",
                out,
            ),
            Component::MinInclusive(b) => self.range(
                sid,
                focus,
                values,
                b,
                &[Ordering::Greater, Ordering::Equal],
                "MinInclusiveConstraintComponent",
                out,
            ),
            Component::MaxExclusive(b) => self.range(
                sid,
                focus,
                values,
                b,
                &[Ordering::Less],
                "MaxExclusiveConstraintComponent",
                out,
            ),
            Component::MaxInclusive(b) => self.range(
                sid,
                focus,
                values,
                b,
                &[Ordering::Less, Ordering::Equal],
                "MaxInclusiveConstraintComponent",
                out,
            ),
            Component::MinLength(n) => {
                for v in values {
                    let ok = string_repr(v)
                        .map(|s| s.chars().count() as u64 >= *n)
                        .unwrap_or(false);
                    if !ok {
                        out.push(self.result(
                            sid,
                            focus,
                            Some(v.clone()),
                            "MinLengthConstraintComponent",
                            format!("Value is shorter than {n} characters"),
                        ));
                    }
                }
            }
            Component::MaxLength(n) => {
                for v in values {
                    let ok = string_repr(v)
                        .map(|s| s.chars().count() as u64 <= *n)
                        .unwrap_or(false);
                    if !ok {
                        out.push(self.result(
                            sid,
                            focus,
                            Some(v.clone()),
                            "MaxLengthConstraintComponent",
                            format!("Value is longer than {n} characters"),
                        ));
                    }
                }
            }
            Component::Pattern { source, flags } => {
                let re = self.regex_for(source, flags.as_deref());
                for v in values {
                    let ok = match (&re, string_repr(v)) {
                        (Some(re), Some(s)) => re.is_match(&s),
                        _ => false,
                    };
                    if !ok {
                        out.push(self.result(
                            sid,
                            focus,
                            Some(v.clone()),
                            "PatternConstraintComponent",
                            format!("Value does not match pattern \"{source}\""),
                        ));
                    }
                }
            }
            Component::LanguageIn(tags) => {
                for v in values {
                    let ok = match v {
                        Term::Literal(l) => match l.language() {
                            Some(lang) => tags.iter().any(|t| lang_matches(lang, t)),
                            None => false,
                        },
                        _ => false,
                    };
                    if !ok {
                        out.push(self.result(
                            sid,
                            focus,
                            Some(v.clone()),
                            "LanguageInConstraintComponent",
                            "Value language is not in the allowed list".into(),
                        ));
                    }
                }
            }
            Component::UniqueLang => {
                let mut counts: FxHashMap<String, usize> = FxHashMap::default();
                for v in values {
                    if let Term::Literal(l) = v {
                        if let Some(lang) = l.language() {
                            *counts.entry(lang.to_lowercase()).or_insert(0) += 1;
                        }
                    }
                }
                let mut langs: Vec<&String> = counts
                    .iter()
                    .filter(|(_, &c)| c > 1)
                    .map(|(l, _)| l)
                    .collect();
                langs.sort();
                for lang in langs {
                    out.push(self.result(
                        sid,
                        focus,
                        None,
                        "UniqueLangConstraintComponent",
                        format!("Language \"{lang}\" is used more than once"),
                    ));
                }
            }
            Component::Equals(p) => {
                let others = self.data.objects(focus, p);
                for v in values {
                    if !others.contains(v) {
                        out.push(self.result(
                            sid,
                            focus,
                            Some(v.clone()),
                            "EqualsConstraintComponent",
                            format!("Value missing from <{p}>"),
                        ));
                    }
                }
                for o in &others {
                    if !values.contains(o) {
                        out.push(self.result(
                            sid,
                            focus,
                            Some(o.clone()),
                            "EqualsConstraintComponent",
                            format!("Value of <{p}> missing from the path values"),
                        ));
                    }
                }
            }
            Component::Disjoint(p) => {
                for v in values {
                    if self.data.contains(focus, p, v) {
                        out.push(self.result(
                            sid,
                            focus,
                            Some(v.clone()),
                            "DisjointConstraintComponent",
                            format!("Value shared with <{p}>"),
                        ));
                    }
                }
            }
            // One result per FAILING PAIR (value, other) — the suite expects e.g.
            // `1 < "a"` and `1 < "b"` to report `sh:value 1` twice.
            Component::LessThan(p) => {
                let others = self.data.objects(focus, p);
                for v in values {
                    for o in &others {
                        if !matches!(cmp_terms(v, o), Some(Ordering::Less)) {
                            out.push(self.result(
                                sid,
                                focus,
                                Some(v.clone()),
                                "LessThanConstraintComponent",
                                format!("Value is not less than {o} (via <{p}>)"),
                            ));
                        }
                    }
                }
            }
            Component::LessThanOrEquals(p) => {
                let others = self.data.objects(focus, p);
                for v in values {
                    for o in &others {
                        if !matches!(cmp_terms(v, o), Some(Ordering::Less | Ordering::Equal)) {
                            out.push(self.result(
                                sid,
                                focus,
                                Some(v.clone()),
                                "LessThanOrEqualsConstraintComponent",
                                format!("Value is not less than or equal to {o} (via <{p}>)"),
                            ));
                        }
                    }
                }
            }
            Component::Not(inner) => {
                for v in values {
                    if self.conforms(v, *inner) {
                        out.push(self.result(
                            sid,
                            focus,
                            Some(v.clone()),
                            "NotConstraintComponent",
                            "Value conforms to the negated shape".into(),
                        ));
                    }
                }
            }
            Component::And(ids) => {
                for v in values {
                    if !ids.iter().all(|&s| self.conforms(v, s)) {
                        out.push(self.result(
                            sid,
                            focus,
                            Some(v.clone()),
                            "AndConstraintComponent",
                            "Value does not conform to every member shape".into(),
                        ));
                    }
                }
            }
            Component::Or(ids) => {
                for v in values {
                    if !ids.iter().any(|&s| self.conforms(v, s)) {
                        out.push(self.result(
                            sid,
                            focus,
                            Some(v.clone()),
                            "OrConstraintComponent",
                            "Value does not conform to any member shape".into(),
                        ));
                    }
                }
            }
            Component::Xone(ids) => {
                for v in values {
                    let n = ids.iter().filter(|&&s| self.conforms(v, s)).count();
                    if n != 1 {
                        out.push(self.result(
                            sid,
                            focus,
                            Some(v.clone()),
                            "XoneConstraintComponent",
                            format!("Value conforms to {n} member shapes, not exactly one"),
                        ));
                    }
                }
            }
            Component::Node(inner) => {
                for v in values {
                    if !self.conforms(v, *inner) {
                        out.push(self.result(
                            sid,
                            focus,
                            Some(v.clone()),
                            "NodeConstraintComponent",
                            "Value does not conform to the node shape".into(),
                        ));
                    }
                }
            }
            Component::Property(child) => {
                for v in values {
                    // [OPUS-4.8] Guard the direct validate_shape recursion against cyclic
                    // sh:property references (e.g. A sh:property B / B sh:property A): without the
                    // (focus, shape) re-entry check this overflows the stack. Re-entry into a
                    // (value, child) pair already on the stack counts as conforming (SHACL leaves
                    // recursion undefined and the validator treats a cycle as satisfied), so we
                    // skip it — emitting no further results — and record the cycle reach so any
                    // enclosing `conforms` frame skips its memo. Mirrors `conforms`'s guard. 1616.
                    if let Some(i) = self.stack.iter().position(|(n, s)| n == v && *s == *child) {
                        self.min_reentry = self.min_reentry.min(i);
                        continue;
                    }
                    self.stack.push((v.clone(), *child));
                    self.validate_shape(*child, v, out);
                    self.stack.pop();
                }
            }
            Component::Qualified {
                shape,
                min,
                max,
                disjoint,
                siblings,
            } => {
                let count = values
                    .iter()
                    .filter(|v| {
                        self.conforms(v, *shape)
                            && (!disjoint || !siblings.iter().any(|&s| self.conforms(v, s)))
                    })
                    .count() as u64;
                if let Some(min) = min {
                    if count < *min {
                        out.push(self.result(
                            sid,
                            focus,
                            None,
                            "QualifiedMinCountConstraintComponent",
                            format!("Fewer than {min} values conform to the qualified shape"),
                        ));
                    }
                }
                if let Some(max) = max {
                    if count > *max {
                        out.push(self.result(
                            sid,
                            focus,
                            None,
                            "QualifiedMaxCountConstraintComponent",
                            format!("More than {max} values conform to the qualified shape"),
                        ));
                    }
                }
            }
            Component::Closed { ignored } => {
                let shape = &self.shapes.shapes[sid];
                let mut allowed: Vec<String> = ignored
                    .iter()
                    .filter_map(|t| match t {
                        Term::NamedNode(n) => Some(n.as_str().to_string()),
                        _ => None,
                    })
                    .collect();
                for &child in &shape.property_children {
                    if let Some(Path::Predicate(p)) = &self.shapes.shapes[child].path {
                        allowed.push(p.clone());
                    }
                }
                for v in values {
                    for (p, o) in self.data.predicate_objects(v) {
                        let p_str = match &p {
                            Term::NamedNode(n) => n.as_str().to_string(),
                            _ => continue,
                        };
                        if !allowed.contains(&p_str) {
                            let mut r = self.result(
                                sid,
                                focus,
                                Some(o),
                                "ClosedConstraintComponent",
                                format!("Predicate <{p_str}> is not allowed (closed shape)"),
                            );
                            r.path = Some(Path::Predicate(p_str));
                            out.push(r);
                        }
                    }
                }
            }
            Component::HasValue(t) => {
                if !values.iter().any(|v| equal_value(v, t)) {
                    out.push(self.result(
                        sid,
                        focus,
                        None,
                        "HasValueConstraintComponent",
                        format!("Missing required value {t}"),
                    ));
                }
            }
            Component::In(list) => {
                for v in values {
                    if !list.iter().any(|m| equal_value(v, m)) {
                        out.push(self.result(
                            sid,
                            focus,
                            Some(v.clone()),
                            "InConstraintComponent",
                            "Value is not in the enumeration".into(),
                        ));
                    }
                }
            }
            // [OPUS-4.8] sh:sparql — SPARQL-based constraints (SHACL §5.2). The
            // sh:select runs against the data graph with $this pre-bound to the
            // focus node; every returned solution is one violation. Note this
            // evaluates over the FOCUS NODE (not the path value nodes): a sh:sparql
            // on a property shape sees `values` only through the query (e.g. by
            // re-traversing sh:path inside the SELECT), which matches the spec —
            // the constraint is responsible for its own value selection.
            Component::Sparql(idx) => self.eval_sparql(sid, focus, *idx, out),
            // [OPUS-4.8] SHACL §6: a SPARQL-based constraint COMPONENT that
            // activated on this shape. ASK validators run per value node
            // (ASK=false → violation); SELECT validators run per focus node
            // (each solution → violation). Parameters are pre-bound as $paramName.
            Component::CustomSparql { component, args } => {
                self.eval_custom_component(sid, focus, values, *component, args, out)
            }
            // [OPUS-4.8] (sq-mk9n, `shacl-af`) `sh:expression`
            // (`sh:ExpressionConstraintComponent`): a value node is violated when
            // the node expression does NOT evaluate to `{ true }` for that value.
            #[cfg(feature = "shacl-af")]
            Component::Expression(idx) => {
                let data = self.data.graph();
                let expr = &self.shapes.expressions[*idx];
                for v in values {
                    let result = crate::rules::eval_parsed_expr(data, self.shapes, expr, v);
                    if !crate::rules::is_true_result(&result) {
                        out.push(self.result(
                            sid,
                            focus,
                            Some(v.clone()),
                            "ExpressionConstraintComponent",
                            "Value does not satisfy the sh:expression".to_string(),
                        ));
                    }
                }
            }
        }
    }

    /// SHACL-SPARQL evaluation for one `sh:sparql` constraint occurrence.
    fn eval_sparql(
        &mut self,
        sid: usize,
        focus: &Term,
        idx: usize,
        out: &mut Vec<ValidationResult>,
    ) {
        let constraint = &self.shapes.sparql[idx];
        if constraint.deactivated {
            return;
        }
        let Some(prepared) = &constraint.prepared else {
            return; // ill-formed sh:select — skipped (lenient, per crate policy)
        };
        let component = sh("SPARQLConstraintComponent");
        let shape = &self.shapes.shapes[sid];
        let (shape_node, shape_path, severity, shape_messages) = (
            shape.node.clone(),
            shape.path.clone(),
            shape.severity.clone(),
            shape.messages.clone(),
        );
        let data = self.data.graph();
        prepared.evaluate(
            data,
            focus,
            constraint,
            |fields| ValidationResult {
                focus_node: focus.clone(),
                // A bound ?path overrides the shape's path; otherwise inherit the
                // (property-shape) path, if any.
                path: fields.path.or_else(|| shape_path.clone()),
                value: fields.value,
                source_shape: shape_node.clone(),
                source_component: component.clone(),
                severity: severity.clone(),
                messages: shape_messages.clone(),
                default_message: fields.default_message,
            },
            out,
        );
    }

    /// [OPUS-4.8] SHACL §6 evaluation for one activated SPARQL-based constraint
    /// component occurrence. The validator (kind-specific or generic) runs with
    /// `$this`, the bound parameter VALUES (`$paramName`) and (on a property
    /// shape) `$PATH` pre-bound; ASK validators additionally run per VALUE NODE
    /// with `$value` pre-bound (ASK=false → one violation for that value),
    /// SELECT validators run once per focus node (§5.2 solution→result mapping).
    fn eval_custom_component(
        &mut self,
        sid: usize,
        focus: &Term,
        values: &[Term],
        cidx: usize,
        args: &[Option<Term>],
        out: &mut Vec<ValidationResult>,
    ) {
        let comp: &ComponentDef = &self.shapes.components[cidx];
        let shape = &self.shapes.shapes[sid];
        let is_property_shape = shape.path.is_some();
        let Some(validator) = comp.validator_for(is_property_shape) else {
            return; // no usable validator for this shape kind (lenient)
        };
        let message = validator.message.clone();
        let component_iri = match &comp.node {
            Term::NamedNode(n) => n.as_str().to_string(),
            // A blank-node component has no IRI for sh:sourceConstraintComponent;
            // fall back to the generic SPARQL component IRI.
            _ => sh("SPARQLConstraintComponent"),
        };

        // Parameter pre-bindings, common to every value node / the SELECT run.
        // `$paramName` ← bound arg; absent optionals contribute no binding.
        let mut param_bindings: Vec<(String, Term)> = Vec::with_capacity(args.len());
        for (p, arg) in comp.parameters.iter().zip(args.iter()) {
            if let Some(term) = arg {
                param_bindings.push((p.var.clone(), term.clone()));
            }
        }
        // NOTE (scope): `$PATH` pre-binding for `sh:propertyValidator` is NOT done.
        // `$PATH` is a SPARQL property PATH (not a term) so it cannot be supplied
        // via the VALUES table the parameter/`$this`/`$value` bindings use, and the
        // component's validator query is pre-parsed (shared across shapes), so the
        // textual `$PATH` substitution the §5.2 `sh:sparql` path performs cannot be
        // applied per-shape here. ASK validators get `$value` (the value node) and
        // the parameters, which covers the common property-validator pattern. See
        // the module/TODO scope note.
        let shape_path = shape.path.clone();
        let severity = shape.severity.clone();
        let shape_messages = shape.messages.clone();
        let shape_node = shape.node.clone();
        let data = self.data.graph();

        match &validator.prepared {
            PreparedValidator::Ask(query) => {
                // One ASK per value node; ASK=false (constraint not satisfied) is a
                // violation reported on that value (SHACL §6.3).
                for v in values {
                    let mut bindings: Vec<(&str, &Term)> = vec![("this", focus), ("value", v)];
                    for (name, term) in &param_bindings {
                        bindings.push((name.as_str(), term));
                    }
                    if crate::sparql::ask_violates(query, data, &bindings) {
                        let default_message = match &message {
                            // {$param}/{$value}/{$this} substitution from the pre-bound terms.
                            Some(tpl) => crate::sparql::substitute_bindings(tpl, &bindings),
                            None => format!(
                                "Value does not satisfy constraint component <{component_iri}>"
                            ),
                        };
                        out.push(ValidationResult {
                            focus_node: focus.clone(),
                            path: shape_path.clone(),
                            value: Some(v.clone()),
                            source_shape: shape_node.clone(),
                            source_component: component_iri.clone(),
                            severity: severity.clone(),
                            messages: shape_messages.clone(),
                            default_message,
                        });
                    }
                }
            }
            PreparedValidator::Select(query) => {
                // One SELECT per focus node; each solution is a violation.
                let mut bindings: Vec<(&str, &Term)> = vec![("this", focus)];
                for (name, term) in &param_bindings {
                    bindings.push((name.as_str(), term));
                }
                crate::sparql::select_validate(
                    query,
                    data,
                    focus,
                    &bindings,
                    message.as_deref(),
                    |fields: ComponentResultFields| ValidationResult {
                        focus_node: fields.focus,
                        path: fields.path.or_else(|| shape_path.clone()),
                        value: fields.value,
                        source_shape: shape_node.clone(),
                        source_component: component_iri.clone(),
                        severity: severity.clone(),
                        messages: shape_messages.clone(),
                        default_message: fields.default_message,
                    },
                    out,
                );
            }
        }
    }

    #[allow(clippy::too_many_arguments)] // one call site per range component; a struct would obscure it
    fn range(
        &mut self,
        sid: usize,
        focus: &Term,
        values: &[Term],
        bound: &Term,
        accept: &[Ordering],
        component: &str,
        out: &mut Vec<ValidationResult>,
    ) {
        for v in values {
            let ok = match cmp_terms(v, bound) {
                Some(ord) => accept.contains(&ord),
                None => false, // not comparable: constraint not satisfied
            };
            if !ok {
                out.push(self.result(
                    sid,
                    focus,
                    Some(v.clone()),
                    component,
                    format!("Value is out of range relative to {bound}"),
                ));
            }
        }
    }

    fn regex_for(&mut self, source: &str, flags: Option<&str>) -> Option<regex::Regex> {
        let key = format!("{}\u{0}{}", source, flags.unwrap_or(""));
        self.regexes
            .entry(key)
            .or_insert_with(|| {
                let mut inline = String::new();
                for f in flags.unwrap_or("").chars() {
                    if matches!(f, 'i' | 'm' | 's' | 'x' | 'U') {
                        inline.push(f);
                    }
                }
                let pat = if inline.is_empty() {
                    source.to_string()
                } else {
                    format!("(?{inline}){source}")
                };
                regex::Regex::new(&pat).ok()
            })
            .clone()
    }
}

/// The string a value node contributes to sh:minLength/maxLength/pattern:
/// the IRI string or the literal's lexical form; blank nodes have none (fail).
fn string_repr(t: &Term) -> Option<String> {
    match t {
        Term::NamedNode(n) => Some(n.as_str().to_string()),
        Term::Literal(l) => Some(l.value().to_string()),
        _ => None,
    }
}

/// HTTP-style basic language-range match: exact (case-insensitive) or prefix
/// followed by `-` (so `en` matches `en-GB`); `*` matches any tag.
fn lang_matches(lang: &str, range: &str) -> bool {
    if range == "*" {
        return !lang.is_empty();
    }
    let lang = lang.to_ascii_lowercase();
    let range = range.to_ascii_lowercase();
    lang == range || (lang.starts_with(&range) && lang.as_bytes().get(range.len()) == Some(&b'-'))
}

/// SHACL "equal values" (sh:hasValue / sh:in): same term, or literals equal
/// under SPARQL `=` value semantics (numeric / boolean / date-time families).
fn equal_value(a: &Term, b: &Term) -> bool {
    if a == b {
        return true;
    }
    matches!(cmp_terms(a, b), Some(Ordering::Equal))
}

/// SPARQL-operator-style comparison of two terms; `None` when not comparable
/// (non-literals, or literals of incompatible / unordered types).
fn cmp_terms(a: &Term, b: &Term) -> Option<Ordering> {
    match (a, b) {
        (Term::Literal(la), Term::Literal(lb)) => cmp_literals(la, lb),
        _ => None,
    }
}

fn cmp_literals(a: &Literal, b: &Literal) -> Option<Ordering> {
    let (da, db) = (a.datatype(), b.datatype());
    let (da, db) = (da.as_str(), db.as_str());
    if is_numeric(da) && is_numeric(db) {
        let fa: f64 = a.value().parse().ok()?;
        let fb: f64 = b.value().parse().ok()?;
        return fa.partial_cmp(&fb);
    }
    if a.language().is_none()
        && b.language().is_none()
        && da == xsd("string")
        && db == xsd("string")
    {
        return Some(a.value().cmp(b.value()));
    }
    if da == xsd("boolean") && db == xsd("boolean") {
        let pa = parse_bool(a.value())?;
        let pb = parse_bool(b.value())?;
        return Some(pa.cmp(&pb));
    }
    if is_date_time(da) && is_date_time(db) {
        let (ta, tza) = timestamp(a.value(), da)?;
        let (tb, tzb) = timestamp(b.value(), db)?;
        if tza != tzb {
            // XSD order between a timezoned and an untimezoned value is
            // INDETERMINATE (within the ±14h window) — not comparable.
            return None;
        }
        return ta.partial_cmp(&tb);
    }
    None
}

fn xsd(local: &str) -> String {
    format!("{XSD}{local}")
}

fn is_numeric(dt: &str) -> bool {
    let Some(local) = dt.strip_prefix(XSD) else {
        return false;
    };
    matches!(
        local,
        "integer"
            | "decimal"
            | "double"
            | "float"
            | "long"
            | "int"
            | "short"
            | "byte"
            | "nonNegativeInteger"
            | "positiveInteger"
            | "nonPositiveInteger"
            | "negativeInteger"
            | "unsignedLong"
            | "unsignedInt"
            | "unsignedShort"
            | "unsignedByte"
    )
}

fn is_date_time(dt: &str) -> bool {
    let Some(local) = dt.strip_prefix(XSD) else {
        return false;
    };
    matches!(local, "dateTime" | "date" | "time" | "dateTimeStamp")
}

fn parse_bool(v: &str) -> Option<bool> {
    match v {
        "true" | "1" => Some(true),
        "false" | "0" => Some(false),
        _ => None,
    }
}

/// Seconds-since-epoch of an XSD date/time lexical value plus whether it
/// carries an explicit timezone (an untimezoned value is normalised AS IF UTC,
/// but only values agreeing on timezone presence are mutually comparable).
fn timestamp(value: &str, dt: &str) -> Option<(f64, bool)> {
    let local = dt.strip_prefix(XSD)?;
    let v = value.trim();
    let (date_part, rest) = match local {
        "time" => ("2000-01-01", v),
        "date" => {
            // The date may carry a timezone suffix directly.
            let split = date_tz_split(v);
            (&v[..split], &v[split..])
        }
        _ => match v.find('T') {
            Some(i) => (&v[..i], &v[i + 1..]),
            None => return None,
        },
    };
    let (y, m, d) = parse_date(date_part)?;
    let (mut rest, mut secs) = (rest, 0.0f64);
    if local != "date" {
        let tz_at = rest
            .char_indices()
            .find(|(i, c)| *c == 'Z' || ((*c == '+' || *c == '-') && *i > 0))
            .map(|(i, _)| i)
            .unwrap_or(rest.len());
        let time_part = &rest[..tz_at];
        rest = &rest[tz_at..];
        // [OPUS-4.8] xsd:dateTime / dateTimeStamp REQUIRE a time after the 'T'; the old code left
        // secs=0 for an empty time part, accepting "2024-01-01T" / "2024-01-01TZ". Reject it. 1616.
        if time_part.is_empty() {
            return None;
        }
        let mut it = time_part.split(':');
        let h: f64 = parse_time_field(it.next()?, 2)?;
        let mi: f64 = parse_time_field(it.next()?, 2)?;
        // Seconds may carry a fractional part (ss.sss…); validate the integral part shape.
        let s = parse_seconds_field(it.next().unwrap_or("00"))?;
        if it.next().is_some() {
            return None;
        }
        // [OPUS-4.8] Range-check the fields: hour 0..=24 (24 only as 24:00:00), minute/second
        // 0..=59. The old code accepted out-of-range values like 99:99:99. 1616.
        if h > 24.0 || mi > 59.0 || s >= 60.0 || (h == 24.0 && (mi != 0.0 || s != 0.0)) {
            return None;
        }
        secs = h * 3600.0 + mi * 60.0 + s;
    }
    let tz_offset_secs = parse_tz(rest)?;
    let has_tz = !rest.is_empty();
    // [OPUS-4.8] xsd:dateTimeStamp requires an explicit timezone; a timezone-less value is NOT in
    // its lexical space (this is the sole lexical difference from xsd:dateTime). See review 1616.
    if local == "dateTimeStamp" && !has_tz {
        return None;
    }
    Some((
        days_from_civil(y, m, d) as f64 * 86_400.0 + secs - tz_offset_secs,
        has_tz,
    ))
}

/// The byte index where a date lexical's optional timezone suffix starts.
fn date_tz_split(v: &str) -> usize {
    if v.ends_with('Z') {
        return v.len() - 1;
    }
    // ±hh:mm — but the leading sign of a negative year is not a timezone.
    if v.len() > 6 {
        let tail = &v[v.len() - 6..];
        if (tail.starts_with('+') || tail.starts_with('-')) && tail.as_bytes()[3] == b':' {
            return v.len() - 6;
        }
    }
    v.len()
}

fn parse_tz(s: &str) -> Option<f64> {
    if s.is_empty() || s == "Z" {
        return Some(0.0);
    }
    let (sign, rest) = match s.as_bytes()[0] {
        b'+' => (1.0, &s[1..]),
        b'-' => (-1.0, &s[1..]),
        _ => return None,
    };
    let mut it = rest.split(':');
    // [OPUS-4.8] Enforce the fixed ±hh:mm shape and the XSD timezone range (≤ ±14:00). The old
    // bare f64 parse accepted "+9:5", "+99:99", etc. 1616.
    let h = parse_time_field(it.next()?, 2)?;
    let m = parse_time_field(it.next()?, 2)?;
    if it.next().is_some() || h > 14.0 || m > 59.0 || (h == 14.0 && m != 0.0) {
        return None;
    }
    Some(sign * (h * 3600.0 + m * 60.0))
}

/// [OPUS-4.8] Parse a fixed-width all-digit time field (hh / mm) of exactly `width` digits.
/// Rejects signs, whitespace, and wrong widths the bare f64 parse would otherwise accept. 1616.
fn parse_time_field(s: &str, width: usize) -> Option<f64> {
    if s.len() != width || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse().ok()
}

/// [OPUS-4.8] Parse the seconds field `ss` or `ss.sss…`: two leading digits, an optional fraction
/// of one-or-more digits. Returns the numeric value (range-checked by the caller). 1616.
fn parse_seconds_field(s: &str) -> Option<f64> {
    let (int_part, frac_ok) = match s.split_once('.') {
        Some((i, f)) => (i, !f.is_empty() && f.bytes().all(|b| b.is_ascii_digit())),
        None => (s, true),
    };
    if !frac_ok || int_part.len() != 2 || !int_part.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse().ok()
}

fn parse_date(s: &str) -> Option<(i64, u32, u32)> {
    let (neg, s) = match s.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, s),
    };
    let mut it = s.split('-');
    let ys = it.next()?;
    let ms = it.next()?;
    let ds = it.next()?;
    // [OPUS-4.8] XSD lexical space is fixed-width: yyyy (≥4 digits), mm, dd, all ASCII digits.
    // The plain f64/u32 parse below accepts e.g. "+04" or whitespace; enforce the shape. 1616.
    if ys.len() < 4
        || !ys.bytes().all(|b| b.is_ascii_digit())
        || ms.len() != 2
        || !ms.bytes().all(|b| b.is_ascii_digit())
        || ds.len() != 2
        || !ds.bytes().all(|b| b.is_ascii_digit())
    {
        return None;
    }
    let y: i64 = ys.parse().ok()?;
    let m: u32 = ms.parse().ok()?;
    let d: u32 = ds.parse().ok()?;
    // [OPUS-4.8] Reject impossible calendar days (e.g. 2024-02-30, 2023-02-29, 2024-04-31): the
    // old check only rejected d>31, so February 30th and April 31st passed as well-formed. 1616.
    if it.next().is_some() || m == 0 || m > 12 || d == 0 || d > days_in_month(y, m) {
        return None;
    }
    Some((if neg { -y } else { y }, m, d))
}

/// [OPUS-4.8] Days in a (proleptic-Gregorian) month, accounting for leap years. The `y` is the
/// signed calendar year as written (sign applied by the caller); leap rules are sign-symmetric for
/// the purpose of day-count here. See review 1616.
fn days_in_month(y: i64, m: u32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// Days since 1970-01-01 of a proleptic-Gregorian civil date (Howard Hinnant's
/// `days_from_civil`).
fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = y - i64::from(m <= 2);
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = i64::from(if m > 2 { m - 3 } else { m + 9 });
    let doy = (153 * mp + 2) / 5 + i64::from(d) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Is the literal's lexical form in the lexical space of its (known XSD)
/// datatype? Unknown datatypes are accepted (no lexical check possible).
fn well_formed(l: &Literal) -> bool {
    let dt = l.datatype();
    let v = l.value();
    let Some(local) = dt.as_str().strip_prefix(XSD) else {
        return dt.as_str() != RDF_LANG_STRING || l.language().is_some();
    };
    match local {
        "integer" => parse_integer(v).is_some(),
        "long" => in_range(v, i64::MIN as i128, i64::MAX as i128),
        "int" => in_range(v, i32::MIN as i128, i32::MAX as i128),
        "short" => in_range(v, i16::MIN as i128, i16::MAX as i128),
        "byte" => in_range(v, i8::MIN as i128, i8::MAX as i128),
        "nonNegativeInteger" => parse_integer(v).map(|n| n >= 0).unwrap_or(false),
        "positiveInteger" => parse_integer(v).map(|n| n > 0).unwrap_or(false),
        "nonPositiveInteger" => parse_integer(v).map(|n| n <= 0).unwrap_or(false),
        "negativeInteger" => parse_integer(v).map(|n| n < 0).unwrap_or(false),
        "unsignedLong" => in_range(v, 0, u64::MAX as i128),
        "unsignedInt" => in_range(v, 0, u32::MAX as i128),
        "unsignedShort" => in_range(v, 0, u16::MAX as i128),
        "unsignedByte" => in_range(v, 0, u8::MAX as i128),
        "decimal" => is_decimal(v),
        "float" | "double" => is_float(v),
        "boolean" => parse_bool(v).is_some(),
        "dateTime" | "dateTimeStamp" | "date" | "time" => timestamp(v, dt.as_str()).is_some(),
        _ => true,
    }
}

fn parse_integer(v: &str) -> Option<i128> {
    let body = v.strip_prefix(['+', '-']).unwrap_or(v);
    if body.is_empty() || !body.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    v.parse().ok()
}

fn in_range(v: &str, lo: i128, hi: i128) -> bool {
    parse_integer(v)
        .map(|n| n >= lo && n <= hi)
        .unwrap_or(false)
}

fn is_decimal(v: &str) -> bool {
    let body = v.strip_prefix(['+', '-']).unwrap_or(v);
    if body.is_empty() {
        return false;
    }
    let mut dots = 0;
    let mut digits = 0;
    for b in body.bytes() {
        match b {
            b'.' => dots += 1,
            b'0'..=b'9' => digits += 1,
            _ => return false,
        }
    }
    dots <= 1 && digits > 0
}

fn is_float(v: &str) -> bool {
    if matches!(v, "NaN" | "INF" | "-INF" | "+INF") {
        return true;
    }
    let (mantissa, exp) = match v.find(['e', 'E']) {
        Some(i) => (&v[..i], Some(&v[i + 1..])),
        None => (v, None),
    };
    if !is_decimal(mantissa) {
        return false;
    }
    match exp {
        None => true,
        Some(e) => parse_integer(e).is_some(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xsd_well_formedness() {
        let lit = |v: &str, dt: &str| {
            Literal::new_typed_literal(v, oxrdf::NamedNode::new(xsd(dt)).unwrap())
        };
        assert!(well_formed(&lit("42", "integer")));
        assert!(!well_formed(&lit("aaa", "integer")));
        assert!(well_formed(&lit("-3.14", "decimal")));
        assert!(!well_formed(&lit("3.1.4", "decimal")));
        assert!(well_formed(&lit("1.0e6", "double")));
        assert!(well_formed(&lit("INF", "float")));
        assert!(well_formed(&lit("2024-02-29", "date")));
        assert!(!well_formed(&lit("2024-13-01", "date")));
        assert!(well_formed(&lit("2024-02-29T12:00:00Z", "dateTime")));

        // [OPUS-4.8] Stricter XSD date/time lexical validation (review 1616):
        // impossible calendar days are rejected.
        assert!(
            !well_formed(&lit("2023-02-29", "date")),
            "Feb 29 in a non-leap year"
        );
        assert!(
            !well_formed(&lit("2024-02-30", "date")),
            "Feb 30 never exists"
        );
        assert!(
            !well_formed(&lit("2024-04-31", "date")),
            "April has 30 days"
        );
        assert!(well_formed(&lit("2024-04-30", "date")));
        assert!(
            well_formed(&lit("2000-02-29", "date")),
            "2000 is a leap year (÷400)"
        );
        assert!(
            !well_formed(&lit("1900-02-29", "date")),
            "1900 is not (÷100, ¬÷400)"
        );
        // dateTime requires a time component after 'T'.
        assert!(
            !well_formed(&lit("2024-01-01T", "dateTime")),
            "missing time part"
        );
        assert!(
            !well_formed(&lit("2024-01-01TZ", "dateTime")),
            "empty time part"
        );
        // Out-of-range hour/minute/second.
        assert!(
            !well_formed(&lit("2024-01-01T25:00:00", "dateTime")),
            "hour 25"
        );
        assert!(
            !well_formed(&lit("2024-01-01T12:60:00", "dateTime")),
            "minute 60"
        );
        assert!(
            !well_formed(&lit("2024-01-01T12:00:61", "dateTime")),
            "second 61"
        );
        assert!(
            well_formed(&lit("2024-01-01T24:00:00", "dateTime")),
            "24:00:00 is legal"
        );
        assert!(
            !well_formed(&lit("2024-01-01T24:00:01", "dateTime")),
            "24:00:01 is not"
        );
        assert!(
            well_formed(&lit("2024-01-01T12:30:45.5", "dateTime")),
            "fractional seconds"
        );
        // Out-of-range / malformed timezone.
        assert!(
            !well_formed(&lit("2024-01-01T12:00:00+15:00", "dateTime")),
            "tz > ±14:00"
        );
        assert!(well_formed(&lit("2024-01-01T12:00:00+14:00", "dateTime")));
        // dateTimeStamp requires an explicit timezone; dateTime does not.
        assert!(
            well_formed(&lit("2024-01-01T12:00:00", "dateTime")),
            "tz-less dateTime ok"
        );
        assert!(
            !well_formed(&lit("2024-01-01T12:00:00", "dateTimeStamp")),
            "tz-less dateTimeStamp is NOT in its lexical space"
        );
        assert!(well_formed(&lit("2024-01-01T12:00:00Z", "dateTimeStamp")));
    }

    #[test]
    fn date_time_ordering() {
        let cmp = |a: &str, b: &str, dt: &str| {
            timestamp(a, &xsd(dt))
                .unwrap()
                .0
                .partial_cmp(&timestamp(b, &xsd(dt)).unwrap().0)
                .unwrap()
        };
        assert_eq!(cmp("2024-01-01", "2024-01-02", "date"), Ordering::Less);
        assert_eq!(
            cmp(
                "2024-01-01T10:00:00+01:00",
                "2024-01-01T09:00:00Z",
                "dateTime"
            ),
            Ordering::Equal
        );
        // Timezone presence must agree for the values to be comparable.
        let lit = |v: &str| {
            Literal::new_typed_literal(v, oxrdf::NamedNode::new(xsd("dateTime")).unwrap())
        };
        assert_eq!(
            cmp_literals(
                &lit("2002-10-10T12:00:00-05:00"),
                &lit("2002-10-10T12:00:00")
            ),
            None
        );
    }
}
