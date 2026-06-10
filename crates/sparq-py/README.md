# sparq (Python bindings)

Python bindings for the [sparq](https://github.com/jeswr/sparq) RDF + SPARQL engine:
a dictionary-encoded triplestore with six permutation indexes, a SPARQL 1.1 SELECT
engine, SPARQL Update, and opt-in RDFS / OWL-RL / Notation3 reasoning — compiled to a
native extension module with [pyo3](https://pyo3.rs) and packaged with
[maturin](https://www.maturin.rs) (abi3: one wheel per platform covers CPython ≥ 3.9).

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

# ASK (rewritten internally to a SELECT solution count, like sparq-server).
g.ask("PREFIX ex: <http://ex/> ASK { ex:alice ex:knows ex:bob }")   # -> True

# SPARQL Update, applied in place (INSERT/DELETE DATA, DELETE/INSERT ... WHERE, CLEAR).
g.update('INSERT DATA { <http://ex/carol> <http://ex/age> 35 }')

# Opt-in reasoning: materialize the RDFS or OWL-RL closure in place.
added = g.reason("rdfs")              # returns the number of entailed triples added
g.reason("owl")                       # OWL 2 RL subset (includes RDFS)

# Notation3 rules live in the N3 document itself, so N3 reasoning is a loader:
g2 = sparq.Graph.load_n3("""
    @prefix ex: <http://ex/> .
    ex:socrates a ex:Man .
    { ?x a ex:Man } => { ?x a ex:Mortal } .
""")
g2.ask("PREFIX ex: <http://ex/> ASK { ex:socrates a ex:Mortal }")   # True

# Persist / reopen with memory-mapped indexes (out-of-core path).
g.save("./mydb")
g3 = sparq.Graph.open("./mydb")
```

### Notes & limits

- `Graph.load` treats a `str` as a **file path** only when a file with that name
  exists (and the string has no newline); otherwise it parses it as RDF content.
  `os.PathLike` (e.g. `pathlib.Path`) is always a file path.
- Query forms: SELECT and ASK. CONSTRUCT / DESCRIBE are not yet exposed by the
  engine (see `TODO.md`).
- `update()` / `reason()` rebuild the immutable store (O(n) per call) and apply to
  the **default graph**; named graphs loaded from N-Quads/TriG are queryable via
  `GRAPH` but are dropped by update/reason rebuilds (engine limitation, see `TODO.md`).
- Long-running calls (load, query, update, reason) release the GIL.
