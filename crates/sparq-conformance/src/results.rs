//! Expected-result parsing: SPARQL XML results (`.srx`), DAWG result-set
//! graphs (`.ttl` / `.rdf` using the `rs:` vocabulary).

use crate::rdf::{as_node, MiniGraph};
use oxrdf::{Literal, NamedNode, Term};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::path::Path;

const RS: &str = "http://www.w3.org/2001/sw/DataAccess/tests/result-set#";

/// One solution: `(variable name, bound term)` pairs.
pub type Binding = Vec<(String, Term)>;

#[derive(Debug)]
pub enum Expected {
    Bindings {
        /// Declared result variables (may be empty for `rs:` graphs that omit them).
        vars: Vec<String>,
        rows: Vec<Binding>,
        /// True when the result set carried explicit `rs:index` ordering.
        indexed: bool,
    },
    Boolean(bool),
}

pub fn parse_expected(path: &Path) -> Result<Expected, String> {
    match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "srx" => parse_srx(path),
        "srj" => parse_srj(path),
        "ttl" | "rdf" | "nt" => parse_rs_graph(path),
        ext => Err(format!("unsupported result format: .{ext}")),
    }
}

/// SPARQL Query Results JSON Format.
fn parse_srj(path: &Path) -> Result<Expected, String> {
    let text =
        std::fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let v: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    if let Some(b) = v.get("boolean").and_then(|b| b.as_bool()) {
        return Ok(Expected::Boolean(b));
    }
    let vars: Vec<String> = v
        .pointer("/head/vars")
        .and_then(|a| a.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|s| s.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let mut rows: Vec<Binding> = Vec::new();
    for sol in v
        .pointer("/results/bindings")
        .and_then(|a| a.as_array())
        .into_iter()
        .flatten()
    {
        let mut row: Binding = Vec::new();
        for (var, val) in sol.as_object().into_iter().flatten() {
            let get = |k: &str| val.get(k).and_then(|s| s.as_str());
            let Some(value) = get("value") else { continue };
            let term = match get("type") {
                Some("uri") => make_term("uri", None, None, value.to_string())?,
                Some("bnode") => make_term("bnode", None, None, value.to_string())?,
                _ => make_term(
                    "literal",
                    get("xml:lang").map(String::from),
                    get("datatype").map(String::from),
                    value.to_string(),
                )?,
            };
            row.push((var.clone(), term));
        }
        rows.push(row);
    }
    Ok(Expected::Bindings {
        vars,
        rows,
        indexed: false,
    })
}

/// SPARQL Query Results XML Format.
fn parse_srx(path: &Path) -> Result<Expected, String> {
    let text =
        std::fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let mut reader = Reader::from_str(&text);
    reader.config_mut().trim_text(false);

    let mut vars: Vec<String> = Vec::new();
    let mut rows: Vec<Binding> = Vec::new();
    let mut cur_row: Binding = Vec::new();
    let mut cur_var: Option<String> = None;
    // Value element currently open: (kind, lang, datatype, text).
    let mut cur_val: Option<(String, Option<String>, Option<String>, String)> = None;
    let mut boolean: Option<bool> = None;
    let mut in_boolean = false;

    loop {
        match reader
            .read_event()
            .map_err(|e| format!("{}: {e}", path.display()))?
        {
            Event::Eof => break,
            ev @ (Event::Start(_) | Event::Empty(_)) => {
                let is_empty = matches!(ev, Event::Empty(_));
                let e = match &ev {
                    Event::Start(e) | Event::Empty(e) => e,
                    _ => unreachable!(),
                };
                let name = e.local_name();
                let name = std::str::from_utf8(name.as_ref()).unwrap_or("").to_string();
                let attr = |key: &str| -> Option<String> {
                    e.attributes().filter_map(|a| a.ok()).find_map(|a| {
                        let k = std::str::from_utf8(a.key.as_ref()).ok()?;
                        // Matches both `datatype` and `xml:lang`-style keys.
                        if k == key || k.ends_with(&format!(":{key}")) {
                            Some(a.unescape_value().ok()?.into_owned())
                        } else {
                            None
                        }
                    })
                };
                match name.as_str() {
                    "variable" => {
                        if let Some(v) = attr("name") {
                            vars.push(v);
                        }
                    }
                    "result" => cur_row = Vec::new(),
                    "binding" => cur_var = attr("name"),
                    "uri" | "bnode" => cur_val = Some((name, None, None, String::new())),
                    "literal" => {
                        cur_val = Some((name, attr("lang"), attr("datatype"), String::new()))
                    }
                    "boolean" => in_boolean = true,
                    _ => {}
                }
                // Self-closing value elements (e.g. `<literal/>`) get no End event:
                // commit the (empty-text) term right away.
                if is_empty {
                    if let (Some((kind, lang, dt, text)), Some(var)) =
                        (cur_val.take(), cur_var.clone())
                    {
                        cur_row.push((var, make_term(&kind, lang, dt, text)?));
                    }
                }
            }
            Event::Text(t) => {
                let s = t.unescape().map_err(|e| e.to_string())?.into_owned();
                if in_boolean {
                    boolean = Some(s.trim() == "true");
                } else if let Some(v) = cur_val.as_mut() {
                    v.3.push_str(&s);
                }
            }
            Event::CData(t) => {
                if let Some(v) = cur_val.as_mut() {
                    v.3.push_str(&String::from_utf8_lossy(&t));
                }
            }
            Event::End(e) => {
                let name = e.local_name();
                match name.as_ref() {
                    b"uri" | b"bnode" | b"literal" => {
                        if let (Some((kind, lang, dt, text)), Some(var)) =
                            (cur_val.take(), cur_var.clone())
                        {
                            let term = make_term(&kind, lang, dt, text)?;
                            cur_row.push((var, term));
                        }
                    }
                    b"binding" => cur_var = None,
                    b"result" => rows.push(std::mem::take(&mut cur_row)),
                    b"boolean" => in_boolean = false,
                    _ => {}
                }
            }
            _ => {}
        }
    }

    if let Some(b) = boolean {
        return Ok(Expected::Boolean(b));
    }
    Ok(Expected::Bindings {
        vars,
        rows,
        indexed: false,
    })
}

fn make_term(
    kind: &str,
    lang: Option<String>,
    dt: Option<String>,
    text: String,
) -> Result<Term, String> {
    Ok(match kind {
        "uri" => Term::NamedNode(NamedNode::new(text).map_err(|e| e.to_string())?),
        "bnode" => Term::BlankNode(oxrdf::BlankNode::new(text).map_err(|e| e.to_string())?),
        _ => {
            if let Some(lang) = lang {
                Term::Literal(
                    Literal::new_language_tagged_literal(text, lang).map_err(|e| e.to_string())?,
                )
            } else if let Some(dt) = dt {
                Term::Literal(Literal::new_typed_literal(
                    text,
                    NamedNode::new(dt).map_err(|e| e.to_string())?,
                ))
            } else {
                Term::Literal(Literal::new_simple_literal(text))
            }
        }
    })
}

/// DAWG result-set graph (`rs:` vocabulary) in Turtle or RDF/XML.
fn parse_rs_graph(path: &Path) -> Result<Expected, String> {
    let g = MiniGraph::load(path)?;
    let sets = g.subjects_with_type(&format!("{RS}ResultSet"));
    let set = sets
        .first()
        .ok_or_else(|| format!("{}: no rs:ResultSet node (graph result?)", path.display()))?;

    if let Some(Term::Literal(l)) = g.object(set, &format!("{RS}boolean")) {
        return Ok(Expected::Boolean(l.value() == "true"));
    }

    let vars: Vec<String> = g
        .objects(set, &format!("{RS}resultVariable"))
        .into_iter()
        .filter_map(|t| match t {
            Term::Literal(l) => Some(l.value().to_string()),
            _ => None,
        })
        .collect();

    // Each rs:solution is a node with rs:binding [rs:variable; rs:value] and optional rs:index.
    let mut rows: Vec<(Option<f64>, Binding)> = Vec::new();
    for sol in g.objects(set, &format!("{RS}solution")) {
        let Some(sol) = as_node(sol) else { continue };
        let mut row: Binding = Vec::new();
        for b in g.objects(&sol, &format!("{RS}binding")) {
            let Some(b) = as_node(b) else { continue };
            let var = g.str_object(&b, &format!("{RS}variable"));
            let val = g.object(&b, &format!("{RS}value"));
            if let (Some(var), Some(val)) = (var, val) {
                row.push((var, val.clone()));
            }
        }
        let index = match g.object(&sol, &format!("{RS}index")) {
            Some(Term::Literal(l)) => l.value().parse::<f64>().ok(),
            _ => None,
        };
        rows.push((index, row));
    }
    let indexed = !rows.is_empty() && rows.iter().all(|(i, _)| i.is_some());
    if indexed {
        rows.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    }
    Ok(Expected::Bindings {
        vars,
        rows: rows.into_iter().map(|(_, r)| r).collect(),
        indexed,
    })
}
