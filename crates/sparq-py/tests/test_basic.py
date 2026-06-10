"""End-to-end tests for the `sparq` Python package (load / query / update / reason).

Run with the extension module built into the active environment:

    python3 -m venv .venv-py && . .venv-py/bin/activate
    pip install maturin pytest
    (cd crates/sparq-py && maturin develop)
    pytest crates/sparq-py/tests
"""

import json

import pytest

import sparq

DATA = """
@prefix ex: <http://ex/> .
ex:alice ex:knows ex:bob ; ex:age 30 ; ex:name "Alice" .
ex:bob   ex:knows ex:carol ; ex:age 25 ; ex:name "Bob"@en .
ex:carol ex:age 35 .
"""


def g():
    return sparq.Graph.load(DATA)


# --- load -------------------------------------------------------------------


def test_load_from_string_and_len():
    graph = g()
    assert len(graph) == 7
    assert "7 triples" in repr(graph)


def test_load_from_path(tmp_path):
    p = tmp_path / "data.ttl"
    p.write_text(DATA)
    assert len(sparq.Graph.load(p)) == 7  # pathlib.Path
    assert len(sparq.Graph.load(str(p))) == 7  # str path
    nt = tmp_path / "data.nt"
    nt.write_text("<http://ex/a> <http://ex/p> <http://ex/b> .\n")
    assert len(sparq.Graph.load(nt)) == 1  # format inferred from .nt


def test_load_named_graphs_nquads():
    nq = (
        "<http://ex/a> <http://ex/p> <http://ex/b> .\n"
        "<http://ex/c> <http://ex/p> <http://ex/d> <http://ex/g1> .\n"
    )
    graph = sparq.Graph.load(nq, format="nquads")
    assert len(graph) == 1  # default graph only
    assert graph.ask("ASK { GRAPH <http://ex/g1> { ?s ?p ?o } }")


def test_load_parse_error():
    with pytest.raises(ValueError):
        sparq.Graph.load("this is not turtle @@@")


# --- query ------------------------------------------------------------------


def test_query_vars_rows_and_terms():
    res = g().query(
        "PREFIX ex: <http://ex/> SELECT ?s ?a WHERE { ?s ex:age ?a } ORDER BY ?a"
    )
    assert res.vars == ["s", "a"]
    assert len(res) == 3
    rows = res.rows
    assert [str(r["a"]) for r in rows] == ["25", "30", "35"]
    s0 = rows[0]["s"]
    assert s0.kind == "uri" and s0.value == "http://ex/bob"
    a0 = rows[0]["a"]
    assert a0.kind == "literal"
    assert a0.datatype == "http://www.w3.org/2001/XMLSchema#integer"
    # sequence protocol: indexing (incl. negative) and iteration
    assert res[0] == rows[0]
    assert res[-1]["a"].value == "35"
    assert sum(1 for _ in res) == 3


def test_term_language_eq_and_repr():
    res = g().query('PREFIX ex: <http://ex/> SELECT ?n WHERE { ex:bob ex:name ?n }')
    (term,) = [r["n"] for r in res.rows]
    assert term.language == "en"
    assert repr(term) == 'Term("Bob"@en)'
    same = g().query('PREFIX ex: <http://ex/> SELECT ?n WHERE { ex:bob ex:name ?n }').rows[0]["n"]
    assert term == same  # value equality
    assert hash(term) == hash(same)
    assert term != res  # not equal to arbitrary objects


def test_optional_leaves_var_out_of_row():
    res = g().query(
        "PREFIX ex: <http://ex/> SELECT ?s ?k WHERE { ?s ex:age ?a OPTIONAL { ?s ex:knows ?k } }"
    )
    assert len(res) == 3
    unbound = [r for r in res.rows if "k" not in r]
    assert len(unbound) == 1  # carol knows nobody


def test_query_json():
    out = g().query_json("PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:age 30 }")
    doc = json.loads(out)
    assert doc["head"]["vars"] == ["s"]
    bindings = doc["results"]["bindings"]
    assert bindings == [{"s": {"type": "uri", "value": "http://ex/alice"}}]


def test_query_errors():
    with pytest.raises(ValueError):
        g().query("SELECT WHERE {")  # malformed
    with pytest.raises(ValueError):
        g().query("CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }")  # unsupported form


# --- ask --------------------------------------------------------------------


def test_ask():
    graph = g()
    assert graph.ask("PREFIX ex: <http://ex/> ASK { ex:alice ex:knows ex:bob }")
    assert not graph.ask("PREFIX ex: <http://ex/> ASK { ex:bob ex:knows ex:alice }")
    # SELECT is accepted too: True iff any row
    assert graph.ask("SELECT * WHERE { ?s ?p ?o }")
    with pytest.raises(ValueError):
        graph.ask("DESCRIBE <http://ex/alice>")


# --- update -----------------------------------------------------------------


def test_update_insert_and_delete_round_trip():
    graph = g()
    graph.update('INSERT DATA { <http://ex/dave> <http://ex/age> 40 }')
    assert len(graph) == 8
    assert graph.ask("PREFIX ex: <http://ex/> ASK { ex:dave ex:age 40 }")
    graph.update('DELETE DATA { <http://ex/dave> <http://ex/age> 40 }')
    assert len(graph) == 7
    assert not graph.ask("PREFIX ex: <http://ex/> ASK { ex:dave ex:age ?a }")


def test_update_delete_insert_where():
    graph = g()
    graph.update(
        "PREFIX ex: <http://ex/> "
        "DELETE { ?s ex:age ?a } INSERT { ?s ex:age 99 } WHERE { ?s ex:age ?a . FILTER(?a > 28) }"
    )
    res = graph.query("PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:age 99 }")
    assert len(res) == 2  # alice (30) and carol (35) rewritten


def test_update_error_leaves_graph_unchanged():
    graph = g()
    with pytest.raises(ValueError):
        graph.update("NOT SPARQL")
    assert len(graph) == 7


# --- reasoning --------------------------------------------------------------

RDFS_DATA = """
@prefix ex: <http://ex/> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
ex:Dog rdfs:subClassOf ex:Animal .
ex:rex rdf:type ex:Dog .
"""


def test_reason_rdfs():
    graph = sparq.Graph.load(RDFS_DATA)
    q = "PREFIX ex: <http://ex/> ASK { ex:rex a ex:Animal }"
    assert not graph.ask(q)
    added = graph.reason("rdfs")
    assert added >= 1
    assert graph.ask(q)
    assert graph.reason("rdfs") == 0  # idempotent


def test_reason_owl():
    data = """
    @prefix ex: <http://ex/> .
    @prefix owl: <http://www.w3.org/2002/07/owl#> .
    ex:knows a owl:SymmetricProperty .
    ex:alice ex:knows ex:bob .
    """
    graph = sparq.Graph.load(data)
    graph.reason("owl")
    assert graph.ask("PREFIX ex: <http://ex/> ASK { ex:bob ex:knows ex:alice }")


def test_reason_bad_profile():
    with pytest.raises(ValueError):
        g().reason("clearly-not-a-profile")
    with pytest.raises(ValueError, match="load_n3"):
        g().reason("n3")


def test_load_n3_forward_chaining():
    n3 = """
    @prefix ex: <http://ex/> .
    ex:socrates a ex:Man .
    { ?x a ex:Man } => { ?x a ex:Mortal } .
    """
    graph = sparq.Graph.load_n3(n3)
    assert graph.ask("PREFIX ex: <http://ex/> ASK { ex:socrates a ex:Mortal }")


# --- save / open ------------------------------------------------------------


def test_save_open_round_trip(tmp_path):
    d = tmp_path / "db"
    graph = g()
    graph.save(d)
    reopened = sparq.Graph.open(d)
    assert len(reopened) == len(graph)
    assert reopened.query_json(
        "PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:age ?a } ORDER BY ?a"
    ) == graph.query_json("PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:age ?a } ORDER BY ?a")


def test_open_missing_dir(tmp_path):
    with pytest.raises(IOError):
        sparq.Graph.open(tmp_path / "nope")


def test_version():
    assert isinstance(sparq.__version__, str) and sparq.__version__
