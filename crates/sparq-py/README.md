# sparq (Python bindings)

Python bindings for the [sparq](https://github.com/jeswr/sparq) RDF + SPARQL engine:
a dictionary-encoded triplestore with six permutation indexes, a SPARQL 1.1 query
engine (SELECT / ASK / CONSTRUCT / DESCRIBE), SPARQL Update over the full dataset
(named graphs included), opt-in RDFS / OWL-RL / Notation3 reasoning with OWL
inconsistency reporting, and opt-in BM25 full-text search (`sparq-text`'s
`text:` magic predicates) — compiled to a native extension module with
[pyo3](https://pyo3.rs) and packaged with [maturin](https://www.maturin.rs)
(abi3: one wheel per platform covers CPython ≥ 3.9).

## Install (development)

```sh
python3 -m venv .venv-py && . .venv-py/bin/activate
pip install maturin pytest
cd crates/sparq-py
maturin develop            # debug build into the venv
pytest tests               # run the test suite
```

Release wheels: `maturin build --profile python-release` (release optimisation with
unwinding panics — the workspace's default `release` profile is `panic = "abort"`,
which would turn any Rust panic into a hard interpreter abort).

## Usage

```python
import sparq

# Load from a string of RDF data, or from a file path (str or pathlib.Path).
g = sparq.Graph.load("""
    @prefix ex: <http://ex/> .
    ex:alice ex:knows ex:bob ; ex:age 30 .
    ex:bob   ex:age 25 .
""")                                  # format defaults to "turtle"
g = sparq.Graph.load("data.nt")       # format inferred from the extension
len(g)                                # number of triples

# SELECT -> QueryResult: .vars + .rows (list of {var: Term} dicts).
res = g.query("PREFIX ex: <http://ex/> SELECT ?s ?a WHERE { ?s ex:age ?a } ORDER BY ?a")
res.vars                              # ['s', 'a']
for row in res:
    print(row["s"].value, row["a"].value)
# Term has .kind ("uri" | "literal" | "bnode"), .value, .language, .datatype,
# value-based __eq__/__hash__, and an N-Triples-ish __repr__.

# Unbound variables (e.g. from OPTIONAL) are simply absent from the row dict.

# The fast path: SPARQL 1.1 JSON results, serialised straight from the dictionary.
g.query_json("SELECT * WHERE { ?s ?p ?o }")   # -> str

# ASK (native: evaluation early-exits at the first solution).
g.ask("PREFIX ex: <http://ex/> ASK { ex:alice ex:knows ex:bob }")   # -> True

# CONSTRUCT / DESCRIBE -> list of (subject, predicate, object) Term triples.
for s, p, o in g.construct(
    "PREFIX ex: <http://ex/> CONSTRUCT { ?s a ex:Person } WHERE { ?s ex:age ?a }"
):
    print(s.value, p.value, o.value)
g.describe("DESCRIBE <http://ex/alice>")  # concise bounded description (CBD)

# SPARQL Update, applied in place (INSERT/DELETE DATA, DELETE/INSERT ... WHERE,
# CLEAR/DROP/CREATE/ADD/COPY/MOVE). Named graphs are fully supported: GRAPH-scoped
# data and templates, USING (NAMED), and the graph-management operations.
g.update('INSERT DATA { <http://ex/carol> <http://ex/age> 35 }')
g.update('INSERT DATA { GRAPH <http://ex/g1> { <http://ex/a> <http://ex/p> <http://ex/b> } }')
g.ask('ASK { GRAPH <http://ex/g1> { ?s ?p ?o } }')                  # -> True

# Opt-in reasoning: materialize the RDFS or OWL-RL closure in place.
added = g.reason("rdfs")              # returns the number of entailed triples added
g.reason("owl")                       # OWL 2 RL subset (includes RDFS)
g.inconsistencies()                   # OWL 2 RL clash report: list of descriptions
                                      # (run reason("owl") first for entailed clashes)

# Notation3 reasoning: load rules + facts from one N3 document ...
g2 = sparq.Graph.load_n3("""
    @prefix ex: <http://ex/> .
    ex:socrates a ex:Man .
    { ?x a ex:Man } => { ?x a ex:Mortal } .
""")
g2.ask("PREFIX ex: <http://ex/> ASK { ex:socrates a ex:Mortal }")   # True

# ... or apply caller-supplied N3 rules to an already-loaded graph, in place:
g3 = sparq.Graph.load("@prefix ex: <http://ex/> . ex:plato a ex:Man .")
g3.reason_n3_with("@prefix ex: <http://ex/> . { ?x a ex:Man } => { ?x a ex:Mortal } .")
g3.ask("PREFIX ex: <http://ex/> ASK { ex:plato a ex:Mortal }")      # True

# Full-text search (BM25 over the default graph's string literals). The index
# is built lazily on first use, cached, and invalidated by update()/reason()/
# reason_n3_with(); build_text_index() builds it eagerly, drop_text_index()
# frees it (the next call lazily rebuilds).
g.text_search("ali*")                 # ranked [(Term, score), ...]; AND of tokens,
g.text_search("alice bob", any=True)  # ... any=True for OR, limit=n for top-n
g.query_text("""
    PREFIX text: <http://sparq.dev/text#>
    SELECT ?s ?score WHERE {
        ?s ?p ?lit . ?lit text:matches "ali*" . ?lit text:score ?score
    } ORDER BY DESC(?score)
""")                                  # text: magic predicates inside plain SPARQL

# Persist / reopen with memory-mapped indexes (out-of-core path).
g.save("./mydb")
g4 = sparq.Graph.open("./mydb")
```

### Notes & limits

- `Graph.load` treats a `str` as a **file path** only when a file with that name
  exists (and the string has no newline); otherwise it parses it as RDF content.
  `os.PathLike` (e.g. `pathlib.Path`) is always a file path.
- All four query forms are native: SELECT (`query` / `query_json`), ASK (`ask`),
  CONSTRUCT (`construct`), DESCRIBE (`describe`, concise-bounded-description
  semantics). Named graphs loaded from N-Quads/TriG or created by updates are
  queryable via `GRAPH` and survive `update()` / `reason()` / `reason_n3_with()`.
- `update()` / `reason()` / `reason_n3_with()` rebuild the immutable store (O(n)
  per call). Reasoning materializes over the **default graph** (named graphs are
  carried across the rebuild untouched). `len(g)` counts default-graph triples.
- `reason_n3_with(rules)` runs the rules document over the graph's triples. The
  graph's blank nodes are renamed under a reserved `sparqg` prefix first, so a
  blank-node label in the rules can NOT alias an existing data node (rule-local
  blanks stay rule-local); RDF-star triple terms have no N3 form and are
  rejected there.
- Full-text (`text_search` / `query_text`) indexes the **default graph**'s plain
  and language-tagged string literals only (named graphs keep their own
  dictionaries and are not indexed); `text:matches` is an AND of tokens,
  `text:matchesAny` an OR, `*`-suffixed tokens match as prefixes, and
  `text:score` binds the BM25 score. Hits are frozen per call: the cached index
  is invalidated by every mutating call and lazily rebuilt.
- Long-running calls (load, query, update, reason, text search) release the GIL.
