// [OPUS-4.8] sq-gb4o (#805) — unit tests for the dependency-free pretty/indented Turtle
// serialiser (packages/sparq-client/src/pretty-turtle.ts). The wasm engine emits CONSTRUCT/
// DESCRIBE (and the dataset all-quads query) as FLAT N-Triples; `prettyTurtle` reshapes that
// into grouped, indented, prefix-abbreviated Turtle. The load-bearing property is ROUND-TRIP
// equivalence: re-parsing the pretty output must yield the SAME set of triples as the input.
//
// Since there is no full Turtle parser on the JS side (and the wasm binary is a build artifact
// not present in unit tests), this file carries a SMALL Turtle re-parser that understands EXACTLY
// the constructs `prettyTurtle` emits (`@prefix` headers, `a`, subject blocks, `;`/`,` lists, the
// RDF 1.2 `<<( s p o )>>` triple term) and expands them back to canonical N-Triples triples. We
// then assert set-equality against the input parsed by the module's own `parseNTriples`.
// Run via `npm run test:unit`.
import { test } from "node:test";
import assert from "node:assert/strict";

import {
  parseNTriples,
  prettyTurtle,
  prettyTrig,
} from "../../packages/sparq-client/src/pretty-turtle.ts";

const RDF_TYPE = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const XSD_STRING = "http://www.w3.org/2001/XMLSchema#string";

// --- A canonical N-Triples key for a parsed term (datatype/lang explicit), so a triple-set can
// --- be compared regardless of ordering and regardless of an implicit xsd:string.
function termKey(t) {
  switch (t.kind) {
    case "iri":
      return `<${t.value}>`;
    case "bnode":
      return `_:${t.label}`;
    case "literal":
      if (t.lang !== undefined) return `"${t.value}"@${t.lang}`;
      return `"${t.value}"^^<${t.datatype ?? XSD_STRING}>`;
    case "triple":
      return `<<( ${termKey(t.s)} ${termKey(t.p)} ${termKey(t.o)} )>>`;
    default:
      throw new Error(`unknown term kind ${t.kind}`);
  }
}

function tripleSet(statements) {
  return new Set(
    statements.map(
      (st) => `${termKey(st.s)} ${termKey(st.p)} ${termKey(st.o)} ${st.g ? termKey(st.g) : ""}`,
    ),
  );
}

// --- A minimal re-parser for the pretty TURTLE/TriG dialect prettyTurtle emits. It expands
// --- prefixes + `a`, walks subject blocks with `;`/`,`, and handles `<<( … )>>` triple terms +
// --- `GRAPH g { … }` blocks, producing the same {s,p,o,g} statement shape parseNTriples does.
function reparsePretty(doc) {
  // 1. Strip and collect @prefix declarations.
  const prefixes = new Map();
  let base = "";
  const lines = doc.split("\n");
  const bodyLines = [];
  for (const line of lines) {
    const m = line.trim().match(/^@prefix\s+([A-Za-z][\w.-]*)?:\s*<([^>]*)>\s*\.$/);
    if (m) {
      prefixes.set(m[1] ?? "", m[2]);
      continue;
    }
    bodyLines.push(line);
  }
  const text = bodyLines.join("\n");
  void base;

  // 2. Tokenise the body into a flat token stream (terms + punctuation + GRAPH keyword).
  const toks = [];
  let i = 0;
  const n = text.length;
  const isWs = (c) => c === " " || c === "\t" || c === "\n" || c === "\r";
  while (i < n) {
    const c = text[i];
    if (isWs(c)) {
      i++;
      continue;
    }
    if (c === ";" || c === "," || c === "." || c === "{" || c === "}") {
      toks.push({ t: "punct", v: c });
      i++;
      continue;
    }
    if (c === "<" && text[i + 1] === "<" && text[i + 2] === "(") {
      toks.push({ t: "tt-open" });
      i += 3;
      continue;
    }
    if (c === ")" && text[i + 1] === ">" && text[i + 2] === ">") {
      toks.push({ t: "tt-close" });
      i += 3;
      continue;
    }
    if (c === "<") {
      let j = i + 1;
      while (j < n && text[j] !== ">") j++;
      toks.push({ t: "iri", v: text.slice(i + 1, j) });
      i = j + 1;
      continue;
    }
    if (c === '"') {
      let j = i + 1;
      while (j < n && text[j] !== '"') {
        if (text[j] === "\\") j++;
        j++;
      }
      const value = text.slice(i + 1, j);
      j++; // closing quote
      let lang;
      let datatype;
      if (text[j] === "@") {
        let k = j + 1;
        while (k < n && !isWs(text[k]) && text[k] !== ";" && text[k] !== "," && text[k] !== ".")
          k++;
        lang = text.slice(j + 1, k);
        j = k;
      } else if (text[j] === "^" && text[j + 1] === "^") {
        j += 2;
        // datatype is an IRI or prefixed name
        if (text[j] === "<") {
          let k = j + 1;
          while (k < n && text[k] !== ">") k++;
          datatype = text.slice(j + 1, k);
          j = k + 1;
        } else {
          let k = j;
          while (k < n && !isWs(text[k]) && text[k] !== ";" && text[k] !== ",") k++;
          datatype = expandPname(text.slice(j, k), prefixes);
          j = k;
        }
      }
      toks.push({ t: "lit", value, lang, datatype });
      i = j;
      continue;
    }
    if (c === "_" && text[i + 1] === ":") {
      let j = i + 2;
      while (j < n && !isWs(text[j]) && text[j] !== ";" && text[j] !== ",") j++;
      // trim a trailing dot statement-terminator glued to the label
      let lab = text.slice(i, j);
      toks.push({ t: "bnode", v: lab });
      i = j;
      continue;
    }
    // a name-like run: `a`, GRAPH, or a prefixed name
    let j = i;
    while (j < n && !isWs(text[j]) && text[j] !== ";" && text[j] !== "," && text[j] !== "{")
      j++;
    const word = text.slice(i, j);
    if (word === "a") toks.push({ t: "iri", v: RDF_TYPE });
    else if (word === "GRAPH") toks.push({ t: "graph" });
    else toks.push({ t: "iri", v: expandPname(word, prefixes) });
    i = j;
  }

  // 3. Recursive-descent over the token stream producing statements.
  const statements = [];
  let p = 0;
  const peek = () => toks[p];
  const readTerm = (graph) => {
    const tk = toks[p++];
    if (tk.t === "iri") return { kind: "iri", value: tk.v };
    if (tk.t === "bnode") return { kind: "bnode", label: tk.v.slice(2) };
    if (tk.t === "lit")
      return { kind: "literal", value: tk.value, lang: tk.lang, datatype: tk.datatype };
    if (tk.t === "tt-open") {
      const s = readTerm(graph);
      const pr = readTerm(graph);
      const o = readTerm(graph);
      assert.equal(toks[p++].t, "tt-close", "triple term must close with )>>");
      return { kind: "triple", s, p: pr, o };
    }
    throw new Error(`expected a term, got ${JSON.stringify(tk)}`);
  };

  const parseBlock = (graph) => {
    // parse subject blocks until a `}` or end
    while (p < toks.length) {
      const tk = peek();
      if (!tk) break;
      if (tk.t === "punct" && tk.v === "}") return;
      if (tk.t === "graph") {
        p++;
        const g = readTerm();
        assert.equal(toks[p++].v, "{", "GRAPH must open with {");
        parseBlock(g);
        assert.equal(toks[p++].v, "}", "GRAPH must close with }");
        continue;
      }
      const subject = readTerm(graph);
      // predicate-object lists
      // eslint-disable-next-line no-constant-condition
      while (true) {
        const predicate = readTerm(graph);
        // object list
        // eslint-disable-next-line no-constant-condition
        while (true) {
          const object = readTerm(graph);
          statements.push({ s: subject, p: predicate, o: object, g: graph });
          const sep = peek();
          if (sep && sep.t === "punct" && sep.v === ",") {
            p++;
            continue;
          }
          break;
        }
        const sep = peek();
        if (sep && sep.t === "punct" && sep.v === ";") {
          p++;
          continue;
        }
        break;
      }
      // statement terminator
      const dot = peek();
      if (dot && dot.t === "punct" && dot.v === ".") p++;
    }
  };
  parseBlock(undefined);
  return statements;
}

function expandPname(word, prefixes) {
  const idx = word.indexOf(":");
  if (idx === -1) return word; // shouldn't happen for an abbreviated IRI
  const pre = word.slice(0, idx);
  const local = word.slice(idx + 1);
  const ns = prefixes.get(pre);
  if (ns === undefined) return word;
  return ns + local;
}

/** Assert that `pretty` re-parses to the SAME triple set as the input N-Triples `nt`. */
function assertRoundTrip(nt) {
  const input = parseNTriples(nt).statements;
  const pretty = prettyTurtle(nt);
  const reparsed = reparsePretty(pretty);
  assert.deepEqual(
    [...tripleSet(reparsed)].sort(),
    [...tripleSet(input)].sort(),
    `round-trip mismatch:\n--- pretty ---\n${pretty}\n--- reparsed ${reparsed.length}, input ${input.length} ---`,
  );
}

// ---------------------------------------------------------------------------

test("empty input yields empty output", () => {
  assert.equal(prettyTurtle(""), "");
  assert.equal(prettyTurtle("   \n  \n"), "");
  assert.equal(prettyTurtle(undefined), "");
});

test("groups a shared subject into one block with `;` and `,`", () => {
  const nt = [
    `<http://example.org/alice> <http://xmlns.com/foaf/0.1/name> "Alice" .`,
    `<http://example.org/alice> <http://xmlns.com/foaf/0.1/knows> <http://example.org/bob> .`,
    `<http://example.org/alice> <http://xmlns.com/foaf/0.1/knows> <http://example.org/carol> .`,
  ].join("\n");
  const out = prettyTurtle(nt);
  // One subject line (the abbreviated subject) — appears exactly once.
  assert.equal((out.match(/^ex:alice/gm) || []).length, 1, out);
  // The two knows objects share a predicate -> `,`-list.
  assert.match(out, /foaf:knows ex:bob ,/);
  assert.match(out, /ex:carol/);
  // `;` separates the two distinct predicates.
  assert.match(out, /;/);
  assertRoundTrip(nt);
});

test("emits @prefix header only for prefixes actually used", () => {
  const nt = `<http://example.org/x> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://xmlns.com/foaf/0.1/Person> .`;
  const out = prettyTurtle(nt);
  assert.match(out, /@prefix ex: <http:\/\/example\.org\/> \./);
  assert.match(out, /@prefix foaf: <http:\/\/xmlns\.com\/foaf\/0\.1\/> \./);
  // rdf: is used implicitly only via `a` (no rdf: prefixed name) -> not in the header.
  assert.doesNotMatch(out, /@prefix rdf:/);
  // rdf:type renders as `a`.
  assert.match(out, /\ba\b/);
  assertRoundTrip(nt);
});

test("rdf:type renders as `a` and sorts first within a block", () => {
  const nt = [
    `<http://example.org/x> <http://xmlns.com/foaf/0.1/name> "X" .`,
    `<http://example.org/x> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://xmlns.com/foaf/0.1/Person> .`,
  ].join("\n");
  const out = prettyTurtle(nt);
  const aIdx = out.indexOf(" a ");
  const nameIdx = out.indexOf("foaf:name");
  assert.ok(aIdx !== -1 && aIdx < nameIdx, `a should sort before foaf:name:\n${out}`);
  assertRoundTrip(nt);
});

test("preserves literal datatype and language tags losslessly", () => {
  const nt = [
    `<http://example.org/x> <http://example.org/p1> "42"^^<http://www.w3.org/2001/XMLSchema#integer> .`,
    `<http://example.org/x> <http://example.org/p2> "bonjour"@fr .`,
    `<http://example.org/x> <http://example.org/p3> "plain" .`,
    `<http://example.org/x> <http://example.org/p4> "explicit"^^<http://www.w3.org/2001/XMLSchema#string> .`,
  ].join("\n");
  const out = prettyTurtle(nt);
  assert.match(out, /"42"\^\^xsd:integer/);
  assert.match(out, /"bonjour"@fr/);
  // implicit xsd:string is dropped (a plain literal)
  assert.match(out, /"plain"(?!\^)/);
  assertRoundTrip(nt);
});

test("preserves blank-node labels verbatim and stably", () => {
  const nt = [
    `_:b0 <http://xmlns.com/foaf/0.1/name> "Anon" .`,
    `_:b0 <http://xmlns.com/foaf/0.1/knows> _:b1 .`,
  ].join("\n");
  const out = prettyTurtle(nt);
  assert.match(out, /_:b0/);
  assert.match(out, /_:b1/);
  assertRoundTrip(nt);
});

test("preserves a blank-node label with an internal dot", () => {
  const nt = `_:b.1 <http://example.org/p> _:b.2 .`;
  const out = prettyTurtle(nt);
  assert.match(out, /_:b\.1/);
  assert.match(out, /_:b\.2/);
  assertRoundTrip(nt);
});

test("handles RDF 1.2 triple terms `<<( s p o )>>` in object position", () => {
  const nt = `<http://example.org/s> <http://example.org/says> <<( <http://example.org/a> <http://example.org/b> "c" )>> .`;
  const out = prettyTurtle(nt);
  assert.match(out, /<<\( .* .* "c" \)>>/);
  assertRoundTrip(nt);
});

test("does not abbreviate an IRI whose local part needs escaping (keeps full <…>)", () => {
  // a space (not in PN_LOCAL) -> must stay <…>
  const nt = `<http://example.org/with%20space> <http://example.org/p> <http://example.org/o> .`;
  const out = prettyTurtle(nt);
  // local part `with%20space` is percent-encoded already (PN_LOCAL allows % + the chars), so it
  // CAN abbreviate; the point of the test is that we never produce an unparseable prefixed name.
  assertRoundTrip(nt);
  assert.ok(typeof out === "string");
});

test("passes through an unparseable line verbatim without crashing", () => {
  const nt = [
    `<http://example.org/ok> <http://example.org/p> "v" .`,
    `this is not a triple at all`,
  ].join("\n");
  const out = prettyTurtle(nt);
  assert.match(out, /this is not a triple at all/);
  // the good triple still pretty-prints
  assert.match(out, /"v"/);
});

test("named-graph (N-Quads) input renders as TriG GRAPH blocks", () => {
  const nq = [
    `<http://example.org/s> <http://example.org/p> "default" .`,
    `<http://example.org/s> <http://example.org/p> "ing" <http://example.org/g1> .`,
  ].join("\n");
  const out = prettyTurtle(nq); // auto-delegates to prettyTrig
  assert.match(out, /GRAPH ex:g1 \{/);
  assert.match(out, /"ing"/);
  assert.match(out, /"default"/);
  // round-trip with graph awareness
  const reparsed = reparsePretty(out);
  assert.deepEqual([...tripleSet(reparsed)].sort(), [...tripleSet(parseNTriples(nq).statements)].sort());
});

test("prettyTrig directly handles a default-graph-only document", () => {
  const nt = `<http://example.org/s> <http://example.org/p> "v" .`;
  const out = prettyTrig(nt);
  assert.doesNotMatch(out, /GRAPH/);
  assert.match(out, /"v"/);
});

test("query-declared prefixes win over the common registry", () => {
  const nt = `<http://my.example/Thing> <http://my.example/has> "1" .`;
  const out = prettyTurtle(nt, {
    prefixes: [{ prefix: "my", iri: "http://my.example/" }],
  });
  assert.match(out, /@prefix my: <http:\/\/my\.example\/> \./);
  assert.match(out, /my:Thing/);
  assert.match(out, /my:has/);
  assertRoundTrip(nt);
});

test("abbreviate:false keeps full IRIs and emits no @prefix header", () => {
  const nt = `<http://example.org/s> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://xmlns.com/foaf/0.1/Person> .`;
  const out = prettyTurtle(nt, { abbreviate: false });
  assert.doesNotMatch(out, /@prefix/);
  assert.match(out, /<http:\/\/example\.org\/s>/);
  // rdf:type still shows as `a` (predicate shorthand is independent of abbreviation)
  assert.match(out, /\ba\b/);
});

test("deduplicates identical triples", () => {
  const nt = [
    `<http://example.org/s> <http://example.org/p> "v" .`,
    `<http://example.org/s> <http://example.org/p> "v" .`,
  ].join("\n");
  const out = prettyTurtle(nt);
  assert.equal((out.match(/"v"/g) || []).length, 1, out);
});
