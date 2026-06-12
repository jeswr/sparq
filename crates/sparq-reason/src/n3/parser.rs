//! A focused Notation3 parser — the subset needed for rule reasoning (EYE-style).
//!
//! oxttl parses Turtle/N-Triples but NOT N3's rule/formula extensions, so we hand-roll a
//! recursive-descent parser over the common N3 surface:
//! - `@prefix`/`@base`, prefixed names, `<iri>`, `a` (= rdf:type)
//! - literals: `"s"`, `"s"^^<dt>`, `"s"@lang`, integers, decimals, doubles, `true`/`false`
//! - `_:blank`, `?var` (universally-quantified N3 variables)
//! - `{ … }` formulae (graph terms), `( … )` collections (RDF lists)
//! - predicate sugar `=>` (log:implies), `<=` (reverse implies), `=` (owl:sameAs),
//!   `is EXPR of` (inverse predicate) and `has EXPR`
//! - paths (`!`/`^`), relative-IRI resolution against a document base, cwm's
//!   undeclared-`:`-prefix-as-`<#>` convention
//! - statement structure with `;` (predicate lists) and `,` (object lists)
//!
//! Not yet covered: explicit `@forAll`/`@forSome`, `@keywords`, nested quoting beyond
//! formulae. These are roadmap items toward full EYE parity.

use super::model::{Rule, Term};

pub const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
pub const RDF_NIL: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#nil";
pub const RDF_FIRST: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#first";
pub const RDF_REST: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#rest";
pub const LOG_IMPLIES: &str = "http://www.w3.org/2000/10/swap/log#implies";
pub const LOG_IMPLIED_BY: &str = "http://www.w3.org/2000/10/swap/log#impliedBy";
pub const OWL_SAME_AS: &str = "http://www.w3.org/2002/07/owl#sameAs";
pub const XSD_INTEGER: &str = "http://www.w3.org/2001/XMLSchema#integer";
pub const XSD_DECIMAL: &str = "http://www.w3.org/2001/XMLSchema#decimal";
pub const XSD_DOUBLE: &str = "http://www.w3.org/2001/XMLSchema#double";
pub const XSD_BOOLEAN: &str = "http://www.w3.org/2001/XMLSchema#boolean";

/// A parsed N3 document: ground/rule statements separated by the caller.
pub struct Parsed {
    /// Top-level ground triples (facts).
    pub facts: Vec<[Term; 3]>,
    /// `{ premise } => { conclusion }` forward rules.
    pub rules: Vec<Rule>,
    /// `{ conclusion } <= { premise }` BACKWARD rules — goal-directed, never fired forward.
    /// `{ conclusion } <= true.` is a backward rule with an empty (always-provable) premise.
    pub backward_rules: Vec<Rule>,
}

pub fn parse(src: &str) -> Result<Parsed, String> {
    parse_with_base(src, "")
}

/// As [`parse`], but resolves relative IRIs (in IRIREFs and `@prefix`/`@base`
/// directives) against `base` — the document's own location, RFC 3986-style.
/// An empty `base` keeps relative IRIs as written (the historical behavior).
pub fn parse_with_base(src: &str, base: &str) -> Result<Parsed, String> {
    let mut p = Parser::new(src);
    p.base = base.to_string();
    let stmts = p.document()?;
    let mut facts = Vec::new();
    let mut rules = Vec::new();
    let mut backward_rules = Vec::new();
    for [s, pred, o] in stmts {
        match (&pred, &s, &o) {
            // { premise } => { conclusion }
            (Term::Iri(i), Term::Formula(prem), Term::Formula(concl)) if i == LOG_IMPLIES => {
                rules.push(Rule { premise: prem.clone(), conclusion: concl.clone() });
            }
            // { conclusion } <= { premise } — a backward rule. EYE never fires these
            // forward: they are tried goal-directed when a forward-rule premise (or an
            // EYE-style query rule `{goal} => {goal}`) needs an atom they can conclude.
            // Verified against eyereasoner/eye reasoning/backward: the premise is often a
            // pure builtin over the goal's bindings, which can ONLY be evaluated once the
            // goal instantiates the variables — a forward reversal would derive nothing.
            (Term::Iri(i), Term::Formula(concl), Term::Formula(prem)) if i == LOG_IMPLIED_BY => {
                backward_rules.push(Rule { premise: prem.clone(), conclusion: concl.clone() });
            }
            // { conclusion } <= true. — an always-provable backward fact schema (EYE's
            // idiom for backward base cases).
            (Term::Iri(i), Term::Formula(concl), Term::Lit(v, _, _))
                if i == LOG_IMPLIED_BY && v == "true" =>
            {
                backward_rules.push(Rule { premise: Vec::new(), conclusion: concl.clone() });
            }
            _ => facts.push([s, pred, o]),
        }
    }
    // N3 semantics: a blank node inside a rule is an EXISTENTIAL — in the
    // premise it matches anything (like a variable), and a premise-bound
    // label shared with the conclusion carries its binding (the cwm/EYE
    // `[is :p of :x]`-in-premise idiom). Rewrite rule-scoped premise blanks
    // to fresh per-rule variables so the matcher treats them so; blanks
    // appearing ONLY in a conclusion stay blanks.
    for (ri, r) in rules.iter_mut().chain(backward_rules.iter_mut()).enumerate() {
        let mut premise_blanks: std::collections::HashSet<String> = Default::default();
        collect_blanks(&r.premise, &mut premise_blanks);
        if premise_blanks.is_empty() {
            continue;
        }
        let rename = |t: &mut Term| {
            if let Term::Blank(l) = t {
                if premise_blanks.contains(l.as_str()) {
                    *t = Term::Var(format!("__bn{ri}_{l}"));
                }
            }
        };
        rewrite_terms(&mut r.premise, &rename);
        rewrite_terms(&mut r.conclusion, &rename);
    }
    Ok(Parsed { facts, rules, backward_rules })
}

/// `rdf:nil` IS the empty list — normalize the IRI spelling to `Term::List([])`
/// so `()` and `rdf:nil` are one value (cwm/EYE agree).
fn nil_to_list(t: Term) -> Term {
    match t {
        Term::Iri(i) if i == RDF_NIL => Term::List(Vec::new()),
        other => other,
    }
}

fn collect_blanks(stmts: &[[Term; 3]], out: &mut std::collections::HashSet<String>) {
    for row in stmts {
        for t in row {
            collect_blanks_term(t, out);
        }
    }
}

fn collect_blanks_term(t: &Term, out: &mut std::collections::HashSet<String>) {
    match t {
        Term::Blank(l) => {
            out.insert(l.clone());
        }
        Term::Formula(inner) => collect_blanks(inner, out),
        Term::List(ms) => {
            for m in ms {
                collect_blanks_term(m, out);
            }
        }
        _ => {}
    }
}

fn rewrite_terms(stmts: &mut [[Term; 3]], f: &impl Fn(&mut Term)) {
    for row in stmts.iter_mut() {
        for t in row.iter_mut() {
            rewrite_term(t, f);
        }
    }
}

fn rewrite_term(t: &mut Term, f: &impl Fn(&mut Term)) {
    match t {
        Term::Formula(inner) => rewrite_terms(inner, f),
        Term::List(ms) => {
            for m in ms.iter_mut() {
                rewrite_term(m, f);
            }
        }
        _ => f(t),
    }
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
    base: String,
    prefixes: std::collections::HashMap<String, String>,
    bnode: usize,
    pathvar: usize,
    /// Current `{`/`(`/`[` nesting depth — bounded so pathological inputs
    /// produce a parse ERROR instead of exhausting the stack (the recursive-
    /// descent parser recurses per nesting level).
    depth: usize,
}

/// Maximum bracket-nesting depth (see `Parser::depth`).
const MAX_DEPTH: usize = 1024;

impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Parser<'a> {
        Parser { s: src.as_bytes(), i: 0, base: String::new(), prefixes: Default::default(), bnode: 0, pathvar: 0, depth: 0 }
    }

    // ---- lexing helpers ------------------------------------------------------
    fn ws(&mut self) {
        loop {
            while self.i < self.s.len() && (self.s[self.i] as char).is_whitespace() {
                self.i += 1;
            }
            if self.i < self.s.len() && self.s[self.i] == b'#' {
                while self.i < self.s.len() && self.s[self.i] != b'\n' {
                    self.i += 1;
                }
            } else {
                break;
            }
        }
    }
    fn peek(&self) -> Option<u8> {
        self.s.get(self.i).copied()
    }
    fn eat(&mut self, b: u8) -> bool {
        self.ws();
        if self.peek() == Some(b) {
            self.i += 1;
            true
        } else {
            false
        }
    }
    fn starts_with(&mut self, kw: &str) -> bool {
        self.ws();
        self.s[self.i..].starts_with(kw.as_bytes())
    }

    // ---- top level -----------------------------------------------------------
    fn document(&mut self) -> Result<Vec<[Term; 3]>, String> {
        let mut out = Vec::new();
        loop {
            self.ws();
            if self.i >= self.s.len() {
                break;
            }
            if self.starts_with("@prefix") || self.starts_with("PREFIX") {
                self.directive_prefix()?;
            } else if self.starts_with("@base") || self.starts_with("BASE") {
                self.directive_base()?;
            } else {
                self.statement(&mut out)?;
            }
        }
        Ok(out)
    }

    fn directive_prefix(&mut self) -> Result<(), String> {
        self.i += if self.starts_with("@prefix") { 7 } else { 6 };
        self.ws();
        let pfx = self.read_pname_prefix()?;
        self.ws();
        let iri = self.read_iriref()?;
        self.prefixes.insert(pfx, iri);
        self.eat(b'.');
        Ok(())
    }
    fn directive_base(&mut self) -> Result<(), String> {
        self.i += if self.starts_with("@base") { 5 } else { 4 };
        self.ws();
        // read_iriref already resolves the new base against the current one.
        self.base = self.read_iriref()?;
        self.eat(b'.');
        Ok(())
    }

    /// `subject predicateObjectList .` with `;` and `,`.
    fn statement(&mut self, out: &mut Vec<[Term; 3]>) -> Result<(), String> {
        let subj = self.term(out)?;
        self.predicate_object_list(&subj, out)?;
        self.eat(b'.');
        Ok(())
    }

    fn predicate_object_list(&mut self, subj: &Term, out: &mut Vec<[Term; 3]>) -> Result<(), String> {
        loop {
            self.ws();
            let (pred, swapped) = self.verb(out)?;
            loop {
                let obj = self.term(out)?;
                if swapped {
                    out.push([obj, pred.clone(), subj.clone()]);
                } else {
                    out.push([subj.clone(), pred.clone(), obj]);
                }
                if !self.eat(b',') {
                    break;
                }
            }
            if !self.eat(b';') {
                break;
            }
            // allow trailing `;`
            self.ws();
            if matches!(self.peek(), Some(b'.') | Some(b']') | Some(b'}') | None) {
                break;
            }
        }
        Ok(())
    }

    /// A predicate position: returns `(predicate, swapped)` — `swapped` is the
    /// classic N3 inverse-predicate syntax `is EXPR of` (object and subject
    /// trade places); `has EXPR` is the explicit forward form.
    fn verb(&mut self, _out: &mut Vec<[Term; 3]>) -> Result<(Term, bool), String> {
        self.ws();
        if self.starts_with("=>") {
            self.i += 2;
            return Ok((Term::Iri(LOG_IMPLIES.into()), false));
        }
        if self.starts_with("<=") {
            self.i += 2;
            // `conclusion <= premise` — emit log:impliedBy; `parse` routes it into the
            // backward-rule set (goal-directed resolution, see the module doc).
            return Ok((Term::Iri(LOG_IMPLIED_BY.into()), false));
        }
        if self.eat(b'=') {
            return Ok((Term::Iri(OWL_SAME_AS.into()), false));
        }
        // `a` keyword (rdf:type) — only when a standalone token.
        if self.starts_with("a") {
            let j = self.i + 1;
            if j >= self.s.len() || (self.s[j] as char).is_whitespace() || self.s[j] == b'<' {
                self.i += 1;
                return Ok((Term::Iri(RDF_TYPE.into()), false));
            }
        }
        // `is EXPR of` — inverse predicate; `has EXPR` — explicit forward.
        if self.keyword("is") {
            let pred = self.term(_out)?;
            self.ws();
            if !self.keyword("of") {
                return Err(format!("expected 'of' after 'is <pred>' at byte {}", self.i));
            }
            return Ok((pred, true));
        }
        if self.keyword("has") {
            return Ok((self.term(_out)?, false));
        }
        Ok((self.atom(_out)?, false))
    }

    /// Consume `kw` when it stands as a whole token (followed by whitespace).
    fn keyword(&mut self, kw: &str) -> bool {
        self.ws();
        let end = self.i + kw.len();
        if self.s[self.i..].starts_with(kw.as_bytes())
            && self.s.get(end).is_none_or(|c| (*c as char).is_whitespace())
        {
            self.i = end;
            true
        } else {
            false
        }
    }

    // ---- terms ---------------------------------------------------------------
    /// A term, including N3 path expressions: `node (! pred | ^ pred)*`. `A!P` is the object
    /// reached by `A P ?o`; `A^P` is the subject reaching `A` via `?s P A`. Each step
    /// introduces a fresh variable and emits the connecting triple into `out` (so paths in
    /// rule premises join correctly).
    fn term(&mut self, out: &mut Vec<[Term; 3]>) -> Result<Term, String> {
        let mut base = self.atom(out)?;
        loop {
            self.ws();
            match self.peek() {
                Some(b'!') => {
                    self.i += 1;
                    let pred = self.atom(out)?;
                    let next = self.fresh_pathvar();
                    out.push([base, pred, next.clone()]);
                    base = next;
                }
                Some(b'^') => {
                    self.i += 1;
                    let pred = self.atom(out)?;
                    let next = self.fresh_pathvar();
                    out.push([next.clone(), pred, base]);
                    base = next;
                }
                _ => break,
            }
        }
        Ok(base)
    }

    fn fresh_pathvar(&mut self) -> Term {
        self.pathvar += 1;
        Term::Var(format!("__path{}", self.pathvar))
    }

    /// A single atomic term (no path operators).
    fn atom(&mut self, out: &mut Vec<[Term; 3]>) -> Result<Term, String> {
        self.ws();
        match self.peek() {
            Some(b'<') => Ok(nil_to_list(Term::Iri(self.read_iriref()?))),
            Some(b'"') | Some(b'\'') => self.read_literal(),
            Some(b'?') => {
                self.i += 1;
                Ok(Term::Var(self.read_name()))
            }
            Some(b'_') => {
                // _:label
                self.i += 1;
                self.eat(b':');
                Ok(Term::Blank(self.read_name()))
            }
            Some(b'{') => self.read_formula(),
            Some(b'(') => self.read_collection(out),
            Some(b'[') => self.read_bnode_propertylist(out),
            Some(c) if c.is_ascii_digit() || c == b'+' || c == b'-' || c == b'.' => self.read_number(),
            Some(_) => {
                // prefixed name, `true`/`false`, or `a`
                if self.starts_with("true") {
                    self.i += 4;
                    return Ok(Term::Lit("true".into(), XSD_BOOLEAN.into(), None));
                }
                if self.starts_with("false") {
                    self.i += 5;
                    return Ok(Term::Lit("false".into(), XSD_BOOLEAN.into(), None));
                }
                self.read_prefixed_name().map(nil_to_list)
            }
            None => Err("unexpected end of input".into()),
        }
    }

    fn read_iriref(&mut self) -> Result<String, String> {
        if !self.eat(b'<') {
            return Err(format!("expected IRI at byte {}", self.i));
        }
        let start = self.i;
        while self.i < self.s.len() && self.s[self.i] != b'>' {
            self.i += 1;
        }
        let iri = std::str::from_utf8(&self.s[start..self.i]).map_err(|e| e.to_string())?.to_string();
        self.i += 1; // '>'
        Ok(resolve_iri(&self.base, &iri))
    }

    fn read_pname_prefix(&mut self) -> Result<String, String> {
        // read up to ':'
        let start = self.i;
        while self.i < self.s.len() && self.s[self.i] != b':' && !(self.s[self.i] as char).is_whitespace() {
            self.i += 1;
        }
        let pfx = std::str::from_utf8(&self.s[start..self.i]).unwrap().to_string();
        self.eat(b':');
        Ok(pfx)
    }

    fn read_prefixed_name(&mut self) -> Result<Term, String> {
        let start = self.i;
        while self.i < self.s.len() {
            let c = self.s[self.i];
            if (c as char).is_whitespace()
                || matches!(c, b'.' | b';' | b',' | b']' | b'}' | b')' | b'(' | b'[' | b'{' | b'!' | b'^')
            {
                break;
            }
            self.i += 1;
        }
        let tok = std::str::from_utf8(&self.s[start..self.i]).unwrap();
        let (pfx, local) = tok.split_once(':').ok_or_else(|| format!("bad token '{tok}' at {start}"))?;
        let ns = match self.prefixes.get(pfx) {
            Some(ns) => ns.clone(),
            // cwm/EYE treat an undeclared default prefix as `@prefix : <#>.`
            // (document-local names); honor that for ':' only.
            None if pfx.is_empty() => resolve_iri(&self.base, "#"),
            None => return Err(format!("unknown prefix '{pfx}:'")),
        };
        Ok(Term::Iri(format!("{ns}{local}")))
    }

    fn read_name(&mut self) -> String {
        let start = self.i;
        while self.i < self.s.len() {
            let c = self.s[self.i];
            if (c as char).is_alphanumeric() || c == b'_' || c == b'-' {
                self.i += 1;
            } else {
                break;
            }
        }
        std::str::from_utf8(&self.s[start..self.i]).unwrap().to_string()
    }

    fn read_literal(&mut self) -> Result<Term, String> {
        let quote = self.s[self.i];
        // triple-quoted?
        let triple = self.s[self.i..].starts_with(&[quote, quote, quote]);
        if triple {
            self.i += 3;
        } else {
            self.i += 1;
        }
        let mut val = String::new();
        loop {
            if self.i >= self.s.len() {
                return Err("unterminated literal".into());
            }
            let c = self.s[self.i];
            if c == b'\\' {
                self.i += 1;
                let e = self.s[self.i];
                val.push(match e {
                    b'n' => '\n',
                    b't' => '\t',
                    b'r' => '\r',
                    b'"' => '"',
                    b'\'' => '\'',
                    b'\\' => '\\',
                    other => other as char,
                });
                self.i += 1;
                continue;
            }
            if c == quote {
                if triple {
                    if self.s[self.i..].starts_with(&[quote, quote, quote]) {
                        self.i += 3;
                        break;
                    }
                } else {
                    self.i += 1;
                    break;
                }
            }
            // utf-8 safe push
            let ch_len = utf8_len(c);
            val.push_str(std::str::from_utf8(&self.s[self.i..self.i + ch_len]).unwrap());
            self.i += ch_len;
        }
        // datatype / lang
        if self.s[self.i..].starts_with(b"^^") {
            self.i += 2;
            let dt = self.read_iri_or_prefixed()?;
            Ok(Term::Lit(val, dt, None))
        } else if self.peek() == Some(b'@') {
            self.i += 1;
            let lang = self.read_name();
            Ok(Term::Lit(val, "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString".into(), Some(lang)))
        } else {
            Ok(Term::Lit(val, "http://www.w3.org/2001/XMLSchema#string".into(), None))
        }
    }

    fn read_iri_or_prefixed(&mut self) -> Result<String, String> {
        self.ws();
        if self.peek() == Some(b'<') {
            self.read_iriref()
        } else {
            match self.read_prefixed_name()? {
                Term::Iri(i) => Ok(i),
                _ => Err("expected datatype IRI".into()),
            }
        }
    }

    fn read_number(&mut self) -> Result<Term, String> {
        let start = self.i;
        let mut is_decimal = false;
        let mut is_double = false;
        if matches!(self.peek(), Some(b'+') | Some(b'-')) {
            self.i += 1;
        }
        while self.i < self.s.len() {
            let c = self.s[self.i];
            if c.is_ascii_digit() {
                self.i += 1;
            } else if c == b'.' && self.i + 1 < self.s.len() && self.s[self.i + 1].is_ascii_digit() {
                is_decimal = true;
                self.i += 1;
            } else if c == b'e' || c == b'E' {
                is_double = true;
                self.i += 1;
                if matches!(self.peek(), Some(b'+') | Some(b'-')) {
                    self.i += 1;
                }
            } else {
                break;
            }
        }
        let txt = std::str::from_utf8(&self.s[start..self.i]).unwrap().to_string();
        let dt = if is_double { XSD_DOUBLE } else if is_decimal { XSD_DECIMAL } else { XSD_INTEGER };
        Ok(Term::Lit(txt, dt.into(), None))
    }

    fn enter(&mut self) -> Result<(), String> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(format!("nesting deeper than {MAX_DEPTH}"));
        }
        Ok(())
    }

    fn read_formula(&mut self) -> Result<Term, String> {
        self.enter()?;
        self.eat(b'{');
        let mut triples = Vec::new();
        loop {
            self.ws();
            if self.eat(b'}') {
                break;
            }
            if self.i >= self.s.len() {
                return Err("unterminated formula".into());
            }
            self.statement(&mut triples)?;
        }
        self.depth -= 1;
        Ok(Term::Formula(triples))
    }

    fn read_collection(&mut self, out: &mut Vec<[Term; 3]>) -> Result<Term, String> {
        self.enter()?;
        self.eat(b'(');
        let mut items = Vec::new();
        loop {
            self.ws();
            if self.eat(b')') {
                break;
            }
            if self.i >= self.s.len() {
                return Err("unterminated collection".into());
            }
            items.push(self.term(out)?);
        }
        self.depth -= 1;
        // A collection is a FIRST-CLASS list term (N3 semantics); `()` = rdf:nil
        // = the empty list.
        Ok(Term::List(items))
    }

    fn read_bnode_propertylist(&mut self, out: &mut Vec<[Term; 3]>) -> Result<Term, String> {
        self.enter()?;
        self.eat(b'[');
        let node = self.fresh_bnode();
        self.ws();
        if !self.eat(b']') {
            self.predicate_object_list(&node, out)?;
            self.ws();
            self.eat(b']');
        }
        self.depth -= 1;
        Ok(node)
    }

    fn fresh_bnode(&mut self) -> Term {
        self.bnode += 1;
        Term::Blank(format!("_b{}", self.bnode))
    }
}

/// Pragmatic RFC 3986 reference resolution: enough for the forms test suites
/// and real documents use (`#frag`, `name.n3#frag`, `../x`, absolute IRIs,
/// scheme-relative and root-relative paths). With no base, or an absolute
/// reference, the reference is returned as written.
pub(super) fn resolve_iri(base: &str, iri: &str) -> String {
    let is_absolute = |s: &str| -> bool {
        match s.find(':') {
            Some(i) if i > 0 => {
                let scheme = &s[..i];
                scheme.starts_with(|c: char| c.is_ascii_alphabetic())
                    && scheme.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
                    // a pname-like "a:b" inside <> is still an IRI; treat any
                    // syntactically valid scheme as absolute (RFC 3986 §4.3)
            }
            _ => false,
        }
    };
    if base.is_empty() || is_absolute(iri) {
        return iri.to_string();
    }
    // Strip any fragment from the base.
    let base = base.split('#').next().unwrap_or(base);
    if iri.is_empty() {
        return base.to_string();
    }
    if let Some(frag) = iri.strip_prefix('#') {
        return format!("{base}#{frag}");
    }
    // scheme = up to ':'; authority = '//…' if present.
    let scheme_end = base.find(':').map(|i| i + 1).unwrap_or(0);
    let (authority_end, has_authority) = if base[scheme_end..].starts_with("//") {
        let rest = &base[scheme_end + 2..];
        (scheme_end + 2 + rest.find('/').unwrap_or(rest.len()), true)
    } else {
        (scheme_end, false)
    };
    if iri.starts_with("//") && has_authority {
        return format!("{}{}", &base[..scheme_end], iri);
    }
    let merged = if iri.starts_with('/') {
        format!("{}{}", &base[..authority_end], iri)
    } else {
        // Merge with the base path minus its last segment.
        let path_end = base.rfind('/').map(|i| i + 1).unwrap_or(base.len());
        let dir = if path_end > authority_end { &base[..path_end] } else { base };
        if dir.ends_with('/') {
            format!("{dir}{iri}")
        } else {
            format!("{dir}/{iri}")
        }
    };
    // Remove dot segments in the path part (after the authority).
    let (prefix, path) = merged.split_at(authority_end.min(merged.len()));
    let mut out: Vec<&str> = Vec::new();
    for seg in path.split('/') {
        match seg {
            "." => {}
            ".." => {
                out.pop();
            }
            s => out.push(s),
        }
    }
    format!("{prefix}{}", out.join("/"))
}

fn utf8_len(b: u8) -> usize {
    if b < 0x80 {
        1
    } else if b >> 5 == 0b110 {
        2
    } else if b >> 4 == 0b1110 {
        3
    } else {
        4
    }
}
