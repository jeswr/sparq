//! Inline `OVER(...)` window-clause SYNTAX (sq-h564) — a NON-STANDARD, OPT-IN
//! extension on top of the programmatic window pass in [`crate::window`].
//!
//! [OPUS-4.8] **There is still no W3C-REC syntax for SPARQL window functions**
//! (see the [`crate::window`] module docs). SPARQL 1.1 has no
//! `OVER (PARTITION BY … ORDER BY …)` clause and no community SEP defines one;
//! engines that offer windowing each invent their own surface. PR #370 landed
//! windowing as a *programmatic* pass over a [`QueryResult`] to keep the engine's
//! SPARQL surface exactly SPARQL 1.1 (no conformance-harness regression). This
//! module adds an INLINE query syntax for the same pass **without touching the
//! (W3C-conformance-tracking) vendored `spargebra` parser**: it is a *source
//! rewrite* in front of the ordinary engine, only enabled on the dedicated
//! [`query_over`] / [`query_over_with_budget`] entry points. Every other entry
//! point — and the plain `spargebra` parser — is untouched, so an `OVER(…)`
//! clause is still a parse error on the standard surface (conformance is
//! unaffected).
//!
//! ## Syntax accepted
//!
//! Inside a top-level `SELECT` projection, a window expression is written
//!
//! ```text
//! ( <FN>() OVER ( [PARTITION BY ?v …] [ORDER BY [ASC|DESC]? ?v …] ) AS ?out )
//! ```
//!
//! where `<FN>` is one of `ROW_NUMBER`, `RANK`, `DENSE_RANK` (case-insensitive),
//! the empty `()` after the function name is required, and the `AS ?out` alias is
//! required. This mirrors the SQL:2003 / Stardog / AnzoGraph window surface (the
//! same model the programmatic [`crate::window`] pass follows) restricted to the
//! three ranking functions.
//!
//! Example:
//!
//! ```text
//! SELECT ?emp ?dept ?sales
//!        (ROW_NUMBER() OVER (PARTITION BY ?dept ORDER BY DESC(?sales)) AS ?rn)
//! WHERE { ?emp :dept ?dept ; :sales ?sales }
//! ```
//!
//! Both `ORDER BY DESC(?sales)` and `ORDER BY ?sales DESC` (the SQL trailing-
//! keyword form) are accepted; bare `?sales` is ASC. `PARTITION BY` and the whole
//! `ORDER BY` are each optional (an absent `PARTITION BY` is a single partition
//! over the whole result; an absent `ORDER BY` makes every row a peer).
//!
//! ## Covered vs deferred
//!
//! * **Covered:** `ROW_NUMBER()`, `RANK()`, `DENSE_RANK()`, `PARTITION BY` over
//!   one or more variables, `ORDER BY` over one or more keys (ASC/DESC, both
//!   `DESC(?v)` and `?v DESC` spellings). An `ORDER BY` key may be a **computed
//!   SPARQL expression** (sq-c1jv) — e.g. `ORDER BY (?a + ?b)`, `DESC(?sales + 0)`,
//!   or `STRLEN(?s)` — not just a projected variable; the rewriter binds each such
//!   expression to a fresh helper var via `(<expr> AS ?__win_orderkeyN)` in the
//!   inner SELECT, orders on that helper, and drops it from the output. Multiple
//!   window columns in one SELECT. Other (non-window) projection items — plain
//!   variables and `(expr AS ?v)` aliases — pass through to the engine unchanged.
//! * **Deferred (beaded):** windowed AGGREGATES inline (`SUM(?x) OVER (…)` etc.),
//!   explicit window frames (`ROWS BETWEEN …`), `LAG`/`LEAD`/`NTILE`, a named
//!   `WINDOW` clause, and `PARTITION BY` over a computed expression (only ORDER BY
//!   keys may be expressions). Those remain available via the programmatic
//!   [`crate::window`] API.

use oxrdf::Variable;
use sparq_core::Graph;

use crate::window::{apply_window, SortKey, WindowFunction, WindowSpec};
use crate::{QueryBudget, QueryResult};

/// One parsed inline window item lifted out of a SELECT projection: the function,
/// its `PARTITION BY` / `ORDER BY` keys, and the output alias.
#[derive(Debug, Clone)]
struct WindowItem {
    func: RankFunc,
    partition_by: Vec<String>,
    order_by: Vec<OrderKey>,
    out: String,
}

/// One `ORDER BY` sort key inside an `OVER ( … )` spec: either a bare projected
/// variable or a computed SPARQL expression (sq-c1jv). [OPUS-4.8] A `Sort::Expr` is bound to
/// a fresh helper var via a `(<expr> AS ?__win_orderkeyN)` projection item in the
/// rewritten inner SELECT, then the window orders on that helper var and the
/// helper is dropped from the final output. A `Sort::Var` is ordered on directly.
#[derive(Debug, Clone, PartialEq, Eq)]
struct OrderKey {
    sort: Sort,
    /// `true` = descending (`DESC`), `false` = ascending (`ASC`, the default).
    descending: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Sort {
    /// A bare projected variable name (no leading `?`/`$`), ordered on directly.
    Var(String),
    /// A computed SPARQL expression (the source text inside the key, e.g.
    /// `?a + ?b`), bound to a fresh helper var in the rewritten inner SELECT.
    Expr(String),
}

/// A parsed `OVER ( … )` spec: the `PARTITION BY` variable names and the
/// `ORDER BY` sort keys.
type OverSpec = (Vec<String>, Vec<OrderKey>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RankFunc {
    RowNumber,
    Rank,
    DenseRank,
}

impl RankFunc {
    fn parse(name: &str) -> Option<Self> {
        match name.to_ascii_uppercase().as_str() {
            "ROW_NUMBER" => Some(Self::RowNumber),
            "RANK" => Some(Self::Rank),
            "DENSE_RANK" => Some(Self::DenseRank),
            _ => None,
        }
    }
    fn to_window_function(self) -> WindowFunction {
        match self {
            Self::RowNumber => WindowFunction::RowNumber,
            Self::Rank => WindowFunction::Rank,
            Self::DenseRank => WindowFunction::DenseRank,
        }
    }
}

/// [`crate::query`] extended with inline `OVER(…)` window syntax. A query with no
/// `OVER` clause is run by the ordinary engine unchanged (so this is a strict
/// superset of [`crate::query`] on the syntactic level — a non-window query is
/// byte-for-byte the same). See the module docs for the exact syntax.
pub fn query_over(graph: &Graph, sparql: &str) -> Result<QueryResult, String> {
    query_over_with_budget(graph, sparql, &QueryBudget::unlimited())
}

/// [`query_over`] under a cooperative [`QueryBudget`]. The budget governs the
/// rewritten inner SELECT; the window pass itself is a post-pass over the already
/// budget-bounded result.
pub fn query_over_with_budget(
    graph: &Graph,
    sparql: &str,
    budget: &QueryBudget,
) -> Result<QueryResult, String> {
    let Some(plan) = extract_windows(sparql)? else {
        // No OVER clause: ordinary SPARQL, evaluated by the unmodified engine.
        return crate::query_with_budget(graph, sparql, budget);
    };
    // Run the rewritten (window-stripped) SELECT, then apply each window spec.
    let mut result = crate::query_with_budget(graph, &plan.rewritten, budget)?;
    for item in &plan.windows {
        let spec = WindowSpec {
            partition_by: item.partition_by.iter().map(|v| var(v)).collect::<Result<_, _>>()?,
            order_by: item
                .order_by
                .iter()
                .map(|key| {
                    // After `extract_windows`, every ORDER BY key resolves to a
                    // variable (a plain var, or the helper bound to a computed expr).
                    let Sort::Var(v) = &key.sort else {
                        return Err(
                            "inline OVER: internal error — unresolved ORDER BY expression".into(),
                        );
                    };
                    Ok(SortKey { var: var(v)?, descending: key.descending })
                })
                .collect::<Result<_, String>>()?,
            function: item.func.to_window_function(),
            new_var: var(&item.out)?,
        };
        result = apply_window(&result, &spec)?;
    }
    // Project to exactly the columns the original SELECT named, in order. The
    // rewritten inner query may have introduced helper columns (partition/order
    // variables referenced only by a window) that the user did not ask for.
    project(result, &plan.output_vars)
}

fn var(name: &str) -> Result<Variable, String> {
    Variable::new(name).map_err(|e| format!("inline OVER: invalid variable ?{name}: {e}"))
}

/// The output of stripping windows from a SELECT: the rewritten plain-SPARQL
/// query, the window items to apply, and the final output column order.
struct RewritePlan {
    rewritten: String,
    windows: Vec<WindowItem>,
    output_vars: Vec<String>,
}

/// Restricts `result` to the columns named in `output_vars` (in that order). An
/// output variable absent from the result is an error (it was never produced).
fn project(result: QueryResult, output_vars: &[String]) -> Result<QueryResult, String> {
    // SELECT * (empty output_vars sentinel handled by the caller) keeps all columns.
    if output_vars.is_empty() {
        return Ok(result);
    }
    let idx: Vec<usize> = output_vars
        .iter()
        .map(|name| {
            result
                .vars
                .iter()
                .position(|v| v.as_str() == name)
                .ok_or_else(|| format!("inline OVER: output ?{name} was not produced"))
        })
        .collect::<Result<_, _>>()?;
    let vars = idx.iter().map(|&i| result.vars[i].clone()).collect();
    let rows = result
        .rows
        .into_iter()
        .map(|row| idx.iter().map(|&i| row[i].clone()).collect())
        .collect();
    Ok(QueryResult { vars, rows })
}

/// Scans `sparql` for inline `OVER(…)` window items in the top-level SELECT
/// projection. Returns `Ok(None)` when there is no `OVER` clause (the query is
/// ordinary SPARQL and must be run unmodified), or `Ok(Some(plan))` with the
/// rewritten query + extracted windows. `Err` for malformed window syntax.
fn extract_windows(sparql: &str) -> Result<Option<RewritePlan>, String> {
    // Fast path: no `OVER` token at all → ordinary SPARQL. The check is a
    // word-boundary, case-insensitive scan so it does not trip on e.g. `?over`.
    if !contains_keyword(sparql, "OVER") {
        return Ok(None);
    }
    // Locate the top-level SELECT projection: between the SELECT keyword (after an
    // optional DISTINCT/REDUCED) and the WHERE clause / the `{` that opens it.
    let (sel_start, proj_start) = find_select_projection_start(sparql)
        .ok_or("inline OVER: could not locate the SELECT projection")?;
    let proj_end = find_projection_end(sparql, proj_start)
        .ok_or("inline OVER: could not locate the end of the SELECT projection (missing WHERE/{)")?;

    let projection = &sparql[proj_start..proj_end];
    let items = split_projection_items(projection)?;

    let mut windows = Vec::new();
    let mut output_vars = Vec::new();
    let mut helper_vars: Vec<String> = Vec::new();
    // Inner-SELECT `(<expr> AS ?helper)` bindings created for computed ORDER BY
    // keys (sq-c1jv), in allocation order. Held separately from `helper_vars`
    // (which are plain `?var` projections) because they must be emitted into the
    // inner projection even under `SELECT *`.
    let mut expr_bindings: Vec<String> = Vec::new();
    let mut next_expr_key = 0usize;
    let mut rewritten_items: Vec<String> = Vec::new();
    let mut select_star = false;

    for item in &items {
        let t = item.trim();
        if t == "*" {
            select_star = true;
            continue;
        }
        if let Some(mut win) = parse_window_item(t)? {
            // PARTITION BY vars must survive the inner SELECT so the post-pass can
            // read them; project them through.
            for v in &win.partition_by {
                if !helper_vars.contains(v) {
                    helper_vars.push(v.clone());
                }
            }
            // Resolve each ORDER BY key to a concrete variable the post-pass can
            // order on: a plain var is projected through; a computed expression is
            // bound to a fresh helper var via `(<expr> AS ?__win_orderkeyN)` in the
            // inner SELECT, then the key orders on that helper (dropped from output).
            for key in &mut win.order_by {
                match &key.sort {
                    Sort::Var(v) => {
                        if !helper_vars.contains(v) {
                            helper_vars.push(v.clone());
                        }
                    }
                    Sort::Expr(expr) => {
                        let helper = format!("__win_orderkey{next_expr_key}");
                        next_expr_key += 1;
                        expr_bindings.push(format!("({expr} AS ?{helper})"));
                        key.sort = Sort::Var(helper);
                    }
                }
            }
            output_vars.push(win.out.clone());
            windows.push(win);
        } else {
            // A non-window projection item: a bare ?var or an (expr AS ?v). Keep it
            // verbatim in the inner query and record its OUTPUT variable.
            rewritten_items.push(t.to_string());
            output_vars.push(projection_item_output_var(t)?);
        }
    }

    if windows.is_empty() {
        // `OVER` appeared as a token but not as a window clause we recognise
        // (e.g. a variable or IRI containing the letters). Leave the query alone.
        return Ok(None);
    }

    // Build the inner projection: every non-window item, the `(<expr> AS ?h)`
    // bindings for computed ORDER BY keys, plus any plain helper var a window needs
    // that is not already projected. With `SELECT *` the inner query keeps `*` (all
    // in-scope vars, covering the plain helpers) but STILL needs the expr bindings.
    let mut inner_proj = rewritten_items.clone();
    inner_proj.extend(expr_bindings.iter().cloned());
    if select_star {
        inner_proj.insert(0, "*".to_string());
    } else {
        for h in &helper_vars {
            let already = rewritten_items.iter().any(|it| projection_item_mentions_output(it, h));
            if !already {
                inner_proj.push(format!("?{h}"));
            }
        }
    }
    if inner_proj.is_empty() {
        return Err("inline OVER: the rewritten query has no projected columns".into());
    }

    let inner = inner_proj.join(" ");
    let rewritten = format!("{} {} {}", &sparql[..proj_start], inner, &sparql[proj_end..]);
    // `select_star` ⇒ keep all produced columns (no explicit reprojection), which
    // also matches `SELECT *` semantics (the window column is appended).
    let output_vars = if select_star { Vec::new() } else { output_vars };

    let _ = sel_start; // retained for clarity / future diagnostics
    Ok(Some(RewritePlan { rewritten, windows, output_vars }))
}

/// Parses one trimmed projection item as a window expression, or `Ok(None)` if it
/// is not a window item. A `(… OVER …)` shape that fails to parse is an `Err`.
fn parse_window_item(item: &str) -> Result<Option<WindowItem>, String> {
    // A window item is parenthesised: `( FN() OVER ( … ) AS ?out )`.
    let inner = match strip_outer_parens(item) {
        Some(i) => i,
        None => return Ok(None),
    };
    // Must contain the OVER keyword to be a window item.
    let over_pos = match find_keyword(inner, "OVER") {
        Some(p) => p,
        None => return Ok(None),
    };
    let head = inner[..over_pos].trim();
    let rest = inner[over_pos + 4..].trim_start();

    // head := FN ( ) — function name then an empty argument list.
    let lp = head.find('(').ok_or("inline OVER: window function call needs '()'")?;
    let fname = head[..lp].trim();
    let args = head[lp..].trim();
    let func = RankFunc::parse(fname)
        .ok_or_else(|| format!("inline OVER: unsupported window function {fname:?} (covered: ROW_NUMBER, RANK, DENSE_RANK)"))?;
    if args != "()" {
        return Err(format!(
            "inline OVER: window function {fname} must be called with empty args '()' (got {args:?}); windowed aggregates are deferred"
        ));
    }

    // rest := ( PARTITION BY … ORDER BY … ) AS ?out — split the balanced `( … )`
    // spec from its trailing `AS ?out` alias.
    let (spec_body, alias) = split_spec_and_alias(rest.trim())?;
    let (partition_by, order_by) = parse_over_spec(&spec_body)?;
    Ok(Some(WindowItem { func, partition_by, order_by, out: alias }))
}

/// Splits `( spec ) AS ?out` into (`spec`, `out`). Requires the `AS ?out` alias.
fn split_spec_and_alias(s: &str) -> Result<(String, String), String> {
    let s = s.trim();
    if !s.starts_with('(') {
        return Err("inline OVER: expected `( … )` after OVER".into());
    }
    // Find the matching close paren for the opening one.
    let close = matching_paren(s, 0)
        .ok_or("inline OVER: unbalanced parentheses in OVER ( … )")?;
    let spec = s[1..close].to_string();
    let tail = s[close + 1..].trim();
    // tail := AS ?out
    let tail_up = tail.to_ascii_uppercase();
    if !tail_up.starts_with("AS") {
        return Err("inline OVER: window expression must end with `AS ?out`".into());
    }
    let after_as = tail[2..].trim();
    let out = after_as
        .strip_prefix('?')
        .or_else(|| after_as.strip_prefix('$'))
        .ok_or("inline OVER: `AS` must be followed by an output variable ?out")?;
    let out = out.trim();
    if out.is_empty() || !out.chars().all(is_varname_char) {
        return Err(format!("inline OVER: invalid output variable {after_as:?} after AS"));
    }
    Ok((spec, out.to_string()))
}

/// Parses the body of an `OVER ( … )` spec into (partition vars, order keys).
fn parse_over_spec(spec: &str) -> Result<OverSpec, String> {
    let spec = spec.trim();
    let spec_up = spec.to_ascii_uppercase();
    let part_kw = find_keyword(spec, "PARTITION");
    let order_kw = find_keyword(spec, "ORDER");

    let mut partition_by = Vec::new();
    let mut order_by = Vec::new();

    // Partition segment runs from after `PARTITION BY` to the ORDER keyword (or end).
    if let Some(p) = part_kw {
        let after = &spec[p..];
        let after_up = &spec_up[p..];
        let by = after_up
            .find("BY")
            .ok_or("inline OVER: PARTITION must be followed by BY")?;
        let seg_start = p + by + 2;
        let seg_end = order_kw.unwrap_or(spec.len());
        if seg_end < seg_start {
            return Err("inline OVER: ORDER BY must come after PARTITION BY".into());
        }
        partition_by = parse_var_list(&spec[seg_start..seg_end])?;
        if partition_by.is_empty() {
            return Err("inline OVER: PARTITION BY needs at least one variable".into());
        }
        let _ = after;
    }

    if let Some(o) = order_kw {
        let after_up = &spec_up[o..];
        let by = after_up.find("BY").ok_or("inline OVER: ORDER must be followed by BY")?;
        let seg_start = o + by + 2;
        order_by = parse_order_list(&spec[seg_start..])?;
        if order_by.is_empty() {
            return Err("inline OVER: ORDER BY needs at least one variable".into());
        }
    }

    Ok((partition_by, order_by))
}

/// Parses a whitespace/comma-separated list of `?var` into bare names.
fn parse_var_list(s: &str) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    for tok in s.split([',', ' ', '\t', '\n', '\r']).filter(|t| !t.is_empty()) {
        let name = tok
            .strip_prefix('?')
            .or_else(|| tok.strip_prefix('$'))
            .ok_or_else(|| format!("inline OVER: expected a ?variable in PARTITION BY, got {tok:?}"))?;
        if name.is_empty() || !name.chars().all(is_varname_char) {
            return Err(format!("inline OVER: invalid variable {tok:?}"));
        }
        out.push(name.to_string());
    }
    Ok(out)
}

/// Parses an ORDER BY list. Each comma-separated key is one sort key in one of:
/// `?v`, `ASC(?v)`, `DESC(?v)`, `?v ASC`/`?v DESC`, OR — sq-c1jv — a computed
/// SPARQL expression in any of those positions, e.g. `(?a + ?b)`, `DESC(?a + ?b)`,
/// `(?a + ?b) DESC`. A bare-var key yields [`Sort::Var`]; anything else (including
/// a `( … )`-wrapped single var) yields [`Sort::Expr`] so the rewriter binds it to
/// a helper var.
fn parse_order_list(s: &str) -> Result<Vec<OrderKey>, String> {
    // Tokenise on commas first; each comma-separated key is one sort key.
    let mut keys = Vec::new();
    for raw in split_top_level_commas(s) {
        let key = raw.trim();
        if key.is_empty() {
            continue;
        }
        keys.push(parse_one_order_key(key)?);
    }
    Ok(keys)
}

fn parse_one_order_key(key: &str) -> Result<OrderKey, String> {
    let up = key.to_ascii_uppercase();
    // Leading direction keyword: `DESC( … )` / `ASC( … )`. The `( … )` payload is a
    // single var OR an arbitrary expression.
    for (kw, descending) in [("DESC", true), ("ASC", false)] {
        if up.starts_with(kw) {
            let after = key[kw.len()..].trim_start();
            if let Some(inner) = strip_outer_parens(after) {
                return Ok(OrderKey { sort: classify_sort(inner.trim())?, descending });
            }
            // A `?v ASC` / `?v DESC` trailing-keyword form whose VAR happens to
            // start with the letters `asc`/`desc` is handled by the trailing-keyword
            // branch below; fall through.
        }
    }
    // Trailing-keyword form: `<key>` | `<key> ASC` | `<key> DESC`, where `<key>` is
    // a bare var or a parenthesised expression (which may itself contain spaces).
    if let Some((body, descending)) = split_trailing_direction(key)? {
        return Ok(OrderKey { sort: classify_sort(body.trim())?, descending });
    }
    // No direction keyword: the whole key is the sort body, ascending.
    Ok(OrderKey { sort: classify_sort(key.trim())?, descending: false })
}

/// Splits an optional trailing ` ASC` / ` DESC` direction keyword off a sort key,
/// returning `Some((body, descending))` when one is present (so `?v DESC` → `("?v",
/// true)` and `(?a + ?b) ASC` → `("(?a + ?b)", false)`), or `None` when the key
/// carries no trailing direction. A direction keyword is only recognised at the
/// very end and outside any parentheses, so `(?a + ?b)` is left intact.
fn split_trailing_direction(key: &str) -> Result<Option<(String, bool)>, String> {
    let trimmed = key.trim_end();
    for (kw, descending) in [("DESC", true), ("ASC", false)] {
        // Match a trailing whole-word keyword: preceded by whitespace, at end.
        if let Some(body) = trimmed
            .get(..trimmed.len().saturating_sub(kw.len()))
            .filter(|_| trimmed.len() > kw.len())
            .filter(|_| trimmed[trimmed.len() - kw.len()..].eq_ignore_ascii_case(kw))
        {
            let last = body.chars().next_back();
            if last.map(|c| c.is_whitespace()).unwrap_or(false) {
                let body = body.trim();
                if body.is_empty() {
                    return Err(format!("inline OVER: bare {kw} is not an ORDER BY key"));
                }
                return Ok(Some((body.to_string(), descending)));
            }
        }
    }
    Ok(None)
}

/// Classifies a direction-stripped sort body: a single bare `?var` is a
/// [`Sort::Var`]; any other (non-empty) text is a computed [`Sort::Expr`] (sq-c1jv).
fn classify_sort(body: &str) -> Result<Sort, String> {
    let body = body.trim();
    if body.is_empty() {
        return Err("inline OVER: empty ORDER BY key".into());
    }
    // A bare `?v` / `$v` with a valid variable name is a plain variable key.
    if let Some(name) = body.strip_prefix('?').or_else(|| body.strip_prefix('$')) {
        if !name.is_empty() && name.chars().all(is_varname_char) {
            return Ok(Sort::Var(name.to_string()));
        }
    }
    // Otherwise it is a computed expression. Strip ONE redundant outer paren layer
    // (`(?a + ?b)` → `?a + ?b`) so the rewriter can re-wrap it as `(<expr> AS ?h)`.
    let expr = strip_outer_parens(body).unwrap_or(body).trim();
    if expr.is_empty() {
        return Err("inline OVER: empty ORDER BY expression".into());
    }
    Ok(Sort::Expr(expr.to_string()))
}

// ---- low-level lexical helpers ----------------------------------------------

fn is_varname_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// `true` if `kw` (case-insensitive) appears in `s` as a whole word (not as a
/// substring of an identifier / IRI). Used as the cheap `OVER`-presence gate.
fn contains_keyword(s: &str, kw: &str) -> bool {
    find_keyword(s, kw).is_some()
}

/// The byte offset of the first whole-word, case-insensitive occurrence of `kw`
/// in `s`, or `None`. A "whole word" is bounded by non-identifier characters on
/// both sides (so `OVER` does not match inside `?leftover` or `<.../over>`).
fn find_keyword(s: &str, kw: &str) -> Option<usize> {
    let su = s.to_ascii_uppercase();
    let ku = kw.to_ascii_uppercase();
    let bytes = su.as_bytes();
    let mut from = 0;
    while let Some(rel) = su[from..].find(&ku) {
        let at = from + rel;
        let before_ok = at == 0 || !is_ident_byte(bytes[at - 1]);
        let after = at + ku.len();
        let after_ok = after >= bytes.len() || !is_ident_byte(bytes[after]);
        if before_ok && after_ok {
            return Some(at);
        }
        from = at + 1;
    }
    None
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Finds (SELECT-keyword offset, projection-start offset) for the top-level
/// SELECT. The projection starts after `SELECT` and any `DISTINCT`/`REDUCED`.
fn find_select_projection_start(s: &str) -> Option<(usize, usize)> {
    let sel = find_keyword(s, "SELECT")?;
    let mut pos = sel + "SELECT".len();
    // Skip whitespace then an optional DISTINCT / REDUCED modifier.
    loop {
        let rest = &s[pos..];
        let trimmed_len = rest.len() - rest.trim_start().len();
        pos += trimmed_len;
        let rest = &s[pos..];
        let up = rest.to_ascii_uppercase();
        if up.starts_with("DISTINCT") && !rest[8..].starts_with(|c: char| is_varname_char(c)) {
            pos += "DISTINCT".len();
            continue;
        }
        if up.starts_with("REDUCED") && !rest[7..].starts_with(|c: char| is_varname_char(c)) {
            pos += "REDUCED".len();
            continue;
        }
        break;
    }
    // Re-skip whitespace before the first projection item.
    let rest = &s[pos..];
    pos += rest.len() - rest.trim_start().len();
    Some((sel, pos))
}

/// Finds the byte offset where the SELECT projection ends: the top-level `WHERE`
/// keyword, or the `{` that opens the group graph pattern (WHERE is optional in
/// SPARQL). Scanning starts at `from` and respects nested parens (so a `(… {…})`
/// inside the projection — which can't legally happen, but be safe — and IRIs are
/// not mistaken for the WHERE block).
fn find_projection_end(s: &str, from: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth: i32 = 0;
    let mut i = from;
    let su = s.to_ascii_uppercase();
    while i < bytes.len() {
        let b = bytes[i];
        match b {
            b'(' => depth += 1,
            b')' => depth -= 1,
            b'<' => {
                // Skip an IRI ref so a '{'/'}' inside it is ignored.
                if let Some(close) = s[i..].find('>') {
                    i += close;
                }
            }
            b'{' if depth == 0 => return Some(i),
            _ => {
                if depth == 0 && su[i..].starts_with("WHERE") {
                    let after = i + 5;
                    let before_ok = i == 0 || !is_ident_byte(bytes[i - 1]);
                    let after_ok = after >= bytes.len() || !is_ident_byte(bytes[after]);
                    if before_ok && after_ok {
                        return Some(i);
                    }
                }
            }
        }
        i += 1;
    }
    None
}

/// Splits a projection string into its top-level items: a bare `?var`, a
/// parenthesised `( … AS ?v )`, or `*`. Items are separated by whitespace EXCEPT
/// inside parentheses (a `( … )` is one item however much whitespace it holds).
fn split_projection_items(proj: &str) -> Result<Vec<String>, String> {
    let mut items = Vec::new();
    let bytes = proj.as_bytes();
    let mut i = 0;
    while i < proj.len() {
        // Skip whitespace.
        while i < proj.len() && (bytes[i] as char).is_whitespace() {
            i += 1;
        }
        if i >= proj.len() {
            break;
        }
        if bytes[i] == b'(' {
            let close =
                matching_paren(proj, i).ok_or("inline OVER: unbalanced '(' in SELECT projection")?;
            items.push(proj[i..=close].to_string());
            i = close + 1;
        } else {
            let start = i;
            while i < proj.len() && !(bytes[i] as char).is_whitespace() && bytes[i] != b'(' {
                i += 1;
            }
            items.push(proj[start..i].to_string());
        }
    }
    Ok(items)
}

/// Splits a string on top-level commas (commas not inside parentheses).
fn split_top_level_commas(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth: i32 = 0;
    let mut start = 0;
    let bytes = s.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => depth -= 1,
            b',' if depth == 0 => {
                out.push(s[start..i].to_string());
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(s[start..].to_string());
    out
}

/// If `s` (trimmed) is a single fully-parenthesised group, returns its interior.
/// `(a (b) c)` → `a (b) c`. Returns `None` if `s` does not start with `(` or the
/// first `(` does not close at the very end.
fn strip_outer_parens(s: &str) -> Option<&str> {
    let s = s.trim();
    if !s.starts_with('(') {
        return None;
    }
    let close = matching_paren(s, 0)?;
    if close == s.len() - 1 {
        Some(&s[1..close])
    } else {
        None
    }
}

/// The byte offset of the `)` matching the `(` at byte offset `open` in `s`.
fn matching_paren(s: &str, open: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    if bytes.get(open) != Some(&b'(') {
        return None;
    }
    let mut depth: i32 = 0;
    let mut i = open;
    while i < s.len() {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            b'<' => {
                if let Some(rel) = s[i..].find('>') {
                    i += rel;
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// The OUTPUT variable a non-window projection item binds: `?v` → `v`, or the
/// `?v` from a `( expr AS ?v )` alias.
fn projection_item_output_var(item: &str) -> Result<String, String> {
    let t = item.trim();
    if let Some(name) = t.strip_prefix('?').or_else(|| t.strip_prefix('$')) {
        if name.chars().all(is_varname_char) && !name.is_empty() {
            return Ok(name.to_string());
        }
    }
    if let Some(inner) = strip_outer_parens(t) {
        // ( expr AS ?v ): the alias var is after the last top-level `AS`.
        if let Some(as_pos) = find_keyword(inner, "AS") {
            let after = inner[as_pos + 2..].trim();
            if let Some(name) = after.strip_prefix('?').or_else(|| after.strip_prefix('$')) {
                let name = name.trim();
                if name.chars().all(is_varname_char) && !name.is_empty() {
                    return Ok(name.to_string());
                }
            }
        }
    }
    Err(format!("inline OVER: cannot determine the output variable of projection item {item:?}"))
}

/// `true` if a non-window projection item's OUTPUT var equals `name` (so a helper
/// var need not be re-projected).
fn projection_item_mentions_output(item: &str, name: &str) -> bool {
    projection_item_output_var(item).map(|v| v == name).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DATA: &str = r#"@prefix ex: <http://ex/> .
        ex:a ex:dept ex:eng   ; ex:sales 30 .
        ex:b ex:dept ex:eng   ; ex:sales 30 .
        ex:c ex:dept ex:eng   ; ex:sales 20 .
        ex:d ex:dept ex:sales ; ex:sales 10 ."#;

    fn g() -> Graph {
        Graph::load_str(DATA, "turtle").unwrap()
    }

    fn cell(r: &QueryResult, row: usize, col: usize) -> String {
        r.rows[row][col].as_ref().map(|t| t.to_string()).unwrap_or_default()
    }

    // ---- unit tests on the rewriter ----

    #[test]
    fn detects_no_over_returns_none() {
        assert!(extract_windows("SELECT ?s WHERE { ?s ?p ?o }").unwrap().is_none());
        // `?over` must NOT trip the keyword scan.
        assert!(extract_windows("SELECT ?over WHERE { ?over ?p ?o }").unwrap().is_none());
    }

    fn var_key(name: &str, descending: bool) -> OrderKey {
        OrderKey { sort: Sort::Var(name.to_string()), descending }
    }
    fn expr_key(expr: &str, descending: bool) -> OrderKey {
        OrderKey { sort: Sort::Expr(expr.to_string()), descending }
    }

    #[test]
    fn parses_row_number_item() {
        let w = parse_window_item("(ROW_NUMBER() OVER (PARTITION BY ?dept ORDER BY DESC(?sales)) AS ?rn)")
            .unwrap()
            .unwrap();
        assert_eq!(w.func, RankFunc::RowNumber);
        assert_eq!(w.partition_by, vec!["dept".to_string()]);
        assert_eq!(w.order_by, vec![var_key("sales", true)]);
        assert_eq!(w.out, "rn");
    }

    #[test]
    fn parses_trailing_keyword_order_form() {
        let w = parse_window_item("(RANK() OVER (PARTITION BY ?dept ORDER BY ?sales DESC) AS ?r)")
            .unwrap()
            .unwrap();
        assert_eq!(w.func, RankFunc::Rank);
        assert_eq!(w.order_by, vec![var_key("sales", true)]);
    }

    #[test]
    fn parses_computed_expression_order_key() {
        // sq-c1jv: a parenthesised computed expression as an ORDER BY key. The
        // redundant outer parens are stripped; the key is an Expr (ascending).
        let w = parse_window_item("(ROW_NUMBER() OVER (ORDER BY (?a + ?b)) AS ?rn)")
            .unwrap()
            .unwrap();
        assert_eq!(w.order_by, vec![expr_key("?a + ?b", false)]);

        // `DESC( <expr> )` → an Expr key, descending.
        let d = parse_window_item("(RANK() OVER (ORDER BY DESC(?sales + 0)) AS ?r)")
            .unwrap()
            .unwrap();
        assert_eq!(d.order_by, vec![expr_key("?sales + 0", true)]);

        // Trailing-keyword form on a parenthesised expression: `(?a * 2) DESC`.
        let t = parse_window_item("(RANK() OVER (ORDER BY (?a * 2) DESC) AS ?r)")
            .unwrap()
            .unwrap();
        assert_eq!(t.order_by, vec![expr_key("?a * 2", true)]);

        // A function-call expression key (not a direction keyword): `STRLEN(?s)`.
        let f = parse_window_item("(ROW_NUMBER() OVER (ORDER BY STRLEN(?s)) AS ?rn)")
            .unwrap()
            .unwrap();
        assert_eq!(f.order_by, vec![expr_key("STRLEN(?s)", false)]);
    }

    #[test]
    fn direction_keyword_detection_edge_cases() {
        // A variable literally NAMED `desc` is a plain var key, not a direction.
        let w = parse_window_item("(ROW_NUMBER() OVER (ORDER BY ?desc) AS ?rn)")
            .unwrap()
            .unwrap();
        assert_eq!(w.order_by, vec![var_key("desc", false)]);
        // A function call ending in `)` is an expression key, not a trailing keyword.
        let f = parse_window_item("(RANK() OVER (ORDER BY STRLEN(?desc)) AS ?r)")
            .unwrap()
            .unwrap();
        assert_eq!(f.order_by, vec![expr_key("STRLEN(?desc)", false)]);
    }

    #[test]
    fn mixed_var_and_expression_order_keys() {
        // A var key and an expr key in one ORDER BY list, with directions.
        let w = parse_window_item(
            "(RANK() OVER (ORDER BY ?dept, DESC(?a + ?b)) AS ?r)",
        )
        .unwrap()
        .unwrap();
        assert_eq!(w.order_by, vec![var_key("dept", false), expr_key("?a + ?b", true)]);
    }

    #[test]
    fn computed_expr_key_rewrites_to_helper_binding() {
        // The rewriter binds a computed ORDER BY key to a fresh helper var via an
        // `(<expr> AS ?__win_orderkeyN)` item in the inner SELECT, and the post-pass
        // window orders on that helper (which is NOT in the final output).
        let plan = extract_windows(
            "PREFIX ex: <http://ex/> SELECT ?emp (ROW_NUMBER() OVER (ORDER BY (?sales * -1)) AS ?rn) WHERE { ?emp ex:sales ?sales }",
        )
        .unwrap()
        .unwrap();
        assert!(
            plan.rewritten.contains("(?sales * -1 AS ?__win_orderkey0)"),
            "inner SELECT must bind the expr to a helper: {}",
            plan.rewritten
        );
        // The window now orders on the helper var, not the raw expression.
        assert_eq!(plan.windows[0].order_by, vec![var_key("__win_orderkey0", false)]);
        // The helper is not a user-named output column.
        assert_eq!(plan.output_vars, vec!["emp".to_string(), "rn".to_string()]);
    }

    #[test]
    fn rejects_windowed_aggregate_inline() {
        // A windowed aggregate (`SUM(?x) OVER …`) is DEFERRED: SUM is not in the
        // covered ranking-function set, so it is rejected as unsupported.
        let e = parse_window_item("(SUM(?sales) OVER (PARTITION BY ?dept) AS ?t)").unwrap_err();
        assert!(e.contains("unsupported window function") && e.contains("SUM"), "{e}");
        // And a covered function called WITH args (the other way to write a windowed
        // aggregate) is rejected for the non-empty arg list, naming the deferral.
        let e2 = parse_window_item("(RANK(?sales) OVER (PARTITION BY ?dept) AS ?t)").unwrap_err();
        assert!(e2.contains("empty args") && e2.contains("aggregate"), "{e2}");
    }

    #[test]
    fn rejects_unknown_window_function() {
        let e = parse_window_item("(NTILE() OVER (ORDER BY ?x) AS ?n)").unwrap_err();
        assert!(e.contains("unsupported window function"), "{e}");
    }

    // ---- end-to-end parse + eval ----

    #[test]
    fn row_number_over_partition_order() {
        let q = "PREFIX ex: <http://ex/> \
            SELECT ?emp ?dept ?sales (ROW_NUMBER() OVER (PARTITION BY ?dept ORDER BY DESC(?sales)) AS ?rn) \
            WHERE { ?emp ex:dept ?dept ; ex:sales ?sales } ORDER BY ?dept DESC(?sales) ?emp";
        let r = query_over(&g(), q).unwrap();
        // Columns: emp, dept, sales, rn.
        assert_eq!(r.vars.iter().map(|v| v.as_str()).collect::<Vec<_>>(), ["emp", "dept", "sales", "rn"]);
        // eng (30,30,20) desc → rn 1,2,3 (a,b tie at 30, broken by ?emp asc); sales (10) → rn 1.
        // Row order by ORDER BY ?dept DESC(?sales) ?emp: sales/d first (dept "sales" > "eng" lexically? IRIs).
        // Be order-agnostic: collect (dept,sales,rn) and check the multiset of rn per dept.
        let mut eng_rns: Vec<i64> = Vec::new();
        let mut sales_rns: Vec<i64> = Vec::new();
        for row in &r.rows {
            let dept = row[1].as_ref().unwrap().to_string();
            let rn: i64 = row[3].as_ref().unwrap().to_string().trim_matches('"').split('"').next().unwrap()
                .parse().unwrap_or_else(|_| literal_int(&row[3]));
            if dept.contains("/eng") {
                eng_rns.push(rn);
            } else {
                sales_rns.push(rn);
            }
        }
        eng_rns.sort();
        sales_rns.sort();
        assert_eq!(eng_rns, vec![1, 2, 3]);
        assert_eq!(sales_rns, vec![1]);
    }

    fn literal_int(cell: &Option<oxrdf::Term>) -> i64 {
        if let Some(oxrdf::Term::Literal(l)) = cell {
            l.value().parse().unwrap()
        } else {
            panic!("not an integer literal: {cell:?}")
        }
    }

    #[test]
    fn rank_and_dense_rank_sequence() {
        // RANK has gaps after ties; DENSE_RANK does not. eng sales 30,30,20.
        let qr = "PREFIX ex: <http://ex/> \
            SELECT ?emp (RANK() OVER (PARTITION BY ?dept ORDER BY DESC(?sales)) AS ?r) \
            WHERE { ?emp ex:dept ?dept ; ex:sales ?sales }";
        let r = query_over(&g(), qr).unwrap();
        // Map emp → rank.
        let rank_of = |r: &QueryResult, emp: &str| -> i64 {
            for row in &r.rows {
                if row[0].as_ref().unwrap().to_string().contains(emp) {
                    return literal_int(&row[1]);
                }
            }
            panic!("emp {emp} not found");
        };
        assert_eq!(rank_of(&r, "/a"), 1);
        assert_eq!(rank_of(&r, "/b"), 1);
        assert_eq!(rank_of(&r, "/c"), 3); // GAP after the 2-way tie
        assert_eq!(rank_of(&r, "/d"), 1); // own partition

        let qd = "PREFIX ex: <http://ex/> \
            SELECT ?emp (DENSE_RANK() OVER (PARTITION BY ?dept ORDER BY DESC(?sales)) AS ?r) \
            WHERE { ?emp ex:dept ?dept ; ex:sales ?sales }";
        let d = query_over(&g(), qd).unwrap();
        let drank_of = |emp: &str| -> i64 {
            for row in &d.rows {
                if row[0].as_ref().unwrap().to_string().contains(emp) {
                    return literal_int(&row[1]);
                }
            }
            panic!();
        };
        assert_eq!(drank_of("/a"), 1);
        assert_eq!(drank_of("/c"), 2); // NO gap
    }

    #[test]
    fn helper_var_not_in_output_is_projected_through_then_dropped() {
        // ?sales is used only by the window ORDER BY and is NOT in the SELECT output;
        // it must be projected through the inner query then dropped from the result.
        let q = "PREFIX ex: <http://ex/> \
            SELECT ?emp (RANK() OVER (ORDER BY DESC(?sales)) AS ?r) \
            WHERE { ?emp ex:dept ?dept ; ex:sales ?sales }";
        let r = query_over(&g(), q).unwrap();
        assert_eq!(r.vars.iter().map(|v| v.as_str()).collect::<Vec<_>>(), ["emp", "r"]);
        // Single partition (no PARTITION BY): 30,30,20,10 → ranks 1,1,3,4.
        let mut ranks: Vec<i64> = r.rows.iter().map(|row| literal_int(&row[1])).collect();
        ranks.sort();
        assert_eq!(ranks, vec![1, 1, 3, 4]);
    }

    #[test]
    fn select_star_with_window_keeps_all_plus_window_col() {
        let q = "PREFIX ex: <http://ex/> \
            SELECT * (ROW_NUMBER() OVER (PARTITION BY ?dept ORDER BY ?sales) AS ?rn) \
            WHERE { ?emp ex:dept ?dept ; ex:sales ?sales }";
        let r = query_over(&g(), q).unwrap();
        let names: Vec<&str> = r.vars.iter().map(|v| v.as_str()).collect();
        assert!(names.contains(&"emp") && names.contains(&"dept") && names.contains(&"sales"));
        assert!(names.contains(&"rn"), "window column appended: {names:?}");
    }

    #[test]
    fn non_window_query_is_unchanged() {
        // A query with no OVER goes straight through the ordinary engine.
        let q = "PREFIX ex: <http://ex/> SELECT ?emp WHERE { ?emp ex:dept ?d }";
        let r = query_over(&g(), q).unwrap();
        assert_eq!(r.vars.iter().map(|v| v.as_str()).collect::<Vec<_>>(), ["emp"]);
        assert_eq!(r.rows.len(), 4);
    }

    #[test]
    fn two_window_columns_in_one_select() {
        let q = "PREFIX ex: <http://ex/> \
            SELECT ?emp \
                (ROW_NUMBER() OVER (PARTITION BY ?dept ORDER BY DESC(?sales)) AS ?rn) \
                (DENSE_RANK() OVER (PARTITION BY ?dept ORDER BY DESC(?sales)) AS ?dr) \
            WHERE { ?emp ex:dept ?dept ; ex:sales ?sales }";
        let r = query_over(&g(), q).unwrap();
        assert_eq!(r.vars.iter().map(|v| v.as_str()).collect::<Vec<_>>(), ["emp", "rn", "dr"]);
        let _ = cell(&r, 0, 1);
    }

    #[test]
    fn malformed_over_is_error() {
        // Missing AS alias.
        assert!(query_over(&g(), "PREFIX ex: <http://ex/> SELECT (ROW_NUMBER() OVER (ORDER BY ?sales)) WHERE { ?emp ex:sales ?sales }").is_err());
        // Non-empty args (a windowed aggregate, deferred).
        assert!(query_over(&g(), "PREFIX ex: <http://ex/> SELECT (SUM(?sales) OVER (PARTITION BY ?dept) AS ?t) WHERE { ?emp ex:dept ?dept ; ex:sales ?sales }").is_err());
    }
}
