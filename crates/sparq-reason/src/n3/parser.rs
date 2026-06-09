//! A focused Notation3 parser — the subset needed for rule reasoning (EYE-style).
//!
//! oxttl parses Turtle/N-Triples but NOT N3's rule/formula extensions, so we hand-roll a
//! recursive-descent parser over the common N3 surface:
//! - `@prefix`/`@base`, prefixed names, `<iri>`, `a` (= rdf:type)
//! - literals: `"s"`, `"s"^^<dt>`, `"s"@lang`, integers, decimals, doubles, `true`/`false`
//! - `_:blank`, `?var` (universally-quantified N3 variables)
//! - `{ … }` formulae (graph terms), `( … )` collections (RDF lists)
//! - predicate sugar `=>` (log:implies), `<=` (reverse implies), `=` (owl:sameAs)
//! - statement structure with `;` (predicate lists) and `,` (object lists)
//!
//! Not yet covered: paths (`!`/`^`), explicit `@forAll`/`@forSome`, `@keywords`, nested
//! quoting beyond formulae. These are roadmap items toward full EYE parity.

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
    /// `{ premise } => { conclusion }` rules.
    pub rules: Vec<Rule>,
}

pub fn parse(src: &str) -> Result<Parsed, String> {
    let mut p = Parser::new(src);
    let stmts = p.document()?;
    let mut facts = Vec::new();
    let mut rules = Vec::new();
    for [s, pred, o] in stmts {
        match (&pred, &s, &o) {
            // { premise } => { conclusion }
            (Term::Iri(i), Term::Formula(prem), Term::Formula(concl)) if i == LOG_IMPLIES => {
                rules.push(Rule { premise: prem.clone(), conclusion: concl.clone() });
            }
            // { conclusion } <= { premise } — for the deductive CLOSURE this is exactly
            // `premise => conclusion`, so we reverse it into a forward rule. (True
            // goal-directed backward chaining + proof output is a later addition.)
            (Term::Iri(i), Term::Formula(concl), Term::Formula(prem)) if i == LOG_IMPLIED_BY => {
                rules.push(Rule { premise: prem.clone(), conclusion: concl.clone() });
            }
            _ => facts.push([s, pred, o]),
        }
    }
    Ok(Parsed { facts, rules })
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
    base: String,
    prefixes: std::collections::HashMap<String, String>,
    bnode: usize,
    pathvar: usize,
}

impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Parser<'a> {
        Parser { s: src.as_bytes(), i: 0, base: String::new(), prefixes: Default::default(), bnode: 0, pathvar: 0 }
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
            let pred = self.verb(out)?;
            loop {
                let obj = self.term(out)?;
                out.push([subj.clone(), pred.clone(), obj]);
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

    fn verb(&mut self, _out: &mut Vec<[Term; 3]>) -> Result<Term, String> {
        self.ws();
        if self.starts_with("=>") {
            self.i += 2;
            return Ok(Term::Iri(LOG_IMPLIES.into()));
        }
        if self.starts_with("<=") {
            self.i += 2;
            // a <= b  means  b => a; we normalize by swapping at the call site is hard here,
            // so emit a reverse-implies marker the reasoner treats like implies with swapped
            // formulae. Simpler: not supported in v1 beyond parse; treat as log:implies marker.
            return Ok(Term::Iri("http://www.w3.org/2000/10/swap/log#impliedBy".into()));
        }
        if self.eat(b'=') {
            return Ok(Term::Iri(OWL_SAME_AS.into()));
        }
        // `a` keyword (rdf:type) — only when a standalone token.
        if self.starts_with("a") {
            let j = self.i + 1;
            if j >= self.s.len() || (self.s[j] as char).is_whitespace() || self.s[j] == b'<' {
                self.i += 1;
                return Ok(Term::Iri(RDF_TYPE.into()));
            }
        }
        self.atom(_out)
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
            Some(b'<') => Ok(Term::Iri(self.read_iriref()?)),
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
                self.read_prefixed_name()
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
        // resolve against base if relative (no scheme) and base set
        if self.base.is_empty() || iri.contains("://") || iri.starts_with("urn:") {
            Ok(iri)
        } else {
            Ok(format!("{}{}", self.base, iri))
        }
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
        let ns = self
            .prefixes
            .get(pfx)
            .ok_or_else(|| format!("unknown prefix '{pfx}:'"))?;
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

    fn read_formula(&mut self) -> Result<Term, String> {
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
        Ok(Term::Formula(triples))
    }

    fn read_collection(&mut self, out: &mut Vec<[Term; 3]>) -> Result<Term, String> {
        self.eat(b'(');
        let mut items = Vec::new();
        loop {
            self.ws();
            if self.eat(b')') {
                break;
            }
            items.push(self.term(out)?);
        }
        // Build an rdf:List (first/rest/nil) and return its head; empty → rdf:nil.
        if items.is_empty() {
            return Ok(Term::Iri(RDF_NIL.into()));
        }
        let mut head = Term::Iri(RDF_NIL.into());
        for it in items.into_iter().rev() {
            let node = self.fresh_bnode();
            out.push([node.clone(), Term::Iri(RDF_FIRST.into()), it]);
            out.push([node.clone(), Term::Iri(RDF_REST.into()), head]);
            head = node;
        }
        Ok(head)
    }

    fn read_bnode_propertylist(&mut self, out: &mut Vec<[Term; 3]>) -> Result<Term, String> {
        self.eat(b'[');
        let node = self.fresh_bnode();
        self.ws();
        if !self.eat(b']') {
            self.predicate_object_list(&node, out)?;
            self.ws();
            self.eat(b']');
        }
        Ok(node)
    }

    fn fresh_bnode(&mut self) -> Term {
        self.bnode += 1;
        Term::Blank(format!("_b{}", self.bnode))
    }
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
